// SPDX-License-Identifier: GPL-3.0-or-later
// Claude G60: options_render_sdk.cpp.obj = hybrid/review/bevy-options-text-fix-port/options_render_sdk.cpp (-std=c++20 -O2) -> Rust:
// render / render_hint / render_titlebar, make_face, decode (u32string push_back growth as inlined by GCC 15.2), the stb text
// callbacks and the render_ops table. residue.cpp keeps Font loading (std::filesystem, catch(...)/rethrow, static Font guards).
// Layouts (arel-pristine wp/options.h, wp/ui_text.h) are asserted by the harness: Row 104 {key,label@32,value@64,live@96,toggle,on},
// HintRow 64 {name,keys@32}, Menu {open,selection@8,hover{row@16,part@20}}, Layout 40, Rect 16, Image 32 {pixels,width@24,height@28},
// Text.menu_title@40 menu_hint@48 menu_restart@56.
#![allow(dead_code, non_upper_case_globals, clippy::missing_safety_doc, clippy::too_many_arguments)]
use core::ffi::{c_char, c_int, c_long, c_void};
include!("../common/cxx.rs");

const ROW: usize = 104; const HINT: usize = 64;
#[repr(C)] #[derive(Clone, Copy)] struct OptBytes { p: *const u8, n: usize }
#[repr(C)] #[derive(Clone, Copy)] struct OptRect { x: c_int, y: c_int, w: c_int, h: c_int }
#[repr(C)] struct OptFrame { unit: c_int, w: c_int, h: c_int, padding: c_int, title: c_int, row: c_int, footer: c_int, rows: usize }
#[repr(C)] #[derive(Clone, Copy)] struct OptRow { label: OptBytes, value: OptBytes, live: bool, toggle: bool, on: bool }
#[repr(C)] #[derive(Clone, Copy)] struct OptHint { name: OptBytes, keys: OptBytes }
#[repr(C)] struct OptStrings { title: OptBytes, restart: OptBytes, hint: OptBytes, hover_row: c_int, hover_part: c_int }
#[repr(C)] struct OptOps {
    face: unsafe extern "C-unwind" fn(*mut c_void, c_int, bool, c_int),
    text: unsafe extern "C-unwind" fn(*mut c_void, *mut u32, c_int, c_int, c_int, u32, OptBytes, c_int, c_int, c_int, c_int, c_int, bool),
    width: unsafe extern "C-unwind" fn(*mut c_void, c_int, OptBytes) -> f32,
    image: unsafe extern "C-unwind" fn(*mut c_void, u32, u32, *const u32, usize),
    strings: unsafe extern "C-unwind" fn(*mut c_void, *mut OptStrings),
    selected: unsafe extern "C-unwind" fn(*mut c_void) -> usize,
    rect: unsafe extern "C-unwind" fn(*mut c_void, bool, usize, *mut OptRect),
}
unsafe impl Sync for OptOps {}
#[repr(C)] #[derive(Clone, Copy)] struct Face { font: *const c_void, scale: f32, ascent: c_int, descent: c_int }
#[repr(C)] #[derive(Clone, Copy)] struct Layout { unit: c_int, width: c_int, height: c_int, padding: c_int, title_height: c_int, row_height: c_int, footer_height: c_int, rows: usize }
#[repr(C)] #[derive(Clone, Copy)] struct Rect { x: c_int, y: c_int, width: c_int, height: c_int }
#[repr(C)] pub struct Image { pixels: CxxVec, width: u32, height: u32 }
#[repr(C)] struct RenderContext { menu: *const u8, layout: *const Layout, output: *mut Image, faces: [Face; 3], strings: *const u8 }
type Metrics = unsafe extern "C-unwind" fn(*const c_void, u32, u32, *mut c_int, *mut c_int);
type BoxFn = unsafe extern "C-unwind" fn(*const c_void, c_int, f32, *mut c_int);
type Raster = unsafe extern "C-unwind" fn(*const c_void, c_int, f32, *mut u8, c_int, c_int);
type TitleText = unsafe extern "C-unwind" fn(*mut c_void, *mut u32, c_int, c_int, c_int, u32, c_int, c_int, c_int);
type Push = unsafe extern "C-unwind" fn(*mut c_void, *const u32, usize);
extern "C-unwind" {
    fn wprt_hw_stb_em_scale(i: *mut c_void, h: f32) -> f32;
    fn wprt_hw_stb_vmetrics(i: *mut c_void, a: *mut c_int, d: *mut c_int, g: *mut c_int);
    fn wprt_hw_stb_hmetrics(i: *mut c_void, c: c_int, w: *mut c_int, b: *mut c_int);
    fn wprt_hw_stb_box(i: *mut c_void, c: c_int, sx: f32, sy: f32, shx: f32, shy: f32, x0: *mut c_int, y0: *mut c_int, x1: *mut c_int, y1: *mut c_int);
    fn wprt_hw_stb_bitmap(i: *mut c_void, p: *mut u8, w: c_int, h: c_int, stride: c_int, sx: f32, sy: f32, shx: f32, shy: f32, c: c_int);
    fn wprt_canvas_decode(p: *const u8, n: usize, f: Push, c: *mut c_void);
    fn wprt_options_text_width(face: *const c_void, scale: f32, p: *const u32, n: usize, m: Metrics) -> f32;
    fn wprt_options_canvas_text(px: *mut u32, w: c_int, h: c_int, face: *const c_void, scale: f32, ascent: c_int, descent: c_int, ok: bool, color: u32,
        p: *const u8, n: usize, l: c_int, t: c_int, r: c_int, b: c_int, align: c_int, ellipsis: bool, m: Metrics, bx: BoxFn, ra: Raster);
    fn wprt_options_titlebar(px: *mut u32, w: c_int, h: c_int, hover: c_int, pressed: c_int, maximized: bool, focused: bool, f: TitleText, c: *mut c_void);
    fn wprt_options_render(c: *mut c_void, ops: *const OptOps, f: *const OptFrame, rows: *const OptRow, n: usize);
    fn wprt_options_face_metrics(scale: f32, a: c_int, d: c_int, r: unsafe extern "C-unwind" fn(f32) -> c_int, out: *mut c_int);
    fn wprt_options_hint(c: *mut c_void, ops: *const OptOps, rows: *const OptHint, n: usize, title: OptBytes, w: c_int, h: c_int);
    fn wphw_options_kern(i: *mut c_void, a: u32, b: u32) -> c_int;
    fn wphw_options_render_gate(w: c_int, h: c_int) -> bool;
    fn wphw_options_hint_gate(empty: bool, w: c_int, h: c_int) -> bool;
    fn wphw_options_titlebar_gate(w: c_int, h: c_int) -> bool;
    fn wphw_options_regular_font_ok() -> bool;
    fn wprt_claude_or_font(bold: bool) -> *const c_void;
    fn wprt_claude_or_font_info(f: *const c_void) -> *mut c_void;
    fn wprt_claude_or_font_ok(f: *const c_void) -> bool;
    #[link_name = "_ZNK2wp7options4Menu4rowsEv"] fn menu_rows(ret: *mut CxxVec, this: *const u8) -> *mut CxxVec;
    #[link_name = "_ZN2wp7options6layoutEiy"] fn layout(h: c_int, rows: usize) -> Layout;
    #[link_name = "_ZN2wp7options12control_rectERKNS0_6LayoutEy"] fn control_rect(l: *const Layout, row: usize) -> Rect;
    #[link_name = "_ZN2wp7options10close_rectERKNS0_6LayoutE"] fn close_rect(l: *const Layout) -> Rect;
    #[link_name = "_ZN2wp2ui4textEv"] fn ui_text() -> *const u8;
}
extern "C" { fn strlen(s: *const c_char) -> usize; fn lroundf(v: f32) -> c_long; }

