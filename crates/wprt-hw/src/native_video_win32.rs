// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine platform/video.cpp native Win32 window functions.
// UNINTEGRATED. Official pinned windows-sys0.61.2 SDK types/constants,no fitted values.
use std::{ffi::c_void,mem::{size_of,zeroed,MaybeUninit},ptr::{null,null_mut}};
use windows_sys::Win32::{Foundation::{HWND,POINT,RECT},Graphics::{Dwm::DwmSetWindowAttribute,Gdi as gdi},System::{LibraryLoader::GetModuleHandleA,Registry as reg,SystemInformation::GetLocalTime},UI::{WindowsAndMessaging as wm,Input::KeyboardAndMouse::GetAsyncKeyState}};
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C"{
 fn wphw_video_window()->usize;
 fn wprt_video_aspect_ptr()->*mut f64;
 fn wprt_video_image_point(x:i32,y:i32,w:i32,h:i32,out_x:*mut f32,out_y:*mut f32)->bool;
 fn wprt_video_window_proc(w:usize,m:u32,wp:usize,lp:isize)->isize;
 fn wprt_settings_flag(key:*const u8,variable:*const u8)->bool;
 fn wphw_vws_store_window(w:usize);
 fn wphw_vc_set_fullscreen(w:usize,enable:bool);
}
// port of video.cpp apply_title_bar_theme:read existing per-user theme,modify only own window attribute.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_theme(w:usize){
 let mut light=1u32;let mut size=size_of::<u32>() as u32;
 reg::RegGetValueA(reg::HKEY_CURRENT_USER,b"Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0".as_ptr(),b"AppsUseLightTheme\0".as_ptr(),reg::RRF_RT_REG_DWORD,null_mut(),(&mut light as *mut u32).cast(),&mut size);
 let dark=(light==0) as i32;DwmSetWindowAttribute(w as HWND,20,(&dark as *const i32).cast(),size_of::<i32>() as u32);
}
// port of video.cpp set_fullscreen. Native SDK records/scalar members passed explicitly,not native C++ class layouts.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_fullscreen(w:usize,enable:bool,placement:*mut wm::WINDOWPLACEMENT,fullscreen:*mut bool){
 let window=w as HWND;let style=wm::GetWindowLongA(window,wm::GWL_STYLE);
 if enable{
  let mut monitor:gdi::MONITORINFO=zeroed();monitor.cbSize=size_of::<gdi::MONITORINFO>() as u32;(*placement).length=size_of::<wm::WINDOWPLACEMENT>() as u32;
  if wm::GetWindowPlacement(window,placement)==0 || gdi::GetMonitorInfoA(gdi::MonitorFromWindow(window,gdi::MONITOR_DEFAULTTONEAREST),&mut monitor)==0{return}
  wm::SetWindowLongA(window,wm::GWL_STYLE,((style as u32)&!wm::WS_OVERLAPPEDWINDOW) as i32);
  wm::SetWindowPos(window,wm::HWND_TOP,monitor.rcMonitor.left,monitor.rcMonitor.top,monitor.rcMonitor.right-monitor.rcMonitor.left,monitor.rcMonitor.bottom-monitor.rcMonitor.top,wm::SWP_NOOWNERZORDER|wm::SWP_FRAMECHANGED);
 }else{
  wm::SetWindowLongA(window,wm::GWL_STYLE,((style as u32)|wm::WS_OVERLAPPEDWINDOW) as i32);wm::SetWindowPlacement(window,placement);
  wm::SetWindowPos(window,null_mut(),0,0,0,0,wm::SWP_NOMOVE|wm::SWP_NOSIZE|wm::SWP_NOZORDER|wm::SWP_NOOWNERZORDER|wm::SWP_FRAMECHANGED);
 }*fullscreen=enable;
}
// Adapter of original native CALLBACK window_proc into accepted Rust message decisions.
unsafe extern "system" fn window_proc(w:HWND,m:u32,wp:usize,lp:isize)->isize{wprt_video_window_proc(w as usize,m,wp,lp)}
// port of video.cpp window_thread. Preserve API result handling,source1280/320/16/+0.5 sizing and1/16 timer.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_window_thread(){
 let mut class:wm::WNDCLASSA=zeroed();class.lpfnWndProc=Some(window_proc);class.hInstance=GetModuleHandleA(null());class.hCursor=wm::LoadCursorA(null_mut(),wm::IDC_ARROW.cast());class.hIcon=wm::LoadIconA(class.hInstance,1usize as *const u8);class.lpszClassName=b"WiiRecompWindow\0".as_ptr();wm::RegisterClassA(&class);
 let Ok(mut title)=window_title(0.0)else{return};title.push(0);
 let mut work=MaybeUninit::<RECT>::uninit();wm::SystemParametersInfoA(wm::SPI_GETWORKAREA,0,work.as_mut_ptr().cast(),0);let work=work.assume_init();
 let mut client_width=1280;let mut client_height=(client_width as f64 / *wprt_video_aspect_ptr()+0.5) as i32;
 let mut rect=RECT{left:0,top:0,right:client_width,bottom:client_height};wm::AdjustWindowRect(&mut rect,wm::WS_OVERLAPPEDWINDOW,0);
 while (rect.right-rect.left>work.right-work.left || rect.bottom-rect.top>work.bottom-work.top) && client_width>320{
  client_width-=16;client_height=(client_width as f64 / *wprt_video_aspect_ptr()+0.5) as i32;rect=RECT{left:0,top:0,right:client_width,bottom:client_height};wm::AdjustWindowRect(&mut rect,wm::WS_OVERLAPPEDWINDOW,0);
 }
 let window=wm::CreateWindowExA(0,b"WiiRecompWindow\0".as_ptr(),title.as_ptr(),wm::WS_OVERLAPPEDWINDOW|wm::WS_VISIBLE,wm::CW_USEDEFAULT,wm::CW_USEDEFAULT,rect.right-rect.left,rect.bottom-rect.top,null_mut(),null_mut(),class.hInstance,null());
 wphw_vws_store_window(window as usize);wprt_vws_theme(window as usize);
 if wprt_settings_flag(b"video.fullscreen\0".as_ptr(),b"WP_FULLSCREEN\0".as_ptr()){wphw_vc_set_fullscreen(window as usize,true)}
 wm::SetTimer(window,1,16,None);let mut message=MaybeUninit::<wm::MSG>::uninit();while wm::GetMessageA(message.as_mut_ptr(),null_mut(),0,0)>0{wm::TranslateMessage(message.as_ptr());wm::DispatchMessageA(message.as_ptr());}
}
// port of video.cpp window_focused.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_focused()->bool{let window=wphw_video_window() as HWND;!window.is_null()&&wm::GetForegroundWindow()==window}
// port of video.cpp client_size;failed query leaves caller outputs unchanged.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_client_size(width:*mut i32,height:*mut i32)->bool{let window=wphw_video_window() as HWND;let mut client:RECT=zeroed();if window.is_null()||wm::GetClientRect(window,&mut client)==0{return false}*width=client.right;*height=client.bottom;true}
// port of video.cpp key_down:source SHORT promoted to int before masking bit15.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_key_down(key:i32)->bool{(GetAsyncKeyState(key) as u16 & 0x8000)!=0}
// port of video.cpp cursor_position;failed call leaves outputs unchanged.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_cursor(x:*mut i32,y:*mut i32)->bool{let mut p=MaybeUninit::<POINT>::uninit();if wm::GetCursorPos(p.as_mut_ptr())==0{return false}let p=p.assume_init();*x=p.x;*y=p.y;true}
// port of video.cpp cursor_on_image,including the original short-circuit call order.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_cursor_image(x:*mut f32,y:*mut f32)->bool{let window=wphw_video_window() as HWND;let mut p=MaybeUninit::<POINT>::uninit();let mut client=MaybeUninit::<RECT>::uninit();if window.is_null()||wm::GetCursorPos(p.as_mut_ptr())==0||gdi::ScreenToClient(window,p.as_mut_ptr())==0||wm::GetClientRect(window,client.as_mut_ptr())==0{return false}let p=p.assume_init();let client=client.assume_init();wprt_video_image_point(p.x,p.y,client.right,client.bottom,x,y)}
// port of video.cpp local_time:only the source's six fields;native tm wrapper keeps all remaining fields zero.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_local_time(out:*mut i32){let mut now=MaybeUninit::uninit();GetLocalTime(now.as_mut_ptr());let now=now.assume_init();let fields=[now.wYear as i32-1900,now.wMonth as i32-1,now.wDay as i32,now.wHour as i32,now.wMinute as i32,now.wSecond as i32];std::ptr::copy_nonoverlapping(fields.as_ptr(),out,fields.len())}
// port of native SDK leaf in window_proc WM_PAINT.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_paint(w:usize){let mut paint=MaybeUninit::<gdi::PAINTSTRUCT>::uninit();gdi::BeginPaint(w as HWND,paint.as_mut_ptr());gdi::EndPaint(w as HWND,paint.as_ptr());}
// port of native SDK leaf in side_key.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_map_shift(scan:u32)->i32{windows_sys::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyA(scan,windows_sys::Win32::UI::Input::KeyboardAndMouse::MAPVK_VSC_TO_VK_EX) as i32}
// port of native SDK leaf in window_proc WM_SETCURSOR.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_null_cursor(){wm::SetCursor(null_mut());}
// port of native SDK leaf in window_proc default.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_default(w:usize,m:u32,wp:usize,lp:isize)->isize{wm::DefWindowProcA(w as HWND,m,wp,lp)}

