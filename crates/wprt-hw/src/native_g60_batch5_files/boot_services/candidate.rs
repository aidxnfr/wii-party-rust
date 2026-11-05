// Claude G60 batch9: Rust port of native/boot_services.cpp (H4 leaf adapters), except wpg_hw_exi_mount (residue.cpp).
#![allow(non_snake_case, non_upper_case_globals, dead_code, clippy::missing_safety_doc)]
use core::ffi::c_char;
include!("../common/cxx.rs");
extern "C-unwind" {
    #[link_name = "_ZN2wp8settings4loadERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn wp_settings_load(s: *const CxxString);
    #[link_name = "_ZN2wp4nand5mountERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn wp_nand_mount(s: *const CxxString) -> bool;
    #[link_name = "_ZN2wp4disc5mountERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn wp_disc_mount(s: *const CxxString) -> bool;
    #[link_name = "_ZN2wp5video5startEv"] fn wp_video_start();
    #[link_name = "_ZN2wp3ipc6reportEv"] fn wp_ipc_report();
}
// Landing pad of the original: out-of-line _M_dispose then _Unwind_Resume.
struct BootStrGuard(*mut CxxString);
impl Drop for BootStrGuard { fn drop(&mut self) { unsafe { cxx_s_dispose(self.0) } } }
// std::string tmp(p); r = f(tmp); ~tmp (inlined: sized delete when heap)
unsafe fn boot_with_str<R>(p: *const c_char, f: impl FnOnce(*const CxxString) -> R) -> R {
    let mut s = core::mem::MaybeUninit::<CxxString>::uninit();
    let sp = s.as_mut_ptr();
    s_construct_cstr(sp, p as *const u8);
    let g = BootStrGuard(sp);
    let r = f(sp);
    core::mem::forget(g);
    if (*sp).p != s_local(sp) { cxx_delete((*sp).p, (*sp).u[0] as usize + 1); }
    r
}
#[no_mangle] pub unsafe extern "C-unwind" fn wpg_hw_settings_load(path: *const c_char) { boot_with_str(path, |s| wp_settings_load(s)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wpg_hw_nand_mount(path: *const c_char) -> bool { boot_with_str(path, |s| wp_nand_mount(s)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wpg_hw_disc_mount(path: *const c_char) -> bool { boot_with_str(path, |s| wp_disc_mount(s)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wpg_hw_video_start() { wp_video_start() }
#[no_mangle] pub unsafe extern "C-unwind" fn wpg_hw_ipc_report() { wp_ipc_report() }
