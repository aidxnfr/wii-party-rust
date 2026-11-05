// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted whole DSPCore.cpp policies;native class lifetime/typed fields/allocations/atomics/events/capture SDK.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,i64,i64,i64,i64,i64)->i64;
const CR_EXTERNAL_INT:u16=2;
const CR_HALT:u16=4;
const CR_INIT:u16=2048;
const DSP_COEF_BYTE_SIZE:u32=4096;
const DSP_DRAM_BYTE_SIZE:u32=8192;
const DSP_DRAM_SIZE:u32=4096;
const DSP_IRAM_BYTE_SIZE:u32=8192;
const DSP_IRAM_SIZE:u32=4096;
const DSP_IROM_BYTE_SIZE:u32=8192;
const DSP_RESET_VECTOR:u32=32768;
const SR_EXT_INT_ENABLE:u16=2048;
const SR_INT_ENABLE:u16=512;
// port of VerifyRoms + actual Common/Hash.h;AskYesNoFmtT is always false in this recomp shim,so both advisory branches cannot reject.
unsafe fn adler(p:*const u8,len:usize)->u32{let mut a=1u32;let mut b=0u32;for i in 0..len{a=(a+*p.add(i)as u32)%65521;b=(b+a)%65521;}(b<<16)|a}
unsafe fn verify_roms(irom:*const u8,coef:*const u8)->bool{let known=[(1727214846u32,4089001255u32),(2626640188u32,268435457u32),(3650125681u32,2954478331u32),(3650125681u32,3681059009u32),(983869331u32,2762307061u32),(311338914u32,2762307061u32),(3884561829u32,2762307061u32)];let hi=adler(irom,8192);let hd=adler(coef,4096);let mut idx=-1;for(i,(a,b))in known.into_iter().enumerate(){if hi==a&&hd==b{idx=i as i32;}}if idx<0&&false{return false;}if idx>=1&&idx<=5&&false{return false;}true}
// port of pristine DSPCore.cpp SDSP::Initialize.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_Initialize(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};s(16,0,0);if q(0,0,0,0,0,0)==0{return 0;}for(slot,size)in[(24,DSP_IROM_BYTE_SIZE),(25,DSP_IRAM_BYTE_SIZE),(26,DSP_DRAM_BYTE_SIZE),(27,DSP_COEF_BYTE_SIZE)]{let ptr=q(1,size as i64,0,0,0,0);s(slot,0,ptr);}std::ptr::copy_nonoverlapping(q(2,0,0,0,0,0)as *const u8,g(24,0)as *mut u8,DSP_IROM_BYTE_SIZE as usize);std::ptr::copy_nonoverlapping(q(2,1,0,0,0,0)as *const u8,g(27,0)as *mut u8,DSP_COEF_BYTE_SIZE as usize);if !verify_roms(g(24,0)as *const u8,g(27,0)as *const u8){wprt_dsp_core_sdsp_FreeMemoryPages(c,0,0,0,0,0,exec);return 0;}q(3,0,0,0,0,0);std::ptr::write_bytes(g(28,0)as *mut u8,0,4);std::ptr::write_bytes(g(29,0)as *mut u16,0,4*32);let iram=g(25,0)as *mut u16;for i in 0..DSP_IRAM_SIZE as usize{*iram.add(i)=0x0021;}std::ptr::write_bytes(g(26,0)as *mut u16,0,DSP_DRAM_SIZE as usize);let wr=g(30,0)as *mut u16;for i in 0..4{*wr.add(i)=0xffff;}s(5,0,((g(5,0)as u16)|SR_INT_ENABLE)as i64);s(5,0,((g(5,0)as u16)|SR_EXT_INT_ENABLE)as i64);s(18,0,(CR_INIT|CR_HALT)as i64);q(4,0,0,0,0,0);q(5,0,0,0,0,0);1}
// port of pristine DSPCore.cpp SDSP::Reset.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_Reset(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};s(17,0,DSP_RESET_VECTOR as i64);let wr=g(30,0)as *mut u16;for i in 0..4{*wr.add(i)=0xffff;}0}
// port of pristine DSPCore.cpp SDSP::Shutdown.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_Shutdown(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};wprt_dsp_core_sdsp_FreeMemoryPages(c,0,0,0,0,0,exec);0}
// port of pristine DSPCore.cpp SDSP::FreeMemoryPages.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_FreeMemoryPages(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};for(slot,size)in[(24,DSP_IROM_BYTE_SIZE),(25,DSP_IRAM_BYTE_SIZE),(26,DSP_DRAM_BYTE_SIZE),(27,DSP_COEF_BYTE_SIZE)]{q(6,g(slot,0),size as i64,0,0,0);}for slot in 24..=27{s(slot,0,0);}0}
// port of pristine DSPCore.cpp SDSP::SetException.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_SetException(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};s(19,0,((g(19,0)as u8 as u32)|(1u32<<(a as u32)))as u8 as i64);0}
// port of pristine DSPCore.cpp SDSP::SetExternalInterrupt.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_SetExternalInterrupt(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};q(9,(a!=0)as i64,0,0,0,0);0}
// port of pristine DSPCore.cpp SDSP::CheckExternalInterrupt.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_CheckExternalInterrupt(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};if g(5,0)as u16&SR_EXT_INT_ENABLE==0{return 0;}wprt_dsp_core_sdsp_SetException(c,7,0,0,0,0,exec);s(18,0,((g(18,0)as u16)&!CR_EXTERNAL_INT)as i64);0}
// port of pristine DSPCore.cpp SDSP::CheckExceptions.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_CheckExceptions(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};if g(19,0)as u8==0{return 0;}for i in(1..=7).rev(){if g(19,0)as u8 as u32&(1u32<<i)!=0{if g(5,0)as u16&SR_INT_ENABLE!=0||i==7{q(7,0,g(17,0)as u16 as i64,0,0,0);q(7,1,g(5,0)as u16 as i64,0,0,0);s(17,0,(i*2)as i64);s(19,0,((g(19,0)as u8 as u32)&!(1u32<<i))as u8 as i64);let mask=if i==7{SR_EXT_INT_ENABLE}else{SR_INT_ENABLE};s(5,0,((g(5,0)as u16)&!mask)as i64);return 1;}}}0}
// port of pristine DSPCore.cpp SDSP::ReadRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_ReadRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};(match a as usize{0..=3=>g(0,a),4..=7=>g(1,a-4),8..=11=>g(2,a-8),12..=15=>g(3,a-12),16..=17=>g(14,a-16),18=>g(4,0),19=>g(5,0),20..=23=>g(6+a-20,0),24..=25=>g(10,a-24),26..=27=>g(11,a-26),28..=29=>g(12,a-28),30..=31=>g(13,a-30),_=>0})as u16 as i64}
// port of pristine DSPCore.cpp SDSP::WriteRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_WriteRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};let v=b as u16 as i64;match a as usize{0..=3=>s(0,a,v),4..=7=>s(1,a-4,v),8..=11=>s(2,a-8,v),12..=15=>s(3,a-12,v),16..=17=>s(14,a-16,v),18=>s(4,0,v),19=>s(5,0,v),20..=23=>s(6+a-20,0,v),24..=25=>s(10,a-24,v),26..=27=>s(11,a-26,v),28..=29=>s(12,a-28,v),30..=31=>s(13,a-30,v),_=>{}}0}
// port of pristine DSPCore.cpp SDSP::DoState.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_sdsp_DoState(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|field,index|exec(c,100,field,index,0,0,0);let s=|field,index,value|{exec(c,101,field,index,value,0,0);};for field in 0..=16{if q(8,field,0,0,0,0)==0{return 0;}}if q(8,18,0,0,0,0)!=0{if q(8,19,0,0,0,0)==0{return 0;}}q(8,17,0,0,0,0);0}
// port of pristine DSPCore.cpp DSPCore::Initialize.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_Initialize(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};if q(0,0,0,0,0,0)==0{return 0;}q(1,0,0,0,0,0);s(1);1}
// port of pristine DSPCore.cpp DSPCore::Shutdown.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_Shutdown(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};if g()==0{return 0;}s(0);q(2,0,0,0,0,0);q(3,0,0,0,0,0);0}
// port of pristine DSPCore.cpp DSPCore::RunCycles.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_RunCycles(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};let mut cycles=a as i32;while cycles>0{match g(){1=>{cycles=q(4,cycles as i64,0,0,0,0)as i32;},2=>{q(5,0,0,0,0,0);if q(999,0,0,0,0,0)!=0{return cycles as i64;}if g()!=2{continue;}q(6,0,0,0,0,0);cycles=cycles.wrapping_sub(1);},_=>{}}}cycles as i64}
// port of pristine DSPCore.cpp DSPCore::Step.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_Step(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};if g()==2{q(7,0,0,0,0,0);}0}
// port of pristine DSPCore.cpp DSPCore::Reset.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_Reset(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(8,0,0,0,0,0);q(9,0,0,0,0,0);0}
// port of pristine DSPCore.cpp DSPCore::ClearIRAM.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_ClearIRAM(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};0}
// port of pristine DSPCore.cpp DSPCore::SetState.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_SetState(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};s(a as i32 as i64);if a as i32==1{q(7,0,0,0,0,0);}0}
// port of pristine DSPCore.cpp DSPCore::GetState.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_GetState(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};g()}
// port of pristine DSPCore.cpp DSPCore::SetException.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_SetException(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(10,a,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::SetExternalInterrupt.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_SetExternalInterrupt(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(11,a,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::CheckExternalInterrupt.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_CheckExternalInterrupt(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(12,0,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::CheckExceptions.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_CheckExceptions(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(13,0,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::ReadRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_ReadRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(14,a,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::WriteRegister.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_WriteRegister(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(15,a,b,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::PeekMailbox.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_PeekMailbox(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(16,a,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::ReadMailboxLow.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_ReadMailboxLow(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(17,a,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::ReadMailboxHigh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_ReadMailboxHigh(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(18,a,0,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::WriteMailboxLow.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_WriteMailboxLow(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(19,a,b,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::WriteMailboxHigh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_WriteMailboxHigh(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(20,a,b,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::LogIFXRead.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_LogIFXRead(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(21,a,b,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::LogIFXWrite.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_LogIFXWrite(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(22,a,b,0,0,0)}
// port of pristine DSPCore.cpp DSPCore::LogDMA.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_LogDMA(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(23,a,b,d,e,f)}
// port of pristine DSPCore.cpp DSPCore::IsJITCreated.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_IsJITCreated(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};0}
// port of pristine DSPCore.cpp DSPCore::DoState.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_core_core_DoState(c:*mut c_void,a:i64,b:i64,d:i64,e:i64,f:i64,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=||exec(c,100,0,0,0,0,0);let s=|value|{exec(c,101,value,0,0,0,0);};q(25,0,0,0,0,0);0}