// port of video.cpp game_code/window_title. Plain records expose named SDK fields,not C++ object layouts.
#[repr(C)]pub struct TitleInfo{
 pub fps_label:*const u8,pub dsp_label:*const u8,pub dsp_native:*const u8,pub dsp_interpreted:*const u8,pub dsp_stopped:*const u8,
 pub status_label:*const u8,pub format:*const u8,pub title:*const u8,pub api:*const u8,
 pub fps:f64,pub load_percent:f64,pub loaded:bool,pub is_native:bool,
}
extern "C"{fn wphw_vws_title_op(op:i32,info:*mut TitleInfo,buffer:*mut u8)->i32;fn wphw_vws_read8(address:u32)->u8;fn wphw_vws_read16(address:u32)->u16;}
// Adapter of original std::string(const char*)/operator+ byte semantics;no UTF8 normalization.
unsafe fn title_cbytes<'a>(p:*const u8)->&'a[u8]{std::ffi::CStr::from_ptr(p.cast()).to_bytes()}
// port of video.cpp window_title;SDK callback2/3 uses ORIGINAL std::snprintf,including MinGW formatting.
pub unsafe fn window_title(fps:f64)->Result<Vec<u8>,()>{
 let mut info:TitleInfo=zeroed();info.fps=fps;
 if wphw_vws_title_op(0,&mut info,null_mut())<0{return Err(())}
 let pal=(wphw_vws_read16(0xCC002002)&0x0300)==0x0100;
 let mut code=Vec::new();for i in 0..6{let ch=wphw_vws_read8(0x80000000+i);if ch!=0{code.push(ch)}}
 let pal_disc=code.len()>=4&&code[3]==b'P';let region:&[u8]=if pal{b"PAL".as_slice()}else if pal_disc{b"PAL60"}else{b"NTSC"};
 if wphw_vws_title_op(1,&mut info,null_mut())<0{return Err(())}
 let op=if info.loaded{info.status_label=if info.is_native{info.dsp_native}else{info.dsp_interpreted};info.format=b"%s: %.0f | %s: %s (%.0f%%)\0".as_ptr();2}else{info.status_label=info.dsp_stopped;info.format=b"%s: %.0f | %s: %s\0".as_ptr();3};
 let mut statistics=MaybeUninit::<[u8;128]>::uninit();
 if wphw_vws_title_op(op,&mut info,statistics.as_mut_ptr().cast())<0{return Err(())}
 // Original source ignores snprintf return and consumes its NUL-terminated128-byte buffer.
 let statistics=title_cbytes(statistics.as_ptr().cast());
 if wphw_vws_title_op(4,&mut info,null_mut())<0{return Err(())}
 let mut title=title_cbytes(info.title).to_vec();title.extend_from_slice(b" (");title.extend_from_slice(&code);title.extend_from_slice(b") | ");
 if wphw_vws_title_op(5,&mut info,null_mut())<0{return Err(())}
 title.extend_from_slice(title_cbytes(info.api));title.extend_from_slice(b" | ");title.extend_from_slice(region);title.extend_from_slice(b" | ");title.extend_from_slice(statistics);Ok(title)
}
static WINDOW_TITLE:std::sync::Mutex<Vec<u8>>=std::sync::Mutex::new(Vec::new());
// port of video.cpp update_statistics. Source window test,title computation,locked compare/store,thenPostMessage order.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_statistics(fps:f64)->i32{
 let window=wphw_video_window();if window==0{return 0}
 let title=match window_title(fps){Ok(v)=>v,Err(())=>return -1};
 {let mut current=WINDOW_TITLE.lock().unwrap();if *current==title{return 0}*current=title;}
 wm::PostMessageA(window as HWND,wm::WM_APP+1,0,0);0
}
// port of source window_proc title-message SDK leaf:copy under the source mutex,thenSetWindowText outside it.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_apply_title(window:usize){let mut title={WINDOW_TITLE.lock().unwrap().clone()};title.push(0);wm::SetWindowTextA(window as HWND,title.as_ptr());}

