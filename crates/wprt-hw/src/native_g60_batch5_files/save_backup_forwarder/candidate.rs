// SPDX-License-Identifier: GPL-3.0-or-later
// Claude G60: save_backup_forwarder.cpp (hybrid/review/save-backup-adopt-port) -> Rust for validate/same_contents/rotate;
// residue.cpp keeps the std::filesystem::path-producing functions and the adopt_root_files exception-capture callback verbatim.
#![allow(dead_code, non_upper_case_globals, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int};
include!("../common/cxx.rs");
#[repr(C)] struct WpSavesPath { count: usize, items: *mut u16 }
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}
extern "C-unwind" {
    fn wpsaves_validate(save: *const WpSavesPath, problem: *mut *mut c_char) -> i32;
    fn wpsaves_same_contents(a: *const WpSavesPath, b: *const WpSavesPath) -> i32;
    fn wpsaves_rotate(root: *const WpSavesPath, keep: i32);
    fn wpsaves_free_string(v: *mut c_char);
}
/// path_view: native() code units of std::filesystem::path (_M_pathname wstring {ptr,len,...} is the first member).
#[inline] unsafe fn path_view(path: *const u8) -> WpSavesPath {
    WpSavesPath { count: *(path.add(8) as *const usize), items: *(path as *const *mut u16) }
}
#[export_name = "_ZN2wp5saves8validateERKNSt10filesystem7__cxx114pathERNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn validate(save: *const u8, problem: *mut CxxString) -> bool {
    let input = path_view(save);
    let mut text: *mut c_char = core::ptr::null_mut();
    let valid = wpsaves_validate(&input, &mut text);
    let src: *const u8 = if text.is_null() { b"\0".as_ptr() } else { text as *const u8 };
    s_assign(problem, src, strlen(src as _));
    wpsaves_free_string(text);
    valid != 0
}
#[export_name = "_ZN2wp5saves13same_contentsERKNSt10filesystem7__cxx114pathES5_"]
pub unsafe extern "C-unwind" fn same_contents(first: *const u8, second: *const u8) -> bool {
    let a = path_view(first); let b = path_view(second);
    wpsaves_same_contents(&a, &b) != 0
}
#[export_name = "_ZN2wp5saves6rotateERKNSt10filesystem7__cxx114pathEi"]
pub unsafe extern "C-unwind" fn rotate(root: *const u8, keep: c_int) {
    let input = path_view(root);
    wpsaves_rotate(&input, keep);
}
