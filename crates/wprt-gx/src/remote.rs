// SPDX-License-Identifier: GPL-3.0-or-later
//! Host transport for the accepted renderer, with one explicit context per game DLL.
//! No wgpu handles, Rust containers, closures or allocator ownership cross the C boundary.
use crate::backend::{Renderer, SharedDevice, Host, ScreenVertex, CopyFilter};
use crate::guest_memory::{MemoryCallbacks, Scope};
use std::{ffi::{c_void,c_char,CStr}, sync::{Arc,Mutex}};
pub type FrameSink=extern "C" fn(*mut c_void,*const u32,u32,u32);
#[repr(C)]#[derive(Clone,Copy)]
pub struct RemoteRenderer {
 pub context:usize,
 pub install:unsafe extern "C" fn(usize,Host,MemoryCallbacks),
 pub draw:unsafe extern "C" fn(usize,*const ScreenVertex,u32),
 pub finish:unsafe extern "C" fn(usize),
 pub invalidate:unsafe extern "C" fn(usize),
 pub tlut:unsafe extern "C" fn(usize,u32,u32,u32),
 pub clear:unsafe extern "C" fn(usize,i32,i32,i32,i32),
 pub framebuffer:unsafe extern "C" fn(usize,i32,i32,i32,i32,bool,*const CopyFilter),
 pub texture:unsafe extern "C" fn(usize,u32,u32,i32,i32,i32,i32,bool,u32,bool,bool,bool,*const CopyFilter),
 pub statistics:unsafe extern "C" fn(usize,*mut u32,*mut u32,*mut f64),
 pub copy_statistics:unsafe extern "C" fn(usize,*mut u32,*mut f64),
 pub dump:unsafe extern "C" fn(usize,*const c_char),
 pub frame:unsafe extern "C" fn(usize,FrameSink,*mut c_void)->bool,
}
static REMOTE:Mutex<Option<RemoteRenderer>>=Mutex::new(None);
pub fn current()->Option<RemoteRenderer>{*REMOTE.lock().unwrap()}
/// Called before wpg_run; the host retains the context through the guest worker lifetime.
#[no_mangle]pub unsafe extern "C" fn wpgx_bind_remote(table:*const RemoteRenderer){
 *REMOTE.lock().unwrap()=Some(unsafe{*table});
}
#[cfg(feature="guest-runtime")]
unsafe extern "C" fn local_host(address:u32)->*mut u8{unsafe{wprt::memory::host(address)}}
#[cfg(feature="guest-runtime")]
extern "C" fn local_range(address:u32,size:usize)->bool{
 (address&wprt::memory::ADDRESS_MASK)as usize+size<=wprt::memory::PHYSICAL_SIZE as usize
}
#[cfg(feature="guest-runtime")]
pub fn local_memory()->MemoryCallbacks{MemoryCallbacks{host:local_host,range:local_range}}
#[cfg(not(feature="guest-runtime"))]
pub fn local_memory()->MemoryCallbacks{panic!("host cannot install a standalone guest renderer")}


