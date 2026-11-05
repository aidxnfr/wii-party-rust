// SPDX-License-Identifier: GPL-3.0-or-later
// port of pristine input/gamepad.cpp SDL operations;bindings from owned SDL3.4.16 headers.
use std::{ffi::{c_void,c_char},mem::{zeroed,size_of,MaybeUninit}};
#[path="sdk_constants.rs"]mod sdk;use sdk::*;
#[repr(C)]#[derive(Clone,Copy)]struct Sensor{kind:i32,rate:f32}
#[repr(C)]struct Touchpad{nfingers:u16,padding:[u16;3]}
// Exact public SDL_joystick.h:477-523 descriptor,including unused optional callback slots.
#[repr(C)]struct VirtualDesc{version:u32,kind:u16,padding:u16,vendor:u16,product:u16,naxes:u16,nbuttons:u16,nballs:u16,nhats:u16,ntouchpads:u16,nsensors:u16,padding2:[u16;2],button_mask:u32,axis_mask:u32,name:*const c_char,touchpads:*const Touchpad,sensors:*const Sensor,userdata:*mut c_void,update:Option<unsafe extern "C" fn(*mut c_void)>,player:Option<unsafe extern "C" fn(*mut c_void,i32)>,rumble:Option<unsafe extern "C" fn(*mut c_void,u16,u16)->bool>,triggers:Option<unsafe extern "C" fn(*mut c_void,u16,u16)->bool>,led:Option<unsafe extern "C" fn(*mut c_void,u8,u8,u8)->bool>,effect:Option<unsafe extern "C" fn(*mut c_void,*const c_void,i32)->bool>,sensors_enabled:Option<unsafe extern "C" fn(*mut c_void,bool)->bool>,cleanup:Option<unsafe extern "C" fn(*mut c_void)>}
#[repr(C)]#[derive(Clone,Copy)]struct DeviceEvent{kind:u32,reserved:u32,timestamp:u64,which:u32}
// Used members plus official SDL_Event padding[128];u64 timestamp retains source union alignment.
#[repr(C)]union Event{kind:u32,gdevice:DeviceEvent,padding:[u8;128]}
#[link(name="SDL3")]extern "C"{
 fn SDL_GetGamepadAxis(p:*mut c_void,a:i32)->i16;fn SDL_OpenGamepad(id:u32)->*mut c_void;fn SDL_CloseGamepad(p:*mut c_void);
 fn SDL_GamepadHasSensor(p:*mut c_void,kind:i32)->bool;fn SDL_SetGamepadSensorEnabled(p:*mut c_void,kind:i32,on:bool)->bool;
 fn SDL_GetGamepadButtonLabel(p:*mut c_void,b:i32)->i32;fn SDL_GetGamepadName(p:*mut c_void)->*const c_char;
 fn SDL_GetGamepadButton(p:*mut c_void,b:i32)->bool;fn SDL_GetGamepadSensorData(p:*mut c_void,kind:i32,out:*mut f32,n:i32)->bool;
 fn SDL_RumbleGamepad(p:*mut c_void,low:u16,high:u16,duration:u32)->bool;fn SDL_SetGamepadPlayerIndex(p:*mut c_void,index:i32)->bool;
 fn SDL_AttachVirtualJoystick(d:*const VirtualDesc)->u32;fn SDL_OpenJoystick(id:u32)->*mut c_void;fn SDL_CloseJoystick(p:*mut c_void);fn SDL_DetachVirtualJoystick(id:u32)->bool;
 fn SDL_SetJoystickVirtualButton(p:*mut c_void,b:i32,on:bool)->bool;fn SDL_GetTicksNS()->u64;
 fn SDL_SendJoystickVirtualSensorData(p:*mut c_void,kind:i32,timestamp:u64,data:*const f32,n:i32)->bool;
 fn SDL_SetHint(name:*const c_char,value:*const c_char)->bool;fn SDL_Init(flags:u32)->bool;fn SDL_UpdateJoysticks();fn SDL_WasInit(flags:u32)->u32;fn SDL_PumpEvents();
 fn SDL_PeepEvents(event:*mut Event,n:i32,action:i32,min:u32,max:u32)->i32;
 fn wphw_gamepad_virtual_log_rumble(number:u32,low:u16,high:u16);fn wphw_gamepad_virtual_log_player(number:u32,index:i32);
}
fn sensor(gyro:bool)->i32{if gyro{SDL_SENSOR_GYRO as i32}else{SDL_SENSOR_ACCEL as i32}}
// port of source axis/open/close/update_snapshot SDL calls;all result/ignored-return semantics preserved.
// port of engine/src/input/gamepad.cpp axis SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_axis(p:*mut c_void,a:i32)->i16{SDL_GetGamepadAxis(p,a)}
// port of engine/src/input/gamepad.cpp open SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_open(id:u32)->*mut c_void{SDL_OpenGamepad(id)}
// port of engine/src/input/gamepad.cpp close SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_close(p:*mut c_void){SDL_CloseGamepad(p)}
// port of engine/src/input/gamepad.cpp open SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_has_sensor(p:*mut c_void,gyro:bool)->bool{SDL_GamepadHasSensor(p,sensor(gyro))}
// port of engine/src/input/gamepad.cpp open SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_enable_sensor(p:*mut c_void,gyro:bool)->bool{SDL_SetGamepadSensorEnabled(p,sensor(gyro),true)}
// port of engine/src/input/gamepad.cpp open SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_nintendo(p:*mut c_void)->bool{SDL_GetGamepadButtonLabel(p,SDL_GAMEPAD_BUTTON_SOUTH as i32)==SDL_GAMEPAD_BUTTON_LABEL_B as i32}
// port of engine/src/input/gamepad.cpp open SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_name(p:*mut c_void)->*const c_char{SDL_GetGamepadName(p)}
// port of engine/src/input/gamepad.cpp read SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_button(p:*mut c_void,b:i32)->bool{SDL_GetGamepadButton(p,b)}
// port of engine/src/input/gamepad.cpp read SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_sensor(p:*mut c_void,gyro:bool,out:*mut f32)->bool{SDL_GetGamepadSensorData(p,sensor(gyro),out,3)}
// port of engine/src/input/gamepad.cpp apply_outputs SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_rumble(p:*mut c_void,low:u16,high:u16,duration:u32){SDL_RumbleGamepad(p,low,high,duration);}
// port of engine/src/input/gamepad.cpp apply_outputs SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_player(p:*mut c_void,index:i32){SDL_SetGamepadPlayerIndex(p,index);}
// port of attach_virtual callbacks;original formatted logger remains the native printf SDK owner.
unsafe extern "C" fn virtual_rumble(p:*mut c_void,low:u16,high:u16)->bool{wphw_gamepad_virtual_log_rumble(*p.cast::<u32>(),low,high);true}
unsafe extern "C" fn virtual_player(p:*mut c_void,index:i32){wphw_gamepad_virtual_log_player(*p.cast::<u32>(),index);}
static mut NUMBERS:[u32;4]=[0;4];static SENSORS:[Sensor;2]=[Sensor{kind:SDL_SENSOR_ACCEL as i32,rate:250.0},Sensor{kind:SDL_SENSOR_GYRO as i32,rate:250.0}];
// port of attach_virtual source descriptor initialization (SDL_INIT_INTERFACE=zeros+sizeof),stable callback userdata.
// port of engine/src/input/gamepad.cpp attach_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_virtual_attach(number:*mut u32)->u32{let value=*number;let p=std::ptr::addr_of_mut!(NUMBERS).cast::<u32>().add(value as usize-1);*p=value;let mut desc:VirtualDesc=zeroed();desc.version=size_of::<VirtualDesc>() as u32;desc.kind=SDL_JOYSTICK_TYPE_GAMEPAD as u16;desc.naxes=SDL_GAMEPAD_AXIS_COUNT as u16;desc.nbuttons=SDL_GAMEPAD_BUTTON_COUNT as u16;desc.nsensors=2;desc.sensors=SENSORS.as_ptr();desc.name=b"Virtual test gamepad\0".as_ptr().cast();desc.userdata=p.cast();desc.rumble=Some(virtual_rumble);desc.player=Some(virtual_player);SDL_AttachVirtualJoystick(&desc)}
// port of engine/src/input/gamepad.cpp attach_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_joystick_open(id:u32)->*mut c_void{SDL_OpenJoystick(id)}
// port of engine/src/input/gamepad.cpp drive_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_joystick_close(p:*mut c_void){SDL_CloseJoystick(p)}
// port of engine/src/input/gamepad.cpp drive_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_virtual_detach(id:u32){SDL_DetachVirtualJoystick(id);}
// port of engine/src/input/gamepad.cpp drive_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_virtual_button(p:*mut c_void,b:i32,on:bool){SDL_SetJoystickVirtualButton(p,b,on);}
// port of engine/src/input/gamepad.cpp drive_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_ticks_ns()->u64{SDL_GetTicksNS()}
// port of engine/src/input/gamepad.cpp drive_virtual SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_virtual_sensor(p:*mut c_void,gyro:bool,time:u64,data:*const f32){SDL_SendJoystickVirtualSensorData(p,sensor(gyro),time,data,3);}
// port of source worker hint ordering;values generated from installed official header,not hand-numbered.
// port of engine/src/input/gamepad.cpp worker SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_hint(which:u32,value:bool){let names=[HINT0,HINT1,HINT2,HINT3,HINT4];SDL_SetHint(names[which as usize].as_ptr().cast(),if value{b"1\0".as_ptr()}else{b"0\0".as_ptr()}.cast());}
// port of engine/src/input/gamepad.cpp worker SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_init()->bool{SDL_Init(SDL_INIT_GAMEPAD)}
// port of engine/src/input/gamepad.cpp worker SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_update(){SDL_UpdateJoysticks()}
// port of engine/src/input/gamepad.cpp worker SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_owns_queue()->bool{SDL_WasInit(SDL_INIT_VIDEO)==0}
// port of engine/src/input/gamepad.cpp worker SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_pump(){SDL_PumpEvents()}
// port of worker's original PeepEvents range/classification;source leaves output fields unchanged on no event.
// port of engine/src/input/gamepad.cpp worker SDL SDK operation.
#[no_mangle]pub unsafe extern "C" fn wphw_gamepad_event(own:bool,kind:*mut u32,id:*mut u32)->bool{let mut event=MaybeUninit::<Event>::uninit();if SDL_PeepEvents(event.as_mut_ptr(),1,SDL_GETEVENT as i32,if own{SDL_EVENT_FIRST}else{SDL_EVENT_JOYSTICK_AXIS_MOTION},if own{SDL_EVENT_LAST}else{SDL_EVENT_FINGER_DOWN-1})<=0{return false}let event=event.assume_init();*kind=if event.kind==SDL_EVENT_GAMEPAD_ADDED{1}else if event.kind==SDL_EVENT_GAMEPAD_REMOVED{2}else{0};*id=if *kind!=0{event.gdevice.which}else{0};true}
