// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of engine/src/platform/options_render_stb.cpp advance,
// text_width and Canvas::text. Native stb font parsing/rasterization remains a leaf.
use std::ffi::c_void;
use crate::native_options_canvas::{wprt_canvas_blend, wprt_canvas_decode};

type Metrics = unsafe extern "C" fn(*const c_void, u32, u32, *mut i32, *mut i32);
type BitmapBox = unsafe extern "C" fn(*const c_void, i32, f32, *mut i32);
type Raster = unsafe extern "C" fn(*const c_void, i32, f32, *mut u8, i32, i32);

// port of advance: source signed width+kern is converted to float before scale.
#[no_mangle]
pub unsafe extern "C" fn wprt_options_text_advance(face:*const c_void, scale:f32,
    code:u32, next:u32, metrics:Metrics)->f32 {
    let mut width=0; let mut kern=0;
    metrics(face,code,next,&mut width,&mut kern);
    (width+kern) as f32 * scale
}

// port of text_width: accumulate one f32 addition per glyph in source order.
#[no_mangle]
pub unsafe extern "C" fn wprt_options_text_width(face:*const c_void,scale:f32,
    text:*const u32,len:usize,metrics:Metrics)->f32 {
    let glyphs=if len==0 { &[] } else { std::slice::from_raw_parts(text,len) };
    let mut width=0.0f32;
    for i in 0..glyphs.len() {
        width+=wprt_options_text_advance(face,scale,glyphs[i],
            if i+1<glyphs.len(){glyphs[i+1]}else{0},metrics);
    }
    width
}

unsafe extern "C" fn decoded(context:*mut c_void,p:*const u32,n:usize) {
    let out=&mut *(context as *mut Vec<u32>);
    if n!=0 { out.extend_from_slice(std::slice::from_raw_parts(p,n)); }
}

// port of Canvas::text. Align enum is source Left=0, Right=1, Center=2.
#[no_mangle]
pub unsafe extern "C" fn wprt_options_canvas_text(pixels:*mut u32,w:i32,h:i32,
    face:*const c_void,scale:f32,ascent:i32,descent:i32,font_ok:bool,color:u32,
    text:*const u8,len:usize,left:i32,top:i32,right:i32,bottom:i32,align:i32,
    ellipsis:bool,metrics:Metrics,bitmap_box:BitmapBox,raster:Raster) {
    if !font_ok { return; }
    let mut glyphs=Vec::<u32>::new();
    wprt_canvas_decode(text,len,decoded,(&mut glyphs as *mut Vec<u32>).cast());
    let width_of=|glyphs:&[u32]| wprt_options_text_width(face,scale,glyphs.as_ptr(),glyphs.len(),metrics);
    if ellipsis && width_of(&glyphs)>(right-left) as f32 {
        while !glyphs.is_empty() {
            // Source measures glyphs + U+2026, including its kerning, before pop.
            let mut trial=glyphs.clone(); trial.push(0x2026);
            if !(width_of(&trial)>(right-left) as f32) { break; }
            glyphs.pop();
        }
        glyphs.push(0x2026);
    }
    let width=width_of(&glyphs);
    let mut x=if align==0 {left as f32} else if align==1 {right as f32-width}
        else {left as f32+((right-left) as f32-width)/2.0f32};
    let baseline=top+(bottom-top-(ascent-descent))/2+ascent;
    for i in 0..glyphs.len() {
        let code=glyphs[i] as i32;
        let origin=x.floor(); let shift=x-origin;
        let mut bounds=[0i32;4];
        bitmap_box(face,code,shift,bounds.as_mut_ptr());
        let [x0,y0,x1,y1]=bounds;
        let glyph_width=x1-x0; let glyph_height=y1-y0;
        if glyph_width>0 && glyph_height>0 {
            let mut bitmap=vec![0u8;glyph_width as usize*glyph_height as usize];
            raster(face,code,shift,bitmap.as_mut_ptr(),glyph_width,glyph_height);
            for gy in 0..glyph_height { for gx in 0..glyph_width {
                // Source static_cast<int>(origin), compiled as CVTTSS2SI by
                // this GNU/x64 target, must not use Rust's saturating cast.
                let origin_i=std::arch::x86_64::_mm_cvttss_si32(std::arch::x86_64::_mm_set_ss(origin));
                wprt_canvas_blend(pixels,w,h,origin_i+x0+gx,baseline+y0+gy,
                    color,bitmap[gy as usize*glyph_width as usize+gx as usize] as u32);
            }}
        }
        x+=wprt_options_text_advance(face,scale,glyphs[i],
            if i+1<glyphs.len(){glyphs[i+1]}else{0},metrics);
    }
}
