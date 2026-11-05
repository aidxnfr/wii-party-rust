// SPDX-License-Identifier: GPL-3.0-or-later
//! 1:1 port of engine/src/gpu/custom_textures.cpp (texture-pack naming, xxHash64, pack index). decode_png stays the
//! engine's C++ leaf (Windows WIC) behind backend.rs PackHost::load_png, so decoded pixels are the original codec's.
//! Differential test: diff/ct_ref.cpp (compiles the pristine file) vs examples/ct_diff.rs -> identical output.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const PRIME1: u64 = 0x9E3779B185EBCA87;
const PRIME2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME3: u64 = 0x165667B19E3779F9;
const PRIME4: u64 = 0x85EBCA77C2B2AE63;
const PRIME5: u64 = 0x27D4EB2F165667C5;

fn read64(p: &[u8]) -> u64 { u64::from_le_bytes(p[..8].try_into().unwrap()) }
fn read32(p: &[u8]) -> u32 { u32::from_le_bytes(p[..4].try_into().unwrap()) }
fn round(acc: u64, input: u64) -> u64 { acc.wrapping_add(input.wrapping_mul(PRIME2)).rotate_left(31).wrapping_mul(PRIME1) }
fn merge(acc: u64, value: u64) -> u64 { (acc ^ round(0, value)).wrapping_mul(PRIME1).wrapping_add(PRIME4) }
fn hex64(value: u64) -> String { format!("{value:016x}") }
/// C++ lowercase(): std::tolower per byte in the "C" locale = ASCII only.
fn lowercase(text: &str) -> String { text.to_ascii_lowercase() }
fn is_palette(format: u32) -> bool { (8..=10).contains(&format) }

/// custom_textures.cpp xxh64.
pub fn xxh64(data: &[u8], seed: u64) -> u64 {
    let size = data.len();
    let mut p = 0usize;
    let mut hash;
    if size >= 32 {
        let mut v1 = seed.wrapping_add(PRIME1).wrapping_add(PRIME2);
        let mut v2 = seed.wrapping_add(PRIME2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME1);
        let limit = size - 32;
        loop {
            v1 = round(v1, read64(&data[p..]));
            v2 = round(v2, read64(&data[p + 8..]));
            v3 = round(v3, read64(&data[p + 16..]));
            v4 = round(v4, read64(&data[p + 24..]));
            p += 32;
            if p > limit { break; }
        }
        hash = v1.rotate_left(1).wrapping_add(v2.rotate_left(7)).wrapping_add(v3.rotate_left(12)).wrapping_add(v4.rotate_left(18));
        hash = merge(hash, v1);
        hash = merge(hash, v2);
        hash = merge(hash, v3);
        hash = merge(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME5);
    }
    hash = hash.wrapping_add(size as u64);
    while size - p >= 8 {
        hash ^= round(0, read64(&data[p..]));
        hash = hash.rotate_left(27).wrapping_mul(PRIME1).wrapping_add(PRIME4);
        p += 8;
    }
    if size - p >= 4 {
        hash ^= (read32(&data[p..]) as u64).wrapping_mul(PRIME1);
        hash = hash.rotate_left(23).wrapping_mul(PRIME2).wrapping_add(PRIME3);
        p += 4;
    }
    while p < size {
        // C++: *p * kPrime5 — uint8_t promoted to int, then converted to uint64_t for the multiply (value < 256: same).
        hash ^= (data[p] as u64).wrapping_mul(PRIME5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME1);
        p += 1;
    }
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME3);
    hash ^= hash >> 32;
    hash
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaletteRange { pub first: u32, pub count: u32 }

