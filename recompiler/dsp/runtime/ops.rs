// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// Copyright 2005 Duddie
// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted source port of engine/include/wp/dsp_inline.h:17-215.
// UNCOMPILED/UNINTEGRATED. Regs follows the explicit POD DSP_Regs,NOT the SDSP SDK class.
include!("status_constants.rs");
include!("register_constants.rs");

// Owned ABI view of explicit SDK fields,filled by native pointer access;SDSP itself stays opaque.
// Pointer-to-pointer RAM fields preserve fresh source pointer loads after SDK callbacks.
#[repr(C)]pub struct Context{pub state:*mut std::ffi::c_void,pub interpreter:*mut std::ffi::c_void,pub regs:*mut Regs,pub pc:*mut u16,pub control:*mut u16,pub exceptions:*mut u8,pub dram:*const *mut u16,pub iram:*const *mut u16,pub irom:*const *mut u16}
extern "C"{fn wphw_ax_read_dmem(state:*mut std::ffi::c_void,address:u16)->u16;fn wphw_ax_write_dmem(state:*mut std::ffi::c_void,address:u16,value:u16);fn wphw_ax_read_imem(state:*mut std::ffi::c_void,address:u16)->u16;fn wphw_ax_pop_stack(state:*mut std::ffi::c_void,which:i32)->u16;fn wphw_ax_store_stack(state:*mut std::ffi::c_void,which:i32,value:u16);}

#[repr(C)]#[derive(Clone,Copy)]pub struct ProductParts{pub l:u16,pub m:u16,pub h:u16,pub m2:u16}
#[repr(C)]#[derive(Clone,Copy)]pub union Product{pub val:u64,pub parts:ProductParts}
#[repr(C)]#[derive(Clone,Copy)]pub struct AuxParts{pub l:u16,pub h:u16}
#[repr(C)]#[derive(Clone,Copy)]pub union Aux{pub val:u32,pub parts:AuxParts}
#[repr(C)]#[derive(Clone,Copy)]pub struct AccumParts{pub l:u16,pub m:u16,pub h:u32}
#[repr(C)]#[derive(Clone,Copy)]pub union Accum{pub val:u64,pub parts:AccumParts}
// port of third_party/dolphin/.../DSPCore.h DSP_Regs public POD declarations.
#[repr(C)]pub struct Regs{pub ar:[u16;4],pub ix:[u16;4],pub wr:[u16;4],pub st:[u16;4],pub cr:u16,pub sr:u16,pub prod:Product,pub ax:[Aux;2],pub ac:[Accum;2]}

