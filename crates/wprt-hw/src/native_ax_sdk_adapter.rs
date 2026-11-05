// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted whole owned dsp-ax-rust-port/sdk_driver.cpp adapter port;typed inline Analyzer methods remain SDK.
use std::ffi::c_void;
extern "C"{
#[link_name="_ZN3DSP11Interpreter11Interpreter17ApplyWriteBackLogEv"]fn target_wphw_ax_sdk_ApplyWriteBackLog(p:*mut c_void);
#[link_name="_ZN3DSP4SDSP15CheckExceptionsEv"]fn target_wphw_ax_sdk_CheckExceptions(p:*mut c_void)->bool;
#[link_name="_ZN3DSP11Interpreter11Interpreter10HandleLoopEv"]fn target_wphw_ax_sdk_HandleLoop(p:*mut c_void);
#[link_name="_ZN3DSP4SDSP8PopStackENS_13StackRegisterE"]fn target_wphw_ax_sdk_PopStack(p:*mut c_void,a0:i32)->u16;
#[link_name="_ZN3DSP4SDSP10StoreStackENS_13StackRegisterEt"]fn target_wphw_ax_sdk_StoreStack(p:*mut c_void,a0:i32,a1:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3absEt"]fn target_wphw_ax_sdk_abs(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4andiEt"]fn target_wphw_ax_sdk_andi(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3aslEt"]fn target_wphw_ax_sdk_asl(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5callrEt"]fn target_wphw_ax_sdk_callr(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4clrlEt"]fn target_wphw_ax_sdk_clrl(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4clrpEt"]fn target_wphw_ax_sdk_clrp(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6cmpaxhEt"]fn target_wphw_ax_sdk_cmpaxh(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3darEt"]fn target_wphw_ax_sdk_dar(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4decmEt"]fn target_wphw_ax_sdk_decm(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4haltEt"]fn target_wphw_ax_sdk_halt(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3iarEt"]fn target_wphw_ax_sdk_iar(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4incmEt"]fn target_wphw_ax_sdk_incm(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter1lEt"]fn target_wphw_ax_sdk_l(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2ldEt"]fn target_wphw_ax_sdk_ld(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4ldaxEt"]fn target_wphw_ax_sdk_ldax(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5ldaxmEt"]fn target_wphw_ax_sdk_ldaxm(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lsmEt"]fn target_wphw_ax_sdk_lsm(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4maddEt"]fn target_wphw_ax_sdk_madd(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5maddxEt"]fn target_wphw_ax_sdk_maddx(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4mulcEt"]fn target_wphw_ax_sdk_mulc(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3negEt"]fn target_wphw_ax_sdk_neg(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3nopEt"]fn target_wphw_ax_sdk_nop(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter7nop_extEt"]fn target_wphw_ax_sdk_nop_ext(p:*mut c_void,a0:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3rtiEt"]fn target_wphw_ax_sdk_rti(p:*mut c_void,a0:u16);
}
// port of wphw_ax_sdk_ApplyWriteBackLog;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_ApplyWriteBackLog(p:*mut c_void){target_wphw_ax_sdk_ApplyWriteBackLog(p)}
// port of wphw_ax_sdk_CheckExceptions;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_CheckExceptions(p:*mut c_void)->bool{target_wphw_ax_sdk_CheckExceptions(p)}
// port of wphw_ax_sdk_HandleLoop;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_HandleLoop(p:*mut c_void){target_wphw_ax_sdk_HandleLoop(p)}
// port of wphw_ax_sdk_PopStack;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_PopStack(p:*mut c_void,a0:i32)->u16{target_wphw_ax_sdk_PopStack(p,a0)}
// port of wphw_ax_sdk_StoreStack;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_StoreStack(p:*mut c_void,a0:i32,a1:u16){target_wphw_ax_sdk_StoreStack(p,a0,a1)}
// port of wphw_ax_sdk_abs;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_abs(p:*mut c_void,a0:u16){target_wphw_ax_sdk_abs(p,a0)}
// port of wphw_ax_sdk_andi;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_andi(p:*mut c_void,a0:u16){target_wphw_ax_sdk_andi(p,a0)}
// port of wphw_ax_sdk_asl;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_asl(p:*mut c_void,a0:u16){target_wphw_ax_sdk_asl(p,a0)}
// port of wphw_ax_sdk_callr;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_callr(p:*mut c_void,a0:u16){target_wphw_ax_sdk_callr(p,a0)}
// port of wphw_ax_sdk_clrl;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_clrl(p:*mut c_void,a0:u16){target_wphw_ax_sdk_clrl(p,a0)}
// port of wphw_ax_sdk_clrp;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_clrp(p:*mut c_void,a0:u16){target_wphw_ax_sdk_clrp(p,a0)}
// port of wphw_ax_sdk_cmpaxh;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_cmpaxh(p:*mut c_void,a0:u16){target_wphw_ax_sdk_cmpaxh(p,a0)}
// port of wphw_ax_sdk_dar;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_dar(p:*mut c_void,a0:u16){target_wphw_ax_sdk_dar(p,a0)}
// port of wphw_ax_sdk_decm;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_decm(p:*mut c_void,a0:u16){target_wphw_ax_sdk_decm(p,a0)}
// port of wphw_ax_sdk_halt;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_halt(p:*mut c_void,a0:u16){target_wphw_ax_sdk_halt(p,a0)}
// port of wphw_ax_sdk_iar;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_iar(p:*mut c_void,a0:u16){target_wphw_ax_sdk_iar(p,a0)}
// port of wphw_ax_sdk_incm;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_incm(p:*mut c_void,a0:u16){target_wphw_ax_sdk_incm(p,a0)}
// port of wphw_ax_sdk_l;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_l(p:*mut c_void,a0:u16){target_wphw_ax_sdk_l(p,a0)}
// port of wphw_ax_sdk_ld;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_ld(p:*mut c_void,a0:u16){target_wphw_ax_sdk_ld(p,a0)}
// port of wphw_ax_sdk_ldax;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_ldax(p:*mut c_void,a0:u16){target_wphw_ax_sdk_ldax(p,a0)}
// port of wphw_ax_sdk_ldaxm;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_ldaxm(p:*mut c_void,a0:u16){target_wphw_ax_sdk_ldaxm(p,a0)}
// port of wphw_ax_sdk_lsm;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_lsm(p:*mut c_void,a0:u16){target_wphw_ax_sdk_lsm(p,a0)}
// port of wphw_ax_sdk_madd;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_madd(p:*mut c_void,a0:u16){target_wphw_ax_sdk_madd(p,a0)}
// port of wphw_ax_sdk_maddx;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_maddx(p:*mut c_void,a0:u16){target_wphw_ax_sdk_maddx(p,a0)}
// port of wphw_ax_sdk_mulc;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_mulc(p:*mut c_void,a0:u16){target_wphw_ax_sdk_mulc(p,a0)}
// port of wphw_ax_sdk_neg;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_neg(p:*mut c_void,a0:u16){target_wphw_ax_sdk_neg(p,a0)}
// port of wphw_ax_sdk_nop;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_nop(p:*mut c_void,a0:u16){target_wphw_ax_sdk_nop(p,a0)}
// port of wphw_ax_sdk_nop_ext;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_nop_ext(p:*mut c_void,a0:u16){target_wphw_ax_sdk_nop_ext(p,a0)}
// port of wphw_ax_sdk_rti;same accepted C++ named method target.
#[no_mangle]pub unsafe extern "C" fn wphw_ax_sdk_rti(p:*mut c_void,a0:u16){target_wphw_ax_sdk_rti(p,a0)}
