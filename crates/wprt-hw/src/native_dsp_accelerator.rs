// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPAccelerator.cpp policy port;native typed fields/virtual memory+exception/PointerWrap SDK.
use std::ffi::c_void;
pub type Exec=unsafe extern "C" fn(*mut c_void,u32,i64,i64)->i64;
// port of pristine DSPAccelerator.cpp Accelerator::GetCurrentSample.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_GetCurrentSample(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};match g(3)as u16&3{0=>{let val=q(0,(g(2)as u32>>1)as i64,0)as u16;if g(2)as u32&1!=0{(val&15)as i64}else{(val>>4)as i64}},1=>q(0,g(2)as u32 as i64,0)as u8 as i64,2=>{let hi=q(0,(g(2)as u32).wrapping_mul(2)as i64,0)as u8 as u16;let lo=q(0,(g(2)as u32).wrapping_mul(2).wrapping_add(1)as i64,0)as u8 as u16;((hi<<8)|lo)as i64},_=>0}}
// port of pristine DSPAccelerator.cpp Accelerator::ReadRaw.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_ReadRaw(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};let val=wprt_dsp_acc_GetCurrentSample(c,0,exec);let addr=g(2)as u32;if g(3)as u16&3!=3{s(2,addr.wrapping_add(1)as i64);}else{s(2,((addr&!3)|(addr.wrapping_add(1)&3))as i64);}if (g(2)as u32).wrapping_sub(1)==g(1)as u32{s(2,g(0));q(2,0,0);}wprt_dsp_acc_SetCurrentAddress(c,g(2),exec);val}
// port of pristine DSPAccelerator.cpp Accelerator::WriteRaw.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_WriteRaw(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};let val=a as u16;if g(2)as u32&0x80000000!=0{q(1,(g(2)as u32).wrapping_mul(2)as i64,(val>>8)as i64);q(1,(g(2)as u32).wrapping_mul(2).wrapping_add(1)as i64,(val&255)as i64);s(2,(g(2)as u32).wrapping_add(1)as i64);q(3,0,0);}0}
// port of pristine DSPAccelerator.cpp Accelerator::ReadSample.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_ReadSample(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};if g(9)!=0{return 0;}let coefs=a as *const i16;let first_decode=(g(3)as u16>>2)&3;let mut raw=if first_decode==1||first_decode==3{g(8)as u16 as i16}else{wprt_dsp_acc_GetCurrentSample(c,0,exec)as u16 as i16};let idx=((g(7)as u16>>4)&7)as usize;let c1=*coefs.add(idx*2)as i32;let c2=*coefs.add(idx*2+1)as i32;let decode=(g(3)as u16>>2)&3;let mut step=2u8;let val:u16;if decode==0{raw &=15;let scale=1i32<<((g(7)as u16)&15);if raw>=8{raw-=16;}let hist=0x400i32.wrapping_add(c1.wrapping_mul(g(5)as i16 as i32)).wrapping_add(c2.wrapping_mul(g(6)as i16 as i32));let v=scale.wrapping_mul(raw as i32).wrapping_add(hist>>11);val=v.clamp(-32768,32767)as i16 as u16;s(6,g(5));s(5,val as i16 as i64);s(2,(g(2)as u32).wrapping_add(1)as i64);if g(1)as u32&15==0 && g(2)as u32==g(1)as u32{s(2,(g(0)as u32).wrapping_add(1)as i64);}else if g(1)as u32&15==1 &&g(2)as u32==(g(1)as u32).wrapping_sub(1){s(2,g(0));}else if g(2)as u32&15==0{let ps=q(0,((g(2)as u32&!15)>>1)as i64,0)as u8;s(7,ps as i64);s(2,(g(2)as u32).wrapping_add(2)as i64);step+=2;}}else{let shift=match(g(3)as u16>>4)&3{0=>11,1=>0,2=>16,_=>0};let v=((g(4)as i16 as i32).wrapping_mul(raw as i32)>>shift).wrapping_add((c1.wrapping_mul(g(5)as i16 as i32)>>shift).wrapping_add(c2.wrapping_mul(g(6)as i16 as i32)>>shift));val=v as i16 as u16;s(6,g(5));s(5,val as i16 as i64);if (g(3)as u16>>2)&3!=1{s(2,(g(2)as u32).wrapping_add(1)as i64);}}if g(2)as u32==(g(1)as u32).wrapping_add(step as u32).wrapping_sub(1){s(2,g(0));s(9,1);q(4,0,0);}wprt_dsp_acc_SetCurrentAddress(c,g(2),exec);val as i64}
// port of pristine DSPAccelerator.cpp Accelerator::DoState.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_DoState(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};for field in [0,1,2,3,5,6,7,9]{if q(5,a,field)==0{break;}}0}
// port of pristine DSPAccelerator.cpp Accelerator::SetStartAddress.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetStartAddress(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(0,((a as u32)&0x3fffffff)as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetEndAddress.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetEndAddress(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(1,((a as u32)&0x3fffffff)as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetCurrentAddress.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetCurrentAddress(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(2,((a as u32)&0xbfffffff)as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetSampleFormat.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetSampleFormat(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(3,a as u16 as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetGain.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetGain(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(4,a as i16 as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetYn1.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetYn1(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(5,a as i16 as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetYn2.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetYn2(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(6,a as i16 as i64);s(9,0);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetPredScale.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetPredScale(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(7,(a as u16&0x7f)as i64);0}
// port of pristine DSPAccelerator.cpp Accelerator::SetInput.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_acc_SetInput(c:*mut c_void,a:i64,exec:Exec)->i64{let q=|k,a,b|exec(c,k,a,b);let g=|f|exec(c,100,f,0);let s=|f,v|{exec(c,101,f,v);};s(8,a as u16 as i64);0}
