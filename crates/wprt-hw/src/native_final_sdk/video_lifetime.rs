static SHOT_NAME:&[u8]=&[42, 78, 83, 116, 54, 116, 104, 114, 101, 97, 100, 49, 49, 95, 83, 116, 97, 116, 101, 95, 105, 109, 112, 108, 73, 78, 83, 95, 56, 95, 73, 110, 118, 111, 107, 101, 114, 73, 83, 116, 53, 116, 117, 112, 108, 101, 73, 74, 90, 50, 55, 119, 112, 114, 116, 95, 99, 108, 97, 117, 100, 101, 95, 118, 119, 115, 95, 115, 104, 111, 116, 95, 116, 104, 114, 101, 97, 100, 69, 85, 108, 118, 69, 95, 69, 69, 69, 69, 69, 69, 0];
static FRAME_NAME:&[u8]=&[42, 90, 50, 50, 119, 112, 104, 119, 95, 118, 105, 100, 101, 111, 95, 113, 117, 101, 117, 101, 95, 102, 114, 97, 109, 101, 69, 85, 108, 118, 69, 95, 0];

use super::*;
use crate::{eh,native_final_lifetimes as threads};
use std::cell::UnsafeCell;
struct ErrorOwner(UnsafeCell<*mut c_void>);
impl Drop for ErrorOwner{fn drop(&mut self){unsafe{eh::clear(self.0.get())}}}
struct Snapshot(UnsafeCell<CxxVec>);
impl Drop for Snapshot{fn drop(&mut self){unsafe{v_free(self.0.get())}}}
std::thread_local!{
 static PENDING:ErrorOwner=const{ErrorOwner(UnsafeCell::new(null_mut()))};
 static SNAPSHOT:Snapshot=const{Snapshot(UnsafeCell::new(CxxVec{b:null_mut(),e:null_mut(),c:null_mut()}))};
}
fn pending()->*mut *mut c_void{PENDING.with(|p|p.0.get())}
static mut HOST_PIXELS:CxxVec=CxxVec{b:null_mut(),e:null_mut(),c:null_mut()};
#[export_name="_ZN2wp5video17g_next_frame_pathB5cxx11E"]
static mut NEXT_PATH:CxxString=CxxString{p:null_mut(),n:0,u:[0;2]};
extern "C"{fn atexit(f:unsafe extern "C" fn())->i32;}
extern "C-unwind"{
 fn wprt_video_aspect_ptr()->*mut f64;fn wprt_vws_window_thread();
 fn wprt_vws_save_screenshot(ctx:*mut c_void)->i32;
 fn wprt_vws_screenshot_write(file:usize,path:*const u8,n:usize,pixels:*const u32,count:usize,w:u32,h:u32)->i32;
 #[link_name="_ZN2wp2gx14run_frame_taskESt8functionIFvvEE"]fn frame_task(f:*mut CxxFunction);
 #[link_name="_ZNSt6thread6_StateD2Ev"]fn state_base_destruct(p:*mut u8);
 #[link_name="_ZTVN10__cxxabiv120__si_class_type_infoE"]static SI_INFO:[usize;3];
}
static mut ASPECT_REF:*mut f64=null_mut();
unsafe extern "C" fn pixels_fini(){v_free(addr_of_mut!(HOST_PIXELS));}
unsafe extern "C" fn path_fini(){str_free_inline(addr_of_mut!(NEXT_PATH));}
unsafe extern "C" fn load_init(){atexit(pixels_fini);ASPECT_REF=wprt_video_aspect_ptr();s_init(addr_of_mut!(NEXT_PATH));atexit(path_fini);}
#[used]#[link_section=".ctors"]static LOAD_INIT:unsafe extern "C" fn()=load_init;
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_video_host_pixels()->*mut CxxVec{addr_of_mut!(HOST_PIXELS)}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_vws_rethrow_pending()->!{eh::take_rethrow(pending())}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_vws_rethrow(p:*mut *mut c_void)->!{eh::rethrow(p)}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_video_snapshot_context()->usize{SNAPSHOT.with(|v|v.0.get() as usize)}
struct ShotGuard(*mut ShotCtx);
impl Drop for ShotGuard{fn drop(&mut self){unsafe{let p=self.0;eh::clear(addr_of_mut!((*p).exception));str_free_inline(addr_of_mut!((*p).path));v_free(addr_of_mut!((*p).pixels));}}}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_vws_save_screenshot(){
 let mut c:ShotCtx=core::mem::zeroed();s_init(addr_of_mut!(c.path));let _g=ShotGuard(&mut c);
 let result=wprt_vws_save_screenshot((&mut c as *mut ShotCtx).cast());
 if !c.exception.is_null(){eh::rethrow(&mut c.exception);}if result<0{eh::take_rethrow(pending());}
}
unsafe extern "C-unwind" fn window_thread(){wprt_vws_window_thread();let p=pending();if !(*p).is_null(){eh::take_rethrow(p);}}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_video_start_window(){threads::spawn_void(window_thread)}
static FRAME_INFO:CxxTypeInfo=CxxTypeInfo{vptr:unsafe{core::ptr::addr_of!(CXX_CLASS_TYPE_INFO_VTABLE).cast::<u8>().add(16)},name:FRAME_NAME.as_ptr()};
unsafe extern "C-unwind" fn frame_manager(dst:*mut u8,src:*const u8,op:i32)->bool{match op{
 0=>dst.cast::<*const CxxTypeInfo>().write(&FRAME_INFO),1=>dst.cast::<*const u8>().write(src),2=>core::ptr::copy_nonoverlapping(src,dst,16),_=>{}}false}
