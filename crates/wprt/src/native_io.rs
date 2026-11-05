// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/core/io.cpp (wp::mmio). Valid emulated addresses follow memory.h.
use crate::{memory::{host,FIFO_ADDRESS},native_si,native_exi,native_hollywood};
use std::{collections::BTreeSet,sync::{Mutex,OnceLock}};
extern "C"{
    fn wphw_io_dsp_read16(address:u32)->u16;fn wphw_io_dsp_write16(address:u32,value:u16);
    fn wphw_io_ipc_read32(address:u32)->u32;fn wphw_io_ipc_write32(address:u32,value:u32);
    fn wphw_io_gx_push(value:u64,bytes:u32);fn wphw_io_pixel_control(value:u16);
    fn wphw_io_log_plain(address:u32,bytes:u32,write:bool);
}
// port of core/io.cpp dsp_register.
fn dsp_register(address:u32)->bool{address>>12==0xcc005}
// port of core/io.cpp ipc_register.
fn ipc_register(address:u32)->bool{address>>8==0xcd0000}
// port of core/io.cpp serial_register.
fn serial_register(address:u32)->bool{address>>12==0xcd006&&address&0xf00==0x400}
// port of core/io.cpp exi_register.
fn exi_register(address:u32)->bool{address>>12==0xcd006&&address&0xfc0==0x800}
// port of core/io.cpp note_plain_access. Preserve cached presence, range, key and log-under-lock order.
fn note_plain_access(address:u32,bytes:u32,write:bool){static ENABLED:OnceLock<bool>=OnceLock::new();let enabled=*ENABLED.get_or_init(||unsafe{!libc::getenv(c"WP_LOG_MMIO".as_ptr()).is_null()});if !enabled||address>>24<0xcc||address>>24>0xcd{return;}static SEEN:OnceLock<Mutex<BTreeSet<u64>>>=OnceLock::new();let mut seen=SEEN.get_or_init(Default::default).lock().unwrap();let key=((address as u64)<<8)|((bytes<<1)as u64)|if write{1}else{0};if seen.insert(key){unsafe{wphw_io_log_plain(address,bytes,write);}}}
// port of core/io.cpp read_memory<uint16_t>, including the source's unconditional swap.
unsafe fn read_memory16(address:u32)->u16{note_plain_access(address,2,false);u16::from_ne_bytes(std::ptr::read_unaligned(host(address).cast::<[u8;2]>())).swap_bytes()}
// port of core/io.cpp read_memory<uint32_t>.
unsafe fn read_memory32(address:u32)->u32{note_plain_access(address,4,false);u32::from_ne_bytes(std::ptr::read_unaligned(host(address).cast::<[u8;4]>())).swap_bytes()}
// port of core/io.cpp write_memory<uint16_t>.
unsafe fn write_memory16(address:u32,value:u16){note_plain_access(address,2,true);let bytes=value.swap_bytes().to_ne_bytes();std::ptr::copy_nonoverlapping(bytes.as_ptr(),host(address),2);}
// port of core/io.cpp write_memory<uint32_t>.
unsafe fn write_memory32(address:u32,value:u32){note_plain_access(address,4,true);let bytes=value.swap_bytes().to_ne_bytes();std::ptr::copy_nonoverlapping(bytes.as_ptr(),host(address),4);}
// port of core/io.cpp write_memory<uint64_t>.
unsafe fn write_memory64(address:u32,value:u64){note_plain_access(address,8,true);let bytes=value.swap_bytes().to_ne_bytes();std::ptr::copy_nonoverlapping(bytes.as_ptr(),host(address),8);}
// port of core/io.cpp read16. Caller supplies a valid original emulated-memory address.
#[no_mangle]pub unsafe extern "C" fn wprt_io_read16(address:u32)->u16{if dsp_register(address){return wphw_io_dsp_read16(address);}read_memory16(address)}
// port of core/io.cpp read32; source device order and DSP high/low reads are retained.
#[no_mangle]pub unsafe extern "C" fn wprt_io_read32(address:u32)->u32{if dsp_register(address){let high=wphw_io_dsp_read16(address)as u32;let low=wphw_io_dsp_read16(address.wrapping_add(2))as u32;return(high<<16)|low;}if ipc_register(address){return wphw_io_ipc_read32(address);}if serial_register(address){return native_si::wprt_si_read32(address);}if native_hollywood::wprt_hollywood_owns(address){return native_hollywood::wprt_hollywood_read32(address);}if exi_register(address){return native_exi::wprt_exi_read32(address);}read_memory32(address)}
// port of core/io.cpp write8: non-FIFO byte writes are raw memory, not device-register dispatch.
#[no_mangle]pub unsafe extern "C" fn wprt_io_write8(address:u32,value:u8){if address==FIFO_ADDRESS{wphw_io_gx_push(value as u64,1);return;}note_plain_access(address,1,true);*host(address)=value;}
// port of core/io.cpp write16, including the exact pixel-engine control special case.
#[no_mangle]pub unsafe extern "C" fn wprt_io_write16(address:u32,value:u16){if address==FIFO_ADDRESS{wphw_io_gx_push(value as u64,2);return;}if address==0xcc00100a{wphw_io_pixel_control(value);return;}if dsp_register(address){wphw_io_dsp_write16(address,value);return;}write_memory16(address,value);}
// port of core/io.cpp write32; preserve DSP half order and split pixel-engine token/control write.
#[no_mangle]pub unsafe extern "C" fn wprt_io_write32(address:u32,value:u32){if address==FIFO_ADDRESS{wphw_io_gx_push(value as u64,4);return;}if dsp_register(address){wphw_io_dsp_write16(address,(value>>16)as u16);wphw_io_dsp_write16(address.wrapping_add(2),value as u16);return;}if ipc_register(address){wphw_io_ipc_write32(address,value);return;}if serial_register(address){native_si::wprt_si_write32(address,value);return;}if native_hollywood::wprt_hollywood_owns(address){native_hollywood::wprt_hollywood_write32(address,value);return;}if exi_register(address){native_exi::wprt_exi_write32(address,value);return;}if address==0xcc001008{write_memory16(address,(value>>16)as u16);wphw_io_pixel_control(value as u16);return;}write_memory32(address,value);}
// port of core/io.cpp write64: source deliberately does not split into device write32 calls.
#[no_mangle]pub unsafe extern "C" fn wprt_io_write64(address:u32,value:u64){if address==FIFO_ADDRESS{wphw_io_gx_push(value,8);return;}write_memory64(address,value);}
