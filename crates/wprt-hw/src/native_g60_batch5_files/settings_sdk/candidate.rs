// Claude G60: whole-member Rust port of hybrid settings_sdk.cpp (wp::settings C++ ABI shims). No residue.
#![allow(non_upper_case_globals, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_void};
use core::sync::atomic::{AtomicU64, Ordering};
include!("../common/cxx.rs");
type Sink = unsafe extern "C" fn(*mut c_void, *const c_char, usize);
extern "C-unwind" {
    fn wprt_settings_load(p: *const c_char, n: usize);
    fn wprt_settings_text(key: *const c_char, var: *const c_char, sink: Sink, ctx: *mut c_void);
    fn wprt_settings_store(key: *const c_char, p: *const c_char, n: usize);
    fn wprt_settings_keys(sink: Sink, ctx: *mut c_void);
    fn wprt_settings_live_flag(key: *const c_char, var: *const c_char, cache: *mut c_void) -> bool;
    fn wprt_settings_live_number(key: *const c_char, var: *const c_char, cache: *mut c_void) -> c_int;
    #[link_name = "_ZN2wp6keymap11setting_keyB5cxx11Ey"] fn keymap_setting_key(ret: *mut CxxString, i: usize) -> *mut CxxString;
    #[link_name = "_ZN2wp6padmap11setting_keyB5cxx11Ey"] fn padmap_setting_key(ret: *mut CxxString, i: usize) -> *mut CxxString;
    #[link_name = "_ZN2wp6keymap6actionEy"] fn keymap_action(i: usize) -> *const u8;
}
struct StrGuard(*mut CxxString); impl Drop for StrGuard { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }
struct VecGuard(*mut CxxVec); impl Drop for VecGuard { fn drop(&mut self) { unsafe { v_destroy_strings(self.0) } } }
unsafe extern "C" fn text_sink(p: *mut c_void, b: *const c_char, n: usize) { s_assign(p as *mut CxxString, b as *const u8, n); }
unsafe extern "C" fn keys_sink(p: *mut c_void, b: *const c_char, n: usize) { v_emplace_string(p as *mut CxxVec, b as *const u8, n); }
#[export_name = "_ZN2wp8settings4loadERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn load(path: *const CxxString) { wprt_settings_load((*path).p as *const c_char, (*path).n) }
#[export_name = "_ZN2wp8settings4textB5cxx11EPKcS2_"]
pub unsafe extern "C-unwind" fn text(ret: *mut CxxString, key: *const c_char, variable: *const c_char) -> *mut CxxString {
    s_init(ret); let g = StrGuard(ret);
    wprt_settings_text(key, variable, text_sink, ret as *mut c_void);
    core::mem::forget(g); ret
}
#[export_name = "_ZN2wp8settings5storeEPKcRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn store(key: *const c_char, value: *const CxxString) { wprt_settings_store(key, (*value).p as *const c_char, (*value).n) }
#[export_name = "_ZN2wp8settings4keysB5cxx11Ev"]
pub unsafe extern "C-unwind" fn keys(ret: *mut CxxVec) -> *mut CxxVec {
    v_init(ret); let g = VecGuard(ret);
    wprt_settings_keys(keys_sink, ret as *mut c_void);
    core::mem::forget(g); ret
}
#[export_name = "_ZNK2wp8settings8LiveFlagclEv"]
pub unsafe extern "C-unwind" fn live_flag(this: *const u8) -> bool { wprt_settings_live_flag(*(this as *const *const c_char), *(this.add(8) as *const *const c_char), this.add(16) as *mut c_void) }
#[export_name = "_ZNK2wp8settings10LiveNumberclEv"]
pub unsafe extern "C-unwind" fn live_number(this: *const u8) -> c_int { wprt_settings_live_number(*(this as *const *const c_char), *(this.add(8) as *const *const c_char), this.add(16) as *mut c_void) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_settings_cache_load(cache: *const c_void) -> u64 { (*(cache as *const AtomicU64)).load(Ordering::Acquire) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_settings_cache_store(cache: *mut c_void, value: u64) { (*(cache as *const AtomicU64)).store(value, Ordering::Release) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_settings_option_row_sdk(i: usize, pad: bool, sink: unsafe extern "C-unwind" fn(*mut c_void, *const c_char, usize, *const c_char, usize), context: *mut c_void) {
    let mut key = core::mem::MaybeUninit::<CxxString>::uninit(); let mut value = core::mem::MaybeUninit::<CxxString>::uninit();
    let (k, v) = (key.as_mut_ptr(), value.as_mut_ptr());
    if pad { padmap_setting_key(k, i); let _gk = StrGuard(k); s_construct_cstr(v, b"auto\0".as_ptr()); let _gv = StrGuard(v); sink(context, (*k).p as _, (*k).n, (*v).p as _, (*v).n); }
    else { keymap_setting_key(k, i); let _gk = StrGuard(k); s_construct_cstr(v, *(keymap_action(i).add(16) as *const *const u8)); let _gv = StrGuard(v); sink(context, (*k).p as _, (*k).n, (*v).p as _, (*v).n); }
}
