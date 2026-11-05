# Keeping it up to date

Everything that can change is pinned in one place:

| What | Where | How to update |
| --- | --- | --- |
| upstream recompiler | `recompiler/upstream.toml` | change `commit`, run `recompiler/pipeline.py --force`, rebuild, run all fixtures |
| Rust | `rust-toolchain.toml`, `launcher/src/tools.rs` (`RUST_TOOLCHAIN`) | change both, rebuild, run fixtures |
| build tools | `launcher/src/tools.rs` | change url + sha256 (+ docs/BUILDING.md) |
| crates (Bevy, wgpu, …) | `Cargo.toml` files, `Cargo.lock` | `cargo update -p <crate>`; wgpu-hal/naga are patched copies in `third_party/` (re-apply `third_party/patches/*.patch` to the new version) |
| game build features | `crates/game/features.txt` | one feature per line |

## Releasing

1. Run the fixtures (`docs/TESTING.md`).
2. Bump `version` in `launcher/Cargo.toml` and add a CHANGELOG entry.
3. Tag `vX.Y.Z` and push the tag. The release workflow builds the host and the launcher (with the source and host
   embedded) and attaches `wii-party-launcher.exe` to a GitHub release.

Players update by running the new launcher and pressing **Update**: the source is replaced, the generated code is
regenerated only if the recompiler/translator changed, the game DLL is rebuilt incrementally, and `data/` (saves,
settings, logs) is never touched.
