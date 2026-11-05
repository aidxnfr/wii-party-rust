// Claude G60: whole-member Rust port of hybrid keymap_sdk.cpp (wp::keymap C++ ABI shims). No residue.
#![allow(non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_long, c_void};
use core::sync::atomic::{AtomicU8, Ordering};
include!("../common/cxx.rs");
type Sink = unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize);
type CodeSink = unsafe extern "C-unwind" fn(*mut c_void, i32);
type CodesSink = unsafe extern "C-unwind" fn(*mut c_void, usize, *const i32, usize);
type Value = unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize, Sink, *mut c_void);
const ACTIONS: usize = 20;
extern "C-unwind" {
    fn wprt_keymap_action(i: usize, name: *mut *const c_char, buttons: *mut u32, defaults: *mut *const c_char);
    fn wprt_keymap_setting_key(i: usize, s: Sink, c: *mut c_void);
    fn wprt_keymap_key_name(code: i32, s: Sink, c: *mut c_void);
    fn wprt_keymap_parse(p: *const u8, n: usize, code: CodeSink, inv: Sink, c: *mut c_void);
    fn wprt_keymap_format(p: *const i32, n: usize, sep: *const c_char, s: Sink, c: *mut c_void);
    fn wprt_keymap_defaults(s: CodesSink, c: *mut c_void);
    fn wprt_keymap_load(v: Value, vc: *mut c_void, codes: CodesSink, inv: Sink, c: *mut c_void);
    fn __cxa_guard_acquire(g: *mut i64) -> c_int;
    fn __cxa_guard_release(g: *mut i64);
    fn __cxa_guard_abort(g: *mut i64);
    #[link_name = "_ZSt24__throw_invalid_argumentPKc"] fn throw_invalid_argument(m: *const c_char) -> !;
    #[link_name = "_ZSt20__throw_out_of_rangePKc"] fn throw_out_of_range(m: *const c_char) -> !;
}
extern "C" { fn strtol(s: *const c_char, e: *mut *mut c_char, b: c_int) -> c_long; fn _errno() -> *mut c_int; }
#[repr(C)] #[derive(Clone, Copy)] struct ActionInfo { name: *const c_char, buttons: u32, defaults: *const c_char }
static mut INFOS: [ActionInfo; ACTIONS] = [ActionInfo { name: core::ptr::null(), buttons: 0, defaults: core::ptr::null() }; ACTIONS];
static mut INFOS_GUARD: i64 = 0;
struct StrGuard(*mut CxxString); impl Drop for StrGuard { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }
struct ParsedGuard(*mut CxxVec); impl Drop for ParsedGuard { fn drop(&mut self) { unsafe { v_destroy_strings(self.0.add(1)); v_free(self.0); } } }
struct BindingsGuard(*mut CxxVec); impl Drop for BindingsGuard { fn drop(&mut self) { unsafe { destroy_bindings(self.0) } } }
struct AbortGuard(*mut i64); impl Drop for AbortGuard { fn drop(&mut self) { unsafe { __cxa_guard_abort(self.0) } } }
unsafe fn destroy_bindings(b: *mut CxxVec) { for i in (0..ACTIONS).rev() { v_free(b.add(i)); } }
unsafe extern "C-unwind" fn string_sink(c: *mut c_void, p: *const u8, n: usize) { s_assign(c as *mut CxxString, p, n); }
unsafe extern "C-unwind" fn parse_code(c: *mut c_void, code: i32) { v_push_pod(c as *mut CxxVec, 4, &code as *const i32 as *const u8); }
unsafe extern "C-unwind" fn parse_invalid(c: *mut c_void, p: *const u8, n: usize) { v_emplace_string((c as *mut CxxVec).add(1), p, n); }
unsafe fn get_codes(c: *const c_void, index: usize, p: *mut *const i32, n: *mut usize) { let v = (c as *const CxxVec).add(index); *p = (*v).b as *const i32; *n = v_len(v, 4); }
/// vector<int>::assign(first, last) (_M_assign_aux, forward iterators).
unsafe extern "C-unwind" fn set_codes(c: *mut c_void, index: usize, p: *const i32, n: usize) {
    let v = (c as *mut CxxVec).add(index);
    let cap = ((*v).c as usize - (*v).b as usize) / 4;
    if n > cap {
        if n > (isize::MAX as usize) / 4 { cxx_throw_length_error(concat!("cannot create std::vector larger than max_size()", "\0").as_ptr() as _); }
        let tmp = cxx_new(n * 4);
        core::ptr::copy_nonoverlapping(p as *const u8, tmp, n * 4);
        v_free(v);
        (*v).b = tmp; (*v).e = tmp.add(n * 4); (*v).c = tmp.add(n * 4);
    } else {
        if n > 0 { core::ptr::copy(p as *const u8, (*v).b, n * 4); }
        (*v).e = (*v).b.add(n * 4);
    }
}
unsafe extern "C-unwind" fn get_value(c: *mut c_void, p: *const u8, n: usize, sink: Sink, context: *mut c_void) {
    let get = c as *const CxxFunction;
    let mut result = core::mem::MaybeUninit::<CxxString>::uninit();
    {
        let mut arg = core::mem::MaybeUninit::<CxxString>::uninit();
        s_construct(arg.as_mut_ptr(), p, n); let _ga = StrGuard(arg.as_mut_ptr());
        f_check(get);
        let inv: unsafe extern "C-unwind" fn(*mut CxxString, *const CxxFunction, *const CxxString) -> *mut CxxString = core::mem::transmute((*get).invoker);
        inv(result.as_mut_ptr(), get, arg.as_ptr());
    }
    let _gr = StrGuard(result.as_mut_ptr());
    sink(context, (*result.as_ptr()).p, (*result.as_ptr()).n);
}
#[repr(C)] struct LoadResult { bindings: *mut CxxVec, invalid: *mut CxxVec }
unsafe extern "C-unwind" fn load_codes(c: *mut c_void, index: usize, p: *const i32, n: usize) { set_codes((*(c as *mut LoadResult)).bindings as *mut c_void, index, p, n); }
unsafe extern "C-unwind" fn load_invalid(c: *mut c_void, p: *const u8, n: usize) { v_emplace_string((*(c as *mut LoadResult)).invalid, p, n); }

