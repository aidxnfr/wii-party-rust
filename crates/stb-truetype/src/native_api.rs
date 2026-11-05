// SPDX-License-Identifier: Unlicense
// Exact used stb_truetype.h API,opaque Rust font state;no C++ class layout shared.
use crate::*;
use std::ffi::c_void;
#[no_mangle] pub unsafe extern "C" fn wprt_stb_offset(data:*const u8,index:i32)->i32 {stbtt_GetFontOffsetForIndex(data,index)}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_init(data:*const u8,offset:i32)->*mut c_void {let mut info=Box::new(stbtt_fontinfo::default());if stbtt_InitFont(&mut *info,data,offset)==0 {std::ptr::null_mut()}else {Box::into_raw(info).cast()}}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_free(info:*mut c_void){if !info.is_null(){drop(Box::from_raw(info.cast::<stbtt_fontinfo>()))}}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_em_scale(info:*mut c_void,pixels:f32)->f32 {stbtt_ScaleForMappingEmToPixels(info.cast(),pixels)}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_vmetrics(info:*mut c_void,a:*mut i32,d:*mut i32,g:*mut i32){stbtt_GetFontVMetrics(info.cast(),a,d,g)}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_hmetrics(info:*mut c_void,c:i32,a:*mut i32,b:*mut i32){stbtt_GetCodepointHMetrics(info.cast(),c,a,b)}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_kern(info:*mut c_void,a:i32,b:i32)->i32 {stbtt_GetCodepointKernAdvance(info.cast(),a,b)}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_box(info:*mut c_void,c:i32,sx:f32,sy:f32,x:f32,y:f32,x0:*mut i32,y0:*mut i32,x1:*mut i32,y1:*mut i32){stbtt_GetCodepointBitmapBoxSubpixel(info.cast(),c,sx,sy,x,y,x0,y0,x1,y1)}
#[no_mangle] pub unsafe extern "C" fn wprt_stb_bitmap(info:*mut c_void,out:*mut u8,w:i32,h:i32,stride:i32,sx:f32,sy:f32,x:f32,y:f32,c:i32){stbtt_MakeCodepointBitmapSubpixel(info.cast(),out,w,h,stride,sx,sy,x,y,c)}
