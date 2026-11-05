// SPDX-License-Identifier: GPL-3.0-or-later
//! 1:1 port of engine/src/gpu/gx.cpp (GX command processor: FIFO, CP/XF/BP registers, vertex decode, transform,
//! lighting, texgen, projection, culling, primitive assembly, EFB copy dispatch, GPU thread). Claude lane.
//!
//! Exactness notes:
//! * The original is compiled with -O3 and no -mfma (no contraction): every f32 expression here keeps the same
//!   operand order and uses plain IEEE f32 ops, so results are bit-identical.
//! * Global state mirrors the C++ globals (one CPU thread pushes, one GPU thread parses, the renderer reads the
//!   register files during draws), so it lives in UnsafeCell statics exactly like the C++ file-scope variables.
//! * Engine services that stay C++ (log, settings, video, profile, thread clocks) come through [`GxHost`].
use crate::backend::{CopyFilter, ScreenVertex};
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use crate::gx_lighting as lighting;
use wprt_ports::gx_state;

/// C++ engine services used by gx.cpp (provided by claude-gx/cpp/gx_rust.cpp).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GxHost {
    /// log::write(category, "%s", text)
    pub log: extern "C" fn(category: *const c_char, text: *const c_char),
    /// static settings::LiveFlag filter_enabled("video.copy_filter", "WP_COPY_FILTER"); filter_enabled()
    pub copy_filter_enabled: extern "C" fn() -> bool,
    /// video::save_next_frame(path)
    pub save_next_frame: extern "C" fn(path: *const c_char),
    /// video::update_statistics(fps)
    pub update_statistics: extern "C" fn(fps: f64),
    /// wp::idle_share()
    pub idle_share: extern "C" fn() -> f64,
    /// gx.cpp private_memory_mb()
    pub private_memory_mb: extern "C" fn() -> usize,
    /// gx.cpp current_thread_clock(): an opaque per-thread clock handle of the calling thread
    pub current_thread_clock: extern "C" fn() -> u64,
    /// gx.cpp thread_seconds(clock)
    pub thread_seconds: extern "C" fn(clock: u64) -> f64,
}

/// Where GX sends renderer calls: the wgpu backend, or a recorder in the differential test.
pub trait RenderSink: Sync {
    fn draw(&self, vertices: &[ScreenVertex]);
    fn finish_copies(&self);
    fn invalidate_textures(&self);
    fn load_tlut(&self, address: u32, tmem_offset: u32, bytes: u32);
    fn clear(&self, x: i32, y: i32, width: i32, height: i32);
    fn copy_to_framebuffer(&self, x: i32, y: i32, width: i32, height: i32, depth: bool, filter: &CopyFilter);
    #[allow(clippy::too_many_arguments)]
    fn copy_to_texture(&self, address: u32, stride: u32, x: i32, y: i32, width: i32, height: i32, half: bool, format: u32, intensity: bool, depth: bool, alpha: bool,
                       filter: &CopyFilter);
    fn take_statistics(&self) -> (u32, u32, f64);
    fn take_copy_statistics(&self) -> (u32, f64);
    fn dump_copies(&self, prefix: Option<&str>);
}
struct Wgpu;
impl RenderSink for Wgpu {
    fn draw(&self, v: &[ScreenVertex]) { crate::backend::render_draw(v) }
    fn finish_copies(&self) { crate::backend::render_finish_copies() }
    fn invalidate_textures(&self) { crate::backend::render_invalidate_textures() }
    fn load_tlut(&self, a: u32, o: u32, b: u32) { crate::backend::render_load_tlut(a, o, b) }
    fn clear(&self, x: i32, y: i32, w: i32, h: i32) { crate::backend::render_clear(x, y, w, h) }
    fn copy_to_framebuffer(&self, x: i32, y: i32, w: i32, h: i32, d: bool, f: &CopyFilter) { crate::backend::render_copy_to_framebuffer(x, y, w, h, d, f) }
    fn copy_to_texture(&self, a: u32, s: u32, x: i32, y: i32, w: i32, h: i32, half: bool, fmt: u32, i: bool, d: bool, al: bool, f: &CopyFilter) {
        crate::backend::render_copy_to_texture(a, s, x, y, w, h, half, fmt, i, d, al, f)
    }
    fn take_statistics(&self) -> (u32, u32, f64) { crate::backend::render_take_statistics() }
    fn take_copy_statistics(&self) -> (u32, f64) { crate::backend::render_take_copy_statistics() }
    fn dump_copies(&self, p: Option<&str>) { crate::backend::render_dump_copies(p) }
}
static SINK: OnceLock<&'static dyn RenderSink> = OnceLock::new();
fn backend() -> &'static dyn RenderSink { *SINK.get_or_init(|| &Wgpu) }
/// Differential tests: route renderer calls to a recorder (must be called before the first GX command).
pub fn set_render_sink(sink: &'static dyn RenderSink) { let _ = SINK.set(sink); }

static HOST: OnceLock<GxHost> = OnceLock::new();
fn host() -> &'static GxHost { HOST.get().expect("wpgx_gx_install not called") }
fn log(category: &str, text: String) {
    let c = CString::new(category).unwrap();
    let t = CString::new(text.replace('\0', " ")).unwrap();
    (host().log)(c.as_ptr(), t.as_ptr());
}

/// A C++-style global: shared without locking, exactly as the original file-scope variables.
struct Global<T>(UnsafeCell<T>);
unsafe impl<T> Sync for Global<T> {}
impl<T> Global<T> {
    const fn new(v: T) -> Self { Global(UnsafeCell::new(v)) }
    #[allow(clippy::mut_from_ref)]
    fn get(&self) -> &mut T { unsafe { &mut *self.0.get() } }
}

const FLUSH_THRESHOLD: usize = 1 << 20;
const MAX_QUEUED_BYTES: usize = 64 << 20;
static QUEUED_BYTES: AtomicUsize = AtomicUsize::new(0);

const COMMAND_NOP: u8 = 0x00;
const COMMAND_LOAD_CP: u8 = 0x08;
const COMMAND_LOAD_XF: u8 = 0x10;
const COMMAND_CALL_LIST: u8 = 0x40;
const COMMAND_METRICS: u8 = 0x44;
const COMMAND_INVALIDATE: u8 = 0x48;
const COMMAND_LOAD_BP: u8 = 0x61;
const COMMAND_DRAW_MASK: u8 = 0x80;
const BP_MASK: u32 = 0xFE;
const BP_COPY_EXECUTE: u32 = 0x52;
const BP_LOAD_TLUT: u32 = 0x65;
const BP_INVALIDATE_TEXTURE: u32 = 0x66;
const CP_FIFO_BASE_LOW: u32 = 0xCC000020;
const CP_FIFO_BASE_HIGH: u32 = 0xCC000022;
const PI_FIFO_BASE: u32 = 0xCC00300C;
const PI_FIFO_END: u32 = 0xCC003010;
const PI_FIFO_WRITE_POINTER: u32 = 0xCC003014;
const CP_CONTROL: u32 = 0xCC000002;
const PE_CONTROL: u32 = 0xCC00100A;
const PE_TOKEN_ENABLE: u16 = 0x0001;
const PE_FINISH_ENABLE: u16 = 0x0002;
const PE_FINISH_CLEAR: u16 = 0x0008;
const CP_FIFO_END_LOW: u32 = 0xCC000024;
const CP_DISTANCE_LOW: u32 = 0xCC000030;
const CP_WRITE_POINTER_LOW: u32 = 0xCC000034;
const CP_READ_POINTER_LOW: u32 = 0xCC000038;
const CP_READ_ENABLE: u16 = 0x0001;
const CP_LINK_ENABLE: u16 = 0x0010;
const GATHER_PIPE_SIZE: usize = 32;
const FIFO_POINTER_MASK: u32 = 0x03FFFFE0;
const FIFO_PHYSICAL_MASK: u32 = 0x03FFFFFF;
const COPY_TO_FRAMEBUFFER: u32 = 1 << 14;
const BP_DRAW_DONE: u32 = 0x45;
const COPY_CLEAR: u32 = 1 << 11;
const PIXEL_FORMAT_RGBA6_Z24: u32 = 1;
const PIXEL_FORMAT_Z24: u32 = 3;
const XF_SIZE: usize = 0x1100;
const ARRAY_COUNT: usize = 16;
const RAM_BASE: u32 = 0x80000000;
const ARRAY_ADDRESS_MASK: u32 = 0x1FFFFFFF;
const VIEWPORT_OFFSET: f32 = 342.0;
const EFB_WIDTH: f32 = 640.0;
const EFB_HEIGHT: f32 = 528.0;
const DEPTH_RANGE: f32 = 16777215.0;
const MAX_LOGGED_COPIES: i32 = 60;
const TEX_COORD_COUNT: usize = 8;
const ARRAY_READ_MARGIN: usize = 256;
const SLOW_FRAME_MS: f64 = 40.0;

const NONE: u32 = 0;
const DIRECT: u32 = 1;
const INDEX8: u32 = 2;

#[derive(Clone, Copy, Default)]
struct Statistics { draws: u32, vertices: u32, bp_writes: u32, cp_writes: u32, xf_words: u32, lists: u32, unknown: u32 }

#[derive(Clone, Copy, Default)]
struct Element { mode: u32, components: u32, format: u32, shift: u32, direct_size: u32 }

#[derive(Clone, Copy, Default)]
struct Layout {
    position_matrix: bool,
    texture_matrices: u32,
    position: Element,
    normal: Element,
    color: [Element; 2],
    texture: [Element; TEX_COORD_COUNT],
    size: u32,
}

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [[f32; 4]; 2],
    texture: [[f32; 2]; TEX_COORD_COUNT],
    position_matrix: u32,
    texture_matrix: [u32; TEX_COORD_COUNT],
    has_color: [bool; 2],
}
impl Default for Vertex {
    fn default() -> Self {
        Vertex { position: [0.0; 3], normal: [0.0, 0.0, 1.0], color: [[1.0; 4]; 2], texture: [[0.0; 2]; TEX_COORD_COUNT], position_matrix: 0,
                 texture_matrix: [0; TEX_COORD_COUNT], has_color: [false; 2] }
    }
}

