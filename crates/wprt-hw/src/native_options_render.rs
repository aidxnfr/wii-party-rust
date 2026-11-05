// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of engine/src/platform/options_render_stb.cpp render/render_hint.
// Native callbacks expose named fields/opaque fonts,never std::string/class layouts.
use std::ffi::c_void;
use crate::native_options_canvas::*;
#[repr(C)]#[derive(Clone,Copy)]pub struct Bytes{p:*const u8,n:usize}
#[repr(C)]#[derive(Clone,Copy,Default)]pub struct Rect{x:i32,y:i32,w:i32,h:i32}
#[repr(C)]pub struct Frame{unit:i32,w:i32,h:i32,padding:i32,title:i32,row:i32,footer:i32,rows:usize}
#[repr(C)]pub struct Row{label:Bytes,value:Bytes,live:bool,toggle:bool,on:bool}
#[repr(C)]pub struct Hint{name:Bytes,keys:Bytes}
#[repr(C)]pub struct Strings{title:Bytes,restart:Bytes,hint:Bytes,hover_row:i32,hover_part:i32}
type Face=unsafe extern "C" fn(*mut c_void,i32,bool,i32);
type Text=unsafe extern "C" fn(*mut c_void,*mut u32,i32,i32,i32,u32,Bytes,i32,i32,i32,i32,i32,bool);
type Width=unsafe extern "C" fn(*mut c_void,i32,Bytes)->f32;
type Image=unsafe extern "C" fn(*mut c_void,u32,u32,*const u32,usize);
#[repr(C)]pub struct Ops{
    face:Face,text:Text,width:Width,image:Image,
    strings:unsafe extern "C" fn(*mut c_void,*mut Strings),
    selected:unsafe extern "C" fn(*mut c_void)->usize,
    rect:unsafe extern "C" fn(*mut c_void,bool,usize,*mut Rect),
}
const PANEL:u32=0x161618;const SELECTED:u32=0x2c2c30;const SEP:u32=0x3a3a3e;
const TEXT:u32=0xffffff;const MUTED:u32=0xa1a1a6;const ACCENT:u32=0xd4146f;
const ACCENT_HOVER:u32=0xe8307f;const HOVER:u32=0x202023;const CONTROL:u32=0x1e1e21;
const CONTROL_HOVER:u32=0x2a2a2e;const OFF:u32=0x4a4a50;const OFF_HOVER:u32=0x5c5c63;
unsafe fn ceil_int(x:f32)->i32{
    std::arch::x86_64::_mm_cvttss_si32(std::arch::x86_64::_mm_set_ss(x.ceil()))
}
// port of load_font, Windows branch (matched_font returns empty).
// Callback is the source filesystem/ifstream/stb operation;ANSI path bytes and
// candidate ordering are retained,including fallback paths on other platforms.
#[no_mangle]pub unsafe extern "C" fn wprt_options_load_font(bold:bool,windows:*const u8,
    windows_len:usize,has_windows:bool,attempt:unsafe extern "C" fn(*mut c_void,Bytes)->bool,c:*mut c_void)->bool{
    let fallbacks:&[&[u8]]=&[
        b"/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",b"/usr/share/fonts/TTF/DejaVuSans.ttf",
        b"/usr/share/fonts/dejavu/DejaVuSans.ttf",b"/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        b"/usr/share/fonts/noto/NotoSans-Regular.ttf",b"/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        b"/System/Library/Fonts/Supplemental/Arial.ttf",b"/Library/Fonts/Arial.ttf",
    ];
    let mut candidates=vec![Vec::<u8>::new()];
    candidates.extend(fallbacks.iter().map(|p|p.to_vec()));
    if has_windows{
        let mut folder=if windows_len==0{Vec::new()}else{std::slice::from_raw_parts(windows,windows_len).to_vec()};
        folder.extend_from_slice(b"/Fonts/");
        let mut first=folder.clone();first.extend_from_slice(if bold{b"seguisb.ttf"}else{b"segoeui.ttf"});candidates.insert(0,first);
        let mut regular=folder.clone();regular.extend_from_slice(b"segoeui.ttf");candidates.push(regular);
        folder.extend_from_slice(b"arial.ttf");candidates.push(folder);
    }
    for path in candidates{if attempt(c,Bytes{p:path.as_ptr(),n:path.len()}){return true}}
    false
}
// port of make_face, keeping exact STB calls native and std::lround CRT behavior.
#[no_mangle]pub unsafe extern "C" fn wprt_options_face_metrics(scale:f32,ascent:i32,descent:i32,
    round:unsafe extern "C" fn(f32)->i32,out:*mut i32){
    *out=round(ascent as f32*scale);*out.add(1)=round(descent as f32*scale);
}
// port of font's original bold-and-usable selection (cache initialization native).
#[no_mangle]pub extern "C" fn wprt_options_choose_bold(bold:bool,semibold_ok:bool)->bool{bold&&semibold_ok}
// port of render: source rows/layout/regular-font precondition supplied by native.
#[no_mangle]pub unsafe extern "C" fn wprt_options_render(c:*mut c_void,ops:*const Ops,
    frame:*const Frame,rows:*const Row,count:usize){
    let o=&*ops;let f=&*frame;
    let rows=if count==0{&[]}else{std::slice::from_raw_parts(rows,count)};
    let mut pixels=vec![PANEL;f.w as usize*f.h as usize];let p=pixels.as_mut_ptr();let u=f.unit;
    (o.face)(c,0,true,u*3/2);(o.face)(c,1,false,u);(o.face)(c,2,false,u*4/5);
    let mut strings=std::mem::MaybeUninit::<Strings>::uninit();(o.strings)(c,strings.as_mut_ptr());let s=strings.assume_init();
    let left=f.padding;let right=f.w-f.padding;let stroke=2.max(u/7);
    let text=|face,color,value,l,t,r,b,align,ellipsis|(o.text)(c,p,f.w,f.h,face,color,value,l,t,r,b,align,ellipsis);
    let fill=|l,t,r,b,col|wprt_canvas_fill(p,f.w,f.h,l,t,r,b,col);
    let round=|l,t,r,b,rad,col|wprt_canvas_round(p,f.w,f.h,l,t,r,b,rad,col);
    text(0,TEXT,s.title,left,f.padding,right,f.padding+f.title-u/2,0,false);
    let mut close=Rect::default();(o.rect)(c,true,0,&mut close);
    let close_hover=s.hover_part==5;
    if close_hover{round(close.x,close.y,close.x+close.w,close.y+close.h,u/2,0xc42b1c)}
    wprt_canvas_cross(p,f.w,f.h,close.x+close.w/2,close.y+close.h/2,u/3,stroke,if close_hover{TEXT}else{MUTED});
    let rows_top=f.padding+f.title;fill(left,rows_top-u/2,right,rows_top-u/2+1,SEP);
    for(i,row)in rows.iter().enumerate(){
        let selected=i==(o.selected)(c);let hovered=s.hover_row==i as i32;
        let top=rows_top+i as i32*f.row;let bottom=top+f.row;
        if selected||hovered{round(left-u/2,top+2,right+u/2,bottom-2,u/2,if selected{SELECTED}else{HOVER})}
        if selected{round(left-u/2+u/4,top+u/2,left-u/2+u/4+3.max(u/5),bottom-u/2,2.max(u/10),ACCENT)}
        let mut control=Rect::default();(o.rect)(c,false,i,&mut control);let mut value_left=control.x;
        if row.toggle{
            let width=u*3;let switch_left=control.x+control.w-width;value_left=switch_left;
            let mut track=if row.on{ACCENT}else{OFF};
            if hovered&&s.hover_part!=1{track=if row.on{ACCENT_HOVER}else{OFF_HOVER}}
            round(switch_left,control.y,switch_left+width,control.y+control.h,control.h/2,track);
            let knob=control.h-u/2;let knob_left=if row.on{switch_left+width-knob-u/4}else{switch_left+u/4};
            round(knob_left,control.y+u/4,knob_left+knob,control.y+u/4+knob,knob/2,TEXT);
        }else{
            round(control.x,control.y,control.x+control.w,control.y+control.h,control.h/2,if hovered{CONTROL_HOVER}else{CONTROL});
            let arrow=u*2;let previous=if hovered&&s.hover_part==2{ACCENT}else{MUTED};
            let next=if hovered&&s.hover_part==4{ACCENT}else{MUTED};let cy=control.y+control.h/2;
            wprt_canvas_chevron(p,f.w,f.h,control.x+arrow/2,cy,u/3,1,stroke,previous);
            wprt_canvas_chevron(p,f.w,f.h,control.x+control.w-arrow/2,cy,u/3,-1,stroke,next);
            text(1,if selected{ACCENT}else{TEXT},row.value,control.x+arrow,control.y,control.x+control.w-arrow,control.y+control.h,2,true);
        }
        let mut label_right=value_left-u/2;
        if !row.live{
            let note_width=ceil_int((o.width)(c,2,s.restart));let note_left=value_left-u/2-note_width;
            text(2,MUTED,s.restart,note_left-u,top,value_left-u/2,bottom,1,false);label_right=note_left-u/2;
        }
        text(1,TEXT,row.label,left+u/2,top,(left+u).max(label_right),bottom,0,true);
    }
    let footer_top=rows_top+count as i32*f.row;fill(left,footer_top+u/2,right,footer_top+u/2+1,SEP);
    text(2,MUTED,s.hint,left,footer_top+u/2,right,footer_top+f.footer,2,true);
    let radius=u*3/4;
    for y in 0..f.h{for x in 0..f.w{let i=y as usize*f.w as usize+x as usize;
        pixels[i]=(pixels[i]&0xffffff)|(wprt_canvas_corner(x,y,f.w,f.h,radius)<<24);
    }}
    (o.image)(c,f.w as u32,f.h as u32,pixels.as_ptr(),pixels.len());
}
// port of render_hint:source preconditions regular font/nonempty/client size native.
#[no_mangle]pub unsafe extern "C" fn wprt_options_hint(c:*mut c_void,ops:*const Ops,
    rows:*const Hint,count:usize,title:Bytes,cw:i32,ch:i32){
    let o=&*ops;let rows=std::slice::from_raw_parts(rows,count);
    let u=(ch/60).clamp(9,24);let padding=u;let row_height=u*3/2;let title_height=u*2;let gap=u*2;
    (o.face)(c,0,true,u*5/4);(o.face)(c,1,false,u);
    let mut name_width=0;let mut keys_width=0;
    for row in rows{name_width=name_width.max(ceil_int((o.width)(c,1,row.name)));keys_width=keys_width.max(ceil_int((o.width)(c,1,row.keys)))}
    let title_width=ceil_int((o.width)(c,0,title));let margin=u*2;
    let width=(cw-margin*2).min(title_width.max(name_width+gap+keys_width)+padding*2);
    let height=padding*2+title_height+count as i32*row_height;
    if width<=padding*2||height>ch-margin*2{return}
    let mut pixels=vec![PANEL;width as usize*height as usize];let p=pixels.as_mut_ptr();let stroke=2.max(u/7);
    let text=|face,col,value,l,t,r,b|(o.text)(c,p,width,height,face,col,value,l,t,r,b,0,true);
    text(0,TEXT,title,padding,padding,width-padding,padding+title_height-u/2);
    let rows_top=padding+title_height;
    wprt_canvas_fill(p,width,height,padding,rows_top-u/2,width-padding,rows_top-u/2+stroke/2+1,SEP);
    for(i,row)in rows.iter().enumerate(){let top=rows_top+i as i32*row_height;
        text(1,MUTED,row.name,padding,top,padding+name_width,top+row_height);
        text(1,TEXT,row.keys,padding+name_width+gap,top,width-padding,top+row_height);
    }
    let mut image=vec![0u32;cw as usize*ch as usize];let left=cw-width-margin;let radius=u*3/4;
    for y in 0..height{for x in 0..width{
        image[(margin+y)as usize*cw as usize+(left+x)as usize]=(pixels[y as usize*width as usize+x as usize]&0xffffff)|(wprt_canvas_corner(x,y,width,height,radius)<<24);
    }}
    (o.image)(c,cw as u32,ch as u32,image.as_ptr(),image.len());
}
