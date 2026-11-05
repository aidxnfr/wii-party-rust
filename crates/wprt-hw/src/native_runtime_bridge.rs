// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete hybrid/cpp/bridge.cpp state/scalar policy port.
// Native SDK keeps ONLY std::vector<string> return construction and std::function invocation;no class layout inferred.
use std::ffi::{c_char,c_void};
#[export_name="_ZN2wp8g_memoryE"]pub static mut MEMORY:*mut u8=std::ptr::null_mut();
// port of pristine core/interrupts.cpp initializer retained by accepted bridge.cpp.
#[export_name="_ZN2wp14g_poll_counterE"]pub static mut POLL_COUNTER:u32=1<<10;
static mut GAME_DESCRIPTION:*const c_void=std::ptr::null();
extern "C"{fn wprt_time_base()->u64;fn wprt_idle_share()->f64;fn wprt_module_loaded(name:*const c_char)->bool;}
// port of wp::time_base / wp::idle_share / wp::module_loaded.
#[export_name="_ZN2wp9time_baseEv"]pub unsafe extern "C" fn time_base()->u64{wprt_time_base()}
#[export_name="_ZN2wp10idle_shareEv"]pub unsafe extern "C" fn idle_share()->f64{wprt_idle_share()}
#[export_name="_ZN2wp13module_loadedEPKc"]pub unsafe extern "C" fn module_loaded(name:*const c_char)->bool{wprt_module_loaded(name)}
// port of wp::game::description reference return:opaque existing immutable Description pointer,never inspect fields.
#[export_name="_ZN2wp4game11descriptionEv"]pub unsafe extern "C" fn description()->*const c_void{GAME_DESCRIPTION}
// port of accepted memory/metadata binds and counter pointer. Valid startup ordering remains caller's source contract.
#[no_mangle]pub unsafe extern "C" fn wphw_bind_memory(memory:*mut u8){MEMORY=memory;}
#[no_mangle]pub unsafe extern "C" fn wphw_bind_game(description:*const c_void){GAME_DESCRIPTION=description;}
#[no_mangle]pub unsafe extern "C" fn wphw_poll_counter_ptr()->*mut u32{std::ptr::addr_of_mut!(POLL_COUNTER)}
