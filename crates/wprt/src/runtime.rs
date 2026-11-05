// SPDX-License-Identifier: GPL-3.0-or-later
// Runtime entry points the translated code calls (dispatch, interrupts, thread resume, traps).
// H3 ports dispatch and threads; hardware interrupt polling remains pending.
use crate::cpu::Cpu;
use std::sync::atomic::{AtomicU32, Ordering};

pub type GuestFn = fn(&mut Cpu);
// Registry caches own only copied function pointers; never hold a lock/TLS borrow across a guest call.
// The live registry remains authoritative on every miss and after every registration mutation.
#[cfg(all(feature="lookup-cache",not(feature="trace")))]
static REGISTRY_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[inline]
fn registry_changed() {
    #[cfg(all(feature="lookup-cache",not(feature="trace")))] {
        // Saturation disables hits forever; version wrap cannot resurrect a stale entry.
        let _ = REGISTRY_EPOCH.fetch_update(Ordering::Release, Ordering::Relaxed, |v| Some(v.saturating_add(1)));
    }
}
#[cfg(all(feature="lookup-cache",not(feature="trace")))]
type AddressEntry = (u64, u32, Option<GuestFn>);
#[cfg(all(feature="lookup-cache",not(feature="trace")))]
thread_local! {
    static LAST_DISPATCH: std::cell::Cell<Option<AddressEntry>> = const { std::cell::Cell::new(None) };
    static LAST_RESUME: std::cell::Cell<Option<AddressEntry>> = const { std::cell::Cell::new(None) };
    static LAST_HLE: std::cell::Cell<Option<(u64, &'static str, GuestFn)>> = const { std::cell::Cell::new(None) };
}
#[cfg(all(feature="lookup-cache",not(feature="trace")))]
fn cached_address(registry: &std::sync::RwLock<Vec<(u32, GuestFn)>>, address: u32,
                  cache: &'static std::thread::LocalKey<std::cell::Cell<Option<AddressEntry>>>) -> Option<GuestFn> {
    let epoch = REGISTRY_EPOCH.load(Ordering::Acquire);
    if epoch != u64::MAX {
        if let Some((generation, key, value)) = cache.with(std::cell::Cell::get) {
            if generation == epoch && key == address { return value; }
        }
    }
    let (value, generation) = {
        let t = registry.read().unwrap();
        let i = t.partition_point(|v| v.0 < address);
        let value = t.get(i).filter(|v| v.0 == address).map(|v| v.1);
        // Read the version under the SAME read lock: a writer cannot retag an old value as new.
        (value, REGISTRY_EPOCH.load(Ordering::Acquire))
    };
    cache.with(|v| v.set(Some((generation, address, value))));
    value
}

static DISPATCH: std::sync::OnceLock<std::sync::RwLock<Vec<(u32, GuestFn)>>> = std::sync::OnceLock::new();
fn table() -> &'static std::sync::RwLock<Vec<(u32, GuestFn)>> { DISPATCH.get_or_init(Default::default) }
/// Registers a guest function at its (relocated) address.
// Binding of pristine g_function_table, kept sorted before guest execution.
pub fn register(address:u32,f:GuestFn){let mut t=table().write().unwrap();let i=t.partition_point(|v|v.0<address);if i<t.len()&&t[i].0==address{t[i].1=f;}else{t.insert(i,(address,f));}registry_changed();}
// Source runtime.cpp lower_bound lookup; never hold the registry lock through a guest call.
pub(crate) fn lookup(address:u32)->Option<GuestFn>{
    #[cfg(all(feature="lookup-cache",not(feature="trace")))] { cached_address(table(), address, &LAST_DISPATCH) }
    #[cfg(not(all(feature="lookup-cache",not(feature="trace"))))] { let t=table().read().unwrap();let i=t.partition_point(|v|v.0<address);t.get(i).filter(|v|v.0==address).map(|v|v.1) }
}
pub(crate) fn registered_functions()->Vec<(u32,GuestFn)>{table().read().unwrap().clone()}

/// Indirect call through a guest address (C++ wp::call).
pub fn call(c: &mut Cpu, address: u32) {
    #[cfg(feature="trace")] { // Existing fdiff-only engine-service stop boundary.
        match lookup(address){Some(f)=>f(c),None=>unresolved_jump(c,address)}
    }
    #[cfg(not(feature="trace"))] crate::dispatch::call(c,address);
}
pub fn unresolved_jump(c: &mut Cpu, target: u32) -> ! {
    #[cfg(feature="trace")] { let _=c;panic!("unresolved jump to {target:#010x}") }
    #[cfg(not(feature="trace"))] crate::dispatch::unresolved_jump(c,target)
}
pub fn bad_resume(c: &mut Cpu) -> ! {
    #[cfg(feature="trace")] { let _=c;panic!("bad resume address") }
    #[cfg(not(feature="trace"))] crate::threads::bad_resume(c)
}

