// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine platform/video.cpp start/image_point/present plus accepted H6 offscreen transport.
// Integrated source;runtime acceptance tracked in HANDOFF. Native Win32 window/events/title/screenshot/file/render SDK leaves remain.
use std::sync::{Mutex, OnceLock};
const STANDARD_ASPECT: f64 = 4.0 / 3.0;
const WIDE_ASPECT: f64 = 16.0 / 9.0;
static mut ASPECT: f64 = STANDARD_ASPECT;
struct Frame {
    width: u32,
    height: u32,
    serial: u64,
    gpu_token: u64,
}
static HOST_FRAME: Mutex<Frame> = Mutex::new(Frame {
    width: 0,
    height: 0,
    serial: 0,
    gpu_token: 0,
});

// Optional scalar-only host publication transport. No GPU/Rust object crosses the DLL.
pub type GpuCapture=unsafe extern "C" fn(usize,*mut u64,*mut u32,*mut u32)->bool;
pub type GpuVisit=unsafe extern "C" fn(usize,u64,u32,u32,u64);
#[derive(Clone,Copy)]struct GpuPublication{context:usize,capture:GpuCapture}
static GPU_PUBLICATION:OnceLock<GpuPublication>=OnceLock::new();
pub type GpuRead=unsafe extern "C" fn(usize,u64,*mut *const u8,*mut u32,*mut u32)->bool;
static GPU_CPU_READER:OnceLock<(usize,GpuRead)>=OnceLock::new();
pub unsafe fn bind_gpu_cpu_reader(context:usize,read:GpuRead){
    assert!(GPU_CPU_READER.set((context,read)).is_ok(),"GPU CPU reader already bound");
}

pub unsafe fn bind_gpu_publication(context:usize,capture:GpuCapture){
    assert!(GPU_PUBLICATION.set(GpuPublication{context,capture}).is_ok(),"host publication already bound");
}
pub unsafe fn visit_gpu_publication(context:usize,visit:GpuVisit)->bool{
    if GPU_PUBLICATION.get().is_none(){return false;}
    let frame=HOST_FRAME.lock().unwrap();if frame.serial==0{return false;}
    // Hold the original publication lock through the host's immutable texture clone.
    visit(context,frame.gpu_token,frame.width,frame.height,frame.serial);true
}

