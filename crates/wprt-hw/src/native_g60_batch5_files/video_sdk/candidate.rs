// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted (Claude, G60): whole-member Rust port of video_sdk.cpp logic (frame transport, snapshot copy-assign,
// menu/settings/binding/memory forwarders, statistics/present/title/shot/present/option op bodies). The verbatim
// C++ residue (residue.cpp) keeps globals' lifetimes, thread_local exception_ptr, thread starts, the std::function
// lambda and the catch(...) capture sinks that call these bodies.
#![allow(non_upper_case_globals, non_snake_case, static_mut_refs, clashing_extern_declarations, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};
include!("../common/cxx.rs");

#[repr(C)] struct Text { fps: *const c_char, dsp: *const c_char, dsp_native: *const c_char, dsp_interpreted: *const c_char, dsp_stopped: *const c_char }
#[repr(C)] struct DspStatus { loaded: bool, native: bool, load_percent: f64 }
#[repr(C)] struct Description { title: *const c_char, short_name: *const c_char }
#[repr(C)] pub struct TitleInfo {
    fps_label: *const c_char, dsp_label: *const c_char, dsp_native: *const c_char, dsp_interpreted: *const c_char, dsp_stopped: *const c_char,
    status_label: *const c_char, format: *const c_char, title: *const c_char, api: *const c_char,
    fps: f64, load_percent: f64, loaded: bool, is_native: bool,
}
#[repr(C)] struct Frame { pixels: CxxVec, width: u32, height: u32 }
#[repr(C)] struct PresentCtx { next: Frame, pipe: Frame, save: Frame, exception: *mut c_void }
#[repr(C)] pub struct PresentInfo { window: usize, aspect: f64, pixels: *const u32, count: usize, width: u32, height: u32, format: *const c_char, index: c_int, path: *mut c_char }
#[repr(C)] struct ShotCtx { pixels: CxxVec, width: u32, height: u32, path: CxxString, exception: *mut c_void }
#[repr(C)] pub struct ShotInfo { pixels: *const u32, count: usize, width: u32, height: u32, folder: *const c_char, prefix: *const c_char, file: usize, path: *const u8, path_len: usize }

extern "C-unwind" {
    #[link_name = "_ZN2wp2ui4textEv"] fn ui_text() -> *const Text;
    #[link_name = "_ZN2wp3dsp6statusEv"] fn dsp_status() -> DspStatus;
    #[link_name = "_ZN2wp4game11descriptionEv"] fn game_description() -> *const Description;
    #[link_name = "_ZN2wp4mmio6read16Ej"] fn mmio_read16(a: u32) -> u16;
    #[link_name = "_ZN2wp5input8bindingsEv"] fn input_bindings() -> *const c_void;
    #[link_name = "_ZNK2wp6keymap8Bindings5boundEi"] fn bindings_bound(b: *const c_void, code: c_int) -> bool;
    #[link_name = "_ZNK2wp6keymap8Bindings3hasENS0_6ActionEi"] fn bindings_has(b: *const c_void, action: c_int, code: c_int) -> bool;
    #[link_name = "_ZN2wp2gx6render10read_frameERSt6vectorIjSaIjEERjS6_"] fn read_frame(v: *mut CxxVec, w: *mut u32, h: *mut u32) -> bool;
    #[link_name = "_ZN2wp2gx6render13present_frameEPvd"] fn present_frame(window: *mut c_void, aspect: f64) -> bool;
    #[link_name = "_ZN2wp2gx6render8api_nameEv"] fn api_name() -> *const c_char;
    #[link_name = "_ZN2wp5video8save_pngEPKcRKSt6vectorIjSaIjEEjj"] fn save_png(path: *const c_char, pixels: *const CxxVec, w: u32, h: u32);
    #[link_name = "_ZN2wp7options8menu_keyB5cxx11EPvjb"] fn menu_key(ret: *mut CxxString, w: *mut c_void, key: u32, repeat: bool) -> *mut CxxString;
    #[link_name = "_ZN2wp7options13poll_gamepadsB5cxx11EPv"] fn poll_gamepads(ret: *mut CxxString, w: *mut c_void) -> *mut CxxString;
    #[link_name = "_ZN2wp7options10menu_clickB5cxx11EPviib"] fn menu_click(ret: *mut CxxString, w: *mut c_void, x: c_int, y: c_int, right: bool) -> *mut CxxString;
    #[link_name = "_ZN2wp7options10menu_wheelB5cxx11EPvi"] fn menu_wheel(ret: *mut CxxString, w: *mut c_void, delta: c_int) -> *mut CxxString;
    #[link_name = "_ZN2wp7options12refresh_menuEPv"] fn refresh_menu(w: *mut c_void);
    #[link_name = "_ZN2wp8settings5storeEPKcRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn settings_store(key: *const c_char, value: *const CxxString);
    // wp::log::write(const char*, const char*, ...): integer/pointer varargs use the same Win64 registers/slots as fixed args.
    #[link_name = "_ZN2wp3log5writeEPKcS2_z"] fn log_write_s(cat: *const c_char, fmt: *const c_char, a: *const c_char);
    #[link_name = "_ZN2wp3log5writeEPKcS2_z"] fn log_write_uu(cat: *const c_char, fmt: *const c_char, a: u32, b: u32);
    fn wprt_vws_option(window: usize, data: *const u8, n: usize) -> c_int;
    fn wprt_vws_statistics(fps: f64) -> c_int;
    fn wprt_vws_present_on_gpu(window: usize, aspect: f64, context: *mut c_void) -> c_int;
    fn wphw_video_screenshot_exchange() -> bool;
    fn wphw_vc_fullscreen() -> bool;
    fn wprt_claude_video_host_pixels() -> *mut CxxVec;
    fn wprt_claude_vws_rethrow_pending() -> !;
    fn wprt_claude_vws_rethrow(exception: *mut *mut c_void) -> !;
    fn wprt_claude_vws_save_screenshot();
    fn wprt_claude_vws_shot_thread(raw: *mut c_void, info: *mut ShotInfo);
    #[link_name = "_ZNSt15__exception_ptr13exception_ptr10_M_releaseEv"] fn exception_release(p: *mut *mut c_void);
    #[link_name = "_ZSt9terminatev"] fn cxx_terminate() -> !;
}
extern "C" {
    #[link_name = "_ZN2wp8g_memoryE"] static g_memory: *mut u8;
    #[link_name = "_ZN2wp5video17g_next_frame_pathB5cxx11E"] static mut g_next_frame_path: CxxString;
    fn __mingw_snprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    fn __mingw_fprintf(stream: *mut c_void, fmt: *const c_char, ...) -> c_int;
    fn __acrt_iob_func(i: u32) -> *mut c_void;
    fn fputc(c: c_int, stream: *mut c_void) -> c_int;
    fn _exit(code: c_int) -> !;
    fn strlen(s: *const c_char) -> usize;
}
macro_rules! cs { ($s:literal) => { concat!($s, "\0").as_ptr() as *const c_char } }

