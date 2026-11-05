// SPDX-License-Identifier: GPL-3.0-or-later
// port of accepted H6 hybrid/cpp/patched/host_io.cpp;host input transport only,no game rules.
use std::sync::Mutex;
#[derive(Clone,Copy,Default)]struct Sample{buttons:u32,pointer_valid:bool,pointer_x:f32,pointer_y:f32,extended:bool,focused:bool,grip:bool}
static SAMPLES:Mutex<[Sample;4]>=Mutex::new([Sample{buttons:0,pointer_valid:false,pointer_x:0.0,pointer_y:0.0,extended:false,focused:false,grip:false};4]);
// port of H6 wphost_input:source channel guard precedes the lock.
#[no_mangle]pub unsafe extern "C" fn wphost_input(channel:u32,buttons:u32,pointer:bool,x:f32,y:f32){if channel>=4{return}let mut s=SAMPLES.lock().unwrap();s[channel as usize]=Sample{buttons,pointer_valid:pointer,pointer_x:x,pointer_y:y,extended:false,focused:false,grip:false};}
// port of H6 wp::host_io::sample:take source lock even for an invalid channel;return source default fields then.
#[no_mangle]pub unsafe extern "C" fn wprt_host_sample(channel:u32,buttons:*mut u32,pointer:*mut bool,x:*mut f32,y:*mut f32){let s=SAMPLES.lock().unwrap();let v=if channel<4{s[channel as usize]}else{Sample::default()};*buttons=v.buttons;*pointer=v.pointer_valid;*x=v.pointer_x;*y=v.pointer_y;}

// H8 transport: one locked snapshot of Bevy buttons/pointer/focus/grip,not guest rules.
#[no_mangle]pub unsafe extern "C" fn wphost_input_v2(channel:u32,buttons:u32,pointer:bool,x:f32,y:f32,focused:bool,grip:bool){if channel>=4{return}let mut s=SAMPLES.lock().unwrap();s[channel as usize]=Sample{buttons,pointer_valid:pointer,pointer_x:x,pointer_y:y,extended:true,focused,grip};}
#[no_mangle]pub unsafe extern "C" fn wprt_host_sample_v2(channel:u32,buttons:*mut u32,pointer:*mut bool,x:*mut f32,y:*mut f32,focused:*mut bool,grip:*mut bool)->bool{let s=SAMPLES.lock().unwrap();let v=if channel<4{s[channel as usize]}else{Sample::default()};*buttons=v.buttons;*pointer=v.pointer_valid;*x=v.pointer_x;*y=v.pointer_y;*focused=v.focused;*grip=v.grip;v.extended}
