// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPIntMisc.cpp member replacement; other-file helpers retained explicitly.
#[cfg(not(all(target_os="windows",target_env="gnu",target_arch="x86_64")))]
compile_error!("DSPIntMisc member exports require pinned Windows GNU x64 metadata");
pub(crate) mod layout {include!(env!("WPRT_DSP_MISC_LAYOUT"));}
use std::ffi::c_void;
extern "C" {
#[link_name="_ZN3DSP11Interpreter11Interpreter14OpReadRegisterEi"] fn read_reg(this:*mut u8,reg:i32)->u16;
#[link_name="_ZN3DSP11Interpreter11Interpreter15OpWriteRegisterEit"] fn write_reg(this:*mut u8,reg:i32,value:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter22ConditionalExtendAccumEi"] fn extend(this:*mut u8,reg:i32);
#[link_name="_ZN3DSP11Interpreter11Interpreter16ZeroWriteBackLogEv"] fn zero_wb(this:*mut u8);
#[link_name="_ZNK3DSP11Interpreter11Interpreter24DecrementAddressRegisterEt"] fn decrement(this:*const u8,reg:u16)->u16;
#[link_name="_ZNK3DSP11Interpreter11Interpreter24IncrementAddressRegisterEt"] fn increment(this:*const u8,reg:u16)->u16;
#[link_name="_ZNK3DSP11Interpreter11Interpreter23DecreaseAddressRegisterEts"] fn decrease(this:*const u8,reg:u16,value:i16)->u16;
#[link_name="_ZNK3DSP11Interpreter11Interpreter23IncreaseAddressRegisterEts"] fn increase(this:*const u8,reg:u16,value:i16)->u16;
#[link_name="_ZN3DSP4SDSP16FetchInstructionEv"] fn fetch(state:*mut u8)->u16;
#[link_name="_ZSt9terminatev"] fn terminate()->!;
}
unsafe fn state(this:*mut u8)->*mut u8{this.add(layout::INTERPRETER_CORE_OFFSET).cast::<*mut u8>().read().add(layout::CORE_STATE_OFFSET)}
unsafe extern "C" fn exec(raw:*mut c_void,kind:u32,reg:u16,value:u16)->u16{let this=raw.cast();match kind{
0=>read_reg(this,reg as i32),1=>{write_reg(this,reg as i32,value);0},2=>{extend(this,reg as i32);0},3=>{zero_wb(this);0},4=>decrement(this,reg),5=>increment(this,reg),6=>decrease(this,reg,value as i16),7=>increase(this,reg,value as i16),8=>fetch(state(this)),_=>terminate()}}
#[export_name="_ZN3DSP11Interpreter11Interpreter3mrrEt"]
pub unsafe extern "C" fn mrr(this:*mut u8,opc:u16){crate::native_dsp_misc::wprt_dsp_misc_mrr(this.cast(),opc,std::ptr::null_mut(),std::ptr::null(),std::ptr::null_mut(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lriEt"]
pub unsafe extern "C" fn lri(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_lri(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4lrisEt"]
pub unsafe extern "C" fn lris(this:*mut u8,opc:u16){crate::native_dsp_misc::wprt_dsp_misc_lris(this.cast(),opc,std::ptr::null_mut(),std::ptr::null(),std::ptr::null_mut(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2nxEt"]
pub unsafe extern "C" fn nx(this:*mut u8,opc:u16){crate::native_dsp_misc::wprt_dsp_misc_nx(this.cast(),opc,std::ptr::null_mut(),std::ptr::null(),std::ptr::null_mut(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3darEt"]
pub unsafe extern "C" fn dar(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_dar(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3iarEt"]
pub unsafe extern "C" fn iar(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_iar(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6subarnEt"]
pub unsafe extern "C" fn subarn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_subarn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6addarnEt"]
pub unsafe extern "C" fn addarn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_addarn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5sbclrEt"]
pub unsafe extern "C" fn sbclr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_sbclr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5sbsetEt"]
pub unsafe extern "C" fn sbset(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_sbset(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6srbithEt"]
pub unsafe extern "C" fn srbith(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_misc::wprt_dsp_misc_srbith(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::SR_OFFSET).cast(),exec);}