struct State{renderer:Renderer,memory:MemoryCallbacks,publication:u64,snapshot:Option<wgpu::Texture>,interps:Vec<wgpu::Texture>}
pub struct HostedRenderer{
 state:Mutex<Option<State>>,
 device:SharedDevice,
 // Callback code/register pointers must not outlive their DLL.
 _owner:Arc<dyn Send+Sync>,
}
impl HostedRenderer{
 pub fn new(device:SharedDevice,owner:Arc<dyn Send+Sync>)->Arc<Self>{Arc::new(Self{state:Mutex::new(None),device,_owner:owner})}
 /// Called under the DLL's existing publication lock, not for diagnostic read_frame calls.
 pub fn capture_publication(&self)->Option<(u64,u32,u32)>{
  let mut state=self.state.lock().unwrap();let state=state.as_mut()?;
  let _scope=Scope::enter(state.memory);
  let snapshot=state.renderer.snapshot_host_frame()?;
  let (w,h)=(snapshot.width(),snapshot.height());
  state.publication=state.publication.wrapping_add(1);state.snapshot=Some(snapshot);state.interps=state.renderer.take_interp_frames();
  Some((state.publication,w,h))
 }
 /// The publication lock remains held by the visiting DLL while this clone is retained.
 pub fn publication(&self,token:u64)->Option<wgpu::Texture>{
  let state=self.state.lock().unwrap();let state=state.as_ref()?;
  if state.publication!=token{return None;}state.snapshot.clone()
 }
 /// Claude D2: interpolated frames shown before this publication (empty unless WP_INTERP).
 pub fn publication_interps(&self,token:u64)->Vec<wgpu::Texture>{
  let state=self.state.lock().unwrap();match state.as_ref(){Some(s) if s.publication==token=>s.interps.clone(),_=>Vec::new()}
 }
 /// Same published token, never the newer mutable guest render target.
 pub fn read_publication(&self,token:u64)->Option<(Vec<u32>,u32,u32)>{
  // Pin the immutable texture under the state lock, then release the renderer.
  // GPU wait and CPU pixel packing use only this handle and the shared device.
  // The caller retains the existing DLL publication lock; its token/order is unchanged.
  let texture={
   let state=self.state.lock().unwrap();let state=state.as_ref()?;
   if state.publication!=token{return None;}state.snapshot.clone()?
  };
  let mut pixels=Vec::new();
  let(w,h)=Renderer::read_host_publication(&self.device,&texture,&mut pixels)?;
  Some((pixels,w,h))
 }
 pub fn table(self:&Arc<Self>)->RemoteRenderer{RemoteRenderer{context:Arc::as_ptr(self)as usize,install,draw,finish,invalidate,tlut,clear,framebuffer,texture,statistics,copy_statistics,dump,frame}}
 fn with<R>(&self,f:impl FnOnce(&mut Renderer)->R)->Option<R>{
  let mut state=self.state.lock().unwrap();
  state.as_mut().map(|s|{let _scope=Scope::enter(s.memory);f(&mut s.renderer)})
 }
}
unsafe fn context<'a>(token:usize)->&'a HostedRenderer{unsafe{&*(token as *const HostedRenderer)}}
unsafe extern "C" fn install(c:usize,host:Host,memory:MemoryCallbacks){
 let c=unsafe{context(c)};*c.state.lock().unwrap()=Some(State{renderer:Renderer::with_shared_device(host,c.device.clone()),memory,publication:0,snapshot:None,interps:Vec::new()});
}
// Narrow performance observation: distinguish host draw/renderer-lock wall time from guest GX parsing.
// Default off; no commands, vertices, memory values or render state are changed.
thread_local!{static DRAW_TIME:std::cell::RefCell<std::collections::BTreeMap<usize,(std::time::Instant,u64,u64,f64,f64)>>=const{std::cell::RefCell::new(std::collections::BTreeMap::new())};}
unsafe extern "C" fn draw(c:usize,v:*const ScreenVertex,n:u32){if n!=0{
 static ENABLED:std::sync::OnceLock<bool>=std::sync::OnceLock::new();
 let started=(*ENABLED.get_or_init(||std::env::var_os("WP_HOST_DRAW_TIMES").is_some())).then(std::time::Instant::now);
 let mut lock_seconds=0.0;
 unsafe{context(c)}.with(|r|{
  if let Some(start)=started{lock_seconds=start.elapsed().as_secs_f64();}
  r.draw(unsafe{std::slice::from_raw_parts(v,n as usize)});
 });
 if let Some(start)=started{
  let now=std::time::Instant::now();let wall_seconds=now.duration_since(start).as_secs_f64();
  DRAW_TIME.with(|all|{
   let mut all=all.borrow_mut();let sample=all.entry(c).or_insert((now,0,0,0.0,0.0));
   sample.1+=1;sample.2+=u64::from(n);sample.3+=wall_seconds;sample.4+=lock_seconds;
   let seconds=now.duration_since(sample.0).as_secs_f64();
   if seconds>=1.0{
    eprintln!("host_draw context={} window_s={:.6} calls={} vertices={} wall_ms={:.6} lock_ms={:.6} inside_ms={:.6}",c,seconds,sample.1,sample.2,sample.3*1000.0,sample.4*1000.0,(sample.3-sample.4)*1000.0);
    *sample=(now,0,0,0.0,0.0);
   }
  });
 }
}}
unsafe extern "C" fn finish(c:usize){unsafe{context(c)}.with(|r|r.finish_copies());}
unsafe extern "C" fn invalidate(c:usize){unsafe{context(c)}.with(|r|r.invalidate_textures());}
unsafe extern "C" fn tlut(c:usize,a:u32,o:u32,n:u32){unsafe{context(c)}.with(|r|r.load_tlut(a,o,n));}
unsafe extern "C" fn clear(c:usize,x:i32,y:i32,w:i32,h:i32){unsafe{context(c)}.with(|r|r.clear(x,y,w,h));}
unsafe extern "C" fn framebuffer(c:usize,x:i32,y:i32,w:i32,h:i32,d:bool,f:*const CopyFilter){unsafe{context(c)}.with(|r|r.copy_to_framebuffer(x,y,w,h,d,unsafe{&*f}));}
unsafe extern "C" fn texture(c:usize,a:u32,s:u32,x:i32,y:i32,w:i32,h:i32,half:bool,fmt:u32,i:bool,d:bool,alpha:bool,f:*const CopyFilter){unsafe{context(c)}.with(|r|r.copy_to_texture(a,s,x,y,w,h,half,fmt,i,d,alpha,unsafe{&*f}));}
unsafe extern "C" fn statistics(c:usize,b:*mut u32,v:*mut u32,s:*mut f64){let(x,y,z)=unsafe{context(c)}.with(|r|r.take_statistics()).unwrap_or_default();unsafe{*b=x;*v=y;*s=z;}}
unsafe extern "C" fn copy_statistics(c:usize,b:*mut u32,s:*mut f64){let(x,y)=unsafe{context(c)}.with(|r|r.take_copy_statistics()).unwrap_or_default();unsafe{*b=x;*s=y;}}
unsafe extern "C" fn dump(c:usize,p:*const c_char){let p=if p.is_null(){None}else{Some(unsafe{CStr::from_ptr(p)}.to_string_lossy())};unsafe{context(c)}.with(|r|r.dump_copies(p.as_deref()));}
unsafe extern "C" fn frame(c:usize,out:FrameSink,data:*mut c_void)->bool{
 let mut pixels=Vec::new();match unsafe{context(c)}.with(|r|r.read_frame(&mut pixels)).flatten(){Some((w,h))=>{out(data,pixels.as_ptr(),w,h);true},None=>false}
}
