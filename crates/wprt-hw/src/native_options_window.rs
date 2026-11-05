// SPDX-License-Identifier: GPL-3.0-or-later
// port of engine/src/platform/options_window.cpp menu/input/hint decisions.
// Native Menu/Repeat storage,clock reads,settings cache,render/device/log leaves remain native.
use std::ffi::c_void;
use super::native_options_policy::Hover;
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C" {
fn wphw_ow_open()->bool;fn wphw_ow_enabled()->bool;fn wphw_ow_set_open(open:bool);
fn wphw_ow_sync(window:*mut c_void);fn wphw_ow_toggle(window:*mut c_void);
fn wphw_ow_act(window:*mut c_void,action:i32,sink:Sink,c:*mut c_void);
fn wphw_ow_hit(x:i32,y:i32,out:*mut Hover)->bool;fn wphw_ow_select(index:usize);fn wphw_ow_set_hover(hit:*const Hover)->bool;
fn wphw_ow_update_hint();fn wphw_ow_now()->i64;fn wphw_ow_focused()->bool;fn wphw_ow_buttons()->u32;fn wphw_ow_repeat(repeat:*mut c_void,held:u32,now:i64)->u32;
fn wphw_ow_hint_enabled()->bool;fn wphw_ow_client(width:*mut i32,height:*mut i32)->bool;fn wphw_ow_instruction()->bool;
fn wphw_ow_hint_chosen()->i32;fn wphw_ow_spanish()->bool;fn wphw_ow_hint_render(width:i32,height:i32);fn wphw_ow_hint_log(shown:bool);
fn wphw_ow_menu_render();fn wphw_ow_hide();fn wphw_ow_block(block:bool);
fn wphw_ow_handle(action:i32,sink:Sink,c:*mut c_void);
fn wphw_ow_instruction_now()->i64;fn wphw_ow_instruction_update()->bool;
}
unsafe fn bytes<'a>(p:*const u8,n:usize)->&'a[u8]{if n==0{&[]}else{std::slice::from_raw_parts(p,n)}}
unsafe extern "C" fn assign(c:*mut c_void,p:*const u8,n:usize){*c.cast::<Vec<u8>>()=bytes(p,n).to_vec()}
unsafe fn act(w:*mut c_void,action:i32)->Vec<u8>{let mut out=Vec::new();wphw_ow_handle(action,assign,(&mut out as *mut Vec<u8>).cast());wphw_ow_sync(w);out}
unsafe fn emit(v:&[u8],sink:Sink,c:*mut c_void){sink(c,v.as_ptr(),v.len())}
// port of options_window.cpp act:handle before sync before returning changed key.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_act(w:*mut c_void,action:i32,sink:Sink,c:*mut c_void){let changed=act(w,action);emit(&changed,sink,c)}
// port of client_height:source client_size failure returns0,not old height.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_client_height()->i32{let mut width=0;let mut height=0;if wphw_ow_client(&mut width,&mut height){height}else{0}}
static mut INSTRUCTION_NEXT:i64=0;
static mut INSTRUCTION_CACHED:bool=false;
// port of instruction_screen. Adapter returns original steady_clock duration
// ticks;native compile asserts nanosecond period.100ms converts exactly100000000.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_instruction()->bool{
    let now=wphw_ow_instruction_now();
    if now>=INSTRUCTION_NEXT{INSTRUCTION_NEXT=now.wrapping_add(100_000_000);INSTRUCTION_CACHED=wphw_ow_instruction_update()}
    INSTRUCTION_CACHED
}
// port of options_window.cpp hint_language,including source && short circuit.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_hint_language()->i32{let chosen=wphw_ow_hint_chosen();if chosen==0&&wphw_ow_spanish(){1}else{chosen}}
// port of options_window.cpp update_hint. Native flags passed individually;no class-layout assumptions.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_update_hint(shown:*mut bool,old_width:*mut i32,old_height:*mut i32){let mut width=0;let mut height=0;let wanted=wphw_ow_hint_enabled()&&!wphw_ow_open()&&wphw_ow_client(&mut width,&mut height)&&wphw_ow_instruction();if wanted{if !*shown||width!=*old_width||height!=*old_height{if !*shown{wphw_ow_hint_log(true)}*shown=true;*old_width=width;*old_height=height;wphw_ow_hint_render(width,height)}}else if *shown{*shown=false;wphw_ow_hide();wphw_ow_hint_log(false)}}
// port of options_window.cpp sync;render/move and input-block calls occur in original source order.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_sync(shown:*mut bool,hint_shown:*mut bool){if wphw_ow_open(){*hint_shown=false;wphw_ow_menu_render();if !*shown{*shown=true;wphw_ow_block(true)}}else if *shown{*shown=false;wphw_ow_hide();wphw_ow_block(false)}}
// port of options_window.cpp menu_open.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_open()->bool{wphw_ow_open()}
// port of options_window.cpp toggle_menu;the source reads open() again after checking menu_enabled().
#[no_mangle]pub unsafe extern "C" fn wprt_ow_toggle(w:*mut c_void){if !wphw_ow_open()&&!wphw_ow_enabled(){return}wphw_ow_set_open(!wphw_ow_open());wphw_ow_sync(w)}
// port of options_window.cpp menu_key;key values extracted directly from source declarations.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_key(w:*mut c_void,key:u32,repeat:bool,sink:Sink,c:*mut c_void){let result=if !wphw_ow_open(){Vec::new()}else{match key{kKeyUp=>act(w,0),kKeyDown=>act(w,1),kKeyLeft=>act(w,2),kKeyRight=>act(w,3),kKeyReturn|kKeySpace=>if repeat{Vec::new()}else{act(w,3)},kKeyEscape|kKeyBack=>if repeat{Vec::new()}else{act(w,4)},_=>Vec::new()}};emit(&result,sink,c)}
// port of options_window.cpp menu_wheel.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_wheel(w:*mut c_void,delta:i32,sink:Sink,c:*mut c_void){let result=if !wphw_ow_open()||delta==0{Vec::new()}else{act(w,if delta>0{0}else{1})};emit(&result,sink,c)}
// port of options_window.cpp menu_click;Part/Action enumerators follow the original wp/options.h.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_click(w:*mut c_void,x:i32,y:i32,right:bool,sink:Sink,c:*mut c_void){let result=if !wphw_ow_open(){Vec::new()}else if right{act(w,4)}else{let mut hit=Hover{row:-1,part:0};if !wphw_ow_hit(x,y,&mut hit){Vec::new()}else if hit.part==5{act(w,4)}else if hit.row<0{Vec::new()}else{wphw_ow_select(hit.row as usize);if hit.part==1{wphw_ow_sync(w);Vec::new()}else{act(w,if hit.part==2{2}else{3})}}};emit(&result,sink,c)}
// port of options_window.cpp menu_hover;failed client_size keeps the source default Hover.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_hover(w:*mut c_void,x:i32,y:i32){if !wphw_ow_open(){return}let mut hit=Hover{row:-1,part:0};wphw_ow_hit(x,y,&mut hit);if wphw_ow_set_hover(&hit){wphw_ow_sync(w)}}
// port of options_window.cpp poll_gamepads;native wrapper constructs static Repeat before this call.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_poll(w:*mut c_void,repeat:*mut c_void,sink:Sink,c:*mut c_void){wphw_ow_update_hint();let now=wphw_ow_now();let focused=wphw_ow_focused();let fired=wphw_ow_repeat(repeat,if focused{wphw_ow_buttons()}else{0},now);let result=if fired&kMenuToggle!=0{wphw_ow_toggle(w);Vec::new()}else if !wphw_ow_open()||fired==0{Vec::new()}else if fired&kMenuCancel!=0{act(w,4)}else{let mut changed=Vec::new();if fired&kMenuUp!=0{act(w,0);}if fired&kMenuDown!=0{act(w,1);}if fired&kMenuLeft!=0{changed=act(w,2)}if fired&(kMenuRight|kMenuAccept)!=0{changed=act(w,3)}changed};emit(&result,sink,c)}
// port of options_window.cpp refresh_menu.
#[no_mangle]pub unsafe extern "C" fn wprt_ow_refresh(w:*mut c_void){if wphw_ow_open(){wphw_ow_sync(w)}}

const kKeyBack:u32=0x08;
const kKeyReturn:u32=0x0D;
const kKeyEscape:u32=0x1B;
const kKeySpace:u32=0x20;
const kKeyLeft:u32=0x25;
const kKeyUp:u32=0x26;
const kKeyRight:u32=0x27;
const kKeyDown:u32=0x28;
const kMenuUp:u32=0x01;
const kMenuDown:u32=0x02;
const kMenuLeft:u32=0x04;
const kMenuRight:u32=0x08;
const kMenuAccept:u32=0x10;
const kMenuCancel:u32=0x20;
const kMenuToggle:u32=0x40;
