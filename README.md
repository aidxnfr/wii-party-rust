# Wii Party (Rust)

Wii Party for Windows PC, running natively in **Rust** with a **Bevy** + **wgpu** (DirectX 12) front end.
The game's own code is statically recompiled from **your** disc on your PC, then runs on a Rust runtime that
reimplements the Wii hardware (CPU support, graphics, audio DSP, input, disc, NAND saves).

> This project is built directly on **[wiiparty-recomp by arelkair](https://github.com/arelkair/wiiparty-recomp)**.
> Its PowerPC + DSP static recompiler, runtime design and years of reverse-engineering research are what made a
> Rust port possible at all. The Rust runtime started as a line-by-line port of its C++ engine, the launcher follows
> its design, and its recompiler is used unchanged (pinned in `recompiler/upstream.toml`). Thank you, arelkair.

No game files are included or downloaded. You need your own legally obtained Wii Party disc image.

## Play

1. Download **`wii-party-launcher.exe`** from the [Releases](../../releases) page.
2. Run it, choose an install folder and your disc image (`.iso`, `.wbfs`, `.rvz`, `.wia`, `.ciso`, `.gcm`), press **Install**.
3. When it finishes, press **Play**.

The launcher builds everything locally: it downloads pinned build tools from their official sites (each checked
against a published SHA-256), recompiles your disc and compiles the game. Nothing from the game leaves your PC.
The first install takes roughly 30–90 minutes and about 25 GB of space (most of it build cache you can delete
afterwards). Updates rebuild only what changed and keep your saves and settings (`<install>/data`).

**Requirements:** Windows 10/11 64-bit, a CPU with AVX2 (Intel Haswell / AMD Zen or newer), a DirectX 12 GPU,
16 GB RAM recommended for the build (8 GB works with fewer parallel jobs in Settings).

Command line (no window): `wii-party-launcher.exe install --disc <image> [--dir <folder>] [--jobs N]`, then
`wii-party-launcher.exe play`.

### Controls

| Key | Wii Remote |
| --- | --- |
| Mouse | pointer |
| Enter / Space | A |
| Backspace | B |
| WASD / arrows | D-pad |
| H | HOME |
| Left Shift / middle mouse | shake |
| T / G or wheel | swing |
| Q / E / R / F | tilt |
| Tab | grip toggle |
| F10 / F11 | options menu / full screen |

Game controllers act as Wii Remotes 1–4.

## How it works

```
your disc ──nodtool──▶ extracted files ──upstream recompiler (Python)──▶ C++ ──recompiler/rust──▶ Rust game crates
                                                     └──DSP recompiler──▶ AX microcode C++ ──recompiler/dsp──▶ Rust
generated Rust + crates/wprt (CPU/runtime) + crates/wprt-hw (hardware) + crates/wprt-gx (GX) ──▶ wiiparty_game.dll
crates/host (Bevy window, audio, input, DX12 presenter) loads the DLL ──▶ wii-party.exe
```

| Part | Where | What it does |
| --- | --- | --- |
| Recompiler | `recompiler/` | Pins upstream, runs it on your disc, translates its C++ output to Rust (`rust/cpp2rs.py`) and the audio microcode (`dsp/`) |
| Runtime | `crates/wprt` | Guest CPU state, memory, threads, interrupts, dispatch, OS/HLE services |
| Hardware | `crates/wprt-hw` | DSP (Dolphin-derived), audio, video interface, NAND, IPC/IOS, input, options UI, screenshots |
| Graphics | `crates/wprt-gx` | GX command parser, software T&L, TEV shaders, EFB copies, wgpu backend, optional smooth motion |
| Host | `crates/host` | Bevy app: window, presentation, audio output, keyboard/mouse/gamepad, host options |
| Game DLL | `crates/game` | Links the generated code with the runtime into `wiiparty_game.dll` |
| Launcher | `launcher/` | Install / Play / Settings / Controls, like the wiiparty-recomp launcher |
| Tools | `tools/` | Accuracy fixtures, performance stats window, profiler |

Accuracy is checked with 23 deterministic replay fixtures: scripted input, a frame-counting clock, a hash of the
emulated state every frame and a screenshot every 60 frames, compared byte for byte with a reference run
(`docs/TESTING.md`).

## Documentation

- [docs/BUILDING.md](docs/BUILDING.md): building everything by hand, toolchain pins
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the pieces fit, where to change what
- [docs/TESTING.md](docs/TESTING.md): accuracy fixtures and how to run them
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md): optimizations done, profiling, ideas not yet done
- [docs/SMOOTH_MOTION.md](docs/SMOOTH_MOTION.md): optional in-between frames for 120/144/180 Hz screens
- [docs/UPDATING.md](docs/UPDATING.md): moving to a newer upstream, Rust, wgpu or Bevy
- [CONTRIBUTING.md](CONTRIBUTING.md)

## Credits

- **[arelkair](https://github.com/arelkair)**: [wiiparty-recomp](https://github.com/arelkair/wiiparty-recomp), the recompiler, runtime and research this is built on.
- **[Dolphin Emulator Project](https://dolphin-emu.org)**: DSP LLE, GX/lighting and Wii Remote behavior references; free replacement DSP ROMs.
- **[encounter](https://github.com/encounter)**: nod / nodtool for reading disc images.
- **[Bevy](https://bevyengine.org)**, **[wgpu](https://wgpu.rs)**, **[egui](https://github.com/emilk/egui)**, **[SDL](https://libsdl.org)**.
- Rust port, optimization and tooling: aidxnfr, with AI assistance (OpenAI Codex, Anthropic Claude, OpenCode).

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Legal

Licensed under GPL-3.0-or-later (see [LICENSE](LICENSE)), the same license as wiiparty-recomp.
This project is not affiliated with or endorsed by Nintendo. Wii Party and Wii are trademarks of Nintendo.
The repository contains no game assets, no code translated from the game, and no Nintendo firmware; all of that
is produced locally from your own disc and is excluded by `.gitignore`.
