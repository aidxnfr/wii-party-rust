// port of engine/src/gpu/gx_lighting.cpp
// Copyright 2009 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

use std::sync::{Mutex, OnceLock};

const K_NORMAL_MATRICES: u32 = 0x400;
const K_LIGHTS: u32 = 0x600;
const K_LIGHT_SIZE: u32 = 16;
const K_AMBIENT_COLOR: u32 = 0x100A;
const K_MATERIAL_COLOR: u32 = 0x100C;
const K_COLOR_CONTROL: u32 = 0x100E;
const K_ALPHA_CONTROL: u32 = 0x1010;

const K_ATTENUATION_NONE: u32 = 0;
const K_ATTENUATION_SPEC: u32 = 1;
const K_ATTENUATION_DIR: u32 = 2;
const K_ATTENUATION_SPOT: u32 = 3;
const K_DIFFUSE_NONE: u32 = 0;
const K_DIFFUSE_SIGN: u32 = 1;
const K_DIFFUSE_CLAMP: u32 = 2;

#[derive(Clone, Copy, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    // port of engine/src/gpu/gx_lighting.cpp operator-
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }

    // port of engine/src/gpu/gx_lighting.cpp operator/
    fn div(self, divisor: f32) -> Vec3 {
        Vec3 { x: self.x / divisor, y: self.y / divisor, z: self.z / divisor }
    }
}

// port of engine/src/gpu/gx_lighting.cpp dot
fn dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

// port of engine/src/gpu/gx_lighting.cpp length_squared
fn length_squared(a: Vec3) -> f32 {
    dot(a, a)
}

// port of engine/src/gpu/gx_lighting.cpp normalized
fn normalized(a: Vec3) -> Vec3 {
    let length = length_squared(a).sqrt();
    if length == 0.0 {
        return a;
    }
    a.div(length)
}

#[derive(Clone, Copy, Default)]
struct Channel {
    material_from_vertex: bool,
    enable_lighting: bool,
    ambient_from_vertex: bool,
    diffuse: u32,
    attenuation: u32,
    mask: u32,
}

// port of engine/src/gpu/gx_lighting.cpp decode_channel
fn decode_channel(hex: u32) -> Channel {
    let mut channel = Channel::default();
    channel.material_from_vertex = (hex & 1) != 0;
    channel.enable_lighting = ((hex >> 1) & 1) != 0;
    channel.ambient_from_vertex = ((hex >> 6) & 1) != 0;
    channel.diffuse = (hex >> 7) & 3;
    channel.attenuation = (hex >> 9) & 3;
    channel.mask = if channel.enable_lighting {
        ((hex >> 2) & 15) | (((hex >> 11) & 15) << 4)
    } else {
        0
    };
    channel
}

#[derive(Clone, Copy, Default)]
struct Light {
    color: [u8; 4],
    cosatt: Vec3,
    distatt: Vec3,
    pos: Vec3,
    dir: Vec3,
}

// port of engine/src/gpu/gx_lighting.cpp word_float
fn word_float(xf: &[u32], index: u32) -> f32 {
    f32::from_bits(xf[index as usize])
}

// port of engine/src/gpu/gx_lighting.cpp word_vec3
fn word_vec3(xf: &[u32], index: u32) -> Vec3 {
    Vec3 {
        x: word_float(xf, index),
        y: word_float(xf, index + 1),
        z: word_float(xf, index + 2),
    }
}

// port of engine/src/gpu/gx_lighting.cpp word_rgba
fn word_rgba(value: u32, out: &mut [u8; 4]) {
    out[0] = (value >> 24) as u8;
    out[1] = (value >> 16) as u8;
    out[2] = (value >> 8) as u8;
    out[3] = value as u8;
}

// port of engine/src/gpu/gx_lighting.cpp read_light
fn read_light(xf: &[u32], number: u32) -> Light {
    let base = K_LIGHTS + number * K_LIGHT_SIZE;
    let mut light = Light::default();
    word_rgba(xf[(base + 3) as usize], &mut light.color);
    light.cosatt = word_vec3(xf, base + 4);
    light.distatt = word_vec3(xf, base + 7);
    light.pos = word_vec3(xf, base + 10);
    light.dir = word_vec3(xf, base + 13);
    light
}

// port of engine/src/gpu/gx_lighting.cpp safe_divide
fn safe_divide(n: f32, d: f32) -> f32 {
    if d == 0.0 {
        if n > 0.0 { 1.0 } else { 0.0 }
    } else {
        n / d
    }
}

// port of engine/src/gpu/gx_lighting.cpp light_attenuation
fn light_attenuation(light: &Light, ldir: &mut Vec3, normal: Vec3, channel: &Channel) -> f32 {
    let mut attn = 1.0f32;
    match channel.attenuation {
        K_ATTENUATION_NONE | K_ATTENUATION_DIR => {
            *ldir = normalized(*ldir);
            if ldir.x == 0.0 && ldir.y == 0.0 && ldir.z == 0.0 {
                *ldir = normal;
            }
        }
        K_ATTENUATION_SPEC => {
            *ldir = normalized(*ldir);
            attn = if dot(*ldir, normal) >= 0.0 {
                dot(light.dir, normal).max(0.0)
            } else {
                0.0
            };
            let att_len = Vec3 { x: 1.0, y: attn, z: attn * attn };
            let cos_attn = light.cosatt;
            let mut dist_attn = light.distatt;
            if channel.diffuse != K_DIFFUSE_NONE {
                dist_attn = normalized(dist_attn);
            }
            attn = safe_divide(dot(att_len, cos_attn).max(0.0), dot(att_len, dist_attn));
        }
        K_ATTENUATION_SPOT => {
            let dist2 = length_squared(*ldir);
            let dist = dist2.sqrt();
            *ldir = ldir.div(dist);
            attn = dot(*ldir, light.dir).max(0.0);
            let cos_att = light.cosatt.x + (light.cosatt.y * attn) + (light.cosatt.z * attn * attn);
            let dist_att = light.distatt.x + (light.distatt.y * dist) + (light.distatt.z * dist2);
            attn = safe_divide(cos_att.max(0.0), dist_att);
        }
        _ => {}
    }
    attn
}

