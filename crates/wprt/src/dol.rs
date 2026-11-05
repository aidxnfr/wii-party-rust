// SPDX-License-Identifier: GPL-3.0-or-later
// port of engine/src/core/dol.cpp; C stdio keeps the pristine path/read semantics.
use std::ffi::{CStr,c_char,c_void};
use crate::memory::host;
extern "C"{fn fopen(path:*const c_char,mode:*const c_char)->*mut c_void;fn fread(dst:*mut c_void,size:usize,count:usize,file:*mut c_void)->usize;fn fclose(file:*mut c_void)->i32;}
// port of engine/src/core/dol.cpp be32.
fn be32(data:&[u8],offset:usize)->u32{u32::from_be_bytes(data[offset..offset+4].try_into().unwrap())}
// port of engine/src/core/dol.cpp load_dol.
pub fn load_dol(path:&CStr,entry:&mut u32)->bool{unsafe{
 let file=fopen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return false;}
 let mut data=Vec::new();let mut buffer=[0u8;65536];loop{let count=fread(buffer.as_mut_ptr().cast(),1,buffer.len(),file);if count==0{break;}data.extend_from_slice(&buffer[..count]);}fclose(file);
 if data.len()<0x100{return false;}
 for i in 0..18{let offset=be32(&data,i*4);let address=be32(&data,0x48+i*4);let size=be32(&data,0x90+i*4);if size==0{continue;}if offset.wrapping_add(size)as usize>data.len(){return false;}std::ptr::copy_nonoverlapping(data.as_ptr().add(offset as usize),host(address),size as usize);}
 *entry=be32(&data,0xe0);true
}}
