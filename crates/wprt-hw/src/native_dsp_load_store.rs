// Copyright 2009 Dolphin Emulator Project
// Additional copyrights Duddie and Tratax(c)2004
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete port of pristine DSPIntLoadStore.cpp;native private register/address/accumulator helpers remain SDK.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,u16,u16)->u16;
const DSP_REG_ACH0:u16=16;
const DSP_REG_ACL0:u16=28;
const DSP_REG_ACM0:u16=30;
const DSP_REG_CR:u16=18;
// port of DSPIntLoadStore.cpp Interpreter::srsh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_srsh(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let reg=((opc>>8)&1)+DSP_REG_ACH0;let a=(*cr<<8)|(opc&0xff);let v=exec(c,0,reg,0);exec(c,10,a,v);}
// port of DSPIntLoadStore.cpp Interpreter::srs.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_srs(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let reg=((opc>>8)&3)+DSP_REG_ACL0;let a=(*cr<<8)|(opc&0xff);let v=exec(c,0,reg,0);exec(c,10,a,v);}
// port of DSPIntLoadStore.cpp Interpreter::lrs.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_lrs(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let reg=((opc>>8)&7)+0x18;let a=(*cr<<8)|(opc&0xff);let v=exec(c,9,a,0);exec(c,1,reg,v);exec(c,2,reg,0);}
// port of DSPIntLoadStore.cpp Interpreter::lr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_lr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let reg=opc&0x1f;let a=exec(c,8,0,0);let v=exec(c,9,a,0);exec(c,1,reg,v);exec(c,2,reg,0);}
// port of DSPIntLoadStore.cpp Interpreter::sr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_sr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let reg=opc&0x1f;let a=exec(c,8,0,0);let v=exec(c,0,reg,0);exec(c,10,a,v);}
// port of DSPIntLoadStore.cpp Interpreter::si.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_si(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let a=opc as i8 as i16 as u16;let imm=exec(c,8,0,0);exec(c,10,a,imm);}
// port of DSPIntLoadStore.cpp Interpreter::lrr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_lrr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=(opc>>5)&3;let d=opc&0x1f;let a=exec(c,0,s,0);let v=exec(c,9,a,0);exec(c,1,d,v);exec(c,2,d,0);}
// port of DSPIntLoadStore.cpp Interpreter::lrrd.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_lrrd(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=(opc>>5)&3;let d=opc&0x1f;let a=exec(c,0,s,0);let v=exec(c,9,a,0);exec(c,1,d,v);exec(c,2,d,0);let v=exec(c,4,s,0);ar.add(s as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::lrri.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_lrri(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=(opc>>5)&3;let d=opc&0x1f;let a=exec(c,0,s,0);let v=exec(c,9,a,0);exec(c,1,d,v);exec(c,2,d,0);let v=exec(c,5,s,0);ar.add(s as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::lrrn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_lrrn(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=(opc>>5)&3;let d=opc&0x1f;let a=exec(c,0,s,0);let v=exec(c,9,a,0);exec(c,1,d,v);exec(c,2,d,0);let step=ix.add(s as usize).read();let v=exec(c,7,s,step);ar.add(s as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::srr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_srr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let d=(opc>>5)&3;let s=opc&0x1f;let v=exec(c,0,s,0);let a=ar.add(d as usize).read();exec(c,10,a,v);}
// port of DSPIntLoadStore.cpp Interpreter::srrd.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_srrd(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let d=(opc>>5)&3;let s=opc&0x1f;let v=exec(c,0,s,0);let a=ar.add(d as usize).read();exec(c,10,a,v);let v=exec(c,4,d,0);ar.add(d as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::srri.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_srri(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let d=(opc>>5)&3;let s=opc&0x1f;let v=exec(c,0,s,0);let a=ar.add(d as usize).read();exec(c,10,a,v);let v=exec(c,5,d,0);ar.add(d as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::srrn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_srrn(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let d=(opc>>5)&3;let s=opc&0x1f;let v=exec(c,0,s,0);let a=ar.add(d as usize).read();exec(c,10,a,v);let step=ix.add(d as usize).read();let v=exec(c,7,d,step);ar.add(d as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::ilrr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_ilrr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=opc&3;let d=DSP_REG_ACM0+((opc>>8)&1);let a=ar.add(s as usize).read();let v=exec(c,11,a,0);exec(c,12,d-DSP_REG_ACM0,v);exec(c,2,d,0);}
// port of DSPIntLoadStore.cpp Interpreter::ilrrd.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_ilrrd(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=opc&3;let d=DSP_REG_ACM0+((opc>>8)&1);let a=ar.add(s as usize).read();let v=exec(c,11,a,0);exec(c,12,d-DSP_REG_ACM0,v);exec(c,2,d,0);let v=exec(c,4,s,0);ar.add(s as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::ilrri.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_ilrri(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=opc&3;let d=DSP_REG_ACM0+((opc>>8)&1);let a=ar.add(s as usize).read();let v=exec(c,11,a,0);exec(c,12,d-DSP_REG_ACM0,v);exec(c,2,d,0);let v=exec(c,5,s,0);ar.add(s as usize).write(v);}
// port of DSPIntLoadStore.cpp Interpreter::ilrrn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ls_ilrrn(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,cr:*const u16,exec:Exec){let s=opc&3;let d=DSP_REG_ACM0+((opc>>8)&1);let a=ar.add(s as usize).read();let v=exec(c,11,a,0);exec(c,12,d-DSP_REG_ACM0,v);exec(c,2,d,0);let step=ix.add(s as usize).read();let v=exec(c,7,s,step);ar.add(s as usize).write(v);}
