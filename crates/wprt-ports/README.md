# wprt-ports (O3)

Standalone, prepared Rust ports of eight pristine C++ leaf files. They are not linked into
`outputs/engine/wprt`; H7/Codex performs that integration.

| Rust file | Pristine source |
| --- | --- |
| `src/keymap.rs` | `engine/src/input/keymap.cpp` |
| `src/padmap.rs` | `engine/src/input/padmap.cpp` |
| `src/png.rs` | `engine/src/platform/png.cpp` |
| `src/settings.rs` | `engine/src/platform/settings.cpp` |
| `src/log.rs` | `engine/src/core/log.cpp` |
| `src/format.rs` | `engine/src/core/format.cpp` |
| `src/gx_state.rs` | `engine/src/gpu/gx_state.cpp` |
| `src/gx_lighting.rs` | `engine/src/gpu/gx_lighting.cpp` |

Each port keeps the original function names in snake_case and has a
`// port of engine/src/<path> <function>` comment. Rust-only constructors are named `new`;
the C++ constructor/operator comment names the original member.

## Differential harness

`diff/ref.cpp` includes the eight pristine `.cpp` files and prints one section per source file.
`examples/diff.rs` prints the same sections. `diff/compare.ps1` compares them exactly; the log
section deliberately removes the wall-clock timestamp (both sides still print the complete
formatted message and category, and `log::format_line` is tested via that shape).

Run the whole check (builds the C++ reference and the Rust example, then prints one result per
source file):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\diff\run.ps1
```

The script uses `work/tools/mingw-native/MinGW/bin/g++.exe`; override with `-Gxx <path>`.
Temporary settings/PNG/log files are written in the crate directory and removed by the script.

## Port notes

- `settings` uses a `Mutex<State>` and `AtomicU32` revision; `keymap`/`padmap` preserve the
  original precedence and fallbacks.
- `format` exposes the same register/stack/floating argument model. `format_guest` accepts a
  `Memory` trait instead of the original global memory functions, and calls the C library
  `snprintf` through `libc` for identical conversion formatting.
- `gx_lighting` uses a `OnceLock<Mutex<State>>` in place of the C++ global, preserving all
  channel, material, light and attenuation arithmetic.
- `png` emits the same raw-DEFLATE PNG bytes; `log` and `settings` write through the standard
  library and retain the original field ordering.
