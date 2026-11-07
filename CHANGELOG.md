# Changelog

## 0.1.0-alpha (2026-11-06)
- First release: launcher (Install / Play / Settings / Controls) that builds the game from your own disc.
- All-Rust runtime, hardware and renderer; 23/23 accuracy fixtures exact (known NaN-only rows in F8/F10/F12).
- Performance: idle sleep, input cache, DSP fast path, release/LTO/AVX2 builds, renderer optimizations
  (about 0.6 CPU core at 60 fps on a 7800X3D).
- Optional smooth motion (2x–4x) for high refresh rate screens, off by default.
- Launcher redesign after arelkair's launcher; install progress with time left; Performance window; USA (SUPE01) and Europe (SUPP01) discs.
- Release notes for each version live in docs/releases/<tag>.md; pushing a v* tag publishes the release.
