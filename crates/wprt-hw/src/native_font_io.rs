// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of options_render_stb.cpp:90-98 font stream/assign lifetime.
// SDK paths: GCC15.2 basic_file_stdio.cc:114-143,236-249,312-329;
// vector.tcc:300-315,865-879;target os_defines.h:97 and errno.h:28.
// UNINTEGRATED. Native vector/Font/STB ownership and exception reporting remain explicit.
use std::ffi::c_void;
extern "C"{fn fopen(path:*const u8,mode:*const u8)->*mut c_void;fn fclose(file:*mut c_void)->i32;fn _fileno(file:*mut c_void)->i32;fn _read(fd:i32,data:*mut c_void,n:u32)->i32;fn _errno()->*mut i32;}
struct FontFile(*mut c_void);
impl Drop for FontFile{fn drop(&mut self){unsafe{fclose(self.0);}}}
#[repr(C)]pub struct FontReadOps{
    overwrite:unsafe extern "C" fn(*mut c_void,usize,*const u8,usize)->i32,
    append:unsafe extern "C" fn(*mut c_void,*const u8,usize)->i32,
    truncate:unsafe extern "C" fn(*mut c_void,usize)->i32,
    finish:unsafe extern "C" fn(*mut c_void)->i32,
}
// port of the ifstream/string overload and input-iterator vector::assign,not read-then-replace.
// Preserve partial overwritten/appended Font.data on a read error;do not trim on failure.
// Result -1 requests native ios_failure;-2 carries a captured native exception;both rethrow AFTER Rust returns.
#[no_mangle]pub unsafe extern "C" fn wprt_font_read(path:*const u8,n:usize,c:*mut c_void,old_len:usize,ops:*const FontReadOps,error:*mut i32)->i32{
    let ops=&*ops;let mut name=if n==0{Vec::new()}else{std::slice::from_raw_parts(path,n).to_vec()};name.push(0);
    let raw=fopen(name.as_ptr(),b"rb\0".as_ptr());
    if raw.is_null(){
        // Source failed-open stream still assigns an empty sequence and attempts STB init.
        // No invented fallback/empty-font guard;native vector capacity remains intact.
        if (ops.truncate)(c,0)<0{return -2}let done=(ops.finish)(c);return if done<0{-2}else{done}
    }
    let file=FontFile(raw);let fd=_fileno(file.0);let mut cursor=0usize;let mut buf=[0u8;4095];
    loop{
        let count=loop{let count=_read(fd,buf.as_mut_ptr().cast(),buf.len() as u32);if count != -1 || *_errno()!=4{break count}};
        if count<0{*error=*_errno();return -1}
        if count==0{break}
        let count=count as usize;let replace=old_len.saturating_sub(cursor).min(count);
        if replace!=0 && (ops.overwrite)(c,cursor,buf.as_ptr(),replace)<0{return -2}
        if replace<count && (ops.append)(c,buf.as_ptr().add(replace),count-replace)<0{return -2}
        cursor+=count;
    }
    if cursor<old_len && (ops.truncate)(c,cursor)<0{return -2}
    // Original ifstream remains alive during offset/init/logging;Drop only after finish.
    let done=(ops.finish)(c);if done<0{-2}else{done}
}
