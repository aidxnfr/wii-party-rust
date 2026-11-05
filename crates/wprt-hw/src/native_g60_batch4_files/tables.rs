// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// Additional copyrights go to Duddie (c) 2005 (duddie@walla.com)
// AI-assisted (Claude, G60) WHOLE DSPTables.cpp replacement. Exports the exact GNU symbols of the pristine object (nm.txt):
// data cw/pdlabels/regnames + 10 functions. Table contents are generated from the pristine object by extract.cpp (tables.rs).
// std::string returns: libstdc++ SSO layout, hidden return pointer (Windows x64 GNU); string_view args by pointer (16-byte aggregate).
#![allow(non_snake_case, non_upper_case_globals, dead_code)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("DSPTables exports require pinned Windows GNU x64 metadata");
#[repr(C)] pub struct P { ty: i32, size: u8, loc: u8, lshift: i8, mask: u16 }
#[repr(C)] pub struct Op { name: *const u8, opcode: u16, opcode_mask: u16, size: u8, param_count: u8, params: [P; 8], extended: bool, branch: bool, uncond_branch: bool, reads_pc: bool, updates_sr: bool }
#[repr(C)] pub struct L { addr: u16, name: *const u8, description: *const u8 }
unsafe impl Sync for Op {}
unsafe impl Sync for L {}
const fn S(b: &'static [u8]) -> *const u8 { b.as_ptr() }
const N: *const u8 = std::ptr::null();
mod t { use super::*; include!(env!("WPRT_DSP_TABLES_DATA")); }
use t::*;
const _: () = assert!(std::mem::size_of::<Op>() == OP_SIZE && std::mem::align_of::<Op>() == OP_ALIGN && std::mem::size_of::<P>() == P_SIZE && std::mem::size_of::<L>() == L_SIZE);
const _: () = assert!(std::mem::offset_of!(Op, params) == OFF_PARAMS && std::mem::offset_of!(Op, extended) == OFF_EXTENDED && std::mem::offset_of!(Op, updates_sr) == OFF_UPDATES_SR && std::mem::offset_of!(P, mask) == P_OFF_MASK && std::mem::offset_of!(L, description) == L_OFF_DESC);

extern "C-unwind" {
    #[link_name = "_Znwy"] fn op_new(n: usize) -> *mut u8;
    #[link_name = "_ZSt19__throw_logic_errorPKc"] fn throw_logic_error(what: *const u8) -> !;
}
#[repr(C)] pub struct StdString { p: *mut u8, len: usize, buf: [u8; 16] }
const _: () = assert!(std::mem::size_of::<StdString>() == 32);
// std::__cxx11::basic_string<char>(const char*) (libstdc++ pinned, C++20).
unsafe fn make_string(ret: *mut StdString, s: *const u8) -> *mut StdString {
    let local = std::ptr::addr_of_mut!((*ret).buf).cast::<u8>();
    (*ret).p = local;
    if s.is_null() { throw_logic_error(b"basic_string: construction from null is not valid\0".as_ptr()); }
    let len = strlen(s);
    let dst = if len > 15 {
        if len > (isize::MAX as usize) / 1 - 1 { throw_length_error(); }
        let d = op_new(len + 1);
        (*ret).p = d;
        local.cast::<usize>().write_unaligned(len); // _M_allocated_capacity
        d
    } else { local };
    if len == 1 { *dst = *s; } else if len != 0 { std::ptr::copy_nonoverlapping(s, dst, len); }
    (*ret).len = len;
    *dst.add(len) = 0;
    ret
}
extern "C" { #[link_name = "strlen"] fn c_strlen(s: *const i8) -> usize; }
unsafe fn strlen(s: *const u8) -> usize { c_strlen(s.cast()) }
extern "C-unwind" { #[link_name = "_ZSt20__throw_length_errorPKc"] fn throw_length_error_c(w: *const u8) -> !; }
unsafe fn throw_length_error() -> ! { throw_length_error_c(b"basic_string::_M_create\0".as_ptr()) }

#[export_name = "_ZN3DSP6pdnameB5cxx11Et"]
pub unsafe extern "C-unwind" fn pdname(ret: *mut StdString, val: u16) -> *mut StdString {
    for l in PDLABELS.iter() { if l.addr == val { return make_string(ret, l.name); } }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let text = [b'0', b'x', HEX[(val >> 12) as usize & 15], HEX[(val >> 8) as usize & 15], HEX[(val >> 4) as usize & 15], HEX[val as usize & 15], 0, 0];
    make_string(ret, text.as_ptr())
}
#[export_name = "_ZN3DSP9pdregnameB5cxx11Ei"]
pub unsafe extern "C-unwind" fn pdregname(ret: *mut StdString, val: i32) -> *mut StdString { make_string(ret, (*REGNAMES.as_ptr().offset(val as isize)).name) }
#[export_name = "_ZN3DSP13pdregnamelongB5cxx11Ei"]
pub unsafe extern "C-unwind" fn pdregnamelong(ret: *mut StdString, val: i32) -> *mut StdString { make_string(ret, (*REGNAMES.as_ptr().offset(val as isize)).description) }

#[repr(C)] pub struct StrView { len: usize, ptr: *const u8 }
unsafe fn find_by_name(name: *const StrView, data: &'static [Op]) -> *const Op {
    let v = &*name;
    for o in data { let n = strlen(o.name); if n == v.len && (n == 0 || std::slice::from_raw_parts(o.name, n) == std::slice::from_raw_parts(v.ptr, n)) { return o; } }
    std::ptr::null()
}
fn find_by_opcode(opcode: u16, data: &'static [Op]) -> *const Op {
    for o in data { if (opcode & o.opcode_mask) == o.opcode { return o; } }
    std::ptr::null()
}
#[export_name = "_ZN3DSP18FindOpInfoByOpcodeEt"] pub extern "C-unwind" fn FindOpInfoByOpcode(opcode: u16) -> *const Op { find_by_opcode(opcode, &S_OPCODES) }
#[export_name = "_ZN3DSP21FindExtOpInfoByOpcodeEt"] pub extern "C-unwind" fn FindExtOpInfoByOpcode(opcode: u16) -> *const Op { find_by_opcode(opcode, &S_OPCODES_EXT) }
#[export_name = "_ZN3DSP16FindOpInfoByNameESt17basic_string_viewIcSt11char_traitsIcEE"] pub unsafe extern "C-unwind" fn FindOpInfoByName(name: *const StrView) -> *const Op { find_by_name(name, &S_OPCODES) }
#[export_name = "_ZN3DSP19FindExtOpInfoByNameESt17basic_string_viewIcSt11char_traitsIcEE"] pub unsafe extern "C-unwind" fn FindExtOpInfoByName(name: *const StrView) -> *const Op { find_by_name(name, &S_OPCODES_EXT) }

static mut S_OP_TABLE: [*const Op; 0x10000] = [std::ptr::null(); 0x10000];
static mut S_EXT_OP_TABLE: [*const Op; 0x100] = [std::ptr::null(); 0x100];
#[export_name = "_ZN3DSP13GetOpTemplateEt"] pub unsafe extern "C-unwind" fn GetOpTemplate(inst: u16) -> *const Op { (*std::ptr::addr_of!(S_OP_TABLE))[inst as usize] }
#[export_name = "_ZN3DSP16GetExtOpTemplateEt"]
pub unsafe extern "C-unwind" fn GetExtOpTemplate(inst: u16) -> *const Op {
    let t = &*std::ptr::addr_of!(S_EXT_OP_TABLE);
    if (inst >> 12) == 0x3 { t[(inst & 0x7f) as usize] } else { t[(inst & 0xff) as usize] }
}
#[export_name = "_ZN3DSP20InitInstructionTableEv"]
pub unsafe extern "C-unwind" fn InitInstructionTable() {
    let cw: *const Op = &CW;
    let ext = &mut *std::ptr::addr_of_mut!(S_EXT_OP_TABLE);
    for i in 0..ext.len() { ext[i] = cw; let it = find_by_opcode(i as u16, &S_OPCODES_EXT); if it.is_null() { continue; } if ext[i] == cw { ext[i] = it; } }
    let op = &mut *std::ptr::addr_of_mut!(S_OP_TABLE);
    op.fill(cw);
    for i in 0..op.len() { let it = find_by_opcode(i as u16, &S_OPCODES); if it.is_null() { continue; } if op[i] == cw { op[i] = it; } }
}
