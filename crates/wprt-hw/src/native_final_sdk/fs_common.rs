
// SPDX-License-Identifier: GPL-3.0-or-later
// Rust ownership/inline-template port for remaining SDK filesystem boundaries.
// Native libstdc++ filesystem/codecvt operations remain explicit dependencies.
// Layout: fs-layout.cpp (GCC15.2 GNU Win64); conversion: bits/locale_conv.h:51-112,
// fs_path.h:785-801,896-935,1109-1135. This is not an alternative Unicode policy.
#![allow(non_snake_case,dead_code)]
use core::ffi::{c_char,c_void};
include!("../native_g60_batch5_files/common/cxx.rs");
#[repr(C)] pub struct WideString{pub p:*mut u16,pub n:usize,pub u:[u64;2]}
#[repr(C)] pub struct FsPath{pub native:WideString,pub parts:*mut c_void}
#[repr(C)]pub struct ErrorCode{pub value:i32,pub category:*const c_void}
#[repr(C,align(8))]struct Codecvt([u8;32]);
extern "C-unwind"{
 #[link_name="_ZNSt7__cxx1112basic_stringIwSt11char_traitsIwESaIwEE9_M_createERyy"]fn wide_create(s:*mut WideString,n:*mut usize,old:usize)->*mut u16;
 #[link_name="_ZNSt7__cxx1112basic_stringIwSt11char_traitsIwESaIwEE10_M_disposeEv"]fn wide_dispose(s:*mut WideString);
 #[link_name="_ZNSt7__cxx1112basic_stringIwSt11char_traitsIwESaIwEE14_M_replace_auxEyyyw"]fn wide_replace(s:*mut WideString,p:usize,n1:usize,n2:usize,ch:u16)->*mut WideString;
 #[link_name="_ZNSt7__cxx1112basic_stringIwSt11char_traitsIwESaIwEE7reserveEy"]fn wide_reserve(s:*mut WideString,n:usize);
 #[link_name="_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE14_M_replace_auxEyyyc"]fn narrow_replace(s:*mut CxxString,p:usize,n1:usize,n2:usize,ch:u8)->*mut CxxString;
 #[link_name="_ZNSt7codecvtIwciEC2Ey"]fn cv_ctor(cv:*mut Codecvt,refs:usize);
 #[link_name="_ZNSt25__codecvt_utf8_utf16_baseIwED2Ev"]fn cv_dtor(cv:*mut Codecvt);
 #[link_name="_ZNKSt25__codecvt_utf8_utf16_baseIwE13do_max_lengthEv"]fn cv_max(cv:*const Codecvt)->i32;
 #[link_name="_ZNKSt25__codecvt_utf8_utf16_baseIwE5do_inERiPKcS3_RS3_PwS5_RS5_"]fn cv_in(cv:*const Codecvt,state:*mut i32,first:*const u8,last:*const u8,next:*mut *const u8,out:*mut u16,end:*mut u16,out_next:*mut *mut u16)->i32;
 #[link_name="_ZNKSt25__codecvt_utf8_utf16_baseIwE6do_outERiPKwS3_RS3_PcS5_RS5_"]fn cv_out(cv:*const Codecvt,state:*mut i32,first:*const u16,last:*const u16,next:*mut *const u16,out:*mut u8,end:*mut u8,out_next:*mut *mut u8)->i32;
 #[link_name="_ZNSt10filesystem7__cxx118__detail24__throw_conversion_errorEv"]fn conversion_error()->!;
 #[link_name="_ZNSt10filesystem7__cxx114path5_ListC1Ev"]fn list_ctor(p:*mut *mut c_void);
 #[link_name="_ZNSt10filesystem7__cxx114path5_ListC1ERKS2_"]fn list_copy(p:*mut *mut c_void,src:*const *mut c_void);
 #[link_name="_ZNKSt10filesystem7__cxx114path5_List13_Impl_deleterclEPNS2_5_ImplE"]fn list_delete(this:*const *mut c_void,p:*mut c_void);
 #[link_name="_ZNSt10filesystem7__cxx114path14_M_split_cmptsEv"]fn split_path(p:*mut FsPath);
 #[link_name="_ZNSt10filesystem7__cxx114pathdVERKS1_"]pub fn path_append(p:*mut FsPath,other:*const FsPath)->*mut FsPath;
 #[link_name="_ZNKSt10filesystem7__cxx114path11parent_pathEv"]pub fn path_parent(out:*mut FsPath,p:*const FsPath)->*mut FsPath;
 #[link_name="_ZNKSt10filesystem7__cxx114path13relative_pathEv"]pub fn path_relative(out:*mut FsPath,p:*const FsPath)->*mut FsPath;
 #[link_name="_ZNKSt10filesystem7__cxx114path16lexically_normalEv"]pub fn path_normal(out:*mut FsPath,p:*const FsPath)->*mut FsPath;
 #[link_name="_ZNKSt10filesystem7__cxx114path5_List5beginEv"]fn path_begin(p:*const *mut c_void)->*const FsComponent;
 #[link_name="_ZNKSt10filesystem7__cxx114path5_List3endEv"]fn path_end(p:*const *mut c_void)->*const FsComponent;
 #[link_name="_ZNSt3_V215system_categoryEv"]pub fn system_category()->*const c_void;
 #[link_name="_ZNSt3_V216generic_categoryEv"]pub fn generic_category()->*const c_void;
 #[link_name="_ZNSt10filesystem6statusERKNS_7__cxx114pathERSt10error_code"]pub fn fs_status(p:*const FsPath,e:*mut ErrorCode)->u64;
 #[link_name="_ZNSt10filesystem9file_sizeERKNS_7__cxx114pathERSt10error_code"]pub fn fs_size(p:*const FsPath,e:*mut ErrorCode)->u64;
 #[link_name="_ZNSt10filesystem18create_directoriesERKNS_7__cxx114pathERSt10error_code"]pub fn fs_mkdir(p:*const FsPath,e:*mut ErrorCode)->bool;
 #[link_name="_ZNSt10filesystem10remove_allERKNS_7__cxx114pathERSt10error_code"]pub fn fs_remove_all(p:*const FsPath,e:*mut ErrorCode)->u64;
 #[link_name="_ZNSt10filesystem6removeERKNS_7__cxx114pathERSt10error_code"]pub fn fs_remove(p:*const FsPath,e:*mut ErrorCode)->bool;
 #[link_name="_ZNSt10filesystem6renameERKNS_7__cxx114pathES3_RSt10error_code"]pub fn fs_rename(a:*const FsPath,b:*const FsPath,e:*mut ErrorCode);
}
#[repr(C)]struct FsComponent{path:FsPath,pos:usize}
struct CvGuard(*mut Codecvt);impl Drop for CvGuard{fn drop(&mut self){unsafe{cv_dtor(self.0)}}}
pub struct WGuard(pub *mut WideString);impl Drop for WGuard{fn drop(&mut self){unsafe{wide_dispose(self.0)}}}
pub struct SGuard(pub *mut CxxString);impl Drop for SGuard{fn drop(&mut self){unsafe{s_dispose(self.0)}}}
pub struct PGuard(pub *mut FsPath);impl Drop for PGuard{fn drop(&mut self){unsafe{path_drop(self.0)}}}
unsafe fn wide_local(s:*mut WideString)->*mut u16{core::ptr::addr_of_mut!((*s).u).cast()}
pub unsafe fn wide_init(s:*mut WideString){(*s).p=wide_local(s);(*s).n=0;*(*s).p=0;}
unsafe fn wide_construct(s:*mut WideString,p:*const u16,n:usize){
 (*s).p=wide_local(s);if p.is_null()&&n!=0{cxx_throw_logic_error(b"basic_string: construction from null is not valid\0".as_ptr().cast())}
 if n>7{let mut cap=n;(*s).p=wide_create(s,&mut cap,0);(*s).u[0]=cap as u64;}
 if n!=0{core::ptr::copy_nonoverlapping(p,(*s).p,n);}(*s).n=n;*(*s).p.add(n)=0;
}
pub unsafe fn wide_resize(s:*mut WideString,n:usize){let old=(*s).n;if n>old{wide_replace(s,old,0,n-old,0);}else{(*s).n=n;*(*s).p.add(n)=0;}}
pub unsafe fn narrow_resize(s:*mut CxxString,n:usize){let old=(*s).n;if n>old{narrow_replace(s,old,0,n-old,0);}else{(*s).n=n;*(*s).p.add(n)=0;}}
unsafe fn cv_init(c:*mut Codecvt){cv_ctor(c,0);c.cast::<u8>().add(24).cast::<u64>().write(0x10ffff);}
pub unsafe fn from_utf8(out:*mut WideString,p:*const u8,n:usize){
 wide_init(out);let wg=WGuard(out);let mut cv=core::mem::MaybeUninit::<Codecvt>::uninit();cv_init(cv.as_mut_ptr());let cg=CvGuard(cv.as_mut_ptr());
 if n!=0{let maxlen=(cv_max(cv.as_ptr())+1)as usize;let last=p.add(n);let mut next=p;let mut state=0_i32;let mut written=0;
 loop{let count=(last as usize-next as usize)*maxlen;wide_resize(out,(*out).n+count);let mut onext=(*out).p.add(written);
 let result=cv_in(cv.as_ptr(),&mut state,next,last,&mut next,onext,(*out).p.add((*out).n),&mut onext);written=onext.offset_from((*out).p)as usize;
 if result==1&&next!=last&&(*out).n-written<maxlen{continue}
 if result==2||next!=last{conversion_error()}wide_resize(out,written);break;}}
 drop(cg);core::mem::forget(wg);
}
pub unsafe fn to_utf8(out:*mut CxxString,p:*const u16,n:usize){
 s_init(out);if n==0{return}let sg=SGuard(out);let mut cv=core::mem::MaybeUninit::<Codecvt>::uninit();cv_init(cv.as_mut_ptr());let cg=CvGuard(cv.as_mut_ptr());
 let maxlen=(cv_max(cv.as_ptr())+1)as usize;let last=p.add(n);let mut next=p;let mut state=0_i32;let mut written=0;
 loop{narrow_resize(out,(*out).n+(last as usize-next as usize)/2*maxlen);let mut onext=(*out).p.add(written);
 let result=cv_out(cv.as_ptr(),&mut state,next,last,&mut next,onext,(*out).p.add((*out).n),&mut onext);written=onext.offset_from((*out).p)as usize;
 if result==1&&next!=last&&(*out).n-written<maxlen{continue}if result==2||next!=last{conversion_error()}narrow_resize(out,written);break;}
 drop(cg);core::mem::forget(sg);
}
pub unsafe fn path_drop(p:*mut FsPath){let parts=(*p).parts;if !parts.is_null(){list_delete(core::ptr::addr_of!((*p).parts),parts);}wide_dispose(core::ptr::addr_of_mut!((*p).native));}
struct IncompletePath{p:*mut FsPath,list_ready:bool}impl Drop for IncompletePath{fn drop(&mut self){unsafe{if self.list_ready{path_drop(self.p)}else{wide_dispose(core::ptr::addr_of_mut!((*self.p).native))}}}}
pub unsafe fn path_from_wide(out:*mut FsPath,p:*const u16,n:usize){
 wide_construct(core::ptr::addr_of_mut!((*out).native),p,n);let mut g=IncompletePath{p:out,list_ready:false};list_ctor(core::ptr::addr_of_mut!((*out).parts));g.list_ready=true;split_path(out);core::mem::forget(g);
}
pub unsafe fn path_from_bytes(out:*mut FsPath,p:*const u8,n:usize){
 from_utf8(core::ptr::addr_of_mut!((*out).native),p,n);let mut g=IncompletePath{p:out,list_ready:false};list_ctor(core::ptr::addr_of_mut!((*out).parts));g.list_ready=true;split_path(out);core::mem::forget(g);
}
pub unsafe fn path_copy(out:*mut FsPath,src:*const FsPath){let v=&(*src).native;wide_construct(core::ptr::addr_of_mut!((*out).native),v.p,v.n);let g=IncompletePath{p:out,list_ready:false};list_copy(core::ptr::addr_of_mut!((*out).parts),core::ptr::addr_of!((*src).parts));core::mem::forget(g);}
pub unsafe fn path_components(p:*const FsPath,mut visit:impl FnMut(*const FsPath)){
 if (*p).parts as usize&3!=0{if (*p).native.n!=0{visit(p);}return}
 let mut at=path_begin(core::ptr::addr_of!((*p).parts));let end=path_end(core::ptr::addr_of!((*p).parts));while at!=end{visit(core::ptr::addr_of!((*at).path));at=at.add(1);}
}
pub unsafe fn path_string(out:*mut CxxString,p:*const FsPath,generic:bool){
 if !generic{to_utf8(out,(*p).native.p,(*p).native.n);return}
 let mut tmp=core::mem::MaybeUninit::<WideString>::uninit();wide_init(tmp.as_mut_ptr());let wg=WGuard(tmp.as_mut_ptr());
 if (*p).parts as usize&3==2{wide_resize(tmp.as_mut_ptr(),1);*(*tmp.as_ptr()).p=47;}
 else{wide_reserve(tmp.as_mut_ptr(),(*p).native.n);let mut slash=false;
 path_components(p,|part|{let ty=(*part).parts as usize&3;
 if ty==2{wide_replace(tmp.as_mut_ptr(),(*tmp.as_ptr()).n,0,1,47);return}
 if slash{wide_replace(tmp.as_mut_ptr(),(*tmp.as_ptr()).n,0,1,47);}
 let src=&(*part).native;let old=(*tmp.as_ptr()).n;wide_resize(tmp.as_mut_ptr(),old+src.n);if src.n!=0{core::ptr::copy_nonoverlapping(src.p,(*tmp.as_ptr()).p.add(old),src.n);}slash=ty==3;});}
 to_utf8(out,(*tmp.as_ptr()).p,(*tmp.as_ptr()).n);drop(wg);
}
pub unsafe fn path_filename(out:*mut FsPath,p:*const FsPath){
 let ty=(*p).parts as usize&3;
 if (*p).native.n!=0&&ty==3{path_copy(out,p);return}
 if ty==0&&(*p).native.n!=0&&*(*p).native.p.add((*p).native.n-1)!=92{
 let end=path_end(core::ptr::addr_of!((*p).parts));let last=end.sub(1);if (*last).path.parts as usize&3==3{path_copy(out,core::ptr::addr_of!((*last).path));return}}
 path_from_wide(out,core::ptr::null(),0);
}
pub unsafe fn error_code()->ErrorCode{ErrorCode{value:0,category:system_category()}}
pub unsafe fn file_type(p:*const FsPath,e:*mut ErrorCode)->i32{fs_status(p,e)as i8 as i32}

