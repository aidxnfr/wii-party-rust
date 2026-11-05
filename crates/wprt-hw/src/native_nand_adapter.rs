// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned cpp/patched/nand.cpp scalar/clock/language/log adapter policy port.
// SDK retains all original filesystem/path/save-backup/container lifetimes;foreign exception scope remains open.
use std::ffi::{c_char,c_void};
extern "C"{fn wprt_nand_widescreen()->bool;fn wprt_nand_read(fd:i32,address:u32,size:u32)->i32;fn wprt_nand_write(fd:i32,address:u32,size:u32)->i32;fn wprt_nand_seek(fd:i32,offset:i32,whence:i32)->i32;fn wprt_nand_close(fd:i32);fn wprt_nand_console_is_spanish()->bool;
#[link_name="towlower"]fn native_towlower(value:u16)->u16;
#[link_name="_ZNSt6chrono3_V212steady_clock3nowEv"]fn clock_now()->i64;
fn wphw_nand_message_sdk(data:*const u8,size:usize,callback:unsafe extern "C" fn(*mut c_void,*const c_char),context:*mut c_void);
fn wphw_nand_adopt_message_sdk(name:*const u8,size:usize,path:*const u16,path_size:usize,callback:unsafe extern "C" fn(*mut c_void,*const c_char,*const c_char),context:*mut c_void);
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
extern "system"{fn GetUserDefaultUILanguage()->u16;}
// port of namespace six primitive forwards;actual accepted archive mangled names.
#[export_name="_ZN2wp4nand10widescreenEv"]pub unsafe extern "C" fn widescreen()->bool{wprt_nand_widescreen()}
#[export_name="_ZN2wp4nand4readEijj"]pub unsafe extern "C" fn read(fd:i32,address:u32,size:u32)->i32{wprt_nand_read(fd,address,size)}
#[export_name="_ZN2wp4nand5writeEijj"]pub unsafe extern "C" fn write(fd:i32,address:u32,size:u32)->i32{wprt_nand_write(fd,address,size)}
#[export_name="_ZN2wp4nand4seekEiii"]pub unsafe extern "C" fn seek(fd:i32,offset:i32,whence:i32)->i32{wprt_nand_seek(fd,offset,whence)}
#[export_name="_ZN2wp4nand5closeEi"]pub unsafe extern "C" fn close(fd:i32){wprt_nand_close(fd);}
#[export_name="_ZN2wp4nand18console_is_spanishEv"]pub unsafe extern "C" fn spanish()->bool{wprt_nand_console_is_spanish()}
// port of target CRT towlower:installed wctype.h explicitly wint_t=unsignedshort,not a guessed 32bit Unicode fold.
#[no_mangle]pub unsafe extern "C" fn wphw_nand_lower(value:u16)->u16{native_towlower(value)}
#[no_mangle]pub unsafe extern "C" fn wphw_nand_steady_ns()->i64{clock_now()}
// port of PRIMARYLANGID from installed winnt.h:((WORD)lgid)&0x3ff;SAME Win32 service.
#[no_mangle]pub unsafe extern "C" fn wphw_nand_language()->u16{GetUserDefaultUILanguage()&0x3ff}
struct BackupLog{moment:*const c_char,result:i32}
unsafe extern "C" fn backup_log(context:*mut c_void,message:*const c_char){let c=&*context.cast::<BackupLog>();match c.result{
3=>{__mingw_fprintf(__acrt_iob_func(2),c"save backup (%s): %s; no backup was made and the existing backups are kept. Restore one from the launcher's Saves page if the game reports damaged data.\n".as_ptr(),c.moment,message);},
4=>{__mingw_fprintf(__acrt_iob_func(2),c"save backup (%s) failed: %s\n".as_ptr(),c.moment,message);},
0=>{__mingw_fprintf(__acrt_iob_func(2),c"save backup (%s): %s\n".as_ptr(),c.moment,message);},
_=>{}}log_write(c"nand".as_ptr(),c"save backup (%s): %s".as_ptr(),c.moment,message);}
// port of source stdstring lifetime then original Result log branches/order. Enum values mechanically read source header.
#[no_mangle]pub unsafe extern "C" fn wphw_nand_log_backup(moment:*const c_char,result:i32,message:*const u8,size:usize){let mut context=BackupLog{moment,result};wphw_nand_message_sdk(message,size,backup_log,std::ptr::addr_of_mut!(context).cast());}
unsafe extern "C" fn adopt_log(_context:*mut c_void,name:*const c_char,path:*const c_char){log_write(c"nand".as_ptr(),c"moved %s from the NAND root into %s".as_ptr(),name,path);}
#[no_mangle]pub unsafe extern "C" fn wphw_nand_log_adopt(name:*const u8,size:usize,path:*const u16,path_size:usize){wphw_nand_adopt_message_sdk(name,size,path,path_size,adopt_log,std::ptr::null_mut());}