pub static RESUME_ADDRESS: AtomicU32 = AtomicU32::new(0);
#[inline(always)] pub fn resume_pending() -> bool { RESUME_ADDRESS.load(Ordering::Relaxed) != 0 }
pub static TAKEN_RESUME_ADDRESS: AtomicU32 = AtomicU32::new(0);
/// C++ take_resume_address: also remembers the taken address (g_taken_resume_address).
#[inline(always)] pub fn take_resume_address() -> u32 { let a = RESUME_ADDRESS.swap(0, Ordering::Relaxed); TAKEN_RESUME_ADDRESS.store(a, Ordering::Relaxed); a }

pub static POLL_COUNTER: AtomicU32 = AtomicU32::new(1_000_000);
/// WP_POLL: counts down and polls interrupts at zero (interrupt delivery = E3).
#[inline(always)]
pub fn poll(c: &mut Cpu) {
    #[cfg(feature="host")] crate::host::service();
    #[cfg(feature="trace")] if POLL_COUNTER.fetch_sub(1, Ordering::Relaxed)==1{poll_interrupts(c);}
    #[cfg(all(not(feature="trace"), not(feature="host-poll-fastpath")))] unsafe{let counter=crate::ffi::wphw_poll_counter_ptr();*counter=(*counter).wrapping_sub(1);if *counter==0{poll_interrupts(c);}}
    #[cfg(all(not(feature="trace"), feature="host-poll-fastpath"))] unsafe {
        let counter=std::ptr::addr_of_mut!(crate::ffi::HOST_POLL_COUNTER);
        *counter=(*counter).wrapping_sub(1);
        if *counter==0 { poll_interrupts(c); }
    }
}
pub fn poll_interrupts(c: &mut Cpu) { (hooks().poll_interrupts)(c) }
#[cfg(feature="trace")] fn e2_poll(_c: &mut Cpu) { POLL_COUNTER.store(1_000_000, Ordering::Relaxed); }

// ---- setjmp replacement (threads.cpp save_context/save_jump + load_context/long_jump) ----
// C++: `if (setjmp(p) == 0) save_context(c, &p); else resume_context(c);` and later std::longjmp(p, 1) from
// load_context on the same fiber. Rust: save_* returns a fresh token kept in the frame; to resume, threads code
// calls `resume_to(token)`, which unwinds to that frame; the frame's catch_resume() matches the token and the
// generated code runs resume_* and continues at the R_ label after the site. Frames in between are plain
// (no destructors), exactly like longjmp skipping them.
pub struct Resume { pub token: u64 }
static NEXT_TOKEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
pub fn new_token() -> u64 { NEXT_TOKEN.fetch_add(1, Ordering::Relaxed) }
/// Unwinds to the frame that owns `token` (C++ std::longjmp(*point, 1)).
pub fn resume_to(token: u64) -> ! { std::panic::resume_unwind(Box::new(Resume { token })) }
/// Called by a frame with a setjmp site when an unwind reaches it. Returns where to continue, or keeps unwinding.
pub fn catch_resume(p: Box<dyn std::any::Any + Send>, own: u64, cont: u32) -> u32 {
    match p.downcast::<Resume>() {
        Ok(r) if own != 0 && r.token == own => cont,
        Ok(r) => std::panic::resume_unwind(r),
        Err(p) => std::panic::resume_unwind(p),
    }
}
pub struct ThreadHooks { pub save_context: fn(&mut Cpu) -> u64, pub resume_context: Hook, pub save_jump: fn(&mut Cpu) -> u64, pub resume_jump: Hook }
fn e3_save(_c: &mut Cpu) -> u64 { panic!("save_context: threads are ticket E3") }
fn e3_save_jump(_c: &mut Cpu) -> u64 { panic!("save_jump: threads are ticket E3") }
fn e3_resume(_c: &mut Cpu) { panic!("resume_context: threads are ticket E3") }
static mut THREAD_HOOKS: ThreadHooks = ThreadHooks { save_context: e3_save, resume_context: e3_resume, save_jump: e3_save_jump, resume_jump: e3_resume };
pub fn set_thread_hooks(h: ThreadHooks) { unsafe { THREAD_HOOKS = h; } }
#[inline(always)] fn thooks() -> &'static ThreadHooks { unsafe { &*std::ptr::addr_of!(THREAD_HOOKS) } }
pub fn save_context(c: &mut Cpu) -> u64 { (thooks().save_context)(c) }
pub fn resume_context(c: &mut Cpu) { (thooks().resume_context)(c) }
pub fn save_jump(c: &mut Cpu) -> u64 { (thooks().save_jump)(c) }
pub fn resume_jump(c: &mut Cpu) { (thooks().resume_jump)(c) }
pub fn external_address(module: u32, section: u32, offset: u32) -> u32 { crate::modules::external_address(module, section, offset) }

