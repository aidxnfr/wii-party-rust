// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/input/keymap.cpp. Tables/enum copied programmatically below.
use std::ffi::{c_char,c_void,CStr};
// Enum/actions/defaults/key names/legacy directions copied programmatically from pristine source/headers.
#[allow(non_upper_case_globals)]const ACTION_A:usize=0;
#[allow(non_upper_case_globals)]const ACTION_B:usize=1;
#[allow(non_upper_case_globals)]const ACTION_One:usize=2;
#[allow(non_upper_case_globals)]const ACTION_Two:usize=3;
#[allow(non_upper_case_globals)]const ACTION_Plus:usize=4;
#[allow(non_upper_case_globals)]const ACTION_Minus:usize=5;
#[allow(non_upper_case_globals)]const ACTION_Home:usize=6;
#[allow(non_upper_case_globals)]const ACTION_Up:usize=7;
#[allow(non_upper_case_globals)]const ACTION_Down:usize=8;
#[allow(non_upper_case_globals)]const ACTION_Left:usize=9;
#[allow(non_upper_case_globals)]const ACTION_Right:usize=10;
#[allow(non_upper_case_globals)]const ACTION_Shake:usize=11;
#[allow(non_upper_case_globals)]const ACTION_SwingUp:usize=12;
#[allow(non_upper_case_globals)]const ACTION_SwingDown:usize=13;
#[allow(non_upper_case_globals)]const ACTION_TiltLeft:usize=14;
#[allow(non_upper_case_globals)]const ACTION_TiltRight:usize=15;
#[allow(non_upper_case_globals)]const ACTION_TiltUp:usize=16;
#[allow(non_upper_case_globals)]const ACTION_TiltDown:usize=17;
#[allow(non_upper_case_globals)]const ACTION_Grip:usize=18;
#[allow(non_upper_case_globals)]const ACTION_Screenshot:usize=19;
#[allow(non_upper_case_globals)]const ACTION_Count:usize=20;
#[allow(non_upper_case_globals)]const kWheelUp:i32=0x100;
#[allow(non_upper_case_globals)]const kWheelDown:i32=0x101;
#[allow(non_upper_case_globals)]const kF1:i32=0x70;
#[allow(non_upper_case_globals)]const kF10:i32=0x79;
#[allow(non_upper_case_globals)]const kF11:i32=0x7A;
#[allow(non_upper_case_globals)]const kF12:i32=0x7B;
#[allow(non_upper_case_globals)]const kF24:i32=0x87;
#[allow(non_upper_case_globals)]const kButtonA:u32=0x0800;
#[allow(non_upper_case_globals)]const kButtonB:u32=0x0400;
#[allow(non_upper_case_globals)]const kButtonOne:u32=0x0200;
#[allow(non_upper_case_globals)]const kButtonTwo:u32=0x0100;
#[allow(non_upper_case_globals)]const kButtonPlus:u32=0x0010;
#[allow(non_upper_case_globals)]const kButtonMinus:u32=0x1000;
#[allow(non_upper_case_globals)]const kButtonHome:u32=0x8000;
#[allow(non_upper_case_globals)]const kButtonUp:u32=0x0008;
#[allow(non_upper_case_globals)]const kButtonDown:u32=0x0004;
#[allow(non_upper_case_globals)]const kButtonLeft:u32=0x0001;
#[allow(non_upper_case_globals)]const kButtonRight:u32=0x0002;
#[allow(non_upper_case_globals)]const kMotionShake:u32=0x10000;
#[allow(non_upper_case_globals)]const kMotionSwingUp:u32=0x20000;
#[allow(non_upper_case_globals)]const kMotionSwingDown:u32=0x40000;
#[allow(non_upper_case_globals)]const kMotionTiltLeft:u32=0x80000;
#[allow(non_upper_case_globals)]const kMotionTiltRight:u32=0x100000;
#[allow(non_upper_case_globals)]const kMotionTiltUp:u32=0x200000;
#[allow(non_upper_case_globals)]const kMotionTiltDown:u32=0x400000;
const ACTIONS:&[(&[u8],u32,&[u8])]=&[(b"a\0",kButtonA,b"Enter,Space,MouseLeft\0"),(b"b\0",kButtonB,b"Backspace,MouseRight\0"),(b"one\0",kButtonOne,b"1\0"),(b"two\0",kButtonTwo,b"2\0"),(b"plus\0",kButtonPlus,b"Plus,NumPlus\0"),(b"minus\0",kButtonMinus,b"Minus,NumMinus\0"),(b"home\0",kButtonHome,b"H\0"),(b"up\0",kButtonUp,b"Up,W\0"),(b"down\0",kButtonDown,b"Down,S\0"),(b"left\0",kButtonLeft,b"Left,A\0"),(b"right\0",kButtonRight,b"Right,D\0"),(b"shake\0",kMotionShake,b"MouseMiddle,LeftShift\0"),(b"swing_up\0",kMotionSwingUp,b"WheelUp,T\0"),(b"swing_down\0",kMotionSwingDown,b"WheelDown,G\0"),(b"tilt_left\0",kMotionTiltLeft,b"Q\0"),(b"tilt_right\0",kMotionTiltRight,b"E\0"),(b"tilt_up\0",kMotionTiltUp,b"R\0"),(b"tilt_down\0",kMotionTiltDown,b"F\0"),(b"grip\0",0,b"Tab\0"),(b"screenshot\0",0,b"F9\0")];
const KEYS:&[(&[u8],i32)]=&[(b"MouseLeft",0x01),(b"MouseRight",0x02),(b"MouseMiddle",0x04),(b"MouseX1",0x05),(b"MouseX2",0x06),(b"WheelUp",kWheelUp),(b"WheelDown",kWheelDown),(b"Backspace",0x08),(b"Tab",0x09),(b"Enter",0x0D),(b"Shift",0x10),(b"Ctrl",0x11),(b"Alt",0x12),(b"Pause",0x13),(b"CapsLock",0x14),(b"Escape",0x1B),(b"Space",0x20),(b"PageUp",0x21),(b"PageDown",0x22),(b"End",0x23),(b"Home",0x24),(b"Left",0x25),(b"Up",0x26),(b"Right",0x27),(b"Down",0x28),(b"Insert",0x2D),(b"Delete",0x2E),(b"Num0",0x60),(b"Num1",0x61),(b"Num2",0x62),(b"Num3",0x63),(b"Num4",0x64),(b"Num5",0x65),(b"Num6",0x66),(b"Num7",0x67),(b"Num8",0x68),(b"Num9",0x69),(b"NumMultiply",0x6A),(b"NumPlus",0x6B),(b"NumMinus",0x6D),(b"NumPeriod",0x6E),(b"NumDivide",0x6F),(b"LeftShift",0xA0),(b"RightShift",0xA1),(b"LeftCtrl",0xA2),(b"RightCtrl",0xA3),(b"LeftAlt",0xA4),(b"RightAlt",0xA5),(b"Semicolon",0xBA),(b"Plus",0xBB),(b"Comma",0xBC),(b"Minus",0xBD),(b"Period",0xBE),(b"Slash",0xBF),(b"Backquote",0xC0),(b"LeftBracket",0xDB),(b"Backslash",0xDC),(b"RightBracket",0xDD),(b"Quote",0xDE)];
const LEGACY:&[(usize,&[u8])]=&[(ACTION_Up,b"Up,A"),(ACTION_Down,b"Down,D"),(ACTION_Left,b"Left,S"),(ACTION_Right,b"Right,W")];
extern "C"{fn wphw_keymap_stoi(p:*const u8,n:usize)->i32;}
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
type CodeSink=unsafe extern "C" fn(*mut c_void,i32);
type CodesSink=unsafe extern "C" fn(*mut c_void,usize,*const i32,usize);
type Value=unsafe extern "C" fn(*mut c_void,*const u8,usize,Sink,*mut c_void);
type Codes=unsafe extern "C" fn(*const c_void,usize,*mut *const i32,*mut usize);
// port of keymap.cpp lower / trimmed. Use original CRT byte ctype, not Unicode case conversion.
fn lower(text:&[u8])->Vec<u8>{text.iter().map(|&c|unsafe{libc::tolower(c as i32)as u8}).collect()}
fn trimmed(text:&[u8])->&[u8]{let white=|c:&u8|b" \t\r\n".contains(c);let first=text.iter().position(|c|!white(c)).unwrap_or(text.len());let last=text.iter().rposition(|c|!white(c)).map_or(first,|n|n+1);&text[first..last]}
// port of keymap.cpp action overloads and setting_key. Native ActionInfo container supplies reference ABI only.
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_action(index:usize,name:*mut *const c_char,buttons:*mut u32,defaults:*mut *const c_char){let a=&ACTIONS[index];*name=a.0.as_ptr().cast();*buttons=a.1;*defaults=a.2.as_ptr().cast();}
fn setting_key(index:usize)->Vec<u8>{let mut key=b"keys.".to_vec();let name=ACTIONS[index].0;key.extend_from_slice(&name[..name.len()-1]);key}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_setting_key(index:usize,sink:Sink,context:*mut c_void){let key=setting_key(index);sink(context,key.as_ptr(),key.len());}
// port of keymap.cpp key_code; f01 remains invalid and function-number parsing occurs only for1-2 digit input.
fn key_code(name:&[u8])->i32{let key=lower(trimmed(name));unsafe{if key.len()==1&&libc::isalnum(key[0]as i32)!=0{return libc::toupper(key[0]as i32);}if key.len()>=2&&key.len()<=3&&key[0]==b'f'&&key[1..].iter().all(|&c|libc::isdigit(c as i32)!=0){let number=wphw_keymap_stoi(key[1..].as_ptr(),key.len()-1);if number>=1&&number<=kF24-kF1+1&&key[1]!=b'0'{return kF1+number-1;}return -1;}for &(name,code)in KEYS{if lower(name)==key{return code;}}}-1}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_key_code(p:*const u8,n:usize)->i32{key_code(std::slice::from_raw_parts(p,n))}
// port of keymap.cpp key_name / reserved.
fn key_name(code:i32)->Vec<u8>{if(code>=b'0' as i32&&code<=b'9' as i32)||(code>=b'A' as i32&&code<=b'Z' as i32){return vec![code as u8];}if code>=kF1&&code<=kF24{let mut name=b"F".to_vec();name.extend_from_slice((code-kF1+1).to_string().as_bytes());return name;}for &(name,key)in KEYS{if key==code{return name.to_vec();}}Vec::new()}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_key_name(code:i32,sink:Sink,context:*mut c_void){let name=key_name(code);sink(context,name.as_ptr(),name.len());}
#[no_mangle]pub extern "C" fn wprt_keymap_reserved(code:i32)->bool{code==kF10||code==kF11||code==kF12}
struct Parsed{codes:Vec<i32>,invalid:Vec<Vec<u8>>}
// port of keymap.cpp parse: trim tokens, preserve invalid spellings/order, deduplicate valid codes by first use.
fn parse(list:&[u8])->Parsed{let mut out=Parsed{codes:Vec::new(),invalid:Vec::new()};for raw in list.split(|&c|c==b','){let token=trimmed(raw);if !token.is_empty(){let code=key_code(token);if code<0||wprt_keymap_reserved(code){out.invalid.push(token.to_vec());}else if !out.codes.contains(&code){out.codes.push(code);}}}out}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_parse(p:*const u8,n:usize,code:CodeSink,invalid:Sink,context:*mut c_void){let result=parse(std::slice::from_raw_parts(p,n));for c in result.codes{code(context,c);}for token in result.invalid{invalid(context,token.as_ptr(),token.len());}}
// port of keymap.cpp format. Separator is read lazily only between nonempty names, matching source.
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_format(codes:*const i32,n:usize,separator:*const c_char,sink:Sink,context:*mut c_void){let mut result=Vec::new();for &code in if n==0{&[]}else{std::slice::from_raw_parts(codes,n)}{let name=key_name(code);if name.is_empty(){continue;}if !result.is_empty(){result.extend_from_slice(CStr::from_ptr(separator).to_bytes());}result.extend_from_slice(&name);}sink(context,result.as_ptr(),result.len());}
// port of keymap.cpp Bindings::has / bound. Caller transports vector spans, not std::vector layout.
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_has(codes:*const i32,n:usize,code:i32)->bool{n!=0&&std::slice::from_raw_parts(codes,n).contains(&code)}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_bound(bindings:*const c_void,get:Codes,code:i32)->bool{for index in 0..ACTIONS.len(){let(mut p,mut n)=(std::ptr::null(),0);get(bindings,index,&mut p,&mut n);if wprt_keymap_has(p,n,code){return true;}}false}
// port of keymap.cpp defaults.
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_defaults(sink:CodesSink,context:*mut c_void){for(i,(_,_,defaults))in ACTIONS.iter().enumerate(){let parsed=parse(&defaults[..defaults.len()-1]);sink(context,i,parsed.codes.as_ptr(),parsed.codes.len());}}
unsafe extern "C" fn bytes(context:*mut c_void,p:*const u8,n:usize){*context.cast::<Vec<u8>>()=std::slice::from_raw_parts(p,n).to_vec();}
unsafe fn value(get:Value,context:*mut c_void,key:&[u8])->Vec<u8>{let mut out=Vec::new();get(context,key.as_ptr(),key.len(),bytes,(&mut out as *mut Vec<u8>).cast());out}
// port of keymap.cpp legacy_directions: exact ordered parse comparison; retain the original legacy WASD permutation.
unsafe fn legacy_directions(get:Value,context:*mut c_void)->bool{for &(action,keys)in LEGACY{if parse(&value(get,context,&setting_key(action))).codes!=parse(keys).codes{return false;}}true}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_legacy(get:Value,context:*mut c_void)->bool{legacy_directions(get,context)}
// port of keymap.cpp direction / load. Source value-callback call order and legacy bypass are retained.
#[no_mangle]pub extern "C" fn wprt_keymap_direction(index:usize)->bool{index>=ACTION_Up&&index<=ACTION_Right}
#[no_mangle]pub unsafe extern "C" fn wprt_keymap_load(get:Value,value_context:*mut c_void,codes:CodesSink,invalid:Sink,result_context:*mut c_void){let legacy=legacy_directions(get,value_context);for index in 0..ACTIONS.len(){let key=setting_key(index);let input=if legacy&&wprt_keymap_direction(index){let d=ACTIONS[index].2;d[..d.len()-1].to_vec()}else{value(get,value_context,&key)};let parsed=parse(&input);for name in parsed.invalid{let mut text=key.clone();text.extend_from_slice(b": ");text.extend_from_slice(&name);invalid(result_context,text.as_ptr(),text.len());}codes(result_context,index,parsed.codes.as_ptr(),parsed.codes.len());}}
