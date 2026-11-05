// SPDX-License-Identifier: GPL-3.0-or-later
// Rust port of arelkair/wiiparty-recomp engine/include/wp/cpu.h @ e8522be (Claude, 2026-10-23).
// Every helper keeps the exact C++ arithmetic (casts, rounding, NaN handling). Verified by the differential test in
// tests/diff.rs, which runs the original cpu.h (compiled with g++) on the same inputs.
use crate::memory::*;

#[derive(Clone)]
#[repr(C)]
pub struct Cpu {
    pub r: [u32; 32],
    pub f: [f64; 32],
    pub ps1: [f64; 32],
    pub cr: [u8; 8],
    pub lr: u32,
    pub ctr: u32,
    pub xer_so: u8,
    pub xer_ov: u8,
    pub xer_ca: u8,
    pub xer_byte_count: u8,
    pub fpscr: u32,
    pub msr: u32,
    pub sr: [u32; 16],
    pub spr: [u32; 1024],
}
impl Default for Cpu {
    fn default() -> Self {
        Cpu { r: [0; 32], f: [0.0; 32], ps1: [0.0; 32], cr: [0; 8], lr: 0, ctr: 0, xer_so: 0, xer_ov: 0, xer_ca: 0,
              xer_byte_count: 0, fpscr: 0, msr: 0, sr: [0; 16], spr: [0; 1024] }
    }
}

pub const SPR_XER: u32 = 1;
pub const SPR_LR: u32 = 8;
pub const SPR_CTR: u32 = 9;
pub const SPR_GQR0: u32 = 912;

#[inline(always)]
pub fn rotl(value: u32, amount: u32) -> u32 { value.rotate_left(amount & 31) }

