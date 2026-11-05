// SPDX-License-Identifier: GPL-3.0-or-later
// Rust port of arelkair/wiiparty-recomp engine/include/wp/memory.h @ e8522be (Claude, 2026-10-23).
// Same address map: MEM1/MEM2 mirrored by masking with 0x1FFFFFFF, locked cache at 0xE0000000,
// accesses at or above 0xCC000000 go to the MMIO handlers (hardware registers).

pub const ADDRESS_MASK: u32 = 0x1FFF_FFFF;
pub const LOCKED_CACHE_BASE: u32 = 0xE000_0000;
pub const LOCKED_CACHE_SIZE: u32 = 0x4000;
pub const PHYSICAL_SIZE: u32 = 0x1400_0000;
pub const MEMORY_SIZE: usize = PHYSICAL_SIZE as usize + LOCKED_CACHE_SIZE as usize;
pub const FIFO_ADDRESS: u32 = 0xCC00_8000;
pub const SLOW_ADDRESSES: u32 = 0xCC00_0000;

static mut MEMORY: *mut u8 = std::ptr::null_mut();

/// Harness-only write trace (feature "trace"): an FNV hash of every guest write, in order.
#[cfg(feature = "trace")] pub static mut WRITE_HASH: u64 = 0;
#[cfg(feature = "trace")] pub static mut WRITE_COUNT: u64 = 0;
#[cfg(feature = "trace")] pub static mut WRITE_LOG: bool = false;
#[cfg(feature = "trace")] pub static mut NAN_CANON: bool = false;
#[cfg(feature = "trace")] static mut UNDO: Vec<(*mut u8, [u8; 32], usize)> = Vec::new();
/// Harness: restore every byte written since the last call (newest first).
#[cfg(feature = "trace")]
pub fn undo_all() { unsafe { let u = &mut *std::ptr::addr_of_mut!(UNDO); while let Some((p, old, n)) = u.pop() { std::ptr::copy_nonoverlapping(old.as_ptr(), p, n); } } }
#[cfg(feature = "trace")]
#[inline(always)]
fn trace_write(a: u32, size: u32, v: u64) {
    unsafe {
        let n = (size & 0xff) as usize;
        let p = if size >= 0x100 { host(a) } else { MEMORY.add((a & ADDRESS_MASK) as usize) };
        let mut old = [0u8; 32]; std::ptr::copy_nonoverlapping(p, old.as_mut_ptr(), n);
        (*std::ptr::addr_of_mut!(UNDO)).push((p, old, n));
        let mut v = v;
        if NAN_CANON {
            if (size == 4 || size == 0x104) && (v & 0x7F800000) == 0x7F800000 && (v & 0x7FFFFF) != 0 { v = 0x7FC00000; }
            if size == 8 && (v & 0x7FF0000000000000) == 0x7FF0000000000000 && (v & 0xFFFFFFFFFFFFF) != 0 { v = 0x7FF8000000000000; }
        }
        WRITE_HASH = (WRITE_HASH ^ (a as u64 | ((size as u64) << 32))).wrapping_mul(0x100000001b3);
        WRITE_HASH = (WRITE_HASH ^ v).wrapping_mul(0x100000001b3); WRITE_COUNT += 1;
        if WRITE_LOG { println!("W {a:08x} {size:x} {v:x}"); }
    }
}
#[cfg(not(feature = "trace"))] #[inline(always)] fn trace_write(_a: u32, _size: u32, _v: u64) {}
/// Raw pointer to guest memory (harness and hardware layer).
pub fn base_ptr() -> *mut u8 { unsafe { MEMORY } }

/// Allocates guest memory once (zeroed). Must be called before any access.
pub fn init() { init_sized(MEMORY_SIZE) }
/// Test harnesses only: a larger buffer so any masked address (up to 0x1FFFFFFF + 8) stays inside the allocation.
pub fn init_sized(bytes: usize) {
    unsafe {
        if MEMORY.is_null() {
            let v = vec![0u8; bytes].into_boxed_slice();
            MEMORY = Box::leak(v).as_mut_ptr();
        }
    }
}

/// MMIO handlers, installed by the hardware layer (GX FIFO, VI, PI, DSP, EXI, ...).
pub struct Mmio {
    pub read16: fn(u32) -> u16,
    pub read32: fn(u32) -> u32,
    pub write8: fn(u32, u8),
    pub write16: fn(u32, u16),
    pub write32: fn(u32, u32),
    pub write64: fn(u32, u64),
}
fn no_r16(a: u32) -> u16 { panic!("mmio read16 {a:#x} before the hardware layer is installed") }
fn no_r32(a: u32) -> u32 { panic!("mmio read32 {a:#x} before the hardware layer is installed") }
fn no_w8(a: u32, _: u8) { panic!("mmio write8 {a:#x} before the hardware layer is installed") }
fn no_w16(a: u32, _: u16) { panic!("mmio write16 {a:#x} before the hardware layer is installed") }
fn no_w32(a: u32, _: u32) { panic!("mmio write32 {a:#x} before the hardware layer is installed") }
fn no_w64(a: u32, _: u64) { panic!("mmio write64 {a:#x} before the hardware layer is installed") }
static mut MMIO: Mmio = Mmio { read16: no_r16, read32: no_r32, write8: no_w8, write16: no_w16, write32: no_w32, write64: no_w64 };
pub fn set_mmio(m: Mmio) { unsafe { MMIO = m; } }
#[inline(always)] fn mmio() -> &'static Mmio { unsafe { &*std::ptr::addr_of!(MMIO) } }

