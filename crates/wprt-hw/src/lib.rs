// SPDX-License-Identifier: GPL-3.0-or-later
// Additional hardware ports share the existing per-DLL runtime unchanged.
pub use wprt::{memory,ffi};
#[cfg(feature="rust-png")]pub mod native_png;
#[cfg(feature="rust-video-transport")]pub mod native_video_transport;
#[cfg(feature="rust-video-controls")]pub mod native_video_controls;
#[cfg(feature="rust-mii-records")]pub mod native_mii_records;
#[cfg(feature="rust-ui-text")]pub mod native_ui_text;
#[cfg(feature="rust-screenshot")]pub mod native_screenshot;
#[cfg(feature="rust-options-policy")]pub mod native_options_policy;
#[cfg(feature="rust-options-state")]pub mod native_options_state;
#[cfg(feature="rust-save-backup-helpers")]pub mod native_save_backup;
#[cfg(feature="rust-options-window")]pub mod native_options_window;

#[cfg(feature="rust-options-canvas")]pub mod native_options_canvas;
#[cfg(feature="rust-options-text")]pub mod native_options_text;
#[cfg(feature="rust-options-titlebar")]pub mod native_options_titlebar;
#[cfg(feature="rust-options-render")]pub mod native_options_render;
#[cfg(feature="rust-save-adopt")]pub mod native_save_adopt;

#[cfg(feature = "rust-stb")]
pub mod native_stb;

#[cfg(feature = "rust-font-io")]
pub mod native_font_io;

#[cfg(feature="rust-video-win32")]pub mod native_video_win32;

#[cfg(feature="rust-audio-win32")]pub mod native_audio_win32;

#[cfg(feature="rust-input-sdl")]
#[path="native_input_sdl/lib.rs"]pub mod native_input_sdl;

#[cfg(feature="rust-fpu-leaf")]pub mod native_fpu_leaf;
#[cfg(feature="rust-host-io")]pub mod native_host_io;
#[cfg(feature="rust-dsp-stacks")]pub mod native_dsp_stacks;

#[cfg(feature="rust-dsp-memory-map")]pub mod native_dsp_memory_map;
#[cfg(feature="rust-dsp-misc")]pub mod native_dsp_misc;
#[cfg(feature="rust-dsp-load-store")]pub mod native_dsp_load_store;
#[cfg(feature="rust-dsp-branch")]pub mod native_dsp_branch;
#[cfg(feature="rust-dsp-interpreter-tables")]pub mod native_dsp_interpreter_tables;

#[cfg(feature="rust-dsp-multiplier")]pub mod native_dsp_multiplier;
#[cfg(feature="rust-dsp-ext-ops")]pub mod native_dsp_ext_ops;
#[cfg(feature="rust-dsp-arithmetic")]pub mod native_dsp_arithmetic;
#[cfg(feature="rust-dsp-interpreter")]pub mod native_dsp_interpreter;
#[cfg(feature="rust-dsp-analyzer")]pub mod native_dsp_analyzer;
#[cfg(feature="rust-dsp-accelerator")]pub mod native_dsp_accelerator;
#[cfg(feature="rust-dsp-ifx")]pub mod native_dsp_ifx;
#[cfg(feature="rust-dsp-metadata-tables")]pub mod native_dsp_metadata_tables;

#[cfg(feature="rust-dsp-core-file")]pub mod native_dsp_core;

#[cfg(feature="rust-scalar-adapters")]pub mod native_scalar_adapters;
#[cfg(feature="rust-ipc-adapter")]pub mod native_ipc_adapter;
#[cfg(feature="rust-log-adapter")]pub mod native_log_adapter;