#[inline(always)]
pub fn record(c: &mut Cpu, value: u32) {
    let s = value as i32;
    c.cr[0] = (if s < 0 { 8 } else if s > 0 { 4 } else { 2 }) | c.xer_so;
}
#[inline(always)]
pub fn cr_get(c: &Cpu, bit: u32) -> bool { (c.cr[(bit >> 2) as usize] >> (3 - (bit & 3))) & 1 != 0 }
#[inline(always)]
pub fn cr_set(c: &mut Cpu, bit: u32, value: bool) {
    let mask = 1u8 << (3 - (bit & 3));
    if value { c.cr[(bit >> 2) as usize] |= mask; } else { c.cr[(bit >> 2) as usize] &= !mask; }
}
pub fn mfcr(c: &Cpu) -> u32 { let mut v = 0u32; for i in 0..8 { v = (v << 4) | (c.cr[i] & 0xF) as u32; } v }
pub fn mtcrf(c: &mut Cpu, mask: u32, value: u32) {
    for i in 0..8 { if mask & (0x80u32 >> i) != 0 { c.cr[i] = ((value >> (28 - 4 * i)) & 0xF) as u8; } }
}
pub fn mfxer(c: &Cpu) -> u32 {
    ((c.xer_so as u32) << 31) | ((c.xer_ov as u32) << 30) | ((c.xer_ca as u32) << 29) | (c.xer_byte_count as u32 & 0x7F)
}
pub fn mtxer(c: &mut Cpu, value: u32) {
    c.xer_so = ((value >> 31) & 1) as u8; c.xer_ov = ((value >> 30) & 1) as u8; c.xer_ca = ((value >> 29) & 1) as u8;
    c.xer_byte_count = (value & 0x7F) as u8;
}
#[inline(always)]
pub fn compare_signed(c: &mut Cpu, field: u32, a: u32, b: u32) {
    let (x, y) = (a as i32, b as i32);
    c.cr[field as usize] = (if x < y { 8 } else if x > y { 4 } else { 2 }) | c.xer_so;
}
#[inline(always)]
pub fn compare_unsigned(c: &mut Cpu, field: u32, a: u32, b: u32) {
    c.cr[field as usize] = (if a < b { 8 } else if a > b { 4 } else { 2 }) | c.xer_so;
}
#[inline(always)]
pub fn compare_float(c: &mut Cpu, field: u32, a: f64, b: f64) {
    let bits: u8 = if a.is_nan() || b.is_nan() { 1 } else if a < b { 8 } else if a > b { 4 } else { 2 };
    c.cr[field as usize] = bits;
    c.fpscr = (c.fpscr & !0xF000u32) | ((bits as u32) << 12);
}
#[inline(always)]
pub fn add_carry(c: &mut Cpu, a: u32, b: u32, carry: u32) -> u32 {
    let sum = a as u64 + b as u64 + carry as u64;
    c.xer_ca = (sum >> 32) as u8;
    sum as u32
}
#[inline(always)] pub fn add_extended(c: &mut Cpu, a: u32, b: u32) -> u32 { let k = c.xer_ca as u32; add_carry(c, a, b, k) }
#[inline(always)] pub fn sub_carry(c: &mut Cpu, a: u32, b: u32) -> u32 { add_carry(c, !a, b, 1) }
#[inline(always)] pub fn sub_extended(c: &mut Cpu, a: u32, b: u32) -> u32 { let k = c.xer_ca as u32; add_carry(c, !a, b, k) }
#[inline(always)] pub fn mulhw(a: u32, b: u32) -> u32 { ((a as i32 as i64 * b as i32 as i64) >> 32) as u32 }
#[inline(always)] pub fn mulhwu(a: u32, b: u32) -> u32 { ((a as u64 * b as u64) >> 32) as u32 }
#[inline(always)]
pub fn divw(a: u32, b: u32) -> u32 {
    let (x, y) = (a as i32, b as i32);
    if y == 0 || (x == i32::MIN && y == -1) { return if x < 0 { 0xFFFF_FFFF } else { 0 }; }
    (x / y) as u32
}
#[inline(always)] pub fn divwu(a: u32, b: u32) -> u32 { if b == 0 { 0 } else { a / b } }
#[inline(always)] pub fn cntlzw(value: u32) -> u32 { value.leading_zeros() }
#[inline(always)] pub fn slw(value: u32, amount: u32) -> u32 { let a = amount & 0x3F; if a > 31 { 0 } else { value << a } }
#[inline(always)] pub fn srw(value: u32, amount: u32) -> u32 { let a = amount & 0x3F; if a > 31 { 0 } else { value >> a } }
#[inline(always)]
pub fn sraw(c: &mut Cpu, value: u32, amount: u32) -> u32 {
    let a = amount & 0x3F; let s = value as i32;
    if a > 31 { c.xer_ca = (s < 0) as u8; return (s >> 31) as u32; }
    c.xer_ca = (s < 0 && (value & ((1u32 << a).wrapping_sub(1))) != 0) as u8;
    (s >> a) as u32
}
#[inline(always)] pub fn sign_extend8(value: u32) -> u32 { value as u8 as i8 as i32 as u32 }
#[inline(always)] pub fn sign_extend16(value: u32) -> u32 { value as u16 as i16 as i32 as u32 }
#[inline(always)] pub fn round_single(value: f64) -> f64 { value as f32 as f64 }
#[inline(always)] pub fn fpr_bits(value: f64) -> u64 { value.to_bits() }
#[inline(always)] pub fn fpr_from_bits(bits: u64) -> f64 { f64::from_bits(bits) }

/// std::nearbyint in the default rounding mode (round half to even).
#[inline(always)]
fn nearbyint(v: f64) -> f64 { v.round_ties_even() }

pub fn convert_to_int(value: f64, truncate: bool) -> f64 {
    let result: i32 = if value.is_nan() { i32::MIN } else {
        let rounded = if truncate { value.trunc() } else { nearbyint(value) };
        if rounded >= 2147483647.0 { i32::MAX } else if rounded <= -2147483648.0 { i32::MIN } else { rounded as i32 }
    };
    fpr_from_bits(0xFFF8_0000_0000_0000u64 | (result as u32) as u64)
}
#[inline(always)] pub fn fsel(a: f64, b: f64, c: f64) -> f64 { if a >= 0.0 { c } else { b } }

