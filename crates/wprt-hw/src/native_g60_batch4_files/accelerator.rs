// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted (Claude, G60) WHOLE DSPAccelerator.cpp replacement: exports the exact GNU member symbols of
// DSP::Accelerator; fields/vtable slots from the pinned compiler (layout.rs, probe.cpp). Logging macros are
// ((void)0) in the shim, so no log calls exist. PointerWrap = dolphin_shim/Common/ChunkFile.h (vector<u8>* buffer, Mode, position).
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("DSPAccelerator member exports require pinned Windows GNU x64 metadata");
mod layout { include!(env!("WPRT_DSP_ACC_LAYOUT")); }
use layout::*;
const _: () = assert!(SIZE == 40 && ALIGN == 8 && BOOL_SIZE == 1);

unsafe fn f<T: Copy>(this: *mut u8, off: usize) -> T { this.add(off).cast::<T>().read() }
unsafe fn w<T: Copy>(this: *mut u8, off: usize, v: T) { this.add(off).cast::<T>().write(v) }
unsafe fn slot(this: *mut u8, i: usize) -> *const () { (*this.cast::<*const *const ()>()).add(i).read() }
unsafe fn read_memory(this: *mut u8, a: u32) -> u8 { let p: unsafe extern "C-unwind" fn(*mut u8, u32) -> u8 = std::mem::transmute(slot(this, VT_READ_MEMORY)); p(this, a) }
unsafe fn write_memory(this: *mut u8, a: u32, v: u8) { let p: unsafe extern "C-unwind" fn(*mut u8, u32, u8) = std::mem::transmute(slot(this, VT_WRITE_MEMORY)); p(this, a, v) }
unsafe fn vcall(this: *mut u8, i: usize) { let p: unsafe extern "C-unwind" fn(*mut u8) = std::mem::transmute(slot(this, i)); p(this) }

#[export_name = "_ZN3DSP11Accelerator16GetCurrentSampleEv"]
pub unsafe extern "C-unwind" fn GetCurrentSample(this: *mut u8) -> u16 {
    // Every member read happens after the previous virtual call, as in the C++ (a ReadMemory override may change `this`).
    match f::<u16>(this, FORMAT) & 3 {
        0 => { let v = read_memory(this, f::<u32>(this, CURRENT) >> 1) as u16; if f::<u32>(this, CURRENT) & 1 != 0 { v & 0xf } else { v >> 4 } }
        1 => read_memory(this, f::<u32>(this, CURRENT)) as u16,
        2 => { let hi = read_memory(this, f::<u32>(this, CURRENT).wrapping_mul(2)) as u16; let lo = read_memory(this, f::<u32>(this, CURRENT).wrapping_mul(2).wrapping_add(1)) as u16; (hi << 8) | lo }
        _ => 0,
    }
}

#[export_name = "_ZN3DSP11Accelerator7ReadRawEv"]
pub unsafe extern "C-unwind" fn ReadRaw(this: *mut u8) -> u16 {
    let val = GetCurrentSample(this);
    let ca: u32 = f(this, CURRENT);
    if f::<u16>(this, FORMAT) & 3 != 3 { w(this, CURRENT, ca.wrapping_add(1)); } else { w(this, CURRENT, (ca & !3) | (ca.wrapping_add(1) & 3)); }
    if f::<u32>(this, CURRENT).wrapping_sub(1) == f::<u32>(this, END) {
        w(this, CURRENT, f::<u32>(this, START));
        vcall(this, VT_RAW_READ_END);
    }
    SetCurrentAddress(this, f(this, CURRENT));
    val
}

#[export_name = "_ZN3DSP11Accelerator8WriteRawEt"]
pub unsafe extern "C-unwind" fn WriteRaw(this: *mut u8, value: u16) {
    let ca: u32 = f(this, CURRENT);
    if ca & 0x8000_0000 != 0 {
        write_memory(this, ca.wrapping_mul(2), (value >> 8) as u8);
        write_memory(this, f::<u32>(this, CURRENT).wrapping_mul(2).wrapping_add(1), (value & 0xff) as u8);
        w(this, CURRENT, f::<u32>(this, CURRENT).wrapping_add(1));
        vcall(this, VT_RAW_WRITE_END);
    }
}

