// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPIntExtOps.cpp port;typed private helpers/backlog remain SDK.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,u16,u16,u16)->u16;
const DSP_REG_AR0:u16=0;
const DSP_REG_AR3:u16=3;
const DSP_REG_AXL0:u16=24;
const DSP_REG_AXL1:u16=25;
const DSP_REG_AXH0:u16=26;
const DSP_REG_ACM0:u16=30;
const DSP_REG_ACH0:u16=16;
const DSP_REG_ACL0:u16=28;
const SR_40_MODE_BIT:u16=16384;
// port of pristine DSPIntExtOps.cpp Interpreter::dr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_dr(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let r=opc&3;let v=q(0,r,0,0);q(6,0,r,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ir.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ir(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let r=opc&3;let v=q(1,r,0,0);q(6,0,r,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::nr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_nr(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let r=opc&3;let v=q(2,r,*ix.add(r as usize),0);q(6,0,r,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::mv.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_mv(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&3)+DSP_REG_ACL0;let d=(opc>>2)&3;let v=q(3,s,0,0);q(6,0,d+DSP_REG_AXL0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::s.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_s(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let d=opc&3;let s=((opc>>3)&3)+DSP_REG_ACL0;let v=q(3,s,0,0);q(5,*ar.add(d as usize),v,0);let next=q(1,d,0,0);q(6,0,d,next);}
// port of pristine DSPIntExtOps.cpp Interpreter::sn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_sn(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let d=opc&3;let s=((opc>>3)&3)+DSP_REG_ACL0;let v=q(3,s,0,0);q(5,*ar.add(d as usize),v,0);let next=q(2,d,*ix.add(d as usize),0);q(6,0,d,next);}
// port of pristine DSPIntExtOps.cpp Interpreter::l.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_l(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=opc&3;let d=((opc>>3)&7)+DSP_REG_AXL0;if d>=DSP_REG_ACM0 && q(7,SR_40_MODE_BIT,0,0)!=0{let v=q(4,*ar.add(s as usize),0,0);q(6,0,d-DSP_REG_ACM0+DSP_REG_ACH0,if v&0x8000!=0{0xffff}else{0});q(6,1,d,v);q(6,2,d-DSP_REG_ACM0+DSP_REG_ACL0,0);let next=q(1,s,0,0);q(6,3,s,next);}else{let v=q(4,*ar.add(s as usize),0,0);q(6,0,d,v);let next=q(1,s,0,0);q(6,1,s,next);}}
// port of pristine DSPIntExtOps.cpp Interpreter::ln.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ln(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=opc&3;let d=((opc>>3)&7)+DSP_REG_AXL0;if d>=DSP_REG_ACM0 && q(7,SR_40_MODE_BIT,0,0)!=0{let v=q(4,*ar.add(s as usize),0,0);q(6,0,d-DSP_REG_ACM0+DSP_REG_ACH0,if v&0x8000!=0{0xffff}else{0});q(6,1,d,v);q(6,2,d-DSP_REG_ACM0+DSP_REG_ACL0,0);let next=q(2,s,*ix.add(s as usize),0);q(6,3,s,next);}else{let v=q(4,*ar.add(s as usize),0,0);q(6,0,d,v);let next=q(2,s,*ix.add(s as usize),0);q(6,1,s,next);}}
// port of pristine DSPIntExtOps.cpp Interpreter::ls.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ls(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(3),v,0);let v=q(4,*ar.add(0),0,0);q(6,0,d,v);let v=q(1,3,0,0);q(6,1,DSP_REG_AR3,v);let v=q(1,0,0,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::lsn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_lsn(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(3),v,0);let v=q(4,*ar.add(0),0,0);q(6,0,d,v);let v=q(1,3,0,0);q(6,1,DSP_REG_AR3,v);let v=q(2,0,*ix,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::lsm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_lsm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(3),v,0);let v=q(4,*ar.add(0),0,0);q(6,0,d,v);let v=q(2,3,*ix.add(3),0);q(6,1,DSP_REG_AR3,v);let v=q(1,0,0,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::lsnm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_lsnm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(3),v,0);let v=q(4,*ar.add(0),0,0);q(6,0,d,v);let v=q(2,3,*ix.add(3),0);q(6,1,DSP_REG_AR3,v);let v=q(2,0,*ix,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::sl.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_sl(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(0),v,0);let v=q(4,*ar.add(3),0,0);q(6,0,d,v);let v=q(1,3,0,0);q(6,1,DSP_REG_AR3,v);let v=q(1,0,0,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::sln.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_sln(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(0),v,0);let v=q(4,*ar.add(3),0,0);q(6,0,d,v);let v=q(1,3,0,0);q(6,1,DSP_REG_AR3,v);let v=q(2,0,*ix,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::slm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_slm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(0),v,0);let v=q(4,*ar.add(3),0,0);q(6,0,d,v);let v=q(2,3,*ix.add(3),0);q(6,1,DSP_REG_AR3,v);let v=q(1,0,0,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::slnm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_slnm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc&1)+DSP_REG_ACM0;let d=((opc>>4)&3)+DSP_REG_AXL0;let v=q(3,s,0,0);q(5,*ar.add(0),v,0);let v=q(4,*ar.add(3),0,0);q(6,0,d,v);let v=q(2,3,*ix.add(3),0);q(6,1,DSP_REG_AR3,v);let v=q(2,0,*ix,0);q(6,2,DSP_REG_AR0,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ld.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ld(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let d=(opc>>5)&1;let r=(opc>>4)&1;let s=opc&3;let v=q(4,*ar.add(s as usize),0,0);q(6,0,(d<<1)+DSP_REG_AXL0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,(r<<1)+DSP_REG_AXL1,v);let v=q(1,s,0,0);q(6,2,s,v);let v=q(1,3,0,0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldax.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldax(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc>>5)&1;let r=(opc>>4)&1;let v=q(4,*ar.add(s as usize),0,0);q(6,0,r+DSP_REG_AXH0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,r+DSP_REG_AXL0,v);let v=q(1,s,0,0);q(6,2,s,v);let v=q(1,3,0,0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldn(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let d=(opc>>5)&1;let r=(opc>>4)&1;let s=opc&3;let v=q(4,*ar.add(s as usize),0,0);q(6,0,(d<<1)+DSP_REG_AXL0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,(r<<1)+DSP_REG_AXL1,v);let v=q(2,s,*ix.add(s as usize),0);q(6,2,s,v);let v=q(1,3,0,0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldaxn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldaxn(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc>>5)&1;let r=(opc>>4)&1;let v=q(4,*ar.add(s as usize),0,0);q(6,0,r+DSP_REG_AXH0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,r+DSP_REG_AXL0,v);let v=q(2,s,*ix.add(s as usize),0);q(6,2,s,v);let v=q(1,3,0,0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let d=(opc>>5)&1;let r=(opc>>4)&1;let s=opc&3;let v=q(4,*ar.add(s as usize),0,0);q(6,0,(d<<1)+DSP_REG_AXL0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,(r<<1)+DSP_REG_AXL1,v);let v=q(1,s,0,0);q(6,2,s,v);let v=q(2,3,*ix.add(3),0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldaxm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldaxm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc>>5)&1;let r=(opc>>4)&1;let v=q(4,*ar.add(s as usize),0,0);q(6,0,r+DSP_REG_AXH0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,r+DSP_REG_AXL0,v);let v=q(1,s,0,0);q(6,2,s,v);let v=q(2,3,*ix.add(3),0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldnm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldnm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let d=(opc>>5)&1;let r=(opc>>4)&1;let s=opc&3;let v=q(4,*ar.add(s as usize),0,0);q(6,0,(d<<1)+DSP_REG_AXL0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,(r<<1)+DSP_REG_AXL1,v);let v=q(2,s,*ix.add(s as usize),0);q(6,2,s,v);let v=q(2,3,*ix.add(3),0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::ldaxnm.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_ldaxnm(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);let s=(opc>>5)&1;let r=(opc>>4)&1;let v=q(4,*ar.add(s as usize),0,0);q(6,0,r+DSP_REG_AXH0,v);let address=if (*ar.add(s as usize)>>10)==(*ar.add(3)>>10){*ar.add(s as usize)}else{*ar.add(3)};let v=q(4,address,0,0);q(6,1,r+DSP_REG_AXL0,v);let v=q(2,s,*ix.add(s as usize),0);q(6,2,s,v);let v=q(2,3,*ix.add(3),0);q(6,3,DSP_REG_AR3,v);}
// port of pristine DSPIntExtOps.cpp Interpreter::nop_ext.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ext_nop_ext(c:*mut c_void,opc:u16,ar:*const u16,ix:*const u16,exec:Exec){let q=|op,a,b,d|exec(c,op,a,b,d);}