/// The recomp uses the hardware FMA when the host has one (g_hardware_fma); a fused multiply-add is exactly
/// rounded either way, so f64::mul_add is the same result.
// engine/src/core/fpu.cpp: hardware FMA when the host CPU has it, else cpu.h software_fma (not always correctly rounded).
pub static HARDWARE_FMA: std::sync::LazyLock<std::sync::atomic::AtomicBool> =
    std::sync::LazyLock::new(|| std::sync::atomic::AtomicBool::new(host_has_fma()));
fn host_has_fma() -> bool {
    #[cfg(target_arch = "x86_64")] { std::arch::is_x86_feature_detected!("fma") }
    #[cfg(not(target_arch = "x86_64"))] { true }
}
#[inline(always)]
fn two_sum(a: f64, b: f64) -> (f64, f64) { let sum = a + b; let b_part = sum - a; (sum, (a - (sum - b_part)) + (b - b_part)) }
#[inline(always)]
fn split(v: f64) -> (f64, f64) { let scaled = v * 134217729.0; let high = scaled - (scaled - v); (high, v - high) }
fn next_toward_zero(x: f64) -> f64 { // std::nextafter(x, 0.0)
    if x == 0.0 || x.is_nan() { return if x.is_nan() { x } else { 0.0 } }
    f64::from_bits(x.to_bits() - 1)
}
pub fn software_fma(a: f64, b: f64, c: f64) -> f64 {
    let product = a * b;
    if !product.is_finite() || !c.is_finite() || a == 0.0 || b == 0.0 || a.abs() > f64::from_bits(0x7E20000000000000) /*0x1p995*/
        || b.abs() > f64::from_bits(0x7E20000000000000) || product.abs() < f64::from_bits(0x07B0000000000000) /*0x1p-900*/ {
        return product + c;
    }
    let (a_high, a_low) = split(a); let (b_high, b_low) = split(b);
    let product_error = ((a_high * b_high - product) + a_high * b_low + a_low * b_high) + a_low * b_low;
    let (high, low) = two_sum(c, product);
    let (mut sum, error) = two_sum(low, product_error);
    if error != 0.0 {
        if (error > 0.0) != (sum > 0.0) { sum = next_toward_zero(sum); }
        sum = f64::from_bits(sum.to_bits() | 1);
    }
    high + sum
}
/// engine/src/core/fpu.cpp hardware_fma: `__attribute__((target("fma"))) __builtin_fma` = the CPU's VFMADD.
/// Do NOT use f64::mul_add here: without a compile-time +fma it calls the C runtime's fma(), and MinGW's
/// fma() is not the hardware result (H0: 20,095 differing lines on Windows until this was fixed).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "fma")]
unsafe fn hardware_fma(a: f64, b: f64, c: f64) -> f64 {
    use std::arch::x86_64::*;
    _mm_cvtsd_f64(_mm_fmadd_sd(_mm_set_sd(a), _mm_set_sd(b), _mm_set_sd(c)))
}
#[inline(always)]
pub fn fused_multiply_add(a: f64, b: f64, c: f64) -> f64 {
    // Claude OPT batch1 R3: the build already uses -C target-feature=+fma, so the host must have FMA and HARDWARE_FMA is
    // always true there; skip the LazyLock/atomic check. Same instruction, same result.
    #[cfg(all(feature = "opt-batch1", target_arch = "x86_64", target_feature = "fma"))]
    {
        return unsafe { hardware_fma(a, b, c) };
    }
    #[allow(unreachable_code)]
    if HARDWARE_FMA.load(std::sync::atomic::Ordering::Relaxed) {
        #[cfg(target_arch = "x86_64")] { return unsafe { hardware_fma(a, b, c) }; }
        #[cfg(not(target_arch = "x86_64"))] { return a.mul_add(b, c); }
    }
    software_fma(a, b, c)
}

#[inline(always)] pub fn set_overflow(c: &mut Cpu, overflow: bool) { c.xer_ov = overflow as u8; c.xer_so |= c.xer_ov; }
#[inline(always)]
pub fn multiply_overflows(a: u32, b: u32) -> bool { let p = a as i32 as i64 * b as i32 as i64; p != p as i32 as i64 }