/// custom_textures.cpp used_palette_range.
pub fn used_palette_range(data: &[u8], format: u32) -> PaletteRange {
    if !is_palette(format) || data.is_empty() { return PaletteRange::default(); }
    let (mut low, mut high) = (0xFFFFu32, 0u32);
    let mut include = |index: u32| { low = low.min(index); high = high.max(index); };
    if format == 8 {
        for &b in data { include((b & 15) as u32); include((b >> 4) as u32); }
    } else if format == 9 {
        for &b in data { include(b as u32); }
    } else {
        let mut i = 0;
        while i + 1 < data.len() { include((((data[i] as u32) << 8) | data[i + 1] as u32) & 0x3FFF); i += 2; }
    }
    if low > high { return PaletteRange::default(); }
    PaletteRange { first: low, count: high + 1 - low }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextureName { pub base: String, pub texture: String, pub tlut: String, pub format: String }
impl TextureName {
    pub fn full(&self) -> String { format!("{}_{}{}_{}", self.base, self.texture, self.tlut, self.format) }
    pub fn wildcard(&self) -> String { format!("{}_{}_$_{}", self.base, self.texture, self.format) }
}

/// custom_textures.cpp name_texture. `tlut` = TMEM palette bytes from the TLUT start (None = nullptr).
/// Safety of the C++ original: it reads tlut + first*2 .. + count*2 unchecked; callers pass the 64 KiB TMEM palette copy.
pub fn name_texture(data: &[u8], width: u32, height: u32, format: u32, mipmaps: bool, tlut: Option<&[u8]>) -> TextureName {
    let mut name = TextureName {
        base: format!("tex1_{width}x{height}{}", if mipmaps { "_m" } else { "" }),
        texture: hex64(xxh64(data, 0)),
        ..Default::default()
    };
    if let Some(t) = tlut {
        if is_palette(format) {
            let range = used_palette_range(data, format);
            if range.count > 0 {
                let start = range.first as usize * 2;
                let end = start + range.count as usize * 2;
                // Out-of-range reads are undefined in the C++ original; here they are clamped to the palette copy.
                name.tlut = format!("_{}", hex64(xxh64(&t[start.min(t.len())..end.min(t.len())], 0)));
            }
        }
    }
    name.format = format.to_string();
    name
}

/// custom_textures.cpp level_name.
pub fn level_name(name: &str, level: u32) -> String { if level == 0 { name.to_string() } else { format!("{name}_mip{level}") } }

/// custom_textures.cpp Index (lowercased stem -> file; first file wins).
#[derive(Default)]
pub struct Index { files: HashMap<String, PathBuf> }
impl Index {
    /// Index::scan: std::filesystem::recursive_directory_iterator (pre-order, directory symlinks not followed,
    /// permission-denied skipped, first error stops), regular files only (symlinks to files followed).
    pub fn scan(&mut self, directory: &Path) -> usize {
        if !directory.is_dir() { return 0; }
        let mut added = 0;
        let mut failed = false;
        self.walk(directory, &mut added, &mut failed);
        added
    }
    fn walk(&mut self, directory: &Path, added: &mut usize, failed: &mut bool) {
        let entries = match std::fs::read_dir(directory) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return,
            Err(_) => { *failed = true; return; }
        };
        for entry in entries {
            if *failed { return; }
            let Ok(entry) = entry else { *failed = true; return };
            let path = entry.path();
            let Ok(kind) = entry.file_type() else { *failed = true; return };
            if path.is_file() && self.add(&path) { *added += 1; }
            if kind.is_dir() { self.walk(&path, added, failed); }
        }
    }
    /// Index::add.
    pub fn add(&mut self, file: &Path) -> bool {
        let extension = file.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        if lowercase(&extension) != ".png" { return false; }
        let mut key = lowercase(&file.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default());
        if key.len() > 4 && key.ends_with("_arb") { key.truncate(key.len() - 4); }
        if !key.starts_with("tex1_") { return false; }
        if self.files.contains_key(&key) { return false; }
        self.files.insert(key, file.to_path_buf());
        true
    }
    /// Index::find.
    pub fn find(&self, name: &str) -> Option<&PathBuf> { self.files.get(&lowercase(name)) }
    /// Index::resolve: wildcard first, then the full name; "" when none.
    pub fn resolve(&self, name: &TextureName) -> String {
        if self.files.is_empty() { return String::new(); }
        let wildcard = name.wildcard();
        if self.find(&wildcard).is_some() { return lowercase(&wildcard); }
        let full = name.full();
        if self.find(&full).is_some() { return lowercase(&full); }
        String::new()
    }
    pub fn size(&self) -> usize { self.files.len() }
}

