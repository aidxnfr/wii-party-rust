// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPAnalyzer.cpp port;SDK owns the code-flag array,IMEM read and opcode metadata.
// Actual DSPAnalyzer.h defines DISABLE_UPDATE_SR_ANALYSIS;source logging shim is no-op.
use std::ffi::c_void;
#[repr(C)]pub struct Template{pub opcode:u16,pub size:u8,pub extended:u8}
pub type Read=unsafe extern "C" fn(*const c_void,u16)->u16;
pub type Op=unsafe extern "C" fn(u16,*mut Template)->bool;
const SIGS:[[u16;7];8]=[
[0x26fc,0x2c0,0x8000,0x29d,0xffff,0x0,0x0],
[0x27fc,0x3c0,0x8000,0x29d,0xffff,0x0,0x0],
[0x26fe,0x2c0,0x8000,0x29c,0xffff,0x0,0x0],
[0x27fe,0x3c0,0x8000,0x29c,0xffff,0x0,0x0],
[0x26fc,0x2a0,0x8000,0x29c,0xffff,0x0,0x0],
[0x27fc,0x3a0,0x8000,0x29c,0xffff,0x0,0x0],
[0xde,0xfffe,0x2c0,0x8000,0x29c,0xffff,0x0],
[0xda,0x352,0x8600,0x295,0xffff,0x0,0x0],
];
// port of Analyzer::Reset.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_analyzer_reset(flags:*mut u8){std::ptr::write_bytes(flags,0,65536);}
// port of Analyzer::Analyze.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_analyzer_analyze(flags:*mut u8,c:*const c_void,read:Read,op:Op){wprt_dsp_analyzer_reset(flags);wprt_dsp_analyzer_range(flags,c,0,0x1000,read,op);wprt_dsp_analyzer_range(flags,c,0x8000,0x9000,read,op);}
// port of Analyzer::AnalyzeRange.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_analyzer_range(flags:*mut u8,c:*const c_void,start:u16,end:u16,read:Read,op:Op){wprt_dsp_analyzer_starts(flags,c,start,end,read,op);wprt_dsp_analyzer_idle(flags,c,start,end,read);}
// port of Analyzer::FindInstructionStarts with source DISABLE_UPDATE_SR_ANALYSIS.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_analyzer_starts(flags:*mut u8,c:*const c_void,start:u16,end:u16,read:Read,op:Op){let mut addr=start;while addr<end{let inst=read(c,addr);let mut meta=Template{opcode:0,size:0,extended:0};if !op(inst,&mut meta){addr=addr.wrapping_add(1);continue;}*flags.add(addr as usize)|=1;if inst&0xffe0==0x0060 || inst&0xff00==0x1100{let last=read(c,addr.wrapping_add(1));*flags.add(addr as usize)|=4;*flags.add(last as usize)|=8;}else if inst&0xffe0==0x0040 || inst&0xff00==0x1000{*flags.add(addr as usize)|=4;*flags.add(addr.wrapping_add(1)as usize)|=8;}if [0x00c0,0x00e0,0x1600,0x1800,0x1880,0x1900,0x1980,0x1a00,0x1a80,0x1b00,0x1b80,0x2000,0x2800,0x2c00].contains(&meta.opcode)||meta.extended!=0{*flags.add(addr.wrapping_add(meta.size as u16)as usize)|=32;}addr=addr.wrapping_add(meta.size as u16);}}
// port of Analyzer::FindIdleSkips:zero sets found but does NOT terminate its scan;later mismatch keeps found.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_analyzer_idle(flags:*mut u8,c:*const c_void,start:u16,end:u16,read:Read){for sig in SIGS{let mut addr=start;while addr<end{let mut found=false;for i in 0..7{if sig[i]==0{found=true;}if sig[i]==0xffff{continue;}if sig[i]!=read(c,addr.wrapping_add(i as u16)){break;}}if found{*flags.add(addr as usize)|=2;}addr=addr.wrapping_add(1);}}}
