// SPDX-License-Identifier: GPL-3.0-or-later
// wgpu replacement for engine/src/gpu/gx_render.cpp (Direct3D 11) @ e8522be (Claude, 2026-10-24).
// Same API (gx_render.h), same state machine: batching rules, texture cache, EFB copies, write-backs into guest RAM,
// clears, XFB copy, read_frame. Each function names the C++ function it ports. Direct3D-only details are mapped:
//   * dual-source blending (SRC1_*)        -> Features::DUAL_SOURCE_BLENDING, @blend_src(1)
//   * DepthClipEnable = FALSE              -> PrimitiveState::unclipped_depth (Features::DEPTH_CLIP_CONTROL)
//   * DepthEnable = FALSE                  -> compare Always + no depth writes (D3D11 disables both)
//   * null SRV (reads 0)                   -> 1x1 (0,0,0,0) texture
//   * sampler MipLODBias                   -> gradient scale in the shader (binding 17)
//   * D3D11_MAP_FLAG_DO_NOT_WAIT           -> device.poll(Poll) + "mapped yet?" flag
//   * immediate-mode context               -> one recorded command stream, submitted before any CPU read-back;
//                                             every uniform/vertex upload gets its own ring offset, so write order is preserved
use crate::common::*;
use std::collections::HashMap;
use std::sync::{mpsc, Mutex};
use wprt_ports::gx_state::{BlendFactor, BlendOperation};

/// gx_render.h ScreenVertex (144 bytes).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub color: [[f32; 4]; 2],
    pub uv: [[f32; 3]; 8],
}

/// gx_render.h CopyFilter.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CopyFilter {
    pub coefficients: [u32; 3],
    pub gamma: f32,
    pub clamp_top: bool,
    pub clamp_bottom: bool,
}

/// Services the C++ side provides (registers live in gx.cpp; settings/log in the engine).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Host {
    pub bp_registers: extern "C" fn() -> *const u32,
    pub konst_registers: extern "C" fn() -> *const u32,
    pub xf_registers: extern "C" fn() -> *const u32,
    /// settings LiveNumber("video.scale", "WP_SCALE") clamped 1..6 (C++ requested_scale()).
    pub requested_scale: extern "C" fn() -> i32,
    /// log::write("video", text).
    pub log_video: extern "C" fn(*const std::os::raw::c_char),
    /// Texture pack + dumps (gx_render.cpp prepare_texture_pack / custom_texture / dump_texture / dump_copy). The file side
    /// (custom_textures::Index, name_texture, decode_png, video::save_png*) stays in the C++ engine; None = feature absent (tests).
    pub pack: Option<&'static PackHost>,
}

/// C++ shims in claude-gx/cpp/gx_render_rust.cpp (engine services that stay C++). Paths are UTF-8, NUL-terminated.
#[repr(C)]
pub struct PackHost {
    /// game::description().textures_directory into `directory` ("" = none); returns bit 0 = settings::flag("video.dump_textures",
    /// "WP_DUMP_TEXTURES"), bit 1 = settings::flag("video.custom_textures", "WP_CUSTOM_TEXTURES").
    pub prepare: extern "C" fn(directory: *mut std::os::raw::c_char, capacity: usize) -> u32,
    /// Unused since decode_png moved to Rust (kept so the C++ forwarders' struct layout is unchanged).
    pub decode_png: extern "C" fn(path: *const std::os::raw::c_char, out: extern "C" fn(*mut std::ffi::c_void, *const u32, u32, u32),
                                   context: *mut std::ffi::c_void) -> bool,
    /// video::save_png_rgba(path, pixels, w, h).
    pub save_png_rgba: extern "C" fn(path: *const std::os::raw::c_char, pixels: *const u32, width: u32, height: u32),
    /// video::save_png(path, pixels 0x00RRGGBB, w, h).
    pub save_png: extern "C" fn(path: *const std::os::raw::c_char, pixels: *const u32, width: u32, height: u32),
}

const VERTEX_CAPACITY: u32 = 1 << 16;
const RING_SIZE: u64 = 16 << 20;

// Diagnostic (env WPGX_COPY_LOG=path): same lines as claude-gx/cpp/diag/gx_render_logged.cpp, to diff copies/write-backs.
fn copy_log() -> Option<&'static Mutex<std::fs::File>> {
    static LOG: std::sync::OnceLock<Option<Mutex<std::fs::File>>> = std::sync::OnceLock::new();
    LOG.get_or_init(|| std::env::var_os("WPGX_COPY_LOG").and_then(|p| std::fs::File::create(p).ok()).map(Mutex::new)).as_ref()
}
/// env WPGX_DRAW_FRAMES=lo-hi: also log every batch and clear in those frames (same format as the logged D3D renderer).
fn draw_log_frame(frame: u64) -> bool {
    static RANGE: std::sync::OnceLock<(u64, u64)> = std::sync::OnceLock::new();
    let r = RANGE.get_or_init(|| std::env::var("WPGX_DRAW_FRAMES").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?; Some((a.parse().ok()?, b.parse().ok()?)) }).unwrap_or((1, 0)));
    copy_log().is_some() && frame >= r.0 && frame <= r.1
}
fn fnv_bytes(data: &[u8], mut h: u64) -> u64 { for b in data { h = (h ^ *b as u64).wrapping_mul(0x100000001b3); } h }
fn log_line(text: String) {
    if let Some(f) = copy_log() { use std::io::Write; let _ = f.lock().unwrap().write_all(text.as_bytes()); }
}
/// printf("%a") of a float promoted to double (MinGW and glibc print e.g. 0x1p+0, 0x1.8p+1).
fn hexf(v: f32) -> String {
    let d = v as f64;
    if d == 0.0 { return if d.is_sign_negative() { "-0x0p+0".into() } else { "0x0p+0".into() }; }
    let bits = d.to_bits();
    let sign = if bits >> 63 != 0 { "-" } else { "" };
    let exp = ((bits >> 52) & 0x7FF) as i64 - 1023;
    let mut mant = bits & 0xF_FFFF_FFFF_FFFF;
    let mut digits = 13;
    while digits > 0 && mant & 0xF == 0 { mant >>= 4; digits -= 1; }
    if digits == 0 { format!("{sign}0x1p{exp:+}") } else { format!("{sign}0x1.{mant:0w$x}p{exp:+}", w = digits) }
}

/// Minimal executor for wgpu's native futures (they complete without an external event loop).
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker { RawWaker::new(p, &VTABLE) }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) { return value; }
        std::thread::yield_now();
    }
}
const WRITE_BACK_FRAMES: u64 = 2;
const MAX_PENDING_WRITE_BACKS: usize = 32;

struct CachedTexture {
    view: Option<(u64, wgpu::TextureView)>,
    hash: u64,
    width: u32,
    height: u32,
    format: u32,
    levels: u32,
    verified_epoch: u64,
    mipmaps: bool,
    custom: bool,
}
impl Default for CachedTexture {
    fn default() -> Self { CachedTexture { view: None, hash: 0, width: 0, height: 0, format: 0, levels: 0, verified_epoch: u64::MAX, mipmaps: false, custom: false } }
}

struct CopiedTexture {
    converted: Option<(wgpu::Texture, u64, wgpu::TextureView)>,
    converted_width: u32,
    converted_height: u32,
    view: Option<(u64, wgpu::TextureView)>,
    logical_width: u32,
    logical_height: u32,
    bytes: u32,
    guest_hash: u64,
    pending_writes: u32,
}
impl Default for CopiedTexture {
    fn default() -> Self {
        CopiedTexture { converted: None, converted_width: 0, converted_height: 0, view: None, logical_width: 0, logical_height: 0, bytes: 0, guest_hash: 0, pending_writes: 0 }
    }
}

struct PendingWriteBack {
    // Last GPU use of this staging buffer; later presentation need not finish.
    submission: wgpu::SubmissionIndex,
    staging: wgpu::Buffer,
    padded_row: u32,
    mapped: mpsc::Receiver<bool>,
    address: u32,
    start: u32,
    end: u32,
    row_bytes: u32,
    logical_width: u32,
    logical_height: u32,
    texture_format: u32,
    frame: u64,
}

#[derive(Clone)]
struct PendingBatch {
    vertices: Vec<ScreenVertex>,
    constants: Constants,
    views: [Option<(u64, wgpu::TextureView)>; 8],
    samplers: [Option<(u64, wgpu::Sampler)>; 8],
    lod_bias: [f32; 8],
    blend: u32,
    depth: u32,
    early_depth: bool,
    top_left: u32,
    bottom_right: u32,
    scissor_offset: u32,
}

impl Device {
    fn submit(&mut self) {
        self.gx_pass.take();
        self.gx_bindings.take();
        if let Some(e) = self.encoder.take() { self.submit_encoder(e); }
    }
    /// Uniform data at a fresh 256-byte-aligned ring offset (wraps after submitting everything that used the ring).
    fn uniform(&mut self, data: &[u8]) -> (u64, u64) {
        let size = (data.len() as u64 + 255) / 256 * 256;
        if self.ring_cursor + size > RING_SIZE { self.submit(); self.ring_cursor = 0; }
        let offset = self.ring_cursor;
        self.ring_cursor += size;
        self.stage_uniform(offset, data);
        (offset, data.len() as u64)
    }
    // Same GX constant and LOD-bias slots/bytes, one upload for the adjacent ranges.
    // Existing flush_pending issued two queue.write_buffer calls for every batch.
    fn uniform_pair(&mut self, first: &[u8], second: &[u8]) -> ((u64,u64),(u64,u64)) {
        let first_size = (first.len() as u64 + 255) / 256 * 256;
        let second_size = (second.len() as u64 + 255) / 256 * 256;
        let size = first_size + second_size;
        assert!(size <= RING_SIZE);
        if self.ring_cursor + size > RING_SIZE { self.submit(); self.ring_cursor = 0; }
        let offset = self.ring_cursor;
        self.ring_cursor += size;
        let length = first_size as usize + second.len();
        self.uniform_scratch.resize(length, 0);
        self.uniform_scratch[..first.len()].copy_from_slice(first);
        self.uniform_scratch[first_size as usize..length].copy_from_slice(second);
        let bytes = std::mem::take(&mut self.uniform_scratch);
        self.stage_uniform(offset, &bytes);
        self.uniform_scratch = bytes;
        ((offset, first.len() as u64), (offset + first_size, second.len() as u64))
    }
    // GX draws retain their color/depth attachments until an encoder copy/clear/submit.
    // wgpu RenderPass::forget_lifetime is safe; take_encoder/submit always end it first.
    // Fresh ring slots are disjoint until submit/ring wrap. Upload their same bytes
    // together immediately before submitting the commands which consume those slots.
    fn stage_uniform(&mut self, offset: u64, data: &[u8]) {
        if self.uniform_upload.is_empty() { self.uniform_upload_start = offset; }
        let start = (offset - self.uniform_upload_start) as usize;
        assert!(start >= self.uniform_upload.len());
        self.uniform_upload.resize(start + data.len(), 0);
        self.uniform_upload[start..].copy_from_slice(data);
    }
    // Vertex offsets advance monotonically until the existing submit-before-wrap boundary.
    // Preserve exact bytes/offsets; reuse CPU staging and upload once per consuming encoder.
    fn stage_vertices(&mut self, offset: u64, data: &[u8]) {
        if self.vertex_upload.is_empty() { self.vertex_upload_start = offset; }
        let start = (offset - self.vertex_upload_start) as usize;
        assert_eq!(start, self.vertex_upload.len());
        self.vertex_upload.extend_from_slice(data);
    }
    fn submit_encoder(&mut self, encoder: wgpu::CommandEncoder) -> wgpu::SubmissionIndex {
        assert!(self.gx_pass.is_none());
        if !self.uniform_upload.is_empty() {
            self.queue.write_buffer(&self.ring, self.uniform_upload_start, &self.uniform_upload);
            self.uniform_upload.clear();
        }
        if !self.vertex_upload.is_empty() {
            self.queue.write_buffer(&self.vertex_buffer, self.vertex_upload_start, &self.vertex_upload);
            self.vertex_upload.clear();
        }
        self.queue.submit([encoder.finish()])
    }
    fn take_encoder(&mut self) -> wgpu::CommandEncoder {
        self.gx_pass.take();
        self.gx_bindings.take();
        self.encoder.take().unwrap_or_else(|| self.device.create_command_encoder(&Default::default()))
    }
    fn ensure_gx_pass(&mut self, width: f32, height: f32) {
        if self.gx_pass.is_some() { return; }
        let device = &self.device;
        let encoder = self.encoder.get_or_insert_with(|| device.create_command_encoder(&Default::default()));
        self.gx_pass = Some(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &self.target_view, depth_slice: None, resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &self.depth_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }), stencil_ops: None }),
            timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
        }).forget_lifetime());
        // These bindings are constant for this GX pass; copies/clears start a new pass.
        let pass = self.gx_pass.as_mut().unwrap();
        pass.set_viewport(0.0, 0.0, width, height, 0.0, 1.0);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    }
    fn ring_binding(&self, slot: (u64, u64)) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &self.ring, offset: slot.0, size: std::num::NonZeroU64::new(slot.1) })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PipelineKey { blend: u32, depth: u32, early: bool }
include!("pipeline_preload.rs");

struct Swapchain {
    surface: wgpu::Surface<'static>,
    window: isize,
    config: wgpu::SurfaceConfiguration,
    present: wgpu::RenderPipeline,
    overlay: wgpu::RenderPipeline,
}

struct GxBindings { views: [u64; 8], samplers: [u64; 8], group: wgpu::BindGroup }

struct Device {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    swapchain: Option<Swapchain>,
    present_layout: wgpu::BindGroupLayout,
    present_pipeline_layout: wgpu::PipelineLayout,
    present_module: wgpu::ShaderModule,
    overlay: Option<(wgpu::Texture, wgpu::TextureView, u32, u32)>,
    overlay_visible: bool,
    device: wgpu::Device,
    queue: wgpu::Queue,
    gx_module: wgpu::ShaderModule,
    /// false on DX12: GX early-Z batches then run with late depth (logged once; measured in the comparison).
    early_depth: bool,
    gx_layout: wgpu::BindGroupLayout,
    gx_pipeline_layout: wgpu::PipelineLayout,
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    vertex_upload: Vec<u8>,
    vertex_upload_start: u64,
    ring: wgpu::Buffer,
    ring_cursor: u64,
    uniform_scratch: Vec<u8>,
    uniform_upload: Vec<u8>,
    uniform_upload_start: u64,
    gx_bindings: Option<GxBindings>,
    gx_pass: Option<wgpu::RenderPass<'static>>,
    encoder: Option<wgpu::CommandEncoder>,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    null_view: wgpu::TextureView,
    null_sampler: wgpu::Sampler,
    // present/copy/clear (create_present_pipeline)
    copy_module: wgpu::ShaderModule,
    copy_layout: wgpu::BindGroupLayout,
    copy_pipeline_layout: wgpu::PipelineLayout,
    copy_pipeline: Option<wgpu::RenderPipeline>,
    present_sampler: wgpu::Sampler,
    clear_module: wgpu::ShaderModule,
    clear_layout: wgpu::BindGroupLayout,
    clear_pipeline_layout: wgpu::PipelineLayout,
    clear_pipelines: HashMap<(u32, bool), wgpu::RenderPipeline>,
    frame: Option<(wgpu::Texture, wgpu::TextureView)>,
    frame_width: u32,
    frame_height: u32,
    write_back: Option<(wgpu::Texture, wgpu::TextureView)>,
    free_staging: Vec<wgpu::Buffer>,
    read_staging: Option<wgpu::Buffer>,
    write_backs: Vec<PendingWriteBack>,
    samplers: HashMap<u64, (u64, wgpu::Sampler)>,
    textures: HashMap<u64, CachedTexture>,
    copies: HashMap<u32, CopiedTexture>,
}

