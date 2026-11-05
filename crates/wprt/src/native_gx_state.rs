// SPDX-License-Identifier: GPL-3.0-or-later
// H7 C ABI adapter reusing OpenCode's O3 gx_state.cpp port unchanged.
#[path = "prepared_gx_state.rs"]
mod prepared;
#[repr(C)]
pub struct RawBlend {
    enable: u32,
    operation: u32,
    alpha_operation: u32,
    source: u32,
    destination: u32,
    source_alpha: u32,
    destination_alpha: u32,
    color_update: u32,
    alpha_update: u32,
    logic_op: u32,
    logic_mode: u32,
    output: u32,
}
// port of gx_state.cpp blend_state; native adapter marshals fields rather than assuming C++ aggregate layout.
#[no_mangle]
pub unsafe extern "C" fn wprt_gx_blend_state(
    mode: u32,
    alpha: bool,
    constant: bool,
    pass: bool,
    out: *mut RawBlend,
) {
    let s = prepared::blend_state(mode, alpha, constant, pass);
    out.write(RawBlend {
        enable: s.enable as u32,
        operation: s.operation as u32,
        alpha_operation: s.alpha_operation as u32,
        source: s.source as u32,
        destination: s.destination as u32,
        source_alpha: s.source_alpha as u32,
        destination_alpha: s.destination_alpha as u32,
        color_update: s.color_update as u32,
        alpha_update: s.alpha_update as u32,
        logic_op: s.logic_op as u32,
        logic_mode: s.logic_mode,
        output: s.output as u32,
    });
}
// port of gx_state.cpp logic_op_exact.
#[no_mangle]
pub extern "C" fn wprt_gx_logic_exact(mode: u32) -> bool {
    prepared::logic_op_exact(mode)
}
// port of gx_state.cpp kLogicNames/logic_op_name. NUL-terminated bytes extracted from pristine source table.
static NAMES: [&[u8]; 16] = [
    b"clear\0",
    b"and\0",
    b"and_reverse\0",
    b"copy\0",
    b"and_inverted\0",
    b"noop\0",
    b"xor\0",
    b"or\0",
    b"nor\0",
    b"equiv\0",
    b"invert\0",
    b"or_reverse\0",
    b"copy_inverted\0",
    b"or_inverted\0",
    b"nand\0",
    b"set\0",
];
#[no_mangle]
pub extern "C" fn wprt_gx_logic_name(mode: u32) -> *const std::ffi::c_char {
    NAMES[(mode & 15) as usize].as_ptr().cast()
}
#[repr(C)]
pub struct RawOffsets {
    line: f32,
    point: f32,
    line_coordinates: u32,
    point_coordinates: u32,
}
// port of gx_state.cpp line_point_offsets. Source's largest read is bp[0x30+2*7].
#[no_mangle]
pub unsafe extern "C" fn wprt_gx_line_point_offsets(bp: *const u32, out: *mut RawOffsets) {
    let s = prepared::line_point_offsets(std::slice::from_raw_parts(bp, 0x30 + 2 * 7 + 1));
    out.write(RawOffsets {
        line: s.line,
        point: s.point,
        line_coordinates: s.line_coordinates,
        point_coordinates: s.point_coordinates,
    });
}
// port of gx_state.cpp line_offset_negative_side.
#[no_mangle]
pub extern "C" fn wprt_gx_line_offset_negative(dx: f32, dy: f32, tall: bool) -> bool {
    prepared::line_offset_negative_side(dx, dy, tall)
}
// port of gx_state.cpp offset_texture_coordinates; marshal only the source-owned uv field.
#[no_mangle]
pub unsafe extern "C" fn wprt_gx_offset_texcoords(uv: *mut f32, coordinates: u32, s: f32, t: f32) {
    let mut v = prepared::ScreenVertex::default();
    for i in 0..8 {
        for j in 0..3 {
            v.uv[i][j] = *uv.add(3 * i + j);
        }
    }
    prepared::offset_texture_coordinates(&mut v, coordinates, s, t);
    for i in 0..8 {
        if coordinates & (1 << i) != 0 {
            *uv.add(3 * i) = v.uv[i][0];
            *uv.add(3 * i + 1) = v.uv[i][1];
        }
    }
}