#[derive(Clone, Copy, Default)]
struct ArrayState { address: u32, stride: usize, base: usize }

#[derive(Clone, Copy, Default)]
struct TexGenUnit { source: u32, projected: bool, abc1: bool, post: u32 }
#[derive(Clone, Copy, Default)]
struct TexGenState { count: u32, dual: bool, unit: [TexGenUnit; TEX_COORD_COUNT] }

#[derive(Clone, Copy, Default)]
struct ProjectionState { a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, orthographic: bool, x_scale: f32, x_offset: f32, y_scale: f32, y_offset: f32, z_scale: f32, z_offset: f32 }

#[derive(Clone, Copy, Default)]
struct Prepared { vertex: ScreenVertex, valid: bool }

#[derive(Clone, Copy, Default)]
struct CachedLayout { key: [u32; 5], valid: bool, layout: Layout }

/// Parser-side state (C++ anonymous-namespace globals touched by parse on the GPU thread).
struct Parser {
    vcd_low: u32,
    vcd_high: u32,
    vat: [[u32; 8]; 3],
    array_base: [u32; ARRAY_COUNT],
    array_stride: [u32; ARRAY_COUNT],
    bp: [u32; 256],
    konst: [u32; 8],
    bp_mask: u32,
    xf: [u32; XF_SIZE],
    stats: Statistics,
    logged_copies: i32,
    copy_total: i32,
    seen_register: Box<[[bool; 0x1100]; 3]>,
    logged_draws: i32,
    logged_prepared: i32,
    frames: i32,
    fps_start: std::time::Instant,
    previous_frame: Option<std::time::Instant>,
    capture: Option<std::fs::File>,
    capture_index: i32,
    capture_draws: i32,
    capture_first: bool,
    captured_bp: [u32; 256],
    captured_konst: [u32; 8],
    captured_xf: Box<[u32; XF_SIZE]>,
    capture_name: String,
    capture_schedule: Option<(std::time::Instant, String, usize)>,
    arrays: [ArrayState; ARRAY_COUNT],
    default_texture_matrix: [u32; TEX_COORD_COUNT],
    layouts: [CachedLayout; 8],
    texgen: TexGenState,
    projection: ProjectionState,
    indirect_draws: u32,
    coordinate_draws: u32,
    vertices: Vec<Prepared>,
    triangles: Vec<ScreenVertex>,
    previous_cpu: f64,
    previous_gpu: f64,
    gpu_clock: Option<u64>,
}

struct Settings { log_from: i32, log: bool, log_draws: bool, render_enabled: bool, log_fps: bool, threaded: bool }
fn settings() -> &'static Settings {
    static S: OnceLock<Settings> = OnceLock::new();
    S.get_or_init(|| {
        let level = std::env::var("WP_LOG_GX").ok();
        Settings {
            // C++ std::atoi: leading integer, 0 when none
            log_from: std::env::var("WP_LOG_FROM").map(|v| atoi(&v)).unwrap_or(0),
            log: level.is_some(),
            log_draws: level.as_deref().map(|l| l.as_bytes().first() == Some(&b'2')).unwrap_or(false),
            render_enabled: std::env::var_os("WP_NO_RENDER").is_none(),
            log_fps: std::env::var_os("WP_LOG_FPS").is_some(),
            threaded: std::env::var("WP_GPU_THREAD").map(|v| atoi(&v) != 0).unwrap_or(true),
        }
    })
}
/// C atoi: optional whitespace, sign, digits; stops at the first other character; 0 when no digits.
fn atoi(text: &str) -> i32 {
    let t = text.trim_start();
    let (neg, rest) = match t.as_bytes().first() { Some(b'-') => (true, &t[1..]), Some(b'+') => (false, &t[1..]), _ => (false, t) };
    let mut v: i64 = 0;
    for b in rest.bytes() { if !b.is_ascii_digit() { break; } v = (v * 10 + (b - b'0') as i64).min(i64::from(i32::MAX) + 1); }
    (if neg { -v } else { v }) as i32
}

static PARSER: OnceLock<Global<Parser>> = OnceLock::new();
fn parser() -> &'static mut Parser {
    PARSER.get_or_init(|| Global::new(Parser {
        vcd_low: 0, vcd_high: 0, vat: [[0; 8]; 3], array_base: [0; ARRAY_COUNT], array_stride: [0; ARRAY_COUNT], bp: [0; 256], konst: [0; 8],
        bp_mask: 0xFFFFFF, xf: [0; XF_SIZE], stats: Statistics::default(), logged_copies: 0, copy_total: 0, seen_register: Box::new([[false; 0x1100]; 3]),
        logged_draws: 0, logged_prepared: 0, frames: 0, fps_start: std::time::Instant::now(), previous_frame: None, capture: None, capture_index: 0,
        capture_draws: 0, capture_first: true, captured_bp: [0; 256], captured_konst: [0; 8], captured_xf: Box::new([0; XF_SIZE]), capture_name: String::new(),
        capture_schedule: None, arrays: [ArrayState::default(); ARRAY_COUNT], default_texture_matrix: [0; TEX_COORD_COUNT], layouts: [CachedLayout::default(); 8],
        texgen: TexGenState::default(), projection: ProjectionState::default(), indirect_draws: 0, coordinate_draws: 0, vertices: Vec::new(),
        triangles: Vec::new(), previous_cpu: 0.0, previous_gpu: 0.0, gpu_clock: None,
    })).get()
}

static FRAMES_DRAWN: AtomicU64 = AtomicU64::new(0);
static CAPTURE_REQUESTED: AtomicBool = AtomicBool::new(false);
static SKIPPED_PRESENTS: AtomicU32 = AtomicU32::new(0);
static FINISH_PENDING: AtomicBool = AtomicBool::new(false);
static CPU_CLOCK_KNOWN: AtomicBool = AtomicBool::new(false);
static CPU_CLOCK: AtomicU64 = AtomicU64::new(0);

#[inline] fn be32(d: &[u8]) -> u32 { u32::from_be_bytes([d[0], d[1], d[2], d[3]]) }
#[inline] fn be16(d: &[u8]) -> u32 { ((d[0] as u32) << 8) | d[1] as u32 }
#[inline] fn mem(address: u32) -> *mut u8 { wprt::memory::host(address) }
/// A guest-memory slice from a host pointer (callers keep it inside the allocation, as the C++ did).
#[inline] unsafe fn at<'a>(p: *const u8, len: usize) -> &'a [u8] { std::slice::from_raw_parts(p, len) }

impl Parser {
    fn xf_float(&self, index: u32) -> f32 { f32::from_bits(self.xf[index as usize]) }

    fn note_register(&mut self, kind: usize, index: u32, value: u32) {
        if settings().log && index < 0x1100 && !self.seen_register[kind][index as usize] {
            self.seen_register[kind][index as usize] = true;
            log("gx", format!("register {} {:04x} first value {:08x}", if kind == 0 { "bp" } else if kind == 1 { "cp" } else { "xf" }, index, value));
        }
    }

    fn build_layout(&self, vat: usize) -> Layout {
        let mut layout = Layout::default();
        let a = self.vat[0][vat];
        let b = self.vat[1][vat];
        let c = self.vat[2][vat];
        layout.position_matrix = (self.vcd_low & 1) != 0;
        layout.texture_matrices = (self.vcd_low >> 1) & 0xFF;
        layout.position.mode = (self.vcd_low >> 9) & 3;
        layout.position.components = if a & 1 != 0 { 3 } else { 2 };
        layout.position.format = (a >> 1) & 7;
        layout.position.shift = (a >> 4) & 31;
        layout.position.direct_size = layout.position.components * component_size(layout.position.format);
        layout.normal.mode = (self.vcd_low >> 11) & 3;
        layout.normal.components = if a & 0x200 != 0 { 9 } else { 3 };
        layout.normal.format = (a >> 10) & 7;
        layout.normal.direct_size = layout.normal.components * component_size(layout.normal.format);
        layout.color[0].mode = (self.vcd_low >> 13) & 3;
        layout.color[0].components = if (a >> 13) & 1 != 0 { 4 } else { 3 };
        layout.color[0].format = (a >> 14) & 7;
        layout.color[0].direct_size = color_size(layout.color[0].format);
        layout.color[1].mode = (self.vcd_low >> 15) & 3;
        layout.color[1].components = if (a >> 17) & 1 != 0 { 4 } else { 3 };
        layout.color[1].format = (a >> 18) & 7;
        layout.color[1].direct_size = color_size(layout.color[1].format);
        let counts = [(a >> 21) & 1, b & 1, (b >> 9) & 1, (b >> 18) & 1, (b >> 27) & 1, (c >> 5) & 1, (c >> 14) & 1, (c >> 23) & 1];
        let formats = [(a >> 22) & 7, (b >> 1) & 7, (b >> 10) & 7, (b >> 19) & 7, (b >> 28) & 7, (c >> 6) & 7, (c >> 15) & 7, (c >> 24) & 7];
        let shifts = [(a >> 25) & 31, (b >> 4) & 31, (b >> 13) & 31, (b >> 22) & 31, c & 31, (c >> 9) & 31, (c >> 18) & 31, (c >> 27) & 31];
        for i in 0..TEX_COORD_COUNT {
            let e = &mut layout.texture[i];
            e.mode = (self.vcd_high >> (2 * i)) & 3;
            e.components = counts[i] + 1;
            e.format = formats[i];
            e.shift = shifts[i];
            e.direct_size = e.components * component_size(e.format);
        }
        let mut size = if layout.position_matrix { 1 } else { 0 };
        size += layout.texture_matrices.count_ones();
        size += element_size(&layout.position);
        size += element_size(&layout.normal);
        size += element_size(&layout.color[0]);
        size += element_size(&layout.color[1]);
        for e in &layout.texture { size += element_size(e); }
        layout.size = size;
        layout
    }