/// Host-owned wgpu handles. This type stays entirely inside the host executable;
/// the game DLL receives only the C dispatch table and an opaque context token.
#[derive(Clone)]
pub struct SharedDevice {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

// Private opt-in existing-draw wall-time breakdown; no guest/render state changes.
struct DrawStages { started:std::time::Instant,calls:u64,seconds:[f64;6] }
pub struct Renderer {
    last_alpha_test: Option<(u32, AlphaTestResult)>,
    decoded_levels: Vec<Vec<u32>>,
    decode_padded: Vec<u32>,
    draw_stages:Option<DrawStages>,
    host: Host,
    shared_device: Option<SharedDevice>,
    device: Option<Device>,
    failed: bool,
    pending: PendingBatch,
    tlut: Tlut,
    next_id: u64,
    frame: u64,
    texture_epoch: u64,
    scale: i32,
    vertex_cursor: u32,
    batches: u32,
    batch_vertices: u32,
    batch_seconds: f64,
    write_backs: u32,
    write_back_seconds: f64,
    logged_palette: bool,
    logged_copy_format: bool,
    // gx_render.cpp TexturePack g_pack, g_custom_maps, texture notes and copy dumps.
    pack_dump: bool,
    pack_load: bool,
    dumped: std::collections::HashSet<String>,
    pack_index: crate::custom_textures::Index,
    dump_directory: std::path::PathBuf,
    loaded: HashMap<String, Option<(u64, wgpu::TextureView)>>,
    custom_maps: u32,
    copy_dump_prefix: String,
    copy_dump_index: i32,
    texture_notes: Option<std::fs::File>,
    noted_textures: std::collections::HashSet<u64>,
    // Claude D2 opt-in display-rate interpolation (WP_INTERP); None = unchanged path.
    interp: Option<Box<Interp>>,
}

const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
        count: None,
    }
}
fn sampler_entry(binding: u32, kind: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(kind), count: None }
}
fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}
// GX uniform ring offsets vary per draw; texture/sampler bindings can stay unchanged.
fn dynamic_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    let mut entry = uniform_entry(binding);
    if let wgpu::BindingType::Buffer { ref mut has_dynamic_offset, .. } = entry.ty { *has_dynamic_offset = true; }
    entry
}
fn bytes_of<T>(value: &T) -> &[u8] { unsafe { std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>()) } }
fn slice_bytes<T>(values: &[T]) -> &[u8] { unsafe { std::slice::from_raw_parts(values.as_ptr() as *const u8, std::mem::size_of_val(values)) } }

/// gx_render.cpp d3d_factor.
fn factor(f: BlendFactor) -> wgpu::BlendFactor {
    match f {
        BlendFactor::Zero => wgpu::BlendFactor::Zero,
        BlendFactor::One => wgpu::BlendFactor::One,
        BlendFactor::SourceColor => wgpu::BlendFactor::Src,
        BlendFactor::InverseSourceColor => wgpu::BlendFactor::OneMinusSrc,
        BlendFactor::SourceAlpha => wgpu::BlendFactor::SrcAlpha,
        BlendFactor::InverseSourceAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
        BlendFactor::DestinationColor => wgpu::BlendFactor::Dst,
        BlendFactor::InverseDestinationColor => wgpu::BlendFactor::OneMinusDst,
        BlendFactor::DestinationAlpha => wgpu::BlendFactor::DstAlpha,
        BlendFactor::InverseDestinationAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
        BlendFactor::TevColor => wgpu::BlendFactor::Src1,
        BlendFactor::TevAlpha => wgpu::BlendFactor::Src1Alpha,
        BlendFactor::InverseTevAlpha => wgpu::BlendFactor::OneMinusSrc1Alpha,
    }
}
/// gx_render.cpp d3d_operation. D3D11 SUBTRACT = src - dst, REV_SUBTRACT = dst - src (same as wgpu).
fn operation(o: BlendOperation) -> wgpu::BlendOperation {
    match o { BlendOperation::Subtract => wgpu::BlendOperation::Subtract, BlendOperation::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract, _ => wgpu::BlendOperation::Add }
}

impl Renderer {
    pub fn new(host: Host) -> Self {
        Renderer {
            host,
            shared_device: None,
            device: None,
            failed: false,
            pending: PendingBatch {
                vertices: Vec::new(), constants: Constants::default(), views: Default::default(), samplers: Default::default(), lod_bias: [0.0; 8],
                blend: 0, depth: 0, early_depth: false, top_left: 0, bottom_right: 0, scissor_offset: 0,
            },
            tlut: Tlut::default(),
            next_id: 1,
            frame: 0,
            texture_epoch: 0,
            scale: 1,
            vertex_cursor: VERTEX_CAPACITY,
            batches: 0,
            batch_vertices: 0,
            batch_seconds: 0.0,
            draw_stages:None,
            last_alpha_test: None, decoded_levels: Vec::new(), decode_padded: Vec::new(),
            write_backs: 0,
            write_back_seconds: 0.0,
            logged_palette: false,
            logged_copy_format: false,
            pack_dump: false,
            pack_load: false,
            dumped: Default::default(),
            pack_index: Default::default(),
            dump_directory: Default::default(),
            loaded: HashMap::new(),
            custom_maps: 0,
            copy_dump_prefix: String::new(),
            copy_dump_index: 0,
            texture_notes: None,
            noted_textures: Default::default(),
            interp: Interp::from_env(),
        }
    }
    /// Transport adaptation only; source resource/pipeline creation is unchanged.
    pub fn with_shared_device(host: Host, shared: SharedDevice) -> Self {
        let mut renderer = Self::new(host);
        renderer.shared_device = Some(shared);
        renderer
    }
    fn bp(&self) -> &'static [u32] { unsafe { std::slice::from_raw_parts((self.host.bp_registers)(), 256) } }
    fn konst(&self) -> &'static [u32] { unsafe { std::slice::from_raw_parts((self.host.konst_registers)(), 8) } }
    fn xf(&self) -> &'static [u32] { unsafe { std::slice::from_raw_parts((self.host.xf_registers)(), 0x1100) } }
    fn log(&self, text: &str) {
        let c = std::ffi::CString::new(text.replace('\0', " ")).unwrap();
        (self.host.log_video)(c.as_ptr());
    }
    fn id(&mut self) -> u64 { self.next_id += 1; self.next_id }
    fn scaled(&self, value: i32) -> i32 { value * self.scale }

