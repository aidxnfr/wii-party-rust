// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted (Claude, G60) DSPCore.cpp -> Rust for every plain member; the C++ lifetime residue (dsp_core_lifetime.cpp, Codex-approved)
// keeps SDSP/DSPCore ctors+dtors, LLEAccelerator (vtable/typeinfo/overrides), RunCycles/Step/SetState (Common::Event).
// Functions that may see a C++ exception (operator new in make_unique) use the "C-unwind" ABI so it propagates exactly as in C++.
// VerifyRoms: shim AskYesNoFmtT is `false` and HashAdler32 is pure, so it always returns true with no observable effect.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("DSPCore exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod layout { include!(env!("WPRT_DSP_CORE_LAYOUT")); }
use layout::*;
use std::sync::atomic::{AtomicBool, Ordering};
const _: () = assert!(REGS_SIZE == 72 && ATOMIC_BOOL_SIZE == 1 && STATE_SIZE == 4 && AX_SIZE == 4 && AC_SIZE == 8);
const IROM_BYTES: usize = 0x2000; const IRAM_BYTES: usize = 0x2000; const DRAM_BYTES: usize = 0x2000; const COEF_BYTES: usize = 0x1000;
const IRAM_SIZE: usize = 0x1000; const DRAM_SIZE: usize = 0x1000; const STACK_DEPTH: usize = 0x20;
const SR_INT_ENABLE: u16 = 0x0200; const SR_EXT_INT_ENABLE: u16 = 0x0800;
const CR_EXTERNAL_INT: u16 = 0x0002; const CR_HALT: u16 = 0x0004; const CR_INIT: u16 = 0x0800;

extern "C-unwind" {
    #[link_name = "wprt_claude_dsp_new_lle_accelerator"] fn new_lle_accelerator(s: *mut u8) -> *mut u8;
}
extern "C-unwind" {
    #[link_name = "_ZN3DSP4SDSP13InitializeIFXEv"] fn initialize_ifx(s: *mut u8);
    #[link_name = "_ZN3DSP4SDSP10StoreStackENS_13StackRegisterEt"] fn store_stack(s: *mut u8, reg: i32, v: u16);
    #[link_name = "_ZNK3DSP4SDSP11PeekMailboxENS_7MailboxE"] fn sdsp_peek(s: *mut u8, m: u32) -> u32;
    #[link_name = "_ZN3DSP4SDSP14ReadMailboxLowENS_7MailboxE"] fn sdsp_read_low(s: *mut u8, m: u32) -> u16;
    #[link_name = "_ZN3DSP4SDSP15ReadMailboxHighENS_7MailboxE"] fn sdsp_read_high(s: *mut u8, m: u32) -> u16;
    #[link_name = "_ZN3DSP4SDSP15WriteMailboxLowENS_7MailboxEt"] fn sdsp_write_low(s: *mut u8, m: u32, v: u16);
    #[link_name = "_ZN3DSP4SDSP16WriteMailboxHighENS_7MailboxEt"] fn sdsp_write_high(s: *mut u8, m: u32, v: u16);
    #[link_name = "_ZN3DSP11Accelerator7DoStateER11PointerWrap"] fn acc_do_state(a: *mut u8, p: *mut u8);
    #[link_name = "_ZN3DSP4Host10CodeLoadedERNS_7DSPCoreEPKhy"] fn host_code_loaded(core: *mut u8, ptr: *const u8, size: u64);
    #[link_name = "_ZN3DSP8Analyzer7AnalyzeERKNS_4SDSPE"] fn analyzer_analyze(an: *mut u8, s: *const u8);
    #[link_name = "_Znwy"] fn op_new(n: usize) -> *mut u8;
    #[link_name = "_ZdlPvy"] fn op_delete_sized(p: *mut u8, n: usize);
    #[link_name = "_ZSt20__throw_length_errorPKc"] fn throw_length_error(what: *const u8) -> !;
}
extern "C" { fn calloc(n: usize, s: usize) -> *mut u8; fn free(p: *mut u8); }
unsafe fn f<T: Copy>(p: *mut u8, off: usize) -> T { p.add(off).cast::<T>().read() }
unsafe fn w<T: Copy>(p: *mut u8, off: usize, v: T) { p.add(off).cast::<T>().write(v) }
unsafe fn vslot(obj: *mut u8, i: usize) -> *const () { (*obj.cast::<*const *const ()>()).add(i).read() }
unsafe fn delete_virtual(obj: *mut u8) { let d: unsafe extern "C-unwind" fn(*mut u8) = std::mem::transmute(vslot(obj, 1)); d(obj) }
unsafe fn sr(s: *mut u8) -> u16 { f(s, R + R_SR) }