#[export_name = "_ZN2wp6keymap6actionEy"]
pub unsafe extern "C-unwind" fn action(index: usize) -> *const ActionInfo {
    let g = core::ptr::addr_of_mut!(INFOS_GUARD);
    if (*(g as *const AtomicU8)).load(Ordering::Acquire) == 0 && __cxa_guard_acquire(g) != 0 {
        let abort = AbortGuard(g);
        let mut out = [ActionInfo { name: core::ptr::null(), buttons: 0, defaults: core::ptr::null() }; ACTIONS];
        for i in 0..ACTIONS { wprt_keymap_action(i, &mut out[i].name, &mut out[i].buttons, &mut out[i].defaults); }
        INFOS = out;
        core::mem::forget(abort);
        __cxa_guard_release(g);
    }
    INFOS.as_ptr().add(index)
}
#[export_name = "_ZN2wp6keymap11setting_keyB5cxx11Ey"]
pub unsafe extern "C-unwind" fn setting_key(ret: *mut CxxString, index: usize) -> *mut CxxString { s_init(ret); let g = StrGuard(ret); wprt_keymap_setting_key(index, string_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp6keymap8key_nameB5cxx11Ei"]
pub unsafe extern "C-unwind" fn key_name(ret: *mut CxxString, code: c_int) -> *mut CxxString { s_init(ret); let g = StrGuard(ret); wprt_keymap_key_name(code, string_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp6keymap5parseERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn parse(ret: *mut CxxVec, p: *const CxxString) -> *mut CxxVec {
    v_init(ret); v_init(ret.add(1)); let g = ParsedGuard(ret);
    wprt_keymap_parse((*p).p, (*p).n, parse_code, parse_invalid, ret as *mut c_void);
    core::mem::forget(g); ret
}
#[export_name = "_ZN2wp6keymap6formatB5cxx11ERKSt6vectorIiSaIiEEPKc"]
pub unsafe extern "C-unwind" fn format(ret: *mut CxxString, codes: *const CxxVec, separator: *const c_char) -> *mut CxxString {
    s_init(ret); let g = StrGuard(ret);
    wprt_keymap_format((*codes).b as *const i32, v_len(codes, 4), separator, string_sink, ret as *mut c_void);
    core::mem::forget(g); ret
}
#[export_name = "_ZN2wp6keymap8defaultsEv"]
pub unsafe extern "C-unwind" fn defaults(ret: *mut CxxVec) -> *mut CxxVec {
    for i in 0..ACTIONS { v_init(ret.add(i)); } let g = BindingsGuard(ret);
    wprt_keymap_defaults(set_codes, ret as *mut c_void);
    core::mem::forget(g); ret
}
#[export_name = "_ZN2wp6keymap4loadERKSt8functionIFNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKS7_EERSt6vectorIS7_SaIS7_EE"]
pub unsafe extern "C-unwind" fn load(ret: *mut CxxVec, value: *const CxxFunction, invalid: *mut CxxVec) -> *mut CxxVec {
    for i in 0..ACTIONS { v_init(ret.add(i)); } let g = BindingsGuard(ret);
    let mut context = LoadResult { bindings: ret, invalid };
    wprt_keymap_load(get_value, value as *mut c_void, load_codes, load_invalid, &mut context as *mut LoadResult as *mut c_void);
    core::mem::forget(g); ret
}
struct ErrnoSave(c_int); impl Drop for ErrnoSave { fn drop(&mut self) { unsafe { if *_errno() == 0 { *_errno() = self.0; } } } }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_keymap_stoi(p: *const u8, n: usize) -> i32 {
    let mut s = core::mem::MaybeUninit::<CxxString>::uninit();
    s_construct(s.as_mut_ptr(), p, n); let _gs = StrGuard(s.as_mut_ptr());
    let str = (*s.as_ptr()).p as *const c_char;
    let _save = ErrnoSave(*_errno()); *_errno() = 0;
    let mut end: *mut c_char = core::ptr::null_mut();
    let tmp = strtol(str, &mut end, 10);
    if end as *const c_char == str { throw_invalid_argument(b"stoi\0".as_ptr() as _); }
    else if *_errno() == 34 || (tmp as i64) < i32::MIN as i64 || (tmp as i64) > i32::MAX as i64 { throw_out_of_range(b"stoi\0".as_ptr() as _); }
    tmp as i32
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_keymap_string_sdk(raw: *const c_void, p: *mut *const u8, n: *mut usize) { let s = raw as *const CxxString; *p = (*s).p; *n = (*s).n; }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_keymap_codes_sdk(raw: *const c_void, index: usize, p: *mut *const i32, n: *mut usize) { get_codes(raw, index, p, n); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_keymap_value_sdk(raw: *mut c_void, p: *const u8, n: usize, sink: Sink, context: *mut c_void) { get_value(raw, p, n, sink, context); }
