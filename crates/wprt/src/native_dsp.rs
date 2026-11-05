// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of pristine engine/src/audio/dsp.cpp runtime/control/DMA algorithms.
// SDK core, generated microcode, original optional verifier and Host callbacks remain native.
#![allow(non_upper_case_globals)]
use crate::memory::{host, rd16, rd32};
use std::collections::HashMap;
const kMailToDspHigh: u32 = 0x5000;
const kMailToDspLow: u32 = 0x5002;
const kMailFromDspHigh: u32 = 0x5004;
const kMailFromDspLow: u32 = 0x5006;
const kControl: u32 = 0x500A;
const kAramInfo: u32 = 0x5012;
const kAramMode: u32 = 0x5016;
const kAramRefresh: u32 = 0x501A;
const kAramDmaMainHigh: u32 = 0x5020;
const kAramDmaMainLow: u32 = 0x5022;
const kAramDmaAramHigh: u32 = 0x5024;
const kAramDmaAramLow: u32 = 0x5026;
const kAramDmaCountHigh: u32 = 0x5028;
const kAramDmaCountLow: u32 = 0x502A;
const kAudioDmaStartHigh: u32 = 0x5030;
const kAudioDmaStartLow: u32 = 0x5032;
const kAudioDmaControl: u32 = 0x5036;
const kAudioDmaBlocksLeft: u32 = 0x503A;
const kControlMask: u16 = 0x0C07;
const kControlReset: u16 = 0x0001;
const kControlHalt: u16 = 0x0004;
const kIntAudioDma: u16 = 0x0008;
const kIntAram: u16 = 0x0020;
const kIntDsp: u16 = 0x0080;
const kIntAll: u16 = kIntAudioDma | kIntAram | kIntDsp;
const kDmaState: u16 = 0x0200;
const kAudioControl: u32 = 0xCD006C00;
const kAudioDmaRate32k: u32 = 0x40;
const kMem1Mask: u32 = 0x01FFFFFF;
const kMem2Base: u32 = 0x10000000;
const kMem2Mask: u32 = 0x03FFFFFF;
const kDspClock: f64 = 729000000.0 / 6.0;
const kMaxLag: f64 = 0.02;
const kSlice: i32 = 512;
const kMailSlice: i32 = 512;
const kFramesPerBlock: u32 = 8;
const kIramWords: u16 = 0x1000;
const kMaxPumpStep: f64 = 0.0005;
const kCatchUpRate: i32 = 4;
extern "C" {
    fn wphw_dsp_now() -> i64;
    fn wphw_dsp_seconds(ticks: i64) -> f64;
    fn wphw_dsp_duration(seconds: f64) -> i64;
    fn wphw_dsp_present() -> bool;
    fn wphw_dsp_halted() -> bool;
    fn wphw_dsp_translated_ready() -> bool;
    fn wphw_dsp_translated(left: i32, idle: *mut bool) -> i32;
    fn wphw_dsp_step();
    fn wphw_dsp_run_cycles(cycles: i32);
    fn wphw_dsp_idle_skip() -> bool;
    fn wphw_dsp_bootstrap() -> bool;
    fn wphw_dsp_start_output();
    fn wphw_dsp_push(frames: *const i16, n: u32, rate: u32);
    fn wphw_dsp_mail_peek(cpu: bool) -> u32;
    fn wphw_dsp_mail_read(cpu: bool, high: bool) -> u16;
    fn wphw_dsp_mail_write(high: bool, value: u16);
    fn wphw_dsp_control_read() -> u16;
    fn wphw_dsp_control_write(value: u16) -> u16;
    fn wphw_dsp_aram_log(main: u32, aram: u32, count: u32, to_main: bool);
    fn wphw_dsp_report(
        seconds: f64,
        buffers: u32,
        late: u32,
        stale: u32,
        overwritten: u32,
        worst: f64,
        cycles: f64,
        dsp_seconds: f64,
        behind: f64,
    );
}
#[derive(Default)]
struct AudioDma {
    source: u32,
    control: u16,
    current: u32,
    remaining: u16,
    source_written: bool,
    started: bool,
    next_block: i64,
    clock: i64,
    last_call: i64,
}
#[derive(Default)]
struct AramDma {
    main: u32,
    aram: u32,
    count: u32,
}
#[derive(Default)]
struct Statistics {
    buffers: u32,
    late_starts: u32,
    stale_blocks: u32,
    overwritten_blocks: u32,
    worst_start: f64,
    dsp_seconds: f64,
    dsp_cycles: f64,
    since: i64,
}
#[derive(Default)]
struct BlockHistory {
    writes: u32,
    played: bool,
}
struct State {
    started: bool,
    control: u16,
    aram_info: u16,
    aram_mode: u16,
    aram_refresh: u16,
    aram: AramDma,
    audio: AudioDma,
    owed: f64,
    last_update: i64,
    stats: Statistics,
    blocks: Option<HashMap<u32, BlockHistory>>,
}
static mut STATE: State = State {
    started: false,
    control: kControlHalt,
    aram_info: 0,
    aram_mode: 1,
    aram_refresh: 156,
    aram: AramDma {
        main: 0,
        aram: 0,
        count: 0,
    },
    audio: AudioDma {
        source: 0,
        control: 0,
        current: 0,
        remaining: 0,
        source_written: false,
        started: false,
        next_block: 0,
        clock: 0,
        last_call: 0,
    },
    owed: 0.,
    last_update: 0,
    stats: Statistics {
        buffers: 0,
        late_starts: 0,
        stale_blocks: 0,
        overwritten_blocks: 0,
        worst_start: 0.,
        dsp_seconds: 0.,
        dsp_cycles: 0.,
        since: 0,
    },
    blocks: None,
};
// Guest thread owns source globals. No Rust reference remains live across native callbacks.
fn state() -> *mut State {
    std::ptr::addr_of_mut!(STATE)
}
// port of dsp.cpp static initialization, retaining four native Clock reads at their original declaration positions.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_initial(clock: i64, last: i64, update: i64, since: i64) {
    let s = state();
    (*s).audio.clock = clock;
    (*s).audio.last_call = last;
    (*s).last_update = update;
    (*s).stats.since = since;
}
// port of dsp.cpp g_control. Native optional self-test saves/restores this exact scalar.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_control_ptr() -> *mut u16 {
    std::ptr::addr_of_mut!((*state()).control)
}
// port of dsp.cpp note_dsp_write.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_note_write(address: u32, size: u32) {
    let s = state();
    let blocks = (*s).blocks.get_or_insert_with(HashMap::new);
    let mut block = address & !31;
    while block < address.wrapping_add(size) {
        let h = blocks.entry(block).or_default();
        h.writes = h.writes.wrapping_add(1);
        block = block.wrapping_add(32);
    }
}
// port of dsp.cpp note_block_played; map iteration is never observed in source.
unsafe fn note_block_played(address: u32) {
    let s = state();
    let (stale, overwritten) = {
        let h = (*s)
            .blocks
            .get_or_insert_with(HashMap::new)
            .entry(address & !31)
            .or_default();
        let r = (h.played && h.writes == 0, h.writes >= 2);
        h.writes = 0;
        h.played = true;
        r
    };
    if stale {
        (*s).stats.stale_blocks = (*s).stats.stale_blocks.wrapping_add(1)
    }
    if overwritten {
        (*s).stats.overwritten_blocks = (*s).stats.overwritten_blocks.wrapping_add(1)
    }
}
// port of dsp.cpp raise/assert_interrupt_line.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_raise(interrupt: u16) {
    (*state()).control |= interrupt;
    *crate::ffi::wphw_poll_counter_ptr() = 1;
}
// port of dsp.cpp run_dsp. SDK state remains opaque; native translated adapter includes the original optional verifier branch.
unsafe fn run_dsp(cycles: i32) -> i32 {
    let start = wphw_dsp_now();
    let mut left = cycles;
    while left > 0 && !wphw_dsp_halted() {
        if wphw_dsp_translated_ready() {
            let mut idle = false;
            let before = left;
            left = wphw_dsp_translated(left, &mut idle);
            if idle {
                break;
            }
            if left == before {
                wphw_dsp_step();
                left -= 1;
            }
        } else {
            wphw_dsp_run_cycles(left);
            left = 0;
        }
    }
    let executed = cycles - left;
    let elapsed = wphw_dsp_seconds(wphw_dsp_now() - start);
    let s = state();
    (*s).stats.dsp_seconds += elapsed;
    (*s).stats.dsp_cycles += executed as f64;
    executed
}
// port of dsp.cpp push_block, including the source's right/left memory order.
unsafe fn push_block(source: u32, enabled: bool, rate: u32) {
    let mut frames = [0i16; 16];
    for i in 0..kFramesPerBlock {
        frames[(2 * i) as usize] = if enabled {
            rd16(source.wrapping_add(4 * i + 2)) as i16
        } else {
            0
        };
        frames[(2 * i + 1) as usize] = if enabled {
            rd16(source.wrapping_add(4 * i)) as i16
        } else {
            0
        };
    }
    wphw_dsp_push(frames.as_ptr(), kFramesPerBlock, rate);
}
// port of dsp.cpp pump_audio_dma. Native duration_cast preserves source chrono rounding.
unsafe fn pump_audio_dma(wall: i64) {
    let s = state();
    let step = wphw_dsp_duration(kMaxPumpStep);
    let elapsed = (wall - (*s).audio.last_call).min(step);
    (*s).audio.last_call = wall;
    (*s).audio.clock = wall.min((*s).audio.clock + elapsed * kCatchUpRate as i64);
    let now = (*s).audio.clock;
    let rate = if rd32(kAudioControl) & kAudioDmaRate32k != 0 {
        32000
    } else {
        48000
    };
    let block = wphw_dsp_duration(kFramesPerBlock as f64 / rate as f64);
    loop {
        let enabled = (*s).audio.control & 0x8000 != 0;
        let mut source = 0;
        if !enabled {
            (*s).audio.next_block = (*s).audio.next_block.max(now - block);
            if (*s).audio.next_block + block > now {
                return;
            }
            (*s).audio.next_block += block;
        } else {
            if (*s).audio.remaining == 0 {
                if !(*s).audio.source_written {
                    return;
                }
                (*s).audio.current = (*s).audio.source;
                (*s).audio.remaining = (*s).audio.control & 0x7fff;
                (*s).audio.source_written = false;
                (*s).audio.started = false;
                wprt_dsp_raise(kIntAudioDma);
            }
            if !(*s).audio.started {
                if (*s).control & kIntAudioDma != 0 {
                    return;
                }
                (*s).audio.started = true;
                (*s).stats.buffers = (*s).stats.buffers.wrapping_add(1);
                let lateness = wphw_dsp_seconds(wall - (*s).audio.next_block);
                if (*s).stats.worst_start < lateness {
                    (*s).stats.worst_start = lateness
                }
                if lateness > 0.001 {
                    (*s).stats.late_starts = (*s).stats.late_starts.wrapping_add(1);
                }
            }
            if (*s).audio.next_block + block > now {
                return;
            }
            (*s).audio.next_block += block;
            source = (*s).audio.current;
            note_block_played(source & kMem1Mask);
            (*s).audio.remaining = (*s).audio.remaining.wrapping_sub(1);
            (*s).audio.current = (*s).audio.current.wrapping_add(32);
        }
        push_block(source, enabled, rate);
    }
}
// port of dsp.cpp report, including Statistics{}'s clock read before since is overwritten.
unsafe fn report(now: i64) {
    let s = state();
    let seconds = wphw_dsp_seconds(now - (*s).stats.since);
    if seconds < 1. {
        return;
    }
    wphw_dsp_report(
        seconds,
        (*s).stats.buffers,
        (*s).stats.late_starts,
        (*s).stats.stale_blocks,
        (*s).stats.overwritten_blocks,
        (*s).stats.worst_start,
        (*s).stats.dsp_cycles,
        (*s).stats.dsp_seconds,
        wphw_dsp_seconds(now - (*s).audio.clock),
    );
    let since = wphw_dsp_now();
    (*s).stats = Statistics {
        since,
        ..Default::default()
    };
    (*s).stats.since = now;
}
// port of dsp.cpp start; native bootstrap is unchanged SDK construction/ROM initialization/error printing.
unsafe fn start() {
    let s = state();
    if (*s).started {
        return;
    }
    (*s).started = true;
    if !wphw_dsp_bootstrap() {
        return;
    }
    (*s).last_update = wphw_dsp_now();
    wphw_dsp_start_output();
}
// port of dsp.cpp do_aram_dma, retaining per-byte copy order even for overlaps.
unsafe fn do_aram_dma() {
    let s = state();
    let main = (*s).aram.main & 0x3ffffff;
    let aram = (*s).aram.aram & 0x3ffffff;
    let count = (*s).aram.count & 0x7fffffff;
    let to_main = (*s).aram.count & 0x80000000 != 0;
    wphw_dsp_aram_log(main, aram, count, to_main);
    for i in 0..count {
        let a = host(kMem2Base | ((aram.wrapping_add(i)) & kMem2Mask));
        let m = host(main.wrapping_add(i));
        if to_main {
            *m = *a
        } else {
            *a = *m
        }
    }
    (*s).aram.main = (*s).aram.main.wrapping_add(count);
    (*s).aram.aram = (*s).aram.aram.wrapping_add(count);
    (*s).aram.count &= 0x80000000;
    (*s).control &= !kDmaState;
    wprt_dsp_raise(kIntAram);
}
// port of dsp.cpp accumulate; std::min retains its first argument for unordered values.
unsafe fn accumulate(now: i64) {
    let s = state();
    let seconds = wphw_dsp_seconds(now - (*s).last_update);
    (*s).last_update = now;
    let owed = (*s).owed + seconds * kDspClock;
    let cap = kMaxLag * kDspClock;
    (*s).owed = if cap < owed { cap } else { owed };
}
// port of dsp.cpp run_slice/run_for_mail.
unsafe fn run_slice(slice: i32) {
    let executed = run_dsp(slice);
    (*state()).owed -= executed as f64;
    if executed < slice || wphw_dsp_idle_skip() {
        (*state()).owed = 0.;
    }
}
unsafe fn run_for_mail() {
    if !wphw_dsp_halted() {
        let executed = run_dsp(kMailSlice);
        (*state()).owed -= executed as f64;
    }
}
// port of dsp.cpp update.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_update() {
    if !wphw_dsp_present() {
        return;
    }
    let now = wphw_dsp_now();
    accumulate(now);
    while (*state()).owed >= kSlice as f64 && !wphw_dsp_halted() {
        run_slice(kSlice)
    }
    if wphw_dsp_halted() {
        (*state()).owed = 0.
    }
    pump_audio_dma(now);
    report(now);
}
// port of dsp.cpp read16.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_read16(address: u32) -> u16 {
    start();
    let s = state();
    match 0x5000 | (address & 0xffe) {
        kMailToDspHigh => {
            if wphw_dsp_present() && wphw_dsp_mail_peek(true) & 0x80000000 != 0 {
                run_for_mail()
            }
            if wphw_dsp_present() {
                wphw_dsp_mail_read(true, true)
            } else {
                0
            }
        }
        kMailToDspLow => {
            if wphw_dsp_present() {
                wphw_dsp_mail_read(true, false)
            } else {
                0
            }
        }
        kMailFromDspHigh => {
            if wphw_dsp_present() && wphw_dsp_mail_peek(false) & 0x80000000 == 0 {
                run_for_mail()
            }
            if wphw_dsp_present() {
                wphw_dsp_mail_read(false, true)
            } else {
                0
            }
        }
        kMailFromDspLow => {
            if wphw_dsp_present() {
                wphw_dsp_mail_read(false, false)
            } else {
                0
            }
        }
        kControl => (*s).control & !kControlMask | wphw_dsp_control_read() & kControlMask,
        kAramInfo => (*s).aram_info,
        kAramMode => (*s).aram_mode,
        kAramRefresh => (*s).aram_refresh,
        kAramDmaMainHigh => ((*s).aram.main >> 16) as u16,
        kAramDmaMainLow => (*s).aram.main as u16,
        kAramDmaAramHigh => ((*s).aram.aram >> 16) as u16,
        kAramDmaAramLow => (*s).aram.aram as u16,
        kAramDmaCountHigh => ((*s).aram.count >> 16) as u16,
        kAramDmaCountLow => (*s).aram.count as u16,
        kAudioDmaStartHigh => ((*s).audio.source >> 16) as u16,
        kAudioDmaStartLow => (*s).audio.source as u16,
        kAudioDmaControl => (*s).audio.control,
        kAudioDmaBlocksLeft => {
            if (*s).audio.remaining > 0 {
                (*s).audio.remaining - 1
            } else {
                0
            }
        }
        _ => 0,
    }
}
// port of dsp.cpp write16, retaining acknowledgements and readonly ARAM mode.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_write16(address: u32, value: u16) {
    start();
    let s = state();
    match 0x5000 | (address & 0xffe) {
        kMailToDspHigh => {
            if wphw_dsp_present() {
                wphw_dsp_mail_write(true, value)
            }
        }
        kMailToDspLow => {
            if wphw_dsp_present() {
                wphw_dsp_mail_write(false, value)
            }
        }
        kControl => {
            let engine = wphw_dsp_control_write(value);
            if value & kControlReset != 0 {
                (*s).audio.control = 0
            }
            let kept = (*s).control & (kIntAll | kDmaState);
            let acknowledged = value & kIntAll;
            (*s).control = value & !(kIntAll | kDmaState | kControlMask)
                | engine & kControlMask
                | kept & !acknowledged;
        }
        kAramInfo => (*s).aram_info = value & 0x7f,
        kAramRefresh => (*s).aram_refresh = value & 0x7ff,
        kAramDmaMainHigh => {
            (*s).aram.main = (*s).aram.main & 0xffff | ((value & 0x3ff) as u32) << 16
        }
        kAramDmaMainLow => (*s).aram.main = (*s).aram.main & 0xffff0000 | (value & 0xffe0) as u32,
        kAramDmaAramHigh => {
            (*s).aram.aram = (*s).aram.aram & 0xffff | ((value & 0x3ff) as u32) << 16
        }
        kAramDmaAramLow => (*s).aram.aram = (*s).aram.aram & 0xffff0000 | (value & 0xffe0) as u32,
        kAramDmaCountHigh => {
            (*s).aram.count = (*s).aram.count & 0xffff | ((value & 0x83ff) as u32) << 16
        }
        kAramDmaCountLow => {
            (*s).aram.count = (*s).aram.count & 0xffff0000 | (value & 0xffe0) as u32;
            do_aram_dma();
        }
        kAudioDmaStartHigh => {
            (*s).audio.source = (*s).audio.source & 0xffff | ((value & 0x1fff) as u32) << 16
        }
        kAudioDmaStartLow => {
            (*s).audio.source = (*s).audio.source & 0xffff0000 | (value & 0xffe0) as u32;
            (*s).audio.source_written = true;
        }
        kAudioDmaControl => {
            let already = (*s).audio.control & 0x8000 != 0;
            (*s).audio.control = value;
            if !already && value & 0x8000 != 0 {
                (*s).audio.current = (*s).audio.source;
                (*s).audio.remaining = value & 0x7fff;
                (*s).audio.source_written = false;
                (*s).audio.started = false;
                (*s).audio.next_block = wphw_dsp_now();
                wprt_dsp_raise(kIntAudioDma);
            }
        }
        _ => {}
    }
}
// port of dsp.cpp pending_interrupt, source priority DSP then ARAM then audio DMA.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_pending() -> u32 {
    let c = (*state()).control;
    let active = (c >> 1) & c & kIntAll;
    if active & kIntDsp != 0 {
        7
    } else if active & kIntAram != 0 {
        6
    } else if active & kIntAudioDma != 0 {
        5
    } else {
        0
    }
}