// Source array-index preconditions stay unchecked,as in the native DSP helper.
// Raw register pointers preserve source aliasing in nested helpers and across SDK callbacks.
// port of dsp_inline.h acc.
#[inline(always)]pub unsafe fn acc(s:*const Regs,reg:i32)->i64{(*(*s).ac.as_ptr().offset(reg as isize)).val as i64}
// port of dsp_inline.h set_acc;explicit 40-bit sign extension,matching native shift operations.
#[inline(always)]pub unsafe fn set_acc(s:*mut Regs,reg:i32,value:i64){(*(*s).ac.as_mut_ptr().offset(reg as isize)).val=(value.wrapping_shl(24)>>24)as u64}
// port of dsp_inline.h acx.
#[inline(always)]pub unsafe fn acx(s:*const Regs,reg:i32)->i64{(*(*s).ax.as_ptr().offset(reg as isize)).val as i32 as i64}
// port of dsp_inline.h prod.
#[inline(always)]pub unsafe fn prod(s:*const Regs)->i64{let p=(*s).prod.parts;let high=(p.h as u8 as i8 as i64)<<32;let low=((p.m as i64+p.m2 as i64)<<16)|p.l as i64;high+low}
// port of dsp_inline.h set_prod.
#[inline(always)]pub unsafe fn set_prod(s:*mut Regs,value:i64){(*s).prod.val=(value&0x000000ffffffffff)as u64}
// port of dsp_inline.h multiply,including unsigned and mixed signedness before scaling.
#[inline(always)]pub unsafe fn multiply(s:*const Regs,a:u16,b:u16,sign:i32)->i64{let mut product=if sign==1&&((*s).sr&SR_MUL_UNSIGNED)!=0{(a as u32*b as u32)as i64}else if sign==2&&((*s).sr&SR_MUL_UNSIGNED)!=0{(a as i32*b as i16 as i32)as i64}else{(a as i16 as i32*b as i16 as i32)as i64};if (*s).sr&SR_MUL_MODIFY==0{product=product.wrapping_shl(1)}product}
// port of dsp_inline.h multiply_mulx;source operand swap in the mixed-sign case is retained.
#[inline(always)]pub unsafe fn multiply_mulx(s:*const Regs,axh0:i32,axh1:i32,val1:u16,val2:u16)->i64{if axh0==0&&axh1==0{return multiply(s,val1,val2,1)}if axh0==0&&axh1==1{return multiply(s,val1,val2,2)}if axh0==1&&axh1==0{return multiply(s,val2,val1,2)}multiply(s,val1,val2,0)}
// port of dsp_inline.h cmp_bits64.
#[inline(always)]pub fn cmp_bits64(value:i64,carry:bool,overflow:bool)->u16{let mut bits=carry as u16;bits|=(overflow as u16)<<1;bits|=((value==0)as u16)<<2;bits|=((value<0)as u16)<<3;bits|=((value!=value as i32 as i64)as u16)<<4;let top=value as u64&0xc0000000;bits|=((top==0||top==0xc0000000)as u16)<<5;bits}
// port of dsp_inline.h carry_add.
#[inline(always)]pub fn carry_add(val1:i64,result:i64)->bool{(val1 as u64)>(result as u64)}
// port of dsp_inline.h overflow_add.
#[inline(always)]pub fn overflow_add(val1:i64,val2:i64,result:i64)->bool{((val1^result)&(val2^result))<0}
// port of dsp_inline.h carry_sub.
#[inline(always)]pub fn carry_sub(val1:i64,result:i64)->bool{(val1 as u64)>=(result as u64)}
// port of dsp_inline.h overflow_sub.
#[inline(always)]pub fn overflow_sub(val1:i64,val2:i64,result:i64)->bool{((val1^result)&(val2.wrapping_neg()^result))<0}
// port of dsp_inline.h sticky.
#[inline(always)]pub unsafe fn sticky(s:*mut Regs,overflow:bool){(*s).sr|=(overflow as u16)*SR_OVERFLOW_STICKY}
// port of dsp_inline.h set_cmp_bits.
#[inline(always)]pub unsafe fn set_cmp_bits(s:*mut Regs,value:i64,carry:bool,overflow:bool){(*s).sr=((*s).sr&!SR_CMP_MASK)|cmp_bits64(value,carry,overflow)}
// port of dsp_inline.h flags64;native defaults carry=false,overflow=false must be supplied by the emitter.
#[inline(always)]pub unsafe fn flags64(s:*mut Regs,value:i64,carry:bool,overflow:bool){set_cmp_bits(s,value,carry,overflow);sticky(s,overflow)}
// port of dsp_inline.h flags64_add.
#[inline(always)]pub unsafe fn flags64_add(s:*mut Regs,val1:i64,val2:i64,result:i64){flags64(s,result,carry_add(val1,result),overflow_add(val1,val2,result))}
// port of dsp_inline.h flags64_sub.
#[inline(always)]pub unsafe fn flags64_sub(s:*mut Regs,val1:i64,val2:i64,result:i64){flags64(s,result,carry_sub(val1,result),overflow_sub(val1,val2,result))}
// port of dsp_inline.h flags16;the previous sticky overflow bit is retained outside the compare mask.
#[inline(always)]pub unsafe fn flags16(s:*mut Regs,value:i16,carry:bool,overflow:bool,over_s32:bool){let mut bits=carry as u16;bits|=(overflow as u16)*(SR_OVERFLOW|SR_OVERFLOW_STICKY);bits|=((value==0)as u16)<<2;bits|=((value<0)as u16)<<3;bits|=(over_s32 as u16)<<4;let top=(value as u16)>>14;bits|=((top==0||top==3)as u16)<<5;(*s).sr=((*s).sr&!SR_CMP_MASK)|bits}
// port of dsp_inline.h logic_zero.
#[inline(always)]pub unsafe fn logic_zero(s:*mut Regs,value:bool){(*s).sr=if value{(*s).sr|SR_LOGIC_ZERO}else{(*s).sr&!SR_LOGIC_ZERO}}
// port of dsp_inline.h condition;all 16 condition codes,including the default unconditional case.
#[inline(always)]pub unsafe fn condition(s:*const Regs,code:i32)->bool{let sr=(*s).sr;let carry=sr&SR_CARRY!=0;let overflow=sr&SR_OVERFLOW!=0;let over32=sr&SR_OVER_S32!=0;let less=overflow!=(sr&SR_SIGN!=0);let zero=sr&SR_ARITH_ZERO!=0;let logic=sr&SR_LOGIC_ZERO!=0;let b=!(over32||sr&SR_TOP2BITS!=0)||zero;match code&0xf{0=>!less,1=>less,2=>!less&&!zero,3=>less||zero,4=>!zero,5=>zero,6=>!carry,7=>carry,8=>!over32,9=>over32,10=>!b,11=>b,12=>!logic,13=>logic,14=>overflow,_=>true}}
// port of dsp_inline.h over_s32.
#[inline(always)]pub fn over_s32(value:i64)->bool{value!=value as i32 as i64}
// port of dsp_inline.h increment_ar,including unsigned wrapping before narrowing to u16.
#[inline(always)]pub unsafe fn increment_ar(s:*const Regs,reg:i32)->u16{let ar=*(*s).ar.as_ptr().offset(reg as isize)as u32;let wr=*(*s).wr.as_ptr().offset(reg as isize)as u32;let mut nar=ar+1;if (nar^ar)>((wr|1)<<1){nar=nar.wrapping_sub(wr+1)}nar as u16}
// port of dsp_inline.h decrement_ar.
#[inline(always)]pub unsafe fn decrement_ar(s:*const Regs,reg:i32)->u16{let ar=*(*s).ar.as_ptr().offset(reg as isize)as u32;let wr=*(*s).wr.as_ptr().offset(reg as isize)as u32;let mut nar=ar+wr;if((nar^ar)&((wr|1)<<1))>wr{nar=nar.wrapping_sub(wr+1)}nar as u16}
// port of dsp_inline.h increase_ar;C++ signed ix participates in unsigned arithmetic/bitwise expressions.
#[inline(always)]pub unsafe fn increase_ar(s:*const Regs,reg:i32,ix_value:i16)->u16{let ar=*(*s).ar.as_ptr().offset(reg as isize)as u32;let wr=*(*s).wr.as_ptr().offset(reg as isize)as u32;let ix=ix_value as i32;let mx=(wr|1)<<1;let mut nar=ar.wrapping_add(ix as u32);let dar=(nar^ar^ix as u32)&mx;if ix>=0{if dar>wr{nar=nar.wrapping_sub(wr+1)}}else if((nar.wrapping_add(wr).wrapping_add(1)^nar)&dar)<=wr{nar=nar.wrapping_add(wr+1)}nar as u16}
// port of dsp_inline.h read_dmem;SDK word arrays are already decoded host u16 values.
#[inline(always)]pub unsafe fn read_dmem(s:&Context,address:u16)->u16{if address>>12==0{*(*s.dram).add((address&DSP_DRAM_MASK)as usize)}else{wphw_ax_read_dmem(s.state,address)}}
// port of dsp_inline.h write_dmem;non-DRAM addresses keep the exact SDK device path.
#[inline(always)]pub unsafe fn write_dmem(s:&Context,address:u16,value:u16){if address>>12==0{*(*s.dram).add((address&DSP_DRAM_MASK)as usize)=value}else{wphw_ax_write_dmem(s.state,address,value)}}
// port of dsp_inline.h read_imem,including the source fallback for all other address spaces.
#[inline(always)]pub unsafe fn read_imem(s:&Context,address:u16)->u16{match address>>12{0=>*(*s.iram).add((address&DSP_IRAM_MASK)as usize),8=>*(*s.irom).add((address&DSP_IROM_MASK)as usize),_=>wphw_ax_read_imem(s.state,address)}}
// port of dsp_inline.h read_reg. SDK stack side effects are called with no live Regs borrow.
#[inline(always)]pub unsafe fn read_reg(s:&Context,reg:i32)->u16{let r=s.regs;match reg&0x1f{
 DSP_REG_ST0|DSP_REG_ST1|DSP_REG_ST2|DSP_REG_ST3=>wphw_ax_pop_stack(s.state,(reg&0x1f)-DSP_REG_ST0),
 DSP_REG_AR0|DSP_REG_AR1|DSP_REG_AR2|DSP_REG_AR3=>(*r).ar[(reg&3)as usize],
 DSP_REG_IX0|DSP_REG_IX1|DSP_REG_IX2|DSP_REG_IX3=>(*r).ix[(reg&3)as usize],
 DSP_REG_WR0|DSP_REG_WR1|DSP_REG_WR2|DSP_REG_WR3=>(*r).wr[(reg&3)as usize],
 DSP_REG_ACH0|DSP_REG_ACH1=>(*r).ac[(reg&1)as usize].parts.h as u16,
 DSP_REG_CR=>(*r).cr,DSP_REG_SR=>(*r).sr,
 DSP_REG_PRODL=>(*r).prod.parts.l,DSP_REG_PRODM=>(*r).prod.parts.m,DSP_REG_PRODH=>(*r).prod.parts.h,DSP_REG_PRODM2=>(*r).prod.parts.m2,
 DSP_REG_AXL0|DSP_REG_AXL1=>(*r).ax[(reg&1)as usize].parts.l,DSP_REG_AXH0|DSP_REG_AXH1=>(*r).ax[(reg&1)as usize].parts.h,
 DSP_REG_ACL0|DSP_REG_ACL1=>(*r).ac[(reg&1)as usize].parts.l,
 DSP_REG_ACM0|DSP_REG_ACM1=>{if (*r).sr&SR_40_MODE_BIT!=0{let value=acc(r,reg&1);if value!=value as i32 as i64{return if value>0{0x7fff}else{0x8000}}}(*r).ac[(reg&1)as usize].parts.m},_=>0}}
