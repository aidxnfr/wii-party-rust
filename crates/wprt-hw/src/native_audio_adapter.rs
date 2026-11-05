// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned review/audio-win32-port/audio_rust.cpp policy port.
// SDK retains original DLL-load chrono value and std::thread construction/detach;no mixer/device changes.
use std::{ffi::c_char,sync::atomic::{AtomicU64,Ordering}};
#[repr(C)]pub struct Frame{pub left:f32,pub right:f32}
extern "C"{fn wprt_audio_update();fn wprt_audio_start_output();fn wprt_audio_push(p:*const i16,n:usize,r:u32);fn wprt_audio_push_speaker(channel:u32,p:*const i16,n:usize,rate:u32,volume:f32);
fn wprt_awd_dump_open(rate:u32)->usize;fn wprt_awd_mixer_dump(file:usize,frames:*const Frame,count:u32,total:*mut u32,rate:u32);fn wprt_awd_push_dump(p:*const i16,n:usize,rate:u32);
fn wphw_audio_initial_ticks_sdk()->i64;
#[link_name="_ZN2wp10comparison3nowEv"]fn comparison_now()->i64;
#[link_name="sin"]fn native_sin(x:f64)->f64;#[link_name="cos"]fn native_cos(x:f64)->f64;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of source scalar source_rate=32000;source non-atomic/unlocked reporting access deliberately retained.
static mut SOURCE_RATE:u32=32000;
// port of namespace wp::audio four primitive forwards;actual source object nm names retained.
#[export_name="_ZN2wp5audio6updateEv"]pub unsafe extern "C" fn update(){wprt_audio_update();}
#[export_name="_ZN2wp5audio12start_outputEv"]pub unsafe extern "C" fn start_output(){wprt_audio_start_output();}
#[export_name="_ZN2wp5audio4pushEPKsyj"]pub unsafe extern "C" fn push(p:*const i16,n:usize,rate:u32){wprt_audio_push(p,n,rate);}
#[export_name="_ZN2wp5audio12push_speakerEjPKsyjf"]pub unsafe extern "C" fn push_speaker(channel:u32,p:*const i16,n:usize,rate:u32,volume:f32){wprt_audio_push_speaker(channel,p,n,rate,volume);}
// port of source native math/clock forwards;original target CRT and shared comparison clock.
#[no_mangle]pub unsafe extern "C" fn wphw_audio_sin(x:f64)->f64{native_sin(x)}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_cos(x:f64)->f64{native_cos(x)}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_now()->i64{comparison_now()}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_initial_time()->i64{wphw_audio_initial_ticks_sdk()}
// port of nanosecond chrono duration<double> and report threshold;actual GCC object sub/cvtsi2sd/divsd/signedcompare.
#[no_mangle]pub extern "C" fn wphw_audio_elapsed(now:i64,last:i64)->f64{now.wrapping_sub(last)as f64/1_000_000_000.0}
#[no_mangle]pub extern "C" fn wphw_audio_report_due(now:i64,last:i64)->bool{now.wrapping_sub(last)>=1_000_000_000}
// port of target static_cast<uint32_t>(double):source actual cvttsd2si rax,then caller uses low32.
// Rust saturating as would change source NaN/negative/out-of-range results;keep exact target instruction.
#[no_mangle]pub unsafe extern "C" fn wphw_audio_counter(value:f64)->u32{let result:i64;std::arch::asm!("cvttsd2si {result}, {value}",result=out(reg)result,value=in(xmm_reg)value,options(nomem,nostack));result as u32}
static MUTE_CACHE:AtomicU64=AtomicU64::new(0);
// port of source static LiveFlag audio.mute/WP_MUTE;Rust owns cache,source revision/acquire/release preserved.
#[no_mangle]pub unsafe extern "C" fn wphw_audio_muted()->bool{let revision=wprt_settings_revision()as u64;let mut cached=MUTE_CACHE.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(c"audio.mute".as_ptr(),c"WP_MUTE".as_ptr()){1}else{0};MUTE_CACHE.store(cached,Ordering::Release);}cached&1!=0}
// port of source optional dump forwards;actual accepted Rust WASAPI/WAV backend unchanged.
#[no_mangle]pub unsafe extern "C" fn wphw_audio_mixer_dump_open(rate:u32)->usize{wprt_awd_dump_open(rate)}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_mixer_dump(file:usize,p:*const Frame,n:u32,frames:*mut u32,rate:u32){wprt_awd_mixer_dump(file,p,n,frames,rate);}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_push_dump(p:*const i16,n:usize,rate:u32){wprt_awd_push_dump(p,n,rate);}
#[no_mangle]pub unsafe extern "C" fn wphw_awd_start_error(){__mingw_fprintf(__acrt_iob_func(2),c"audio output could not start\n".as_ptr());}
// port of original log expression order/format;no normalization of zero source-rate division/NaN/Inf.
#[no_mangle]pub unsafe extern "C" fn wphw_audio_report(rate:f64,lowest:usize,highest:usize,missing:u32,trims:u32){let source=SOURCE_RATE as f64;log_write(c"output".as_ptr(),c"device %.0f Hz, queue %.1f-%.1f ms, %u frames missing, %u trims".as_ptr(),rate,lowest as f64*1000.0/source,highest as f64*1000.0/source,missing,trims);}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_source_rate()->u32{SOURCE_RATE}
#[no_mangle]pub unsafe extern "C" fn wphw_audio_set_source_rate(rate:u32){SOURCE_RATE=rate;}
