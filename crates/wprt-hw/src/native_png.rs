// SPDX-License-Identifier: GPL-3.0-or-later
// H7 adapter reuses OpenCode O3 png.cpp algorithms; PNG algorithms and ANSI CRT file-write control are Rust;C++ only marshals its vector API.
#[path = "prepared_png.rs"]
mod prepared;
// port of png.cpp encode_png/encode_raw_png as used by save_png_rgba. Callback is synchronous vector byte marshaling.
#[no_mangle]
pub unsafe extern "C" fn wprt_png_encode(
    pixels: *const u32,
    n: usize,
    width: u32,
    height: u32,
    rgba: bool,
    context: *mut std::ffi::c_void,
    emit: unsafe extern "C" fn(*mut std::ffi::c_void, *const u8, usize),
) {
    // C++ empty vector data may be null;the source accepts an empty pixel span.
    let pixels = if n == 0 { &[] } else { std::slice::from_raw_parts(pixels, n) };
    let file = if rgba {
        prepared::encode_png_rgba(pixels, width, height)
    } else {
        prepared::encode_png(pixels, width, height)
    };
    emit(context, file.as_ptr(), file.len());
}

// port of engine/src/platform/png.cpp write_file: retain the same ANSI CRT,
// mode, element size, ignored write result, and conditional close ordering.
extern "C" {
    fn fopen(path: *const std::ffi::c_char, mode: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn fwrite(data: *const std::ffi::c_void, size: usize, count: usize, file: *mut std::ffi::c_void) -> usize;
    fn fclose(file: *mut std::ffi::c_void) -> std::ffi::c_int;
}
unsafe fn write_file(path: *const std::ffi::c_char, file: &[u8]) {
    let handle=fopen(path,b"wb\0".as_ptr().cast());
    if !handle.is_null() {
        fwrite(file.as_ptr().cast(),1,file.len(),handle);
        fclose(handle);
    }
}
// port of png.cpp save_png/save_png_rgba; encode before attempting fopen.
#[no_mangle]
pub unsafe extern "C" fn wprt_png_save(path:*const std::ffi::c_char,pixels:*const u32,n:usize,width:u32,height:u32,rgba:bool) {
    let pixels=if n==0 {&[]} else {std::slice::from_raw_parts(pixels,n)};
    let file=if rgba {prepared::encode_png_rgba(pixels,width,height)} else {prepared::encode_png(pixels,width,height)};
    write_file(path,&file);
}
