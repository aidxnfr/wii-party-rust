// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted whole DSPInterpreter.cpp policy port;SDK class lifetime/fields/atomic/analyzer/Host remain native.
// Actual build has PRECISE_BACKLOG undefined;source DEBUG_ASSERT/logging/protection shims are no-ops.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,i64,i64,i64,i64)->i64;
const CR_RESET:u16=1;
const CR_EXTERNAL_INT:u16=2;
const CR_HALT:u16=4;
const CR_INIT_CODE:u16=1024;
const CR_INIT:u16=2048;
const SR_CARRY:u16=1;
const SR_OVERFLOW:u16=2;
const SR_ARITH_ZERO:u16=4;
const SR_SIGN:u16=8;
const SR_OVER_S32:u16=16;
const SR_TOP2BITS:u16=32;
const SR_LOGIC_ZERO:u16=64;
const SR_OVERFLOW_STICKY:u16=128;
const SR_100:u16=256;
const SR_INT_ENABLE:u16=512;
const SR_400:u16=1024;
const SR_EXT_INT_ENABLE:u16=2048;
const SR_1000:u16=4096;
const SR_MUL_MODIFY:u16=8192;
const SR_40_MODE_BIT:u16=16384;
const SR_MUL_UNSIGNED:u16=32768;
const SR_CMP_MASK:u16=63;
// port of constructor body;C++ owns its reference and initialized std::arrays/destructor.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_init(c:*mut c_void,exec:Exec){exec(c,0,0,0,0,0);}
// port of pristine DSPInterpreter.cpp Interpreter::ExecuteInstruction.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_ExecuteInstruction(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let inst=a as u16;let extended=q(1,inst as i64,0,0,0)!=0;if extended{q(2,inst as i64,0,0,0);}q(3,inst as i64,0,0,0);if extended{wprt_dsp_interp_ApplyWriteBackLog(c,0,0,0,0,exec);}0}
// port of pristine DSPInterpreter.cpp Interpreter::Step.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_Step(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};q(4,0,0,0,0);q(5,0,0,0,0);let opc=q(6,0,0,0,0);wprt_dsp_interp_ExecuteInstruction(c,opc,0,0,0,exec);let pc=g(16,0)as u16;if q(7,pc.wrapping_sub(1)as i64,0,0,0)!=0{q(8,0,0,0,0);}0}
// port of pristine DSPInterpreter.cpp Interpreter::RunCyclesThread.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_RunCyclesThread(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let mut cycles=a as i32;loop{if g(17,0)as u16 & CR_HALT!=0{return 0;}if q(9,0,0,0,0)!=0{q(10,0,0,0,0);}wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}}
// port of pristine DSPInterpreter.cpp Interpreter::RunCyclesDebug.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_RunCyclesDebug(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let mut cycles=a as i32;for _ in 0..8{if g(17,0)as u16 &CR_HALT!=0{return 0;}if q(11,g(16,0),0,0,0)!=0{q(12,0,0,0,0);return cycles as i64;}wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}loop{for _ in 0..8{if g(17,0)as u16 &CR_HALT!=0{return 0;}if q(11,g(16,0),0,0,0)!=0{q(12,0,0,0,0);return cycles as i64;}if q(13,g(16,0),0,0,0)!=0{return 0;}wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}for _ in 0..200{if q(11,g(16,0),0,0,0)!=0{q(12,0,0,0,0);return cycles as i64;}wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}}}
// port of pristine DSPInterpreter.cpp Interpreter::RunCycles.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_RunCycles(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let mut cycles=a as i32;for _ in 0..8{if g(17,0)as u16 &CR_HALT!=0{return 0;}wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}loop{for _ in 0..8{if g(17,0)as u16 &CR_HALT!=0{return 0;}if q(13,g(16,0),0,0,0)!=0{return 0;}wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}for _ in 0..200{wprt_dsp_interp_Step(c,0,0,0,0,exec);cycles=cycles.wrapping_sub(1);if cycles<=0{return 0;}}}}
// port of pristine DSPInterpreter.cpp Interpreter::WriteControlRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_WriteControlRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let mut val=a as u16;if val&CR_RESET!=0{q(15,0,0,0,0);val &= !CR_RESET;}if g(17,0)as u16&CR_INIT!=0 && val&CR_INIT==0{s(16,0,0);q(16,0,0,0,0);q(17,0,0,0,0);val &= !CR_INIT;val |= CR_INIT_CODE;let time=(q(18,0,0,0,0)as u64).wrapping_add(130);s(18,0,time as i64);}s(17,0,val as i64);0}
// port of pristine DSPInterpreter.cpp Interpreter::ReadControlRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_ReadControlRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};if g(17,0)as u16 &CR_INIT_CODE!=0{if q(18,0,0,0,0)as u64>=g(18,0)as u64{s(17,0,((g(17,0)as u16)&!CR_INIT_CODE)as i64);}}g(17,0)as u16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::SetSRFlag.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_SetSRFlag(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};s(15,0,((g(15,0)as u16)|(a as u16))as i64);0}
// port of pristine DSPInterpreter.cpp Interpreter::IsSRFlagSet.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_IsSRFlagSet(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};((g(15,0)as u16 &a as u16)!=0)as i64}
// port of pristine DSPInterpreter.cpp Interpreter::CheckCondition.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_CheckCondition(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let flag=|f:u16|g(15,0)as u16&f!=0;let less=||flag(SR_OVERFLOW)!=flag(SR_SIGN);let cb=||!(flag(SR_OVER_S32)||flag(SR_TOP2BITS))||flag(SR_ARITH_ZERO);(match a as u8 &15{15=>true,0=>!less(),1=>less(),2=>!less()&&!flag(SR_ARITH_ZERO),3=>less()||flag(SR_ARITH_ZERO),4=>!flag(SR_ARITH_ZERO),5=>flag(SR_ARITH_ZERO),6=>!flag(SR_CARRY),7=>flag(SR_CARRY),8=>!flag(SR_OVER_S32),9=>flag(SR_OVER_S32),10=>!cb(),11=>cb(),12=>!flag(SR_LOGIC_ZERO),13=>flag(SR_LOGIC_ZERO),14=>flag(SR_OVERFLOW),_=>true})as i64}
// port of pristine DSPInterpreter.cpp Interpreter::IncrementAddressRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_IncrementAddressRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let ar=g(0,a)as u16 as u32;let wr=g(2,a)as u16 as u32;let mut nar=ar+1;if (nar^ar)>((wr|1)<<1){nar=nar.wrapping_sub(wr+1);}nar as u16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::DecrementAddressRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_DecrementAddressRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let ar=g(0,a)as u16 as u32;let wr=g(2,a)as u16 as u32;let mut nar=ar+wr;if ((nar^ar)&((wr|1)<<1))>wr{nar=nar.wrapping_sub(wr+1);}nar as u16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::IncreaseAddressRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_IncreaseAddressRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let ar=g(0,a)as u16 as u32;let wr=g(2,a)as u16 as u32;let ix=b as i16 as i32;let mx=(wr|1)<<1;let mut nar=ar.wrapping_add(ix as u32);let dar=(nar^ar^ix as u32)&mx;if ix>=0{if dar>wr{nar=nar.wrapping_sub(wr+1);}}else if ((nar.wrapping_add(wr+1)^nar)&dar)<=wr{nar=nar.wrapping_add(wr+1);}nar as u16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::DecreaseAddressRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_DecreaseAddressRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let ar=g(0,a)as u16 as u32;let wr=g(2,a)as u16 as u32;let ix=b as i16 as i32;let mx=(wr|1)<<1;let mut nar=ar.wrapping_sub(ix as u32);let dar=(nar^ar^(!ix)as u32)&mx;if ix as u32>0xffff8000{if dar>wr{nar=nar.wrapping_sub(wr+1);}}else if ((nar.wrapping_add(wr+1)^nar)&dar)<=wr{nar=nar.wrapping_add(wr+1);}nar as u16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetLongACX.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetLongACX(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};(((g(8,a)as u16 as u32)<<16)|(g(7,a)as u16 as u32))as i32 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetAXLow.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetAXLow(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};g(7,a)as i16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetAXHigh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetAXHigh(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};g(8,a)as i16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetLongAcc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetLongAcc(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};g(3,a)}
// port of pristine DSPInterpreter.cpp Interpreter::SetLongAcc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_SetLongAcc(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};s(3,a,b.wrapping_shl(24)>>24);0}
// port of pristine DSPInterpreter.cpp Interpreter::GetAccLow.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetAccLow(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};g(4,a)as i16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetAccMid.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetAccMid(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};g(5,a)as i16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetAccHigh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetAccHigh(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};g(6,a)as i16 as i64}
// port of pristine DSPInterpreter.cpp Interpreter::GetLongProduct.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetLongProduct(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let hi=(g(12,0)as u8 as i8 as i64)<<32;let low=(((g(11,0)as u16 as i64)+(g(13,0)as u16 as i64))<<16)|(g(10,0)as u16 as i64);hi+low}
// port of pristine DSPInterpreter.cpp Interpreter::GetLongProductRounded.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetLongProductRounded(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let prod=wprt_dsp_interp_GetLongProduct(c,0,0,0,0,exec);(prod+if prod&0x10000!=0{0x8000}else{0x7fff})&!0xffff}
// port of pristine DSPInterpreter.cpp Interpreter::SetLongProduct.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_SetLongProduct(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};s(9,0,a&0xffffffffff);0}
// port of pristine DSPInterpreter.cpp Interpreter::GetMultiplyProduct.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_GetMultiplyProduct(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let a=a as u16;let b=b as u16;let sign=d as u8;let unsigned=g(15,0)as u16 &SR_MUL_UNSIGNED!=0;let mut prod=if sign==1 &&unsigned{(a as u32*b as u32)as i64}else if sign==2&&unsigned{(a as i32*b as i16 as i32)as i64}else{(a as i16 as i32*b as i16 as i32)as i64};if g(15,0)as u16&SR_MUL_MODIFY==0{prod<<=1;}prod}
// port of pristine DSPInterpreter.cpp Interpreter::Multiply.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_Multiply(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};wprt_dsp_interp_GetMultiplyProduct(c,a,b,d,0,exec)}
// port of pristine DSPInterpreter.cpp Interpreter::MultiplyAdd.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_MultiplyAdd(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};wprt_dsp_interp_GetLongProduct(c,0,0,0,0,exec)+wprt_dsp_interp_GetMultiplyProduct(c,a,b,d,0,exec)}
// port of pristine DSPInterpreter.cpp Interpreter::MultiplySub.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_MultiplySub(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};wprt_dsp_interp_GetLongProduct(c,0,0,0,0,exec)-wprt_dsp_interp_GetMultiplyProduct(c,a,b,d,0,exec)}
// port of pristine DSPInterpreter.cpp Interpreter::MultiplyMulX.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_MultiplyMulX(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let x=a as u8;let y=b as u8;if x==0&&y==0{wprt_dsp_interp_Multiply(c,d,e,1,0,exec)}else if x==0&&y==1{wprt_dsp_interp_Multiply(c,d,e,2,0,exec)}else if x==1&&y==0{wprt_dsp_interp_Multiply(c,e,d,2,0,exec)}else{wprt_dsp_interp_Multiply(c,d,e,0,0,exec)}}
// port of pristine DSPInterpreter.cpp Interpreter::UpdateSR16.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_UpdateSR16(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let value=a as i16 as i64;s(15,0,((g(15,0)as u16)&!SR_CMP_MASK)as i64);if b!=0{s(15,0,((g(15,0)as u16)|SR_CARRY)as i64);}if d!=0{s(15,0,((g(15,0)as u16)|SR_OVERFLOW)as i64);s(15,0,((g(15,0)as u16)|SR_OVERFLOW_STICKY)as i64);}if value==0{s(15,0,((g(15,0)as u16)|SR_ARITH_ZERO)as i64);}if value<0{s(15,0,((g(15,0)as u16)|SR_SIGN)as i64);}if e!=0{s(15,0,((g(15,0)as u16)|SR_OVER_S32)as i64);}if ((value as u16>>14)==0)||((value as u16>>14)==3){s(15,0,((g(15,0)as u16)|SR_TOP2BITS)as i64);}0}
// port of pristine DSPInterpreter.cpp Interpreter::UpdateSR64.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_UpdateSR64(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let value=a;s(15,0,((g(15,0)as u16)&!SR_CMP_MASK)as i64);if b!=0{s(15,0,((g(15,0)as u16)|SR_CARRY)as i64);}if d!=0{s(15,0,((g(15,0)as u16)|SR_OVERFLOW)as i64);s(15,0,((g(15,0)as u16)|SR_OVERFLOW_STICKY)as i64);}if value==0{s(15,0,((g(15,0)as u16)|SR_ARITH_ZERO)as i64);}if value<0{s(15,0,((g(15,0)as u16)|SR_SIGN)as i64);}if value!=value as i32 as i64{s(15,0,((g(15,0)as u16)|SR_OVER_S32)as i64);}if (value&0xc0000000)==0||(value&0xc0000000)==0xc0000000{s(15,0,((g(15,0)as u16)|SR_TOP2BITS)as i64);}0}
// port of pristine DSPInterpreter.cpp Interpreter::UpdateSR64Add.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_UpdateSR64Add(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let carry=(a as u64)>(d as u64);let overflow=((a^d)&(b^d))<0;wprt_dsp_interp_UpdateSR64(c,d,carry as i64,overflow as i64,0,exec);0}
// port of pristine DSPInterpreter.cpp Interpreter::UpdateSR64Sub.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_UpdateSR64Sub(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let carry=(a as u64)>=(d as u64);let overflow=((a^d)&((-b)^d))<0;wprt_dsp_interp_UpdateSR64(c,d,carry as i64,overflow as i64,0,exec);0}
// port of pristine DSPInterpreter.cpp Interpreter::UpdateSRLogicZero.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_UpdateSRLogicZero(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let sr=g(15,0)as u16;s(15,0,(if a!=0{sr|SR_LOGIC_ZERO}else{sr&!SR_LOGIC_ZERO})as i64);0}
// port of pristine DSPInterpreter.cpp Interpreter::OpReadRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_OpReadRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let reg=(a as i32&31)as i64;match reg{12..=15=>q(19,reg-12,0,0,0)as u16 as i64,0..=3=>g(0,reg)as u16 as i64,4..=7=>g(1,reg-4)as u16 as i64,8..=11=>g(2,reg-8)as u16 as i64,16..=17=>g(6,reg-16)as u16 as i64,18=>g(14,0)as u16 as i64,19=>g(15,0)as u16 as i64,20..=23=>g(10+reg-20,0)as u16 as i64,24..=25=>g(7,reg-24)as u16 as i64,26..=27=>g(8,reg-26)as u16 as i64,28..=29=>g(4,reg-28)as u16 as i64,30..=31=>{if g(15,0)as u16&SR_40_MODE_BIT!=0{let acc=wprt_dsp_interp_GetLongAcc(c,reg-30,0,0,0,exec);if acc!=acc as i32 as i64{return if acc>0{0x7fff}else{0x8000};}}g(5,reg-30)as u16 as i64},_=>0}}
// port of pristine DSPInterpreter.cpp Interpreter::OpWriteRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_OpWriteRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let reg=(a as i32&31)as i64;let val=b as u16;match reg{16..=17=>s(6,reg-16,val as i8 as i32 as u32 as i64),12..=15=>{q(20,reg-12,val as i64,0,0);},0..=3=>s(0,reg,val as i64),4..=7=>s(1,reg-4,val as i64),8..=11=>s(2,reg-8,val as i64),18=>s(14,0,(val&0xff)as i64),19=>s(15,0,(val&!SR_100)as i64),20..=21=>s(10+reg-20,0,val as i64),22=>s(12,0,(val&0xff)as i64),23=>s(13,0,val as i64),24..=25=>s(7,reg-24,val as i64),26..=27=>s(8,reg-26,val as i64),28..=29=>s(4,reg-28,val as i64),30..=31=>s(5,reg-30,val as i64),_=>{}}0}
// port of pristine DSPInterpreter.cpp Interpreter::ConditionalExtendAccum.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_ConditionalExtendAccum(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let reg=a as i32;if reg!=30&&reg!=31{return 0;}if g(15,0)as u16&SR_40_MODE_BIT==0{return 0;}let m=g(5,(reg-30)as i64)as u16;s(6,(reg-30)as i64,if m&0x8000!=0{0xffffffffu32 as i64}else{0});s(4,(reg-30)as i64,0);0}
// port of pristine DSPInterpreter.cpp Interpreter::ApplyWriteBackLog.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_ApplyWriteBackLog(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};let mut i=0;loop{let idx=g(20,i)as i32;if idx == -1{break;}let value=g(19,i)as u16;wprt_dsp_interp_OpWriteRegister(c,idx as i64,value as i64,0,0,exec);s(20,i,-1);i+=1;}0}
// port of pristine DSPInterpreter.cpp Interpreter::WriteToBackLog.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_WriteToBackLog(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};s(19,a,d as u16 as i64);s(20,a,b as i32 as i64);0}
// port of pristine DSPInterpreter.cpp Interpreter::ZeroWriteBackLog.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_ZeroWriteBackLog(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};0}
// port of pristine DSPInterpreter.cpp Interpreter::ZeroWriteBackLogPreserveAcc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_ZeroWriteBackLogPreserveAcc(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};0}
// port of pristine DSPInterpreter.cpp Interpreter::nop.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_interp_nop(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,exec:Exec)->i64{let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let g=|f,i|exec(c,100,f,i,0,0);let s=|f,i,v|{exec(c,101,f,i,v,0);};0}
