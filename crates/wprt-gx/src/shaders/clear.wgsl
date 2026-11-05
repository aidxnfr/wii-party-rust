// SPDX-License-Identifier: GPL-3.0-or-later
// WGSL port of kClearShaderSource from engine/src/gpu/gx_render.cpp @ e8522be (full-screen triangle = kPresentShaderSource vertex_main).

struct FullScreen {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// present vertex_main: uv = float2((id << 1) & 2, id & 2); position = uv * (2,-2) + (-1,1)
@vertex
fn vertex_main(@builtin(vertex_index) id: u32) -> FullScreen {
    var output: FullScreen;
    let uv = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
    output.position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    output.uv = uv;
    return output;
}

// ---- clear (group 1: constants) ----
struct ClearConstants {
    color: vec4<f32>,
    depth: vec4<f32>,
}
@group(0) @binding(0) var<uniform> clear_constants: ClearConstants;

struct ClearOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn clear_main(input: FullScreen) -> ClearOutput {
    var output: ClearOutput;
    output.color = clear_constants.color;
    output.depth = clear_constants.depth.x;
    return output;
}