// Traps and system hooks (engine/src/core). Bodies are ported in E2/E3; E1 keeps the C++ behaviour where it is fixed.
pub fn illegal_instruction(c: &mut Cpu, address: u32, word: u32) -> ! {
    #[cfg(feature="trace")] { let _=c;panic!("illegal instruction {word:#010x} at {address:#010x}") }
    #[cfg(not(feature="trace"))] crate::dispatch::illegal_instruction(c,address,word)
}
pub fn unsupported_instruction(c: &mut Cpu, address: u32, name: &str) -> ! {
    #[cfg(feature="trace")] { let _=c;panic!("unsupported instruction {name} at {address:#010x}") }
    #[cfg(not(feature="trace"))] crate::dispatch::unsupported_instruction(c,address,name)
}
pub type Hook = fn(&mut Cpu);
pub type Hook32 = fn(&mut Cpu, u32);
pub struct Hooks { pub trap: Hook32, pub system_call: Hook, pub locked_cache_dma: Hook, pub set_decrementer: Hook32,
    pub get_decrementer: fn(&mut Cpu) -> u32, pub time_base: fn() -> u64, pub poll_interrupts: Hook }
#[cfg(feature="trace")] fn e2(_c: &mut Cpu) { panic!("system hook before the E2 runtime is installed") }
#[cfg(feature="trace")] fn e2_32(_c: &mut Cpu, _v: u32) { panic!("system hook before the E2 runtime is installed") }
#[cfg(feature="trace")] fn e2_get(_c: &mut Cpu) -> u32 { panic!("decrementer before the E2 runtime is installed") }
#[cfg(feature="trace")] fn e2_tb() -> u64 { panic!("time base before the E2 runtime is installed") }
#[cfg(feature="trace")]
static mut HOOKS: Hooks = Hooks { trap: e2_32, system_call: e2, locked_cache_dma: e2, set_decrementer: e2_32, get_decrementer: e2_get, time_base: e2_tb, poll_interrupts: e2_poll };
// port of engine/src/core/runtime.cpp service bindings; trace keeps its fdiff stop boundaries.
#[cfg(not(feature="trace"))]
static mut HOOKS: Hooks = Hooks { trap: crate::dispatch::trap, system_call: crate::dispatch::system_call,
    locked_cache_dma: crate::dispatch::locked_cache_dma, set_decrementer: crate::dispatch::set_decrementer,
    get_decrementer: crate::dispatch::get_decrementer, time_base: crate::dispatch::time_base, poll_interrupts: crate::interrupts::poll_interrupts };
