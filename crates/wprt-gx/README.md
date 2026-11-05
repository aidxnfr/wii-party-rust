# wprt-gx — renderer backend (Claude's lane, see outputs/EXACT_CODEX_MAP.md "LANES 2026-10-24")
Rust port of engine/src/gpu/gx_render_common.cpp (src/common.rs, done) and a wgpu replacement for the Direct3D 11
backend engine/src/gpu/gx_render.cpp behind the same gx_render.h API (in progress). Codex: read-only for you.

## G1 common.rs differential (PASSED 2026-10-24: 8,600 lines IDENTICAL)
    P=work/private/arel-pristine/engine
    g++ -std=c++20 -O1 -I$P/include -I$P/src/gpu diff/ref.cpp $P/src/gpu/gx_render_common.cpp $P/src/gpu/gx_state.cpp -o ref
    ./ref > ref.txt ; cargo run --release --example common_diff > rust.txt ; compare (LF-normalise only)
Covers decode_texture (all formats, TLUT formats, odd sizes), hash_bytes/sample_hash, blend_state_for,
alpha_test_result, copy formats, expand/quantize, encode_texel, fill_constants (byte hash of the whole struct),
load_tlut, describe_texture (+mips), texture_hash, decode_levels, sampler_params.
