// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/input/input.cpp, retaining H5 clock/H6 transport adapters.
use std::{
    ffi::{c_char, c_void, CStr},
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        OnceLock,
    },
};
const SIDEWAYS: u32 = 0x800000;
const A: u32 = 0x0800;
// port of engine/include/wp/keymap.h Action::Grip and kActionCount (same enum order).
const GRIP_ACTION: usize = 18;
const ACTION_COUNT: usize = 20;
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
    fn wphw_input_now_ms() -> i64;
    fn wphw_input_origin();
    fn wphw_input_elapsed() -> i32;
    fn wphw_input_frames() -> u64;
    fn wphw_input_flag(which: u32) -> bool;
    fn wphw_input_sideways_count() -> usize;
    fn wphw_input_sideways_loaded(index: usize) -> bool;
    fn wphw_input_notice_module() -> *const c_char;
    fn wphw_input_module_loaded(name: *const c_char) -> bool;
    fn wphw_input_bindings();
    fn wphw_input_key_down(key: i32) -> bool;
    fn wphw_input_codes(index: usize, p: *mut *const i32, n: *mut usize);
    fn wphw_input_action_buttons(index: usize) -> u32;
    fn wphw_input_pad(channel: u32, sideways: bool, out: *mut Sample);
    fn wphw_input_pad_connected(channel: u32) -> bool;
    fn wphw_input_host(channel: u32, out: *mut Sample);
    fn wphw_input_window() -> *mut c_void;
    fn wphw_input_focused() -> bool;
    fn wphw_input_log_focus(window: *mut c_void, focused: bool);
    fn wphw_input_log_buttons(buttons: u32);
    fn wphw_input_cursor(x: *mut i32, y: *mut i32) -> bool;
    fn wphw_input_image(x: *mut f32, y: *mut f32) -> bool;
}
static WHEEL_UP: AtomicI64 = AtomicI64::new(0);
static WHEEL_DOWN: AtomicI64 = AtomicI64::new(0);
static BLOCKED: AtomicBool = AtomicBool::new(false);
static RELEASE_WAIT: [AtomicBool; 4] = [const { AtomicBool::new(false) }; 4];
// port of input.cpp now_ms; chrono remains a native clock leaf to preserve duration_cast precision.
unsafe fn now_ms() -> i64 {
    wphw_input_now_ms()
}
// port of input.cpp sideways_minigame. Non-atomic statics have the original guest-thread ownership.
unsafe fn sideways_minigame() -> bool {
    if !wphw_input_flag(0) {
        return false;
    }
    static mut NEXT: i64 = 0;
    static mut CACHED: bool = false;
    let now = now_ms();
    if now >= NEXT {
        NEXT = now + 100;
        CACHED = false;
        for i in 0..wphw_input_sideways_count() {
            if wphw_input_sideways_loaded(i) {
                CACHED = true;
                break;
            }
        }
    }
    CACHED
}
// port of input.cpp pressed / any_pressed; wheel timestamps retain sequential consistency.
unsafe fn pressed(key: i32) -> bool {
    if key == 0x100 || key == 0x101 {
        let last = if key == 0x100 {
            WHEEL_UP.load(Ordering::SeqCst)
        } else {
            WHEEL_DOWN.load(Ordering::SeqCst)
        };
        return now_ms() - last < 120;
    }
    wphw_input_key_down(key)
}
unsafe fn any_pressed(index: usize) -> bool {
    let (mut p, mut n) = (std::ptr::null(), 0);
    wphw_input_codes(index, &mut p, &mut n);
    if n > 0 {
        for &key in std::slice::from_raw_parts(p, n) {
            if pressed(key) {
                return true;
            }
        }
    }
    false
}
#[derive(Default)]
struct ScriptEntry {
    from: i32,
    to: i32,
    buttons: u32,
    pointer: bool,
    x: f32,
    y: f32,
}
fn terminated(p: &[u8]) -> Vec<u8> {
    let mut r = p.to_vec();
    r.push(0);
    r
}
// port of input.cpp parse_script. getline_parts omits only the terminal empty extraction.
fn getline_parts(text: &[u8], delimiter: u8) -> Vec<&[u8]> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut parts: Vec<_> = text.split(|&c| c == delimiter).collect();
    if text.last() == Some(&delimiter) {
        parts.pop();
    }
    parts
}
unsafe fn parse_script(text: &[u8]) -> Vec<ScriptEntry> {
    let text = &text[..text.iter().position(|&c| c == 0).unwrap_or(text.len())];
    let mut entries = Vec::new();
    for line in getline_parts(text, b';') {
        let parts: Vec<_> = getline_parts(line, b',');
        if parts.len() < 3 {
            continue;
        }
        let mut entry = ScriptEntry {
            from: libc::atoi(terminated(parts[0]).as_ptr().cast()),
            to: libc::atoi(terminated(parts[1]).as_ptr().cast()),
            buttons: libc::strtoul(
                terminated(parts[2]).as_ptr().cast(),
                std::ptr::null_mut(),
                16,
            ) as u32,
            ..Default::default()
        };
        if parts.len() >= 5 {
            entry.pointer = true;
            entry.x = libc::atof(terminated(parts[3]).as_ptr().cast()) as f32;
            entry.y = libc::atof(terminated(parts[4]).as_ptr().cast()) as f32;
        }
        entries.push(entry);
    }
    entries
}
// port of input.cpp scripted: OR overlapping buttons; last active pointer wins.
fn scripted(script: &[ScriptEntry], milliseconds: i32) -> Sample {
    let mut result = Sample::default();
    for entry in script {
        if milliseconds >= entry.from && milliseconds < entry.to {
            result.buttons |= entry.buttons;
            if entry.pointer {
                result.pointer_valid = 1;
                result.pointer_x = entry.x;
                result.pointer_y = entry.y;
            }
        }
    }
    result
}
// port of input.cpp note_wheel.
#[no_mangle]
pub unsafe extern "C" fn wprt_input_note_wheel(delta: i32) {
    if delta > 0 {
        WHEEL_UP.store(now_ms(), Ordering::SeqCst);
    } else if delta < 0 {
        WHEEL_DOWN.store(now_ms(), Ordering::SeqCst);
    }
}
// port of input.cpp sideways_grip; toggled before applying automatic orientation.
unsafe fn sideways_grip(read_toggle: bool) -> bool {
    static mut FLIPPED: bool = false;
    static mut TOGGLE_HELD: bool = false;
    if read_toggle {
        let toggle = any_pressed(GRIP_ACTION);
        if toggle && !TOGGLE_HELD {
            FLIPPED = !FLIPPED;
        }
        TOGGLE_HELD = toggle;
    }
    sideways_minigame() != FLIPPED
}
// port of input.cpp gamepad_sample. Native shim transports the original gamepad::State fields.
unsafe fn gamepad_sample(channel: u32, sideways: bool) -> Sample {
    let mut result = Sample::default();
    wphw_input_pad(channel, sideways, &mut result);
    if sideways {
        result.buttons |= SIDEWAYS;
    }
    result
}
// port of input.cpp connected.
#[no_mangle]
pub unsafe extern "C" fn wprt_input_connected(channel: u32) -> bool {
    channel == 0 || wphw_input_pad_connected(channel)
}
// port of input.cpp device_sample script loading. Binary narrow-path read, missing file gives empty script.
unsafe fn load_script(text: *const c_char) -> Vec<ScriptEntry> {
    let mut text = CStr::from_ptr(text).to_bytes().to_vec();
    if text.first() == Some(&b'@') {
        let path = terminated(&text[1..]);
        let file = libc::fopen(path.as_ptr().cast(), c"rb".as_ptr());
        text.clear();
        if !file.is_null() {
            let mut buffer = [0u8; 4096];
            loop {
                let n = libc::fread(buffer.as_mut_ptr().cast(), 1, buffer.len(), file);
                if n == 0 {
                    break;
                }
                text.extend_from_slice(&buffer[..n]);
            }
            libc::fclose(file);
        }
    }
    parse_script(&text)
}
#[cfg(not(feature = "opt-batch1"))]
#[inline(always)]
unsafe fn device_sample(channel: u32) -> Sample {
    device_sample_uncached(channel)
}
// Claude OPT batch1 R1 (timing-only, normal play only): the raw host device read (Win32 focus/cursor/keyboard, SDL pad)
// is reused for <= WP_INPUT_CACHE_US (default 1000 us, max 4000, 0 = off) per channel. sample()'s per-call logic
// (BLOCKED, RELEASE_WAIT, skipping_notice) and wakes_remote pointer history still run on EVERY call. The cache is never
// used under WP_INPUT_SCRIPT, WP_INPUT_BUTTONS or comparison mode, so the fixtures read the exact original path.
#[cfg(feature = "opt-batch1")]
unsafe fn device_sample(channel: u32) -> Sample {
    static WINDOW: OnceLock<Option<std::time::Duration>> = OnceLock::new();
    let window = *WINDOW.get_or_init(|| {
        let set = |name: &CStr| !libc::getenv(name.as_ptr()).is_null();
        if set(c"WP_INPUT_SCRIPT") || set(c"WP_INPUT_BUTTONS") {
            return None;
        }
        #[cfg(feature = "comparison")]
        if crate::ffi::wphw_comparison_enabled() {
            return None;
        }
        let v = libc::getenv(c"WP_INPUT_CACHE_US".as_ptr());
        let us = if v.is_null() { 1000 } else { libc::strtoul(v, std::ptr::null_mut(), 10) as u64 };
        if us == 0 { None } else { Some(std::time::Duration::from_micros(us.min(4000))) }
    });
    let Some(window) = window else { return device_sample_uncached(channel) };
    if channel >= 4 {
        return device_sample_uncached(channel);
    }
    thread_local! {
        static CACHE: std::cell::Cell<[Option<(std::time::Instant, Sample)>; 4]> = const { std::cell::Cell::new([None; 4]) };
    }
    let now = std::time::Instant::now();
    if let Some((at, cached)) = CACHE.with(std::cell::Cell::get)[channel as usize] {
        if now.duration_since(at) < window {
            return cached;
        }
    }
    let fresh = device_sample_uncached(channel);
    CACHE.with(|c| {
        let mut entries = c.get();
        entries[channel as usize] = Some((now, fresh));
        c.set(entries);
    });
    fresh
}
// port of input.cpp device_sample; H5 scripted clock has priority over H6 host snapshots as before.
unsafe fn device_sample_uncached(channel: u32) -> Sample {
    static SCRIPT_TEXT: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    let text = SCRIPT_TEXT.get_or_init(|| {
        let p = libc::getenv(c"WP_INPUT_SCRIPT".as_ptr());
        if p.is_null() {
            None
        } else {
            Some(terminated(CStr::from_ptr(p).to_bytes()))
        }
    });
    if let Some(text) = text {
        static SCRIPT: OnceLock<Vec<ScriptEntry>> = OnceLock::new();
        let script = SCRIPT.get_or_init(|| load_script(text.as_ptr().cast()));
        wphw_input_origin();
        static BY_FRAMES: OnceLock<bool> = OnceLock::new();
        let by_frames = *BY_FRAMES.get_or_init(|| {
            let p = libc::getenv(c"WP_SCRIPT_CLOCK".as_ptr());
            !p.is_null() && CStr::from_ptr(p).to_bytes() == b"frames"
        });
        let elapsed = if by_frames {
            wphw_input_frames().wrapping_mul(1000).wrapping_div(60) as i32
        } else {
            wphw_input_elapsed()
        };
        if channel != 0 {
            return gamepad_sample(channel, sideways_grip(false));
        }
        return scripted(script, elapsed);
    }
    if !libc::getenv(c"WP_HOST_INPUT".as_ptr()).is_null() {
        let mut result = Sample::default();
        wphw_input_host(channel, &mut result);
        return result;
    }
    let mut result = Sample::default();
    let forced = libc::getenv(c"WP_INPUT_BUTTONS".as_ptr());
    if !forced.is_null() {
        static mut FRAMES: i32 = 0;
        let previous = FRAMES;
        FRAMES = FRAMES.wrapping_add(1);
        if previous > 150 {
            result.buttons = libc::strtoul(forced, std::ptr::null_mut(), 16) as u32;
        }
        return result;
    }
    let window = wphw_input_window();
    static LOG_INPUT: OnceLock<bool> = OnceLock::new();
    let log_input = *LOG_INPUT.get_or_init(|| !libc::getenv(c"WP_LOG_INPUT".as_ptr()).is_null());
    static mut LOGGED_FOCUS: bool = false;
    let focused = wphw_input_focused();
    if log_input && channel == 0 && focused != LOGGED_FOCUS {
        LOGGED_FOCUS = focused;
        wphw_input_log_focus(window, focused);
    }
    if !window.is_null() && !focused {
        return result;
    }
    if channel != 0 {
        return gamepad_sample(channel, sideways_grip(false));
    }
    if window.is_null() {
        return result;
    }
    wphw_input_bindings();
    for i in 0..ACTION_COUNT {
        let buttons = wphw_input_action_buttons(i);
        if buttons != 0 && any_pressed(i) {
            result.buttons |= buttons;
        }
    }
    let sideways = sideways_grip(true);
    if sideways {
        let held = result.buttons;
        result.buttons &= !(0x8 | 0x4 | 0x1 | 0x2);
        result.buttons |= if held & 1 != 0 { 8 } else { 0 }
            | if held & 2 != 0 { 4 } else { 0 }
            | if held & 8 != 0 { 2 } else { 0 }
            | if held & 4 != 0 { 1 } else { 0 };
        result.buttons |= SIDEWAYS;
    }
    let mut pad = Sample::default();
    wphw_input_pad(channel, sideways, &mut pad);
    result.buttons |= pad.buttons;
    if pad.motion_valid != 0 {
        result.motion_valid = 1;
        result.accel = pad.accel;
    }
    let now = now_ms();
    static mut LOGGED_BUTTONS: u32 = 0;
    if log_input && result.buttons != LOGGED_BUTTONS {
        LOGGED_BUTTONS = result.buttons;
        wphw_input_log_buttons(result.buttons);
    }
    static mut LAST_X: i32 = 0;
    static mut LAST_Y: i32 = 0;
    static mut LAST_MOUSE_MOVE: i64 = -1000000;
    let (mut x, mut y) = (0, 0);
    if wphw_input_cursor(&mut x, &mut y) && (x != LAST_X || y != LAST_Y) {
        LAST_X = x;
        LAST_Y = y;
        LAST_MOUSE_MOVE = now;
    }
    let recent = now - LAST_MOUSE_MOVE < 1000;
    if (recent || pad.pointer_valid == 0)
        && wphw_input_image(&mut result.pointer_x, &mut result.pointer_y)
    {
        result.pointer_valid = 1;
    } else if pad.pointer_valid != 0 {
        result.pointer_valid = 1;
        result.pointer_x = pad.pointer_x;
        result.pointer_y = pad.pointer_y;
    }
    result
}
// port of input.cpp skipping_notice: cache script presence, fetch module, then short-circuit the LiveFlag call.
unsafe fn skipping_notice() -> bool {
    static SCRIPTED: OnceLock<bool> = OnceLock::new();
    let scripted = *SCRIPTED.get_or_init(|| {
        !libc::getenv(c"WP_INPUT_SCRIPT".as_ptr()).is_null()
            || !libc::getenv(c"WP_INPUT_BUTTONS".as_ptr()).is_null()
    });
    let module = wphw_input_notice_module();
    if scripted || !wphw_input_flag(2) || module.is_null() || !wphw_input_module_loaded(module) {
        return false;
    }
    now_ms() % 500 < 100
}
// port of input.cpp set_blocked; unblocking arms all four release gates BEFORE clearing block.
#[no_mangle]
pub extern "C" fn wprt_input_set_blocked(blocked: bool) {
    if !blocked {
        for wait in &RELEASE_WAIT {
            wait.store(true, Ordering::SeqCst);
        }
    }
    BLOCKED.store(blocked, Ordering::SeqCst);
}
// port of input.cpp sample.
unsafe fn sample(channel: u32) -> Sample {
    let mut result = device_sample(channel);
    let held = result.buttons & !SIDEWAYS;
    let waiting = channel < 4 && RELEASE_WAIT[channel as usize].load(Ordering::SeqCst);
    if BLOCKED.load(Ordering::SeqCst) || (waiting && held != 0) {
        result.buttons &= SIDEWAYS;
        return result;
    }
    if waiting {
        RELEASE_WAIT[channel as usize].store(false, Ordering::SeqCst);
    }
    if channel == 0 && skipping_notice() {
        result.buttons |= A;
    }
    result
}
#[no_mangle]
pub unsafe extern "C" fn wprt_input_sample(channel: u32, out: *mut Sample) {
    out.write(sample(channel));
}
// port of input.cpp wakes_remote. Pointer history updates even when wake_on_mouse is false.
#[no_mangle]
pub unsafe extern "C" fn wprt_input_wakes_remote(channel: u32) -> bool {
    let current = sample(channel);
    if current.buttons & !SIDEWAYS != 0 {
        return true;
    }
    let wake = wphw_input_flag(1);
    static mut HAD_POINTER: bool = false;
    static mut LAST_X: f32 = 0.;
    static mut LAST_Y: f32 = 0.;
    let moved = wake
        && channel == 0
        && current.pointer_valid != 0
        && HAD_POINTER
        && (current.pointer_x != LAST_X || current.pointer_y != LAST_Y);
    if channel == 0 {
        HAD_POINTER = current.pointer_valid != 0;
        LAST_X = current.pointer_x;
        LAST_Y = current.pointer_y;
    }
    moved
}
