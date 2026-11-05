// SPDX-License-Identifier: GPL-3.0-or-later
// H8 Bevy adapter of pristine video.cpp window_proc/kMenuTimer and gx_render.cpp options overlay.
// UNACCEPTED: original policies/fonts/rendering remain in the guest, not recreated with Bevy widgets.
use bevy::{prelude::*,input::{keyboard::KeyboardInput,ButtonState,mouse::MouseWheel},window::PrimaryWindow};
use std::ffi::c_void;
type Open=unsafe extern "C" fn()->bool;
type Client=unsafe extern "C" fn(i32,i32,bool);
type Void=unsafe extern "C" fn();
type Key=unsafe extern "C" fn(u32,bool)->bool;
type Wheel=unsafe extern "C" fn(i32)->bool;
type Click=unsafe extern "C" fn(i32,i32,bool)->bool;
type Hover=unsafe extern "C" fn(i32,i32);
type Receive=unsafe extern "C" fn(*mut c_void,*const u32,usize,u32,u32);
type Take=unsafe extern "C" fn(*mut u64,*mut bool,*mut c_void,Receive)->bool;
#[repr(C)]#[derive(Clone,Copy,Default)]pub struct Rect{pub x:i32,pub y:i32,pub width:i32,pub height:i32}
type Place=unsafe extern "C" fn(i32,i32,i32,i32,*mut Rect);
pub struct Controls{
 open:Open,client:Client,toggle:Void,key:Key,wheel:Wheel,click:Click,hover:Hover,
 refresh:Void,poll:Open,fullscreen:Open,take:Take,place:Place,
 version:u64,visible:bool,texture:Option<wgpu::Texture>,width:u32,height:u32,
 dimensions:Option<(i32,i32)>,
}
#[derive(Clone)]pub struct Overlay{pub texture:wgpu::Texture,pub rect:Rect}
// Presentation textures must use Bevy's device even in the accepted two-device compatibility mode.
#[derive(Resource)]pub struct PresentationGpu{pub device:wgpu::Device,pub queue:wgpu::Queue}
// Every function pointer belongs to its Guest's retained library.
pub unsafe fn bind(lib:&libloading::Library)->Result<Option<Controls>,String>{
 let open=match unsafe{lib.get::<Open>(b"wpg_options_open\0")}{Ok(f)=>*f,Err(_)=>return Ok(None)};
 macro_rules! get{($ty:ty,$name:literal)=>{*unsafe{lib.get::<$ty>(concat!($name,"\0").as_bytes())}.map_err(|e|e.to_string())?};}
 Ok(Some(Controls{open,client:get!(Client,"wpg_options_client"),toggle:get!(Void,"wpg_options_toggle"),key:get!(Key,"wpg_options_key"),
 wheel:get!(Wheel,"wpg_options_wheel"),click:get!(Click,"wpg_options_click"),hover:get!(Hover,"wpg_options_hover"),
 refresh:get!(Void,"wpg_options_refresh"),poll:get!(Open,"wpg_options_poll"),fullscreen:get!(Open,"wpg_options_fullscreen"),
 take:get!(Take,"wpg_options_overlay"),place:get!(Place,"wpg_options_place"),version:0,visible:false,texture:None,width:0,height:0,dimensions:None}))
}
impl Controls{
 pub fn is_open(&self)->bool{unsafe{(self.open)()}}
 pub fn close(&mut self){if self.is_open(){unsafe{(self.toggle)();}self.visible=false;}}
 pub fn overlay(&self,cw:i32,ch:i32)->Option<Overlay>{
  if !self.visible{return None;}let texture=self.texture.as_ref()?;
  let mut rect=Rect::default();unsafe{(self.place)(cw,ch,self.width as i32,self.height as i32,&mut rect);}
  Some(Overlay{texture:texture.clone(),rect})
 }
 fn collect(&mut self,gpu:&PresentationGpu){
  struct Pixels{bytes:Vec<u8>,width:u32,height:u32}
  unsafe extern "C" fn receive(c:*mut c_void,p:*const u32,n:usize,w:u32,h:u32){
   let out=unsafe{&mut *c.cast::<Pixels>()};out.width=w;out.height=h;
   if n>0{out.bytes=unsafe{std::slice::from_raw_parts(p.cast::<u8>(),n*4)}.to_vec();}
  }
  let mut pixels=Pixels{bytes:Vec::new(),width:0,height:0};
  let changed=unsafe{(self.take)(&mut self.version,&mut self.visible,(&mut pixels as *mut Pixels).cast(),receive)};
  if !changed||pixels.bytes.is_empty(){return;}
  let(width,height)=(pixels.width,pixels.height);
  assert_eq!(pixels.bytes.len(),width as usize*height as usize*4,"source options image length");
  // port of backend.rs upload_overlay: original little-endian B8G8R8A8, not sRGB.
  if self.texture.is_none()||self.width!=width||self.height!=height{
   self.texture=Some(gpu.device.create_texture(&wgpu::TextureDescriptor{label:Some("source options overlay"),
    size:wgpu::Extent3d{width,height,depth_or_array_layers:1},mip_level_count:1,sample_count:1,
    dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Bgra8Unorm,
    usage:wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST,view_formats:&[]}));
   self.width=width;self.height=height;
  }
  gpu.queue.write_texture(self.texture.as_ref().unwrap().as_image_copy(),&pixels.bytes,
   wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(width*4),rows_per_image:None},
   wgpu::Extent3d{width,height,depth_or_array_layers:1});
 }
}
// Source options_window.cpp kKey* declarations. These menu keys are not custom game bindings.
fn source_key(code:KeyCode)->Option<u32>{match code{
 KeyCode::Backspace=>Some(0x08),KeyCode::Enter=>Some(0x0d),KeyCode::Escape=>Some(0x1b),KeyCode::Space=>Some(0x20),
 KeyCode::ArrowLeft=>Some(0x25),KeyCode::ArrowUp=>Some(0x26),KeyCode::ArrowRight=>Some(0x27),KeyCode::ArrowDown=>Some(0x28),_=>None}}
