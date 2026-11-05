// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPStacks.cpp replacement; native member wrappers are removed.
// Field offsets are emitted by the pinned C++ compiler/header, never inferred.
#[cfg(not(all(target_os="windows",target_env="gnu",target_arch="x86_64")))]
compile_error!("DSPStacks member exports require the pinned Windows GNU x64 header metadata");
mod layout { include!(env!("WPRT_DSP_STACKS_LAYOUT")); }
const _:()=assert!(layout::STACK_REGISTER_BYTES==std::mem::size_of::<i32>());
#[export_name="_ZN3DSP4SDSP10StoreStackENS_13StackRegisterEt"]
pub unsafe extern "C" fn store(this:*mut u8,index:i32,value:u16){
 crate::native_dsp_stacks::wprt_dsp_stack_store(this.add(layout::ST_OFFSET).cast(),this.add(layout::PTRS_OFFSET),this.add(layout::STACKS_OFFSET).cast(),index as usize,value);
}
#[export_name="_ZN3DSP4SDSP8PopStackENS_13StackRegisterE"]
pub unsafe extern "C" fn pop(this:*mut u8,index:i32)->u16{
 crate::native_dsp_stacks::wprt_dsp_stack_pop(this.add(layout::ST_OFFSET).cast(),this.add(layout::PTRS_OFFSET),this.add(layout::STACKS_OFFSET).cast(),index as usize)
}
