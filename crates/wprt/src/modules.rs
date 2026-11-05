// SPDX-License-Identifier: GPL-3.0-or-later
// REL module section bases (C++ g_<name>_bases + wp::external_address). Filled by the module loader (E2).
use std::sync::atomic::{AtomicU32, Ordering};
pub const MAX_MODULES: usize = 128;
pub const MAX_SECTIONS: usize = 32;
static BASES: [[AtomicU32; MAX_SECTIONS]; MAX_MODULES] = [const { [const { AtomicU32::new(0) }; MAX_SECTIONS] }; MAX_MODULES];
pub fn set_base(module: u32, section: u32, base: u32) { BASES[module as usize][section as usize].store(base, Ordering::Relaxed) }
#[inline(always)] pub fn base(module: u32, section: u32) -> u32 { BASES[module as usize][section as usize].load(Ordering::Relaxed) }
// port of engine/src/core/modules.cpp external_address; the guest list is authoritative.
pub fn external_address(module:u32,section:u32,offset:u32)->u32 {
    let mut info=crate::memory::rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        if crate::memory::rd32(info)==module {
            return (crate::memory::rd32(section_table(info).wrapping_add(section.wrapping_mul(8)))&!1).wrapping_add(offset);
        }
        info=crate::memory::rd32(info.wrapping_add(4));
    }
    0
}

use crate::{cpu::Cpu,memory::rd32,runtime::GuestFn};
const HEAD:u32=0x800030c8;
pub struct ModuleDescriptor {
    pub name:&'static str,
    pub identifier:u32,
    pub signature:u32,
    pub section_count:u32,
    pub functions:&'static [(u32,u32,GuestFn)],
    pub resumes:&'static [(u32,u32,GuestFn)],
}
#[derive(Clone,Copy)]
struct LoadedModule { info:u32, identifier:u32, descriptor:Option<usize> }
static mut TABLE:&[ModuleDescriptor]=&[];
static mut LOADED:Vec<LoadedModule>=Vec::new();
// Binding of engine/include/wp/modules.h g_module_table/g_module_count, installed before guest execution.
pub fn set_descriptors(table:&'static [ModuleDescriptor]) { unsafe { TABLE=table; } }

// port of engine/src/core/modules.cpp mix (least-significant byte first).
pub fn mix(mut hash:u32,value:u32)->u32 {
    for shift in (0..32).step_by(8) { hash=(hash^((value>>shift)&0xff)).wrapping_mul(16777619); }
    hash
}
// port of engine/src/core/modules.cpp section_table.
fn section_table(info:u32)->u32 {
    let table=rd32(info.wrapping_add(0x10));
    if table<0x80000000 { info.wrapping_add(table) } else { table }
}
// port of engine/src/core/modules.cpp signature_of.
pub fn signature_of(info:u32)->u32 {
    let count=rd32(info.wrapping_add(0xc));
    let table=section_table(info);
    let mut hash=mix(2166136261,rd32(info));
    hash=mix(hash,count);
    for i in 0..count { hash=mix(hash,rd32(table.wrapping_add(i.wrapping_mul(8)).wrapping_add(4))); }
    mix(hash,rd32(info.wrapping_add(0x20)))
}
// port of engine/src/core/modules.cpp find_descriptor (first matching signature).
fn find_descriptor(signature:u32)->Option<usize> {
    unsafe { let table=TABLE; table.iter().position(|d|d.signature==signature) }
}
// port of engine/src/core/modules.cpp update_bases.
fn update_bases(d:&ModuleDescriptor,info:u32) {
    let table=section_table(info);
    for i in 0..d.section_count { set_base(d.identifier,i,rd32(table.wrapping_add(i.wrapping_mul(8)))&!1); }
}
// port of engine/src/core/modules.cpp descriptor_at. Keep the native identifier-only invalidation quirk.
fn descriptor_at(info:u32)->Option<&'static ModuleDescriptor> { unsafe {
    let identifier=rd32(info);
    let loaded=&mut *std::ptr::addr_of_mut!(LOADED);
    let index=if let Some(i)=loaded.iter().position(|v|v.info==info) {
        if loaded[i].identifier!=identifier || loaded[i].descriptor.is_none() {
            loaded[i].identifier=identifier;
            loaded[i].descriptor=find_descriptor(signature_of(info));
        }
        loaded[i].descriptor
    } else {
        let descriptor=find_descriptor(signature_of(info));
        loaded.push(LoadedModule{info,identifier,descriptor});
        descriptor
    };
    index.map(|i|&TABLE[i])
} }
// port of engine/src/core/modules.cpp find_function (lower_bound of section/offset).
fn find_function(d:&ModuleDescriptor,section:u32,offset:u32)->Option<GuestFn> {
    // Cache ONLY immutable static function-table lookup, never guest REL membership/bases/header data.
    #[cfg(all(feature="lookup-cache",not(feature="trace")))]
    thread_local! { static LAST:std::cell::Cell<Option<(usize,usize,u32,u32,Option<GuestFn>)>> = const { std::cell::Cell::new(None) }; }
    #[cfg(all(feature="lookup-cache",not(feature="trace")))]
    let identity=(d.functions.as_ptr() as usize,d.functions.len());
    #[cfg(all(feature="lookup-cache",not(feature="trace")))]
    if let Some((table,len,s,o,f))=LAST.with(std::cell::Cell::get) {
        if (table,len)==identity && s==section && o==offset { return f; }
    }
    let i=d.functions.partition_point(|f|(f.0,f.1)<(section,offset));
    let result=d.functions.get(i).filter(|f|f.0==section&&f.1==offset).map(|f|f.2);
    #[cfg(all(feature="lookup-cache",not(feature="trace")))] LAST.with(|v|v.set(Some((identity.0,identity.1,section,offset,result))));
    result
}
// port of engine/src/core/modules.cpp call_module_function.
pub fn call_module_function(c:&mut Cpu,address:u32)->bool {
    let mut info=rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        if let Some(d)=descriptor_at(info) {
            update_bases(d,info);
            for section in 1..d.section_count {
                let size=rd32(section_table(info).wrapping_add(section.wrapping_mul(8)).wrapping_add(4));
                let b=base(d.identifier,section);
                if b!=0&&address>=b&&address<b.wrapping_add(size) {
                    if let Some(f)=find_function(d,section,address-b) { f(c); return true; }
                }
            }
        }
        info=rd32(info.wrapping_add(4));
    }
    false
}
// port of engine/src/core/modules.cpp find_module_resume; multiple loaded matches are rejected.
pub fn find_module_resume(address:u32)->Option<GuestFn> {
    let section=address>>24;
    let offset=address&0xffffff;
    let (mut found,mut matches)=(None,0);
    let mut info=rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        if let Some(d)=descriptor_at(info) {
            update_bases(d,info);
            for &(s,o,f) in d.resumes { if s==section&&o==offset { found=Some(f); matches+=1; break; } }
        }
        info=rd32(info.wrapping_add(4));
    }
    if matches>1 { eprintln!("resume address {address:08x} matches {matches} loaded modules"); None } else { found }
}
// port of engine/src/core/modules.cpp loaded_module_names (guest list order).
pub fn loaded_module_names()->Vec<String> {
    let mut names=Vec::new();
    let mut info=rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        let identifier=rd32(info);
        unsafe { for d in TABLE { if d.identifier==identifier { names.push(d.name.to_owned()); break; } } }
        info=rd32(info.wrapping_add(4));
    }
    names
}
// port of engine/src/core/modules.cpp module_loaded.
pub fn module_loaded(name:&str)->bool {
    let mut info=rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        let identifier=rd32(info);
        unsafe { for d in TABLE { if d.identifier==identifier&&d.name==name { return true; } } }
        info=rd32(info.wrapping_add(4));
    }
    false
}
// port of engine/src/core/modules.cpp module_name_at; native list-count ambiguity retained.
pub fn module_name_at(address:u32)->&'static str {
    if address>=0x80000000 { return "dol"; }
    let mut found=None;
    let mut info=rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        let mut name="?";
        unsafe { for d in TABLE { if d.identifier==rd32(info) { name=d.name; } } }
        if found.is_some() { return "?"; }
        found=Some(name);
        info=rd32(info.wrapping_add(4));
    }
    found.unwrap_or("?")
}

