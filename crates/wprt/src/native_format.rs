// SPDX-License-Identifier: GPL-3.0-or-later
// Port of arelkair pristine engine/src/core/format.cpp; preserve argument order and byte-string behaviour.
use crate::{cpu::Cpu,memory::{rd8,rd32,rdf64}};
use std::{ffi::{c_char,CStr,CString},mem::MaybeUninit};
// Same backend selected by the installed GNU C++17 stdio.h (__USE_MINGW_ANSI_STDIO=1).
extern "C"{fn __mingw_snprintf(buffer:*mut c_char,count:usize,format:*const c_char,...)->i32;}
const FIRST_REGISTER:u32=3;const LAST_REGISTER:u32=10;const STACK_OFFSET:u32=8;
const FIRST_FLOAT:u32=1;const LAST_FLOAT:u32=8;const MAX_TEXT:usize=4096;
// port of core/format.cpp read_string: the source reads the byte before checking the size cap.
fn read_string(mut address:u32)->Vec<u8>{let mut text=Vec::new();loop{let ch=rd8(address)as u8;if ch==0||text.len()>=MAX_TEXT{break;}text.push(ch);address=address.wrapping_add(1);}text}
struct Arguments<'a>{cpu:&'a Cpu,next_register:u32,next_float:u32,stack:u32}
impl<'a> Arguments<'a>{
    // port of core/format.cpp Arguments::Arguments.
    fn new(cpu:&'a Cpu,first:u32)->Self{Self{cpu,next_register:first,next_float:FIRST_FLOAT,stack:cpu.r[1].wrapping_add(STACK_OFFSET)}}
    // port of core/format.cpp Arguments::integer.
    fn integer(&mut self)->u32{if self.next_register<=LAST_REGISTER{let index=self.next_register;self.next_register=self.next_register.wrapping_add(1);return self.cpu.r[index as usize];}let value=rd32(self.stack);self.stack=self.stack.wrapping_add(4);value}
    // port of core/format.cpp Arguments::wide, including the register-pair and stack alignment branches.
    fn wide(&mut self)->u64{if self.next_register<=LAST_REGISTER{if self.next_register.wrapping_sub(FIRST_REGISTER)%2!=0{self.next_register=self.next_register.wrapping_add(1);}}else if self.stack%8!=0{self.stack=self.stack.wrapping_add(4);}let high=self.integer()as u64;let low=self.integer()as u64;(high<<32)|low}
    // port of core/format.cpp Arguments::floating: independent floating-register cursor, then aligned stack.
    fn floating(&mut self)->f64{if self.next_float<=LAST_FLOAT{let index=self.next_float;self.next_float=self.next_float.wrapping_add(1);return self.cpu.f[index as usize];}if self.stack%8!=0{self.stack=self.stack.wrapping_add(4);}let value=rdf64(self.stack);self.stack=self.stack.wrapping_add(8);value}
}
// ABI adapter for the source's (spec + suffix).c_str(), with the same bytes and terminating NUL.
fn specifier(spec:&[u8],suffix:&[u8])->CString{let mut text=spec.to_vec();text.extend_from_slice(suffix);CString::new(text).unwrap()}
// port of core/format.cpp format_guest. Native snprintf preserves locale, numeric formatting and truncation.
pub fn format_guest(cpu:&Cpu,format:u32,first:u32)->Vec<u8>{
    let mut arguments=Arguments::new(cpu,first);let text=read_string(format);let mut output=Vec::new();let mut i=0;
    while i<text.len(){
        if text[i]!=b'%'{output.push(text[i]);i+=1;continue;}
        let mut spec=vec![b'%'];let mut length=0;i+=1;
        while i<text.len()&&b"-+ #0123456789.*hlLzjt".contains(&text[i]){
            if text[i]==b'*'{spec.extend_from_slice((arguments.integer()as i32).to_string().as_bytes());}
            else if text[i]==b'l'{length+=1;}
            else if !b"hzjtL".contains(&text[i]){spec.push(text[i]);}
            i+=1;
        }
        if i>=text.len(){break;}
        let conversion=text[i];let mut buffer=MaybeUninit::<[c_char;512]>::uninit();let pointer=buffer.as_mut_ptr().cast::<c_char>();
        let formatted=unsafe{match conversion{
            b'%'=>{output.push(b'%');false}
            b'd'|b'i'=>{if length>=2{let f=specifier(&spec,b"lld");__mingw_snprintf(pointer,512,f.as_ptr(),arguments.wide()as i64);}else{let f=specifier(&spec,b"d");__mingw_snprintf(pointer,512,f.as_ptr(),arguments.integer()as i32);}true}
            b'u'|b'x'|b'X'|b'o'=>{if length>=2{let suffix=[b'l',b'l',conversion];let f=specifier(&spec,&suffix);__mingw_snprintf(pointer,512,f.as_ptr(),arguments.wide());}else{let f=specifier(&spec,&[conversion]);__mingw_snprintf(pointer,512,f.as_ptr(),arguments.integer());}true}
            b'c'=>{let f=specifier(&spec,b"c");__mingw_snprintf(pointer,512,f.as_ptr(),(arguments.integer()&0xff)as i32);true}
            b's'=>{let address=arguments.integer();let value=if address==0{None}else{Some(CString::new(read_string(address)).unwrap())};let value=value.as_ref().map_or(c"(null)".as_ptr(),|s|s.as_ptr());let f=specifier(&spec,b"s");__mingw_snprintf(pointer,512,f.as_ptr(),value);true}
            b'p'=>{__mingw_snprintf(pointer,512,c"0x%08x".as_ptr(),arguments.integer());true}
            b'f'|b'F'|b'e'|b'E'|b'g'|b'G'=>{let f=specifier(&spec,&[conversion]);__mingw_snprintf(pointer,512,f.as_ptr(),arguments.floating());true}
            _=>{output.extend_from_slice(&spec);output.push(conversion);false}
        }};
        if formatted{output.extend_from_slice(unsafe{CStr::from_ptr(pointer)}.to_bytes());}
        i+=1;
    }
    output
}
