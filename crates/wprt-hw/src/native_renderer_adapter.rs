// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted whole owned cpp/patched/gx_render_g9_rust.cpp adapter policy port.
// Accepted renderer/backend unchanged;native SDK retains original guard,Image/vector lifetime and filesystem conversion.
use std::{ffi::{c_char,c_void},sync::atomic::{AtomicU64,Ordering},mem::MaybeUninit};
#[repr(C)]pub struct Filter{pub coefficients:[u32;3],pub gamma:f32,pub clamp_top:bool,pub clamp_bottom:bool}
#[repr(C)]pub struct Rect{pub x:i32,pub y:i32,pub width:i32,pub height:i32}
#[repr(C)]pub struct ClientRect{pub left:i32,pub top:i32,pub right:i32,pub bottom:i32}
#[repr(C)]pub struct OverlayInfo{pub pixels:*const u32,pub width:u32,pub height:u32,pub empty:bool}
#[repr(C)]struct Description{title:*const c_char,short_name:*const c_char,data_directory:*const c_char,nand_directory:*const c_char,settings_file:*const c_char,textures_directory:*const c_char,notice_module:*const c_char,sideways_modules:*const *const c_char,sideways_module_count:usize,instruction_module:*const c_char,minigame_prefix:*const c_char}
extern "C"{fn wphw_gr_ensure_sdk();fn wphw_gr_overlay_sdk(version:*mut u64,image:*mut c_void,visible:*mut bool,info:*mut OverlayInfo)->bool;
fn wpgx_api_name()->*const c_char;fn wpgx_guest_range_valid(address:u32,size:usize)->bool;fn wpgx_take_statistics(batches:*mut u32,vertices:*mut u32,seconds:*mut f64);fn wpgx_take_copy_statistics(writebacks:*mut u32,seconds:*mut f64);fn wpgx_draw(vertices:*const c_void,count:u32);fn wpgx_finish_copies();fn wpgx_copy_to_framebuffer(x:i32,y:i32,width:i32,height:i32,depth:bool,filter:*const Filter);fn wpgx_copy_to_texture(address:u32,stride:u32,x:i32,y:i32,width:i32,height:i32,half:bool,format:u32,intensity:bool,depth:bool,alpha:bool,filter:*const Filter);fn wpgx_invalidate_textures();fn wpgx_load_tlut(address:u32,tmem:u32,bytes:u32);fn wpgx_clear(x:i32,y:i32,width:i32,height:i32);fn wpgx_dump_copies(prefix:*const c_char);fn wpgx_present_frame(window:isize,cw:i32,ch:i32,aspect:f64,pixels:*const u32,width:u32,height:u32,visible:bool,placed:*const i32)->bool;
#[link_name="_ZN2wp2gx12bp_registersEv"]fn bp()->*const u32;#[link_name="_ZN2wp2gx15konst_registersEv"]fn konst()->*const u32;#[link_name="_ZN2wp2gx12xf_registersEv"]fn xf()->*const u32;
#[link_name="_ZN2wp4game11descriptionEv"]fn description()->*const Description;
#[link_name="_ZN2wp7options5placeEiiii"]fn place(cw:i32,ch:i32,width:i32,height:i32)->Rect;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn wprt_settings_revision()->u32;fn wprt_settings_number(key:*const c_char,variable:*const c_char)->i32;
#[link_name="_ZN2wp8settings4flagEPKcS2_"]fn settings_flag(key:*const c_char,variable:*const c_char)->bool;
fn __mingw_snprintf(buffer:*mut c_char,capacity:usize,format:*const c_char,...)->i32;}
extern "system"{fn GetClientRect(window:*mut c_void,rect:*mut ClientRect)->i32;}
// port of source requested_scale static LiveNumber,original source acquire/release cache and clamp1..6.
static SCALE_CACHE:AtomicU64=AtomicU64::new(0);
#[no_mangle]pub unsafe extern "C" fn wphw_gr_requested_scale()->i32{let revision=wprt_settings_revision()as u64;let mut cached=SCALE_CACHE.load(Ordering::Acquire);if cached>>32!=revision{cached=(revision<<32)|(wprt_settings_number(c"video.scale".as_ptr(),c"WP_SCALE".as_ptr())as u32 as u64);SCALE_CACHE.store(cached,Ordering::Release);}(cached as u32 as i32).clamp(1,6)}
#[no_mangle]pub unsafe extern "C" fn wphw_gr_log_video(text:*const c_char){log_write(c"video".as_ptr(),c"%s".as_ptr(),text);}
#[no_mangle]pub unsafe extern "C" fn wphw_gr_bp()->*const u32{bp()}
#[no_mangle]pub unsafe extern "C" fn wphw_gr_konst()->*const u32{konst()}
#[no_mangle]pub unsafe extern "C" fn wphw_gr_xf()->*const u32{xf()}
// port of pack_prepare:format even whennull/empty;settings queried only for nonempty source directory.
#[no_mangle]pub unsafe extern "C" fn wphw_gr_pack_prepare(directory:*mut c_char,capacity:usize)->u32{let textures=(*description()).textures_directory;__mingw_snprintf(directory,capacity,c"%s".as_ptr(),if textures.is_null(){c"".as_ptr()}else{textures});if textures.is_null()||*textures==0{return 0;}let dump=settings_flag(c"video.dump_textures".as_ptr(),c"WP_DUMP_TEXTURES".as_ptr());let custom=settings_flag(c"video.custom_textures".as_ptr(),c"WP_CUSTOM_TEXTURES".as_ptr());(if dump{1}else{0})|(if custom{2}else{0})}
// port of obsolete source pack decode slot,retained false;actualacceptedPNGdecode is Rust elsewhere.
#[no_mangle]pub extern "C" fn wphw_gr_pack_decode_png(_path:*const c_char,_out:Option<unsafe extern "C" fn(*mut c_void,*const u32,u32,u32)>,_context:*mut c_void)->bool{false}
// port of convert(CopyFilter):copy namedpublicfields only,never inspect padding.
unsafe fn convert(filter:*const Filter)->Filter{let f=&*filter;Filter{coefficients:f.coefficients,gamma:f.gamma,clamp_top:f.clamp_top,clamp_bottom:f.clamp_bottom}}
#[export_name="_ZN2wp2gx6render16install_rendererEv"]pub unsafe extern "C" fn install(){wphw_gr_ensure_sdk();}
#[export_name="_ZN2wp2gx6render17guest_range_validEjy"]pub unsafe extern "C" fn range(address:u32,size:usize)->bool{wpgx_guest_range_valid(address,size)}
#[export_name="_ZN2wp2gx6render15take_statisticsERjS2_Rd"]pub unsafe extern "C" fn statistics(batches:*mut u32,vertices:*mut u32,seconds:*mut f64){wphw_gr_ensure_sdk();wpgx_take_statistics(batches,vertices,seconds);}
#[export_name="_ZN2wp2gx6render20take_copy_statisticsERjRd"]pub unsafe extern "C" fn copy_statistics(writebacks:*mut u32,seconds:*mut f64){wphw_gr_ensure_sdk();wpgx_take_copy_statistics(writebacks,seconds);}
#[export_name="_ZN2wp2gx6render8api_nameEv"]pub unsafe extern "C" fn api_name()->*const c_char{wpgx_api_name()}
#[export_name="_ZN2wp2gx6render4drawEPKNS0_12ScreenVertexEj"]pub unsafe extern "C" fn draw(vertices:*const c_void,count:u32){wphw_gr_ensure_sdk();wpgx_draw(vertices,count);}
#[export_name="_ZN2wp2gx6render13finish_copiesEv"]pub unsafe extern "C" fn finish(){wphw_gr_ensure_sdk();wpgx_finish_copies();}
#[export_name="_ZN2wp2gx6render19copy_to_framebufferEiiiibRKNS1_10CopyFilterE"]pub unsafe extern "C" fn framebuffer(x:i32,y:i32,width:i32,height:i32,depth:bool,filter:*const Filter){wphw_gr_ensure_sdk();let f=convert(filter);wpgx_copy_to_framebuffer(x,y,width,height,depth,&f);}
// port of source static presentation overlay metadata;no invented mutex or frame fallback.
static mut OVERLAY_VERSION:u64=0;static mut OVERLAY_VISIBLE:bool=false;static mut OVERLAY_WIDTH:u32=0;static mut OVERLAY_HEIGHT:u32=0;
// Native factory owns default Image lifetime across thisRust policy call. Source ignores GetClientRect result;
// valid-window contract required exactly as source (undefineduninitialized RECT on API failure,notinventedzero).
#[no_mangle]pub unsafe extern "C" fn wprt_gr_present(window:*mut c_void,aspect:f64,image:*mut c_void)->bool{if window.is_null(){return false;}wphw_gr_ensure_sdk();let mut client=MaybeUninit::<ClientRect>::uninit();GetClientRect(window,client.as_mut_ptr());let client=client.assume_init();let mut info=OverlayInfo{pixels:std::ptr::null(),width:0,height:0,empty:true};let changed=wphw_gr_overlay_sdk(std::ptr::addr_of_mut!(OVERLAY_VERSION),image,std::ptr::addr_of_mut!(OVERLAY_VISIBLE),&mut info);let mut pixels=std::ptr::null();if changed&&!info.empty{pixels=info.pixels;OVERLAY_WIDTH=info.width;OVERLAY_HEIGHT=info.height;}let mut placed=Rect{x:0,y:0,width:0,height:0};if OVERLAY_WIDTH!=0&&OVERLAY_HEIGHT!=0{placed=place(client.right,client.bottom,OVERLAY_WIDTH as i32,OVERLAY_HEIGHT as i32);}let rect=[placed.x,placed.y,placed.width,placed.height];wpgx_present_frame(window as isize,client.right,client.bottom,aspect,pixels,info.width,info.height,OVERLAY_VISIBLE,rect.as_ptr())}
#[export_name="_ZN2wp2gx6render11dump_copiesEPKc"]pub unsafe extern "C" fn dump(prefix:*const c_char){wphw_gr_ensure_sdk();wpgx_dump_copies(prefix);}
#[export_name="_ZN2wp2gx6render15copy_to_textureEjjiiiibjbbbRKNS1_10CopyFilterE"]pub unsafe extern "C" fn texture(address:u32,stride:u32,x:i32,y:i32,width:i32,height:i32,half:bool,format:u32,intensity:bool,depth:bool,alpha:bool,filter:*const Filter){wphw_gr_ensure_sdk();let f=convert(filter);wpgx_copy_to_texture(address,stride,x,y,width,height,half,format,intensity,depth,alpha,&f);}
#[export_name="_ZN2wp2gx6render19invalidate_texturesEv"]pub unsafe extern "C" fn invalidate(){wphw_gr_ensure_sdk();wpgx_invalidate_textures();}
#[export_name="_ZN2wp2gx6render9load_tlutEjjj"]pub unsafe extern "C" fn load_tlut(address:u32,tmem:u32,bytes:u32){wphw_gr_ensure_sdk();wpgx_load_tlut(address,tmem,bytes);}
#[export_name="_ZN2wp2gx6render5clearEiiii"]pub unsafe extern "C" fn clear(x:i32,y:i32,width:i32,height:i32){wphw_gr_ensure_sdk();wpgx_clear(x,y,width,height);}
