// SPDX-License-Identifier: GPL-3.0-or-later
// Claude G60: oracle-build-src/comparison_bridge.cpp (H5 shared comparison clock C ABI) -> whole Rust, no residue.
#![allow(dead_code)]
include!("../common/cmp.rs");
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_comparison_enabled() -> bool { cmp_enabled() }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_comparison_advance() { cmp_advance() }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_comparison_ticks() -> u64 { cmp_ticks() }
