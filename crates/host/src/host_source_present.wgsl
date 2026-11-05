// SPDX-License-Identifier: GPL-3.0-or-later
// WGSL port of kPresentShaderSource + kOverlayShaderSource from engine/src/gpu/gx_render.cpp @ e8522be (full-screen triangle = kPresentShaderSource vertex_main).

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

// ---- present / overlay (group 0: texture, sampler) ----
@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

@fragment
fn present_main(input: FullScreen) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(frame, frame_sampler, input.uv).rgb, 1.0);
}

@fragment
fn overlay_main(input: FullScreen) -> @location(0) vec4<f32> {
    return textureSample(frame, frame_sampler, input.uv);
}

