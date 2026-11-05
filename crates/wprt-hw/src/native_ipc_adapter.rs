// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete hybrid/cpp/patched/ipc.cpp scalar/CRT adapter port;accepted IPC policy unchanged.
use std::ffi::c_char;
extern "C"{fn wprt_ipc_read32(a:u32)->u32;fn wprt_ipc_write32(a:u32,v:u32);fn wprt_ipc_reply(a:u32,r:i32,t:u64);fn wprt_ipc_update();fn wprt_ipc_interrupt_pending()->bool;fn wprt_ipc_report();
fn __acrt_iob_func(index:u32)->*mut libc::FILE;
fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of namespace wp::ipc scalar forwards;names confirmed OWNarchive nm.
#[export_name="_ZN2wp3ipc6read32Ej"]pub unsafe extern "C" fn read32(a:u32)->u32{wprt_ipc_read32(a)}
#[export_name="_ZN2wp3ipc7write32Ejj"]pub unsafe extern "C" fn write32(a:u32,v:u32){wprt_ipc_write32(a,v);}
#[export_name="_ZN2wp3ipc5replyEjiy"]pub unsafe extern "C" fn reply(a:u32,r:i32,t:u64){wprt_ipc_reply(a,r,t);}
#[export_name="_ZN2wp3ipc6updateEv"]pub unsafe extern "C" fn update(){wprt_ipc_update();}
#[export_name="_ZN2wp3ipc17interrupt_pendingEv"]pub unsafe extern "C" fn pending()->bool{wprt_ipc_interrupt_pending()}
#[export_name="_ZN2wp3ipc6reportEv"]pub unsafe extern "C" fn report(){wprt_ipc_report();}
// port of native wphw_ipc_report_header;target GNU ANSI std::fprintf uses __mingw_vfprintf,stderr __acrt_iob_func(2).
#[no_mangle]pub unsafe extern "C" fn wphw_ipc_report_header(control:u32,flags:u32,mask:u32,ppc:u32,arm:u32,requests:usize,replies:usize,events:usize){let err=__acrt_iob_func(2);__mingw_fprintf(err,c"IPC control=%02x flags=%08x mask=%08x ppc message=%08x arm message=%08x requests=%zu replies=%zu events=%zu".as_ptr(),control,flags,mask,ppc,arm,requests,replies,events);libc::fputc(10,err);}
// port of native wphw_ipc_report_trace:kind u8 promotes to C int for %c.
#[no_mangle]pub unsafe extern "C" fn wphw_ipc_report_trace(delta:i64,kind:u8,a:u32,b:u32){let err=__acrt_iob_func(2);__mingw_fprintf(err,c"IPC trace %lld %c %08x %08x".as_ptr(),delta,kind as i32,a,b);libc::fputc(10,err);}
// port of native wphw_ipc_report_event.
#[no_mangle]pub unsafe extern "C" fn wphw_ipc_report_event(kind:i32,address:u32,delta:i64){let err=__acrt_iob_func(2);__mingw_fprintf(err,c"IPC event kind=%d address=%08x due in %lld ticks".as_ptr(),kind,address,delta);libc::fputc(10,err);}
