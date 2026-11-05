// SPDX-License-Identifier: GPL-3.0-or-later
// Runtime acceptance tracked in HANDOFF.md. port of platform/save_backup.cpp.
use std::ffi::c_void;
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C"{fn wphw_saves_isdigit(value:u8)->bool;fn wphw_saves_read_pair(c:*mut c_void,out:*mut Pair);fn wphw_saves_eof(c:*mut c_void,right:bool)->bool;fn wphw_saves_bad(c:*mut c_void,right:bool)->bool;fn wphw_saves_name_exists(c:*mut c_void,p:*const u8,n:usize,partial:bool)->bool;}
unsafe fn bytes<'a>(p:*const u8,n:usize)->&'a[u8]{if n==0{&[]}else{std::slice::from_raw_parts(p,n)}}
// port of save_backup.cpp stamp_name. Output suffix is modified at the same source branches,including failures.
#[no_mangle]pub unsafe extern "C" fn wprt_saves_stamp_name(p:*const u8,n:usize,suffix:*mut u32)->bool{let name=bytes(p,n);if name.len()<STAMP.len(){return false}for i in 0..STAMP.len(){let digit=wphw_saves_isdigit(name[i]);if if STAMP[i]==b'0'{!digit}else{name[i]!=STAMP[i]}{return false}}*suffix=1;if name.len()==STAMP.len(){return true}if name[STAMP.len()]!=b'_'||name.len()==STAMP.len()+1||name.len()>STAMP.len()+1+MAXIMUM_SUFFIX_DIGITS{return false}*suffix=0;for &ch in &name[STAMP.len()+1..]{if !wphw_saves_isdigit(ch){return false}*suffix=(*suffix).wrapping_mul(10).wrapping_add((ch as i8 as i32).wrapping_sub(b'0' as i32)as u32)}true}
// port of save_backup.cpp ascii:Windows native path characters are UTF16 units,not Unicode scalar values.
#[no_mangle]pub unsafe extern "C" fn wprt_saves_ascii(p:*const u16,n:usize,sink:Sink,c:*mut c_void){let chars=if n==0{&[]}else{std::slice::from_raw_parts(p,n)};let mut value=Vec::new();for &ch in chars{value.push(if (ch as u32)<0x80{ch as u8}else{b'?'})}sink(c,value.as_ptr(),value.len())}
// port of save_backup.cpp save_relative's %08x name. Native wrapper retains all fs::path joins.
#[no_mangle]pub unsafe extern "C" fn wprt_saves_title_name(title:u32,sink:Sink,c:*mut c_void){let name=format!("{:08x}",title);sink(c,name.as_ptr(),name.len())}
#[repr(C)]pub struct Pair{left:*const u8,right:*const u8,count:i64,right_count:i64}
// port of save_backup.cpp files_equal after native stream-open and vector allocation checks.
// Read-pair callback ends before buffers are borrowed;the borrows end before subsequent native reads.
#[no_mangle]pub unsafe extern "C" fn wprt_saves_files_equal(c:*mut c_void)->bool{loop{let mut pair=Pair{left:std::ptr::null(),right:std::ptr::null(),count:0,right_count:0};wphw_saves_read_pair(c,&mut pair);if pair.count!=pair.right_count{return false}let equal={bytes(pair.left,pair.count as usize)==bytes(pair.right,pair.count as usize)};if !equal{return false}if pair.count==0||wphw_saves_eof(c,false)||wphw_saves_eof(c,true){return wphw_saves_eof(c,false)==wphw_saves_eof(c,true)&&!wphw_saves_bad(c,false)&&!wphw_saves_bad(c,true)}}}
// port of save_backup.cpp unique_name. Native exists callback preserves original shared error_code and short-circuit.
#[no_mangle]pub unsafe extern "C" fn wprt_saves_unique_name(c:*mut c_void,p:*const u8,n:usize,sink:Sink,out:*mut c_void){let base=bytes(p,n);let mut name=base.to_vec();let mut number=2i32;while wphw_saves_name_exists(c,name.as_ptr(),name.len(),false)||wphw_saves_name_exists(c,name.as_ptr(),name.len(),true){name=base.to_vec();name.push(b'_');name.extend_from_slice(number.to_string().as_bytes());number=number.wrapping_add(1)}sink(out,name.as_ptr(),name.len())}
const STAMP:&[u8]=b"0000-00-00_00-00-00";
const MAXIMUM_SUFFIX_DIGITS:usize=6;
