// SPDX-License-Identifier: GPL-3.0-or-later
// H8 adapter of pristine platform/video.cpp window_proc/apply_option, options_window.cpp.
// UNCOMPILED/UNINTEGRATED. Include in private game crate, feature host-options (requires host-window-mode).
// All Menu/Repeat/InstructionWatch/render/font policies are the already accepted implementations.
use std::ffi::c_void;
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
type Overlay=unsafe extern "C" fn(*mut c_void,*const u32,usize,u32,u32);
extern "C" {
 fn wprt_ow_open()->bool;
 fn wprt_ow_toggle(window:*mut c_void);
 fn wprt_ow_key(window:*mut c_void,key:u32,repeat:bool,sink:Sink,c:*mut c_void);
 fn wprt_ow_wheel(window:*mut c_void,delta:i32,sink:Sink,c:*mut c_void);
 fn wprt_ow_click(window:*mut c_void,x:i32,y:i32,right:bool,sink:Sink,c:*mut c_void);
 fn wprt_ow_hover(window:*mut c_void,x:i32,y:i32);
 fn wprt_ow_refresh(window:*mut c_void);
 fn wprt_vws_language();
 fn wphw_host_options_poll(c:*mut c_void,receive:Sink);
 fn wphw_host_options_overlay(version:*mut u64,visible:*mut bool,c:*mut c_void,receive:Overlay)->bool;
}
unsafe extern "C" fn changed(c:*mut c_void,p:*const u8,n:usize){
 // port of video.cpp apply_option: language application BEFORE refresh, fullscreen applied by Bevy.
 let key=if n==0{&[]}else{std::slice::from_raw_parts(p,n)};
 if key==b"system.interface_language"{wprt_vws_language();wprt_ow_refresh(std::ptr::null_mut());}
 if key==b"video.fullscreen"{*c.cast::<bool>()=true;}
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_open()->bool{
 crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire)&&wprt_ow_open()
}
// Distinct scalar operations are host transport discriminants, not game enums or guessed source values.
// Toggle/key/wheel/click/hover/refresh are driven on the Bevy main thread only.
#[no_mangle]pub unsafe extern "C" fn wpg_options_toggle(){
 if crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){wprt_ow_toggle(std::ptr::null_mut());}
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_key(key:u32,repeat:bool)->bool{
 if !crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 let mut fullscreen=false;wprt_ow_key(std::ptr::null_mut(),key,repeat,changed,(&mut fullscreen as *mut bool).cast());fullscreen
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_wheel(delta:i32)->bool{
 if !crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 let mut fullscreen=false;wprt_ow_wheel(std::ptr::null_mut(),delta,changed,(&mut fullscreen as *mut bool).cast());fullscreen
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_click(x:i32,y:i32,right:bool)->bool{
 if !crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 let mut fullscreen=false;wprt_ow_click(std::ptr::null_mut(),x,y,right,changed,(&mut fullscreen as *mut bool).cast());fullscreen
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_hover(x:i32,y:i32){
 if crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){wprt_ow_hover(std::ptr::null_mut(),x,y);}
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_refresh(){
 if crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){wprt_ow_refresh(std::ptr::null_mut());}
}
// Source video.cpp kMenuTimerPeriod=16ms; the host supplies that cadence, not guest frames.
#[no_mangle]pub unsafe extern "C" fn wpg_options_poll()->bool{
 if !crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 let mut fullscreen=false;wphw_host_options_poll((&mut fullscreen as *mut bool).cast(),changed);fullscreen
}
#[no_mangle]pub unsafe extern "C" fn wpg_options_overlay(version:*mut u64,visible:*mut bool,c:*mut c_void,receive:Overlay)->bool{
 if !crate::WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 wphw_host_options_overlay(version,visible,c,receive)
}

// Host physical client dimensions/focus replace only the source window queries.
#[no_mangle]pub extern "C" fn wpg_options_client(width:i32,height:i32,focused:bool){
 wprt_hw::native_host_client::update(width,height,focused);
}
// Source apply_option reads the stored fullscreen setting without an environment override.
#[no_mangle]pub unsafe extern "C" fn wpg_options_fullscreen()->bool{
 wprt::native_settings::wprt_settings_flag(b"video.fullscreen\0".as_ptr().cast(),std::ptr::null())
}
// wp/options.h Rect: public POD only. Calls the already accepted exact placement policy.
#[repr(C)]pub struct Rect{x:i32,y:i32,width:i32,height:i32}
extern "C" {fn wprt_options_place(cw:i32,ch:i32,w:i32,h:i32,out:*mut Rect);}
#[no_mangle]pub unsafe extern "C" fn wpg_options_place(cw:i32,ch:i32,w:i32,h:i32,out:*mut Rect){wprt_options_place(cw,ch,w,h,out);}
