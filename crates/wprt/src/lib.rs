// SPDX-License-Identifier: GPL-3.0-or-later
//! wprt: Rust port of the arelkair/wiiparty-recomp runtime (Claude, 2026-10-23). See outputs/EXACT_PLAN.md.
pub mod memory;
pub mod cpu;
pub mod runtime;
pub mod modules;
pub mod ffi;
pub mod threads;
pub mod dispatch;
pub mod interrupts;
pub mod hle;
#[cfg(windows)] pub mod fiber;
pub mod dol;
pub mod boot;
/// Everything the translated game code calls, under the same names as the C++ `wp::` namespace.
pub mod wp {
    pub use crate::cpu::*;
    pub use crate::memory::*;
    pub use crate::runtime::*;
}

#[cfg(feature="host")] pub mod host;

#[cfg(feature="rust-log")] pub mod native_log;

#[cfg(feature="rust-format")] pub mod native_format;

#[cfg(feature="rust-settings")] pub mod native_settings;

#[cfg(feature="rust-si")] pub mod native_si;

#[cfg(feature="rust-exi")] pub mod native_exi;

#[cfg(feature="rust-hollywood")] pub mod native_hollywood;

#[cfg(feature="rust-io")] pub mod native_io;

#[cfg(feature="rust-ios")] pub mod native_ios;

#[cfg(feature="rust-ipc")] pub mod native_ipc;

#[cfg(feature="rust-nand")] pub mod native_nand;

#[cfg(feature="rust-disc")] pub mod native_disc;

#[cfg(feature="rust-padmap")] pub mod native_padmap;

#[cfg(feature="rust-keymap")] pub mod native_keymap;

#[cfg(feature="rust-input")] pub mod native_input;

#[cfg(feature="rust-gamepad")] pub mod native_gamepad;

#[cfg(feature="rust-wiimote")] pub mod native_wiimote;

#[cfg(feature="rust-bluetooth")] pub mod native_bluetooth;

#[cfg(feature="rust-real-wiimote")] pub mod native_real_wiimote;

#[cfg(feature="rust-audio")] pub mod native_audio;

#[cfg(feature="rust-dsp-runtime")] pub mod native_dsp;

#[cfg(feature="rust-dsp-host")] pub mod native_dsp_host;

#[cfg(feature="rust-gx-state")] pub mod native_gx_state;
