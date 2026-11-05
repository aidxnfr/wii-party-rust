// AI-assisted (Claude, G60) ax_sdk.cpp -> Rust: whole member, no C++ residue. Offsets are compiler-derived (probe.cpp -> layout.rs).
// Source: SDSP::GetAnalyzer().IsIdleSkip/IsLoopEnd (inline header bodies): (m_code_flags[address] & flag) != 0.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("ax_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod layout { include!("layout.rs"); }
use layout::*;
#[inline] unsafe fn flags(p: *mut u8, a: u16) -> u8 { *p.add(ANALYZER + CODE_FLAGS + a as usize) }
#[export_name = "wphw_ax_sdk_IsIdleSkip"]
pub unsafe extern "C-unwind" fn IsIdleSkip(p: *mut u8, a: u16) -> bool { flags(p, a) & CODE_IDLE_SKIP != 0 }
#[export_name = "wphw_ax_sdk_IsLoopEnd"]
pub unsafe extern "C-unwind" fn IsLoopEnd(p: *mut u8, a: u16) -> bool { flags(p, a) & CODE_LOOP_END != 0 }