/// custom_textures.cpp kMaxDimension.
const MAX_DIMENSION: u32 = 16384;

/// custom_textures.cpp decode_png: the image as 32bpp RGBA (byte order R,G,B,A in memory, i.e. u32 = R | G<<8 | B<<16 | A<<24),
/// or None. The original asks Windows WIC for GUID_WICPixelFormat32bppRGBA; this reproduces WIC's conversion as measured on
/// Windows (claude-gx/png-diff: 40 colour type / depth / tRNS / gAMA / sRGB / sBIT / interlace cases + every 16-bit value):
/// - 16-bit RGB, RGBA, gray+alpha and gray-with-tRNS: high byte (truncation); gAMA/sRGB/sBIT ignored.
/// - 16-bit gray without tRNS: WIC's own table (src/wic_gray16.bin, all 65536 values measured; not a plain formula).
/// - 1/2/4/8-bit gray with tRNS: pixels equal to the key become 0x00000000 (colour zeroed); other kinds keep the colour.
pub fn decode_png(path: &Path) -> Option<(Vec<u32>, u32, u32)> {
    let file = std::fs::File::open(path).ok()?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND);
    decoder.set_ignore_text_chunk(true);
    let mut reader = decoder.read_info().ok()?;
    let (width, height) = (reader.info().width, reader.info().height);
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return None;
    }
    let mut buffer = vec![0u8; reader.output_buffer_size()?];
    let frame = reader.next_frame(&mut buffer).ok()?;
    let (color, depth) = reader.output_color_type();
    let source = reader.info().color_type;
    let keyed_gray = source == png::ColorType::Grayscale && reader.info().trns.is_some();
    let gray_table = source == png::ColorType::Grayscale && !keyed_gray && depth == png::BitDepth::Sixteen;
    let data = &buffer[..frame.buffer_size()];
    let channels = color.samples();
    let wide = depth == png::BitDepth::Sixteen;
    let step = channels * if wide { 2 } else { 1 };
    let sample = |p: &[u8], c: usize| -> u8 { if wide { narrow16(u16::from_be_bytes([p[2 * c], p[2 * c + 1]])) } else { p[c] } };
    let mut pixels = Vec::with_capacity(width as usize * height as usize);
    for row in data.chunks_exact(frame.line_size).take(height as usize) {
        for p in row.chunks_exact(step).take(width as usize) {
            let (r, g, b, a) = match color {
                png::ColorType::Grayscale if gray_table => { let v = WIC_GRAY16[u16::from_be_bytes([p[0], p[1]]) as usize]; (v, v, v, 255) }
                png::ColorType::Grayscale => { let v = sample(p, 0); (v, v, v, 255) }
                png::ColorType::GrayscaleAlpha if keyed_gray && !wide && p[1] == 0 => (0, 0, 0, 0),
                png::ColorType::GrayscaleAlpha => { let v = sample(p, 0); (v, v, v, sample(p, 1)) }
                png::ColorType::Rgb => (sample(p, 0), sample(p, 1), sample(p, 2), 255),
                png::ColorType::Rgba => (sample(p, 0), sample(p, 1), sample(p, 2), sample(p, 3)),
                png::ColorType::Indexed => return None,
            };
            pixels.push(u32::from_le_bytes([r, g, b, a]));
        }
    }
    Some((pixels, width, height))
}

/// 16-bit sample -> 8-bit as WIC's converter does it for every kind except plain 16-bit gray (measured: high byte).
fn narrow16(v: u16) -> u8 { (v >> 8) as u8 }

/// WIC 16bppGray -> 32bppRGBA grey level for each 16-bit value, measured on Windows (claude-gx/png-diff corpus2 gray16_all).
static WIC_GRAY16: &[u8; 65536] = include_bytes!("wic_gray16.bin");
