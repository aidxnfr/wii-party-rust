
// SPDX-License-Identifier: GPL-3.0-or-later
// Remaining save_backup_forwarder.cpp ownership and filesystem operations.
// C++ exception relays become direct C-unwind propagation through Rust Drop owners;
// no exception is swallowed, mapped to success, or replaced with abort.
use super::filesystem::{fs::*,SaveOutcome,OutcomeGuard};
use core::ffi::{c_char,c_void};
use core::mem::MaybeUninit;
use super::eh;
#[repr(C)]struct WpPath{count:usize,items:*mut u16}
#[repr(C)]struct WpOutcome{result:i32,message:*mut c_char,backup:WpPath}
#[repr(C)]struct WpBackup{name:*mut c_char,path:WpPath}
#[repr(C)]struct WpList{count:usize,items:*mut WpBackup}
#[repr(C)]struct Backup{name:CxxString,path:FsPath}
extern "C-unwind"{
 fn wpsaves_save_relative(title:u32)->WpPath;
 fn wpsaves_backups_beside(root:*const WpPath)->WpPath;
 fn wpsaves_saved_relative(name:*const c_char,path:*const WpPath)->WpPath;
 fn wpsaves_back_up(root:*const WpPath,relative:*const WpPath,backups:*const WpPath,keep:i32,now:i64)->WpOutcome;
 fn wpsaves_restore(root:*const WpPath,backups:*const WpPath,name:*const c_char,path:*const WpPath,keep:i32,now:i64)->WpOutcome;
 fn wpsaves_list(root:*const WpPath)->WpList;
 fn wpsaves_free_path(path:*mut WpPath);
 fn wpsaves_free_outcome(out:*mut WpOutcome);
 fn wpsaves_free_list(list:*mut WpList);
 fn wprt_saves_adopt(c:*mut c_void,op:unsafe extern "C" fn(*mut c_void,i32)->i32);
}
extern "C"{fn strlen(p:*const c_char)->usize;fn abort()->!;}
unsafe fn view(p:*const FsPath)->WpPath{WpPath{count:(*p).native.n,items:(*p).native.p}}
unsafe fn from_native(out:*mut FsPath,p:&WpPath){if p.count!=0{path_from_wide(out,p.items,p.count)}else{path_default(out)}}
unsafe fn emit_owned_path(out:*mut FsPath,mut raw:WpPath)->*mut FsPath{from_native(out,&raw);wpsaves_free_path(&mut raw);out}
#[export_name="_ZN2wp5saves13save_relativeB5cxx11Ej"]pub unsafe extern "C-unwind" fn save_relative(out:*mut FsPath,title:u32)->*mut FsPath{emit_owned_path(out,wpsaves_save_relative(title))}
#[export_name="_ZN2wp5saves14backups_besideERKNSt10filesystem7__cxx114pathE"]pub unsafe extern "C-unwind" fn backups_beside(out:*mut FsPath,p:*const FsPath)->*mut FsPath{let v=view(p);emit_owned_path(out,wpsaves_backups_beside(&v))}
#[export_name="_ZN2wp5saves14saved_relativeB5cxx11ERKNS0_6BackupE"]pub unsafe extern "C-unwind" fn saved_relative(out:*mut FsPath,p:*const Backup)->*mut FsPath{let v=view(core::ptr::addr_of!((*p).path));emit_owned_path(out,wpsaves_saved_relative((*p).name.p.cast(),&v))}
unsafe fn convert(out:*mut SaveOutcome,mut raw:WpOutcome)->*mut SaveOutcome{
 (*out).result=4;s_init(core::ptr::addr_of_mut!((*out).message));let msg=SGuard(core::ptr::addr_of_mut!((*out).message));path_default(core::ptr::addr_of_mut!((*out).backup));core::mem::forget(msg);let guard=OutcomeGuard(out);
 (*out).result=raw.result;let p=if raw.message.is_null(){b"\0".as_ptr()}else{raw.message.cast()};s_assign(core::ptr::addr_of_mut!((*out).message),p,strlen(p.cast()));
 let mut path=MaybeUninit::<FsPath>::uninit();from_native(path.as_mut_ptr(),&raw.backup);let pguard=PGuard(path.as_mut_ptr());path_move_assign(core::ptr::addr_of_mut!((*out).backup),path.as_mut_ptr());drop(pguard);
 wpsaves_free_outcome(&mut raw);core::mem::forget(guard);out
}
#[export_name="_ZN2wp5saves7back_upERKNSt10filesystem7__cxx114pathES5_S5_ix"]pub unsafe extern "C-unwind" fn back_up(out:*mut SaveOutcome,a:*const FsPath,b:*const FsPath,c:*const FsPath,keep:i32,now:i64)->*mut SaveOutcome{let a=view(a);let b=view(b);let c=view(c);convert(out,wpsaves_back_up(&a,&b,&c,keep,now))}
#[export_name="_ZN2wp5saves7restoreERKNSt10filesystem7__cxx114pathES5_RKNS0_6BackupEix"]pub unsafe extern "C-unwind" fn restore(out:*mut SaveOutcome,a:*const FsPath,b:*const FsPath,c:*const Backup,keep:i32,now:i64)->*mut SaveOutcome{let a=view(a);let b=view(b);let path=view(core::ptr::addr_of!((*c).path));convert(out,wpsaves_restore(&a,&b,(*c).name.p.cast(),&path,keep,now))}
unsafe fn backup_drop(p:*mut Backup){path_drop(core::ptr::addr_of_mut!((*p).path));s_dispose(core::ptr::addr_of_mut!((*p).name));}
struct BackupGuard(*mut Backup);impl Drop for BackupGuard{fn drop(&mut self){unsafe{backup_drop(self.0)}}}
unsafe fn backup_move(dst:*mut u8,src:*mut u8){let d=dst.cast::<Backup>();let s=src.cast::<Backup>();s_move_construct(core::ptr::addr_of_mut!((*d).name),core::ptr::addr_of_mut!((*s).name));path_move_construct(core::ptr::addr_of_mut!((*d).path),core::ptr::addr_of_mut!((*s).path));}
unsafe fn backup_relocate(dst:*mut u8,src:*mut u8){backup_move(dst,src);backup_drop(src.cast());}
struct BackupVecGuard(*mut CxxVec);impl Drop for BackupVecGuard{fn drop(&mut self){unsafe{for i in 0..v_len(self.0,72){backup_drop((*self.0).b.add(i*72).cast());}v_free(self.0);}}}
#[export_name="_ZN2wp5saves4listERKNSt10filesystem7__cxx114pathE"]pub unsafe extern "C-unwind" fn list(out:*mut CxxVec,p:*const FsPath)->*mut CxxVec{
 let input=view(p);let mut raw=wpsaves_list(&input);v_init(out);let vector=BackupVecGuard(out);
 for index in 0..raw.count{let item=&*raw.items.add(index);let mut tmp=MaybeUninit::<Backup>::uninit();let ptr=tmp.as_mut_ptr();let text=if item.name.is_null(){b"\0".as_ptr()}else{item.name.cast()};
 s_construct(core::ptr::addr_of_mut!((*ptr).name),text,strlen(text.cast()));let name=SGuard(core::ptr::addr_of_mut!((*ptr).name));from_native(core::ptr::addr_of_mut!((*ptr).path),&item.path);core::mem::forget(name);let entry=BackupGuard(ptr);
 if (*out).e==(*out).c{let old=v_len(out,72);let (new,cap)=v_grow(out,72,b"vector::_M_realloc_insert\0".as_ptr().cast());backup_move(new.add(old*72),ptr.cast());v_finish(out,72,new,cap,Some(backup_relocate));}
 else{backup_move((*out).e,ptr.cast());(*out).e=(*out).e.add(72);}drop(entry);
 }
 wpsaves_free_list(&mut raw);core::mem::forget(vector);out
}
struct AdoptCtx{result:*mut CxxVec,error:ErrorCode,data:MaybeUninit<FsPath>,it:MaybeUninit<DirectoryIterator>,target:MaybeUninit<FsPath>,exception:*mut c_void}
struct AdoptGuard(*mut AdoptCtx);impl Drop for AdoptGuard{fn drop(&mut self){unsafe{let c=&mut*self.0;eh::clear(core::ptr::addr_of_mut!(c.exception));path_drop(c.target.as_mut_ptr());drop(DGuard(c.it.as_mut_ptr()));path_drop(c.data.as_mut_ptr());}}}
unsafe fn strings_drop(v:*mut CxxVec){for i in 0..v_len(v,32){s_dispose((*v).b.add(i*32).cast());}v_free(v);}
struct StringsGuard(*mut CxxVec);impl Drop for StringsGuard{fn drop(&mut self){unsafe{strings_drop(self.0)}}}
unsafe fn str_relocate(dst:*mut u8,src:*mut u8){s_move_construct(dst.cast(),src.cast());s_dispose(src.cast());}
unsafe fn push_string(v:*mut CxxVec,s:*mut CxxString){if (*v).e==(*v).c{let n=v_len(v,32);let(p,cap)=v_grow(v,32,b"vector::_M_realloc_insert\0".as_ptr().cast());s_move_construct(p.add(n*32).cast(),s);v_finish(v,32,p,cap,Some(str_relocate));}else{s_move_construct((*v).e.cast(),s);(*v).e=(*v).e.add(32);}}
unsafe extern "C-unwind" fn adopt_operation_body(raw:*mut c_void,op:i32)->i32{let c=&mut*raw.cast::<AdoptCtx>();match op{
 0=>(*c.it.as_ptr()).object.is_null()as i32,
 1=>(entry_file_type(dir_entry(c.it.as_ptr()),&mut c.error)==1)as i32,
 2=>{let mut name=MaybeUninit::<FsPath>::uninit();path_filename(name.as_mut_ptr(),dir_entry(c.it.as_ptr()));let a=PGuard(name.as_mut_ptr());let mut path=MaybeUninit::<FsPath>::uninit();path_copy(path.as_mut_ptr(),c.data.as_ptr());let b=PGuard(path.as_mut_ptr());path_append(path.as_mut_ptr(),name.as_ptr());path_move_assign(c.target.as_mut_ptr(),path.as_mut_ptr());drop(b);drop(a);0},
 3=>{let ty=file_type(c.target.as_ptr(),&mut c.error);(ty!=0&&ty != -1)as i32},
 4=>{fs_mkdir(c.data.as_ptr(),&mut c.error);0},
 5=>{fs_rename(dir_entry(c.it.as_ptr()),c.target.as_ptr(),&mut c.error);0},
 6=>(c.error.value==0)as i32,
 7=>{let mut name=MaybeUninit::<FsPath>::uninit();path_filename(name.as_mut_ptr(),dir_entry(c.it.as_ptr()));let a=PGuard(name.as_mut_ptr());let mut text=MaybeUninit::<CxxString>::uninit();path_string(text.as_mut_ptr(),name.as_ptr(),false);let b=SGuard(text.as_mut_ptr());push_string(c.result,text.as_mut_ptr());drop(b);drop(a);0},
 8=>{dir_next(c.it.as_mut_ptr());0},_=>eh::terminate(),
}}
#[export_name="_ZN2wp5saves16adopt_root_filesERKNSt10filesystem7__cxx114pathES5_"]pub unsafe extern "C-unwind" fn adopt_root_files(out:*mut CxxVec,root:*const FsPath,relative:*const FsPath)->*mut CxxVec{
 v_init(out);let result=StringsGuard(out);let mut c=AdoptCtx{result:out,error:error_code(),data:MaybeUninit::uninit(),it:MaybeUninit::uninit(),target:MaybeUninit::uninit(),exception:core::ptr::null_mut()};
 path_copy(c.data.as_mut_ptr(),root);let data=PGuard(c.data.as_mut_ptr());path_append(c.data.as_mut_ptr(),relative);dir_construct(c.it.as_mut_ptr(),root,0,&mut c.error);let it=DGuard(c.it.as_mut_ptr());path_default(c.target.as_mut_ptr());core::mem::forget(it);core::mem::forget(data);
 let ctx=AdoptGuard(&mut c);wprt_saves_adopt((&mut c as*mut AdoptCtx).cast(),adopt_operation);if !c.exception.is_null(){eh::rethrow(core::ptr::addr_of_mut!(c.exception));}drop(ctx);core::mem::forget(result);out
}

struct AdoptCall{raw:*mut c_void,op:i32}
unsafe extern "C-unwind" fn adopt_invoke(p:*mut c_void)->i32{let call=&*p.cast::<AdoptCall>();adopt_operation_body(call.raw,call.op)}
unsafe extern "C" fn adopt_operation(raw:*mut c_void,op:i32)->i32{let mut call=AdoptCall{raw,op};eh::capture(adopt_invoke,(&mut call as*mut AdoptCall).cast(),core::ptr::addr_of_mut!((*raw.cast::<AdoptCtx>()).exception))}
