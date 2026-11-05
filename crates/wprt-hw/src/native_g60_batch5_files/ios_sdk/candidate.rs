// Claude G60: Rust port of hybrid ios_sdk.cpp (all non-std::filesystem bodies). Residue: ios_filesystem_residue.cpp.
#![allow(non_upper_case_globals, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_void};
include!("../common/cxx.rs");
extern "C-unwind" {
    fn wprt_ios_open(p: *const u8, n: usize, mode: u32) -> i32;
    #[link_name = "_ZN2wp4nand4openERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEj"] fn nand_open(p: *const CxxString, mode: u32) -> i32;
    #[link_name = "_ZN2wp4nand16create_directoryERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn nand_create_directory(p: *const CxxString) -> i32;
    #[link_name = "_ZN2wp4nand11create_fileERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn nand_create_file(p: *const CxxString) -> i32;
    #[link_name = "_ZN2wp4nand6removeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn nand_remove(p: *const CxxString) -> i32;
    #[link_name = "_ZN2wp4nand6renameERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_"] fn nand_rename(a: *const CxxString, b: *const CxxString) -> i32;
    #[link_name = "_ZN2wp4nand6existsERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn nand_exists(p: *const CxxString) -> bool;
    #[link_name = "_ZN2wp9bluetooth7handlesERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"] fn bluetooth_handles(p: *const CxxString) -> bool;
    #[link_name = "_ZN2wp4mmio6read32Ej"] fn mmio_read32(a: u32) -> u32;
}
extern "C" {
    #[link_name = "_ZN2wp8g_memoryE"] static g_memory: *mut u8;
    fn __acrt_iob_func(i: u32) -> *mut c_void;
    fn __mingw_fprintf(f: *mut c_void, fmt: *const c_char, ...) -> c_int;
    fn fputc(c: c_int, f: *mut c_void) -> c_int;
}
struct StrGuard(*mut CxxString); impl Drop for StrGuard { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }
/// static std::string bytes(p, n) { return n ? std::string(p, n) : std::string(); }
unsafe fn bytes(s: *mut CxxString, p: *const u8, n: usize) -> *mut CxxString { if n != 0 { s_construct(s, p, n); } else { s_init(s); } s }
macro_rules! with_bytes { ($p:expr, $n:expr, |$s:ident| $body:expr) => {{ let mut slot = core::mem::MaybeUninit::<CxxString>::uninit(); let $s = bytes(slot.as_mut_ptr(), $p, $n); let _g = StrGuard($s); $body }}; }
#[inline] unsafe fn rd32(address: u32) -> u32 {
    if address >= 0xCC00_0000 { return mmio_read32(address); }
    u32::from_be((g_memory.add((address & 0x1FFF_FFFF) as usize) as *const u32).read_unaligned())
}
#[export_name = "_ZN2wp3ios4openERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEj"]
pub unsafe extern "C-unwind" fn ios_open(path: *const CxxString, mode: u32) -> i32 { wprt_ios_open((*path).p, (*path).n, mode) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_nand_open(p: *const u8, n: usize, mode: u32) -> i32 { with_bytes!(p, n, |s| nand_open(s, mode)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_bt_handles(p: *const u8, n: usize) -> bool { with_bytes!(p, n, |s| bluetooth_handles(s)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_log_request(request: u32) {
    let d = rd32(request.wrapping_add(24)); let c = rd32(request.wrapping_add(20)); let b = rd32(request.wrapping_add(16));
    let a = rd32(request.wrapping_add(12)); let fd = rd32(request.wrapping_add(8)); let cmd = rd32(request);
    __mingw_fprintf(__acrt_iob_func(2), b"IOS request cmd=%u fd=%d a=%08x b=%08x c=%08x d=%08x\0".as_ptr() as *const c_char, cmd, fd as i32, a, b, c, d);
    fputc(10, __acrt_iob_func(2));
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_message_sdk(p: *const u8, n: usize, callback: unsafe extern "C-unwind" fn(*mut c_void, *const c_char), context: *mut c_void) { with_bytes!(p, n, |s| callback(context, (*s).p as *const c_char)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_path_sdk(op: u32, p: *const u8, n: usize, q: *const u8, m: usize, callback: unsafe extern "C-unwind" fn(u32, *const c_void, *const u8, usize) -> i32) -> i32 { with_bytes!(p, n, |s| callback(op, s as *const c_void, q, m)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_mkdir_sdk(p: *const c_void) -> i32 { nand_create_directory(p as *const CxxString) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_mkfile_sdk(p: *const c_void) -> i32 { nand_create_file(p as *const CxxString) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_remove_sdk(p: *const c_void) -> i32 { nand_remove(p as *const CxxString) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_rename_sdk(p: *const c_void, q: *const u8, n: usize) -> i32 { with_bytes!(q, n, |s| nand_rename(p as *const CxxString, s)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_ios_exists_sdk(p: *const c_void) -> bool { nand_exists(p as *const CxxString) }
