// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete hybrid/cpp/patched/exi.cpp scalar/stdio policy port.
// Native SDK keeps only std::string named-byte access;accepted EXI policy unchanged.
use std::ffi::c_char;
extern "C"{fn wprt_exi_read32(address:u32)->u32;fn wprt_exi_write32(address:u32,value:u32);fn wprt_exi_pending()->u32;fn __acrt_iob_func(index:u32)->*mut libc::FILE;}
// port of wp::exi scalar forwards;actual source archive nm names retained.
#[export_name="_ZN2wp3exi6read32Ej"]pub unsafe extern "C" fn read32(address:u32)->u32{wprt_exi_read32(address)}
#[export_name="_ZN2wp3exi7write32Ejj"]pub unsafe extern "C" fn write32(address:u32,value:u32){wprt_exi_write32(address,value);}
#[export_name="_ZN2wp3exi17pending_interruptEv"]pub unsafe extern "C" fn pending()->u32{wprt_exi_pending()}
// port of wphw_exi_uart_line;SAME CRT stream/fputs,including text mode/first-NUL termination.
#[no_mangle]pub unsafe extern "C" fn wphw_exi_uart_line(line:*const c_char){libc::fputs(line,__acrt_iob_func(2));}