    /// gx_render.cpp create_target (EFB colour RGBA8 + D32 depth, both sampleable).
    fn create_target(device: &wgpu::Device, scale: i32) -> (wgpu::Texture, wgpu::TextureView, wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d { width: (EFB_WIDTH * scale) as u32, height: (EFB_HEIGHT * scale) as u32, depth_or_array_layers: 1 };
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("efb"), size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: COLOR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("efb depth"), size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
        });
        let tv = target.create_view(&Default::default());
        let dv = depth.create_view(&Default::default());
        (target, tv, depth, dv)
    }

    /// gx_render.cpp initialize + create_pipeline + create_present_pipeline (resources only; pipelines are created on demand).
    fn initialize(&mut self) -> bool {
        if self.device.is_some() { return true; }
        if self.failed { return false; }
        self.scale = (self.host.requested_scale)().clamp(1, 6);
        let device = match self.shared_device.take() {
            Some(shared) => Self::create_resources(shared, self.scale),
            None => Self::create_device(self.scale),
        };
        match device {
            Some(d) => {
                self.device = Some(d);
                // Compile only previously observed valid pipelines before the first game draw.
                // No draw/submit or guest-state access: the normal pipeline cache owns results.
                for &key in OBSERVED_STARTUP_PIPELINES { self.pipeline(key); }
                // prepare_texture_pack()
                self.prepare_texture_pack();
                true
            }
            None => {
                self.log("cannot initialise the wgpu renderer");
                eprintln!("cannot initialise the wgpu renderer");
                self.failed = true;
                false
            }
        }
    }

    fn create_device(scale: i32) -> Option<Device> {
        Self::create_resources(SharedDevice::source_device()?, scale)
    }

    // Same source resources for standalone and Bevy-owned devices. Features must be
    // enabled on the actual device, not merely supported by its physical adapter.
    fn create_resources(shared: SharedDevice, scale: i32) -> Option<Device> {
        let SharedDevice { instance, adapter, device, queue } = shared;
        let required = wgpu::Features::DUAL_SOURCE_BLENDING | wgpu::Features::DEPTH_CLIP_CONTROL;
        if !device.features().contains(required) { return None; }
        let early = device.features().contains(wgpu::Features::SHADER_EARLY_DEPTH_TEST);
        let gx_source = if early { format!("{}\n{}", include_str!("shaders/gx.wgsl"), include_str!("shaders/gx_early.wgsl")) } else { include_str!("shaders/gx.wgsl").to_string() };
        let gx_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("gx"), source: wgpu::ShaderSource::Wgsl(gx_source.into()) });
        let copy_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("copy"), source: wgpu::ShaderSource::Wgsl(include_str!("shaders/copy.wgsl").into()) });
        let clear_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("clear"), source: wgpu::ShaderSource::Wgsl(include_str!("shaders/clear.wgsl").into()) });
        let present_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("present"), source: wgpu::ShaderSource::Wgsl(include_str!("shaders/present.wgsl").into()) });
        let present_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("present"), entries: &[texture_entry(0, wgpu::TextureSampleType::Float { filterable: true }), sampler_entry(1, wgpu::SamplerBindingType::Filtering)],
        });
        let present_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("present"), bind_group_layouts: &[Some(&present_layout)], immediate_size: 0 });
        let mut entries = vec![dynamic_uniform_entry(0)];
        for i in 0..8 { entries.push(texture_entry(1 + i, wgpu::TextureSampleType::Float { filterable: true })); }
        for i in 0..8 { entries.push(sampler_entry(9 + i, wgpu::SamplerBindingType::Filtering)); }
        entries.push(dynamic_uniform_entry(17));
        let gx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("gx"), entries: &entries });
        let gx_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("gx"), bind_group_layouts: &[Some(&gx_layout)], immediate_size: 0 });
        let copy_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("copy"),
            entries: &[texture_entry(0, wgpu::TextureSampleType::Float { filterable: true }), texture_entry(1, wgpu::TextureSampleType::Depth),
                       sampler_entry(2, wgpu::SamplerBindingType::Filtering), uniform_entry(3)],
        });
        let copy_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("copy"), bind_group_layouts: &[Some(&copy_layout)], immediate_size: 0 });
        let clear_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("clear"), entries: &[uniform_entry(0)] });
        let clear_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("clear"), bind_group_layouts: &[Some(&clear_layout)], immediate_size: 0 });
        let buffer = |label, size, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false });
        let vertex_buffer = buffer("vertices", VERTEX_CAPACITY as u64 * std::mem::size_of::<ScreenVertex>() as u64, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST);
        let ring = buffer("uniform ring", RING_SIZE, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let (target, target_view, depth, depth_view) = Self::create_target(&device, scale);
        let null = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("null srv"), size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: COLOR_FORMAT, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
        });
        queue.write_texture(null.as_image_copy(), &[0, 0, 0, 0], wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: None },
                            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 });
        let null_view = null.create_view(&Default::default());
        // D3D11 default sampler state: MIN_MAG_MIP_LINEAR, CLAMP. Also the present/copy sampler (create_present_pipeline).
        let linear_clamp = wgpu::SamplerDescriptor {
            label: Some("linear clamp"), address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge, mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear, lod_min_clamp: 0.0, lod_max_clamp: 32.0, compare: None, anisotropy_clamp: 1, border_color: None,
        };
        let null_sampler = device.create_sampler(&linear_clamp);
        let present_sampler = device.create_sampler(&linear_clamp);
        Some(Device {
            instance, adapter, swapchain: None, present_layout, present_pipeline_layout, present_module, overlay: None, overlay_visible: false,
            device, queue, gx_module, early_depth: early, gx_layout, gx_pipeline_layout, pipelines: HashMap::new(), vertex_buffer, vertex_upload: Vec::new(), vertex_upload_start: 0, ring, ring_cursor: 0, uniform_scratch: Vec::new(), uniform_upload: Vec::new(), uniform_upload_start: 0, gx_bindings: None, gx_pass: None, encoder: None,
            target, target_view, depth, depth_view, null_view, null_sampler, copy_module, copy_layout, copy_pipeline_layout, copy_pipeline: None,
            present_sampler, clear_module, clear_layout, clear_pipeline_layout, clear_pipelines: HashMap::new(),
            frame: None, frame_width: 0, frame_height: 0, write_back: None, free_staging: Vec::new(), read_staging: None, write_backs: Vec::new(),
            samplers: HashMap::new(), textures: HashMap::new(), copies: HashMap::new(),
        })
    }

    /// gx_render.cpp blend_for + depth_for + rasterizer + early_pixel_shader, folded into one wgpu pipeline.
    fn pipeline(&mut self, key: PipelineKey) -> wgpu::RenderPipeline {
        let d = self.device.as_mut().unwrap();
        if let Some(p) = d.pipelines.get(&key) { return p.clone(); }
        let blend = blend_state_for(key.blend);
        let mut mask = wgpu::ColorWrites::empty();
        if blend.color_update { mask |= wgpu::ColorWrites::RED | wgpu::ColorWrites::GREEN | wgpu::ColorWrites::BLUE; }
        if blend.alpha_update { mask |= wgpu::ColorWrites::ALPHA; }
        let blend_state = if blend.enable {
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: factor(blend.source), dst_factor: factor(blend.destination), operation: operation(blend.operation) },
                alpha: wgpu::BlendComponent { src_factor: factor(blend.source_alpha), dst_factor: factor(blend.destination_alpha), operation: operation(blend.alpha_operation) },
            })
        } else { None };
        // depth_for: DepthEnable = word&1, DepthFunc = ((word>>1)&7)+1 (D3D11 NEVER..ALWAYS), write = word bit 4.
        // The dual-source variant only when the blend reads the second output (TEV colour/alpha factors); see gx.wgsl pixel_main_single.
        let src1 = |f: wgpu::BlendFactor| matches!(f, wgpu::BlendFactor::Src1 | wgpu::BlendFactor::OneMinusSrc1 | wgpu::BlendFactor::Src1Alpha | wgpu::BlendFactor::OneMinusSrc1Alpha);
        let dual_source = blend_state.map(|b| src1(b.color.src_factor) || src1(b.color.dst_factor) || src1(b.alpha.src_factor) || src1(b.alpha.dst_factor)).unwrap_or(false);
        let depth_enable = key.depth & 1 != 0;
        let funcs = [wgpu::CompareFunction::Never, wgpu::CompareFunction::Less, wgpu::CompareFunction::Equal, wgpu::CompareFunction::LessEqual,
                     wgpu::CompareFunction::Greater, wgpu::CompareFunction::NotEqual, wgpu::CompareFunction::GreaterEqual, wgpu::CompareFunction::Always];
        let depth_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(depth_enable && (key.depth & (1 << 4)) != 0),
            depth_compare: Some(if depth_enable { funcs[((key.depth >> 1) & 7) as usize] } else { wgpu::CompareFunction::Always }),
            stencil: Default::default(),
            bias: Default::default(),
        };
        let mut attributes = vec![
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 1 },
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 2 },
        ];
        for i in 0..8u32 { attributes.push(wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 48 + 12 * i as u64, shader_location: 3 + i }); }
        let pipeline = d.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gx"),
            layout: Some(&d.gx_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &d.gx_module, entry_point: Some("vertex_main"), compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<ScreenVertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &attributes })],
            },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None,
                unclipped_depth: true, ..Default::default() },
            depth_stencil: Some(depth_stencil),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &d.gx_module, entry_point: Some(match (key.early && d.early_depth, dual_source) {
                    (true, true) => "pixel_main_early", (true, false) => "pixel_main_early_single", (false, true) => "pixel_main", (false, false) => "pixel_main_single" }), compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: blend_state, write_mask: mask })],
            }),
            multiview_mask: None,
            cache: None,
        });
        d.pipelines.insert(key, pipeline.clone());
        pipeline
    }

    /// gx_render.cpp sampler_for.
    fn sampler_for(&mut self, map: u32) -> ((u64, wgpu::Sampler), f32) {
        let params = sampler_params(map, (self.custom_maps >> map) & 1 != 0, self.bp());
        let id = self.next_id + 1;
        let d = self.device.as_mut().unwrap();
        if let Some(s) = d.samplers.get(&params.key) { return (s.clone(), Self::shader_lod_bias(params.lod_bias)); }
        self.next_id = id;
        let address = |wrap: u32| if wrap == 1 { wgpu::AddressMode::Repeat } else if wrap == 2 { wgpu::AddressMode::MirrorRepeat } else { wgpu::AddressMode::ClampToEdge };
        let filter = |linear: bool| if linear { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest };
        // D3D11 MipLODBias: exact sampler bias with the patched wgpu-hal; otherwise the shader scales the gradients by 2^bias.
        #[cfg(feature = "lod-bias-patch")]
        wgpu_hal::lod_bias_patch::set_next_sampler_lod_bias(params.lod_bias);
        let sampler = d.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: address(params.wrap_s),
            address_mode_v: address(params.wrap_t),
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter(params.mag_linear),
            min_filter: filter(params.min_linear),
            mipmap_filter: if params.mip_mode == 2 { wgpu::MipmapFilterMode::Linear } else { wgpu::MipmapFilterMode::Nearest },
            lod_min_clamp: params.min_lod,
            lod_max_clamp: if params.unlimited_lod { 32.0 } else { params.max_lod },
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        #[cfg(feature = "lod-bias-patch")]
        let _ = wgpu_hal::lod_bias_patch::take_next_sampler_lod_bias();
        d.samplers.insert(params.key, (id, sampler.clone()));
        ((id, sampler), Self::shader_lod_bias(params.lod_bias))
    }

    /// gx_render.cpp prepare_texture_pack.
    fn prepare_texture_pack(&mut self) {
        let Some(pack) = self.host.pack else { return };
        let mut buf = vec![0u8; 4096];
        let bits = (pack.prepare)(buf.as_mut_ptr() as *mut _, buf.len());
        let directory = String::from_utf8_lossy(&buf[..buf.iter().position(|&c| c == 0).unwrap_or(0)]).into_owned();
        if directory.is_empty() { return; }
        let base = std::path::PathBuf::from(&directory);
        self.pack_dump = bits & 1 != 0;
        self.dump_directory = base.join("dump");
        if bits & 2 != 0 {
            let count = self.pack_index.scan(&base.join("load"));
            self.pack_load = count > 0;
            if count > 0 { eprintln!("custom textures: {count} files in {directory}/load"); }
        }
        if self.pack_dump { eprintln!("dumping textures to {directory}/dump"); }
    }

    /// custom_textures::decode_png, now Rust (custom_textures.rs, verified against WIC: claude-gx/png-diff). Only used with a pack host.
    fn decode_png(&self, path: &std::path::Path) -> Option<(Vec<u32>, u32, u32)> {
        self.host.pack?;
        crate::custom_textures::decode_png(path)
    }

    /// gx_render.cpp note_texture (texture notes file opened by dump_copies). The C++ file is fopen("w"): text mode on Windows.
    fn note_texture(&mut self, map: u32, address: u32, width: u32, height: u32, format: u32, source: &str) {
        let Some(notes) = self.texture_notes.as_mut() else { return };
        let key = ((address as u64) << 24) ^ ((width as u64) << 12) ^ height as u64 ^ ((format as u64) << 56) ^ ((map as u64) << 60);
        if !self.noted_textures.insert(key) { return; }
        use std::io::Write;
        let newline = if cfg!(windows) { "\r\n" } else { "\n" };
        let _ = write!(notes, "map {map} address {address:08x} {width}x{height} format {format}: {source}{newline}");
    }

    /// gx_render.cpp custom_texture: one cached view per resolved key (a failed load is cached as None, like the C++ map entry).
    fn custom_texture(&mut self, key: &str, mipmaps: bool) -> Option<(u64, wgpu::TextureView)> {
        if let Some(v) = self.loaded.get(key) { return v.clone(); }
        self.loaded.insert(key.to_string(), None);
        let Some(first) = self.pack_index.find(key).cloned().and_then(|path| self.decode_png(&path)) else {
            eprintln!("custom texture {key} cannot be read, using the original");
            return None;
        };
        let (width, height) = (first.1, first.2);
        let mut images = vec![first.0];
        if mipmaps {
            let mut level = 1u32;
            while level < 32 && ((width >> level) != 0 || (height >> level) != 0) {
                let next = self.pack_index.find(&crate::custom_textures::level_name(key, level)).cloned().and_then(|path| self.decode_png(&path));
                let Some(next) = next else { break };
                if next.1 != std::cmp::max(width >> level, 1) || next.2 != std::cmp::max(height >> level, 1) { break; }
                images.push(next.0);
                level += 1;
            }
        }
        // D3D11 MipLevels = 0 + GenerateMips: full chain. GenerateMips' filter is driver-defined; a 2x2 box average is used
        // here (only reached for user packs that ship a single level of a mipmapped texture; not part of any fixture).
        let generate = mipmaps && images.len() == 1 && (width > 1 || height > 1);
        if generate {
            let mut level = 1u32;
            while (width >> level) != 0 || (height >> level) != 0 {
                let (pw, ph) = (std::cmp::max(width >> (level - 1), 1), std::cmp::max(height >> (level - 1), 1));
                let (lw, lh) = (std::cmp::max(width >> level, 1), std::cmp::max(height >> level, 1));
                let prev = images.last().unwrap();
                let mut out = vec![0u32; (lw * lh) as usize];
                for y in 0..lh {
                    for x in 0..lw {
                        let mut acc = [0u32; 4];
                        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let sx = std::cmp::min(2 * x + dx, pw - 1);
                            let sy = std::cmp::min(2 * y + dy, ph - 1);
                            let v = prev[(sy * pw + sx) as usize];
                            for c in 0..4 { acc[c] += (v >> (8 * c)) & 255; }
                        }
                        out[(y * lw + x) as usize] = (0..4).fold(0u32, |o, c| o | (((acc[c] + 2) / 4) << (8 * c)));
                    }
                }
                images.push(out);
                level += 1;
            }
        }
        let id = self.id();
        let d = self.device.as_mut().unwrap();
        let limit = d.device.limits().max_texture_dimension_2d;
        if width == 0 || height == 0 || width > limit || height > limit {
            eprintln!("custom texture {key} cannot be created ({width}x{height}), using the original");
            return None;
        }
        let texture = d.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("custom"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: images.len() as u32, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: COLOR_FORMAT, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
        });
        for (level, pixels) in images.iter().enumerate() {
            let lw = std::cmp::max(width >> level, 1);
            let lh = std::cmp::max(height >> level, 1);
            d.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: level as u32, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                slice_bytes(pixels),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(lw * 4), rows_per_image: Some(lh) },
                wgpu::Extent3d { width: lw, height: lh, depth_or_array_layers: 1 },
            );
        }
        let view = Some((id, texture.create_view(&Default::default())));
        self.loaded.insert(key.to_string(), view.clone());
        view
    }

    /// gx_render.cpp dump_texture(TextureName, pixels, width, height): once per full name, every level, never overwriting.
    fn dump_texture(&mut self, full: &str, pixels: &[Vec<u32>], width: u32, height: u32) {
        if !self.dumped.insert(full.to_string()) { return; }
        let Some(pack) = self.host.pack else { return };
        let _ = std::fs::create_dir_all(&self.dump_directory);
        for (level, p) in pixels.iter().enumerate() {
            let path = self.dump_directory.join(format!("{}.png", crate::custom_textures::level_name(full, level as u32)));
            if path.exists() { continue; }
            if let Ok(c) = std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
                (pack.save_png_rgba)(c.as_ptr(), p.as_ptr(), std::cmp::max(width >> level, 1), std::cmp::max(height >> level, 1));
            }
        }
    }

    /// Bias the shader applies by gradient scaling: none when the sampler carries it (lod-bias-patch).
    fn shader_lod_bias(bias: f32) -> f32 { if cfg!(feature = "lod-bias-patch") { 0.0 } else { bias } }

    /// gx_render.cpp texture_for, incl. texture notes, custom texture packs and texture dumps.
    fn texture_for(&mut self, map: u32) -> Option<(u64, wgpu::TextureView)> {
        self.custom_maps &= !(1u32 << map);
        let bp = self.bp();
        let request = describe_texture(map, bp, &self.tlut);
        if !request.supported && !self.logged_palette {
            self.logged_palette = true;
            eprintln!("unsupported texture format {}", request.format);
        }
        if !request.valid { return None; }
        let (address, width, height, format, levels, size) = (request.address, request.width, request.height, request.format, request.levels, request.size);
        let frame = self.frame;
        let note = |kind: &str| if copy_log().is_some() { log_line(format!("tex frame {frame} map {map} addr {address:08x} {width}x{height} fmt {format} {kind}\n")); };
        let pending_copy = {
            let d = self.device.as_ref().unwrap();
            d.copies.get(&address).filter(|c| c.pending_writes > 0 && c.logical_width == width && c.logical_height == height && format <= 6).map(|c| c.view.clone())
        };
        if let Some(view) = pending_copy {
            note("copy");
            self.note_texture(map, address, width, height, format, "EFB copy");
            return view;
        }
        if !self.device.as_ref().unwrap().write_backs.is_empty() && self.write_back_overlaps(address, size) { self.finish_write_backs(); }
        let copied = self.device.as_ref().unwrap().copies.contains_key(&address);
        let copy_state = {
            let d = self.device.as_ref().unwrap();
            d.copies.get(&address).map(|c| {
                let same_hash = c.guest_hash == unsafe { sample_hash(request.source, c.bytes as usize) };
                (c.logical_width == width && c.logical_height == height && format <= 6 && same_hash, c.view.clone(), c.logical_width, c.logical_height, same_hash)
            })
        };
        match copy_state {
            Some((true, view, ..)) => {
                note("copy");
                self.note_texture(map, address, width, height, format, "EFB copy");
                return view;
            }
            Some((false, _, cw, ch, same_hash)) => {
                note("rejected");
                let reason = format!("EFB copy rejected (copy {cw}x{ch}, hash {}), read from RAM", if same_hash { "same" } else { "changed" });
                self.note_texture(map, address, width, height, format, &reason);
            }
            None => {
                note("ram");
                self.note_texture(map, address, width, height, format, "RAM");
            }
        }
        let key = request.key;
        let mipmaps = request.mipmaps;
        let epoch = self.texture_epoch;
        let custom_bit = 1u32 << map;
        {
            let d = self.device.as_mut().unwrap();
            let entry = d.textures.entry(key).or_default();
            if entry.view.is_some() && entry.verified_epoch == epoch && entry.width == width && entry.height == height && entry.format == format
                && entry.levels == levels && entry.mipmaps == mipmaps {
                if entry.custom { self.custom_maps |= custom_bit; }
                return entry.view.clone();
            }
        }
        let hash = texture_cache_hash(&request);
        {
            let d = self.device.as_mut().unwrap();
            let entry = d.textures.get_mut(&key).unwrap();
            if entry.view.is_some() && entry.hash == hash && entry.width == width && entry.height == height && entry.format == format && entry.levels == levels
                && entry.mipmaps == mipmaps {
                entry.verified_epoch = epoch;
                if entry.custom { self.custom_maps |= custom_bit; }
                return entry.view.clone();
            }
        }
        self.flush_pending();
        {
            let entry = self.device.as_mut().unwrap().textures.get_mut(&key).unwrap();
            entry.view = None;
            entry.custom = false;
        }
        let named = request.supported && !copied && (self.pack_dump || self.pack_load);
        let mut full = String::new();
        let mut custom: Option<(u64, wgpu::TextureView)> = None;
        if named {
            let data = unsafe { std::slice::from_raw_parts(request.source, size) };
            // C++ passes the TMEM palette pointer; name_texture reads tlut[first*2 .. (first+count)*2] (bounded by the TMEM copy).
            let tlut = if request.tlut.is_null() { None } else { Some(&self.tlut.0[request.tlut as usize - self.tlut.0.as_ptr() as usize..]) };
            let name = crate::custom_textures::name_texture(data, width, height, format, mipmaps, tlut);
            full = name.full();
            if self.pack_load {
                let resolved = self.pack_index.resolve(&name);
                if !resolved.is_empty() { custom = self.custom_texture(&resolved, mipmaps); }
            }
        }
        let finish = |r: &mut Renderer, view: Option<(u64, wgpu::TextureView)>, is_custom: bool| {
            let entry = r.device.as_mut().unwrap().textures.get_mut(&key).unwrap();
            entry.view = view;
            entry.custom = is_custom;
            entry.hash = hash;
            entry.width = width;
            entry.height = height;
            entry.format = format;
            entry.levels = levels;
            entry.mipmaps = mipmaps;
            entry.verified_epoch = epoch;
            if is_custom { r.custom_maps |= custom_bit; }
            entry.view.clone()
        };
        if custom.is_some() && !self.pack_dump { return finish(self, custom, true); }
        let mut pixels = std::mem::take(&mut self.decoded_levels);
        decode_levels_into(&request, &mut pixels, &mut self.decode_padded);
        if named && self.pack_dump { self.dump_texture(&full, &pixels, width, height); }
        if custom.is_some() { self.decoded_levels = pixels; self.trim_decode_storage(); return finish(self, custom, true); }
        let id = self.id();
        let d = self.device.as_mut().unwrap();
        let texture = d.device.create_texture(&wgpu::TextureDescriptor {
            label: None, size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: levels, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: COLOR_FORMAT, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
        });
        for level in 0..levels {
            let lw = std::cmp::max(width >> level, 1);
            let lh = std::cmp::max(height >> level, 1);
            d.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: level, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                slice_bytes(&pixels[level as usize]),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(lw * 4), rows_per_image: Some(lh) },
                wgpu::Extent3d { width: lw, height: lh, depth_or_array_layers: 1 },
            );
        }
        let view = Some((id, texture.create_view(&Default::default())));
        self.decoded_levels = pixels;
        self.trim_decode_storage();
        finish(self, view, false)
    }

    // Retain at most the existing ring-size CPU budget, not an unbounded peak-sized texture cache.
    fn trim_decode_storage(&mut self) {
        let words = self.decoded_levels.iter().map(Vec::capacity).sum::<usize>() + self.decode_padded.capacity();
        if words > RING_SIZE as usize / 4 {
            self.decoded_levels = Vec::new();
            self.decode_padded = Vec::new();
        }
    }

    /// gx_render.cpp flush_pending.
    pub fn flush_pending(&mut self) {
        if self.pending.vertices.is_empty() { return; }
        if self.interp.is_some() { self.interp_record_batch(); }
        let started = std::time::Instant::now();
        self.batches += 1;
        self.batch_vertices += self.pending.vertices.len() as u32;
        let key = PipelineKey { blend: self.pending.blend, depth: self.pending.depth, early: self.pending.early_depth };
        if draw_log_frame(self.frame) {
            let p = &self.pending;
            let (mut zmin, mut zmax) = (f32::INFINITY, f32::NEG_INFINITY);
            for v in &p.vertices { zmin = zmin.min(v.z / v.w); zmax = zmax.max(v.z / v.w); }
            log_line(format!("draw frame {} n {} blend {:08x} depth {:x} early {} sc {:08x} {:08x} {:08x} c {:016x} v {:016x} z {} {}\n", self.frame, p.vertices.len(),
                p.blend, p.depth, p.early_depth as u32, p.top_left, p.bottom_right, p.scissor_offset, fnv_bytes(bytes_of(&p.constants), 0xcbf29ce484222325),
                fnv_bytes(slice_bytes(&p.vertices), 0xcbf29ce484222325), hexf(zmin), hexf(zmax)));
            if std::env::var_os("WPGX_DRAW_CONSTANTS").is_some() {
                let c = &p.constants;
                let mut t = format!("  hdr {:08x?} swaps {:08x?} ind {:08x?} init {:?} konst {:?} fog {:?}/{:?}\n", c.header, c.swaps, c.indirect, c.initial, c.konst, c.fog, c.fog_integer);
                for i in 0..(c.header[0] as usize).min(MAX_STAGES) { t += &format!("  st{} {:08x?} ind {:08x}\n", i, c.stage[i], c.tevind[i][0]); }
                let v = &p.vertices[0];
                t += &format!("  v0 pos {:?} c0 {:?} c1 {:?} uv0 {:?} uv1 {:?} uv2 {:?}\n", [v.x, v.y, v.z, v.w], v.color[0], v.color[1], v.uv[0], v.uv[1], v.uv[2]);
                log_line(t);
            }
        }
        let scale = self.scale;
        let mut pending = std::mem::replace(&mut self.pending.vertices, Vec::new());
        let p = &self.pending;
        // scissor (lines 1258-1270)
        let mut left = (((p.top_left >> 12) & 0x7FF) as i32) - scissor_offset_x(p.scissor_offset);
        let mut top = ((p.top_left & 0x7FF) as i32) - scissor_offset_y(p.scissor_offset);
        let mut right = (((p.bottom_right >> 12) & 0x7FF) as i32) - scissor_offset_x(p.scissor_offset) + 1;
        let mut bottom = ((p.bottom_right & 0x7FF) as i32) - scissor_offset_y(p.scissor_offset) + 1;
        left = left.max(0); top = top.max(0); right = right.min(EFB_WIDTH); bottom = bottom.min(EFB_HEIGHT);
        left *= scale; top *= scale; right *= scale; bottom *= scale;
        // Empty scissor cannot produce fragments. Preserve statistics and retained
        // vertex capacity, without allocating pipelines/bindings or staging uploads.
        if right <= left || bottom <= top {
            pending.clear();
            self.pending.vertices = pending;
            return;
        }
        let pipeline = self.pipeline(key);
        let p = &self.pending;
        let d = self.device.as_mut().unwrap();
        let (constants_slot, bias_slot) = d.uniform_pair(bytes_of(&p.constants), slice_bytes(&p.lod_bias));
        let view_ids = std::array::from_fn(|i| p.views[i].as_ref().map(|v|v.0).unwrap_or(0));
        let sampler_ids = std::array::from_fn(|i| p.samplers[i].as_ref().map(|v|v.0).unwrap_or(0));
        // Complete exact resource IDs, one group for the current pass only. IDs start
        // above zero and are never reused; null view/sampler are the zero sentinel.
        if !d.gx_bindings.as_ref().map(|b| b.views == view_ids && b.samplers == sampler_ids).unwrap_or(false) {
            let entries = std::array::from_fn::<_, 18, _>(|i| {
                let resource = match i {
                    0 => d.ring_binding((0, constants_slot.1)),
                    1..=8 => wgpu::BindingResource::TextureView(p.views[i-1].as_ref().map(|v| &v.1).unwrap_or(&d.null_view)),
                    9..=16 => wgpu::BindingResource::Sampler(p.samplers[i-9].as_ref().map(|s| &s.1).unwrap_or(&d.null_sampler)),
                    _ => d.ring_binding((0, bias_slot.1)),
                };
                wgpu::BindGroupEntry { binding: i as u32, resource }
            });
            let group = d.device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &d.gx_layout, entries: &entries });
            d.gx_bindings = Some(GxBindings { views: view_ids, samplers: sampler_ids, group });
        }
        let bind_group = d.gx_bindings.as_ref().unwrap().group.clone();
        let total = pending.len();
        let mut offset = 0usize;
        while offset < total {
            let batch = std::cmp::min(total - offset, VERTEX_CAPACITY as usize) as u32;
            if self.vertex_cursor + batch > VERTEX_CAPACITY { d.submit(); self.vertex_cursor = 0; }
            let stride = std::mem::size_of::<ScreenVertex>() as u64;
            d.stage_vertices(self.vertex_cursor as u64 * stride, slice_bytes(&pending[offset..offset + batch as usize]));
            d.ensure_gx_pass((EFB_WIDTH * scale) as f32, (EFB_HEIGHT * scale) as f32);
            {
                let pass = d.gx_pass.as_mut().unwrap();
                // D3D11 draws nothing for an empty scissor; wgpu rejects it, so skip the draw instead.
                if right > left && bottom > top {
                    pass.set_scissor_rect(left as u32, top as u32, (right - left) as u32, (bottom - top) as u32);
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &bind_group, &[constants_slot.0 as u32, bias_slot.0 as u32]);
                    pass.draw(self.vertex_cursor..self.vertex_cursor + batch, 0..1);
                }
            }
            self.vertex_cursor += batch;
            offset += batch as usize;
        }
        // src: pristine gx_render.cpp:1302 clears pending.vertices while retaining capacity.
        pending.clear();
        self.pending.vertices = pending;
        self.batch_seconds += started.elapsed().as_secs_f64();
    }

    /// gx_render.cpp draw.
    pub fn draw(&mut self, vertices: &[ScreenVertex]) {
        if vertices.is_empty() || !self.initialize() { return; }
        static TIMED:std::sync::OnceLock<bool>=std::sync::OnceLock::new();
        let timed=*TIMED.get_or_init(||std::env::var_os("WP_HOST_DRAW_TIMES").is_some());
        let mut mark=timed.then(std::time::Instant::now);
        let mut seconds=[0.0f64;6];
        let bp = self.bp();
        let mut constants = Constants::default();
        fill_constants(&mut constants, self.scale, bp, self.konst(), self.xf());
        let mut views: [Option<(u64, wgpu::TextureView)>; 8] = Default::default();
        let mut samplers: [Option<(u64, wgpu::Sampler)>; 8] = Default::default();
        let mut lod_bias = [0.0f32; 8];
        let mut used_maps = 0u32;
        let mut i = 0;
        while i < constants.header[0] && (i as usize) < MAX_STAGES {
            if constants.stage[i as usize][2] & 0x40 != 0 { used_maps |= 1 << (constants.stage[i as usize][2] & 7); }
            i += 1;
        }
        if let Some(t)=mark{seconds[0]+=t.elapsed().as_secs_f64();}
        for map in 0..TEXTURE_MAPS {
            if used_maps & (1 << map) != 0 {
                mark=timed.then(std::time::Instant::now);
                views[map as usize] = self.texture_for(map);
                if let Some(t)=mark{seconds[1]+=t.elapsed().as_secs_f64();}
                mark=timed.then(std::time::Instant::now);
                let (s, bias) = self.sampler_for(map);
                samplers[map as usize] = Some(s);
                lod_bias[map as usize] = bias;
                if let Some(t)=mark{seconds[2]+=t.elapsed().as_secs_f64();}
            }
        }
        mark=timed.then(std::time::Instant::now);
        // src: gx_render_common.cpp352 exact full-word cache; per Renderer, under its existing mutex.
        let alpha_word = self.bp()[0xF3];
        let alpha_test = if let Some((key, value)) = self.last_alpha_test.filter(|(key, _)| *key == alpha_word) {
            let _ = key; value
        } else {
            let value = alpha_test_result(alpha_word);
            self.last_alpha_test = Some((alpha_word, value));
            value
        };
        let bp = self.bp();
        let pixel_format = bp[0x43] & 7;
        let blend = (bp[0x41] & 0xFFFF) | (((pixel_format == 1) as u32) << 16) | (((bp[0x42] >> 8) & 1) << 17) | (((!alpha_test.can_pass) as u32) << 18);
        let depth = bp[0x40] & 0x1F;
        let early_depth = (depth & 1) != 0 && (bp[0x43] & (1 << 6)) != 0 && alpha_test.can_pass && alpha_test.can_fail;
        // Source gx_render.cpp1342 compares the fixed resource arrays; no temporary heap vectors.
        let same_views = self.pending.views.iter().zip(views.iter()).all(|(a,b)| a.as_ref().map(|v|v.0)==b.as_ref().map(|v|v.0));
        let same_samplers = self.pending.samplers.iter().zip(samplers.iter()).all(|(a,b)| a.as_ref().map(|v|v.0)==b.as_ref().map(|v|v.0));
        let p = &self.pending;
        let same = !p.vertices.is_empty() && p.blend == blend && p.depth == depth && p.early_depth == early_depth && p.top_left == bp[0x20]
            && p.bottom_right == bp[0x21] && p.scissor_offset == bp[BP_SCISSOR_OFFSET as usize] && bytes_of(&p.constants) == bytes_of(&constants)
            && same_views && same_samplers;
        if let Some(t)=mark{seconds[3]+=t.elapsed().as_secs_f64();}
        mark=timed.then(std::time::Instant::now);
        if !same {
            self.flush_pending();
            let p = &mut self.pending;
            p.constants = constants;
            p.views = views;
            p.samplers = samplers;
            p.lod_bias = lod_bias;
            p.blend = blend;
            p.depth = depth;
            p.early_depth = early_depth;
            p.top_left = bp[0x20];
            p.bottom_right = bp[0x21];
            p.scissor_offset = bp[BP_SCISSOR_OFFSET as usize];
        }
        if let Some(t)=mark{seconds[4]+=t.elapsed().as_secs_f64();}
        mark=timed.then(std::time::Instant::now);
        self.pending.vertices.extend_from_slice(vertices);
        if let Some(t)=mark{seconds[5]+=t.elapsed().as_secs_f64();}
        if timed {
            let sample=self.draw_stages.get_or_insert_with(||DrawStages{started:std::time::Instant::now(),calls:0,seconds:[0.0;6]});
            sample.calls+=1;for(i,s)in seconds.iter().enumerate(){sample.seconds[i]+=s;}
            let elapsed=sample.started.elapsed().as_secs_f64();
            if elapsed>=1.0 {
                eprintln!("draw_stages window_s={:.6} calls={} constants_ms={:.6} textures_ms={:.6} samplers_ms={:.6} compare_ms={:.6} flush_ms={:.6} append_ms={:.6}",elapsed,sample.calls,sample.seconds[0]*1000.0,sample.seconds[1]*1000.0,sample.seconds[2]*1000.0,sample.seconds[3]*1000.0,sample.seconds[4]*1000.0,sample.seconds[5]*1000.0);
                *sample=DrawStages{started:std::time::Instant::now(),calls:0,seconds:[0.0;6]};
            }
        }
    }

    pub fn take_statistics(&mut self) -> (u32, u32, f64) {
        let r = (self.batches, self.batch_vertices, self.batch_seconds);
        self.batches = 0; self.batch_vertices = 0; self.batch_seconds = 0.0;
        r
    }
    pub fn take_copy_statistics(&mut self) -> (u32, f64) {
        let r = (self.write_backs, self.write_back_seconds);
        self.write_backs = 0; self.write_back_seconds = 0.0;
        r
    }
    pub fn invalidate_textures(&mut self) { self.texture_epoch += 1; }
    pub fn load_tlut(&mut self, address: u32, tmem_offset: u32, bytes: u32) { load_tlut(&mut self.tlut, address, tmem_offset, bytes); }

    /// gx_render.cpp create_present_pipeline (copy pipeline part).
    fn copy_pipeline(&mut self) -> wgpu::RenderPipeline {
        let d = self.device.as_mut().unwrap();
        if let Some(p) = &d.copy_pipeline { return p.clone(); }
        let p = d.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("copy"), layout: Some(&d.copy_pipeline_layout),
            vertex: wgpu::VertexState { module: &d.copy_module, entry_point: Some("vertex_main"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, unclipped_depth: true, ..Default::default() },
            depth_stencil: None, multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &d.copy_module, entry_point: Some("copy_main"), compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None, cache: None,
        });
        d.copy_pipeline = Some(p.clone());
        p
    }

    /// gx_render.cpp run_copy.
    fn run_copy(&mut self, target: &wgpu::TextureView, target_width: u32, target_height: u32, x: i32, y: i32, width: i32, height: i32, format: u32,
                intensity: bool, alpha: bool, depth: bool, filter: &CopyFilter) -> bool {
        let pipeline = self.copy_pipeline();
        #[repr(C)]
        struct CopyConstants { params: [u32; 4], region: [f32; 4], filter: [u32; 4], rows: [f32; 4], extent: [f32; 4] }
        let sum = filter.coefficients[0] + filter.coefficients[1] + filter.coefficients[2];
        let efb_height = self.scaled(EFB_HEIGHT);
        let top = if filter.clamp_top { self.scaled(y) } else { 0 };
        let bottom = (if filter.clamp_bottom { self.scaled(y + height) } else { efb_height }) - 1;
        let c = CopyConstants {
            params: [format, intensity as u32, alpha as u32, depth as u32],
            region: [self.scaled(x) as f32, self.scaled(y) as f32, (self.scaled(x + width) - self.scaled(x)) as f32, (self.scaled(y + height) - self.scaled(y)) as f32],
            filter: [filter.coefficients[0], filter.coefficients[1], filter.coefficients[2], if sum >= 128 { 1 } else { 0 }],
            rows: [1.0f32 / filter.gamma, top as f32 + 0.5f32, bottom as f32 + 0.5f32, 0.0],
            extent: [self.scaled(EFB_WIDTH) as f32, efb_height as f32, 0.0, 0.0],
        };
        let d = self.device.as_mut().unwrap();
        let copy_slot = d.uniform(bytes_of(&c));
        let color_view = d.target.create_view(&Default::default());
        let depth_view = d.depth.create_view(&wgpu::TextureViewDescriptor { aspect: wgpu::TextureAspect::DepthOnly, ..Default::default() });
        let bind_group = d.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None, layout: &d.copy_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&color_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&depth_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&d.present_sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: d.ring_binding(copy_slot) },
            ],
        });
        let mut encoder = d.take_encoder();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_viewport(0.0, 0.0, target_width as f32, target_height as f32, 0.0, 1.0);
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        d.encoder = Some(encoder);
        true
    }

    fn create_copy_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let t = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("copy"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: COLOR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
        });
        let v = t.create_view(&Default::default());
        (t, v)
    }

    /// gx_render.cpp queue_write_back.
    fn queue_write_back(&mut self, address: u32, stride: u32, x: i32, y: i32, width: i32, height: i32, logical_width: u32, logical_height: u32, format: u32,
                        intensity: bool, alpha: bool, depth: bool, filter: &CopyFilter) -> bool {
        {
            let d = self.device.as_mut().unwrap();
            if d.write_back.is_none() { d.write_back = Some(Self::create_copy_texture(&d.device, EFB_WIDTH as u32, EFB_HEIGHT as u32)); }
        }
        if self.device.as_ref().unwrap().write_backs.len() >= MAX_PENDING_WRITE_BACKS { self.complete_write_backs(true); }
        let texture_format = copy_texture_format(format);
        let layout = layout_for(texture_format);
        let blocks_x = (logical_width + layout.block_width - 1) / layout.block_width;
        let blocks_y = (logical_height + layout.block_height - 1) / layout.block_height;
        let row_bytes = stride * 32;
        let bytes = block_bytes(texture_format);
        let span = (blocks_y as usize - 1) * row_bytes as usize + (blocks_x * bytes) as usize;
        if blocks_x * bytes > row_bytes || !guest_range_valid(address, span) { return false; }
        let target = self.device.as_ref().unwrap().write_back.as_ref().unwrap().1.clone();
        if !self.run_copy(&target, logical_width, logical_height, x, y, width, height, format, intensity, alpha, depth, filter) { return false; }
        let d = self.device.as_mut().unwrap();
        let padded_row = (EFB_WIDTH as u32 * 4 + 255) / 256 * 256;
        let staging = d.free_staging.pop().unwrap_or_else(|| d.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("write back staging"), size: padded_row as u64 * EFB_HEIGHT as u64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        }));
        let mut encoder = d.take_encoder();
        encoder.copy_texture_to_buffer(
            d.write_back.as_ref().unwrap().0.as_image_copy(),
            wgpu::TexelCopyBufferInfo { buffer: &staging, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded_row), rows_per_image: Some(EFB_HEIGHT as u32) } },
            wgpu::Extent3d { width: logical_width, height: logical_height, depth_or_array_layers: 1 },
        );
        let submission = d.submit_encoder(encoder);
        let (result, mapped) = mpsc::channel();
        // The callback may run on another thread polling the shared device.
        // Preserve the completed map RESULT, not a pending/success conflating flag.
        staging.map_async(wgpu::MapMode::Read, .., move |r| { let _ = result.send(r.is_ok()); });
        let start = address & crate::guest_memory::ADDRESS_MASK;
        d.write_backs.push(PendingWriteBack {
            submission, staging, padded_row, mapped, address, start, end: start + span as u32, row_bytes, logical_width, logical_height, texture_format, frame: self.frame,
        });
        true
    }

    /// gx_render.cpp write_back. Returns false when not ready and `wait` is false (DXGI_ERROR_WAS_STILL_DRAWING).
    fn write_back(&mut self, index: usize, wait: bool) -> bool {
        let d = self.device.as_mut().unwrap();
        let wait_start = std::time::Instant::now();
        // D3D11_MAP_FLAG_DO_NOT_WAIT: measured on the oracle (claude-gx/observe/ref002, logged original renderer), the
        // non-blocking map returned DXGI_ERROR_WAS_STILL_DRAWING for 2,406 of 2,406 write-backs; every write-back landed
        // through a waiting map. wgpu would usually report the copy as finished already, which moves guest RAM writes one
        // frame earlier, so the non-blocking path reports "still drawing" exactly as observed.
        if !wait { return false; }
        // port of gx_render.cpp Map(..., wait ? 0 : DO_NOT_WAIT): a waiting
        // map must resolve before success/failure is decided or staging is reused.
        // wgpu-core LifetimeTracker::map attaches the callback to this staging
        // buffer's last submission. Reuse occurs only after mapped pixels have
        // been consumed and unmap finishes. Keep the same guest writeback point,
        // without waiting for unrelated later work on the shared GX/Bevy queue.
        let submission = d.write_backs[index].submission.clone();
        let polled = d.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission), timeout: None,
        }).is_ok();
        let mapped = polled && d.write_backs[index].mapped.recv().unwrap_or(false);
        if !mapped {
            // Cancel any remaining map before the caller returns this buffer to its free list.
            d.write_backs[index].staging.unmap();
            self.write_back_seconds += wait_start.elapsed().as_secs_f64();
            self.write_backs += 1;
            return true;
        }
        self.write_back_seconds += wait_start.elapsed().as_secs_f64();
        self.write_backs += 1;
        let p = &d.write_backs[index];
        let layout = layout_for(p.texture_format);
        let blocks_x = (p.logical_width + layout.block_width - 1) / layout.block_width;
        let blocks_y = (p.logical_height + layout.block_height - 1) / layout.block_height;
        let bytes = block_bytes(p.texture_format) as usize;
        {
            let view = p.staging.get_mapped_range(..).unwrap();
            let pixels: &[u8] = &view;
            // Claude OPT B3a: exact specialized encode (common::encode_write_back), same output bytes.
            let (address, row_bytes) = (p.address, p.row_bytes);
            unsafe { crate::common::encode_write_back(p.texture_format, pixels, p.padded_row as usize, p.logical_width, p.logical_height, layout.block_width, layout.block_height, blocks_x, blocks_y, bytes, &|by| crate::guest_memory::host(address.wrapping_add(by * row_bytes))); }
        }
        if copy_log().is_some() {
            let mut h: u64 = 0xcbf29ce484222325;
            for by in 0..blocks_y {
                let row = crate::guest_memory::host(p.address.wrapping_add(by * p.row_bytes));
                for i in 0..(blocks_x as usize * bytes) { h = (h ^ unsafe { *row.add(i) } as u64).wrapping_mul(0x100000001b3); }
            }
            log_line(format!("wb frame {} queued {} addr {:08x} fmt {} {}x{} wait {} hash {:016x}\n", self.frame, p.frame, p.address, p.texture_format,
                p.logical_width, p.logical_height, wait as i32, h));
            if let Some(want) = std::env::var("WPGX_DUMP_ADDR").ok().and_then(|v| u32::from_str_radix(&v, 16).ok()) {
                if want == p.address {
                    let mut bytes_out = Vec::new();
                    for by in 0..blocks_y {
                        let row = crate::guest_memory::host(p.address.wrapping_add(by * p.row_bytes));
                        bytes_out.extend_from_slice(unsafe { std::slice::from_raw_parts(row, blocks_x as usize * bytes) });
                    }
                    let path = format!("{}.{:08x}.{}.bin", std::env::var("WPGX_COPY_LOG").unwrap_or_default(), p.address, self.frame);
                    let _ = std::fs::write(path, bytes_out);
                }
            }
        }
        p.staging.unmap();
        true
    }

    /// gx_render.cpp complete_write_backs.
    fn complete_write_backs(&mut self, wait: bool) {
        let mut done = 0usize;
        let count = self.device.as_ref().unwrap().write_backs.len();
        while done < count {
            let w = wait || self.device.as_ref().unwrap().write_backs[done].frame + WRITE_BACK_FRAMES <= self.frame;
            if !self.write_back(done, w) { break; }
            let d = self.device.as_mut().unwrap();
            let address = d.write_backs[done].address;
            let staging = d.write_backs[done].staging.clone();
            d.free_staging.push(staging);
            if let Some(entry) = d.copies.get_mut(&address) {
                if entry.pending_writes > 0 {
                    entry.pending_writes -= 1;
                    if entry.pending_writes == 0 {
                        entry.guest_hash = if guest_range_valid(address, entry.bytes as usize) { unsafe { sample_hash(crate::guest_memory::host(address), entry.bytes as usize) } } else { 0 };
                    }
                }
            }
            done += 1;
        }
        self.device.as_mut().unwrap().write_backs.drain(..done);
    }
    fn finish_write_backs(&mut self) { self.complete_write_backs(true); }
    fn write_back_overlaps(&self, address: u32, size: usize) -> bool {
        let start = address & crate::guest_memory::ADDRESS_MASK;
        let end = start.wrapping_add(size as u32);
        self.device.as_ref().unwrap().write_backs.iter().any(|p| start < p.end && p.start < end)
    }

    /// gx_render.cpp copy_to_texture.
    pub fn copy_to_texture(&mut self, address: u32, stride: u32, x: i32, y: i32, width: i32, height: i32, half: bool, format: u32, intensity: bool, depth: bool,
                           alpha: bool, filter: &CopyFilter) {
        if !self.initialize() { return; }
        if copy_log().is_some() {
            log_line(format!("copy frame {} addr {:08x} stride {} {},{} {}x{} half {} fmt {} int {} depth {} alpha {} filt {},{},{} g {} {}{}\n", self.frame, address, stride,
                x, y, width, height, half as i32, format, intensity as i32, depth as i32, alpha as i32, filter.coefficients[0], filter.coefficients[1], filter.coefficients[2],
                hexf(filter.gamma), filter.clamp_top as i32, filter.clamp_bottom as i32));
        }
        if format > 12 {
            if !self.logged_copy_format { self.logged_copy_format = true; eprintln!("unsupported EFB copy format {format}"); }
            self.device.as_mut().unwrap().copies.remove(&address);
            return;
        }
        let x = x.max(0);
        let y = y.max(0);
        let width = width.min(EFB_WIDTH - x);
        let height = height.min(EFB_HEIGHT - y);
        if width <= 0 || height <= 0 { return; }
        self.flush_pending();
        let (w, h) = (width as u32, height as u32);
        let scale = self.scale as u32;
        let id = self.id();
        let (target_view, target_width, target_height) = {
            let d = self.device.as_mut().unwrap();
            let entry = d.copies.entry(address).or_default();
            entry.logical_width = if half { std::cmp::max(1, w / 2) } else { w };
            entry.logical_height = if half { std::cmp::max(1, h / 2) } else { h };
            let tw = std::cmp::max(1, entry.logical_width * scale);
            let th = std::cmp::max(1, entry.logical_height * scale);
            if entry.converted.is_none() || entry.converted_width != tw || entry.converted_height != th {
                let (t, v) = Self::create_copy_texture(&d.device, tw, th);
                entry.converted = Some((t, id, v));
                entry.converted_width = tw;
                entry.converted_height = th;
            }
            (entry.converted.as_ref().unwrap().2.clone(), tw, th)
        };
        if self.interp.is_some() { self.interp_before_copy(address); }
        if !self.run_copy(&target_view, target_width, target_height, x, y, width, height, format, intensity, alpha || depth, depth, filter) {
            self.device.as_mut().unwrap().copies.remove(&address);
            return;
        }
        let (lw, lh) = {
            let d = self.device.as_mut().unwrap();
            let entry = d.copies.get_mut(&address).unwrap();
            let conv = entry.converted.as_ref().unwrap();
            entry.view = Some((conv.1, conv.2.clone()));
            entry.bytes = entry.logical_width * entry.logical_height * copy_bits(format) / 8;
            (entry.logical_width, entry.logical_height)
        };
        if self.interp.is_some() {
            let conv = self.device.as_ref().unwrap().copies.get(&address).unwrap().converted.as_ref().unwrap().1;
            self.interp_record_copy(InterpCopy { address, conv, tw: target_width, th: target_height, x, y, width, height, format, intensity, alpha: alpha || depth, depth, filter: *filter });
        }
        if self.queue_write_back(address, stride, x, y, width, height, lw, lh, format, intensity, alpha || depth, depth, filter) {
            self.device.as_mut().unwrap().copies.get_mut(&address).unwrap().pending_writes += 1;
        } else {
            let entry = self.device.as_mut().unwrap().copies.get_mut(&address).unwrap();
            if entry.pending_writes == 0 {
                entry.guest_hash = if guest_range_valid(address, entry.bytes as usize) { unsafe { sample_hash(crate::guest_memory::host(address), entry.bytes as usize) } } else { 0 };
            }
        }
        self.dump_copy(address);
    }

    /// gx_render.cpp finish_copies.
    pub fn finish_copies(&mut self) { if self.device.is_some() { self.complete_write_backs(true); } }

    /// gx_render.cpp copy_to_framebuffer.
    pub fn copy_to_framebuffer(&mut self, x: i32, y: i32, width: i32, height: i32, depth: bool, filter: &CopyFilter) {
        if !self.initialize() { return; }
        self.flush_pending();
        self.complete_write_backs(false);
        self.frame += 1;
        self.texture_epoch += 1;
        self.interp_poll();
        let x = x.max(0);
        let y = y.max(0);
        let width = width.min(EFB_WIDTH - x);
        let height = height.min(EFB_HEIGHT - y);
        if width <= 0 || height <= 0 { self.interp_frame_end(None); return; }
        let pw = self.scaled(width) as u32;
        let ph = self.scaled(height) as u32;
        let view = {
            let d = self.device.as_mut().unwrap();
            if d.frame.is_none() || d.frame_width != pw || d.frame_height != ph {
                d.frame = Some(Self::create_copy_texture(&d.device, pw, ph));
                d.frame_width = pw;
                d.frame_height = ph;
            }
            d.frame.as_ref().unwrap().1.clone()
        };
        self.run_copy(&view, pw, ph, x, y, width, height, XFB_FORMAT, false, false, depth, filter);
        // one submission per guest frame (the D3D11 immediate context flushes on its own)
        self.device.as_mut().unwrap().submit();
        if self.interp.is_some() { let f = *filter; self.interp_frame_end(Some((x, y, width, height, depth, f, pw, ph))); }
    }

    /// gx_render.cpp read_texture: whole texture to CPU as 0x00RRGGBB (bytes R,G,B of RGBA8; alpha ignored).
    fn read_texture(&mut self, texture: &wgpu::Texture, pixels: &mut Vec<u32>) -> Option<(u32, u32)> {
        let d = self.device.as_mut().unwrap();
        let (width, height) = (texture.width(), texture.height());
        let padded_row = (width * 4 + 255) / 256 * 256;
        let required = padded_row as u64 * height as u64;
        // This synchronous path finishes mapping and unmaps before returning. The
        // renderer mutex serializes reuse; retain only one capacity-sized buffer.
        if d.read_staging.as_ref().map(|b| b.size() < required).unwrap_or(true) {
            d.read_staging = Some(d.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("read texture"), size: required,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        let staging = d.read_staging.as_ref().unwrap().clone();
        let mut encoder = d.take_encoder();
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo { buffer: &staging, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded_row), rows_per_image: Some(height) } },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        let submission = d.submit_encoder(encoder);
        let (result, mapped) = mpsc::channel();
        staging.map_async(wgpu::MapMode::Read, .., move |r| { let _ = result.send(r.is_ok()); });
        let polled = d.device.poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: None }).is_ok();
        if !polled || !mapped.recv().unwrap_or(false) { staging.unmap(); return None; }
        {
            let view = staging.get_mapped_range(..).unwrap();
            pixels.resize(width as usize * height as usize, 0);
            for row in 0..height as usize {
                let src = &view[row * padded_row as usize..];
                for col in 0..width as usize {
                    pixels[row * width as usize + col] = ((src[col * 4] as u32) << 16) | ((src[col * 4 + 1] as u32) << 8) | src[col * 4 + 2] as u32;
                }
            }
        }
        staging.unmap();
        Some((width, height))
    }

    /// Host-only adaptation of read_frame: immutable GPU copy at the SAME publication point.
    /// Submit immutable copy before publication; shared-queue ordering avoids a CPU completion wait.
    pub fn snapshot_host_frame(&mut self) -> Option<wgpu::Texture> {
        if self.device.is_none() || self.device.as_ref().unwrap().frame.is_none() { return None; }
        self.flush_pending();
        let d = self.device.as_mut().unwrap();
        let source = d.frame.as_ref().unwrap().0.clone();
        let snapshot = d.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("published host frame"), size: source.size(), mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: source.format(),
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mut encoder = d.take_encoder();
        encoder.copy_texture_to_texture(source.as_image_copy(), snapshot.as_image_copy(), source.size());
        d.submit_encoder(encoder);
        // The immutable copy is submitted BEFORE its token is published under the original lock.
        // Bevy samples it on the SAME shared queue after acquiring that publication.
        // wgpu-hal30 Queue::submit contract (lib.rs1229): ordered submissions observe prior writes.
        // CPU consumers still wait/map in read_host_publication; guest RAM readbacks are unchanged.
        Some(snapshot)
    }

    /// Host consumer of the immutable publication: source read_texture packing/map policy,
    /// but a separate command encoder so a CPU consumer cannot flush queued guest work.
    pub fn read_host_publication(shared:&SharedDevice, texture:&wgpu::Texture, pixels:&mut Vec<u32>)->Option<(u32,u32)>{
        let d=shared;let(width,height)=(texture.width(),texture.height());
        let padded_row=(width*4+255)/256*256;
        let staging=d.device.create_buffer(&wgpu::BufferDescriptor{label:Some("published frame readback"),size:padded_row as u64*height as u64,usage:wgpu::BufferUsages::MAP_READ|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
        let mut encoder=d.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(texture.as_image_copy(),wgpu::TexelCopyBufferInfo{buffer:&staging,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(padded_row),rows_per_image:Some(height)}},texture.size());
        // Immutable publication copy consumes no uniform ring; do not flush guest uploads.
        let submission=d.queue.submit([encoder.finish()]);
        let(result,mapped)=mpsc::channel();staging.map_async(wgpu::MapMode::Read,..,move|r|{let _=result.send(r.is_ok());});
        let polled=d.device.poll(wgpu::PollType::Wait{submission_index:Some(submission),timeout:None}).is_ok();
        if !polled||!mapped.recv().unwrap_or(false){staging.unmap();return None;}
        {
            let view=staging.get_mapped_range(..).unwrap();pixels.resize(width as usize*height as usize,0);
            for row in 0..height as usize{let src=&view[row*padded_row as usize..];
                for col in 0..width as usize{pixels[row*width as usize+col]=((src[col*4]as u32)<<16)|((src[col*4+1]as u32)<<8)|src[col*4+2]as u32;}
            }
        }
        staging.unmap();Some((width,height))
    }

    /// gx_render.cpp read_frame: read_texture of the XFB frame.
    pub fn read_frame(&mut self, pixels: &mut Vec<u32>) -> Option<(u32, u32)> {
        if self.device.is_none() || self.device.as_ref().unwrap().frame.is_none() { return None; }
        self.flush_pending();
        let texture = self.device.as_ref().unwrap().frame.as_ref().unwrap().0.clone();
        self.read_texture(&texture, pixels)
    }

    /// gx_render.cpp dump_texture(ID3D11Texture2D*, suffix): <prefix>_copy<index><suffix>.png via video::save_png.
    fn dump_texture_png(&mut self, texture: &wgpu::Texture, suffix: &str) {
        let mut pixels = Vec::new();
        if let Some((w, h)) = self.read_texture(texture, &mut pixels) {
            if let (Some(pack), Ok(path)) = (self.host.pack, std::ffi::CString::new(format!("{}_copy{}{}.png", self.copy_dump_prefix, self.copy_dump_index, suffix))) {
                (pack.save_png)(path.as_ptr(), pixels.as_ptr(), w, h);
            }
        }
    }

    /// gx_render.cpp dump_copy: after every EFB copy while dump_copies(prefix) is active: the EFB, then the converted copy.
    fn dump_copy(&mut self, address: u32) {
        if self.copy_dump_prefix.is_empty() { return; }
        self.copy_dump_index += 1;
        let target = self.device.as_ref().unwrap().target.clone();
        self.dump_texture_png(&target, "_efb");
        let converted = self.device.as_ref().unwrap().copies.get(&address).and_then(|c| c.converted.as_ref().map(|t| t.0.clone()));
        if let Some(t) = converted { self.dump_texture_png(&t, ""); }
    }

    /// gx_render.cpp dump_copies(prefix): None stops dumping. Also (re)opens the texture notes file <prefix>_textures.txt.
    pub fn dump_copies(&mut self, prefix: Option<&str>) {
        self.copy_dump_prefix = prefix.unwrap_or("").to_string();
        self.copy_dump_index = 0;
        self.texture_notes = None;
        self.noted_textures.clear();
        if prefix.is_some() { self.texture_notes = std::fs::File::create(format!("{}_textures.txt", self.copy_dump_prefix)).ok(); }
    }

    /// gx_render.cpp clear.
    pub fn clear(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if !self.initialize() { return; }
        self.flush_pending();
        let bp = self.bp();
        if draw_log_frame(self.frame) {
            log_line(format!("clear frame {} {} {} {} {} bp {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}\n", self.frame, x, y, width, height, bp[0x40], bp[0x41], bp[0x43], bp[0x4F], bp[0x50], bp[0x51]));
        }
        let words = [bp[0x40], bp[0x41], bp[0x43], bp[0x4F], bp[0x50], bp[0x51]];
        if self.interp.is_some() { self.interp_record_clear(x, y, width, height, words); }
        self.clear_words(x, y, width, height, words);
    }
    fn clear_words(&mut self, x: i32, y: i32, width: i32, height: i32, words: [u32; 6]) {
        let mut bp = [0u32; 0x52];
        bp[0x40] = words[0]; bp[0x41] = words[1]; bp[0x43] = words[2]; bp[0x4F] = words[3]; bp[0x50] = words[4]; bp[0x51] = words[5];
        let pixel_format = bp[0x43] & 7;
        let color_enable = (bp[0x41] & (1 << 3)) != 0;
        let has_alpha = pixel_format != 0 && pixel_format != 2 && pixel_format != 3;
        let mut alpha_enable = (bp[0x41] & (1 << 4)) != 0 && has_alpha;
        let depth_enable = (bp[0x40] & (1 << 4)) != 0;
        if !color_enable && !alpha_enable && !depth_enable { return; }
        let mut red = bp[0x4F] & 0xFF;
        let mut alpha = (bp[0x4F] >> 8) & 0xFF;
        let mut green = (bp[0x50] >> 8) & 0xFF;
        let mut blue = bp[0x50] & 0xFF;
        let mut depth = bp[0x51] & 0xFFFFFF;
        if pixel_format == 1 {
            red = quantize(red, 6); green = quantize(green, 6); blue = quantize(blue, 6); alpha = quantize(alpha, 6);
        } else if pixel_format == 2 {
            red = quantize(red, 5); green = quantize(green, 6); blue = quantize(blue, 5);
            depth = (depth & 0xFFFF00) | (depth >> 16);
        }
        if !has_alpha { alpha_enable = true; alpha = 0; }
        let mask = (if color_enable { 7u32 } else { 0 }) | (if alpha_enable { 8 } else { 0 });
        let x = x.max(0);
        let y = y.max(0);
        let width = width.min(EFB_WIDTH - x);
        let height = height.min(EFB_HEIGHT - y);
        if width <= 0 || height <= 0 { return; }
        let constants: [f32; 8] = [red as f32 / 255.0f32, green as f32 / 255.0f32, blue as f32 / 255.0f32, alpha as f32 / 255.0f32,
                                    depth as f32 / 16777215.0f32, 0.0, 0.0, 0.0];
        let (sx, sy, sx2, sy2) = (self.scaled(x), self.scaled(y), self.scaled(x + width), self.scaled(y + height));
        let scale = self.scale;
        let d = self.device.as_mut().unwrap();
        let pipeline = d.clear_pipelines.entry((mask, depth_enable)).or_insert_with(|| {
            let mut writes = wgpu::ColorWrites::empty();
            if mask & 7 != 0 { writes |= wgpu::ColorWrites::RED | wgpu::ColorWrites::GREEN | wgpu::ColorWrites::BLUE; }
            if mask & 8 != 0 { writes |= wgpu::ColorWrites::ALPHA; }
            d.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("clear"), layout: Some(&d.clear_pipeline_layout),
                vertex: wgpu::VertexState { module: &d.clear_module, entry_point: Some("vertex_main"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, unclipped_depth: true, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: Some(depth_enable), depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: Default::default(), bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState { module: &d.clear_module, entry_point: Some("clear_main"), compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: None, write_mask: writes })] }),
                multiview_mask: None, cache: None,
            })
        }).clone();
        let clear_slot = d.uniform(slice_bytes(&constants));
        let bind_group = d.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None, layout: &d.clear_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: d.ring_binding(clear_slot) }],
        });
        let mut encoder = d.take_encoder();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &d.target_view, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &d.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }), stencil_ops: None }),
                timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_viewport(0.0, 0.0, (EFB_WIDTH * scale) as f32, (EFB_HEIGHT * scale) as f32, 0.0, 1.0);
            pass.set_scissor_rect(sx as u32, sy as u32, (sx2 - sx) as u32, (sy2 - sy) as u32);
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        d.encoder = Some(encoder);
    }
}