    fn make_layout(&mut self, vat: usize) -> Layout {
        let key = [self.vat[0][vat], self.vat[1][vat], self.vat[2][vat], self.vcd_low, self.vcd_high];
        if !self.layouts[vat].valid || self.layouts[vat].key != key {
            let layout = self.build_layout(vat);
            let entry = &mut self.layouts[vat];
            entry.key = key;
            entry.layout = layout;
            entry.valid = true;
        }
        self.layouts[vat].layout
    }

    fn begin_vertex_arrays(&mut self) {
        for i in 0..ARRAY_COUNT {
            let address = RAM_BASE | (self.array_base[i] & ARRAY_ADDRESS_MASK);
            self.arrays[i] = ArrayState { address, stride: self.array_stride[i] as usize, base: mem(address) as usize };
        }
        for i in 0..TEX_COORD_COUNT {
            self.default_texture_matrix[i] = if i < 4 { (self.xf[0x1018] >> (6 + 6 * i)) & 0x3F } else { (self.xf[0x1019] >> (6 * (i - 4))) & 0x3F };
        }
    }

    fn array_element_live(&self, array: usize, index: u32) -> *const u8 {
        static ZEROS: [u8; ARRAY_READ_MARGIN] = [0; ARRAY_READ_MARGIN];
        let address = RAM_BASE | (self.array_base[array] & ARRAY_ADDRESS_MASK);
        let offset = index as usize * self.array_stride[array] as usize;
        // C++: render::guest_range_valid(address + offset, kArrayReadMargin) with uint32_t address + size_t offset (64-bit sum, then
        // narrowed to the uint32_t parameter).
        if !crate::common::guest_range_valid((address as usize).wrapping_add(offset) as u32, ARRAY_READ_MARGIN) { return ZEROS.as_ptr(); }
        unsafe { mem(address).add(offset) }
    }

    fn array_element(&self, array: usize, index: u32) -> *const u8 {
        static ZEROS: [u8; ARRAY_READ_MARGIN] = [0; ARRAY_READ_MARGIN];
        let state = &self.arrays[array];
        let offset = index as usize * state.stride;
        if ((state.address as usize).wrapping_add(offset) & wprt::memory::ADDRESS_MASK as usize) + ARRAY_READ_MARGIN > wprt::memory::PHYSICAL_SIZE as usize {
            return ZEROS.as_ptr();
        }
        (state.base + offset) as *const u8
    }

    fn fetch(&self, element: &Element, array: usize, stream: &mut *const u8) -> *const u8 {
        unsafe {
            if element.mode == DIRECT {
                let p = *stream;
                *stream = stream.add(element.direct_size as usize);
                return p;
            }
            let index;
            if element.mode == INDEX8 {
                index = **stream as u32;
                *stream = stream.add(1);
            } else {
                index = be16(at(*stream, 2));
                *stream = stream.add(2);
            }
            self.array_element(array, index)
        }
    }

    fn decode_vertex(&self, layout: &Layout, stream: &mut *const u8) -> Vertex {
        let mut v = Vertex::default();
        unsafe {
            v.position_matrix = self.xf[0x1018] & 0x3F;
            if layout.position_matrix {
                v.position_matrix = (**stream & 0x3F) as u32;
                *stream = stream.add(1);
            }
            for i in 0..TEX_COORD_COUNT {
                v.texture_matrix[i] = self.default_texture_matrix[i];
                if layout.texture_matrices & (1 << i) != 0 {
                    v.texture_matrix[i] = (**stream & 0x3F) as u32;
                    *stream = stream.add(1);
                }
            }
            if layout.position.mode != NONE {
                let p = self.fetch(&layout.position, 0, stream);
                let step = component_size(layout.position.format) as usize;
                for i in 0..layout.position.components as usize {
                    v.position[i] = read_number(at(p.add(i * step), step), layout.position.format, layout.position.shift);
                }
            }
            if layout.normal.mode != NONE {
                let p = self.fetch(&layout.normal, 1, stream);
                let step = component_size(layout.normal.format) as usize;
                v.normal[0] = read_normal_component(at(p, step), layout.normal.format);
                v.normal[1] = read_normal_component(at(p.add(step), step), layout.normal.format);
                v.normal[2] = read_normal_component(at(p.add(2 * step), step), layout.normal.format);
            }
            for k in 0..2 {
                if layout.color[k].mode != NONE {
                    let p = self.fetch(&layout.color[k], 2 + k, stream);
                    read_color(at(p, color_size(layout.color[k].format) as usize), layout.color[k].format, &mut v.color[k]);
                    v.has_color[k] = true;
                }
            }
            for k in 0..TEX_COORD_COUNT {
                let e = layout.texture[k];
                if e.mode != NONE {
                    let p = self.fetch(&e, 4 + k, stream);
                    let step = component_size(e.format) as usize;
                    let mut i = 0;
                    while i < e.components as usize && i < 2 {
                        v.texture[k][i] = read_number(at(p.add(i * step), step), e.format, e.shift);
                        i += 1;
                    }
                }
            }
        }
        v
    }

    fn rasterize_colors(&self, vertex: &Vertex, eye: &[f32; 3], out: &mut [[f32; 4]; 2]) {
        let mut vertex_color = [[0u8; 4]; 2];
        for k in 0..2 {
            let material = self.xf[0x100C + k];
            for i in 0..4 {
                vertex_color[k][i] = if vertex.has_color[k] { color_byte(vertex.color[k][i]) } else { (material >> (24 - 8 * i)) as u8 };
            }
        }
        let mut normal = [0.0f32, 0.0, 1.0];
        if (self.xf[0x100E] | self.xf[0x100F] | self.xf[0x1010] | self.xf[0x1011]) & 2 != 0 {
            lighting::transform_normal(&self.xf, vertex.position_matrix, &vertex.normal, &mut normal);
        }
        let mut result = [[0u8; 4]; 2];
        lighting::light_channels(eye, &normal, &vertex_color, &mut result);
        // Claude OPT G6: exact table of (u8 as f32 / 255.0f32).
        let t = unit_bytes();
        for k in 0..2 { for i in 0..4 { out[k][i] = t[result[k][i] as usize]; } }
    }

    fn transform_position(&self, vertex: &Vertex, eye: &mut [f32; 3]) {
        let base = vertex.position_matrix * 4;
        let (x, y, z) = (vertex.position[0], vertex.position[1], vertex.position[2]);
        for row in 0..3 {
            eye[row as usize] = self.xf_float(base + row * 4) * x + self.xf_float(base + row * 4 + 1) * y + self.xf_float(base + row * 4 + 2) * z
                + self.xf_float(base + row * 4 + 3);
        }
    }

    fn begin_texture_coordinates(&mut self) {
        self.texgen.count = std::cmp::min(self.bp[0x00] & 15, TEX_COORD_COUNT as u32);
        self.texgen.dual = (self.xf[0x1012] & 1) != 0;
        for i in 0..self.texgen.count as usize {
            let info = self.xf[0x1040 + i];
            self.texgen.unit[i] = TexGenUnit { source: (info >> 7) & 31, projected: (info >> 1) & 1 != 0, abc1: (info >> 2) & 1 != 0, post: self.xf[0x1050 + i] };
        }
    }

    fn generate_texture_coordinates(&self, vertex: &Vertex, out: &mut [[f32; 3]; 8]) {
        let dual = self.texgen.dual;
        for i in 0..self.texgen.count as usize {
            let unit = self.texgen.unit[i];
            let source = unit.source;
            let mut input = [0.0f32, 0.0, 1.0];
            if source == 0 {
                input = vertex.position;
            } else if source == 1 {
                input = vertex.normal;
            } else if (5..13).contains(&source) {
                input[0] = vertex.texture[(source - 5) as usize][0];
                input[1] = vertex.texture[(source - 5) as usize][1];
            }
            if !unit.abc1 { input[2] = 1.0; }
            let base = vertex.texture_matrix[i] * 4;
            let mut coord = [0.0f32; 3];
            for row in 0..3u32 {
                let r = base + row * 4;
                coord[row as usize] = self.xf_float(r) * input[0] + self.xf_float(r + 1) * input[1] + self.xf_float(r + 2) * input[2] + self.xf_float(r + 3);
            }
            if !unit.projected { coord[2] = 1.0; }
            if dual {
                let post = unit.post;
                if (post >> 8) & 1 != 0 {
                    let length = (coord[0] * coord[0] + coord[1] * coord[1] + coord[2] * coord[2]).sqrt();
                    if length > 0.0 {
                        coord[0] /= length;
                        coord[1] /= length;
                        coord[2] /= length;
                    }
                }
                let post_base = 0x500 + (post & 0x3F) * 4;
                let mut result = [0.0f32; 3];
                for row in 0..3u32 {
                    let r = post_base + row * 4;
                    result[row as usize] = self.xf_float(r) * coord[0] + self.xf_float(r + 1) * coord[1] + self.xf_float(r + 2) * coord[2] + self.xf_float(r + 3);
                }
                coord = result;
            }
            if coord[2] == 0.0 {
                coord[0] = clampf(coord[0] / 2.0, -1.0, 1.0);
                coord[1] = clampf(coord[1] / 2.0, -1.0, 1.0);
                coord[2] = 1.0;
            }
            out[i] = coord;
        }
    }

    fn begin_projection(&mut self) {
        let x = |i: u32| self.xf_float(i);
        let p = ProjectionState {
            a: x(0x1020), b: x(0x1021), c: x(0x1022), d: x(0x1023), e: x(0x1024), f: x(0x1025),
            orthographic: self.xf[0x1026] != 0,
            x_scale: 2.0f32 * x(0x101A) / EFB_WIDTH,
            x_offset: 2.0f32 * (x(0x101D) - VIEWPORT_OFFSET) / EFB_WIDTH - 1.0f32,
            y_scale: -2.0f32 * x(0x101B) / EFB_HEIGHT,
            y_offset: 1.0f32 - 2.0f32 * (x(0x101E) - VIEWPORT_OFFSET) / EFB_HEIGHT,
            z_scale: x(0x101C),
            z_offset: x(0x101F),
        };
        self.projection = p;
    }

