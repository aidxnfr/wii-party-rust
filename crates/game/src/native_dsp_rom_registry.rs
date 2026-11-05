// PRIVATE:source-derived generated DSP registry + same private ROM blobs as the accepted native library.
// AI-assisted port of clean-gen/dsp/registry.cpp and pristine engine/src/audio/dsp_roms.cpp.
use std::ffi::c_void;
#[repr(C)]pub struct RomBlob{pub data:*const u8,pub size:usize}
// port of dsp_roms.cpp WP_EMBED for the Windows target:exact section/alignment/incbin/end-marker bytes.
std::arch::global_asm!(concat!(".section .rdata,\"dr\"\n.balign 16\n.globl wp_dsp_irom\nwp_dsp_irom:\n.incbin \"",env!("WP_PRIVATE_DSP_IROM"),"\"\n.globl wp_dsp_irom_end\nwp_dsp_irom_end:\n.byte 0\n.text\n"));
std::arch::global_asm!(concat!(".section .rdata,\"dr\"\n.balign 16\n.globl wp_dsp_coef\nwp_dsp_coef:\n.incbin \"",env!("WP_PRIVATE_DSP_COEF"),"\"\n.globl wp_dsp_coef_end\nwp_dsp_coef_end:\n.byte 0\n.text\n"));
extern "C"{static wp_dsp_irom:u8;static wp_dsp_irom_end:u8;static wp_dsp_coef:u8;static wp_dsp_coef_end:u8;
#[link_name="_ZN2wp3dsp9generated6run_axERN3DSP11Interpreter11InterpreterERNS2_4SDSPEiRb"]fn run_ax(in_:*mut c_void,state:*mut c_void,cycles:i32,idle:*mut bool)->i32;}
// port of wp::dsp::instruction_rom;existing public POD returned with the same target C ABI.
#[export_name="_ZN2wp3dsp15instruction_romEv"]pub extern "C" fn instruction_rom()->RomBlob{let start=std::ptr::addr_of!(wp_dsp_irom);let end=std::ptr::addr_of!(wp_dsp_irom_end);RomBlob{data:start,size:(end as usize).wrapping_sub(start as usize)}}
// port of wp::dsp::coefficient_rom.
#[export_name="_ZN2wp3dsp15coefficient_romEv"]pub extern "C" fn coefficient_rom()->RomBlob{let start=std::ptr::addr_of!(wp_dsp_coef);let end=std::ptr::addr_of!(wp_dsp_coef_end);RomBlob{data:start,size:(end as usize).wrapping_sub(start as usize)}}
#[repr(C)]pub struct TranslatedCode{pub checksum:u32,pub function:unsafe extern "C" fn(*mut c_void,*mut c_void,i32,*mut bool)->i32}
// port of generated registry.cpp:checksum and native typed AX adapter symbol extracted/confirmed by source+nm.
#[export_name="_ZN2wp3dsp17g_translated_codeE"]pub static REGISTRY:[TranslatedCode;1]=[TranslatedCode{checksum:3155333021,function:run_ax}];
#[export_name="_ZN2wp3dsp23g_translated_code_countE"]pub static REGISTRY_COUNT:usize=REGISTRY.len();
