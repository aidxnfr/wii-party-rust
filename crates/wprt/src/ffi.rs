// SPDX-License-Identifier: GPL-3.0-or-later
// Leaf hardware/runtime service ABI from EXACT_CODEX_MAP section2.
use std::ffi::{c_char,CString};
use crate::cpu::Cpu;
#[cfg(feature="comparison")]
extern "C" {
    pub fn wphw_comparison_enabled()->bool;
    pub fn wphw_comparison_advance();
    pub fn wphw_comparison_ticks()->u64;
    pub fn wphw_comparison_frame(cpu:*const Cpu);
}
use std::ffi::{CStr,c_void};
extern "C" {
    pub fn wphw_bind_memory(memory:*mut u8);
    pub fn wphw_bind_game(description:*const c_void);
    pub fn wphw_poll_counter_ptr()->*mut u32;
    // Same object returned by native_runtime_bridge::wphw_poll_counter_ptr.
    // Opt-in build only; no cached pointer, shadow counter, or changed poll interval.
    #[cfg(feature="host-poll-fastpath")]
    #[link_name="_ZN2wp14g_poll_counterE"]
    pub static mut HOST_POLL_COUNTER: u32;
    pub fn wphw_audio_update();pub fn wphw_dsp_update();pub fn wphw_dsp_pending()->u32;
    pub fn wphw_ipc_update();pub fn wphw_ipc_pending()->bool;
    pub fn wphw_gx_process();pub fn wphw_gx_take_finish()->bool;
    pub fn wphw_exi_pending()->u32;pub fn wphw_si_pending()->bool;pub fn wphw_si_poll();
    pub fn wphw_video_present();
    fn wphw_mmio_read16(a:u32)->u16;fn wphw_mmio_read32(a:u32)->u32;
    fn wphw_mmio_write8(a:u32,v:u8);fn wphw_mmio_write16(a:u32,v:u16);
    fn wphw_mmio_write32(a:u32,v:u32);fn wphw_mmio_write64(a:u32,v:u64);
}
// ABI adapter installing pristine MMIO leaves; call after guest memory allocation, before hardware startup.
pub fn bind_memory(){unsafe{wphw_bind_memory(crate::memory::base_ptr());}crate::memory::set_mmio(crate::memory::Mmio{
    read16:|a|unsafe{wphw_mmio_read16(a)},read32:|a|unsafe{wphw_mmio_read32(a)},
    write8:|a,v|unsafe{wphw_mmio_write8(a,v)},write16:|a,v|unsafe{wphw_mmio_write16(a,v)},
    write32:|a,v|unsafe{wphw_mmio_write32(a,v)},write64:|a,v|unsafe{wphw_mmio_write64(a,v)},
});}
// C++ hardware service forwards to pristine runtime.cpp time_base, now Rust.
#[no_mangle] pub extern "C" fn wprt_time_base()->u64{crate::dispatch::time_base()}
// C++ GX statistics forward to pristine interrupts.cpp idle_share, now Rust.
#[no_mangle] pub extern "C" fn wprt_idle_share()->f64{crate::interrupts::idle_share()}
// C++ hardware module query forwards to the Rust core without guest control flow.
#[no_mangle] pub unsafe extern "C" fn wprt_module_loaded(name:*const c_char)->bool{crate::modules::module_loaded(CStr::from_ptr(name).to_str().unwrap())}
// C++ vector adapter; preserve native list order and bytes, without shared container layouts.
#[no_mangle] pub unsafe extern "C" fn wprt_loaded_module_names(sink:unsafe extern "C" fn(*mut c_void,*const u8,usize),context:*mut c_void){for name in crate::modules::loaded_module_names(){sink(context,name.as_ptr(),name.len());}}
// port of engine/src/core/big_stack.cpp _WIN32 run/run_with_stack; only hardware callbacks cross this ABI.
#[no_mangle] pub unsafe extern "C" fn wprt_run_with_stack(size:usize,callback:unsafe extern "C" fn(*mut c_void),parameter:*mut c_void){
    use windows_sys::Win32::{System::Threading::{CreateThread,WaitForSingleObject,STACK_SIZE_PARAM_IS_A_RESERVATION,INFINITE},Foundation::CloseHandle};
    struct Work{callback:unsafe extern "C" fn(*mut c_void),parameter:*mut c_void}
    unsafe extern "system" fn run(parameter:*mut c_void)->u32{let work=&*parameter.cast::<Work>();(work.callback)(work.parameter);0}
    let work=Work{callback,parameter};let thread=CreateThread(std::ptr::null(),size,Some(run),(&work as *const Work).cast(),STACK_SIZE_PARAM_IS_A_RESERVATION,std::ptr::null_mut());WaitForSingleObject(thread,INFINITE);CloseHandle(thread);
}
#[cfg(not(feature="rust-format"))]
extern "C" {fn wphw_format_guest(cpu:*const Cpu,format:u32,first:u32,sink:unsafe extern "C" fn(*mut std::ffi::c_void,*const u8,usize),context:*mut std::ffi::c_void);}
// ABI adapter for pristine format_guest; callback only copies bytes and never calls guest code.
pub fn format_guest(cpu:&Cpu,format:u32,first:u32)->Vec<u8>{
    #[cfg(feature="rust-format")] {crate::native_format::format_guest(cpu,format,first)}
    #[cfg(not(feature="rust-format"))] {
    unsafe extern "C" fn sink(context:*mut std::ffi::c_void,data:*const u8,len:usize){(*context.cast::<Vec<u8>>()).extend_from_slice(std::slice::from_raw_parts(data,len));}
    let mut bytes=Vec::new();unsafe{wphw_format_guest(cpu,format,first,sink,(&mut bytes as *mut Vec<u8>).cast());}bytes
    }
}

extern "C" { fn wphw_log_enabled()->bool; fn wphw_log_line(category:*const c_char,message:*const c_char); }
// port of engine/src/core/modules.cpp log::watch_modules call to log::enabled.
pub fn log_enabled()->bool { unsafe { wphw_log_enabled() } }
// port of engine/src/core/modules.cpp log::watch_modules call to log::write(category,"%s",message).
pub fn log(category:&str,message:&str) {
    let category=CString::new(category).unwrap(); let message=CString::new(message).unwrap();
    unsafe { wphw_log_line(category.as_ptr(),message.as_ptr()); }
}