// ---------------- SDSP ----------------
#[export_name = "_ZN3DSP4SDSP10InitializeERKNS_14DSPInitOptionsE"]
pub unsafe extern "C-unwind" fn SDSP_Initialize(s: *mut u8, opts: *const u8) -> bool {
    w::<u64>(s, STEP_COUNTER, 0);
    let new = new_lle_accelerator(s);
    let old: *mut u8 = f(s, ACCELERATOR);
    w(s, ACCELERATOR, new);
    if !old.is_null() { delete_virtual(old); }
    w(s, IROM, calloc(1, IROM_BYTES));
    w(s, IRAM, calloc(1, IRAM_BYTES));
    w(s, DRAM, calloc(1, DRAM_BYTES));
    w(s, COEF, calloc(1, COEF_BYTES));
    std::ptr::copy_nonoverlapping(opts.add(OPT_IROM), f::<*mut u8>(s, IROM), IROM_BYTES);
    std::ptr::copy_nonoverlapping(opts.add(OPT_COEF), f::<*mut u8>(s, COEF), COEF_BYTES);
    std::ptr::write_bytes(s.add(R), 0, REGS_SIZE);
    std::ptr::write_bytes(s.add(REG_STACK_PTRS), 0, 4);
    std::ptr::write_bytes(s.add(REG_STACKS), 0, 4 * STACK_DEPTH * 2);
    let iram: *mut u16 = f(s, IRAM); for i in 0..IRAM_SIZE { iram.add(i).write(0x0021); }
    let dram: *mut u16 = f(s, DRAM); for i in 0..DRAM_SIZE { dram.add(i).write(0); }
    for i in 0..4 { w::<u16>(s, R + R_WR + 2 * i, 0xffff); }
    w(s, R + R_SR, sr(s) | SR_INT_ENABLE);
    w(s, R + R_SR, sr(s) | SR_EXT_INT_ENABLE);
    w::<u16>(s, CONTROL_REG, CR_INIT | CR_HALT);
    initialize_ifx(s);
    true
}
#[export_name = "_ZN3DSP4SDSP5ResetEv"]
pub unsafe extern "C-unwind" fn SDSP_Reset(s: *mut u8) { w::<u16>(s, PC, 0x8000); for i in 0..4 { w::<u16>(s, R + R_WR + 2 * i, 0xffff); } }
#[export_name = "_ZN3DSP4SDSP15FreeMemoryPagesEv"]
pub unsafe extern "C-unwind" fn SDSP_FreeMemoryPages(s: *mut u8) {
    free(f(s, IROM)); free(f(s, IRAM)); free(f(s, DRAM)); free(f(s, COEF));
    for o in [IROM, IRAM, DRAM, COEF] { w::<*mut u8>(s, o, std::ptr::null_mut()); }
}
#[export_name = "_ZN3DSP4SDSP8ShutdownEv"]
pub unsafe extern "C-unwind" fn SDSP_Shutdown(s: *mut u8) { SDSP_FreeMemoryPages(s) }
#[export_name = "_ZN3DSP4SDSP12SetExceptionENS_13ExceptionTypeE"]
pub unsafe extern "C-unwind" fn SDSP_SetException(s: *mut u8, e: i32) { w(s, EXCEPTIONS, f::<u8>(s, EXCEPTIONS) | (1i32.wrapping_shl(e as u32) as u8)) }
#[export_name = "_ZN3DSP4SDSP20SetExternalInterruptEb"]
pub unsafe extern "C-unwind" fn SDSP_SetExternalInterrupt(s: *mut u8, val: bool) { (*s.add(EXT_INT_WAITING).cast::<AtomicBool>()).store(val, Ordering::Release) }
#[export_name = "_ZN3DSP4SDSP22CheckExternalInterruptEv"]
pub unsafe extern "C-unwind" fn SDSP_CheckExternalInterrupt(s: *mut u8) {
    if sr(s) & SR_EXT_INT_ENABLE == 0 { return; }
    SDSP_SetException(s, 7);
    w(s, CONTROL_REG, f::<u16>(s, CONTROL_REG) & !CR_EXTERNAL_INT);
}
#[export_name = "_ZN3DSP4SDSP15CheckExceptionsEv"]
pub unsafe extern "C-unwind" fn SDSP_CheckExceptions(s: *mut u8) -> bool {
    if f::<u8>(s, EXCEPTIONS) == 0 { return false; }
    let mut i = 7i32;
    while i > 0 {
        if (f::<u8>(s, EXCEPTIONS) as u32 & (1u32 << i)) != 0 {
            if sr(s) & SR_INT_ENABLE != 0 || i == 7 {
                store_stack(s, 0, f(s, PC));
                store_stack(s, 1, sr(s));
                w::<u16>(s, PC, (i * 2) as u16);
                w(s, EXCEPTIONS, f::<u8>(s, EXCEPTIONS) & !((1i32 << i) as u8));
                if i == 7 { w(s, R + R_SR, sr(s) & !SR_EXT_INT_ENABLE); } else { w(s, R + R_SR, sr(s) & !SR_INT_ENABLE); }
                return true;
            }
        }
        i -= 1;
    }
    false
}
fn reg_off(reg: usize) -> Option<(usize, u8)> {
    // (byte offset inside DSP_Regs, width: 2 = u16 field, 4 = u32 field (ac.h))
    Some(match reg {
        0x00..=0x03 => (R_AR + 2 * reg, 2),
        0x04..=0x07 => (R_IX + 2 * (reg - 0x04), 2),
        0x08..=0x0b => (R_WR + 2 * (reg - 0x08), 2),
        0x0c..=0x0f => (R_ST + 2 * (reg - 0x0c), 2),
        0x10 | 0x11 => (R_AC + AC_SIZE * (reg - 0x10) + 4, 4),
        0x12 => (R_CR, 2), 0x13 => (R_SR, 2),
        0x14 => (R_PROD, 2), 0x15 => (R_PROD + 2, 2), 0x16 => (R_PROD + 4, 2), 0x17 => (R_PROD + 6, 2),
        0x18 | 0x19 => (R_AX + AX_SIZE * (reg - 0x18), 2),
        0x1a | 0x1b => (R_AX + AX_SIZE * (reg - 0x1a) + 2, 2),
        0x1c | 0x1d => (R_AC + AC_SIZE * (reg - 0x1c), 2),
        0x1e | 0x1f => (R_AC + AC_SIZE * (reg - 0x1e) + 2, 2),
        _ => return None,
    })
}
#[export_name = "_ZNK3DSP4SDSP12ReadRegisterEy"]
pub unsafe extern "C-unwind" fn SDSP_ReadRegister(s: *mut u8, reg: usize) -> u16 {
    match reg_off(reg) { Some((o, 4)) => f::<u32>(s, R + o) as u16, Some((o, _)) => f::<u16>(s, R + o), None => 0 }
}
#[export_name = "_ZN3DSP4SDSP13WriteRegisterEyt"]
pub unsafe extern "C-unwind" fn SDSP_WriteRegister(s: *mut u8, reg: usize, val: u16) {
    match reg_off(reg) { Some((o, 4)) => w::<u32>(s, R + o, val as u32), Some((o, _)) => w::<u16>(s, R + o, val), None => {} }
}

