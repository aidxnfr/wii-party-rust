// Claude G60: whole-member Rust port of hybrid options_state_sdk.cpp (wp::options native boundary). Residue: static-init hook.
#![allow(non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, clippy::too_many_arguments)]
use core::ffi::{c_char, c_int, c_void};
include!("../common/cxx.rs");
type Sink = unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize);
type RowSink = unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize, *const u8, usize, *const u8, usize, bool, bool, bool);
type HintSink = unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize, *const u8, usize);
type Mover = unsafe extern "C-unwind" fn(*mut c_void);
#[repr(C)] struct Span { p: *const u8, n: usize }
extern "C-unwind" {
    fn wprt_options_live(p: *const u8, n: usize) -> bool;
    fn wprt_options_choices(p: *const u8, n: usize, s: Sink, c: *mut c_void);
    fn wprt_options_shown(k: *const u8, kn: usize, v: *const u8, vn: usize, s: Sink, c: *mut c_void);
    fn wprt_options_step(k: *const u8, kn: usize, v: *const u8, vn: usize, step: i32, s: Sink, c: *mut c_void);
    fn wprt_options_repeat(held: u32, now: i64, repeating: u32, delay: i64, interval: i64, previous: *mut u32, next: *mut i64) -> u32;
    fn wprt_options_set_open(open: bool, open_: *mut bool, hover: *mut c_void);
    fn wprt_options_set_hover(hover: *const c_void, hover_: *mut c_void) -> bool;
    fn wprt_options_select(index: usize, selection: *mut usize);
    fn wprt_options_rows(s: RowSink, c: *mut c_void);
    fn wprt_options_handle(action: i32, selection: *mut usize, open: *mut bool, s: Sink, c: *mut c_void);
    fn wprt_options_watch(spans: *const Span, n: usize, instruction: *const c_char, prefix: *const c_char, after: *mut bool) -> bool;
    fn wprt_options_hints(b: *const c_void, language: i32, s: HintSink, c: *mut c_void);
    fn wprt_options_overlay_show(visible: *mut bool, version: *mut u64, image: *mut c_void, mover: Mover);
    fn wprt_options_overlay_hide(visible: *mut bool, version: *mut u64);
    fn wprt_options_overlay_take(version: *const u64, visible: *const bool, out_version: *mut u64, out_visible: *mut bool, image: *mut c_void, taker: Mover) -> bool;
    #[link_name = "_ZN2wp6keymap6formatB5cxx11ERKSt6vectorIiSaIiEEPKc"] fn keymap_format(ret: *mut CxxString, codes: *const CxxVec, sep: *const c_char) -> *mut CxxString;
    #[link_name = "_ZSt20__throw_system_errori"] fn throw_system_error(e: c_int) -> !;
    fn pthread_mutex_init(m: *mut usize, a: *const c_void) -> c_int;
    fn pthread_mutex_destroy(m: *mut usize) -> c_int;
    fn pthread_mutex_lock(m: *mut usize) -> c_int;
    fn pthread_mutex_unlock(m: *mut usize) -> c_int;
}
extern "C" { fn atexit(f: unsafe extern "C" fn()) -> c_int; fn atoi(s: *const c_char) -> c_int; }
#[repr(C)] struct Image { pixels: CxxVec, width: u32, height: u32 }
const ROW: usize = 104; const HINT: usize = 64;
static mut G_OVERLAY_MUTEX: usize = 0;
static mut G_OVERLAY: Image = Image { pixels: CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() }, width: 0, height: 0 };
static mut G_OVERLAY_VISIBLE: bool = false;
static mut G_OVERLAY_VERSION: u64 = 0;
unsafe extern "C" fn destroy_mutex() { pthread_mutex_destroy(core::ptr::addr_of_mut!(G_OVERLAY_MUTEX)); }
unsafe extern "C" fn destroy_overlay() { v_free(core::ptr::addr_of_mut!(G_OVERLAY.pixels)); }
#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_options_state_static_init() {
    pthread_mutex_init(core::ptr::addr_of_mut!(G_OVERLAY_MUTEX), core::ptr::null());
    atexit(destroy_mutex);
    G_OVERLAY = Image { pixels: CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() }, width: 0, height: 0 };
    atexit(destroy_overlay);
}
struct StrGuard(*mut CxxString); impl Drop for StrGuard { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }
struct VecGuard(*mut CxxVec); impl Drop for VecGuard { fn drop(&mut self) { unsafe { v_destroy_strings(self.0) } } }
struct RowsGuard(*mut CxxVec); impl Drop for RowsGuard { fn drop(&mut self) { unsafe { destroy_rows(self.0) } } }
struct HintsGuard(*mut CxxVec); impl Drop for HintsGuard { fn drop(&mut self) { unsafe { destroy_hints(self.0) } } }
struct PodGuard(*mut CxxVec); impl Drop for PodGuard { fn drop(&mut self) { unsafe { v_free(self.0) } } }
struct StorageGuard(*mut u8, usize, bool); impl Drop for StorageGuard { fn drop(&mut self) { if self.2 { unsafe { cxx_delete(self.0, self.1) } } } }
struct Lock; impl Lock { unsafe fn new() -> Lock { let e = pthread_mutex_lock(core::ptr::addr_of_mut!(G_OVERLAY_MUTEX)); if e != 0 { throw_system_error(e); } Lock } }
impl Drop for Lock { fn drop(&mut self) { unsafe { pthread_mutex_unlock(core::ptr::addr_of_mut!(G_OVERLAY_MUTEX)); } } }
unsafe fn destroy_row(r: *mut u8) { s_dispose(r.add(64) as *mut CxxString); s_dispose(r.add(32) as *mut CxxString); s_dispose(r as *mut CxxString); }
unsafe fn destroy_rows(v: *mut CxxVec) { let n = v_len(v, ROW); for i in 0..n { destroy_row((*v).b.add(i * ROW)); } v_free(v); }
unsafe fn destroy_hints(v: *mut CxxVec) { let n = v_len(v, HINT); for i in 0..n { let h = (*v).b.add(i * HINT); s_dispose(h.add(32) as *mut CxxString); s_dispose(h as *mut CxxString); } v_free(v); }
unsafe fn copy_row(d: *mut u8, s: *const u8) {
    s_copy_construct(d as *mut CxxString, s as *const CxxString); let g0 = StrGuard(d as *mut CxxString);
    s_copy_construct(d.add(32) as *mut CxxString, s.add(32) as *const CxxString); let g1 = StrGuard(d.add(32) as *mut CxxString);
    s_copy_construct(d.add(64) as *mut CxxString, s.add(64) as *const CxxString);
    core::mem::forget(g1); core::mem::forget(g0);
    *d.add(96) = *s.add(96); *d.add(97) = *s.add(97); *d.add(98) = *s.add(98);
}
unsafe fn relocate_row(d: *mut u8, s: *mut u8) {
    for k in 0..3 { s_move_construct(d.add(32 * k) as *mut CxxString, s.add(32 * k) as *mut CxxString); }
    *d.add(96) = *s.add(96); *d.add(97) = *s.add(97); *d.add(98) = *s.add(98);
}
unsafe fn relocate_hint(d: *mut u8, s: *mut u8) { s_move_construct(d as *mut CxxString, s as *mut CxxString); s_move_construct(d.add(32) as *mut CxxString, s.add(32) as *mut CxxString); }
unsafe extern "C-unwind" fn assign_sink(c: *mut c_void, p: *const u8, n: usize) { s_assign(c as *mut CxxString, p, n); }
unsafe extern "C-unwind" fn choices_sink(c: *mut c_void, p: *const u8, n: usize) { v_emplace_string(c as *mut CxxVec, p, n); }
unsafe extern "C-unwind" fn rows_sink(c: *mut c_void, k: *const u8, kn: usize, l: *const u8, ln: usize, v: *const u8, vn: usize, live: bool, toggle: bool, on: bool) {
    let mut row = [0u64; ROW / 8]; let r = row.as_mut_ptr() as *mut u8;
    s_init(r as *mut CxxString); s_init(r.add(32) as *mut CxxString); s_init(r.add(64) as *mut CxxString); *r.add(96) = 0; *r.add(97) = 0; *r.add(98) = 0;
    struct RowGuard(*mut u8); impl Drop for RowGuard { fn drop(&mut self) { unsafe { destroy_row(self.0) } } }
    let _g = RowGuard(r);
    s_assign(r as *mut CxxString, k, kn); s_assign(r.add(32) as *mut CxxString, l, ln); s_assign(r.add(64) as *mut CxxString, v, vn);
    *r.add(96) = live as u8; *r.add(97) = toggle as u8; *r.add(98) = on as u8;
    let vec = c as *mut CxxVec;
    if (*vec).e != (*vec).c { copy_row((*vec).e, r); (*vec).e = (*vec).e.add(ROW); return; }
    let (new, len) = v_grow(vec, ROW, concat!("vector::_M_realloc_append", "\0").as_ptr() as _);
    let mut sg = StorageGuard(new, len * ROW, true);
    copy_row(new.add(v_len(vec, ROW) * ROW), r);
    sg.2 = false;
    v_finish(vec, ROW, new, len, Some(relocate_row));
}
unsafe extern "C-unwind" fn hint_sink(c: *mut c_void, n: *const u8, nn: usize, k: *const u8, kn: usize) {
    let mut tmp = [0u64; HINT / 8]; let t = tmp.as_mut_ptr() as *mut u8;
    s_construct(t as *mut CxxString, n, nn); let g0 = StrGuard(t as *mut CxxString);
    s_construct(t.add(32) as *mut CxxString, k, kn); let _g1 = StrGuard(t.add(32) as *mut CxxString);
    let vec = c as *mut CxxVec;
    if (*vec).e != (*vec).c { relocate_hint((*vec).e, t); (*vec).e = (*vec).e.add(HINT); }
    else {
        let (new, len) = v_grow(vec, HINT, concat!("vector::_M_realloc_append", "\0").as_ptr() as _);
        relocate_hint(new.add(v_len(vec, HINT) * HINT), t);
        v_finish(vec, HINT, new, len, Some(relocate_hint));
    }
    drop(_g1); drop(g0);
}
/// std::vector<uint32_t> move assignment (swap-through-temporary) + width/height.
unsafe fn image_move_assign(d: *mut Image, s: *mut Image) {
    let old = core::ptr::read(&(*d).pixels);
    (*d).pixels = core::ptr::read(&(*s).pixels);
    (*s).pixels = CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() };
    let mut old = old; v_free(&mut old);
    (*d).width = (*s).width; (*d).height = (*s).height;
}
unsafe extern "C-unwind" fn show_mover(c: *mut c_void) { image_move_assign(core::ptr::addr_of_mut!(G_OVERLAY), c as *mut Image); }
unsafe extern "C-unwind" fn take_mover(c: *mut c_void) {
    image_move_assign(c as *mut Image, core::ptr::addr_of_mut!(G_OVERLAY));
    let mut empty = Image { pixels: CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() }, width: 0, height: 0 };
    image_move_assign(core::ptr::addr_of_mut!(G_OVERLAY), &mut empty);
    v_free(&mut empty.pixels);
}

