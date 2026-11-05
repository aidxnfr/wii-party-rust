// Claude G60: whole-member Rust port of hybrid input_sdk.cpp. Residue input_lifetime.cpp holds COMDAT anchors only.
#![allow(non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_void};
use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};
include!("../common/cxx.rs");
const ACTIONS: usize = 20;
extern "C-unwind" {
    #[link_name = "_ZN2wp6keymap4loadERKSt8functionIFNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKS7_EERSt6vectorIS7_SaIS7_EE"] fn keymap_load(ret: *mut CxxVec, value: *const CxxFunction, invalid: *mut CxxVec) -> *mut CxxVec;
    #[link_name = "_ZN2wp8settings4textB5cxx11EPKcS2_"] fn settings_text(ret: *mut CxxString, key: *const c_char, var: *const c_char) -> *mut CxxString;
    #[link_name = "_ZNSt6chrono3_V212steady_clock3nowEv"] fn steady_now() -> i64;
    fn wprt_input_ignored_list(list: *const c_void, n: usize);
    fn __cxa_guard_acquire(g: *mut i64) -> c_int;
    fn __cxa_guard_release(g: *mut i64);
    fn __cxa_guard_abort(g: *mut i64);
}
extern "C" {
    fn getenv(n: *const c_char) -> *const c_char;
    fn atexit(f: unsafe extern "C" fn()) -> c_int;
    #[link_name = "_ZZN2wp10comparison7enabledEvE5value"] static mut cmp_enabled_value: bool;
    #[link_name = "_ZGVZN2wp10comparison7enabledEvE5value"] static mut cmp_enabled_guard: i64;
    #[link_name = "_ZN2wp10comparison8progressE"] static cmp_progress: AtomicU64;
}
static mut BINDINGS: [CxxVec; ACTIONS] = [const { CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() } }; ACTIONS];
static mut BINDINGS_GUARD: i64 = 0;
static mut ORIGIN: i64 = 0;
static mut ORIGIN_GUARD: i64 = 0;
static TI_GETTER: CxxTypeInfo = CxxTypeInfo { vptr: unsafe { (&raw const CXX_CLASS_TYPE_INFO_VTABLE as *const u8).add(16) }, name: b"*ZZN2wp5input8bindingsEvENKUlvE_clEvEUlRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEE_\0".as_ptr() };
struct AbortGuard(*mut i64); impl Drop for AbortGuard { fn drop(&mut self) { unsafe { __cxa_guard_abort(self.0) } } }
struct FnGuard(*mut CxxFunction); impl Drop for FnGuard { fn drop(&mut self) { unsafe { f_destroy(self.0) } } }
struct VecGuard(*mut CxxVec); impl Drop for VecGuard { fn drop(&mut self) { unsafe { v_destroy_strings(self.0) } } }
struct BindingsGuard(*mut CxxVec); impl Drop for BindingsGuard { fn drop(&mut self) { unsafe { for i in (0..ACTIONS).rev() { v_free(self.0.add(i)); } } } }
#[inline] unsafe fn guard_done(g: *mut i64) -> bool { (*(g as *const AtomicU8)).load(Ordering::Acquire) != 0 }
unsafe extern "C-unwind" fn getter_manager(dest: *mut CxxFunction, source: *const CxxFunction, op: i32) -> bool {
    match op { 0 => { *(dest as *mut *const CxxTypeInfo) = &TI_GETTER; } 1 => { *(dest as *mut *const CxxFunction) = source; } _ => {} }
    false
}
unsafe extern "C-unwind" fn getter_invoke(ret: *mut CxxString, _functor: *const CxxFunction, key: *const CxxString) -> *mut CxxString { settings_text(ret, (*key).p as *const c_char, core::ptr::null()) }
unsafe extern "C" fn destroy_bindings() { for i in (0..ACTIONS).rev() { v_free(core::ptr::addr_of_mut!(BINDINGS[i])); } }
unsafe fn comparison_now() -> i64 {
    let g = core::ptr::addr_of_mut!(cmp_enabled_guard);
    if !guard_done(g) && __cxa_guard_acquire(g) != 0 { cmp_enabled_value = !getenv(b"WP_DETERMINISTIC\0".as_ptr() as _).is_null(); __cxa_guard_release(g); }
    if !cmp_enabled_value { return steady_now(); }
    let value = cmp_progress.load(Ordering::Relaxed);
    (value / 60750000).wrapping_mul(1000000000).wrapping_add((value % 60750000).wrapping_mul(1000000000) / 60750000) as i64
}
#[export_name = "_ZN2wp5input8bindingsEv"]
pub unsafe extern "C-unwind" fn bindings() -> *const CxxVec {
    let g = core::ptr::addr_of_mut!(BINDINGS_GUARD);
    let value = core::ptr::addr_of_mut!(BINDINGS) as *mut CxxVec;
    if !guard_done(g) && __cxa_guard_acquire(g) != 0 {
        let abort = AbortGuard(g);
        let mut invalid = CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() };
        let gi = VecGuard(&mut invalid);
        {
            let mut f = CxxFunction { data: [0; 2], manager: getter_manager as usize, invoker: getter_invoke as usize };
            let _gf = FnGuard(&mut f);
            keymap_load(value, &f, &mut invalid);
        }
        let gv = BindingsGuard(value);
        wprt_input_ignored_list(&invalid as *const CxxVec as *const c_void, v_len(&invalid, 32));
        core::mem::forget(gv);
        drop(gi);
        atexit(destroy_bindings);
        core::mem::forget(abort);
        __cxa_guard_release(g);
    }
    value
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_input_invalid_at_sdk(list: *const c_void, index: usize) -> *const c_char { (*((*(list as *const CxxVec)).b as *const CxxString).add(index)).p as *const c_char }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_input_origin_ticks_sdk() -> i64 {
    let g = core::ptr::addr_of_mut!(ORIGIN_GUARD);
    if !guard_done(g) && __cxa_guard_acquire(g) != 0 { let abort = AbortGuard(g); ORIGIN = comparison_now(); core::mem::forget(abort); __cxa_guard_release(g); }
    ORIGIN
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_input_codes(index: usize, p: *mut *const i32, n: *mut usize) { let v = bindings().add(index); *p = (*v).b as *const i32; *n = v_len(v, 4); }
