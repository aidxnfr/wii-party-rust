// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned options-window-complete-port/options_window_rust.cpp policy port.
// SDK retains original Menu/InstructionWatch/Repeat objects,containers,render-image lifetimes;no class layout guessed.
use std::{ffi::{c_char,c_void},sync::atomic::{AtomicU64,Ordering}};
#[repr(C)]pub struct Hover{pub row:i32,pub part:i32}
#[repr(C)]pub struct Layout{pub unit:i32,pub width:i32,pub height:i32,pub padding:i32,pub title_height:i32,pub row_height:i32,pub footer_height:i32,pub rows:usize}
#[repr(C)]pub struct Rect{pub x:i32,pub y:i32,pub width:i32,pub height:i32}
extern "C"{fn wprt_ow_open()->bool;fn wprt_ow_toggle(window:*mut c_void);fn wprt_ow_hover(window:*mut c_void,x:i32,y:i32);fn wprt_ow_refresh(window:*mut c_void);fn wprt_ow_instruction()->bool;fn wprt_ow_update_hint(shown:*mut bool,width:*mut i32,height:*mut i32);fn wprt_ow_sync(shown:*mut bool,hint_shown:*mut bool);fn wphw_ow_layout_sdk(height:i32,out:*mut Layout);
#[link_name="_ZN2wp5video11client_sizeERiS1_"]fn client_size(width:*mut i32,height:*mut i32)->bool;
#[link_name="_ZN2wp7options5placeEiiii"]fn place(cw:i32,ch:i32,width:i32,height:i32)->Rect;
#[link_name="_ZN2wp7options8hit_testERKNS0_6LayoutERKNS0_4RectEii"]fn hit_test(layout:*const Layout,placed:*const Rect,x:i32,y:i32)->Hover;
#[link_name="_ZN2wp5video14window_focusedEv"]fn focused()->bool;
#[link_name="_ZN2wp7gamepad12menu_buttonsEv"]fn buttons()->u32;
#[link_name="_ZN2wp4nand18console_is_spanishEv"]fn spanish()->bool;
#[link_name="_ZN2wp7options12hide_overlayEv"]fn hide_overlay();
#[link_name="_ZN2wp5input11set_blockedEb"]fn set_blocked(value:bool);
#[link_name="_ZNSt6chrono3_V212steady_clock3nowEv"]fn clock_now()->i64;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;}
// port of original anonymous source scalar owners;C++ class owners stay SDK.
static mut SHOWN:bool=false;static mut HINT_SHOWN:bool=false;static mut HINT_WIDTH:i32=0;static mut HINT_HEIGHT:i32=0;
static MENU_CACHE:AtomicU64=AtomicU64::new(0);static HINT_CACHE:AtomicU64=AtomicU64::new(0);
unsafe fn flag(cache:&AtomicU64,key:*const c_char,var:*const c_char)->bool{let revision=wprt_settings_revision()as u64;let mut cached=cache.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(key,var){1}else{0};cache.store(cached,Ordering::Release);}cached&1!=0}
// port of public namespace primitive forwards;source object namespace names retained.
#[export_name="_ZN2wp7options9menu_openEv"]pub unsafe extern "C" fn menu_open()->bool{wprt_ow_open()}
#[export_name="_ZN2wp7options11toggle_menuEPv"]pub unsafe extern "C" fn toggle_menu(window:*mut c_void){wprt_ow_toggle(window);}
#[export_name="_ZN2wp7options10menu_hoverEPvii"]pub unsafe extern "C" fn menu_hover(window:*mut c_void,x:i32,y:i32){wprt_ow_hover(window,x,y);}
#[export_name="_ZN2wp7options12refresh_menuEPv"]pub unsafe extern "C" fn refresh_menu(window:*mut c_void){wprt_ow_refresh(window);}
// port of source real steady clock ticks/duration_cast milliseconds,not the guest comparison clock.
#[no_mangle]pub unsafe extern "C" fn wphw_ow_instruction_now()->i64{clock_now()}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_now()->i64{clock_now()/1_000_000}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_enabled()->bool{flag(&MENU_CACHE,c"system.options_menu".as_ptr(),c"WP_OPTIONS_MENU".as_ptr())}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_hint_enabled()->bool{flag(&HINT_CACHE,c"input.controls_hint".as_ptr(),c"WP_CONTROLS_HINT".as_ptr())}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_sync(_window:*mut c_void){wprt_ow_sync(std::ptr::addr_of_mut!(SHOWN),std::ptr::addr_of_mut!(HINT_SHOWN));}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_toggle(window:*mut c_void){wprt_ow_toggle(window);}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_update_hint(){wprt_ow_update_hint(std::ptr::addr_of_mut!(HINT_SHOWN),std::ptr::addr_of_mut!(HINT_WIDTH),std::ptr::addr_of_mut!(HINT_HEIGHT));}
// port of source hit helper;zero client locals,return false WITHOUT writing output;typed layout SDK keeps temporary keys lifetime.
#[no_mangle]pub unsafe extern "C" fn wphw_ow_hit(x:i32,y:i32,out:*mut Hover)->bool{let(mut width,mut height)=(0,0);if !client_size(&mut width,&mut height){return false;}let mut frame=std::mem::MaybeUninit::<Layout>::uninit();wphw_ow_layout_sdk(height,frame.as_mut_ptr());let frame=frame.assume_init();let placed=place(width,height,frame.width,frame.height);*out=hit_test(&frame,&placed,x,y);true}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_focused()->bool{focused()}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_buttons()->u32{buttons()}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_client(width:*mut i32,height:*mut i32)->bool{client_size(width,height)}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_instruction()->bool{wprt_ow_instruction()}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_spanish()->bool{spanish()}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_hint_log(shown:bool){if shown{log_write(c"options".as_ptr(),c"controls window shown".as_ptr());}else{log_write(c"options".as_ptr(),c"controls window hidden".as_ptr());}}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_hide(){hide_overlay();}
#[no_mangle]pub unsafe extern "C" fn wphw_ow_block(value:bool){set_blocked(value);}
