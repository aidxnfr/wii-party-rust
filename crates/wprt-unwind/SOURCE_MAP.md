# Rust replacement for GNU C++ catch sinks

Scope: existing project SDK catch(...)/std::current_exception()/return-1 relays. The pinned compiler is rustc1.99.0(b940084d7), Windows GNU x64. The compiler intrinsic is intentionally isolated in this small crate; production must opt in using RUSTC_BOOTSTRAP=wprt_final_unwind, and must retain panic=unwind. No nightly installation or system setting changes.

Primary documentation: https://doc.rust-lang.org/core/intrinsics/fn.catch_unwind.html explicitly documents both Rust and foreign unwinds and the runtime-specific pointer. This implementation never reads that exception object as Rust data; it passes the pointer to the corresponding __cxa_begin_catch and uses native noexcept current_exception/end-catch operations, exactly as the original generated GNU catch blocks.

Compiler-generated LLVM and the existing video SDK differential (including its already-existing throw scenarios) are required before integration. This does not remove the third-party GNU C++ runtime dependency, nor establish portability to other compiler/runtime pairs. No std::panic::catch_unwind or abort replacement is used.
