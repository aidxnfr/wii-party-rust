// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted (Claude, G60) DSPInterpreter.cpp -> Rust for every member except the ctor/dtor (exact C++ residue interpreter_lifetime.cpp,
// Codex-approved). PRECISE_BACKLOG is not defined (as in pristine). Log/assert macros are ((void)0) in the shim; MemoryUtil protect no-ops.
// Inline header helpers (SDSP::SetSRFlag/IsSRFlagSet/AdvanceStepCounter, Analyzer flags, DSPBreakpoints::IsAddressBreakPoint,
// DSPIntCCUtil) are expanded here from the pinned headers. All ABIs "C-unwind" so C++ exceptions propagate exactly.
#![allow(non_snake_case, clippy::all)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("DSPInterpreter exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod layout { include!(env!("WPRT_DSP_INTERP_LAYOUT")); }
use layout::*;
use std::sync::atomic::{AtomicBool, Ordering};
const _: () = assert!(I_SIZE == 40 && I_CORE == 0 && LOG_SIZE == 5 && PMF_SIZE == 16 && CORE_DSP == 0 && R == 0);
const SR_CARRY: u16 = 0x0001; const SR_OVERFLOW: u16 = 0x0002; const SR_ARITH_ZERO: u16 = 0x0004; const SR_SIGN: u16 = 0x0008;
const SR_OVER_S32: u16 = 0x0010; const SR_TOP2BITS: u16 = 0x0020; const SR_LOGIC_ZERO: u16 = 0x0040; const SR_OVERFLOW_STICKY: u16 = 0x0080;
const SR_100: u16 = 0x0100; const SR_MUL_MODIFY: u16 = 0x2000; const SR_40_MODE_BIT: u16 = 0x4000; const SR_MUL_UNSIGNED: u16 = 0x8000; const SR_CMP_MASK: u16 = 0x3f;
const CR_RESET: u16 = 0x0001; const CR_HALT: u16 = 0x0004; const CR_INIT_CODE: u16 = 0x0400; const CR_INIT: u16 = 0x0800;
const OP_EXTENDED: usize = 112; // DSPOPCTemplate::extended (tables probe)

#[repr(C)] #[derive(Clone, Copy)] pub struct Pmf { ptr: usize, adj: isize }
extern "C-unwind" {
    #[link_name = "_ZN3DSP11Interpreter11Interpreter10HandleLoopEv"] fn handle_loop(i: *mut u8);
    #[link_name = "_ZN3DSP11Interpreter5GetOpEt"] fn get_op(ret: *mut Pmf, inst: u16) -> *mut Pmf;
    #[link_name = "_ZN3DSP11Interpreter8GetExtOpEt"] fn get_ext_op(ret: *mut Pmf, inst: u16) -> *mut Pmf;
    #[link_name = "_ZN3DSP13GetOpTemplateEt"] fn get_op_template(inst: u16) -> *const u8;
    #[link_name = "_ZN3DSP4Host10CodeLoadedERNS_7DSPCoreEjy"] fn host_code_loaded(core: *mut u8, addr: u32, size: u64);
    #[link_name = "_ZN3DSP4Host8DMAToDSPEPtjj"] fn host_dma_to_dsp(dst: *mut u16, addr: u32, size: u32);
    #[link_name = "_ZN3DSP4Host8TimeBaseEv"] fn host_time_base() -> u64;
    #[link_name = "_ZN3DSP4SDSP10StoreStackENS_13StackRegisterEt"] fn store_stack(s: *mut u8, reg: i32, v: u16);
    #[link_name = "_ZN3DSP4SDSP16FetchInstructionEv"] fn fetch_instruction(s: *mut u8) -> u16;
    #[link_name = "_ZN3DSP4SDSP8PopStackENS_13StackRegisterE"] fn pop_stack(s: *mut u8, reg: i32) -> u16;
    #[link_name = "_ZN3DSP7DSPCore15CheckExceptionsEv"] fn core_check_exceptions(c: *mut u8) -> bool;
    #[link_name = "_ZN3DSP7DSPCore22CheckExternalInterruptEv"] fn core_check_external_interrupt(c: *mut u8);
    #[link_name = "_ZN3DSP7DSPCore5ResetEv"] fn core_reset(c: *mut u8);
    #[link_name = "_ZN3DSP7DSPCore8SetStateENS_5StateE"] fn core_set_state(c: *mut u8, s: i32);
}
unsafe fn f<T: Copy>(p: *mut u8, off: usize) -> T { p.add(off).cast::<T>().read() }
unsafe fn w<T: Copy>(p: *mut u8, off: usize, v: T) { p.add(off).cast::<T>().write(v) }
unsafe fn core(i: *mut u8) -> *mut u8 { f(i, I_CORE) }
unsafe fn st(i: *mut u8) -> *mut u8 { core(i).add(CORE_DSP) }
unsafe fn sr(i: *mut u8) -> u16 { f(st(i), R + R_SR) }
unsafe fn set_sr(i: *mut u8, v: u16) { w(st(i), R + R_SR, v) }
unsafe fn ar(i: *mut u8, r: u16) -> u16 { f(st(i), R + R_AR + 2 * r as usize) }
unsafe fn wr(i: *mut u8, r: u16) -> u16 { f(st(i), R + R_WR + 2 * r as usize) }
unsafe fn ax_l(i: *mut u8, r: i32) -> u16 { f(st(i), (R + R_AX) .wrapping_add((4 * r as isize) as usize)) }
unsafe fn ax_h(i: *mut u8, r: i32) -> u16 { f(st(i), (R + R_AX + 2).wrapping_add((4 * r as isize) as usize)) }
unsafe fn ac_off(r: i32) -> usize { (R + R_AC).wrapping_add((8 * r as isize) as usize) }
unsafe fn call_pmf(i: *mut u8, p: Pmf, inst: u16) {
    let this = i.offset(p.adj);
    let fp: unsafe extern "C-unwind" fn(*mut u8, u16) = if p.ptr & 1 != 0 {
        let vt = *this.cast::<*const u8>();
        std::mem::transmute(*vt.add(p.ptr - 1).cast::<*const ()>())
    } else { std::mem::transmute(p.ptr as *const ()) };
    fp(this, inst)
}

