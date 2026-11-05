// SPDX-License-Identifier: GPL-3.0-or-later
//! PRIVATE HOST COLLECTOR DRAFT. Source worker/sensor/pointer/rumble policy retained;
//! guest mapping/sideways rules intentionally remain in each owning game DLL.
#![allow(dead_code,non_snake_case)]
mod policy;mod sdk;mod services;
pub use policy::{RawSnapshot,raw_snapshot,wprt_gamepad_set_outputs};
static SDL:std::sync::OnceLock<libloading::Library>=std::sync::OnceLock::new();
// Caller provides actual source settings and pinned official SDL path, before polling.
// Source detached worker and loaded SDL are retained for the process lifetime.
pub unsafe fn configure(path:&std::path::Path,enabled:bool,joycon_pairs:bool,real_wiimotes:bool)->Result<(),String>{
 if SDL.get().is_none(){let library=libloading::Library::new(path).map_err(|e|e.to_string())?;
  SDL.set(library).map_err(|_|"collector already configured".to_string())?;}
 services::configure(enabled,joycon_pairs,real_wiimotes);Ok(())
}

// Enable/disable source snapshot guards without loading/opening SDL while disabled.
pub fn set_settings(enabled:bool,joycon_pairs:bool,real_wiimotes:bool){services::configure(enabled,joycon_pairs,real_wiimotes);}
