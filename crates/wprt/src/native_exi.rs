// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of pristine engine/src/core/exi.cpp; comparison RTC follows the existing H5 patch.
#![allow(non_upper_case_globals)]
use crate::memory::{rd8,wr8};
use std::{ffi::c_char,sync::Mutex};
extern "C"{fn wphw_exi_uart_line(text:*const c_char);}
// Same MinGW64 time aliases used by pristine C++ and the existing runtime port.
#[cfg(windows)]extern "C"{
    #[link_name="_localtime64"]fn native_localtime(t:*const libc::time_t)->*mut libc::tm;
    #[link_name="_gmtime64"]fn native_gmtime(t:*const libc::time_t)->*mut libc::tm;
    #[link_name="_mktime64"]fn native_mktime(t:*mut libc::tm)->libc::time_t;
    #[link_name="_difftime64"]fn native_difftime(a:libc::time_t,b:libc::time_t)->f64;
}
#[cfg(unix)]use libc::{localtime as native_localtime,gmtime as native_gmtime,mktime as native_mktime,difftime as native_difftime};
// C-string adapter for source std::string::c_str, retaining original embedded-NUL semantics.
fn terminated(bytes:&[u8])->Vec<u8>{let mut out=bytes.to_vec();out.push(0);out}
struct Ipl{command:u32,command_bytes:u32,cursor:u32,rtc_flags:u8,sram:[u8;kSramSize as usize],sram_dirty:bool,sram_file:Vec<u8>,line:Vec<u8>}
impl Ipl{
    // port of core/exi.cpp Ipl member initializers.
    const fn new()->Self{Self{command:0,command_bytes:0,cursor:0,rtc_flags:0,sram:kDefaultSram,sram_dirty:false,sram_file:Vec::new(),line:Vec::new()}}
    // port of core/exi.cpp Ipl::select.
    fn select(&mut self){self.command_bytes=0;self.command=0;self.cursor=0;}
    // port of core/exi.cpp Ipl::deselect; source clears dirty even if the file write fails.
    fn deselect(&mut self){if self.sram_dirty{self.save_sram();self.sram_dirty=false;}}
    // port of core/exi.cpp Ipl::transfer, including UART subranges which leave data unchanged.
    fn transfer(&mut self,mut data:u8)->u8{
        if self.command_bytes<4{self.command=(self.command<<8)|data as u32;data=0xff;self.command_bytes+=1;if self.command_bytes==4{self.update_rtc();}return data;}
        let write=self.command>>31!=0;let address=(self.command>>6)&0x1ffffff;
        if address>=kSramBase&&address<kSramBase+kSramSize{let offset=address.wrapping_sub(kSramBase).wrapping_add(self.cursor)%kSramSize;self.cursor=self.cursor.wrapping_add(1);if write{self.sram_dirty=self.sram_dirty||self.sram[offset as usize]!=data;self.sram[offset as usize]=data;}else{data=self.sram[offset as usize];}}
        else if address>=kUartBase&&address<kUartBase+kUartSize{if address==kUartBase{self.uart(write,&mut data);}}
        else if address==kWiiRtcBase+kWiiRtcFlags{if write{self.rtc_flags=data;}else{data=self.rtc_flags;}}
        else if address>=kEuartBase&&address<kEuartBase+kEuartSize{if address==kEuartBase+4{self.uart(write,&mut data);}}
        else if !write{data=0;}data
    }
    // port of core/exi.cpp Ipl::load; only an exact-size saved SRAM replaces the source default.
    fn load(&mut self,file:&[u8]){self.sram_file=file.to_vec();self.sram=kDefaultSram;let path=terminated(file);unsafe{let input=libc::fopen(path.as_ptr().cast(),c"rb".as_ptr());let mut saved=Vec::new();if !input.is_null(){loop{let ch=libc::fgetc(input);if ch==libc::EOF{break;}saved.push(ch as u8);}}if saved.len()==kSramSize as usize{self.sram.copy_from_slice(&saved);}if !input.is_null(){libc::fclose(input);}}}
    // port of core/exi.cpp Ipl::update_rtc + existing H5 opt-in fixed epoch/timezone patch.
    fn update_rtc(&mut self){
        #[cfg(feature="comparison")]let comparison=unsafe{crate::ffi::wphw_comparison_enabled()};
        #[cfg(not(feature="comparison"))]let comparison=false;
        unsafe{let now=if comparison{kSecondsFrom1970To2000 as libc::time_t}else{libc::time(std::ptr::null_mut())};let mut local=*native_localtime(&now);let mut utc=*native_gmtime(&now);utc.tm_isdst=local.tm_isdst;let offset=if comparison{0}else{native_difftime(native_mktime(&mut local),native_mktime(&mut utc))as i64};let rtc=(now as i64).wrapping_add(offset).wrapping_sub(kSecondsFrom1970To2000 as i64)as u32;self.sram[..4].copy_from_slice(&rtc.to_be_bytes());}
    }
    // port of core/exi.cpp Ipl::uart; native stderr preserves the original CRT stream behaviour.
    fn uart(&mut self,write:bool,data:&mut u8){if !write{*data=0;return;}if *data==b'\r'{return;}if *data!=0{self.line.push(*data);}if *data==b'\n'||self.line.len()>=4096{let line=terminated(&self.line);unsafe{wphw_exi_uart_line(line.as_ptr().cast());}self.line.clear();}}
    // port of core/exi.cpp Ipl::save_sram, binary/truncate write and ignored stream failures.
    fn save_sram(&self){if self.sram_file.is_empty(){return;}let path=terminated(&self.sram_file);unsafe{let out=libc::fopen(path.as_ptr().cast(),c"wb".as_ptr());if !out.is_null(){libc::fwrite(self.sram.as_ptr().cast(),1,kSramSize as usize,out);libc::fclose(out);}}}
}
#[derive(Clone,Copy)]struct Channel{status:u32,dma_address:u32,dma_length:u32,control:u32,data:u32}
impl Channel{const ZERO:Self=Self{status:0,dma_address:0,dma_length:0,control:0,data:0};}
struct State{ipl:Ipl,channels:[Channel;kChannels as usize]}
// Binding of core/exi.cpp g_ipl and the source channel initializer.
static STATE:Mutex<State>=Mutex::new(State{ipl:Ipl::new(),channels:{let mut c=[Channel::ZERO;kChannels as usize];c[0].status=kExtInt;c[1].status=kExtInt|(1<<kChipSelectShift);c}});
// port of core/exi.cpp ipl_selected.
fn ipl_selected(channel:u32,chip_select:u32)->bool{channel==0&&chip_select==2}
impl State{
    // port of core/exi.cpp transfer_byte.
    fn transfer_byte(&mut self,channel:u32,chip_select:u32,byte:u8)->u8{if ipl_selected(channel,chip_select){self.ipl.transfer(byte)}else{0}}
    // port of core/exi.cpp run_transfer; preserve invalid-selection and unsupported-mode effects.
    fn run_transfer(&mut self,index:usize){let chip_select=(self.channels[index].status&kChipSelectMask)>>kChipSelectShift;if chip_select!=1&&chip_select!=2&&chip_select!=4{return;}let mode=(self.channels[index].control>>kReadWriteShift)&3;
        if self.channels[index].control&kDma!=0{let mut i=0;while i<self.channels[index].dma_length{let address=self.channels[index].dma_address.wrapping_add(i);let byte=if mode==kWrite{rd8(address)as u8}else{0};let byte=self.transfer_byte(index as u32,chip_select,byte);if mode==kRead{wr8(address,byte as u32);}i=i.wrapping_add(1);}}
        else{let length=((self.channels[index].control>>kLengthShift)&3)+1;if mode==kWrite{let mut data=self.channels[index].data;for _ in 0..length{self.transfer_byte(index as u32,chip_select,(data>>24)as u8);data<<=8;}}else if mode==kRead{let mut result=0;for i in 0..length{result|=(self.transfer_byte(index as u32,chip_select,0)as u32)<<(24-i*8);}self.channels[index].data=result;}}
        self.channels[index].control&=!kTransferStart;self.channels[index].status|=kTcInt;
    }
    // port of core/exi.cpp write_status; preserve XOR selection transition and interrupt ordering.
    fn write_status(&mut self,index:usize,value:u32){let mut status=self.channels[index].status;status=(status&!kExiIntMask)|(value&kExiIntMask);if value&kExiInt!=0{status&=!kExiInt;}status=(status&!kTcIntMask)|(value&kTcIntMask);if value&kTcInt!=0{status&=!kTcInt;}status=(status&!kClockMask)|(value&kClockMask);if index<2{status=(status&!kExtIntMask)|(value&kExtIntMask);if value&kExtInt!=0{status&=!kExtInt;}}if index==0{status=(status&!kRomDisable)|(value&kRomDisable);}let old_select=(self.channels[index].status&kChipSelectMask)>>kChipSelectShift;let new_select=(value&kChipSelectMask)>>kChipSelectShift;status=(status&!kChipSelectMask)|(value&kChipSelectMask);self.channels[index].status=status;if index==0&&(old_select^new_select)==2{if new_select==2{self.ipl.select();}else{self.ipl.deselect();}}}
}
// port of core/exi.cpp mount; byte-copy adapter for the original std::string path.
#[no_mangle]pub unsafe extern "C" fn wprt_exi_mount(file:*const u8,n:usize){STATE.lock().unwrap().ipl.load(std::slice::from_raw_parts(file,n));}
// port of core/exi.cpp read32.
#[no_mangle]pub extern "C" fn wprt_exi_read32(address:u32)->u32{let s=STATE.lock().unwrap();let offset=address.wrapping_sub(kBase);let index=offset/kChannelStride;if index>=kChannels{return 0;}let channel=&s.channels[index as usize];match offset%kChannelStride{kStatus=>channel.status&!kExt,kDmaAddress=>channel.dma_address,kDmaLength=>channel.dma_length,kControl=>channel.control,kData=>channel.data,_=>0}}
// port of core/exi.cpp write32.
#[no_mangle]pub extern "C" fn wprt_exi_write32(address:u32,value:u32){let mut s=STATE.lock().unwrap();let offset=address.wrapping_sub(kBase);let index=offset/kChannelStride;if index>=kChannels{return;}let index=index as usize;match offset%kChannelStride{kStatus=>s.write_status(index,value),kDmaAddress=>s.channels[index].dma_address=value,kDmaLength=>s.channels[index].dma_length=value,kControl=>{s.channels[index].control=value;if value&kTransferStart!=0{s.run_transfer(index);}},kData=>s.channels[index].data=value,_=>{}}}
// port of core/exi.cpp pending_interrupt: preserve channel and EXI/TC/EXT priority.
#[no_mangle]pub extern "C" fn wprt_exi_pending()->u32{let s=STATE.lock().unwrap();for index in 0..kChannels{let status=s.channels[index as usize].status;let first=kFirstInterrupt+index*kInterruptsPerChannel;if status&kExiInt!=0&&status&kExiIntMask!=0{return first;}if status&kTcInt!=0&&status&kTcIntMask!=0{return first+1;}if index<2&&status&kExtInt!=0&&status&kExtIntMask!=0{return first+2;}}0}
const kBase:u32=0xCD006800;
const kChannelStride:u32=0x14;
const kChannels:u32=3;
const kStatus:u32=0x00;
const kDmaAddress:u32=0x04;
const kDmaLength:u32=0x08;
const kControl:u32=0x0C;
const kData:u32=0x10;
const kExiIntMask:u32=1 << 0;
const kExiInt:u32=1 << 1;
const kTcIntMask:u32=1 << 2;
const kTcInt:u32=1 << 3;
const kClockMask:u32=7 << 4;
const kChipSelectShift:u32=7;
const kChipSelectMask:u32=7 << kChipSelectShift;
const kExtIntMask:u32=1 << 10;
const kExtInt:u32=1 << 11;
const kExt:u32=1 << 12;
const kRomDisable:u32=1 << 13;
const kTransferStart:u32=1 << 0;
const kDma:u32=1 << 1;
const kReadWriteShift:u32=2;
const kLengthShift:u32=4;
const kRead:u32=0;
const kWrite:u32=1;
const kFirstInterrupt:u32=9;
const kInterruptsPerChannel:u32=3;
const kSramBase:u32=0x800000;
const kSramSize:u32=0x44;
const kUartBase:u32=0x800400;
const kUartSize:u32=0x50;
const kWiiRtcBase:u32=0x840000;
const kWiiRtcFlags:u32=0x20;
const kEuartBase:u32=0xC00000;
const kEuartSize:u32=8;
const kSecondsFrom1970To2000:u32=946684800;
const kDefaultSram:[u8;kSramSize as usize]=[0x0,0x0,0x0,0x0,0x0,0x2c,0xff,0xd0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x2c,0x44,0x4f,0x4c,0x50,0x48,0x49,0x4e,0x53,0x4c,0x4f,0x54,0x41,0x44,0x4f,0x4c,0x50,0x48,0x49,0x4e,0x53,0x4c,0x4f,0x54,0x42,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x6e,0x6d,0x0,0x0,0x0,0x0];
