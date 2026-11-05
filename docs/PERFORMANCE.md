# Performance

Target: a steady 60 fps with as little CPU as possible. On a Ryzen 7 7800X3D + RX 7900 XT the game uses about
0.6 of one core in total during play (it started at one core pinned at 100%, and 3.5–4 cores while clicking menus).

## What was done (all exact: every fixture still matches byte for byte)

| | Change | Where |
| --- | --- | --- |
| Idle sleep | The guest idle loop (OSSelectThread) used to spin until the next video interrupt. Now the thread sleeps in 250 µs slices once the guest has been idle for 300 µs, or 1.5 ms while disc/NAND requests are in flight (so loading is not slowed). Never inside the last 0.5 ms before a VI. Off in fixture mode. | `crates/wprt/src/interrupts.rs` (`idle-nap`) |
| Input cache | Controller state was sampled on every Bluetooth poll (≈40% of the game thread). Samples are now reused for 1 ms. | `native_input.rs` (`opt-batch1`) |
| Audio DSP | `CheckExceptions` called on every DSP instruction now has Dolphin's early-out; AX driver built at opt-level 3. | `recompiler/dsp/build_ax.py` |
| Builds | Release host profile (was dev), thin LTO, opt-level 3 for wgpu/naga/png/renderer/runtime, opt-level 2 for the DOL crate, `x86-64-v3` (AVX2) everywhere. | `Cargo.toml`, `crates/game/Cargo.toml`, `.cargo/config.toml` |
| Presentation | Bevy vsync, 4 task threads instead of 6 spinning; mailbox present mode. | `crates/host`, `wprt-gx/backend.rs` |
| Graphics | FIFO buffer pool, lighting state without a per-vertex lock, per-draw vertex T&L cache (exact key compare), faster texture hashing, exact LUTs for colour conversion, faster EFB write-back encoding. | `crates/wprt-gx` |

## Measuring

- `game/stats.exe` (Settings → "Open the performance stats window"): fps graph, CPU %, per-thread CPU, memory, log.
- `tools/profiler/sampler.cpp`: no-admin sampling profiler (suspend thread + read RIP). Build with MinGW:
  `g++ -O2 sampler.cpp -o sampler.exe`. Run `sampler.exe <pid> <seconds> <interval_us> samples.txt`, then
  `python tools/profiler/symbolize.py samples.txt game-syms.txt [host-syms.txt]` where `game-syms.txt` comes from
  `nm -C wiiparty_game.dll` and `host-syms.txt` from `python tools/profiler/pdbsyms.py wii-party.pdb host-syms.txt`.
  `wprt-ax-driver` and `wprt-gx` are built with line tables, so `addr2line -e wiiparty_game.dll <addr>` gives source lines.

## Where the time goes now

- Game thread: guest code, then the AX audio microcode (register state lives behind raw pointers, so every DSP
  instruction loads/stores memory), then the remaining idle spin.
- Graphics thread: software T&L (~40%), vertex copies (each vertex is 144 bytes and is copied three times on its
  way to the GPU), the wgpu/driver submission.

## Bigger ideas not done yet

1. **Registers as locals in generated code** (`cpp2rs.py`): keep guest GPRs/FPRs/CR in Rust locals inside each
   function and merge single-predecessor blocks. Largest expected gain (XenonRecomp reports the same), but it
   touches all generated code, so it needs the full fixture set.
2. Same idea for the AX microcode (`emit_ax.py`).
3. Compact vertex format (only the texture coordinates a draw uses) and writing vertices straight into the
   upload buffer.
4. GPU-side EFB copy encoding and asynchronous read-back.
5. Profile-guided optimization (needs `llvm-tools` and a `profiler_builtins`-enabled std for the GNU target).
