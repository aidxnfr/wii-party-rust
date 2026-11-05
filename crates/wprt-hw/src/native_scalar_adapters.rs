// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete scalar adapter ports;accepted policy implementations unchanged.
// Mangled exports mechanically confirmed against OWN nativearchive nm/public source declarations.
use std::ffi::c_char;
extern "C"{
#[link_name="wprt_hollywood_owns"]fn target_0(address:u32)->bool;
#[link_name="wprt_hollywood_read32"]fn target_1(address:u32)->u32;
#[link_name="wprt_hollywood_write32"]fn target_2(address:u32,value:u32)->();
#[link_name="wprt_si_read32"]fn target_3(address:u32)->u32;
#[link_name="wprt_si_write32"]fn target_4(address:u32,value:u32)->();
#[link_name="wprt_si_poll"]fn target_5()->();
#[link_name="wprt_si_pending"]fn target_6()->bool;
#[link_name="wprt_io_read16"]fn target_7(address:u32)->u16;
#[link_name="wprt_io_read32"]fn target_8(address:u32)->u32;
#[link_name="wprt_io_write8"]fn target_9(address:u32,value:u8)->();
#[link_name="wprt_io_write16"]fn target_10(address:u32,value:u16)->();
#[link_name="wprt_io_write32"]fn target_11(address:u32,value:u32)->();
#[link_name="wprt_io_write64"]fn target_12(address:u32,value:u64)->();
#[link_name="_ZN2wp3dsp6read16Ej"]fn target_13(address:u32)->u16;
#[link_name="_ZN2wp3dsp7write16Ejt"]fn target_14(address:u32,value:u16)->();
#[link_name="_ZN2wp3ipc6read32Ej"]fn target_15(address:u32)->u32;
#[link_name="_ZN2wp3ipc7write32Ejj"]fn target_16(address:u32,value:u32)->();
#[link_name="_ZN2wp2gx4pushEyj"]fn target_17(value:u64,bytes:u32)->();
#[link_name="_ZN2wp2gx26write_pixel_engine_controlEt"]fn target_18(value:u16)->();
#[link_name="_ZN2wp3log7enabledEv"]fn target_19()->bool;
#[link_name="_ZN2wp5audio6updateEv"]fn target_20()->();
#[link_name="_ZN2wp3dsp6updateEv"]fn target_21()->();
#[link_name="_ZN2wp3dsp17pending_interruptEv"]fn target_22()->u32;
#[link_name="_ZN2wp3ipc6updateEv"]fn target_23()->();
#[link_name="_ZN2wp3ipc17interrupt_pendingEv"]fn target_24()->bool;
#[link_name="_ZN2wp2gx7processEv"]fn target_25()->();
#[link_name="_ZN2wp2gx21take_finish_interruptEv"]fn target_26()->bool;
#[link_name="_ZN2wp3exi17pending_interruptEv"]fn target_27()->u32;
#[link_name="_ZN2wp2si17interrupt_pendingEv"]fn target_28()->bool;
#[link_name="_ZN2wp2si4pollEv"]fn target_29()->();
#[link_name="_ZN2wp5video7presentEv"]fn target_30()->();
#[link_name="_ZN2wp4mmio6read16Ej"]fn target_31(a:u32)->u16;
#[link_name="_ZN2wp4mmio6read32Ej"]fn target_32(a:u32)->u32;
#[link_name="_ZN2wp4mmio6write8Ejh"]fn target_33(a:u32,v:u8)->();
#[link_name="_ZN2wp4mmio7write16Ejt"]fn target_34(a:u32,v:u16)->();
#[link_name="_ZN2wp4mmio7write32Ejj"]fn target_35(a:u32,v:u32)->();
#[link_name="_ZN2wp4mmio7write64Ejy"]fn target_36(a:u32,v:u64)->();
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
}
// port of cpp/patched/hollywood.cpp wp::hollywood::owns.
#[export_name="_ZN2wp9hollywood4ownsEj"]pub unsafe extern "C" fn entry_0(address:u32)->bool{target_0(address)}
// port of cpp/patched/hollywood.cpp wp::hollywood::read32.
#[export_name="_ZN2wp9hollywood6read32Ej"]pub unsafe extern "C" fn entry_1(address:u32)->u32{target_1(address)}
// port of cpp/patched/hollywood.cpp wp::hollywood::write32.
#[export_name="_ZN2wp9hollywood7write32Ejj"]pub unsafe extern "C" fn entry_2(address:u32,value:u32)->(){target_2(address,value);}
// port of cpp/patched/si.cpp wp::si::read32.
#[export_name="_ZN2wp2si6read32Ej"]pub unsafe extern "C" fn entry_3(address:u32)->u32{target_3(address)}
// port of cpp/patched/si.cpp wp::si::write32.
#[export_name="_ZN2wp2si7write32Ejj"]pub unsafe extern "C" fn entry_4(address:u32,value:u32)->(){target_4(address,value);}
// port of cpp/patched/si.cpp wp::si::poll.
#[export_name="_ZN2wp2si4pollEv"]pub unsafe extern "C" fn entry_5()->(){target_5();}
// port of cpp/patched/si.cpp wp::si::interrupt_pending.
#[export_name="_ZN2wp2si17interrupt_pendingEv"]pub unsafe extern "C" fn entry_6()->bool{target_6()}
// port of cpp/patched/io.cpp wp::mmio::read16.
#[export_name="_ZN2wp4mmio6read16Ej"]pub unsafe extern "C" fn entry_7(address:u32)->u16{target_7(address)}
// port of cpp/patched/io.cpp wp::mmio::read32.
#[export_name="_ZN2wp4mmio6read32Ej"]pub unsafe extern "C" fn entry_8(address:u32)->u32{target_8(address)}
// port of cpp/patched/io.cpp wp::mmio::write8.
#[export_name="_ZN2wp4mmio6write8Ejh"]pub unsafe extern "C" fn entry_9(address:u32,value:u8)->(){target_9(address,value);}
// port of cpp/patched/io.cpp wp::mmio::write16.
#[export_name="_ZN2wp4mmio7write16Ejt"]pub unsafe extern "C" fn entry_10(address:u32,value:u16)->(){target_10(address,value);}
// port of cpp/patched/io.cpp wp::mmio::write32.
#[export_name="_ZN2wp4mmio7write32Ejj"]pub unsafe extern "C" fn entry_11(address:u32,value:u32)->(){target_11(address,value);}
// port of cpp/patched/io.cpp wp::mmio::write64.
#[export_name="_ZN2wp4mmio7write64Ejy"]pub unsafe extern "C" fn entry_12(address:u32,value:u64)->(){target_12(address,value);}
// port of cpp/patched/io.cpp wphw_io_dsp_read16.
#[export_name="wphw_io_dsp_read16"]pub unsafe extern "C" fn entry_13(address:u32)->u16{target_13(address)}
// port of cpp/patched/io.cpp wphw_io_dsp_write16.
#[export_name="wphw_io_dsp_write16"]pub unsafe extern "C" fn entry_14(address:u32,value:u16)->(){target_14(address,value);}
// port of cpp/patched/io.cpp wphw_io_ipc_read32.
#[export_name="wphw_io_ipc_read32"]pub unsafe extern "C" fn entry_15(address:u32)->u32{target_15(address)}
// port of cpp/patched/io.cpp wphw_io_ipc_write32.
#[export_name="wphw_io_ipc_write32"]pub unsafe extern "C" fn entry_16(address:u32,value:u32)->(){target_16(address,value);}
// port of cpp/patched/io.cpp wphw_io_gx_push.
#[export_name="wphw_io_gx_push"]pub unsafe extern "C" fn entry_17(value:u64,bytes:u32)->(){target_17(value,bytes);}
// port of cpp/patched/io.cpp wphw_io_pixel_control.
#[export_name="wphw_io_pixel_control"]pub unsafe extern "C" fn entry_18(value:u16)->(){target_18(value);}
// port of cpp/patched/io.cpp wphw_io_log_plain.
#[export_name="wphw_io_log_plain"]pub unsafe extern "C" fn entry_19(address:u32,bytes:u32,write:bool)->(){log_write(c"mmio".as_ptr(),c"plain memory %s%u %08x".as_ptr(),if write{c"write".as_ptr()}else{c"read".as_ptr()},bytes.wrapping_mul(8),address);}
// port of cpp/leaf_logging.cpp wphw_log_enabled.
#[export_name="wphw_log_enabled"]pub unsafe extern "C" fn entry_20()->bool{target_19()}
// port of cpp/leaf_logging.cpp wphw_log_line.
#[export_name="wphw_log_line"]pub unsafe extern "C" fn entry_21(category:*const c_char,message:*const c_char)->(){log_write(category,c"%s".as_ptr(),message);}
// port of cpp/leaf_interrupts.cpp wphw_audio_update.
#[export_name="wphw_audio_update"]pub unsafe extern "C" fn entry_22()->(){target_20();}
// port of cpp/leaf_interrupts.cpp wphw_dsp_update.
#[export_name="wphw_dsp_update"]pub unsafe extern "C" fn entry_23()->(){target_21();}
// port of cpp/leaf_interrupts.cpp wphw_dsp_pending.
#[export_name="wphw_dsp_pending"]pub unsafe extern "C" fn entry_24()->u32{target_22()}
// port of cpp/leaf_interrupts.cpp wphw_ipc_update.
#[export_name="wphw_ipc_update"]pub unsafe extern "C" fn entry_25()->(){target_23();}
// port of cpp/leaf_interrupts.cpp wphw_ipc_pending.
#[export_name="wphw_ipc_pending"]pub unsafe extern "C" fn entry_26()->bool{target_24()}
// port of cpp/leaf_interrupts.cpp wphw_gx_process.
#[export_name="wphw_gx_process"]pub unsafe extern "C" fn entry_27()->(){target_25();}
// port of cpp/leaf_interrupts.cpp wphw_gx_take_finish.
#[export_name="wphw_gx_take_finish"]pub unsafe extern "C" fn entry_28()->bool{target_26()}
// port of cpp/leaf_interrupts.cpp wphw_exi_pending.
#[export_name="wphw_exi_pending"]pub unsafe extern "C" fn entry_29()->u32{target_27()}
// port of cpp/leaf_interrupts.cpp wphw_si_pending.
#[export_name="wphw_si_pending"]pub unsafe extern "C" fn entry_30()->bool{target_28()}
// port of cpp/leaf_interrupts.cpp wphw_si_poll.
#[export_name="wphw_si_poll"]pub unsafe extern "C" fn entry_31()->(){target_29();}
// port of cpp/leaf_interrupts.cpp wphw_video_present.
#[export_name="wphw_video_present"]pub unsafe extern "C" fn entry_32()->(){target_30();}
// port of cpp/leaf_interrupts.cpp wphw_mmio_read16.
#[export_name="wphw_mmio_read16"]pub unsafe extern "C" fn entry_33(a:u32)->u16{target_31(a)}
// port of cpp/leaf_interrupts.cpp wphw_mmio_read32.
#[export_name="wphw_mmio_read32"]pub unsafe extern "C" fn entry_34(a:u32)->u32{target_32(a)}
// port of cpp/leaf_interrupts.cpp wphw_mmio_write8.
#[export_name="wphw_mmio_write8"]pub unsafe extern "C" fn entry_35(a:u32,v:u8)->(){target_33(a,v);}
// port of cpp/leaf_interrupts.cpp wphw_mmio_write16.
#[export_name="wphw_mmio_write16"]pub unsafe extern "C" fn entry_36(a:u32,v:u16)->(){target_34(a,v);}
// port of cpp/leaf_interrupts.cpp wphw_mmio_write32.
#[export_name="wphw_mmio_write32"]pub unsafe extern "C" fn entry_37(a:u32,v:u32)->(){target_35(a,v);}
// port of cpp/leaf_interrupts.cpp wphw_mmio_write64.
#[export_name="wphw_mmio_write64"]pub unsafe extern "C" fn entry_38(a:u32,v:u64)->(){target_36(a,v);}