#[export_name = "_ZN3DSP11Accelerator10ReadSampleEPKs"]
pub unsafe extern "C-unwind" fn ReadSample(this: *mut u8, coefs: *const i16) -> u16 {
    if f::<u8>(this, STOPPED) != 0 { return 0; }
    let fmt0: u16 = f(this, FORMAT);
    let decode0 = (fmt0 >> 2) & 3;
    let mut raw: i16 = if decode0 == 1 || decode0 == 3 { f::<u16>(this, INPUT) as i16 } else { GetCurrentSample(this) as i16 };
    let pred: u16 = f(this, PRED);
    let idx = ((pred >> 4) & 7) as usize;
    let c1 = *coefs.add(idx * 2) as i32;
    let c2 = *coefs.add(idx * 2 + 1) as i32;
    let mut val: u16 = 0;
    let mut step: u8 = 0;
    let fmt: u16 = f(this, FORMAT);
    match (fmt >> 2) & 3 {
        0 => {
            raw &= 0xf;
            let scale = 1i32 << (f::<u16>(this, PRED) & 0xf);
            if raw >= 8 { raw -= 16; }
            let yn1 = f::<i16>(this, YN1) as i32; let yn2 = f::<i16>(this, YN2) as i32;
            let v32 = scale.wrapping_mul(raw as i32).wrapping_add(0x400i32.wrapping_add(c1.wrapping_mul(yn1)).wrapping_add(c2.wrapping_mul(yn2)) >> 11);
            val = v32.clamp(i16::MIN as i32, i16::MAX as i32) as i16 as u16;
            step = 2;
            w(this, YN2, f::<i16>(this, YN1));
            w(this, YN1, val as i16);
            w(this, CURRENT, f::<u32>(this, CURRENT).wrapping_add(1));
            let ea: u32 = f(this, END); let ca: u32 = f(this, CURRENT);
            if ea & 0xf == 0 && ca == ea {
                w(this, CURRENT, f::<u32>(this, START).wrapping_add(1));
            } else if ea & 0xf == 1 && ca == ea.wrapping_sub(1) {
                w(this, CURRENT, f::<u32>(this, START));
            } else if ca & 15 == 0 {
                w(this, PRED, read_memory(this, (ca & !15) >> 1) as u16);
                w(this, CURRENT, f::<u32>(this, CURRENT).wrapping_add(2));
                step += 2;
            }
        }
        _ => {
            let shift: u32 = match (fmt >> 4) & 3 { 0 => 11, 1 => 0, 2 => 16, _ => 0 };
            let yn1 = f::<i16>(this, YN1) as i32; let yn2 = f::<i16>(this, YN2) as i32;
            let v32 = ((f::<i16>(this, GAIN) as i32).wrapping_mul(raw as i32) >> shift)
                .wrapping_add((c1.wrapping_mul(yn1) >> shift).wrapping_add(c2.wrapping_mul(yn2) >> shift));
            val = v32 as i16 as u16;
            w(this, YN2, f::<i16>(this, YN1));
            w(this, YN1, val as i16);
            step = 2;
            if (fmt >> 2) & 3 != 1 { w(this, CURRENT, f::<u32>(this, CURRENT).wrapping_add(1)); }
        }
    }
    if f::<u32>(this, CURRENT) == f::<u32>(this, END).wrapping_add(step as u32).wrapping_sub(1) {
        w(this, CURRENT, f::<u32>(this, START));
        w(this, STOPPED, 1u8);
        vcall(this, VT_SAMPLE_READ_END);
    }
    SetCurrentAddress(this, f(this, CURRENT));
    val
}

