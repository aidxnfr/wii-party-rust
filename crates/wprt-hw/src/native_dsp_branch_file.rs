// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted whole DSPIntBranch.cpp member replacement; unchanged policy and explicit other-file helpers.
#[cfg(not(all(target_os="windows",target_env="gnu",target_arch="x86_64")))]
compile_error!("DSPIntBranch member exports require pinned Windows GNU x64 metadata");
pub(crate) mod layout {include!(env!("WPRT_DSP_BRANCH_LAYOUT"));}
const _:()=assert!(layout::STACK_ENUM_SIZE==4 && layout::STACK_ENUM_ALIGN==4);
use std::ffi::c_void;
extern "C" {
#[link_name="_ZNK3DSP11Interpreter11Interpreter14CheckConditionEh"] fn condition(this:*const u8,reg:u8)->bool;
#[link_name="_ZN3DSP11Interpreter11Interpreter14OpReadRegisterEi"] fn read_reg(this:*mut u8,reg:i32)->u16;
#[link_name="_ZN3DSP4SDSP16FetchInstructionEv"] fn fetch(s:*mut u8)->u16;
#[link_name="_ZN3DSP4SDSP15SkipInstructionEv"] fn skip(s:*mut u8);
#[link_name="_ZN3DSP4SDSP10StoreStackENS_13StackRegisterEt"] fn store(s:*mut u8,reg:i32,value:u16);
#[link_name="_ZN3DSP4SDSP8PopStackENS_13StackRegisterE"] fn pop(s:*mut u8,reg:i32)->u16;
#[link_name="_ZSt9terminatev"] fn terminate()->!;
}
unsafe fn state(this:*mut u8)->*mut u8{this.add(layout::INTERPRETER_CORE_OFFSET).cast::<*mut u8>().read().add(layout::CORE_STATE_OFFSET)}
unsafe extern "C" fn exec(raw:*mut c_void,kind:u32,reg:u16,value:u16)->u16{let this=raw.cast();match kind{
0=>condition(this,reg as u8) as u16,1=>read_reg(this,reg as i32),2=>fetch(state(this)),3=>{skip(state(this));0},4=>{store(state(this),reg as i32,value);0},5=>pop(state(this),reg as i32),_=>terminate()}}
#[export_name="_ZN3DSP11Interpreter11Interpreter4callEt"]
pub unsafe extern "C" fn call(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_call(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5callrEt"]
pub unsafe extern "C" fn callr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_callr(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4ifccEt"]
pub unsafe extern "C" fn ifcc(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_ifcc(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3jccEt"]
pub unsafe extern "C" fn jcc(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_jcc(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6jmprccEt"]
pub unsafe extern "C" fn jmprcc(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_jmprcc(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3retEt"]
pub unsafe extern "C" fn ret(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_ret(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3rtiEt"]
pub unsafe extern "C" fn rti(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_rti(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4haltEt"]
pub unsafe extern "C" fn halt(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_halt(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter10HandleLoopEv"]
pub unsafe extern "C" fn HandleLoop(this:*mut u8){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_HandleLoop(this.cast(),0,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4loopEt"]
pub unsafe extern "C" fn op_loop(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_loop(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5loopiEt"]
pub unsafe extern "C" fn loopi(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_loopi(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5bloopEt"]
pub unsafe extern "C" fn bloop(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_bloop(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6bloopiEt"]
pub unsafe extern "C" fn bloopi(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_branch::wprt_dsp_branch_bloopi(this.cast(),opc,s.add(layout::PC_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),s.add(layout::CONTROL_OFFSET).cast(),s.add(layout::ST_OFFSET).cast(),exec);}
