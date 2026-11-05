// SPDX-License-Identifier: GPL-3.0-or-later
// H6 host adapter, not guest behaviour. Source RAM mapping: memory.h; dispatch boundary: runtime.cpp call.
use crate::{cpu::Cpu,memory};
use std::{cell::Cell,collections::BTreeMap,sync::{Mutex,OnceLock,RwLock,mpsc,atomic::{AtomicBool,Ordering}},time::Duration};
pub type Callback=unsafe extern "C" fn(*mut Cpu);
static HOOKS:OnceLock<RwLock<BTreeMap<u32,Callback>>>=OnceLock::new();
enum Operation{Read(u32),Write(u32,u32)}
struct Request{operation:Operation,reply:mpsc::SyncSender<Option<u32>>}
static REQUESTS:Mutex<Vec<Request>>=Mutex::new(Vec::new());
static PENDING:AtomicBool=AtomicBool::new(false);
static HAS_HOOKS:AtomicBool=AtomicBool::new(false);
thread_local!{static GUEST_THREAD:Cell<bool>=const{Cell::new(false)};}
pub fn bind_thread(){GUEST_THREAD.with(|v|v.set(true));}
fn execute(operation:Operation)->Option<u32>{
    let address=match operation{Operation::Read(a)|Operation::Write(a,_)=>a};
    // Host RAM access excludes hardware registers. Locked cache uses memory.h host(), not MMIO rd32.
    let valid=if address&0xf0000000==memory::LOCKED_CACHE_BASE{(address&(memory::LOCKED_CACHE_SIZE-1))<=memory::LOCKED_CACHE_SIZE-4}
        else{address<memory::SLOW_ADDRESSES&&(address&memory::ADDRESS_MASK)<=memory::PHYSICAL_SIZE-4};
    if !valid||memory::base_ptr().is_null(){return None;}
    unsafe{let p=memory::host(address).cast::<[u8;4]>();match operation{Operation::Read(_)=>Some(u32::from_be_bytes(*p)),Operation::Write(_,v)=>{*p=v.to_be_bytes();Some(v)}}}
}
// Called only on the guest's Rust stack. Never hold the queue lock while executing a request/callback.
#[cfg(not(feature="host-poll-fastpath"))]
pub fn service(){if !PENDING.load(Ordering::Acquire){return;}let pending={let mut q=REQUESTS.lock().unwrap();PENDING.store(false,Ordering::Release);std::mem::take(&mut *q)};for r in pending{let _=r.reply.send(execute(r.operation));}}
// The common no-request path is just the original Acquire load at the original
// poll point. Queue draining stays out of the generated game's inner byte loops.
#[cfg(feature="host-poll-fastpath")]
#[inline(always)]
pub fn service() {
    if PENDING.load(Ordering::Acquire) { service_pending(); }
}
#[cfg(feature="host-poll-fastpath")]
#[cold]
#[inline(never)]
fn service_pending() {
    let pending = {
        let mut q = REQUESTS.lock().unwrap();
        PENDING.store(false, Ordering::Release);
        std::mem::take(&mut *q)
    };
    for r in pending { let _ = r.reply.send(execute(r.operation)); }
}
fn request(operation:Operation)->Option<u32>{
    if GUEST_THREAD.with(Cell::get){return execute(operation);}
    let(tx,rx)=mpsc::sync_channel(1);{let mut q=REQUESTS.lock().unwrap();q.push(Request{operation,reply:tx});PENDING.store(true,Ordering::Release);}
    // Transport timeout only, never a game timing value. Timeout means no acknowledged result.
    rx.recv_timeout(Duration::from_secs(2)).ok().flatten()
}
pub fn read(address:u32)->Option<u32>{request(Operation::Read(address))}
pub fn write(address:u32,value:u32)->bool{request(Operation::Write(address,value)).is_some()}
pub fn hook(address:u32,callback:Option<Callback>){let mut h=HOOKS.get_or_init(Default::default).write().unwrap();if let Some(f)=callback{h.insert(address,f);}else{h.remove(&address);}HAS_HOOKS.store(!h.is_empty(),Ordering::Release);}
pub fn call(cpu:&mut Cpu,address:u32)->bool{if !HAS_HOOKS.load(Ordering::Acquire){return false;}let callback=HOOKS.get_or_init(Default::default).read().unwrap().get(&address).copied();if let Some(f)=callback{unsafe{f(cpu);}true}else{false}}
