// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned video-win32-port/video_controls_rust.cpp primitive/window-state adapter policy port.
// SDK retains original exception_ptr barriers,thread/container/render/context lifetimes;no renderer/Win32 algorithm redesign.
use std::{ffi::{c_char,c_void},sync::atomic::{AtomicPtr,AtomicBool,AtomicU64,Ordering}};
use windows_sys::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT;
static WINDOW:AtomicPtr<c_void>=AtomicPtr::new(std::ptr::null_mut());static SCREENSHOT:AtomicBool=AtomicBool::new(false);
static mut FULLSCREEN:bool=false;static mut PLACEMENT:WINDOWPLACEMENT=unsafe{std::mem::zeroed()};static HIDE_CACHE:AtomicU64=AtomicU64::new(0);
extern "C"{fn wprt_video_start();fn wprt_video_present();fn wprt_video_image_point(x:i32,y:i32,width:i32,height:i32,out_x:*mut f32,out_y:*mut f32)->bool;
fn wprt_vws_focused()->bool;fn wprt_vws_client_size(w:*mut i32,h:*mut i32)->bool;fn wprt_vws_key_down(key:i32)->bool;fn wprt_vws_cursor(x:*mut i32,y:*mut i32)->bool;fn wprt_vws_cursor_image(x:*mut f32,y:*mut f32)->bool;fn wprt_vws_language();fn wprt_vws_headless()->bool;fn wprt_vws_saving()->bool;fn wprt_vws_map_shift(scan:u32)->i32;fn wprt_vws_apply_title(window:usize);fn wprt_vws_fullscreen(window:usize,enable:bool,placement:*mut WINDOWPLACEMENT,current:*mut bool);fn wprt_vws_null_cursor();fn wprt_vws_theme(window:usize);fn wprt_vws_paint(window:usize);fn wprt_vws_default(window:usize,msg:u32,wparam:usize,lparam:isize)->isize;
#[link_name="_ZN2wp4nand10widescreenEv"]fn widescreen()->bool;
#[link_name="_ZN2wp2gx7processEv"]fn gx_process();
#[link_name="_ZN2wp2gx15request_captureEv"]fn gx_capture();
#[link_name="_ZN2wp7options12refresh_menuEPv"]fn refresh_menu(window:*mut c_void);
#[link_name="_ZN2wp7options9menu_openEv"]fn menu_open()->bool;
#[link_name="_ZN2wp7options11toggle_menuEPv"]fn toggle_menu(window:*mut c_void);
#[link_name="_ZN2wp7options10menu_hoverEPvii"]fn menu_hover(window:*mut c_void,x:i32,y:i32);
#[link_name="_ZN2wp5input10note_wheelEi"]fn note_wheel(delta:i32);
fn wprt_settings_revision()->u32;fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;}
// port of source namespace primitive forwards;Windowslong32 matches target actual public header.
#[export_name="_ZN2wp5video5startEv"]pub unsafe extern "C" fn start(){wprt_video_start();}
#[export_name="_ZN2wp5video7presentEv"]pub unsafe extern "C" fn present(){wprt_video_present();}
#[export_name="_ZN2wp5video13window_handleEv"]pub extern "C" fn window_handle()->*mut c_void{WINDOW.load(Ordering::SeqCst)}
#[export_name="_ZN2wp5video11image_pointEllllRfS1_"]pub unsafe extern "C" fn image_point(x:i32,y:i32,w:i32,h:i32,out_x:*mut f32,out_y:*mut f32)->bool{wprt_video_image_point(x,y,w,h,out_x,out_y)}
#[export_name="_ZN2wp5video14window_focusedEv"]pub unsafe extern "C" fn focused()->bool{if let Some(client)=super::native_host_client::get(){client.focused}else{wprt_vws_focused()}}
#[export_name="_ZN2wp5video11client_sizeERiS1_"]pub unsafe extern "C" fn client_size(w:*mut i32,h:*mut i32)->bool{if let Some(client)=super::native_host_client::get(){*w=client.width;*h=client.height;true}else{wprt_vws_client_size(w,h)}}
#[export_name="_ZN2wp5video8key_downEi"]pub unsafe extern "C" fn key_down(key:i32)->bool{wprt_vws_key_down(key)}
#[export_name="_ZN2wp5video15cursor_positionERlS1_"]pub unsafe extern "C" fn cursor(x:*mut i32,y:*mut i32)->bool{wprt_vws_cursor(x,y)}
#[export_name="_ZN2wp5video15cursor_on_imageERfS1_"]pub unsafe extern "C" fn cursor_image(x:*mut f32,y:*mut f32)->bool{wprt_vws_cursor_image(x,y)}
// port of original atomic HWND/screenshot operations;seq_cstdefault preserved,no C++atomic representations shared.
#[no_mangle]pub extern "C" fn wphw_video_window()->usize{WINDOW.load(Ordering::SeqCst)as usize}
#[no_mangle]pub extern "C" fn wphw_vws_store_window(window:usize){WINDOW.store(window as *mut c_void,Ordering::SeqCst);}
#[no_mangle]pub extern "C" fn wphw_vc_request_screenshot(){SCREENSHOT.store(true,Ordering::SeqCst);}
#[no_mangle]pub extern "C" fn wphw_video_screenshot_exchange()->bool{SCREENSHOT.swap(false,Ordering::SeqCst)}
// port of source fullscreen scalar+PUBLIC Win32 WINDOWPLACEMENT storage;original source passes both bypointer.
#[no_mangle]pub unsafe extern "C" fn wphw_vc_fullscreen()->bool{FULLSCREEN}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_set_fullscreen(window:usize,enable:bool){wprt_vws_fullscreen(window,enable,std::ptr::addr_of_mut!(PLACEMENT),std::ptr::addr_of_mut!(FULLSCREEN));}
// port of source LiveFlag input.hide_cursor,primary revision/acquire/release cache.
#[no_mangle]pub unsafe extern "C" fn wphw_vc_hide_cursor()->bool{let revision=wprt_settings_revision()as u64;let mut cached=HIDE_CACHE.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|if wprt_settings_flag(c"input.hide_cursor".as_ptr(),c"WP_HIDE_CURSOR".as_ptr()){1}else{0};HIDE_CACHE.store(cached,Ordering::Release);}cached&1!=0}
// port of exact source SDK forwards;no extra guards/fallbacks.
#[no_mangle]pub unsafe extern "C" fn wphw_video_widescreen()->bool{widescreen()}
#[no_mangle]pub unsafe extern "C" fn wphw_video_apply_language(){wprt_vws_language();}
#[no_mangle]pub unsafe extern "C" fn wphw_video_headless()->bool{wprt_vws_headless()}
#[no_mangle]pub unsafe extern "C" fn wphw_video_saving()->bool{wprt_vws_saving()}
#[no_mangle]pub unsafe extern "C" fn wphw_video_gx_process(){gx_process();}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_map_shift(scan:u32)->i32{wprt_vws_map_shift(scan)}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_title(window:usize){wprt_vws_apply_title(window);}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_capture(){gx_capture();}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_refresh_menu(window:usize){refresh_menu(window as *mut c_void);}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_menu_open()->bool{menu_open()}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_toggle_menu(window:usize){toggle_menu(window as *mut c_void);}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_menu_hover(window:usize,x:i32,y:i32){menu_hover(window as *mut c_void,x,y);}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_set_null_cursor(){wprt_vws_null_cursor();}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_note_wheel(delta:i32){note_wheel(delta);}
// source keymap.h wheel constants0x100/0x101,not a guessed input mapping.
#[no_mangle]pub extern "C" fn wphw_vc_wheel_key(up:bool)->i32{if up{0x100}else{0x101}}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_theme(window:usize){wprt_vws_theme(window);}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_paint(window:usize){wprt_vws_paint(window);}
#[no_mangle]pub unsafe extern "C" fn wphw_vc_default(window:usize,message:u32,wparam:usize,lparam:isize)->isize{wprt_vws_default(window,message,wparam,lparam)}
