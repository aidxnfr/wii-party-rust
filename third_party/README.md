# Vendored, patched crates

`wgpu-hal/` and `naga/` are crates.io **30.0.1** with two small patches (`patches/`):

- `naga`: the HLSL backend writes `[earlydepthstencil]` for pixel shaders that request forced early depth,
  matching the original renderer's early-Z pipelines on DX12.
- `wgpu-hal`: plumbing for that flag in the DX12 adapter/device, plus a matching no-op path for Vulkan.

They are used through `[patch.crates-io]` in `Cargo.toml` and `crates/game/Cargo.toml`.
To move to a newer wgpu: copy the new crates.io sources here, apply `patches/*.patch` (`git apply -p1` from this
folder), fix any conflicts, and run the fixtures. Licenses: MIT OR Apache-2.0 (see each folder).
