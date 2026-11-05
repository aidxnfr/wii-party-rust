// SPDX-License-Identifier: GPL-3.0-or-later
// Used stb API forwarding only;tested Rust backend owns opaque font state.
use std::ffi::c_void;
use stb_truetype_rust::native_api as backend;
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_offset(p:*const u8,i:i32)->i32{backend::wprt_stb_offset(p,i)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_init(p:*const u8,i:i32)->*mut c_void{backend::wprt_stb_init(p,i)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_free(p:*mut c_void){backend::wprt_stb_free(p)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_em_scale(p:*mut c_void,h:f32)->f32{backend::wprt_stb_em_scale(p,h)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_vmetrics(p:*mut c_void,a:*mut i32,d:*mut i32,g:*mut i32){backend::wprt_stb_vmetrics(p,a,d,g)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_hmetrics(p:*mut c_void,c:i32,a:*mut i32,b:*mut i32){backend::wprt_stb_hmetrics(p,c,a,b)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_kern(p:*mut c_void,a:i32,b:i32)->i32{backend::wprt_stb_kern(p,a,b)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_box(p:*mut c_void,c:i32,sx:f32,sy:f32,x:f32,y:f32,a:*mut i32,b:*mut i32,d:*mut i32,e:*mut i32){backend::wprt_stb_box(p,c,sx,sy,x,y,a,b,d,e)}
#[no_mangle] pub unsafe extern "C" fn wprt_hw_stb_bitmap(p:*mut c_void,out:*mut u8,w:i32,h:i32,stride:i32,sx:f32,sy:f32,x:f32,y:f32,c:i32){backend::wprt_stb_bitmap(p,out,w,h,stride,sx,sy,x,y,c)}
