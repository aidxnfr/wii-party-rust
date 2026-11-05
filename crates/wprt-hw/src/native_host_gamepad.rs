// SPDX-License-Identifier: GPL-3.0-or-later
// PRIVATE G32 GUEST DRAFT - UNCOMPILED/UNINTEGRATED/UNACCEPTED.
// Source: accepted native_gamepad.rs / pristine input/gamepad.cpp menu_buttons+poll.
use std::sync::Mutex;
const SLOTS: usize = 4; // port of wp/gamepad.h kSlots.
const GRAVITY: f32 = 9.80665;
const STICK_THRESHOLD: f32 = 0.5;
const TRIGGER_THRESHOLD: f32 = 0.5;
use crate::native_gamepad_adapter::State;
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
#[repr(C)]#[derive(Clone,Copy,Default)]
pub struct RawSnapshot{
 pub connected:u32,pub buttons:[u8;26],pub left_x:f32,pub left_y:f32,
 pub right_trigger:f32,pub left_trigger:f32,pub has_accel:u32,pub accel:[f32;3],
 pub pointer_valid:u32,pub pointer_x:f32,pub pointer_y:f32,pub nintendo_layout:u32,
}
#[repr(C)]#[derive(Clone,Copy)]pub struct Collector{
 pub context:usize,
 pub snapshot:unsafe extern "C" fn(usize,u32,*mut RawSnapshot),
 pub outputs:unsafe extern "C" fn(usize,u32,bool,u8),
}
static COLLECTOR:Mutex<Option<Collector>>=Mutex::new(None);
pub fn bound()->bool{COLLECTOR.lock().unwrap().is_some()}
#[no_mangle]pub unsafe extern "C" fn wphost_controller_bind(value:*const Collector){*COLLECTOR.lock().unwrap()=Some(*value);}
extern "C"{
 fn wphw_gamepad_enabled()->bool;
 fn wprt_padmap_current_mapping(values:*mut i32);
 fn wprt_padmap_buttons(held:*const u8,nintendo:bool,sideways:bool,mapping:*const i32)->u32;
}
// Source Snapshot defaults and enabled/slot guards, new raw host service only.
unsafe fn snapshot(slot:u32)->Snapshot{
 if slot>=4||!wphw_gamepad_enabled(){return Snapshot::default();}
 let Some(c)=*COLLECTOR.lock().unwrap()else{return Snapshot::default()};
 let mut r=RawSnapshot::default();(c.snapshot)(c.context,slot,&mut r);
 Snapshot{connected:r.connected!=0,buttons:r.buttons.map(|v|v!=0),left_x:r.left_x,left_y:r.left_y,
 right_trigger:r.right_trigger,left_trigger:r.left_trigger,has_accel:r.has_accel!=0,accel:r.accel,
 pointer_valid:r.pointer_valid!=0,pointer_x:r.pointer_x,pointer_y:r.pointer_y,nintendo_layout:r.nintendo_layout!=0}
}
pub unsafe fn connected(slot:u32)->bool{snapshot(slot).connected}
pub unsafe fn set_outputs(slot:u32,rumble:bool,leds:u8){
 if slot>=4||!wphw_gamepad_enabled(){return;}
 if let Some(c)=*COLLECTOR.lock().unwrap(){(c.outputs)(c.context,slot,rumble,leds);}
}
pub unsafe fn menu_buttons() -> u32 {
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
pub unsafe fn poll(slot: u32, sideways: bool) -> State {
    let c = snapshot(slot);
    let mut s = State{connected:false,buttons:0,pointer_valid:false,pointer_x:0.0,pointer_y:0.0,motion_valid:false,accel:[0.0,0.0,1.0]};
    if !c.connected {
        return s;
    }
    s.connected = true;
    let mut held = [0u8; PAD_KPHYSICALCOUNT];
    for i in 0..PAD_KLEFTTRIGGER {
        held[i] = c.buttons[i] as u8;
    }
    held[PAD_KLEFTTRIGGER] = (c.left_trigger > TRIGGER_THRESHOLD) as u8;
    held[PAD_KRIGHTTRIGGER] = (c.right_trigger > TRIGGER_THRESHOLD) as u8;
    let mut mapping = [0i32; PAD_ACTION_COUNT];
    wprt_padmap_current_mapping(mapping.as_mut_ptr());
    s.buttons |= wprt_padmap_buttons(
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
    s.pointer_valid = c.pointer_valid;
    s.pointer_x = c.pointer_x;
    s.pointer_y = c.pointer_y;
    if c.has_accel {
        let x = c.accel[0] / GRAVITY;
        let y = c.accel[1] / GRAVITY;
        let z = c.accel[2] / GRAVITY;
        s.motion_valid = true;
        if sideways {
            s.accel[0] = z;
            s.accel[1] = -x;
        } else {
            s.accel[0] = -x;
            s.accel[1] = -z;
        }
        s.accel[2] = y;
    }
    s
}

