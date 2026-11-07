#![allow(unused_parens,arithmetic_overflow)]
// The translated game code (generated locally from your own disc) is linked here.
// AI-assisted H4 port of pristine engine/src/app/main.cpp run(); Rust guest/core plus C++ hardware.
// G8 supplied Rust renderer C ABI, linked into this private game DLL.
use wprt_gx as _;
// H7 supplied PNG entry points;game owns changing native archive link inputs.
use wprt_hw as _;
// PRIVATE generated AX microcode Rust driver;SDK instruction fallback remains native.
#[cfg(feature="rust-dsp-ax")]use wprt_ax_driver as _;
// H7 native-path-preserving save flow;original adopt_root_files leaf remains C++.
#[cfg(feature="rust-save-backup-flow")]use wprt_save_backup as _;
use std::ffi::{c_char,c_void,CStr,CString};
#[cfg(feature="host-options")]mod host_options_transport;
use wprt::{cpu::Cpu,dispatch,ffi,modules::{self,ModuleDescriptor},runtime,threads};
#[cfg(feature="all-modules")]pub use wpgame_all::dol;
#[cfg(feature="all-modules")]include!(concat!(env!("WP_GENERATED_DIR"), "/all_modules.rs"));
#[cfg(feature="host")]
extern "C" {
    fn wphost_frame(data:*mut *const u8,width:*mut u32,height:*mut u32,serial:*mut u64)->bool;
    fn wphost_input(channel:u32,buttons:u32,pointer:bool,x:f32,y:f32);
}
// H6 transport adapter only. Returned bytes remain valid until this host thread's next wpg_frame call.
#[cfg(feature="host")]
#[no_mangle]pub unsafe extern "C" fn wpg_frame(data:*mut *const u8,width:*mut u32,height:*mut u32,serial:*mut u64)->bool{wphost_frame(data,width,height,serial)}
// H6 sends the source input::Sample button/pointer fields, with no game-rule changes.
#[cfg(feature="host")]
#[no_mangle]pub unsafe extern "C" fn wpg_input(channel:u32,buttons:u32,pointer:bool,x:f32,y:f32){wphost_input(channel,buttons,pointer,x,y);}
extern "C"{
    fn wpg_hw_game_description()->*const c_void;fn wpg_hw_settings_file()->*const c_char;
    fn wpg_hw_settings_load(path:*const c_char);fn wpg_hw_nand_mount(path:*const c_char)->bool;
    fn wpg_hw_exi_mount(nand:*const c_char);fn wpg_hw_disc_mount(path:*const c_char)->bool;
    fn wpg_hw_video_start();fn wpg_hw_ipc_report();
}
// port of the original DOL registration tables; module package supplies identical typed function/resume slices.
fn register_guest_dol(){for &(address,function)in dol::FUNCTIONS{runtime::register(address,function);}for &(address,function)in dol::RESUMES{runtime::register_resume(address,function);}}
// H4 ABI revision is a host protocol version, not a guest value.
#[no_mangle]pub extern "C" fn wpg_abi_version()->u32{1}
// port of pristine app/main.cpp run: helper for native std::string path concatenation bytes.
fn suffix(path:&CStr,suffix:&[u8])->CString{let mut text=path.to_bytes().to_vec();text.extend_from_slice(suffix);CString::new(text).unwrap()}
// port of engine/src/app/main.cpp run (non-WP_TRACE watch/profile functions are native no-ops).
unsafe fn run(extracted:&CStr,nand:&CStr,timeout:i32)->i32{
    #[cfg(feature="host")] wprt::host::bind_thread();
    ffi::wphw_bind_game(wpg_hw_game_description());wpg_hw_settings_load(wpg_hw_settings_file());
    #[cfg(feature="host-controller")]CONTROLLER_SETTINGS_READY.store(true,std::sync::atomic::Ordering::Release);
    wprt::memory::init();ffi::bind_memory();let mut entry=0;
    if !wprt::dol::load_dol(&suffix(extracted,b"/sys/main.dol"),&mut entry){eprintln!("cannot load {}/sys/main.dol",extracted.to_string_lossy());return 1;}
    if !wprt::boot::init_boot_memory(extracted){eprintln!("cannot read boot files in {}/sys",extracted.to_string_lossy());return 1;}
    if !wpg_hw_nand_mount(nand.as_ptr()){eprintln!("cannot prepare the virtual NAND in {}",nand.to_string_lossy());return 1;}
    wpg_hw_exi_mount(nand.as_ptr());if !wpg_hw_disc_mount(extracted.as_ptr()){eprintln!("cannot read the disc layout in {}",extracted.to_string_lossy());return 1;}
    if timeout>0{std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_secs(timeout as u64));eprintln!("no progress after {timeout} seconds");dispatch::print_call_stack();unsafe{wpg_hw_ipc_report();libc::_exit(3);}});}
    register_guest_dol();dispatch::set_names(dol::NAMES);modules::set_descriptors(&MODULES);wprt::hle::register();
    wpg_hw_video_start();
    #[cfg(feature="host-window-mode")] WINDOW_SETTINGS_READY.store(true,std::sync::atomic::Ordering::Release);
    let mut cpu=Cpu::default();threads::init_threads(&mut cpu);runtime::call(&mut cpu,entry);println!("entry returned");0
}
// H4 requires the guest on a 512 MiB Rust-owned stack; token unwinds stay entirely on Rust frames.
#[no_mangle]pub unsafe extern "C" fn wpg_run(data_dir:*const c_char,nand_dir:*const c_char,timeout_s:i32)->i32{
    let data=CStr::from_ptr(data_dir).to_owned();let nand=CStr::from_ptr(nand_dir).to_owned();
    match std::thread::Builder::new().stack_size(512<<20).spawn(move||run(&data,&nand,timeout_s)){
        Ok(thread)=>match thread.join(){Ok(result)=>result,Err(_)=>std::process::abort()},Err(_)=>1,
    }
}

