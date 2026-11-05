// Copyright 2009 Dolphin Emulator Project
// Additional copyrights Duddie and Tratax (c) 2004
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPIntMultiplier.cpp port;native private register/math/flag helpers remain SDK.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,i64,i64,i64,i64)->i64;
// port of pristine DSPIntMultiplier.cpp Interpreter::clrp.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_clrp(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);q(0,0,0,0,0);q(15,0,0,0,0);q(15,1,0xfff0,0,0);q(15,2,0x00ff,0,0);q(15,3,0x0010,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::tstprod.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_tstprod(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let prod=q(1,0,0,0,0);q(14,prod,0,0,0);q(0,0,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::movp.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_movp(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let acc=q(1,0,0,0,0);q(0,0,0,0,0);q(4,d,acc,0,0);q(14,acc,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::movnp.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_movnp(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let acc=-q(1,0,0,0,0);q(0,0,0,0,0);q(4,d,acc,0,0);q(14,acc,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::movpz.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_movpz(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let acc=q(2,0,0,0,0);q(0,0,0,0,0);q(4,d,acc,0,0);q(14,acc,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::addpaxz.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_addpaxz(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let oldprod=q(1,0,0,0,0);let prod=q(2,0,0,0,0);let ax=q(5,s,0,0,0);let res=prod+(ax & !0xffffi64);q(0,0,0,0,0);q(4,d,res,0,0);let res=q(3,d,0,0,0);q(14,res,((oldprod as u64)>(res as u64))as i64,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulaxh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulaxh(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let value=q(7,0,0,0,0) as i16;let prod=q(9,value as u16 as i64,value as u16 as i64,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mul.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mul(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let s=((opc>>11)&1)as i64;let axl=q(6,s,0,0,0)as u16 as i64;let axh=q(7,s,0,0,0)as u16 as i64;let prod=q(9,axh,axl,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulac.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulac(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>11)&1)as i64;let acc=q(3,d,0,0,0)+q(1,0,0,0,0);let axl=q(6,s,0,0,0)as u16 as i64;let axh=q(7,s,0,0,0)as u16 as i64;let prod=q(9,axl,axh,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulmv.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulmv(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>11)&1)as i64;let acc=q(1,0,0,0,0);let axl=q(6,s,0,0,0)as u16 as i64;let axh=q(7,s,0,0,0)as u16 as i64;let prod=q(9,axl,axh,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulmvz.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulmvz(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>11)&1)as i64;let acc=q(2,0,0,0,0);let axl=q(6,s,0,0,0)as u16 as i64;let axh=q(7,s,0,0,0)as u16 as i64;let prod=q(9,axl,axh,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulx.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulx(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let a=q(if s==0{6}else{7},0,0,0,0)as u16 as i64;let b=q(if t==0{6}else{7},1,0,0,0)as u16 as i64;let prod=q(10,s,t,a,b);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulxac.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulxac(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let acc=q(3,d,0,0,0)+q(1,0,0,0,0);let a=q(if s==0{6}else{7},0,0,0,0)as u16 as i64;let b=q(if t==0{6}else{7},1,0,0,0)as u16 as i64;let prod=q(10,s,t,a,b);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulxmv.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulxmv(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let acc=q(1,0,0,0,0);let a=q(if s==0{6}else{7},0,0,0,0)as u16 as i64;let b=q(if t==0{6}else{7},1,0,0,0)as u16 as i64;let prod=q(10,s,t,a,b);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulxmvz.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulxmvz(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let acc=q(2,0,0,0,0);let a=q(if s==0{6}else{7},0,0,0,0)as u16 as i64;let b=q(if t==0{6}else{7},1,0,0,0)as u16 as i64;let prod=q(10,s,t,a,b);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let a=q(8,s,0,0,0)as u16 as i64;let b=q(7,t,0,0,0)as u16 as i64;let prod=q(9,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulcac.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulcac(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let acc=q(3,d,0,0,0)+q(1,0,0,0,0);let a=q(8,s,0,0,0)as u16 as i64;let b=q(7,t,0,0,0)as u16 as i64;let prod=q(9,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulcmv.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulcmv(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let acc=q(1,0,0,0,0);let a=q(8,s,0,0,0)as u16 as i64;let b=q(7,t,0,0,0)as u16 as i64;let prod=q(9,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::mulcmvz.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_mulcmvz(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let t=((opc>>11)&1)as i64;let s=((opc>>12)&1)as i64;let acc=q(2,0,0,0,0);let a=q(8,s,0,0,0)as u16 as i64;let b=q(7,t,0,0,0)as u16 as i64;let prod=q(9,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);q(4,d,acc,0,0);let result=q(3,d,0,0,0);q(14,result,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::maddx.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_maddx(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let t=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let a=q(if s==0{6}else{7},0,0,0,0)as u16 as i64;let b=q(if t==0{6}else{7},1,0,0,0)as u16 as i64;let prod=q(11,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::msubx.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_msubx(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let t=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let a=q(if s==0{6}else{7},0,0,0,0)as u16 as i64;let b=q(if t==0{6}else{7},1,0,0,0)as u16 as i64;let prod=q(12,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::maddc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_maddc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let t=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let a=q(8,s,0,0,0)as u16 as i64;let b=q(7,t,0,0,0)as u16 as i64;let prod=q(11,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::msubc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_msubc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let t=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let a=q(8,s,0,0,0)as u16 as i64;let b=q(7,t,0,0,0)as u16 as i64;let prod=q(12,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::madd.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_madd(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let s=((opc>>8)&1)as i64;let a=q(6,s,0,0,0)as u16 as i64;let b=q(7,s,0,0,0)as u16 as i64;let prod=q(11,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
// port of pristine DSPIntMultiplier.cpp Interpreter::msub.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mul_msub(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let s=((opc>>8)&1)as i64;let a=q(6,s,0,0,0)as u16 as i64;let b=q(7,s,0,0,0)as u16 as i64;let prod=q(12,a,b,0,0);q(0,0,0,0,0);q(13,prod,0,0,0);}