#[export_name = "_ZN3DSP11Interpreter11Interpreter18ExecuteInstructionEt"]
pub unsafe extern "C-unwind" fn ExecuteInstruction(i: *mut u8, inst: u16) {
    let t = get_op_template(inst);
    if *t.add(OP_EXTENDED) != 0 { let mut p = Pmf { ptr: 0, adj: 0 }; get_ext_op(&mut p, inst); call_pmf(i, p, inst); }
    let mut p = Pmf { ptr: 0, adj: 0 }; get_op(&mut p, inst); call_pmf(i, p, inst);
    if *t.add(OP_EXTENDED) != 0 { ApplyWriteBackLog(i); }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter4StepEv"]
pub unsafe extern "C-unwind" fn Step(i: *mut u8) {
    let c = core(i); let s = st(i);
    core_check_exceptions(c);
    w(s, STEP_COUNTER, f::<u64>(s, STEP_COUNTER).wrapping_add(1));
    let opc = fetch_instruction(s);
    ExecuteInstruction(i, opc);
    let pc: u16 = f(s, PC);
    if f::<u8>(s, ANALYZER + AN_FLAGS + pc.wrapping_sub(1) as usize) & 8 != 0 { handle_loop(i); }
}
unsafe fn halted(i: *mut u8) -> bool { f::<u16>(st(i), CONTROL_REG) & CR_HALT != 0 }
unsafe fn idle_skip(i: *mut u8) -> bool { let s = st(i); f::<u8>(s, ANALYZER + AN_FLAGS + f::<u16>(s, PC) as usize) & 2 != 0 }
unsafe fn breakpoint(i: *mut u8) -> bool { let c = core(i); f::<u8>(c, CORE_BP + BP_B + f::<u16>(st(i), PC) as usize) != 0 }
#[export_name = "_ZN3DSP11Interpreter11Interpreter15RunCyclesThreadEi"]
pub unsafe extern "C-unwind" fn RunCyclesThread(i: *mut u8, mut cycles: i32) -> i32 {
    loop {
        if halted(i) { return 0; }
        if (*st(i).add(EXT_INT_WAITING).cast::<AtomicBool>()).swap(false, Ordering::Acquire) { core_check_external_interrupt(core(i)); }
        Step(i);
        cycles = cycles.wrapping_sub(1);
        if cycles <= 0 { return 0; }
    }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter14RunCyclesDebugEi"]
pub unsafe extern "C-unwind" fn RunCyclesDebug(i: *mut u8, mut cycles: i32) -> i32 {
    for _ in 0..8 {
        if halted(i) { return 0; }
        if breakpoint(i) { core_set_state(core(i), STATE_STEPPING); return cycles; }
        Step(i); cycles = cycles.wrapping_sub(1); if cycles <= 0 { return 0; }
    }
    loop {
        for _ in 0..8 {
            if halted(i) { return 0; }
            if breakpoint(i) { core_set_state(core(i), STATE_STEPPING); return cycles; }
            if idle_skip(i) { return 0; }
            Step(i); cycles = cycles.wrapping_sub(1); if cycles <= 0 { return 0; }
        }
        for _ in 0..200 {
            if breakpoint(i) { core_set_state(core(i), STATE_STEPPING); return cycles; }
            Step(i); cycles = cycles.wrapping_sub(1); if cycles <= 0 { return 0; }
        }
    }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter9RunCyclesEi"]
pub unsafe extern "C-unwind" fn RunCycles(i: *mut u8, mut cycles: i32) -> i32 {
    for _ in 0..8 {
        if halted(i) { return 0; }
        Step(i); cycles = cycles.wrapping_sub(1); if cycles <= 0 { return 0; }
    }
    loop {
        for _ in 0..8 {
            if halted(i) { return 0; }
            if idle_skip(i) { return 0; }
            Step(i); cycles = cycles.wrapping_sub(1); if cycles <= 0 { return 0; }
        }
        for _ in 0..200 { Step(i); cycles = cycles.wrapping_sub(1); if cycles <= 0 { return 0; } }
    }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter20WriteControlRegisterEt"]
pub unsafe extern "C-unwind" fn WriteControlRegister(i: *mut u8, mut val: u16) {
    let c = core(i); let s = st(i);
    if val & CR_RESET != 0 { core_reset(c); val &= !CR_RESET; }
    if (f::<u16>(s, CONTROL_REG) & CR_INIT) != 0 && (val & CR_INIT) == 0 {
        w::<u16>(s, PC, 0);
        host_dma_to_dsp(f(s, IRAM), 0x8100_0000, 0x1000);
        host_code_loaded(c, 0x8100_0000, 0x1000);
        val &= !CR_INIT;
        val |= CR_INIT_CODE;
        w::<u64>(s, INIT_CLEAR_TIME, host_time_base().wrapping_add(130));
    }
    w(s, CONTROL_REG, val);
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter19ReadControlRegisterEv"]
pub unsafe extern "C-unwind" fn ReadControlRegister(i: *mut u8) -> u16 {
    let s = st(i);
    if f::<u16>(s, CONTROL_REG) & CR_INIT_CODE != 0 {
        if host_time_base() >= f::<u64>(s, INIT_CLEAR_TIME) { w(s, CONTROL_REG, f::<u16>(s, CONTROL_REG) & !CR_INIT_CODE); }
    }
    f(s, CONTROL_REG)
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter9SetSRFlagEt"] pub unsafe extern "C-unwind" fn SetSRFlag(i: *mut u8, flag: u16) { set_sr(i, sr(i) | flag) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter11IsSRFlagSetEt"] pub unsafe extern "C-unwind" fn IsSRFlagSet(i: *mut u8, flag: u16) -> bool { sr(i) & flag != 0 }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter14CheckConditionEh"]
pub unsafe extern "C-unwind" fn CheckCondition(i: *mut u8, condition: u8) -> bool {
    let is = |fl: u16| IsSRFlagSet(i, fl);
    let less = || is(SR_OVERFLOW) != is(SR_SIGN);
    let cond_b = || (!(is(SR_OVER_S32) || is(SR_TOP2BITS))) || is(SR_ARITH_ZERO);
    match condition & 0xf {
        0xf => true, 0x0 => !less(), 0x1 => less(), 0x2 => !less() && !is(SR_ARITH_ZERO), 0x3 => less() || is(SR_ARITH_ZERO),
        0x4 => !is(SR_ARITH_ZERO), 0x5 => is(SR_ARITH_ZERO), 0x6 => !is(SR_CARRY), 0x7 => is(SR_CARRY), 0x8 => !is(SR_OVER_S32), 0x9 => is(SR_OVER_S32),
        0xa => !cond_b(), 0xb => cond_b(), 0xc => !is(SR_LOGIC_ZERO), 0xd => is(SR_LOGIC_ZERO), 0xe => is(SR_OVERFLOW), _ => true,
    }
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter24IncrementAddressRegisterEt"]
pub unsafe extern "C-unwind" fn IncrementAddressRegister(i: *mut u8, reg: u16) -> u16 {
    let a = ar(i, reg) as u32; let wv = wr(i, reg) as u32; let mut nar = a.wrapping_add(1);
    if (nar ^ a) > ((wv | 1) << 1) { nar = nar.wrapping_sub(wv + 1); }
    nar as u16
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter24DecrementAddressRegisterEt"]
pub unsafe extern "C-unwind" fn DecrementAddressRegister(i: *mut u8, reg: u16) -> u16 {
    let a = ar(i, reg) as u32; let wv = wr(i, reg) as u32; let mut nar = a.wrapping_add(wv);
    if ((nar ^ a) & ((wv | 1) << 1)) > wv { nar = nar.wrapping_sub(wv + 1); }
    nar as u16
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter23IncreaseAddressRegisterEts"]
pub unsafe extern "C-unwind" fn IncreaseAddressRegister(i: *mut u8, reg: u16, ix_: i16) -> u16 {
    let a = ar(i, reg) as u32; let wv = wr(i, reg) as u32; let ix = ix_ as i32;
    let mx = (wv | 1) << 1; let mut nar = a.wrapping_add(ix as u32); let dar = (nar ^ a ^ ix as u32) & mx;
    if ix >= 0 { if dar > wv { nar = nar.wrapping_sub(wv + 1); } }
    else if ((nar.wrapping_add(wv).wrapping_add(1) ^ nar) & dar) <= wv { nar = nar.wrapping_add(wv + 1); }
    nar as u16
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter23DecreaseAddressRegisterEts"]
pub unsafe extern "C-unwind" fn DecreaseAddressRegister(i: *mut u8, reg: u16, ix_: i16) -> u16 {
    let a = ar(i, reg) as u32; let wv = wr(i, reg) as u32; let ix = ix_ as i32;
    let mx = (wv | 1) << 1; let mut nar = a.wrapping_sub(ix as u32); let dar = (nar ^ a ^ !(ix as u32)) & mx;
    if (ix as u32) > 0xFFFF_8000 { if dar > wv { nar = nar.wrapping_sub(wv + 1); } }
    else if ((nar.wrapping_add(wv).wrapping_add(1) ^ nar) & dar) <= wv { nar = nar.wrapping_add(wv + 1); }
    nar as u16
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter10GetLongACXEi"] pub unsafe extern "C-unwind" fn GetLongACX(i: *mut u8, r: i32) -> i32 { (((ax_h(i, r) as u32) << 16) | ax_l(i, r) as u32) as i32 }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter8GetAXLowEi"] pub unsafe extern "C-unwind" fn GetAXLow(i: *mut u8, r: i32) -> i16 { ax_l(i, r) as i16 }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter9GetAXHighEi"] pub unsafe extern "C-unwind" fn GetAXHigh(i: *mut u8, r: i32) -> i16 { ax_h(i, r) as i16 }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter10GetLongAccEi"] pub unsafe extern "C-unwind" fn GetLongAcc(i: *mut u8, r: i32) -> i64 { f::<u64>(st(i), ac_off(r)) as i64 }
#[export_name = "_ZN3DSP11Interpreter11Interpreter10SetLongAccEix"] pub unsafe extern "C-unwind" fn SetLongAcc(i: *mut u8, r: i32, v: i64) { w::<u64>(st(i), ac_off(r), ((v << 24) >> 24) as u64) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter9GetAccLowEi"] pub unsafe extern "C-unwind" fn GetAccLow(i: *mut u8, r: i32) -> i16 { f::<u16>(st(i), ac_off(r)) as i16 }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter9GetAccMidEi"] pub unsafe extern "C-unwind" fn GetAccMid(i: *mut u8, r: i32) -> i16 { f::<u16>(st(i), ac_off(r) + 2) as i16 }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter10GetAccHighEi"] pub unsafe extern "C-unwind" fn GetAccHigh(i: *mut u8, r: i32) -> i16 { f::<u32>(st(i), ac_off(r) + 4) as i16 }
unsafe fn prod(i: *mut u8, k: usize) -> u16 { f(st(i), R + R_PROD + 2 * k) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter14GetLongProductEv"]
pub unsafe extern "C-unwind" fn GetLongProduct(i: *mut u8) -> i64 {
    let mut val = (prod(i, 2) as u8 as i8) as i64; val <<= 32;
    let mut low = prod(i, 1) as i64; low += prod(i, 3) as i64; low <<= 16; low |= prod(i, 0) as i64;
    val.wrapping_add(low)
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter21GetLongProductRoundedEv"]
pub unsafe extern "C-unwind" fn GetLongProductRounded(i: *mut u8) -> i64 {
    let p = GetLongProduct(i);
    if p & 0x10000 != 0 { p.wrapping_add(0x8000) & !0xffff } else { p.wrapping_add(0x7fff) & !0xffff }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter14SetLongProductEx"] pub unsafe extern "C-unwind" fn SetLongProduct(i: *mut u8, v: i64) { w::<u64>(st(i), R + R_PROD, (v as u64) & 0x0000_00FF_FFFF_FFFF) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter18GetMultiplyProductEtth"]
pub unsafe extern "C-unwind" fn GetMultiplyProduct(i: *mut u8, a: u16, b: u16, sign: u8) -> i64 {
    let mut p: i64 = if sign == 1 && IsSRFlagSet(i, SR_MUL_UNSIGNED) { ((a as u32).wrapping_mul(b as u32)) as i64 }
        else if sign == 2 && IsSRFlagSet(i, SR_MUL_UNSIGNED) { ((a as i32).wrapping_mul(b as i16 as i32)) as i64 }
        else { ((a as i16 as i32).wrapping_mul(b as i16 as i32)) as i64 };
    if !IsSRFlagSet(i, SR_MUL_MODIFY) { p <<= 1; }
    p
}
#[export_name = "_ZNK3DSP11Interpreter11Interpreter8MultiplyEtth"] pub unsafe extern "C-unwind" fn Multiply(i: *mut u8, a: u16, b: u16, s: u8) -> i64 { GetMultiplyProduct(i, a, b, s) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter11MultiplyAddEtth"] pub unsafe extern "C-unwind" fn MultiplyAdd(i: *mut u8, a: u16, b: u16, s: u8) -> i64 { GetLongProduct(i).wrapping_add(GetMultiplyProduct(i, a, b, s)) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter11MultiplySubEtth"] pub unsafe extern "C-unwind" fn MultiplySub(i: *mut u8, a: u16, b: u16, s: u8) -> i64 { GetLongProduct(i).wrapping_sub(GetMultiplyProduct(i, a, b, s)) }
#[export_name = "_ZNK3DSP11Interpreter11Interpreter12MultiplyMulXEhhtt"]
pub unsafe extern "C-unwind" fn MultiplyMulX(i: *mut u8, axh0: u8, axh1: u8, v1: u16, v2: u16) -> i64 {
    if axh0 == 0 && axh1 == 0 { Multiply(i, v1, v2, 1) } else if axh0 == 0 && axh1 == 1 { Multiply(i, v1, v2, 2) }
    else if axh0 == 1 && axh1 == 0 { Multiply(i, v2, v1, 2) } else { Multiply(i, v1, v2, 0) }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter10UpdateSR16Esbbb"]
pub unsafe extern "C-unwind" fn UpdateSR16(i: *mut u8, value: i16, carry: bool, overflow: bool, over_s32: bool) {
    let mut s = sr(i) & !SR_CMP_MASK;
    if carry { s |= SR_CARRY; }
    if overflow { s |= SR_OVERFLOW; s |= SR_OVERFLOW_STICKY; }
    if value == 0 { s |= SR_ARITH_ZERO; }
    if value < 0 { s |= SR_SIGN; }
    if over_s32 { s |= SR_OVER_S32; }
    let t = (value as u16) >> 14; if t == 0 || t == 3 { s |= SR_TOP2BITS; }
    set_sr(i, s);
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter10UpdateSR64Exbb"]
pub unsafe extern "C-unwind" fn UpdateSR64(i: *mut u8, value: i64, carry: bool, overflow: bool) {
    let mut s = sr(i) & !SR_CMP_MASK;
    if carry { s |= SR_CARRY; }
    if overflow { s |= SR_OVERFLOW; s |= SR_OVERFLOW_STICKY; }
    if value == 0 { s |= SR_ARITH_ZERO; }
    if value < 0 { s |= SR_SIGN; }
    if value != value as i32 as i64 { s |= SR_OVER_S32; }
    let t = value & 0xc000_0000; if t == 0 || t == 0xc000_0000 { s |= SR_TOP2BITS; }
    set_sr(i, s);
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter13UpdateSR64AddExxx"]
pub unsafe extern "C-unwind" fn UpdateSR64Add(i: *mut u8, v1: i64, v2: i64, res: i64) { UpdateSR64(i, res, (v1 as u64) > (res as u64), ((v1 ^ res) & (v2 ^ res)) < 0) }
#[export_name = "_ZN3DSP11Interpreter11Interpreter13UpdateSR64SubExxx"]
pub unsafe extern "C-unwind" fn UpdateSR64Sub(i: *mut u8, v1: i64, v2: i64, res: i64) { let nv2 = v2.wrapping_neg(); UpdateSR64(i, res, (v1 as u64) >= (res as u64), ((v1 ^ res) & (nv2 ^ res)) < 0) }
#[export_name = "_ZN3DSP11Interpreter11Interpreter17UpdateSRLogicZeroEb"]
pub unsafe extern "C-unwind" fn UpdateSRLogicZero(i: *mut u8, v: bool) { if v { set_sr(i, sr(i) | SR_LOGIC_ZERO) } else { set_sr(i, sr(i) & !SR_LOGIC_ZERO) } }
#[export_name = "_ZN3DSP11Interpreter11Interpreter14OpReadRegisterEi"]
pub unsafe extern "C-unwind" fn OpReadRegister(i: *mut u8, reg_: i32) -> u16 {
    let reg = reg_ & 0x1f; let s = st(i);
    match reg {
        0x0c..=0x0f => pop_stack(s, reg - 0x0c),
        0x00..=0x03 => f(s, R + R_AR + 2 * reg as usize),
        0x04..=0x07 => f(s, R + R_IX + 2 * (reg - 4) as usize),
        0x08..=0x0b => f(s, R + R_WR + 2 * (reg - 8) as usize),
        0x10 | 0x11 => f::<u32>(s, ac_off(reg - 0x10) + 4) as u16,
        0x12 => f(s, R + R_CR), 0x13 => f(s, R + R_SR),
        0x14 => prod(i, 0), 0x15 => prod(i, 1), 0x16 => prod(i, 2), 0x17 => prod(i, 3),
        0x18 | 0x19 => ax_l(i, reg - 0x18), 0x1a | 0x1b => ax_h(i, reg - 0x1a),
        0x1c | 0x1d => f(s, ac_off(reg - 0x1c)),
        _ => {
            if IsSRFlagSet(i, SR_40_MODE_BIT) {
                let acc = GetLongAcc(i, reg - 0x1e);
                if acc != acc as i32 as i64 { return if acc > 0 { 0x7fff } else { 0x8000 }; }
                return f(s, ac_off(reg - 0x1e) + 2);
            }
            f(s, ac_off(reg - 0x1e) + 2)
        }
    }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter15OpWriteRegisterEit"]
pub unsafe extern "C-unwind" fn OpWriteRegister(i: *mut u8, reg_: i32, val: u16) {
    let reg = reg_ & 0x1f; let s = st(i);
    match reg {
        0x10 | 0x11 => w::<u32>(s, ac_off(reg - 0x10) + 4, val as i8 as i32 as u32),
        0x0c..=0x0f => store_stack(s, reg - 0x0c, val),
        0x00..=0x03 => w(s, R + R_AR + 2 * reg as usize, val),
        0x04..=0x07 => w(s, R + R_IX + 2 * (reg - 4) as usize, val),
        0x08..=0x0b => w(s, R + R_WR + 2 * (reg - 8) as usize, val),
        0x12 => w(s, R + R_CR, val & 0x00ff),
        0x13 => w(s, R + R_SR, val & !SR_100),
        0x14 => w(s, R + R_PROD, val), 0x15 => w(s, R + R_PROD + 2, val), 0x16 => w(s, R + R_PROD + 4, val & 0x00ff), 0x17 => w(s, R + R_PROD + 6, val),
        0x18 | 0x19 => w(s, (R + R_AX).wrapping_add(4 * (reg - 0x18) as usize), val),
        0x1a | 0x1b => w(s, (R + R_AX + 2).wrapping_add(4 * (reg - 0x1a) as usize), val),
        0x1c | 0x1d => w(s, ac_off(reg - 0x1c), val),
        _ => w(s, ac_off(reg - 0x1e) + 2, val),
    }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter22ConditionalExtendAccumEi"]
pub unsafe extern "C-unwind" fn ConditionalExtendAccum(i: *mut u8, reg: i32) {
    if reg != 0x1e && reg != 0x1f { return; }
    if !IsSRFlagSet(i, SR_40_MODE_BIT) { return; }
    let s = st(i); let o = ac_off(reg - 0x1e);
    let val: u16 = f(s, o + 2);
    w::<u32>(s, o + 4, if val & 0x8000 != 0 { 0xFFFF_FFFF } else { 0 });
    w::<u16>(s, o, 0);
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter17ApplyWriteBackLogEv"]
pub unsafe extern "C-unwind" fn ApplyWriteBackLog(i: *mut u8) {
    // Same unchecked loop as C++ (std::array operator[] without bounds check).
    let idx = i.add(I_LOG_IDX).cast::<i32>(); let log = i.add(I_LOG).cast::<u16>();
    let mut k = 0usize;
    while idx.add(k).read() != -1 { let v = log.add(k).read(); OpWriteRegister(i, idx.add(k).read(), v); idx.add(k).write(-1); k += 1; }
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter14WriteToBackLogEiit"]
pub unsafe extern "C-unwind" fn WriteToBackLog(i: *mut u8, k: i32, idx: i32, value: u16) {
    i.add(I_LOG).cast::<u16>().offset(k as isize).write(value);
    i.add(I_LOG_IDX).cast::<i32>().offset(k as isize).write(idx);
}
#[export_name = "_ZN3DSP11Interpreter11Interpreter16ZeroWriteBackLogEv"] pub unsafe extern "C-unwind" fn ZeroWriteBackLog(_i: *mut u8) {}
#[export_name = "_ZN3DSP11Interpreter11Interpreter27ZeroWriteBackLogPreserveAccEh"] pub unsafe extern "C-unwind" fn ZeroWriteBackLogPreserveAcc(_i: *mut u8, _acc: u8) {}
#[export_name = "_ZN3DSP11Interpreter11Interpreter3nopEt"] pub unsafe extern "C-unwind" fn nop(_i: *mut u8, _opc: u16) {}

// Interpreter lifetime port: pristine DSPInterpreter.cpp ctor/dtor;
// DSPInterpreter.h:241-245 member initializers; compiler layout.rs + lifetime.obj disassembly.
// Padding at bytes18..19 is intentionally left untouched, exactly as the original.
extern "C-unwind" {
    #[link_name = "_ZN3DSP11Interpreter21InitInstructionTablesEv"]
    fn lifetime_init_instruction_tables();
}
unsafe fn lifetime_construct(i: *mut u8, dsp: *mut u8) {
    w(i, I_CORE, dsp);
    core::ptr::write_bytes(i.add(I_LOG), 0, LOG_SIZE * core::mem::size_of::<u16>());
    for slot in 0..LOG_SIZE { w(i, I_LOG_IDX + slot * core::mem::size_of::<i32>(), -1_i32); }
    lifetime_init_instruction_tables();
}
#[export_name = "_ZN3DSP11Interpreter11InterpreterC1ERNS_7DSPCoreE"]
pub unsafe extern "C-unwind" fn lifetime_c1(i: *mut u8, dsp: *mut u8) { lifetime_construct(i, dsp); }
#[export_name = "_ZN3DSP11Interpreter11InterpreterC2ERNS_7DSPCoreE"]
pub unsafe extern "C-unwind" fn lifetime_c2(i: *mut u8, dsp: *mut u8) { lifetime_construct(i, dsp); }
#[export_name = "_ZN3DSP11Interpreter11InterpreterD1Ev"]
pub unsafe extern "C-unwind" fn lifetime_d1(_i: *mut u8) {}
#[export_name = "_ZN3DSP11Interpreter11InterpreterD2Ev"]
pub unsafe extern "C-unwind" fn lifetime_d2(_i: *mut u8) {}
