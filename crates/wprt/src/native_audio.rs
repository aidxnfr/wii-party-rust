// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/audio/audio.cpp mixer/queue/register algorithms.
// Windows source port; native WASAPI device, rate scalar, file, chrono and CRT math/round leaves remain. H7 evidence is tracked separately.
#![allow(non_upper_case_globals)]
const kControl: u32 = 0xCD006C00;
const kSampleCounter: u32 = 0xCD006C08;
const kPlaying: u32 = 0x01;
const kSampleRate48k: u32 = 0x02;
const kCounterReset: u32 = 0x20;
const kRate32k: f64 = 32000.0;
const kRate48k: f64 = 48000.0;
const kTargetLatency: f64 = 0.06;
const kMaxLatency: f64 = 0.2;
const kMaxRateCorrection: f64 = 0.005;
const kFadeSeconds: f64 = 0.004;
const kTaps: usize = 16;
const kPhases: usize = 512;
const kCutoff: f64 = 0.95;
const kPi: f64 = 3.14159265358979323846;
const kSpeakerChannels: u32 = 4;
use crate::memory::{rd32, wr32};
use std::{
    collections::VecDeque,
    ffi::c_void,
    sync::{Mutex, OnceLock},
};
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Frame {
    left: f32,
    right: f32,
}
extern "C" {
    fn wphw_audio_sin(x: f64) -> f64;
    fn wphw_audio_cos(x: f64) -> f64;
    fn wphw_audio_now() -> i64;
    fn wphw_audio_initial_time() -> i64;
    fn wphw_audio_elapsed(now: i64, last: i64) -> f64;
    fn wphw_audio_report_due(now: i64, last: i64) -> bool;
    fn wphw_audio_counter(samples: f64) -> u32;
    fn wphw_audio_muted() -> bool;
    fn wphw_audio_mixer_dump_open(rate: u32) -> usize;
    fn wphw_audio_mixer_dump(file: usize, mixed: *const Frame, n: u32, frames: *mut u32, rate: u32);
    fn wphw_audio_push_dump(frames: *const i16, n: usize, rate: u32);
    fn wphw_audio_report(rate: f64, lowest: usize, highest: usize, missing: u32, trims: u32);
    fn wphw_audio_start_backend();
    fn wphw_audio_source_rate() -> u32;
    fn wphw_audio_set_source_rate(rate: u32);
}
#[derive(Default)]
struct Speaker {
    queue: VecDeque<f32>,
    phase: f64,
    last: f32,
}
struct Queue {
    frames: VecDeque<Frame>,
    speakers: [Speaker; 4],
}
impl Queue {
    const fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            speakers: [const {
                Speaker {
                    queue: VecDeque::new(),
                    phase: 0.,
                    last: 0.,
                }
            }; 4],
        }
    }
}
// port of audio.cpp g_queue_mutex/g_queue/g_speakers. Original g_source_rate remains native so its unlocked native logging read gains no new Rust synchronization.
static QUEUE: Mutex<Queue> = Mutex::new(Queue::new());
// port of audio.cpp Kernel::Kernel and kernel. Native CRT sin/cos kept; each double intermediate and f32 storage point follows source.
fn kernel() -> &'static [[f32; kTaps]; kPhases + 1] {
    static KERNEL: OnceLock<Box<[[f32; kTaps]; kPhases + 1]>> = OnceLock::new();
    KERNEL.get_or_init(|| {
        let mut weights = Box::new([[0f32; kTaps]; kPhases + 1]);
        for phase in 0..=kPhases {
            let fraction = phase as f64 / kPhases as f64;
            let mut sum = 0.;
            for tap in 0..kTaps {
                let x = tap as f64 - (kTaps / 2 - 1) as f64 - fraction;
                let sinc = if x == 0. {
                    kCutoff
                } else {
                    (unsafe { wphw_audio_sin(kPi * kCutoff * x) }) / (kPi * x)
                };
                let w = (x + kTaps as f64 / 2.) / kTaps as f64;
                let window = unsafe {
                    0.42 - 0.5 * wphw_audio_cos(2. * kPi * w) + 0.08 * wphw_audio_cos(4. * kPi * w)
                };
                weights[phase][tap] = (sinc * window) as f32;
                sum += sinc * window;
            }
            for tap in 0..kTaps {
                weights[phase][tap] = (weights[phase][tap] as f64 / sum) as f32;
            }
        }
        weights
    })
}
// port of source std::min/max/clamp, preserving first operand for equal/unordered comparisons.
fn minf(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn maxf(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
fn clampd(v: f64, lo: f64, hi: f64) -> f64 {
    if v < lo {
        lo
    } else if hi < v {
        hi
    } else {
        v
    }
}
struct Mixer {
    device_rate: f64,
    filter: &'static [[f32; kTaps]; kPhases + 1],
    position: f64,
    missing: u32,
    trims: u32,
    lowest: usize,
    highest: usize,
    report: i64,
    dump: usize,
    dump_frames: u32,
    primed: bool,
    gain: f32,
    history: [Frame; kTaps],
}
// port of audio.cpp Mixer::Mixer. Source initializer order: kernel before report clock before optional dump file open.
#[no_mangle]
pub unsafe extern "C" fn wprt_audio_mixer_new(device_rate: f64) -> *mut c_void {
    let filter = kernel();
    let report = wphw_audio_now();
    let dump = wphw_audio_mixer_dump_open(device_rate as u32);
    Box::into_raw(Box::new(Mixer {
        device_rate,
        filter,
        position: 0.,
        missing: 0,
        trims: 0,
        lowest: usize::MAX,
        highest: 0,
        report,
        dump,
        dump_frames: 0,
        primed: false,
        gain: 0.,
        history: [Frame::default(); kTaps],
    }))
    .cast()
}
// port of audio.cpp Mixer::fill. Lock and queue/speaker consumption order, source underrun history quirk and dump-before-mute retained.
#[no_mangle]
pub unsafe extern "C" fn wprt_audio_mixer_fill(
    ptr: *mut c_void,
    mixed: *mut Frame,
    available: u32,
) {
    let s = &mut *ptr.cast::<Mixer>();
    let mixed = std::slice::from_raw_parts_mut(mixed, available as usize);
    {
        let mut q = QUEUE.lock().unwrap();
        let limit = (kMaxLatency * wphw_audio_source_rate() as f64) as usize;
        let target = (kTargetLatency * wphw_audio_source_rate() as f64) as usize;
        s.lowest = s.lowest.min(q.frames.len());
        s.highest = s.highest.max(q.frames.len());
        if q.frames.len() > limit {
            let n = q.frames.len() - target;
            q.frames.drain(..n);
            s.gain = 0.;
            s.trims = s.trims.wrapping_add(1);
        }
        if !s.primed && q.frames.len() >= target {
            s.primed = true;
        }
        let error = (q.frames.len() as f64 - target as f64) / target as f64;
        let correction = clampd(
            error * kMaxRateCorrection,
            -kMaxRateCorrection,
            kMaxRateCorrection,
        );
        let step = wphw_audio_source_rate() as f64 / s.device_rate * (1. + correction);
        let fade = (1. / (kFadeSeconds * s.device_rate)) as f32;
        for i in 0..available as usize {
            if s.primed {
                s.position += step;
                while s.position >= 1. {
                    s.position -= 1.;
                    s.history.copy_within(1..kTaps, 0);
                    if let Some(frame) = q.frames.pop_front() {
                        s.history[kTaps - 1] = frame;
                        for speaker in &mut q.speakers {
                            if let Some(x) = speaker.queue.pop_front() {
                                s.history[kTaps - 1].left += x;
                                s.history[kTaps - 1].right += x;
                            }
                        }
                    } else {
                        s.missing = s.missing.wrapping_add(1);
                        s.primed = false;
                    }
                }
            }
            s.gain = if s.primed {
                minf(1., s.gain + fade)
            } else {
                maxf(0., s.gain - fade)
            };
            let weights = &s.filter[(s.position * kPhases as f64) as usize];
            let mut sample = Frame::default();
            for tap in 0..kTaps {
                sample.left += s.history[tap].left * weights[tap];
                sample.right += s.history[tap].right * weights[tap];
            }
            mixed[i] = Frame {
                left: sample.left * s.gain,
                right: sample.right * s.gain,
            };
        }
    }
    if s.dump != 0 {
        wphw_audio_mixer_dump(
            s.dump,
            mixed.as_ptr(),
            available,
            &mut s.dump_frames,
            s.device_rate as u32,
        );
    }
    if wphw_audio_muted() {
        mixed.fill(Frame::default());
    }
    let now = wphw_audio_now();
    if wphw_audio_report_due(now, s.report) {
        wphw_audio_report(s.device_rate, s.lowest, s.highest, s.missing, s.trims);
        s.missing = 0;
        s.trims = 0;
        s.lowest = usize::MAX;
        s.highest = 0;
        s.report = now;
    }
}
// port of audio.cpp g_last/g_samples/update; native initialization clock and duration_cast remain exact.
struct Counter {
    last: i64,
    samples: f64,
}
struct CounterCell(std::cell::UnsafeCell<Counter>);
unsafe impl Sync for CounterCell {}
static COUNTER: CounterCell = CounterCell(std::cell::UnsafeCell::new(Counter {
    last: 0,
    samples: 0.,
}));
static COUNTER_INIT: std::sync::Once = std::sync::Once::new();
#[no_mangle]
pub unsafe extern "C" fn wprt_audio_update() {
    let s = &mut *COUNTER.0.get();
    COUNTER_INIT.call_once(|| {
        s.last = wphw_audio_initial_time();
    });
    let now = wphw_audio_now();
    let elapsed = wphw_audio_elapsed(now, s.last);
    s.last = now;
    let mut control = rd32(kControl);
    if control & kCounterReset != 0 {
        s.samples = 0.;
        wr32(kControl, control & !kCounterReset);
        control &= !kCounterReset;
    }
    if control & kPlaying != 0 {
        s.samples += elapsed
            * if control & kSampleRate48k != 0 {
                kRate48k
            } else {
                kRate32k
            };
    }
    wr32(kSampleCounter, wphw_audio_counter(s.samples) & 0x7fff_ffff);
}
// port of audio.cpp start_output. WASAPI/SDL platform device transport remains native; it calls the Rust Mixer above.
#[no_mangle]
pub unsafe extern "C" fn wprt_audio_start_output() {
    // A host binding is installed before wpg_run, never after the native thread starts.
    if HOST_OUTPUT.get().is_some() {
        HOST_OUTPUT_READY.store(true, std::sync::atomic::Ordering::Release);
    } else {
        wphw_audio_start_backend();
    }
}
// port of audio.cpp push_speaker. Preserve phase interpolation, queue limit and last update even when queue is full.
#[no_mangle]
pub unsafe extern "C" fn wprt_audio_push_speaker(
    channel: u32,
    samples: *const i16,
    count: usize,
    rate: u32,
    volume: f32,
) {
    if channel >= kSpeakerChannels || rate == 0 {
        return;
    }
    let mut q = QUEUE.lock().unwrap();
    let step = rate as f64 / wphw_audio_source_rate() as f64;
    let limit = (kMaxLatency * wphw_audio_source_rate() as f64) as usize;
    let s = &mut q.speakers[channel as usize];
    for i in 0..count {
        let next = *samples.add(i) as f32 / 32768. * volume;
        while s.phase < 1. {
            if s.queue.len() < limit {
                s.queue.push_back(s.last + (next - s.last) * s.phase as f32);
            }
            s.phase += step;
        }
        s.phase -= 1.;
        s.last = next;
    }
}
// port of audio.cpp push. Native optional WAV file transport occurs under the source queue lock before enqueueing converted frames.
#[no_mangle]
pub unsafe extern "C" fn wprt_audio_push(frames: *const i16, count: usize, rate: u32) {
    let mut q = QUEUE.lock().unwrap();
    wphw_audio_set_source_rate(rate);
    wphw_audio_push_dump(frames, count, rate);
    for i in 0..count {
        q.frames.push_back(Frame {
            left: *frames.add(2 * i) as f32 / 32768.,
            right: *frames.add(2 * i + 1) as f32 / 32768.,
        });
    }
}

// H8 host transport only. Existing Mixer consumes the same game and Wii Remote queues.
// A per-DLL binding is immutable and must be installed before guest execution begins.
struct HostOutput { rate: f64, mixer: Option<Box<Mixer>> }
static HOST_OUTPUT: OnceLock<Mutex<HostOutput>> = OnceLock::new();
static HOST_OUTPUT_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn bind_host_output(rate: u32) -> bool {
    if rate == 0 { return false; }
    HOST_OUTPUT.set(Mutex::new(HostOutput { rate: rate as f64, mixer: None })).is_ok()
}
// Caller owns the interleaved stereo-f32 output buffer. No Rust allocation crosses the DLL ABI.
pub unsafe fn read_host_output(output: *mut f32, frames: u32) -> bool {
    if output.is_null() || frames == 0 { return false; }
    let samples = std::slice::from_raw_parts_mut(output, frames as usize * 2);
    samples.fill(0.0);
    let Some(host) = HOST_OUTPUT.get() else { return false; };
    if !HOST_OUTPUT_READY.load(std::sync::atomic::Ordering::Acquire) { return false; }
    let mut host = host.lock().unwrap();
    if host.mixer.is_none() {
        host.mixer = Some(Box::from_raw(wprt_audio_mixer_new(host.rate).cast::<Mixer>()));
    }
    // Frame is repr(C) { f32 left, f32 right }; both views have identical alignment and size.
    let mixer = &mut **host.mixer.as_mut().unwrap() as *mut Mixer;
    wprt_audio_mixer_fill(mixer.cast(), output.cast::<Frame>(), frames);
    true
}
