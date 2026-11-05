// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted entire owned screenshot_rust.cpp adapter policy;native stdstring/stdtm namedfield SDK retained.
use std::ffi::{c_char,c_void};
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C"{fn wphw_screenshot_tm_sdk(raw:*const c_void,fields:*mut i32);fn wprt_screenshot_file_name(prefix:*const u8,n:usize,year:i32,month:i32,day:i32,hour:i32,minute:i32,second:i32,attempt:i32,context:*mut c_void,sink:Sink);fn strlen(p:*const c_char)->usize;}
// port of file_name forwarding;SAME CRT strlen and actual named stdtm fields,not a guessed struct layout.
#[no_mangle]pub unsafe extern "C" fn wphw_screenshot_adapter(prefix:*const c_char,time:*const c_void,attempt:i32,context:*mut c_void,sink:Sink){let n=strlen(prefix);let mut f=[0i32;6];wphw_screenshot_tm_sdk(time,f.as_mut_ptr());wprt_screenshot_file_name(prefix.cast(),n,f[0],f[1],f[2],f[3],f[4],f[5],attempt,context,sink)}
