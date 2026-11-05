// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned cpp/patched/ios.cpp primitive/CRT/log policy;native string/filesystem SDK retained.
use std::ffi::{c_char,c_void};
extern "C"{
#[link_name="wprt_ios_send"]fn target_0(request:u32,ticks:*mut u64)->i32;
#[link_name="wprt_ios_update"]fn target_1()->();
#[link_name="wprt_ios_take_completion"]fn target_2(request:*mut u32,result:*mut i32)->bool;
#[link_name="wprt_ios_ioctl"]fn target_3(fd:i32,command:u32,input:u32,isize:u32,output:u32,osize:u32)->i32;
#[link_name="wprt_ios_ioctlv"]fn target_4(fd:i32,command:u32,inputs:u32,outputs:u32,vectors:u32)->i32;
#[link_name="_ZN2wp4nand4seekEiii"]fn target_5(fd:i32,offset:i32,whence:i32)->i32;
#[link_name="_ZN2wp4nand4readEijj"]fn target_6(fd:i32,address:u32,size:u32)->i32;
#[link_name="_ZN2wp4nand5writeEijj"]fn target_7(fd:i32,address:u32,size:u32)->i32;
#[link_name="_ZN2wp4nand5closeEi"]fn target_8(fd:i32)->();
#[link_name="_ZN2wp9bluetooth5closeEv"]fn target_9()->();
#[link_name="_ZN2wp9bluetooth6updateEv"]fn target_10()->();
#[link_name="_ZN2wp9bluetooth15take_completionERjRi"]fn target_11(request:*mut u32,result:*mut i32)->bool;
#[link_name="_ZN2wp9bluetooth6ioctlvEjjjjj"]fn target_12(request:u32,command:u32,inputs:u32,outputs:u32,vectors:u32)->i32;
#[link_name="_ZN2wp4disc4readEyjj"]fn target_13(offset:u64,length:u32,destination:u32)->();
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;fn __mingw_snprintf(out:*mut c_char,n:usize,format:*const c_char,...)->i32;fn fputc(c:i32,file:*mut libc::FILE)->i32;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn wphw_ios_message_sdk(data:*const u8,size:usize,callback:unsafe extern "C" fn(*mut c_void,*const c_char),context:*mut c_void);}
// port of cpp/patched/ios.cpp _ZN2wp3ios4sendEjRy;target name read actual OWN object.
#[export_name="_ZN2wp3ios4sendEjRy"]pub unsafe extern "C" fn entry_0(request:u32,ticks:*mut u64)->i32{target_0(request,ticks)}
// port of cpp/patched/ios.cpp _ZN2wp3ios6updateEv;target name read actual OWN object.
#[export_name="_ZN2wp3ios6updateEv"]pub unsafe extern "C" fn entry_1()->(){target_1()}
// port of cpp/patched/ios.cpp _ZN2wp3ios15take_completionERjRi;target name read actual OWN object.
#[export_name="_ZN2wp3ios15take_completionERjRi"]pub unsafe extern "C" fn entry_2(request:*mut u32,result:*mut i32)->bool{target_2(request,result)}
// port of cpp/patched/ios.cpp _ZN2wp3ios5ioctlEijjjjj;target name read actual OWN object.
#[export_name="_ZN2wp3ios5ioctlEijjjjj"]pub unsafe extern "C" fn entry_3(fd:i32,command:u32,input:u32,isize:u32,output:u32,osize:u32)->i32{target_3(fd,command,input,isize,output,osize)}
// port of cpp/patched/ios.cpp _ZN2wp3ios6ioctlvEijjjj;target name read actual OWN object.
#[export_name="_ZN2wp3ios6ioctlvEijjjj"]pub unsafe extern "C" fn entry_4(fd:i32,command:u32,inputs:u32,outputs:u32,vectors:u32)->i32{target_4(fd,command,inputs,outputs,vectors)}
// port of cpp/patched/ios.cpp wphw_ios_nand_seek;target name read actual OWN object.
#[export_name="wphw_ios_nand_seek"]pub unsafe extern "C" fn entry_5(fd:i32,offset:i32,whence:i32)->i32{target_5(fd,offset,whence)}
// port of cpp/patched/ios.cpp wphw_ios_nand_read;target name read actual OWN object.
#[export_name="wphw_ios_nand_read"]pub unsafe extern "C" fn entry_6(fd:i32,address:u32,size:u32)->i32{target_6(fd,address,size)}
// port of cpp/patched/ios.cpp wphw_ios_nand_write;target name read actual OWN object.
#[export_name="wphw_ios_nand_write"]pub unsafe extern "C" fn entry_7(fd:i32,address:u32,size:u32)->i32{target_7(fd,address,size)}
// port of cpp/patched/ios.cpp wphw_ios_nand_close;target name read actual OWN object.
#[export_name="wphw_ios_nand_close"]pub unsafe extern "C" fn entry_8(fd:i32)->(){target_8(fd)}
// port of cpp/patched/ios.cpp wphw_ios_bt_close;target name read actual OWN object.
#[export_name="wphw_ios_bt_close"]pub unsafe extern "C" fn entry_9()->(){target_9()}
// port of cpp/patched/ios.cpp wphw_ios_bt_update;target name read actual OWN object.
#[export_name="wphw_ios_bt_update"]pub unsafe extern "C" fn entry_10()->(){target_10()}
// port of cpp/patched/ios.cpp wphw_ios_bt_completion;target name read actual OWN object.
#[export_name="wphw_ios_bt_completion"]pub unsafe extern "C" fn entry_11(request:*mut u32,result:*mut i32)->bool{target_11(request,result)}
// port of cpp/patched/ios.cpp wphw_ios_bt_ioctlv;target name read actual OWN object.
#[export_name="wphw_ios_bt_ioctlv"]pub unsafe extern "C" fn entry_12(request:u32,command:u32,inputs:u32,outputs:u32,vectors:u32)->i32{target_12(request,command,inputs,outputs,vectors)}
// port of cpp/patched/ios.cpp wphw_ios_disc_read;target name read actual OWN object.
#[export_name="wphw_ios_disc_read"]pub unsafe extern "C" fn entry_13(offset:u64,length:u32,destination:u32)->(){target_13(offset,length,destination)}

