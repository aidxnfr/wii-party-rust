// SPDX-License-Identifier: GPL-3.0-or-later
// Host adaptation of native_video_transport publication/snapshot; no guest-rule change.
use std::{sync::Arc,cell::RefCell,collections::BTreeMap};
use wprt_gx::remote::HostedRenderer;
type Capture=unsafe extern "C" fn(usize,*mut u64,*mut u32,*mut u32)->bool;
type Visit=unsafe extern "C" fn(usize,u64,u32,u32,u64);
type Query=unsafe extern "C" fn(usize,Visit)->bool;
type CpuRead=unsafe extern "C" fn(usize,u64,*mut *const u8,*mut u32,*mut u32)->bool;
// Optional normal CPU view exercises the existing frame-consumer path while GPU publication stays bound.
pub fn cpu_view()->bool{std::env::var_os("WPGX_HOST_GPU_CPU_VIEW").is_some()}
pub struct Controls{query:Query,renderer:Arc<HostedRenderer>}
pub struct Frame{pub texture:wgpu::Texture,pub width:u32,pub height:u32,pub serial:u64,pub interps:Vec<wgpu::Texture>}
pub unsafe fn bind(library:&libloading::Library,renderer:Option<&Arc<HostedRenderer>>)->Result<Option<Controls>,String>{
 if std::env::var_os("WPGX_HOST_GPU_PUBLICATION").is_none(){return Ok(None);}
 if std::env::var_os("WPGX_HOST_DX12_FORCE_EARLY").is_none()||!crate::host_present::enabled(){return Err("GPU publication requires accepted shared DX12/source presenter".into());}
 let renderer=renderer.ok_or("GPU publication requires a host renderer")?.clone();
 let bind=unsafe{library.get::<unsafe extern "C" fn(usize,Capture)>(b"wpg_gpu_publication_bind\0")}.map_err(|e|e.to_string())?;
 let query=*unsafe{library.get::<Query>(b"wpg_gpu_publication_visit\0")}.map_err(|e|e.to_string())?;
 let bind_cpu=unsafe{library.get::<unsafe extern "C" fn(usize,CpuRead)>(b"wpg_gpu_publication_cpu_bind\0")}.map_err(|e|e.to_string())?;
 unsafe{bind_cpu(Arc::as_ptr(&renderer) as usize,cpu_read);
 bind(Arc::as_ptr(&renderer) as usize,capture);}Ok(Some(Controls{query,renderer}))
}
unsafe extern "C" fn capture(context:usize,token:*mut u64,width:*mut u32,height:*mut u32)->bool{
 let renderer=unsafe{&*(context as *const HostedRenderer)};
 // Authorized H8 performance observation only. Existing guest CPU/GPU-wait counters remain in gx.rs.
 // Capture wall time includes CPU work and completion waits; it is NOT GPU execution time.
 static ENABLED:std::sync::OnceLock<bool>=std::sync::OnceLock::new();
 let enabled=*ENABLED.get_or_init(||std::env::var_os("WP_HOST_PUBLISH_TIMES").is_some());
 let started=enabled.then(std::time::Instant::now);
 let frame=renderer.capture_publication();
 if let Some(started)=started{if frame.is_some(){
  let now=std::time::Instant::now();
  PUBLICATION_TIMES.with(|all|{
   let mut all=all.borrow_mut();let sample=all.entry(context).or_insert((now,0u64,0.0f64));
   sample.1+=1;sample.2+=now.duration_since(started).as_secs_f64();
   let seconds=now.duration_since(sample.0).as_secs_f64();
   if seconds>=1.0{
    eprintln!("publication context={} frames={} window_s={:.6} interval_ms={:.6} capture_wall_ms_per_frame={:.6}",context,sample.1,seconds,seconds*1000.0/sample.1 as f64,sample.2*1000.0/sample.1 as f64);
    *sample=(now,0,0.0);
   }
  });
 }}
 if let Some((t,w,h))=frame{unsafe{*token=t;*width=w;*height=h;}true}else{false}
}
// Same-thread returned storage lifetime as the existing wpg_frame snapshot contract.
// Original TLS snapshot is per DLL; retain one buffer per host renderer context on each thread.
thread_local!{static CPU_PIXELS:RefCell<BTreeMap<usize,Vec<u8>>>=const{RefCell::new(BTreeMap::new())};}
unsafe extern "C" fn cpu_read(context:usize,token:u64,data:*mut *const u8,width:*mut u32,height:*mut u32)->bool{
 let renderer=unsafe{&*(context as *const HostedRenderer)};
 let Some((pixels,w,h))=renderer.read_publication(token)else{return false;};
 CPU_PIXELS.with(|storage|{
  let mut contexts=storage.borrow_mut();let bytes=contexts.entry(context).or_default();bytes.clear();bytes.reserve(pixels.len()*4);
  // Source wphost_frame RGB->opaque RGBA packing, without a native vector leaf.
  for p in pixels{bytes.extend_from_slice(&[(p>>16)as u8,(p>>8)as u8,p as u8,255]);}
  unsafe{*data=bytes.as_ptr();*width=w;*height=h;}true
 })
}
thread_local!{static PUBLICATION_TIMES:RefCell<BTreeMap<usize,(std::time::Instant,u64,f64)>>=const{RefCell::new(BTreeMap::new())};}
struct Read<'a>{renderer:&'a HostedRenderer,last:u64,frame:Option<Frame>}
unsafe extern "C" fn visit(context:usize,token:u64,width:u32,height:u32,serial:u64){
 let read=unsafe{&mut*(context as *mut Read<'_>)};if read.last==serial{return;}
 let texture=read.renderer.publication(token).expect("publication token changed under source lock");
 let interps=read.renderer.publication_interps(token);
 read.frame=Some(Frame{texture,width,height,serial,interps});
}
impl Controls{pub fn read(&self,last:u64)->Option<Frame>{
 let mut read=Read{renderer:&self.renderer,last,frame:None};
 unsafe{(self.query)(&mut read as *mut Read<'_> as usize,visit);}
 read.frame
}}
