// Copyright 2010 Dolphin Emulator Project
// Copyright 2019 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted pristine input/wiimote.cpp port. PRIVATE DRAFT, UNBUILT/UNINTEGRATED.
// Scalar constants and tables extracted programmatically from the source.
const kMaxWiimotes: u32 = 4;
const kEepromSize: u32 = 0x1700;
const kCameraAddress: u8 = 0x58;
const kSpeakerAddress: u8 = 0x51;
const kCameraDataOffset: u8 = 0x37;
const kCameraTrackingRegister: u8 = 0x30;
const kCameraModeRegister: u8 = 0x33;
const kCameraTrackingEnable: u8 = 0x08;
const kCameraModeBasic: u8 = 1;
const kCameraModeExtended: u8 = 3;
const kCameraModeFull: u8 = 5;
const kCameraWidth: u16 = 1024;
const kCameraHeight: u16 = 768;
const kReportRumble: u8 = 0x10;
const kReportLeds: u8 = 0x11;
const kReportMode: u8 = 0x12;
const kReportIrEnable: u8 = 0x13;
const kReportSpeakerEnable: u8 = 0x14;
const kReportRequestStatus: u8 = 0x15;
const kReportWriteData: u8 = 0x16;
const kReportReadData: u8 = 0x17;
const kReportSpeakerData: u8 = 0x18;
const kReportSpeakerMute: u8 = 0x19;
const kReportIrEnable2: u8 = 0x1a;
const kInputStatus: u8 = 0x20;
const kInputReadReply: u8 = 0x21;
const kInputAck: u8 = 0x22;
const kInputCore: u8 = 0x30;
const kErrorSuccess: u8 = 0;
const kErrorBusy: u8 = 4;
const kErrorInvalidSpace: u8 = 6;
const kErrorNack: u8 = 7;
const kErrorInvalidAddress: u8 = 8;
const kIrLowX: u16 = 0x7F;
const kIrLowY: u16 = 0x5D;
const kIrHighX: u16 = 0x380;
const kIrHighY: u16 = 0x2A2;
const kAccelZeroG: u8 = 0x80;
const kAccelOneG: u8 = 0x9A;
const kReportSeconds: f64 = 0.005;
const kPi: f64 = 3.14159265358979323846;
const kGravity: f64 = 9.80665;
const kMaxTilt: f64 = kPi / 3.0;
const kTiltSpeed: f64 = 2.0 * kPi;
const kShakeFrequency: f64 = 6.0;
const kShakeTravel: f64 = 0.10;
const kSwingSeconds: f64 = 0.2;
const kSwingAcceleration: f64 = 2.5;
const kAccelMax: i32 = 1023;
const kSize: u8 = 3;
const kEeprom16D0: [u8; 24] = [
    0x00, 0x00, 0x00, 0xFF, 0x11, 0xEE, 0x00, 0x00, 0x33, 0xCC, 0x44, 0xBB, 0x00, 0x00, 0x66, 0x99,
    0x77, 0x88, 0x00, 0x00, 0x2B, 0x01, 0xE8, 0x13,
];
const kYamahaScale: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 230, 230, 230, 230, 307, 409, 512, 614,
];
const kYamahaDifference: [i32; 16] = [1, 3, 5, 7, 9, 11, 13, 15, -1, -3, -5, -7, -9, -11, -13, -15];
const kMotionShake: u32 = 0x10000;
const kMotionSwingUp: u32 = 0x20000;
const kMotionSwingDown: u32 = 0x40000;
const kMotionTiltLeft: u32 = 0x80000;
const kMotionTiltRight: u32 = 0x100000;
const kMotionTiltUp: u32 = 0x200000;
const kMotionTiltDown: u32 = 0x400000;
const kMotionSideways: u32 = 0x800000;
use std::{cell::UnsafeCell, ffi::c_void};
type Sender = unsafe extern "C" fn(*mut c_void, *const u8, usize);
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sample {
    buttons: u32,
    pointer_valid: u32,
    pointer_x: f32,
    pointer_y: f32,
    motion_valid: u32,
    accel: [f32; 3],
}
impl Default for Sample {
    fn default() -> Self {
        Self {
            buttons: 0,
            pointer_valid: 0,
            pointer_x: 0.,
            pointer_y: 0.,
            motion_valid: 0,
            accel: [0., 0., 1.],
        }
    }
}
extern "C" {
    fn wphw_wiimote_now() -> i64;
    fn wphw_wiimote_after_interval(now: i64) -> i64;
    fn wphw_wiimote_sample(index: u32, out: *mut Sample);
    fn wphw_wiimote_outputs(index: u32, rumble: bool, leds: u8);
    fn wphw_wiimote_sin(x: f64) -> f64;
    fn wphw_wiimote_cos(x: f64) -> f64;
    fn wphw_wiimote_lround(x: f64) -> i32;
    fn wphw_wiimote_lroundf(x: f32) -> i32;
    fn wphw_wiimote_log_enabled() -> bool;
    fn wphw_wiimote_log_output(index: u32, id: u8, data: *const u8, size: u32);
    fn wphw_wiimote_log_unknown(index: u32, id: u8);
    fn wphw_wiimote_speaker_enabled() -> bool;
    fn wphw_wiimote_push_speaker(
        index: u32,
        samples: *const i16,
        count: u32,
        rate: u32,
        volume: f32,
    );
}
#[derive(Clone, Copy)]
struct ReadRequest {
    space: u8,
    slave: u8,
    address: u16,
    size: u16,
}
#[derive(Clone, Copy)]
struct Decoder {
    predictor: i32,
    step: i32,
}
struct State {
    mode: u8,
    continuous: bool,
    leds: u8,
    rumble: bool,
    ir: bool,
    speaker: bool,
    speaker_mute: bool,
    buttons: u16,
    reported_buttons: u16,
    first_report: bool,
    eeprom: [u8; kEepromSize as usize],
    camera: [u8; 0x100],
    speaker_registers: [u8; 0x100],
    decoder: Decoder,
    read: ReadRequest,
    next_report: i64,
    roll: f64,
    pitch: f64,
    shake_phase: f64,
    swing_time: f64,
    swing_direction: f64,
    motion_held: u32,
    accel: [u16; 3],
    reported_accel: [u16; 3],
}
impl State {
    const fn new() -> Self {
        Self {
            mode: kInputCore,
            continuous: false,
            leds: 0,
            rumble: false,
            ir: false,
            speaker: false,
            speaker_mute: false,
            buttons: 0,
            reported_buttons: 0,
            first_report: true,
            eeprom: [0; kEepromSize as usize],
            camera: [0; 0x100],
            speaker_registers: [0; 0x100],
            decoder: Decoder {
                predictor: 0,
                step: 127,
            },
            read: ReadRequest {
                space: 0,
                slave: 0,
                address: 0,
                size: 0,
            },
            next_report: 0,
            roll: 0.,
            pitch: 0.,
            shake_phase: 0.,
            swing_time: kSwingSeconds,
            swing_direction: 0.,
            motion_held: 0,
            accel: [0; 3],
            reported_accel: [0; 3],
        }
    }
}
// port of wiimote.cpp g_state; original single guest-thread ownership. Sender callbacks enqueue Bluetooth frames.
struct States(UnsafeCell<[State; kMaxWiimotes as usize]>);
unsafe impl Sync for States {}
static STATES: States = States(UnsafeCell::new(
    [const { State::new() }; kMaxWiimotes as usize],
));
// port of wiimote.cpp add_checksum.
fn add_checksum(data: &mut [u8]) {
    let n = data.len();
    data[n - 1] = data[..n - 1].iter().fold(0x55u32, |a, &b| a + b as u32) as u8;
}
// port of wiimote.cpp fill_eeprom; retain exact duplicated calibration blocks and source table.
fn fill_eeprom(s: &mut State) {
    s.eeprom.fill(0);
    let mut ir = [
        (kIrLowX & 0xff) as u8,
        (kIrLowY & 0xff) as u8,
        (((kIrLowY & 0x300) >> 2)
            | ((kIrLowX & 0x300) >> 4)
            | ((kIrLowY & 0x300) >> 6)
            | ((kIrHighX & 0x300) >> 8)) as u8,
        (kIrHighX & 0xff) as u8,
        (kIrLowY & 0xff) as u8,
        (kIrHighX & 0xff) as u8,
        (kIrHighY & 0xff) as u8,
        (((kIrHighY & 0x300) >> 2)
            | ((kIrHighX & 0x300) >> 4)
            | ((kIrHighY & 0x300) >> 6)
            | ((kIrLowX & 0x300) >> 8)) as u8,
        (kIrLowX & 0xff) as u8,
        (kIrHighY & 0xff) as u8,
        0,
    ];
    add_checksum(&mut ir);
    let mut accel = [
        kAccelZeroG,
        kAccelZeroG,
        kAccelZeroG,
        0,
        kAccelOneG,
        kAccelOneG,
        kAccelOneG,
        0,
        0,
        0,
    ];
    add_checksum(&mut accel);
    s.eeprom[..11].copy_from_slice(&ir);
    s.eeprom[11..22].copy_from_slice(&ir);
    s.eeprom[22..32].copy_from_slice(&accel);
    s.eeprom[32..42].copy_from_slice(&accel);
    s.eeprom[0x16d0..0x16d0 + kEeprom16D0.len()].copy_from_slice(&kEeprom16D0);
}
// port of wiimote.cpp put_buttons (little endian in HID reports).
fn put_buttons(report: &mut Vec<u8>, buttons: u16) {
    report.push(buttons as u8);
    report.push((buttons >> 8) as u8);
}
// port of wiimote.cpp send_input. Callback is the original native Sender transport, not a game decision.
unsafe fn send_input(send: Sender, context: *mut c_void, payload: Vec<u8>) {
    let mut frame = vec![0xa1];
    frame.extend_from_slice(&payload);
    send(context, frame.as_ptr(), frame.len());
}
// port of wiimote.cpp send_ack.
unsafe fn send_ack(s: &State, send: Sender, context: *mut c_void, report: u8, error: u8) {
    let mut p = vec![kInputAck];
    put_buttons(&mut p, s.buttons);
    p.push(report);
    p.push(error);
    send_input(send, context, p);
}
// port of wiimote.cpp send_status. Keep float arithmetic and native lround(float) overload.
unsafe fn send_status(s: &State, send: Sender, context: *mut c_void) {
    let mut p = vec![kInputStatus];
    put_buttons(&mut p, s.buttons);
    p.push((if s.speaker { 4 } else { 0 }) | (if s.ir { 8 } else { 0 }) | (s.leds << 4));
    p.push(0);
    p.push(0);
    p.push(wphw_wiimote_lroundf((1.0f32 + 0.013f32) / 2.46f32 * 255.0f32) as u8);
    send_input(send, context, p);
}
// port of wiimote.cpp i2c_device; IR-off makes the camera NACK, speaker is still addressable.
fn i2c_device(s: &mut State, slave: u8) -> Option<&mut [u8; 0x100]> {
    if slave == kCameraAddress {
        if s.ir {
            Some(&mut s.camera)
        } else {
            None
        }
    } else if slave == kSpeakerAddress {
        Some(&mut s.speaker_registers)
    } else {
        None
    }
}
// port of wiimote.cpp process_read. Full remaining request bounds, zero-filled16-byte error replies, state-before-callback order.
unsafe fn process_read(s: &mut State, send: Sender, context: *mut c_void) -> bool {
    let count = s.read.size.min(16);
    if count == 0 {
        return false;
    }
    let mut p = vec![kInputReadReply];
    put_buttons(&mut p, s.buttons);
    let mut data = [0u8; 16];
    let mut error = kErrorSuccess;
    let request = s.read;
    if request.space == 0 {
        if request.address as u32 + request.size as u32 > kEepromSize {
            error = kErrorInvalidAddress;
        } else {
            data[..count as usize].copy_from_slice(
                &s.eeprom[request.address as usize..request.address as usize + count as usize],
            );
        }
    } else if request.space == 1 || request.space == 2 {
        let offset = request.address as u8 as usize;
        match i2c_device(s, request.slave) {
            None => error = kErrorNack,
            Some(_) if offset + count as usize > 0x100 => error = kErrorNack,
            Some(device) => {
                data[..count as usize].copy_from_slice(&device[offset..offset + count as usize])
            }
        }
    } else {
        error = kErrorInvalidSpace;
    }
    let size_minus_one = if error == kErrorSuccess {
        (count - 1) as u8
    } else {
        0x0f
    };
    p.push((size_minus_one << 4) | error);
    p.push((request.address >> 8) as u8);
    p.push(request.address as u8);
    p.extend_from_slice(&data);
    if error != kErrorSuccess {
        s.read.size = 0;
    } else {
        s.read.address = s.read.address.wrapping_add(count);
        s.read.size = s.read.size.wrapping_sub(count);
    }
    send_input(send, context, p);
    true
}
// port of wiimote.cpp valid_mode.
fn valid_mode(mode: u8) -> bool {
    (mode >= 0x30 && mode <= 0x37) || mode == 0x3d
}
#[derive(Clone, Copy, Default)]
struct Point {
    visible: bool,
    x: u16,
    y: u16,
}
// port of wiimote.cpp camera_points. Windows long/native lround overloads and source single precision retained.
unsafe fn camera_points(sample: &Sample) -> [Point; 2] {
    let mut points = [Point::default(); 2];
    if sample.pointer_valid == 0 {
        return points;
    }
    let mid_x = kCameraWidth as f32 / 2.0 - sample.pointer_x * 225.5;
    let mid_y = kCameraHeight as f32 / 2.0 + sample.pointer_y * 225.5 + 102.0;
    let xs = [mid_x - 90.0, mid_x + 90.0];
    for i in 0..2 {
        let x = wphw_wiimote_lroundf(xs[i]);
        let y = wphw_wiimote_lroundf(mid_y);
        if x >= 0 && y >= 0 && x < kCameraWidth as i32 && y < kCameraHeight as i32 {
            points[i] = Point {
                visible: true,
                x: x as u16,
                y: y as u16,
            };
        }
    }
    points
}
// port of wiimote.cpp camera_objects, all three source IR layouts, no synthetic sensor layout.
unsafe fn camera_objects(mode: u8, sample: &Sample, data: &mut [u8]) {
    data[..36].fill(0xff);
    let points = camera_points(sample);
    if mode == kCameraModeBasic {
        if points[0].visible || points[1].visible {
            let x1 = if points[0].visible {
                points[0].x
            } else {
                0x3ff
            };
            let y1 = if points[0].visible {
                points[0].y
            } else {
                0x3ff
            };
            let x2 = if points[1].visible {
                points[1].x
            } else {
                0x3ff
            };
            let y2 = if points[1].visible {
                points[1].y
            } else {
                0x3ff
            };
            data[0] = x1 as u8;
            data[1] = y1 as u8;
            data[2] = (((y1 >> 8) << 6) | ((x1 >> 8) << 4) | ((y2 >> 8) << 2) | (x2 >> 8)) as u8;
            data[3] = x2 as u8;
            data[4] = y2 as u8;
        }
    } else if mode == kCameraModeExtended || mode == kCameraModeFull {
        let stride = if mode == kCameraModeExtended { 3 } else { 9 };
        for i in 0..2 {
            let p = points[i];
            if !p.visible {
                continue;
            }
            let object = &mut data[i * stride..];
            object[0] = p.x as u8;
            object[1] = p.y as u8;
            object[2] = (((p.y >> 8) << 6) | ((p.x >> 8) << 4) | kSize as u16) as u8;
            if mode == kCameraModeFull {
                object[3] = (p.x as i32 - kSize as i32).max(0) as u8;
                object[4] = (p.y as i32 - kSize as i32).max(0) as u8;
                object[5] = 0;
                object[6] = (p.x as i32 + kSize as i32).min(kCameraWidth as i32) as u8;
                object[7] = (p.y as i32 + kSize as i32).min(kCameraHeight as i32) as u8;
                object[8] = 0x20;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wprt_wiimote_camera_objects(
    mode: u8,
    sample: *const Sample,
    data: *mut u8,
) {
    camera_objects(mode, &*sample, std::slice::from_raw_parts_mut(data, 36));
}
// port of wiimote.cpp update_camera.
unsafe fn update_camera(s: &mut State, sample: &Sample) {
    let mode = s.camera[kCameraModeRegister as usize];
    let tracking = s.camera[kCameraTrackingRegister as usize];
    let data = &mut s.camera[kCameraDataOffset as usize..];
    if tracking != kCameraTrackingEnable {
        data[..36].fill(0xff);
        return;
    }
    camera_objects(mode, sample, data);
}
// port of wiimote.cpp accel_mode.
fn accel_mode(mode: u8) -> bool {
    mode == 0x31 || mode == 0x33 || mode == 0x35 || mode == 0x37
}
// port of source std::min/max<double>: do not replace unordered handling with Rust f64::min/max.
fn source_min(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}
fn source_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}
// port of wiimote.cpp approach.
fn approach(current: f64, target: f64, step: f64) -> f64 {
    if current < target {
        source_min(current + step, target)
    } else {
        source_max(current - step, target)
    }
}
// port of wiimote.cpp update_motion. Same order, fixed report step, edge-triggered swing, native trig/round leaves.
unsafe fn update_motion(s: &mut State, sample: &Sample) {
    let held = sample.buttons;
    let pressed = held & !s.motion_held;
    s.motion_held = held;
    let (mut side, mut far) = (0., 0.);
    if held & kMotionTiltLeft != 0 {
        side -= kMaxTilt;
    }
    if held & kMotionTiltRight != 0 {
        side += kMaxTilt;
    }
    if held & kMotionTiltUp != 0 {
        far += kMaxTilt;
    }
    if held & kMotionTiltDown != 0 {
        far -= kMaxTilt;
    }
    let sideways = held & kMotionSideways != 0;
    let roll = if sideways { -far } else { side };
    let pitch = if sideways { side } else { far };
    s.roll = approach(s.roll, roll, kTiltSpeed * kReportSeconds);
    s.pitch = approach(s.pitch, pitch, kTiltSpeed * kReportSeconds);
    let mut accel = [
        wphw_wiimote_sin(s.roll) * wphw_wiimote_cos(s.pitch),
        wphw_wiimote_sin(s.pitch),
        wphw_wiimote_cos(s.roll) * wphw_wiimote_cos(s.pitch),
    ];
    if sample.motion_valid != 0 {
        for i in 0..3 {
            accel[i] = sample.accel[i] as f64;
        }
    }
    if pressed & (kMotionSwingUp | kMotionSwingDown) != 0 {
        s.swing_time = 0.;
        s.swing_direction = if pressed & kMotionSwingUp != 0 {
            1.
        } else {
            -1.
        };
    }
    if s.swing_time < kSwingSeconds {
        let sign = if s.swing_time < kSwingSeconds / 2.0 {
            1.
        } else {
            -1.
        };
        accel[2] += sign * s.swing_direction * kSwingAcceleration;
        s.swing_time += kReportSeconds;
    }
    if held & kMotionShake != 0 {
        let omega = 2.0 * kPi * kShakeFrequency;
        let amplitude = omega * omega * (kShakeTravel / 2.0) / kGravity;
        let value = amplitude * wphw_wiimote_sin(s.shake_phase);
        for i in 0..3 {
            accel[i] += value;
        }
        s.shake_phase += omega * kReportSeconds;
    } else {
        s.shake_phase = 0.;
    }
    let zero = (kAccelZeroG as i32) << 2;
    let one = ((kAccelOneG as i32) << 2) - zero;
    for i in 0..3 {
        s.accel[i] =
            wphw_wiimote_lround(zero as f64 + accel[i] * one as f64).clamp(0, kAccelMax) as u16;
    }
}
// port of wiimote.cpp send_data_report. Preserve mode-specific IR and extension byte counts.
unsafe fn send_data_report(s: &mut State, send: Sender, context: *mut c_void, sample: &Sample) {
    let mode = s.mode;
    let accel = accel_mode(mode);
    let ir_size = if mode == 0x33 {
        12
    } else if mode == 0x36 || mode == 0x37 {
        10
    } else {
        0
    };
    let ext_size = match mode {
        0x32 => 8,
        0x34 => 19,
        0x35 => 16,
        0x36 => 9,
        0x37 => 6,
        0x3d => 21,
        _ => 0,
    };
    let mut p = vec![mode];
    if mode != 0x3d {
        let mut buttons = s.buttons;
        let [x, y, z] = s.accel;
        if accel {
            buttons |= ((x & 3) << 5) | (((y >> 1) & 1) << 13) | (((z >> 1) & 1) << 14);
        }
        put_buttons(&mut p, buttons);
        if accel {
            p.push((x >> 2) as u8);
            p.push((y >> 2) as u8);
            p.push((z >> 2) as u8);
        }
    }
    if ir_size != 0 {
        update_camera(s, sample);
        if s.ir {
            p.extend_from_slice(
                &s.camera[kCameraDataOffset as usize..kCameraDataOffset as usize + ir_size],
            );
        } else {
            p.resize(p.len() + ir_size, 0xff);
        }
    }
    p.resize(p.len() + ext_size, 0xff);
    send_input(send, context, p);
}
// port of wiimote.cpp SpeakerDecoder::decode; high nibble is expanded before low nibble.
unsafe fn decode(
    decoder: &mut Decoder,
    pcm: bool,
    data: *const u8,
    size: u32,
    out: *mut i16,
) -> u32 {
    if pcm {
        for i in 0..size as usize {
            out.add(i).write((*data.add(i) as i8 as i16) * 256);
        }
        return size;
    }
    fn expand(d: &mut Decoder, n: usize) -> i16 {
        d.predictor = (d.predictor + d.step * kYamahaDifference[n] / 8).clamp(-32768, 32767);
        d.step = ((d.step * kYamahaScale[n]) >> 8).clamp(127, 24576);
        d.predictor as i16
    }
    for i in 0..size as usize {
        out.add(2 * i)
            .write(expand(decoder, ((*data.add(i) >> 4) & 0xf) as usize));
        out.add(2 * i + 1)
            .write(expand(decoder, (*data.add(i) & 0xf) as usize));
    }
    size.wrapping_mul(2)
}
#[no_mangle]
pub unsafe extern "C" fn wprt_wiimote_decode(
    predictor: *mut i32,
    step: *mut i32,
    pcm: bool,
    data: *const u8,
    size: u32,
    out: *mut i16,
) -> u32 {
    let mut decoder = Decoder {
        predictor: *predictor,
        step: *step,
    };
    let n = decode(&mut decoder, pcm, data, size, out);
    predictor.write(decoder.predictor);
    step.write(decoder.step);
    n
}
// port of wiimote.cpp speaker_rate: integer division BEFORE multiply by2.
#[no_mangle]
pub extern "C" fn wprt_wiimote_speaker_rate(pcm: bool, divisor: u16) -> u32 {
    if divisor == 0 {
        0
    } else {
        (if pcm { 12000000u32 } else { 6000000u32 }) / divisor as u32 * 2
    }
}
// port of wiimote.cpp reset.
#[no_mangle]
pub unsafe extern "C" fn wprt_wiimote_reset(index: u32) {
    if index >= kMaxWiimotes {
        return;
    }
    let s = &mut (*STATES.0.get())[index as usize];
    *s = State::new();
    fill_eeprom(s);
    wphw_wiimote_outputs(index, false, 0);
}
// port of wiimote.cpp output_report; preserve every short-packet, busy, invalid-space and muted-decoder branch.
#[no_mangle]
pub unsafe extern "C" fn wprt_wiimote_output_report(
    index: u32,
    data: *const u8,
    size: u32,
    send: Sender,
    context: *mut c_void,
) {
    if index >= kMaxWiimotes || size < 2 {
        return;
    }
    let s = &mut (*STATES.0.get())[index as usize];
    let data = std::slice::from_raw_parts(data, size as usize);
    let id = data[0];
    let body = &data[1..];
    let length = size - 1;
    if wphw_wiimote_log_enabled() {
        wphw_wiimote_log_output(index, id, data.as_ptr(), size);
    }
    let ack = body[0] & 2 != 0;
    let rumble = body[0] & 1 != 0;
    if id == kReportLeds {
        s.leds = body[0] >> 4;
    }
    if rumble != s.rumble || id == kReportLeds {
        s.rumble = rumble;
        wphw_wiimote_outputs(index, s.rumble, s.leds);
    }
    match id {
        kReportRumble => {}
        kReportLeds => {
            if ack {
                send_ack(s, send, context, id, kErrorSuccess);
            }
        }
        kReportMode => {
            if length >= 2 && valid_mode(body[1]) {
                s.continuous = body[0] & 4 != 0;
                s.mode = body[1];
                if ack {
                    send_ack(s, send, context, id, kErrorSuccess);
                }
            }
        }
        kReportIrEnable => {
            s.ir = body[0] & 4 != 0;
            if ack {
                send_ack(s, send, context, id, kErrorSuccess);
            }
        }
        kReportIrEnable2 => {
            if ack {
                send_ack(s, send, context, id, kErrorSuccess);
            }
        }
        kReportSpeakerEnable => {
            s.speaker = body[0] & 4 != 0;
            if ack {
                send_ack(s, send, context, id, kErrorSuccess);
            }
        }
        kReportSpeakerMute => {
            s.speaker_mute = body[0] & 4 != 0;
            if ack {
                send_ack(s, send, context, id, kErrorSuccess);
            }
        }
        kReportRequestStatus => send_status(s, send, context),
        kReportWriteData => {
            if length < 21 {
                return;
            }
            let space = (body[0] >> 2) & 3;
            let slave = body[1] >> 1;
            let address = u16::from_be_bytes([body[2], body[3]]);
            let count = body[4] as usize;
            if count == 0 || count > 16 {
                return;
            }
            let mut error = kErrorSuccess;
            if space == 0 {
                if address as usize + count > kEepromSize as usize {
                    error = kErrorInvalidAddress;
                } else {
                    s.eeprom[address as usize..address as usize + count]
                        .copy_from_slice(&body[5..5 + count]);
                }
            } else if space == 1 || space == 2 {
                let offset = address as u8 as usize;
                match i2c_device(s, slave) {
                    None => error = kErrorNack,
                    Some(_) if offset + count > 0x100 => error = kErrorNack,
                    Some(device) => {
                        device[offset..offset + count].copy_from_slice(&body[5..5 + count])
                    }
                }
            } else {
                error = kErrorInvalidSpace;
            }
            send_ack(s, send, context, id, error);
        }
        kReportReadData => {
            if length < 6 {
                return;
            }
            if s.read.size != 0 {
                send_ack(s, send, context, id, kErrorBusy);
                return;
            }
            s.read.space = (body[0] >> 2) & 3;
            s.read.slave = body[1] >> 1;
            s.read.address = u16::from_be_bytes([body[2], body[3]]);
            s.read.size = u16::from_be_bytes([body[4], body[5]]);
            process_read(s, send, context);
        }
        kReportSpeakerData => {
            let bytes = (body[0] >> 3) as u32;
            if !s.speaker || bytes > 20 || bytes + 1 > length {
                return;
            }
            let format = s.speaker_registers[2];
            let pcm = format == 0x40;
            let divisor = u16::from_le_bytes([s.speaker_registers[3], s.speaker_registers[4]]);
            let rate = wprt_wiimote_speaker_rate(pcm, divisor);
            if rate == 0 || bytes == 0 || (format != 0 && format != 0x40) {
                return;
            }
            let mut samples = [0i16; 40];
            let count = decode(
                &mut s.decoder,
                pcm,
                body.as_ptr().add(1),
                bytes,
                samples.as_mut_ptr(),
            );
            if s.speaker_mute || !wphw_wiimote_speaker_enabled() {
                return;
            }
            let level = s.speaker_registers[5] as u32;
            let divisor_max = if pcm { 0xff } else { 0x7f };
            let gain = (level * 256 / level.max(divisor_max)).min(255);
            wphw_wiimote_push_speaker(index, samples.as_ptr(), count, rate, gain as f32 / 256.0);
        }
        _ => {
            if wphw_wiimote_log_enabled() {
                wphw_wiimote_log_unknown(index, id);
            }
        }
    }
}
// port of wiimote.cpp update; exact original interval skip/read-before-data priority and state-before-Sender ordering.
#[no_mangle]
pub unsafe extern "C" fn wprt_wiimote_update(index: u32, send: Sender, context: *mut c_void) {
    if index >= kMaxWiimotes {
        return;
    }
    let s = &mut (*STATES.0.get())[index as usize];
    let now = wphw_wiimote_now();
    if now < s.next_report {
        return;
    }
    s.next_report = wphw_wiimote_after_interval(now);
    let mut sample = Sample::default();
    wphw_wiimote_sample(index, &mut sample);
    s.buttons = (sample.buttons & 0x9f1f) as u16;
    update_motion(s, &sample);
    if process_read(s, send, context) {
        return;
    }
    let changed = s.buttons != s.reported_buttons
        || (accel_mode(s.mode) && s.accel != s.reported_accel)
        || s.first_report;
    if s.mode == kInputCore && !s.continuous && !changed {
        return;
    }
    s.reported_buttons = s.buttons;
    s.reported_accel = s.accel;
    s.first_report = false;
    send_data_report(s, send, context, &sample);
}