// ---- PointerWrap (dolphin_shim) : { std::vector<u8>* m_buffer; Mode m_mode; size_t m_position } ----
extern "C-unwind" {
    #[link_name = "_Znwy"] fn op_new(n: usize) -> *mut u8;
    #[link_name = "_ZdlPvy"] fn op_delete_sized(p: *mut u8, n: usize);
    #[link_name = "_ZSt20__throw_length_errorPKc"] fn throw_length_error(what: *const u8) -> !;
}
#[repr(C)] struct Vec8 { begin: *mut u8, finish: *mut u8, eos: *mut u8 }
#[repr(C)] struct PW { buffer: *mut Vec8, mode: i32, position: usize }
const _: () = assert!(std::mem::size_of::<PW>() == 24);
// std::vector<u8>::insert(end(), first, last) (libstdc++ _M_range_insert, forward iterators, GCC pinned).
unsafe fn vec_append(v: *mut Vec8, src: *const u8, n: usize) {
    if n == 0 { return; }
    let v = &mut *v;
    let size = v.finish as usize - v.begin as usize;
    let cap = v.eos as usize - v.begin as usize;
    if cap - size >= n {
        std::ptr::copy_nonoverlapping(src, v.finish, n);
        v.finish = v.finish.add(n);
        return;
    }
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
unsafe fn pw_bytes(p: *mut PW, data: *mut u8, size: usize) {
    let p = &mut *p;
    if p.buffer.is_null() { return; }
    if p.mode == 1 {
        vec_append(p.buffer, data, size);
    } else if p.mode == 2 {
        let b = &*p.buffer;
        let len = b.finish as usize - b.begin as usize;
        if p.position + size <= len { std::ptr::copy_nonoverlapping(b.begin.add(p.position), data, size); }
    }
    p.position += size;
}
#[export_name = "_ZN3DSP11Accelerator7DoStateER11PointerWrap"]
pub unsafe extern "C-unwind" fn DoState(this: *mut u8, p: *mut u8) {
    let p = p.cast::<PW>();
    pw_bytes(p, this.add(START), 4);
    pw_bytes(p, this.add(END), 4);
    pw_bytes(p, this.add(CURRENT), 4);
    pw_bytes(p, this.add(FORMAT), 2);
    pw_bytes(p, this.add(YN1), 2);
    pw_bytes(p, this.add(YN2), 2);
    pw_bytes(p, this.add(PRED), 2);
    pw_bytes(p, this.add(STOPPED), 1);
}

#[export_name = "_ZN3DSP11Accelerator15SetStartAddressEj"] pub unsafe extern "C-unwind" fn SetStartAddress(this: *mut u8, a: u32) { w(this, START, a & 0x3fff_ffff) }
#[export_name = "_ZN3DSP11Accelerator13SetEndAddressEj"] pub unsafe extern "C-unwind" fn SetEndAddress(this: *mut u8, a: u32) { w(this, END, a & 0x3fff_ffff) }
#[export_name = "_ZN3DSP11Accelerator17SetCurrentAddressEj"] pub unsafe extern "C-unwind" fn SetCurrentAddress(this: *mut u8, a: u32) { w(this, CURRENT, a & 0xbfff_ffff) }
#[export_name = "_ZN3DSP11Accelerator15SetSampleFormatEt"] pub unsafe extern "C-unwind" fn SetSampleFormat(this: *mut u8, v: u16) { w(this, FORMAT, v) }
#[export_name = "_ZN3DSP11Accelerator7SetGainEs"] pub unsafe extern "C-unwind" fn SetGain(this: *mut u8, v: i16) { w(this, GAIN, v) }
#[export_name = "_ZN3DSP11Accelerator6SetYn1Es"] pub unsafe extern "C-unwind" fn SetYn1(this: *mut u8, v: i16) { w(this, YN1, v) }
#[export_name = "_ZN3DSP11Accelerator6SetYn2Es"] pub unsafe extern "C-unwind" fn SetYn2(this: *mut u8, v: i16) { w(this, YN2, v); w(this, STOPPED, 0u8) }
#[export_name = "_ZN3DSP11Accelerator12SetPredScaleEt"] pub unsafe extern "C-unwind" fn SetPredScale(this: *mut u8, v: u16) { w(this, PRED, v & 0x7f) }
#[export_name = "_ZN3DSP11Accelerator8SetInputEt"] pub unsafe extern "C-unwind" fn SetInput(this: *mut u8, v: u16) { w(this, INPUT, v) }