#[export_name = "_ZN2wp7options4liveERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn live(key: *const CxxString) -> bool { wprt_options_live((*key).p, (*key).n) }
#[export_name = "_ZN2wp7options7choicesERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn choices(ret: *mut CxxVec, key: *const CxxString) -> *mut CxxVec { v_init(ret); let g = VecGuard(ret); wprt_options_choices((*key).p, (*key).n, choices_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp7options11shown_valueERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_"]
pub unsafe extern "C-unwind" fn shown_value(ret: *mut CxxString, key: *const CxxString, value: *const CxxString) -> *mut CxxString { s_init(ret); let g = StrGuard(ret); wprt_options_shown((*key).p, (*key).n, (*value).p, (*value).n, assign_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp7options10step_valueERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_i"]
pub unsafe extern "C-unwind" fn step_value(ret: *mut CxxString, key: *const CxxString, value: *const CxxString, step: c_int) -> *mut CxxString { s_init(ret); let g = StrGuard(ret); wprt_options_step((*key).p, (*key).n, (*value).p, (*value).n, step, assign_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp7options4Menu8set_openEb"]
pub unsafe extern "C-unwind" fn menu_set_open(this: *mut u8, open: bool) { wprt_options_set_open(open, this as *mut bool, this.add(16) as *mut c_void) }
#[export_name = "_ZN2wp7options4Menu9set_hoverERKNS0_5HoverE"]
pub unsafe extern "C-unwind" fn menu_set_hover(this: *mut u8, hover: *const c_void) -> bool { wprt_options_set_hover(hover, this.add(16) as *mut c_void) }
#[export_name = "_ZN2wp7options4Menu6selectEy"]
pub unsafe extern "C-unwind" fn menu_select(this: *mut u8, index: usize) { wprt_options_select(index, this.add(8) as *mut usize) }
#[export_name = "_ZNK2wp7options4Menu4rowsEv"]
pub unsafe extern "C-unwind" fn menu_rows(ret: *mut CxxVec, _this: *const u8) -> *mut CxxVec { v_init(ret); let g = RowsGuard(ret); wprt_options_rows(rows_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp7options4Menu6handleB5cxx11ENS0_6ActionE"]
pub unsafe extern "C-unwind" fn menu_handle(ret: *mut CxxString, this: *mut u8, action: i32) -> *mut CxxString { s_init(ret); let g = StrGuard(ret); wprt_options_handle(action, this.add(8) as *mut usize, this as *mut bool, assign_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp7options6Repeat6updateEjx"]
pub unsafe extern "C-unwind" fn repeat_update(this: *mut u8, held: u32, now: i64) -> u32 { wprt_options_repeat(held, now, *(this as *const u32), *(this.add(8) as *const i64), *(this.add(16) as *const i64), this.add(24) as *mut u32, this.add(32) as *mut i64) }
#[export_name = "_ZN2wp7options9hint_rowsERKNS_6keymap8BindingsENS_2ui8LanguageE"]
pub unsafe extern "C-unwind" fn hint_rows(ret: *mut CxxVec, bindings: *const c_void, language: i32) -> *mut CxxVec { v_init(ret); let g = HintsGuard(ret); wprt_options_hints(bindings, language, hint_sink, ret as *mut c_void); core::mem::forget(g); ret }
#[export_name = "_ZN2wp7options16InstructionWatch6updateERKSt6vectorINSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEESaIS8_EEPKcSE_"]
pub unsafe extern "C-unwind" fn watch_update(this: *mut u8, modules: *const CxxVec, instruction: *const c_char, prefix: *const c_char) -> bool {
    let mut spans = CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() };
    let _g = PodGuard(&mut spans);
    let n = v_len(modules, 32);
    for i in 0..n { let s = ((*modules).b as *const CxxString).add(i); let span = Span { p: (*s).p, n: (*s).n }; v_push_pod(&mut spans, 16, &span as *const Span as *const u8); }
    wprt_options_watch(spans.b as *const Span, v_len(&spans, 16), instruction, prefix, this as *mut bool)
}
#[export_name = "_ZN2wp7options12show_overlayENS0_5ImageE"]
pub unsafe extern "C-unwind" fn show_overlay(image: *mut c_void) { let _l = Lock::new(); wprt_options_overlay_show(core::ptr::addr_of_mut!(G_OVERLAY_VISIBLE), core::ptr::addr_of_mut!(G_OVERLAY_VERSION), image, show_mover); }
#[export_name = "_ZN2wp7options12hide_overlayEv"]
pub unsafe extern "C-unwind" fn hide_overlay() { let _l = Lock::new(); wprt_options_overlay_hide(core::ptr::addr_of_mut!(G_OVERLAY_VISIBLE), core::ptr::addr_of_mut!(G_OVERLAY_VERSION)); }
#[export_name = "_ZN2wp7options12take_overlayERyRNS0_5ImageERb"]
pub unsafe extern "C-unwind" fn take_overlay(version: *mut u64, image: *mut c_void, visible: *mut bool) -> bool { let _l = Lock::new(); wprt_options_overlay_take(core::ptr::addr_of!(G_OVERLAY_VERSION), core::ptr::addr_of!(G_OVERLAY_VISIBLE), version, visible, image, take_mover) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_options_atoi(p: *const u8, n: usize) -> i32 { let mut s = core::mem::MaybeUninit::<CxxString>::uninit(); s_construct(s.as_mut_ptr(), p, n); let _g = StrGuard(s.as_mut_ptr()); atoi((*s.as_ptr()).p as *const c_char) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_options_hint_keys(b: *const c_void, action: i32, sink: Sink, c: *mut c_void) {
    let mut value = core::mem::MaybeUninit::<CxxString>::uninit();
    keymap_format(value.as_mut_ptr(), (b as *const CxxVec).offset(action as isize), b"/\0".as_ptr() as *const c_char);
    let _g = StrGuard(value.as_mut_ptr());
    sink(c, (*value.as_ptr()).p, (*value.as_ptr()).n);
}
