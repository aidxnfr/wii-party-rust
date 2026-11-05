# Architecture

## Two processes in one: host and game DLL

`wii-party.exe` (crates/host, Windows MSVC target) is a Bevy app. It owns the window, the DX12 device, audio output
and input. It loads `wiiparty_game.dll` (crates/game, Windows GNU target), which contains the translated game and
the emulated hardware, and runs the game on its own thread with a 512 MiB stack.

The two sides talk through small C ABIs:
- **Graphics**: the DLL parses GX commands and does vertex transform/lighting (`crates/wprt-gx/src/gx.rs`), then hands
  screen-space triangles to the host's renderer through `wprt_gx::remote` (`RemoteRenderer` function table). The host
  renderer (`backend.rs`) batches draws, runs TEV shaders, handles EFB copies and publishes finished frames to Bevy.
- **Audio**: the DLL runs the DSP (Dolphin-derived LLE plus the recompiled AX microcode in `generated/dsp-ax`) and
  passes PCM to the host's audio output.
- **Input**: the host samples keyboard/mouse/gamepads and the DLL turns them into Wii Remote reports.

## The game code

`recompiler/` turns the disc's PowerPC code into Rust in two stages:
1. upstream wiiparty-recomp (pinned) decodes the DOL and every REL module into C++, one function per guest function,
   with guest registers in a `Cpu` struct and control flow as labels/gotos;
2. `recompiler/rust/cpp2rs.py` rewrites that fixed C++ dialect into Rust (`loop { match pc { … } }` per function),
   keeping every operation and its order. `translate.py` packages it as one crate for the DOL and one per module.

Generated functions call into `crates/wprt` for memory access, condition register helpers, paired-singles,
floating-point ordering, dispatch of indirect calls, threads and interrupts.

## Crates

| Crate | Notes |
| --- | --- |
| `wprt` | runtime: `cpu.rs` (helpers), `memory.rs`, `dispatch.rs`/`runtime.rs` (function lookup, resume points), `threads.rs`, `interrupts.rs` (VI/DSP/IPC/decrementer + idle sleep), `hle.rs`, native_* (IOS, NAND, disc, input, audio) |
| `wprt-ports` | small shared ports (GX state structs, settings formats) |
| `wprt-hw` | hardware layer formerly in C++: DSP core/interpreter/accelerator (Dolphin), AX SDK glue, video interface, options window, screenshots, fonts; `layouts/` holds struct layouts and tables measured from the Dolphin sources |
| `wprt-gx` | GX: command parser + T&L (DLL side), renderer backend + shaders (host side), texture decoding/cache, EFB copies and write-backs, smooth motion (`backend.rs`, `Interp`) |
| `wprt-unwind` | lets Rust frames catch the C++-style unwinds the SDK code expects |
| `save-backup`, `stb-truetype`, `host-controller` | save backup flow, font rasterizer, host gamepad bridge |
| `host` | Bevy app (`hybrid_host.rs`), presenter (`host_present.rs`), audio, keyboard, options overlay |
| `game` | the DLL: boots the DOL, registers functions and modules, starts video/audio; feature list in `features.txt` |

Feature flags in `crates/game/features.txt` select the all-Rust hardware paths that passed the accuracy fixtures;
keep that file as the single source of truth.

## Environment variables (runtime)

| Variable | Effect |
| --- | --- |
| `WP_LOG_FILE` | runtime log (fps, threads, copies, audio) |
| `WP_INTERP_FILE` / `WP_INTERP` | smooth motion: 0–4 extra frames (see SMOOTH_MOTION.md) |
| `WP_IDLE_SPIN`, `WP_IDLE_NAP_US`, `WP_IDLE_NAP_AFTER_US`, `WP_IDLE_NAP_QUIET_US` | idle sleep tuning (see PERFORMANCE.md) |
| `WP_INPUT_CACHE_US` | input sampling cache window (default 1000 µs, 0 = off) |
| `WP_PRESENT_MODE`, `WP_BEVY_VSYNC`, `WP_BEVY_THREADS` | presentation / Bevy tuning |
| `WP_INPUT_SCRIPT`, `WP_DETERMINISTIC`, `WP_COMPARE_LOG`, `WP_SAVE_FRAME` … | fixture mode (TESTING.md) |