unsafe extern "C-unwind" fn frame_invoke(p:*const u8){super::present_on_gpu(p.cast::<usize>().read() as *mut c_void,p.add(8).cast::<f64>().read());}
struct FunctionGuard(*mut CxxFunction);impl Drop for FunctionGuard{fn drop(&mut self){unsafe{let f=self.0;if (*f).manager!=0{let m:unsafe extern "C-unwind" fn(*mut u8,*const u8,i32)->bool=core::mem::transmute((*f).manager);let p=addr_of_mut!((*f).data).cast();m(p,p,3);}}}}
#[no_mangle]pub unsafe extern "C-unwind" fn wphw_video_queue_frame(window:usize,aspect:f64){let mut f=CxxFunction{data:[window as u64,aspect.to_bits()],manager:frame_manager as usize,invoker:frame_invoke as usize};let _g=FunctionGuard(&mut f);frame_task(&mut f);}
#[repr(C)]struct ShotState{vptr:*const c_void,file:usize,path:CxxString,pixels:CxxVec,width:u32,height:u32}
#[repr(C)]struct ShotTable{offset:isize,ti:*const threads::SingleInheritanceInfo,d1:unsafe extern "C-unwind" fn(*mut u8),d0:unsafe extern "C-unwind" fn(*mut u8),run:unsafe extern "C-unwind" fn(*mut u8)}
unsafe impl Sync for ShotTable{}
static SHOT_INFO:threads::SingleInheritanceInfo=threads::SingleInheritanceInfo{vptr:unsafe{core::ptr::addr_of!(SI_INFO).cast::<usize>().add(2)},name:SHOT_NAME.as_ptr(),base:&threads::THREAD_BASE_INFO};
static SHOT_TABLE:ShotTable=ShotTable{offset:0,ti:&SHOT_INFO,d1:shot_d1,d0:shot_d0,run:shot_run};
unsafe extern "C-unwind" fn shot_d1(raw:*mut u8){let s=raw.cast::<ShotState>();(*s).vptr=core::ptr::addr_of!(SHOT_TABLE.d1).cast();v_free(addr_of_mut!((*s).pixels));str_free_inline(addr_of_mut!((*s).path));state_base_destruct(raw);}
unsafe extern "C-unwind" fn shot_d0(p:*mut u8){shot_d1(p);cxx_delete(p,80);}
unsafe extern "C-unwind" fn shot_run(p:*mut u8){let s=&*p.cast::<ShotState>();if wprt_vws_screenshot_write(s.file,s.path.p,s.path.n,s.pixels.b.cast(),v_len(&s.pixels,4),s.width,s.height)<0{eh::take_rethrow(pending());}}
#[repr(C)]struct ShotCapture{file:usize,path:CxxString,pixels:CxxVec,width:u32,height:u32}
struct CaptureGuard(*mut ShotCapture);impl Drop for CaptureGuard{fn drop(&mut self){unsafe{let p=self.0;v_free(addr_of_mut!((*p).pixels));str_free_inline(addr_of_mut!((*p).path));}}}
#[no_mangle]pub unsafe extern "C-unwind" fn wprt_claude_vws_shot_thread(raw:*mut c_void,info:*mut ShotInfo){
 let c=&mut *raw.cast::<ShotCtx>();let mut cap:ShotCapture=core::mem::zeroed();cap.file=(*info).file;
 s_copy_construct(&mut cap.path,&c.path);let _guard=CaptureGuard(&mut cap);
 cap.pixels=core::ptr::read(&c.pixels);v_init(&mut c.pixels);cap.width=c.width;cap.height=c.height;
 let p=cxx_new(80).cast::<ShotState>();(*p).vptr=core::ptr::addr_of!(SHOT_TABLE.d1).cast();(*p).file=cap.file;
 s_move_construct(addr_of_mut!((*p).path),&mut cap.path);addr_of_mut!((*p).pixels).write(core::ptr::read(&cap.pixels));v_init(&mut cap.pixels);(*p).width=cap.width;(*p).height=cap.height;
 threads::launch_cpp_state(p.cast());
}
// All catch/status sinks call the unchanged Rust policy through a raw foreign-unwind catch.
struct Call{kind:u8,raw:*mut c_void,op:i32,info:*mut c_void,other:*mut c_void}
unsafe extern "C-unwind" fn call_body(data:*mut c_void)->i32{let p=&mut *data.cast::<Call>();match p.kind{
 0=>wprt_claude_vws_title_body(p.op,p.info.cast(),p.other.cast()),
 1=>wprt_claude_vws_screenshot_log_body(p.info.cast(),p.other.cast()),
 2=>wprt_claude_vws_shot_body(p.raw,p.op,p.info.cast()),
 3=>wprt_claude_vws_present_body(p.raw,p.op,p.info.cast()),
 _=>wprt_claude_vws_option_body(p.raw as usize,p.op)}}