// ---------------- global instance + C ABI (one renderer per game DLL, like the C++ globals) ----------------
static RENDERER: Mutex<Option<Renderer>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Renderer) -> R) -> Option<R> {
    let mut guard = RENDERER.lock().unwrap();
    guard.as_mut().map(f)
}

// Plain Rust and C entry points share the same dispatch, preserving standalone fallback.
pub fn render_draw(v:&[ScreenVertex]){if v.is_empty(){return;}if let Some(t)=crate::remote::current(){unsafe{(t.draw)(t.context,v.as_ptr(),v.len()as u32)};}else{with(|r|r.draw(v));}}
pub fn render_finish_copies(){if let Some(t)=crate::remote::current(){unsafe{(t.finish)(t.context)};}else{with(|r|r.finish_copies());}}
pub fn render_invalidate_textures(){if let Some(t)=crate::remote::current(){unsafe{(t.invalidate)(t.context)};}else{with(|r|r.invalidate_textures());}}
pub fn render_load_tlut(a:u32,o:u32,n:u32){if let Some(t)=crate::remote::current(){unsafe{(t.tlut)(t.context,a,o,n)};}else{with(|r|r.load_tlut(a,o,n));}}
pub fn render_clear(x:i32,y:i32,w:i32,h:i32){if let Some(t)=crate::remote::current(){unsafe{(t.clear)(t.context,x,y,w,h)};}else{with(|r|r.clear(x,y,w,h));}}
pub fn render_copy_to_framebuffer(x:i32,y:i32,w:i32,h:i32,d:bool,f:&CopyFilter){if let Some(t)=crate::remote::current(){unsafe{(t.framebuffer)(t.context,x,y,w,h,d,f)};}else{with(|r|r.copy_to_framebuffer(x,y,w,h,d,f));}}
pub fn render_copy_to_texture(a:u32,s:u32,x:i32,y:i32,w:i32,h:i32,half:bool,fmt:u32,i:bool,d:bool,al:bool,f:&CopyFilter){if let Some(t)=crate::remote::current(){unsafe{(t.texture)(t.context,a,s,x,y,w,h,half,fmt,i,d,al,f)};}else{with(|r|r.copy_to_texture(a,s,x,y,w,h,half,fmt,i,d,al,f));}}
pub fn render_take_statistics()->(u32,u32,f64){if let Some(t)=crate::remote::current(){let(mut b,mut v,mut s)=(0,0,0.0);unsafe{(t.statistics)(t.context,&mut b,&mut v,&mut s)};(b,v,s)}else{with(|r|r.take_statistics()).unwrap_or_default()}}
pub fn render_take_copy_statistics()->(u32,f64){if let Some(t)=crate::remote::current(){let(mut b,mut s)=(0,0.0);unsafe{(t.copy_statistics)(t.context,&mut b,&mut s)};(b,s)}else{with(|r|r.take_copy_statistics()).unwrap_or_default()}}
pub fn render_dump_copies(p:Option<&str>){if let Some(t)=crate::remote::current(){let p=p.map(|v|std::ffi::CString::new(v).unwrap());unsafe{(t.dump)(t.context,p.as_ref().map_or(std::ptr::null(),|v|v.as_ptr()))};}else{with(|r|r.dump_copies(p));}}
#[no_mangle]pub extern "C" fn wpgx_install(host:Host){if let Some(t)=crate::remote::current(){unsafe{(t.install)(t.context,host,crate::remote::local_memory())};}else{*RENDERER.lock().unwrap()=Some(Renderer::new(host));}}
#[no_mangle]pub unsafe extern "C" fn wpgx_dump_copies(p:*const std::os::raw::c_char){let p=if p.is_null(){None}else{Some(std::ffi::CStr::from_ptr(p).to_string_lossy())};render_dump_copies(p.as_deref());}
#[no_mangle]pub extern "C" fn wpgx_api_name()->*const std::os::raw::c_char{b"wgpu\0".as_ptr().cast()}
#[no_mangle]pub unsafe extern "C" fn wpgx_draw(v:*const ScreenVertex,n:u32){if n!=0{render_draw(std::slice::from_raw_parts(v,n as usize));}}
#[no_mangle]pub extern "C" fn wpgx_finish_copies(){render_finish_copies();}
#[no_mangle]pub unsafe extern "C" fn wpgx_copy_to_framebuffer(x:i32,y:i32,w:i32,h:i32,d:bool,f:*const CopyFilter){render_copy_to_framebuffer(x,y,w,h,d,&*f);}
#[no_mangle]pub unsafe extern "C" fn wpgx_copy_to_texture(a:u32,s:u32,x:i32,y:i32,w:i32,h:i32,half:bool,fmt:u32,i:bool,d:bool,al:bool,f:*const CopyFilter){render_copy_to_texture(a,s,x,y,w,h,half,fmt,i,d,al,&*f);}
#[no_mangle]pub extern "C" fn wpgx_invalidate_textures(){render_invalidate_textures();}
#[no_mangle]pub extern "C" fn wpgx_load_tlut(a:u32,o:u32,n:u32){render_load_tlut(a,o,n);}
#[no_mangle]pub extern "C" fn wpgx_clear(x:i32,y:i32,w:i32,h:i32){render_clear(x,y,w,h);}
#[no_mangle]pub extern "C" fn wpgx_guest_range_valid(a:u32,n:usize)->bool{guest_range_valid(a,n)}
#[no_mangle]pub unsafe extern "C" fn wpgx_take_statistics(b:*mut u32,v:*mut u32,s:*mut f64){let(x,y,z)=render_take_statistics();*b=x;*v=y;*s=z;}
#[no_mangle]pub unsafe extern "C" fn wpgx_take_copy_statistics(b:*mut u32,s:*mut f64){let(x,y)=render_take_copy_statistics();*b=x;*s=y;}
#[no_mangle]pub unsafe extern "C" fn wpgx_read_frame(out:crate::remote::FrameSink,c:*mut std::ffi::c_void)->bool{
 if let Some(t)=crate::remote::current(){return (t.frame)(t.context,out,c);}
 let mut p=Vec::new();match with(|r|r.read_frame(&mut p)).flatten(){Some((w,h))=>{out(c,p.as_ptr(),w,h);true},None=>false}
}

