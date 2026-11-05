// SPDX-License-Identifier: GPL-3.0-or-later
// WGSL port of kShaderSource (engine/src/gpu/gx_render.cpp, the GX TEV ubershader) @ e8522be, line by line.
// Bindings: 0 = Constants, 1..8 = t0..t7, 9..16 = s0..s7. Dual-source output replaces SV_Target0/SV_Target1.
enable dual_source_blending;

struct Constants {
    initial: array<vec4<f32>, 4>,
    konst: array<vec4<f32>, 4>,
    stage: array<vec4<u32>, 16>,
    header: vec4<u32>,
    swaps: vec4<u32>,
    texdims: array<vec4<f32>, 8>,
    indmtx: array<vec4<i32>, 6>,
    indirect: vec4<u32>,
    tevind: array<vec4<u32>, 16>,
    fog: vec4<f32>,
    fog_integer: vec4<i32>,
    fog_range: array<vec4<f32>, 3>,
}

@group(0) @binding(0) var<uniform> k_: Constants;
@group(0) @binding(1) var t0: texture_2d<f32>;
@group(0) @binding(2) var t1: texture_2d<f32>;
@group(0) @binding(3) var t2: texture_2d<f32>;
@group(0) @binding(4) var t3: texture_2d<f32>;
@group(0) @binding(5) var t4: texture_2d<f32>;
@group(0) @binding(6) var t5: texture_2d<f32>;
@group(0) @binding(7) var t6: texture_2d<f32>;
@group(0) @binding(8) var t7: texture_2d<f32>;
@group(0) @binding(9) var s0: sampler;
@group(0) @binding(10) var s1: sampler;
@group(0) @binding(11) var s2: sampler;
@group(0) @binding(12) var s3: sampler;
@group(0) @binding(13) var s4: sampler;
@group(0) @binding(14) var s5: sampler;
@group(0) @binding(15) var s6: sampler;
@group(0) @binding(16) var s7: sampler;
// D3D11 sampler MipLODBias has no wgpu sampler field: applied by scaling the explicit gradients by 2^bias
// (LOD = log2(|grad|) + bias). Bias is 0 (factor exactly 1.0) unless the GX mip mode is on.
@group(0) @binding(17) var<uniform> lod_bias: array<vec4<f32>, 2>;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) c0: vec4<f32>,
    @location(2) c1: vec4<f32>,
    @location(3) uv0: vec3<f32>,
    @location(4) uv1: vec3<f32>,
    @location(5) uv2: vec3<f32>,
    @location(6) uv3: vec3<f32>,
    @location(7) uv4: vec3<f32>,
    @location(8) uv5: vec3<f32>,
    @location(9) uv6: vec3<f32>,
    @location(10) uv7: vec3<f32>,
}