// port of dsp_inline.h write_reg;source high-register sign extension and reserved-bit masks retained.
#[inline(always)]pub unsafe fn write_reg(s:&Context,reg:i32,value:u16){let r=s.regs;match reg&0x1f{
 DSP_REG_ACH0|DSP_REG_ACH1=>(*r).ac[(reg&1)as usize].parts.h=value as i8 as i32 as u32,
 DSP_REG_ST0|DSP_REG_ST1|DSP_REG_ST2|DSP_REG_ST3=>wphw_ax_store_stack(s.state,(reg&0x1f)-DSP_REG_ST0,value),
 DSP_REG_AR0|DSP_REG_AR1|DSP_REG_AR2|DSP_REG_AR3=>(*r).ar[(reg&3)as usize]=value,
 DSP_REG_IX0|DSP_REG_IX1|DSP_REG_IX2|DSP_REG_IX3=>(*r).ix[(reg&3)as usize]=value,
 DSP_REG_WR0|DSP_REG_WR1|DSP_REG_WR2|DSP_REG_WR3=>(*r).wr[(reg&3)as usize]=value,
 DSP_REG_CR=>(*r).cr=value&0x00ff,DSP_REG_SR=>(*r).sr=value&!SR_100,
 DSP_REG_PRODL=>(*r).prod.parts.l=value,DSP_REG_PRODM=>(*r).prod.parts.m=value,DSP_REG_PRODH=>(*r).prod.parts.h=value&0x00ff,DSP_REG_PRODM2=>(*r).prod.parts.m2=value,
 DSP_REG_AXL0|DSP_REG_AXL1=>(*r).ax[(reg&1)as usize].parts.l=value,DSP_REG_AXH0|DSP_REG_AXH1=>(*r).ax[(reg&1)as usize].parts.h=value,
 DSP_REG_ACL0|DSP_REG_ACL1=>(*r).ac[(reg&1)as usize].parts.l=value,DSP_REG_ACM0|DSP_REG_ACM1=>(*r).ac[(reg&1)as usize].parts.m=value,_=>{}}}
// port of dsp_inline.h extend_acc:the reg test is intentionally NOT masked to five bits.
#[inline(always)]pub unsafe fn extend_acc(s:&Context,reg:i32){if reg!=DSP_REG_ACM0&&reg!=DSP_REG_ACM1{return}let r=s.regs;if (*r).sr&SR_40_MODE_BIT==0{return}let value=(*r).ac[(reg&1)as usize].parts.m;(*r).ac[(reg&1)as usize].parts.h=if value&0x8000!=0{0xffffffff}else{0};(*r).ac[(reg&1)as usize].parts.l=0}