// ---- std::u32string (SSO capacity 3) ----
#[inline] unsafe fn u_local(s: *mut CxxString) -> *mut u32 { core::ptr::addr_of_mut!((*s).u) as *mut u32 }
unsafe fn u_init(s: *mut CxxString) { (*s).p = u_local(s) as *mut u8; (*s).n = 0; *u_local(s) = 0; }
unsafe fn u_dispose(s: *mut CxxString) { if (*s).p != u_local(s) as *mut u8 { cxx_delete((*s).p, ((*s).u[0] as usize + 1) * 4); } }
/// basic_string<char32_t>::push_back with _M_mutate/_M_create inlined (GCC 15.2 -O2): local->heap at size 4 allocates (6+1)*4,
/// heap growth max(2*cap, size+1) capped at max_size 0x1FFFFFFFFFFFFFFE; allocate, copy size chars, sized-dispose old.
unsafe fn u_push(s: *mut CxxString, c: u32) {
    const MAX: usize = 0x1FFF_FFFF_FFFF_FFFE;
    let size = (*s).n; let newsize = size + 1;
    if (*s).p == u_local(s) as *mut u8 {
        if newsize == 4 { let q = cxx_new(0x1c); core::ptr::copy_nonoverlapping(u_local(s) as *const u8, q, size * 4); (*s).p = q; (*s).u[0] = 6; }
    } else {
        let cap = (*s).u[0] as usize;
        if cap < newsize {
            if newsize == MAX + 1 { cxx_throw_length_error(concat!("basic_string::_M_create", "\0").as_ptr() as _); }
            let newcap = if newsize < cap * 2 { if MAX >= cap * 2 { cap * 2 } else { MAX } } else { newsize };
            let q = cxx_new((newcap + 1) * 4);
            core::ptr::copy_nonoverlapping((*s).p, q, size * 4);
            cxx_delete((*s).p, (cap + 1) * 4);
            (*s).p = q; (*s).u[0] = newcap as u64;
        }
    }
    let p = (*s).p as *mut u32; *p.add(size) = c; (*s).n = newsize; *p.add(newsize) = 0;
}
struct U32Guard(*mut CxxString); impl Drop for U32Guard { fn drop(&mut self) { unsafe { u_dispose(self.0) } } }
struct StrGuard(*mut CxxString); impl Drop for StrGuard { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }
struct PodGuard(*mut CxxVec); impl Drop for PodGuard { fn drop(&mut self) { unsafe { v_free(self.0) } } }
unsafe fn destroy_rows(v: *mut CxxVec) { let n = v_len(v, ROW); for i in 0..n { let r = (*v).b.add(i * ROW); s_dispose(r.add(64) as *mut CxxString); s_dispose(r.add(32) as *mut CxxString); s_dispose(r as *mut CxxString); } v_free(v); }
struct RowsGuard(*mut CxxVec); impl Drop for RowsGuard { fn drop(&mut self) { unsafe { destroy_rows(self.0) } } }
struct ImageGuard(*mut Image); impl Drop for ImageGuard { fn drop(&mut self) { unsafe { v_free(&mut (*self.0).pixels) } } }