const RSQRT: [(i32, i32); 32] = [
    (0x1a7e800, -0x568), (0x17cb800, -0x4f3), (0x1552800, -0x48d), (0x130c000, -0x435), (0x10f2000, -0x3e7), (0x0eff000, -0x3a2),
    (0x0d2e000, -0x365), (0x0b7c000, -0x32e), (0x09e5000, -0x2fc), (0x0867000, -0x2d0), (0x06ff000, -0x2a8), (0x05ab800, -0x283),
    (0x046a000, -0x261), (0x0339800, -0x243), (0x0218800, -0x226), (0x0105800, -0x20b), (0x3ffa000, -0x7a4), (0x3c29000, -0x700),
    (0x38aa000, -0x670), (0x3572000, -0x5f2), (0x3279000, -0x584), (0x2fb7000, -0x524), (0x2d26000, -0x4cc), (0x2ac0000, -0x47e),
    (0x2881000, -0x43a), (0x2665000, -0x3fa), (0x2468000, -0x3c2), (0x2287000, -0x38e), (0x20c1000, -0x35e), (0x1f12000, -0x332),
    (0x1d79000, -0x30a), (0x1bf4000, -0x2e6),
];
const RECIP: [(i32, i32); 32] = [
    (0x7ff800, 0x3e1), (0x783800, 0x3a7), (0x70ea00, 0x371), (0x6a0800, 0x340), (0x638800, 0x313), (0x5d6200, 0x2ea),
    (0x579000, 0x2c4), (0x520800, 0x2a0), (0x4cc800, 0x27f), (0x47ca00, 0x261), (0x430800, 0x245), (0x3e8000, 0x22a),
    (0x3a2c00, 0x212), (0x360800, 0x1fb), (0x321400, 0x1e5), (0x2e4a00, 0x1d1), (0x2aa800, 0x1be), (0x272c00, 0x1ac),
    (0x23d600, 0x19b), (0x209e00, 0x18b), (0x1d8800, 0x17c), (0x1a9000, 0x16e), (0x17ae00, 0x15b), (0x14f800, 0x15b),
    (0x124400, 0x143), (0x0fbe00, 0x143), (0x0d3800, 0x12d), (0x0ade00, 0x12d), (0x088400, 0x11a), (0x065000, 0x11a),
    (0x041c00, 0x108), (0x020c00, 0x106),
];
#[inline(always)] fn quiet_nan_of(v: f64) -> f64 { fpr_from_bits(fpr_bits(v) | 0x0008_0000_0000_0000) }

pub fn reciprocal_estimate(value: f64) -> f64 {
    let mut integral = fpr_bits(value) as i64;
    let mantissa = integral & ((1i64 << 52) - 1);
    let sign = integral & (1u64 << 63) as i64;
    let mut exponent = integral & (0x7FFi64 << 52);
    if mantissa == 0 && exponent == 0 { return f64::INFINITY.copysign(value); }
    if exponent == (0x7FFi64 << 52) { return if mantissa == 0 { 0.0f64.copysign(value) } else { quiet_nan_of(value) }; }
    if exponent < (895i64 << 52) { return 3.4028234663852886e38f64.copysign(value); }
    if exponent >= (1149i64 << 52) { return 0.0f64.copysign(value); }
    exponent = (0x7FDi64 << 52) - exponent;
    let index = (mantissa >> 37) as i32;
    let (base, dec) = RECIP[(index / 1024) as usize];
    integral = sign | exponent;
    integral |= ((base - (dec * (index % 1024) + 1) / 2) as i64) << 29;
    fpr_from_bits(integral as u64)
}

