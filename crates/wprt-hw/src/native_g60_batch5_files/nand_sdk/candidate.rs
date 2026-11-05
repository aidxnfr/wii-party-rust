// AI-assisted (Claude, G60) nand_sdk.cpp -> Rust for every non-filesystem member; residue.cpp keeps the std::filesystem leaves.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("nand_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
type S = *const CxxString;
extern "C-unwind" {
    fn wprt_nand_mount(p: *const u8, n: usize) -> bool;
    fn wprt_nand_host_directory(p: *const u8, n: usize, sink: unsafe extern "C-unwind" fn(*mut u8, *const u8, usize), c: *mut u8);
    fn wprt_nand_exists(p: *const u8, n: usize) -> bool;
    fn wprt_nand_open(p: *const u8, n: usize, mode: u32) -> i32;
    fn wprt_nand_create_file(p: *const u8, n: usize) -> i32;
    fn wprt_nand_create_directory(p: *const u8, n: usize) -> i32;
    fn wprt_nand_remove(p: *const u8, n: usize) -> i32;
    fn wprt_nand_rename(a: *const u8, an: usize, b: *const u8, bn: usize) -> i32;
}
#[export_name = "_ZN2wp4nand5mountERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn mount(r: S) -> bool { wprt_nand_mount((*r).p, (*r).len) }
/// std::string wp::nand::host_directory(const std::string&)  (sret)
#[export_name = "_ZN2wp4nand14host_directoryERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn host_directory(ret: *mut CxxString, p: S) -> *mut CxxString {
    str_init(ret);
    let mut g = StrRet(ret, true);
    wprt_nand_host_directory((*p).p, (*p).len, assign_sink, ret.cast());
    g.1 = false;
    ret
}
#[export_name = "_ZN2wp4nand6existsERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn exists(p: S) -> bool { wprt_nand_exists((*p).p, (*p).len) }
#[export_name = "_ZN2wp4nand4openERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEj"]
pub unsafe extern "C-unwind" fn open(p: S, mode: u32) -> i32 { wprt_nand_open((*p).p, (*p).len, mode) }
#[export_name = "_ZN2wp4nand11create_fileERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn create_file(p: S) -> i32 { wprt_nand_create_file((*p).p, (*p).len) }
#[export_name = "_ZN2wp4nand16create_directoryERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn create_directory(p: S) -> i32 { wprt_nand_create_directory((*p).p, (*p).len) }
#[export_name = "_ZN2wp4nand6removeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn remove(p: S) -> i32 { wprt_nand_remove((*p).p, (*p).len) }
#[export_name = "_ZN2wp4nand6renameERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES8_"]
pub unsafe extern "C-unwind" fn rename(a: S, b: S) -> i32 { wprt_nand_rename((*a).p, (*a).len, (*b).p, (*b).len) }
#[export_name = "wphw_nand_message_sdk"]
pub unsafe extern "C-unwind" fn wphw_nand_message_sdk(p: *const u8, n: usize, callback: unsafe extern "C-unwind" fn(*mut u8, *const u8), c: *mut u8) {
    let mut slot = std::mem::MaybeUninit::<CxxString>::uninit(); let s = slot.as_mut_ptr();
    str_construct(s, p, n);
    let _g = StrDrop(s);
    callback(c, (*s).p);
}