// H6 host adapters. RAM operations are executed on the guest thread at WP_POLL, never from a UI thread.
#[cfg(feature="host")]
#[no_mangle]pub unsafe extern "C" fn wpg_read_u32(address:u32,value:*mut u32)->bool{if value.is_null(){return false;}match wprt::host::read(address){Some(v)=>{*value=v;true},None=>false}}
#[cfg(feature="host")]
#[no_mangle]pub extern "C" fn wpg_write_u32(address:u32,value:u32)->bool{wprt::host::write(address,value)}
// Callback must not unwind, block, or outlive its owning library. None removes a dispatch replacement.
#[cfg(feature="host")]
#[no_mangle]pub extern "C" fn wpg_hook(address:u32,callback:Option<wprt::host::Callback>){wprt::host::hook(address,callback);}

#[cfg(feature="rust-game-description")]mod native_game_description;

#[cfg(feature="rust-dsp-rom-registry")]mod native_dsp_rom_registry;

// H8 optional audio transport, configured before wpg_run. Legacy hosts retain native output.
#[cfg(all(feature="host",feature="bevy-audio-host"))]
#[no_mangle]pub extern "C" fn wpg_audio_bind(rate:u32)->bool{wprt::native_audio::bind_host_output(rate)}
#[cfg(all(feature="host",feature="bevy-audio-host"))]
#[no_mangle]pub unsafe extern "C" fn wpg_audio_read(output:*mut f32,frames:u32)->bool{wprt::native_audio::read_host_output(output,frames)}

// H8 explicit host snapshot. Grip/orientation policy remains on the guest thread.
#[cfg(feature="host-input-grip")]
#[no_mangle]pub unsafe extern "C" fn wpg_input_v2(channel:u32,buttons:u32,pointer:bool,x:f32,y:f32,focused:bool,grip:bool){extern "C"{fn wphost_input_v2(channel:u32,buttons:u32,pointer:bool,x:f32,y:f32,focused:bool,grip:bool);}wphost_input_v2(channel,buttons,pointer,x,y,focused,grip);}

// H8 primitive renderer callbacks only; binding happens before the guest worker starts.
#[cfg(feature="host-gpu-bridge")]
#[no_mangle]pub unsafe extern "C" fn wpg_renderer_bind(table:*const wprt_gx::remote::RemoteRenderer){unsafe{wprt_gx::remote::wpgx_bind_remote(table)}}

#[cfg(feature="host-controller")]
static CONTROLLER_SETTINGS_READY:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
// Host callbacks must be bound before guest execution;only public C primitives cross DLL.
#[cfg(feature="host-controller")]
#[no_mangle]pub unsafe extern "C" fn wpg_controller_bind(value:*const wprt_hw::native_host_gamepad::Collector){wprt_hw::native_host_gamepad::wphost_controller_bind(value)}
// Source flag reads ONLY after source settings_load,not guessed preboot defaults.
#[cfg(feature="host-controller")]
#[no_mangle]pub unsafe extern "C" fn wpg_controller_settings(enabled:*mut bool,pairs:*mut bool,wiimotes:*mut bool)->bool{
 if !CONTROLLER_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 *enabled=wprt_hw::native_gamepad_adapter::wphw_gamepad_enabled();
 *pairs=wprt_hw::native_gamepad_adapter::wphw_gamepad_setting(0);
 *wiimotes=wprt_hw::native_gamepad_adapter::wphw_gamepad_setting(1);true
}

