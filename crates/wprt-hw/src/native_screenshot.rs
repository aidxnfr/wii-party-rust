// SPDX-License-Identifier: GPL-3.0-or-later
// port of engine/src/platform/screenshot.cpp file_name. Runtime acceptance tracked in HANDOFF.md.
// Native std::tm/std::string storage stays native;the callback copies synchronously.
#[no_mangle]
pub unsafe extern "C" fn wprt_screenshot_file_name(prefix:*const u8,n:usize,year:i32,month:i32,day:i32,hour:i32,minute:i32,second:i32,attempt:i32,context:*mut std::ffi::c_void,emit:unsafe extern "C" fn(*mut std::ffi::c_void,*const u8,usize)){
    let prefix=if n==0{&[]}else{std::slice::from_raw_parts(prefix,n)};
    let stamp=format!("_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",year.wrapping_add(1900),month.wrapping_add(1),day,hour,minute,second);
    let mut name=prefix.to_vec();name.extend_from_slice(stamp.as_bytes());
    if attempt>1{name.push(b'_');name.extend_from_slice(attempt.to_string().as_bytes());}
    name.extend_from_slice(b".png");emit(context,name.as_ptr(),name.len());
}
