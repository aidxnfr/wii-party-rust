// SPDX-License-Identifier: GPL-3.0-or-later
// Port of engine/src/core/runtime.cpp (normal non-WP_TRACE runtime).
use crate::{cpu::Cpu,memory::*,runtime};
use std::{ffi::c_void,sync::{OnceLock,RwLock},time::Instant};
// MinGW time.h's default 64-bit aliases, as used by pristine runtime.cpp.
#[cfg(windows)]
extern "C" {
    #[link_name="_localtime64"] fn native_localtime(time:*const libc::time_t)->*mut libc::tm;
    #[link_name="_gmtime64"] fn native_gmtime(time:*const libc::time_t)->*mut libc::tm;
    #[link_name="_mktime64"] fn native_mktime(time:*mut libc::tm)->libc::time_t;
    #[link_name="_difftime64"] fn native_difftime(first:libc::time_t,second:libc::time_t)->f64;
}
#[cfg(unix)]
use libc::{localtime as native_localtime,gmtime as native_gmtime,mktime as native_mktime,difftime as native_difftime};
const TIME_BASE_HZ:u64=60750000;
static NAMES:OnceLock<RwLock<Vec<(u32,&'static str)>>>=OnceLock::new();
// Binding of pristine g_name_table; preserve the supplied source order.
pub fn set_names(names:&'static[(u32,&'static str)]){*NAMES.get_or_init(Default::default).write().unwrap()=names.to_vec();}
// port of engine/src/core/runtime.cpp find_name.
pub fn find_name(address:u32)->&'static str{let t=NAMES.get_or_init(Default::default).read().unwrap();let i=t.partition_point(|v|v.0<address);t.get(i).filter(|v|v.0==address).map(|v|v.1).unwrap_or("")}
// port of engine/src/core/threads.cpp symbol_address (same source name-table order).
pub fn symbol_address(name:&str)->u32{for &(a,n)in NAMES.get_or_init(Default::default).read().unwrap().iter(){if n==name{return a;}}eprintln!("unknown symbol {name}");std::process::abort();}
// port of engine/src/core/runtime.cpp call.
pub fn call(c:&mut Cpu,address:u32){#[cfg(feature="host")] if crate::host::call(c,address){return;}if let Some(f)=runtime::lookup(address){f(c);}else if !crate::modules::call_module_function(c,address){missing_function(c,address);}}
struct HostFunction{start:usize,guest:u32,module:Option<&'static str>}
// port of engine/src/core/runtime.cpp host_functions.
fn host_functions()->Vec<HostFunction>{let mut out=Vec::new();for(a,f)in runtime::registered_functions(){out.push(HostFunction{start:f as usize,guest:a,module:None});}for d in crate::modules::descriptors(){for &(s,o,f)in d.functions{out.push(HostFunction{start:f as usize,guest:(s<<24)|o,module:Some(d.name)});}}out.sort_unstable_by_key(|f|f.start);out}
// port of engine/src/core/runtime.cpp capture_host_frames.
fn capture_host_frames(frames:&mut[*mut c_void;64])->usize{
    #[cfg(windows)]{unsafe{windows_sys::Win32::System::Diagnostics::Debug::RtlCaptureStackBackTrace(0,64,frames.as_mut_ptr(),std::ptr::null_mut())as usize}}
    #[cfg(unix)]{unsafe{libc::backtrace(frames.as_mut_ptr(),64)as usize}}
}
// port of engine/src/core/runtime.cpp print_call_stack, non-WP_TRACE branch.
pub fn print_call_stack(){let mut frames=[std::ptr::null_mut();64];let count=capture_host_frames(&mut frames);static FUNCTIONS:OnceLock<Vec<HostFunction>>=OnceLock::new();let functions=FUNCTIONS.get_or_init(host_functions);eprint!("call stack from the host stack (innermost first):");let mut shown=0;for frame in frames.iter().take(count){if shown>=24{break;}let address=*frame as usize;let next=functions.partition_point(|f|f.start<=address);if next==0||address-functions[next-1].start>0x100000{continue;}let f=&functions[next-1];if let Some(m)=f.module{eprint!(" {m}:{:08x}",f.guest);}else{let name=find_name(f.guest);eprint!(" {:08x}{}{}{}",f.guest,if name.is_empty(){""}else{"("},name,if name.is_empty(){""}else{")"});}shown+=1;}eprintln!();}
// port of engine/src/core/runtime.cpp dump_memory; use the same C stdio/environment service.
fn dump_memory(){unsafe{let path=libc::getenv(c"WP_DUMP".as_ptr());if path.is_null(){return;}let file=libc::fopen(path,c"wb".as_ptr());if !file.is_null(){libc::fwrite(base_ptr().cast(),1,MEMORY_SIZE,file);libc::fclose(file);}}}
// port of engine/src/core/runtime.cpp fail/fatal_error.
pub fn fatal_error(message:&str)->!{fatal_error_bytes(message.as_bytes())}
// port of engine/src/core/runtime.cpp fail("%s", message); retain non-UTF-8 guest panic bytes.
pub fn fatal_error_bytes(message:&[u8])->!{use std::io::Write;let end=message.iter().position(|&b|b==0).unwrap_or(message.len());let _=std::io::stderr().write_all(&message[..end]);let _=std::io::stderr().write_all(b"\n");print_call_stack();dump_memory();std::process::abort();}
// port of engine/src/core/runtime.cpp missing_function.
pub fn missing_function(c:&mut Cpu,address:u32)->!{crate::modules::describe_loaded_modules(address);eprintln!("registers at the call: lr={:08x} r3={:08x} r4={:08x} r5={:08x} r12={:08x} r30={:08x} r31={:08x}",c.lr,c.r[3],c.r[4],c.r[5],c.r[12],c.r[30],c.r[31]);let object=c.r[28];let table=rd32(object);eprint!("r28={object:08x} vtable={table:08x} slots:");for i in 0..12{eprint!(" {:08x}",rd32(table.wrapping_add(i*4)));}eprintln!();fatal_error(&format!("call to unknown function {address:08x}"));}
// port of engine/src/core/runtime.cpp illegal_instruction.
pub fn illegal_instruction(_: &mut Cpu,address:u32,word:u32)->!{fatal_error(&format!("illegal instruction at {address:08x}: {word:08x}"))}
// port of engine/src/core/runtime.cpp unsupported_instruction.
pub fn unsupported_instruction(_: &mut Cpu,address:u32,name:&str)->!{fatal_error(&format!("unsupported instruction at {address:08x}: {name}"))}
// port of engine/src/core/runtime.cpp unresolved_jump.
pub fn unresolved_jump(_: &mut Cpu,target:u32)->!{fatal_error(&format!("unresolved jump to {target:08x}"))}
// port of engine/src/core/runtime.cpp trap.
pub fn trap(_: &mut Cpu,address:u32){fatal_error(&format!("trap at {address:08x}"));}
// port of engine/src/core/runtime.cpp system_call.
pub fn system_call(_: &mut Cpu){}
// port of engine/src/core/runtime.cpp locked_cache_dma.
pub fn locked_cache_dma(c:&mut Cpu){let dmal=c.spr[923];if dmal&2==0{return;}let dmau=c.spr[922];let mut blocks=((dmau&0x1f)<<2)|((dmal>>2)&3);if blocks==0{blocks=128;}let memory=(dmau&0xffffffe0)&ADDRESS_MASK;let cache=dmal&0xffffffe0;let bytes=blocks*32;let cache_offset=cache&(LOCKED_CACHE_SIZE-1);if cache_offset+bytes>LOCKED_CACHE_SIZE||memory+bytes>PHYSICAL_SIZE{fatal_error(&format!("locked cache DMA out of range: memory={memory:08x} cache={cache:08x} blocks={blocks}"));}unsafe{if dmal&0x10!=0{std::ptr::copy_nonoverlapping(host(memory),host(cache),bytes as usize);}else{std::ptr::copy_nonoverlapping(host(cache),host(memory),bytes as usize);}}c.spr[923]&=!3;}
static mut DECREMENTER_ARMED:bool=false;
static mut DECREMENTER_DEADLINE:u64=0;
// port of engine/src/core/runtime.cpp set_decrementer.
pub fn set_decrementer(c:&mut Cpu,value:u32){c.spr[22]=value;unsafe{DECREMENTER_DEADLINE=time_base().wrapping_add(if value&0x80000000!=0{0}else{u64::from(value)});DECREMENTER_ARMED=true;}}
// port of engine/src/core/runtime.cpp get_decrementer.
pub fn get_decrementer(_: &mut Cpu)->u32{unsafe{if !DECREMENTER_ARMED{return 0x7fffffff;}(DECREMENTER_DEADLINE as i64).wrapping_sub(time_base()as i64)as u32}}
// port of engine/src/core/runtime.cpp decrementer_due.
pub fn decrementer_due()->bool{unsafe{DECREMENTER_ARMED&&time_base()>=DECREMENTER_DEADLINE}}
// port of engine/src/core/runtime.cpp decrementer_fired.
pub fn decrementer_fired(){unsafe{DECREMENTER_ARMED=false;}}
// port of engine/src/core/runtime.cpp wall_clock_ticks; native timezone/DST conversion order retained.
pub fn wall_clock_ticks()->u64{
#[cfg(feature="comparison")] if unsafe{crate::ffi::wphw_comparison_enabled()}{return 0;}
unsafe{let now=libc::time(std::ptr::null_mut());let mut local=*native_localtime(&now);let mut utc=*native_gmtime(&now);utc.tm_isdst=local.tm_isdst;let offset=native_difftime(native_mktime(&mut local),native_mktime(&mut utc))as i64;((now as i64).wrapping_sub(946684800).wrapping_add(offset)as u64).wrapping_mul(TIME_BASE_HZ)}}
// port of engine/src/core/runtime.cpp time_base (origin is initialized before wall-clock start).
pub fn time_base()->u64{
#[cfg(feature="comparison")] if unsafe{crate::ffi::wphw_comparison_enabled()}{return unsafe{crate::ffi::wphw_comparison_ticks()};}
static ORIGIN:OnceLock<Instant>=OnceLock::new();static START:OnceLock<u64>=OnceLock::new();let origin=ORIGIN.get_or_init(Instant::now);let start=*START.get_or_init(wall_clock_ticks);let elapsed=origin.elapsed().as_nanos()as u64;start.wrapping_add(elapsed/1000000000*TIME_BASE_HZ).wrapping_add(elapsed%1000000000*TIME_BASE_HZ/1000000000)}