unsafe extern "C-unwind" fn decode_push(c: *mut c_void, p: *const u32, n: usize) { let out = c as *mut CxxString; for i in 0..n { u_push(out, *p.add(i)); } }
unsafe fn decode(ret: *mut CxxString, text: *const CxxString) { u_init(ret); let g = U32Guard(ret); wprt_canvas_decode((*text).p, (*text).n, decode_push, ret as *mut c_void); core::mem::forget(g); }
unsafe extern "C-unwind" fn round_value(v: f32) -> c_int { lroundf(v) as c_int }
unsafe fn make_face(bold: bool, height: c_int) -> Face {
    let chosen = wprt_claude_or_font(bold);
    let mut face = Face { font: chosen, scale: wprt_hw_stb_em_scale(wprt_claude_or_font_info(chosen), height as f32), ascent: 0, descent: 0 };
    let (mut ascent, mut descent, mut gap) = (0, 0, 0);
    wprt_hw_stb_vmetrics(wprt_claude_or_font_info(chosen), &mut ascent, &mut descent, &mut gap);
    let mut metrics = [0 as c_int; 2];
    wprt_options_face_metrics(face.scale, ascent, descent, round_value, metrics.as_mut_ptr());
    face.ascent = metrics[0]; face.descent = metrics[1];
    face
}
unsafe extern "C-unwind" fn text_metrics(raw: *const c_void, code: u32, next: u32, width: *mut c_int, kern: *mut c_int) {
    let face = &*(raw as *const Face); let mut bearing = 0;
    wprt_hw_stb_hmetrics(wprt_claude_or_font_info(face.font), code as c_int, width, &mut bearing);
    *kern = wphw_options_kern(wprt_claude_or_font_info(face.font), code, next);
}
unsafe extern "C-unwind" fn text_box(raw: *const c_void, code: c_int, shift: f32, b: *mut c_int) {
    let face = &*(raw as *const Face);
    wprt_hw_stb_box(wprt_claude_or_font_info(face.font), code, face.scale, face.scale, shift, 0.0, b, b.add(1), b.add(2), b.add(3));
}
unsafe extern "C-unwind" fn text_raster(raw: *const c_void, code: c_int, shift: f32, pixels: *mut u8, width: c_int, height: c_int) {
    let face = &*(raw as *const Face);
    wprt_hw_stb_bitmap(wprt_claude_or_font_info(face.font), pixels, width, height, width, face.scale, face.scale, shift, 0.0, code);
}
unsafe fn text_width(face: *const Face, text: *const CxxString) -> f32 { wprt_options_text_width(face as *const c_void, (*face).scale, (*text).p as *const u32, (*text).n, text_metrics) }
#[inline] unsafe fn str_bytes(s: *const u8) -> OptBytes { let s = s as *const CxxString; OptBytes { p: (*s).p, n: (*s).n } }
#[inline] unsafe fn cstr_bytes(s: *const u8) -> OptBytes { OptBytes { p: s, n: strlen(s as *const c_char) } }
/// vector<uint32_t>::assign(first,last) forward-iterator path (_M_assign_aux).
unsafe fn u32_assign(v: *mut CxxVec, p: *const u32, n: usize) {
    let cap = ((*v).c as usize - (*v).b as usize) / 4; let size = v_len(v, 4);
    if n > cap {
        if n > 0x1FFF_FFFF_FFFF_FFFF { cxx_throw_length_error(concat!("cannot create std::vector larger than max_size()", "\0").as_ptr() as _); }
        let q = cxx_new(n * 4); core::ptr::copy_nonoverlapping(p as *const u8, q, n * 4);
        if !(*v).b.is_null() { cxx_delete((*v).b, cap * 4); }
        (*v).b = q; (*v).e = q.add(n * 4); (*v).c = (*v).e;
    } else if size >= n { core::ptr::copy(p as *const u8, (*v).b, n * 4); (*v).e = (*v).b.add(n * 4); }
    else { core::ptr::copy(p as *const u8, (*v).b, size * 4); core::ptr::copy((p as *const u8).add(size * 4), (*v).e, (n - size) * 4); (*v).e = (*v).b.add(n * 4); }
}
/// vector<uint32_t>::resize(n) from the current size: _M_default_append (value-initialised) or erase at end.
unsafe fn u32_resize(v: *mut CxxVec, n: usize) {
    let size = v_len(v, 4);
    if n > size {
        let add = n - size; let navail = ((*v).c as usize - (*v).e as usize) / 4;
        if navail >= add { core::ptr::write_bytes((*v).e, 0, add * 4); (*v).e = (*v).e.add(add * 4); return; }
        const MAX: usize = 0x1FFF_FFFF_FFFF_FFFF;
        if MAX - size < add { cxx_throw_length_error(concat!("vector::_M_default_append", "\0").as_ptr() as _); }
        let mut len = size + core::cmp::max(size, add); if len < size || len > MAX { len = MAX; }
        let q = cxx_new(len * 4);
        core::ptr::write_bytes(q.add(size * 4), 0, add * 4);
        if size != 0 { core::ptr::copy_nonoverlapping((*v).b, q, size * 4); }
        if !(*v).b.is_null() { cxx_delete((*v).b, ((*v).c as usize - (*v).b as usize)); }
        (*v).b = q; (*v).e = q.add(n * 4); (*v).c = q.add(len * 4);
    } else if n < size { (*v).e = (*v).b.add(n * 4); }
}

