// SPDX-License-Identifier: GPL-3.0-or-later
//! Renderer backend for the Wii Party Rust build (Claude's lane). Ports gx_render_common.cpp 1:1 (common.rs)
//! and replaces the Direct3D 11 backend gx_render.cpp with wgpu behind the same gx_render.h API.
pub mod common;
pub mod backend;
pub mod custom_textures;
#[cfg(feature="guest-runtime")]
pub mod gx;
pub mod gx_lighting;
pub mod screenshot;

pub mod guest_memory;

pub mod remote;
