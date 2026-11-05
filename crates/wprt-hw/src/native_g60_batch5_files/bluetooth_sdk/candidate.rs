// Claude G60: whole-member Rust port of hybrid bluetooth_sdk.cpp. std::function<void(const std::vector<uint8_t>&)>
// objects for the three [i] sender lambdas are built in Rust with libstdc++'s _Function_handler semantics. No residue.
#![allow(non_upper_case_globals, clippy::missing_safety_doc)]
use core::ffi::c_void;
include!("../common/cxx.rs");
extern "C-unwind" {
    fn wprt_bt_handles(p: *const u8, n: usize) -> bool;
    fn wprt_bt_send_interrupt(i: u32, p: *const u8, n: usize);
    #[link_name = "_ZN2wp7wiimote13output_reportEjPKhjRKSt8functionIFvRKSt6vectorIhSaIhEEEE"] fn wiimote_output_report(i: u32, p: *const u8, n: u32, f: *const CxxFunction);
    #[link_name = "_ZN2wp7wiimote6updateEjRKSt8functionIFvRKSt6vectorIhSaIhEEEE"] fn wiimote_update(i: u32, f: *const CxxFunction);
    #[link_name = "_ZN2wp12real_wiimote6updateEjRKSt8functionIFvRKSt6vectorIhSaIhEEEE"] fn real_wiimote_update(i: u32, f: *const CxxFunction);
}
static TI_EMULATED_OUTPUT: CxxTypeInfo = CxxTypeInfo { vptr: unsafe { (&raw const CXX_CLASS_TYPE_INFO_VTABLE as *const u8).add(16) }, name: b"*Z27wphw_bt_emulated_output_sdkEUlRKSt6vectorIhSaIhEEE_\0".as_ptr() };
static TI_EMULATED_UPDATE: CxxTypeInfo = CxxTypeInfo { vptr: unsafe { (&raw const CXX_CLASS_TYPE_INFO_VTABLE as *const u8).add(16) }, name: b"*Z27wphw_bt_emulated_update_sdkEUlRKSt6vectorIhSaIhEEE_\0".as_ptr() };
static TI_REAL_UPDATE: CxxTypeInfo = CxxTypeInfo { vptr: unsafe { (&raw const CXX_CLASS_TYPE_INFO_VTABLE as *const u8).add(16) }, name: b"*Z23wphw_bt_real_update_sdkEUlRKSt6vectorIhSaIhEEE_\0".as_ptr() };
// _Function_handler<void(const vector<u8>&), [i]>::_M_manager for a locally stored, trivially copyable functor.
unsafe fn manage(dest: *mut CxxFunction, source: *const CxxFunction, op: i32, ti: &'static CxxTypeInfo) -> bool {
    match op {
        0 => { *(dest as *mut *const CxxTypeInfo) = ti; }
        1 => { *(dest as *mut *const CxxFunction) = source; }
        2 => { *(dest as *mut u32) = *(source as *const u32); }
        _ => {}
    }
    false
}
unsafe extern "C-unwind" fn manager_emulated_output(d: *mut CxxFunction, s: *const CxxFunction, op: i32) -> bool { manage(d, s, op, &TI_EMULATED_OUTPUT) }
unsafe extern "C-unwind" fn manager_emulated_update(d: *mut CxxFunction, s: *const CxxFunction, op: i32) -> bool { manage(d, s, op, &TI_EMULATED_UPDATE) }
unsafe extern "C-unwind" fn manager_real_update(d: *mut CxxFunction, s: *const CxxFunction, op: i32) -> bool { manage(d, s, op, &TI_REAL_UPDATE) }
unsafe extern "C-unwind" fn invoke(functor: *const CxxFunction, frame: *const CxxVec) {
    let i = *(functor as *const u32);
    wprt_bt_send_interrupt(i, (*frame).b, (*frame).e as usize - (*frame).b as usize);
}
struct FnGuard(*mut CxxFunction); impl Drop for FnGuard { fn drop(&mut self) { unsafe { f_destroy(self.0) } } }
unsafe fn sender(i: u32, manager: unsafe extern "C-unwind" fn(*mut CxxFunction, *const CxxFunction, i32) -> bool) -> CxxFunction {
    let mut f = CxxFunction { data: [0; 2], manager: 0, invoker: 0 };
    *(f.data.as_mut_ptr() as *mut u32) = i;
    f.invoker = invoke as usize; f.manager = manager as usize;
    f
}
#[export_name = "_ZN2wp9bluetooth7handlesERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn handles(p: *const CxxString) -> bool { wprt_bt_handles((*p).p, (*p).n) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_bt_emulated_output_sdk(i: u32, p: *const u8, n: u32) { let mut f = sender(i, manager_emulated_output); let _g = FnGuard(&mut f); wiimote_output_report(i, p, n, &f); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_bt_emulated_update_sdk(i: u32) { let mut f = sender(i, manager_emulated_update); let _g = FnGuard(&mut f); wiimote_update(i, &f); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_bt_real_update_sdk(i: u32) { let mut f = sender(i, manager_real_update); let _g = FnGuard(&mut f); real_wiimote_update(i, &f); }
#[allow(dead_code)] fn _unused(_: *mut c_void) {}
