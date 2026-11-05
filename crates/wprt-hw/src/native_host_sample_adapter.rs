// SPDX-License-Identifier: GPL-3.0-or-later
// H6 legacy field transport plus H8 source input.cpp auto-orientation/grip for extended Bevy channel0.
#[repr(C)]pub struct Sample{pub buttons:u32,pub pointer_valid:bool,pub pointer_x:f32,pub pointer_y:f32,pub motion_valid:bool,pub accel:[f32;3]}
extern "C"{fn wprt_host_sample_v2(channel:u32,buttons:*mut u32,pointer:*mut bool,x:*mut f32,y:*mut f32,focused:*mut bool,grip:*mut bool)->bool;
fn wphw_input_flag(which:u32)->bool;fn wphw_input_now_ms()->i64;fn wphw_input_sideways_count()->usize;fn wphw_input_sideways_loaded(index:usize)->bool;}
// port of input.cpp sideways_minigame: source LiveFlag,100ms cache,real module descriptors.
// Called only from guest input sampling,never from the Bevy/UI thread.
unsafe fn sideways_minigame()->bool{if !wphw_input_flag(0){return false;}static mut NEXT:i64=0;static mut CACHED:bool=false;let now=unsafe{wphw_input_now_ms()};if now>=NEXT{NEXT=now+100;CACHED=false;for i in 0..wphw_input_sideways_count(){if wphw_input_sideways_loaded(i){CACHED=true;break}}}CACHED}
// port of input.cpp sideways_grip with explicit host Grip action replacing video::key_down.
unsafe fn sideways_grip(toggle:bool)->bool{sideways_grip_read(toggle,true)}
unsafe fn sideways_grip_read(toggle:bool,read_toggle:bool)->bool{static mut FLIPPED:bool=false;static mut TOGGLE_HELD:bool=false;if read_toggle{if toggle&&!TOGGLE_HELD{FLIPPED=!FLIPPED;}TOGGLE_HELD=toggle;}sideways_minigame()!=FLIPPED}
// port of host_io::sample defaults;extended channel0 also follows input.cpp focus/grip/D-pad order.
// Legacy snapshots and other channels retain the accepted H6 transport behavior.
#[export_name="_ZN2wp7host_io6sampleEj"]pub unsafe extern "C" fn sample(channel:u32)->Sample{
 #[cfg(feature="rust-host-gamepad")]if crate::native_host_gamepad::bound(){return controller_sample(channel);}
 let mut value=Sample{buttons:0,pointer_valid:false,pointer_x:0.0,pointer_y:0.0,motion_valid:false,accel:[0.0,0.0,1.0]};let(mut focused,mut grip)=(false,false);
 let extended=wprt_host_sample_v2(channel,&mut value.buttons,&mut value.pointer_valid,&mut value.pointer_x,&mut value.pointer_y,&mut focused,&mut grip);
 if !extended||channel!=0{return value;}
 if !focused{return Sample{buttons:0,pointer_valid:false,pointer_x:0.0,pointer_y:0.0,motion_valid:false,accel:[0.0,0.0,1.0]};}
 if sideways_grip(grip){let held=value.buttons;value.buttons&=!0x0f;value.buttons|=if held&0x1!=0{0x8}else{0}|if held&0x2!=0{0x4}else{0}|if held&0x8!=0{0x2}else{0}|if held&0x4!=0{0x1}else{0};value.buttons|=0x800000;}
 value
}

// PRIVATE G32 candidate. Source input.cpp device_sample/gamepad_sample ordering.
// Reuses EXISTING accepted cursor service,not a new Win32/SDK leaf port or guessed event timer.
#[cfg(feature="rust-host-gamepad")]
unsafe fn controller_sample(channel:u32)->Sample{
 let empty=||Sample{buttons:0,pointer_valid:false,pointer_x:0.0,pointer_y:0.0,motion_valid:false,accel:[0.0,0.0,1.0]};
 let mut keys=empty();let(mut focused,mut grip)=(false,false);
 let extended=wprt_host_sample_v2(0,&mut keys.buttons,&mut keys.pointer_valid,&mut keys.pointer_x,&mut keys.pointer_y,&mut focused,&mut grip);
 if !extended||!focused{return empty();}
 #[cfg(feature="rust-host-keyboard")]if channel==0&&crate::native_host_keyboard::bound(){let state=crate::native_host_keyboard::sample();keys.buttons=state.0;grip=state.1;}
 let sideways=sideways_grip_read(grip,channel==0);
 if channel!=0{let pad=crate::native_gamepad_adapter::poll(channel,sideways);return Sample{buttons:pad.buttons|if sideways{0x800000}else{0},pointer_valid:pad.pointer_valid,pointer_x:pad.pointer_x,pointer_y:pad.pointer_y,motion_valid:pad.motion_valid,accel:pad.accel};}
 if sideways{let held=keys.buttons;keys.buttons&=!0x0f;keys.buttons|=if held&1!=0{8}else{0}|if held&2!=0{4}else{0}|if held&8!=0{2}else{0}|if held&4!=0{1}else{0};keys.buttons|=0x800000;}
 let pad=crate::native_gamepad_adapter::poll(channel,sideways);
 keys.buttons|=pad.buttons;
 if pad.motion_valid{keys.motion_valid=true;keys.accel=pad.accel;}
 // Source records screen cursor coordinates AFTER pad sample;strict1000ms recency.
 extern "C"{fn wphw_input_cursor(x:*mut i32,y:*mut i32)->bool;}
 static mut LAST_X:i32=0;static mut LAST_Y:i32=0;static mut LAST_MOVE:i64=-1000000;
 let now=wphw_input_now_ms();let(mut x,mut y)=(0,0);
 if wphw_input_cursor(&mut x,&mut y){if x!=LAST_X||y!=LAST_Y{LAST_X=x;LAST_Y=y;LAST_MOVE=now;}}
 if (now-LAST_MOVE<1000||!pad.pointer_valid)&&keys.pointer_valid{
  // Keep Bevy's existing source image-relative mouse coordinates.
 }else if pad.pointer_valid{keys.pointer_valid=true;keys.pointer_x=pad.pointer_x;keys.pointer_y=pad.pointer_y;}
 else{keys.pointer_valid=false;}
 keys
}
