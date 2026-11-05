// SPDX-License-Identifier: GPL-3.0-or-later
// Whole mii_sdk residue: field-vector ownership and Database::save filesystem callback.
use super::filesystem::fs::*;
use core::{ffi::c_void,mem::MaybeUninit};
use super::eh;
#[repr(C)]struct Field{key:*const u8,values:[u8;6],padding:[u8;2]}unsafe impl Sync for Field{}
static FIELD_VALUES:&[Field]=&[
 Field{key:b"gender\0".as_ptr(),values:[0, 2, 14, 1, 1, 0],padding:[0;2]},
 Field{key:b"birth_month\0".as_ptr(),values:[0, 2, 10, 4, 12, 0],padding:[0;2]},
 Field{key:b"birth_day\0".as_ptr(),values:[0, 2, 5, 5, 31, 0],padding:[0;2]},
 Field{key:b"favorite_color\0".as_ptr(),values:[0, 2, 1, 4, 11, 0],padding:[0;2]},
 Field{key:b"favorite\0".as_ptr(),values:[0, 2, 0, 1, 1, 0],padding:[0;2]},
 Field{key:b"height\0".as_ptr(),values:[22, 1, 0, 7, 127, 0],padding:[0;2]},
 Field{key:b"weight\0".as_ptr(),values:[23, 1, 0, 7, 127, 0],padding:[0;2]},
 Field{key:b"face_shape\0".as_ptr(),values:[32, 2, 13, 3, 7, 0],padding:[0;2]},
 Field{key:b"skin_color\0".as_ptr(),values:[32, 2, 10, 3, 5, 0],padding:[0;2]},
 Field{key:b"facial_feature\0".as_ptr(),values:[32, 2, 6, 4, 11, 0],padding:[0;2]},
 Field{key:b"hair_type\0".as_ptr(),values:[34, 2, 9, 7, 71, 0],padding:[0;2]},
 Field{key:b"hair_color\0".as_ptr(),values:[34, 2, 6, 3, 7, 0],padding:[0;2]},
 Field{key:b"hair_flip\0".as_ptr(),values:[34, 2, 5, 1, 1, 0],padding:[0;2]},
 Field{key:b"eyebrow_type\0".as_ptr(),values:[36, 4, 27, 5, 23, 0],padding:[0;2]},
 Field{key:b"eyebrow_rotation\0".as_ptr(),values:[36, 4, 22, 4, 11, 0],padding:[0;2]},
 Field{key:b"eyebrow_color\0".as_ptr(),values:[36, 4, 13, 3, 7, 0],padding:[0;2]},
 Field{key:b"eyebrow_size\0".as_ptr(),values:[36, 4, 9, 4, 8, 0],padding:[0;2]},
 Field{key:b"eyebrow_vertical\0".as_ptr(),values:[36, 4, 4, 5, 18, 3],padding:[0;2]},
 Field{key:b"eyebrow_spacing\0".as_ptr(),values:[36, 4, 0, 4, 12, 0],padding:[0;2]},
 Field{key:b"eye_type\0".as_ptr(),values:[40, 4, 26, 6, 47, 0],padding:[0;2]},
 Field{key:b"eye_rotation\0".as_ptr(),values:[40, 4, 21, 3, 7, 0],padding:[0;2]},
 Field{key:b"eye_vertical\0".as_ptr(),values:[40, 4, 16, 5, 18, 0],padding:[0;2]},
 Field{key:b"eye_color\0".as_ptr(),values:[40, 4, 13, 3, 5, 0],padding:[0;2]},
 Field{key:b"eye_size\0".as_ptr(),values:[40, 4, 9, 3, 7, 0],padding:[0;2]},
 Field{key:b"eye_spacing\0".as_ptr(),values:[40, 4, 5, 4, 12, 0],padding:[0;2]},
 Field{key:b"nose_type\0".as_ptr(),values:[44, 2, 12, 4, 11, 0],padding:[0;2]},
 Field{key:b"nose_size\0".as_ptr(),values:[44, 2, 8, 4, 8, 0],padding:[0;2]},
 Field{key:b"nose_vertical\0".as_ptr(),values:[44, 2, 3, 5, 18, 0],padding:[0;2]},
 Field{key:b"mouth_type\0".as_ptr(),values:[46, 2, 11, 5, 23, 0],padding:[0;2]},
 Field{key:b"mouth_color\0".as_ptr(),values:[46, 2, 9, 2, 2, 0],padding:[0;2]},
 Field{key:b"mouth_size\0".as_ptr(),values:[46, 2, 5, 4, 8, 0],padding:[0;2]},
 Field{key:b"mouth_vertical\0".as_ptr(),values:[46, 2, 0, 5, 18, 0],padding:[0;2]},
 Field{key:b"glasses_type\0".as_ptr(),values:[48, 2, 12, 4, 8, 0],padding:[0;2]},
 Field{key:b"glasses_color\0".as_ptr(),values:[48, 2, 9, 3, 5, 0],padding:[0;2]},
 Field{key:b"glasses_size\0".as_ptr(),values:[48, 2, 5, 3, 7, 0],padding:[0;2]},
 Field{key:b"glasses_vertical\0".as_ptr(),values:[48, 2, 0, 5, 20, 0],padding:[0;2]},
 Field{key:b"mustache_type\0".as_ptr(),values:[50, 2, 14, 2, 3, 0],padding:[0;2]},
 Field{key:b"beard_type\0".as_ptr(),values:[50, 2, 12, 2, 3, 0],padding:[0;2]},
 Field{key:b"facial_hair_color\0".as_ptr(),values:[50, 2, 9, 3, 7, 0],padding:[0;2]},
 Field{key:b"mustache_size\0".as_ptr(),values:[50, 2, 5, 4, 8, 0],padding:[0;2]},
 Field{key:b"mustache_vertical\0".as_ptr(),values:[50, 2, 0, 5, 16, 0],padding:[0;2]},
 Field{key:b"mole\0".as_ptr(),values:[52, 2, 15, 1, 1, 0],padding:[0;2]},
 Field{key:b"mole_size\0".as_ptr(),values:[52, 2, 11, 4, 8, 0],padding:[0;2]},
 Field{key:b"mole_vertical\0".as_ptr(),values:[52, 2, 6, 5, 30, 0],padding:[0;2]},
 Field{key:b"mole_horizontal\0".as_ptr(),values:[52, 2, 1, 5, 16, 0],padding:[0;2]},
];
static mut FIELDS:CxxVec=CxxVec{b:core::ptr::null_mut(),e:core::ptr::null_mut(),c:core::ptr::null_mut()};
extern "C"{fn atexit(f:unsafe extern "C" fn())->i32;fn abort()->!;}
unsafe extern "C" fn field_destruct(){v_free(core::ptr::addr_of_mut!(FIELDS));}
unsafe extern "C-unwind" fn field_construct(){let n=FIELD_VALUES.len()*core::mem::size_of::<Field>();let p=cxx_new(n);core::ptr::copy_nonoverlapping(FIELD_VALUES.as_ptr().cast::<u8>(),p,n);FIELDS=CxxVec{b:p,e:p.add(n),c:p.add(n)};atexit(field_destruct);}
#[used]#[link_section=".ctors"]static FIELD_INIT:unsafe extern "C-unwind" fn()=field_construct;
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_mii_kfields()->*const CxxVec{core::ptr::addr_of!(FIELDS)}
#[repr(C)]struct MiiPath{p:*const u16,n:usize}
extern "C-unwind"{fn wprt_mii_save_file(p:*const u8,n:usize,valid:*mut bool,existed:*mut bool,c:*mut c_void,op:unsafe extern "C" fn(*mut c_void,i32,*mut MiiPath)->i32)->bool;}
struct SaveCtx{path:*const FsPath,temp:MaybeUninit<FsPath>,error:ErrorCode,exception:*mut c_void}
unsafe extern "C-unwind" fn save_operation_body(raw:*mut c_void,op:i32,span:*mut MiiPath)->i32{let c=&mut*raw.cast::<SaveCtx>();match op{
 0=>{let mut parent=MaybeUninit::<FsPath>::uninit();path_parent(parent.as_mut_ptr(),c.path);let p=PGuard(parent.as_mut_ptr());fs_mkdir(parent.as_ptr(),&mut c.error);drop(p);0},
 1=>{let mut tmp=MaybeUninit::<FsPath>::uninit();path_copy(tmp.as_mut_ptr(),c.path);let g=PGuard(tmp.as_mut_ptr());path_move_assign(c.temp.as_mut_ptr(),tmp.as_mut_ptr());drop(g);path_suffix(c.temp.as_mut_ptr(),b".tmp".as_ptr(),4);(*span).p=(*c.temp.as_ptr()).native.p;(*span).n=(*c.temp.as_ptr()).native.n;0},
 2=>{fs_rename(c.temp.as_ptr(),c.path,&mut c.error);(c.error.value!=0)as i32},
 3=>{fs_remove(c.temp.as_ptr(),&mut c.error);0},_=>eh::terminate(),
}}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_mii_save_io(p:*const u8,n:usize,valid:*mut bool,existed:*mut bool,path:*const FsPath)->bool{
 let mut c=SaveCtx{path,temp:MaybeUninit::uninit(),error:error_code(),exception:core::ptr::null_mut()};path_default(c.temp.as_mut_ptr());let g=PGuard(c.temp.as_mut_ptr());let eg=eh::PtrGuard(core::ptr::addr_of_mut!(c.exception));let result=wprt_mii_save_file(p,n,valid,existed,(&mut c as*mut SaveCtx).cast(),save_operation);if !c.exception.is_null(){eh::rethrow(core::ptr::addr_of_mut!(c.exception));}drop(eg);drop(g);result
}

struct SaveCall{raw:*mut c_void,op:i32,span:*mut MiiPath}
unsafe extern "C-unwind" fn save_invoke(p:*mut c_void)->i32{let call=&*p.cast::<SaveCall>();save_operation_body(call.raw,call.op,call.span)}
unsafe extern "C" fn save_operation(raw:*mut c_void,op:i32,span:*mut MiiPath)->i32{let mut call=SaveCall{raw,op,span};eh::capture(save_invoke,(&mut call as*mut SaveCall).cast(),core::ptr::addr_of_mut!((*raw.cast::<SaveCtx>()).exception))}
