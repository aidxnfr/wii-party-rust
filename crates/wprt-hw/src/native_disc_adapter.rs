// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned cpp/patched/disc.cpp adapter policy port.
// Native SDK retains original std::string lifetimes and exact std::sort tie ordering;accepted disc policy unchanged.
use std::ffi::{c_char,c_void};
extern "C"{fn wprt_disc_read(offset:u64,length:u32,destination:u32);
#[link_name="_ZNSt6chrono3_V212steady_clock3nowEv"]fn clock_now()->i64;
fn wphw_disc_path_sdk(data:*const u8,size:usize,callback:unsafe extern "C" fn(*mut c_void,*const c_char),context:*mut c_void);
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;}
// port of namespace wp::disc::read;actual mangled name from accepted archive.
#[export_name="_ZN2wp4disc4readEyjj"]pub unsafe extern "C" fn read(offset:u64,length:u32,destination:u32){wprt_disc_read(offset,length,destination);}
// port of source steady-clock count and duration<double,milli>,nanosecond difference with no intermediate rounding.
#[no_mangle]pub unsafe extern "C" fn wphw_disc_clock()->i64{clock_now()}
#[no_mangle]pub unsafe extern "C" fn wphw_disc_spent(began:i64)->f64{clock_now().wrapping_sub(began)as f64/1_000_000.0}
struct Report{offset:u64,length:u32,destination:u32,found:bool,spent:f64}
unsafe extern "C" fn stderr_report(context:*mut c_void,path:*const c_char){let c=&*context.cast::<Report>();__mingw_fprintf(__acrt_iob_func(2),c"disc read %llx+%x -> %08x from %s%s\n".as_ptr(),c.offset,c.length,c.destination,path,if c.found{c"".as_ptr()}else{c" (missing)".as_ptr()});}
unsafe extern "C" fn log_report(context:*mut c_void,path:*const c_char){let c=&*context.cast::<Report>();log_write(c"disc".as_ptr(),c"read %u bytes from %s%s in %.2f ms".as_ptr(),c.length,path,if c.found{c"".as_ptr()}else{c" (missing)".as_ptr()},c.spent);}
// port of counted native-string lifetime then exact source formatter/missing suffix;no byte/text normalization.
#[no_mangle]pub unsafe extern "C" fn wphw_disc_stderr(offset:u64,length:u32,destination:u32,path:*const u8,n:usize,found:bool){let mut report=Report{offset,length,destination,found,spent:0.0};wphw_disc_path_sdk(path,n,stderr_report,std::ptr::addr_of_mut!(report).cast());}
#[no_mangle]pub unsafe extern "C" fn wphw_disc_log(count:u32,path:*const u8,n:usize,found:bool,spent:f64){let mut report=Report{offset:0,length:count,destination:0,found,spent};wphw_disc_path_sdk(path,n,log_report,std::ptr::addr_of_mut!(report).cast());}
