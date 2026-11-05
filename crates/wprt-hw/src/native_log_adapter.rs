// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete hybrid/cpp/patched/log.cpp port. GNU CRT formatting remains the original SDK.
// Win64 varargs register homes/va_list offset confirmed from actual accepted log.cpp.obj disassembly.
use std::ffi::c_char;
extern "C"{fn wprt_log_enabled()->bool;fn wprt_log_seconds()->f64;fn wprt_log_emit(seconds:f64,category:*const c_char,message:*const c_char);fn __mingw_vsnprintf(out:*mut c_char,size:usize,format:*const c_char,args:*mut c_char)->i32;}
// port of wp::log::enabled.
#[export_name="_ZN2wp3log7enabledEv"]pub unsafe extern "C" fn enabled()->bool{wprt_log_enabled()}
// Win64 variadic ABI adapter only:home R8/R9 at originalentryRSP+24/+32,pass va_list to fixed Rust body.
// Source GCC stores precisely these homes;remaining stack arguments already follow them contiguously.
std::arch::global_asm!(r#"
.text
.p2align 4
.globl _ZN2wp3log5writeEPKcS2_z
.def _ZN2wp3log5writeEPKcS2_z; .scl 2; .type 32; .endef
.seh_proc _ZN2wp3log5writeEPKcS2_z
_ZN2wp3log5writeEPKcS2_z:
sub rsp, 40
.seh_stackalloc 40
.seh_endprologue
mov [rsp+64], r8
mov [rsp+72], r9
lea r8, [rsp+64]
call wprt_log_write_v
add rsp, 40
ret
.seh_endproc
"#);
// port of wp::log::write:checkenabled->seconds->same512bytevsnprintf->emit.
#[no_mangle]pub unsafe extern "C" fn wprt_log_write_v(category:*const c_char,format:*const c_char,args:*mut c_char){if !wprt_log_enabled(){return;}let seconds=wprt_log_seconds();let mut message=std::mem::MaybeUninit::<[c_char;512]>::uninit();__mingw_vsnprintf(message.as_mut_ptr().cast(),512,format,args);wprt_log_emit(seconds,category,message.as_ptr().cast());}