    fn project(&self, eye: &[f32; 3], screen: &mut [f32; 4]) -> bool {
        let p = &self.projection;
        let (x, y, z, w);
        if p.orthographic {
            x = p.a * eye[0] + p.b;
            y = p.c * eye[1] + p.d;
            z = p.e * eye[2] + p.f;
            w = 1.0f32;
        } else {
            x = p.a * eye[0] + p.b * eye[2];
            y = p.c * eye[1] + p.d * eye[2];
            z = p.e * eye[2] + p.f;
            w = -eye[2];
        }
        screen[0] = x * p.x_scale + w * p.x_offset;
        screen[1] = y * p.y_scale + w * p.y_offset;
        screen[2] = (z * p.z_scale + w * p.z_offset) / DEPTH_RANGE;
        screen[3] = w;
        screen[0].is_finite() && screen[1].is_finite() && screen[2].is_finite() && w.is_finite()
    }

    fn prepare(&self, vertex: &Vertex) -> Prepared {
        let mut prepared = Prepared::default();
        let mut eye = [0.0f32; 3];
        self.transform_position(vertex, &mut eye);
        let mut screen = [0.0f32, 0.0, 0.0, 1.0];
        prepared.valid = self.project(&eye, &mut screen);
        prepared.vertex.x = screen[0];
        prepared.vertex.y = screen[1];
        prepared.vertex.z = screen[2];
        prepared.vertex.w = screen[3];
        self.rasterize_colors(vertex, &eye, &mut prepared.vertex.color);
        self.generate_texture_coordinates(vertex, &mut prepared.vertex.uv);
        prepared
    }

    fn culled(&self, a: &ScreenVertex, b: &ScreenVertex, c: &ScreenVertex) -> bool {
        let mode = (self.bp[0x00] >> 14) & 3;
        if mode == 0 { return false; }
        if mode == 3 { return true; }
        let area = a.x * (b.y * c.w - c.y * b.w) - a.y * (b.x * c.w - c.x * b.w) + a.w * (b.x * c.y - c.x * b.y);
        let front = area < 0.0;
        if mode == 1 { !front } else { front }
    }

    fn count_indirect(&mut self) {
        let stages = ((self.bp[0x00] >> 10) & 15) + 1;
        let mut indirect = false;
        let mut coordinates = false;
        for i in 0..stages as usize {
            let stage = self.bp[0x10 + i];
            indirect |= (stage & (3 << 7)) != 0 || (stage & (3 << 9)) != 0;
            coordinates |= (stage & ((63 << 13) | (1 << 20))) != 0;
        }
        self.indirect_draws += indirect as u32;
        self.coordinate_draws += (coordinates && !indirect) as u32;
    }
}

// Claude OPT G3: per-draw cache of prepare() results keyed on the full decoded vertex (bit-exact, full compare).
const PREPARE_CACHE: usize = 64;
fn vertex_key(v: &Vertex) -> [u32; 40] {
    let mut k = [0u32; 40];
    let mut n = 0;
    for x in v.position.iter().chain(v.normal.iter()) { k[n] = x.to_bits(); n += 1; }
    for c in &v.color { for x in c { k[n] = x.to_bits(); n += 1; } }
    for t in &v.texture { for x in t { k[n] = x.to_bits(); n += 1; } }
    k[n] = v.position_matrix; n += 1;
    for m in &v.texture_matrix { k[n] = *m; n += 1; }
    k[n] = v.has_color[0] as u32 | ((v.has_color[1] as u32) << 1);
    k
}
// Claude OPT G7: slot choice only (hits are still confirmed by the full 40-word compare); 4 independent lanes + one final mix.
fn key_slot(k: &[u32; 40]) -> usize {
    let mut lanes = [0u64; 4];
    for (i, &w) in k.iter().enumerate() { let l = &mut lanes[i & 3]; *l = l.rotate_left(7) ^ w as u64; }
    let h = (lanes[0] ^ lanes[1].rotate_left(17) ^ lanes[2].rotate_left(31) ^ lanes[3].rotate_left(47)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (h >> 58) as usize
}
fn unit_bytes() -> &'static [f32; 256] {
    static T: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    T.get_or_init(|| std::array::from_fn(|i| i as f32 / 255.0f32))
}
fn component_size(format: u32) -> u32 { match format { 0 | 1 => 1, 2 | 3 => 2, _ => 4 } }
fn color_size(format: u32) -> u32 { [2, 3, 4, 2, 3, 4, 4, 4][(format & 7) as usize] }
fn element_size(e: &Element) -> u32 { match e.mode { 1 => e.direct_size, 2 => 1, 3 => 2, _ => 0 } }
/// kShiftScale[i] = std::ldexp(1.0f, -i): exact powers of two.
fn shift_scale(i: u32) -> f32 { f32::from_bits((127 - i) << 23) }
fn read_number(p: &[u8], format: u32, shift: u32) -> f32 {
    let value = match format {
        0 => p[0] as f32,
        1 => p[0] as i8 as f32,
        2 => be16(p) as f32,
        3 => be16(p) as u16 as i16 as f32,
        _ => return f32::from_bits(be32(p)),
    };
    value * shift_scale(shift)
}
fn read_normal_component(p: &[u8], format: u32) -> f32 {
    match format {
        0 => p[0] as f32 * shift_scale(7),
        1 => p[0] as i8 as f32 * shift_scale(6),
        2 => be16(p) as f32 * shift_scale(15),
        3 => be16(p) as u16 as i16 as f32 * shift_scale(14),
        _ => f32::from_bits(be32(p)),
    }
}
#[allow(unused_assignments)]
fn read_color(p: &[u8], format: u32, out: &mut [f32; 4]) {
    let (mut r, mut g, mut b, mut a) = (1.0f32, 1.0f32, 1.0f32, 1.0f32);
    match format {
        0 => { let v = be16(p); r = ((v >> 11) & 31) as f32 / 31.0; g = ((v >> 5) & 63) as f32 / 63.0; b = (v & 31) as f32 / 31.0; }
        1 | 2 => { r = p[0] as f32 / 255.0; g = p[1] as f32 / 255.0; b = p[2] as f32 / 255.0; }
        3 => { let v = be16(p); r = ((v >> 12) & 15) as f32 / 15.0; g = ((v >> 8) & 15) as f32 / 15.0; b = ((v >> 4) & 15) as f32 / 15.0; a = (v & 15) as f32 / 15.0; }
        4 => {
            let v = ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32;
            r = ((v >> 18) & 63) as f32 / 63.0; g = ((v >> 12) & 63) as f32 / 63.0; b = ((v >> 6) & 63) as f32 / 63.0; a = (v & 63) as f32 / 63.0;
        }
        _ => { r = p[0] as f32 / 255.0; g = p[1] as f32 / 255.0; b = p[2] as f32 / 255.0; a = p[3] as f32 / 255.0; }
    }
    *out = [r, g, b, a];
}
/// std::clamp(v, lo, hi): returns lo if v < lo, hi if hi < v, else v (NaN passes through).
#[inline] fn clampf(v: f32, lo: f32, hi: f32) -> f32 { if v < lo { lo } else if hi < v { hi } else { v } }
/// static_cast<uint8_t>(std::clamp(value, 0.0f, 1.0f) * 255.0f + 0.5f): float->int truncation, then to 8 bits.
fn color_byte(value: f32) -> u8 { let f = clampf(value, 0.0, 1.0) * 255.0f32 + 0.5f32; (f as i32) as u8 }

fn to_ports(v: &ScreenVertex) -> gx_state::ScreenVertex {
    gx_state::ScreenVertex { x: v.x, y: v.y, z: v.z, w: v.w, color: v.color, uv: v.uv }
}
fn offset_texture_coordinates(v: &mut ScreenVertex, coordinates: u32, s: f32, t: f32) {
    let mut p = to_ports(v);
    gx_state::offset_texture_coordinates(&mut p, coordinates, s, t);
    v.uv = p.uv;
}
fn shifted(vertex: &ScreenVertex, dx: f32, dy: f32) -> ScreenVertex {
    let mut r = *vertex;
    r.x += dx * vertex.w;
    r.y += dy * vertex.w;
    r
}

impl Parser {
    fn emit_triangle(&self, out: &mut Vec<ScreenVertex>, a: &Prepared, b: &Prepared, c: &Prepared) {
        if !a.valid || !b.valid || !c.valid || self.culled(&a.vertex, &b.vertex, &c.vertex) { return; }
        out.push(a.vertex);
        out.push(b.vertex);
        out.push(c.vertex);
    }

    fn emit_line(&self, out: &mut Vec<ScreenVertex>, a: &Prepared, b: &Prepared) {
        if !a.valid || !b.valid || a.vertex.w <= 0.0 || b.vertex.w <= 0.0 { return; }
        let width = (self.bp[0x22] & 0xFF) as f32 / 6.0f32;
        let sx = b.vertex.x / b.vertex.w - a.vertex.x / a.vertex.w;
        let sy = b.vertex.y / b.vertex.w - a.vertex.y / a.vertex.w;
        let dx = sx.abs() * EFB_WIDTH;
        let dy = sy.abs() * EFB_HEIGHT;
        let (mut ox, mut oy) = (0.0f32, 0.0f32);
        if dy > dx { ox = width / EFB_WIDTH; } else { oy = width / EFB_HEIGHT; }
        let mut a0 = shifted(&a.vertex, -ox, -oy);
        let mut a1 = shifted(&a.vertex, ox, oy);
        let mut b0 = shifted(&b.vertex, -ox, -oy);
        let mut b1 = shifted(&b.vertex, ox, oy);
        let offsets = gx_state::line_point_offsets(&self.bp);
        if offsets.line_coordinates != 0 {
            let negative = gx_state::line_offset_negative_side(sx, sy, dy > dx);
            offset_texture_coordinates(if negative { &mut a0 } else { &mut a1 }, offsets.line_coordinates, offsets.line, 0.0);
            offset_texture_coordinates(if negative { &mut b0 } else { &mut b1 }, offsets.line_coordinates, offsets.line, 0.0);
        }
        out.extend_from_slice(&[a0, a1, b1]);
        out.extend_from_slice(&[a0, b1, b0]);
    }

