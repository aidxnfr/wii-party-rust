# Third-party notices

This project is GPL-3.0-or-later. It includes or is derived from:

| Project | License | Used for |
| --- | --- | --- |
| [wiiparty-recomp](https://github.com/arelkair/wiiparty-recomp) by arelkair | GPL-3.0-or-later | The recompiler (used unchanged, downloaded at the pinned commit). The Rust runtime (`crates/wprt`, `wprt-hw`, `wprt-gx`, `wprt-ports`, `crates/game`) is a port of its C++ engine; ported files carry a "port of …" note. |
| [Dolphin](https://github.com/dolphin-emu/dolphin) | GPL-2.0-or-later | DSP LLE core/interpreter/accelerator/analyzer, DSP tables and struct layouts (`crates/wprt-hw`), DSP register helpers (`recompiler/dsp/runtime/ops.rs`), Wii Remote IR/motion behavior, GX lighting reference. Dolphin's free replacement DSP ROMs (from the upstream checkout) are embedded into the locally built game DLL. |
| [wgpu, naga](https://github.com/gfx-rs/wgpu) | MIT OR Apache-2.0 | renderer; patched copies in `third_party/` |
| [Bevy](https://github.com/bevyengine/bevy) | MIT OR Apache-2.0 | host application |
| [egui / eframe](https://github.com/emilk/egui) | MIT OR Apache-2.0 | launcher UI |
| [stb_truetype](https://github.com/nothings/stb) (Rust port) | Public domain / MIT | font rasterizing (`crates/stb-truetype`) |
| [SDL](https://github.com/libsdl-org/SDL) | zlib | downloaded at install; gamepads |
| [nod / nodtool](https://github.com/encounter/nod) | MIT OR Apache-2.0 | downloaded at install; disc extraction |
| [pycparser](https://github.com/eliben/pycparser) | BSD-3-Clause | downloaded at install; DSP microcode translation |
| MinGW-w64 (nuwen.net distro), Python | various (GPL with runtime exception, PSF) | downloaded at install; build tools |
| Rust crates from crates.io | see each crate | see `Cargo.lock` |

Wii Party is © Nintendo. No part of the game is included in this repository.
