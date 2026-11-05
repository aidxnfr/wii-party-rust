// Copyright 2009 Dolphin Emulator Project
// Additional copyrights Duddie and Tratax(c)2004
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPIntArithmetic.cpp port;typed private helpers/union-field access remain SDK.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,i64,i64,i64,i64)->i64;
// port of DSPIntUtil.h dsp_convert_long_acc/dsp_round_long_acc and DSPIntCCUtil.h isOverS32.
fn convert(v:i64)->i64{v.wrapping_shl(24)>>24}
fn round(v:i64)->i64{(v+if v&0x10000!=0{0x8000}else{0x7fff}) & !0xffff}
fn over32(v:i64)->bool{v!=v as i32 as i64}
// port of pristine DSPIntArithmetic.cpp Interpreter::clr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_clr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>11)&1)as i64;q(2,d,0,0,0);q(3,0,0,0,0);q(0,0,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::clrl.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_clrl(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let acc=round(q(1,d,0,0,0));q(0,0,0,0,0);q(2,d,acc,0,0);q(3,acc,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::andcf.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_andcf(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let imm=q(8,0,0,0,0)as u16;let v=q(9,d,0,0,0)as u16;q(5,((v&imm)==imm)as i64,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::andf.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_andf(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let imm=q(8,0,0,0,0)as u16;let v=q(9,d,0,0,0)as u16;q(5,((v&imm)==0)as i64,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::tst.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_tst(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>11)&1)as i64;let acc=q(1,d,0,0,0);q(3,acc,0,0,0);q(0,0,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::tstaxh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_tstaxh(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let v=q(10,d,0,0,0)as i16 as i64;q(4,v,0,0,0);q(0,0,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::cmp.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_cmp(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let a=q(1,0,0,0,0);let b=q(1,1,0,0,0);let res=convert(a-b);q(7,a,b,res,0);q(0,0,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::cmpaxh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_cmpaxh(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let r=((opc>>12)&1)as i64;let s=((opc>>11)&1)as i64;let a=q(1,s,0,0,0);let b=q(10,r,0,0,0)<<16;let res=convert(a-b);q(7,a,b,res,0);q(0,0,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::cmpi.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_cmpi(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=(q(8,0,0,0,0)as i16 as i64)<<16;let res=convert(a-b);q(7,a,b,res,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::cmpis.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_cmpis(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=(opc as i8 as i64)<<16;let res=convert(a-b);q(7,a,b,res,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::xorr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_xorr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let m=(q(15,d,0,0,0)as u16)^(q(17,s,0,0,0)as u16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::andr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_andr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let m=(q(15,d,0,0,0)as u16)&(q(17,s,0,0,0)as u16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::orr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_orr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let m=(q(15,d,0,0,0)as u16)|(q(17,s,0,0,0)as u16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::andc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_andc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let m=(q(15,d,0,0,0)as u16)&(q(15,1-d,0,0,0)as u16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::orc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_orc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let m=(q(15,d,0,0,0)as u16)|(q(15,1-d,0,0,0)as u16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::xorc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_xorc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let m=(q(15,d,0,0,0)as u16)^(q(15,1-d,0,0,0)as u16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::notc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_notc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let m=(q(15,d,0,0,0)as u16)^(0xffffu16);q(14,d,0,0,0);q(18,d,m as i64,0,0);let over=over32(q(1,d,0,0,0));q(4,m as i16 as i64,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::xori.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_xori(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let imm=q(8,0,0,0,0)as u16;let m=(q(15,d,0,0,0)as u16)^imm;q(18,d,m as i64,0,0);let m=q(15,d,0,0,0)as i16 as i64;let over=over32(q(1,d,0,0,0));q(4,m,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::andi.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_andi(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let imm=q(8,0,0,0,0)as u16;let m=(q(15,d,0,0,0)as u16)&imm;q(18,d,m as i64,0,0);let m=q(15,d,0,0,0)as i16 as i64;let over=over32(q(1,d,0,0,0));q(4,m,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::ori.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_ori(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let imm=q(8,0,0,0,0)as u16;let m=(q(15,d,0,0,0)as u16)|imm;q(18,d,m as i64,0,0);let m=q(15,d,0,0,0)as i16 as i64;let over=over32(q(1,d,0,0,0));q(4,m,0,0,over as i64);}
// port of pristine DSPIntArithmetic.cpp Interpreter::addr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_addr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&3)as i64;let a=q(1,d,0,0,0);let b=((if s<2{q(16,s,0,0,0)}else{q(17,s-2,0,0,0)})as i16 as i64)<<16;let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::addax.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_addax(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let a=q(1,d,0,0,0);let b=q(12,s,0,0,0);let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::add.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_add(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=q(1,1-d,0,0,0);let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::addp.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_addp(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=q(13,0,0,0,0);let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::addaxl.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_addaxl(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let s=((opc>>9)&1)as i64;let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=q(11,s,0,0,0)as u16 as i64;let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::addi.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_addi(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=(q(8,0,0,0,0)as i16 as i64)<<16;let res=a.wrapping_add(b);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::addis.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_addis(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=(opc as i8 as i64)<<16;let res=a.wrapping_add(b);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::incm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_incm(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=0x10000i64;let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::inc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_inc(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=1i64;let res=a.wrapping_add(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(6,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::subr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_subr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&3)as i64;let a=q(1,d,0,0,0);let b=((if s<2{q(16,s,0,0,0)}else{q(17,s-2,0,0,0)})as i16 as i64)<<16;let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::subax.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_subax(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let a=q(1,d,0,0,0);let b=q(12,s,0,0,0);let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::sub.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_sub(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=q(1,1-d,0,0,0);let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::subp.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_subp(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=q(13,0,0,0,0);let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::decm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_decm(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=0x10000i64;let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::dec.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_dec(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=q(1,d,0,0,0);let b=1i64;let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::neg.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_neg(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let a=0i64;let b=q(1,d,0,0,0);let res=a.wrapping_sub(b);q(0,0,0,0,0);q(2,d,res,0,0);let actual=q(1,d,0,0,0);q(7,a,b,actual,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::abs.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_abs(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>11)&1)as i64;let mut acc=q(1,d,0,0,0);if acc<0{acc=-acc;}q(0,0,0,0,0);q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::movr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_movr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&3)as i64;let ax=((if s<2{q(16,s,0,0,0)}else{q(17,s-2,0,0,0)})as i16 as i64)<<16;q(0,0,0,0,0);q(2,d,ax,0,0);q(3,ax,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::movax.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_movax(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let ax=q(12,s,0,0,0);q(0,0,0,0,0);q(2,d,ax,0,0);q(3,ax,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::mov.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_mov(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let acc=q(1,1-d,0,0,0);q(0,0,0,0,0);q(2,d,acc,0,0);q(3,acc,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsl16.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsl16(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let shift=16u32;let acc=q(1,d,0,0,0);let acc=acc.wrapping_shl(shift);q(0,0,0,0,0);q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsr16.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsr16(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let shift=16u32;let acc=q(1,d,0,0,0);let acc=(((acc as u64)&0xffffffffff)>>shift)as i64;q(0,0,0,0,0);q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::asr16.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_asr16(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>11)&1)as i64;let shift=16u32;let acc=q(1,d,0,0,0);let acc=acc>>shift;q(0,0,0,0,0);q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsl.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsl(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let shift=(opc&0x3f)as u32;let acc=q(1,d,0,0,0);let acc=acc.wrapping_shl(shift);q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let shift=if opc&0x3f==0{0}else{0x40-(opc&0x3f)}as u32;let acc=q(1,d,0,0,0);let acc=(((acc as u64)&0xffffffffff)>>shift)as i64;q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::asl.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_asl(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let shift=(opc&0x3f)as u32;let acc=q(1,d,0,0,0);let acc=acc.wrapping_shl(shift);q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::asr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_asr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let shift=if opc&0x3f==0{0}else{0x40-(opc&0x3f)}as u32;let acc=q(1,d,0,0,0);let acc=acc>>shift;q(2,d,acc,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsrn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsrn(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=0i64;let m=q(9,1,0,0,0)as u16;let mut acc=q(1,d,0,0,0) as u64 &0xffffffffff;let shift=if m&0x3f==0{0}else if m&0x40!=0{-0x40+(m&0x3f)as i32}else{(m&0x3f)as i32};if shift>0{acc>>=shift;}else if shift<0{acc=acc.wrapping_shl((-shift)as u32);}q(2,d,acc as i64,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::asrn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_asrn(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=0i64;let m=q(9,1,0,0,0)as u16;let mut acc=q(1,d,0,0,0);let shift=if m&0x3f==0{0}else if m&0x40!=0{-0x40+(m&0x3f)as i32}else{(m&0x3f)as i32};if shift>0{acc>>=shift;}else if shift<0{acc=acc.wrapping_shl((-shift)as u32);}q(2,d,acc as i64,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsrnrx.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsrnrx(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let m=q(17,s,0,0,0)as u16;let mut acc=q(1,d,0,0,0) as u64 &0xffffffffff;let shift=if m&0x3f==0{0}else if m&0x40!=0{-0x40+(m&0x3f)as i32}else{(m&0x3f)as i32};if shift>0{acc=acc.wrapping_shl(shift as u32);}else if shift<0{acc>>=-shift;}q(0,0,0,0,0);q(2,d,acc as i64,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::asrnrx.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_asrnrx(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let s=((opc>>9)&1)as i64;let m=q(17,s,0,0,0)as u16;let mut acc=q(1,d,0,0,0);let shift=if m&0x3f==0{0}else if m&0x40!=0{-0x40+(m&0x3f)as i32}else{(m&0x3f)as i32};if shift>0{acc=acc.wrapping_shl(shift as u32);}else if shift<0{acc>>=-shift;}q(0,0,0,0,0);q(2,d,acc as i64,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::lsrnr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_lsrnr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let m=q(9,1-d,0,0,0)as u16;let mut acc=q(1,d,0,0,0) as u64 &0xffffffffff;let shift=if m&0x3f==0{0}else if m&0x40!=0{-0x40+(m&0x3f)as i32}else{(m&0x3f)as i32};if shift>0{acc=acc.wrapping_shl(shift as u32);}else if shift<0{acc>>=-shift;}q(0,0,0,0,0);q(2,d,acc as i64,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
// port of pristine DSPIntArithmetic.cpp Interpreter::asrnr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_arith_asrnr(c:*mut c_void,opc:u16,exec:Exec){let q=|op,a,b,d,e|exec(c,op,a,b,d,e);let d=((opc>>8)&1)as i64;let m=q(9,1-d,0,0,0)as u16;let mut acc=q(1,d,0,0,0);let shift=if m&0x3f==0{0}else if m&0x40!=0{-0x40+(m&0x3f)as i32}else{(m&0x3f)as i32};if shift>0{acc=acc.wrapping_shl(shift as u32);}else if shift<0{acc>>=-shift;}q(0,0,0,0,0);q(2,d,acc as i64,0,0);let actual=q(1,d,0,0,0);q(3,actual,0,0,0);}