pub fn reciprocal_sqrt_estimate(value: f64) -> f64 {
    let mut integral = fpr_bits(value) as i64;
    let mut mantissa = integral & ((1i64 << 52) - 1);
    let sign = integral & (1u64 << 63) as i64;
    let mut exponent = integral & (0x7FFi64 << 52);
    if mantissa == 0 && exponent == 0 { return if sign != 0 { f64::NEG_INFINITY } else { f64::INFINITY }; }
    if exponent == (0x7FFi64 << 52) {
        if mantissa == 0 { return if sign != 0 { cpp_nan() } else { 0.0 }; }
        return quiet_nan_of(value);
    }
    if sign != 0 { return cpp_nan(); }
    if exponent == 0 {
        loop {
            exponent -= 1i64 << 52; mantissa <<= 1;
            if mantissa & (1i64 << 52) != 0 { break; }
        }
        mantissa &= (1i64 << 52) - 1;
        exponent += 1i64 << 52;
    }
    let exponent_lsb = exponent & (1i64 << 52);
    exponent = ((0x3FFi64 << 52) - ((exponent - (0x3FEi64 << 52)) / 2)) & (0x7FFi64 << 52);
    integral = sign | exponent;
    let index = ((exponent_lsb | mantissa) >> 37) as i32;
    let (base, dec) = RSQRT[(index / 2048) as usize];
    integral |= ((base + dec * (index % 2048)) as i64) << 26;
    fpr_from_bits(integral as u64)
}
/// std::nan("") on glibc/MSVC = positive quiet NaN 0x7FF8000000000000.
#[inline(always)] fn cpp_nan() -> f64 { f64::from_bits(0x7FF8_0000_0000_0000) }

#[cfg_attr(feature = "opt-batch1", inline)]
pub fn force25(value: f64) -> f64 {
    let mut integral = fpr_bits(value);
    let exponent = integral & 0x7FF0_0000_0000_0000;
    let fraction = integral & 0x000F_FFFF_FFFF_FFFF;
    if exponent == 0 && fraction != 0 {
        let mut keep: i64 = 0xFFFF_FFFF_F800_0000u64 as i64;
        let mut round: u64 = 0x800_0000;
        let shift = fraction.leading_zeros() as i64 - 11;
        keep >>= shift; round >>= shift;
        integral = (integral & keep as u64).wrapping_add(integral & round);
    } else {
        integral = (integral & 0xFFFF_FFFF_F800_0000).wrapping_add(integral & 0x800_0000);
    }
    fpr_from_bits(integral)
}

pub fn madd_single(a: f64, c: f64, b: f64) -> f64 {
    let rounded = force25(c);
    let mut result = fused_multiply_add(a, rounded, b);
    let bits = fpr_bits(result);
    if bits & 0x1FFF_FFFF == 0x1000_0000 {
        let a_prime = b - result;
        let b_prime = result + a_prime;
        let delta_a = fused_multiply_add(a, rounded, a_prime);
        let delta_b = b - b_prime;
        let error = delta_a + delta_b;
        if error != 0.0 {
            result = fpr_from_bits(if (error > 0.0) == (result > 0.0) { bits.wrapping_add(1) } else { bits.wrapping_sub(1) });
        }
    }
    result
}
#[inline(always)] pub fn update_cr1(c: &mut Cpu) { c.cr[1] = ((c.fpscr >> 28) & 0xF) as u8; }

