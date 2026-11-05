// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete owned cpp/patched/bluetooth_rust.cpp adapter policy port.
// Native SDK retains typed std::string/Sender/vector marshaling;existing comparison clock is called unchanged.
use std::ffi::c_char;
extern "C"{fn wprt_bt_ioctlv(r:u32,c:u32,n:u32,o:u32,v:u32)->i32;fn wprt_bt_close();fn wprt_bt_update();fn wprt_bt_take_completion(r:*mut u32,v:*mut i32)->bool;
fn wphw_bt_emulated_output_sdk(i:u32,p:*const u8,n:u32);fn wphw_bt_emulated_update_sdk(i:u32);fn wphw_bt_real_update_sdk(i:u32);
#[link_name="_ZN2wp10comparison3nowEv"]fn comparison_now()->i64;
#[link_name="_ZN2wp12real_wiimote7presentEj"]fn real_present(i:u32)->bool;
#[link_name="_ZN2wp12real_wiimote9take_wakeEj"]fn real_wake(i:u32)->bool;
#[link_name="_ZN2wp12real_wiimote5startEv"]fn real_start();
#[link_name="_ZN2wp12real_wiimote5resetEj"]fn real_reset(i:u32);
#[link_name="_ZN2wp12real_wiimote13output_reportEjPKhj"]fn real_output(i:u32,p:*const u8,n:u32);
#[link_name="_ZN2wp5input9connectedEj"]fn input_connected(i:u32)->bool;
#[link_name="_ZN2wp5input12wakes_remoteEj"]fn input_wake(i:u32)->bool;
#[link_name="_ZN2wp7wiimote5resetEj"]fn wii_reset(i:u32);
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of namespace wp::bluetooth primitive forwards;names from accepted archive nm.
#[export_name="_ZN2wp9bluetooth6ioctlvEjjjjj"]pub unsafe extern "C" fn ioctlv(r:u32,c:u32,n:u32,o:u32,v:u32)->i32{wprt_bt_ioctlv(r,c,n,o,v)}
#[export_name="_ZN2wp9bluetooth5closeEv"]pub unsafe extern "C" fn close(){wprt_bt_close();}
#[export_name="_ZN2wp9bluetooth6updateEv"]pub unsafe extern "C" fn update(){wprt_bt_update();}
#[export_name="_ZN2wp9bluetooth15take_completionERjRi"]pub unsafe extern "C" fn completion(r:*mut u32,v:*mut i32)->bool{wprt_bt_take_completion(r,v)}
// port of wphw_bt_now/deadline;source target nanosecond duration,500ms exactly.
#[no_mangle]pub unsafe extern "C" fn wphw_bt_now()->i64{comparison_now()}
#[no_mangle]pub extern "C" fn wphw_bt_deadline(now:i64)->i64{now.wrapping_add(500_000_000)}
// port of source DLL-load-time getenv;do not replace with a runtime setting.
static mut BT_LOG:bool=false;
unsafe extern "C" fn initialize(){BT_LOG=!libc::getenv(c"WP_LOG_BT".as_ptr()).is_null();}
#[used]#[link_section=".CRT$XCU"]static INITIALIZE:unsafe extern "C" fn()=initialize;
#[no_mangle]pub unsafe extern "C" fn wphw_bt_log_enabled()->bool{BT_LOG}
// port of original log switch;exact source strings/C promotions/CRT stream.
#[no_mangle]pub unsafe extern "C" fn wphw_bt_log(kind:u32,a:u32,b:u32,c:u32,d:u32){match kind{
0=>{__mingw_fprintf(__acrt_iob_func(2),c"BT event %02x length %u\n".as_ptr(),a,b);},
1=>{__mingw_fprintf(__acrt_iob_func(2),c"BT l2cap send code %02x ident %02x\n".as_ptr(),a,b);},
2=>{__mingw_fprintf(__acrt_iob_func(2),c"BT l2cap receive code %02x ident %02x length %u\n".as_ptr(),a,b,c);},
3=>{__mingw_fprintf(__acrt_iob_func(2),c"BT l2cap connect response dcid %04x scid %04x result %04x status %04x\n".as_ptr(),a,b,c,d);},
4=>{__mingw_fprintf(__acrt_iob_func(2),c"BT l2cap signal %02x not handled\n".as_ptr(),a);},
5=>{__mingw_fprintf(__acrt_iob_func(2),c"BT unhandled data on psm %04x type %02x\n".as_ptr(),a,b);},
6=>{__mingw_fprintf(__acrt_iob_func(2),c"BT command %04x\n".as_ptr(),a);},
7=>{__mingw_fprintf(__acrt_iob_func(2),c"BT unhandled ioctlv %u\n".as_ptr(),a);},
_=>{}}}
// port of real/input/native-controller service forwards.
#[no_mangle]pub unsafe extern "C" fn wphw_bt_real_present(i:u32)->bool{real_present(i)}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_real_wake(i:u32)->bool{real_wake(i)}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_input_connected(i:u32)->bool{input_connected(i)}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_input_wake(i:u32)->bool{input_wake(i)}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_real_start(){real_start();}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_real_reset(i:u32){real_reset(i);}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_wii_reset(i:u32){wii_reset(i);}
// port of original routing;SDK only constructs captured Sender/vector callback,never decides routing.
#[no_mangle]pub unsafe extern "C" fn wphw_bt_output(i:u32,real:bool,p:*const u8,n:u32){if real{real_output(i,p,n);}else{wphw_bt_emulated_output_sdk(i,p,n);}}
#[no_mangle]pub unsafe extern "C" fn wphw_bt_update(i:u32,real:bool){if real{wphw_bt_real_update_sdk(i);}else{wphw_bt_emulated_update_sdk(i);}}