    fn emit_point(&self, out: &mut Vec<ScreenVertex>, a: &Prepared) {
        if !a.valid || a.vertex.w <= 0.0 { return; }
        let size = ((self.bp[0x22] >> 8) & 0xFF) as f32 / 6.0f32;
        let ox = size / EFB_WIDTH;
        let oy = size / EFB_HEIGHT;
        let mut v0 = shifted(&a.vertex, -ox, -oy);
        let mut v1 = shifted(&a.vertex, ox, -oy);
        let mut v2 = shifted(&a.vertex, ox, oy);
        let v3 = shifted(&a.vertex, -ox, oy);
        let offsets = gx_state::line_point_offsets(&self.bp);
        if offsets.point_coordinates != 0 {
            offset_texture_coordinates(&mut v0, offsets.point_coordinates, 0.0, offsets.point);
            offset_texture_coordinates(&mut v1, offsets.point_coordinates, offsets.point, offsets.point);
            offset_texture_coordinates(&mut v2, offsets.point_coordinates, offsets.point, 0.0);
        }
        out.extend_from_slice(&[v0, v1, v2]);
        out.extend_from_slice(&[v0, v2, v3]);
    }

    fn capture_registers(&mut self) {
        use std::io::Write;
        let Some(file) = self.capture.as_mut() else { return };
        let mut written = 0;
        let mut text = String::new();
        let mut field = |text: &mut String, s: String| {
            text.push_str(&s);
            written += 1;
            if written % 8 == 0 { text.push('\n'); }
        };
        for i in 0..256 {
            if self.capture_first || self.bp[i] != self.captured_bp[i] {
                field(&mut text, format!(" bp{:02x}={:06x}", i, self.bp[i]));
                self.captured_bp[i] = self.bp[i];
            }
        }
        for i in 0..8 {
            if self.capture_first || self.konst[i] != self.captured_konst[i] {
                field(&mut text, format!(" konst{}={:06x}", i, self.konst[i]));
                self.captured_konst[i] = self.konst[i];
            }
        }
        for i in 0..XF_SIZE {
            if (self.capture_first && self.xf[i] != 0) || self.xf[i] != self.captured_xf[i] {
                field(&mut text, format!(" xf{:04x}={:08x}", i, self.xf[i]));
                self.captured_xf[i] = self.xf[i];
            }
        }
        self.capture_first = false;
        text.push('\n');
        let _ = file.write_all(crlf(&text).as_bytes());
    }

    fn capture_draw(&mut self, command: u8, count: u32, triangles: usize, layout: &Layout, raw: &[Vertex; 3]) {
        use std::io::Write;
        let vat = (command & 7) as usize;
        let mut text = format!("draw {} primitive={} vat={} vertices={} triangles={} vcd={:08x} {:08x} vat={:08x} {:08x} {:08x} size={}\n", self.capture_draws,
            (command >> 3) & 7, vat, count, triangles, self.vcd_low, self.vcd_high, self.vat[0][vat], self.vat[1][vat], self.vat[2][vat], layout.size);
        self.capture_draws += 1;
        text.push_str(" arrays");
        for i in 0..ARRAY_COUNT {
            if self.array_base[i] != 0 { text.push_str(&format!(" {}={:08x}/{}", i, self.array_base[i], self.array_stride[i])); }
        }
        text.push('\n');
        if let Some(f) = self.capture.as_mut() { let _ = f.write_all(crlf(&text).as_bytes()); }
        self.capture_registers();
        let texgens = self.bp[0x00] & 15;
        let mut text = String::new();
        let mut i = 0usize;
        while (i as u32) < count && i < 3 {
            let v = &raw[i];
            let s = &self.vertices[i].vertex;
            text.push_str(&format!("  v{} pos=({} {} {}) normal=({} {} {}) pnmtx={} screen=({} {} {} {}){}", i, fmt_g(v.position[0]), fmt_g(v.position[1]),
                fmt_g(v.position[2]), fmt_g(v.normal[0]), fmt_g(v.normal[1]), fmt_g(v.normal[2]), v.position_matrix, fmt_g(s.x), fmt_g(s.y), fmt_g(s.z), fmt_g(s.w),
                if self.vertices[i].valid { "" } else { " invalid" }));
            for t in 0..TEX_COORD_COUNT {
                if layout.texture[t].mode != 0 { text.push_str(&format!(" tex{}=({} {})", t, fmt_g(v.texture[t][0]), fmt_g(v.texture[t][1]))); }
            }
            let mut t = 0usize;
            while (t as u32) < texgens && t < TEX_COORD_COUNT {
                text.push_str(&format!(" uv{}=({} {} {}) mtx{}={}", t, fmt_g(s.uv[t][0]), fmt_g(s.uv[t][1]), fmt_g(s.uv[t][2]), t, v.texture_matrix[t]));
                t += 1;
            }
            text.push('\n');
            i += 1;
        }
        if let Some(f) = self.capture.as_mut() { let _ = f.write_all(crlf(&text).as_bytes()); }
    }

    fn draw_primitive(&mut self, command: u8, data: *const u8, count: u32) {
        if !settings().render_enabled { return; }
        self.count_indirect();
        self.begin_vertex_arrays();
        lighting::begin(&self.xf);
        self.begin_texture_coordinates();
        self.begin_projection();
        let layout = self.make_layout((command & 7) as usize);
        let mut vertices = std::mem::take(&mut self.vertices);
        vertices.clear();
        vertices.reserve(count as usize);
        let mut stream = data;
        let mut raw = [Vertex::default(); 3];
        let use_cache = count >= 16 && (layout.position.mode >= INDEX8 || layout.normal.mode >= INDEX8
            || layout.color.iter().any(|e| e.mode >= INDEX8) || layout.texture.iter().any(|e| e.mode >= INDEX8));
        let mut cache_tag = [usize::MAX; PREPARE_CACHE];
        let mut cache_key = [[0u32; 40]; PREPARE_CACHE];
        for i in 0..count as usize {
            let vertex = self.decode_vertex(&layout, &mut stream);
            if i < 3 { raw[i] = vertex; }
            if use_cache {
                let key = vertex_key(&vertex);
                let slot = key_slot(&key);
                if cache_tag[slot] != usize::MAX && cache_key[slot] == key {
                    let hit = vertices[cache_tag[slot]];
                    vertices.push(hit);
                    continue;
                }
                cache_tag[slot] = vertices.len();
                cache_key[slot] = key;
            }
            vertices.push(self.prepare(&vertex));
        }
        let mut triangles = std::mem::take(&mut self.triangles);
        triangles.clear();
        let n = count as usize;
        match (command >> 3) & 7 {
            0 | 1 => { let mut i = 0; while i + 3 < n { self.emit_triangle(&mut triangles, &vertices[i], &vertices[i + 1], &vertices[i + 2]);
                                                        self.emit_triangle(&mut triangles, &vertices[i], &vertices[i + 2], &vertices[i + 3]); i += 4; } }
            2 => { let mut i = 0; while i + 2 < n { self.emit_triangle(&mut triangles, &vertices[i], &vertices[i + 1], &vertices[i + 2]); i += 3; } }
            3 => {
                let mut i = 0;
                while i + 2 < n {
                    if i & 1 != 0 { self.emit_triangle(&mut triangles, &vertices[i + 1], &vertices[i], &vertices[i + 2]); }
                    else { self.emit_triangle(&mut triangles, &vertices[i], &vertices[i + 1], &vertices[i + 2]); }
                    i += 1;
                }
            }
            4 => { let mut i = 1; while i + 1 < n { self.emit_triangle(&mut triangles, &vertices[0], &vertices[i], &vertices[i + 1]); i += 1; } }
            5 => { let mut i = 0; while i + 1 < n { self.emit_line(&mut triangles, &vertices[i], &vertices[i + 1]); i += 2; } }
            6 => { let mut i = 0; while i + 1 < n { self.emit_line(&mut triangles, &vertices[i], &vertices[i + 1]); i += 1; } }
            _ => { for i in 0..n { self.emit_point(&mut triangles, &vertices[i]); } }
        }
        if settings().log_draws && self.copy_total >= settings().log_from && self.logged_prepared < 4000 {
            self.logged_prepared += 1;
            let image0 = self.bp[0x88];
            // C++ reads vertices[0] unconditionally (count 0 is undefined behaviour there); a default vertex is used here.
            let first = vertices.first().copied().unwrap_or_default();
            let v0 = first.vertex;
            let bp = &self.bp;
            eprintln!("P copy={} cmd={:02x} n={} tri={} stg={} tg={} cull={} blend={:06x} z={:06x} map0={}/{}x{} v0=({:.2} {:.2} {:.2}){} tev0={:06x}/{:06x}",
                self.copy_total, command, count, triangles.len() / 3, ((bp[0x00] >> 10) & 15) + 1, bp[0x00] & 15, (bp[0x00] >> 14) & 3, bp[0x41], bp[0x40],
                (image0 >> 20) & 15, (image0 & 0x3FF) + 1, ((image0 >> 10) & 0x3FF) + 1, v0.x, v0.y, v0.z, if first.valid { "" } else { " INVALID" }, bp[0xC0], bp[0xC1]);
            eprintln!("  regs={:06x} {:06x} {:06x} {:06x} {:06x} {:06x} {:06x} {:06x} konst={:06x} {:06x} {:06x} {:06x} order={:06x} ksel={:06x} c0=({:.2} {:.2} {:.2} {:.2}) mat={:08x} ctl={:08x}",
                bp[0xE0], bp[0xE1], bp[0xE2], bp[0xE3], bp[0xE4], bp[0xE5], bp[0xE6], bp[0xE7], self.konst[0], self.konst[1], self.konst[2], self.konst[3], bp[0x28],
                bp[0xF6], v0.color[0][0], v0.color[0][1], v0.color[0][2], v0.color[0][3], self.xf[0x100C], self.xf[0x100E]);
        }
        self.vertices = vertices;
        if self.capture.is_some() { self.capture_draw(command, count, triangles.len() / 3, &layout, &raw); }
        backend().draw(&triangles);
        self.triangles = triangles;
    }

