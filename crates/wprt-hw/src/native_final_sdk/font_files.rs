
// SPDX-License-Identifier: GPL-3.0-or-later
// Whole options_render_sdk font lifetime/file boundary in Rust; original callbacks and STB backend.
// Compiler layout: Font48/data0/info24(pointer32)/ok40; source residue.cpp:107-166.
use super::filesystem::fs::*;
use super::eh;
use core::{ffi::{c_char,c_void},mem::MaybeUninit,sync::atomic::{AtomicU8,Ordering}};
#[repr(C)]struct Font{data:CxxVec,deleter:unsafe extern "C-unwind" fn(*mut c_void),info:*mut c_void,ok:bool}
#[repr(C)]#[derive(Clone,Copy)]struct Bytes{p:*const u8,n:usize}
#[repr(C)]struct FontReadOps{overwrite:unsafe extern "C" fn(*mut c_void,usize,*const u8,usize)->i32,append:unsafe extern "C" fn(*mut c_void,*const u8,usize)->i32,truncate:unsafe extern "C" fn(*mut c_void,usize)->i32,finish:unsafe extern "C" fn(*mut c_void)->i32}
extern "C-unwind"{
 fn wprt_hw_stb_free(p:*mut c_void);fn wprt_hw_stb_offset(p:*const u8,face:i32)->i32;fn wprt_hw_stb_init(p:*const u8,offset:i32)->*mut c_void;
 fn wprt_font_read(path:*const u8,n:usize,c:*mut c_void,old:usize,ops:*const FontReadOps,error:*mut i32)->i32;
 fn wprt_options_load_font(bold:bool,windows:*const u8,n:usize,has:bool,attempt:unsafe extern "C-unwind" fn(*mut c_void,Bytes)->bool,c:*mut c_void)->bool;
 fn wphw_options_font_result(ok:bool);fn wphw_options_font_bold(bold:bool,ok:bool)->bool;
 #[link_name="_ZSt19__throw_ios_failurePKci"]fn ios_failure(m:*const c_char,e:i32)->!;
 #[link_name="_ZN2wp3log5writeEPKcS2_z"]fn font_log(category:*const c_char,format:*const c_char,path:*const c_char);
 fn __cxa_guard_acquire(g:*mut i64)->i32;fn __cxa_guard_release(g:*mut i64);fn __cxa_guard_abort(g:*mut i64);
}
extern "C"{fn getenv(p:*const c_char)->*mut c_char;fn strlen(p:*const c_char)->usize;fn atexit(f:unsafe extern "C" fn())->i32;}
const fn empty_font()->Font{Font{data:CxxVec{b:core::ptr::null_mut(),e:core::ptr::null_mut(),c:core::ptr::null_mut()},deleter:wprt_hw_stb_free,info:core::ptr::null_mut(),ok:false}}
static mut REGULAR:Font=empty_font();static mut SEMIBOLD:Font=empty_font();static mut REGULAR_GUARD:i64=0;static mut SEMIBOLD_GUARD:i64=0;
unsafe fn font_drop(f:*mut Font){if !(*f).info.is_null(){((*f).deleter)((*f).info);}v_free(core::ptr::addr_of_mut!((*f).data));}
struct FontGuard(*mut Font);impl Drop for FontGuard{fn drop(&mut self){unsafe{font_drop(self.0)}}}
struct AbortGuard(*mut i64);impl Drop for AbortGuard{fn drop(&mut self){unsafe{__cxa_guard_abort(self.0)}}}
struct ReadCtx{font:*mut Font,path:*const CxxString,exception:*mut c_void}
unsafe extern "C-unwind" fn overwrite_body(raw:*mut c_void,off:usize,p:*const u8,n:usize)->i32{let c=&mut*raw.cast::<ReadCtx>();core::ptr::copy_nonoverlapping(p,(*c.font).data.b.add(off),n);0}
unsafe extern "C-unwind" fn append_body(raw:*mut c_void,p:*const u8,n:usize)->i32{let v=core::ptr::addr_of_mut!((*(*raw.cast::<ReadCtx>()).font).data);
 for i in 0..n{if (*v).e!=(*v).c{*(*v).e=*p.add(i);(*v).e=(*v).e.add(1);}else{let old=v_len(v,1);let(new,cap)=v_grow(v,1,b"vector::_M_realloc_insert\0".as_ptr().cast());*new.add(old)=*p.add(i);v_finish(v,1,new,cap,None);}}0
}
unsafe extern "C-unwind" fn truncate_body(raw:*mut c_void,n:usize)->i32{let v=core::ptr::addr_of_mut!((*(*raw.cast::<ReadCtx>()).font).data);let old=v_len(v,1);
 if n<=old{(*v).e=(*v).b.add(n);}else{let add=n-old;if add>isize::MAX as usize-old{cxx_throw_length_error(b"vector::_M_default_append\0".as_ptr().cast())}
 if (*v).c as usize-(*v).e as usize>=add{core::ptr::write_bytes((*v).e,0,add);(*v).e=(*v).b.add(n);}else{let growth=old+old.max(add);let cap=if growth<old||growth>isize::MAX as usize{isize::MAX as usize}else{growth};let new=cxx_new(cap);core::ptr::write_bytes(new.add(old),0,add);if old!=0{core::ptr::copy_nonoverlapping((*v).b,new,old);}v_free(v);*v=CxxVec{b:new,e:new.add(n),c:new.add(cap)};}}0
}
unsafe extern "C-unwind" fn finish_body(raw:*mut c_void)->i32{let c=&mut*raw.cast::<ReadCtx>();let f=&mut*c.font;let offset=wprt_hw_stb_offset(f.data.b,0);if offset>=0{let next=wprt_hw_stb_init(f.data.b,offset);let old=f.info;f.info=next;if !old.is_null(){(f.deleter)(old)}if !next.is_null(){f.ok=true;font_log(b"options\0".as_ptr().cast(),b"menu font %s\0".as_ptr().cast(),(*c.path).p.cast());return 1}}0}
static READ_OPS:FontReadOps=FontReadOps{overwrite,append,truncate,finish};
unsafe extern "C-unwind" fn try_font(raw:*mut c_void,bytes:Bytes)->bool{
 let f=raw.cast::<Font>();let mut text=MaybeUninit::<CxxString>::uninit();s_construct(text.as_mut_ptr(),bytes.p,bytes.n);let sg=SGuard(text.as_mut_ptr());if (*text.as_ptr()).n==0{drop(sg);return false}
 let mut path=MaybeUninit::<FsPath>::uninit();path_from_bytes(path.as_mut_ptr(),(*text.as_ptr()).p,(*text.as_ptr()).n);let pg=PGuard(path.as_mut_ptr());let mut error=error_code();let regular=file_type(path.as_ptr(),&mut error)==1;drop(pg);if !regular{drop(sg);return false}
 let mut context=ReadCtx{font:f,path:text.as_ptr(),exception:core::ptr::null_mut()};let eg=eh::PtrGuard(core::ptr::addr_of_mut!(context.exception));let mut io_error=0;let result=wprt_font_read((*text.as_ptr()).p,(*text.as_ptr()).n,(&mut context as*mut ReadCtx).cast(),v_len(core::ptr::addr_of!((*f).data),1),&READ_OPS,&mut io_error);
 if !context.exception.is_null(){eh::rethrow(core::ptr::addr_of_mut!(context.exception));}if result<0{ios_failure(b"basic_filebuf::underflow error reading the file\0".as_ptr().cast(),io_error)}drop(eg);drop(sg);result!=0
}
unsafe fn load_font(out:*mut Font,bold:bool){out.write(empty_font());let fg=FontGuard(out);let windows=getenv(b"WINDIR\0".as_ptr().cast());let ok=wprt_options_load_font(bold,windows.cast(),if windows.is_null(){0}else{strlen(windows)},!windows.is_null(),try_font,out.cast());wphw_options_font_result(ok);core::mem::forget(fg);}
unsafe extern "C" fn regular_destroy(){font_drop(core::ptr::addr_of_mut!(REGULAR))}
unsafe extern "C" fn semibold_destroy(){font_drop(core::ptr::addr_of_mut!(SEMIBOLD))}
unsafe fn init_font(g:*mut i64,out:*mut Font,bold:bool,dtor:unsafe extern "C" fn()){
 if (*(g.cast::<AtomicU8>())).load(Ordering::Acquire)==0&&__cxa_guard_acquire(g)!=0{let guard=AbortGuard(g);load_font(out,bold);atexit(dtor);core::mem::forget(guard);__cxa_guard_release(g);}
}
unsafe fn font(bold:bool)->*const Font{init_font(core::ptr::addr_of_mut!(REGULAR_GUARD),core::ptr::addr_of_mut!(REGULAR),false,regular_destroy);init_font(core::ptr::addr_of_mut!(SEMIBOLD_GUARD),core::ptr::addr_of_mut!(SEMIBOLD),true,semibold_destroy);if wphw_options_font_bold(bold,SEMIBOLD.ok){core::ptr::addr_of!(SEMIBOLD)}else{core::ptr::addr_of!(REGULAR)}}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_or_font(bold:bool)->*const c_void{font(bold).cast()}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_or_font_info(f:*const c_void)->*mut c_void{(*f.cast::<Font>()).info}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_or_font_ok(f:*const c_void)->bool{(*f.cast::<Font>()).ok}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_options_regular_font_sdk()->bool{(*font(false)).ok}

