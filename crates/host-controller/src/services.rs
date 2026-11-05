// SPDX-License-Identifier: GPL-3.0-or-later
// Host boundary; source elapsed conversions and log text, supported Rust clock/sleep.
use std::{ffi::{c_char,CStr},sync::{OnceLock,atomic::{AtomicU8,Ordering}},time::{Instant,Duration}};
static SETTINGS:AtomicU8=AtomicU8::new(0);
pub fn configure(enabled:bool,pairs:bool,wiimotes:bool){SETTINGS.store(enabled as u8|((pairs as u8)<<1)|((wiimotes as u8)<<2),Ordering::Release);}
#[no_mangle]pub extern "C" fn wphw_gamepad_enabled()->bool{SETTINGS.load(Ordering::Acquire)&1!=0}
#[no_mangle]pub extern "C" fn wphw_gamepad_setting(which:u32)->bool{SETTINGS.load(Ordering::Acquire)&(if which==0{2}else{4})!=0}
#[no_mangle]pub extern "C" fn wphw_gamepad_now()->i64{static ORIGIN:OnceLock<Instant>=OnceLock::new();ORIGIN.get_or_init(Instant::now).elapsed().as_nanos()as i64}
#[no_mangle]pub extern "C" fn wphw_gamepad_after_ms(now:i64,ms:i64)->i64{now.wrapping_add(ms.wrapping_mul(1_000_000))}
#[no_mangle]pub extern "C" fn wphw_gamepad_elapsed_ms(now:i64,origin:i64)->i64{now.wrapping_sub(origin)/1_000_000}
#[no_mangle]pub extern "C" fn wphw_gamepad_elapsed_float(now:i64,previous:i64)->f32{now.wrapping_sub(previous)as f32/1_000_000_000.0f32}
#[no_mangle]pub extern "C" fn wphw_gamepad_sleep_ms(ms:u64){if(ms as i64)>0{std::thread::sleep(Duration::from_millis(ms));}}
unsafe fn name(p:*const c_char)->String{if p.is_null(){"?".into()}else{CStr::from_ptr(p).to_string_lossy().into_owned()}}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_log_open(n:*const c_char,slot:u32,gyro:bool,accel:bool,nintendo:bool){eprintln!("gamepad {} connected as Wii Remote {}, gyroscope {}, accelerometer {}, {} layout",name(n),slot,if gyro{"yes"}else{"no"},if accel{"yes"}else{"no"},if nintendo{"Nintendo"}else{"Xbox"});}
#[no_mangle]pub extern "C" fn wphw_gamepad_log_close(slot:u32){eprintln!("gamepad of Wii Remote {} disconnected",slot);}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_log_unavailable(){eprintln!("SDL gamepad support unavailable: {}",name(crate::sdk::error()));}
#[no_mangle]pub extern "C" fn wphw_gamepad_virtual_log_rumble(number:u32,low:u16,high:u16){eprintln!("virtual gamepad {} rumble {} {}",number,low,high);}
#[no_mangle]pub extern "C" fn wphw_gamepad_virtual_log_player(number:u32,index:i32){eprintln!("virtual gamepad {} player index {}",number,index);}
