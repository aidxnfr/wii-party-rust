// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of pristine Dolphin Core/DSP/DSPMemoryMap.cpp.
// Its ERROR/DEBUG_LOG_FMT macros are ((void)0) in this recomp's dolphin_shim/Common/Logging/Log.h.
use std::ffi::c_void;
pub type Read=unsafe extern "C" fn(*mut c_void,u16)->u16;
pub type Write=unsafe extern "C" fn(*mut c_void,u16,u16);
pub type Peek=unsafe extern "C" fn(*mut c_void)->u16;
pub type Size=unsafe extern "C" fn(u16)->u8;
// port of SDSP::ReadIMEM:source IRAM/IROM bank selection and masks.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mem_read_imem(iram:*const u16,irom:*const u16,address:u16)->u16{match address>>12{0=>iram.add((address&0x0fff) as usize).read(),8=>irom.add((address&0x0fff) as usize).read(),_=>0}}
// port of SDSP::ReadDMEM:coefficient mirror mask and unmodified full-address IFX callback.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mem_read_dmem(dram:*const u16,coef:*const u16,c:*mut c_void,address:u16,read:Read)->u16{match address>>12{0=>dram.add((address&0x0fff) as usize).read(),1=>coef.add((address&0x07ff) as usize).read(),15=>read(c,address),_=>0}}
// port of SDSP::WriteDMEM;unmapped writes are ignored by source.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mem_write_dmem(dram:*mut u16,c:*mut c_void,address:u16,value:u16,write:Write){match address>>12{0=>dram.add((address&0x0fff) as usize).write(value),15=>write(c,address,value),_=>()};}
// port of SDSP::FetchInstruction;source PeekInstruction completes before u16 PC increment.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mem_fetch(pc:*mut u16,c:*mut c_void,peek:Peek)->u16{let value=peek(c);*pc=(*pc).wrapping_add(1);value}
// port of SDSP::PeekInstruction.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mem_peek(pc:*const u16,iram:*const u16,irom:*const u16)->u16{wprt_dsp_mem_read_imem(iram,irom,*pc)}
// port of SDSP::SkipInstruction;source RHS Peek/table-size read precedes compound assignment to PC.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_mem_skip(pc:*mut u16,c:*mut c_void,peek:Peek,size:Size){let count=size(peek(c));*pc=(*pc).wrapping_add(count as u16);}