extern "C" {
    fn wphw_video_read_frame(receive: unsafe extern "C" fn(usize, u32, u32));
    fn wphw_video_move_frame(frame: usize);
    fn wphw_video_snapshot_context() -> usize;
    fn wphw_video_snapshot(context: usize, pixels: *mut *mut u32, n: *mut usize) -> bool;
    fn wphw_video_log_build();
    fn wphw_video_widescreen() -> bool;
    fn wphw_video_apply_language();
    fn wphw_video_headless() -> bool;
    fn wphw_video_start_window();
    fn wphw_video_window() -> usize;
    fn wphw_video_saving() -> bool;
    fn wphw_video_gx_process();
    fn wphw_video_queue_frame(window: usize, aspect: f64);
}
// port of video.cpp g_aspect. Native Win32 helpers retain their original scalar reference to this stable field.
#[no_mangle]
pub unsafe extern "C" fn wprt_video_aspect_ptr() -> *mut f64 {
    std::ptr::addr_of_mut!(ASPECT)
}
// port of video.cpp start, including the accepted H6 WP_OFFSCREEN early return.
#[no_mangle]
pub unsafe extern "C" fn wprt_video_start() {
    wphw_video_log_build();
    ASPECT = if wphw_video_widescreen() {
        WIDE_ASPECT
    } else {
        STANDARD_ASPECT
    };
    wphw_video_apply_language();
    if wphw_video_headless() {
        return;
    }
    wphw_video_start_window();
}
// port of video.cpp image_point. Source uses Win32 long (32-bit), double fitting, float outputs, excludes right/bottom edge.
#[no_mangle]
pub unsafe extern "C" fn wprt_video_image_point(
    x: i32,
    y: i32,
    client_width: i32,
    client_height: i32,
    out_x: *mut f32,
    out_y: *mut f32,
) -> bool {
    if client_width <= 0 || client_height <= 0 {
        return false;
    }
    let mut width = client_width as f64;
    let mut height = width / ASPECT;
    if height > client_height as f64 {
        height = client_height as f64;
        width = height * ASPECT;
    }
    let left = (client_width as f64 - width) / 2.;
    let top = (client_height as f64 - height) / 2.;
    let u = (x as f64 - left) / width;
    let v = (y as f64 - top) / height;
    if u < 0. || u >= 1. || v < 0. || v >= 1. {
        return false;
    }
    *out_x = (u * 2. - 1.) as f32;
    *out_y = (v * 2. - 1.) as f32;
    true
}
// port of H6 publish_host_frame. Native local vector survives this synchronous callback, and is moved under the original lock scope.
unsafe extern "C" fn receive_frame(input: usize, width: u32, height: u32) {
    let mut frame = HOST_FRAME.lock().unwrap();
    wphw_video_move_frame(input);
    frame.width = width;
    frame.height = height;
    frame.serial = frame.serial.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn wprt_video_publish_host_frame() {
    if let Some(gpu)=GPU_PUBLICATION.get(){
        let mut frame=HOST_FRAME.lock().unwrap();let(mut token,mut width,mut height)=(0,0,0);
        if (gpu.capture)(gpu.context,&mut token,&mut width,&mut height){
            frame.gpu_token=token;frame.width=width;frame.height=height;frame.serial=frame.serial.wrapping_add(1);
        }
        return;
    }
    wphw_video_read_frame(receive_frame);
}
// port of accepted H6 wphost_frame. Native per-thread snapshot vector retains original lifetime; Rust performs the exact RGB->RGBA loop.
#[no_mangle]
pub unsafe extern "C" fn wphost_frame(
    data: *mut *const u8,
    width: *mut u32,
    height: *mut u32,
    serial: *mut u64,
) -> bool {
    if GPU_PUBLICATION.get().is_some(){
        let Some((context,read))=GPU_CPU_READER.get()else{return false;};
        let frame=HOST_FRAME.lock().unwrap();if frame.serial==0{return false;}
        if !read(*context,frame.gpu_token,data,width,height){return false;}
        *serial=frame.serial;return true;
    }
    let context = wphw_video_snapshot_context();
    let frame = HOST_FRAME.lock().unwrap();
    let mut pixels = std::ptr::null_mut();
    let mut n = 0;
    if !wphw_video_snapshot(context, &mut pixels, &mut n) {
        return false;
    }
    for i in 0..n {
        let p = *pixels.add(i);
        *pixels.add(i) = ((p >> 16) & 0xff) | (p & 0xff00) | ((p & 0xff) << 16) | 0xff000000;
    }
    *data = pixels.cast();
    *width = frame.width;
    *height = frame.height;
    *serial = frame.serial;
    true
}
// port of video.cpp present, keeping its first-call saving flag and original source ordering.
#[no_mangle]
pub unsafe extern "C" fn wprt_video_present() {
    let window = wphw_video_window();
    static SAVING: OnceLock<bool> = OnceLock::new();
    let saving = *SAVING.get_or_init(|| wphw_video_saving());
    if window == 0 && !saving {
        wphw_video_gx_process();
        return;
    }
    let aspect = ASPECT;
    wphw_video_queue_frame(window, aspect);
}

// H8 primitive geometry transport. The source sets ASPECT before first publish;
// its existing HOST_FRAME lock establishes readiness and synchronizes the host read.
#[no_mangle]pub unsafe extern "C" fn wphost_source_aspect(out:*mut f64)->bool{
 let frame=HOST_FRAME.lock().unwrap();if frame.serial==0{return false;}*out=ASPECT;true
}
#[no_mangle]pub unsafe extern "C" fn wphost_source_image_point(x:i32,y:i32,width:i32,height:i32,out_x:*mut f32,out_y:*mut f32)->bool{
 *out_x=0.0;*out_y=0.0;let frame=HOST_FRAME.lock().unwrap();if frame.serial==0{return false;}
 // Reuse the WHOLE accepted source routine;strict upper edges/f64 order stay there.
 wprt_video_image_point(x,y,width,height,out_x,out_y)
}
