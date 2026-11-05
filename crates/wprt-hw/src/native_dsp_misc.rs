// Copyright 2009 Dolphin Emulator Project
// Additional copyrights Duddie and Tratax(c)2004
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted whole-file port of pristine Core/DSP/Interpreter/DSPIntMisc.cpp.
// Native private register/address/accumulator/writeback helpers remain explicit SDK operations.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,u16,u16)->u16;
const DSP_REG_AXL0:u16=24;
const SR_MUL_MODIFY:u16=8192;
const SR_MUL_UNSIGNED:u16=32768;
const SR_40_MODE_BIT:u16=16384;
// port of DSPIntMisc.cpp Interpreter::mrr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_mrr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let sreg=opc&0x1f;let dreg=(opc>>5)&0x1f;let v=exec(c,0,sreg,0);exec(c,1,dreg,v);exec(c,2,dreg,0);}
// port of DSPIntMisc.cpp Interpreter::lri.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_lri(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let reg=opc&0x1f;let imm=exec(c,8,0,0);exec(c,1,reg,imm);exec(c,2,reg,0);}
// port of DSPIntMisc.cpp Interpreter::lris.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_lris(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let reg=((opc>>8)&7)+DSP_REG_AXL0;let imm=opc as i8 as i16 as u16;exec(c,1,reg,imm);exec(c,2,reg,0);}
// port of DSPIntMisc.cpp Interpreter::nx.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_nx(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){exec(c,3,0,0);}
// port of DSPIntMisc.cpp Interpreter::dar.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_dar(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let i=opc&3;let v=exec(c,4,i,0);ar.add(i as usize).write(v);}
// port of DSPIntMisc.cpp Interpreter::iar.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_iar(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let i=opc&3;let v=exec(c,5,i,0);ar.add(i as usize).write(v);}
// port of DSPIntMisc.cpp Interpreter::subarn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_subarn(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let d=opc&3;let v=exec(c,6,d,ix.add(d as usize).read());ar.add(d as usize).write(v);}
// port of DSPIntMisc.cpp Interpreter::addarn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_addarn(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let d=opc&3;let s=(opc>>2)&3;let v=exec(c,7,d,ix.add(s as usize).read());ar.add(d as usize).write(v);}
// port of DSPIntMisc.cpp Interpreter::sbclr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_sbclr(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let bit=(opc&7)+6;*sr&=!(1u16<<bit);}
// port of DSPIntMisc.cpp Interpreter::sbset.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_sbset(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){let bit=(opc&7)+6;*sr|=1u16<<bit;}
// port of DSPIntMisc.cpp Interpreter::srbith.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_misc_srbith(c:*mut c_void,opc:u16,ar:*mut u16,ix:*const u16,sr:*mut u16,exec:Exec){exec(c,3,0,0);match(opc>>8)&7{2=>*sr&=!SR_MUL_MODIFY,3=>*sr|=SR_MUL_MODIFY,4=>*sr&=!SR_MUL_UNSIGNED,5=>*sr|=SR_MUL_UNSIGNED,6=>*sr&=!SR_40_MODE_BIT,7=>*sr|=SR_40_MODE_BIT,_=>()};}
