// SPDX-License-Identifier: GPL-3.0-or-later
// WGSL port of kCopyShaderSource from engine/src/gpu/gx_render.cpp @ e8522be (full-screen triangle = kPresentShaderSource vertex_main).

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

// ---- EFB copy (group 2: color source, depth source, sampler, constants) ----
struct CopyConstants {
    params: vec4<u32>,
    region: vec4<f32>,
    filter_: vec4<u32>,
    rows: vec4<f32>,
    extent: vec4<f32>,
}
@group(0) @binding(0) var color_source: texture_2d<f32>;
@group(0) @binding(1) var depth_source: texture_depth_2d;
@group(0) @binding(2) var source_sampler: sampler;
@group(0) @binding(3) var<uniform> copy_constants: CopyConstants;

fn convert(raw_in: vec4<u32>) -> vec4<f32> {
    var raw = raw_in;
    let params = copy_constants.params;
    if (params.z == 0u) {
        raw.a = 255u;
    }
    if (params.y != 0u) {
        let y_const = vec4<f32>(66.0, 129.0, 25.0, 16.0);
        let u_const = vec4<f32>(-38.0, -74.0, 112.0, 128.0);
        let v_const = vec4<f32>(112.0, -94.0, -18.0, 128.0);
        let rgb = vec4<f32>(vec3<f32>(raw.rgb), 256.0);
        // HLSL uint3(dot(...)) : float -> uint conversion (negative values clamp to 0 in both)
        let yuv = vec3<u32>(u32(dot(y_const, rgb)), u32(dot(u_const, rgb)), u32(dot(v_const, rgb)));
        raw = vec4<u32>(min((yuv >> vec3<u32>(8u)) + ((yuv >> vec3<u32>(7u)) & vec3<u32>(1u)), vec3<u32>(255u)), raw.a);
    }
    let value = vec4<f32>(raw) / 255.0;
    switch (params.x) {
        case 0u: {
            let red = f32(raw.r & 0xF0u) / 240.0;
            return vec4<f32>(red, red, red, red);
        }
        case 1u, 8u: { return value.rrrr; }
        case 2u: {
            let red_alpha = vec2<f32>(vec2<u32>(raw.r, raw.a) & vec2<u32>(0xF0u)) / 240.0;
            return red_alpha.xxxy;
        }
        case 3u: { return value.rrra; }
        case 4u: {
            let red_blue = vec2<f32>(vec2<u32>(raw.r, raw.b) & vec2<u32>(0xF8u)) / 248.0;
            let green = f32(raw.g & 0xFCu) / 252.0;
            return vec4<f32>(red_blue.x, green, red_blue.y, 1.0);
        }
        case 5u: { return vec4<f32>(vec3<f32>(raw.rgb & vec3<u32>(0xF8u)) / 248.0, f32(raw.a & 0xE0u) / 224.0); }
        case 7u: { return value.aaaa; }
        case 9u: { return value.gggg; }
        case 10u: { return value.bbbb; }
        case 11u: { return value.rrrg; }
        case 12u: { return value.gggb; }
        case 13u: { return vec4<f32>(value.rgb, 1.0); }
        default: { return value; }
    }
}

fn sample_efb(position: vec2<f32>, offset: f32) -> vec4<u32> {
    let rows = copy_constants.rows;
    let y = clamp(position.y + offset, rows.y, rows.z);
    if (copy_constants.params.w != 0u) {
        // HLSL depth_source.Load(int3(int(x), int(y), 0)) on an R32_FLOAT view of the D32 depth buffer.
        let depth = min(u32(saturate(textureLoad(depth_source, vec2<i32>(i32(position.x), i32(y)), 0)) * 16777216.0), 0xFFFFFFu);
        return vec4<u32>((depth >> 16u) & 255u, (depth >> 8u) & 255u, depth & 255u, 255u);
    }
    return vec4<u32>(round(saturate(textureSampleLevel(color_source, source_sampler, vec2<f32>(position.x, y) / copy_constants.extent.xy, 0.0)) * 255.0));
}

@fragment
fn copy_main(input: FullScreen) -> @location(0) vec4<f32> {
    let filt = copy_constants.filter_;
    let source = copy_constants.region.xy + input.uv * copy_constants.region.zw;
    let current = sample_efb(source, 0.0);
    var combined = current.rgb * filt.y;
    if (filt.x != 0u || filt.z != 0u) {
        combined += sample_efb(source, -1.0).rgb * filt.x + sample_efb(source, 1.0).rgb * filt.z;
    }
    var raw = vec4<u32>(combined >> vec3<u32>(6u), current.a);
    if (filt.w != 0u) {
        raw &= vec4<u32>(0x1FFu);
    }
    raw = min(raw, vec4<u32>(255u));
    if (copy_constants.rows.x != 1.0) {
        raw = vec4<u32>(vec3<u32>(round(pow(vec3<f32>(raw.rgb) / 255.0, vec3<f32>(copy_constants.rows.x)) * 255.0)), raw.a);
    }
    return convert(raw);
}
