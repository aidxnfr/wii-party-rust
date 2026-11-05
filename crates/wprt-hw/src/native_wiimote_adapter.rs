// Copyright 2010 Dolphin Emulator Project
// Copyright 2019 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete owned cpp/patched/wiimote_rust.cpp adapter policy port.
// Public input::Sample/SpeakerDecoder fields only;native SDK retains std::function/vector marshalling.
use std::{ffi::{c_char,c_void},sync::atomic::{AtomicU64,Ordering}};
#[repr(C)]pub struct Sample{pub buttons:u32,pub pointer_valid:bool,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:bool,pub accel:[f32;3]}
#[repr(C)]pub struct WiiSample{pub buttons:u32,pub pointer_valid:u32,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:u32,pub accel:[f32;3]}
#[repr(C)]pub struct SpeakerDecoder{pub predictor:i32,pub step:i32}
type Sender=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C"{fn wprt_wiimote_reset(index:u32);fn wprt_wiimote_output_report(index:u32,data:*const u8,size:u32,send:Sender,context:*mut c_void);fn wprt_wiimote_update(index:u32,send:Sender,context:*mut c_void);fn wprt_wiimote_camera_objects(mode:u8,sample:*const WiiSample,data:*mut u8);fn wprt_wiimote_decode(predictor:*mut i32,step:*mut i32,pcm:bool,data:*const u8,size:u32,out:*mut i16)->u32;fn wprt_wiimote_speaker_rate(pcm:bool,divisor:u16)->u32;
fn wphw_wiimote_send_sdk(context:*mut c_void,data:*const u8,size:usize);
#[link_name="_ZN2wp10comparison3nowEv"]fn comparison_now()->i64;
#[link_name="_ZN2wp5input6sampleEj"]fn input_sample(index:u32)->Sample;
#[link_name="_ZN2wp7gamepad11set_outputsEjbh"]fn gamepad_outputs(index:u32,rumble:bool,leds:u8);
#[link_name="_ZN2wp5audio12push_speakerEjPKsyjf"]fn push_speaker(index:u32,samples:*const i16,count:usize,rate:u32,volume:f32);
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;
#[link_name="sin"]fn native_sin(x:f64)->f64;#[link_name="cos"]fn native_cos(x:f64)->f64;#[link_name="lround"]fn native_lround(x:f64)->i32;#[link_name="lroundf"]fn native_lroundf(x:f32)->i32;
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of source transport;all named fields assigned,including raw floating-point bit patterns.
fn transport(sample:&Sample)->WiiSample{let mut raw=WiiSample{buttons:0,pointer_valid:0,pointer_x:0.0,pointer_y:0.0,motion_valid:0,accel:[0.0;3]};raw.buttons=sample.buttons;raw.pointer_valid=if sample.pointer_valid{1}else{0};raw.pointer_x=sample.pointer_x;raw.pointer_y=sample.pointer_y;raw.motion_valid=if sample.motion_valid{1}else{0};for i in 0..3{raw.accel[i]=sample.accel[i];}raw}
// port of namespace wp::wiimote methods;export names from source archive nm.
#[export_name="_ZN2wp7wiimote5resetEj"]pub unsafe extern "C" fn reset(index:u32){wprt_wiimote_reset(index);}
#[export_name="_ZN2wp7wiimote13output_reportEjPKhjRKSt8functionIFvRKSt6vectorIhSaIhEEEE"]pub unsafe extern "C" fn output_report(index:u32,data:*const u8,size:u32,send:*mut c_void){wprt_wiimote_output_report(index,data,size,wphw_wiimote_send_sdk,send);}
#[export_name="_ZN2wp7wiimote6updateEjRKSt8functionIFvRKSt6vectorIhSaIhEEEE"]pub unsafe extern "C" fn update(index:u32,send:*mut c_void){wprt_wiimote_update(index,wphw_wiimote_send_sdk,send);}
#[export_name="_ZN2wp7wiimote14camera_objectsEhRKNS_5input6SampleEPh"]pub unsafe extern "C" fn camera_objects(mode:u8,sample:*const Sample,data:*mut u8){let raw=transport(&*sample);wprt_wiimote_camera_objects(mode,&raw,data);}
#[export_name="_ZN2wp7wiimote14SpeakerDecoder6decodeEbPKhjPs"]pub unsafe extern "C" fn decode(this:*mut SpeakerDecoder,pcm:bool,data:*const u8,size:u32,out:*mut i16)->u32{wprt_wiimote_decode(std::ptr::addr_of_mut!((*this).predictor),std::ptr::addr_of_mut!((*this).step),pcm,data,size,out)}
#[export_name="_ZN2wp7wiimote12speaker_rateEbt"]pub unsafe extern "C" fn speaker_rate(pcm:bool,divisor:u16)->u32{wprt_wiimote_speaker_rate(pcm,divisor)}
// port of comparison-clock forward and exact source microseconds(5000) interval.
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_now()->i64{comparison_now()}
#[no_mangle]pub extern "C" fn wphw_wiimote_after_interval(now:i64)->i64{now.wrapping_add(5_000_000)}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_sample(index:u32,out:*mut WiiSample){*out=transport(&input_sample(index));}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_outputs(index:u32,rumble:bool,leds:u8){gamepad_outputs(index,rumble,leds);}
// port of source std math overloads;nm proves SAME MinGW CRT sin/cos/lround/lroundf symbols,Windowslong32.
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_sin(x:f64)->f64{native_sin(x)}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_cos(x:f64)->f64{native_cos(x)}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_lround(x:f64)->i32{native_lround(x)}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_lroundf(x:f32)->i32{native_lroundf(x)}
// port of source DLL-load-time getenv,not a runtime setting.
static mut WII_LOG:bool=false;unsafe extern "C" fn initialize(){WII_LOG=!libc::getenv(c"WP_LOG_WIIMOTE".as_ptr()).is_null();}
#[used]#[link_section=".CRT$XCU"]static INITIALIZE:unsafe extern "C" fn()=initialize;
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_log_enabled()->bool{WII_LOG}
// port of original format/byte loop bounds and CRT stream;u8 C promotion remains int.
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_log_output(index:u32,id:u8,data:*const u8,size:u32){let err=__acrt_iob_func(2);__mingw_fprintf(err,c"WIIMOTE %u output %02x size %u:".as_ptr(),index,id as i32,size);let mut i=1;while i<size&&i<12{__mingw_fprintf(err,c" %02x".as_ptr(),*data.add(i as usize)as i32);i+=1;}__mingw_fprintf(err,c"\n".as_ptr());}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_log_unknown(index:u32,id:u8){__mingw_fprintf(__acrt_iob_func(2),c"WIIMOTE %u unknown output report %02x\n".as_ptr(),index,id as i32);}
// port of source LiveFlag cache;Rust owns its atomic,never reinterprets a C++ atomic.
static SPEAKER_CACHE:AtomicU64=AtomicU64::new(0);
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_speaker_enabled()->bool{let revision=wprt_settings_revision()as u64;let mut cached=SPEAKER_CACHE.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(c"audio.wiimote_speaker".as_ptr(),std::ptr::null()){1}else{0};SPEAKER_CACHE.store(cached,Ordering::Release);}cached&1!=0}
#[no_mangle]pub unsafe extern "C" fn wphw_wiimote_push_speaker(index:u32,samples:*const i16,count:u32,rate:u32,volume:f32){push_speaker(index,samples,count as usize,rate,volume);}