fn quantizer_factors() -> &'static ([f32; 64], [f32; 64]) {
    static T: std::sync::OnceLock<([f32; 64], [f32; 64])> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let (mut load, mut store) = ([0f32; 64], [0f32; 64]);
        for bits in 0..64i32 {
            let scale = if bits & 0x20 != 0 { bits - 0x40 } else { bits };
            let (mut up, mut down) = (1.0f32, 1.0f32);
            for _ in 0..scale.abs() { up *= 2.0; down *= 0.5; }
            store[bits as usize] = if scale < 0 { down } else { up };
            load[bits as usize] = if scale < 0 { up } else { down };
        }
        (load, store)
    })
}
#[cfg_attr(feature = "opt-batch1", inline(always))]
fn dequantize(address: u32, ty: u32, scale_bits: u32, size: &mut u32) -> f32 {
    let factor = quantizer_factors().0[(scale_bits & 0x3F) as usize];
    match ty {
        4 => { *size = 1; rd8(address) as u8 as f32 * factor }
        5 => { *size = 2; rd16(address) as u16 as f32 * factor }
        6 => { *size = 1; rd8(address) as u8 as i8 as f32 * factor }
        7 => { *size = 2; rd16(address) as u16 as i16 as f32 * factor }
        _ => { *size = 4; rdf32_raw(address) }
    }
}
fn clamp_quantized(value: f32, ty: u32) -> u32 {
    let (lo, hi): (i64, i64) = match ty { 4 => (0, 255), 5 => (0, 65535), 6 => (-128, 127), _ => (-32768, 32767) };
    if value.is_nan() || value <= lo as f32 { return lo as u32; }
    if value >= hi as f32 { return hi as u32; }
    value as i64 as u32
}
#[cfg_attr(feature = "opt-batch1", inline(always))]
fn quantize(address: u32, value: f32, ty: u32, scale_bits: u32, size: &mut u32) {
    let scaled = value * quantizer_factors().1[(scale_bits & 0x3F) as usize];
    match ty {
        4 | 6 => { *size = 1; wr8(address, clamp_quantized(scaled, ty) as u8 as u32); }
        5 | 7 => { *size = 2; wr16(address, clamp_quantized(scaled, ty) as u16 as u32); }
        _ => { *size = 4; wrf32(address, value); }
    }
}
pub fn load_quantized(c: &mut Cpu, d: u32, address: u32, w: u32, index: u32, size: &mut u32) {
    let gqr = c.spr[(SPR_GQR0 + index) as usize];
    let ty = (gqr >> 16) & 7; let scale = (gqr >> 24) & 0x3F;
    let mut first = 0;
    c.f[d as usize] = dequantize(address, ty, scale, &mut first) as f64;
    if w != 0 { c.ps1[d as usize] = 1.0; *size = first; } else {
        let mut second = 0;
        c.ps1[d as usize] = dequantize(address.wrapping_add(first), ty, scale, &mut second) as f64;
        *size = first + second;
    }
}
pub fn store_quantized(c: &mut Cpu, s: u32, address: u32, w: u32, index: u32, size: &mut u32) {
    let gqr = c.spr[(SPR_GQR0 + index) as usize];
    let ty = gqr & 7; let scale = (gqr >> 8) & 0x3F;
    let mut first = 0;
    quantize(address, c.f[s as usize] as f32, ty, scale, &mut first);
    if w != 0 { *size = first; } else {
        let mut second = 0;
        let a2 = if address == FIFO_ADDRESS { address } else { address.wrapping_add(first) };
        quantize(a2, c.ps1[s as usize] as f32, ty, scale, &mut second);
        *size = first + second;
    }
}

