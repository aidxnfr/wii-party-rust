// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted H6 host: displays original DLL frames; all rules/assets remain in the private guest DLL.
use bevy::{prelude::*,asset::RenderAssetUsages,render::render_resource::{Extent3d,TextureDimension,TextureFormat},window::PrimaryWindow};
use std::{ffi::{c_char,CString},path::PathBuf,sync::Arc};
mod devshot;
mod host_audio;
mod host_controller;
mod host_present;
mod host_window;
mod host_gpu_frame;
mod host_options;
mod host_keyboard;
use bevy::audio::AddAudioSource;
// Compatibility shim for the existing owned-render devshot module; separate request root from frozen apps.
mod scene_view{pub fn private_root()->Option<std::path::PathBuf>{std::env::var_os("WII_PARTY_PRIVATE_ROOT").map(Into::into)}}
use bevy::render::{RenderApp,RenderPlugin,renderer::{RenderDevice,RenderQueue,RenderAdapter,RenderInstance},settings::{RenderCreation,WgpuSettings,WgpuFeatures,WgpuSettingsPriority}};
use wprt_gx::{backend::SharedDevice,remote::HostedRenderer};
#[derive(Resource)]struct SharedGpu(SharedDevice);
struct SharedGpuPlugin;
impl Plugin for SharedGpuPlugin{
 fn build(&self,_app:&mut App){}
 fn finish(&self,app:&mut App){
  let (device,queue)={let world=app.sub_app(RenderApp).world();
   (world.resource::<RenderDevice>().wgpu_device().clone(),world.resource::<RenderQueue>().clone().into_inner())};
  app.insert_resource(host_options::PresentationGpu{device,queue});
  // Compatibility stage: Vulkan offscreen GX and DX12 Bevy presentation are BOTH host-owned.
  // Vulkan surface creation fails on this PC; DX12 cannot preserve forced early-depth.
  // Keep the accepted source device policy instead of changing shader/depth semantics.
  let shared=if std::env::var_os("WPGX_HOST_SHARED_MINIMAL").is_some()||std::env::var_os("WPGX_HOST_DX12_FORCE_EARLY").is_some(){
   let world=app.sub_app(RenderApp).world();
   SharedDevice{instance:world.resource::<RenderInstance>().clone().into_inner(),adapter:world.resource::<RenderAdapter>().clone().into_inner(),device:world.resource::<RenderDevice>().wgpu_device().clone(),queue:world.resource::<RenderQueue>().clone().into_inner()}
  }else{SharedDevice::source_device().expect("source-compatible offscreen GX device")};
  assert!(shared.device.features().contains(WgpuFeatures::SHADER_EARLY_DEPTH_TEST),"faithful GX requires forced early depth");
  app.insert_resource(SharedGpu(shared));
 }
}
type Run=unsafe extern "C" fn(*const c_char,*const c_char,i32)->i32;
type Frame=unsafe extern "C" fn(*mut *const u8,*mut u32,*mut u32,*mut u64)->bool;
type Input=unsafe extern "C" fn(u32,u32,bool,f32,f32);
type SourceAspect=unsafe extern "C" fn(*mut f64)->bool;
type SourceImagePoint=unsafe extern "C" fn(i32,i32,i32,i32,*mut f32,*mut f32)->bool;
type KeyboardWheel=unsafe extern "C" fn(i32);
type InputV2=unsafe extern "C" fn(u32,u32,bool,f32,f32,bool,bool);
struct Guest {
    options:Option<host_options::Controls>,
    gpu_publication:Option<host_gpu_frame::Controls>,gpu_image:Option<wgpu::Texture>,
    window_controls:Option<host_window::Controls>,
    _library:Arc<libloading::Library>,_gpu:Option<Arc<HostedRenderer>>,_controller:Option<Arc<host_controller::DeviceContext>>,frame:Frame,input:Input,input_v2:Option<InputV2>,keyboard_wheel:Option<KeyboardWheel>,source_aspect:Option<SourceAspect>,source_point:Option<SourceImagePoint>,aspect:Option<f64>,
    image:Handle<Image>,serial:u64,width:u32,height:u32,
    // Claude D2: interpolated frames (then the real frame) spread across one guest frame interval.
    interp_seq:Vec<wgpu::Texture>,interp_start:Option<std::time::Instant>,interp_period:f64,last_arrival:Option<std::time::Instant>,
}
#[derive(Resource)]struct Guests{items:Vec<Guest>,active:usize}
#[derive(Resource)]struct Configuration{dlls:Vec<PathBuf>,data:PathBuf,nands:Vec<PathBuf>,settings:Vec<PathBuf>}
#[derive(Component)]struct GuestView(usize);
fn main(){
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=3&&args.len()!=5{eprintln!("usage: wii-party-hybrid <dll> <extracted> <nand-copy> [second-dll second-nand-copy]");return;}
    let mut dlls=vec![PathBuf::from(&args[0])];let mut nands=vec![PathBuf::from(&args[2])];
    if args.len()==5{dlls.push(PathBuf::from(&args[3]));nands.push(PathBuf::from(&args[4]));}
    // H8 multi-guest policy: each NAND parent owns its settings file; single/native path unchanged.
    // Resolve aliases before starting ANY guest, so two DLLs cannot write the same settings file.
    let mut settings=Vec::new();
    if nands.len()>1{
        for nand in &nands{
            let parent=nand.parent().expect("NAND parent").canonicalize().expect("existing private NAND parent");
            let path=parent.join("settings.ini");
            let resolved=if path.exists(){path.canonicalize().expect("resolve private settings")}else{path};
            assert!(!settings.iter().any(|p:&PathBuf|p.to_string_lossy().eq_ignore_ascii_case(&resolved.to_string_lossy())),"multi-guest settings paths must be distinct");
            settings.push(resolved);
        }
        for path in &settings{
            if !path.exists() && std::path::Path::new("games/wiiparty/settings.ini").exists(){
                std::fs::copy("games/wiiparty/settings.ini",path).expect("seed private guest settings from source configuration");
            }
        }
    }
    // Must occur before the Bevy/game worker threads exist. Native scripts retain priority for H5 reruns.
    unsafe{std::env::set_var("WP_OFFSCREEN","1");std::env::set_var("WP_HOST_INPUT","1");std::env::set_var("WP_REAL_WIIMOTES","0");}
    let mut app=App::new();
    let mut gpu_settings=WgpuSettings::default();
    // Presentation device only; the host-owned source GX device requests its own exact features.
    gpu_settings.features|=WgpuFeatures::DUAL_SOURCE_BLENDING|WgpuFeatures::DEPTH_CLIP_CONTROL;
    // Source-compatible required set instead of Bevy's Functionality/all-features default.
    // Opt-in experiment only; the accepted TWO-device compatibility route remains default.
    if std::env::var_os("WPGX_HOST_SHARED_MINIMAL").is_some()||std::env::var_os("WPGX_HOST_DX12_FORCE_EARLY").is_some(){
        gpu_settings.priority=WgpuSettingsPriority::WebGPU;
        gpu_settings.features|=WgpuFeatures::SHADER_EARLY_DEPTH_TEST;
    }
    gpu_settings.limits.max_bind_groups=gpu_settings.limits.max_bind_groups.max(4);
    gpu_settings.limits.max_sampled_textures_per_shader_stage=gpu_settings.limits.max_sampled_textures_per_shader_stage.max(16);
    gpu_settings.limits.max_samplers_per_shader_stage=gpu_settings.limits.max_samplers_per_shader_stage.max(16);
    app.add_plugins(DefaultPlugins.set(bevy::app::TaskPoolPlugin{task_pool_options:bevy::app::TaskPoolOptions::with_num_threads(std::env::var("WP_BEVY_THREADS").ok().and_then(|v|v.parse().ok()).unwrap_or(4))}).set(RenderPlugin{render_creation:RenderCreation::Automatic(Box::new(gpu_settings)),..default()}).set(WindowPlugin{primary_window:Some(Window{
        title:"Wii party rust rewrite".into(),
        // port of gx_render.cpp Present(0,0) / accepted backend.rs ensure_swapchain: Immediate -> Mailbox -> Fifo.
        // Claude OPT A6b: the Bevy window loop ran uncapped (AutoNoVsync) and spun 6 task-pool threads; vsync paces it to the display.
        // WP_BEVY_VSYNC=0 restores the old uncapped loop.
        present_mode:if std::env::var("WP_BEVY_VSYNC").as_deref()==Ok("0"){bevy::window::PresentMode::AutoNoVsync}else{bevy::window::PresentMode::AutoVsync},
        resolution:(1280,720).into(),focused:false,..default()}),..default()}))
        .add_plugins((SharedGpuPlugin,host_present::SourcePresentPlugin))
        .insert_resource(ClearColor(Color::BLACK)).insert_resource(Configuration{dlls,data:PathBuf::from(&args[1]),nands,settings})
        .add_systems(Startup,start).add_systems(Update,(select_active,host_window::update,host_options::update,update_frames,layout,configure_controllers,send_input,limited_run).chain());
    // Optional authorized performance measure: Bevy Time<Real> CPU app delta, NOT guest/GPU/display FPS.
    if std::env::var_os("WP_HOST_FRAME_TIMES").is_some(){
        use bevy::diagnostic::{FrameTimeDiagnosticsPlugin,LogDiagnosticsPlugin};
        app.add_plugins((FrameTimeDiagnosticsPlugin::default(),LogDiagnosticsPlugin{
            filter:Some([FrameTimeDiagnosticsPlugin::FRAME_TIME].into_iter().collect()),..default()
        }));
    }
    app.add_systems(PreUpdate,host_keyboard::update);
    app.add_audio_source::<host_audio::GuestAudio>();
    devshot::install(&mut app);let exit=app.run();
    // Workers retain their DLLs for the process lifetime. Never unload code from a running guest stack.
    std::process::exit(match exit{bevy::app::AppExit::Success=>0,bevy::app::AppExit::Error(code)=>i32::from(code.get())});
}
// Optional bounded development launch, with the normal window-close path used for cleanup.
fn limited_run(time:Res<Time<Real>>,mut exit:MessageWriter<bevy::app::AppExit>){
    if let Ok(value)=std::env::var("WP_HOST_RUN_SECONDS"){if time.elapsed_secs_f64()>value.parse::<f64>().unwrap(){exit.write(bevy::app::AppExit::Success);}}
}
fn start(mut commands:Commands,mut images:ResMut<Assets<Image>>,config:Res<Configuration>,mut audio:ResMut<Assets<host_audio::GuestAudio>>,gpu:Res<SharedGpu>){
    commands.spawn(Camera2d);let mut guests=Vec::new();
    for (i,path)in config.dlls.iter().enumerate(){
        let library=Arc::new(unsafe{libloading::Library::new(path)}.expect("load private guest DLL"));
        let (run,frame,input)=unsafe{
            let version=*library.get::<unsafe extern "C" fn()->u32>(b"wpg_abi_version\0").expect("guest ABI version");assert_eq!(version(),1);
            (*library.get::<Run>(b"wpg_run\0").unwrap(),*library.get::<Frame>(b"wpg_frame\0").unwrap(),*library.get::<Input>(b"wpg_input\0").unwrap())
        };
        if let Some(path)=config.settings.get(i){
            let path=CString::new(path.to_str().expect("settings path UTF-8")).expect("settings path without NUL");
            let bind=unsafe{library.get::<unsafe extern "C" fn(*const c_char)->bool>(b"wpg_settings_path_bind\0")}.expect("multi-guest requires private settings transport");
            assert!(unsafe{bind(path.as_ptr())},"bind private guest settings before execution");
        }
        if let Some(provider)=unsafe{host_keyboard::bind(&library)}.expect("bind Bevy keyboard event transport"){
            if i==0{commands.insert_resource(provider);}
        }
        let input_v2=unsafe{library.get::<InputV2>(b"wpg_input_v2\0").ok().map(|f|*f)};
        let keyboard_wheel=unsafe{match(library.get::<unsafe extern "C" fn()>(b"wpg_keyboard_bind\0"),library.get::<KeyboardWheel>(b"wpg_keyboard_wheel\0")){
            (Ok(bind),Ok(wheel))=>{bind();Some(*wheel)},(Err(_),Err(_))=>None,_=>panic!("incomplete source keyboard transport")
        }};

        let (source_aspect,source_point)=unsafe{match(library.get::<SourceAspect>(b"wpg_source_aspect\0"),library.get::<SourceImagePoint>(b"wpg_source_image_point\0")){
            (Ok(aspect),Ok(point))=>(Some(*aspect),Some(*point)),(Err(_),Err(_))=>(None,None),_=>panic!("incomplete source geometry transport")
        }};
        if host_present::enabled(){assert!(source_aspect.is_some(),"source presenter requires the source aspect interface");}
        // Bind one host-owned renderer before execution; legacy DLLs keep their accepted standalone GPU.
        let renderer=unsafe{library.get::<unsafe extern "C" fn(*const wprt_gx::remote::RemoteRenderer)>(b"wpg_renderer_bind\0").ok().map(|bind|{
            let context=HostedRenderer::new(gpu.0.clone(),library.clone());
            bind(&context.table());context
        })};
        let gpu_publication=unsafe{host_gpu_frame::bind(&library,renderer.as_ref())}.expect("bind host GPU publication");
        let window_controls=unsafe{host_window::bind(&library)}.expect("bind source window settings");
        let options=unsafe{host_options::bind(&library)}.expect("bind source options transport");
        let controller=unsafe{host_controller::bind(library.clone())}.expect("bind source controller callbacks");
        // Empty transport texture only; never a replacement game asset.
        let image=images.add(Image::new_fill(Extent3d{width:1,height:1,depth_or_array_layers:1},TextureDimension::D2,&[0,0,0,255],host_present::input_format(),RenderAssetUsages::default()));
        if !host_present::enabled(){commands.spawn((ImageNode::new(image.clone()),Node{position_type:PositionType::Absolute,..default()},GuestView(i)));}
        if let Some(source)=unsafe{host_audio::bind(library.clone(),&mut audio)}.expect("bind guest audio before execution"){commands.spawn(AudioPlayer(source));}
        let retained=library.clone();let retained_renderer=renderer.clone();let retained_controller=controller.clone();let data=CString::new(config.data.to_str().unwrap()).unwrap();let nand=CString::new(config.nands[i].to_str().unwrap()).unwrap();
        std::thread::spawn(move||{let _library=retained;let _renderer=retained_renderer;let _controller=retained_controller;let result=unsafe{run(data.as_ptr(),nand.as_ptr(),0)};eprintln!("guest {i} returned {result}");});
        guests.push(Guest{options,gpu_publication,gpu_image:None,window_controls,_library:library,_gpu:renderer,_controller:controller,frame,input,input_v2,keyboard_wheel,source_aspect,source_point,aspect:None,image,serial:0,width:640,height:480,interp_seq:Vec::new(),interp_start:None,interp_period:1.0/60.0,last_arrival:None});
    }
    host_present::spawn(&mut commands,&mut images);
    commands.insert_resource(Guests{items:guests,active:0});
}
fn update_frames(mut guests:ResMut<Guests>,mut images:ResMut<Assets<Image>>){
    for guest in &mut guests.items{
        if guest.gpu_publication.is_some() && !host_gpu_frame::cpu_view(){
            let controls=guest.gpu_publication.as_ref().unwrap();
            if let Some(frame)=controls.read(guest.serial){
                guest.width=frame.width;guest.height=frame.height;guest.serial=frame.serial;
                let now=std::time::Instant::now();
                if let Some(last)=guest.last_arrival{let dt=now.duration_since(last).as_secs_f64();if dt>0.004&&dt<0.1{guest.interp_period=guest.interp_period*0.9+dt.min(0.05)*0.1;}}
                guest.last_arrival=Some(now);
                if frame.interps.is_empty(){guest.interp_seq.clear();guest.gpu_image=Some(frame.texture);}
                else{guest.interp_seq=frame.interps;guest.interp_seq.push(frame.texture);guest.interp_start=Some(now);guest.gpu_image=Some(guest.interp_seq[0].clone());}
                if let Some(query)=guest.source_aspect{let mut aspect=0.0;if unsafe{query(&mut aspect)}{guest.aspect=Some(aspect);}}
            }
            if let(false,Some(start))=(guest.interp_seq.is_empty(),guest.interp_start){
                let slot=guest.interp_period/guest.interp_seq.len()as f64;
                let index=((start.elapsed().as_secs_f64()/slot)as usize).min(guest.interp_seq.len()-1);
                guest.gpu_image=Some(guest.interp_seq[index].clone());
            }
            continue;
        }
        let mut data=std::ptr::null();let(mut width,mut height,mut serial)=(0,0,0);
        if !unsafe{(guest.frame)(&mut data,&mut width,&mut height,&mut serial)}||serial==guest.serial{continue;}
        if data.is_null()||width==0||height==0||width>4096||height>4096{continue;}
        let bytes=unsafe{std::slice::from_raw_parts(data,width as usize*height as usize*4)}.to_vec();
        if let Some(mut image)=images.get_mut(&guest.image){
            if guest.width!=width||guest.height!=height||image.texture_descriptor.size.width==1{
                *image=Image::new(Extent3d{width,height,depth_or_array_layers:1},TextureDimension::D2,bytes,host_present::input_format(),RenderAssetUsages::default());
            }else{image.data=Some(bytes);}
        }
        if let Some(query)=guest.source_aspect{let mut aspect=0.0;if unsafe{query(&mut aspect)}{guest.aspect=Some(aspect);}}
        guest.width=width;guest.height=height;guest.serial=serial;
    }
}
// Host mashup adapter:partition physical client pixels without fractional overlap.
// A single guest gets the exact source Win32 client width/height.
fn physical_viewport(index:usize,count:usize,window:&Window)->(i32,i32,i32){
 let full=window.physical_width()as u64;let left=full*index as u64/count as u64;let right=full*(index+1)as u64/count as u64;
 (left as i32,(right-left)as i32,window.physical_height()as i32)
}
fn placement(index:usize,count:usize,window:&Window,width:u32,height:u32,source_aspect:Option<f64>)->(Vec2,Vec2){
 if let Some(aspect)=source_aspect{
  let(left,tile,height)=physical_viewport(index,count,window);
  // Source video.cpp image_point/present fitting order,then physical-to-Bevy UI units.
  let mut w=tile as f64;let mut h=w/aspect;if h>height as f64{h=height as f64;w=h*aspect;}
  let dpi=window.scale_factor()as f64;
  return(Vec2::new(((left as f64+tile as f64/2.0)/dpi)as f32,(height as f64/2.0/dpi)as f32),Vec2::new((w/dpi)as f32,(h/dpi)as f32));
 }
 // Legacy DLL path retains the accepted H6 frame-aspect placement.

    let tile=window.width()/count as f32;let scale=(tile/width as f32).min(window.height()/height as f32);
    let size=Vec2::new(width as f32*scale,height as f32*scale);
    let center=Vec2::new(tile*(index as f32+0.5),window.height()*0.5);(center,size)
}
fn layout(guests:Res<Guests>,windows:Query<&Window,With<PrimaryWindow>>,mut views:Query<(&GuestView,&mut Node)>){
    let Ok(window)=windows.single()else{return;};
    for (view,mut node)in &mut views{let g=&guests.items[view.0];let(center,size)=placement(view.0,guests.items.len(),window,g.width,g.height,g.aspect);let left=Val::Px(center.x-size.x/2.0);let top=Val::Px(center.y-size.y/2.0);let width=Val::Px(size.x);let height=Val::Px(size.y);if node.left!=left||node.top!=top||node.width!=width||node.height!=height{node.left=left;node.top=top;node.width=width;node.height=height;}}
}
// HOST POLICY: Claude LANES 2026-10-27 18:10; source single-guest behavior is unchanged.
// F6 selection persists until the pointer moves; close the old guest's menu BEFORE switching.
fn select_active(mut guests:ResMut<Guests>,keys:Res<ButtonInput<KeyCode>>,windows:Query<(Entity,&Window),With<PrimaryWindow>>,mut cursor:MessageReader<bevy::window::CursorMoved>){
 let Ok((entity,window))=windows.single()else{return;};
 let moved=cursor.read().filter(|event|event.window==entity).count()>0;
 let count=guests.items.len();let old=guests.active;let mut next=old;
 let cycle=(keys.just_pressed(KeyCode::F6)&&(count>1||guests.items[old].keyboard_wheel.is_none()))||(keys.just_pressed(KeyCode::Tab)&&guests.items[old].input_v2.is_none());
 if cycle{next=(old+1)%count;}else if moved{if let Some(point)=window.cursor_position(){for(i,g)in guests.items.iter().enumerate(){
  let(center,size)=placement(i,count,window,g.width,g.height,g.aspect);
  if point.x>=center.x-size.x/2.0&&point.x<center.x+size.x/2.0&&point.y>=center.y-size.y/2.0&&point.y<center.y+size.y/2.0{next=i;break;}
 }}}
 if next!=old{if let Some(options)=guests.items[old].options.as_mut(){options.close();}guests.active=next;}
}
fn send_input(guests:Res<Guests>,keys:Res<ButtonInput<KeyCode>>,mouse:Res<ButtonInput<MouseButton>>,windows:Query<(Entity,&Window),With<PrimaryWindow>>,time:Res<Time<Real>>,mut wheel:MessageReader<bevy::input::mouse::MouseWheel>,mut wheel_at:Local<[Option<std::time::Duration>;2]>){
    let Ok((window_entity,window))=windows.single()else{return;};
    let pointer=window.cursor_position();
    let mut buttons=0;
    // src: pristine engine/include/wp/input.h; engine/src/input/keymap.cpp kActions.
    // Only device mapping: the existing wiimote.cpp motion function interprets these bits.
    for (key,mask)in [
        (KeyCode::ArrowLeft,1),(KeyCode::KeyA,1),(KeyCode::ArrowRight,2),(KeyCode::KeyD,2),
        (KeyCode::ArrowDown,4),(KeyCode::KeyS,4),(KeyCode::ArrowUp,8),(KeyCode::KeyW,8),
        (KeyCode::Equal,0x10),(KeyCode::NumpadAdd,0x10),(KeyCode::Digit2,0x100),(KeyCode::Digit1,0x200),
        (KeyCode::Backspace,0x400),(KeyCode::Enter,0x800),(KeyCode::Space,0x800),
        (KeyCode::Minus,0x1000),(KeyCode::NumpadSubtract,0x1000),(KeyCode::Home,0x8000),(KeyCode::KeyH,0x8000),
        (KeyCode::ShiftLeft,0x10000),(KeyCode::KeyT,0x20000),(KeyCode::KeyG,0x40000),
        (KeyCode::KeyQ,0x80000),(KeyCode::KeyE,0x100000),(KeyCode::KeyR,0x200000),(KeyCode::KeyF,0x400000),
    ]{if keys.pressed(key){buttons|=mask;}}
    if mouse.pressed(MouseButton::Left){buttons|=0x800;}if mouse.pressed(MouseButton::Right){buttons|=0x400;}
    if mouse.pressed(MouseButton::Middle){buttons|=0x10000;}
    // src: input.cpp note_wheel/pressed: each sign remains held for strictly less than120ms.
    let now=time.elapsed();
    for event in wheel.read(){if event.window==window_entity&&!guests.items[guests.active].options.as_ref().is_some_and(|v|v.is_open()){
        let sign=if event.y>0.0{wheel_at[0]=Some(now);1}else if event.y<0.0{wheel_at[1]=Some(now);-1}else{0};
        if sign!=0{if let Some(note)=guests.items[guests.active].keyboard_wheel{unsafe{note(sign);}}}
    }}
    for (i,mask) in [(0,0x20000),(1,0x40000)]{if wheel_at[i].is_some_and(|last|now.saturating_sub(last)<std::time::Duration::from_millis(120)){buttons|=mask;}}
    for(i,g)in guests.items.iter().enumerate(){let(c,s)=placement(i,guests.items.len(),window,g.width,g.height,g.aspect);let mut valid=false;let(mut x,mut y)=(0.0,0.0);if i==guests.active&&window.focused{
        if let Some(point)=g.source_point{
            if let Some(cursor)=window.physical_cursor_position(){let(left,width,height)=physical_viewport(i,guests.items.len(),window);valid=unsafe{point((cursor.x as i32).wrapping_sub(left),cursor.y as i32,width,height,&mut x,&mut y)};}
        }else if let Some(cursor)=pointer{x=(cursor.x-c.x)*2.0/s.x;y=(cursor.y-c.y)*2.0/s.y;valid=x.abs()<=1.0&&y.abs()<=1.0;}
    }
        let focused=i==guests.active&&window.focused;
        unsafe{if let Some(input)=g.input_v2{input(0,if focused{buttons}else{0},valid,x,y,focused,focused&&keys.pressed(KeyCode::Tab));}else{(g.input)(0,if focused{buttons}else{0},valid,x,y);}}
    }
}


// Source configuration handshake;no controller/device initialization from guessed defaults.
fn configure_controllers(guests:Res<Guests>,mut error:Local<Option<String>>){
 let contexts:Vec<_>=guests.items.iter().filter_map(|v|v._controller.clone()).collect();
 let sdl=std::env::var_os("WII_PARTY_SDL3").map(PathBuf::from);
 if let Err(message)=unsafe{host_controller::configure(&contexts,sdl.as_deref())}{
  if error.as_ref()!=Some(&message){eprintln!("{message}");*error=Some(message);}
 }else{*error=None;}
}