pub fn update(mut guests:ResMut<crate::Guests>,mut windows:Query<(Entity,&mut Window),With<PrimaryWindow>>,
 mut keys:MessageReader<KeyboardInput>,mut wheel:MessageReader<MouseWheel>,mouse:Res<ButtonInput<MouseButton>>,
 time:Res<Time<Real>>,gpu:Res<PresentationGpu>,mut timer:Local<Option<Timer>>){
 let Ok((entity,mut window))=windows.single_mut()else{return;};
 // HOST POLICY: Claude LANES 2026-10-27 18:10. Each DLL owns its settings/menu;
 // only the active guest receives menu events/timer. This is not an original-game claim.
 let count=guests.items.len();let active=guests.active;
 for(i,guest)in guests.items.iter_mut().enumerate(){
  let(_,width,height)=crate::physical_viewport(i,count,&window);
  if let Some(options)=&mut guest.options{
   unsafe{(options.client)(width,height,window.focused&&i==active);}
   if options.dimensions!=Some((width,height)){unsafe{(options.refresh)();}options.dimensions=Some((width,height));}
  }
 }
 let(left,_,_)=crate::physical_viewport(active,count,&window);
 let Some(options)=guests.items[active].options.as_mut()else{keys.clear();wheel.clear();return;};
 let mut fullscreen=false;
 for event in keys.read(){
  if event.window!=entity||event.state!=ButtonState::Pressed{continue;}
  // Source WM_SYSKEYDOWN/F10 toggles only on a non-repeat press, even while the menu is open.
  if event.key_code==KeyCode::F10&&!event.repeat{unsafe{(options.toggle)();}continue;}
  if options.is_open(){if let Some(code)=source_key(event.key_code){fullscreen|=unsafe{(options.key)(code,event.repeat)};}}
 }
 if let Some(cursor)=window.physical_cursor_position(){
  if options.is_open(){
   let x=(cursor.x as i32).wrapping_sub(left);let y=cursor.y as i32;
   unsafe{(options.hover)(x,y);}
   for(button,right)in [(MouseButton::Left,false),(MouseButton::Right,true)]{
    if mouse.just_pressed(button){fullscreen|=unsafe{(options.click)(x,y,right)};}
   }
  }
 }
 for event in wheel.read(){if event.window==entity&&options.is_open(){
  let delta=if event.y>0.0{1}else if event.y<0.0{-1}else{0};fullscreen|=unsafe{(options.wheel)(delta)};
 }}
 // Source video.cpp kMenuTimerPeriod=16ms. No guest frame counter or fabricated simulation cadence.
 let timer=timer.get_or_insert_with(||Timer::new(std::time::Duration::from_millis(16),TimerMode::Repeating));
 if timer.tick(time.delta()).just_finished(){fullscreen|=unsafe{(options.poll)()};}
 if fullscreen{
  let enabled=unsafe{(options.fullscreen)()};
  let mode=crate::host_window::mode(enabled);if window.mode!=mode{window.mode=mode;}
 }
 options.collect(&gpu);
}