#[inline(always)] fn ps_set(c: &mut Cpu, d: u32, x: f64, y: f64) { c.f[d as usize] = round_single(x); c.ps1[d as usize] = round_single(y); }
macro_rules! f { ($c:ident, $i:expr) => { $c.f[$i as usize] } }
macro_rules! p { ($c:ident, $i:expr) => { $c.ps1[$i as usize] } }
pub fn ps_add(c: &mut Cpu, d: u32, a: u32, b: u32) { let (x, y) = (f!(c, a) + f!(c, b), p!(c, a) + p!(c, b)); ps_set(c, d, x, y) }
pub fn ps_sub(c: &mut Cpu, d: u32, a: u32, b: u32) { let (x, y) = (f!(c, a) - f!(c, b), p!(c, a) - p!(c, b)); ps_set(c, d, x, y) }
pub fn ps_mul(c: &mut Cpu, d: u32, a: u32, f: u32) { let (x, y) = (f!(c, a) * force25(f!(c, f)), p!(c, a) * force25(p!(c, f))); ps_set(c, d, x, y) }
pub fn ps_div(c: &mut Cpu, d: u32, a: u32, b: u32) { let (x, y) = (f!(c, a) / f!(c, b), p!(c, a) / p!(c, b)); ps_set(c, d, x, y) }
pub fn ps_muls0(c: &mut Cpu, d: u32, a: u32, f: u32) { let (x, y) = (f!(c, a) * force25(f!(c, f)), p!(c, a) * force25(f!(c, f))); ps_set(c, d, x, y) }
pub fn ps_muls1(c: &mut Cpu, d: u32, a: u32, f: u32) { let (x, y) = (f!(c, a) * force25(p!(c, f)), p!(c, a) * force25(p!(c, f))); ps_set(c, d, x, y) }
/// The generated code always passes literal +-1.0 signs, and the recomp is built with GCC -O1, which folds
/// `x * 1.0` to `x` and `x * -1.0` to `-x` (a sign flip, also for NaN). The differential test against the original
/// cpu.h confirms it, so the multiplies by the signs are written as those exact folds here.
#[inline(always)] fn by_sign(x: f64, sign: f64) -> f64 { if sign < 0.0 { -x } else { x } }
pub fn ps_madd(c: &mut Cpu, d: u32, a: u32, f: u32, b: u32, product_sign: f64, addend_sign: f64) {
    let x = by_sign(madd_single(f!(c, a), f!(c, f), by_sign(by_sign(f!(c, b), addend_sign), product_sign)), product_sign);
    let y = by_sign(madd_single(p!(c, a), p!(c, f), by_sign(by_sign(p!(c, b), addend_sign), product_sign)), product_sign);
    ps_set(c, d, x, y)
}
pub fn ps_madds0(c: &mut Cpu, d: u32, a: u32, f: u32, b: u32) { let (x, y) = (madd_single(f!(c, a), f!(c, f), f!(c, b)), madd_single(p!(c, a), f!(c, f), p!(c, b))); ps_set(c, d, x, y) }
pub fn ps_madds1(c: &mut Cpu, d: u32, a: u32, f: u32, b: u32) { let (x, y) = (madd_single(f!(c, a), p!(c, f), f!(c, b)), madd_single(p!(c, a), p!(c, f), p!(c, b))); ps_set(c, d, x, y) }
pub fn ps_sum0(c: &mut Cpu, d: u32, a: u32, f: u32, b: u32) { let (x, y) = (f!(c, a) + p!(c, b), p!(c, f)); ps_set(c, d, x, y) }
pub fn ps_sum1(c: &mut Cpu, d: u32, a: u32, f: u32, b: u32) { let (x, y) = (f!(c, f), f!(c, a) + p!(c, b)); ps_set(c, d, x, y) }
pub fn ps_sel(c: &mut Cpu, d: u32, a: u32, f: u32, b: u32) {
    let x = if f!(c, a) >= 0.0 { f!(c, f) } else { f!(c, b) }; let y = if p!(c, a) >= 0.0 { p!(c, f) } else { p!(c, b) };
    f!(c, d) = x; p!(c, d) = y;
}
pub fn ps_res(c: &mut Cpu, d: u32, b: u32) { let (x, y) = (reciprocal_estimate(f!(c, b)), reciprocal_estimate(p!(c, b))); ps_set(c, d, x, y) }
pub fn ps_rsqrte(c: &mut Cpu, d: u32, b: u32) { let (x, y) = (reciprocal_sqrt_estimate(f!(c, b)), reciprocal_sqrt_estimate(p!(c, b))); ps_set(c, d, x, y) }
pub fn ps_merge(c: &mut Cpu, d: u32, a: u32, b: u32, high_a: bool, high_b: bool) {
    let x = if high_a { p!(c, a) } else { f!(c, a) }; let y = if high_b { p!(c, b) } else { f!(c, b) };
    f!(c, d) = x; p!(c, d) = y;
}
pub fn ps_mr(c: &mut Cpu, d: u32, b: u32) { let (x, y) = (f!(c, b), p!(c, b)); f!(c, d) = x; p!(c, d) = y; }
pub fn ps_neg(c: &mut Cpu, d: u32, b: u32) { let (x, y) = (-f!(c, b), -p!(c, b)); f!(c, d) = x; p!(c, d) = y; }
pub fn ps_abs(c: &mut Cpu, d: u32, b: u32) { let (x, y) = (f!(c, b).abs(), p!(c, b).abs()); f!(c, d) = x; p!(c, d) = y; }
pub fn ps_nabs(c: &mut Cpu, d: u32, b: u32) { let (x, y) = (-f!(c, b).abs(), -p!(c, b).abs()); f!(c, d) = x; p!(c, d) = y; }

// conversions used by the translated code (C++ static_cast semantics)
#[inline(always)] pub fn to_f32(v: f64) -> f32 { v as f32 }
#[inline(always)] pub fn to_u32(v: u64) -> u32 { v as u32 } // C++ static_cast<uint32_t> of a 64-bit value (time base, fpr bits)
#[inline(always)] pub fn fabs(v: f64) -> f64 { v.abs() }
