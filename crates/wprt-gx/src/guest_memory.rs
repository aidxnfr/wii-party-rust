// SPDX-License-Identifier: GPL-3.0-or-later
// Host transport boundary only: retain source address semantics through the owning game DLL.
use std::cell::Cell;
// src: engine/include/wp/memory.h (same constants as runtime memory.rs).
pub const ADDRESS_MASK:u32=0x1fff_ffff;
pub const PHYSICAL_SIZE:u32=0x1400_0000;
#[repr(C)]#[derive(Clone,Copy)]pub struct MemoryCallbacks {
 pub host:unsafe extern "C" fn(u32)->*mut u8,
 pub range:extern "C" fn(u32,usize)->bool,
}
thread_local!{static MEMORY:Cell<Option<MemoryCallbacks>>=const{Cell::new(None)};}
// Scoped to one renderer call; no process-global RAM pointer or cross-DLL Rust object.
pub struct Scope(Option<MemoryCallbacks>);
impl Scope{pub fn enter(memory:MemoryCallbacks)->Self{Self(MEMORY.with(|slot|slot.replace(Some(memory))))}}
impl Drop for Scope{fn drop(&mut self){MEMORY.with(|slot|slot.set(self.0));}}
#[inline]pub fn host(address:u32)->*mut u8{match MEMORY.with(Cell::get){Some(m)=>unsafe{(m.host)(address)},None=>local_host(address)}}
#[inline]pub fn range_valid(address:u32,size:usize)->bool{match MEMORY.with(Cell::get){Some(m)=>(m.range)(address,size),None=>(address&ADDRESS_MASK)as usize+size<=PHYSICAL_SIZE as usize}}

#[cfg(feature="guest-runtime")]
fn local_host(address:u32)->*mut u8{wprt::memory::host(address)}
#[cfg(not(feature="guest-runtime"))]
fn local_host(_address:u32)->*mut u8{panic!("host renderer requires owning guest memory scope")}
