// SPDX-License-Identifier: GPL-3.0-or-later
// port of engine/src/platform/options.cpp Menu,InstructionWatch,hints and overlay decisions.
use std::ffi::{c_char,c_void,CStr};
use super::native_options_policy::{wprt_options_choices,wprt_options_live,wprt_options_shown,wprt_options_step,Hover};
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
type RowSink=unsafe extern "C" fn(*mut c_void,*const u8,usize,*const u8,usize,*const u8,usize,bool,bool,bool);
type HintSink=unsafe extern "C" fn(*mut c_void,*const u8,usize,*const u8,usize);
extern "C"{fn wprt_settings_keys(sink:Sink,c:*mut c_void);fn wprt_settings_text(key:*const c_char,var:*const c_char,sink:Sink,c:*mut c_void);fn wprt_settings_store(key:*const c_char,v:*const u8,n:usize);fn wprt_ui_label(key:*const c_char)->*const c_char;fn wprt_ui_label_language(key:*const c_char,language:i32)->*const c_char;fn wphw_options_hint_keys(binding:*const c_void,action:i32,sink:Sink,c:*mut c_void);}
unsafe fn bytes<'a>(p:*const u8,n:usize)->&'a[u8]{if n==0{&[]}else{std::slice::from_raw_parts(p,n)}}
unsafe extern "C" fn collect(c:*mut c_void,p:*const u8,n:usize){(&mut *c.cast::<Vec<Vec<u8>>>()).push(bytes(p,n).to_vec())}
unsafe extern "C" fn assign(c:*mut c_void,p:*const u8,n:usize){*c.cast::<Vec<u8>>()=bytes(p,n).to_vec()}
fn terminated(v:&[u8])->Vec<u8>{let mut out=v.to_vec();out.push(0);out}
unsafe fn keys()->Vec<Vec<u8>>{let mut out=Vec::new();wprt_settings_keys(collect,(&mut out as *mut Vec<Vec<u8>>).cast());out}
unsafe fn text(key:&[u8])->Vec<u8>{let mut out=Vec::new();let key=terminated(key);wprt_settings_text(key.as_ptr().cast(),std::ptr::null(),assign,(&mut out as *mut Vec<u8>).cast());out}
fn enabled(v:&[u8])->bool{![b"0".as_slice(),b"false",b"off",b"no"].contains(&v)}
// port of options.cpp Menu::set_open. Native private fields passed individually.
#[no_mangle]pub unsafe extern "C" fn wprt_options_set_open(value:bool,open:*mut bool,hover:*mut Hover){*open=value;*hover=Hover{row:-1,part:0}}
// port of options.cpp Menu::set_hover.
#[no_mangle]pub unsafe extern "C" fn wprt_options_set_hover(value:*const Hover,hover:*mut Hover)->bool{if (*hover).row==(*value).row&&(*hover).part==(*value).part{return false}*hover=Hover{row:(*value).row,part:(*value).part};true}
// port of options.cpp Menu::select.
#[no_mangle]pub unsafe extern "C" fn wprt_options_select(index:usize,selection:*mut usize){let count=keys().len();if index<count{*selection=index}}
// port of options.cpp Menu::rows;native sink appends each row at the original loop point.
#[no_mangle]pub unsafe extern "C" fn wprt_options_rows(sink:RowSink,c:*mut c_void){for key in keys(){let zkey=terminated(&key);let label=CStr::from_ptr(wprt_ui_label(zkey.as_ptr().cast())).to_bytes().to_vec();let current=text(&key);let mut shown=Vec::new();wprt_options_shown(key.as_ptr(),key.len(),current.as_ptr(),current.len(),assign,(&mut shown as *mut Vec<u8>).cast());let live=wprt_options_live(key.as_ptr(),key.len());let mut choices=Vec::new();wprt_options_choices(key.as_ptr(),key.len(),collect,(&mut choices as *mut Vec<Vec<u8>>).cast());let toggle=choices.len()==2&&choices[0]==b"0"&&choices[1]==b"1";let on=toggle&&enabled(&text(&key));sink(c,key.as_ptr(),key.len(),label.as_ptr(),label.len(),shown.as_ptr(),shown.len(),live,toggle,on)}}
// port of options.cpp Menu::handle;source Action enum follows wp/options.h.
#[no_mangle]pub unsafe extern "C" fn wprt_options_handle(action:i32,selection:*mut usize,open:*mut bool,sink:Sink,c:*mut c_void){let all=keys();let mut result=Vec::new();if !all.is_empty(){let count=all.len();*selection=(*selection).min(count-1);match action{0=>*selection=selection.read().wrapping_add(count).wrapping_sub(1)%count,1=>*selection=selection.read().wrapping_add(1)%count,4=>*open=false,2|3=>{let key=&all[*selection];let current=text(key);let mut next=Vec::new();wprt_options_step(key.as_ptr(),key.len(),current.as_ptr(),current.len(),if action==3{1}else{-1},assign,(&mut next as *mut Vec<u8>).cast());if next!=current{let z=terminated(key);wprt_settings_store(z.as_ptr().cast(),next.as_ptr(),next.len());result=key.clone()}},_=>{}}}sink(c,result.as_ptr(),result.len())}
#[repr(C)]pub struct Span{pub p:*const u8,pub n:usize}
// port of options.cpp InstructionWatch::update;string lengths preserve embedded NUL in module names.
#[no_mangle]pub unsafe extern "C" fn wprt_options_watch(modules:*const Span,n:usize,instruction:*const c_char,prefix:*const c_char,after:*mut bool)->bool{let ins=if instruction.is_null(){None}else{Some(CStr::from_ptr(instruction).to_bytes())};let prefix=if prefix.is_null(){&[]}else{CStr::from_ptr(prefix).to_bytes()};let mut instruction_loaded=false;let mut minigame_loaded=false;let mut other_loaded=false;for i in 0..n{let s=&*modules.add(i);let name=bytes(s.p,s.n);if ins==Some(name){instruction_loaded=true}else if !prefix.is_empty()&&name.starts_with(prefix){minigame_loaded=true}else{other_loaded=true}}if minigame_loaded{*after=true}else if other_loaded{*after=false}instruction_loaded&&!*after}
unsafe fn label(key:&[u8],language:i32)->Vec<u8>{let z=terminated(key);CStr::from_ptr(wprt_ui_label_language(z.as_ptr().cast(),language)).to_bytes().to_vec()}
unsafe fn hint_keys(binding:*const c_void,action:i32)->Vec<u8>{let mut out=Vec::new();wphw_options_hint_keys(binding,action,assign,(&mut out as *mut Vec<u8>).cast());out}
unsafe fn joined(binding:*const c_void,actions:&[i32])->Vec<u8>{let mut out=Vec::new();for &action in actions{let keys=hint_keys(binding,action);if !keys.is_empty(){if !out.is_empty(){out.extend_from_slice(b"  ")}out.extend_from_slice(&keys)}}out}
// port of options.cpp hint_rows,with native Bindings/format and native vector append leaves.
#[no_mangle]pub unsafe extern "C" fn wprt_options_hints(binding:*const c_void,language:i32,sink:HintSink,c:*mut c_void){let pointer=label(b"hint.pointer",language);let mouse=label(b"hint.mouse",language);sink(c,pointer.as_ptr(),pointer.len(),mouse.as_ptr(),mouse.len());for (name,action) in [(b"A".as_slice(),ACTION_A),(b"B",ACTION_B),(b"1",ACTION_ONE),(b"2",ACTION_TWO),(b"+",ACTION_PLUS),(b"-",ACTION_MINUS)]{let keys=hint_keys(binding,action);if !keys.is_empty(){sink(c,name.as_ptr(),name.len(),keys.as_ptr(),keys.len())}}for (name,actions) in [(b"hint.dpad".as_slice(),&[ACTION_UP,ACTION_DOWN,ACTION_LEFT,ACTION_RIGHT][..]),(b"hint.shake",&[ACTION_SHAKE][..]),(b"hint.swing",&[ACTION_SWINGUP,ACTION_SWINGDOWN][..]),(b"hint.tilt",&[ACTION_TILTLEFT,ACTION_TILTRIGHT,ACTION_TILTUP,ACTION_TILTDOWN][..])]{let name=label(name,language);let keys=if actions.len()==1{hint_keys(binding,actions[0])}else{joined(binding,actions)};if !keys.is_empty(){sink(c,name.as_ptr(),name.len(),keys.as_ptr(),keys.len())}}}
// port of options.cpp show_overlay;native source mutex is held and callback moves original vector once.
#[no_mangle]pub unsafe extern "C" fn wprt_options_overlay_show(visible:*mut bool,version:*mut u64,c:*mut c_void,move_image:unsafe extern "C" fn(*mut c_void)){move_image(c);*visible=true;*version=(*version).wrapping_add(1)}
// port of options.cpp hide_overlay under the original native mutex.
#[no_mangle]pub unsafe extern "C" fn wprt_options_overlay_hide(visible:*mut bool,version:*mut u64){if !*visible{return}*visible=false;*version=(*version).wrapping_add(1)}
// port of options.cpp take_overlay;native callback preserves original move-then-reset order.
#[no_mangle]pub unsafe extern "C" fn wprt_options_overlay_take(current:*const u64,current_visible:*const bool,version:*mut u64,visible:*mut bool,c:*mut c_void,move_reset:unsafe extern "C" fn(*mut c_void))->bool{if *version==*current{return false}*version=*current;*visible=*current_visible;move_reset(c);true}

const ACTION_A:i32=0;
const ACTION_B:i32=1;
const ACTION_ONE:i32=2;
const ACTION_TWO:i32=3;
const ACTION_PLUS:i32=4;
const ACTION_MINUS:i32=5;
const ACTION_HOME:i32=6;
const ACTION_UP:i32=7;
const ACTION_DOWN:i32=8;
const ACTION_LEFT:i32=9;
const ACTION_RIGHT:i32=10;
const ACTION_SHAKE:i32=11;
const ACTION_SWINGUP:i32=12;
const ACTION_SWINGDOWN:i32=13;
const ACTION_TILTLEFT:i32=14;
const ACTION_TILTRIGHT:i32=15;
const ACTION_TILTUP:i32=16;
const ACTION_TILTDOWN:i32=17;
const ACTION_GRIP:i32=18;
const ACTION_SCREENSHOT:i32=19;
const ACTION_COUNT:i32=20;