impl Renderer {
    /// gx_render.cpp apply_scale.
    fn apply_scale(&mut self) {
        let scale = (self.host.requested_scale)().clamp(1, 6);
        if scale == self.scale { return; }
        self.flush_pending();
        self.scale = scale;
        let d = self.device.as_mut().unwrap();
        let (t, tv, dt, dv) = Self::create_target(&d.device, scale);
        d.target = t; d.target_view = tv; d.depth = dt; d.depth_view = dv;
        let mut encoder = d.take_encoder();
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &d.target_view, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &d.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
        }
        d.submit_encoder(encoder);
    }

    fn present_pipeline(d: &Device, format: wgpu::TextureFormat, entry: &str, blend: Option<wgpu::BlendState>) -> wgpu::RenderPipeline {
        d.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(entry), layout: Some(&d.present_pipeline_layout),
            vertex: wgpu::VertexState { module: &d.present_module, entry_point: Some("vertex_main"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, unclipped_depth: true, ..Default::default() },
            depth_stencil: None, multisample: Default::default(),
            fragment: Some(wgpu::FragmentState { module: &d.present_module, entry_point: Some(entry), compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None, cache: None,
        })
    }

    /// gx_render.cpp ensure_swapchain (DXGI flip-discard, R8G8B8A8_UNORM, Present(0,0) = no vsync).
    fn ensure_swapchain(&mut self, window: isize, width: u32, height: u32) -> bool {
        let d = self.device.as_mut().unwrap();
        if d.swapchain.as_ref().map(|s| s.window != window).unwrap_or(false) { d.swapchain = None; }
        if d.swapchain.is_none() {
            let Some(hwnd) = std::num::NonZeroIsize::new(window) else { return false };
            let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new())),
                raw_window_handle: wgpu::rwh::RawWindowHandle::Win32(wgpu::rwh::Win32WindowHandle::new(hwnd)),
            };
            let surface = match unsafe { d.instance.create_surface_unsafe(target) } { Ok(s) => s, Err(_) => { self.failed = true; return false; } };
            let caps = surface.get_capabilities(&d.adapter);
            let format = [wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Bgra8Unorm].into_iter().find(|f| caps.formats.contains(f))
                .or_else(|| caps.formats.first().copied());
            let Some(format) = format else { return false };
            // Claude OPT A6: WP_PRESENT_MODE = mailbox (default: no tearing, never blocks) | immediate (old default; best with VRR) | fifo (vsync).
            let preferred: &[wgpu::PresentMode] = match std::env::var("WP_PRESENT_MODE").ok().as_deref() { Some("immediate") => &[wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox], Some("fifo") => &[wgpu::PresentMode::Fifo], _ => &[wgpu::PresentMode::Mailbox, wgpu::PresentMode::Immediate] };
            let present_mode = preferred.iter().copied().find(|m| caps.present_modes.contains(m)).unwrap_or(wgpu::PresentMode::Fifo);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width, height, present_mode, desired_maximum_frame_latency: 2,
                alpha_mode: caps.alpha_modes[0], view_formats: vec![], color_space: wgpu::SurfaceColorSpace::Auto,
            };
            surface.configure(&d.device, &config);
            let present = Self::present_pipeline(d, format, "present_main", None);
            // create_overlay_pipeline blend: SRC_ALPHA/INV_SRC_ALPHA add; alpha ZERO/ONE add.
            let overlay = Self::present_pipeline(d, format, "overlay_main", Some(wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
            }));
            d.swapchain = Some(Swapchain { surface, window, config, present, overlay });
        } else {
            let sc = d.swapchain.as_mut().unwrap();
            if sc.config.width != width || sc.config.height != height {
                sc.config.width = width;
                sc.config.height = height;
                sc.surface.configure(&d.device, &sc.config);
            }
        }
        true
    }

    /// gx_render.cpp upload_overlay (options::Image pixels are B8G8R8A8).
    fn upload_overlay(&mut self, pixels: &[u32], width: u32, height: u32) {
        if pixels.len() != width as usize * height as usize || width == 0 || height == 0 { return; }
        let d = self.device.as_mut().unwrap();
        if d.overlay.as_ref().map(|o| o.2 != width || o.3 != height).unwrap_or(true) {
            let t = d.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("overlay"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
                dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Bgra8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
            });
            let v = t.create_view(&Default::default());
            d.overlay = Some((t, v, width, height));
        }
        let o = d.overlay.as_ref().unwrap();
        d.queue.write_texture(o.0.as_image_copy(), slice_bytes(pixels), wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(width * 4), rows_per_image: None },
                              wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
    }

    /// gx_render.cpp present_frame + draw_overlay. `overlay`: (pixels, w, h) when a new image was taken; placed = options::place().
    pub fn present_frame(&mut self, window: isize, client_width: i32, client_height: i32, aspect: f64, overlay: Option<(&[u32], u32, u32)>,
                         overlay_visible: bool, placed: [i32; 4]) -> bool {
        if self.device.is_none() || self.device.as_ref().unwrap().frame.is_none() || window == 0 { return false; }
        if client_width <= 0 || client_height <= 0 || !self.ensure_swapchain(window, client_width as u32, client_height as u32) { return false; }
        self.flush_pending();
        if let Some((pixels, w, h)) = overlay { self.upload_overlay(pixels, w, h); }
        self.device.as_mut().unwrap().overlay_visible = overlay_visible;
        let d = self.device.as_mut().unwrap();
        let sc = d.swapchain.as_ref().unwrap();
        let frame = match sc.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            _ => return false,
        };
        let view = frame.texture.create_view(&Default::default());
        let mut width = client_width as f64;
        let mut height = width / aspect;
        if height > client_height as f64 { height = client_height as f64; width = height * aspect; }
        let frame_bind = d.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None, layout: &d.present_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&d.frame.as_ref().unwrap().1) },
                       wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&d.present_sampler) }],
        });
        let overlay_bind = if d.overlay_visible && placed[2] > 0 && placed[3] > 0 {
            d.overlay.as_ref().map(|o| d.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None, layout: &d.present_layout,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&o.1) },
                           wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&d.present_sampler) }],
            }))
        } else { None };
        d.gx_pass.take();
        d.gx_bindings.take();
        let mut encoder = d.encoder.take().unwrap_or_else(|| d.device.create_command_encoder(&Default::default()));
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_viewport(((client_width as f64 - width) / 2.0) as f32, ((client_height as f64 - height) / 2.0) as f32, width as f32, height as f32, 0.0, 1.0);
            pass.set_pipeline(&sc.present);
            pass.set_bind_group(0, &frame_bind, &[]);
            pass.draw(0..3, 0..1);
            if let Some(bind) = &overlay_bind {
                pass.set_viewport(placed[0] as f32, placed[1] as f32, placed[2] as f32, placed[3] as f32, 0.0, 1.0);
                pass.set_pipeline(&sc.overlay);
                pass.set_bind_group(0, bind, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        d.submit_encoder(encoder);
        d.queue.present(frame);
        self.apply_scale();
        true
    }
}

