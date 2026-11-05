// AI-assisted (Claude, G60) wiimote_sdk.cpp -> Rust: whole member, no C++ residue.
// send(std::vector<uint8_t>(p,p+n)) on a wp::wiimote::Sender = std::function<void(const std::vector<uint8_t>&)>;
// the temporary vector is a Drop guard, so a throwing sender / bad_function_call frees it during unwinding exactly as C++ does.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("wiimote_sdk export requires pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
#[export_name = "wphw_wiimote_send_sdk"]
pub unsafe extern "C-unwind" fn wphw_wiimote_send_sdk(context: *mut u8, p: *const u8, n: usize) {
    let v = VecDrop(vec_u8_from_range(p, n));
    let invoke: unsafe extern "C-unwind" fn(*const u8, *const RawVec) = std::mem::transmute(function_check(context));
    invoke(context, &v.0);
}