// port of engine/src/gpu/gx_lighting.cpp light_factor
fn light_factor(light: &Light, position: Vec3, normal: Vec3, channel: &Channel) -> f32 {
    let mut ldir = light.pos.sub(position);
    let attn = light_attenuation(light, &mut ldir, normal, channel);
    let dif_attn = dot(ldir, normal);
    match channel.diffuse {
        K_DIFFUSE_SIGN => attn * dif_attn,
        K_DIFFUSE_CLAMP => attn * dif_attn.max(0.0),
        _ => attn,
    }
}

// port of engine/src/gpu/gx_lighting.cpp modulate
fn modulate(material: u8, light: f32) -> u8 {
    let value = (light as i32).clamp(0, 255);
    ((material as i32 * (value + (value >> 7))) >> 8) as u8
}

#[derive(Default)]
struct State {
    color: [Channel; 2],
    alpha: [Channel; 2],
    material: [[u8; 4]; 2],
    ambient: [[u8; 4]; 2],
    lights: [Light; 8],
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn state() -> &'static Mutex<State> {
    STATE.get_or_init(|| Mutex::new(State::default()))
}

// port of engine/src/gpu/gx_lighting.cpp transform_normal
pub fn transform_normal(xf: &[u32], position_matrix: u32, normal: &[f32; 3], out: &mut [f32; 3]) {
    let base = K_NORMAL_MATRICES + (position_matrix & 31) * 3;
    let mut result = Vec3 {
        x: word_float(xf, base) * normal[0] + word_float(xf, base + 1) * normal[1] + word_float(xf, base + 2) * normal[2],
        y: word_float(xf, base + 3) * normal[0] + word_float(xf, base + 4) * normal[1] + word_float(xf, base + 5) * normal[2],
        z: word_float(xf, base + 6) * normal[0] + word_float(xf, base + 7) * normal[1] + word_float(xf, base + 8) * normal[2],
    };
    result = normalized(result);
    out[0] = result.x;
    out[1] = result.y;
    out[2] = result.z;
}

// port of engine/src/gpu/gx_lighting.cpp begin
pub fn begin(xf: &[u32]) {
    let mut state = state().lock().unwrap();
    let mut used = 0u32;
    for k in 0..2usize {
        word_rgba(xf[(K_MATERIAL_COLOR as usize) + k], &mut state.material[k]);
        word_rgba(xf[(K_AMBIENT_COLOR as usize) + k], &mut state.ambient[k]);
        state.color[k] = decode_channel(xf[(K_COLOR_CONTROL as usize) + k]);
        state.alpha[k] = decode_channel(xf[(K_ALPHA_CONTROL as usize) + k]);
        used |= state.color[k].mask | state.alpha[k].mask;
    }
    for i in 0..8 {
        if used & (1 << i) != 0 {
            state.lights[i] = read_light(xf, i as u32);
        }
    }
}

// port of engine/src/gpu/gx_lighting.cpp light_channels
pub fn light_channels(position: &[f32; 3], normal: &[f32; 3], vertex_color: &[[u8; 4]; 2], out: &mut [[u8; 4]; 2]) {
    let state = state().lock().unwrap();
    let pos = Vec3 { x: position[0], y: position[1], z: position[2] };
    let nrm = Vec3 { x: normal[0], y: normal[1], z: normal[2] };
    for k in 0..2usize {
        let material = &state.material[k];
        let ambient = &state.ambient[k];
        let color = &state.color[k];
        let color_material: &[u8; 4] = if color.material_from_vertex { &vertex_color[k] } else { material };
        if color.enable_lighting {
            let base: &[u8; 4] = if color.ambient_from_vertex { &vertex_color[k] } else { ambient };
            let mut light = [base[0] as f32, base[1] as f32, base[2] as f32];
            for i in 0..8usize {
                if color.mask & (1 << i) != 0 {
                    let source = &state.lights[i];
                    let factor = light_factor(source, pos, nrm, color);
                    for c in 0..3 {
                        light[c] += source.color[c] as f32 * factor;
                    }
                }
            }
            for c in 0..3 {
                out[k][c] = modulate(color_material[c], light[c]);
            }
        } else {
            for c in 0..3 {
                out[k][c] = color_material[c];
            }
        }

        let alpha = &state.alpha[k];
        let alpha_material = if alpha.material_from_vertex { vertex_color[k][3] } else { material[3] };
        if alpha.enable_lighting {
            let mut light = if alpha.ambient_from_vertex { vertex_color[k][3] as f32 } else { ambient[3] as f32 };
            for i in 0..8usize {
                if alpha.mask & (1 << i) != 0 {
                    let source = &state.lights[i];
                    light += source.color[3] as f32 * light_factor(source, pos, nrm, alpha);
                }
            }
            out[k][3] = modulate(alpha_material, light);
        } else {
            out[k][3] = alpha_material;
        }
    }
}