// PointerWrap (dolphin_shim ChunkFile.h): { std::vector<u8>* buffer; Mode mode; size_t position }, Measure 0 / Write 1 / Read 2.
#[repr(C)] struct Vec8 { begin: *mut u8, finish: *mut u8, eos: *mut u8 }
#[repr(C)] struct PW { buffer: *mut Vec8, mode: i32, position: usize }
unsafe fn vec_append(v: *mut Vec8, src: *const u8, n: usize) {
    if n == 0 { return; }
    let v = &mut *v;
    let size = v.finish as usize - v.begin as usize;
    let cap = v.eos as usize - v.begin as usize;
    if cap - size >= n { std::ptr::copy_nonoverlapping(src, v.finish, n); v.finish = v.finish.add(n); return; }
    const MAX: usize = isize::MAX as usize;
    if MAX - size < n { throw_length_error(b"vector::_M_range_insert\0".as_ptr()); }
    let mut len = size + size.max(n);
    if len < size || len > MAX { len = MAX; }
    let nb = if len != 0 { op_new(len) } else { std::ptr::null_mut() };
    if size != 0 { std::ptr::copy_nonoverlapping(v.begin, nb, size); }
    std::ptr::copy_nonoverlapping(src, nb.add(size), n);
    if !v.begin.is_null() { op_delete_sized(v.begin, cap); }
    v.begin = nb; v.finish = nb.add(size + n); v.eos = nb.add(len);
}
unsafe fn pw(p: *mut u8, data: *mut u8, size: usize) {
    let p = &mut *p.cast::<PW>();
    if p.buffer.is_null() { return; }
    if p.mode == 1 { vec_append(p.buffer, data, size); }
    else if p.mode == 2 { let b = &*p.buffer; let len = b.finish as usize - b.begin as usize; if p.position + size <= len { std::ptr::copy_nonoverlapping(b.begin.add(p.position), data, size); } }
    p.position += size;
}
#[export_name = "_ZN3DSP4SDSP7DoStateER11PointerWrap"]
pub unsafe extern "C-unwind" fn SDSP_DoState(s: *mut u8, p: *mut u8) {
    pw(p, s.add(R), REGS_SIZE);
    pw(p, s.add(PC), 2);
    pw(p, s.add(CONTROL_REG), 2);
    pw(p, s.add(INIT_CLEAR_TIME), 8);
    pw(p, s.add(REG_STACK_PTRS), 4);
    pw(p, s.add(EXCEPTIONS), 1);
    pw(p, s.add(EXT_INT_WAITING), 1);
    for k in 0..4 { pw(p, s.add(REG_STACKS + k * STACK_DEPTH * 2), STACK_DEPTH * 2); }
    pw(p, s.add(STEP_COUNTER), 8);
    pw(p, s.add(IFX_REGS), 512);
    acc_do_state(f(s, ACCELERATOR), p);
    pw(p, s.add(MAILBOX), 4);
    pw(p, s.add(MAILBOX + 4), 4);
    pw(p, f(s, IRAM), IRAM_SIZE * 2);
    if (*p.cast::<PW>()).mode == 2 { host_code_loaded(f(s, DSP_CORE), f(s, IRAM), IRAM_BYTES as u64); }
    pw(p, f(s, DRAM), DRAM_SIZE * 2);
}

