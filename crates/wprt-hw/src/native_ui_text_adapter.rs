// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete owned cpp/patched/ui_text_rust.cpp adapter policy port.
// Text remains an opaque existing pointer;two std::string SDK bodies retained,never infer container layout.
use std::ffi::{c_char,c_void};
extern "C"{fn wprt_ui_set_language(language:i32);fn wprt_ui_text()->*const c_void;fn wprt_ui_label(key:*const c_char)->*const c_char;fn wprt_ui_label_language(key:*const c_char,language:i32)->*const c_char;}
// port of set_language(Language);public enum underlying int32,including source numeric conversions.
#[export_name="_ZN2wp2ui12set_languageENS0_8LanguageE"]pub unsafe extern "C" fn set_language(language:i32){wprt_ui_set_language(language);}
// port of Text reference return;same existing pointer,no field interpretation or invented defaults.
#[export_name="_ZN2wp2ui4textEv"]pub unsafe extern "C" fn text()->*const c_void{wprt_ui_text()}
// port of both label overloads exact forwards;names confirmed OWNarchive nm.
#[export_name="_ZN2wp2ui5labelEPKc"]pub unsafe extern "C" fn label(key:*const c_char)->*const c_char{wprt_ui_label(key)}
#[export_name="_ZN2wp2ui5labelEPKcNS0_8LanguageE"]pub unsafe extern "C" fn label_language(key:*const c_char,language:i32)->*const c_char{wprt_ui_label_language(key,language)}
