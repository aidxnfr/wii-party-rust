// SPDX-License-Identifier: GPL-3.0-or-later
// Rust port of arelkair/wiiparty-recomp engine/src/gpu/gx_render_common.cpp + gx_render_common.h @ e8522be
// (Claude, 2026-10-24). Register files are passed as slices (C++ reads them through gx::bp_registers() etc.).
use wprt_ports::gx_state::{blend_state, BlendState};

pub const EFB_WIDTH: i32 = 640;
pub const EFB_HEIGHT: i32 = 528;
pub const MAX_STAGES: usize = 16;
pub const TEXTURE_MAPS: u32 = 8;
pub const BP_SCISSOR_OFFSET: u32 = 0x59;
#[inline] pub fn scissor_offset_x(reg: u32) -> i32 { ((reg & 0x3FF) * 2) as i32 }
#[inline] pub fn scissor_offset_y(reg: u32) -> i32 { (((reg >> 10) & 0x3FF) * 2) as i32 }
pub const TLUT_SIZE: u32 = 0x100000;
pub const TLUT_MASK: u32 = 0x7FE00;
pub const XFB_FORMAT: u32 = 13;

/// gx_render_common.h Constants (uploaded as one constant buffer; layout must stay std140-compatible).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Constants {
    pub initial: [[f32; 4]; 4],
    pub konst: [[f32; 4]; 4],
    pub stage: [[u32; 4]; MAX_STAGES],
    pub header: [u32; 4],
    pub swaps: [u32; 4],
    pub texdims: [[f32; 4]; 8],
    pub indmtx: [[i32; 4]; 6],
    pub indirect: [u32; 4],
    pub tevind: [[u32; 4]; MAX_STAGES],
    pub fog: [f32; 4],
    pub fog_integer: [i32; 4],
    pub fog_range: [[f32; 4]; 3],
}
impl Default for Constants { fn default() -> Self { unsafe { std::mem::zeroed() } } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout { pub block_width: u32, pub block_height: u32, pub supported: bool }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AlphaTestResult { pub can_pass: bool, pub can_fail: bool }

#[derive(Clone, Debug)]
pub struct TextureRequest {
    pub valid: bool,
    pub address: u32,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub levels: u32,
    pub size: usize,
    pub total: usize,
    pub offsets: [usize; 12],
    pub source: *const u8,
    pub tlut: *const u8,
    pub tlut_format: u32,
    pub entries: u32,
    pub key: u64,
    pub mipmaps: bool,
    pub supported: bool,
}
impl Default for TextureRequest {
    fn default() -> Self {
        TextureRequest { valid: false, address: 0, width: 0, height: 0, format: 0, levels: 1, size: 0, total: 0, offsets: [0; 12],
            source: std::ptr::null(), tlut: std::ptr::null(), tlut_format: 0, entries: 0, key: 0, mipmaps: false, supported: true }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SamplerParams {
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub min_linear: bool,
    pub mag_linear: bool,
    pub mip_mode: u32,
    pub min_lod: f32,
    pub max_lod: f32,
    pub lod_bias: f32,
    pub unlimited_lod: bool,
    pub key: u64,
}

/// TMEM palette memory (C++ global g_tlut). One per renderer instance (= per game DLL).
pub struct Tlut(pub Box<[u8]>);
impl Default for Tlut { fn default() -> Self { Tlut(vec![0u8; TLUT_SIZE as usize].into_boxed_slice()) } }

#[inline] fn be16(p: &[u8]) -> u16 { ((p[0] as u16) << 8) | p[1] as u16 }
#[inline] fn rgba(r: u32, g: u32, b: u32, a: u32) -> u32 { r | (g << 8) | (b << 16) | (a << 24) }
#[inline] fn expand5(v: u32) -> u32 { (v << 3) | (v >> 2) }
#[inline] fn expand6(v: u32) -> u32 { (v << 2) | (v >> 4) }
#[inline] fn expand4(v: u32) -> u32 { v * 17 }
#[inline] fn expand3(v: u32) -> u32 { (v << 5) | (v << 2) | (v >> 1) }
#[inline] fn rgb565(v: u16) -> u32 { let v = v as u32; rgba(expand5((v >> 11) & 31), expand6((v >> 5) & 63), expand5(v & 31), 255) }
#[inline]
fn rgb5a3(v: u16) -> u32 {
    let v = v as u32;
    if v & 0x8000 != 0 { return rgba(expand5((v >> 10) & 31), expand5((v >> 5) & 31), expand5(v & 31), 255); }
    rgba(expand4((v >> 8) & 15), expand4((v >> 4) & 15), expand4(v & 15), expand3((v >> 12) & 7))
}

fn decode_cmpr_block(source: &[u8], out: &mut [u32], stride: usize) {
    let c0 = be16(source);
    let c1 = be16(&source[2..]);
    let split = |c: u16| { let c = c as u32; (expand5((c >> 11) & 31), expand6((c >> 5) & 63), expand5(c & 31)) };
    let (r0, g0, b0) = split(c0);
    let (r1, g1, b1) = split(c1);
    let mut palette = [rgba(r0, g0, b0, 255), rgba(r1, g1, b1, 255), 0, 0];
    if c0 > c1 {
        palette[2] = rgba((2 * r0 + r1) / 3, (2 * g0 + g1) / 3, (2 * b0 + b1) / 3, 255);
        palette[3] = rgba((r0 + 2 * r1) / 3, (g0 + 2 * g1) / 3, (b0 + 2 * b1) / 3, 255);
    } else {
        palette[2] = rgba((r0 + r1) / 2, (g0 + g1) / 2, (b0 + b1) / 2, 255);
        palette[3] = rgba(0, 0, 0, 0);
    }
    for row in 0..4usize {
        let bits = source[4 + row] as u32;
        for column in 0..4usize {
            out[row * stride + column] = palette[((bits >> (6 - 2 * column as u32)) & 3) as usize];
        }
    }
}

/// C++ palette_entry(tlut, format, index). `tlut` points into TMEM; reads 2 bytes at index*2.
#[inline]
unsafe fn palette_entry(tlut: *const u8, tlut_format: u32, index: u32) -> u32 {
    let p = tlut.add(index as usize * 2);
    let entry = ((*p as u16) << 8) | *p.add(1) as u16;
    match tlut_format {
        0 => { let e = entry as u32; rgba(e & 0xFF, e & 0xFF, e & 0xFF, e >> 8) }
        1 => rgb565(entry),
        _ => rgb5a3(entry),
    }
}

#[inline]
fn signed11(value: u32) -> f32 {
    let mut v = (value & 0x7FF) as i32;
    if v & 0x400 != 0 { v -= 0x800; }
    v as f32 / 255.0f32
}

pub fn layout_for(format: u32) -> Layout {
    match format {
        0 | 8 => Layout { block_width: 8, block_height: 8, supported: true },
        1 | 2 | 9 => Layout { block_width: 8, block_height: 4, supported: true },
        3 | 4 | 5 | 6 | 10 => Layout { block_width: 4, block_height: 4, supported: true },
        14 => Layout { block_width: 8, block_height: 8, supported: true },
        _ => Layout { block_width: 4, block_height: 4, supported: false },
    }
}

#[inline] pub fn block_bytes(format: u32) -> u32 { if format == 6 { 64 } else { 32 } }

/// C++ decode_texture. `data` must hold every block of the padded image; `tlut` is the TMEM palette base.
/// Safety: same contract as the C++ (raw guest pointers; caller validated the range).
unsafe fn decode_texture_padded(mut data: *const u8, width: u32, height: u32, format: u32, tlut: *const u8, tlut_format: u32, pixels: &mut Vec<u32>) {
    let layout = layout_for(format);
    let padded_width = (width + layout.block_width - 1) / layout.block_width * layout.block_width;
    let padded_height = (height + layout.block_height - 1) / layout.block_height * layout.block_height;
    pixels.resize(padded_width as usize * padded_height as usize, 0);
    if !layout.supported { pixels.fill(rgba(255, 0, 255, 255)); return; }
    let bytes = block_bytes(format) as usize;
    let mut by = 0;
    while by < padded_height {
        let mut bx = 0;
        while bx < padded_width {
            let base = by as usize * padded_width as usize + bx as usize;
            let block = std::slice::from_raw_parts(data, bytes);
            data = data.add(bytes);
            // Same four source subblocks and palette math; decode each 4x4 once, not once per output texel.
            if format == 14 {
                for sub in 0..4usize {
                    let dst = base + (sub / 2) * 4 * padded_width as usize + (sub % 2) * 4;
                    decode_cmpr_block(&block[sub * 8..], &mut pixels[dst..], padded_width as usize);
                }
                bx += layout.block_width;
                continue;
            }
            for y in 0..layout.block_height {
                for x in 0..layout.block_width {
                    let (yu, xu) = (y as usize, x as usize);
                    let pixel = match format {
                        0 => { let v = ((block[yu * 4 + xu / 2] >> if x & 1 != 0 { 0 } else { 4 }) & 15) as u32; rgba(expand4(v), expand4(v), expand4(v), expand4(v)) }
                        1 => { let v = block[yu * 8 + xu] as u32; rgba(v, v, v, v) }
                        2 => { let v = block[yu * 8 + xu] as u32; let i = expand4(v & 15); rgba(i, i, i, expand4(v >> 4)) }
                        3 => { let a = block[(yu * 4 + xu) * 2] as u32; let i = block[(yu * 4 + xu) * 2 + 1] as u32; rgba(i, i, i, a) }
                        4 => rgb565(be16(&block[(yu * 4 + xu) * 2..])),
                        5 => rgb5a3(be16(&block[(yu * 4 + xu) * 2..])),
                        6 => { let i = (yu * 4 + xu) * 2; rgba(block[i + 1] as u32, block[32 + i] as u32, block[32 + i + 1] as u32, block[i] as u32) }
                        8 => palette_entry(tlut, tlut_format, ((block[yu * 4 + xu / 2] >> if x & 1 != 0 { 0 } else { 4 }) & 15) as u32),
                        9 => palette_entry(tlut, tlut_format, block[yu * 8 + xu] as u32),
                        10 => palette_entry(tlut, tlut_format, (be16(&block[(yu * 4 + xu) * 2..]) & 0x3FFF) as u32),
                        _ => 0,
                    };
                    pixels[base + yu * padded_width as usize + xu] = pixel;
                }
            }
            bx += layout.block_width;
        }
        by += layout.block_height;
    }
}

/// Source decode formulas above, with caller-owned CPU buffers only. No guest pointer escapes this call.
pub unsafe fn decode_texture_into(data: *const u8, width: u32, height: u32, format: u32, tlut: *const u8, tlut_format: u32,
                                  pixels: &mut Vec<u32>, padded: &mut Vec<u32>) {
    let layout = layout_for(format);
    let pw = (width + layout.block_width - 1) / layout.block_width * layout.block_width;
    let ph = (height + layout.block_height - 1) / layout.block_height * layout.block_height;
    if !layout.supported || (pw == width && ph == height) {
        decode_texture_padded(data, width, height, format, tlut, tlut_format, pixels);
    } else {
        decode_texture_padded(data, width, height, format, tlut, tlut_format, padded);
        pixels.resize(width as usize * height as usize, 0);
        for y in 0..height as usize {
            let w = width as usize;
            pixels[y * w..y * w + w].copy_from_slice(&padded[y * pw as usize..y * pw as usize + w]);
        }
    }
}

pub unsafe fn decode_texture(data: *const u8, width: u32, height: u32, format: u32, tlut: *const u8, tlut_format: u32) -> Vec<u32> {
    let mut pixels = Vec::new();
    decode_texture_into(data, width, height, format, tlut, tlut_format, &mut pixels, &mut Vec::new());
    pixels
}

/// C++ hash_bytes: reads 8-byte words in HOST (little-endian) order, then the tail bytes.
pub unsafe fn hash_bytes(data: *const u8, size: usize) -> u64 {
    let mut hash: u64 = 1469598103934665603;
    let words = size / 8;
    for i in 0..words { hash = (hash ^ std::ptr::read_unaligned(data.add(i * 8) as *const u64)).wrapping_mul(1099511628211); }
    for i in words * 8..size { hash = (hash ^ *data.add(i) as u64).wrapping_mul(1099511628211); }
    hash
}

/// C++ sample_hash (32 samples of up to 8 bytes, little-endian, zero-extended when size < 8).
pub unsafe fn sample_hash(data: *const u8, size: usize) -> u64 {
    const SAMPLES: usize = 32;
    let mut hash: u64 = 1469598103934665603;
    let step = size / SAMPLES;
    for i in 0..SAMPLES {
        let mut word = [0u8; 8];
        let at = std::cmp::min(i * step, if size > 8 { size - 8 } else { 0 });
        std::ptr::copy_nonoverlapping(data.add(at), word.as_mut_ptr(), std::cmp::min(size, 8));
        hash = (hash ^ u64::from_le_bytes(word)).wrapping_mul(1099511628211);
    }
    hash
}

pub fn blend_state_for(word: u32) -> BlendState {
    blend_state(word & 0xFFFF, (word >> 16) & 1 != 0, (word >> 17) & 1 != 0, (word >> 18) & 1 == 0)
}

/// C++ fill_constants(constants, scale) with bp = bp_registers(), konst = konst_registers(), xf = xf_registers().
pub fn fill_constants(constants: &mut Constants, scale: i32, bp: &[u32], konst: &[u32], xf: &[u32]) {
    *constants = Constants::default();
    for i in 0..4usize {
        let ra = bp[0xE0 + 2 * i];
        let bg = bp[0xE1 + 2 * i];
        constants.initial[i][0] = signed11(ra);
        constants.initial[i][3] = signed11(ra >> 12);
        constants.initial[i][2] = signed11(bg);
        constants.initial[i][1] = signed11(bg >> 12);
    }
    for i in 0..4usize {
        let ra = konst[2 * i];
        let bg = konst[2 * i + 1];
        constants.konst[i][0] = signed11(ra);
        constants.konst[i][3] = signed11(ra >> 12);
        constants.konst[i][2] = signed11(bg);
        constants.konst[i][1] = signed11(bg >> 12);
    }
    for i in 0..8usize {
        let image0 = bp[if i < 4 { 0x88 + i } else { 0xA8 + (i - 4) }];
        constants.texdims[i][0] = ((image0 & 0x3FF) + 1) as f32;
        constants.texdims[i][1] = (((image0 >> 10) & 0x3FF) + 1) as f32;
        constants.texdims[i][2] = ((bp[0x30 + 2 * i] & 0xFFFF) + 1) as f32;
        constants.texdims[i][3] = ((bp[0x31 + 2 * i] & 0xFFFF) + 1) as f32;
    }
    for i in 0..3usize {
        let a = bp[0x06 + 3 * i];
        let b = bp[0x07 + 3 * i];
        let c = bp[0x08 + 3 * i];
        let field = |value: u32| -> i32 { let v = (value & 0x7FF) as i32; if v & 0x400 != 0 { v - 0x800 } else { v } };
        let scale = (((a >> 22) & 3) | (((b >> 22) & 3) << 2) | (((c >> 22) & 3) << 4)) as i32;
        constants.indmtx[2 * i] = [field(a), field(b), field(c), 17 - scale];
        constants.indmtx[2 * i + 1] = [field(a >> 11), field(b >> 11), field(c >> 11), 17 - scale];
    }
    constants.indirect[0] = ((bp[0x00] >> 16) & 7) | ((bp[0x00] & 15) << 8);
    constants.indirect[1] = bp[0x27];
    constants.indirect[2] = bp[0x25];
    constants.indirect[3] = bp[0x26];
    let stages = ((bp[0x00] >> 10) & 15) + 1;
    let mut i = 0;
    while i < stages && (i as usize) < MAX_STAGES { constants.tevind[i as usize][0] = bp[0x10 + i as usize]; i += 1; }
    constants.header[0] = stages;
    constants.header[1] = bp[0xF3];
    constants.header[2] = bp[0x42];
    let rgba6 = (bp[0x43] & 7) == 1;
    constants.header[3] = (if rgba6 { 1 } else { 0 }) | (if rgba6 && (bp[0x41] & 4) != 0 { 2 } else { 0 })
        | ((blend_state(bp[0x41], true, false, true).output as u32) << 2);
    let fog_float = |value: u32| -> f32 {
        f32::from_bits((((value >> 19) & 1) << 31) | (((value >> 11) & 0xFF) << 23) | ((value & 0x7FF) << 12))
    };
    let fog_a = bp[0xEE];
    let fog_c = bp[0xF1];
    let nan_case = ((fog_a >> 11) & 0xFF) == 255 && ((fog_c >> 11) & 0xFF) == 255;
    constants.fog[0] = if nan_case { 0.0 } else { fog_float(fog_a) };
    constants.fog[1] = if nan_case {
        if ((fog_a >> 19) & 1) == 0 && ((fog_c >> 19) & 1) == 0 { f32::NEG_INFINITY } else { f32::INFINITY }
    } else { fog_float(fog_c) };
    constants.fog[2] = 0.0;
    constants.fog[3] = 1.0;
    constants.fog_integer[0] = ((fog_c >> 20) & 15) as i32;
    constants.fog_integer[1] = (bp[0xEF] & 0xFFFFFF) as i32;
    constants.fog_integer[2] = (bp[0xF2] & 0xFFFFFF) as i32;
    constants.fog_integer[3] = (bp[0xF0] & 0x1F) as i32;
    if bp[0xE8] & (1 << 10) != 0 {
        let width = f32::from_bits(xf[0x101A]);
        let center = (bp[0xE8] & 0x3FF) as i32 - 342;
        constants.fog[2] = (center as f32 / (2.0f32 * width)) * 2.0f32 - 1.0f32;
        constants.fog[3] = 2.0f32 * width * scale as f32;
        for i in 0..5usize {
            let k = bp[0xE9 + i];
            let (low, high) = (2 * i, 2 * i + 1);
            constants.fog_range[low / 4][low % 4] = ((k >> 12) & 0xFFF) as f32 / 256.0f32 * 4.0f32;
            constants.fog_range[high / 4][high % 4] = (k & 0xFFF) as f32 / 256.0f32 * 4.0f32;
        }
        constants.fog_integer[0] |= 16;
    }
    for table in 0..4usize {
        let low = bp[0xF6 + 2 * table];
        let high = bp[0xF7 + 2 * table];
        constants.swaps[table] = (low & 3) | (((low >> 2) & 3) << 2) | ((high & 3) << 4) | (((high >> 2) & 3) << 6);
    }
    let mut i = 0usize;
    while (i as u32) < stages && i < MAX_STAGES {
        let order_word = bp[0x28 + i / 2];
        let order = if i & 1 != 0 { order_word >> 12 } else { order_word };
        let map = order & 7;
        let coord = (order >> 3) & 7;
        let enable = (order >> 6) & 1;
        let channel = (order >> 7) & 7;
        let ksel_word = bp[0xF6 + i / 2];
        let ksel = if i & 1 != 0 { (ksel_word >> 14) & 0x3FF } else { (ksel_word >> 4) & 0x3FF };
        constants.stage[i][0] = bp[0xC0 + 2 * i];
        constants.stage[i][1] = bp[0xC1 + 2 * i];
        constants.stage[i][2] = map | (coord << 3) | (enable << 6) | (channel << 7);
        constants.stage[i][3] = (ksel & 31) | (((ksel >> 5) & 31) << 5);
        i += 1;
    }
}

/// C++ alpha_test_result. Renderer owns its exact-key cache; this pure computation also stops once both OR reductions saturate.
pub fn alpha_test_result(test: u32) -> AlphaTestResult {
    let compare = |mode: u32, value: u32, reference: u32| match mode {
        0 => false, 1 => value < reference, 2 => value == reference, 3 => value <= reference,
        4 => value > reference, 5 => value != reference, 6 => value >= reference, _ => true,
    };
    let mut result = AlphaTestResult::default();
    for alpha in 0..256u32 {
        let first = compare((test >> 16) & 7, alpha, test & 255);
        let second = compare((test >> 19) & 7, alpha, (test >> 8) & 255);
        let logic = (test >> 22) & 3;
        let accepted = if logic == 0 { first && second } else if logic == 1 { first || second } else if logic == 2 { first != second } else { first == second };
        result.can_pass |= accepted;
        result.can_fail |= !accepted;
        if result.can_pass && result.can_fail { break; }
    }
    result
}

pub fn copy_bits(format: u32) -> u32 {
    match format { 0 => 4, 1 | 2 | 7 | 8 | 9 | 10 => 8, 6 => 32, _ => 16 }
}
pub fn copy_texture_format(format: u32) -> u32 {
    const FORMATS: [u32; 13] = [0, 1, 2, 3, 4, 5, 6, 1, 1, 1, 1, 3, 3];
    FORMATS[format as usize]
}
#[inline] pub fn expand(value: u32, bits: i32) -> u32 { (value << (8 - bits)) | (value >> (2 * bits - 8)) }
#[inline] pub fn quantize(value: u32, bits: i32) -> u32 { expand(value >> (8 - bits), bits) }
#[inline] pub fn scale_bits(value: u32, bits: i32) -> u32 { (value * ((1u32 << bits) - 1) + 127) / 255 }

/// C++ encode_texel (gx_render_common.h).
pub fn encode_texel(block: &mut [u8], format: u32, index: u32, rgba: &[u8; 4]) {
    let (r, g, b, a) = (rgba[0] as u32, rgba[1] as u32, rgba[2] as u32, rgba[3] as u32);
    let i = index as usize;
    let put16 = |block: &mut [u8], offset: usize, value: u32| { block[offset] = (value >> 8) as u8; block[offset + 1] = value as u8; };
    match format {
        0 => {
            let byte = &mut block[i / 2];
            let nibble = scale_bits(r, 4);
            *byte = if index & 1 != 0 { ((*byte as u32 & 0xF0) | nibble) as u8 } else { ((*byte as u32 & 0x0F) | (nibble << 4)) as u8 };
        }
        1 => block[i] = r as u8,
        2 => block[i] = ((scale_bits(a, 4) << 4) | scale_bits(r, 4)) as u8,
        3 => { block[i * 2] = a as u8; block[i * 2 + 1] = r as u8; }
        4 => put16(block, i * 2, (scale_bits(r, 5) << 11) | (scale_bits(g, 6) << 5) | scale_bits(b, 5)),
        5 => {
            let alpha = scale_bits(a, 3);
            put16(block, i * 2, if alpha == 7 { 0x8000 | (scale_bits(r, 5) << 10) | (scale_bits(g, 5) << 5) | scale_bits(b, 5) }
                else { (alpha << 12) | (scale_bits(r, 4) << 8) | (scale_bits(g, 4) << 4) | scale_bits(b, 4) });
        }
        _ => { block[i * 2] = a as u8; block[i * 2 + 1] = r as u8; block[32 + i * 2] = g as u8; block[32 + i * 2 + 1] = b as u8; }
    }
}

/// Claude OPT B3a: exact specialization of the EFB write-back encode loop. Same formulas as encode_texel
/// (scale_bits through a 256-entry table per bit width), format match hoisted out of the per-pixel loop.
pub fn scale_lut() -> &'static [[u8; 256]; 9] {
    static L: std::sync::OnceLock<[[u8; 256]; 9]> = std::sync::OnceLock::new();
    L.get_or_init(|| { let mut t = [[0u8; 256]; 9]; for b in 1..9 { for v in 0..256u32 { t[b][v as usize] = scale_bits(v, b as i32) as u8; } } t })
}
#[inline(always)]
fn enc_fast<const F: u32>(block: &mut [u8; 64], i: usize, rgba: [u8; 4], l: &[[u8; 256]; 9]) {
    let (r, g, b, a) = (rgba[0] as usize, rgba[1] as usize, rgba[2] as usize, rgba[3] as usize);
    match F {
        0 => { let nibble = l[4][r]; let byte = &mut block[i / 2]; *byte = if i & 1 != 0 { (*byte & 0xF0) | nibble } else { (*byte & 0x0F) | (nibble << 4) }; }
        1 => block[i] = r as u8,
        2 => block[i] = (l[4][a] << 4) | l[4][r],
        3 => { block[i * 2] = a as u8; block[i * 2 + 1] = r as u8; }
        4 => { let v = ((l[5][r] as u32) << 11) | ((l[6][g] as u32) << 5) | l[5][b] as u32; block[i * 2] = (v >> 8) as u8; block[i * 2 + 1] = v as u8; }
        5 => { let alpha = l[3][a] as u32; let v = if alpha == 7 { 0x8000 | ((l[5][r] as u32) << 10) | ((l[5][g] as u32) << 5) | l[5][b] as u32 }
                 else { (alpha << 12) | ((l[4][r] as u32) << 8) | ((l[4][g] as u32) << 4) | l[4][b] as u32 }; block[i * 2] = (v >> 8) as u8; block[i * 2 + 1] = v as u8; }
        _ => { block[i * 2] = a as u8; block[i * 2 + 1] = r as u8; block[32 + i * 2] = g as u8; block[32 + i * 2 + 1] = b as u8; }
    }
}
/// Encodes all blocks exactly like the original nested loop in backend write_back. `row(by)` = guest row pointer.
pub unsafe fn encode_write_back(format: u32, pixels: &[u8], padded_row: usize, lw: u32, lh: u32, bw: u32, bh: u32, blocks_x: u32, blocks_y: u32, bytes: usize, row: &dyn Fn(u32) -> *mut u8) {
    unsafe fn run<const F: u32>(pixels: &[u8], padded_row: usize, lw: u32, lh: u32, bw: u32, bh: u32, blocks_x: u32, blocks_y: u32, bytes: usize, row: &dyn Fn(u32) -> *mut u8) {
        let l = scale_lut();
        for by in 0..blocks_y {
            let dst = row(by);
            for bx in 0..blocks_x {
                let mut block = [0u8; 64];
                // Claude OPT W1: interior blocks need no edge clamp; same pixels, same order, same bytes.
                if (bx + 1) * bw <= lw && (by + 1) * bh <= lh {
                    let base = pixels.as_ptr();
                    for ty in 0..bh {
                        let line = base.add(((by * bh + ty) as usize) * padded_row + (bx * bw) as usize * 4);
                        for tx in 0..bw {
                            let q = line.add(tx as usize * 4);
                            enc_fast::<F>(&mut block, (ty * bw + tx) as usize, [*q, *q.add(1), *q.add(2), *q.add(3)], l);
                        }
                    }
                    std::ptr::copy_nonoverlapping(block.as_ptr(), dst.add(bx as usize * bytes), bytes);
                    continue;
                }
                for ty in 0..bh {
                    let py = std::cmp::min(by * bh + ty, lh - 1) as usize;
                    let line = &pixels[py * padded_row..];
                    for tx in 0..bw {
                        let px = std::cmp::min(bx * bw + tx, lw - 1) as usize;
                        let o = px * 4;
                        enc_fast::<F>(&mut block, (ty * bw + tx) as usize, [line[o], line[o + 1], line[o + 2], line[o + 3]], l);
                    }
                }
                std::ptr::copy_nonoverlapping(block.as_ptr(), dst.add(bx as usize * bytes), bytes);
            }
        }
    }
    match format {
        0 => run::<0>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
        1 => run::<1>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
        2 => run::<2>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
        3 => run::<3>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
        4 => run::<4>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
        5 => run::<5>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
        _ => run::<255>(pixels, padded_row, lw, lh, bw, bh, blocks_x, blocks_y, bytes, row),
    }
}

/// C++ guest_range_valid (gx_render.cpp).
#[inline] pub fn guest_range_valid(address: u32, size: usize) -> bool { crate::guest_memory::range_valid(address,size) }

/// C++ load_tlut.
pub fn load_tlut(tlut: &mut Tlut, address: u32, tmem_offset: u32, bytes: u32) {
    let tmem_offset = tmem_offset & TLUT_MASK;
    let bytes = std::cmp::min(bytes, TLUT_SIZE - tmem_offset);
    if !guest_range_valid(address, bytes as usize) { return; }
    unsafe { std::ptr::copy_nonoverlapping(crate::guest_memory::host(address), tlut.0.as_mut_ptr().add(tmem_offset as usize), bytes as usize); }
}

/// C++ describe_texture(map).
pub fn describe_texture(map: u32, bp: &[u32], tlut: &Tlut) -> TextureRequest {
    let m = map as usize;
    let mut request = TextureRequest::default();
    let image0 = bp[if m < 4 { 0x88 + m } else { 0xA8 + (m - 4) }];
    let image3 = bp[if m < 4 { 0x94 + m } else { 0xB4 + (m - 4) }];
    request.width = (image0 & 0x3FF) + 1;
    request.height = ((image0 >> 10) & 0x3FF) + 1;
    request.format = (image0 >> 20) & 15;
    request.address = 0x80000000u32 | ((image3 & 0x00FFFFFF) << 5);
    let layout = layout_for(request.format);
    request.supported = layout.supported;
    let mode0 = bp[if m < 4 { 0x80 + m } else { 0xA0 + (m - 4) }];
    let mode1 = bp[if m < 4 { 0x84 + m } else { 0xA4 + (m - 4) }];
    request.mipmaps = ((mode0 >> 5) & 3) != 0;
    if request.mipmaps {
        let requested = (((mode1 >> 8) & 0xFF) + 15) / 16 + 1;
        let largest = std::cmp::max(request.width, request.height);
        let mut available = 1u32;
        while (largest >> available) != 0 { available += 1; }
        request.levels = *[requested, available, 12].iter().min().unwrap();
    }
    let format = request.format;
    let (w0, h0) = (request.width, request.height);
    let level_size = |level: u32| -> usize {
        let lw = std::cmp::max(w0 >> level, 1);
        let lh = std::cmp::max(h0 >> level, 1);
        let pw = (lw + layout.block_width - 1) / layout.block_width * layout.block_width;
        let ph = (lh + layout.block_height - 1) / layout.block_height * layout.block_height;
        (pw / layout.block_width) as usize * (ph / layout.block_height) as usize * block_bytes(format) as usize
    };
    request.size = level_size(0);
    if !guest_range_valid(request.address, request.size) { return request; }
    request.total = request.size;
    let mut level = 1;
    while level < request.levels {
        let next = request.total + level_size(level);
        if !guest_range_valid(request.address, next) { request.levels = level; break; }
        request.offsets[level as usize] = request.total;
        request.total = next;
        level += 1;
    }
    request.source = crate::guest_memory::host(request.address);
    request.key = ((request.address as u64) << 20) ^ ((request.width as u64) << 8) ^ ((request.height as u64) << 32)
        ^ request.format as u64 ^ ((request.levels as u64) << 4);
    if (8..=10).contains(&request.format) {
        let tlut_register = bp[if m < 4 { 0x98 + m } else { 0xB8 + (m - 4) }];
        let tlut_offset = (tlut_register & 0x3FF) << 9;
        request.tlut_format = (tlut_register >> 10) & 3;
        request.tlut = unsafe { tlut.0.as_ptr().add((tlut_offset & TLUT_MASK) as usize) };
        request.entries = if request.format == 8 { 16 } else if request.format == 9 { 256 } else { 16384 };
        request.key ^= (((tlut_offset >> 9) as u64) << 54) ^ ((request.tlut_format as u64) << 52);
    }
    request.valid = true;
    request
}

/// Claude OPT G4: process-local texture-cache key (never persisted or compared with C++; texture_hash/hash_bytes stay for
/// the common_diff parity tool). 4 independent multiply lanes overlap instead of one serial FNV multiply chain.
unsafe fn fast_hash_bytes(data: *const u8, size: usize) -> u64 {
    const M: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut lane = [0x243F_6A88_85A3_08D3u64, 0x1319_8A2E_0370_7344, 0xA409_3822_299F_31D0, 0x082E_FA98_EC4E_6C89];
    let blocks = size / 32;
    for i in 0..blocks {
        let p = data.add(i * 32);
        for l in 0..4 {
            let w = std::ptr::read_unaligned(p.add(l * 8) as *const u64);
            lane[l] = (lane[l] ^ w).wrapping_mul(M).rotate_left(31);
        }
    }
    let mut hash = (size as u64).wrapping_mul(M);
    for l in 0..4 { hash = (hash ^ lane[l]).wrapping_mul(M).rotate_left(27); }
    let mut i = blocks * 32;
    while i + 8 <= size { hash = (hash ^ std::ptr::read_unaligned(data.add(i) as *const u64)).wrapping_mul(M).rotate_left(27); i += 8; }
    while i < size { hash = (hash ^ *data.add(i) as u64).wrapping_mul(M).rotate_left(27); i += 1; }
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    hash ^ (hash >> 33)
}

/// Claude OPT G4: same inputs as texture_hash, used only for the in-process entry.hash equality check.
pub fn texture_cache_hash(request: &TextureRequest) -> u64 {
    unsafe {
        let mut hash = fast_hash_bytes(request.source, request.total);
        if !request.tlut.is_null() {
            hash = (hash ^ fast_hash_bytes(request.tlut, request.entries as usize * 2)).wrapping_mul(1099511628211) ^ request.tlut_format as u64;
        }
        hash
    }
}

/// C++ texture_hash.
pub fn texture_hash(request: &TextureRequest) -> u64 {
    unsafe {
        let mut hash = hash_bytes(request.source, request.total);
        if !request.tlut.is_null() {
            hash = (hash ^ hash_bytes(request.tlut, request.entries as usize * 2)).wrapping_mul(1099511628211) ^ request.tlut_format as u64;
        }
        hash
    }
}

/// C++ decode_levels.
pub fn decode_levels(request: &TextureRequest) -> Vec<Vec<u32>> {
    (0..request.levels).map(|level| {
        let lw = std::cmp::max(request.width >> level, 1);
        let lh = std::cmp::max(request.height >> level, 1);
        unsafe { decode_texture(request.source.add(request.offsets[level as usize]), lw, lh, request.format, request.tlut, request.tlut_format) }
    }).collect()
}

/// Reuse CPU decode storage across uploads. Queue::write_texture consumes borrowed bytes before returning.
pub fn decode_levels_into(request: &TextureRequest, levels: &mut Vec<Vec<u32>>, padded: &mut Vec<u32>) {
    levels.resize_with(request.levels as usize, Vec::new);
    levels.truncate(request.levels as usize);
    for level in 0..request.levels {
        let lw = std::cmp::max(request.width >> level, 1);
        let lh = std::cmp::max(request.height >> level, 1);
        unsafe { decode_texture_into(request.source.add(request.offsets[level as usize]), lw, lh, request.format,
                                     request.tlut, request.tlut_format, &mut levels[level as usize], padded); }
    }
}

/// C++ sampler_params(map, custom).
pub fn sampler_params(map: u32, custom: bool, bp: &[u32]) -> SamplerParams {
    let m = map as usize;
    let mode = bp[if m < 4 { 0x80 + m } else { 0xA0 + (m - 4) }];
    let lod = bp[if m < 4 { 0x84 + m } else { 0xA4 + (m - 4) }];
    let mut params = SamplerParams::default();
    params.key = (mode & 0x3FFFFF) as u64 | (((lod & 0xFFFF) as u64) << 32) | ((custom as u64) << 48);
    params.wrap_s = mode & 3;
    params.wrap_t = (mode >> 2) & 3;
    params.mag_linear = ((mode >> 4) & 1) != 0;
    params.mip_mode = (mode >> 5) & 3;
    params.min_linear = ((mode >> 7) & 1) != 0;
    if params.mip_mode != 0 {
        let max_lod = (lod >> 8) & 0xFF;
        let min_lod = std::cmp::min(lod & 0xFF, max_lod);
        params.min_lod = min_lod as f32 / 16.0f32;
        params.max_lod = max_lod as f32 / 16.0f32;
        params.unlimited_lod = custom;
        params.lod_bias = (((mode >> 9) & 0xFF) as u8 as i8) as f32 / 32.0f32;
    }
    params
}
