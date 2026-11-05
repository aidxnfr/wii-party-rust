// SPDX-License-Identifier: GPL-3.0-or-later
// Pinned GNU Win64 foreign-exception capture, replacing project C++ catch(...) sinks.
// core::intrinsics::catch_unwind explicitly supplies a runtime-specific foreign
// exception pointer; it is handed straight back to the matching GNU C++ runtime.
// Unlike std::panic::catch_unwind, no Rust panic-payload cleanup interprets it.
#![feature(core_intrinsics)]
#![allow(internal_features)]
#[cfg(not(all(target_os="windows",target_env="gnu",target_arch="x86_64")))]
compile_error!("This bridge requires the pinned Windows GNU x64 exception runtime");
use core::ffi::c_void;
type Work=unsafe extern "C-unwind" fn(*mut c_void)->i32;
extern "C-unwind"{
 fn __cxa_begin_catch(exception:*mut c_void)->*mut c_void;
 fn __cxa_end_catch();
 #[link_name="_ZSt17current_exceptionv"]fn current_exception(out:*mut *mut c_void);
 #[link_name="_ZNSt15__exception_ptr13exception_ptr10_M_releaseEv"]fn release_exception(slot:*mut *mut c_void);
}
struct Frame{work:Work,data:*mut c_void,exception:*mut *mut c_void,result:i32}
unsafe fn invoke(frame:*mut Frame){(*frame).result=((*frame).work)((*frame).data);}
unsafe fn caught(frame:*mut Frame,exception:*mut u8){
 // Exactly the original C++ catch block: begin, current_exception, move-assign,
 // release displaced owner, end. These runtime operations are noexcept.
 __cxa_begin_catch(exception.cast());
 let mut next=core::ptr::null_mut();current_exception(&mut next);
 let mut old=(*frame).exception.replace(next);
 if !old.is_null(){release_exception(&mut old);}
 __cxa_end_catch();(*frame).result=-1;
}
#[no_mangle]
pub unsafe extern "C-unwind" fn wprt_final_cxx_try(work:Work,data:*mut c_void,exception:*mut *mut c_void)->i32{
 let mut frame=Frame{work,data,exception,result:0};
 core::intrinsics::catch_unwind(invoke,&mut frame,caught);
 frame.result
}
