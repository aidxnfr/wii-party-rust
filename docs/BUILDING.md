# Building by hand

The launcher does all of this automatically (see `launcher/src/install.rs`). These are the same steps for
developers who want to work on the code.

## Pinned toolchain

| Tool | Version | Used for |
| --- | --- | --- |
| Rust | 1.99.0 (`rust-toolchain.toml`) | everything Rust; the game DLL targets `x86_64-pc-windows-gnu` |
| MinGW-w64 | GCC 15.2, nuwen.net 20.0 | linker for the game DLL, stats window |
| Python | 3.11+ (launcher uses 3.12.10 embeddable) | recompiler |
| pycparser | 2.23 | DSP microcode translation |
| nodtool | 1.4.4 | disc extraction |
| SDL | 3.4.16 (MinGW dev package) | gamepads / options window in the game DLL |
| upstream recompiler | `recompiler/upstream.toml` | PowerPC + DSP recompilation |

Download URLs and SHA-256 values: `launcher/src/tools.rs`.
The game DLL uses the unstable `catch_unwind` intrinsic in `crates/wprt-unwind`, enabled only for that crate with
`RUSTC_BOOTSTRAP=wprt_final_unwind` (no nightly toolchain needed).

## 1. Generate the game code from your disc

```
pip install -r recompiler/requirements.txt
python recompiler/pipeline.py --disc path/to/your/wiiparty.rvz --nodtool path/to/nodtool.exe
```

Output goes to `generated/` (ignored by git): the pinned upstream checkout with your extracted disc, the C++ it
generates, and the Rust crates built from it (`game-code/`, `game-code-link/`, `all_modules.rs`, `dsp-ax/`).
Re-running only redoes steps whose inputs changed.

## 2. Build the game DLL

From the repository root (so `.cargo/config.toml` applies), with MinGW's `bin` on `PATH`:

```
mkdir build\link-compat
copy <mingw>\lib\gcc\x86_64-w64-mingw32\15.2.0\libgcc.a build\link-compat\libgcc_eh.a
set RUSTC_BOOTSTRAP=wprt_final_unwind
set WP_SDL3_DIR=<SDL3-3.4.16>\x86_64-w64-mingw32
set CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=<mingw>\bin\g++.exe
cargo build --release --target x86_64-pc-windows-gnu --manifest-path crates/game/Cargo.toml --features <contents of crates/game/features.txt joined with commas>
```

The generated code is about 35 million lines; a full build takes 30–90 minutes. Each `rustc` job can need several
GB of RAM, so lower `-j` on machines with less than 16 GB. Only crates you change are rebuilt afterwards.

## 3. Build the host and the launcher

```
cargo build --release -p wii-party-host          # target/release/wii-party.exe
set WP_HOST_EXE=%CD%\target\release\wii-party.exe
cargo build --release -p wii-party-launcher      # embeds the source and the host
```

## 4. Run

```
target\release\wii-party.exe <wiiparty_game.dll> generated\upstream\games\wiiparty\extracted <nand folder>
```
with the environment the launcher sets in `launcher/src/play.rs` (DX12 publication path, logging, SDL path).
