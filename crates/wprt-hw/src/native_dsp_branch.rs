// Copyright 2009 Dolphin Emulator Project
// Additional copyrights Duddie and Tratax(c)2004
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete port of DSPIntBranch.cpp;condition/register/fetch/stack SDK operations remain native interfaces.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,u16,u16)->u16;
// port of DSPIntBranch.cpp Interpreter::call.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_call(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let dest=exec(c,2,0,0);if exec(c,0,opc&15,0)!=0{exec(c,4,0,*pc);*pc=dest;}}
// port of DSPIntBranch.cpp Interpreter::callr.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_callr(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){if exec(c,0,opc&15,0)==0{return}let reg=(opc>>5)&7;let addr=exec(c,1,reg,0);exec(c,4,0,*pc);*pc=addr;}
// port of DSPIntBranch.cpp Interpreter::ifcc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_ifcc(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){if exec(c,0,opc&15,0)!=0{return}exec(c,3,0,0);}
// port of DSPIntBranch.cpp Interpreter::jcc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_jcc(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let dest=exec(c,2,0,0);if exec(c,0,opc&15,0)!=0{*pc=dest;}}
// port of DSPIntBranch.cpp Interpreter::jmprcc.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_jmprcc(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){if exec(c,0,opc&15,0)==0{return}let reg=(opc>>5)&7;*pc=exec(c,1,reg,0);}
// port of DSPIntBranch.cpp Interpreter::ret.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_ret(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){if exec(c,0,opc&15,0)==0{return}*pc=exec(c,5,0,0);}
// port of DSPIntBranch.cpp Interpreter::rti.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_rti(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){if exec(c,0,opc&15,0)==0{return}*sr=exec(c,5,1,0);*pc=exec(c,5,0,0);}
// port of DSPIntBranch.cpp Interpreter::halt.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_halt(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){*control|=4;*pc=(*pc).wrapping_sub(1);}
// port of DSPIntBranch.cpp Interpreter::HandleLoop.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_HandleLoop(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let call=st.read();let address=st.add(2).read();let count=st.add(3);if address>0&&*count>0{if (*pc as i32)-1==address as i32{*count=(*count).wrapping_sub(1);if *count>0{*pc=call;}else{exec(c,5,0,0);exec(c,5,2,0);exec(c,5,3,0);}}}}
// port of DSPIntBranch.cpp Interpreter::loop.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_loop(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let count=exec(c,1,opc&31,0);let loop_pc=*pc;if count!=0{exec(c,4,0,*pc);exec(c,4,2,loop_pc);exec(c,4,3,count);}else{exec(c,3,0,0);}}
// port of DSPIntBranch.cpp Interpreter::loopi.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_loopi(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let count=opc&255;let loop_pc=*pc;if count!=0{exec(c,4,0,*pc);exec(c,4,2,loop_pc);exec(c,4,3,count);}else{exec(c,3,0,0);}}
// port of DSPIntBranch.cpp Interpreter::bloop.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_bloop(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let count=exec(c,1,opc&31,0);let loop_pc=exec(c,2,0,0);if count!=0{exec(c,4,0,*pc);exec(c,4,2,loop_pc);exec(c,4,3,count);}else{*pc=loop_pc;exec(c,3,0,0);}}
// port of DSPIntBranch.cpp Interpreter::bloopi.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_branch_bloopi(c:*mut c_void,opc:u16,pc:*mut u16,sr:*mut u16,control:*mut u16,st:*mut u16,exec:Exec){let count=opc&255;let loop_pc=exec(c,2,0,0);if count!=0{exec(c,4,0,*pc);exec(c,4,2,loop_pc);exec(c,4,3,count);}else{*pc=loop_pc;exec(c,3,0,0);}}