struct PixelInput {
    @builtin(position) position: vec4<f32>,
    @location(0) c0: vec4<f32>,
    @location(1) c1: vec4<f32>,
    @location(2) uv0: vec3<f32>,
    @location(3) uv1: vec3<f32>,
    @location(4) uv2: vec3<f32>,
    @location(5) uv3: vec3<f32>,
    @location(6) uv4: vec3<f32>,
    @location(7) uv5: vec3<f32>,
    @location(8) uv6: vec3<f32>,
    @location(9) uv7: vec3<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> PixelInput {
    var output: PixelInput;
    output.position = input.position;
    output.c0 = input.c0;
    output.c1 = input.c1;
    output.uv0 = input.uv0;
    output.uv1 = input.uv1;
    output.uv2 = input.uv2;
    output.uv3 = input.uv3;
    output.uv4 = input.uv4;
    output.uv5 = input.uv5;
    output.uv6 = input.uv6;
    output.uv7 = input.uv7;
    return output;
}

fn projected(uv: vec3<f32>) -> vec2<f32> {
    // HLSL: uv.z == 0.0 ? clamp(uv.xy / 2.0, -1.0, 1.0) : uv.xy / uv.z   (only the taken branch matters: no side effects)
    if (uv.z == 0.0) {
        return clamp(uv.xy / 2.0, vec2<f32>(-1.0), vec2<f32>(1.0));
    }
    return uv.xy / uv.z;
}

fn select_uv(p: PixelInput, i: u32) -> vec2<f32> {
    switch (i) {
        case 0u: { return projected(p.uv0); }
        case 1u: { return projected(p.uv1); }
        case 2u: { return projected(p.uv2); }
        case 3u: { return projected(p.uv3); }
        case 4u: { return projected(p.uv4); }
        case 5u: { return projected(p.uv5); }
        case 6u: { return projected(p.uv6); }
        default: { return projected(p.uv7); }
    }
}

fn sample_map(m: u32, uv: vec2<f32>, dx_in: vec2<f32>, dy_in: vec2<f32>) -> vec4<f32> {
    let bias = lod_bias[m >> 2u][m & 3u];
    var dx = dx_in;
    var dy = dy_in;
    if (bias != 0.0) {
        dx = dx_in * exp2(bias);
        dy = dy_in * exp2(bias);
    }
    switch (m) {
        case 0u: { return textureSampleGrad(t0, s0, uv, dx, dy); }
        case 1u: { return textureSampleGrad(t1, s1, uv, dx, dy); }
        case 2u: { return textureSampleGrad(t2, s2, uv, dx, dy); }
        case 3u: { return textureSampleGrad(t3, s3, uv, dx, dy); }
        case 4u: { return textureSampleGrad(t4, s4, uv, dx, dy); }
        case 5u: { return textureSampleGrad(t5, s5, uv, dx, dy); }
        case 6u: { return textureSampleGrad(t6, s6, uv, dx, dy); }
        default: { return textureSampleGrad(t7, s7, uv, dx, dy); }
    }
}

// HLSL kFractions[8] = {255, 223, 191, 159, 128, 96, 64, 32}. Written as a switch: AMD's Vulkan compiler (LLPC, driver 26.10.1)
// returned 0 from the function-local constant-array version (measured: Mii konst alpha 255 came out 0 on Vulkan, exact on DX12).
fn fractions(i: u32) -> i32 {
    switch (i) {
        case 0u: { return 255; }
        case 1u: { return 223; }
        case 2u: { return 191; }
        case 3u: { return 159; }
        case 4u: { return 128; }
        case 5u: { return 96; }
        case 6u: { return 64; }
        default: { return 32; }
    }
}

fn quantized(value: vec4<f32>) -> vec4<i32> {
    return vec4<i32>(round(value * 255.0));
}

fn konst_color(sel: u32, k: array<vec4<i32>, 4>) -> vec3<i32> {
    var kk = k;
    if (sel < 8u) {
        return vec3<i32>(1, 1, 1) * fractions(sel);
    }
    if (sel >= 12u && sel < 16u) {
        return kk[sel - 12u].rgb;
    }
    if (sel >= 16u) {
        let index = (sel - 16u) & 3u;
        let component = (sel - 16u) >> 2u;
        var v = kk[index];
        return vec3<i32>(1, 1, 1) * v[component];
    }
    return vec3<i32>(0, 0, 0);
}

fn konst_alpha(sel: u32, k: array<vec4<i32>, 4>) -> i32 {
    var kk = k;
    if (sel < 8u) {
        return fractions(sel);
    }
    if (sel >= 16u) {
        let index = (sel - 16u) & 3u;
        let component = (sel - 16u) >> 2u;
        var v = kk[index];
        return v[component];
    }
    return 0;
}

fn color_input(sel: u32, r: array<vec4<i32>, 4>, tex: vec4<i32>, ras: vec4<i32>, kc: vec3<i32>) -> vec3<i32> {
    switch (sel) {
        case 0u: { return r[0].rgb; }
        case 1u: { return r[0].aaa; }
        case 2u: { return r[1].rgb; }
        case 3u: { return r[1].aaa; }
        case 4u: { return r[2].rgb; }
        case 5u: { return r[2].aaa; }
        case 6u: { return r[3].rgb; }
        case 7u: { return r[3].aaa; }
        case 8u: { return tex.rgb; }
        case 9u: { return tex.aaa; }
        case 10u: { return ras.rgb; }
        case 11u: { return ras.aaa; }
        case 12u: { return vec3<i32>(255, 255, 255); }
        case 13u: { return vec3<i32>(128, 128, 128); }
        case 14u: { return kc; }
        default: { return vec3<i32>(0, 0, 0); }
    }
}

fn alpha_input(sel: u32, r: array<vec4<i32>, 4>, tex: vec4<i32>, ras: vec4<i32>, ka: i32) -> i32 {
    switch (sel) {
        case 0u: { return r[0].a; }
        case 1u: { return r[1].a; }
        case 2u: { return r[2].a; }
        case 3u: { return r[3].a; }
        case 4u: { return tex.a; }
        case 5u: { return ras.a; }
        case 6u: { return ka; }
        default: { return 0; }
    }
}

fn compare_alpha(mode: u32, value: u32, reference: u32) -> bool {
    switch (mode) {
        case 0u: { return false; }
        case 1u: { return value < reference; }
        case 2u: { return value == reference; }
        case 3u: { return value <= reference; }
        case 4u: { return value > reference; }
        case 5u: { return value != reference; }
        case 6u: { return value >= reference; }
        default: { return true; }
    }
}

fn compare_inputs(mode: u32, equal: bool, a: vec3<i32>, b: vec3<i32>) -> bool {
    var left = a.r;
    var right = b.r;
    if (mode == 1u) {
        left = a.r + a.g * 256;
        right = b.r + b.g * 256;
    } else if (mode == 2u) {
        left = a.r + a.g * 256 + a.b * 65536;
        right = b.r + b.g * 256 + b.b * 65536;
    }
    if (equal) { return left == right; }
    return left > right;
}

fn compare_color(mode: u32, equal: bool, a: vec3<i32>, b: vec3<i32>, c: vec3<i32>) -> vec3<i32> {
    if (mode == 3u) {
        var passed: vec3<bool>;
        if (equal) { passed = a == b; } else { passed = a > b; }
        return select(vec3<i32>(0, 0, 0), c, passed);
    }
    if (compare_inputs(mode, equal, a, b)) { return c; }
    return vec3<i32>(0, 0, 0);
}

fn apply_swap(value: vec4<i32>, table: u32) -> vec4<i32> {
    let packed = k_.swaps[table];
    return vec4<i32>(value[packed & 3u], value[(packed >> 2u) & 3u], value[(packed >> 4u) & 3u], value[(packed >> 6u) & 3u]);
}

fn combine(a: vec3<i32>, b: vec3<i32>, c: vec3<i32>, d: vec3<i32>, bias_code: u32, subtract: bool, scale_code: u32) -> vec3<i32> {
    var bias = 0;
    if (bias_code == 1u) { bias = 128; } else if (bias_code == 2u) { bias = -128; }
    var shift = 0u;
    if (scale_code == 1u) { shift = 1u; } else if (scale_code == 2u) { shift = 2u; }
    var rounding = 128;
    if (scale_code == 3u) { rounding = 0; } else if (subtract) { rounding = 127; }
    let lerp_v = ((((a << vec3<u32>(8u)) + (b - a) * (c + (c >> vec3<u32>(7u)))) << vec3<u32>(shift)) + vec3<i32>(rounding)) >> vec3<u32>(8u);
    let base = (d + vec3<i32>(bias)) << vec3<u32>(shift);
    var value: vec3<i32>;
    if (subtract) { value = base - lerp_v; } else { value = base + lerp_v; }
    if (scale_code == 3u) { return value >> vec3<u32>(1u); }
    return value;
}

// kWraps / kFormatShifts / kBumpShifts as switches (same reason as fractions).
fn wraps(i: u32) -> i32 {
    switch (i) {
        case 0u: { return 256 << 7u; }
        case 1u: { return 128 << 7u; }
        case 2u: { return 64 << 7u; }
        case 3u: { return 32 << 7u; }
        default: { return 16 << 7u; }
    }
}
fn format_shift(i: u32) -> u32 {
    switch (i) {
        case 0u: { return 0u; }
        case 1u: { return 3u; }
        case 2u: { return 4u; }
        default: { return 5u; }
    }
}
fn bump_shift(i: u32) -> u32 {
    switch (i) {
        case 0u: { return 0u; }
        case 1u: { return 5u; }
        case 2u: { return 4u; }
        default: { return 3u; }
    }
}

fn fixpoint(p: PixelInput, coord: u32) -> vec2<i32> {
    return vec2<i32>(select_uv(p, coord) * k_.texdims[coord].zw * 128.0);
}

fn sample_fixed(map: u32, coords: vec2<i32>, dx: vec2<f32>, dy: vec2<f32>, coord: u32) -> vec4<i32> {
    let size = k_.texdims[map].xy;
    let ratio = k_.texdims[coord].zw / size;
    return quantized(sample_map(map, vec2<f32>(coords) / (size * 128.0), dx * ratio, dy * ratio));
}

struct PixelOutput {
    @location(0) @blend_src(0) color: vec4<f32>,
    @location(0) @blend_src(1) blend: vec4<f32>,
}

// HLSL ints are 32-bit two's complement with wrapping arithmetic; WGSL i32 arithmetic wraps too.
fn pixel_body(p: PixelInput) -> PixelOutput {
    var gradient_x = array<vec2<f32>, 8>(dpdx(projected(p.uv0)), dpdx(projected(p.uv1)), dpdx(projected(p.uv2)), dpdx(projected(p.uv3)),
                                         dpdx(projected(p.uv4)), dpdx(projected(p.uv5)), dpdx(projected(p.uv6)), dpdx(projected(p.uv7)));
    var gradient_y = array<vec2<f32>, 8>(dpdy(projected(p.uv0)), dpdy(projected(p.uv1)), dpdy(projected(p.uv2)), dpdy(projected(p.uv3)),
                                         dpdy(projected(p.uv4)), dpdy(projected(p.uv5)), dpdy(projected(p.uv6)), dpdy(projected(p.uv7)));
    let k = array<vec4<i32>, 4>(quantized(k_.konst[0]), quantized(k_.konst[1]), quantized(k_.konst[2]), quantized(k_.konst[3]));
    var r = array<vec4<i32>, 4>(quantized(k_.initial[0]), quantized(k_.initial[1]), quantized(k_.initial[2]), quantized(k_.initial[3]));
    let c0 = quantized(p.c0);
    let c1 = quantized(p.c1);
    let count = k_.header.x;
    if (count == 0u) {
        r[0] = c0;
    }
    let indirect_stages = k_.indirect.x & 7u;
    let texgens = (k_.indirect.x >> 8u) & 15u;
    var indirect_texel = array<vec3<i32>, 4>(vec3<i32>(0), vec3<i32>(0), vec3<i32>(0), vec3<i32>(0));
    for (var j = 0u; j < indirect_stages && j < 4u; j++) {
        let reference = k_.indirect.y >> (6u * j);
        let map = reference & 7u;
        var coord = (reference >> 3u) & 7u;
        if (coord >= texgens) {
            coord = 0u;
        }
        var scales = k_.indirect.w;
        if (j < 2u) { scales = k_.indirect.z; }
        let shift_s = (scales >> ((j & 1u) * 8u)) & 15u;
        let shift_t = (scales >> ((j & 1u) * 8u + 4u)) & 15u;
        let base = fixpoint(p, coord);
        indirect_texel[j] = sample_fixed(map, vec2<i32>(base.x >> shift_s, base.y >> shift_t), gradient_x[coord], gradient_y[coord], coord).abg;
    }
    var tevcoord = vec2<i32>(0, 0);
    var alphabump = 0;
    for (var i = 0u; i < count; i++) {
        let ce = k_.stage[i].x;
        let ae = k_.stage[i].y;
        let order = k_.stage[i].z;
        let ksel = k_.stage[i].w;
        var coord = (order >> 3u) & 7u;
        if (coord >= texgens) {
            coord = 0u;
        }
        let ind = k_.tevind[i].x;
        let bt = ind & 3u;
        let format = (ind >> 2u) & 3u;
        let bias = (ind >> 4u) & 7u;
        let bump = (ind >> 7u) & 3u;
        let matrix_index = (ind >> 9u) & 3u;
        let matrix_id = (ind >> 11u) & 3u;
        let wrap_s = (ind >> 13u) & 7u;
        let wrap_t = (ind >> 16u) & 7u;
        let has_stage = bt < indirect_stages;
        let base = fixpoint(p, coord);
        var offset = vec2<i32>(0, 0);
        if (has_stage && bump != 0u) {
            var texel = indirect_texel[bt];
            alphabump = (texel[bump - 1u] << bump_shift(format)) & 248;
        }
        if (has_stage && matrix_index != 0u) {
            var crd = indirect_texel[bt] >> vec3<u32>(format_shift(format));
            var add = 1;
            if (format == 0u) { add = -128; }
            if ((bias & 1u) != 0u) { crd.x += add; }
            if ((bias & 2u) != 0u) { crd.y += add; }
            if ((bias & 4u) != 0u) { crd.z += add; }
            let m = 2u * (matrix_index - 1u);
            if (matrix_id == 0u) {
                offset = vec2<i32>(dot(k_.indmtx[m].xyz, crd), dot(k_.indmtx[m + 1u].xyz, crd)) >> vec2<u32>(3u);
            } else if (matrix_id == 1u) {
                offset = (base * crd.xx) >> vec2<u32>(8u);
            } else if (matrix_id == 2u) {
                offset = (base * crd.yy) >> vec2<u32>(8u);
            }
            let shift = k_.indmtx[m].w;
            if (shift >= 0) { offset = offset >> vec2<u32>(u32(shift)); } else { offset = offset << vec2<u32>(u32(-shift)); }
        }
        var wrapped: vec2<i32>;
        if (wrap_s == 0u) { wrapped.x = base.x; } else if (wrap_s >= 6u) { wrapped.x = 0; } else { wrapped.x = base.x & (wraps(wrap_s - 1u) - 1); }
        if (wrap_t == 0u) { wrapped.y = base.y; } else if (wrap_t >= 6u) { wrapped.y = 0; } else { wrapped.y = base.y & (wraps(wrap_t - 1u) - 1); }
        if (((ind >> 20u) & 1u) != 0u) { tevcoord = tevcoord + wrapped + offset; } else { tevcoord = wrapped + offset; }
        tevcoord = (tevcoord << vec2<u32>(8u)) >> vec2<u32>(8u);
        var tex = vec4<i32>(255, 255, 255, 255);
        if ((order & 0x40u) != 0u) {
            if (texgens == 0u) {
                tex = vec4<i32>(0, 0, 0, 0);
            } else {
                tex = apply_swap(sample_fixed(order & 7u, tevcoord, gradient_x[coord], gradient_y[coord], coord), (ae >> 2u) & 3u);
            }
        }
        var ras = vec4<i32>(0, 0, 0, 0);
        let chan = (order >> 7u) & 7u;
        if (chan == 0u) {
            ras = c0;
        } else if (chan == 1u) {
            ras = c1;
        } else if (chan == 5u) {
            ras = vec4<i32>(1, 1, 1, 1) * alphabump;
        } else if (chan == 6u) {
            ras = vec4<i32>(1, 1, 1, 1) * (alphabump | (alphabump >> 5u));
        }
        ras = apply_swap(ras, ae & 3u);
        let kc = konst_color(ksel & 31u, k);
        let ka = konst_alpha((ksel >> 5u) & 31u, k);
        let a = color_input((ce >> 12u) & 15u, r, tex, ras, kc) & vec3<i32>(255);
        let b = color_input((ce >> 8u) & 15u, r, tex, ras, kc) & vec3<i32>(255);
        let c = color_input((ce >> 4u) & 15u, r, tex, ras, kc) & vec3<i32>(255);
        let d = color_input(ce & 15u, r, tex, ras, kc);
        let bias_code = (ce >> 16u) & 3u;
        let scale_code = (ce >> 20u) & 3u;
        var color: vec3<i32>;
        if (bias_code == 3u) {
            color = d + compare_color(scale_code, ((ce >> 18u) & 1u) != 0u, a, b, c);
        } else {
            color = combine(a, b, c, d, bias_code, ((ce >> 18u) & 1u) != 0u, scale_code);
        }
        if (((ce >> 19u) & 1u) != 0u) { color = clamp(color, vec3<i32>(0), vec3<i32>(255)); } else { color = clamp(color, vec3<i32>(-1024), vec3<i32>(1023)); }
        let aa = alpha_input((ae >> 13u) & 7u, r, tex, ras, ka) & 255;
        let ab = alpha_input((ae >> 10u) & 7u, r, tex, ras, ka) & 255;
        let ac = alpha_input((ae >> 7u) & 7u, r, tex, ras, ka) & 255;
        let ad = alpha_input((ae >> 4u) & 7u, r, tex, ras, ka);
        let abias_code = (ae >> 16u) & 3u;
        let ascale_code = (ae >> 20u) & 3u;
        var alpha: i32;
        if (abias_code == 3u) {
            let aequal = ((ae >> 18u) & 1u) != 0u;
            var apass: bool;
            if (ascale_code == 3u) {
                if (aequal) { apass = aa == ab; } else { apass = aa > ab; }
            } else {
                // HLSL passes the COLOR inputs a, b here (original behaviour, kept as is).
                apass = compare_inputs(ascale_code, aequal, a, b);
            }
            alpha = ad;
            if (apass) { alpha = ad + ac; }
        } else {
            alpha = combine(vec3<i32>(aa, 0, 0), vec3<i32>(ab, 0, 0), vec3<i32>(ac, 0, 0), vec3<i32>(ad, 0, 0), abias_code, ((ae >> 18u) & 1u) != 0u, ascale_code).x;
        }
        if (((ae >> 19u) & 1u) != 0u) { alpha = clamp(alpha, 0, 255); } else { alpha = clamp(alpha, -1024, 1023); }
        let color_dest = (ce >> 22u) & 3u;
        let alpha_dest = (ae >> 22u) & 3u;
        r[color_dest] = vec4<i32>(color, r[color_dest].a);
        r[alpha_dest].a = alpha;
    }
    var result = r[0];
    if (count > 0u) {
        let last_color = (k_.stage[count - 1u].x >> 22u) & 3u;
        let last_alpha = (k_.stage[count - 1u].y >> 22u) & 3u;
        result = vec4<i32>(r[last_color].rgb, r[last_alpha].a);
    }
    result = result & vec4<i32>(255);
    if ((k_.header.w & 2u) != 0u) {
        let dither = vec2<i32>(p.position.xy) & vec2<i32>(1);
        result = vec4<i32>((result.rgb - (result.rgb >> vec3<u32>(6u))) + vec3<i32>((dither.x ^ dither.y) * 2 + dither.y), result.a);
    }
    let fog_select = u32(k_.fog_integer.x) & 15u;
    let fog_type = fog_select >> 1u;
    if (fog_type != 0u) {
        let z = i32(min(bitcast<u32>(i32(p.position.z * 16777216.0)), 0xFFFFFFu));
        var ze: f32;
        if ((fog_select & 1u) == 0u) {
            ze = (k_.fog.x * 16777216.0) / f32(k_.fog_integer.y - (z >> u32(k_.fog_integer.w)));
        } else {
            ze = k_.fog.x * f32(z) / 16777216.0;
        }
        if ((k_.fog_integer.x & 16) != 0) {
            let offset = (2.0 * (p.position.x / k_.fog.w)) - 1.0 - k_.fog.z;
            let index = clamp(9.0 - abs(offset) * 9.0, 0.0, 9.0);
            let lower = u32(index);
            let upper = min(lower + 1u, 9u);
            let kk = mix(k_.fog_range[lower >> 2u][lower & 3u], k_.fog_range[upper >> 2u][upper & 3u], fract(index));
            ze *= sqrt(offset * offset + kk * kk) / kk;
        }
        var amount = clamp(ze - k_.fog.y, 0.0, 1.0);
        if (fog_type == 4u) {
            amount = 1.0 - exp2(-8.0 * amount);
        } else if (fog_type == 5u) {
            amount = 1.0 - exp2(-8.0 * amount * amount);
        } else if (fog_type == 6u) {
            amount = exp2(-8.0 * (1.0 - amount));
        } else if (fog_type == 7u) {
            amount = exp2(-8.0 * (1.0 - amount) * (1.0 - amount));
        }
        let weight = i32(round(amount * 256.0));
        let fog_color = vec3<i32>((k_.fog_integer.z >> 16u) & 255, (k_.fog_integer.z >> 8u) & 255, k_.fog_integer.z & 255);
        result = vec4<i32>((result.rgb * (256 - weight) + fog_color * weight) >> vec3<u32>(8u), result.a);
    }
    let compare = k_.header.y;
    let value = u32(result.a);
    let first = compare_alpha((compare >> 16u) & 7u, value, compare & 255u);
    let second = compare_alpha((compare >> 19u) & 7u, value, (compare >> 8u) & 255u);
    let logic = (compare >> 22u) & 3u;
    var accepted: bool;
    if (logic == 0u) { accepted = first && second; } else if (logic == 1u) { accepted = first || second; } else if (logic == 2u) { accepted = first != second; } else { accepted = first == second; }
    if (!accepted) {
        discard;
    }
    var output: PixelOutput;
    output.blend = vec4<f32>(result) / 255.0;
    let source = (k_.header.w >> 2u) & 3u;
    var written: vec4<i32>;
    if (source == 0u) { written = result; } else if (source == 1u) { written = vec4<i32>(0); } else if (source == 2u) { written = vec4<i32>(255); } else { written = vec4<i32>(255) - result; }
    output.color = vec4<f32>(written) / 255.0;
    if ((k_.header.z & 0x100u) != 0u) {
        output.color.a = f32(k_.header.z & 255u) / 255.0;
    }
    if ((k_.header.w & 1u) != 0u) {
        output.color = vec4<f32>(vec3<f32>(written.rgb >> vec3<u32>(2u)) / 63.0, output.color.a);
        var src_alpha = written.a;
        if ((k_.header.z & 0x100u) != 0u) { src_alpha = i32(k_.header.z & 255u); }
        output.color.a = f32(src_alpha >> 2u) / 63.0;
    }
    return output;
}

@fragment
fn pixel_main(p: PixelInput) -> PixelOutput {
    return pixel_body(p);
}

// Same values without the second (blend) output, for pipelines whose blend state never reads Src1. Needed for exactness on
// AMD's Vulkan driver (26.10.1 LLPC, RX 7900 XT): with a dual-source shader and blending disabled it wrote alpha 0 instead of
// SV_Target0.a (measured: claude-gx observe gx013/gx014 vs ref003 and the DX12 run gx016, copy 9358ce40 at frame 523).
struct SingleOutput {
    @location(0) color: vec4<f32>,
}

@fragment
fn pixel_main_single(p: PixelInput) -> SingleOutput {
    var output: SingleOutput;
    output.color = pixel_body(p).color;
    return output;
}
