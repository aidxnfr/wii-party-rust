// SPDX-License-Identifier: GPL-3.0-or-later
// H8 Bevy-owned key events with source-compatible fallback, not an all-keys/pure-Bevy claim.
// Sources: Bevy0.20 RawWinitWindowEvent; winit0.30.13 PhysicalKeyExtScancode;
// source video.cpp side_key/MapVirtualKeyA; windows-sys0.61.2 VK_* for mouse buttons.
use bevy::{prelude::*,winit::RawWinitWindowEvent};
use std::sync::atomic::{AtomicU8,Ordering};
use winit::{event::{WindowEvent,ElementState,MouseButton},keyboard::{PhysicalKey,KeyCode},platform::scancode::PhysicalKeyExtScancode};
type Query=unsafe extern "C" fn(i32,*mut bool)->bool;
type Bind=unsafe extern "C" fn(Option<Query>)->bool;
type Scan=unsafe extern "C" fn(u32)->i32;
#[derive(Resource)]pub struct Translation(Scan);
// One atomic holds BOTH validity and state; guest workers cannot observe a half-published key.
static KEYS:[AtomicU8;256]=[const{AtomicU8::new(0)};256];
unsafe extern "C" fn query(code:i32,out:*mut bool)->bool{
 if !(1..256).contains(&code)||out.is_null(){return false;}
 let value=KEYS[code as usize].load(Ordering::Acquire);
 // VK_RETURN is shared by Enter and keypad Enter (winit Windows keyboard.rs888/1029).
 // A cached release must not suppress a still-held aliased physical key: delegate to source SDK.
 if value&3!=3{return false;}
 unsafe{*out=true;}true
}
pub unsafe fn bind(lib:&libloading::Library)->Result<Option<Translation>,String>{
 let binder=unsafe{lib.get::<Bind>(b"wpg_keyboard_events_bind\0")};
 let scan=unsafe{lib.get::<Scan>(b"wpg_keyboard_scan_code\0")};
 match(binder,scan){
  (Err(_),Err(_))=>Ok(None),
  (Ok(bind),Ok(scan))=>{if !unsafe{bind(Some(query))}{return Err("keyboard events already bound".into());}Ok(Some(Translation(*scan)))},
  _=>Err("incomplete keyboard event transport".into())
 }
}
fn record(code:i32,down:bool){if (1..256).contains(&code){KEYS[code as usize].store(2|u8::from(down),Ordering::Release);}}
// Only unambiguous standard keys use scan translation. Keypad/modifiers/Pause/PrintScreen,
// unknown/layout-specific keys keep the ORIGINAL SDK provider, not an invented inverse map.
fn event_key(key:PhysicalKey)->bool{
 matches!(key,PhysicalKey::Code(
  KeyCode::KeyA|KeyCode::KeyB|KeyCode::KeyC|KeyCode::KeyD|KeyCode::KeyE|KeyCode::KeyF|
  KeyCode::KeyG|KeyCode::KeyH|KeyCode::KeyI|KeyCode::KeyJ|KeyCode::KeyK|KeyCode::KeyL|
  KeyCode::KeyM|KeyCode::KeyN|KeyCode::KeyO|KeyCode::KeyP|KeyCode::KeyQ|KeyCode::KeyR|
  KeyCode::KeyS|KeyCode::KeyT|KeyCode::KeyU|KeyCode::KeyV|KeyCode::KeyW|KeyCode::KeyX|KeyCode::KeyY|KeyCode::KeyZ|
  KeyCode::Digit0|KeyCode::Digit1|KeyCode::Digit2|KeyCode::Digit3|KeyCode::Digit4|
  KeyCode::Digit5|KeyCode::Digit6|KeyCode::Digit7|KeyCode::Digit8|KeyCode::Digit9|
  KeyCode::ArrowLeft|KeyCode::ArrowRight|KeyCode::ArrowUp|KeyCode::ArrowDown|
  KeyCode::Enter|KeyCode::Backspace|KeyCode::Tab|KeyCode::Space|KeyCode::Escape|
  KeyCode::Insert|KeyCode::Delete|KeyCode::Home|KeyCode::End|KeyCode::PageUp|KeyCode::PageDown|
  KeyCode::F1|KeyCode::F2|KeyCode::F3|KeyCode::F4|KeyCode::F5|KeyCode::F6|
  KeyCode::F7|KeyCode::F8|KeyCode::F9|KeyCode::F10|KeyCode::F11|KeyCode::F12))
}
pub fn update(mut events:MessageReader<RawWinitWindowEvent>,translation:Option<Res<Translation>>){
 let Some(translation)=translation else{return;};
 for raw in events.read(){match &raw.event{
  // On either focus edge, unknown state delegates to source GetAsyncKeyState. This preserves
  // keys held BEFORE focus returns, rather than inventing releases/held-key synchronization.
  WindowEvent::Focused(_)=>{for key in &KEYS{key.store(0,Ordering::Release);}},
  WindowEvent::KeyboardInput{event,..} if event_key(event.physical_key)=>{
   if let Some(scan)=event.physical_key.to_scancode(){record(unsafe{(translation.0)(scan)},event.state==ElementState::Pressed);}
  },
  // An unmodelled physical key can share a VK with a cached standard key (e.g. keypad Enter).
  // Invalidate cached positives too; delegate alias resolution to the unchanged source SDK.
  WindowEvent::KeyboardInput{..}=>{for key in &KEYS{key.store(0,Ordering::Release);}},
  WindowEvent::MouseInput{state,button,..}=>{
   let code=match button{MouseButton::Left=>1,MouseButton::Right=>2,MouseButton::Middle=>4,MouseButton::Back=>5,MouseButton::Forward=>6,_=>0};
   record(code,*state==ElementState::Pressed);
  },
  _=>{}
 }}
}
