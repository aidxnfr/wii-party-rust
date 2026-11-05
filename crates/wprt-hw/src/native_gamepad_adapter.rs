// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete hybrid/review/input-sdl-port/gamepad_rust.cpp adapter port.
// Public plain State source:pristine include/wp/gamepad.h;private class layouts never used.
use std::{ffi::c_char,sync::atomic::{AtomicU64,Ordering}};
#[repr(C)]pub struct State{pub connected:bool,pub buttons:u32,pub pointer_valid:bool,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:bool,pub accel:[f32;3]}
#[repr(C)]struct PadState{connected:u32,buttons:u32,pointer_valid:u32,pointer_x:f32,pointer_y:f32,motion_valid:u32,accel:[f32;3]}
extern "C"{fn wprt_gamepad_connected(slot:u32)->bool;fn wprt_gamepad_set_outputs(slot:u32,rumble:bool,leds:u8);fn wprt_gamepad_menu_buttons()->u32;fn wprt_gamepad_poll(slot:u32,sideways:bool,state:*mut PadState);fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn SDL_GetError()->*const c_char;
// Actual GCC adapter disassembly confirms now returns raw duration ticks in RAX;__sleep_for takes seconds RCX/nanoseconds RDX.
#[link_name="_ZNSt6chrono3_V212steady_clock3nowEv"]fn clock_now()->i64;
#[link_name="_ZNSt11this_thread11__sleep_forENSt6chrono8durationIxSt5ratioILx1ELx1EEEENS1_IxS2_ILx1ELx1000000000EEEE"]fn clock_sleep(seconds:i64,nanos:i64);}
// port of namespace wp::gamepad four public methods.
#[export_name="_ZN2wp7gamepad9connectedEj"]pub unsafe extern "C" fn connected(slot:u32)->bool{#[cfg(feature="rust-host-gamepad")]if crate::native_host_gamepad::bound(){return crate::native_host_gamepad::connected(slot);}wprt_gamepad_connected(slot)}
#[export_name="_ZN2wp7gamepad11set_outputsEjbh"]pub unsafe extern "C" fn set_outputs(slot:u32,rumble:bool,leds:u8){#[cfg(feature="rust-host-gamepad")]if crate::native_host_gamepad::bound(){crate::native_host_gamepad::set_outputs(slot,rumble,leds);return;}wprt_gamepad_set_outputs(slot,rumble,leds);}
#[export_name="_ZN2wp7gamepad12menu_buttonsEv"]pub unsafe extern "C" fn menu_buttons()->u32{#[cfg(feature="rust-host-gamepad")]if crate::native_host_gamepad::bound(){return crate::native_host_gamepad::menu_buttons();}wprt_gamepad_menu_buttons()}
#[export_name="_ZN2wp7gamepad4pollEjb"]pub unsafe extern "C" fn poll(slot:u32,sideways:bool)->State{#[cfg(feature="rust-host-gamepad")]if crate::native_host_gamepad::bound(){return crate::native_host_gamepad::poll(slot,sideways);}let mut raw=PadState{connected:0,buttons:0,pointer_valid:0,pointer_x:0.0,pointer_y:0.0,motion_valid:0,accel:[0.0;3]};wprt_gamepad_poll(slot,sideways,&mut raw);State{connected:raw.connected!=0,buttons:raw.buttons,pointer_valid:raw.pointer_valid!=0,pointer_x:raw.pointer_x,pointer_y:raw.pointer_y,motion_valid:raw.motion_valid!=0,accel:raw.accel}}
// port of source static LiveFlag("input.gamepads","WP_GAMEPAD") and settings.cpp LiveFlag::operator().
// Cache owned by Rust;no reinterpret_cast between Rust/C++ atomic representations.
static ENABLED_CACHE:AtomicU64=AtomicU64::new(0);
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_enabled()->bool{let current=wprt_settings_revision()as u64;let mut cached=ENABLED_CACHE.load(Ordering::Acquire);if cached>>32!=current{cached=(current<<32)|if wprt_settings_flag(c"input.gamepads".as_ptr(),c"WP_GAMEPAD".as_ptr()){1}else{0};ENABLED_CACHE.store(cached,Ordering::Release);}cached&1!=0}
// port of source formatted log callbacks;C argument promotions retained.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_log_open(name:*const c_char,slot:u32,gyro:bool,accel:bool,nintendo:bool){log_write(c"input".as_ptr(),c"gamepad %s connected as Wii Remote %u, gyroscope %s, accelerometer %s, %s layout".as_ptr(),if name.is_null(){c"?".as_ptr()}else{name},slot,if gyro{c"yes".as_ptr()}else{c"no".as_ptr()},if accel{c"yes".as_ptr()}else{c"no".as_ptr()},if nintendo{c"Nintendo".as_ptr()}else{c"Xbox".as_ptr()});}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_log_close(slot:u32){log_write(c"input".as_ptr(),c"gamepad of Wii Remote %u disconnected".as_ptr(),slot);}
// port of target steady_clock count/duration_cast;actual compiled code uses nanosecond ticks/signed integer truncation/f32 division.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_now()->i64{clock_now()}
#[no_mangle]pub extern "C" fn wphw_gamepad_after_ms(now:i64,ms:i64)->i64{now.wrapping_add(ms.wrapping_mul(1_000_000))}
#[no_mangle]pub extern "C" fn wphw_gamepad_elapsed_ms(now:i64,origin:i64)->i64{now.wrapping_sub(origin)/1_000_000}
#[no_mangle]pub extern "C" fn wphw_gamepad_elapsed_float(now:i64,previous:i64)->f32{now.wrapping_sub(previous)as f32/1_000_000_000.0f32}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_setting(which:u32)->bool{if which==0{wprt_settings_flag(c"input.joycon_pairs".as_ptr(),c"WP_JOYCON_PAIRS".as_ptr())}else{wprt_settings_flag(c"input.real_wiimotes".as_ptr(),c"WP_REAL_WIIMOTES".as_ptr())}}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_log_unavailable(){log_write(c"input".as_ptr(),c"SDL gamepad support unavailable: %s".as_ptr(),SDL_GetError());}
// port of std::this_thread::sleep_for(milliseconds(ms)):target guard/split confirmed actual GCC assembly.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_sleep_ms(ms:u64){let ms=ms as i64;if ms<=0{return;}let seconds=ms/1000;let nanos=(ms-seconds*1000)*1_000_000;clock_sleep(seconds,nanos);}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_virtual_log_rumble(number:u32,low:u16,high:u16){log_write(c"input".as_ptr(),c"virtual gamepad %u rumble %u %u".as_ptr(),number,low as i32,high as i32);}
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_virtual_log_player(number:u32,player:i32){log_write(c"input".as_ptr(),c"virtual gamepad %u player index %d".as_ptr(),number,player);}
