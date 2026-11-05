// AI-assisted (Claude, G60) overlay_sdk.cpp -> Rust: whole member, no C++ residue.
// wp::options::Image {std::vector<uint32_t> pixels; uint32_t width,height;} lives on the Rust stack through the callback;
// poll_gamepads' std::string is received via sret and destroyed after the callback (Drop guards keep unwinding cleanup exact).
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("overlay_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
use std::mem::MaybeUninit;
#[repr(C)] pub struct Image { pub pixels: RawVec, pub width: u32, pub height: u32 }
extern "C-unwind" {
    #[link_name = "_ZN2wp7options12take_overlayERyRNS0_5ImageERb"] fn take_overlay(version: *mut u64, image: *mut Image, visible: *mut bool) -> bool;
    #[link_name = "_ZN2wp7options13poll_gamepadsB5cxx11EPv"] fn poll_gamepads(ret: *mut CxxString, window: *mut u8) -> *mut CxxString;
}
struct ImageDrop(*mut Image);
impl Drop for ImageDrop { fn drop(&mut self) { unsafe { vec_dealloc(&(*self.0).pixels) } } }
#[export_name = "wphw_host_options_overlay"]
pub unsafe extern "C-unwind" fn wphw_host_options_overlay(version: *mut u64, visible: *mut bool, context: *mut u8,
    receive: unsafe extern "C-unwind" fn(*mut u8, *const u32, usize, u32, u32)) -> bool {
    let mut slot = MaybeUninit::<Image>::uninit(); let image = slot.as_mut_ptr();
    image.write(Image { pixels: RawVec { start: std::ptr::null_mut(), finish: std::ptr::null_mut(), eos: std::ptr::null_mut() }, width: 0, height: 0 });
    let _g = ImageDrop(image);
    if !take_overlay(version, image, visible) { return false; }
    let px = &(*image).pixels;
    receive(context, px.start.cast(), (px.finish as usize - px.start as usize) / 4, (*image).width, (*image).height);
    true
}
#[export_name = "wphw_host_options_poll"]
pub unsafe extern "C-unwind" fn wphw_host_options_poll(context: *mut u8, receive: unsafe extern "C-unwind" fn(*mut u8, *const u8, usize)) {
    let mut slot = MaybeUninit::<CxxString>::uninit(); let s = slot.as_mut_ptr();
    poll_gamepads(s, std::ptr::null_mut());
    let _g = StrDrop(s);
    receive(context, (*s).p, (*s).len);
}
