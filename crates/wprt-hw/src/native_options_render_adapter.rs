// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted owned font-file-port/options_render_rust.cpp scalar adapter policy.
// UNINTEGRATED:typed original Font/Face/Image/container/exception/cache factories remain SDK.
use std::ffi::{c_char,c_void};
extern "C"{fn wphw_options_regular_font_sdk()->bool;fn wprt_options_choose_bold(bold:bool,ok:bool)->bool;fn wprt_hw_stb_kern(font:*mut c_void,code:i32,next:i32)->i32;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);}
// port of load_font final branch;exact log text/source order,source factory owns chosenFont.
#[no_mangle]pub unsafe extern "C" fn wphw_options_font_result(ok:bool){if !ok{log_write(c"options".as_ptr(),c"no usable font for the options menu".as_ptr());}}
// port of font selection through existing accepted policy.
#[no_mangle]pub unsafe extern "C" fn wphw_options_font_bold(bold:bool,ok:bool)->bool{wprt_options_choose_bold(bold,ok)}
// port of text_metrics kern expression;do NOT call kern provider when next==0.
#[no_mangle]pub unsafe extern "C" fn wphw_options_kern(font:*mut c_void,code:u32,next:u32)->i32{if next!=0{wprt_hw_stb_kern(font,code as i32,next as i32)}else{0}}
// port of render/hint/titlebar short-circuit order;regularFont cache accessed ONLY after valid dimensions/nonempty rows.
#[no_mangle]pub unsafe extern "C" fn wphw_options_render_gate(width:i32,height:i32)->bool{width>0&&height>0&&wphw_options_regular_font_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_options_hint_gate(empty:bool,width:i32,height:i32)->bool{!empty&&width>0&&height>0&&wphw_options_regular_font_sdk()}
#[no_mangle]pub extern "C" fn wphw_options_titlebar_gate(width:i32,height:i32)->bool{width>0&&height>0}
#[no_mangle]pub unsafe extern "C" fn wphw_options_regular_font_ok()->bool{wphw_options_regular_font_sdk()}
