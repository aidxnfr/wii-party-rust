// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted entire owned png-complete-port/png_rust.cpp adapter policy;native vector factories retained.
use std::ffi::{c_char,c_void};
extern "C"{fn wphw_png_vector_sdk(raw:*const c_void,p:*mut *const u32,n:*mut usize);fn wphw_png_receive_sdk(context:*mut c_void,p:*const u8,n:usize);fn wprt_png_save(path:*const c_char,pixels:*const u32,n:usize,width:u32,height:u32,rgba:bool);fn wprt_png_encode(pixels:*const u32,n:usize,width:u32,height:u32,rgba:bool,context:*mut c_void,sink:unsafe extern "C" fn(*mut c_void,*const u8,usize));}
// port of save_png/save_png_rgba;SAME source public vectors through typed SDK,original false/true constants.
unsafe fn save(path:*const c_char,raw:*const c_void,width:u32,height:u32,rgba:bool){let mut p=std::ptr::null();let mut n=0;wphw_png_vector_sdk(raw,&mut p,&mut n);wprt_png_save(path,p,n,width,height,rgba)}
#[export_name="_ZN2wp5video8save_pngEPKcRKSt6vectorIjSaIjEEjj"]pub unsafe extern "C" fn save_png(path:*const c_char,raw:*const c_void,width:u32,height:u32){save(path,raw,width,height,false)}
#[export_name="_ZN2wp5video13save_png_rgbaEPKcRKSt6vectorIjSaIjEEjj"]pub unsafe extern "C" fn save_png_rgba(path:*const c_char,raw:*const c_void,width:u32,height:u32){save(path,raw,width,height,true)}
// port of encode_png/encode scalar call;source output vector constructed before reading spans by its SDK factory.
#[no_mangle]pub unsafe extern "C" fn wphw_png_encode_adapter(raw:*const c_void,width:u32,height:u32,out:*mut c_void){let mut p=std::ptr::null();let mut n=0;wphw_png_vector_sdk(raw,&mut p,&mut n);wprt_png_encode(p,n,width,height,false,out,wphw_png_receive_sdk)}
