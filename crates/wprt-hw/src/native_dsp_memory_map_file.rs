// Copyright 2008 Dolphin Emulator Project
// Copyright 2004 Duddie & Tratax
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted complete DSPMemoryMap.cpp member replacement, pinned GNU metadata.
#[cfg(not(all(target_os="windows",target_env="gnu",target_arch="x86_64")))]
compile_error!("DSPMemoryMap member exports require pinned Windows GNU x64 metadata");
pub(crate) mod layout { include!(env!("WPRT_DSP_MEMORY_MAP_LAYOUT")); }
const _:()=assert!(layout::SIZE_BYTES==std::mem::size_of::<u8>());
extern "C" {
 #[link_name="_ZN3DSP4SDSP7ReadIFXEt"] fn native_read_ifx(this:*mut u8,address:u16)->u16;
 #[link_name="_ZN3DSP4SDSP8WriteIFXEjt"] fn native_write_ifx(this:*mut u8,address:u32,value:u16);
 #[link_name="_ZN3DSP13GetOpTemplateEt"] fn native_op_template(op:u16)->*const u8;
}
unsafe fn memory(this:*const u8,offset:usize)->*mut u16 {this.add(offset).cast::<*mut u16>().read()}
unsafe extern "C" fn read_ifx(this:*mut std::ffi::c_void,address:u16)->u16 {native_read_ifx(this.cast(),address)}
unsafe extern "C" fn write_ifx(this:*mut std::ffi::c_void,address:u16,value:u16){native_write_ifx(this.cast(),address as u32,value)}
unsafe extern "C" fn peek_callback(this:*mut std::ffi::c_void)->u16 {peek(this.cast())}
unsafe extern "C" fn instruction_size(op:u16)->u8 {native_op_template(op).add(layout::SIZE_OFFSET).read()}
#[export_name="_ZNK3DSP4SDSP8ReadIMEMEt"]
pub unsafe extern "C" fn read_imem(this:*const u8,address:u16)->u16 {
 crate::native_dsp_memory_map::wprt_dsp_mem_read_imem(memory(this,layout::IRAM_OFFSET),memory(this,layout::IROM_OFFSET),address)
}
#[export_name="_ZN3DSP4SDSP8ReadDMEMEt"]
pub unsafe extern "C" fn read_dmem(this:*mut u8,address:u16)->u16 {
 crate::native_dsp_memory_map::wprt_dsp_mem_read_dmem(memory(this,layout::DRAM_OFFSET),memory(this,layout::COEF_OFFSET),this.cast(),address,read_ifx)
}
#[export_name="_ZN3DSP4SDSP9WriteDMEMEtt"]
pub unsafe extern "C" fn write_dmem(this:*mut u8,address:u16,value:u16){
 crate::native_dsp_memory_map::wprt_dsp_mem_write_dmem(memory(this,layout::DRAM_OFFSET),this.cast(),address,value,write_ifx)
}
#[export_name="_ZN3DSP4SDSP16FetchInstructionEv"]
pub unsafe extern "C" fn fetch(this:*mut u8)->u16 {
 crate::native_dsp_memory_map::wprt_dsp_mem_fetch(this.add(layout::PC_OFFSET).cast(),this.cast(),peek_callback)
}
#[export_name="_ZNK3DSP4SDSP15PeekInstructionEv"]
pub unsafe extern "C" fn peek(this:*const u8)->u16 {
 crate::native_dsp_memory_map::wprt_dsp_mem_peek(this.add(layout::PC_OFFSET).cast(),memory(this,layout::IRAM_OFFSET),memory(this,layout::IROM_OFFSET))
}
#[export_name="_ZN3DSP4SDSP15SkipInstructionEv"]
pub unsafe extern "C" fn skip(this:*mut u8){
 crate::native_dsp_memory_map::wprt_dsp_mem_skip(this.add(layout::PC_OFFSET).cast(),this.cast(),peek_callback,instruction_size)
}
