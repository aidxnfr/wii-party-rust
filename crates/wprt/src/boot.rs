// SPDX-License-Identifier: GPL-3.0-or-later
// port of engine/src/core/boot.cpp; native low-memory constants/order retained.
#![allow(non_upper_case_globals)]
use std::ffi::{CStr,CString,c_char,c_void};
use crate::memory::{host,wr16,wr32};
extern "C"{fn fopen(path:*const c_char,mode:*const c_char)->*mut c_void;fn fread(dst:*mut c_void,size:usize,count:usize,file:*mut c_void)->usize;fn fclose(file:*mut c_void)->i32;}
const kMem1Size:u32=0x01800000;
const kMem1End:u32=0x81800000;
const kMem2Size:u32=0x04000000;
const kMem2End:u32=0x93600000;
const kMem2ArenaLow:u32=0x90000800;
const kIosHeapSize:u32=0x00020000;
const kBi2Size:u32=0x2000;
const kBusClock:u32=243000000;
const kCpuClock:u32=729000000;
const kPalVideoMode:u32=1;
const kNtscVideoMode:u32=0;
const kVideoControlRegister:u32=0xCC002002;
const kVideoClockRegister:u32=0xCC00206C;
const kVideoDtvStatusRegister:u32=0xCC00206E;
const kDiConfigRegister:u32=0xCD006024;
const kVideoControlPal:u16=0x0101;
const kVideoControlNtsc:u16=0x0001;
const kVideoInterruptEnabled:u16=0x1001;
const kHeaderSize:u32=0x20;
const kVideoInterruptRegisters:[u32;2]=[0xcc002030,0xcc002034];
// port of engine/src/core/boot.cpp read_file.
fn read_file(path:&CStr,data:&mut Vec<u8>)->bool{unsafe{let file=fopen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return false;}let mut buffer=[0u8;65536];loop{let count=fread(buffer.as_mut_ptr().cast(),1,buffer.len(),file);if count==0{break;}data.extend_from_slice(&buffer[..count]);}fclose(file);true}}
// port of engine/src/core/boot.cpp init_boot_memory.
pub fn init_boot_memory(extracted_directory:&CStr)->bool{
 let path=|suffix:&[u8]|{let mut bytes=extracted_directory.to_bytes().to_vec();bytes.extend_from_slice(suffix);CString::new(bytes).unwrap()};
 let(mut header,mut bi2,mut fst)=(Vec::new(),Vec::new(),Vec::new());
 if !read_file(&path(b"/sys/boot.bin"),&mut header)||!read_file(&path(b"/sys/bi2.bin"),&mut bi2)||!read_file(&path(b"/sys/fst.bin"),&mut fst)||header.len()<kHeaderSize as usize{return false;}
 let fst_address=kMem1End.wrapping_sub(fst.len()as u32)&!0x1f;let bi2_address=fst_address.wrapping_sub(kBi2Size);
 unsafe{std::ptr::copy_nonoverlapping(header.as_ptr(),host(0x80000000),kHeaderSize as usize);std::ptr::copy_nonoverlapping(bi2.as_ptr(),host(bi2_address),bi2.len());std::ptr::copy_nonoverlapping(fst.as_ptr(),host(fst_address),fst.len());}
    wr32(0x80000020, 0x0D15EA5E);
    wr32(0x80000024, 1);
    wr32(0x80000028, kMem1Size);
    wr32(0x8000002C, 0x00000023);
    wr32(0x80000034, fst_address);
    wr32(0x80000038, fst_address);
    wr32(0x8000003C, fst.len() as u32);
    let region=header[3];
    let ntsc=matches!(region,b'E'|b'J'|b'K'|b'W');
    wr32(0x800000CC, if ntsc{kNtscVideoMode}else{kPalVideoMode});
    wr16(kVideoControlRegister, if ntsc{u32::from(kVideoControlNtsc)}else{u32::from(kVideoControlPal)});
    wr16(kVideoClockRegister, if ntsc{1}else{0});
    wr16(kVideoDtvStatusRegister, if region==b'J'{2}else{0});
    wr32(kDiConfigRegister, 1);
    for address in kVideoInterruptRegisters {
        wr16(address, u32::from(kVideoInterruptEnabled));
    }
    wr32(0x800000EC, kMem1End);
    wr32(0x800000F0, kMem1Size);
    wr32(0x800000F4, bi2_address);
    wr32(0x800000F8, kBusClock);
    wr32(0x800000FC, kCpuClock);

    wr32(0x800030D8, 0xFFFFFFFF);
    wr32(0x800030E4, 0x00008201);
    wr32(0x800030E8, 0x00050000);
    wr32(0x80003100, kMem1Size);
    wr32(0x80003104, kMem1Size);
    wr32(0x80003108, kMem1End);
    wr32(0x80003110, fst_address);
    wr32(0x80003114, 0xDEADBEEF);
    wr32(0x80003118, kMem2Size);
    wr32(0x8000311C, kMem2Size);
    wr32(0x80003120, kMem2End);
    wr32(0x80003124, kMem2ArenaLow);
    wr32(0x80003128, kMem2End - kIosHeapSize);
    wr32(0x8000312C, 0xDEADBEEF);
    wr32(0x80003130, kMem2End - kIosHeapSize);
    wr32(0x80003134, kMem2End);
    wr32(0x80003138, 0x00000011);
    wr32(0x80003140, 0x0038161E);
    wr32(0x80003144, 0x00030310);
    wr32(0x80003148, kMem2End);
    wr32(0x8000314C, kMem2End.wrapping_add(0x20000));
    wr32(0x80003158, 0x0000FF01);
    wr32(0x8000315C, 0x80800113);
    unsafe{std::ptr::copy_nonoverlapping(header.as_ptr(),host(0x80003180),4);}
    wr32(0x80003184, 0x80000000);
    wr32(0x80003188, 0x0038151D);
    wr32(0x80003198, 0x03E00000);
    wr32(0x8000319C, 0x80000000);
 true
}
