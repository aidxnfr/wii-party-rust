// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted (Claude, G60) WHOLE DSPHWInterface.cpp replacement: the 14 SDSP member symbols of the pristine object (nm.txt).
// Layout/constants from probe.cpp (pinned headers + MinGW g++). Shim MemoryUtil (Un)WriteProtectMemory are inline no-ops and
// all log macros are ((void)0), so neither produces calls. Accelerator getters are inline field reads; other Accelerator,
// DSPCore::Log*, Host::* calls go to the same external GNU symbols as the C++ object (nm-undef.txt).
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("DSPHWInterface exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod layout { include!(env!("WPRT_DSP_HW_LAYOUT")); }
use layout::*;
use std::sync::atomic::{AtomicU32, Ordering};
const _: () = assert!(ATOMIC_U32_SIZE == 4 && UNIQUE_PTR_SIZE == 8 && MB_CPU == 0 && MB_DSP == 1);

extern "C-unwind" {
    #[link_name = "_ZN3DSP11Accelerator10ReadSampleEPKs"] fn acc_read_sample(a: *mut u8, coefs: *const i16) -> u16;
    #[link_name = "_ZN3DSP11Accelerator12SetPredScaleEt"] fn acc_set_pred_scale(a: *mut u8, v: u16);
    #[link_name = "_ZN3DSP11Accelerator13SetEndAddressEj"] fn acc_set_end(a: *mut u8, v: u32);
    #[link_name = "_ZN3DSP11Accelerator15SetSampleFormatEt"] fn acc_set_format(a: *mut u8, v: u16);
    #[link_name = "_ZN3DSP11Accelerator15SetStartAddressEj"] fn acc_set_start(a: *mut u8, v: u32);
    #[link_name = "_ZN3DSP11Accelerator17SetCurrentAddressEj"] fn acc_set_current(a: *mut u8, v: u32);
    #[link_name = "_ZN3DSP11Accelerator6SetYn1Es"] fn acc_set_yn1(a: *mut u8, v: i16);
    #[link_name = "_ZN3DSP11Accelerator6SetYn2Es"] fn acc_set_yn2(a: *mut u8, v: i16);
    #[link_name = "_ZN3DSP11Accelerator7ReadRawEv"] fn acc_read_raw(a: *mut u8) -> u16;
    #[link_name = "_ZN3DSP11Accelerator7SetGainEs"] fn acc_set_gain(a: *mut u8, v: i16);
    #[link_name = "_ZN3DSP11Accelerator8SetInputEt"] fn acc_set_input(a: *mut u8, v: u16);
    #[link_name = "_ZN3DSP11Accelerator8WriteRawEt"] fn acc_write_raw(a: *mut u8, v: u16);
    #[link_name = "_ZN3DSP4Host10CodeLoadedERNS_7DSPCoreEjy"] fn host_code_loaded(core: *mut u8, addr: u32, size: u64);
    #[link_name = "_ZN3DSP4Host10DMAFromDSPEPKtjj"] fn host_dma_from_dsp(src: *const u16, addr: u32, size: u32);
    #[link_name = "_ZN3DSP4Host16InterruptRequestEv"] fn host_interrupt_request();
    #[link_name = "_ZN3DSP4Host8DMAToDSPEPtjj"] fn host_dma_to_dsp(dst: *mut u16, addr: u32, size: u32);
    #[link_name = "_ZN3DSP7DSPCore10LogIFXReadEtt"] fn core_log_ifx_read(core: *mut u8, address: u16, value: u16);
    #[link_name = "_ZN3DSP7DSPCore11LogIFXWriteEtt"] fn core_log_ifx_write(core: *mut u8, address: u16, value: u16);
    #[link_name = "_ZN3DSP7DSPCore6LogDMAEtjttPKh"] fn core_log_dma(core: *mut u8, control: u16, gc: u32, dsp: u16, len: u16, data: *const u8);
}

unsafe fn mailbox<'a>(s: *mut u8, m: u32) -> &'a AtomicU32 { &*s.add(MAILBOX + 4 * m as usize).cast::<AtomicU32>() }
unsafe fn regs(s: *mut u8) -> *mut u16 { s.add(IFX_REGS).cast::<u16>() }
unsafe fn reg(s: *mut u8, i: u32) -> u16 { regs(s).add(i as usize).read() }
unsafe fn set_reg(s: *mut u8, i: u32, v: u16) { regs(s).add(i as usize).write(v) }
unsafe fn acc(s: *mut u8) -> *mut u8 { s.add(ACCELERATOR).cast::<*mut u8>().read() }
unsafe fn core(s: *mut u8) -> *mut u8 { s.add(DSP_CORE).cast::<*mut u8>().read() }
unsafe fn accf<T: Copy>(a: *mut u8, off: usize) -> T { a.add(off).cast::<T>().read() }

#[export_name = "_ZN3DSP4SDSP13InitializeIFXEv"]
pub unsafe extern "C-unwind" fn InitializeIFX(s: *mut u8) {
    std::ptr::write_bytes(regs(s), 0, 256);
    mailbox(s, MB_CPU as u32).store(0, Ordering::SeqCst);
    mailbox(s, MB_DSP as u32).store(0, Ordering::SeqCst);
}
#[export_name = "_ZNK3DSP4SDSP11PeekMailboxENS_7MailboxE"]
pub unsafe extern "C-unwind" fn PeekMailbox(s: *mut u8, m: u32) -> u32 { mailbox(s, m).load(Ordering::SeqCst) }
#[export_name = "_ZN3DSP4SDSP14ReadMailboxLowENS_7MailboxE"]
pub unsafe extern "C-unwind" fn ReadMailboxLow(s: *mut u8, m: u32) -> u16 {
    let value = mailbox(s, m).load(Ordering::Acquire);
    mailbox(s, m).store(value & !0x8000_0000, Ordering::Release);
    value as u16
}
#[export_name = "_ZN3DSP4SDSP15ReadMailboxHighENS_7MailboxE"]
pub unsafe extern "C-unwind" fn ReadMailboxHigh(s: *mut u8, m: u32) -> u16 { (PeekMailbox(s, m) >> 16) as u16 }
#[export_name = "_ZN3DSP4SDSP15WriteMailboxLowENS_7MailboxEt"]
pub unsafe extern "C-unwind" fn WriteMailboxLow(s: *mut u8, m: u32, value: u16) {
    let old = mailbox(s, m).load(Ordering::Acquire);
    let new = (old & !0xffff) | value as u32;
    mailbox(s, m).store(new | 0x8000_0000, Ordering::Release);
}
#[export_name = "_ZN3DSP4SDSP16WriteMailboxHighENS_7MailboxEt"]
pub unsafe extern "C-unwind" fn WriteMailboxHigh(s: *mut u8, m: u32, value: u16) {
    let old = mailbox(s, m).load(Ordering::Acquire);
    let new = (old & 0xffff) | ((value as u32) << 16);
    mailbox(s, m).store(new & !0x8000_0000, Ordering::Release);
}

#[export_name = "_ZN3DSP4SDSP8WriteIFXEjt"]
pub unsafe extern "C-unwind" fn WriteIFX(s: *mut u8, address: u32, value: u16) {
    core_log_ifx_write(core(s), address as u16, value);
    let a = acc(s);
    match address & 0xff {
        DSP_DIRQ => { if value & 1 != 0 { host_interrupt_request(); } }
        DSP_DMBH => WriteMailboxHigh(s, MB_DSP as u32, value),
        DSP_DMBL => WriteMailboxLow(s, MB_DSP as u32, value),
        DSP_CMBH => WriteMailboxHigh(s, MB_CPU as u32, value),
        DSP_CMBL => WriteMailboxLow(s, MB_CPU as u32, value),
        DSP_DSBL => {
            set_reg(s, DSP_DSBL, value);
            set_reg(s, DSP_DSCR, reg(s, DSP_DSCR) | 4);
            if reg(s, DSP_AMDM) == 0 { DoDMA(s); }
            set_reg(s, DSP_DSCR, reg(s, DSP_DSCR) & !4);
            set_reg(s, DSP_DSBL, 0);
        }
        DSP_DSPA | DSP_DSMAH | DSP_DSMAL | DSP_DSCR => set_reg(s, address & 0xff, value),
        DSP_ACSAH => acc_set_start(a, ((value as u32) << 16) | accf::<u32>(a, ACC_START) as u16 as u32),
        DSP_ACSAL => acc_set_start(a, (((accf::<u32>(a, ACC_START) >> 16) as u16 as u32) << 16) | value as u32),
        DSP_ACEAH => acc_set_end(a, ((value as u32) << 16) | accf::<u32>(a, ACC_END) as u16 as u32),
        DSP_ACEAL => acc_set_end(a, (((accf::<u32>(a, ACC_END) >> 16) as u16 as u32) << 16) | value as u32),
        DSP_ACCAH => acc_set_current(a, ((value as u32) << 16) | accf::<u32>(a, ACC_CURRENT) as u16 as u32),
        DSP_ACCAL => acc_set_current(a, (((accf::<u32>(a, ACC_CURRENT) >> 16) as u16 as u32) << 16) | value as u32),
        DSP_FORMAT => acc_set_format(a, value),
        DSP_GAIN => acc_set_gain(a, value as i16),
        DSP_YN1 => acc_set_yn1(a, value as i16),
        DSP_YN2 => acc_set_yn2(a, value as i16),
        DSP_PRED_SCALE => acc_set_pred_scale(a, value),
        DSP_ACDRAW => acc_write_raw(a, value),
        DSP_ACIN => acc_set_input(a, value),
        _ => set_reg(s, address & 0xff, value),
    }
}

#[export_name = "_ZN3DSP4SDSP11ReadIFXImplEt"]
pub unsafe extern "C-unwind" fn ReadIFXImpl(s: *mut u8, address: u16) -> u16 {
    let a = acc(s);
    match (address & 0xff) as u32 {
        DSP_DMBH => ReadMailboxHigh(s, MB_DSP as u32),
        DSP_DMBL => ReadMailboxLow(s, MB_DSP as u32),
        DSP_CMBH => ReadMailboxHigh(s, MB_CPU as u32),
        DSP_CMBL => ReadMailboxLow(s, MB_CPU as u32),
        DSP_DSCR => reg(s, (address & 0xff) as u32),
        DSP_ACSAH => (accf::<u32>(a, ACC_START) >> 16) as u16,
        DSP_ACSAL => accf::<u32>(a, ACC_START) as u16,
        DSP_ACEAH => (accf::<u32>(a, ACC_END) >> 16) as u16,
        DSP_ACEAL => accf::<u32>(a, ACC_END) as u16,
        DSP_ACCAH => (accf::<u32>(a, ACC_CURRENT) >> 16) as u16,
        DSP_ACCAL => accf::<u32>(a, ACC_CURRENT) as u16,
        DSP_FORMAT => accf::<u16>(a, ACC_FORMAT),
        DSP_GAIN => accf::<i16>(a, ACC_GAIN) as u16,
        DSP_YN1 => accf::<i16>(a, ACC_YN1) as u16,
        DSP_YN2 => accf::<i16>(a, ACC_YN2) as u16,
        DSP_PRED_SCALE => accf::<u16>(a, ACC_PRED),
        DSP_ACDSAMP => acc_read_sample(a, regs(s).add(DSP_COEF_A1_0 as usize).cast::<i16>()),
        DSP_ACDRAW => acc_read_raw(a),
        DSP_ACIN => accf::<u16>(a, ACC_INPUT),
        _ => reg(s, (address & 0xff) as u32),
    }
}
#[export_name = "_ZN3DSP4SDSP7ReadIFXEt"]
pub unsafe extern "C-unwind" fn ReadIFX(s: *mut u8, address: u16) -> u16 {
    let retval = ReadIFXImpl(s, address);
    core_log_ifx_read(core(s), address, retval);
    retval
}

unsafe fn iram(s: *mut u8) -> *mut u16 { s.add(IRAM).cast::<*mut u16>().read() }
unsafe fn dram(s: *mut u8) -> *mut u16 { s.add(DRAM).cast::<*mut u16>().read() }
#[export_name = "_ZN3DSP4SDSP6IDMAInEtjj"]
pub unsafe extern "C-unwind" fn IDMAIn(s: *mut u8, dsp_addr: u16, addr: u32, size: u32) -> *const u8 {
    host_dma_to_dsp(iram(s).wrapping_add((dsp_addr / 2) as usize), addr, size);
    host_code_loaded(core(s), addr, size as u64);
    iram(s).cast::<u8>().wrapping_add(dsp_addr as usize)
}
#[export_name = "_ZN3DSP4SDSP7IDMAOutEtjj"]
pub unsafe extern "C-unwind" fn IDMAOut(_s: *mut u8, _dsp_addr: u16, _addr: u32, _size: u32) -> *const u8 { std::ptr::null() }
#[export_name = "_ZN3DSP4SDSP6DDMAInEtjj"]
pub unsafe extern "C-unwind" fn DDMAIn(s: *mut u8, dsp_addr: u16, addr: u32, size: u32) -> *const u8 {
    host_dma_to_dsp(dram(s).wrapping_add((dsp_addr / 2) as usize), addr, size);
    dram(s).cast::<u8>().wrapping_add(dsp_addr as usize)
}
#[export_name = "_ZN3DSP4SDSP7DDMAOutEtjj"]
pub unsafe extern "C-unwind" fn DDMAOut(s: *mut u8, dsp_addr: u16, addr: u32, size: u32) -> *const u8 {
    host_dma_from_dsp(dram(s).wrapping_add((dsp_addr / 2) as usize), addr, size);
    dram(s).cast::<u8>().wrapping_add(dsp_addr as usize)
}
#[export_name = "_ZN3DSP4SDSP5DoDMAEv"]
pub unsafe extern "C-unwind" fn DoDMA(s: *mut u8) {
    let addr = ((reg(s, DSP_DSMAH) as u32) << 16) | reg(s, DSP_DSMAL) as u32;
    let ctl = reg(s, DSP_DSCR);
    let dsp_addr = (reg(s, DSP_DSPA) as u32 * 2) as u16;
    let len = reg(s, DSP_DSBL);
    if len > 0x4000 { return; }
    let p = match (ctl & 3) as u32 {
        x if x == (DSP_CR_DMEM | DSP_CR_TO_CPU) => DDMAOut(s, dsp_addr, addr, len as u32),
        x if x == (DSP_CR_DMEM | DSP_CR_FROM_CPU) => DDMAIn(s, dsp_addr, addr, len as u32),
        x if x == (DSP_CR_IMEM | DSP_CR_TO_CPU) => IDMAOut(s, dsp_addr, addr, len as u32),
        _ => IDMAIn(s, dsp_addr, addr, len as u32),
    };
    if !p.is_null() { core_log_dma(core(s), ctl, addr, dsp_addr, len, p); }
}
