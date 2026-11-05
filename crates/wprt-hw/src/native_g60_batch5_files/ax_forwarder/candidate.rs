// AI-assisted (Claude, G60) ax_forwarder.cpp -> Rust: whole member, no C++ residue.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("ax_forwarder export requires pinned Windows GNU x64 metadata");
#[repr(C)] pub struct WprtAxContext { pub fields: [*mut u8; 9] }
extern "C-unwind" {
    fn wphw_ax_context(interpreter: *mut u8, state: *mut u8, context: *mut WprtAxContext);
    fn wprt_ax_run(context: *const WprtAxContext, cycles: i32, idle: *mut bool) -> i32;
}
/// int wp::dsp::generated::run_ax(DSP::Interpreter::Interpreter&, DSP::SDSP&, int, bool&)
#[export_name = "_ZN2wp3dsp9generated6run_axERN3DSP11Interpreter11InterpreterERNS2_4SDSPEiRb"]
pub unsafe extern "C-unwind" fn run_ax(interpreter: *mut u8, state: *mut u8, cycles: i32, idle: *mut bool) -> i32 {
    let mut context = std::mem::MaybeUninit::<WprtAxContext>::uninit();
    wphw_ax_context(interpreter, state, context.as_mut_ptr());
    wprt_ax_run(context.as_ptr(), cycles, idle)
}