// ---------------- DSPCore ----------------
unsafe fn dsp(c: *mut u8) -> *mut u8 { c.add(CORE_DSP) }
#[export_name = "_ZN3DSP7DSPCore10InitializeERKNS_14DSPInitOptionsE"]
pub unsafe extern "C-unwind" fn Core_Initialize(c: *mut u8, opts: *const u8) -> bool {
    if !SDSP_Initialize(dsp(c), opts) { return false; }
    let new: *mut u8 = f(opts.cast_mut(), OPT_CAPTURE);
    let old: *mut u8 = f(c, CORE_CAP);
    w(c, CORE_CAP, new);
    if !old.is_null() { delete_virtual(old); }
    w::<u32>(c, CORE_STATE, STATE_RUNNING);
    true
}
#[export_name = "_ZN3DSP7DSPCore8ShutdownEv"]
pub unsafe extern "C-unwind" fn Core_Shutdown(c: *mut u8) {
    if f::<u32>(c, CORE_STATE) == STATE_STOPPED { return; }
    w::<u32>(c, CORE_STATE, STATE_STOPPED);
    SDSP_Shutdown(dsp(c));
    let old: *mut u8 = f(c, CORE_CAP);
    w::<*mut u8>(c, CORE_CAP, std::ptr::null_mut());
    if !old.is_null() { delete_virtual(old); }
}
#[export_name = "_ZN3DSP7DSPCore5ResetEv"]
pub unsafe extern "C-unwind" fn Core_Reset(c: *mut u8) { SDSP_Reset(dsp(c)); analyzer_analyze(dsp(c).add(ANALYZER), dsp(c)); }
#[export_name = "_ZN3DSP7DSPCore9ClearIRAMEv"] pub unsafe extern "C-unwind" fn Core_ClearIRAM(_c: *mut u8) {}
#[export_name = "_ZNK3DSP7DSPCore8GetStateEv"] pub unsafe extern "C-unwind" fn Core_GetState(c: *mut u8) -> u32 { f(c, CORE_STATE) }
#[export_name = "_ZN3DSP7DSPCore12SetExceptionENS_13ExceptionTypeE"] pub unsafe extern "C-unwind" fn Core_SetException(c: *mut u8, e: i32) { SDSP_SetException(dsp(c), e) }
#[export_name = "_ZN3DSP7DSPCore20SetExternalInterruptEb"] pub unsafe extern "C-unwind" fn Core_SetExternalInterrupt(c: *mut u8, v: bool) { SDSP_SetExternalInterrupt(dsp(c), v) }
#[export_name = "_ZN3DSP7DSPCore22CheckExternalInterruptEv"] pub unsafe extern "C-unwind" fn Core_CheckExternalInterrupt(c: *mut u8) { SDSP_CheckExternalInterrupt(dsp(c)) }
#[export_name = "_ZN3DSP7DSPCore15CheckExceptionsEv"] pub unsafe extern "C-unwind" fn Core_CheckExceptions(c: *mut u8) -> bool { SDSP_CheckExceptions(dsp(c)) }
#[export_name = "_ZNK3DSP7DSPCore12ReadRegisterEy"] pub unsafe extern "C-unwind" fn Core_ReadRegister(c: *mut u8, r: usize) -> u16 { SDSP_ReadRegister(dsp(c), r) }
#[export_name = "_ZN3DSP7DSPCore13WriteRegisterEyt"] pub unsafe extern "C-unwind" fn Core_WriteRegister(c: *mut u8, r: usize, v: u16) { SDSP_WriteRegister(dsp(c), r, v) }
#[export_name = "_ZNK3DSP7DSPCore11PeekMailboxENS_7MailboxE"] pub unsafe extern "C-unwind" fn Core_PeekMailbox(c: *mut u8, m: u32) -> u32 { sdsp_peek(dsp(c), m) }
#[export_name = "_ZN3DSP7DSPCore14ReadMailboxLowENS_7MailboxE"] pub unsafe extern "C-unwind" fn Core_ReadMailboxLow(c: *mut u8, m: u32) -> u16 { sdsp_read_low(dsp(c), m) }
#[export_name = "_ZN3DSP7DSPCore15ReadMailboxHighENS_7MailboxE"] pub unsafe extern "C-unwind" fn Core_ReadMailboxHigh(c: *mut u8, m: u32) -> u16 { sdsp_read_high(dsp(c), m) }
#[export_name = "_ZN3DSP7DSPCore15WriteMailboxLowENS_7MailboxEt"] pub unsafe extern "C-unwind" fn Core_WriteMailboxLow(c: *mut u8, m: u32, v: u16) { sdsp_write_low(dsp(c), m, v) }
#[export_name = "_ZN3DSP7DSPCore16WriteMailboxHighENS_7MailboxEt"] pub unsafe extern "C-unwind" fn Core_WriteMailboxHigh(c: *mut u8, m: u32, v: u16) { sdsp_write_high(dsp(c), m, v) }
unsafe fn cap(c: *mut u8) -> *mut u8 { f(c, CORE_CAP) }
#[export_name = "_ZN3DSP7DSPCore10LogIFXReadEtt"]
pub unsafe extern "C-unwind" fn Core_LogIFXRead(c: *mut u8, a: u16, v: u16) { let o = cap(c); let fp: unsafe extern "C-unwind" fn(*mut u8, u16, u16) = std::mem::transmute(vslot(o, VT_LOG_IFX_READ)); fp(o, a, v) }
#[export_name = "_ZN3DSP7DSPCore11LogIFXWriteEtt"]
pub unsafe extern "C-unwind" fn Core_LogIFXWrite(c: *mut u8, a: u16, v: u16) { let o = cap(c); let fp: unsafe extern "C-unwind" fn(*mut u8, u16, u16) = std::mem::transmute(vslot(o, VT_LOG_IFX_WRITE)); fp(o, a, v) }
#[export_name = "_ZN3DSP7DSPCore6LogDMAEtjttPKh"]
pub unsafe extern "C-unwind" fn Core_LogDMA(c: *mut u8, ctl: u16, g: u32, d: u16, l: u16, data: *const u8) { let o = cap(c); let fp: unsafe extern "C-unwind" fn(*mut u8, u16, u32, u16, u16, *const u8) = std::mem::transmute(vslot(o, VT_LOG_DMA)); fp(o, ctl, g, d, l, data) }
#[export_name = "_ZNK3DSP7DSPCore12IsJITCreatedEv"] pub unsafe extern "C-unwind" fn Core_IsJITCreated(_c: *mut u8) -> bool { false }
#[export_name = "_ZN3DSP7DSPCore7DoStateER11PointerWrap"] pub unsafe extern "C-unwind" fn Core_DoState(c: *mut u8, p: *mut u8) { SDSP_DoState(dsp(c), p) }