#[inline(always)]
pub const fn slow_address(address: u32) -> bool { address >= SLOW_ADDRESSES }

#[inline(always)]
pub fn host(address: u32) -> *mut u8 {
    unsafe {
        if address & 0xF000_0000 == LOCKED_CACHE_BASE {
            MEMORY.add(PHYSICAL_SIZE as usize + (address & (LOCKED_CACHE_SIZE - 1)) as usize)
        } else {
            MEMORY.add((address & ADDRESS_MASK) as usize)
        }
    }
}
#[inline(always)]
fn plain(address: u32) -> *mut u8 { unsafe { MEMORY.add((address & ADDRESS_MASK) as usize) } }

#[inline(always)] pub fn rd8(address: u32) -> u32 { unsafe { *host(address) as u32 } }
#[inline(always)]
pub fn rd16(address: u32) -> u32 {
    if slow_address(address) { return (mmio().read16)(address) as u32; }
    unsafe { u16::from_be_bytes(*(plain(address) as *const [u8; 2])) as u32 }
}
#[inline(always)]
pub fn rd32(address: u32) -> u32 {
    if slow_address(address) { return (mmio().read32)(address); }
    unsafe { u32::from_be_bytes(*(plain(address) as *const [u8; 4])) }
}
#[inline(always)] pub fn rd64(address: u32) -> u64 { unsafe { u64::from_be_bytes(*(host(address) as *const [u8; 8])) } }
#[inline(always)] pub fn rd16_reversed(address: u32) -> u32 { unsafe { u16::from_le_bytes(*(host(address) as *const [u8; 2])) as u32 } }
#[inline(always)] pub fn rd32_reversed(address: u32) -> u32 { unsafe { u32::from_le_bytes(*(host(address) as *const [u8; 4])) } }
// NOTE the C++ rd16/rd32_reversed read native (little-endian host) order = byte-reversed guest value.

#[inline(always)]
pub fn wr8(address: u32, value: u32) {
    if slow_address(address) { (mmio().write8)(address, value as u8); return; }
    trace_write(address, 1, value as u8 as u64);
    unsafe { *plain(address) = value as u8; }
}
#[inline(always)]
pub fn wr16(address: u32, value: u32) {
    if slow_address(address) { (mmio().write16)(address, value as u16); return; }
    trace_write(address, 2, value as u16 as u64);
    unsafe { *(plain(address) as *mut [u8; 2]) = (value as u16).to_be_bytes(); }
}
#[inline(always)]
pub fn wr32(address: u32, value: u32) {
    if slow_address(address) { (mmio().write32)(address, value); return; }
    trace_write(address, 4, value as u64);
    unsafe { *(plain(address) as *mut [u8; 4]) = value.to_be_bytes(); }
}
#[inline(always)]
pub fn wr64(address: u32, value: u64) {
    if slow_address(address) { (mmio().write64)(address, value); return; }
    trace_write(address, 8, value);
    unsafe { *(plain(address) as *mut [u8; 8]) = value.to_be_bytes(); }
}
#[inline(always)] pub fn wr16_reversed(address: u32, value: u32) { trace_write(address, 0x102, value as u16 as u64); unsafe { *(host(address) as *mut [u8; 2]) = (value as u16).to_le_bytes(); } }
#[inline(always)] pub fn wr32_reversed(address: u32, value: u32) { trace_write(address, 0x104, value as u64); unsafe { *(host(address) as *mut [u8; 4]) = value.to_le_bytes(); } }

#[inline(always)] pub fn rdf32(address: u32) -> f64 { f32::from_bits(rd32(address)) as f64 }
#[inline(always)] pub fn rdf32_raw(address: u32) -> f32 { f32::from_bits(rd32(address)) }
#[inline(always)] pub fn rdf64(address: u32) -> f64 { f64::from_bits(rd64(address)) }
#[inline(always)] pub fn wrf32(address: u32, value: f32) { wr32(address, value.to_bits()) }
#[inline(always)] pub fn wrf64(address: u32, value: f64) { wr64(address, value.to_bits()) }
#[inline(always)] pub fn dcbz(address: u32) { trace_write(address & !31u32, 0x120, 0); unsafe { std::ptr::write_bytes(host(address & !31u32), 0, 32) } }
