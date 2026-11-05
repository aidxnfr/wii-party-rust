// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted owned cpp/patched/gx_command_rust.cpp Windows host/forward policy port.
// Native SDK keeps original C++ local-static guard and std::function task lifetime;renderer unchanged.
use std::{ffi::{c_char,c_void},mem::MaybeUninit,sync::atomic::{AtomicU64,Ordering}};
extern "C"{fn wphw_gx_ensure_sdk();fn wpgx_gx_push(value:u64,bytes:u32);fn wpgx_gx_process();fn wpgx_gx_take_finish_interrupt()->bool;fn wpgx_gx_write_pixel_engine_control(value:u16);fn wpgx_gx_request_capture();fn wpgx_gx_frames_drawn()->u64;fn wpgx_gx_bp_registers()->*const u32;fn wpgx_gx_konst_registers()->*const u32;fn wpgx_gx_xf_registers()->*const u32;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
#[link_name="_ZN2wp5video15save_next_frameEPKc"]fn save_next_frame(path:*const c_char);
#[link_name="_ZN2wp5video17update_statisticsEd"]fn update_statistics(fps:f64);
#[link_name="_ZN2wp10idle_shareEv"]fn idle_share()->f64;
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;}
// Public Windows SDK fields copied from installed MinGW psapi.h:122-134,not a C++ class.
#[repr(C)]struct MemoryCounters{cb:u32,PageFaultCount:u32,PeakWorkingSetSize:usize,WorkingSetSize:usize,QuotaPeakPagedPoolUsage:usize,QuotaPagedPoolUsage:usize,QuotaPeakNonPagedPoolUsage:usize,QuotaNonPagedPoolUsage:usize,PagefileUsage:usize,PeakPagefileUsage:usize,PrivateUsage:usize}
#[repr(C)]#[derive(Clone,Copy)]struct FileTime{dwLowDateTime:u32,dwHighDateTime:u32}
extern "system"{fn GetCurrentProcess()->*mut c_void;fn GetCurrentThread()->*mut c_void;fn DuplicateHandle(source_process:*mut c_void,source_handle:*mut c_void,target_process:*mut c_void,target_handle:*mut *mut c_void,access:u32,inherit:i32,options:u32)->i32;fn GetThreadTimes(thread:*mut c_void,created:*mut FileTime,exited:*mut FileTime,kernel:*mut FileTime,user:*mut FileTime)->i32;fn K32GetProcessMemoryInfo(process:*mut c_void,counters:*mut MemoryCounters,size:u32)->i32;}
// port of host_log/copy_filter_enabled/save_next_frame/update_statistics/idle_share callbacks.
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_log(category:*const c_char,text:*const c_char){log_write(category,c"%s".as_ptr(),text);}
static FILTER_CACHE:AtomicU64=AtomicU64::new(0);
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_copy_filter_enabled()->bool{let revision=wprt_settings_revision()as u64;let mut cached=FILTER_CACHE.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(c"video.copy_filter".as_ptr(),c"WP_COPY_FILTER".as_ptr()){1}else{0};FILTER_CACHE.store(cached,Ordering::Release);}cached&1!=0}
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_save_next_frame(path:*const c_char){save_next_frame(path);}
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_update_statistics(fps:f64){update_statistics(fps);}
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_idle_share()->f64{idle_share()}
// port of Windows host_private_memory_mb:source zeroes all fields,sets cb,and returns0 on query failure.
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_private_memory_mb()->usize{let mut counters:MemoryCounters=std::mem::zeroed();counters.cb=std::mem::size_of::<MemoryCounters>()as u32;if K32GetProcessMemoryInfo(GetCurrentProcess(),&mut counters,std::mem::size_of::<MemoryCounters>()as u32)!=0{return counters.PrivateUsage>>20;}0}
// port of Windows host_current_thread_clock:source ignores DuplicateHandle's result and retains returned handle.
// Winnt.h THREAD_QUERY_LIMITED_INFORMATION=0x0800;source has no CloseHandle here (quirk kept).
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_current_thread_clock()->u64{let mut thread=std::ptr::null_mut();let target=GetCurrentProcess();let source_thread=GetCurrentThread();let source=GetCurrentProcess();DuplicateHandle(source,source_thread,target,&mut thread,0x0800,0,0);thread as u64}
// port of Windows host_thread_seconds:guard null,query four FILETIMEs,wrap unsigned sum,then source f64 scaling.
#[no_mangle]pub unsafe extern "C" fn wphw_gx_host_thread_seconds(clock:u64)->f64{let thread=clock as *mut c_void;if thread.is_null(){return 0.0;}let mut times=[MaybeUninit::<FileTime>::uninit();4];let p=times.as_mut_ptr().cast::<FileTime>();if GetThreadTimes(thread,p,p.add(1),p.add(2),p.add(3))==0{return 0.0;}let kernel=times[2].assume_init();let user=times[3].assume_init();let ticks=|time:FileTime|((time.dwHighDateTime as u64)<<32)|time.dwLowDateTime as u64;ticks(kernel).wrapping_add(ticks(user))as f64*1e-7}
// port of public scalar methods. Register getters intentionally do NOT ensure(),matching source.
#[export_name="_ZN2wp2gx4pushEyj"]pub unsafe extern "C" fn push(value:u64,bytes:u32){wphw_gx_ensure_sdk();wpgx_gx_push(value,bytes);}
#[export_name="_ZN2wp2gx7processEv"]pub unsafe extern "C" fn process(){wphw_gx_ensure_sdk();wpgx_gx_process();}
#[export_name="_ZN2wp2gx21take_finish_interruptEv"]pub unsafe extern "C" fn finish()->bool{wphw_gx_ensure_sdk();wpgx_gx_take_finish_interrupt()}
#[export_name="_ZN2wp2gx26write_pixel_engine_controlEt"]pub unsafe extern "C" fn write_control(value:u16){wphw_gx_ensure_sdk();wpgx_gx_write_pixel_engine_control(value);}
#[export_name="_ZN2wp2gx15request_captureEv"]pub unsafe extern "C" fn capture(){wphw_gx_ensure_sdk();wpgx_gx_request_capture();}
#[export_name="_ZN2wp2gx12frames_drawnEv"]pub unsafe extern "C" fn frames()->u64{wphw_gx_ensure_sdk();wpgx_gx_frames_drawn()}
#[export_name="_ZN2wp2gx12bp_registersEv"]pub unsafe extern "C" fn bp()->*const u32{wpgx_gx_bp_registers()}
#[export_name="_ZN2wp2gx15konst_registersEv"]pub unsafe extern "C" fn konst()->*const u32{wpgx_gx_konst_registers()}
#[export_name="_ZN2wp2gx12xf_registersEv"]pub unsafe extern "C" fn xf()->*const u32{wpgx_gx_xf_registers()}