    fn report_copy(&mut self) {
        if !settings().log || self.logged_copies >= MAX_LOGGED_COPIES { return; }
        self.logged_copies += 1;
        let s = self.stats;
        eprintln!("GX copy: draws={} vertices={} bp={} cp={} xf={} lists={} unknown={}", s.draws, s.vertices, s.bp_writes, s.cp_writes, s.xf_words, s.lists, s.unknown);
        self.stats = Statistics::default();
    }

    fn capture_frame_boundary(&mut self) {
        if self.capture.is_some() {
            use std::io::Write;
            let line = format!("end of frame, {} draws\n", self.capture_draws);
            if let Some(f) = self.capture.as_mut() { let _ = f.write_all(crlf(&line).as_bytes()); }
            self.capture = None;
            backend().dump_copies(None);
            let image = format!("{}.png", self.capture_name);
            let c = CString::new(image).unwrap();
            (host().save_next_frame)(c.as_ptr());
            eprintln!("GX capture saved to {}.txt and {}.png", self.capture_name, self.capture_name);
            log("gx", format!("capture saved to {}.txt and {}.png", self.capture_name, self.capture_name));
            return;
        }
        // static const char* scheduled = getenv("WP_CAPTURE_AT"); static start = now(); static next = scheduled
        let schedule = self.capture_schedule.get_or_insert_with(|| (std::time::Instant::now(), std::env::var("WP_CAPTURE_AT").unwrap_or_default(), 0));
        let mut due = false;
        if std::env::var_os("WP_CAPTURE_AT").is_some() {
            let (start, text, next) = (schedule.0, schedule.1.clone(), schedule.2);
            let rest = &text[next..];
            if !rest.is_empty() && start.elapsed().as_secs_f64() >= atof(rest) {
                due = true;
                schedule.2 = match rest.find(',') { Some(c) => next + c + 1, None => text.len() };
            }
        }
        if !CAPTURE_REQUESTED.swap(false, Ordering::SeqCst) && !due { return; }
        let _ = std::fs::create_dir_all("captures");
        let mut path;
        loop {
            self.capture_index += 1;
            self.capture_name = format!("captures/gx_capture_{:03}", self.capture_index);
            path = format!("{}.txt", self.capture_name);
            if std::fs::File::open(&path).is_err() { break; }
        }
        self.capture = std::fs::File::create(&path).ok();
        backend().dump_copies(Some(&self.capture_name));
        self.capture_draws = 0;
        self.capture_first = true;
    }

    fn copy_filter(&self, control: u32) -> CopyFilter {
        const GAMMA: [f32; 4] = [1.0, 1.7, 2.2, 2.2];
        let bits = self.bp[0x53] as u64 | ((self.bp[0x54] as u64) << 24);
        let mut weights = [0u32; 7];
        for (i, w) in weights.iter_mut().enumerate() { *w = ((bits >> (6 * i)) & 0x3F) as u32; }
        let mut filter = CopyFilter { coefficients: [weights[0] + weights[1], weights[2] + weights[3] + weights[4], weights[5] + weights[6]], gamma: 1.0,
                                      clamp_top: false, clamp_bottom: false };
        if !(host().copy_filter_enabled)() {
            filter.coefficients[1] += filter.coefficients[0] + filter.coefficients[2];
            filter.coefficients[0] = 0;
            filter.coefficients[2] = 0;
        }
        filter.gamma = GAMMA[((control >> 7) & 3) as usize];
        filter.clamp_top = (control & 1) != 0;
        filter.clamp_bottom = (control & 2) != 0;
        filter
    }

    fn execute_copy(&mut self, value: u32) {
        self.copy_total += 1;
        self.report_copy();
        if !settings().render_enabled { return; }
        if self.capture.is_some() && value & COPY_TO_FRAMEBUFFER == 0 {
            use std::io::Write;
            let line = format!("copy to texture at {:06x}, source {:06x} size {:06x}, control {:06x}, pixel format {:06x}\n", self.bp[0x4B] << 5, self.bp[0x49],
                               self.bp[0x4A], value, self.bp[0x43]);
            if let Some(f) = self.capture.as_mut() { let _ = f.write_all(crlf(&line).as_bytes()); }
        }
        let source = self.bp[0x49];
        let size = self.bp[0x4A];
        let x = (source & 0x3FF) as i32;
        let y = ((source >> 10) & 0x3FF) as i32;
        let width = (size & 0x3FF) as i32 + 1;
        let height = ((size >> 10) & 0x3FF) as i32 + 1;
        if value & COPY_TO_FRAMEBUFFER != 0 {
            self.capture_frame_boundary();
            self.frames += 1;
            FRAMES_DRAWN.fetch_add(1, Ordering::SeqCst);
            let now = std::time::Instant::now();
            let previous = *self.previous_frame.get_or_insert(now);
            let frame_ms = now.duration_since(previous).as_secs_f64() * 1000.0;
            self.previous_frame = Some(now);
            if frame_ms > SLOW_FRAME_MS { log("frame", format!("slow frame: {:.0} ms", frame_ms)); }
            let seconds = now.duration_since(self.fps_start).as_secs_f64();
            if seconds >= 1.0 {
                let (batches, vertices, batch_seconds) = backend().take_statistics();
                let (write_backs, write_back_seconds) = backend().take_copy_statistics();
                if settings().log_fps {
                    eprintln!("fps {:.1} batches {} vertices {} cpu-side draw time {:.0} ms", self.frames as f64 / seconds, batches, vertices, batch_seconds * 1000.0);
                }
                log("fps", format!("{:.1} fps, {} draw batches, {} vertices, {:.0} ms drawing, {} draws with indirect texturing, {} with wrapped coordinates, {} presents skipped",
                    self.frames as f64 / seconds, batches, vertices, batch_seconds * 1000.0, self.indirect_draws, self.coordinate_draws, SKIPPED_PRESENTS.swap(0, Ordering::SeqCst)));
                let cpu = if CPU_CLOCK_KNOWN.load(Ordering::SeqCst) { (host().thread_seconds)(CPU_CLOCK.load(Ordering::SeqCst)) } else { 0.0 };
                let gpu_clock = *self.gpu_clock.get_or_insert_with(|| (host().current_thread_clock)());
                let gpu = (host().thread_seconds)(gpu_clock);
                log("copies", format!("{} EFB copies written back to RAM, {:.1} ms waiting for the GPU", write_backs, write_back_seconds * 1000.0));
                log("memory", format!("{} MB private memory, {} KB of graphics commands queued", (host().private_memory_mb)(), QUEUED_BYTES.load(Ordering::SeqCst) >> 10));
                let idle = (host().idle_share)();
                if idle >= 0.0 {
                    log("threads", format!("CPU thread busy {:.0}% (guest idle {:.0}%), GPU thread busy {:.0}%", (cpu - self.previous_cpu) * 100.0 / seconds, idle * 100.0,
                        (gpu - self.previous_gpu) * 100.0 / seconds));
                } else {
                    log("threads", format!("CPU thread busy {:.0}%, GPU thread busy {:.0}%", (cpu - self.previous_cpu) * 100.0 / seconds, (gpu - self.previous_gpu) * 100.0 / seconds));
                }
                self.previous_cpu = cpu;
                self.previous_gpu = gpu;
                self.indirect_draws = 0;
                self.coordinate_draws = 0;
                (host().update_statistics)(self.frames as f64 / seconds);
                self.frames = 0;
                self.fps_start = now;
            }
            backend().copy_to_framebuffer(x, y, width, height, (self.bp[0x43] & 7) == PIXEL_FORMAT_Z24, &self.copy_filter(value));
        } else {
            let address = RAM_BASE | ((self.bp[0x4B] & 0xFFFFFF) << 5);
            let coded = (value >> 3) & 15;
            let format = coded / 2 + (coded & 1) * 8;
            backend().copy_to_texture(address, self.bp[0x4D] & 0x3FF, x, y, width, height, (value & (1 << 9)) != 0, format, (value & (1 << 15)) != 0,
                (self.bp[0x43] & 7) == PIXEL_FORMAT_Z24, (self.bp[0x43] & 7) == PIXEL_FORMAT_RGBA6_Z24, &self.copy_filter(value));
        }
        if value & COPY_CLEAR != 0 {
            let source = self.bp[0x49];
            let size = self.bp[0x4A];
            backend().clear((source & 0x3FF) as i32, ((source >> 10) & 0x3FF) as i32, (size & 0x3FF) as i32 + 1, ((size >> 10) & 0x3FF) as i32 + 1);
        }
    }

    fn load_bp(&mut self, word: u32) {
        let reg = word >> 24;
        let value = word & 0xFFFFFF;
        self.stats.bp_writes += 1;
        self.note_register(0, reg, value);
        if reg == BP_MASK {
            self.bp_mask = value;
            return;
        }
        let mask = self.bp_mask;
        self.bp_mask = 0xFFFFFF;
        if (0xE0..=0xE7).contains(&reg) && (value & (1 << 23)) != 0 {
            let k = (reg - 0xE0) as usize;
            self.konst[k] = (self.konst[k] & !mask) | (value & mask);
            return;
        }
        let r = reg as usize;
        self.bp[r] = (self.bp[r] & !mask) | (value & mask);
        if reg == BP_DRAW_DONE && (value & 2) != 0 {
            backend().finish_copies();
            FINISH_PENDING.store(true, Ordering::SeqCst);
        } else if reg == BP_COPY_EXECUTE {
            self.execute_copy(self.bp[r]);
        } else if reg == BP_INVALIDATE_TEXTURE {
            backend().invalidate_textures();
        } else if reg == BP_LOAD_TLUT {
            let address = RAM_BASE | ((self.bp[0x64] & 0xFFFFFF) << 5);
            backend().load_tlut(address, (self.bp[r] & 0x3FF) << 9, ((self.bp[r] >> 10) & 0x7FF) << 5);
        }
    }

