// AI-assisted (Claude, G60) gx_host_sdk.cpp -> Rust: whole member, no C++ residue.
// wphw_gx_ensure_sdk keeps GCC's guarded local-static semantics via __cxa_guard_* (abort on unwind, retry next call);
// run_frame_task moves its by-value std::function into a heap std::function (operator new 32) owned by the Rust GX queue.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("gx_host_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
#[repr(C)] pub struct GxHost { f: [*const (); 8] }
extern "C-unwind" {
    fn wpgx_gx_install(host: GxHost);
    fn wpgx_gx_run_frame_task(run: unsafe extern "C-unwind" fn(*mut u8), drop: unsafe extern "C-unwind" fn(*mut u8), context: *mut u8);
    #[link_name = "_ZN2wp2gx6render16install_rendererEv"] fn install_renderer();
    fn wphw_gx_host_log(c: *const u8, t: *const u8); fn wphw_gx_host_copy_filter_enabled() -> bool; fn wphw_gx_host_save_next_frame(p: *const u8);
    fn wphw_gx_host_update_statistics(fps: f64); fn wphw_gx_host_idle_share() -> f64; fn wphw_gx_host_private_memory_mb() -> usize;
    fn wphw_gx_host_current_thread_clock() -> u64; fn wphw_gx_host_thread_seconds(c: u64) -> f64;
}
static mut GUARD: u64 = 0;
static mut INSTALLED: bool = false;
#[export_name = "wphw_gx_ensure_sdk"]
pub unsafe extern "C-unwind" fn wphw_gx_ensure_sdk() {
    guarded_static(std::ptr::addr_of_mut!(GUARD), || {
        wpgx_gx_install(GxHost { f: [wphw_gx_host_log as *const (), wphw_gx_host_copy_filter_enabled as *const (), wphw_gx_host_save_next_frame as *const (),
            wphw_gx_host_update_statistics as *const (), wphw_gx_host_idle_share as *const (), wphw_gx_host_private_memory_mb as *const (),
            wphw_gx_host_current_thread_clock as *const (), wphw_gx_host_thread_seconds as *const ()] });
        install_renderer();
        std::ptr::addr_of_mut!(INSTALLED).write(true);
    });
}
unsafe extern "C-unwind" fn run_task(c: *mut u8) {
    let invoke: unsafe extern "C-unwind" fn(*const u8) = std::mem::transmute(function_check(c));
    invoke(c);
}
unsafe extern "C-unwind" fn drop_task(c: *mut u8) {
    if c.is_null() { return; }
    let manager = c.add(16).cast::<usize>().read();
    if manager != 0 { let m: unsafe extern "C" fn(*mut u8, *const u8, i32) -> bool = std::mem::transmute(manager); m(c, c, 3); }
    op_delete(c, 32);
}
/// void wp::gx::run_frame_task(std::function<void()>)  (by-value parameter: caller-owned temporary passed by address)
#[export_name = "_ZN2wp2gx14run_frame_taskESt8functionIFvvEE"]
pub unsafe extern "C-unwind" fn run_frame_task(task: *mut usize) {
    wphw_gx_ensure_sdk();
    let held = op_new(32).cast::<usize>();
    held.write(0); held.add(1).write(0); held.add(2).write(0); held.add(3).write(0);
    if task.add(2).read() != 0 {
        held.write(task.read()); held.add(1).write(task.add(1).read());
        held.add(2).write(task.add(2).read()); task.add(2).write(0);
        held.add(3).write(task.add(3).read()); task.add(3).write(0);
    }
    wpgx_gx_run_frame_task(run_task, drop_task, held.cast());
}