// port of video.cpp save_screenshot + detached screenshot writer;native thread/vector ownership remains opaque.
#[repr(C)]pub struct ShotInfo{pub pixels:*const u32,pub count:usize,pub width:u32,pub height:u32,pub folder:*const u8,pub prefix:*const u8,pub file:usize,pub path:*const u8,pub path_len:usize}
extern "C"{
 fn wphw_vws_shot_op(c:*mut c_void,op:i32,info:*mut ShotInfo)->i32;
 fn wphw_vws_screenshot_log(format:*const u8,path:*const u8)->i32;
 fn wprt_screenshot_file_name(prefix:*const u8,n:usize,year:i32,month:i32,day:i32,hour:i32,minute:i32,second:i32,attempt:i32,c:*mut c_void,emit:Sink);
 fn wprt_png_encode(pixels:*const u32,n:usize,width:u32,height:u32,rgba:bool,c:*mut c_void,emit:Sink);
}
// Adapter of source vector/string byte ownership;callback receives bytes synchronously.
unsafe extern "C" fn collect_bytes(c:*mut c_void,p:*const u8,n:usize){let out=&mut *c.cast::<Vec<u8>>();out.clear();if n!=0{out.extend_from_slice(std::slice::from_raw_parts(p,n))}}
// port of video.cpp save_screenshot:read-frame first,one local-time snapshot,source attempts1..999 andFILE_EXISTS-only retry.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_save_screenshot(c:*mut c_void)->i32{
 use windows_sys::Win32::{Foundation::{INVALID_HANDLE_VALUE,GENERIC_WRITE,GetLastError,ERROR_FILE_EXISTS},Storage::FileSystem as file};
 let mut info:ShotInfo=zeroed();let read=wphw_vws_shot_op(c,0,&mut info);if read<0{return -1}if read==0{return 0}
 if wphw_vws_shot_op(c,1,&mut info)<0{return -1}
 file::CreateDirectoryA(info.folder,null());
 let folder=title_cbytes(info.folder).to_vec();let mut time=[0i32;6];wprt_vws_local_time(time.as_mut_ptr());
 let mut path=Vec::new();let mut handle=INVALID_HANDLE_VALUE;
 for attempt in 1..1000{
  if wphw_vws_shot_op(c,2,&mut info)<0{return -1}
  let prefix=title_cbytes(info.prefix);let mut name=Vec::new();wprt_screenshot_file_name(prefix.as_ptr(),prefix.len(),time[0],time[1],time[2],time[3],time[4],time[5],attempt,(&mut name as *mut Vec<u8>).cast(),collect_bytes);
  path.clear();path.extend_from_slice(&folder);path.push(b'/');path.extend_from_slice(&name);info.path=path.as_ptr();info.path_len=path.len();
  if wphw_vws_shot_op(c,3,&mut info)<0{return -1}
  path.push(0);handle=file::CreateFileA(path.as_ptr(),GENERIC_WRITE,0,null(),file::CREATE_NEW,file::FILE_ATTRIBUTE_NORMAL,null_mut());
  if handle==INVALID_HANDLE_VALUE{if GetLastError()!=ERROR_FILE_EXISTS{break}}else{break}
 }
 if handle==INVALID_HANDLE_VALUE{if wphw_vws_screenshot_log(b"screenshot could not be saved to %s\0".as_ptr(),path.as_ptr())<0{return -2}return 0}
 info.file=handle as usize;
 // Keep original native std::thread launch/capture and its failure semantics;no new CloseHandle on launch failure.
 if wphw_vws_shot_op(c,4,&mut info)<0{return -1}0
}
// port of the source std::thread lambda:original PNG encoder,WriteFile/DWORD comparison,CloseHandle,conditionalDelete,thenlogs.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_screenshot_write(handle:usize,path:*const u8,_path_len:usize,pixels:*const u32,n:usize,width:u32,height:u32)->i32{
 use windows_sys::Win32::{Foundation::{HANDLE,CloseHandle},Storage::FileSystem as file};
 let mut png:Vec<u8>=Vec::new();wprt_png_encode(pixels,n,width,height,false,(&mut png as *mut Vec<u8>).cast(),collect_bytes);
 let mut written=0u32;let ok=file::WriteFile(handle as HANDLE,png.as_ptr().cast(),png.len() as u32,&mut written,null_mut())!=0 && written as usize==png.len();
 CloseHandle(handle as HANDLE);if !ok{file::DeleteFileA(path);}
 let format=if ok{b"screenshot saved to %s\0".as_slice()}else{b"screenshot could not be written to %s\0"};
 if wphw_vws_screenshot_log(format.as_ptr(),path)<0{-1}else{0}
}

