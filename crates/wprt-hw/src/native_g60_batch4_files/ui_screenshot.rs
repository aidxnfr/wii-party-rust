// SPDX-License-Identifier: GPL-3.0-or-later
// PRIVATE whole ui_text_sdk.cpp + screenshot_sdk.cpp. UNINTEGRATED.
// Source: review/{ui-text,screenshot}-adapter-port/*_sdk.cpp and exact accepted object disassembly.
// GNU string data0/len8/storage16 from accepted batch1 compiler+object proof; tm offsets emitted.
use std::ffi::{c_char,c_void};
mod layout {include!(env!("WPRT_NON_DSP_UI_LAYOUT"));}
#[repr(C)] union Storage {local:[u8;16],capacity:usize}
#[repr(C)] pub struct GnuString {data:*mut u8,len:usize,storage:Storage}
const _:()=assert!(std::mem::size_of::<GnuString>()==layout::STRING_SIZE && std::mem::align_of::<GnuString>()==layout::STRING_ALIGN);
type Sink=unsafe extern "C-unwind" fn(*mut c_void,*const u8,usize);
extern "C-unwind" {
 fn wprt_ui_language_of(p:*const u8,n:usize)->i32;
 #[link_name="_ZN2wp2ui12set_languageENS0_8LanguageE"]fn set_language_value(v:i32);
 fn wphw_screenshot_adapter(prefix:*const c_char,tm:*const c_void,attempt:i32,out:*mut c_void,sink:Sink);
 #[link_name="_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEyyPKcy"]fn string_replace(s:*mut GnuString,pos:usize,old:usize,p:*const u8,n:usize)->*mut GnuString;
 #[link_name="_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv"]fn string_dispose(s:*mut GnuString);
}
struct Guard(*mut GnuString);
impl Drop for Guard {fn drop(&mut self){unsafe{string_dispose(self.0)}}}
// ui_text_sdk.cpp entire two members; exact GNU names from accepted nm.
#[export_name="_ZN2wp2ui11language_ofERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn language_of(s:*const GnuString)->i32{wprt_ui_language_of((*s).data,(*s).len)}
#[export_name="_ZN2wp2ui12set_languageERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn set_language(s:*const GnuString){set_language_value(language_of(s));}
// screenshot_sdk.cpp: compiler-emitted named tm field access in original order.
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_screenshot_tm_sdk(raw:*const c_void,out:*mut i32){
 for (i,off) in layout::TM_FIELDS.iter().enumerate(){out.add(i).write(raw.cast::<u8>().add(*off).cast::<i32>().read());}
}
unsafe extern "C-unwind" fn sink(out:*mut c_void,p:*const u8,n:usize){let s=out.cast::<GnuString>();string_replace(s,0,(*s).len,p,n);}
// Accepted screenshot .text230: hidden output RCX;prefix RDX;tm R8;attempt R9d;returns output RAX.
#[export_name="_ZN2wp10screenshot9file_nameB5cxx11EPKcRK2tmi"]
pub unsafe extern "C-unwind" fn file_name(out:*mut GnuString,prefix:*const c_char,tm:*const c_void,attempt:i32)->*mut GnuString{
 std::ptr::addr_of_mut!((*out).data).write(std::ptr::addr_of_mut!((*out).storage).cast());
 std::ptr::addr_of_mut!((*out).len).write(0);
 std::ptr::addr_of_mut!((*out).storage).cast::<u8>().write(0);
 let guard=Guard(out);wphw_screenshot_adapter(prefix,tm,attempt,out.cast(),sink);std::mem::forget(guard);out
}
