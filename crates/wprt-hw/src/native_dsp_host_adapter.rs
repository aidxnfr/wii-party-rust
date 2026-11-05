// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted owned dsp_host_rust.cpp normal adapter policy;native class/atomic/container/old-verifier SDK retained.
// PRIVATE WORKING CANDIDATE:NOT COMPARED/INTEGRATED.
use std::ffi::{c_char,c_void};
extern "C"{
#[link_name="wprt_dsp_update"]fn target_0()->();
#[link_name="wprt_dsp_read16"]fn target_1(address:u32)->u16;
#[link_name="wprt_dsp_write16"]fn target_2(address:u32,value:u16)->();
#[link_name="wprt_dsp_pending"]fn target_3()->u32;
#[link_name="wprt_dsp_host_read"]fn target_4(address:u32)->u8;
#[link_name="wprt_dsp_host_write"]fn target_5(value:u8,address:u32)->();
#[link_name="wprt_dsp_host_to_dsp"]fn target_6(destination:*mut u16,address:u32,size:u32)->();
#[link_name="wprt_dsp_host_from_dsp"]fn target_7(source:*const u16,address:u32,size:u32)->();
#[link_name="wprt_dsp_host_on_thread"]fn target_8()->bool;
#[link_name="wprt_dsp_host_is_wii"]fn target_9()->bool;
#[link_name="wprt_dsp_host_interrupt"]fn target_10()->();
#[link_name="wprt_dsp_host_time_base"]fn target_11()->u64;
fn wprt_dsp_host_code_address(core:*mut c_void,address:u32,size:usize);fn wprt_dsp_host_code(core:*mut c_void,pointer:*const u8,size:usize);
#[link_name="_ZN2wp5audio12start_outputEv"]fn audio_start()->();
#[link_name="_ZN2wp5audio4pushEPKsyj"]fn audio_push(frames:*const i16,n:usize,rate:u32)->();
#[link_name="_ZN2wp3log7enabledEv"]fn log_enabled()->bool;
#[link_name="_ZN2wp3log5writeEPKcS2_z"]fn log_write(category:*const c_char,format:*const c_char,...);
fn __acrt_iob_func(index:u32)->*mut libc::FILE;fn __mingw_fprintf(file:*mut libc::FILE,format:*const c_char,...)->i32;fn getenv(name:*const c_char)->*mut c_char;
fn wphw_dsp_clock_sdk()->i64;fn wphw_dsp_present_sdk()->bool;fn wphw_dsp_logging_sdk()->bool;fn wphw_dsp_verifying_sdk()->bool;fn wphw_dsp_translated_present_sdk()->bool;fn wphw_dsp_state_control_sdk()->u16;fn wphw_dsp_pc_sdk()->u16;
fn wphw_dsp_state_sdk()->*mut c_void;fn wphw_dsp_translated_direct_sdk(state:*mut c_void,left:i32,idle:*mut bool)->i32;fn wphw_dsp_translated_verified_sdk(state:*mut c_void,left:i32,idle:*mut bool)->i32;
fn wphw_dsp_step_sdk();fn wphw_dsp_run_cycles_sdk(cycles:i32);fn wphw_dsp_idle_skip_sdk()->bool;fn wphw_dsp_bootstrap_sdk()->bool;
fn wphw_dsp_mail_peek_sdk(mailbox:i32)->u32;fn wphw_dsp_mail_high_sdk(mailbox:i32)->u16;fn wphw_dsp_mail_low_sdk(mailbox:i32)->u16;fn wphw_dsp_write_mail_high_sdk(value:u16);fn wphw_dsp_write_mail_low_sdk(value:u16);
fn wphw_dsp_control_read_sdk()->u16;fn wphw_dsp_control_write_sdk(value:u16);fn wphw_dsp_external_sdk();fn wphw_dsp_exceptions_sdk();
fn wphw_dsp_load_store_sdk(value:f64);fn wphw_dsp_loaded_sdk()->bool;fn wphw_dsp_native_sdk()->bool;fn wphw_dsp_load_sdk()->f64;
fn wphw_dsp_dram_sdk()->*const u16;fn wphw_dsp_iram_sdk()->*const u16;fn wphw_dsp_write_log_sdk()->*mut c_void;fn wphw_dsp_host_record_write_sdk(address:u32,value:u8);fn wphw_dsp_self_testing_sdk()->bool;fn wphw_dsp_restoring_sdk()->bool;fn wphw_dsp_iram_written_sdk();fn wphw_dsp_crc_sdk(core:*mut c_void,crc:u32);fn wphw_dsp_clear_translated_sdk();fn wphw_dsp_count_sdk()->usize;fn wphw_dsp_forced_sdk()->bool;fn wphw_dsp_crc_at_sdk(index:usize)->u32;fn wphw_dsp_select_sdk(index:usize);fn wphw_dsp_loaded_store_sdk();fn wphw_dsp_native_store_sdk(value:bool);fn wphw_dsp_dump_sdk(directory:*const c_char,pointer:*const u8,size:usize,crc:u32);fn wphw_dsp_host_analyze_sdk(core:*mut c_void);fn wphw_dsp_host_existing_selftest_sdk();
}
// port of wp::dsp::update;actual OWN nm symbol.
#[export_name="_ZN2wp3dsp6updateEv"]pub unsafe extern "C" fn entry_0()->(){target_0()}
// port of wp::dsp::read16;actual OWN nm symbol.
#[export_name="_ZN2wp3dsp6read16Ej"]pub unsafe extern "C" fn entry_1(address:u32)->u16{target_1(address)}
// port of wp::dsp::write16;actual OWN nm symbol.
#[export_name="_ZN2wp3dsp7write16Ejt"]pub unsafe extern "C" fn entry_2(address:u32,value:u16)->(){target_2(address,value)}
// port of wp::dsp::pending_interrupt;actual OWN nm symbol.
#[export_name="_ZN2wp3dsp17pending_interruptEv"]pub unsafe extern "C" fn entry_3()->u32{target_3()}
// port of DSP::Host::ReadHostMemory;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host14ReadHostMemoryEj"]pub unsafe extern "C" fn entry_4(address:u32)->u8{target_4(address)}
// port of DSP::Host::WriteHostMemory;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host15WriteHostMemoryEhj"]pub unsafe extern "C" fn entry_5(value:u8,address:u32)->(){target_5(value,address)}
// port of DSP::Host::DMAToDSP;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host8DMAToDSPEPtjj"]pub unsafe extern "C" fn entry_6(destination:*mut u16,address:u32,size:u32)->(){target_6(destination,address,size)}
// port of DSP::Host::DMAFromDSP;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host10DMAFromDSPEPKtjj"]pub unsafe extern "C" fn entry_7(source:*const u16,address:u32,size:u32)->(){target_7(source,address,size)}
// port of DSP::Host::OnThread;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host8OnThreadEv"]pub unsafe extern "C" fn entry_8()->bool{target_8()}
// port of DSP::Host::IsWiiHost;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host9IsWiiHostEv"]pub unsafe extern "C" fn entry_9()->bool{target_9()}
// port of DSP::Host::InterruptRequest;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host16InterruptRequestEv"]pub unsafe extern "C" fn entry_10()->(){target_10()}
// port of DSP::Host::TimeBase;actual OWN nm symbol.
#[export_name="_ZN3DSP4Host8TimeBaseEv"]pub unsafe extern "C" fn entry_11()->u64{target_11()}
// port of DSP::Host::CodeLoaded overload through opaque SDK core.
#[export_name="_ZN3DSP4Host10CodeLoadedERNS_7DSPCoreEjy"]pub unsafe extern "C" fn wprt_dsp_host_code_address_adapter(core:*mut c_void,address:u32,size:usize){wprt_dsp_host_code_address(core,address,size);}
// port of DSP::Host::CodeLoaded overload through opaque SDK core.
#[export_name="_ZN3DSP4Host10CodeLoadedERNS_7DSPCoreEPKhy"]pub unsafe extern "C" fn wprt_dsp_host_code_adapter(core:*mut c_void,pointer:*const u8,size:usize){wprt_dsp_host_code(core,pointer,size);}
// port of PUBLIC wp/dsp.h Status,named-field aggregate loads in source order.
#[repr(C)]pub struct Status{loaded:bool,native:bool,load_percent:f64}
#[export_name="_ZN2wp3dsp6statusEv"]pub unsafe extern "C" fn status()->Status{let loaded=wphw_dsp_loaded_sdk();let native=wphw_dsp_native_sdk();let load_percent=wphw_dsp_load_sdk();Status{loaded,native,load_percent}}