// port of video.cpp present_on_gpu,including accepted H6 offscreen publish before present.
#[repr(C)]pub struct PresentInfo{pub window:usize,pub aspect:f64,pub pixels:*const u32,pub count:usize,pub width:u32,pub height:u32,pub format:*const u8,pub index:i32,pub path:*mut u8}
extern "C"{
 fn wphw_vws_present_op(c:*mut c_void,op:i32,info:*mut PresentInfo)->i32;
 fn wprt_video_publish_host_frame();
 fn getenv(name:*const u8)->*mut u8;
 fn atoi(text:*const u8)->i32;
 fn _popen(command:*const u8,mode:*const u8)->*mut c_void;
 fn fwrite(p:*const c_void,size:usize,count:usize,file:*mut c_void)->usize;
}
static FRAME_PIPE:std::sync::OnceLock<usize>=std::sync::OnceLock::new();
static FRAME_EVERY:std::sync::OnceLock<i32>=std::sync::OnceLock::new();
static mut PIPE_PIXELS:usize=0;
static mut SAVED_FRAMES:i32=0;
// port of video.cpp native environment saving predicate;presence,not parsed booleans,as source H6 adapter.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_saving()->bool{!getenv(b"WP_OFFSCREEN\0".as_ptr()).is_null()||!getenv(b"WP_SAVE_FRAME\0".as_ptr()).is_null()||!getenv(b"WP_CAPTURE_AT\0".as_ptr()).is_null()||!getenv(b"WP_SAVE_PIPE\0".as_ptr()).is_null()}
// port of source H6 headless predicate.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_headless()->bool{!getenv(b"WP_HEADLESS\0".as_ptr()).is_null()||!getenv(b"WP_OFFSCREEN\0".as_ptr()).is_null()}
// port of present_on_gpu:original one-time pipe/every init,read/present/screenshot/next/pipe/save order andignored fwrite result.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_present_on_gpu(window:usize,aspect:f64,c:*mut c_void)->i32{
 let mut info:PresentInfo=zeroed();info.window=window;info.aspect=aspect;
 if !getenv(b"WP_OFFSCREEN\0".as_ptr()).is_null(){wprt_video_publish_host_frame()}
 if window!=0&&wphw_vws_present_op(c,0,&mut info)<0{return -1}
 let shot=wphw_vws_present_op(c,1,&mut info);if shot<0{return -1}if shot!=0&&wphw_vws_present_op(c,2,&mut info)<0{return -1}
 let next=wphw_vws_present_op(c,3,&mut info);if next<0{return -1}if next!=0{
  let read=wphw_vws_present_op(c,4,&mut info);if read<0{return -1}if read!=0&&wphw_vws_present_op(c,5,&mut info)<0{return -1}
  if wphw_vws_present_op(c,6,&mut info)<0{return -1}if wphw_vws_present_op(c,12,&mut info)<0{return -1}
 }
 let pipe=*FRAME_PIPE.get_or_init(||{if getenv(b"WP_SAVE_PIPE\0".as_ptr()).is_null(){0}else{_popen(getenv(b"WP_SAVE_PIPE\0".as_ptr()),b"wb\0".as_ptr()) as usize}});
 if pipe!=0{
  let read=wphw_vws_present_op(c,7,&mut info);if read<0{return -1}if read!=0{
   if PIPE_PIXELS==0{PIPE_PIXELS=info.count;if wphw_vws_present_op(c,8,&mut info)<0{return -1}}
   if info.count==PIPE_PIXELS{fwrite(info.pixels.cast(),size_of::<u32>(),info.count,pipe as *mut c_void);}
  }if wphw_vws_present_op(c,13,&mut info)<0{return -1}
 }
 let save=getenv(b"WP_SAVE_FRAME\0".as_ptr());
 let every=*FRAME_EVERY.get_or_init(||{if getenv(b"WP_SAVE_EVERY\0".as_ptr()).is_null(){100}else{atoi(getenv(b"WP_SAVE_EVERY\0".as_ptr()))}});
 if !save.is_null(){SAVED_FRAMES=SAVED_FRAMES.wrapping_add(1);if SAVED_FRAMES%every==0{
  let read=wphw_vws_present_op(c,9,&mut info);if read<0{return -1}if read!=0{
   let mut path=MaybeUninit::<[u8;512]>::uninit();info.format=save;info.index=SAVED_FRAMES/every;info.path=path.as_mut_ptr().cast();
   if wphw_vws_present_op(c,10,&mut info)<0{return -1}if wphw_vws_present_op(c,11,&mut info)<0{return -1}
  }if wphw_vws_present_op(c,14,&mut info)<0{return -1}
 }}0
}

