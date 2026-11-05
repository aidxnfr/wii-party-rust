// SPDX-License-Identifier: GPL-3.0-or-later
// H8 Bevy provider for video.cpp client_size/window_focused; no HWND or Rust object crosses DLLs.
// UNACCEPTED until the next host-options gate. Unbound native hosts keep the original SDK path.
use std::sync::Mutex;
#[derive(Clone,Copy)]pub struct Client{pub width:i32,pub height:i32,pub focused:bool}
static CLIENT:Mutex<Option<Client>>=Mutex::new(None);
pub fn update(width:i32,height:i32,focused:bool){*CLIENT.lock().unwrap()=Some(Client{width,height,focused});}
pub fn get()->Option<Client>{*CLIENT.lock().unwrap()}