// port of owned chrono conversions;actual object cvttsd2si64,not Rust saturating cast.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_now()->i64{wphw_dsp_clock_sdk()}
#[no_mangle]pub extern "C" fn wphw_dsp_seconds(ticks:i64)->f64{ticks as f64/1_000_000_000.0}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_duration(seconds:f64)->i64{let value=seconds*1_000_000_000.0;let out:i64;core::arch::asm!("cvttsd2si {out}, {value}",out=out(reg)out,value=in(xmm_reg)value,options(pure,nomem,nostack));out}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_present()->bool{wphw_dsp_present_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_halted()->bool{!wphw_dsp_present_sdk()||(wphw_dsp_state_control_sdk()&0x0004)!=0}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_translated_ready()->bool{wphw_dsp_translated_present_sdk()&&wphw_dsp_pc_sdk()<0x1000}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_translated(left:i32,idle:*mut bool)->i32{let state=wphw_dsp_state_sdk();if wphw_dsp_verifying_sdk(){wphw_dsp_translated_verified_sdk(state,left,idle)}else{wphw_dsp_translated_direct_sdk(state,left,idle)}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_step(){wphw_dsp_step_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_run_cycles(cycles:i32){wphw_dsp_run_cycles_sdk(cycles)}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_idle_skip()->bool{wphw_dsp_idle_skip_sdk()}
// port of load_rom;actual native RomBlob and options arrays supplied by typed SDK.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_rom_words(data:*const u8,size:usize,words:*mut u16,count:usize)->bool{if size!=count.wrapping_mul(2){return false;}for i in 0..count{*words.add(i)=((*data.add(i*2) as u16)<<8)|(*data.add(i*2+1) as u16);}true}
// port of original bootstrap branch/order;operation ids are private SDK ABI,not game constants.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_bootstrap_flow(raw:*mut c_void,op:unsafe extern "C" fn(*mut c_void,u32)->bool)->bool{if !op(raw,0)||!op(raw,1){__mingw_fprintf(__acrt_iob_func(2),c"DSP ROMs missing from the executable resources
".as_ptr());return false;}op(raw,2);if !op(raw,3){__mingw_fprintf(__acrt_iob_func(2),c"DSP core failed to initialize
".as_ptr());return false;}op(raw,4);op(raw,5);op(raw,6);true}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_bootstrap()->bool{wphw_dsp_bootstrap_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_start_output(){audio_start()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_push(frames:*const i16,n:u32,rate:u32){audio_push(frames,n as usize,rate)}
// port of public Mailbox enum CPU0/DSP1,source high/low dispatch and short-circuit control order.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_mail_peek(cpu:bool)->u32{wphw_dsp_mail_peek_sdk(if cpu{0}else{1})}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_mail_read(cpu:bool,high:bool)->u16{let mailbox=if cpu{0}else{1};if high{wphw_dsp_mail_high_sdk(mailbox)}else{wphw_dsp_mail_low_sdk(mailbox)}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_mail_write(high:bool,value:u16){if high{wphw_dsp_write_mail_high_sdk(value)}else{wphw_dsp_write_mail_low_sdk(value)}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_control_read()->u16{if wphw_dsp_present_sdk(){wphw_dsp_control_read_sdk()}else{0x0004}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_control_write(value:u16)->u16{let mut engine=0x0004;if wphw_dsp_present_sdk(){wphw_dsp_control_write_sdk(value);if value&0x0002!=0{wphw_dsp_external_sdk();wphw_dsp_exceptions_sdk();}engine=wphw_dsp_control_read_sdk();}engine}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_aram_log(main:u32,aram:u32,count:u32,to_main:bool){if wphw_dsp_logging_sdk(){__mingw_fprintf(__acrt_iob_func(2),c"DSP aram dma %s main %08x aram %08x count %x
".as_ptr(),if to_main{c"in".as_ptr()}else{c"out".as_ptr()},main,aram,count);}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_report(seconds:f64,buffers:u32,late:u32,stale:u32,overwritten:u32,worst:f64,cycles:f64,dsp_seconds:f64,behind:f64){wphw_dsp_load_store_sdk(100.0*dsp_seconds/seconds);if log_enabled(){log_write(c"audio".as_ptr(),c"buffers %.1f/s, late starts %u, %u stale and %u overwritten blocks, worst start %.2f ms, DSP %.1f Mcycles/s using %.1f%% of the CPU thread, DMA %.2f ms behind".as_ptr(),buffers as f64/seconds,late,stale,overwritten,worst*1000.0,cycles/seconds/1e6,100.0*dsp_seconds/seconds,behind*1000.0);}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_osd_message(message:*const c_char){__mingw_fprintf(__acrt_iob_func(2),c"DSP %s
".as_ptr(),message);}
// port of Host words:read BOTH source initializer-list bases before loop;actual DSP_DRAM_SIZE4096.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_words(pointer:*const u16,size:u32)->u32{let bases=[wphw_dsp_dram_sdk(),wphw_dsp_iram_sdk()];for base in bases{let start=base as usize;let end=start.wrapping_add(0x1000*2);let p=pointer as usize;if p>=start&&p<end{return (size/2).min((end.wrapping_sub(p)/2) as u32);}}0}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_logging()->bool{!wphw_dsp_write_log_sdk().is_null()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_record_write(address:u32,value:u8){wphw_dsp_host_record_write_sdk(address,value)}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_self_testing()->bool{wphw_dsp_self_testing_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_code_begin()->u32{if wphw_dsp_restoring_sdk(){return 0;}wphw_dsp_iram_written_sdk();if wphw_dsp_self_testing_sdk(){return 1;}2}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_crc(core:*mut c_void,crc:u32){wphw_dsp_crc_sdk(core,crc);wphw_dsp_clear_translated_sdk();}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_code_count()->usize{wphw_dsp_count_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_forced()->bool{wphw_dsp_forced_sdk()}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_code_crc(index:usize)->u32{wphw_dsp_crc_at_sdk(index)}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_code_select(index:usize){wphw_dsp_select_sdk(index)}
// port of source log/state/env/dump sequence;do not cache the two translated-pointer reads across logger calls.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_code_report(pointer:*const u8,size:usize,crc:u32){if wphw_dsp_logging_sdk(){__mingw_fprintf(__acrt_iob_func(2),c"DSP code loaded, %zu bytes, checksum %08x
".as_ptr(),size,crc);}wphw_dsp_loaded_store_sdk();wphw_dsp_native_store_sdk(wphw_dsp_translated_present_sdk());log_write(c"audio".as_ptr(),c"DSP microcode loaded, %u bytes, checksum %08x, %s".as_ptr(),size as u32,crc,if wphw_dsp_translated_present_sdk(){c"recompiled".as_ptr()}else{c"interpreted".as_ptr()});let directory=getenv(c"WP_DUMP_DSP_CODE".as_ptr());if !directory.is_null(){wphw_dsp_dump_sdk(directory,pointer,size,crc);}}
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_analyze(core:*mut c_void){wphw_dsp_host_analyze_sdk(core)}
// ORIGINAL optional verifier/selftest SDK unchanged,no new checker or check execution.
#[no_mangle]pub unsafe extern "C" fn wphw_dsp_host_existing_selftest(){wphw_dsp_host_existing_selftest_sdk()}
