// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole review/input-sdl-port/real_wiimote_rust.cpp policy port.
// Native SDK retains only std::function/vector/string construction;no private layouts used.
use std::{ffi::{c_char,c_void},sync::atomic::{AtomicU64,Ordering}};
#[repr(C)]pub struct Sample{pub buttons:u32,pub pointer_valid:bool,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:bool,pub accel:[f32;3]}
type Sender=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C"{fn wprt_real_start();fn wprt_real_present(slot:u32)->bool;fn wprt_real_take_wake(slot:u32)->bool;fn wprt_real_reset(slot:u32);fn wprt_real_output_report(slot:u32,data:*const u8,size:u32);fn wprt_real_update(slot:u32,send:Sender,context:*mut c_void);
fn wphw_real_send_sdk(context:*mut c_void,data:*const u8,size:usize);
fn wphw_real_path_sdk(data:*const u8,size:usize,print:unsafe extern "C" fn(*const c_char));
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;
#[link_name="_ZNSt6chrono3_V212steady_clock3nowEv"]fn clock_now()->i64;
#[link_name="_ZNSt11this_thread11__sleep_forENSt6chrono8durationIxSt5ratioILx1ELx1EEEENS1_IxS2_ILx1ELx1000000000EEEE"]fn clock_sleep(seconds:i64,nanos:i64);
#[link_name="_ZN2wp5input6sampleEj"]fn input_sample(slot:u32)->Sample;
#[link_name="_ZN2wp7wiimote14camera_objectsEhRKNS_5input6SampleEPh"]fn camera_objects(mode:u8,sample:*const Sample,data:*mut u8);
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of wp::real_wiimote public forwards;names confirmed source object nm.
#[export_name="_ZN2wp12real_wiimote5startEv"]pub unsafe extern "C" fn start(){wprt_real_start();}
#[export_name="_ZN2wp12real_wiimote7presentEj"]pub unsafe extern "C" fn present(slot:u32)->bool{wprt_real_present(slot)}
#[export_name="_ZN2wp12real_wiimote9take_wakeEj"]pub unsafe extern "C" fn take_wake(slot:u32)->bool{wprt_real_take_wake(slot)}
#[export_name="_ZN2wp12real_wiimote5resetEj"]pub unsafe extern "C" fn reset(slot:u32){wprt_real_reset(slot);}
#[export_name="_ZN2wp12real_wiimote13output_reportEjPKhj"]pub unsafe extern "C" fn output_report(slot:u32,data:*const u8,size:u32){wprt_real_output_report(slot,data,size);}
#[export_name="_ZN2wp12real_wiimote6updateEjRKSt8functionIFvRKSt6vectorIhSaIhEEEE"]pub unsafe extern "C" fn update(slot:u32,sender:*mut c_void){wprt_real_update(slot,wphw_real_send_sdk,sender);}
// port of settings flag and source local static LiveFlag;constructor only keys/zero cache.
#[no_mangle]pub unsafe extern "C" fn wphw_real_enabled()->bool{wprt_settings_flag(c"input.real_wiimotes".as_ptr(),c"WP_REAL_WIIMOTES".as_ptr())}
static MOUSE_CACHE:AtomicU64=AtomicU64::new(0);
#[no_mangle]pub unsafe extern "C" fn wphw_real_mouse()->bool{let revision=wprt_settings_revision()as u64;let mut cached=MOUSE_CACHE.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(c"input.real_wiimote_mouse".as_ptr(),c"WP_REAL_WIIMOTE_MOUSE".as_ptr()){1}else{0};MOUSE_CACHE.store(cached,Ordering::Release);}cached&1!=0}
// port of steady_clock methods;source object after_scan adds 0x77359400 ticks,sleep passes0/1000000 to same SDK.
#[no_mangle]pub unsafe extern "C" fn wphw_real_now()->i64{clock_now()}
#[no_mangle]pub extern "C" fn wphw_real_after_scan(now:i64)->i64{now.wrapping_add(2_000_000_000)}
#[no_mangle]pub unsafe extern "C" fn wphw_real_sleep(){clock_sleep(0,1_000_000);}
unsafe extern "C" fn print_path(path:*const c_char){__mingw_fprintf(__acrt_iob_func(2),c"real wiimote at %s is paired but not connected\n".as_ptr(),path);}
// port of log switch;native string marshalling preserves counted bytes/embedded NUL and SDK exception behavior.
#[no_mangle]pub unsafe extern "C" fn wphw_real_log(kind:u32,slot:u32,product:u16,data:*const u8,size:usize){match kind{0=>{__mingw_fprintf(__acrt_iob_func(2),c"real wiimote %u disconnected\n".as_ptr(),slot.wrapping_add(1));},1=>wphw_real_path_sdk(data,size,print_path),2=>{__mingw_fprintf(__acrt_iob_func(2),c"real wiimote %u connected (%04x)\n".as_ptr(),slot.wrapping_add(1),product as i32);},_=>{}}}
// port of camera callback: source input sample(0) precedes actual Wii Remote camera implementation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_camera_objects(mode:u8,objects:*mut u8){let sample=input_sample(0);camera_objects(mode,&sample,objects);}
