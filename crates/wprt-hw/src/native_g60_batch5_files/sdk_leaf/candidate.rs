// Claude G60: whole-member Rust port of hybrid sdk_leaf.cpp (AX SDSP accessor leaves). Offsets compiler-derived (probe.cpp -> layout.rs).
#![allow(clippy::missing_safety_doc)]
use core::ffi::{c_int, c_void};
include!("layout.rs");
extern "C-unwind" {
    #[link_name = "_ZN3DSP4SDSP8ReadDMEMEt"] fn read_dmem(this: *mut u8, a: u16) -> u16;
    #[link_name = "_ZN3DSP4SDSP9WriteDMEMEtt"] fn write_dmem(this: *mut u8, a: u16, v: u16);
    #[link_name = "_ZNK3DSP4SDSP8ReadIMEMEt"] fn read_imem(this: *const u8, a: u16) -> u16;
    #[link_name = "_ZN3DSP4SDSP8PopStackENS_13StackRegisterE"] fn pop_stack(this: *mut u8, w: c_int) -> u16;
    #[link_name = "_ZN3DSP4SDSP10StoreStackENS_13StackRegisterEt"] fn store_stack(this: *mut u8, w: c_int, v: u16);
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ax_context(interpreter: *mut c_void, state: *mut c_void, out: *mut [*mut u8; 9]) {
    let s = state as *mut u8;
    *out = [s, interpreter as *mut u8, s.add(S_R), s.add(S_PC), s.add(S_CONTROL), s.add(S_EXC), s.add(S_DRAM), s.add(S_IRAM), s.add(S_IROM)];
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ax_read_dmem(state: *mut c_void, address: u16) -> u16 { read_dmem(state as *mut u8, address) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ax_write_dmem(state: *mut c_void, address: u16, value: u16) { write_dmem(state as *mut u8, address, value) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ax_read_imem(state: *mut c_void, address: u16) -> u16 { read_imem(state as *const u8, address) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ax_pop_stack(state: *mut c_void, which: i32) -> u16 { pop_stack(state as *mut u8, which) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ax_store_stack(state: *mut c_void, which: i32, value: u16) { store_stack(state as *mut u8, which, value) }
