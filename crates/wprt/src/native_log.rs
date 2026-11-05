// SPDX-License-Identifier: GPL-3.0-or-later
// Port of arelkair pristine engine/src/core/log.cpp. Native varargs packaging stays in a leaf C++ adapter.
use std::{ffi::c_char,sync::{Mutex,OnceLock},time::Instant};
// Installed GNU C++17 stdio.h selects __USE_MINGW_ANSI_STDIO=1; use its same printf backend.
extern "C"{fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->libc::c_int;}
struct LogFile{file:usize,origin:Instant,mutex:Mutex<()>}
// port of core/log.cpp LogFile::LogFile: initialize the origin before opening the optional output.
impl LogFile{fn new()->Self{let origin=Instant::now();let file=unsafe{let path=libc::getenv(c"WP_LOG_FILE".as_ptr());if path.is_null(){0}else{libc::fopen(path,c"w".as_ptr())as usize}};Self{file,origin,mutex:Mutex::new(())}}}
// port of core/log.cpp instance (one instance per linked game DLL).
fn instance()->&'static LogFile{static LOG:OnceLock<LogFile>=OnceLock::new();LOG.get_or_init(LogFile::new)}
// port of core/log.cpp enabled.
pub fn enabled()->bool{instance().file!=0}
// port of core/log.cpp write: timestamp is sampled before the native vsnprintf leaf adapter.
#[no_mangle]pub extern "C" fn wprt_log_seconds()->f64{instance().origin.elapsed().as_secs_f64()}
// ABI adapter for core/log.cpp enabled.
#[no_mangle]pub extern "C" fn wprt_log_enabled()->bool{enabled()}
// port of core/log.cpp write: preserve C byte strings, printf layout, mutex scope and per-line fflush.
#[no_mangle]pub unsafe extern "C" fn wprt_log_emit(seconds:f64,category:*const c_char,message:*const c_char){
    let log=instance();if log.file==0{return;}
    let _hold=log.mutex.lock().unwrap();let file=log.file as *mut libc::FILE;
    __mingw_fprintf(file,c"[%10.3f] %-7s %s\n".as_ptr(),seconds,category,message);libc::fflush(file);
}
