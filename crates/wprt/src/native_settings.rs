// SPDX-License-Identifier: GPL-3.0-or-later
// Port of pristine engine/src/platform/settings.cpp. AI-assisted; preserve raw byte strings and source ordering.
const BASE:&[(&[u8],&[u8])]=&[
    (b"video.scale",b"1"),
    (b"video.fullscreen",b"0"),
    (b"video.copy_filter",b"1"),
    (b"video.dump_textures",b"0"),
    (b"video.custom_textures",b"1"),
    (b"input.gamepads",b"1"),
    (b"input.auto_grip",b"1"),
    (b"input.joycon_pairs",b"1"),
    (b"input.real_wiimotes",b"1"),
    (b"input.real_wiimote_mouse",b"1"),
    (b"input.wake_on_mouse",b"1"),
    (b"input.hide_cursor",b"0"),
    (b"input.controls_hint",b"0"),
    (b"system.interface_language",b"en"),
    (b"system.language",b"auto"),
    (b"system.pal60",b"1"),
    (b"system.skip_notices",b"1"),
    (b"system.options_menu",b"1"),
    (b"audio.mute",b"0"),
    (b"audio.wiimote_speaker",b"1"),
    (b"saves.backups",b"5"),
];
use std::{collections::BTreeMap,ffi::{c_char,c_void,CStr},sync::{Mutex,OnceLock,atomic::{AtomicU32,Ordering}}};
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
type OptionSink=unsafe extern "C" fn(*mut c_void,*const u8,usize,*const u8,usize);
extern "C"{fn wphw_settings_options(sink:OptionSink,context:*mut c_void);fn wphw_settings_cache_load(cache:*const c_void)->u64;fn wphw_settings_cache_store(cache:*mut c_void,value:u64);}
#[derive(Default)]struct State{path:Vec<u8>,values:BTreeMap<Vec<u8>,Vec<u8>>}
static REVISION:AtomicU32=AtomicU32::new(1);
// Binding of source g_mutex/g_path/g_values, initialized once per game DLL.
fn state()->&'static Mutex<State>{static STATE:OnceLock<Mutex<State>>=OnceLock::new();STATE.get_or_init(Default::default)}
// port of platform/settings.cpp options; native keymap/padmap supply their own ordered descriptors.
fn options()->&'static Vec<(Vec<u8>,Vec<u8>)>{static OPTIONS:OnceLock<Vec<(Vec<u8>,Vec<u8>)>>=OnceLock::new();OPTIONS.get_or_init(||{
    let mut result:Vec<_>=BASE.iter().map(|(k,v)|(k.to_vec(),v.to_vec())).collect();
    unsafe extern "C" fn append(context:*mut c_void,key:*const u8,kn:usize,value:*const u8,vn:usize){let result=&mut *context.cast::<Vec<(Vec<u8>,Vec<u8>)>>();result.push((std::slice::from_raw_parts(key,kn).to_vec(),std::slice::from_raw_parts(value,vn).to_vec()));}
    unsafe{wphw_settings_options(append,(&mut result as *mut Vec<(Vec<u8>,Vec<u8>)>).cast());}result
})}
// port of platform/settings.cpp fallback_of.
fn fallback_of(key:&[u8])->Vec<u8>{options().iter().find(|(k,_)|k==key).map(|(_,v)|v.clone()).unwrap_or_default()}
// port of platform/settings.cpp trimmed (only the source's space, tab and CR bytes).
fn trimmed(text:&[u8])->Vec<u8>{let Some(first)=text.iter().position(|b|!b" \t\r".contains(b))else{return Vec::new();};let last=text.iter().rposition(|b|!b" \t\r".contains(b)).unwrap();text[first..=last].to_vec()}
// C-string adapter for std::string::c_str; preserve embedded NUL bytes rather than reject them.
fn terminated(text:&[u8])->Vec<u8>{let mut value=text.to_vec();value.push(0);value}
// port of platform/settings.cpp write_file; caller owns g_mutex. Both source streams use binary mode.
fn write_file(s:&mut State){if s.path.is_empty(){return;}let mut out=Vec::new();let mut section=Vec::new();
    for(key,_)in options(){let dot=key.iter().position(|&b|b==b'.').unwrap_or(usize::MAX);let group=&key[..dot.min(key.len())];if group!=section{if !section.is_empty(){out.push(b'\n');}out.push(b'[');out.extend_from_slice(group);out.extend_from_slice(b"]\n");section=group.to_vec();}out.extend_from_slice(&key[dot.wrapping_add(1)..]);out.push(b'=');out.extend_from_slice(s.values.entry(key.clone()).or_default());out.push(b'\n');}
    let path=terminated(&s.path);unsafe{let file=libc::fopen(path.as_ptr().cast(),c"wb".as_ptr());if !file.is_null(){libc::fwrite(out.as_ptr().cast(),1,out.len(),file);libc::fclose(file);}}
}
// port of platform/settings.cpp value_of: environment has priority even when it is an empty string.
unsafe fn value_of(key:*const c_char,variable:*const c_char)->Vec<u8>{if !variable.is_null(){let setting=libc::getenv(variable);if !setting.is_null(){return CStr::from_ptr(setting).to_bytes().to_vec();}}let key=CStr::from_ptr(key).to_bytes();let s=state().lock().unwrap();s.values.get(key).cloned().unwrap_or_else(||fallback_of(key))}
// port of platform/settings.cpp load. Keep existing unknown map entries, as the source does.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_load(path:*const u8,n:usize){let mut s=state().lock().unwrap();s.path=std::slice::from_raw_parts(path,n).to_vec();for(k,v)in options(){s.values.insert(k.clone(),v.clone());}
    let name=terminated(&s.path);let file=libc::fopen(name.as_ptr().cast(),c"rb".as_ptr());let mut section=Vec::new();
    if !file.is_null(){loop{let mut line=Vec::new();let mut end=false;loop{let ch=libc::fgetc(file);if ch==libc::EOF{end=true;break;}if ch==10{break;}line.push(ch as u8);}if end&&line.is_empty(){break;}let line=trimmed(&line);if line.is_empty()||line[0]==b';'||line[0]==b'#'{continue;}if line[0]==b'['&&line[line.len()-1]==b']'{section=trimmed(&line[1..line.len()-1]);continue;}let Some(equals)=line.iter().position(|&b|b==b'=')else{continue;};let mut key=section.clone();key.push(b'.');key.extend_from_slice(&trimmed(&line[..equals]));if s.values.contains_key(&key){s.values.insert(key,trimmed(&line[equals+1..]));}}}
    write_file(&mut s);REVISION.fetch_add(1,Ordering::SeqCst);if !file.is_null(){libc::fclose(file);}
}
// port of platform/settings.cpp flag: comparisons deliberately remain case-sensitive.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_flag(key:*const c_char,variable:*const c_char)->bool{let value=value_of(key,variable);![b"0".as_slice(),b"false",b"off",b"no"].contains(&value.as_slice())}
// port of platform/settings.cpp number; use the same native atoi, not strict Rust parsing.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_number(key:*const c_char,variable:*const c_char)->i32{let value=terminated(&value_of(key,variable));libc::atoi(value.as_ptr().cast())}
// port of platform/settings.cpp text, with a byte-copy ABI replacing std::string return layout.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_text(key:*const c_char,variable:*const c_char,sink:Sink,context:*mut c_void){let value=value_of(key,variable);sink(context,value.as_ptr(),value.len());}
// port of platform/settings.cpp store.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_store(key:*const c_char,value:*const u8,n:usize){let mut s=state().lock().unwrap();s.values.insert(CStr::from_ptr(key).to_bytes().to_vec(),std::slice::from_raw_parts(value,n).to_vec());write_file(&mut s);REVISION.fetch_add(1,Ordering::SeqCst);}
// port of platform/settings.cpp keys: base descriptors only, not dynamic keymap/padmap options.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_keys(sink:Sink,context:*mut c_void){for(k,_)in BASE{sink(context,k.as_ptr(),k.len());}}
// port of platform/settings.cpp revision (default C++ sequentially-consistent load).
#[no_mangle]pub extern "C" fn wprt_settings_revision()->u32{REVISION.load(Ordering::SeqCst)}
// port of platform/settings.cpp LiveFlag::operator(); opaque native atomic leaf avoids C++ layout assumptions.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_live_flag(key:*const c_char,variable:*const c_char,cache:*mut c_void)->bool{let current=wprt_settings_revision()as u64;let mut cached=wphw_settings_cache_load(cache);if cached>>32!=current{cached=(current<<32)|if wprt_settings_flag(key,variable){1}else{0};wphw_settings_cache_store(cache,cached);}cached&1!=0}
// port of platform/settings.cpp LiveNumber::operator(), including signed low-word conversion.
#[no_mangle]pub unsafe extern "C" fn wprt_settings_live_number(key:*const c_char,variable:*const c_char,cache:*mut c_void)->i32{let current=wprt_settings_revision()as u64;let mut cached=wphw_settings_cache_load(cache);if cached>>32!=current{cached=(current<<32)|(wprt_settings_number(key,variable)as u32 as u64);wphw_settings_cache_store(cache,cached);}cached as u32 as i32}
