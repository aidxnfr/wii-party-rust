// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned cpp/patched/settings.cpp adapter policy port.
// Native SDK retains std containers,private LiveFlag/Number field access and original C++ atomics.
use std::ffi::{c_char,c_void};
type OptionSink=unsafe extern "C" fn(*mut c_void,*const c_char,usize,*const c_char,usize);
extern "C"{fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool;fn wprt_settings_number(key:*const c_char,variable:*const c_char)->i32;fn wprt_settings_revision()->u32;fn wphw_settings_option_row_sdk(index:usize,pad:bool,sink:OptionSink,context:*mut c_void);}
// port of namespace flag/number/revision scalar forwards,actual object namespace names.
#[export_name="_ZN2wp8settings4flagEPKcS2_"]pub unsafe extern "C" fn flag(key:*const c_char,variable:*const c_char)->bool{wprt_settings_flag(key,variable)}
#[export_name="_ZN2wp8settings6numberEPKcS2_"]pub unsafe extern "C" fn number(key:*const c_char,variable:*const c_char)->i32{wprt_settings_number(key,variable)}
#[export_name="_ZN2wp8settings8revisionEv"]pub unsafe extern "C" fn revision()->u32{wprt_settings_revision()}
// port of wphw_settings_options loops;counts mechanically parsed public keymap Action::Count/padmap kActionCount.
// SDK builds BOTH original temporary strings for each row and invokes sink while they remain alive.
#[no_mangle]pub unsafe extern "C" fn wphw_settings_options(sink:OptionSink,context:*mut c_void){for i in 0..20{wphw_settings_option_row_sdk(i,false,sink,context);}for i in 0..7{wphw_settings_option_row_sdk(i,true,sink,context);}}