#[cfg(feature="rust-host-sample-adapter")]pub mod native_host_sample_adapter;
#[cfg(feature="rust-runtime-bridge")]pub mod native_runtime_bridge;
#[cfg(feature="rust-gamepad-adapter")]pub mod native_gamepad_adapter;
#[cfg(feature="rust-real-wiimote-adapter")]pub mod native_real_wiimote_adapter;
#[cfg(feature="rust-padmap-adapter")]pub mod native_padmap_adapter;
#[cfg(feature="rust-exi-adapter")]pub mod native_exi_adapter;
#[cfg(feature="rust-bluetooth-adapter")]pub mod native_bluetooth_adapter;
#[cfg(feature="rust-wiimote-adapter")]pub mod native_wiimote_adapter;
#[cfg(feature="rust-gx-host-adapter")]pub mod native_gx_host_adapter;
#[cfg(feature="rust-ui-text-adapter")]pub mod native_ui_text_adapter;
#[cfg(feature="rust-input-adapter")]pub mod native_input_adapter;
#[cfg(feature="rust-audio-adapter")]pub mod native_audio_adapter;
#[cfg(feature="rust-settings-adapter")]pub mod native_settings_adapter;
#[cfg(feature="rust-renderer-adapter")]pub mod native_renderer_adapter;
#[cfg(feature="rust-ax-sdk-adapter")]pub mod native_ax_sdk_adapter;
#[cfg(feature="rust-options-window-adapter")]pub mod native_options_window_adapter;
#[cfg(feature="rust-options-state-adapter")]pub mod native_options_state_adapter;
#[cfg(feature="rust-video-adapter")]pub mod native_video_adapter;
#[cfg(feature="rust-video-adapter")]pub mod native_host_client;
#[cfg(feature="rust-disc-adapter")]pub mod native_disc_adapter;

#[cfg(feature="rust-nand-adapter")]pub mod native_nand_adapter;

#[cfg(feature="rust-ios-adapter")]pub mod native_ios_adapter;

#[cfg(feature="rust-keymap-adapter")]pub mod native_keymap_adapter;

#[cfg(feature="rust-mii-adapter")]pub mod native_mii_adapter;

#[cfg(feature="rust-options-render-adapter")]pub mod native_options_render_adapter;

#[cfg(feature="rust-png-adapter")]pub mod native_png_adapter;

#[cfg(feature="rust-screenshot-adapter")]pub mod native_screenshot_adapter;

#[cfg(feature="rust-dsp-host-adapter")]pub mod native_dsp_host_adapter;

#[cfg(feature="rust-host-gamepad")]pub mod native_host_gamepad;

#[cfg(feature="rust-host-keyboard")]pub mod native_host_keyboard;

#[cfg(feature="rust-dsp-stacks-file")]pub mod native_dsp_stacks_file;

#[cfg(feature="rust-dsp-memory-map-file")]pub mod native_dsp_memory_map_file;

#[cfg(feature="rust-dsp-misc-file")]pub mod native_dsp_misc_file;

#[cfg(feature="rust-dsp-branch-file")]pub mod native_dsp_branch_file;

#[cfg(feature="rust-container-sdk-files")]pub mod native_container_sdk_files;

#[cfg(feature="rust-dsp-extra-files")]pub mod native_dsp_extra_files;

#[cfg(feature="rust-dsp-last-files")]pub mod native_dsp_last_files;

#[cfg(feature="rust-g60-batch4-files")]pub mod native_g60_batch4_files;
#[cfg(feature="rust-g60-batch5-files")]pub mod native_g60_batch5_files;

#[cfg(feature="rust-final-lifetimes")] mod native_final_lifetimes;

#[cfg(feature="rust-final-lifetimes")]#[path="native_final_sdk/filesystem.rs"]mod filesystem;
#[cfg(feature="rust-final-lifetimes")]#[path="native_final_sdk/save_files.rs"]mod save_files;
#[cfg(feature="rust-final-lifetimes")]#[path="native_final_sdk/mii_files.rs"]mod mii_files;
#[cfg(feature="rust-final-lifetimes")]#[path="native_final_sdk/font_files.rs"]mod font_files;
#[cfg(feature="rust-final-lifetimes")]#[path="native_final_sdk/eh.rs"]mod eh;