// port of engine/src/core/modules.cpp describe_loaded_modules.
pub fn describe_loaded_modules(address:u32) {
    eprintln!("address {address:08x}, loaded modules:");
    let mut info=rd32(HEAD);
    for _ in 0..64 {
        if info==0 { break; }
        let count=rd32(info.wrapping_add(0xc));
        let table=section_table(info);
        let mut name="?";
        unsafe { for d in TABLE { if d.identifier==rd32(info) { name=d.name; } } }
        eprint!("  module {name} id {} at {info:08x} signature {:08x}, {count} sections:",rd32(info),signature_of(info));
        for i in 0..count {
            let b=rd32(table.wrapping_add(i.wrapping_mul(8)))&!1;
            let size=rd32(table.wrapping_add(i.wrapping_mul(8)).wrapping_add(4));
            eprint!(" [{i}]{b:08x}+{size:x}");
            if address>=b&&address<b.wrapping_add(size) { eprint!("<-contains {address:08x} (section {i} offset {:x})",address-b); }
        }
        eprintln!();
        info=rd32(info.wrapping_add(4));
    }
}
// port of engine/src/core/modules.cpp log::watch_modules; same100ms poll gate and list order.
pub fn watch_modules() {
    if !crate::ffi::log_enabled() { return; }
    use std::time::{Duration,Instant};
    static mut NEXT:Option<Instant>=None;
    static mut PREVIOUS:String=String::new();
    unsafe {
        if (*std::ptr::addr_of!(NEXT)).is_none() { NEXT=Some(Instant::now()); }
        let now=Instant::now();
        if now<(*std::ptr::addr_of!(NEXT)).unwrap() { return; }
        NEXT=Some(now+Duration::from_millis(100));
        let mut current=String::new();
        let mut info=rd32(HEAD);
        for _ in 0..64 {
            if info==0 { break; }
            let mut name="unknown";
            for d in TABLE { if d.identifier==rd32(info) { name=d.name; } }
            if !current.is_empty() { current.push(' '); }
            current.push_str(name);
            info=rd32(info.wrapping_add(4));
        }
        if current!=*std::ptr::addr_of!(PREVIOUS) {
            crate::ffi::log("module",&format!("loaded modules: {}",if current.is_empty(){"(none)"}else{&current}));
            PREVIOUS=current;
        }
    }
}

// Binding of pristine g_module_table for runtime.cpp host_functions.
pub(crate) fn descriptors()->&'static[ModuleDescriptor]{unsafe{TABLE}}