// ~vector<uint32_t>() / ~basic_string() as inlined on the normal path (sized delete).
#[inline] unsafe fn str_free_inline(s: *mut CxxString) { if (*s).p != s_local(s) { cxx_delete((*s).p, (*s).u[0] as usize + 1); } }
struct VecGuard(*mut CxxVec);
impl Drop for VecGuard { fn drop(&mut self) { unsafe { v_free(self.0) } } }
// landing-pad destruction of a returned std::string (out-of-line _M_dispose), disarmed on the normal path.
struct StrUnwind(*mut CxxString);
impl Drop for StrUnwind { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }

#[no_mangle] pub unsafe extern "C-unwind" fn wphw_video_read_frame(receive: unsafe extern "C-unwind" fn(usize, u32, u32)) {
    let mut pixels = CxxVec { b: null_mut(), e: null_mut(), c: null_mut() };
    let (mut width, mut height) = (0u32, 0u32);
    let p = addr_of_mut!(pixels); let _g = VecGuard(p);
    if !read_frame(p, &mut width, &mut height) { return; }
    receive(p as usize, width, height);
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_video_move_frame(frame: usize) {
    let h = wprt_claude_video_host_pixels(); let src = frame as *mut CxxVec;
    let (ob, oc) = ((*h).b, (*h).c);
    (*h).b = (*src).b; (*h).e = (*src).e; (*h).c = (*src).c;
    (*src).b = null_mut(); (*src).e = null_mut(); (*src).c = null_mut();
    if !ob.is_null() { cxx_delete(ob, oc as usize - ob as usize); }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_video_snapshot(context: usize, data: *mut *mut u32, count: *mut usize) -> bool {
    let h = wprt_claude_video_host_pixels();
    if (*h).e == (*h).b { return false; }
    let s = context as *mut CxxVec;
    if s != h {
        // vector<uint32_t>::operator=(const vector&)
        let xlen = (*h).e as usize - (*h).b as usize;
        let cap = (*s).c as usize - (*s).b as usize;
        if xlen > cap {
            let tmp = cxx_new(xlen);
            core::ptr::copy_nonoverlapping((*h).b, tmp, xlen);
            if !(*s).b.is_null() { cxx_delete((*s).b, cap); }
            (*s).b = tmp; (*s).c = tmp.add(xlen);
        } else {
            let size = (*s).e as usize - (*s).b as usize;
            if size >= xlen { if xlen != 0 { core::ptr::copy((*h).b, (*s).b, xlen); } }
            else {
                if size != 0 { core::ptr::copy((*h).b, (*s).b, size); }
                core::ptr::copy_nonoverlapping((*h).b.add(size), (*s).e, xlen - size);
            }
        }
        (*s).e = (*s).b.add(xlen);
    }
    *data = (*s).b as *mut u32; *count = v_len(s, 4);
    true
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_video_log_build() { log_write_s(cs!("video"), cs!("build %s"), cs!("2610-012")); }

#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_bound(code: c_int) -> bool { bindings_bound(input_bindings(), code) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_screenshot_binding(code: c_int) -> bool { bindings_has(input_bindings(), 0x13, code) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_exit() -> ! { _exit(0) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_store_fullscreen() {
    let full = wphw_vc_fullscreen();
    let mut value = CxxString { p: null_mut(), n: 1, u: [0; 2] };
    let v = addr_of_mut!(value);
    (*v).p = s_local(v); *(*v).p = b'0' + full as u8; *(*v).p.add(1) = 0;
    let g = StrUnwind(v);
    settings_store(cs!("video.fullscreen"), v);
    core::mem::forget(g); str_free_inline(v);
}
unsafe fn apply_option(window: usize, data: *const u8, n: usize) { if wprt_vws_option(window, data, n) < 0 { wprt_claude_vws_rethrow_pending(); } }
unsafe fn option_then_apply(window: usize, make: impl FnOnce(*mut CxxString)) {
    let mut s = core::mem::MaybeUninit::<CxxString>::uninit(); let sp = s.as_mut_ptr();
    make(sp);
    let g = StrUnwind(sp);
    apply_option(window, (*sp).p, (*sp).n);
    core::mem::forget(g); str_free_inline(sp);
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_menu_key(raw: usize, key: u32, repeat: bool) { option_then_apply(raw, |sp| { menu_key(sp, raw as *mut c_void, key, repeat); }); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_poll_gamepads(raw: usize) { option_then_apply(raw, |sp| { poll_gamepads(sp, raw as *mut c_void); }); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_menu_click(raw: usize, x: c_int, y: c_int, right: bool) { option_then_apply(raw, |sp| { menu_click(sp, raw as *mut c_void, x, y, right); }); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vc_menu_wheel(raw: usize, delta: c_int) {
    let mut s = core::mem::MaybeUninit::<CxxString>::uninit(); let sp = s.as_mut_ptr();
    menu_wheel(sp, raw as *mut c_void, delta); str_free_inline(sp);
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vws_read8(address: u32) -> u8 {
    let m = g_memory;
    let p = if (address & 0xF000_0000) == 0xE000_0000 { m.add(0x1400_0000 + (address & 0x3FFF) as usize) } else { m.add((address & 0x1FFF_FFFF) as usize) };
    *p
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_vws_read16(address: u32) -> u16 {
    if address >= 0xCC00_0000 { return mmio_read16(address); }
    u16::from_be(core::ptr::read_unaligned(g_memory.add((address & 0x1FFF_FFFF) as usize) as *const u16))
}

#[export_name = "_ZN2wp5video17update_statisticsEd"]
pub unsafe extern "C-unwind" fn update_statistics(fps: f64) { if wprt_vws_statistics(fps) < 0 { wprt_claude_vws_rethrow_pending(); } }
#[export_name = "_ZN2wp5video15save_next_frameEPKc"]
pub unsafe extern "C-unwind" fn save_next_frame(path: *const c_char) {
    let n = strlen(path); let s = addr_of_mut!(g_next_frame_path);
    cxx_s_replace(s, 0, (*s).n, path as *const u8, n);
}
struct PresentGuard(*mut PresentCtx);
impl Drop for PresentGuard { fn drop(&mut self) { unsafe {
    let c = self.0;
    if !(*c).exception.is_null() { exception_release(addr_of_mut!((*c).exception)); }
    v_free(addr_of_mut!((*c).save.pixels)); v_free(addr_of_mut!((*c).pipe.pixels)); v_free(addr_of_mut!((*c).next.pixels));
} } }
#[export_name = "_ZN2wp5video14present_on_gpuEP6HWND__d"]
pub unsafe extern "C-unwind" fn present_on_gpu(window: *mut c_void, aspect: f64) {
    let mut context: PresentCtx = core::mem::zeroed();
    let c = addr_of_mut!(context); let _g = PresentGuard(c);
    wprt_vws_present_on_gpu(window as usize, aspect, c as *mut c_void);
    if !(*c).exception.is_null() { wprt_claude_vws_rethrow(addr_of_mut!((*c).exception)); }
}

#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_vws_title_body(op: c_int, info: *mut TitleInfo, buffer: *mut u8) -> c_int {
    let i = &mut *info;
    match op {
        0 => { let t = &*ui_text(); i.fps_label = t.fps; i.dsp_label = t.dsp; i.dsp_native = t.dsp_native; i.dsp_interpreted = t.dsp_interpreted; i.dsp_stopped = t.dsp_stopped; 0 }
        1 => { let s = dsp_status(); i.loaded = s.loaded; i.is_native = s.native; i.load_percent = s.load_percent; 0 }
        2 => { __mingw_snprintf(buffer as *mut c_char, 128, i.format, i.fps_label, i.fps, i.dsp_label, i.status_label, i.load_percent); 0 }
        3 => { __mingw_snprintf(buffer as *mut c_char, 128, i.format, i.fps_label, i.fps, i.dsp_label, i.status_label); 0 }
        4 => { i.title = (*game_description()).title; 0 }
        5 => { i.api = api_name(); 0 }
        _ => cxx_terminate(),
    }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_vws_screenshot_log_body(format: *const u8, path: *const u8) -> c_int {
    let f = format as *const c_char; let p = path as *const c_char;
    __mingw_fprintf(__acrt_iob_func(2), f, p); fputc(10, __acrt_iob_func(2)); log_write_s(cs!("video"), f, p); 0
}
unsafe fn frame_read(f: *mut Frame, i: &mut PresentInfo) -> c_int {
    let ok = read_frame(addr_of_mut!((*f).pixels), addr_of_mut!((*f).width), addr_of_mut!((*f).height));
    i.pixels = (*f).pixels.b as *const u32; i.count = v_len(addr_of_mut!((*f).pixels), 4); i.width = (*f).width; i.height = (*f).height; ok as c_int
}
unsafe fn frame_reset(f: *mut Frame) {
    let (ob, oc) = ((*f).pixels.b, (*f).pixels.c);
    (*f).pixels.b = null_mut(); (*f).pixels.e = null_mut(); (*f).pixels.c = null_mut(); (*f).width = 0; (*f).height = 0;
    if !ob.is_null() { cxx_delete(ob, oc as usize - ob as usize); }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_vws_present_body(raw: *mut c_void, op: c_int, info: *mut PresentInfo) -> c_int {
    let c = raw as *mut PresentCtx; let i = &mut *info; let next = addr_of_mut!(g_next_frame_path);
    match op {
        0 => { present_frame(i.window as *mut c_void, i.aspect); 0 }
        1 => wphw_video_screenshot_exchange() as c_int,
        2 => { wprt_claude_vws_save_screenshot(); 0 }
        3 => ((*next).n != 0) as c_int,
        4 => frame_read(addr_of_mut!((*c).next), i),
        5 => { save_png((*next).p as *const c_char, addr_of_mut!((*c).next.pixels), (*c).next.width, (*c).next.height); 0 }
        6 => { (*next).n = 0; *(*next).p = 0; 0 }
        7 => frame_read(addr_of_mut!((*c).pipe), i),
        8 => { log_write_uu(cs!("video"), cs!("frame pipe %u x %u"), i.width, i.height); 0 }
        9 => frame_read(addr_of_mut!((*c).save), i),
        10 => { __mingw_snprintf(i.path, 512, i.format, i.index); 0 }
        11 => { save_png(i.path, addr_of_mut!((*c).save.pixels), (*c).save.width, (*c).save.height); 0 }
        12 => { frame_reset(addr_of_mut!((*c).next)); 0 }
        13 => { frame_reset(addr_of_mut!((*c).pipe)); 0 }
        14 => { frame_reset(addr_of_mut!((*c).save)); 0 }
        _ => cxx_terminate(),
    }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_vws_shot_body(raw: *mut c_void, op: c_int, info: *mut ShotInfo) -> c_int {
    let c = raw as *mut ShotCtx; let i = &mut *info;
    match op {
        0 => { let ok = read_frame(addr_of_mut!((*c).pixels), addr_of_mut!((*c).width), addr_of_mut!((*c).height));
               i.pixels = (*c).pixels.b as *const u32; i.count = v_len(addr_of_mut!((*c).pixels), 4); i.width = (*c).width; i.height = (*c).height; ok as c_int }
        1 => { i.folder = cs!("screenshots"); 0 }
        2 => { i.prefix = (*game_description()).short_name; 0 }
        3 => { let s = addr_of_mut!((*c).path); cxx_s_replace(s, 0, (*s).n, i.path, i.path_len); 0 }
        4 => { wprt_claude_vws_shot_thread(raw, info); 0 }
        _ => cxx_terminate(),
    }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_vws_option_body(window: usize, op: c_int) -> c_int {
    match op { 0 => { refresh_menu(window as *mut c_void); 0 } _ => cxx_terminate() }
}

#[path="../../native_final_sdk/video_lifetime.rs"]mod lifetime;