pub fn set_hooks(h: Hooks) { unsafe { HOOKS = h; } }
#[inline(always)] fn hooks() -> &'static Hooks { unsafe { &*std::ptr::addr_of!(HOOKS) } }
pub fn trap(c: &mut Cpu, address: u32) { (hooks().trap)(c, address) }
pub fn system_call(c: &mut Cpu) { (hooks().system_call)(c) }
pub fn locked_cache_dma(c: &mut Cpu) { (hooks().locked_cache_dma)(c) }
pub fn set_decrementer(c: &mut Cpu, v: u32) { (hooks().set_decrementer)(c, v) }
pub fn get_decrementer(c: &mut Cpu) -> u32 { (hooks().get_decrementer)(c) }
pub fn time_base() -> u64 { (hooks().time_base)() }

/// HLE replacements (C++ wp::find_replacement(name)): the 5 OS functions the recomp implements natively.
static HLE: std::sync::OnceLock<std::sync::RwLock<std::collections::HashMap<&'static str, GuestFn>>> = std::sync::OnceLock::new();
fn hle_table() -> &'static std::sync::RwLock<std::collections::HashMap<&'static str, GuestFn>> { HLE.get_or_init(Default::default) }
pub fn register_hle(name: &'static str, f: GuestFn) { let mut t=hle_table().write().unwrap(); t.insert(name, f); registry_changed(); }
pub fn hle(c: &mut Cpu, name: &str) {
    #[cfg(all(feature="lookup-cache",not(feature="trace")))] {
        let epoch=REGISTRY_EPOCH.load(Ordering::Acquire);
        if epoch!=u64::MAX {
            if let Some((generation,key,f))=LAST_HLE.with(std::cell::Cell::get) {
                if generation==epoch && key==name { f(c); return; }
            }
        }
    }
    #[cfg(all(feature="lookup-cache",not(feature="trace")))]
    let f = {
        let (entry,generation) = {
            let t=hle_table().read().unwrap();
            (t.get_key_value(name).map(|(key,f)|(*key,*f)), REGISTRY_EPOCH.load(Ordering::Acquire))
        };
        entry.map(|(key,f)| { LAST_HLE.with(|v|v.set(Some((generation,key,f)))); f })
    };
    #[cfg(not(all(feature="lookup-cache",not(feature="trace"))))]
    let f = hle_table().read().unwrap().get(name).copied();
    match f { Some(f) => f(c), None => {
        #[cfg(feature="trace")] panic!("HLE replacement {name} is not installed (ticket E3)");
        #[cfg(not(feature="trace"))] (crate::hle::find_replacement(name))(c);
    } }
}

/// Resume table (C++ g_resume_table): guest address inside a function -> the function that contains it.
/// A thread switched out mid-function re-enters that function with RESUME_ADDRESS set (E3).
static RESUMES: std::sync::OnceLock<std::sync::RwLock<Vec<(u32, GuestFn)>>> = std::sync::OnceLock::new();
fn resume_table() -> &'static std::sync::RwLock<Vec<(u32, GuestFn)>> { RESUMES.get_or_init(Default::default) }
pub fn register_resume(address:u32,f:GuestFn){let mut t=resume_table().write().unwrap();let i=t.partition_point(|v|v.0<address);if i<t.len()&&t[i].0==address{t[i].1=f;}else{t.insert(i,(address,f));}registry_changed();}
pub fn resume_function(address:u32)->Option<GuestFn>{
    #[cfg(all(feature="lookup-cache",not(feature="trace")))] { cached_address(resume_table(), address, &LAST_RESUME) }
    #[cfg(not(all(feature="lookup-cache",not(feature="trace"))))] { let t=resume_table().read().unwrap();let i=t.partition_point(|v|v.0<address);t.get(i).filter(|v|v.0==address).map(|v|v.1) }
}

/// WP_ENTER. A no-op in the game; with feature "trace" it is the harness recursion guard (fdiff cpu.h EnterGuard).
#[cfg(feature = "trace")] pub static mut DEPTH: u32 = 0;
pub struct EnterGuard;
#[inline(always)]
pub fn enter() -> EnterGuard {
    #[cfg(feature = "trace")] unsafe { DEPTH += 1; if DEPTH > 3000 { std::panic::panic_any(crate::runtime::DepthStop); } }
    EnterGuard
}
pub struct DepthStop;
#[cfg(feature = "trace")] impl Drop for EnterGuard { #[inline(always)] fn drop(&mut self) { unsafe { DEPTH -= 1; } } }
