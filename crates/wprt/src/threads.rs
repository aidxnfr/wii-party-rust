// SPDX-License-Identifier: GPL-3.0-or-later
// Port of engine/src/core/threads.cpp (non-WP_TRACE scheduler).
use crate::{cpu::{Cpu,SPR_GQR0,mfcr,mtcrf,mfxer,mtxer},memory::{rd32,rd64,wr32,wr64}};
const GPR:u32=0x000;
const CONDITION:u32=0x080;
const LINK:u32=0x084;
const COUNTER:u32=0x088;
const EXCEPTION:u32=0x08c;
const FPR:u32=0x090;
const FPSCR:u32=0x194;
const SRR0:u32=0x198;
const SRR1:u32=0x19c;
const GQR:u32=0x1a4;
const PAIRED:u32=0x1c8;
const JUMP_LINK:u32=0x00;
const JUMP_CONDITION:u32=0x04;
const JUMP_STACK:u32=0x08;
const JUMP_TOC:u32=0x0c;
const JUMP_GPR:u32=0x14;
const JUMP_FPR:u32=0x60;
const JUMP_FPSCR:u32=0x180;
const FIRST_SAVED_REGISTER:u32=13;
const FIRST_SAVED_FLOAT:u32=14;
// port of engine/src/core/threads.cpp store; retain interleaved GPR/FPR/paired write order.
pub fn store(c:&Cpu,context:u32) {
    for i in 0..32u32 {
        wr32(context.wrapping_add(GPR+4*i),c.r[i as usize]);
        wr64(context.wrapping_add(FPR+8*i),c.f[i as usize].to_bits());
        wr64(context.wrapping_add(PAIRED+8*i),c.ps1[i as usize].to_bits());
    }
    wr32(context.wrapping_add(CONDITION),mfcr(c));
    wr32(context.wrapping_add(LINK),c.lr);
    wr32(context.wrapping_add(COUNTER),c.ctr);
    wr32(context.wrapping_add(EXCEPTION),mfxer(c));
    wr32(context.wrapping_add(FPSCR),c.fpscr);
    wr32(context.wrapping_add(SRR0),c.lr);
    wr32(context.wrapping_add(SRR1),c.msr);
    for i in 0..8u32 { wr32(context.wrapping_add(GQR+4*i),c.spr[(SPR_GQR0+i)as usize]); }
}
// port of engine/src/core/threads.cpp restore; MSR is deliberately not restored here (native behavior).
pub fn restore(c:&mut Cpu,context:u32) {
    for i in 0..32u32 {
        c.r[i as usize]=rd32(context.wrapping_add(GPR+4*i));
        c.f[i as usize]=f64::from_bits(rd64(context.wrapping_add(FPR+8*i)));
        c.ps1[i as usize]=f64::from_bits(rd64(context.wrapping_add(PAIRED+8*i)));
    }
    mtcrf(c,0xff,rd32(context.wrapping_add(CONDITION)));
    c.lr=rd32(context.wrapping_add(LINK));
    c.ctr=rd32(context.wrapping_add(COUNTER));
    mtxer(c,rd32(context.wrapping_add(EXCEPTION)));
    c.fpscr=rd32(context.wrapping_add(FPSCR));
    for i in 0..8u32 { c.spr[(SPR_GQR0+i)as usize]=rd32(context.wrapping_add(GQR+4*i)); }
}
// port of engine/src/core/threads.cpp float_bits.
fn float_bits(value:f64)->u32 { (value as f32).to_bits() }
// port of engine/src/core/threads.cpp float_from_bits.
fn float_from_bits(bits:u32)->f64 { f32::from_bits(bits)as f64 }
// port of engine/src/core/threads.cpp store_jump.
pub fn store_jump(c:&Cpu,buffer:u32) {
    wr32(buffer.wrapping_add(JUMP_LINK),c.lr);
    wr32(buffer.wrapping_add(JUMP_CONDITION),mfcr(c));
    wr32(buffer.wrapping_add(JUMP_STACK),c.r[1]);
    wr32(buffer.wrapping_add(JUMP_TOC),c.r[2]);
    for i in FIRST_SAVED_REGISTER..32 { wr32(buffer.wrapping_add(JUMP_GPR+4*(i-FIRST_SAVED_REGISTER)),c.r[i as usize]); }
    for i in FIRST_SAVED_FLOAT..32 {
        let slot=buffer.wrapping_add(JUMP_FPR+0x10*(i-FIRST_SAVED_FLOAT));
        wr64(slot,c.f[i as usize].to_bits());
        wr32(slot.wrapping_add(8),float_bits(c.f[i as usize]));
        wr32(slot.wrapping_add(12),float_bits(c.ps1[i as usize]));
    }
    wr64(buffer.wrapping_add(JUMP_FPSCR),u64::from(c.fpscr));
}
// port of engine/src/core/threads.cpp restore_jump; native float-slot restoration and FPSCR low word retained.
pub fn restore_jump(c:&mut Cpu,buffer:u32) {
    c.lr=rd32(buffer.wrapping_add(JUMP_LINK));
    mtcrf(c,0xff,rd32(buffer.wrapping_add(JUMP_CONDITION)));
    c.r[1]=rd32(buffer.wrapping_add(JUMP_STACK));
    c.r[2]=rd32(buffer.wrapping_add(JUMP_TOC));
    for i in FIRST_SAVED_REGISTER..32 { c.r[i as usize]=rd32(buffer.wrapping_add(JUMP_GPR+4*(i-FIRST_SAVED_REGISTER))); }
    for i in FIRST_SAVED_FLOAT..32 {
        let slot=buffer.wrapping_add(JUMP_FPR+0x10*(i-FIRST_SAVED_FLOAT));
        c.f[i as usize]=float_from_bits(rd32(slot.wrapping_add(8)));
        c.ps1[i as usize]=float_from_bits(rd32(slot.wrapping_add(12)));
    }
    c.fpscr=rd32(buffer.wrapping_add(JUMP_FPSCR+4));
}

