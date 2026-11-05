// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned cpp/patched/options_state_rust.cpp POD adapter policy port.
// SDK keeps original private Menu/Repeat/InstructionWatch fields,containers,mutex/Image lifetimes;accepted policies unchanged.
use std::ffi::{c_char,c_void};
#[repr(C)]pub struct Layout{pub unit:i32,pub width:i32,pub height:i32,pub padding:i32,pub title_height:i32,pub row_height:i32,pub footer_height:i32,pub rows:usize}
#[repr(C)]pub struct Rect{pub x:i32,pub y:i32,pub width:i32,pub height:i32}
#[repr(C)]pub struct Hover{pub row:i32,pub part:i32}
#[repr(C)]struct Text{fps:*const c_char,dsp:*const c_char,dsp_native:*const c_char,dsp_interpreted:*const c_char,dsp_stopped:*const c_char,menu_title:*const c_char,menu_hint:*const c_char,menu_restart:*const c_char,menu_on:*const c_char,menu_off:*const c_char,menu_native:*const c_char,menu_system:*const c_char,labels:*const c_void,label_count:usize}
#[repr(C)]pub struct Ui{pub native:*const c_char,pub system:*const c_char,pub on:*const c_char,pub off:*const c_char}
extern "C"{fn wprt_options_layout(height:i32,rows:usize,out:*mut Layout);fn wprt_options_place(cw:i32,ch:i32,w:i32,h:i32,out:*mut Rect);fn wprt_options_row(layout:*const Layout,placed:*const Rect,x:i32,y:i32)->i32;fn wprt_options_control(layout:*const Layout,row:usize,out:*mut Rect);fn wprt_options_close(layout:*const Layout,out:*mut Rect);fn wprt_options_hit(layout:*const Layout,placed:*const Rect,x:i32,y:i32,out:*mut Hover);
#[link_name="_ZN2wp2ui4textEv"]fn ui_text()->*const Text;}
// port of wphw_options_ui from PUBLIC ui_text.h fields;no private layout/native object casts.
#[no_mangle]pub unsafe extern "C" fn wphw_options_ui(out:*mut Ui){let t=&*ui_text();*out=Ui{native:t.menu_native,system:t.menu_system,on:t.menu_on,off:t.menu_off};}
// port of source public POD default initialization BEFORE provider fills output.
#[export_name="_ZN2wp7options6layoutEiy"]pub unsafe extern "C" fn layout(height:i32,rows:usize)->Layout{let mut out=Layout{unit:0,width:0,height:0,padding:0,title_height:0,row_height:0,footer_height:0,rows:0};wprt_options_layout(height,rows,&mut out);out}
#[export_name="_ZN2wp7options5placeEiiii"]pub unsafe extern "C" fn place(cw:i32,ch:i32,w:i32,h:i32)->Rect{let mut out=Rect{x:0,y:0,width:0,height:0};wprt_options_place(cw,ch,w,h,&mut out);out}
#[export_name="_ZN2wp7options6row_atERKNS0_6LayoutERKNS0_4RectEii"]pub unsafe extern "C" fn row_at(layout:*const Layout,placed:*const Rect,x:i32,y:i32)->i32{wprt_options_row(layout,placed,x,y)}
#[export_name="_ZN2wp7options12control_rectERKNS0_6LayoutEy"]pub unsafe extern "C" fn control(layout:*const Layout,row:usize)->Rect{let mut out=Rect{x:0,y:0,width:0,height:0};wprt_options_control(layout,row,&mut out);out}
#[export_name="_ZN2wp7options10close_rectERKNS0_6LayoutE"]pub unsafe extern "C" fn close(layout:*const Layout)->Rect{let mut out=Rect{x:0,y:0,width:0,height:0};wprt_options_close(layout,&mut out);out}
#[export_name="_ZN2wp7options8hit_testERKNS0_6LayoutERKNS0_4RectEii"]pub unsafe extern "C" fn hit(layout:*const Layout,placed:*const Rect,x:i32,y:i32)->Hover{let mut out=Hover{row:-1,part:0};wprt_options_hit(layout,placed,x,y,&mut out);out}