extern "C"{
 fn wprt_settings_text(key:*const u8,variable:*const u8,sink:Sink,c:*mut c_void);
 fn wprt_ui_language_of(p:*const u8,n:usize)->i32;
 fn wprt_ui_set_language(language:i32);
 fn wphw_vws_option_op(window:usize,op:i32)->i32;
 fn wphw_vc_fullscreen()->bool;
}
// port of video.cpp apply_interface_language:settings std::string counted bytes into the existing Rust UI language selector.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_language(){let mut text:Vec<u8>=Vec::new();wprt_settings_text(b"system.interface_language\0".as_ptr(),b"WP_INTERFACE_LANGUAGE\0".as_ptr(),collect_bytes,(&mut text as *mut Vec<u8>).cast());wprt_ui_set_language(wprt_ui_language_of(text.as_ptr(),text.len()));}
// port of video.cpp apply_option:two independent exact-key checks,language before refresh,fullscreen only on a changed flag.
#[no_mangle]pub unsafe extern "C" fn wprt_vws_option(window:usize,key:*const u8,n:usize)->i32{
 let key=if n==0{&[]}else{std::slice::from_raw_parts(key,n)};
 if key==b"system.interface_language"{wprt_vws_language();if wphw_vws_option_op(window,0)<0{return -1}}
 if key==b"video.fullscreen"{let enable=wprt_settings_flag(b"video.fullscreen\0".as_ptr(),null());if enable!=wphw_vc_fullscreen(){wphw_vc_set_fullscreen(window,enable)}}0
}
