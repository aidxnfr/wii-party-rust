// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of engine/src/platform/options_render_stb.cpp render_titlebar.
use std::ffi::c_void;
use crate::native_options_canvas::wprt_canvas_fill;
type Title = unsafe extern "C" fn(*mut c_void,*mut u32,i32,i32,i32,u32,i32,i32,i32);

// port of render_titlebar. Callback retains original font lookup/make_face/text
// leaves; image ownership passes back to native only after source composition.
#[no_mangle]
pub unsafe extern "C" fn wprt_options_titlebar(p:*mut u32,width:i32,height:i32,
    hover:i32,pressed:i32,maximized:bool,focused:bool,title:Title,context:*mut c_void) {
    if width<=0||height<=0{return}
    let pixels=std::slice::from_raw_parts_mut(p,width as usize*height as usize);
    pixels.fill(0x161618);
    let button=height*46/32;
    let stroke=1.max(height/32);
    let half=4.max(height*5/32);
    let step=2.max(height*2/32);
    let buttons_left=width-button*3;
    let margin=height*12/32;
    // Original font(false).ok check and make_face evaluation stay in callback.
    title(context,p,width,height,8.max(height*13/32),
        if focused{0xffffff}else{0xa1a1a6},margin,margin.max(buttons_left-margin),height-stroke);
    let fill=|l,t,r,b,c|wprt_canvas_fill(p,width,height,l,t,r,b,c);
    fill(0,height-stroke,width,height,0x3a3a3e);
    for i in 0..3 {
        let left=buttons_left+i*button;
        let close=i==2;let over=i==hover;
        if over {
            let color=if close {if pressed==i{0xb50e1c}else{0xe81123}}
                else if pressed==i{0x3a3a3e}else{0x2c2c30};
            fill(left,0,left+button,height-stroke,color);
        }
        let color=if over&&close||focused{0xffffff}else{0xa1a1a6};
        let cx=left+button/2;let cy=(height-stroke)/2;
        if i==0 {
            fill(cx-half,cy,cx+half+1,cy+stroke,color);
        }else if i==1&&!maximized{
            fill(cx-half,cy-half,cx+half+1,cy-half+stroke,color);
            fill(cx-half,cy+half,cx+half+1,cy+half+stroke,color);
            fill(cx-half,cy-half,cx-half+stroke,cy+half+stroke,color);
            fill(cx+half,cy-half,cx+half+stroke,cy+half+stroke,color);
        }else if i==1{
            fill(cx-half,cy-half+step,cx+half-step+1,cy-half+step+stroke,color);
            fill(cx-half,cy+half,cx+half-step+1,cy+half+stroke,color);
            fill(cx-half,cy-half+step,cx-half+stroke,cy+half+stroke,color);
            fill(cx+half-step,cy-half+step,cx+half-step+stroke,cy+half+stroke,color);
            fill(cx-half+step,cy-half,cx+half+1,cy-half+stroke,color);
            fill(cx+half,cy-half,cx+half+stroke,cy+half-step+stroke,color);
        }else{
            for k in -half..=half{
                fill(cx+k,cy+k,cx+k+stroke,cy+k+stroke,color);
                fill(cx+k,cy-k,cx+k+stroke,cy-k+stroke,color);
            }
        }
    }
    // No Rust slice reference crosses callbacks/aliases to the native buffer.
    for i in 0..width as usize*height as usize{
        *p.add(i)=(*p.add(i)&0xffffff)|0xff000000;
    }
}
