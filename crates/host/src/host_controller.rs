// SPDX-License-Identifier: GPL-3.0-or-later
// PRIVATE G32 host wiring candidate: NOT INTEGRATED/COMPILED/ACCEPTED.
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},ffi::c_void,path::Path};
use wprt_host_controller::RawSnapshot;
type Settings=unsafe extern "C" fn(*mut bool,*mut bool,*mut bool)->bool;
#[repr(C)]struct Collector{
 context:usize,
 snapshot:unsafe extern "C" fn(usize,u32,*mut RawSnapshot),
 outputs:unsafe extern "C" fn(usize,u32,bool,u8),
}
pub struct DeviceContext{_library:Arc<libloading::Library>,settings:Settings,enabled:AtomicBool}
unsafe extern "C" fn snapshot(token:usize,slot:u32,out:*mut RawSnapshot){
 let ctx=&*(token as *const DeviceContext);
 *out=if ctx.enabled.load(Ordering::Acquire){wprt_host_controller::raw_snapshot(slot)}else{RawSnapshot::default()};
}
unsafe extern "C" fn outputs(token:usize,slot:u32,rumble:bool,leds:u8){
 let ctx=&*(token as *const DeviceContext);
 if ctx.enabled.load(Ordering::Acquire){wprt_host_controller::wprt_gamepad_set_outputs(slot,rumble,leds);}
}
// Context retained by both Guest resource AND its worker,like renderer/audio ownership.
pub unsafe fn bind(library:Arc<libloading::Library>)->Result<Option<Arc<DeviceContext>>,String>{
 type Bind=unsafe extern "C" fn(*const Collector);
 let bind=library.get::<Bind>(b"wpg_controller_bind\0").ok().map(|f|*f);
 let settings=library.get::<Settings>(b"wpg_controller_settings\0").ok().map(|f|*f);
 match(bind,settings){(None,None)=>Ok(None),(Some(bind),Some(settings))=>{
  let ctx=Arc::new(DeviceContext{_library:library,settings,enabled:AtomicBool::new(false)});
  bind(&Collector{context:Arc::as_ptr(&ctx)as usize,snapshot,outputs});Ok(Some(ctx))
 },_=>Err("Incomplete source controller transport exports".into())}
}
// Each guest supplies actual source settings only after its settings_load handshake.
// Physical SDL device pairing is host-wide: reject conflicting source configurations;
// do not choose a vendor/name guess or silently overwrite one guest's policy.
pub unsafe fn configure(contexts:&[Arc<DeviceContext>],sdl:Option<&Path>)->Result<(),String>{
 if contexts.is_empty(){return Ok(());}
 let mut config=Vec::new();
 for ctx in contexts{let(mut enabled,mut pairs,mut wiimotes)=(false,false,false);
  if !(ctx.settings)(&mut enabled,&mut pairs,&mut wiimotes){return Ok(());}
  config.push((enabled,pairs,wiimotes));
 }
 let any=config.iter().any(|v|v.0);
 let chosen=config.iter().find(|v|v.0).unwrap_or(&config[0]);
 if config.iter().filter(|v|v.0).any(|v|v.1!=chosen.1||v.2!=chosen.2){return Err("TODO(source): active guests have conflicting physical SDL pairing settings".into());}
 if any{let sdl=sdl.ok_or("TODO(source): WII_PARTY_SDL3 must name the pinned official project SDL3.dll")?;
  wprt_host_controller::configure(sdl,true,chosen.1,chosen.2)?;
 }else{wprt_host_controller::set_settings(false,chosen.1,chosen.2);}
 for(ctx,value)in contexts.iter().zip(config.iter()){ctx.enabled.store(value.0,Ordering::Release);}
 Ok(())
}
