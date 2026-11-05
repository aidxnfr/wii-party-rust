// AI-assisted (Claude, G60) options_window_sdk.cpp -> Rust: whole member, no C++ residue.
// g_menu / g_instruction are constant-initialized trivially-destructible PODs (Rust statics with compiler-derived layout);
// poll_gamepads' `static Repeat` keeps GCC's guarded dynamic initialization. All std::string / std::vector / Image
// temporaries are built in place with exact libstdc++ semantics and destroyed in the original order (Drop guards on unwind).
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("options_window_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
#[allow(dead_code)] mod layout { include!("layout.rs"); }
use stdabi::*; use layout::*;
use std::mem::MaybeUninit;
const _: () = assert!(MENU_ALIGN == 8 && MENU_SIZE == 24 && WATCH_SIZE == 1 && REPEAT_SIZE == 40 && HINTROW_SIZE == 64);
#[repr(C, align(8))] struct Store<const N: usize>([u8; N]);
const fn menu_init() -> [u8; MENU_SIZE] { let mut a = [0u8; MENU_SIZE]; let mut i = 0; while i < 4 { a[MENU_HOVER_ROW + i] = 0xFF; i += 1; } a }
static mut G_MENU: Store<MENU_SIZE> = Store(menu_init());
static mut G_INSTRUCTION: Store<WATCH_SIZE> = Store([0u8; WATCH_SIZE]);
static mut REPEAT_GUARD: u64 = 0;
static mut REPEAT: Store<REPEAT_SIZE> = Store([0u8; REPEAT_SIZE]);
#[inline] unsafe fn menu() -> *mut u8 { std::ptr::addr_of_mut!(G_MENU).cast() }
#[repr(C)] pub struct Image { pixels: RawVec, width: u32, height: u32 }
type Sink = unsafe extern "C-unwind" fn(*mut u8, *const u8, usize);
extern "C" { fn strlen(s: *const std::ffi::c_char) -> usize; }
extern "C-unwind" {
    fn wprt_ow_client_height() -> i32;
    fn wprt_ow_act(w: *mut u8, a: i32, s: Sink, c: *mut u8);
    fn wprt_ow_hint_language() -> i32;
    fn wprt_ow_key(w: *mut u8, k: u32, r: bool, s: Sink, c: *mut u8);
    fn wprt_ow_wheel(w: *mut u8, d: i32, s: Sink, c: *mut u8);
    fn wprt_ow_click(w: *mut u8, x: i32, y: i32, r: bool, s: Sink, c: *mut u8);
    fn wprt_ow_poll(w: *mut u8, repeat: *mut u8, s: Sink, c: *mut u8);
    #[link_name = "_ZN2wp7options4Menu6handleB5cxx11ENS0_6ActionE"] fn menu_handle(ret: *mut CxxString, this: *mut u8, a: i32) -> *mut CxxString;
    #[link_name = "_ZN2wp7options4Menu8set_openEb"] fn menu_set_open(this: *mut u8, v: bool);
    #[link_name = "_ZN2wp7options4Menu6selectEy"] fn menu_select(this: *mut u8, row: usize);
    #[link_name = "_ZN2wp7options4Menu9set_hoverERKNS0_5HoverE"] fn menu_set_hover(this: *mut u8, h: *const u8) -> bool;
    #[link_name = "_ZN2wp7options6Repeat6updateEjx"] fn repeat_update(this: *mut u8, held: u32, now: i64) -> u32;
    #[link_name = "_ZN2wp7options16InstructionWatch6updateERKSt6vectorINSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEESaIS8_EEPKcSE_"]
    fn watch_update(this: *mut u8, modules: *const RawVec, instruction: *const u8, prefix: *const u8) -> bool;
    #[link_name = "_ZN2wp4game11descriptionEv"] fn game_description() -> *const u8;
    #[link_name = "_ZN2wp19loaded_module_namesB5cxx11Ev"] fn loaded_module_names(ret: *mut RawVec) -> *mut RawVec;
    #[link_name = "_ZN2wp8settings4textB5cxx11EPKcS2_"] fn settings_text(ret: *mut CxxString, key: *const u8, var: *const u8) -> *mut CxxString;
    #[link_name = "_ZN2wp8settings4keysB5cxx11Ev"] fn settings_keys(ret: *mut RawVec) -> *mut RawVec;
    #[link_name = "_ZN2wp2ui11language_ofERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn language_of(s: *const CxxString) -> i32;
    #[link_name = "_ZN2wp2ui5labelEPKcNS0_8LanguageE"] fn ui_label(key: *const u8, l: i32) -> *const u8;
    #[link_name = "_ZN2wp5input8bindingsEv"] fn input_bindings() -> *const u8;
    #[link_name = "_ZN2wp7options9hint_rowsERKNS_6keymap8BindingsENS_2ui8LanguageE"] fn hint_rows(ret: *mut RawVec, b: *const u8, l: i32) -> *mut RawVec;
    #[link_name = "_ZN2wp7options11render_hintERKSt6vectorINS0_7HintRowESaIS2_EERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEii"]
    fn render_hint(ret: *mut Image, rows: *const RawVec, title: *const CxxString, w: i32, h: i32) -> *mut Image;
    #[link_name = "_ZN2wp7options6renderERKNS0_4MenuEi"] fn render(ret: *mut Image, m: *const u8, h: i32) -> *mut Image;
    #[link_name = "_ZN2wp7options12show_overlayENS0_5ImageE"] fn show_overlay(image: *mut Image);
    #[link_name = "_ZN2wp7options6layoutEiy"] fn layout_fn(ret: *mut u8, h: i32, rows: usize) -> *mut u8;
}
struct ImageDrop(*mut Image);
impl Drop for ImageDrop { fn drop(&mut self) { unsafe { vec_dealloc(&(*self.0).pixels) } } }
struct HintRowsDrop(*mut RawVec);
impl Drop for HintRowsDrop { fn drop(&mut self) { unsafe {
    let v = self.0; let mut e = (*v).start;
    while e != (*v).finish { str_destroy(e.add(HINTROW_KEYS).cast()); str_destroy(e.add(HINTROW_NAME).cast()); e = e.add(HINTROW_SIZE); }
    vec_dealloc(&*v);
} } }
/// std::string(const char*)
unsafe fn str_from_cstr(s: *mut CxxString, p: *const u8) {
    (*s).p = local(s);
    if p.is_null() { throw_logic_error(b"basic_string: construction from null is not valid\0".as_ptr()) }
    str_construct(s, p, strlen(p.cast()));
}
macro_rules! string_ret { ($ret:ident, $call:expr) => {{ str_init($ret); let mut g = StrRet($ret, true); $call; g.1 = false; $ret }} }
#[export_name = "_ZN2wp7options8menu_keyB5cxx11EPvjb"]
pub unsafe extern "C-unwind" fn menu_key(ret: *mut CxxString, w: *mut u8, key: u32, repeat: bool) -> *mut CxxString { string_ret!(ret, wprt_ow_key(w, key, repeat, assign_sink, ret.cast())) }
#[export_name = "_ZN2wp7options10menu_wheelB5cxx11EPvi"]
pub unsafe extern "C-unwind" fn menu_wheel(ret: *mut CxxString, w: *mut u8, delta: i32) -> *mut CxxString { string_ret!(ret, wprt_ow_wheel(w, delta, assign_sink, ret.cast())) }
#[export_name = "_ZN2wp7options10menu_clickB5cxx11EPviib"]
pub unsafe extern "C-unwind" fn menu_click(ret: *mut CxxString, h: *mut u8, x: i32, y: i32, right: bool) -> *mut CxxString { string_ret!(ret, wprt_ow_click(h, x, y, right, assign_sink, ret.cast())) }
#[export_name = "_ZN2wp7options13poll_gamepadsB5cxx11EPv"]
pub unsafe extern "C-unwind" fn poll_gamepads(ret: *mut CxxString, w: *mut u8) -> *mut CxxString {
    let r: *mut u8 = std::ptr::addr_of_mut!(REPEAT).cast();
    guarded_static(std::ptr::addr_of_mut!(REPEAT_GUARD), || {
        r.add(REPEAT_REPEATING).cast::<u32>().write(K_REPEATING); r.add(REPEAT_DELAY).cast::<i64>().write(400); r.add(REPEAT_INTERVAL).cast::<i64>().write(120);
        r.add(REPEAT_PREVIOUS).cast::<u32>().write(0); r.add(REPEAT_NEXT).cast::<i64>().write(0);
    });
    string_ret!(ret, wprt_ow_poll(w, r, assign_sink, ret.cast()))
}
#[export_name = "wphw_ow_handle"]
pub unsafe extern "C-unwind" fn wphw_ow_handle(action: i32, sink: Sink, c: *mut u8) {
    let mut slot = MaybeUninit::<CxxString>::uninit(); let v = slot.as_mut_ptr();
    menu_handle(v, menu(), action);
    let _g = StrDrop(v);
    sink(c, (*v).p, (*v).len);
}
#[export_name = "wphw_ow_instruction_update"]
pub unsafe extern "C-unwind" fn wphw_ow_instruction_update() -> bool {
    let game = game_description();
    let mut slot = MaybeUninit::<RawVec>::uninit(); let names = slot.as_mut_ptr();
    loaded_module_names(names);
    let _g = VecStrDrop(names);
    watch_update(std::ptr::addr_of_mut!(G_INSTRUCTION).cast(), names, game.add(DESC_INSTRUCTION).cast::<*const u8>().read(), game.add(DESC_PREFIX).cast::<*const u8>().read())
}
#[export_name = "wphw_ow_open"]
pub unsafe extern "C-unwind" fn wphw_ow_open() -> bool { *menu().add(MENU_OPEN) != 0 }
#[export_name = "wphw_ow_set_open"]
pub unsafe extern "C-unwind" fn wphw_ow_set_open(v: bool) { menu_set_open(menu(), v) }
#[export_name = "wphw_ow_act"]
pub unsafe extern "C-unwind" fn wphw_ow_act(w: *mut u8, action: i32, sink: Sink, c: *mut u8) {
    let mut slot = MaybeUninit::<CxxString>::uninit(); let v = slot.as_mut_ptr();
    str_init(v);
    let _g = StrDrop(v);
    wprt_ow_act(w, action, assign_sink, v.cast());
    sink(c, (*v).p, (*v).len);
}
#[export_name = "wphw_ow_select"]
pub unsafe extern "C-unwind" fn wphw_ow_select(row: usize) { menu_select(menu(), row) }
#[export_name = "wphw_ow_set_hover"]
pub unsafe extern "C-unwind" fn wphw_ow_set_hover(hit: *const u8) -> bool { menu_set_hover(menu(), hit) }
#[export_name = "wphw_ow_repeat"]
pub unsafe extern "C-unwind" fn wphw_ow_repeat(repeat: *mut u8, held: u32, now: i64) -> u32 { repeat_update(repeat, held, now) }
#[export_name = "wphw_ow_hint_chosen"]
pub unsafe extern "C-unwind" fn wphw_ow_hint_chosen() -> i32 {
    let mut slot = MaybeUninit::<CxxString>::uninit(); let s = slot.as_mut_ptr();
    settings_text(s, b"system.interface_language\0".as_ptr(), b"WP_INTERFACE_LANGUAGE\0".as_ptr());
    let _g = StrDrop(s);
    language_of(s)
}
#[export_name = "wphw_ow_hint_render"]
pub unsafe extern "C-unwind" fn wphw_ow_hint_render(width: i32, height: i32) {
    let language = wprt_ow_hint_language();
    let mut ts = MaybeUninit::<CxxString>::uninit(); let title = ts.as_mut_ptr();
    str_from_cstr(title, ui_label(b"hint.title\0".as_ptr(), language));
    let _gt = StrDrop(title);
    let mut rs = MaybeUninit::<RawVec>::uninit(); let rows = rs.as_mut_ptr();
    hint_rows(rows, input_bindings(), language);
    let _gr = HintRowsDrop(rows);
    let mut is = MaybeUninit::<Image>::uninit(); let image = is.as_mut_ptr();
    render_hint(image, rows, title, width, height);
    let _gi = ImageDrop(image);
    show_overlay(image);
}
#[export_name = "wphw_ow_menu_render"]
pub unsafe extern "C-unwind" fn wphw_ow_menu_render() {
    let h = wprt_ow_client_height();
    let mut is = MaybeUninit::<Image>::uninit(); let image = is.as_mut_ptr();
    render(image, menu(), h);
    let _gi = ImageDrop(image);
    show_overlay(image);
}
#[export_name = "wphw_ow_layout_sdk"]
pub unsafe extern "C-unwind" fn wphw_ow_layout_sdk(height: i32, out: *mut u8) {
    let mut ks = MaybeUninit::<RawVec>::uninit(); let keys = ks.as_mut_ptr();
    settings_keys(keys);
    let _gk = VecStrDrop(keys);
    let n = ((*keys).finish as usize - (*keys).start as usize) / 32;
    let mut tmp = MaybeUninit::<[u8; LAYOUT_SIZE]>::uninit();
    layout_fn(tmp.as_mut_ptr().cast(), height, n);
    std::ptr::copy_nonoverlapping(tmp.as_ptr().cast::<u8>(), out, LAYOUT_SIZE);
}