struct ReadCall{raw:*mut c_void,kind:u8,offset:usize,p:*const u8,n:usize}
unsafe extern "C-unwind" fn read_invoke(p:*mut c_void)->i32{let c=&*p.cast::<ReadCall>();match c.kind{0=>overwrite_body(c.raw,c.offset,c.p,c.n),1=>append_body(c.raw,c.p,c.n),2=>truncate_body(c.raw,c.n),_=>finish_body(c.raw)}}
unsafe fn read_capture(mut call:ReadCall)->i32{let slot=core::ptr::addr_of_mut!((*call.raw.cast::<ReadCtx>()).exception);eh::capture(read_invoke,(&mut call as*mut ReadCall).cast(),slot)}
unsafe extern "C" fn overwrite(raw:*mut c_void,offset:usize,p:*const u8,n:usize)->i32{read_capture(ReadCall{raw,kind:0,offset,p,n})}
unsafe extern "C" fn append(raw:*mut c_void,p:*const u8,n:usize)->i32{read_capture(ReadCall{raw,kind:1,offset:0,p,n})}
unsafe extern "C" fn truncate(raw:*mut c_void,n:usize)->i32{read_capture(ReadCall{raw,kind:2,offset:0,p:core::ptr::null(),n})}
unsafe extern "C" fn finish(raw:*mut c_void)->i32{read_capture(ReadCall{raw,kind:3,offset:0,p:core::ptr::null(),n:0})}