    fn load_cp(&mut self, reg: u8, value: u32) {
        self.stats.cp_writes += 1;
        self.note_register(1, reg as u32, value);
        if reg == 0x50 { self.vcd_low = value; }
        else if reg == 0x60 { self.vcd_high = value; }
        else if (0x70..0x98).contains(&reg) { self.vat[((reg >> 4) - 7) as usize][(reg & 7) as usize] = value; }
        else if (0xA0..0xB0).contains(&reg) { self.array_base[(reg - 0xA0) as usize] = value; }
        else if (0xB0..0xC0).contains(&reg) { self.array_stride[(reg - 0xB0) as usize] = value & 0xFF; }
    }

    fn write_xf(&mut self, address: u32, value: u32) {
        if (address as usize) < XF_SIZE {
            if address >= 0x1000 { self.note_register(2, address, value); }
            self.xf[address as usize] = value;
        }
    }

    fn indexed_xf_load(&mut self, command: u8, word: u32) {
        let array = 12 + ((command >> 3) & 3) as usize;
        let index = word >> 16;
        let count = ((word >> 12) & 15) + 1;
        let address = word & 0xFFF;
        let source = self.array_element_live(array, index);
        for i in 0..count {
            let v = be32(unsafe { at(source.add(4 * i as usize), 4) });
            self.write_xf(address + i, v);
        }
    }

    fn parse_one(&mut self, data: *const u8, size: usize, list: bool) -> usize {
        let d = |n: usize| unsafe { at(data, n) };
        let command = d(1)[0];
        if command == COMMAND_NOP || command == COMMAND_METRICS || command == COMMAND_INVALIDATE { return 1; }
        if command == COMMAND_LOAD_CP {
            if size < 6 { return 0; }
            let b = d(6);
            self.load_cp(b[1], be32(&b[2..]));
            return 6;
        }
        if command == COMMAND_LOAD_XF {
            if size < 5 { return 0; }
            let header = be32(&d(5)[1..]);
            let count = (header >> 16) + 1;
            if size < 5 + 4 * count as usize { return 0; }
            let b = d(5 + 4 * count as usize);
            for i in 0..count { self.write_xf((header & 0xFFFF) + i, be32(&b[5 + 4 * i as usize..])); }
            self.stats.xf_words += count;
            return 5 + 4 * count as usize;
        }
        if (command & 0xE7) == 0x20 {
            if size < 5 { return 0; }
            self.indexed_xf_load(command, be32(&d(5)[1..]));
            self.stats.xf_words += 1;
            return 5;
        }
        if command == COMMAND_CALL_LIST {
            if size < 9 { return 0; }
            let b = d(9);
            let address = be32(&b[1..]);
            let length = be32(&b[5..]);
            self.stats.lists += 1;
            if (address & 0x1FFFFFFF) as u64 + length as u64 > wprt::memory::PHYSICAL_SIZE as u64 {
                eprintln!("GX display list out of range: address={:08x} length={:08x} list={}", address, length, list as i32);
                self.stats.unknown += 1;
                return 9;
            }
            if !list { self.parse(mem(address), length as usize, true); }
            return 9;
        }
        if command == COMMAND_LOAD_BP {
            if size < 5 { return 0; }
            self.load_bp(be32(&d(5)[1..]));
            return 5;
        }
        if command & COMMAND_DRAW_MASK != 0 {
            if size < 3 { return 0; }
            let count = be16(&d(3)[1..]);
            let layout = self.make_layout((command & 7) as usize);
            let total = 3 + count as usize * layout.size as usize;
            if size < total { return 0; }
            if settings().log_draws && self.copy_total >= settings().log_from && self.logged_draws < 60 {
                self.logged_draws += 1;
                eprintln!("GX draw {:02x} count={} size={} vcd={:08x}/{:08x}", command, count, layout.size, self.vcd_low, self.vcd_high);
            }
            self.draw_primitive(command, unsafe { data.add(3) }, count);
            self.stats.draws += 1;
            self.stats.vertices += count;
            return total;
        }
        self.stats.unknown += 1;
        1
    }

    fn parse(&mut self, data: *const u8, size: usize, list: bool) -> usize {
        let mut offset = 0usize;
        while offset < size {
            let used = self.parse_one(unsafe { data.add(offset) }, size - offset, list);
            if used == 0 { break; }
            offset += used;
        }
        offset
    }
}

/// printf("%g"): 6 significant digits, shortest of %e/%f style with trailing zeros removed.
pub fn fmt_g(v: f32) -> String {
    let v = v as f64;
    if v.is_nan() { return if v.is_sign_negative() { "-nan".into() } else { "nan".into() }; }
    if v.is_infinite() { return if v < 0.0 { "-inf".into() } else { "inf".into() }; }
    if v == 0.0 { return if v.is_sign_negative() { "-0".into() } else { "0".into() }; }
    let e = format!("{:.5e}", v);
    let (mantissa, exp) = e.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let strip = |s: &str| -> String { if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() } };
    if exp < -4 || exp >= 6 {
        format!("{}e{}{:02}", strip(mantissa), if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        strip(&format!("{:.*}", (5 - exp) as usize, v))
    }
}
/// C atof prefix parse (digits, sign, '.', exponent); 0.0 when no number.
fn atof(text: &str) -> f64 {
    let t = text.trim_start();
    let mut end = 0;
    let b = t.as_bytes();
    let mut seen_digit = false;
    if end < b.len() && (b[end] == b'+' || b[end] == b'-') { end += 1; }
    while end < b.len() && b[end].is_ascii_digit() { end += 1; seen_digit = true; }
    if end < b.len() && b[end] == b'.' { end += 1; while end < b.len() && b[end].is_ascii_digit() { end += 1; seen_digit = true; } }
    if seen_digit && end < b.len() && (b[end] == b'e' || b[end] == b'E') {
        let mut e = end + 1;
        if e < b.len() && (b[e] == b'+' || b[e] == b'-') { e += 1; }
        let digits = e;
        while e < b.len() && b[e].is_ascii_digit() { e += 1; }
        if e > digits { end = e; }
    }
    if !seen_digit { return 0.0; }
    t[..end].parse().unwrap_or(0.0)
}
/// The C++ capture file is fopen("w"): text mode on Windows writes "\r\n".
fn crlf(text: &str) -> String { if cfg!(windows) { text.replace('\n', "\r\n") } else { text.to_string() } }

// ---------------------------------------------------------------------------------------------------------------
// Public API (gx.h + gx_render.h register accessors) — CPU-thread side.

struct CpuState { fifo: Vec<u8>, gather_pipe: [u8; GATHER_PIPE_SIZE], gather_count: usize }
static CPU: Global<CpuState> = Global::new(CpuState { fifo: Vec::new(), gather_pipe: [0; GATHER_PIPE_SIZE], gather_count: 0 });

pub fn bp_registers() -> *const u32 { parser().bp.as_ptr() }
pub fn konst_registers() -> *const u32 { parser().konst.as_ptr() }
pub fn xf_registers() -> *const u32 { parser().xf.as_ptr() }

fn register32(address: u32) -> u32 { u32::from_be_bytes(unsafe { *(mem(address) as *const [u8; 4]) }) }
fn register16(address: u32) -> u16 { u16::from_be_bytes(unsafe { *(mem(address) as *const [u8; 2]) }) }
fn store16(address: u32, value: u16) { unsafe { *(mem(address) as *mut [u8; 2]) = value.to_be_bytes(); } }
fn store32(address: u32, value: u32) { unsafe { *(mem(address) as *mut [u8; 4]) = value.to_be_bytes(); } }
fn cp_register(low: u32) -> u32 { register16(low) as u32 | ((register16(low + 2) as u32) << 16) }
fn set_cp_register(low: u32, value: u32) { store16(low, value as u16); store16(low + 2, (value >> 16) as u16); }

fn record_display_list(data: &[u8]) -> bool {
    let pi_base = register32(PI_FIFO_BASE) & FIFO_PHYSICAL_MASK;
    let cp_base = (register16(CP_FIFO_BASE_LOW) as u32 | ((register16(CP_FIFO_BASE_HIGH) as u32) << 16)) & FIFO_PHYSICAL_MASK;
    if cp_base == 0 || pi_base == cp_base { return false; }
    let pointer = register32(PI_FIFO_WRITE_POINTER);
    let address = pointer & FIFO_PHYSICAL_MASK;
    for (i, b) in data.iter().enumerate() { unsafe { *mem(RAM_BASE | (address.wrapping_add(i as u32))) = *b; } }
    store32(PI_FIFO_WRITE_POINTER, (pointer & !FIFO_PHYSICAL_MASK) | address.wrapping_add(data.len() as u32));
    true
}

fn gather_pipe_burst(cpu: &CpuState) {
    let pi_write = register32(PI_FIFO_WRITE_POINTER);
    let pi_end = register32(PI_FIFO_END) & FIFO_POINTER_MASK;
    let pi_base = register32(PI_FIFO_BASE) & FIFO_POINTER_MASK;
    let address = pi_write & FIFO_POINTER_MASK;
    if address != 0 && crate::common::guest_range_valid(RAM_BASE | address, GATHER_PIPE_SIZE) {
        unsafe { std::ptr::copy_nonoverlapping(cpu.gather_pipe.as_ptr(), mem(RAM_BASE | address), GATHER_PIPE_SIZE); }
    }
    let next = if address == pi_end { pi_base } else { address + GATHER_PIPE_SIZE as u32 };
    store32(PI_FIFO_WRITE_POINTER, (pi_write & !FIFO_PHYSICAL_MASK) | next);
    let cp_end = cp_register(CP_FIFO_END_LOW) & FIFO_POINTER_MASK;
    let cp_base = cp_register(CP_FIFO_BASE_LOW) & FIFO_POINTER_MASK;
    let mut cp_write = cp_register(CP_WRITE_POINTER_LOW) & FIFO_POINTER_MASK;
    cp_write = if cp_write == cp_end { cp_base } else { cp_write + GATHER_PIPE_SIZE as u32 };
    set_cp_register(CP_WRITE_POINTER_LOW, cp_write);
    set_cp_register(CP_READ_POINTER_LOW, cp_write);
    set_cp_register(CP_DISTANCE_LOW, 0);
}

