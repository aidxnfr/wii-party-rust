// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of Dolphin Core/DSP/DSPStacks.cpp,from the recomp's pristine third_party source.
// Named primitive-array pointers supplied by the native SDK;no SDSP class layout is inferred.
// port of SDSP::StoreStack,including u8 increment before source DSP_STACK_MASK(0x1f).
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_stack_store(st:*mut u16,ptrs:*mut u8,stacks:*mut u16,index:usize,value:u16){
 let ptr=ptrs.add(index);*ptr=(*ptr).wrapping_add(1);*ptr&=0x1f;stacks.add(index*0x20+*ptr as usize).write(st.add(index).read());st.add(index).write(value);
}
// port of SDSP::PopStack;read result,restore prior top,then decrement+mask in the exact source order.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_stack_pop(st:*mut u16,ptrs:*mut u8,stacks:*const u16,index:usize)->u16{
 let value=st.add(index).read();let ptr=ptrs.add(index);st.add(index).write(stacks.add(index*0x20+*ptr as usize).read());*ptr=(*ptr).wrapping_sub(1);*ptr&=0x1f;value
}
