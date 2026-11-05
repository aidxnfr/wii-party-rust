// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine input/real_wiimote.cpp remaining SDL HID service calls.
// SDK declarations from owned official SDL3-3.4.16 include/SDL3/SDL_hidapi.h:113-156,177-473.
use std::ffi::{c_void,c_char};
// Exact Windows SDK public struct definition;SDL enum bus_type uses the source C int representation.
#[repr(C)]pub struct HidInfo{path:*mut c_char,vendor_id:u16,product_id:u16,serial_number:*mut u16,release_number:u16,manufacturer_string:*mut u16,product_string:*mut u16,usage_page:u16,usage:u16,interface_number:i32,interface_class:i32,interface_subclass:i32,interface_protocol:i32,bus_type:i32,next:*mut HidInfo}
#[link(name="SDL3")]extern "C"{
 fn SDL_hid_init()->i32;
 fn SDL_hid_enumerate(v:u16,p:u16)->*mut HidInfo;
 fn SDL_hid_free_enumeration(p:*mut HidInfo);
 fn SDL_hid_open_path(p:*const c_char)->*mut c_void;
 fn SDL_hid_close(p:*mut c_void)->i32;
 fn SDL_hid_write(p:*mut c_void,data:*const u8,n:usize)->i32;
 fn SDL_hid_read_timeout(p:*mut c_void,data:*mut u8,n:usize,timeout:i32)->i32;
 fn SDL_hid_device_change_count()->u32;
}
// port of source real_wiimote run/open_new SDL service leaves. Keep return values and ignored close result exactly.
// port of engine/src/input/real_wiimote.cpp start SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_hid_init()->i32{SDL_hid_init()}
// port of engine/src/input/real_wiimote.cpp open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_enumerate(v:u16,p:u16)->*mut c_void{SDL_hid_enumerate(v,p).cast()}
// port of engine/src/input/real_wiimote.cpp open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_next(p:*mut c_void)->*mut c_void{(*(p.cast::<HidInfo>())).next.cast()}
// port of engine/src/input/real_wiimote.cpp open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_product(p:*mut c_void)->u16{(*(p.cast::<HidInfo>())).product_id}
// port of engine/src/input/real_wiimote.cpp open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_path(p:*mut c_void)->*const c_char{(*(p.cast::<HidInfo>())).path}
// port of engine/src/input/real_wiimote.cpp open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_free(p:*mut c_void){SDL_hid_free_enumeration(p.cast())}
// port of engine/src/input/real_wiimote.cpp open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_open(p:*const c_char)->usize{SDL_hid_open_path(p) as usize}
// port of engine/src/input/real_wiimote.cpp close_remote/open_new SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_close(p:usize){SDL_hid_close(p as *mut c_void);}
// port of engine/src/input/real_wiimote.cpp open_new/service SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_write(p:usize,data:*const u8,n:usize)->i32{SDL_hid_write(p as *mut c_void,data,n)}
// port of engine/src/input/real_wiimote.cpp open_new/service SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_read(p:usize,data:*mut u8,n:usize,timeout:i32)->i32{SDL_hid_read_timeout(p as *mut c_void,data,n,timeout)}
// port of engine/src/input/real_wiimote.cpp run SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_real_change_count()->u32{SDL_hid_device_change_count()}

mod gamepad;