fn linked_mode() -> bool {
    let control = register16(CP_CONTROL);
    if (control & (CP_READ_ENABLE | CP_LINK_ENABLE)) != (CP_READ_ENABLE | CP_LINK_ENABLE) { return false; }
    let pi_base = register32(PI_FIFO_BASE) & FIFO_PHYSICAL_MASK;
    let cp_base = cp_register(CP_FIFO_BASE_LOW) & FIFO_PHYSICAL_MASK;
    cp_base != 0 && pi_base == cp_base
}

/// gx.cpp push(value, bytes): bytes 1..8 of `value`, big-endian.
pub fn push(value: u64, bytes: u32) {
    // C++: __builtin_bswap64(value << (8 * (8 - bytes))) then the first `bytes` bytes (bytes == 8 shifts by 0).
    let shifted = if bytes >= 8 { value } else { value << (8 * (8 - bytes)) };
    let swapped = shifted.to_be_bytes();
    let data = &swapped[..bytes as usize];
    if record_display_list(data) { return; }
    let cpu = CPU.get();
    if linked_mode() {
        // Claude OPT G5: slice copy up to the burst boundary; same bytes, same burst points.
        let mut rest = data;
        while !rest.is_empty() {
            let start = cpu.gather_count;
            let take = rest.len().min(GATHER_PIPE_SIZE - start);
            cpu.gather_pipe[start..start + take].copy_from_slice(&rest[..take]);
            cpu.gather_count = start + take;
            rest = &rest[take..];
            if cpu.gather_count == GATHER_PIPE_SIZE {
                cpu.gather_count = 0;
                gather_pipe_burst(cpu);
            }
        }
    }
    cpu.fifo.extend_from_slice(data);
    if cpu.fifo.len() >= FLUSH_THRESHOLD { process(); }
}

pub fn take_finish_interrupt() -> bool {
    if register16(PE_CONTROL) & PE_FINISH_ENABLE == 0 { return false; }
    FINISH_PENDING.swap(false, Ordering::SeqCst)
}

pub fn write_pixel_engine_control(value: u16) {
    store16(PE_CONTROL, value & (PE_TOKEN_ENABLE | PE_FINISH_ENABLE));
    if value & PE_FINISH_CLEAR != 0 { FINISH_PENDING.store(false, Ordering::SeqCst); }
}

pub fn frames_drawn() -> u64 { FRAMES_DRAWN.load(Ordering::SeqCst) }
pub fn request_capture() { CAPTURE_REQUESTED.store(true, Ordering::SeqCst); }

// ---------------------------------------------------------------------------------------------------------------
// GPU thread (gx.cpp GpuQueue / gpu_thread / enqueue / process / run_frame_task).

const MAX_FRAMES_AHEAD: i32 = 2;

/// A frame task from C++ (std::function moved to the heap by the forwarder): run(context) then drop(context).
pub struct Task { run: extern "C" fn(*mut c_void), drop: extern "C" fn(*mut c_void), context: *mut c_void }
unsafe impl Send for Task {}
impl Task {
    pub fn new(run: extern "C" fn(*mut c_void), drop: extern "C" fn(*mut c_void), context: *mut c_void) -> Task { Task { run, drop, context } }
    fn call(self) { (self.run)(self.context); (self.drop)(self.context); std::mem::forget(self); }
}
impl Drop for Task { fn drop(&mut self) { (self.drop)(self.context); } }

struct GpuItem { bytes: Vec<u8>, task: Option<Task>, frame: bool }
struct GpuQueue { items: VecDeque<GpuItem>, frames: i32, started: bool, free: Vec<Vec<u8>> }
static GPU: Mutex<GpuQueue> = Mutex::new(GpuQueue { items: VecDeque::new(), frames: 0, started: false, free: Vec::new() });
// Claude OPT G1: spent FIFO buffers are recycled instead of allocating (and page-faulting) a fresh one per poll.
const FREE_BUFFERS: usize = 4;
static WORK: Condvar = Condvar::new();
static SPACE: Condvar = Condvar::new();

fn gpu_thread() {
    let mut buffer: Vec<u8> = Vec::new();
    let mut recycle: Option<Vec<u8>> = None;
    loop {
        let mut hold = GPU.lock().unwrap();
        if let Some(spent) = recycle.take() { if hold.free.len() < FREE_BUFFERS { hold.free.push(spent); } }
        if hold.items.is_empty() {
            drop(hold);
            backend().finish_copies();
            hold = GPU.lock().unwrap();
        }
        hold = WORK.wait_while(hold, |q| q.items.is_empty()).unwrap();
        let item = hold.items.pop_front().unwrap();
        QUEUED_BYTES.fetch_sub(item.bytes.len(), Ordering::SeqCst);
        drop(hold);
        SPACE.notify_all();
        let GpuItem { bytes: mut spent, task, frame } = item;
        if !spent.is_empty() {
            buffer.extend_from_slice(&spent);
            let used = parser().parse(buffer.as_ptr(), buffer.len(), false);
            buffer.drain(..used);
        }
        if spent.capacity() != 0 { spent.clear(); recycle = Some(spent); }
        if let Some(task) = task { task.call(); }
        if frame { GPU.lock().unwrap().frames -= 1; }
    }
}

fn enqueue(item: GpuItem) {
    let mut hold = GPU.lock().unwrap();
    if !hold.started {
        hold.started = true;
        std::thread::spawn(gpu_thread);
    }
    if item.frame {
        if hold.frames >= MAX_FRAMES_AHEAD {
            drop(hold);
            SKIPPED_PRESENTS.fetch_add(1, Ordering::SeqCst);
            return; // the task is dropped unrun, as the C++ GpuItem was
        }
        hold.frames += 1;
    }
    hold = SPACE.wait_while(hold, |_| QUEUED_BYTES.load(Ordering::SeqCst) >= MAX_QUEUED_BYTES).unwrap();
    QUEUED_BYTES.fetch_add(item.bytes.len(), Ordering::SeqCst);
    hold.items.push_back(item);
    drop(hold);
    WORK.notify_one();
}

pub fn process() {
    if !CPU_CLOCK_KNOWN.load(Ordering::SeqCst) {
        CPU_CLOCK.store((host().current_thread_clock)(), Ordering::SeqCst);
        CPU_CLOCK_KNOWN.store(true, Ordering::SeqCst);
    }
    let cpu = CPU.get();
    if cpu.fifo.is_empty() { return; }
    if !settings().threaded {
        let used = parser().parse(cpu.fifo.as_ptr(), cpu.fifo.len(), false);
        cpu.fifo.drain(..used);
        return;
    }
    let capacity = cpu.fifo.capacity();
    let fresh = GPU.lock().unwrap().free.pop().unwrap_or_else(|| Vec::with_capacity(capacity));
    let bytes = std::mem::replace(&mut cpu.fifo, fresh);
    enqueue(GpuItem { bytes, task: None, frame: false });
}

pub fn run_frame_task(task: Task) {
    process();
    if !settings().threaded {
        task.call();
        return;
    }
    enqueue(GpuItem { bytes: Vec::new(), task: Some(task), frame: true });
}

// ---------------------------------------------------------------------------------------------------------------
// C ABI for claude-gx/cpp/gx_rust.cpp (replaces engine/src/gpu/gx.cpp).

#[no_mangle] pub extern "C" fn wpgx_gx_install(host: GxHost) { let _ = HOST.set(host); }
/// Tests: install host services from Rust.
pub fn install(host: GxHost) { let _ = HOST.set(host); }
#[no_mangle] pub extern "C" fn wpgx_gx_push(value: u64, bytes: u32) { push(value, bytes) }
#[no_mangle] pub extern "C" fn wpgx_gx_process() { process() }
#[no_mangle] pub extern "C" fn wpgx_gx_take_finish_interrupt() -> bool { take_finish_interrupt() }
#[no_mangle] pub extern "C" fn wpgx_gx_write_pixel_engine_control(value: u16) { write_pixel_engine_control(value) }
#[no_mangle] pub extern "C" fn wpgx_gx_request_capture() { request_capture() }
#[no_mangle] pub extern "C" fn wpgx_gx_frames_drawn() -> u64 { frames_drawn() }
#[no_mangle] pub extern "C" fn wpgx_gx_run_frame_task(run: extern "C" fn(*mut c_void), drop: extern "C" fn(*mut c_void), context: *mut c_void) {
    run_frame_task(Task::new(run, drop, context))
}
#[no_mangle] pub extern "C" fn wpgx_gx_bp_registers() -> *const u32 { bp_registers() }
#[no_mangle] pub extern "C" fn wpgx_gx_konst_registers() -> *const u32 { konst_registers() }
#[no_mangle] pub extern "C" fn wpgx_gx_xf_registers() -> *const u32 { xf_registers() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn g_format_matches_printf() {
        for (v, s) in [(0.0f32, "0"), (1.0, "1"), (0.5, "0.5"), (-2.25, "-2.25"), (123456.0, "123456"), (1234567.0, "1.23457e+06"), (0.0001, "0.0001"),
                       (0.00001, "1e-05"), (3.14159265, "3.14159"), (100.0, "100"), (-0.0, "-0")] {
            assert_eq!(fmt_g(v), s, "{v}");
        }
    }
    #[test]
    fn shift_scale_is_ldexp() { for i in 0..32 { assert_eq!(shift_scale(i), 2.0f32.powi(-(i as i32))); } }
    #[test]
    fn atoi_and_atof() {
        assert_eq!(atoi("0"), 0); assert_eq!(atoi(" 12x"), 12); assert_eq!(atoi("-3"), -3); assert_eq!(atoi("x"), 0);
        assert_eq!(atof("1.5,3"), 1.5); assert_eq!(atof("2e1x"), 20.0); assert_eq!(atof(""), 0.0);
    }
}
