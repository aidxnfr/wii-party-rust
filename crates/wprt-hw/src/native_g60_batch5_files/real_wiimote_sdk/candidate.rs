// AI-assisted (Claude, G60) real_wiimote_sdk.cpp -> Rust: whole member, no C++ residue.
// Sender temporaries and the std::string path are Drop guards (exact unwinding cleanup); strings are built in place.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("real_wiimote_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
use std::mem::MaybeUninit;
#[export_name = "wphw_real_send_sdk"]
pub unsafe extern "C-unwind" fn wphw_real_send_sdk(c: *mut u8, p: *const u8, n: usize) {
    let v = VecDrop(vec_u8_from_range(p, n));
    let invoke: unsafe extern "C-unwind" fn(*const u8, *const RawVec) = std::mem::transmute(function_check(c));
    invoke(c, &v.0);
}
#[export_name = "wphw_real_path_sdk"]
pub unsafe extern "C-unwind" fn wphw_real_path_sdk(p: *const u8, n: usize, print: unsafe extern "C-unwind" fn(*const u8)) {
    let mut slot = MaybeUninit::<CxxString>::uninit(); let s = slot.as_mut_ptr();
    str_construct(s, p, n);
    let _g = StrDrop(s);
    print((*s).p);
}
