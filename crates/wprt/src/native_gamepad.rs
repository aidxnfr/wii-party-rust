// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted source port of pristine engine/src/input/gamepad.cpp. PRIVATE DRAFT: UNBUILT/UNINTEGRATED.
// SDL enums extracted programmatically from pinned SDL3-3.4.16/SDL_gamepad.h.
const SDL_GAMEPAD_BUTTON_INVALID: i32 = -1;
const SDL_GAMEPAD_BUTTON_SOUTH: i32 = 0;
const SDL_GAMEPAD_BUTTON_EAST: i32 = 1;
const SDL_GAMEPAD_BUTTON_WEST: i32 = 2;
const SDL_GAMEPAD_BUTTON_NORTH: i32 = 3;
const SDL_GAMEPAD_BUTTON_BACK: i32 = 4;
const SDL_GAMEPAD_BUTTON_GUIDE: i32 = 5;
const SDL_GAMEPAD_BUTTON_START: i32 = 6;
const SDL_GAMEPAD_BUTTON_LEFT_STICK: i32 = 7;
const SDL_GAMEPAD_BUTTON_RIGHT_STICK: i32 = 8;
const SDL_GAMEPAD_BUTTON_LEFT_SHOULDER: i32 = 9;
const SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER: i32 = 10;
const SDL_GAMEPAD_BUTTON_DPAD_UP: i32 = 11;
const SDL_GAMEPAD_BUTTON_DPAD_DOWN: i32 = 12;
const SDL_GAMEPAD_BUTTON_DPAD_LEFT: i32 = 13;
const SDL_GAMEPAD_BUTTON_DPAD_RIGHT: i32 = 14;
const SDL_GAMEPAD_BUTTON_MISC1: i32 = 15;
const SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1: i32 = 16;
const SDL_GAMEPAD_BUTTON_LEFT_PADDLE1: i32 = 17;
const SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2: i32 = 18;
const SDL_GAMEPAD_BUTTON_LEFT_PADDLE2: i32 = 19;
const SDL_GAMEPAD_BUTTON_TOUCHPAD: i32 = 20;
const SDL_GAMEPAD_BUTTON_MISC2: i32 = 21;
const SDL_GAMEPAD_BUTTON_MISC3: i32 = 22;
const SDL_GAMEPAD_BUTTON_MISC4: i32 = 23;
const SDL_GAMEPAD_BUTTON_MISC5: i32 = 24;
const SDL_GAMEPAD_BUTTON_MISC6: i32 = 25;
const SDL_GAMEPAD_BUTTON_COUNT: i32 = 26;
const SDL_GAMEPAD_AXIS_INVALID: i32 = -1;
const SDL_GAMEPAD_AXIS_LEFTX: i32 = 0;
const SDL_GAMEPAD_AXIS_LEFTY: i32 = 1;
const SDL_GAMEPAD_AXIS_RIGHTX: i32 = 2;
const SDL_GAMEPAD_AXIS_RIGHTY: i32 = 3;
const SDL_GAMEPAD_AXIS_LEFT_TRIGGER: i32 = 4;
const SDL_GAMEPAD_AXIS_RIGHT_TRIGGER: i32 = 5;
const SDL_GAMEPAD_AXIS_COUNT: i32 = 6;
// Enum counts extracted from pristine wp/padmap.h; no physical-ID assumptions.
const PAD_KLEFTTRIGGER: usize = 15;
const PAD_KRIGHTTRIGGER: usize = 16;
const PAD_KPHYSICALCOUNT: usize = 17;
const PAD_ACTION_COUNT: usize = 7;
use std::{
    ffi::{c_char, c_void},
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Mutex, Once,
    },
};
const SLOTS: usize = 4; // port of wp/gamepad.h kSlots.
const GRAVITY: f32 = 9.80665;
const STICK_THRESHOLD: f32 = 0.5;
const TRIGGER_THRESHOLD: f32 = 0.5;
const POINTER_PER_RADIAN: f32 = 2.3;
const STICK_POINTER_SPEED: f32 = 1.6;
const STICK_DEAD_ZONE: f32 = 0.15;
const POINTER_LIMIT: f32 = 1.3;
const STILL_GYRO: f32 = 0.08;
const STILL_ACCEL: f32 = 0.4;
const STILL_SECONDS: f32 = 1.0;
// Narrow native SDK/chrono/format leaves; adapter not built yet. All gamepad decisions below are Rust.
extern "C" {
    fn wphw_gamepad_enabled() -> bool;
    fn wphw_gamepad_axis(handle: *mut c_void, which: i32) -> i16;
    fn wphw_gamepad_open(id: u32) -> *mut c_void;
    fn wphw_gamepad_close(handle: *mut c_void);
    fn wphw_gamepad_has_sensor(handle: *mut c_void, gyro: bool) -> bool;
    fn wphw_gamepad_enable_sensor(handle: *mut c_void, gyro: bool) -> bool;
    fn wphw_gamepad_nintendo(handle: *mut c_void) -> bool;
    fn wphw_gamepad_name(handle: *mut c_void) -> *const c_char;
    fn wphw_gamepad_log_open(
        name: *const c_char,
        slot: u32,
        gyro: bool,
        accel: bool,
        nintendo: bool,
    );
    fn wphw_gamepad_log_close(slot: u32);
    fn wphw_gamepad_button(handle: *mut c_void, button: i32) -> bool;
    fn wphw_gamepad_sensor(handle: *mut c_void, gyro: bool, out: *mut f32) -> bool;
    fn wphw_gamepad_now() -> i64;
    fn wphw_gamepad_after_ms(now: i64, ms: i64) -> i64;
    fn wphw_gamepad_elapsed_ms(now: i64, origin: i64) -> i64;
    fn wphw_gamepad_elapsed_float(now: i64, previous: i64) -> f32;
    fn wphw_gamepad_rumble(handle: *mut c_void, low: u16, high: u16, duration: u32);
    fn wphw_gamepad_player(handle: *mut c_void, player: i32);
    fn wphw_gamepad_virtual_attach(number: *mut u32) -> u32;
    fn wphw_gamepad_joystick_open(id: u32) -> *mut c_void;
    fn wphw_gamepad_joystick_close(handle: *mut c_void);
    fn wphw_gamepad_virtual_detach(id: u32);
    fn wphw_gamepad_virtual_button(handle: *mut c_void, button: i32, pressed: bool);
    fn wphw_gamepad_ticks_ns() -> u64;
    fn wphw_gamepad_virtual_sensor(
        handle: *mut c_void,
        gyro: bool,
        timestamp: u64,
        data: *const f32,
    );
    fn wphw_gamepad_hint(which: u32, value: bool);
    fn wphw_gamepad_setting(which: u32) -> bool;
    fn wphw_gamepad_init() -> bool;
    fn wphw_gamepad_log_unavailable();
    fn wphw_gamepad_update();
    fn wphw_gamepad_owns_queue() -> bool;
    fn wphw_gamepad_pump();
    fn wphw_gamepad_event(own_queue: bool, kind: *mut u32, id: *mut u32) -> bool;
    fn wphw_gamepad_sleep_ms(ms: u64);
}
#[derive(Clone, Copy)]
struct Snapshot {
    connected: bool,
    buttons: [bool; SDL_GAMEPAD_BUTTON_COUNT as usize],
    left_x: f32,
    left_y: f32,
    right_trigger: f32,
    left_trigger: f32,
    has_accel: bool,
    accel: [f32; 3],
    pointer_valid: bool,
    pointer_x: f32,
    pointer_y: f32,
    nintendo_layout: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            connected: false,
            buttons: [false; SDL_GAMEPAD_BUTTON_COUNT as usize],
            left_x: 0.,
            left_y: 0.,
            right_trigger: 0.,
            left_trigger: 0.,
            has_accel: false,
            accel: [0.; 3],
            pointer_valid: false,
            pointer_x: 0.,
            pointer_y: 0.,
            nintendo_layout: false,
        }
    }
}
struct Pad {
    handle: *mut c_void,
    id: u32,
    has_gyro: bool,
    has_accel: bool,
    nintendo_layout: bool,
    pointer_x: f32,
    pointer_y: f32,
    pointer_active: bool,
    recenter_held: bool,
    bias: [f32; 3],
    still_sum: [f32; 3],
    still_accel: [f32; 3],
    still_time: f32,
    still_samples: i32,
    rumble: bool,
    rumble_refresh: i64,
    player: i32,
}
impl Default for Pad {
    fn default() -> Self {
        Self {
            handle: std::ptr::null_mut(),
            id: 0,
            has_gyro: false,
            has_accel: false,
            nintendo_layout: false,
            pointer_x: 0.,
            pointer_y: 0.,
            pointer_active: false,
            recenter_held: false,
            bias: [0.; 3],
            still_sum: [0.; 3],
            still_accel: [0.; 3],
            still_time: 0.,
            still_samples: 0,
            rumble: false,
            rumble_refresh: 0,
            player: -1,
        }
    }
}
static SNAPSHOTS: Mutex<[Snapshot; SLOTS]> = Mutex::new(
    [Snapshot {
        connected: false,
        buttons: [false; SDL_GAMEPAD_BUTTON_COUNT as usize],
        left_x: 0.,
        left_y: 0.,
        right_trigger: 0.,
        left_trigger: 0.,
        has_accel: false,
        accel: [0.; 3],
        pointer_valid: false,
        pointer_x: 0.,
        pointer_y: 0.,
        nintendo_layout: false,
    }; SLOTS],
);
static RUMBLE: [AtomicBool; SLOTS] = [const { AtomicBool::new(false) }; SLOTS];
static PLAYER: [AtomicI32; SLOTS] = [const { AtomicI32::new(0) }; SLOTS];
static STARTED: Once = Once::new();
// port of input/gamepad.cpp axis. Keep asymmetric signed-axis normalization by 32767.
unsafe fn axis(handle: *mut c_void, which: i32) -> f32 {
    wphw_gamepad_axis(handle, which) as f32 / 32767.0
}
// port of input/gamepad.cpp open: duplicate IDs return; first free slot and first open failure return.
unsafe fn open(pads: &mut [Pad; SLOTS], id: u32) {
    for pad in pads.iter() {
        if !pad.handle.is_null() && pad.id == id {
            return;
        }
    }
    for (slot, pad) in pads.iter_mut().enumerate() {
        if !pad.handle.is_null() {
            continue;
        }
        let handle = wphw_gamepad_open(id);
        if handle.is_null() {
            return;
        }
        *pad = Pad::default();
        pad.handle = handle;
        pad.id = id;
        pad.has_gyro =
            wphw_gamepad_has_sensor(handle, true) && wphw_gamepad_enable_sensor(handle, true);
        pad.has_accel =
            wphw_gamepad_has_sensor(handle, false) && wphw_gamepad_enable_sensor(handle, false);
        pad.pointer_active = pad.has_gyro;
        pad.nintendo_layout = wphw_gamepad_nintendo(handle);
        let name = wphw_gamepad_name(handle);
        wphw_gamepad_log_open(
            name,
            slot as u32 + 1,
            pad.has_gyro,
            pad.has_accel,
            pad.nintendo_layout,
        );
        return;
    }
}
// port of input/gamepad.cpp close; source visits all matching slots rather than stopping after one.
unsafe fn close(pads: &mut [Pad; SLOTS], id: u32) {
    for (slot, pad) in pads.iter_mut().enumerate() {
        if !pad.handle.is_null() && pad.id == id {
            wphw_gamepad_close(pad.handle);
            *pad = Pad::default();
            wphw_gamepad_log_close(slot as u32 + 1);
        }
    }
}
// port of std::max<float> as used by source calibrate: preserve first operand on unordered/equal.
fn source_max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
// port of input/gamepad.cpp calibrate, preserving reset/accumulation order and integer sample count.
fn calibrate(pad: &mut Pad, gyro: &[f32; 3], accel: &[f32; 3], dt: f32) {
    let (mut change, mut rate) = (0., 0.);
    for i in 0..3 {
        rate = source_max(rate, (gyro[i] - pad.bias[i]).abs());
        if pad.still_samples > 0 {
            change = source_max(change, (accel[i] - pad.still_accel[i]).abs());
        }
    }
    if rate > STILL_GYRO || change > STILL_ACCEL {
        pad.still_time = 0.;
        pad.still_samples = 0;
        pad.still_sum = [0.; 3];
        return;
    }
    if pad.still_samples == 0 {
        pad.still_accel = *accel;
    }
    for i in 0..3 {
        pad.still_sum[i] += gyro[i];
    }
    pad.still_samples = pad.still_samples.wrapping_add(1);
    pad.still_time += dt;
    if pad.still_time >= STILL_SECONDS {
        for i in 0..3 {
            pad.bias[i] = pad.still_sum[i] / pad.still_samples as f32;
            pad.still_sum[i] = 0.;
        }
        pad.still_time = 0.;
        pad.still_samples = 0;
    }
}
// port of std::clamp<float> for source pointer limits (no invented NaN normalization).
fn pointer_clamp(v: f32) -> f32 {
    if v < -POINTER_LIMIT {
        -POINTER_LIMIT
    } else if POINTER_LIMIT < v {
        POINTER_LIMIT
    } else {
        v
    }
}
// port of input/gamepad.cpp read. Stick, sensors, recenter edge and strict inside-image checks keep source order.
unsafe fn read(pad: &mut Pad, dt: f32) -> Snapshot {
    let mut s = Snapshot::default();
    let handle = pad.handle;
    s.connected = true;
    s.nintendo_layout = pad.nintendo_layout;
    for i in 0..SDL_GAMEPAD_BUTTON_COUNT {
        s.buttons[i as usize] = wphw_gamepad_button(handle, i);
    }
    s.left_x = axis(handle, SDL_GAMEPAD_AXIS_LEFTX);
    s.left_y = axis(handle, SDL_GAMEPAD_AXIS_LEFTY);
    s.right_trigger = axis(handle, SDL_GAMEPAD_AXIS_RIGHT_TRIGGER);
    s.left_trigger = axis(handle, SDL_GAMEPAD_AXIS_LEFT_TRIGGER);
    let x = axis(handle, SDL_GAMEPAD_AXIS_RIGHTX);
    let y = axis(handle, SDL_GAMEPAD_AXIS_RIGHTY);
    if x.abs() > STICK_DEAD_ZONE || y.abs() > STICK_DEAD_ZONE {
        pad.pointer_active = true;
        pad.pointer_x += x * STICK_POINTER_SPEED * dt;
        pad.pointer_y += y * STICK_POINTER_SPEED * dt;
    }
    if pad.has_accel {
        s.has_accel = wphw_gamepad_sensor(handle, false, s.accel.as_mut_ptr());
    }
    let mut gyro = [0.; 3];
    if pad.has_gyro && wphw_gamepad_sensor(handle, true, gyro.as_mut_ptr()) {
        if s.has_accel {
            calibrate(pad, &gyro, &s.accel, dt);
        }
        pad.pointer_x -= (gyro[1] - pad.bias[1]) * dt * POINTER_PER_RADIAN;
        pad.pointer_y -= (gyro[0] - pad.bias[0]) * dt * POINTER_PER_RADIAN;
    }
    let recenter = s.buttons[SDL_GAMEPAD_BUTTON_RIGHT_STICK as usize];
    if recenter && !pad.recenter_held {
        pad.pointer_x = 0.;
        pad.pointer_y = 0.;
        pad.pointer_active = true;
    }
    pad.recenter_held = recenter;
    pad.pointer_x = pointer_clamp(pad.pointer_x);
    pad.pointer_y = pointer_clamp(pad.pointer_y);
    s.pointer_valid = pad.pointer_active && pad.pointer_x.abs() < 1. && pad.pointer_y.abs() < 1.;
    s.pointer_x = pad.pointer_x;
    s.pointer_y = pad.pointer_y;
    s
}
// port of input/gamepad.cpp apply_outputs; compare original chrono ticks, refresh only active rumble.
unsafe fn apply_outputs(pad: &mut Pad, slot: usize) {
    let rumble = RUMBLE[slot].load(Ordering::SeqCst);
    let now = wphw_gamepad_now();
    if rumble != pad.rumble || (rumble && now >= pad.rumble_refresh) {
        pad.rumble = rumble;
        pad.rumble_refresh = wphw_gamepad_after_ms(now, 1000);
        let strength = if rumble { 0xC000 } else { 0 };
        wphw_gamepad_rumble(
            pad.handle,
            strength,
            strength,
            if rumble { 2000 } else { 0 },
        );
    }
    let player = PLAYER[slot].load(Ordering::SeqCst);
    if player != pad.player {
        pad.player = player;
        wphw_gamepad_player(pad.handle, player);
    }
}
struct Virtual {
    id: u32,
    joystick: *mut c_void,
    number: u32,
}
// port of input/gamepad.cpp Virtual member initializers.
impl Default for Virtual { fn default()->Self { Self{id:0,joystick:std::ptr::null_mut(),number:0} } }
// port of input/gamepad.cpp attach_virtual. Native SDL descriptor/callback ABI stays in the SDK leaf.
unsafe fn attach_virtual(virtuals: &mut [Virtual; SLOTS]) {
    let p = libc::getenv(c"WP_VIRTUAL_GAMEPADS".as_ptr());
    let count = if p.is_null() {
        0
    } else {
        libc::atoi(p).clamp(0, SLOTS as i32)
    };
    for i in 0..count as usize {
        let pad = &mut virtuals[i];
        pad.number = i as u32 + 1;
        pad.id = wphw_gamepad_virtual_attach(&mut pad.number);
        pad.joystick = if pad.id != 0 {
            wphw_gamepad_joystick_open(pad.id)
        } else {
            std::ptr::null_mut()
        };
    }
}
// port of input/gamepad.cpp drive_virtual, including source first-slot detach guard and repeated clock reads.
unsafe fn drive_virtual(virtuals: &mut [Virtual; SLOTS]) {
    static ORIGIN: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
    let origin = *ORIGIN.get_or_init(|| wphw_gamepad_now());
    // Source caches getenv's pointer, then calls atoll AFTER elapsed on each iteration.
    static UNTIL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let until = *UNTIL.get_or_init(|| libc::getenv(c"WP_VIRTUAL_GAMEPADS_UNTIL".as_ptr()) as usize) as *const c_char;
    let elapsed = wphw_gamepad_elapsed_ms(wphw_gamepad_now(), origin);
    if !until.is_null() && elapsed >= libc::atoll(until) && !virtuals[0].joystick.is_null() {
        for pad in virtuals.iter_mut() {
            if !pad.joystick.is_null() {
                wphw_gamepad_joystick_close(pad.joystick);
                wphw_gamepad_virtual_detach(pad.id);
                pad.joystick = std::ptr::null_mut();
            }
        }
    }
    let tap = elapsed % 3000 < 100;
    let accel = [0., GRAVITY, 0.];
    let gyro = [0.; 3];
    for pad in virtuals.iter_mut() {
        if !pad.joystick.is_null() {
            wphw_gamepad_virtual_button(pad.joystick, SDL_GAMEPAD_BUTTON_SOUTH, tap);
            wphw_gamepad_virtual_sensor(
                pad.joystick,
                false,
                wphw_gamepad_ticks_ns(),
                accel.as_ptr(),
            );
            wphw_gamepad_virtual_sensor(pad.joystick, true, wphw_gamepad_ticks_ns(), gyro.as_ptr());
        }
    }
}
// port of input/gamepad.cpp worker; same single detached polling worker and one snapshot-array commit/iteration.
unsafe fn worker() {
    wphw_gamepad_hint(0, true);
    wphw_gamepad_hint(1, true);
    wphw_gamepad_hint(2, wphw_gamepad_setting(0));
    wphw_gamepad_hint(3, false);
    if wphw_gamepad_setting(1) {
        wphw_gamepad_hint(4, false);
    }
    if !wphw_gamepad_init() {
        wphw_gamepad_log_unavailable();
        return;
    }
    let mut pads = std::array::from_fn(|_| Pad::default());
    let mut virtuals = std::array::from_fn(|_| Virtual::default());
    attach_virtual(&mut virtuals);
    let mut previous = wphw_gamepad_now();
    loop {
        drive_virtual(&mut virtuals);
        wphw_gamepad_update();
        let own = wphw_gamepad_owns_queue();
        if own {
            wphw_gamepad_pump();
        }
        let (mut kind, mut id) = (0, 0);
        while wphw_gamepad_event(own, &mut kind, &mut id) {
            if kind == 1 {
                open(&mut pads, id);
            } else if kind == 2 {
                close(&mut pads, id);
            }
        }
        let now = wphw_gamepad_now();
        let dt = wphw_gamepad_elapsed_float(now, previous);
        previous = now;
        let mut snapshots = std::array::from_fn(|_| Snapshot::default());
        for slot in 0..SLOTS {
            if !pads[slot].handle.is_null() {
                snapshots[slot] = read(&mut pads[slot], dt);
                apply_outputs(&mut pads[slot], slot);
            }
        }
        {
            *SNAPSHOTS.lock().unwrap() = snapshots;
        }
        wphw_gamepad_sleep_ms(4);
    }
}
// port of input/gamepad.cpp start; release JoinHandle detaches the one source worker.
fn start() {
    STARTED.call_once(|| {
        for player in &PLAYER {
            player.store(-1, Ordering::SeqCst);
        }
        std::thread::spawn(|| unsafe { worker() });
    });
}
// port of input/gamepad.cpp snapshot; invalid/disabled slot must not start worker.
unsafe fn snapshot(slot: u32) -> Snapshot {
    if slot >= SLOTS as u32 || !wphw_gamepad_enabled() {
        return Snapshot::default();
    }
    start();
    SNAPSHOTS.lock().unwrap()[slot as usize]
}
// port of input/gamepad.cpp connected.
#[no_mangle]
pub unsafe extern "C" fn wprt_gamepad_connected(slot: u32) -> bool {
    snapshot(slot).connected
}
// port of input/gamepad.cpp set_outputs; only a single LED selects a player, last exact match preserved.
#[no_mangle]
pub unsafe extern "C" fn wprt_gamepad_set_outputs(slot: u32, rumble: bool, leds: u8) {
    if slot >= SLOTS as u32 || !wphw_gamepad_enabled() {
        return;
    }
    let mut player = -1;
    for i in 0..4 {
        if leds == 1 << i {
            player = i;
        }
    }
    RUMBLE[slot as usize].store(rumble, Ordering::SeqCst);
    PLAYER[slot as usize].store(player, Ordering::SeqCst);
}
// port of input/gamepad.cpp menu_buttons. Flags from wp/gamepad.h, exact layout-dependent face buttons.
#[no_mangle]
pub unsafe extern "C" fn wprt_gamepad_menu_buttons() -> u32 {
    let mut result = 0;
    for slot in 0..SLOTS as u32 {
        let c = snapshot(slot);
        if !c.connected {
            continue;
        }
        let held = |b: i32| c.buttons[b as usize];
        let accept = if c.nintendo_layout {
            SDL_GAMEPAD_BUTTON_EAST
        } else {
            SDL_GAMEPAD_BUTTON_SOUTH
        };
        let cancel = if c.nintendo_layout {
            SDL_GAMEPAD_BUTTON_SOUTH
        } else {
            SDL_GAMEPAD_BUTTON_EAST
        };
        if held(SDL_GAMEPAD_BUTTON_DPAD_UP) || c.left_y < -STICK_THRESHOLD {
            result |= 1;
        }
        if held(SDL_GAMEPAD_BUTTON_DPAD_DOWN) || c.left_y > STICK_THRESHOLD {
            result |= 2;
        }
        if held(SDL_GAMEPAD_BUTTON_DPAD_LEFT) || c.left_x < -STICK_THRESHOLD {
            result |= 4;
        }
        if held(SDL_GAMEPAD_BUTTON_DPAD_RIGHT) || c.left_x > STICK_THRESHOLD {
            result |= 8;
        }
        if held(accept) {
            result |= 0x10;
        }
        if held(cancel) {
            result |= 0x20;
        }
        if held(SDL_GAMEPAD_BUTTON_START)
            && (held(SDL_GAMEPAD_BUTTON_GUIDE) || held(SDL_GAMEPAD_BUTTON_BACK))
        {
            result |= 0x40;
        }
    }
    result
}
#[repr(C)]
pub struct State {
    connected: u32,
    buttons: u32,
    pointer_valid: u32,
    pointer_x: f32,
    pointer_y: f32,
    motion_valid: u32,
    accel: [f32; 3],
}
impl Default for State {
    fn default() -> Self {
        Self {
            connected: 0,
            buttons: 0,
            pointer_valid: 0,
            pointer_x: 0.,
            pointer_y: 0.,
            motion_valid: 0,
            accel: [0., 0., 1.],
        }
    }
}
// port of input/gamepad.cpp poll; reuse accepted Rust padmap rules and native field-independent byte transport.
#[no_mangle]
pub unsafe extern "C" fn wprt_gamepad_poll(slot: u32, sideways: bool, out: *mut State) {
    let c = snapshot(slot);
    let mut s = State::default();
    if !c.connected {
        out.write(s);
        return;
    }
    s.connected = 1;
    let mut held = [0u8; PAD_KPHYSICALCOUNT];
    for i in 0..PAD_KLEFTTRIGGER {
        held[i] = c.buttons[i] as u8;
    }
    held[PAD_KLEFTTRIGGER] = (c.left_trigger > TRIGGER_THRESHOLD) as u8;
    held[PAD_KRIGHTTRIGGER] = (c.right_trigger > TRIGGER_THRESHOLD) as u8;
    let mut mapping = [0i32; PAD_ACTION_COUNT];
    crate::native_padmap::wprt_padmap_current_mapping(mapping.as_mut_ptr());
    s.buttons |= crate::native_padmap::wprt_padmap_buttons(
        held.as_ptr(),
        c.nintendo_layout,
        sideways,
        mapping.as_ptr(),
    );
    let left = c.buttons[SDL_GAMEPAD_BUTTON_DPAD_LEFT as usize] || c.left_x < -STICK_THRESHOLD;
    let right = c.buttons[SDL_GAMEPAD_BUTTON_DPAD_RIGHT as usize] || c.left_x > STICK_THRESHOLD;
    let up = c.buttons[SDL_GAMEPAD_BUTTON_DPAD_UP as usize] || c.left_y < -STICK_THRESHOLD;
    let down = c.buttons[SDL_GAMEPAD_BUTTON_DPAD_DOWN as usize] || c.left_y > STICK_THRESHOLD;
    if sideways {
        s.buttons |= if left { 8 } else { 0 }
            | if right { 4 } else { 0 }
            | if up { 2 } else { 0 }
            | if down { 1 } else { 0 };
    } else {
        s.buttons |= if left { 1 } else { 0 }
            | if right { 2 } else { 0 }
            | if up { 8 } else { 0 }
            | if down { 4 } else { 0 };
    }
    s.pointer_valid = c.pointer_valid as u32;
    s.pointer_x = c.pointer_x;
    s.pointer_y = c.pointer_y;
    if c.has_accel {
        let x = c.accel[0] / GRAVITY;
        let y = c.accel[1] / GRAVITY;
        let z = c.accel[2] / GRAVITY;
        s.motion_valid = 1;
        if sideways {
            s.accel[0] = z;
            s.accel[1] = -x;
        } else {
            s.accel[0] = -x;
            s.accel[1] = -z;
        }
        s.accel[2] = y;
    }
    out.write(s);
}
