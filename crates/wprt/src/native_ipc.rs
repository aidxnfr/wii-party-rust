// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/ios/ipc.cpp. Source event order and control bits are retained.
use crate::{memory::{base_ptr,ADDRESS_MASK,rd32,wr32},native_ios,runtime::time_base};
use std::{collections::VecDeque,sync::{Mutex,OnceLock}};
// Constants copied programmatically from pristine ipc.cpp.
#[allow(non_upper_case_globals)]const kPpcMessage:u32=0x00;
#[allow(non_upper_case_globals)]const kPpcControl:u32=0x04;
#[allow(non_upper_case_globals)]const kArmMessage:u32=0x08;
#[allow(non_upper_case_globals)]const kPpcInterruptFlags:u32=0x30;
#[allow(non_upper_case_globals)]const kPpcInterruptMask:u32=0x34;
#[allow(non_upper_case_globals)]const kBroadwayCause:u32=0x40000000;
#[allow(non_upper_case_globals)]const kReplyCommand:u32=8;
#[allow(non_upper_case_globals)]const kAcknowledgeTicks:u64=500;
#[allow(non_upper_case_globals)]const kInterruptTicks:u64=100;
extern "C"{
 fn wphw_ipc_report_header(control:u32,flags:u32,mask:u32,ppc:u32,arm:u32,requests:usize,replies:usize,events:usize);
 fn wphw_ipc_report_trace(delta:i64,kind:u8,a:u32,b:u32);
 fn wphw_ipc_report_event(kind:i32,address:u32,delta:i64);
}
#[derive(Default)]struct Control{x1:bool,x2:bool,y1:bool,y2:bool,iy1:bool,iy2:bool}
impl Control{
 // port of ios/ipc.cpp Control::ppc() const.
 fn ppc(&self)->u32{((self.iy2 as u32)<<5)|((self.iy1 as u32)<<4)|((self.x2 as u32)<<3)|((self.y1 as u32)<<2)|((self.y2 as u32)<<1)|(self.x1 as u32)}
 // port of ios/ipc.cpp Control::ppc(uint32_t).
 fn set_ppc(&mut self,v:u32){self.x1=v&1!=0;self.x2=(v>>3)&1!=0;if (v>>2)&1!=0{self.y1=false;}if (v>>1)&1!=0{self.y2=false;}self.iy1=(v>>4)&1!=0;self.iy2=(v>>5)&1!=0;}
}
#[derive(Clone,Copy)]enum EventKind{Request,Reply,Interrupt}
#[derive(Clone,Copy)]struct Event{due:u64,order:u64,kind:EventKind,address:u32}
#[derive(Clone,Copy,Default)]struct Trace{time:u64,kind:u8,a:u32,b:u32}
struct State{ppc_message:u32,arm_message:u32,control:Control,interrupt_flags:u32,interrupt_mask:u32,events:Vec<Event>,event_order:u64,requests:VecDeque<u32>,replies:VecDeque<u32>,last_reply:u64,trace:[Trace;64],trace_next:u32}
// port of ipc.cpp namespace globals; initialized once per DLL, preserving source defaults.
fn state()->&'static Mutex<State>{static S:OnceLock<Mutex<State>>=OnceLock::new();S.get_or_init(||Mutex::new(State{ppc_message:0,arm_message:0,control:Control::default(),interrupt_flags:0,interrupt_mask:kBroadwayCause,events:Vec::new(),event_order:0,requests:VecDeque::new(),replies:VecDeque::new(),last_reply:0,trace:[Trace::default();64],trace_next:0}))}
// port of ipc.cpp trace / schedule / ready / update_interrupts.
fn trace(s:&mut State,kind:u8,a:u32,b:u32){let index=s.trace_next%64;s.trace_next=s.trace_next.wrapping_add(1);s.trace[index as usize]=Trace{time:time_base(),kind,a,b};}
fn schedule(s:&mut State,delay:u64,kind:EventKind,address:u32){let due=time_base().wrapping_add(delay);let order=s.event_order;s.event_order=s.event_order.wrapping_add(1);s.events.push(Event{due,order,kind,address});}
fn ready(s:&State)->bool{!s.control.y1&&!s.control.y2&&(s.interrupt_flags&kBroadwayCause)==0}
fn update_interrupts(s:&mut State){if(s.control.y1&&s.control.iy1)||(s.control.y2&&s.control.iy2){s.interrupt_flags|=kBroadwayCause;}}
// port of ipc.cpp reply. Keep guest writes before scheduling the reply event.
fn reply(s:&mut State,request:u32,result:i32,mut ticks:u64){let now=time_base();if s.last_reply>now{ticks=ticks.wrapping_add(s.last_reply-now);}s.last_reply=now.wrapping_add(ticks);let command=rd32(request);wr32(request.wrapping_add(4),result as u32);wr32(request.wrapping_add(8),command);wr32(request,kReplyCommand);schedule(s,ticks,EventKind::Reply,request);}
// port of ipc.cpp execute.
unsafe fn execute(s:&mut State,address:u32){let mut ticks=0;let result=native_ios::wprt_ios_send(address,&mut ticks);if result!=i32::MIN{reply(s,address,result,ticks);}}
// port of ipc.cpp update_ipc. Requests retain priority; acknowledging a request returns immediately.
unsafe fn update_ipc(s:&mut State){if !ready(s){return;}if let Some(address)=s.requests.pop_front(){trace(s,b'A',address,rd32(address));s.control.x1=false;s.control.y2=true;schedule(s,kInterruptTicks,EventKind::Interrupt,0);execute(s,address);return;}if let Some(address)=s.replies.pop_front(){s.arm_message=address;trace(s,b'R',address,rd32(address.wrapping_add(4)));s.control.y1=true;schedule(s,kInterruptTicks,EventKind::Interrupt,0);}}
// port of ipc.cpp backing. Unlike memory::host, the source explicitly uses only the address mask here.
unsafe fn backing(address:u32)->*mut u32{base_ptr().add((address&ADDRESS_MASK)as usize).cast()}
// port of ipc.cpp read32.
#[no_mangle]pub unsafe extern "C" fn wprt_ipc_read32(address:u32)->u32{let mut s=state().lock().unwrap();let control=s.control.ppc();trace(&mut s,b'r',address&0xff,control);match address&0xff{kPpcControl=>s.control.ppc(),kArmMessage=>s.arm_message,_=>std::ptr::read_unaligned(backing(address)).swap_bytes()}}
// port of ipc.cpp write32; preserve clear-bit interrupt effects before dispatching ready work.
#[no_mangle]pub unsafe extern "C" fn wprt_ipc_write32(address:u32,value:u32){let mut s=state().lock().unwrap();trace(&mut s,b'w',address&0xff,value);match address&0xff{
 kPpcMessage=>s.ppc_message=value,
 kPpcControl=>{s.control.set_ppc(value);if (((value>>2)&1)!=0&&s.control.iy1)||(((value>>1)&1)!=0&&s.control.iy2){s.interrupt_flags|=kBroadwayCause;}if s.control.x1{let message=s.ppc_message;schedule(&mut s,kAcknowledgeTicks,EventKind::Request,message);}update_ipc(&mut s);update_interrupts(&mut s);},
 kPpcInterruptFlags=>{s.interrupt_flags&=!value;update_ipc(&mut s);update_interrupts(&mut s);},
 kPpcInterruptMask=>{s.interrupt_mask=value;update_ipc(&mut s);update_interrupts(&mut s);},
 _=>std::ptr::write_unaligned(backing(address),value.swap_bytes())
}}
// port of ipc.cpp reply public entry point.
#[no_mangle]pub extern "C" fn wprt_ipc_reply(request:u32,result:i32,ticks:u64){reply(&mut state().lock().unwrap(),request,result,ticks);}
// port of ipc.cpp update. Min-element search keeps source tie order and erase order; no heap substitution.
#[no_mangle]pub unsafe extern "C" fn wprt_ipc_update(){let mut s=state().lock().unwrap();native_ios::wprt_ios_update();let(mut request,mut result)=(0,0);while native_ios::wprt_ios_take_completion(&mut request,&mut result){reply(&mut s,request,result,0);}let now=time_base();while !s.events.is_empty(){let mut next=0;for i in 1..s.events.len(){let(a,b)=(&s.events[i],&s.events[next]);if if a.due!=b.due{a.due<b.due}else{a.order<b.order}{next=i;}}if s.events[next].due>now{break;}let event=s.events.remove(next);match event.kind{EventKind::Request=>{s.requests.push_back(event.address);update_ipc(&mut s);},EventKind::Reply=>{s.replies.push_back(event.address);update_ipc(&mut s);},EventKind::Interrupt=>update_interrupts(&mut s)}}}
// port of ipc.cpp interrupt_pending.
#[no_mangle]pub extern "C" fn wprt_ipc_interrupt_pending()->bool{let s=state().lock().unwrap();s.interrupt_flags&s.interrupt_mask!=0}
// port of ipc.cpp report. Native leaves preserve only the source CRT variadic formatting.
#[no_mangle]pub unsafe extern "C" fn wprt_ipc_report(){let s=state().lock().unwrap();wphw_ipc_report_header(s.control.ppc(),s.interrupt_flags,s.interrupt_mask,s.ppc_message,s.arm_message,s.requests.len(),s.replies.len(),s.events.len());let now=time_base();for i in 0..64{let t=s.trace[s.trace_next.wrapping_add(i)as usize%64];if t.kind!=0{wphw_ipc_report_trace((t.time as i64).wrapping_sub(now as i64),t.kind,t.a,t.b);}}for event in &s.events{let kind=match event.kind{EventKind::Request=>0,EventKind::Reply=>1,EventKind::Interrupt=>2};wphw_ipc_report_event(kind,event.address,(event.due as i64).wrapping_sub(now as i64));}}
