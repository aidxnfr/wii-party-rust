// SPDX-License-Identifier: GPL-3.0-or-later
// Port of engine/src/core/hle.cpp; guest printf formatting remains a pristine C++ leaf.
use crate::{cpu::Cpu,memory::rd8,runtime::{self,GuestFn},threads,dispatch,ffi};
// port of engine/src/core/hle.cpp read_guest_string.
fn read_guest_string(mut address:u32)->Vec<u8>{let mut text=Vec::new();loop{let ch=rd8(address)as u8;if ch==0||text.len()>=256{break;}text.push(ch);address=address.wrapping_add(1);}text}
// port of engine/src/core/hle.cpp do_nothing.
fn do_nothing(_: &mut Cpu){}
// port of engine/src/core/hle.cpp return_zero.
fn return_zero(c:&mut Cpu){c.r[3]=0;}
// port of engine/src/core/hle.cpp os_panic; preserve guest bytes rather than decoding as UTF-8.
fn os_panic(c:&mut Cpu){let mut message=b"PANIC in ".to_vec();message.extend(read_guest_string(c.r[3]));message.extend(format!(" on line {}: ",c.r[4]).bytes());message.extend(ffi::format_guest(c,c.r[5],6));dispatch::fatal_error_bytes(&message);}
// port of engine/src/core/hle.cpp switch_fiber.
fn switch_fiber(c:&mut Cpu){let function=c.r[3];c.r[1]=c.r[4];runtime::call(c,function);}
const REPLACEMENTS:[(&str,GuestFn);5]=[("OSRealModeCall",do_nothing),("OSPanic",os_panic),("OSLoadContext",threads::load_context),("OSSwitchFiber",switch_fiber),("longjmp",threads::long_jump)];
// port of engine/src/core/hle.cpp kReplacements binding to the existing runtime registry.
pub fn register(){for (name,function)in REPLACEMENTS{runtime::register_hle(name,function);}}
// port of engine/src/core/hle.cpp find_replacement.
pub fn find_replacement(name:&str)->GuestFn{if name.starts_with("HleZero_"){return return_zero;}for(n,function)in REPLACEMENTS{if name==n{return function;}}eprintln!("no runtime handler for replaced function {name}");std::process::abort();}