struct Format{kind:u32,command:u32,out:*mut u8}
unsafe extern "C" fn format_once(c:*mut c_void,device:*const c_char){let c=&*c.cast::<Format>();__mingw_snprintf(c.out.cast(),160,c"%s %s command %#x".as_ptr(),if c.kind==0{c"ioctl".as_ptr()}else{c"ioctlv".as_ptr()},device,c.command);}
// port of wphw_ios_format_once;source counted-string lifetime retained by typed SDK.
#[no_mangle]pub unsafe extern "C" fn wphw_ios_format_once(kind:u32,p:*const u8,n:usize,command:u32,out:*mut u8){let mut c=Format{kind,command,out};wphw_ios_message_sdk(p,n,format_once,std::ptr::addr_of_mut!(c).cast());}
// port of wphw_ios_print_once,including separate trailing fputc.
#[no_mangle]pub unsafe extern "C" fn wphw_ios_print_once(p:*const u8){__mingw_fprintf(__acrt_iob_func(2),c"IOS %s".as_ptr(),p.cast::<c_char>());fputc(10,__acrt_iob_func(2));}
struct Open{result:i32,failed:bool}
unsafe extern "C" fn log_open(c:*mut c_void,path:*const c_char){let c=&*c.cast::<Open>();if c.failed{__mingw_fprintf(__acrt_iob_func(2),c"IOS open %s failed (%d)\n".as_ptr(),path,c.result);log_write(c"ios".as_ptr(),c"open %s failed (%d)".as_ptr(),path,c.result);}else{__mingw_fprintf(__acrt_iob_func(2),c"IOS open %s\n".as_ptr(),path);log_write(c"ios".as_ptr(),c"open %s".as_ptr(),path);}}
// port of wphw_ios_log_open,source order preserved.
#[no_mangle]pub unsafe extern "C" fn wphw_ios_log_open(p:*const u8,n:usize,result:i32,failed:bool){let mut c=Open{result,failed};wphw_ios_message_sdk(p,n,log_open,std::ptr::addr_of_mut!(c).cast());}
unsafe extern "C" fn log_unknown(c:*mut c_void,device:*const c_char){let command=*c.cast::<u32>();__mingw_fprintf(__acrt_iob_func(2),c"IOS request %u on %s\n".as_ptr(),command,device);}
// port of wphw_ios_log_unknown.
#[no_mangle]pub unsafe extern "C" fn wphw_ios_log_unknown(command:u32,p:*const u8,n:usize){let mut c=command;wphw_ios_message_sdk(p,n,log_unknown,std::ptr::addr_of_mut!(c).cast());}
// port of wphw_ios_title_path;SAME CRT,size,format,argument widths.
#[no_mangle]pub unsafe extern "C" fn wphw_ios_title_path(high:u32,low:u32,out:*mut u8){__mingw_snprintf(out.cast(),32,c"/title/%08x/%08x/data".as_ptr(),high,low);}

extern "C"{fn wphw_ios_path_sdk(op:u32,p:*const u8,n:usize,q:*const u8,m:usize,callback:unsafe extern "C" fn(u32,*const c_void,*const u8,usize)->i32)->i32;
fn wphw_ios_mkdir_sdk(p:*const c_void)->i32;fn wphw_ios_mkfile_sdk(p:*const c_void)->i32;fn wphw_ios_remove_sdk(p:*const c_void)->i32;fn wphw_ios_rename_sdk(p:*const c_void,q:*const u8,n:usize)->i32;fn wphw_ios_exists_sdk(p:*const c_void)->bool;}
// port of wphw_ios_nand_path source switch;path object survives full call,second string created ONLY on rename branch.
unsafe extern "C" fn path_policy(op:u32,p:*const c_void,q:*const u8,n:usize)->i32{match op{0x03=>wphw_ios_mkdir_sdk(p),0x09=>wphw_ios_mkfile_sdk(p),0x07=>wphw_ios_remove_sdk(p),0x08=>wphw_ios_rename_sdk(p,q,n),0x06=>if wphw_ios_exists_sdk(p){0}else{-106},_=>libc::abort()}}
#[no_mangle]pub unsafe extern "C" fn wphw_ios_nand_path(op:u32,p:*const u8,n:usize,q:*const u8,m:usize)->i32{wphw_ios_path_sdk(op,p,n,q,m,path_policy)}
