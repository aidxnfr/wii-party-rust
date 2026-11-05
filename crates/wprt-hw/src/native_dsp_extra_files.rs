// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// Candidate: existing68member differential byte-exact; quick live gate pending. Whole DSPIntLoadStore/Multiplier/ExtOps member wrappers.
// Existing accepted opcode policies unchanged; other-file helper calls remain explicit.
use std::ffi::c_void;
pub(crate) mod layout {include!(env!("WPRT_DSP_EXTRA_LAYOUT"));}
unsafe fn state(this:*mut u8)->*mut u8{
 this.add(crate::native_dsp_misc_file::layout::INTERPRETER_CORE_OFFSET).cast::<*mut u8>().read()
 .add(crate::native_dsp_misc_file::layout::CORE_STATE_OFFSET)
}
extern "C" {
 #[link_name="_ZN3DSP11Interpreter11Interpreter14OpReadRegisterEi"] fn read_reg(t:*mut u8,r:i32)->u16;
 #[link_name="_ZN3DSP11Interpreter11Interpreter15OpWriteRegisterEit"] fn write_reg(t:*mut u8,r:i32,v:u16);
 #[link_name="_ZN3DSP11Interpreter11Interpreter22ConditionalExtendAccumEi"] fn extend(t:*mut u8,r:i32);
 #[link_name="_ZNK3DSP11Interpreter11Interpreter24DecrementAddressRegisterEt"] fn dec(t:*const u8,r:u16)->u16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter24IncrementAddressRegisterEt"] fn inc(t:*const u8,r:u16)->u16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter23IncreaseAddressRegisterEts"] fn increase(t:*const u8,r:u16,v:i16)->u16;
 #[link_name="_ZN3DSP4SDSP16FetchInstructionEv"] fn fetch(s:*mut u8)->u16;
 #[link_name="_ZN3DSP4SDSP8ReadDMEMEt"] fn read_dmem(s:*mut u8,a:u16)->u16;
 #[link_name="_ZN3DSP4SDSP9WriteDMEMEtt"] fn write_dmem(s:*mut u8,a:u16,v:u16);
 #[link_name="_ZNK3DSP4SDSP8ReadIMEMEt"] fn read_imem(s:*const u8,a:u16)->u16;
 #[link_name="_ZN3DSP11Interpreter11Interpreter14WriteToBackLogEiit"] fn backlog(t:*mut u8,i:i32,idx:i32,v:u16);
 #[link_name="_ZNK3DSP11Interpreter11Interpreter11IsSRFlagSetEt"] fn flag(t:*const u8,flag:u16)->bool;
 #[link_name="_ZN3DSP11Interpreter11Interpreter16ZeroWriteBackLogEv"] fn zero(t:*mut u8);
 #[link_name="_ZNK3DSP11Interpreter11Interpreter14GetLongProductEv"] fn product(t:*const u8)->i64;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter21GetLongProductRoundedEv"] fn rounded(t:*const u8)->i64;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter10GetLongAccEi"] fn acc(t:*const u8,r:i32)->i64;
 #[link_name="_ZN3DSP11Interpreter11Interpreter10SetLongAccEix"] fn set_acc(t:*mut u8,r:i32,v:i64);
 #[link_name="_ZNK3DSP11Interpreter11Interpreter10GetLongACXEi"] fn acx(t:*const u8,r:i32)->i32;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter8GetAXLowEi"] fn ax_low(t:*const u8,r:i32)->i16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter9GetAXHighEi"] fn ax_high(t:*const u8,r:i32)->i16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter9GetAccMidEi"] fn acc_mid(t:*const u8,r:i32)->i16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter8MultiplyEtth"] fn multiply(t:*const u8,a:u16,b:u16,sign:u8)->i64;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter12MultiplyMulXEhhtt"] fn mulx(t:*const u8,a:u8,b:u8,d:u16,e:u16)->i64;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter11MultiplyAddEtth"] fn mul_add(t:*const u8,a:u16,b:u16,sign:u8)->i64;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter11MultiplySubEtth"] fn mul_sub(t:*const u8,a:u16,b:u16,sign:u8)->i64;
 #[link_name="_ZN3DSP11Interpreter11Interpreter14SetLongProductEx"] fn set_product(t:*mut u8,v:i64);
 #[link_name="_ZN3DSP11Interpreter11Interpreter10UpdateSR64Exbb"] fn sr64(t:*mut u8,v:i64,c:bool,o:bool);
 #[link_name="_ZSt9terminatev"] fn terminate()->!;
}
unsafe extern "C" fn ls_exec(raw:*mut c_void,op:u32,reg:u16,value:u16)->u16{
 let t=raw.cast();match op{
 0=>read_reg(t,reg as i32),1=>{write_reg(t,reg as i32,value);0},2=>{extend(t,reg as i32);0},
 4=>dec(t,reg),5=>inc(t,reg),7=>increase(t,reg,value as i16),8=>fetch(state(t)),
 9=>read_dmem(state(t),reg),10=>{write_dmem(state(t),reg,value);0},11=>read_imem(state(t),reg),
 12=>{state(t).add(layout::AC_M_OFFSET+reg as usize*layout::AC_STRIDE).cast::<u16>().write(value);0},
 _=>terminate()
 }
}
unsafe extern "C" fn ext_exec(raw:*mut c_void,op:u32,a:u16,b:u16,d:u16)->u16{
 let t=raw.cast();match op{
 0=>dec(t,a),1=>inc(t,a),2=>increase(t,a,b as i16),3=>read_reg(t,a as i32),
 4=>read_dmem(state(t),a),5=>{write_dmem(state(t),a,b);0},
 6=>{backlog(t,a as i32,b as i32,d);0},7=>flag(t,a) as u16,_=>terminate()
 }
}
unsafe extern "C" fn mul_exec(raw:*mut c_void,op:u32,a:i64,b:i64,d:i64,e:i64)->i64{
 let t=raw.cast();match op{
 0=>{zero(t);0},1=>product(t),2=>rounded(t),3=>acc(t,a as i32),4=>{set_acc(t,a as i32,b);0},
 5=>acx(t,a as i32) as i64,6=>ax_low(t,a as i32) as i64,7=>ax_high(t,a as i32) as i64,
 8=>acc_mid(t,a as i32) as i64,9=>multiply(t,a as u16,b as u16,0),
 10=>mulx(t,a as u8,b as u8,d as u16,e as u16),11=>mul_add(t,a as u16,b as u16,0),
 12=>mul_sub(t,a as u16,b as u16,0),13=>{set_product(t,a);0},14=>{sr64(t,a,b!=0,d!=0);0},
 15=>{let off=match a{0=>layout::PROD_L_OFFSET,1=>layout::PROD_M_OFFSET,2=>layout::PROD_H_OFFSET,3=>layout::PROD_M2_OFFSET,_=>terminate()};state(t).add(off).cast::<u16>().write(b as u16);0},
 _=>terminate()
 }
}
pub mod load_store { use super::*;
#[export_name="_ZN3DSP11Interpreter11Interpreter4srshEt"] pub unsafe extern "C" fn srsh(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_srsh(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3srsEt"] pub unsafe extern "C" fn srs(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_srs(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lrsEt"] pub unsafe extern "C" fn lrs(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_lrs(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2lrEt"] pub unsafe extern "C" fn lr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_lr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2srEt"] pub unsafe extern "C" fn sr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_sr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2siEt"] pub unsafe extern "C" fn si(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_si(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lrrEt"] pub unsafe extern "C" fn lrr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_lrr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4lrrdEt"] pub unsafe extern "C" fn lrrd(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_lrrd(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4lrriEt"] pub unsafe extern "C" fn lrri(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_lrri(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4lrrnEt"] pub unsafe extern "C" fn lrrn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_lrrn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3srrEt"] pub unsafe extern "C" fn srr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_srr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4srrdEt"] pub unsafe extern "C" fn srrd(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_srrd(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4srriEt"] pub unsafe extern "C" fn srri(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_srri(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4srrnEt"] pub unsafe extern "C" fn srrn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_srrn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4ilrrEt"] pub unsafe extern "C" fn ilrr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_ilrr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5ilrrdEt"] pub unsafe extern "C" fn ilrrd(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_ilrrd(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5ilrriEt"] pub unsafe extern "C" fn ilrri(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_ilrri(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5ilrrnEt"] pub unsafe extern "C" fn ilrrn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_load_store::wprt_dsp_ls_ilrrn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),s.add(layout::CR_OFFSET).cast(),ls_exec);}
}
pub mod multiplier { use super::*;
#[export_name="_ZN3DSP11Interpreter11Interpreter4clrpEt"] pub unsafe extern "C" fn clrp(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_clrp(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter7tstprodEt"] pub unsafe extern "C" fn tstprod(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_tstprod(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4movpEt"] pub unsafe extern "C" fn movp(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_movp(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5movnpEt"] pub unsafe extern "C" fn movnp(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_movnp(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5movpzEt"] pub unsafe extern "C" fn movpz(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_movpz(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter7addpaxzEt"] pub unsafe extern "C" fn addpaxz(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_addpaxz(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6mulaxhEt"] pub unsafe extern "C" fn mulaxh(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulaxh(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3mulEt"] pub unsafe extern "C" fn mul(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mul(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5mulacEt"] pub unsafe extern "C" fn mulac(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulac(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5mulmvEt"] pub unsafe extern "C" fn mulmv(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulmv(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6mulmvzEt"] pub unsafe extern "C" fn mulmvz(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulmvz(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4mulxEt"] pub unsafe extern "C" fn mulx(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulx(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6mulxacEt"] pub unsafe extern "C" fn mulxac(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulxac(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6mulxmvEt"] pub unsafe extern "C" fn mulxmv(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulxmv(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter7mulxmvzEt"] pub unsafe extern "C" fn mulxmvz(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulxmvz(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4mulcEt"] pub unsafe extern "C" fn mulc(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulc(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6mulcacEt"] pub unsafe extern "C" fn mulcac(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulcac(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6mulcmvEt"] pub unsafe extern "C" fn mulcmv(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulcmv(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter7mulcmvzEt"] pub unsafe extern "C" fn mulcmvz(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_mulcmvz(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5maddxEt"] pub unsafe extern "C" fn maddx(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_maddx(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5msubxEt"] pub unsafe extern "C" fn msubx(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_msubx(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5maddcEt"] pub unsafe extern "C" fn maddc(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_maddc(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5msubcEt"] pub unsafe extern "C" fn msubc(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_msubc(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4maddEt"] pub unsafe extern "C" fn madd(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_madd(this.cast(),opc,mul_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4msubEt"] pub unsafe extern "C" fn msub(this:*mut u8,opc:u16){crate::native_dsp_multiplier::wprt_dsp_mul_msub(this.cast(),opc,mul_exec);}
}
pub mod ext_ops { use super::*;
#[export_name="_ZN3DSP11Interpreter11Interpreter2drEt"] pub unsafe extern "C" fn dr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_dr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2irEt"] pub unsafe extern "C" fn ir(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ir(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2nrEt"] pub unsafe extern "C" fn nr(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_nr(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2mvEt"] pub unsafe extern "C" fn mv(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_mv(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter1sEt"] pub unsafe extern "C" fn s(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_s(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2snEt"] pub unsafe extern "C" fn sn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_sn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter1lEt"] pub unsafe extern "C" fn l(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_l(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2lnEt"] pub unsafe extern "C" fn ln(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ln(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2lsEt"] pub unsafe extern "C" fn ls(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ls(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lsnEt"] pub unsafe extern "C" fn lsn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_lsn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lsmEt"] pub unsafe extern "C" fn lsm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_lsm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4lsnmEt"] pub unsafe extern "C" fn lsnm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_lsnm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2slEt"] pub unsafe extern "C" fn sl(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_sl(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3slnEt"] pub unsafe extern "C" fn sln(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_sln(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3slmEt"] pub unsafe extern "C" fn slm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_slm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4slnmEt"] pub unsafe extern "C" fn slnm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_slnm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter2ldEt"] pub unsafe extern "C" fn ld(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ld(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4ldaxEt"] pub unsafe extern "C" fn ldax(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldax(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3ldnEt"] pub unsafe extern "C" fn ldn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5ldaxnEt"] pub unsafe extern "C" fn ldaxn(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldaxn(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3ldmEt"] pub unsafe extern "C" fn ldm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5ldaxmEt"] pub unsafe extern "C" fn ldaxm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldaxm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4ldnmEt"] pub unsafe extern "C" fn ldnm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldnm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6ldaxnmEt"] pub unsafe extern "C" fn ldaxnm(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_ldaxnm(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter7nop_extEt"] pub unsafe extern "C" fn nop_ext(this:*mut u8,opc:u16){let s=state(this);crate::native_dsp_ext_ops::wprt_dsp_ext_nop_ext(this.cast(),opc,s.add(layout::AR_OFFSET).cast(),s.add(layout::IX_OFFSET).cast(),ext_exec);}
}
