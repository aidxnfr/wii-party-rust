// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of engine/src/platform/options_render_stb.cpp Canvas helpers.
// Original native stb fonts,text shaping/rasterization and window remain native.
use std::ffi::c_void;
unsafe fn pixel(p:*mut u32,width:i32,x:i32,y:i32)->*mut u32{p.offset(y as isize*width as isize+x as isize)}
// port of Canvas::fill;source clipped-loop/range preconditions retained.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_fill(p:*mut u32,w:i32,h:i32,l:i32,t:i32,r:i32,b:i32,color:u32){let l=l.max(0);let t=t.max(0);let r=r.min(w);let b=b.min(h);for y in t..b{let mut x=l;while x<r{*pixel(p,w,x,y)=color;x+=1}}}
// port of Canvas::blend;source integer channel order and alpha rounding retained.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_blend(p:*mut u32,w:i32,h:i32,x:i32,y:i32,color:u32,coverage:u32){if x<0||y<0||x>=w||y>=h||coverage==0{return}let target=pixel(p,w,x,y);let below=*target;let mut result=0;for shift in (0..24).step_by(8){let b=(below>>shift)&255;let a=(color>>shift)&255;let mixed=(b*(255-coverage)+a*coverage+127)/255;result|=mixed<<shift}*target=result}
// port of Canvas::fill_round;no fitted radius/filter constants.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_round(p:*mut u32,w:i32,h:i32,l:i32,t:i32,r:i32,b:i32,radius:i32,color:u32){let radius=radius.min((r-l)/2).min((b-t)/2).max(0);for y in t.max(0)..b.min(h){for x in l.max(0)..r.min(w){let cx=x as f64+0.5;let cy=y as f64+0.5;let ox=if cx<(l+radius)as f64{(l+radius)as f64-cx}else if cx>(r-radius)as f64{cx-(r-radius)as f64}else{0.0};let oy=if cy<(t+radius)as f64{(t+radius)as f64-cy}else if cy>(b-radius)as f64{cy-(b-radius)as f64}else{0.0};let mut coverage=1.0;if ox>0.0&&oy>0.0{coverage=(radius as f64-(ox*ox+oy*oy).sqrt()+0.5).clamp(0.0,1.0)}wprt_canvas_blend(p,w,h,x,y,color,(coverage*255.0+0.5)as u32)}}}
// port of Canvas::dot.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_dot(p:*mut u32,w:i32,h:i32,x:i32,y:i32,stroke:i32,color:u32){wprt_canvas_fill(p,w,h,x-stroke/2,y-stroke/2,x-stroke/2+stroke,y-stroke/2+stroke,color)}
// port of Canvas::cross.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_cross(p:*mut u32,w:i32,h:i32,x:i32,y:i32,half:i32,stroke:i32,color:u32){for k in -half..=half{wprt_canvas_dot(p,w,h,x+k,y+k,stroke,color);wprt_canvas_dot(p,w,h,x+k,y-k,stroke,color)}}
// port of Canvas::chevron.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_chevron(p:*mut u32,w:i32,h:i32,cx:i32,cy:i32,half:i32,direction:i32,stroke:i32,color:u32){for k in 0..=half{let x=cx+direction*(half/2-k);wprt_canvas_dot(p,w,h,x,cy-half+k,stroke,color);wprt_canvas_dot(p,w,h,x,cy+half-k,stroke,color)}}
// port of corner_alpha;panel alpha236 comes directly from source kPanelAlpha.
#[no_mangle]pub extern "C" fn wprt_canvas_corner(x:i32,y:i32,width:i32,height:i32,radius:i32)->u32{let cx=x as f64+0.5;let cy=y as f64+0.5;let ox=if cx<(radius as f64){radius as f64-cx}else if cx>((width-radius)as f64){cx-(width-radius)as f64}else{0.0};let oy=if cy<(radius as f64){radius as f64-cy}else if cy>((height-radius)as f64){cy-(height-radius)as f64}else{0.0};if ox<=0.0||oy<=0.0{return 236}let coverage=(radius as f64-(ox*ox+oy*oy).sqrt()+0.5).clamp(0.0,1.0);(236.0*coverage+0.5)as u32}
// port of decode;preserve malformed/truncated UTF8 handling,including char32_t values outside Unicode.
#[no_mangle]pub unsafe extern "C" fn wprt_canvas_decode(p:*const u8,n:usize,sink:unsafe extern "C" fn(*mut c_void,*const u32,usize),c:*mut c_void){let data=if n==0{&[]}else{std::slice::from_raw_parts(p,n)};let mut out=Vec::<u32>::new();let mut i=0;while i<n{let lead=data[i]as u32;let length=if lead<128{1}else if lead>>5==6{2}else if lead>>4==14{3}else if lead>>3==30{4}else{1};let mut value=if length==1{lead}else{lead&(127>>length)};let mut k=1;while k<length&&i+k<n{value=(value<<6)|(data[i+k]as u32&63);k+=1}out.push(value);i+=length}sink(c,out.as_ptr(),out.len())}