use crate::{fiber,runtime,ffi,dispatch};
use crate::memory::{rd16,wr16};
use std::{collections::BTreeMap,ffi::c_void,sync::atomic::Ordering};
#[derive(Clone,Copy)]struct SavedContext{fiber:fiber::Handle,point:u64,resume:u32}
#[derive(Clone,Copy)]struct SavedJump{fiber:fiber::Handle,point:u64,link:u32,stack:u32}
#[derive(Clone,Copy)]struct FiberRecord{fiber:fiber::Handle,thread:u32}
#[derive(Clone,Copy)]struct ContextEvent{kind:char,context:u32,detail:u32}
struct State{
    saved:BTreeMap<u32,SavedContext>,jumps:BTreeMap<u32,SavedJump>,fibers:Vec<FiberRecord>,
    history:[ContextEvent;64],history_next:usize,fallback_resumes:u32,logged_histories:u32,
    most_fibers:usize,jump_buffer:u32,jump_value:u32,start_value:u32,cpu:*mut Cpu,
    resume_point:u64,resume_context:u32,
}
// Pristine globals are confined to the single guest execution thread. No borrow may span a fiber switch.
thread_local!{static STATE:std::cell::UnsafeCell<State>=std::cell::UnsafeCell::new(State{
    saved:BTreeMap::new(),jumps:BTreeMap::new(),fibers:Vec::new(),
    history:[ContextEvent{kind:'\0',context:0,detail:0};64],history_next:0,fallback_resumes:0,logged_histories:0,
    most_fibers:0,jump_buffer:0,jump_value:0,start_value:1,cpu:std::ptr::null_mut(),resume_point:0,resume_context:0,
});}
fn state()->*mut State{STATE.with(|s|s.get())}
// port of engine/src/core/threads.cpp note.
fn note(kind:char,context:u32,detail:u32){unsafe{let s=&mut*state();s.history[s.history_next%64]=ContextEvent{kind,context,detail};s.history_next=s.history_next.wrapping_add(1);}}
// port of engine/src/core/threads.cpp print_history.
fn print_history(){ffi::log("threads","recent thread context events, oldest first (s save, l load, n load without a record, d discard, f forget)");unsafe{let s=&*state();for i in 0..64{let e=s.history[(s.history_next.wrapping_add(i))%64];if e.kind!='\0'{ffi::log("threads",&format!("  {} context {:08x} {:08x}",e.kind,e.context,e.detail));}}}}
// port of engine/src/core/threads.cpp switch_to (non-WP_TRACE).
fn switch_to(target:fiber::Handle){crate::interrupts::interrupt_left();fiber::switch_to(target);}
// port of engine/src/core/threads.cpp resume_pending; approved token substitution for longjmp.
fn resume_pending()->!{let point=unsafe{let s=&mut*state();let p=s.resume_point;s.resume_point=0;p};runtime::resume_to(point)}
// port of engine/src/core/threads.cpp thread_active.
fn thread_active(thread:u32)->bool{let mut item=rd32(0x800000dc);for _ in 0..256{if item==0{break;}if item==thread{return true;}item=rd32(item.wrapping_add(0x2fc));}false}
// port of engine/src/core/threads.cpp fiber_reachable.
fn fiber_reachable(record:FiberRecord)->bool{if record.fiber==fiber::current(){return true;}if record.thread!=0&&!thread_active(record.thread){return false;}unsafe{let s=&*state();s.saved.values().any(|v|v.fiber==record.fiber)||s.jumps.values().any(|v|v.fiber==record.fiber)}}
// port of engine/src/core/threads.cpp forget_fiber (non-WP_TRACE).
fn forget_fiber(target:fiber::Handle){unsafe{let s=&mut*state();s.saved.retain(|_,v|v.fiber!=target);s.jumps.retain(|_,v|v.fiber!=target);}}
// port of engine/src/core/threads.cpp create_fiber.
fn create_fiber(entry:fiber::Entry,parameter:u32,thread:u32)->fiber::Handle{unsafe{
    (*state()).jumps.retain(|&buffer,j|j.link==rd32(buffer.wrapping_add(JUMP_LINK))&&j.stack==rd32(buffer.wrapping_add(JUMP_STACK)));
    let mut i=0;while i<(*state()).fibers.len(){let record=(&(*state()).fibers)[i];if fiber_reachable(record){i+=1;continue;}(*state()).fibers.remove(i);forget_fiber(record.fiber);fiber::destroy(record.fiber);}
    let created=fiber::create(4<<20,entry,parameter as usize as *mut c_void);
    (*state()).fibers.push(FiberRecord{fiber:created,thread:if thread_active(thread){thread}else{0}});
    if (*state()).fibers.len()>(*state()).most_fibers{(*state()).most_fibers=(*state()).fibers.len();ffi::log("threads",&format!("{} host stacks for guest threads",(*state()).most_fibers));}created
}}
// port of engine/src/core/threads.cpp discard_fiber (non-WP_TRACE).
fn discard_fiber(target:fiber::Handle){forget_fiber(target);if target==fiber::current(){return;}unsafe{(*state()).fibers.retain(|r|r.fiber!=target);}fiber::destroy(target);}
// port of engine/src/core/threads.cpp find_resume.
fn find_resume(address:u32)->Option<runtime::GuestFn>{if address<0x80000000{crate::modules::find_module_resume(address)}else{runtime::resume_function(address)}}
// port of engine/src/core/threads.cpp continue_at.
fn continue_at(c:&mut Cpu,mut address:u32){while address!=0{if let Some(function)=find_resume(address){runtime::RESUME_ADDRESS.store(address,Ordering::Relaxed);function(c);}else{runtime::call(c,address);}address=c.lr;}}
// port of engine/src/core/threads.cpp start_thread.
unsafe fn start_thread(parameter:*mut c_void){
    let context=parameter as usize as u32;let c=&mut*(*state()).cpu;restore(c,context);c.msr=rd32(context.wrapping_add(SRR1));let entry=rd32(context.wrapping_add(SRR0));let exit=c.lr;
    if find_resume(entry).is_some(){
        ffi::log("threads",&format!("context {context:08x} resumed at {entry:08x} from its guest stack, without its saved host stack"));
        if (*state()).logged_histories<3{(*state()).logged_histories+=1;print_history();}
        (*state()).fallback_resumes+=1;if (*state()).fallback_resumes>100{print_history();ffi::log("threads",&format!("guest thread contexts were resumed from their guest stacks {} times",(*state()).fallback_resumes));dispatch::fatal_error("guest thread contexts were resumed without their saved host stacks over and over; stopping before the memory runs out");}
        c.r[3]=1;continue_at(c,entry);eprintln!("resumed context {context:08x} returned without exiting");std::process::abort();
    }
    runtime::call(c,entry);runtime::call(c,exit);eprintln!("thread function {entry:08x} returned without exiting (exit function {exit:08x})");std::process::abort();
}
// port of engine/src/core/threads.cpp start_jump.
unsafe fn start_jump(parameter:*mut c_void){let buffer=parameter as usize as u32;let c=&mut*(*state()).cpu;restore_jump(c,buffer);let target=c.lr;if find_resume(target).is_some(){c.r[3]=(*state()).start_value;continue_at(c,target);}else{c.lr=0;runtime::call(c,target);}eprintln!("jump function returned without exiting");std::process::abort();}
// port of engine/src/core/threads.cpp init_threads; install the E1 token hook interface.
pub fn init_threads(c:&mut Cpu){unsafe{(*state()).cpu=c;}fiber::adopt_current_thread();runtime::set_thread_hooks(runtime::ThreadHooks{save_context,resume_context,save_jump,resume_jump});}
// port of engine/src/core/threads.cpp save_context; point becomes its approved u64 token.
pub fn save_context(c:&mut Cpu)->u64{let context=c.r[3];store(c,context);note('s',context,c.lr);let point=runtime::new_token();unsafe{(*state()).saved.insert(context,SavedContext{fiber:fiber::current(),point,resume:c.lr});}c.r[3]=0;point}
// port of engine/src/core/threads.cpp resume_context.
pub fn resume_context(c:&mut Cpu){let context=unsafe{(*state()).resume_context};restore(c,context);c.r[3]=1;}
// port of engine/src/core/threads.cpp load_context.
pub fn load_context(c:&mut Cpu){let context=c.r[3];wr16(context.wrapping_add(0x1a2),rd16(context.wrapping_add(0x1a2))&!2);unsafe{
    let mut saved=(*state()).saved.get(&context).copied();if let Some(record)=saved{if rd32(context.wrapping_add(SRR0))!=record.resume{note('d',context,record.resume);discard_fiber(record.fiber);saved=(*state()).saved.get(&context).copied();}}
    let Some(saved)=saved else{note('n',context,rd32(context.wrapping_add(SRR0)));let target=create_fiber(start_thread,context,context);switch_to(target);if (*state()).resume_point!=0{resume_pending();}return;};
    (*state()).saved.remove(&context);note('l',context,saved.resume);(*state()).resume_point=saved.point;(*state()).resume_context=context;
    if saved.fiber!=fiber::current(){switch_to(saved.fiber);}if (*state()).resume_point!=0{resume_pending();}
}}
// port of engine/src/core/threads.cpp enter_exception_context.
pub fn enter_exception_context(context:u32){wr16(context.wrapping_add(0x1a2),rd16(context.wrapping_add(0x1a2))|2);}
// port of engine/src/core/threads.cpp forget_saved_context.
pub fn forget_saved_context(context:u32,point:u64){wr16(context.wrapping_add(0x1a2),rd16(context.wrapping_add(0x1a2))&!2);unsafe{if let Some(saved)=(*state()).saved.get(&context).copied(){if saved.point==point{note('f',context,saved.resume);(*state()).saved.remove(&context);}}}}
// port of engine/src/core/threads.cpp save_jump.
pub fn save_jump(c:&mut Cpu)->u64{let buffer=c.r[3];store_jump(c,buffer);let point=runtime::new_token();unsafe{(*state()).jumps.insert(buffer,SavedJump{fiber:fiber::current(),point,link:rd32(buffer.wrapping_add(JUMP_LINK)),stack:rd32(buffer.wrapping_add(JUMP_STACK))});}c.r[3]=0;point}
// port of engine/src/core/threads.cpp resume_jump.
pub fn resume_jump(c:&mut Cpu){unsafe{restore_jump(c,(*state()).jump_buffer);c.r[3]=(*state()).jump_value;}}
// port of engine/src/core/threads.cpp long_jump.
pub fn long_jump(c:&mut Cpu){let buffer=c.r[3];let value=if c.r[4]!=0{c.r[4]}else{1};unsafe{
    let saved=(*state()).jumps.get(&buffer).copied();let resumable=saved.is_some_and(|s|s.link==rd32(buffer.wrapping_add(JUMP_LINK))&&s.stack==rd32(buffer.wrapping_add(JUMP_STACK)));
    if !resumable{(*state()).jumps.remove(&buffer);(*state()).start_value=value;let target=create_fiber(start_jump,buffer,rd32(0x800000e4));switch_to(target);if (*state()).resume_point!=0{resume_pending();}return;}
    let saved=saved.unwrap();(*state()).jumps.remove(&buffer);(*state()).resume_point=saved.point;(*state()).jump_buffer=buffer;(*state()).jump_value=value;
    if saved.fiber!=fiber::current(){switch_to(saved.fiber);}if (*state()).resume_point!=0{resume_pending();}
}}
// port of engine/src/core/threads.cpp bad_resume.
pub fn bad_resume(_: &mut Cpu)->!{eprintln!("no resume point at {:08x} in the function found for it",runtime::TAKEN_RESUME_ADDRESS.load(Ordering::Relaxed));std::process::abort();}
// port of engine/src/core/threads.cpp print_guest_registers.
pub fn print_guest_registers(){unsafe{let p=(*state()).cpu;if p.is_null(){return;}let c=&*p;eprintln!("guest registers: lr={:08x} sp={:08x} r3={:08x} r4={:08x} r5={:08x} r6={:08x} r7={:08x} r30={:08x} r31={:08x}",c.lr,c.r[1],c.r[3],c.r[4],c.r[5],c.r[6],c.r[7],c.r[30],c.r[31]);crate::modules::describe_loaded_modules(c.lr);}}
