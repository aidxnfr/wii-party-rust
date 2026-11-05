// SPDX-License-Identifier: GPL-3.0-or-later
// PRIVATE G33 source key-binding compatibility candidate, UNCOMPILED/UNINTEGRATED.
// Port of pristine input.cpp pressed/any_pressed/action loop;existing accepted
// video::key_down SDK service retained,NOT a new Win32 leaf port or pure-Bevy claim.
use std::sync::atomic::{AtomicBool,AtomicI64,Ordering};
// H8 optional Bevy event provider. False means unsupported/unobserved: preserve original SDK query.
pub type EventQuery=unsafe extern "C" fn(i32,*mut bool)->bool;
static EVENTS:std::sync::OnceLock<EventQuery>=std::sync::OnceLock::new();
pub fn bind_events(query:EventQuery)->bool{EVENTS.set(query).is_ok()}
static BOUND:AtomicBool=AtomicBool::new(false);
static WHEEL_UP:AtomicI64=AtomicI64::new(0);
static WHEEL_DOWN:AtomicI64=AtomicI64::new(0);
extern "C"{
 fn wphw_input_codes(index:usize,p:*mut *const i32,n:*mut usize);
 fn wphw_input_action_buttons(index:usize)->u32;
 fn wphw_input_key_down(code:i32)->bool;
 fn wphw_input_now_ms()->i64;
}
#[no_mangle]pub extern "C" fn wphost_keyboard_bind(){BOUND.store(true,Ordering::Release);}
pub fn bound()->bool{BOUND.load(Ordering::Acquire)}
// Source note_wheel sign semantics;clock/120ms decision retained in the DLL.
#[no_mangle]pub unsafe extern "C" fn wphost_keyboard_wheel(delta:i32){
 if delta>0{WHEEL_UP.store(wphw_input_now_ms(),Ordering::SeqCst);}
 else if delta<0{WHEEL_DOWN.store(wphw_input_now_ms(),Ordering::SeqCst);}
}
unsafe fn pressed(key:i32)->bool{
 if key==0x100||key==0x101{
  let last=if key==0x100{WHEEL_UP.load(Ordering::SeqCst)}else{WHEEL_DOWN.load(Ordering::SeqCst)};
  return wphw_input_now_ms()-last<120;
 }
 if let Some(query)=EVENTS.get(){let mut down=false;if query(key,&mut down){return down;}}
 wphw_input_key_down(key)
}
unsafe fn any_pressed(action:usize)->bool{
 let(mut codes,mut count)=(std::ptr::null(),0);wphw_input_codes(action,&mut codes,&mut count);
 if count>0{for &key in std::slice::from_raw_parts(codes,count){if pressed(key){return true;}}}false
}
// Source game action count and Grip index from keymap.h (20 actions,Grip18).
// Screenshot19 belongs to platform video note_press,not this game action poll.
pub unsafe fn sample()->(u32,bool){
 let mut buttons=0;for action in 0..20{let mask=wphw_input_action_buttons(action);if mask!=0&&any_pressed(action){buttons|=mask;}}
 (buttons,any_pressed(18))
}