#[no_mangle] pub unsafe extern "C" fn wpgx_present_frame(window: isize, client_width: i32, client_height: i32, aspect: f64, overlay_pixels: *const u32,
                                                         overlay_width: u32, overlay_height: u32, overlay_visible: bool, placed: *const i32) -> bool {
    let overlay = if overlay_pixels.is_null() { None } else {
        Some((std::slice::from_raw_parts(overlay_pixels, overlay_width as usize * overlay_height as usize), overlay_width, overlay_height))
    };
    let placed = [*placed, *placed.add(1), *placed.add(2), *placed.add(3)];
    with(|r| r.present_frame(window, client_width, client_height, aspect, overlay, overlay_visible, placed)).unwrap_or(false)
}

impl SharedDevice {
    /// Original accepted offscreen renderer acquisition, now available to the host owner.
    /// No window/surface is created here; all adapter ordering/features remain the source policy.
    pub fn source_device() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let features = wgpu::Features::DUAL_SOURCE_BLENDING | wgpu::Features::DEPTH_CLIP_CONTROL;
        // [earlydepthstencil] needs SHADER_EARLY_DEPTH_TEST, which wgpu offers on Vulkan only: Vulkan is tried first.
        // D3D11CreateDevice(HARDWARE) then WARP: hardware adapter first, then the fallback (software) adapter.
        let mut adapter = None;
        // Diagnostic: WPGX_BACKEND=dx12 skips Vulkan.
        let order: &[wgpu::Backends] = if std::env::var("WPGX_BACKEND").map(|v| v == "dx12").unwrap_or(false) { &[wgpu::Backends::DX12] } else { &[wgpu::Backends::VULKAN, wgpu::Backends::DX12] };
        'search: for &backend in order {
            for a in block_on(instance.enumerate_adapters(backend)) {
                let need = if order.len() == 1 { features } else { features | wgpu::Features::SHADER_EARLY_DEPTH_TEST };
                if a.features().contains(need) && a.get_info().device_type != wgpu::DeviceType::Cpu {
                    adapter = Some(a);
                    break 'search;
                }
            }
        }
        if adapter.is_none() {
            for fallback in [false, true] {
                if let Ok(a) = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: fallback, compatible_surface: None, apply_limit_buckets: false,
                })) {
                    if a.features().contains(features) { adapter = Some(a); break; }
                }
            }
        }
        let adapter = adapter?;
        if copy_log().is_some() { let i = adapter.get_info(); log_line(format!("adapter {:?} {} {:?} driver {} {}\n", i.backend, i.name, i.device_type, i.driver, i.driver_info)); }
        let early = adapter.features().contains(wgpu::Features::SHADER_EARLY_DEPTH_TEST);
        let features = if early { features | wgpu::Features::SHADER_EARLY_DEPTH_TEST } else { features };
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("wprt-gx"),
            required_features: features,
            required_limits: wgpu::Limits { max_bind_groups: 4, max_sampled_textures_per_shader_stage: 16, max_samplers_per_shader_stage: 16, ..wgpu::Limits::default() },
            ..Default::default()
        })).ok()?;
        Some(SharedDevice { instance, adapter, device, queue })
    }
}