// H8 scalar event transport; source binding/action/wheel/focus policies remain in the DLL.
#[cfg(feature="host-source-keyboard")]
#[no_mangle]pub extern "C" fn wpg_keyboard_events_bind(query:Option<wprt_hw::native_host_keyboard::EventQuery>)->bool{
 query.map(wprt_hw::native_host_keyboard::bind_events).unwrap_or(false)
}
// Reuse the accepted original Win32 MapVirtualKeyA(MAPVK_VSC_TO_VK_EX) service; no invented VK table.
#[cfg(feature="host-source-keyboard")]
#[no_mangle]pub unsafe extern "C" fn wpg_keyboard_scan_code(scan:u32)->i32{wprt_hw::native_video_win32::wprt_vws_map_shift(scan)}
// Source custom game action/grip bindings use the existing accepted native VK provider.
// Retained SDK service,not pure-Bevy keyboard completion. No game rules in host.
#[cfg(feature="host-source-keyboard")]
#[no_mangle]pub extern "C" fn wpg_keyboard_bind(){wprt_hw::native_host_keyboard::wphost_keyboard_bind();}
#[cfg(feature="host-source-keyboard")]
#[no_mangle]pub unsafe extern "C" fn wpg_keyboard_wheel(delta:i32){wprt_hw::native_host_keyboard::wphost_keyboard_wheel(delta);}

#[cfg(feature="host-source-geometry")]
#[no_mangle]pub unsafe extern "C" fn wpg_source_aspect(out:*mut f64)->bool{extern "C"{fn wphost_source_aspect(out:*mut f64)->bool;}wphost_source_aspect(out)}
#[cfg(feature="host-source-geometry")]
#[no_mangle]pub unsafe extern "C" fn wpg_source_image_point(x:i32,y:i32,w:i32,h:i32,out_x:*mut f32,out_y:*mut f32)->bool{extern "C"{fn wphost_source_image_point(x:i32,y:i32,w:i32,h:i32,out_x:*mut f32,out_y:*mut f32)->bool;}wphost_source_image_point(x,y,w,h,out_x,out_y)}

// Source window_thread initial video.fullscreen flag, read only after source video startup.
#[cfg(feature="host-window-mode")]
static WINDOW_SETTINGS_READY:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
#[cfg(feature="host-window-mode")]
#[no_mangle]pub unsafe extern "C" fn wpg_window_fullscreen(out:*mut bool)->bool{
 if !WINDOW_SETTINGS_READY.load(std::sync::atomic::Ordering::Acquire){return false;}
 *out=wprt::native_settings::wprt_settings_flag(b"video.fullscreen\0".as_ptr().cast(),b"WP_FULLSCREEN\0".as_ptr().cast());true
}
// port of video.cpp F11 settings::store; native window SDK state is not reused by Bevy.
#[cfg(feature="host-window-mode")]
#[no_mangle]pub unsafe extern "C" fn wpg_window_store_fullscreen(enabled:bool){
 wprt::native_settings::wprt_settings_store(b"video.fullscreen\0".as_ptr().cast(),if enabled{b"1".as_ptr()}else{b"0".as_ptr()},1);
}

// H8 optional host-owned immutable GPU publication; only primitive callbacks cross C ABI.
#[cfg(feature="host-source-geometry")]
#[no_mangle]pub unsafe extern "C" fn wpg_gpu_publication_bind(context:usize,capture:wprt_hw::native_video_transport::GpuCapture){
 wprt_hw::native_video_transport::bind_gpu_publication(context,capture);
}
#[cfg(feature="host-source-geometry")]
#[no_mangle]pub unsafe extern "C" fn wpg_gpu_publication_visit(context:usize,visit:wprt_hw::native_video_transport::GpuVisit)->bool{
 wprt_hw::native_video_transport::visit_gpu_publication(context,visit)
}

#[cfg(feature="host-source-geometry")]
#[no_mangle]pub unsafe extern "C" fn wpg_gpu_publication_cpu_bind(context:usize,read:wprt_hw::native_video_transport::GpuRead){
 wprt_hw::native_video_transport::bind_gpu_cpu_reader(context,read);
}
