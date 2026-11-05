// SPDX-License-Identifier: GPL-3.0-or-later
// GNU exception_ptr ownership matching the generated C++ catch/rethrow sequences.
use core::ffi::c_void;
extern "C-unwind"{
 #[link_name="_ZNSt15__exception_ptr13exception_ptr9_M_addrefEv"]fn addref(p:*mut *mut c_void);
 #[link_name="_ZNSt15__exception_ptr13exception_ptr10_M_releaseEv"]fn release(p:*mut *mut c_void);
 #[link_name="_ZSt17rethrow_exceptionNSt15__exception_ptr13exception_ptrE"]fn native_rethrow(p:*mut *mut c_void)->!;
 #[link_name="_ZSt9terminatev"]pub fn terminate()->!;
}
pub unsafe fn capture(work:unsafe extern "C-unwind" fn(*mut c_void)->i32,data:*mut c_void,exception:*mut *mut c_void)->i32{wprt_final_unwind::wprt_final_cxx_try(work,data,exception)}
pub unsafe fn clear(slot:*mut *mut c_void){if !(*slot).is_null(){release(slot);*slot=core::ptr::null_mut();}}
pub struct PtrGuard(pub *mut *mut c_void);impl Drop for PtrGuard{fn drop(&mut self){unsafe{clear(self.0)}}}
pub unsafe fn rethrow(slot:*mut *mut c_void)->!{let mut arg=*slot;if !arg.is_null(){addref(&mut arg);}let _guard=PtrGuard(&mut arg);native_rethrow(&mut arg)}
pub unsafe fn take_rethrow(slot:*mut *mut c_void)->!{let mut value=slot.replace(core::ptr::null_mut());let _owner=PtrGuard(&mut value);rethrow(&mut value)}
