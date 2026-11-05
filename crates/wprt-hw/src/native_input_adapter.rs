// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete owned cpp/patched/input_rust.cpp adapter policy port.
// Native SDK retains only Bindings/vector/string ownership/access and original script-origin C++ guard.
use std::{ffi::{c_char,c_void},sync::atomic::{AtomicU64,Ordering}};
#[repr(C)]pub struct Sample{pub buttons:u32,pub pointer_valid:bool,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:bool,pub accel:[f32;3]}
#[repr(C)]pub struct PadState{pub connected:bool,pub buttons:u32,pub pointer_valid:bool,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:bool,pub accel:[f32;3]}
#[repr(C)]pub struct RawSample{pub buttons:u32,pub pointer_valid:u32,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:u32,pub accel:[f32;3]}
// PUBLIC game.h and keymap.h POD fields;no private class/container representations.
#[repr(C)]struct Description{title:*const c_char,short_name:*const c_char,data_directory:*const c_char,nand_directory:*const c_char,settings_file:*const c_char,textures_directory:*const c_char,notice_module:*const c_char,sideways_modules:*const *const c_char,sideways_module_count:usize,instruction_module:*const c_char,minigame_prefix:*const c_char}
#[repr(C)]struct ActionInfo{name:*const c_char,buttons:u32,defaults:*const c_char}
extern "C"{fn wprt_input_connected(channel:u32)->bool;fn wprt_input_sample(channel:u32,out:*mut RawSample);fn wprt_input_wakes_remote(channel:u32)->bool;fn wprt_input_note_wheel(delta:i32);fn wprt_input_set_blocked(blocked:bool);
fn wphw_input_invalid_at_sdk(list:*const c_void,index:usize)->*const c_char;fn wphw_input_origin_ticks_sdk()->i64;
#[link_name="_ZN2wp5input8bindingsEv"]fn bindings()->*const c_void;
#[link_name="_ZN2wp10comparison3nowEv"]fn comparison_now()->i64;
#[link_name="_ZN2wp2gx12frames_drawnEv"]fn frames_drawn()->u64;
#[link_name="_ZN2wp4game11descriptionEv"]fn description()->*const Description;
#[link_name="_ZN2wp13module_loadedEPKc"]fn module_loaded(name:*const c_char)->bool;
#[link_name="_ZN2wp5video8key_downEi"]fn key_down(key:i32)->bool;
#[link_name="_ZN2wp6keymap6actionEy"]fn key_action(index:usize)->*const ActionInfo;
#[link_name="_ZN2wp7gamepad4pollEjb"]fn pad_poll(channel:u32,sideways:bool)->PadState;
#[link_name="_ZN2wp7gamepad9connectedEj"]fn pad_connected(channel:u32)->bool;
#[link_name="_ZN2wp7host_io6sampleEj"]fn host_sample(channel:u32)->Sample;
#[link_name="_ZN2wp5video13window_handleEv"]fn window_handle()->*mut c_void;
#[link_name="_ZN2wp5video14window_focusedEv"]fn window_focused()->bool;
#[link_name="_ZN2wp5video15cursor_positionERlS1_"]fn cursor_position(x:*mut i32,y:*mut i32)->bool;
#[link_name="_ZN2wp5video15cursor_on_imageERfS1_"]fn cursor_on_image(x:*mut f32,y:*mut f32)->bool;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of namespace wp::input five value methods;source zeroes RawSample BEFORE provider fills it.
#[export_name="_ZN2wp5input9connectedEj"]pub unsafe extern "C" fn connected(channel:u32)->bool{wprt_input_connected(channel)}
#[export_name="_ZN2wp5input6sampleEj"]pub unsafe extern "C" fn sample(channel:u32)->Sample{let mut raw:RawSample=std::mem::zeroed();wprt_input_sample(channel,&mut raw);Sample{buttons:raw.buttons,pointer_valid:raw.pointer_valid!=0,pointer_x:raw.pointer_x,pointer_y:raw.pointer_y,motion_valid:raw.motion_valid!=0,accel:raw.accel}}
#[export_name="_ZN2wp5input12wakes_remoteEj"]pub unsafe extern "C" fn wakes_remote(channel:u32)->bool{wprt_input_wakes_remote(channel)}
#[export_name="_ZN2wp5input10note_wheelEi"]pub unsafe extern "C" fn note_wheel(delta:i32){wprt_input_note_wheel(delta);}
#[export_name="_ZN2wp5input11set_blockedEb"]pub unsafe extern "C" fn set_blocked(blocked:bool){wprt_input_set_blocked(blocked);}
// port of Bindings initialization warning loop;SDK supplies typed vector element C strings while alive.
#[no_mangle]pub unsafe extern "C" fn wprt_input_ignored_list(list:*const c_void,count:usize){for index in 0..count{let entry=wphw_input_invalid_at_sdk(list,index);let err=__acrt_iob_func(2);__mingw_fprintf(err,c"settings: ignored key name %s".as_ptr(),entry);libc::fputc(10,err);log_write(c"input".as_ptr(),c"ignored key name %s".as_ptr(),entry);}}
// port of clock helpers. Actual GCC object initializes/reads origin BEFORE reading now in elapsed().
#[no_mangle]pub unsafe extern "C" fn wphw_input_now_ms()->i64{comparison_now()/1_000_000}
#[no_mangle]pub unsafe extern "C" fn wphw_input_origin(){let _=wphw_input_origin_ticks_sdk();}
#[no_mangle]pub unsafe extern "C" fn wphw_input_elapsed()->i32{let origin=wphw_input_origin_ticks_sdk();(comparison_now().wrapping_sub(origin)/1_000_000)as i32}
#[no_mangle]pub unsafe extern "C" fn wphw_input_frames()->u64{frames_drawn()}
// port of the three source static LiveFlags;each Rust-owned cache retains primary acquire/release semantics.
static AUTO_GRIP:AtomicU64=AtomicU64::new(0);static WAKE_MOUSE:AtomicU64=AtomicU64::new(0);static SKIP_NOTICES:AtomicU64=AtomicU64::new(0);
#[no_mangle]pub unsafe extern "C" fn wphw_input_flag(which:u32)->bool{let (cache,key,var)=match which{0=>(&AUTO_GRIP,c"input.auto_grip".as_ptr(),c"WP_AUTO_ORIENTATION".as_ptr()),1=>(&WAKE_MOUSE,c"input.wake_on_mouse".as_ptr(),c"WP_RECONNECT_ON_POINTER".as_ptr()),_=>(&SKIP_NOTICES,c"system.skip_notices".as_ptr(),c"WP_SKIP_NOTICES".as_ptr())};let revision=wprt_settings_revision()as u64;let mut cached=cache.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(key,var){1}else{0};cache.store(cached,Ordering::Release);}cached&1!=0}
// port of public Description field reads and unchanged namespace service forwards.
#[no_mangle]pub unsafe extern "C" fn wphw_input_sideways_count()->usize{(*description()).sideways_module_count}
#[no_mangle]pub unsafe extern "C" fn wphw_input_sideways_loaded(index:usize)->bool{module_loaded(*(*description()).sideways_modules.add(index))}
#[no_mangle]pub unsafe extern "C" fn wphw_input_notice_module()->*const c_char{(*description()).notice_module}
#[no_mangle]pub unsafe extern "C" fn wphw_input_module_loaded(name:*const c_char)->bool{module_loaded(name)}
#[no_mangle]pub unsafe extern "C" fn wphw_input_key_down(key:i32)->bool{key_down(key)}
#[no_mangle]pub unsafe extern "C" fn wphw_input_bindings(){let _=bindings();}
#[no_mangle]pub unsafe extern "C" fn wphw_input_action_buttons(index:usize)->u32{(*key_action(index)).buttons}
// port of source transport<T> named field assignments for each PUBLIC source POD.
fn raw_sample(sample:&Sample)->RawSample{RawSample{buttons:sample.buttons,pointer_valid:if sample.pointer_valid{1}else{0},pointer_x:sample.pointer_x,pointer_y:sample.pointer_y,motion_valid:if sample.motion_valid{1}else{0},accel:sample.accel}}
#[no_mangle]pub unsafe extern "C" fn wphw_input_pad(channel:u32,sideways:bool,out:*mut RawSample){let p=pad_poll(channel,sideways);*out=RawSample{buttons:p.buttons,pointer_valid:if p.pointer_valid{1}else{0},pointer_x:p.pointer_x,pointer_y:p.pointer_y,motion_valid:if p.motion_valid{1}else{0},accel:p.accel};}
#[no_mangle]pub unsafe extern "C" fn wphw_input_pad_connected(channel:u32)->bool{pad_connected(channel)}
#[no_mangle]pub unsafe extern "C" fn wphw_input_host(channel:u32,out:*mut RawSample){*out=raw_sample(&host_sample(channel));}
#[no_mangle]pub unsafe extern "C" fn wphw_input_window()->*mut c_void{window_handle()}
#[no_mangle]pub unsafe extern "C" fn wphw_input_focused()->bool{window_focused()}
// port of original CRT logs,including bool promotion and source newline call.
#[no_mangle]pub unsafe extern "C" fn wphw_input_log_focus(window:*mut c_void,focused:bool){let err=__acrt_iob_func(2);__mingw_fprintf(err,c"input window=%p focused=%d".as_ptr(),window,if focused{1i32}else{0i32});libc::fputc(10,err);}
#[no_mangle]pub unsafe extern "C" fn wphw_input_log_buttons(buttons:u32){let err=__acrt_iob_func(2);__mingw_fprintf(err,c"input buttons=%04x".as_ptr(),buttons);libc::fputc(10,err);}
// port of source Windows-long cursor bridge:zero locals,always write both outputs even on false,source alias order.
#[no_mangle]pub unsafe extern "C" fn wphw_input_cursor(x:*mut i32,y:*mut i32)->bool{let(mut cx,mut cy)=(0,0);let ok=cursor_position(&mut cx,&mut cy);*x=cx;*y=cy;ok}
#[no_mangle]pub unsafe extern "C" fn wphw_input_image(x:*mut f32,y:*mut f32)->bool{cursor_on_image(x,y)}
