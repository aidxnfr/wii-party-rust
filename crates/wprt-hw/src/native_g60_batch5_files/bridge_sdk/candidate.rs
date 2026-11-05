// AI-assisted (Claude, G60) bridge_sdk.cpp -> Rust: whole member, no C++ residue.
// loaded_module_names: sret std::vector<std::string>, emplace_back(name,len) with exact libstdc++ _M_realloc_append growth/relocation;
// run_with_stack: const std::function<void()>& invoked from the Rust stack trampoline.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("bridge_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
extern "C-unwind" {
    fn wprt_loaded_module_names(cb: unsafe extern "C-unwind" fn(*mut u8, *const u8, usize), ctx: *mut u8);
    fn wprt_run_with_stack(size: usize, cb: unsafe extern "C-unwind" fn(*mut u8), ctx: *mut u8);
}
const ESZ: usize = 32; const VMAX: usize = isize::MAX as usize / ESZ;
unsafe fn destroy_all(v: *mut RawVec) {
    let mut e = (*v).start; while e != (*v).finish { str_destroy(e.cast()); e = e.add(ESZ); }
    vec_dealloc(&*v);
}
struct NamesGuard(*mut RawVec, bool);
impl Drop for NamesGuard { fn drop(&mut self) { if self.1 { unsafe { destroy_all(self.0) } } } }
struct NewStorage(*mut u8, usize, bool);
impl Drop for NewStorage { fn drop(&mut self) { if self.2 { unsafe { op_delete(self.0, self.1 * ESZ) } } } }
unsafe extern "C-unwind" fn emplace(ctx: *mut u8, name: *const u8, len: usize) {
    let v = ctx.cast::<RawVec>();
    if (*v).finish != (*v).eos { str_construct((*v).finish.cast(), name, len); (*v).finish = (*v).finish.add(ESZ); return; }
    let size = ((*v).finish as usize - (*v).start as usize) / ESZ;
    if VMAX - size < 1 { throw_length_error(b"vector::_M_realloc_append\0".as_ptr()) }
    let mut n = size + if size > 1 { size } else { 1 }; if n < size || n > VMAX { n = VMAX; }
    let ns = op_new(n * ESZ);
    let mut g = NewStorage(ns, n, true);
    str_construct(ns.add(size * ESZ).cast(), name, len);
    g.2 = false;
    let mut src = (*v).start; let mut dst = ns;
    while src != (*v).finish { str_move_construct(dst.cast(), src.cast()); str_destroy(src.cast()); src = src.add(ESZ); dst = dst.add(ESZ); }
    vec_dealloc(&*v);
    (*v).start = ns; (*v).finish = dst.add(ESZ); (*v).eos = ns.add(n * ESZ);
}
/// std::vector<std::string> wp::loaded_module_names()
#[export_name = "_ZN2wp19loaded_module_namesB5cxx11Ev"]
pub unsafe extern "C-unwind" fn loaded_module_names(ret: *mut RawVec) -> *mut RawVec {
    ret.write(RawVec { start: std::ptr::null_mut(), finish: std::ptr::null_mut(), eos: std::ptr::null_mut() });
    let mut g = NamesGuard(ret, true);
    wprt_loaded_module_names(emplace, ret.cast());
    g.1 = false;
    ret
}
unsafe extern "C-unwind" fn call_work(ctx: *mut u8) {
    let invoke: unsafe extern "C-unwind" fn(*const u8) = std::mem::transmute(function_check(ctx));
    invoke(ctx);
}
/// void wp::run_with_stack(size_t, const std::function<void()>&)
#[export_name = "_ZN2wp14run_with_stackEyRKSt8functionIFvvEE"]
pub unsafe extern "C-unwind" fn run_with_stack(size: usize, work: *const u8) { wprt_run_with_stack(size, call_work, work as *mut u8) }