unsafe extern "C-unwind" fn op_face(c: *mut c_void, slot: c_int, bold: bool, height: c_int) {
    let f = make_face(bold, height); let ctx = c as *mut RenderContext;
    (core::ptr::addr_of_mut!((*ctx).faces) as *mut Face).offset(slot as isize).write(f);
}
unsafe extern "C-unwind" fn op_text(c: *mut c_void, p: *mut u32, w: c_int, h: c_int, slot: c_int, color: u32, value: OptBytes, l: c_int, t: c_int, r: c_int, b: c_int, align: c_int, ellipsis: bool) {
    let ctx = c as *mut RenderContext; let face = (core::ptr::addr_of!((*ctx).faces) as *const Face).offset(slot as isize);
    wprt_options_canvas_text(p, w, h, face as *const c_void, (*face).scale, (*face).ascent, (*face).descent, wprt_claude_or_font_ok((*face).font), color,
        value.p, value.n, l, t, r, b, align, ellipsis, text_metrics, text_box, text_raster);
}
unsafe extern "C-unwind" fn op_width(c: *mut c_void, slot: c_int, value: OptBytes) -> f32 {
    let ctx = c as *mut RenderContext; let face = (core::ptr::addr_of!((*ctx).faces) as *const Face).offset(slot as isize);
    let mut tmp = core::mem::MaybeUninit::<CxxString>::uninit(); let tp = tmp.as_mut_ptr();
    s_construct(tp, value.p, value.n); let _g1 = StrGuard(tp);
    let mut u = core::mem::MaybeUninit::<CxxString>::uninit(); let up = u.as_mut_ptr();
    decode(up, tp); let _g2 = U32Guard(up);
    text_width(face, up)
}
unsafe extern "C-unwind" fn op_image(c: *mut c_void, w: u32, h: u32, p: *const u32, n: usize) {
    let im = (*(c as *mut RenderContext)).output; (*im).width = w; (*im).height = h; u32_assign(&mut (*im).pixels, p, n);
}
unsafe extern "C-unwind" fn op_strings(c: *mut c_void, out: *mut OptStrings) {
    let ctx = c as *mut RenderContext; (*ctx).strings = ui_text(); let menu = (*ctx).menu; let s = (*ctx).strings;
    let row = *(menu.add(16) as *const c_int); let part = *(menu.add(20) as *const c_int);
    *out = OptStrings { title: cstr_bytes(*(s.add(40) as *const *const u8)), restart: cstr_bytes(*(s.add(56) as *const *const u8)), hint: cstr_bytes(*(s.add(48) as *const *const u8)), hover_row: row, hover_part: part };
}
unsafe extern "C-unwind" fn op_selected(c: *mut c_void) -> usize { *((*(c as *mut RenderContext)).menu.add(8) as *const usize) }
unsafe extern "C-unwind" fn op_rect(c: *mut c_void, close: bool, row: usize, out: *mut OptRect) {
    let f = (*(c as *mut RenderContext)).layout; let r = if close { close_rect(f) } else { control_rect(f, row) };
    *out = OptRect { x: r.x, y: r.y, w: r.width, h: r.height };
}
static RENDER_OPS: OptOps = OptOps { face: op_face, text: op_text, width: op_width, image: op_image, strings: op_strings, selected: op_selected, rect: op_rect };
const ZERO_FACE: Face = Face { font: core::ptr::null(), scale: 0.0, ascent: 0, descent: 0 };
unsafe fn image_init(ret: *mut Image) { v_init(&mut (*ret).pixels); (*ret).width = 0; (*ret).height = 0; }
unsafe fn reserve(v: *mut CxxVec, n: usize, size: usize) { v_init(v); if n != 0 { let q = cxx_new(n * size); (*v).b = q; (*v).e = q; (*v).c = q.add(n * size); } }
unsafe fn push<T>(v: *mut CxxVec, item: T) { ((*v).e as *mut T).write(item); (*v).e = (*v).e.add(core::mem::size_of::<T>()); }

