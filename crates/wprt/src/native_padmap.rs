// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/input/padmap.cpp; source descriptors/physical enum copied below.
use crate::native_settings;
use std::ffi::{c_char,c_void};
// Enum, names and action/button tables copied programmatically from pristine headers/source.
#[allow(non_upper_case_globals)]const kSouth:usize=0;
#[allow(non_upper_case_globals)]const kEast:usize=1;
#[allow(non_upper_case_globals)]const kWest:usize=2;
#[allow(non_upper_case_globals)]const kNorth:usize=3;
#[allow(non_upper_case_globals)]const kBack:usize=4;
#[allow(non_upper_case_globals)]const kGuide:usize=5;
#[allow(non_upper_case_globals)]const kStart:usize=6;
#[allow(non_upper_case_globals)]const kLeftStick:usize=7;
#[allow(non_upper_case_globals)]const kRightStick:usize=8;
#[allow(non_upper_case_globals)]const kLeftShoulder:usize=9;
#[allow(non_upper_case_globals)]const kRightShoulder:usize=10;
#[allow(non_upper_case_globals)]const kDpadUp:usize=11;
#[allow(non_upper_case_globals)]const kDpadDown:usize=12;
#[allow(non_upper_case_globals)]const kDpadLeft:usize=13;
#[allow(non_upper_case_globals)]const kDpadRight:usize=14;
#[allow(non_upper_case_globals)]const kLeftTrigger:usize=15;
#[allow(non_upper_case_globals)]const kRightTrigger:usize=16;
#[allow(non_upper_case_globals)]const kPhysicalCount:usize=17;
#[allow(non_upper_case_globals)]const kAutomatic:i32=-1;
#[allow(non_upper_case_globals)]const kActionCount:usize=7;
#[allow(non_upper_case_globals)]const kButtonA:u32=0x0800;
#[allow(non_upper_case_globals)]const kButtonB:u32=0x0400;
#[allow(non_upper_case_globals)]const kButtonOne:u32=0x0200;
#[allow(non_upper_case_globals)]const kButtonTwo:u32=0x0100;
#[allow(non_upper_case_globals)]const kButtonPlus:u32=0x0010;
#[allow(non_upper_case_globals)]const kButtonMinus:u32=0x1000;
#[allow(non_upper_case_globals)]const kButtonHome:u32=0x8000;
const ACTIONS:&[(&[u8],u32)]=&[(b"a\0",kButtonA),(b"b\0",kButtonB),(b"one\0",kButtonOne),(b"two\0",kButtonTwo),(b"plus\0",kButtonPlus),(b"minus\0",kButtonMinus),(b"home\0",kButtonHome)];
const PHYSICAL_NAMES:&[&[u8]]=&[b"south\0",b"east\0",b"west\0",b"north\0",b"back\0",b"guide\0",b"start\0",b"left_stick\0",b"right_stick\0",b"left_shoulder\0",b"right_shoulder\0",b"dpad_up\0",b"dpad_down\0",b"dpad_left\0",b"dpad_right\0",b"left_trigger\0",b"right_trigger\0"];
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
// port of padmap.cpp action_name / action_button / setting_key.
#[no_mangle]pub extern "C" fn wprt_padmap_action_name(action:usize)->*const c_char{if action<ACTIONS.len(){ACTIONS[action].0.as_ptr().cast()}else{c"".as_ptr()}}
#[no_mangle]pub extern "C" fn wprt_padmap_action_button(action:usize)->u32{ACTIONS.get(action).map_or(0,|a|a.1)}
#[no_mangle]pub unsafe extern "C" fn wprt_padmap_setting_key(action:usize,sink:Sink,context:*mut c_void){let mut key=b"pad.".to_vec();if let Some((name,_))=ACTIONS.get(action){key.extend_from_slice(&name[..name.len()-1]);}sink(context,key.as_ptr(),key.len());}
// port of padmap.cpp physical_name / physical_from_name.
#[no_mangle]pub extern "C" fn wprt_padmap_physical_name(physical:i32)->*const c_char{if physical>=0&&(physical as usize)<PHYSICAL_NAMES.len(){PHYSICAL_NAMES[physical as usize].as_ptr().cast()}else{c"auto".as_ptr()}}
fn physical_from_name(name:&[u8])->i32{for(i,physical)in PHYSICAL_NAMES.iter().enumerate(){if name==&physical[..physical.len()-1]{return i as i32;}}kAutomatic}
#[no_mangle]pub unsafe extern "C" fn wprt_padmap_physical_from_name(p:*const u8,n:usize)->i32{physical_from_name(std::slice::from_raw_parts(p,n))}
unsafe extern "C" fn bytes(context:*mut c_void,p:*const u8,n:usize){*context.cast::<Vec<u8>>()=std::slice::from_raw_parts(p,n).to_vec();}
// port of padmap.cpp current_mapping. Native caller marshals int/std::array without guessing object layout.
#[no_mangle]pub unsafe extern "C" fn wprt_padmap_current_mapping(out:*mut i32){for(i,(name,_))in ACTIONS.iter().enumerate(){let mut key=b"pad.".to_vec();key.extend_from_slice(name);let mut value=Vec::new();native_settings::wprt_settings_text(key.as_ptr().cast(),std::ptr::null(),bytes,(&mut value as *mut Vec<u8>).cast());*out.add(i)=physical_from_name(&value);}}
// port of padmap.cpp automatic; D-pad is intentionally not part of this source helper.
fn automatic(held:&[u8;kPhysicalCount],nintendo:bool,sideways:bool)->u32{let h=|i:usize|held[i]!=0;let mut b=0;if h(kRightShoulder){b|=kButtonA;}if h(kRightTrigger){b|=kButtonB;}if sideways{if h(kSouth){b|=kButtonTwo;}if h(kWest)||h(kEast){b|=kButtonOne;}if h(kNorth){b|=kButtonA;}}else{if h(if nintendo{kEast}else{kSouth}){b|=kButtonA;}if h(if nintendo{kSouth}else{kEast}){b|=kButtonB;}if h(kWest){b|=kButtonOne;}if h(kNorth){b|=kButtonTwo;}}if h(kBack){b|=kButtonMinus;}if h(kStart){b|=kButtonPlus;}if h(kGuide){b|=kButtonHome;}b}
// port of padmap.cpp buttons; keep sideways override, physical consumption and per-action defaults.
#[no_mangle]pub unsafe extern "C" fn wprt_padmap_buttons(held:*const u8,nintendo:bool,sideways:bool,mapping:*const i32)->u32{let held=&*held.cast::<[u8;kPhysicalCount]>();let mapping=&*mapping.cast::<[i32;kActionCount]>();let mut custom=false;for physical in mapping{custom=custom||*physical!=kAutomatic;}if !custom||sideways{return automatic(held,nintendo,sideways);}let mut unmapped=*held;for &physical in mapping{if physical!=kAutomatic{unmapped[physical as usize]=0;}}let defaults=automatic(&unmapped,nintendo,false);let mut b=0;for i in 0..kActionCount{if mapping[i]==kAutomatic{b|=defaults&ACTIONS[i].1;}else if held[mapping[i]as usize]!=0{b|=ACTIONS[i].1;}}b}