pub unsafe fn path_default(out:*mut FsPath){wide_init(core::ptr::addr_of_mut!((*out).native));list_ctor(core::ptr::addr_of_mut!((*out).parts));}
pub unsafe fn path_move_assign(out:*mut FsPath,src:*mut FsPath){
 if out==src{return}let d=core::ptr::addr_of_mut!((*out).native);let s=core::ptr::addr_of_mut!((*src).native);
 if (*s).p==wide_local(s){let n=(*s).n;if n!=0{core::ptr::copy_nonoverlapping((*s).p,(*d).p,n);}(*d).n=n;*(*d).p.add(n)=0;}
 else{let old=if (*d).p==wide_local(d){core::ptr::null_mut()}else{(*d).p};let cap=(*d).u[0];(*d).p=(*s).p;(*d).u[0]=(*s).u[0];(*d).n=(*s).n;
 if old.is_null(){(*s).p=wide_local(s);}else{(*s).p=old;(*s).u[0]=cap;}}
 (*s).n=0;*(*s).p=0;let old=(*out).parts;(*out).parts=(*src).parts;(*src).parts=core::ptr::null_mut();
 if !old.is_null(){list_delete(core::ptr::addr_of!((*out).parts),old);}split_path(src);
}
pub unsafe fn path_move_construct(out:*mut FsPath,src:*mut FsPath){
 let d=core::ptr::addr_of_mut!((*out).native);let s=core::ptr::addr_of_mut!((*src).native);(*d).p=wide_local(d);
 if (*s).p==wide_local(s){core::ptr::copy_nonoverlapping((*s).p,(*d).p,(*s).n+1);}else{(*d).p=(*s).p;(*d).u[0]=(*s).u[0];}
 (*d).n=(*s).n;(*s).p=wide_local(s);(*s).n=0;*(*s).p=0;(*out).parts=(*src).parts;(*src).parts=core::ptr::null_mut();split_path(src);
}
#[repr(C)]pub struct WideView{pub n:usize,pub p:*const u16}
extern "C-unwind"{#[link_name="_ZNSt10filesystem7__cxx114path9_M_concatESt17basic_string_viewIwSt11char_traitsIwEE"]pub fn path_concat(p:*mut FsPath,view:WideView);}
pub unsafe fn path_suffix(p:*mut FsPath,bytes:*const u8,n:usize){let mut w=core::mem::MaybeUninit::<WideString>::uninit();from_utf8(w.as_mut_ptr(),bytes,n);let guard=WGuard(w.as_mut_ptr());path_concat(p,WideView{n:(*w.as_ptr()).n,p:(*w.as_ptr()).p});drop(guard);}
#[repr(C)]pub struct DirectoryIterator{pub object:*mut c_void,pub control:*mut c_void}
extern "C-unwind"{
 #[link_name="_ZNSt10filesystem7__cxx1118directory_iteratorC1ERKNS0_4pathENS_17directory_optionsEPSt10error_code"]pub fn dir_construct(out:*mut DirectoryIterator,p:*const FsPath,options:u8,e:*mut ErrorCode);
 #[link_name="_ZNSt10filesystem7__cxx1118directory_iteratorppEv"]pub fn dir_next(it:*mut DirectoryIterator)->*mut DirectoryIterator;
 #[link_name="_ZNKSt10filesystem7__cxx1118directory_iteratordeEv"]pub fn dir_entry(it:*const DirectoryIterator)->*const FsPath;
 #[link_name="_ZNSt10filesystem7__cxx1128recursive_directory_iteratorC1ERKNS0_4pathENS_17directory_optionsEPSt10error_code"]pub fn recursive_construct(out:*mut DirectoryIterator,p:*const FsPath,options:u8,e:*mut ErrorCode);
 #[link_name="_ZNSt10filesystem7__cxx1128recursive_directory_iteratorD1Ev"]pub fn recursive_destruct(it:*mut DirectoryIterator);
 #[link_name="_ZNSt10filesystem7__cxx1128recursive_directory_iteratorppEv"]pub fn recursive_next(it:*mut DirectoryIterator)->*mut DirectoryIterator;
 #[link_name="_ZNKSt10filesystem7__cxx1128recursive_directory_iteratordeEv"]pub fn recursive_entry(it:*const DirectoryIterator)->*const FsPath;
 #[link_name="_ZNSt16_Sp_counted_baseILN9__gnu_cxx12_Lock_policyE2EE10_M_releaseEv"]fn shared_release(p:*mut c_void);
}
pub struct DGuard(pub *mut DirectoryIterator);impl Drop for DGuard{fn drop(&mut self){unsafe{let c=(*self.0).control;if !c.is_null(){shared_release(c);}}}}
pub struct RGuard(pub *mut DirectoryIterator);impl Drop for RGuard{fn drop(&mut self){unsafe{recursive_destruct(self.0)}}}
pub unsafe fn entry_file_type(entry:*const FsPath,e:*mut ErrorCode)->i32{
 let cached=entry.cast::<u8>().add(40).cast::<i8>().read() as i32;if cached!=0&&cached!=3{(*e).value=0;(*e).category=system_category();cached}else{file_type(entry,e)}
}