// Remaining lifetime/Event/LLE implementation: source dsp_core_lifetime.cpp and pinned object disassembly.
static LC_BASE_NAME: &[u8] = &[78, 51, 68, 83, 80, 49, 49, 65, 99, 99, 101, 108, 101, 114, 97, 116, 111, 114, 69, 0];
static LC_LLE_NAME: &[u8] = &[78, 51, 68, 83, 80, 49, 52, 76, 76, 69, 65, 99, 99, 101, 108, 101, 114, 97, 116, 111, 114, 69, 0];

// Offsets below are actual operands in lifetime.obj, consistent with accepted core/layout.rs.
const LC_BP: usize = 0x103a8;
const LC_MUTEX: usize = 0x203c0;
const LC_COND: usize = 0x203c8;
const LC_SET: usize = 0x203d0;
extern "C-unwind" {
    #[link_name="_ZN3DSP8AnalyzerC1Ev"] fn lc_analyzer_ctor(p:*mut u8);
    #[link_name="_ZN3DSP8AnalyzerD1Ev"] fn lc_analyzer_dtor(p:*mut u8);
    #[link_name="_ZN3DSP11Interpreter11InterpreterC1ERNS_7DSPCoreE"] fn lc_interp_ctor(p:*mut u8,c:*mut u8);
    #[link_name="_ZN3DSP11Interpreter11InterpreterD1Ev"] fn lc_interp_dtor(p:*mut u8);
    #[link_name="_ZN3DSP11Interpreter11Interpreter9RunCyclesEi"] fn lc_interp_cycles(p:*mut u8,n:i32)->i32;
    #[link_name="_ZN3DSP11Interpreter11Interpreter4StepEv"] fn lc_interp_step(p:*mut u8);
    fn pthread_mutex_init(p:*mut u8,a:*const u8)->i32;
    fn pthread_mutex_destroy(p:*mut u8)->i32;
    fn pthread_mutex_lock(p:*mut u8)->i32;
    fn pthread_mutex_unlock(p:*mut u8)->i32;
    #[link_name="_ZNSt18condition_variableC1Ev"] fn lc_cond_ctor(p:*mut u8);
    #[link_name="_ZNSt18condition_variableD1Ev"] fn lc_cond_dtor(p:*mut u8);
    #[link_name="_ZNSt18condition_variable10notify_oneEv"] fn lc_cond_notify(p:*mut u8);
    #[link_name="_ZNSt18condition_variable4waitERSt11unique_lockISt5mutexE"] fn lc_cond_wait(p:*mut u8,l:*mut LCUniqueLock);
    #[link_name="_ZSt20__throw_system_errori"] fn lc_system_error(e:i32)->!;
    #[link_name="_ZN3DSP4Host14ReadHostMemoryEj"] fn lc_host_read(a:u32)->u8;
    #[link_name="_ZN3DSP4Host15WriteHostMemoryEhj"] fn lc_host_write(v:u8,a:u32);
}
extern "C" {
    #[link_name="_ZTVN10__cxxabiv117__class_type_infoE"] static LC_CLASS_TI:[usize;3];
    #[link_name="_ZTVN10__cxxabiv120__si_class_type_infoE"] static LC_SI_TI:[usize;3];
}
#[repr(C)]struct LCBaseInfo{vp:*const usize,name:*const u8} unsafe impl Sync for LCBaseInfo{}
#[repr(C)]struct LCSiInfo{vp:*const usize,name:*const u8,base:*const LCBaseInfo}unsafe impl Sync for LCSiInfo{}
#[export_name="_ZTIN3DSP11AcceleratorE"]static LC_ACC_INFO:LCBaseInfo=LCBaseInfo{vp:unsafe{core::ptr::addr_of!(LC_CLASS_TI).cast::<usize>().add(2)},name:LC_BASE_NAME.as_ptr()};
#[export_name="_ZTIN3DSP14LLEAcceleratorE"]static LC_LLE_INFO:LCSiInfo=LCSiInfo{vp:unsafe{core::ptr::addr_of!(LC_SI_TI).cast::<usize>().add(2)},name:LC_LLE_NAME.as_ptr(),base:&LC_ACC_INFO};
#[repr(C)]struct LCVtable{
    offset:isize,ti:*const LCSiInfo,
    d1:unsafe extern "C-unwind" fn(*mut u8), d0:unsafe extern "C-unwind" fn(*mut u8),
    raw_read:unsafe extern "C-unwind" fn(*mut u8),raw_write:unsafe extern "C-unwind" fn(*mut u8),sample_read:unsafe extern "C-unwind" fn(*mut u8),
    read:unsafe extern "C-unwind" fn(*mut u8,u32)->u8,write:unsafe extern "C-unwind" fn(*mut u8,u32,u8),
}unsafe impl Sync for LCVtable{}
#[export_name="_ZTVN3DSP14LLEAcceleratorE"]static LC_VTABLE:LCVtable=LCVtable{offset:0,ti:&LC_LLE_INFO,d1:lc_lle_d1,d0:lc_lle_d0,raw_read:lc_raw_read,raw_write:lc_raw_write,sample_read:lc_sample_read,read:lc_read,write:lc_write};
#[export_name="_ZN3DSP14LLEAcceleratorD1Ev"]unsafe extern "C-unwind" fn lc_lle_d1(_: *mut u8){}
#[export_name="_ZN3DSP14LLEAcceleratorD0Ev"]unsafe extern "C-unwind" fn lc_lle_d0(p:*mut u8){op_delete_sized(p,48)}
#[export_name="_ZN3DSP14LLEAccelerator21OnRawReadEndExceptionEv"]unsafe extern "C-unwind" fn lc_raw_read(p:*mut u8){SDSP_SetException(f(p,40),3)}
#[export_name="_ZN3DSP14LLEAccelerator22OnRawWriteEndExceptionEv"]unsafe extern "C-unwind" fn lc_raw_write(p:*mut u8){SDSP_SetException(f(p,40),4)}
#[export_name="_ZN3DSP14LLEAccelerator24OnSampleReadEndExceptionEv"]unsafe extern "C-unwind" fn lc_sample_read(p:*mut u8){SDSP_SetException(f(p,40),5)}
#[export_name="_ZN3DSP14LLEAccelerator10ReadMemoryEj"]unsafe extern "C-unwind" fn lc_read(_: *mut u8,a:u32)->u8{lc_host_read(a)}
#[export_name="_ZN3DSP14LLEAccelerator11WriteMemoryEjh"]unsafe extern "C-unwind" fn lc_write(_: *mut u8,a:u32,v:u8){lc_host_write(v,a)}
#[export_name="wprt_claude_dsp_new_lle_accelerator"]pub unsafe extern "C-unwind" fn lc_new_lle(s:*mut u8)->*mut u8{
    let p=op_new(48);core::ptr::write_bytes(p.add(8),0,32);w(p,0,core::ptr::addr_of!(LC_VTABLE.d1));w(p,40,s);p
}
struct LCPartialSdsp(*mut u8);
impl Drop for LCPartialSdsp{fn drop(&mut self){unsafe{let p:*mut u8=f(self.0,ACCELERATOR);if !p.is_null(){delete_virtual(p)}}}}
unsafe fn lc_sdsp_construct(s:*mut u8,c:*mut u8){
    core::ptr::write_bytes(s,0,76);
    core::ptr::write_bytes(s.add(80),0,15);
    core::ptr::write_bytes(s.add(96),0,256);
    core::ptr::write_bytes(s.add(352),0,36);
    core::ptr::write_bytes(s.add(392),0,536);
    w(s,DSP_CORE,c);
    let cleanup=LCPartialSdsp(s);lc_analyzer_ctor(s.add(ANALYZER));core::mem::forget(cleanup);
}
#[export_name="_ZN3DSP4SDSPC1ERNS_7DSPCoreE"]pub unsafe extern "C-unwind" fn lc_sdsp_c1(s:*mut u8,c:*mut u8){lc_sdsp_construct(s,c)}
#[export_name="_ZN3DSP4SDSPC2ERNS_7DSPCoreE"]pub unsafe extern "C-unwind" fn lc_sdsp_c2(s:*mut u8,c:*mut u8){lc_sdsp_construct(s,c)}
unsafe fn lc_sdsp_destruct(s:*mut u8){lc_analyzer_dtor(s.add(ANALYZER));let p:*mut u8=f(s,ACCELERATOR);if !p.is_null(){delete_virtual(p)}}
#[export_name="_ZN3DSP4SDSPD1Ev"]pub unsafe extern "C-unwind" fn lc_sdsp_d1(s:*mut u8){lc_sdsp_destruct(s)}
#[export_name="_ZN3DSP4SDSPD2Ev"]pub unsafe extern "C-unwind" fn lc_sdsp_d2(s:*mut u8){lc_sdsp_destruct(s)}
struct LCCoreCleanup{c:*mut u8,interp:*mut u8,interp_ready:bool,mutex_ready:bool}
impl Drop for LCCoreCleanup{fn drop(&mut self){unsafe{
    if self.mutex_ready{pthread_mutex_destroy(self.c.add(LC_MUTEX));}
    if !self.interp.is_null(){if self.interp_ready{lc_interp_dtor(self.interp);}op_delete_sized(self.interp,40);}
    lc_sdsp_destruct(self.c);
}}}
unsafe fn lc_core_construct(c:*mut u8){
    lc_sdsp_construct(c,c);
    let mut guard=LCCoreCleanup{c,interp:core::ptr::null_mut(),interp_ready:false,mutex_ready:false};
    core::ptr::write_bytes(c.add(LC_BP),0,65536);w(c,CORE_STATE,0_u32);
    guard.interp=op_new(40);lc_interp_ctor(guard.interp,c);guard.interp_ready=true;
    w(c,CORE_INTERP,guard.interp);w(c,CORE_CAP,core::ptr::null_mut::<u8>());
    pthread_mutex_init(c.add(LC_MUTEX),core::ptr::null());guard.mutex_ready=true;
    lc_cond_ctor(c.add(LC_COND));w(c,LC_SET,false);core::mem::forget(guard);
}
#[export_name="_ZN3DSP7DSPCoreC1Ev"]pub unsafe extern "C-unwind" fn lc_core_c1(c:*mut u8){lc_core_construct(c)}
#[export_name="_ZN3DSP7DSPCoreC2Ev"]pub unsafe extern "C-unwind" fn lc_core_c2(c:*mut u8){lc_core_construct(c)}
unsafe fn lc_core_destruct(c:*mut u8){
    lc_cond_dtor(c.add(LC_COND));pthread_mutex_destroy(c.add(LC_MUTEX));
    let capture:*mut u8=f(c,CORE_CAP);if !capture.is_null(){delete_virtual(capture);}
    let interp:*mut u8=f(c,CORE_INTERP);if !interp.is_null(){lc_interp_dtor(interp);op_delete_sized(interp,40);}
    lc_sdsp_destruct(c);
}
#[export_name="_ZN3DSP7DSPCoreD1Ev"]pub unsafe extern "C-unwind" fn lc_core_d1(c:*mut u8){lc_core_destruct(c)}
#[export_name="_ZN3DSP7DSPCoreD2Ev"]pub unsafe extern "C-unwind" fn lc_core_d2(c:*mut u8){lc_core_destruct(c)}
#[repr(C)]struct LCUniqueLock{mutex:*mut u8,owned:bool}
impl Drop for LCUniqueLock{fn drop(&mut self){if self.owned && !self.mutex.is_null(){unsafe{pthread_mutex_unlock(self.mutex);}}}}
unsafe fn lc_lock(p:*mut u8)->LCUniqueLock{let e=pthread_mutex_lock(p);if e!=0{lc_system_error(e)}LCUniqueLock{mutex:p,owned:true}}
unsafe fn lc_set_event(c:*mut u8){let lock=lc_lock(c.add(LC_MUTEX));w(c,LC_SET,true);lc_cond_notify(c.add(LC_COND));drop(lock);}
#[export_name="_ZN3DSP7DSPCore4StepEv"]pub unsafe extern "C-unwind" fn lc_core_step(c:*mut u8){if f::<u32>(c,CORE_STATE)==2{lc_set_event(c)}}
#[export_name="_ZN3DSP7DSPCore8SetStateENS_5StateE"]pub unsafe extern "C-unwind" fn lc_core_state(c:*mut u8,s:u32){w(c,CORE_STATE,s);if s==1{lc_set_event(c)}}
#[export_name="_ZN3DSP7DSPCore9RunCyclesEi"]pub unsafe extern "C-unwind" fn lc_core_run(c:*mut u8,mut cycles:i32)->i32{
    while cycles>0{match f::<u32>(c,CORE_STATE){
        1=>cycles=lc_interp_cycles(f(c,CORE_INTERP),cycles),
        2=>{let mut lock=lc_lock(c.add(LC_MUTEX));while !f::<bool>(c,LC_SET){lc_cond_wait(c.add(LC_COND),&mut lock)}w(c,LC_SET,false);drop(lock);
            if f::<u32>(c,CORE_STATE)!=2{continue}lc_interp_step(f(c,CORE_INTERP));cycles-=1;},
        _=>{},
    }}cycles
}
