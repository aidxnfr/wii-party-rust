
// SPDX-License-Identifier: GPL-3.0-or-later
// Remaining NAND/IOS/renderer/boot SDK file bodies, expressed in Rust.
// Source: g60-claude/{nand_sdk,ios_sdk,renderer_sdk,boot_services} original residue.cpp files.
#[path="fs_common.rs"]pub mod fs;
use fs::*;
use core::ffi::{c_char,c_void};
use core::mem::MaybeUninit;
type WideSink=unsafe extern "C-unwind" fn(*mut c_void,*const u16,usize);
type ByteSink=unsafe extern "C-unwind" fn(*mut c_void,*const u8,usize);
unsafe fn with_path<R>(p:*const u16,n:usize,op:impl FnOnce(*mut FsPath)->R)->R{let mut x=MaybeUninit::<FsPath>::uninit();path_from_wide(x.as_mut_ptr(),p,n);let guard=PGuard(x.as_mut_ptr());let result=op(x.as_mut_ptr());drop(guard);result}
unsafe fn with_bytes<R>(p:*const u8,n:usize,op:impl FnOnce(*mut FsPath)->R)->R{let mut x=MaybeUninit::<FsPath>::uninit();path_from_bytes(x.as_mut_ptr(),p,n);let guard=PGuard(x.as_mut_ptr());let result=op(x.as_mut_ptr());drop(guard);result}
unsafe fn emit_wide(p:*const FsPath,emit:WideSink,c:*mut c_void){emit(c,(*p).native.p,(*p).native.n)}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_path_bytes(p:*const u8,n:usize,s:WideSink,c:*mut c_void){with_bytes(p,n,|p|emit_wide(p,s,c))}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_path_join(a:*const u16,an:usize,b:*const u16,bn:usize,s:WideSink,c:*mut c_void){
 with_path(b,bn,|b|with_path(a,an,|a|{let mut out=MaybeUninit::<FsPath>::uninit();path_copy(out.as_mut_ptr(),a);let g=PGuard(out.as_mut_ptr());path_append(out.as_mut_ptr(),b);emit_wide(out.as_ptr(),s,c);drop(g)}))
}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_path_components(p:*const u16,n:usize,relative:bool,s:WideSink,c:*mut c_void){with_path(p,n,|p|{
 let mut rel=MaybeUninit::<FsPath>::uninit();if relative{path_relative(rel.as_mut_ptr(),p);}else{path_copy(rel.as_mut_ptr(),p);}let g=PGuard(rel.as_mut_ptr());path_components(rel.as_ptr(),|part|emit_wide(part,s,c));drop(g);
})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_path_string(p:*const u16,n:usize,generic:bool,s:ByteSink,c:*mut c_void){with_path(p,n,|p|{let mut text=MaybeUninit::<CxxString>::uninit();path_string(text.as_mut_ptr(),p,generic);let g=SGuard(text.as_mut_ptr());s(c,(*text.as_ptr()).p,(*text.as_ptr()).n);drop(g)})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_parent(p:*const u16,n:usize,s:WideSink,c:*mut c_void){with_path(p,n,|p|{let mut out=MaybeUninit::<FsPath>::uninit();path_parent(out.as_mut_ptr(),p);let g=PGuard(out.as_mut_ptr());emit_wide(out.as_ptr(),s,c);drop(g)})}
extern "C"{fn abort()->!;fn strlen(p:*const c_char)->usize;#[link_name="_time64"]fn native_time(out:*mut i64)->i64;#[link_name="_ZN2wp8g_memoryE"]static G_MEMORY:*mut u8;}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_fs_query(op:u32,p:*const u16,n:usize)->bool{with_path(p,n,|p|{if op>1{abort();}let mut e=error_code();let ty=file_type(p,&mut e);match op{0=>ty!=0&&ty != -1,1=>ty==1,_=>core::hint::unreachable_unchecked()}})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_fs_create(p:*const u16,n:usize)->bool{with_path(p,n,|p|{let mut e=error_code();fs_mkdir(p,&mut e);e.value==0})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_fs_remove(p:*const u16,n:usize)->u64{with_path(p,n,|p|{let mut e=error_code();fs_remove_all(p,&mut e)})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_fs_rename(a:*const u16,an:usize,b:*const u16,bn:usize)->bool{with_path(b,bn,|b|with_path(a,an,|a|{let mut e=error_code();fs_rename(a,b,&mut e);e.value==0}))}
extern "C-unwind"{
 #[link_name="_ZN2wp5saves13save_relativeB5cxx11Ej"]fn save_relative(out:*mut FsPath,title:u32)->*mut FsPath;
 #[link_name="_ZN2wp5saves14backups_besideERKNSt10filesystem7__cxx114pathE"]fn backups_beside(out:*mut FsPath,p:*const FsPath)->*mut FsPath;
 #[link_name="_ZN2wp5saves16adopt_root_filesERKNSt10filesystem7__cxx114pathES5_"]fn adopt_files(out:*mut CxxVec,a:*const FsPath,b:*const FsPath)->*mut CxxVec;
 #[link_name="_ZN2wp5saves7back_upERKNSt10filesystem7__cxx114pathES5_S5_ix"]fn save_back_up(out:*mut SaveOutcome,a:*const FsPath,b:*const FsPath,c:*const FsPath,keep:i32,now:i64)->*mut SaveOutcome;
 #[link_name="_ZN2wp4nand14host_directoryERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]fn nand_host(out:*mut CxxString,p:*const CxxString)->*mut CxxString;
 #[link_name="_ZN2wp5video13save_png_rgbaEPKcRKSt6vectorIjSaIjEEjj"]fn save_png_rgba(path:*const c_char,pixels:*const CxxVec,w:u32,h:u32);
 #[link_name="_ZN2wp3exi5mountERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]fn exi_mount(path:*const CxxString);
}
#[repr(C)]pub struct SaveOutcome{pub result:i32,pub message:CxxString,pub backup:FsPath}
pub struct OutcomeGuard(pub *mut SaveOutcome);impl Drop for OutcomeGuard{fn drop(&mut self){unsafe{path_drop(core::ptr::addr_of_mut!((*self.0).backup));s_dispose(core::ptr::addr_of_mut!((*self.0).message));}}}
struct StringsGuard(*mut CxxVec);impl Drop for StringsGuard{fn drop(&mut self){unsafe{let v=&mut *self.0;let n=v_len(v,32);for i in 0..n{s_dispose(v.b.add(i*32).cast());}v_free(v);}}}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_save_relative(title:u32,s:WideSink,c:*mut c_void){let mut p=MaybeUninit::<FsPath>::uninit();save_relative(p.as_mut_ptr(),title);let g=PGuard(p.as_mut_ptr());emit_wide(p.as_ptr(),s,c);drop(g)}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_backups_beside(p:*const u16,n:usize,s:WideSink,c:*mut c_void){with_path(p,n,|p|{let mut b=MaybeUninit::<FsPath>::uninit();backups_beside(b.as_mut_ptr(),p);let g=PGuard(b.as_mut_ptr());emit_wide(b.as_ptr(),s,c);drop(g)})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_adopt(r:*const u16,rn:usize,s:*const u16,sn:usize,emit:ByteSink,c:*mut c_void){with_path(s,sn,|s|with_path(r,rn,|r|{let mut v=MaybeUninit::<CxxVec>::uninit();adopt_files(v.as_mut_ptr(),r,s);let g=StringsGuard(v.as_mut_ptr());for i in 0..v_len(v.as_ptr(),32){let text=&*(*v.as_ptr()).b.add(i*32).cast::<CxxString>();emit(c,text.p,text.n);}drop(g)}))}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_back_up(r:*const u16,rn:usize,s:*const u16,sn:usize,b:*const u16,bn:usize,keep:i32,emit:ByteSink,c:*mut c_void)->i32{let now=native_time(core::ptr::null_mut());with_path(b,bn,|b|with_path(s,sn,|s|with_path(r,rn,|r|{let mut out=MaybeUninit::<SaveOutcome>::uninit();save_back_up(out.as_mut_ptr(),r,s,b,keep,now);let g=OutcomeGuard(out.as_mut_ptr());let x=&*out.as_ptr();emit(c,x.message.p,x.message.n);let result=x.result;drop(g);result})))}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_nand_adopt_message_sdk(p:*const u8,n:usize,s:*const u16,sn:usize,cb:unsafe extern "C-unwind" fn(*mut c_void,*const c_char,*const c_char),c:*mut c_void){let mut name=MaybeUninit::<CxxString>::uninit();s_construct(name.as_mut_ptr(),p,n);let a=SGuard(name.as_mut_ptr());with_path(s,sn,|s|{let mut text=MaybeUninit::<CxxString>::uninit();path_string(text.as_mut_ptr(),s,true);let b=SGuard(text.as_mut_ptr());cb(c,(*name.as_ptr()).p.cast(),(*text.as_ptr()).p.cast());drop(b)});drop(a)}
unsafe fn with_host_path<R>(p:*const u8,n:usize,op:impl FnOnce(*mut FsPath)->R)->R{
 let mut text=MaybeUninit::<CxxString>::uninit();if n==0{s_init(text.as_mut_ptr());}else{s_construct(text.as_mut_ptr(),p,n);}let a=SGuard(text.as_mut_ptr());
 let mut host=MaybeUninit::<CxxString>::uninit();nand_host(host.as_mut_ptr(),text.as_ptr());let b=SGuard(host.as_mut_ptr());let result=with_bytes((*host.as_ptr()).p,(*host.as_ptr()).n,op);drop(b);drop(a);result
}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_ios_directory(p:*const u8,n:usize,sink:unsafe extern "C-unwind" fn(*mut c_void,*const c_void),c:*mut c_void){with_host_path(p,n,|p|{let mut e=error_code();let mut it=MaybeUninit::<DirectoryIterator>::uninit();dir_construct(it.as_mut_ptr(),p,0,&mut e);let g=DGuard(it.as_mut_ptr());while !(*it.as_ptr()).object.is_null(){sink(c,dir_entry(it.as_ptr()).cast());dir_next(it.as_mut_ptr());}drop(g)})}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_ios_copy_name(entry:*const c_void,address:u32){
 let mut path=MaybeUninit::<FsPath>::uninit();path_filename(path.as_mut_ptr(),entry.cast());let a=PGuard(path.as_mut_ptr());let mut name=MaybeUninit::<CxxString>::uninit();path_string(name.as_mut_ptr(),path.as_ptr(),false);let b=SGuard(name.as_mut_ptr());narrow_resize(name.as_mut_ptr(),12);
 let dst=if address&0xf0000000==0xe0000000{G_MEMORY.add(0x14000000+(address&0x3fff)as usize)}else{G_MEMORY.add((address&0x1fffffff)as usize)};core::ptr::copy_nonoverlapping((*name.as_ptr()).p,dst,13);drop(b);drop(a);
}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_ios_usage(p:*const u8,n:usize,sink:unsafe extern "C-unwind" fn(*mut c_void,bool,u64),c:*mut c_void)->i32{with_host_path(p,n,|p|{
 let mut e=error_code();let ty=file_type(p,&mut e);if ty==0||ty == -1{return -106}if file_type(p,&mut e)!=2{return -101}
 let mut it=MaybeUninit::<DirectoryIterator>::uninit();recursive_construct(it.as_mut_ptr(),p,0,&mut e);let g=RGuard(it.as_mut_ptr());while !(*it.as_ptr()).object.is_null(){let entry=recursive_entry(it.as_ptr());let regular=entry_file_type(entry,&mut e)==1;let size=if regular{fs_size(entry,&mut e)}else{0};sink(c,regular,size);recursive_next(it.as_mut_ptr());}drop(g);0
})}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_gr_pack_save_png_rgba(path:*const c_char,pixels:*const u32,w:u32,h:u32){
 let n=w as usize*h as usize;let mut vector=CxxVec{b:core::ptr::null_mut(),e:core::ptr::null_mut(),c:core::ptr::null_mut()};
 // Source argument evaluation constructs the vector before converting the path.
 if n!=0{if n>isize::MAX as usize/4{cxx_throw_length_error(b"cannot create std::vector larger than max_size()\0".as_ptr().cast())}vector.b=cxx_new(n*4);vector.e=vector.b.add(n*4);vector.c=vector.e;core::ptr::copy_nonoverlapping(pixels,vector.b.cast(),n);}
 struct V(*mut CxxVec);impl Drop for V{fn drop(&mut self){unsafe{v_free(self.0)}}}let guard=V(&mut vector);
 with_bytes(path.cast(),strlen(path),|p|{let mut text=MaybeUninit::<CxxString>::uninit();path_string(text.as_mut_ptr(),p,false);let g=SGuard(text.as_mut_ptr());save_png_rgba((*text.as_ptr()).p.cast(),&vector,w,h);drop(g)});drop(guard);
}
#[no_mangle]pub unsafe extern "C-unwind" fn wpg_hw_exi_mount(path:*const c_char){with_bytes(path.cast(),strlen(path),|p|{
 let mut norm=MaybeUninit::<FsPath>::uninit();path_normal(norm.as_mut_ptr(),p);let a=PGuard(norm.as_mut_ptr());let mut parent=MaybeUninit::<FsPath>::uninit();path_parent(parent.as_mut_ptr(),norm.as_ptr());let b=PGuard(parent.as_mut_ptr());
 with_bytes(b"sram.bin".as_ptr(),8,|sram|{path_append(parent.as_mut_ptr(),sram);let mut text=MaybeUninit::<CxxString>::uninit();path_string(text.as_mut_ptr(),parent.as_ptr(),false);let c=SGuard(text.as_mut_ptr());exi_mount(text.as_ptr());drop(c)});drop(b);drop(a);
})}
