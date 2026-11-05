# Changelog

## 0.1.0
- First release: launcher (Install / Play / Settings / Controls) that builds the game from your own disc.
- All-Rust runtime, hardware and renderer; 23/23 accuracy fixtures exact (known NaN-only rows in F8/F10/F12).
- Performance: idle sleep, input cache, DSP fast path, release/LTO/AVX2 builds, renderer optimizations
  (about 0.6 CPU core at 60 fps on a 7800X3D).
- Optional smooth motion (2x–4x) for high refresh rate screens, off by default.