// ---------------- Claude D2: optional display-rate interpolation (WP_INTERP=steps, 1..4; off by default) ----------------
// Records the last EFB segment of each guest frame (everything after the last EFB-to-texture copy) and,
// after the XFB copy, replays it into private targets with clip-space positions blended from the previous
// frame. The guest EFB, copies, write-backs and guest RAM are never touched; the extra frames only feed
// the host presenter. Pairs draws by (pipeline, vertex count) in order; cuts/teleports snap to the real frame.
#[derive(Clone, Copy)]
struct InterpCopy { address: u32, conv: u64, tw: u32, th: u32, x: i32, y: i32, width: i32, height: i32, format: u32, intensity: bool, alpha: bool, depth: bool, filter: CopyFilter }
enum InterpOp { Batch(PendingBatch), Clear(i32, i32, i32, i32, [u32; 6]), Copy(InterpCopy) }
struct Interp {
    steps: u32,
    cur: Vec<InterpOp>,
    prev: Vec<InterpOp>,
    seg_started: bool,
    seg: Option<(wgpu::Texture, wgpu::Texture)>,
    replay: Option<(wgpu::Texture, wgpu::TextureView, wgpu::Texture, wgpu::TextureView)>,
    ready: Vec<wgpu::Texture>,
    mixed: Vec<ScreenVertex>,
    stats: (u64, u64, u64),
    // replay-private EFB copy targets by guest address: (texture, view, private id)
    copies: HashMap<u32, (wgpu::Texture, wgpu::TextureView, u64)>,
    reasons: [u64; 4],
    // EFB copy addresses that are sampled before being re-copied in the same frame, and their pre-copy snapshots.
    feedback: std::collections::HashSet<u32>,
    before: HashMap<u32, (wgpu::Texture, wgpu::TextureView, u64)>,
    before_now: Vec<(u64, u32)>,
}
impl Interp {
    fn from_env() -> Option<Box<Interp>> {
        let steps = std::env::var("WP_INTERP_FILE").ok().and_then(|f| std::fs::read_to_string(f).ok())
            .or_else(|| std::env::var("WP_INTERP").ok())?.trim().parse::<u32>().ok()?;
        Self::with_steps(steps)
    }
    fn with_steps(steps: u32) -> Option<Box<Interp>> {
        let steps = steps.min(4);
        if steps == 0 { return None; }
        eprintln!("interp: display-rate interpolation on, {} extra frame(s) per guest frame", steps);
        Some(Box::new(Interp { steps, cur: Vec::new(), prev: Vec::new(), seg_started: false, seg: None, replay: None, ready: Vec::new(), mixed: Vec::new(), stats: (0, 0, 0), copies: HashMap::new(), reasons: [0; 4], feedback: Default::default(), before: HashMap::new(), before_now: Vec::new() }))
    }
}
fn interp_texture(device: &wgpu::Device, like: &wgpu::Texture, label: &'static str) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor { label: Some(label), size: like.size(), mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: like.format(), usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        view_formats: &[] })
}
/// Blend clip-space positions from the previous frame (t=0) to this frame (t=1). Whole batch snaps on
/// a w sign change or a jump of more than half the screen (camera cut, teleport, reused slot).
fn batch_center(v: &[ScreenVertex]) -> Option<(f32, f32)> {
    let step = (v.len() / 8).max(1);
    let (mut x, mut y, mut n) = (0.0f32, 0.0f32, 0u32);
    for q in v.iter().step_by(step) { if q.w > 1e-6 && q.x.is_finite() && q.y.is_finite() { x += q.x / q.w; y += q.y / q.w; n += 1; } }
    (n > 0).then(|| (x / n as f32, y / n as f32))
}
// Pairing key: pipeline, textures, vertex count and the full TEV constant block (material colours etc.), so two
// look-alike objects with different materials (e.g. Miis with different shirt colours) are never paired.
type InterpKey = (u32, u32, bool, usize, [u64; 8], u64);
fn interp_key(b: &PendingBatch) -> InterpKey {
    (b.blend, b.depth, b.early_depth, b.vertices.len(), std::array::from_fn(|i| b.views[i].as_ref().map(|v| v.0).unwrap_or(0)), words_hash(bytes_of(&b.constants)))
}
fn words_hash(data: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    let mut chunks = data.chunks_exact(8);
    for c in &mut chunks { h = (h ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(0x100000001b3).rotate_left(29); }
    for &b in chunks.remainder() { h = (h ^ b as u64).wrapping_mul(0x100000001b3); }
    h
}
fn uv_hash(v: &[ScreenVertex]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for x in v {
        for f in [x.uv[0][0], x.uv[0][1], x.uv[1][0], x.uv[1][1], x.color[0][0], x.color[0][1], x.color[0][2], x.color[0][3], x.color[1][3]] { h = (h ^ f.to_bits() as u64).wrapping_mul(0x100000001b3); }
    }
    h
}
fn blend_positions(cur: &mut [ScreenVertex], prev: &[ScreenVertex], t: f32, jump: f32) -> bool {
    if cur.len() != prev.len() { return false; }
    for (c, p) in cur.iter().zip(prev) {
        // different texture coordinates = a different object/glyph reusing the slot: snap.
        for m in 0..2 { for a in 0..2 { let (u, w) = (c.uv[m][a], p.uv[m][a]); if !(u.is_finite() && w.is_finite()) || (u - w).abs() > 0.05 { return false; } } }
        if !(c.w.is_finite() && p.w.is_finite() && c.x.is_finite() && p.x.is_finite() && c.y.is_finite() && p.y.is_finite() && c.z.is_finite() && p.z.is_finite()) { return false; }
        if (c.w > 0.0) != (p.w > 0.0) || c.w.abs() < 1e-6 || p.w.abs() < 1e-6 { return false; }
        if (c.x / c.w - p.x / p.w).abs() > jump || (c.y / c.w - p.y / p.w).abs() > jump { return false; }
    }
    // Non-rigid jumble (a particle pool whose slots were reshuffled): every vertex moves differently.
    if cur.len() >= 6 {
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for (c, p) in cur.iter().zip(prev) { mx += c.x / c.w - p.x / p.w; my += c.y / c.w - p.y / p.w; }
        let n = cur.len() as f32; mx /= n; my /= n;
        for (c, p) in cur.iter().zip(prev) {
            if ((c.x / c.w - p.x / p.w) - mx).abs() > 0.08 || ((c.y / c.w - p.y / p.w) - my).abs() > 0.08 { return false; }
        }
    }
    for (c, p) in cur.iter_mut().zip(prev) {
        c.x = p.x + (c.x - p.x) * t; c.y = p.y + (c.y - p.y) * t; c.z = p.z + (c.z - p.z) * t; c.w = p.w + (c.w - p.w) * t;
    }
    true
}
impl Renderer {
    /// Live switch: WP_INTERP_FILE holds 0..4 (the stats window writes it); checked every 30 guest frames.
    fn interp_poll(&mut self) {
        static FILE: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        let Some(path) = FILE.get_or_init(|| std::env::var_os("WP_INTERP_FILE").map(Into::into)) else { return; };
        if self.frame % 30 != 0 { return; }
        let want = std::fs::read_to_string(path).ok().and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(0).min(4);
        let have = self.interp.as_ref().map(|i| i.steps).unwrap_or(0);
        if want == have { return; }
        if want == 0 { self.interp = None; eprintln!("interp: off"); }
        else if let Some(it) = self.interp.as_mut() { it.steps = want; eprintln!("interp: {} extra frame(s)", want); }
        else { self.interp = Interp::with_steps(want); }
    }
    fn interp_snapshot(&mut self) {
        let Some(it) = self.interp.as_mut() else { return; };
        if it.seg_started { return; }
        it.seg_started = true;
        let d = self.device.as_mut().unwrap();
        if it.seg.as_ref().map(|(c, _)| c.size() != d.target.size()).unwrap_or(true) {
            it.seg = Some((interp_texture(&d.device, &d.target, "interp segment color"), interp_texture(&d.device, &d.depth, "interp segment depth")));
            it.replay = None;
        }
        let (c, z) = it.seg.as_ref().unwrap();
        let mut e = d.take_encoder();
        e.copy_texture_to_texture(d.target.as_image_copy(), c.as_image_copy(), d.target.size());
        e.copy_texture_to_texture(d.depth.as_image_copy(), z.as_image_copy(), d.depth.size());
        d.encoder = Some(e);
    }
    fn interp_record_batch(&mut self) {
        self.interp_snapshot();
        let b = self.pending.clone();
        if let Some(it) = self.interp.as_mut() { it.cur.push(InterpOp::Batch(b)); }
    }
    fn interp_record_clear(&mut self, x: i32, y: i32, w: i32, h: i32, words: [u32; 6]) {
        self.interp_snapshot();
        if let Some(it) = self.interp.as_mut() { it.cur.push(InterpOp::Clear(x, y, w, h, words)); }
    }
    fn interp_before_copy(&mut self, address: u32) {
        let Some(it) = self.interp.as_mut() else { return; };
        if !it.feedback.contains(&address) || it.before_now.iter().any(|x| x.1 == address) { return; }
        let d = self.device.as_mut().unwrap();
        let Some((tex, conv)) = d.copies.get(&address).and_then(|e| e.converted.as_ref().map(|c| (c.0.clone(), c.1))) else { return; };
        if it.before.get(&address).map(|b| b.0.size() != tex.size()).unwrap_or(true) {
            self.next_id += 1;
            let pid = self.next_id;
            let b = interp_texture(&d.device, &tex, "interp feedback before");
            let v = b.create_view(&Default::default());
            it.before.insert(address, (b, v, pid));
        }
        let b = &it.before[&address].0;
        let mut e = d.take_encoder();
        e.copy_texture_to_texture(tex.as_image_copy(), b.as_image_copy(), tex.size());
        d.encoder = Some(e);
        it.before_now.push((conv, address));
    }
    fn interp_record_copy(&mut self, c: InterpCopy) {
        self.interp_snapshot();
        if let Some(it) = self.interp.as_mut() { it.cur.push(InterpOp::Copy(c)); }
    }
    /// Interpolated frames rendered for the latest XFB copy (oldest first); empty when off or skipped.
    pub fn take_interp_frames(&mut self) -> Vec<wgpu::Texture> {
        self.interp.as_mut().map(|it| std::mem::take(&mut it.ready)).unwrap_or_default()
    }
    fn interp_swap_targets(&mut self, it: &mut Interp) {
        let d = self.device.as_mut().unwrap();
        let r = it.replay.as_mut().unwrap();
        let e = d.take_encoder(); d.encoder = Some(e);
        std::mem::swap(&mut d.target, &mut r.0); std::mem::swap(&mut d.target_view, &mut r.1);
        std::mem::swap(&mut d.depth, &mut r.2); std::mem::swap(&mut d.depth_view, &mut r.3);
    }
    /// After the XFB copy of a guest frame has been submitted.
    fn interp_frame_end(&mut self, xfb: Option<(i32, i32, i32, i32, bool, CopyFilter, u32, u32)>) {
        let Some(mut it) = self.interp.take() else { return; };
        it.ready.clear();
        it.stats.0 += 1;
        let mut done = false;
        if let Some((x, y, w, h, depth, filter, pw, ph)) = xfb {
            if !(it.seg_started && !it.cur.is_empty() && it.seg.is_some()) { it.reasons[0] += 1; }
            else if it.prev.is_empty() { it.reasons[1] += 1; }
            else {
                // Pass 1 pairs identical draws (same pipeline, textures, vertex count and texture coordinates);
                // pass 2 pairs the rest by pipeline/textures/count, and blend_positions then requires close coordinates.
                let mut exact: HashMap<(InterpKey, u64), std::collections::VecDeque<usize>> = HashMap::new();
                let mut loose: HashMap<InterpKey, std::collections::VecDeque<usize>> = HashMap::new();
                let mut used = vec![false; it.prev.len()];
                for (j, op) in it.prev.iter().enumerate() {
                    if let InterpOp::Batch(b) = op { let k = interp_key(b); exact.entry((k, uv_hash(&b.vertices))).or_default().push_back(j); loose.entry(k).or_default().push_back(j); }
                }
                let mut pairs: Vec<Option<usize>> = vec![None; it.cur.len()];
                let mut loose_pair = vec![false; it.cur.len()];
                let (mut total, mut matched) = (0usize, 0usize);
                let mut used_ambiguous = 0u64;
                // Identical objects (same model/texture) can swap draw order (depth sorting): take the candidate
                // whose screen position is nearest, never simply the next one in order.
                let centers: Vec<Option<(f32, f32)>> = it.prev.iter().map(|op| if let InterpOp::Batch(b) = op { batch_center(&b.vertices) } else { None }).collect();
                let pick = |q: &mut std::collections::VecDeque<usize>, used: &Vec<bool>, at: Option<(f32, f32)>| -> Option<usize> {
                    let mut best: Option<(usize, f32)> = None;
                    for (n, &j) in q.iter().enumerate() {
                        if used[j] { continue; }
                        let d = match (at, centers[j]) { (Some(a), Some(b)) => (a.0 - b.0).abs() + (a.1 - b.1).abs(), _ => 1.0e9 };
                        if best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some((n, d)); }
                    }
                    best.map(|(n, _)| q.remove(n).unwrap())
                };
                let mut seen: HashMap<InterpKey, u32> = HashMap::new();
                for op in it.cur.iter() { if let InterpOp::Batch(b) = op { *seen.entry(interp_key(b)).or_default() += 1; } }
                for (i, op) in it.cur.iter().enumerate() {
                    if let InterpOp::Batch(b) = op {
                        total += b.vertices.len();
                        // Many look-alike draws (particles, coins): pairing is a guess, so they are not blended.
                        if seen.get(&interp_key(b)).copied().unwrap_or(0) >= 3 { used_ambiguous += 1; matched += b.vertices.len(); continue; }
                        let at = batch_center(&b.vertices);
                        if let Some(q) = exact.get_mut(&(interp_key(b), uv_hash(&b.vertices))) { if let Some(j) = pick(q, &used, at) { used[j] = true; pairs[i] = Some(j); matched += b.vertices.len(); } }
                    }
                }
                for (i, op) in it.cur.iter().enumerate() {
                    if let (InterpOp::Batch(b), None) = (op, pairs[i]) {
                        if seen.get(&interp_key(b)).copied().unwrap_or(0) >= 3 { continue; }
                        let at = batch_center(&b.vertices);
                        if let Some(q) = loose.get_mut(&interp_key(b)) {
                            let _ = at;
                            while let Some(j) = q.pop_front() { if !used[j] { used[j] = true; pairs[i] = Some(j); loose_pair[i] = true; matched += b.vertices.len(); break; } }
                        }
                    }
                }
                // Feedback: a draw that samples an EFB copy which is overwritten later in this frame must see the
                // copy's previous content in the replay, or the in-between frames show the future (ghosting).
                let mut copied_later: Vec<(u64, u32)> = Vec::new();
                let mut feedback_missing = false;
                for op in it.cur.iter().rev() {
                    match op {
                        InterpOp::Copy(c) => copied_later.push((c.conv, c.address)),
                        InterpOp::Batch(b) => for v in b.views.iter().flatten() {
                            if let Some(&(_, address)) = copied_later.iter().find(|x| x.0 == v.0) {
                                if it.feedback.insert(address) { it.reasons[3] += 0; }
                                if !it.before_now.iter().any(|x| x.1 == address) { feedback_missing = true; }
                            }
                        },
                        _ => {}
                    }
                }
                if feedback_missing { it.reasons[3] += 1; matched = 0; }
                let _ = used_ambiguous;
                if !(total > 0 && matched * 100 >= total * 35) { it.reasons[2] += 1; }
                if total > 0 && matched * 100 >= total * 35 {
                    done = true;
                    if it.replay.is_none() {
                        let d = self.device.as_mut().unwrap();
                        let (c, z) = it.seg.as_ref().unwrap();
                        let rc = interp_texture(&d.device, c, "interp replay color");
                        let rz = interp_texture(&d.device, z, "interp replay depth");
                        let rcv = rc.create_view(&Default::default());
                        let rzv = rz.create_view(&Default::default());
                        it.replay = Some((rc, rcv, rz, rzv));
                    }
                    for k in 1..=it.steps {
                        let t = k as f32 / (it.steps + 1) as f32;
                        {
                            let d = self.device.as_mut().unwrap();
                            let (c, z) = it.seg.as_ref().unwrap();
                            let r = it.replay.as_ref().unwrap();
                            let mut e = d.take_encoder();
                            e.copy_texture_to_texture(c.as_image_copy(), r.0.as_image_copy(), c.size());
                            e.copy_texture_to_texture(z.as_image_copy(), r.2.as_image_copy(), z.size());
                            d.encoder = Some(e);
                        }
                        self.interp_swap_targets(&mut it);
                        let mut subst: Vec<(u64, u64, wgpu::TextureView)> = Vec::new();
                        for &(conv, address) in &it.before_now {
                            if let Some(b) = it.before.get(&address) { subst.push((conv, b.2, b.1.clone())); }
                        }
                        for i in 0..it.cur.len() {
                            match &it.cur[i] {
                                InterpOp::Batch(b) => {
                                    let mut v = std::mem::take(&mut it.mixed);
                                    v.clear();
                                    v.extend_from_slice(&b.vertices);
                                    if let Some(j) = pairs[i] { if let InterpOp::Batch(p) = &it.prev[j] {
                                        // Not blendable (cut, jump, different object): show the nearest real frame's version.
                                        if !blend_positions(&mut v, &p.vertices, t, if loose_pair[i] { 0.2 } else { 0.5 }) && t < 0.5 { v.clear(); v.extend_from_slice(&p.vertices); }
                                    } }
                                    let mut views = b.views.clone();
                                    for slot in views.iter_mut() {
                                        if let Some((id, _)) = slot { if let Some(s) = subst.iter().find(|s| s.0 == *id) { *slot = Some((s.1, s.2.clone())); } }
                                    }
                                    self.pending = PendingBatch { vertices: v, constants: b.constants, views, samplers: b.samplers.clone(), lod_bias: b.lod_bias,
                                        blend: b.blend, depth: b.depth, early_depth: b.early_depth, top_left: b.top_left, bottom_right: b.bottom_right, scissor_offset: b.scissor_offset };
                                    self.flush_pending();
                                    it.mixed = std::mem::take(&mut self.pending.vertices);
                                }
                                InterpOp::Clear(cx, cy, cw, ch, words) => { let (cx, cy, cw, ch, words) = (*cx, *cy, *cw, *ch, *words); self.clear_words(cx, cy, cw, ch, words); }
                                InterpOp::Copy(c) => {
                                    let c = *c;
                                    let need = it.copies.get(&c.address).map(|e| e.0.width() != c.tw || e.0.height() != c.th).unwrap_or(true);
                                    if need {
                                        let pid = self.id();
                                        let (tex, view) = Self::create_copy_texture(&self.device.as_ref().unwrap().device, c.tw, c.th);
                                        it.copies.insert(c.address, (tex, view, pid));
                                    }
                                    let (view, pid) = { let e = it.copies.get(&c.address).unwrap(); (e.1.clone(), e.2) };
                                    self.run_copy(&view, c.tw, c.th, c.x, c.y, c.width, c.height, c.format, c.intensity, c.alpha, c.depth, &c.filter);
                                    subst.retain(|s| s.0 != c.conv);
                                    subst.push((c.conv, pid, view));
                                }
                            }
                        }
                        let (tex, view) = Self::create_copy_texture(&self.device.as_ref().unwrap().device, pw, ph);
                        self.run_copy(&view, pw, ph, x, y, w, h, XFB_FORMAT, false, false, depth, &filter);
                        self.interp_swap_targets(&mut it);
                        it.ready.push(tex);
                    }
                    self.device.as_mut().unwrap().submit();
                    if let Some(dir) = std::env::var_os("WP_INTERP_DUMP") {
                        if it.stats.0 % 240 == 0 {
                            let mut pixels = Vec::new();
                            let frames: Vec<wgpu::Texture> = it.ready.iter().cloned().chain(self.device.as_ref().unwrap().frame.as_ref().map(|f| f.0.clone())).collect();
                            for (k, tex) in frames.iter().enumerate() {
                                if let Some((w, h)) = self.read_texture(tex, &mut pixels) {
                                    let path = std::path::Path::new(&dir).join(format!("f{:06}_{}.bmp", it.stats.0, k));
                                    let _ = std::fs::write(path, bmp24(&pixels, w, h));
                                }
                            }
                        }
                    }
                }
            }
        }
        if done { it.stats.1 += 1; } else { it.stats.2 += 1; }
        if let Some(dir) = std::env::var_os("WP_INTERP_DUMP") {
            if it.stats.0 % 240 == 239 {
                let mut pixels = Vec::new();
                if let Some(tex) = self.device.as_ref().unwrap().frame.as_ref().map(|f| f.0.clone()) {
                    if let Some((w, h)) = self.read_texture(&tex, &mut pixels) {
                        let _ = std::fs::write(std::path::Path::new(&dir).join(format!("f{:06}_prev.bmp", it.stats.0 + 1)), bmp24(&pixels, w, h));
                    }
                }
            }
        }
        if it.stats.0 % 600 == 0 { eprintln!("interp steps {} frames {} interpolated {} skipped {} (empty {} first {} low-match {} feedback-wait {}) ops {} feedback-copies {}", it.steps, it.stats.0, it.stats.1, it.stats.2, it.reasons[0], it.reasons[1], it.reasons[2], it.reasons[3], it.cur.len(), it.feedback.len()); }
        it.prev = std::mem::take(&mut it.cur);
        it.seg_started = false;
        it.before_now.clear();
        self.interp = Some(it);
    }
}

fn bmp24(pixels: &[u32], w: u32, h: u32) -> Vec<u8> {
    let row = ((w * 3 + 3) / 4 * 4) as usize;
    let size = 54 + row * h as usize;
    let mut b = Vec::with_capacity(size);
    b.extend_from_slice(b"BM"); b.extend_from_slice(&(size as u32).to_le_bytes()); b.extend_from_slice(&0u32.to_le_bytes()); b.extend_from_slice(&54u32.to_le_bytes());
    b.extend_from_slice(&40u32.to_le_bytes()); b.extend_from_slice(&(w as i32).to_le_bytes()); b.extend_from_slice(&(h as i32).to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); b.extend_from_slice(&24u16.to_le_bytes()); b.extend_from_slice(&[0u8; 24]);
    for y in (0..h as usize).rev() {
        let start = b.len();
        for x in 0..w as usize { let p = pixels[y * w as usize + x]; b.push(p as u8); b.push((p >> 8) as u8); b.push((p >> 16) as u8); }
        while b.len() - start < row { b.push(0); }
    }
    b
}
