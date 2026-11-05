// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of engine/src/platform/screenshot.cpp (Claude lane). Differential: claude-gx/png-enc (identical).

/// screenshot.h kFolder.
pub const FOLDER: &str = "screenshots";

/// The std::tm fields screenshot.cpp reads.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Time { pub year: i32, pub mon: i32, pub mday: i32, pub hour: i32, pub min: i32, pub sec: i32 }

/// screenshot.cpp file_name: prefix + "_%04d-%02d-%02d_%02d-%02d-%02d" (+ "_<attempt>" when attempt > 1) + ".png".
/// The original adds 1900 / 1 in int (two's complement on x86: wrapping).
pub fn file_name(prefix: &str, time: &Time, attempt: i32) -> String {
    let stamp = format!("_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", time.year.wrapping_add(1900), time.mon.wrapping_add(1), time.mday, time.hour, time.min, time.sec);
    let mut name = format!("{prefix}{stamp}");
    if attempt > 1 {
        name += &format!("_{attempt}");
    }
    name + ".png"
}

// ---- C ABI used by claude-gx/cpp/png_rust.cpp (replaces engine/src/platform/png.cpp + screenshot.cpp) ----
use std::ffi::{c_char, c_void, CStr};

/// Byte sink: the C++ side appends `len` bytes to its std::vector / std::string.
pub type Sink = extern "C" fn(context: *mut c_void, data: *const u8, len: usize);

unsafe fn pixels<'a>(data: *const u32, count: usize) -> &'a [u32] { if count == 0 { &[] } else { std::slice::from_raw_parts(data, count) } }
/// fopen(path, "wb") takes the narrow path as-is; engine paths are ASCII (UTF-8 here).
unsafe fn path(p: *const c_char) -> String { CStr::from_ptr(p).to_string_lossy().into_owned() }

#[no_mangle]
pub unsafe extern "C" fn wpgx_png_encode(data: *const u32, count: usize, width: u32, height: u32, sink: Sink, context: *mut c_void) {
    let file = wprt_ports::png::encode_png(pixels(data, count), width, height);
    sink(context, file.as_ptr(), file.len());
}
#[no_mangle]
pub unsafe extern "C" fn wpgx_png_save(file: *const c_char, data: *const u32, count: usize, width: u32, height: u32) {
    wprt_ports::png::save_png(&path(file), pixels(data, count), width, height);
}
#[no_mangle]
pub unsafe extern "C" fn wpgx_png_save_rgba(file: *const c_char, data: *const u32, count: usize, width: u32, height: u32) {
    wprt_ports::png::save_png_rgba(&path(file), pixels(data, count), width, height);
}
#[no_mangle]
pub unsafe extern "C" fn wpgx_screenshot_file_name(prefix: *const c_char, time: *const Time, attempt: i32, sink: Sink, context: *mut c_void) {
    let name = file_name(&CStr::from_ptr(prefix).to_string_lossy(), &*time, attempt);
    sink(context, name.as_ptr(), name.len());
}
