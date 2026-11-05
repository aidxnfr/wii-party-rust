// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of pristine engine/src/audio/dsp.cpp DSP::Host callbacks.
// PRIVATE UNBUILT/UNINTEGRATED. Native SDK opaque-state/descriptor/log/dump/existing verifier leaves retained.
use crate::memory::{host, rd16_reversed, rd8, wr16_reversed, wr8};
use std::ffi::c_void;
const MEM1_MASK: u32 = 0x01ffffff;
const MEM2_BASE: u32 = 0x10000000;
const MEM2_MASK: u32 = 0x03ffffff;
extern "C" {
    fn wphw_dsp_host_logging() -> bool;
    fn wphw_dsp_host_record_write(address: u32, value: u8);
    fn wphw_dsp_host_words(pointer: *const u16, size: u32) -> u32;
    fn wphw_dsp_host_self_testing() -> bool;
    fn wphw_dsp_host_code_begin() -> u32;
    fn wphw_dsp_host_crc(core: *mut c_void, crc: u32);
    fn wphw_dsp_host_code_count() -> usize;
    fn wphw_dsp_host_forced() -> bool;
    fn wphw_dsp_host_code_crc(index: usize) -> u32;
    fn wphw_dsp_host_code_select(index: usize);
    fn wphw_dsp_host_code_report(pointer: *const u8, size: usize, crc: u32);
    fn wphw_dsp_host_analyze(core: *mut c_void);
    fn wphw_dsp_host_existing_selftest();
}
// port of dsp.cpp DSP::Host::ReadHostMemory.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_read(address: u32) -> u8 {
    if address & MEM2_BASE != 0 {
        rd8(MEM2_BASE | (address & MEM2_MASK)) as u8
    } else {
        rd8(address & MEM1_MASK) as u8
    }
}
// port of dsp.cpp DSP::Host::WriteHostMemory. Quirk kept: writes ALWAYS use MEM2, unlike reads.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_write(value: u8, address: u32) {
    let target = MEM2_BASE | (address & MEM2_MASK);
    if wphw_dsp_host_logging() {
        wphw_dsp_host_record_write(target, value);
    }
    wr8(target, value as u32);
}
// port of dsp.cpp DSP::Host::dma_address.
fn dma_address(address: u32) -> u32 {
    if address & MEM2_BASE != 0 {
        MEM2_BASE | (address & MEM2_MASK)
    } else {
        address & MEM1_MASK
    }
}
// port of dsp.cpp DSP::Host::DMAToDSP. SDK-only word-range lookup is native, every endian conversion/order is unchanged.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_to_dsp(destination: *mut u16, address: u32, size: u32) {
    let words = wphw_dsp_host_words(destination, size);
    for i in 0..words {
        *destination.add(i as usize) =
            (rd16_reversed(dma_address(address.wrapping_add(2 * i))) as u16).swap_bytes();
    }
}
// port of dsp.cpp DSP::Host::DMAFromDSP. Record ALL old values before any source word is written back.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_from_dsp(source: *const u16, address: u32, size: u32) {
    let words = wphw_dsp_host_words(source, size);
    if !wphw_dsp_host_self_testing() {
        crate::native_dsp::wprt_dsp_note_write(dma_address(address), words * 2);
    }
    if wphw_dsp_host_logging() {
        for i in 0..words {
            let value = *source.add(i as usize);
            let a = dma_address(address.wrapping_add(2 * i));
            wphw_dsp_host_record_write(a, (value >> 8) as u8);
            wphw_dsp_host_record_write(a.wrapping_add(1), value as u8);
        }
    }
    for i in 0..words {
        wr16_reversed(
            dma_address(address.wrapping_add(2 * i)),
            (*source.add(i as usize)).swap_bytes() as u32,
        );
    }
}
// port of dsp.cpp DSP::Host::OnThread/IsWiiHost/InterruptRequest/TimeBase.
#[no_mangle]
pub extern "C" fn wprt_dsp_host_on_thread() -> bool {
    false
}
#[no_mangle]
pub extern "C" fn wprt_dsp_host_is_wii() -> bool {
    true
}
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_interrupt() {
    crate::native_dsp::wprt_dsp_raise(0x80);
}
#[no_mangle]
pub extern "C" fn wprt_dsp_host_time_base() -> u64 {
    crate::dispatch::time_base()
}
// port of dsp.cpp DSP::Host::CodeLoaded(address overload).
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_code_address(core: *mut c_void, address: u32, size: usize) {
    wprt_dsp_host_code(core, host(address), size);
}
// port of dsp.cpp DSP::Host::CodeLoaded(pointer overload), preserving restored/self-test early returns and LAST matching descriptor.
#[no_mangle]
pub unsafe extern "C" fn wprt_dsp_host_code(core: *mut c_void, pointer: *const u8, size: usize) {
    if wphw_dsp_host_code_begin() != 2 {
        return;
    }
    let mut crc = 0u32;
    for i in 0..size {
        crc = crc.wrapping_mul(31) ^ *pointer.add(i) as u32;
    }
    wphw_dsp_host_crc(core, crc);
    let mut i = 0;
    while i < wphw_dsp_host_code_count() && !wphw_dsp_host_forced() {
        if wphw_dsp_host_code_crc(i) == crc {
            wphw_dsp_host_code_select(i);
        }
        i += 1;
    }
    wphw_dsp_host_code_report(pointer, size, crc);
    wphw_dsp_host_analyze(core);
    wphw_dsp_host_existing_selftest();
}
