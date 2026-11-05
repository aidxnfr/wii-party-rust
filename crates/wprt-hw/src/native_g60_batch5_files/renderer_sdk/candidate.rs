// AI-assisted (Claude, G60) renderer_sdk.cpp -> Rust; residue.cpp keeps only pack_save_png_rgba (std::filesystem::u8path codecvt).
// ensure_sdk keeps GCC guarded-static semantics; kPack/Host tables, overlay/present/read_frame and pack_save_png are Rust,
// with exact std::vector range-construct / assign and Image lifetimes (Drop guards for C++ unwinding).
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("renderer_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
#[repr(C)] pub struct PackHost { f: [*const (); 4] }
unsafe impl Sync for PackHost {}
#[repr(C)] pub struct Host { f: [*const (); 5], pack: *const PackHost }
#[repr(C)] pub struct Image { pixels: RawVec, width: u32, height: u32 }
#[repr(C)] pub struct OverlayInfo { pixels: *const u32, width: u32, height: u32, empty: bool }
extern "C-unwind" {
    fn wpgx_install(host: Host);
    fn wpgx_read_frame(out: unsafe extern "C-unwind" fn(*mut u8, *const u32, u32, u32), context: *mut u8) -> bool;
    fn wphw_gr_requested_scale() -> i32; fn wphw_gr_log_video(t: *const u8);
    fn wphw_gr_bp() -> *const u32; fn wphw_gr_konst() -> *const u32; fn wphw_gr_xf() -> *const u32;
    fn wphw_gr_pack_prepare(d: *mut u8, c: usize) -> u32;
    fn wphw_gr_pack_decode_png(p: *const u8, out: unsafe extern "C-unwind" fn(*mut u8, *const u32, u32, u32), c: *mut u8) -> bool;
    fn wprt_gr_present(window: *mut u8, aspect: f64, image: *mut Image) -> bool;
    fn wprt_claude_gr_pack_save_png_rgba(path: *const u8, pixels: *const u32, w: u32, h: u32);
    #[link_name = "_ZN2wp5video8save_pngEPKcRKSt6vectorIjSaIjEEjj"] fn save_png(path: *const u8, v: *const RawVec, w: u32, h: u32);
    #[link_name = "_ZN2wp7options12take_overlayERyRNS0_5ImageERb"] fn take_overlay(version: *mut u64, image: *mut Image, visible: *mut bool) -> bool;
}
unsafe extern "C-unwind" fn pack_save_png(path: *const u8, pixels: *const u32, w: u32, h: u32) {
    let v = VecDrop(vec_from_range(pixels.cast(), w as usize * h as usize, 4));
    save_png(path, &v.0, w, h);
}
static K_PACK: PackHost = PackHost { f: [wphw_gr_pack_prepare as *const (), wphw_gr_pack_decode_png as *const (), wprt_claude_gr_pack_save_png_rgba as *const (), pack_save_png as *const ()] };
static mut GUARD: u64 = 0;
static mut INSTALLED: bool = false;
#[export_name = "wphw_gr_ensure_sdk"]
pub unsafe extern "C-unwind" fn wphw_gr_ensure_sdk() {
    guarded_static(std::ptr::addr_of_mut!(GUARD), || {
        wpgx_install(Host { f: [wphw_gr_bp as *const (), wphw_gr_konst as *const (), wphw_gr_xf as *const (), wphw_gr_requested_scale as *const (), wphw_gr_log_video as *const ()], pack: &K_PACK });
        std::ptr::addr_of_mut!(INSTALLED).write(true);
    });
}
#[export_name = "wphw_gr_overlay_sdk"]
pub unsafe extern "C-unwind" fn wphw_gr_overlay_sdk(version: *mut u64, raw: *mut Image, visible: *mut bool, out: *mut OverlayInfo) -> bool {
    let changed = take_overlay(version, raw, visible);
    let px = &(*raw).pixels;
    (*out).pixels = px.start.cast(); (*out).width = (*raw).width; (*out).height = (*raw).height; (*out).empty = px.start == px.finish;
    changed
}
struct ImageDrop(*mut Image);
impl Drop for ImageDrop { fn drop(&mut self) { unsafe { vec_dealloc(&(*self.0).pixels) } } }
/// bool wp::gx::render::present_frame(void*, double)
#[export_name = "_ZN2wp2gx6render13present_frameEPvd"]
pub unsafe extern "C-unwind" fn present_frame(window: *mut u8, aspect: f64) -> bool {
    if window.is_null() { return false; }
    let mut slot = std::mem::MaybeUninit::<Image>::uninit(); let image = slot.as_mut_ptr();
    image.write(Image { pixels: RawVec { start: std::ptr::null_mut(), finish: std::ptr::null_mut(), eos: std::ptr::null_mut() }, width: 0, height: 0 });
    let _g = ImageDrop(image);
    wprt_gr_present(window, aspect, image)
}
#[repr(C)] struct Out { pixels: *mut RawVec, width: *mut u32, height: *mut u32 }
unsafe extern "C-unwind" fn read_out(c: *mut u8, data: *const u32, w: u32, h: u32) {
    let o = &*c.cast::<Out>();
    vec_assign(o.pixels, data.cast(), w as usize * h as usize, 4);
    *o.width = w; *o.height = h;
}
/// bool wp::gx::render::read_frame(std::vector<uint32_t>&, uint32_t&, uint32_t&)
#[export_name = "_ZN2wp2gx6render10read_frameERSt6vectorIjSaIjEERjS6_"]
pub unsafe extern "C-unwind" fn read_frame(pixels: *mut RawVec, width: *mut u32, height: *mut u32) -> bool {
    wphw_gr_ensure_sdk();
    let mut out = Out { pixels, width, height };
    wpgx_read_frame(read_out, std::ptr::addr_of_mut!(out).cast())
}