#[export_name = "_ZN2wp7options6renderERKNS0_4MenuEi"]
pub unsafe extern "C-unwind" fn render(ret: *mut Image, menu: *const u8, client_height: c_int) -> *mut Image {
    image_init(ret); let ig = ImageGuard(ret);
    let mut rows = core::mem::MaybeUninit::<CxxVec>::uninit(); let rp = rows.as_mut_ptr();
    menu_rows(rp, menu); let _rg = RowsGuard(rp);
    let n = v_len(rp, ROW);
    let frame = layout(client_height, n);
    if !wphw_options_render_gate(frame.width, frame.height) { core::mem::forget(ig); return ret; }
    let mut values = core::mem::MaybeUninit::<CxxVec>::uninit(); let vp = values.as_mut_ptr();
    reserve(vp, n, core::mem::size_of::<OptRow>()); let _vg = PodGuard(vp);
    for i in 0..n { let r = (*rp).b.add(i * ROW); push(vp, OptRow { label: str_bytes(r.add(32)), value: str_bytes(r.add(64)), live: *r.add(96) != 0, toggle: *r.add(97) != 0, on: *r.add(98) != 0 }); }
    let f = OptFrame { unit: frame.unit, w: frame.width, h: frame.height, padding: frame.padding, title: frame.title_height, row: frame.row_height, footer: frame.footer_height, rows: frame.rows };
    let mut context = RenderContext { menu, layout: &frame, output: ret, faces: [ZERO_FACE; 3], strings: core::ptr::null() };
    wprt_options_render(&mut context as *mut RenderContext as *mut c_void, &RENDER_OPS, &f, (*vp).b as *const OptRow, n);
    core::mem::forget(ig); ret
}
#[export_name = "_ZN2wp7options11render_hintERKSt6vectorINS0_7HintRowESaIS2_EERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEii"]
pub unsafe extern "C-unwind" fn render_hint(ret: *mut Image, rows: *const CxxVec, title: *const CxxString, client_width: c_int, client_height: c_int) -> *mut Image {
    image_init(ret); let ig = ImageGuard(ret);
    if !wphw_options_hint_gate((*rows).b == (*rows).e, client_width, client_height) { core::mem::forget(ig); return ret; }
    let n = v_len(rows, HINT);
    let mut values = core::mem::MaybeUninit::<CxxVec>::uninit(); let vp = values.as_mut_ptr();
    reserve(vp, n, core::mem::size_of::<OptHint>()); let _vg = PodGuard(vp);
    for i in 0..n { let r = (*rows).b.add(i * HINT); push(vp, OptHint { name: str_bytes(r), keys: str_bytes(r.add(32)) }); }
    let mut context = RenderContext { menu: core::ptr::null(), layout: core::ptr::null(), output: ret, faces: [ZERO_FACE; 3], strings: core::ptr::null() };
    wprt_options_hint(&mut context as *mut RenderContext as *mut c_void, &RENDER_OPS, (*vp).b as *const OptHint, n, OptBytes { p: (*title).p, n: (*title).n }, client_width, client_height);
    core::mem::forget(ig); ret
}
unsafe extern "C-unwind" fn title_text(c: *mut c_void, p: *mut u32, w: c_int, h: c_int, font_height: c_int, color: u32, left: c_int, right: c_int, bottom: c_int) {
    if wphw_options_regular_font_ok() {
        let face = make_face(false, font_height);
        let title = c as *const CxxString;
        wprt_options_canvas_text(p, w, h, &face as *const Face as *const c_void, face.scale, face.ascent, face.descent, wprt_claude_or_font_ok(face.font), color,
            (*title).p, (*title).n, left, 0, right, bottom, 0, true, text_metrics, text_box, text_raster);
    }
}
#[export_name = "_ZN2wp7options15render_titlebarEiiRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEiibb"]
pub unsafe extern "C-unwind" fn render_titlebar(ret: *mut Image, width: c_int, height: c_int, title: *const CxxString, hover: c_int, pressed: c_int, maximized: bool, focused: bool) -> *mut Image {
    image_init(ret); let ig = ImageGuard(ret);
    if !wphw_options_titlebar_gate(width, height) { core::mem::forget(ig); return ret; }
    (*ret).width = width as u32; (*ret).height = height as u32;
    u32_resize(&mut (*ret).pixels, (width as isize as usize).wrapping_mul(height as isize as usize));
    wprt_options_titlebar((*ret).pixels.b as *mut u32, width, height, hover, pressed, maximized, focused, title_text, title as *mut c_void);
    core::mem::forget(ig); ret
}
