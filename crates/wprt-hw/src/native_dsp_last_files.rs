// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// G60 batch3 whole-file port; existing member differentials exact; runtime gate pending. whole Arithmetic + Analyzer + DSPIntTables SDK members.
// Existing accepted Rust policies unchanged; native other-file helpers explicitly retained.
use std::ffi::c_void;
pub(crate) mod layout{include!(env!("WPRT_DSP_BATCH3_LAYOUT"));}
unsafe fn state(t:*mut u8)->*mut u8{t.add(crate::native_dsp_misc_file::layout::INTERPRETER_CORE_OFFSET).cast::<*mut u8>().read().add(crate::native_dsp_misc_file::layout::CORE_STATE_OFFSET)}
extern "C" {
 #[link_name="_ZN3DSP11Interpreter11Interpreter16ZeroWriteBackLogEv"]fn zero(t:*mut u8);
 #[link_name="_ZNK3DSP11Interpreter11Interpreter10GetLongAccEi"]fn acc(t:*const u8,r:i32)->i64;
 #[link_name="_ZN3DSP11Interpreter11Interpreter10SetLongAccEix"]fn set_acc(t:*mut u8,r:i32,v:i64);
 #[link_name="_ZN3DSP11Interpreter11Interpreter10UpdateSR64Exbb"]fn sr64(t:*mut u8,v:i64,c:bool,o:bool);
 #[link_name="_ZN3DSP11Interpreter11Interpreter10UpdateSR16Esbbb"]fn sr16(t:*mut u8,v:i16,c:bool,o:bool,over:bool);
 #[link_name="_ZN3DSP11Interpreter11Interpreter17UpdateSRLogicZeroEb"]fn logic_zero(t:*mut u8,v:bool);
 #[link_name="_ZN3DSP11Interpreter11Interpreter13UpdateSR64AddExxx"]fn sr_add(t:*mut u8,a:i64,b:i64,d:i64);
 #[link_name="_ZN3DSP11Interpreter11Interpreter13UpdateSR64SubExxx"]fn sr_sub(t:*mut u8,a:i64,b:i64,d:i64);
 #[link_name="_ZN3DSP4SDSP16FetchInstructionEv"]fn fetch(s:*mut u8)->u16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter9GetAccMidEi"]fn acc_mid(t:*const u8,r:i32)->i16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter9GetAXHighEi"]fn ax_high(t:*const u8,r:i32)->i16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter8GetAXLowEi"]fn ax_low(t:*const u8,r:i32)->i16;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter10GetLongACXEi"]fn acx(t:*const u8,r:i32)->i32;
 #[link_name="_ZNK3DSP11Interpreter11Interpreter14GetLongProductEv"]fn product(t:*const u8)->i64;
 #[link_name="_ZN3DSP11Interpreter11Interpreter27ZeroWriteBackLogPreserveAccEh"]fn preserve(t:*mut u8,r:u8);
 #[link_name="_ZNK3DSP4SDSP8ReadIMEMEt"]fn read_imem(s:*const u8,a:u16)->u16;
 #[link_name="_ZN3DSP13GetOpTemplateEt"]fn get_template(a:u16)->*const u8;
 #[link_name="_ZSt9terminatev"]fn terminate()->!;
}
unsafe extern "C" fn arith_exec(raw:*mut c_void,op:u32,a:i64,b:i64,d:i64,e:i64)->i64{
 let t=raw.cast();match op{
 0=>{zero(t);0},1=>acc(t,a as i32),2=>{set_acc(t,a as i32,b);0},3=>{sr64(t,a,b!=0,d!=0);0},
 4=>{sr16(t,a as i16,b!=0,d!=0,e!=0);0},5=>{logic_zero(t,a!=0);0},6=>{sr_add(t,a,b,d);0},
 7=>{sr_sub(t,a,b,d);0},8=>fetch(state(t)) as i64,9=>acc_mid(t,a as i32) as i64,
 10=>ax_high(t,a as i32) as i64,11=>ax_low(t,a as i32) as i64,12=>acx(t,a as i32) as i64,
 13=>product(t),14=>{preserve(t,a as u8);0},
 15=>state(t).add(layout::AC_M_OFFSET+a as usize*layout::AC_STRIDE).cast::<u16>().read() as i64,
 16=>state(t).add(layout::AX_L_OFFSET+a as usize*layout::AX_STRIDE).cast::<u16>().read() as i64,
 17=>state(t).add(layout::AX_H_OFFSET+a as usize*layout::AX_STRIDE).cast::<u16>().read() as i64,
 18=>{state(t).add(layout::AC_M_OFFSET+a as usize*layout::AC_STRIDE).cast::<u16>().write(b as u16);0},
 _=>terminate()
 }
}
pub mod arithmetic{use super::*;
#[export_name="_ZN3DSP11Interpreter11Interpreter3clrEt"]pub unsafe extern "C" fn clr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_clr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4clrlEt"]pub unsafe extern "C" fn clrl(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_clrl(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5andcfEt"]pub unsafe extern "C" fn andcf(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_andcf(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4andfEt"]pub unsafe extern "C" fn andf(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_andf(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3tstEt"]pub unsafe extern "C" fn tst(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_tst(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6tstaxhEt"]pub unsafe extern "C" fn tstaxh(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_tstaxh(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3cmpEt"]pub unsafe extern "C" fn cmp(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_cmp(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6cmpaxhEt"]pub unsafe extern "C" fn cmpaxh(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_cmpaxh(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4cmpiEt"]pub unsafe extern "C" fn cmpi(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_cmpi(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5cmpisEt"]pub unsafe extern "C" fn cmpis(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_cmpis(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4xorrEt"]pub unsafe extern "C" fn xorr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_xorr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4andrEt"]pub unsafe extern "C" fn andr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_andr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3orrEt"]pub unsafe extern "C" fn orr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_orr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4andcEt"]pub unsafe extern "C" fn andc(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_andc(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3orcEt"]pub unsafe extern "C" fn orc(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_orc(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4xorcEt"]pub unsafe extern "C" fn xorc(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_xorc(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4notcEt"]pub unsafe extern "C" fn notc(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_notc(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4xoriEt"]pub unsafe extern "C" fn xori(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_xori(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4andiEt"]pub unsafe extern "C" fn andi(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_andi(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3oriEt"]pub unsafe extern "C" fn ori(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_ori(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4addrEt"]pub unsafe extern "C" fn addr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_addr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5addaxEt"]pub unsafe extern "C" fn addax(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_addax(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3addEt"]pub unsafe extern "C" fn add(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_add(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4addpEt"]pub unsafe extern "C" fn addp(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_addp(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6addaxlEt"]pub unsafe extern "C" fn addaxl(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_addaxl(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4addiEt"]pub unsafe extern "C" fn addi(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_addi(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5addisEt"]pub unsafe extern "C" fn addis(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_addis(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4incmEt"]pub unsafe extern "C" fn incm(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_incm(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3incEt"]pub unsafe extern "C" fn inc(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_inc(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4subrEt"]pub unsafe extern "C" fn subr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_subr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5subaxEt"]pub unsafe extern "C" fn subax(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_subax(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3subEt"]pub unsafe extern "C" fn sub(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_sub(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4subpEt"]pub unsafe extern "C" fn subp(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_subp(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4decmEt"]pub unsafe extern "C" fn decm(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_decm(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3decEt"]pub unsafe extern "C" fn dec(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_dec(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3negEt"]pub unsafe extern "C" fn neg(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_neg(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3absEt"]pub unsafe extern "C" fn abs(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_abs(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4movrEt"]pub unsafe extern "C" fn movr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_movr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5movaxEt"]pub unsafe extern "C" fn movax(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_movax(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3movEt"]pub unsafe extern "C" fn mov(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_mov(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5lsl16Et"]pub unsafe extern "C" fn lsl16(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsl16(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5lsr16Et"]pub unsafe extern "C" fn lsr16(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsr16(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5asr16Et"]pub unsafe extern "C" fn asr16(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_asr16(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lslEt"]pub unsafe extern "C" fn lsl(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsl(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3lsrEt"]pub unsafe extern "C" fn lsr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3aslEt"]pub unsafe extern "C" fn asl(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_asl(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter3asrEt"]pub unsafe extern "C" fn asr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_asr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4lsrnEt"]pub unsafe extern "C" fn lsrn(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsrn(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter4asrnEt"]pub unsafe extern "C" fn asrn(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_asrn(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6lsrnrxEt"]pub unsafe extern "C" fn lsrnrx(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsrnrx(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter6asrnrxEt"]pub unsafe extern "C" fn asrnrx(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_asrnrx(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5lsrnrEt"]pub unsafe extern "C" fn lsrnr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_lsrnr(t.cast(),op,arith_exec);}
#[export_name="_ZN3DSP11Interpreter11Interpreter5asrnrEt"]pub unsafe extern "C" fn asrnr(t:*mut u8,op:u16){crate::native_dsp_arithmetic::wprt_dsp_arith_asrnr(t.cast(),op,arith_exec);}
}
pub mod analyzer{use super::*;
unsafe extern "C" fn read(c:*const c_void,a:u16)->u16{read_imem(c.cast(),a)}
unsafe extern "C" fn meta(op:u16,out:*mut crate::native_dsp_analyzer::Template)->bool{
 let t=get_template(op);if t.is_null(){return false}
 out.write(crate::native_dsp_analyzer::Template{opcode:t.add(layout::OPCODE_OFFSET).cast::<u16>().read_unaligned(),size:t.add(layout::OP_SIZE_OFFSET).read(),extended:t.add(layout::OP_EXTENDED_OFFSET).read()});true
}
#[export_name="_ZN3DSP8AnalyzerC1Ev"]pub unsafe extern "C" fn construct1(t:*mut u8){std::ptr::write_bytes(t,0,layout::ANALYZER_SIZE);}
#[export_name="_ZN3DSP8AnalyzerC2Ev"]pub unsafe extern "C" fn construct2(t:*mut u8){std::ptr::write_bytes(t,0,layout::ANALYZER_SIZE);}
#[export_name="_ZN3DSP8AnalyzerD1Ev"]pub unsafe extern "C" fn destroy1(_t:*mut u8){}
#[export_name="_ZN3DSP8AnalyzerD2Ev"]pub unsafe extern "C" fn destroy2(_t:*mut u8){}
#[export_name="_ZN3DSP8Analyzer5ResetEv"]pub unsafe extern "C" fn reset(t:*mut u8){crate::native_dsp_analyzer::wprt_dsp_analyzer_reset(t.add(layout::ANALYZER_FLAGS_OFFSET));}
#[export_name="_ZN3DSP8Analyzer7AnalyzeERKNS_4SDSPE"]pub unsafe extern "C" fn analyze(t:*mut u8,s:*const c_void){crate::native_dsp_analyzer::wprt_dsp_analyzer_analyze(t.add(layout::ANALYZER_FLAGS_OFFSET),s,read,meta);}
#[export_name="_ZN3DSP8Analyzer12AnalyzeRangeERKNS_4SDSPEtt"]pub unsafe extern "C" fn range(t:*mut u8,s:*const c_void,a:u16,b:u16){crate::native_dsp_analyzer::wprt_dsp_analyzer_range(t.add(layout::ANALYZER_FLAGS_OFFSET),s,a,b,read,meta);}
#[export_name="_ZN3DSP8Analyzer21FindInstructionStartsERKNS_4SDSPEtt"]pub unsafe extern "C" fn starts(t:*mut u8,s:*const c_void,a:u16,b:u16){crate::native_dsp_analyzer::wprt_dsp_analyzer_starts(t.add(layout::ANALYZER_FLAGS_OFFSET),s,a,b,read,meta);}
#[export_name="_ZN3DSP8Analyzer13FindIdleSkipsERKNS_4SDSPEtt"]pub unsafe extern "C" fn idle(t:*mut u8,s:*const c_void,a:u16,b:u16){crate::native_dsp_analyzer::wprt_dsp_analyzer_idle(t.add(layout::ANALYZER_FLAGS_OFFSET),s,a,b,read);}
}
pub mod tables{use super::*;
#[repr(C)]#[derive(Copy,Clone)]pub struct MemberPointer{pub function:usize,pub adjustment:isize}
extern "C"{
#[link_name="_ZN3DSP11Interpreter11Interpreter3nopEt"]fn opcode_0(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3darEt"]fn opcode_1(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3iarEt"]fn opcode_2(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6subarnEt"]fn opcode_3(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6addarnEt"]fn opcode_4(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4haltEt"]fn opcode_5(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3retEt"]fn opcode_6(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3rtiEt"]fn opcode_7(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4callEt"]fn opcode_8(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4ifccEt"]fn opcode_9(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3jccEt"]fn opcode_10(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6jmprccEt"]fn opcode_11(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5callrEt"]fn opcode_12(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5sbclrEt"]fn opcode_13(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5sbsetEt"]fn opcode_14(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lslEt"]fn opcode_15(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lsrEt"]fn opcode_16(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3aslEt"]fn opcode_17(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3asrEt"]fn opcode_18(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4lsrnEt"]fn opcode_19(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4asrnEt"]fn opcode_20(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lriEt"]fn opcode_21(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2lrEt"]fn opcode_22(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2srEt"]fn opcode_23(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3mrrEt"]fn opcode_24(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2siEt"]fn opcode_25(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5addisEt"]fn opcode_26(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5cmpisEt"]fn opcode_27(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4lrisEt"]fn opcode_28(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4addiEt"]fn opcode_29(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4xoriEt"]fn opcode_30(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4andiEt"]fn opcode_31(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3oriEt"]fn opcode_32(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4cmpiEt"]fn opcode_33(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4andfEt"]fn opcode_34(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5andcfEt"]fn opcode_35(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4ilrrEt"]fn opcode_36(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5ilrrdEt"]fn opcode_37(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5ilrriEt"]fn opcode_38(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5ilrrnEt"]fn opcode_39(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4loopEt"]fn opcode_40(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5bloopEt"]fn opcode_41(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5loopiEt"]fn opcode_42(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6bloopiEt"]fn opcode_43(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lrrEt"]fn opcode_44(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4lrrdEt"]fn opcode_45(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4lrriEt"]fn opcode_46(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4lrrnEt"]fn opcode_47(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3srrEt"]fn opcode_48(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4srrdEt"]fn opcode_49(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4srriEt"]fn opcode_50(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4srrnEt"]fn opcode_51(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lrsEt"]fn opcode_52(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4srshEt"]fn opcode_53(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3srsEt"]fn opcode_54(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4xorrEt"]fn opcode_55(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4andrEt"]fn opcode_56(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3orrEt"]fn opcode_57(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4andcEt"]fn opcode_58(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3orcEt"]fn opcode_59(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4xorcEt"]fn opcode_60(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4notcEt"]fn opcode_61(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6lsrnrxEt"]fn opcode_62(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6asrnrxEt"]fn opcode_63(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5lsrnrEt"]fn opcode_64(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5asrnrEt"]fn opcode_65(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4addrEt"]fn opcode_66(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5addaxEt"]fn opcode_67(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3addEt"]fn opcode_68(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4addpEt"]fn opcode_69(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4subrEt"]fn opcode_70(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5subaxEt"]fn opcode_71(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3subEt"]fn opcode_72(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4subpEt"]fn opcode_73(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4movrEt"]fn opcode_74(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5movaxEt"]fn opcode_75(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3movEt"]fn opcode_76(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4movpEt"]fn opcode_77(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6addaxlEt"]fn opcode_78(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4incmEt"]fn opcode_79(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3incEt"]fn opcode_80(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4decmEt"]fn opcode_81(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3decEt"]fn opcode_82(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3negEt"]fn opcode_83(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5movnpEt"]fn opcode_84(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2nxEt"]fn opcode_85(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3clrEt"]fn opcode_86(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3cmpEt"]fn opcode_87(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6mulaxhEt"]fn opcode_88(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4clrpEt"]fn opcode_89(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter7tstprodEt"]fn opcode_90(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6tstaxhEt"]fn opcode_91(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6srbithEt"]fn opcode_92(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3mulEt"]fn opcode_93(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5asr16Et"]fn opcode_94(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6mulmvzEt"]fn opcode_95(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5mulacEt"]fn opcode_96(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5mulmvEt"]fn opcode_97(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4mulxEt"]fn opcode_98(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3absEt"]fn opcode_99(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter7mulxmvzEt"]fn opcode_100(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6mulxacEt"]fn opcode_101(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6mulxmvEt"]fn opcode_102(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3tstEt"]fn opcode_103(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4mulcEt"]fn opcode_104(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6cmpaxhEt"]fn opcode_105(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter7mulcmvzEt"]fn opcode_106(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6mulcacEt"]fn opcode_107(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6mulcmvEt"]fn opcode_108(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5maddxEt"]fn opcode_109(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5msubxEt"]fn opcode_110(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5maddcEt"]fn opcode_111(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5msubcEt"]fn opcode_112(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5lsl16Et"]fn opcode_113(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4maddEt"]fn opcode_114(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5lsr16Et"]fn opcode_115(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4msubEt"]fn opcode_116(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter7addpaxzEt"]fn opcode_117(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4clrlEt"]fn opcode_118(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5movpzEt"]fn opcode_119(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter7nop_extEt"]fn opcode_120(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2drEt"]fn opcode_121(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2irEt"]fn opcode_122(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2nrEt"]fn opcode_123(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2mvEt"]fn opcode_124(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter1sEt"]fn opcode_125(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2snEt"]fn opcode_126(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter1lEt"]fn opcode_127(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2lnEt"]fn opcode_128(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2lsEt"]fn opcode_129(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2slEt"]fn opcode_130(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lsnEt"]fn opcode_131(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3slnEt"]fn opcode_132(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3lsmEt"]fn opcode_133(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3slmEt"]fn opcode_134(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4lsnmEt"]fn opcode_135(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4slnmEt"]fn opcode_136(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4ldaxEt"]fn opcode_137(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5ldaxnEt"]fn opcode_138(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter5ldaxmEt"]fn opcode_139(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter6ldaxnmEt"]fn opcode_140(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter2ldEt"]fn opcode_141(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3ldnEt"]fn opcode_142(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter3ldmEt"]fn opcode_143(t:*mut u8,op:u16);
#[link_name="_ZN3DSP11Interpreter11Interpreter4ldnmEt"]fn opcode_144(t:*mut u8,op:u16);
}
static FUNCTIONS:[unsafe extern "C" fn(*mut u8,u16);145]=[opcode_0,opcode_1,opcode_2,opcode_3,opcode_4,opcode_5,opcode_6,opcode_7,opcode_8,opcode_9,opcode_10,opcode_11,opcode_12,opcode_13,opcode_14,opcode_15,opcode_16,opcode_17,opcode_18,opcode_19,opcode_20,opcode_21,opcode_22,opcode_23,opcode_24,opcode_25,opcode_26,opcode_27,opcode_28,opcode_29,opcode_30,opcode_31,opcode_32,opcode_33,opcode_34,opcode_35,opcode_36,opcode_37,opcode_38,opcode_39,opcode_40,opcode_41,opcode_42,opcode_43,opcode_44,opcode_45,opcode_46,opcode_47,opcode_48,opcode_49,opcode_50,opcode_51,opcode_52,opcode_53,opcode_54,opcode_55,opcode_56,opcode_57,opcode_58,opcode_59,opcode_60,opcode_61,opcode_62,opcode_63,opcode_64,opcode_65,opcode_66,opcode_67,opcode_68,opcode_69,opcode_70,opcode_71,opcode_72,opcode_73,opcode_74,opcode_75,opcode_76,opcode_77,opcode_78,opcode_79,opcode_80,opcode_81,opcode_82,opcode_83,opcode_84,opcode_85,opcode_86,opcode_87,opcode_88,opcode_89,opcode_90,opcode_91,opcode_92,opcode_93,opcode_94,opcode_95,opcode_96,opcode_97,opcode_98,opcode_99,opcode_100,opcode_101,opcode_102,opcode_103,opcode_104,opcode_105,opcode_106,opcode_107,opcode_108,opcode_109,opcode_110,opcode_111,opcode_112,opcode_113,opcode_114,opcode_115,opcode_116,opcode_117,opcode_118,opcode_119,opcode_120,opcode_121,opcode_122,opcode_123,opcode_124,opcode_125,opcode_126,opcode_127,opcode_128,opcode_129,opcode_130,opcode_131,opcode_132,opcode_133,opcode_134,opcode_135,opcode_136,opcode_137,opcode_138,opcode_139,opcode_140,opcode_141,opcode_142,opcode_143,opcode_144];
unsafe fn select(out:*mut MemberPointer,id:i32)->*mut MemberPointer{
 let f=if id<0{0}else{*FUNCTIONS.as_ptr().add(id as usize) as usize};
 out.write(MemberPointer{function:f,adjustment:0});out
}
#[export_name="_ZN3DSP11Interpreter5GetOpEt"]pub unsafe extern "C" fn get(out:*mut MemberPointer,op:u16)->*mut MemberPointer{select(out,crate::native_dsp_interpreter_tables::wprt_dsp_it_get(op))}
#[export_name="_ZN3DSP11Interpreter8GetExtOpEt"]pub unsafe extern "C" fn get_ext(out:*mut MemberPointer,op:u16)->*mut MemberPointer{select(out,crate::native_dsp_interpreter_tables::wprt_dsp_it_get_ext(op))}
#[export_name="_ZN3DSP11Interpreter21InitInstructionTablesEv"]pub unsafe extern "C" fn init(){crate::native_dsp_interpreter_tables::wprt_dsp_it_init();}
}
