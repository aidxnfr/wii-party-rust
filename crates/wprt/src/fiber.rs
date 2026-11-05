// SPDX-License-Identifier: GPL-3.0-or-later
// port of engine/src/core/fiber.cpp (_WIN32 branch).
use std::{cell::Cell,ffi::c_void};
use windows_sys::Win32::System::Threading::{ConvertThreadToFiber,CreateFiber,SwitchToFiber,DeleteFiber};
pub type Handle=*mut c_void;
pub type Entry=unsafe fn(*mut c_void);
#[derive(Clone,Copy)]struct Start{entry:Entry,parameter:*mut c_void}
thread_local!{static CURRENT:Cell<Handle>=const{Cell::new(std::ptr::null_mut())};}
// port of engine/src/core/fiber.cpp run: copy Start, delete it, then invoke its entry.
unsafe extern "system" fn run(data:*mut c_void){let start=*Box::from_raw(data.cast::<Start>());(start.entry)(start.parameter);}
// port of engine/src/core/fiber.cpp adopt_current_thread.
pub fn adopt_current_thread()->Handle{let h=unsafe{ConvertThreadToFiber(std::ptr::null())};CURRENT.set(h);h}
// port of engine/src/core/fiber.cpp current.
pub fn current()->Handle{CURRENT.get()}
// port of engine/src/core/fiber.cpp create.
pub fn create(stack_size:usize,entry:Entry,parameter:*mut c_void)->Handle{unsafe{CreateFiber(stack_size,Some(run),Box::into_raw(Box::new(Start{entry,parameter})).cast())}}
// port of engine/src/core/fiber.cpp switch_to.
pub fn switch_to(target:Handle){CURRENT.set(target);unsafe{SwitchToFiber(target)}}
// port of engine/src/core/fiber.cpp destroy.
pub fn destroy(fiber:Handle){unsafe{DeleteFiber(fiber)}}
