// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted Bevy adapter of pristine platform/video.cpp window_thread and F11 branch.
// Source settings/order/repeat rule retained; Bevy/winit owns monitor and placement restoration.
use bevy::{prelude::*,input::{keyboard::KeyboardInput,ButtonState},window::{PrimaryWindow,MonitorSelection,WindowMode}};
type FullscreenQuery=unsafe extern "C" fn(*mut bool)->bool;
type Store=unsafe extern "C" fn(bool);
pub struct Controls{query:FullscreenQuery,store:Store}
pub unsafe fn bind(library:&libloading::Library)->Result<Option<Controls>,String>{
 match(unsafe{library.get::<FullscreenQuery>(b"wpg_window_fullscreen\0")},unsafe{library.get::<Store>(b"wpg_window_store_fullscreen\0")}){
  (Ok(query),Ok(store))=>Ok(Some(Controls{query:*query,store:*store})),
  (Err(_),Err(_))=>Ok(None),_=>Err("Incomplete source window settings interface".into())
 }
}
pub(super) fn mode(enabled:bool)->WindowMode{if enabled{WindowMode::BorderlessFullscreen(MonitorSelection::Current)}else{WindowMode::Windowed}}
// Each callback remains owned by its Guest's retained Library. Never use it after unloading.
pub fn update(guests:Res<crate::Guests>,mut windows:Query<(Entity,&mut Window),With<PrimaryWindow>>,mut keys:MessageReader<KeyboardInput>,mut applied:Local<bool>,mut reported:Local<Option<&'static str>>){
 let Ok((entity,mut window))=windows.single_mut()else{return;};
 if !*applied{
  let mut values=Vec::new();
  for guest in &guests.items{
   let Some(controls)=&guest.window_controls else{keys.clear();return;};
   let mut value=false;if !unsafe{(controls.query)(&mut value)}{keys.clear();return;}
   values.push(value);
  }
  if values.iter().any(|value|*value!=values[0]){
   let message="TODO(source): guests disagree on one host window's startup fullscreen setting";
   if *reported!=Some(message){eprintln!("{message}");*reported=Some(message);}
   keys.clear();return;
  }
  // port of video.cpp window_thread: read actual source flag once after video startup.
  window.mode=mode(values[0]);*applied=true;
 }
 for event in keys.read(){
  // port of window_proc WM_KEYDOWN/F11: key-up and OS repeats do not toggle.
  if event.window!=entity||event.key_code!=KeyCode::F11||event.state!=ButtonState::Pressed||event.repeat{continue;}
  if guests.items.len()!=1{
   let message="TODO(source): choose fullscreen persistence ownership before F11 can change a multi-guest window";
   if *reported!=Some(message){eprintln!("{message}");*reported=Some(message);}
   continue;
  }
  let enabled=matches!(window.mode,WindowMode::Windowed);
  window.mode=mode(enabled);
  // Source set_fullscreen precedes settings::store. OS failure/async acknowledgement is OPEN.
  if let Some(controls)=&guests.items[0].window_controls{unsafe{(controls.store)(enabled);}}
  // Source refresh_menu has no effect here: host options menu has not yet been bound.
 }
}