unsafe fn capture(mut c:Call,slot:*mut *mut c_void)->i32{eh::capture(call_body,(&mut c as *mut Call).cast(),slot)}
#[no_mangle]pub unsafe extern "C" fn wphw_vws_title_op(op:i32,info:*mut TitleInfo,buf:*mut u8)->i32{capture(Call{kind:0,raw:null_mut(),op,info:info.cast(),other:buf.cast()},pending())}
#[no_mangle]pub unsafe extern "C" fn wphw_vws_screenshot_log(format:*const u8,path:*const u8)->i32{capture(Call{kind:1,raw:null_mut(),op:0,info:format as *mut c_void,other:path as *mut c_void},pending())}
#[no_mangle]pub unsafe extern "C" fn wphw_vws_shot_op(raw:*mut c_void,op:i32,info:*mut ShotInfo)->i32{capture(Call{kind:2,raw,op,info:info.cast(),other:null_mut()},addr_of_mut!((*raw.cast::<ShotCtx>()).exception))}
#[no_mangle]pub unsafe extern "C" fn wphw_vws_present_op(raw:*mut c_void,op:i32,info:*mut PresentInfo)->i32{capture(Call{kind:3,raw,op,info:info.cast(),other:null_mut()},addr_of_mut!((*raw.cast::<PresentCtx>()).exception))}
#[no_mangle]pub unsafe extern "C" fn wphw_vws_option_op(window:usize,op:i32)->i32{capture(Call{kind:4,raw:window as *mut c_void,op,info:null_mut(),other:null_mut()},pending())}
