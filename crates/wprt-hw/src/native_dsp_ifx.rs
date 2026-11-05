// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted WHOLE DSPHWInterface.cpp port;native typed atomic/accelerator/Host/logger SDK remains.
// Source logging shims are no-ops;pdlabel reads only choose those no-ops.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,i64,i64,i64,i64,i64)->i64;
const DSP_ACCAH:u16=216;
const DSP_ACCAL:u16=217;
const DSP_ACDRAW:u16=211;
const DSP_ACDSAMP:u16=221;
const DSP_ACEAH:u16=214;
const DSP_ACEAL:u16=215;
const DSP_ACIN:u16=223;
const DSP_ACSAH:u16=212;
const DSP_ACSAL:u16=213;
const DSP_AMDM:u16=239;
const DSP_CMBH:u16=254;
const DSP_CMBL:u16=255;
const DSP_DIRQ:u16=251;
const DSP_DMBH:u16=252;
const DSP_DMBL:u16=253;
const DSP_DSBL:u16=203;
const DSP_DSCR:u16=201;
const DSP_DSMAH:u16=206;
const DSP_DSMAL:u16=207;
const DSP_DSPA:u16=205;
const DSP_FORMAT:u16=209;
const DSP_GAIN:u16=222;
const DSP_PRED_SCALE:u16=218;
const DSP_YN1:u16=219;
const DSP_YN2:u16=220;
// port of pristine DSPHWInterface.cpp SDSP::InitializeIFX.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_InitializeIFX(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};std::ptr::write_bytes(regs,0,256);q(2,0,0,0,0,0);q(2,1,0,0,0,0);0}
// port of pristine DSPHWInterface.cpp SDSP::PeekMailbox.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_PeekMailbox(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};q(0,a as u32 as i64,0,0,0,0)as u32 as i64}
// port of pristine DSPHWInterface.cpp SDSP::ReadMailboxLow.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_ReadMailboxLow(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let v=q(1,a as u32 as i64,0,0,0,0)as u32;q(3,a as u32 as i64,(v&!0x80000000)as i64,0,0,0);v as u16 as i64}
// port of pristine DSPHWInterface.cpp SDSP::ReadMailboxHigh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_ReadMailboxHigh(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};(wprt_dsp_ifx_PeekMailbox(c,a,0,0,regs,exec)as u32>>16)as u16 as i64}
// port of pristine DSPHWInterface.cpp SDSP::WriteMailboxLow.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_WriteMailboxLow(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let old=q(1,a as u32 as i64,0,0,0,0)as u32;let value=(old&!0xffff)|(b as u16 as u32);q(3,a as u32 as i64,(value|0x80000000)as i64,0,0,0);0}
// port of pristine DSPHWInterface.cpp SDSP::WriteMailboxHigh.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_WriteMailboxHigh(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let old=q(1,a as u32 as i64,0,0,0,0)as u32;let value=(old&0xffff)|((b as u16 as u32)<<16);q(3,a as u32 as i64,(value&!0x80000000)as i64,0,0,0);0}
// port of pristine DSPHWInterface.cpp SDSP::WriteIFX.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_WriteIFX(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let address=a as u32;let v=b as u16;q(5,address as i64,v as i64,0,0,0);match(address&255)as u16{DSP_DMBH=>{wprt_dsp_ifx_WriteMailboxHigh(c,1,v as i64,0,regs,exec);},DSP_DMBL=>{wprt_dsp_ifx_WriteMailboxLow(c,1,v as i64,0,regs,exec);},DSP_CMBH=>{wprt_dsp_ifx_WriteMailboxHigh(c,0,v as i64,0,regs,exec);},DSP_CMBL=>{wprt_dsp_ifx_WriteMailboxLow(c,0,v as i64,0,regs,exec);},DSP_DIRQ=>{if v&1!=0{q(4,0,0,0,0,0);}},DSP_DSBL=>{s(DSP_DSBL,v);s(DSP_DSCR,g(DSP_DSCR)|4);if g(DSP_AMDM)==0{wprt_dsp_ifx_DoDMA(c,0,0,0,regs,exec);}s(DSP_DSCR,g(DSP_DSCR)&!4);s(DSP_DSBL,0);},DSP_DSPA|DSP_DSMAH|DSP_DSMAL|DSP_DSCR=>s((address&255)as u16,v),DSP_ACSAH=>{let old=q(7,0,0,0,0,0)as u32;let next=((v as u32)<<16)|(old as u16 as u32);q(8,0,next as i64,0,0,0);},DSP_ACSAL=>{let old=q(7,0,0,0,0,0)as u32;let next=((old>>16)as u16 as u32)<<16|v as u32;q(8,0,next as i64,0,0,0);},DSP_ACEAH=>{let old=q(7,1,0,0,0,0)as u32;let next=((v as u32)<<16)|(old as u16 as u32);q(8,1,next as i64,0,0,0);},DSP_ACEAL=>{let old=q(7,1,0,0,0,0)as u32;let next=((old>>16)as u16 as u32)<<16|v as u32;q(8,1,next as i64,0,0,0);},DSP_ACCAH=>{let old=q(7,2,0,0,0,0)as u32;let next=((v as u32)<<16)|(old as u16 as u32);q(8,2,next as i64,0,0,0);},DSP_ACCAL=>{let old=q(7,2,0,0,0,0)as u32;let next=((old>>16)as u16 as u32)<<16|v as u32;q(8,2,next as i64,0,0,0);},DSP_FORMAT=>{q(8,3,v as i64,0,0,0);},DSP_GAIN=>{q(8,4,v as i64,0,0,0);},DSP_YN1=>{q(8,5,v as i64,0,0,0);},DSP_YN2=>{q(8,6,v as i64,0,0,0);},DSP_PRED_SCALE=>{q(8,7,v as i64,0,0,0);},DSP_ACIN=>{q(8,8,v as i64,0,0,0);},DSP_ACDRAW=>{q(11,v as i64,0,0,0,0);},_=>s((address&255)as u16,v)}0}
// port of pristine DSPHWInterface.cpp SDSP::ReadIFXImpl.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_ReadIFXImpl(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let address=a as u16;match address&255{DSP_DMBH=>wprt_dsp_ifx_ReadMailboxHigh(c,1,0,0,regs,exec),DSP_DMBL=>wprt_dsp_ifx_ReadMailboxLow(c,1,0,0,regs,exec),DSP_CMBH=>wprt_dsp_ifx_ReadMailboxHigh(c,0,0,0,regs,exec),DSP_CMBL=>wprt_dsp_ifx_ReadMailboxLow(c,0,0,0,regs,exec),DSP_ACSAH=>(q(7,0,0,0,0,0)as u32>>16)as u16 as i64,DSP_ACSAL=>q(7,0,0,0,0,0)as u16 as i64,DSP_ACEAH=>(q(7,1,0,0,0,0)as u32>>16)as u16 as i64,DSP_ACEAL=>q(7,1,0,0,0,0)as u16 as i64,DSP_ACCAH=>(q(7,2,0,0,0,0)as u32>>16)as u16 as i64,DSP_ACCAL=>q(7,2,0,0,0,0)as u16 as i64,DSP_FORMAT=>q(7,3,0,0,0,0)as u16 as i64,DSP_GAIN=>q(7,4,0,0,0,0)as u16 as i64,DSP_YN1=>q(7,5,0,0,0,0)as u16 as i64,DSP_YN2=>q(7,6,0,0,0,0)as u16 as i64,DSP_PRED_SCALE=>q(7,7,0,0,0,0)as u16 as i64,DSP_ACIN=>q(7,8,0,0,0,0)as u16 as i64,DSP_ACDSAMP=>q(9,0,0,0,0,0)as u16 as i64,DSP_ACDRAW=>q(10,0,0,0,0,0)as u16 as i64,_=>g(address&255)as i64}}
// port of pristine DSPHWInterface.cpp SDSP::ReadIFX.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_ReadIFX(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let ret=wprt_dsp_ifx_ReadIFXImpl(c,a as u16 as i64,0,0,regs,exec)as u16;q(6,a as u16 as i64,ret as i64,0,0,0);ret as i64}
// port of pristine DSPHWInterface.cpp SDSP::IDMAIn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_IDMAIn(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let dsp=a as u16;let addr=b as u32;let size=d as u32;q(12,0,dsp as i64,addr as i64,size as i64,0);q(13,addr as i64,size as i64,0,0,0);(q(20,0,0,0,0,0)as usize).wrapping_add(dsp as usize)as i64}
// port of pristine DSPHWInterface.cpp SDSP::IDMAOut.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_IDMAOut(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};0}
// port of pristine DSPHWInterface.cpp SDSP::DDMAIn.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_DDMAIn(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let dsp=a as u16;q(12,1,dsp as i64,b as u32 as i64,d as u32 as i64,0);(q(21,0,0,0,0,0)as usize).wrapping_add(dsp as usize)as i64}
// port of pristine DSPHWInterface.cpp SDSP::DDMAOut.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_DDMAOut(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let dsp=a as u16;q(14,dsp as i64,b as u32 as i64,d as u32 as i64,0,0);(q(21,0,0,0,0,0)as usize).wrapping_add(dsp as usize)as i64}
// port of pristine DSPHWInterface.cpp SDSP::DoDMA.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_ifx_DoDMA(c:*mut c_void,a:i64,b:i64,d:i64,regs:*mut u16,exec:Exec)->i64{let q=|k,a,b,d,e,f|exec(c,k,a,b,d,e,f);let g=|i:u16|*regs.add(i as usize);let s=|i:u16,v:u16|{*regs.add(i as usize)=v;};let addr=((g(DSP_DSMAH)as u32)<<16)|(g(DSP_DSMAL)as u32);let ctl=g(DSP_DSCR);let dsp=g(DSP_DSPA).wrapping_mul(2);let len=g(DSP_DSBL);if len>0x4000{return 0;}let ptr=match ctl&3{1=>wprt_dsp_ifx_DDMAOut(c,dsp as i64,addr as i64,len as i64,regs,exec),0=>wprt_dsp_ifx_DDMAIn(c,dsp as i64,addr as i64,len as i64,regs,exec),3=>wprt_dsp_ifx_IDMAOut(c,dsp as i64,addr as i64,len as i64,regs,exec),2=>wprt_dsp_ifx_IDMAIn(c,dsp as i64,addr as i64,len as i64,regs,exec),_=>0};if ptr!=0{q(15,ctl as i64,addr as i64,dsp as i64,len as i64,ptr);}0}
