// Claude G60: wp/comparison.h inline clock (enabled()/advance()/ticks()) over the SAME C++ COMDAT objects
// (guard _ZGVZN2wp10comparison7enabledEvE5value, value _ZZN2wp10comparison7enabledEvE5value, progress _ZN2wp10comparison8progressE),
// which remaining C++ TUs (dsp_host/input lifetime residues, harness) define. Same Itanium guard protocol as GCC's inlined code.
extern "C" {
    #[link_name = "_ZGVZN2wp10comparison7enabledEvE5value"] static mut CMP_GUARD: u64;
    #[link_name = "_ZZN2wp10comparison7enabledEvE5value"] static mut CMP_VALUE: u8;
    #[link_name = "_ZN2wp10comparison8progressE"] static CMP_PROGRESS: core::sync::atomic::AtomicU64;
    #[link_name = "getenv"] fn cmp_getenv(n: *const core::ffi::c_char) -> *mut core::ffi::c_char;
}
extern "C-unwind" {
    #[link_name = "__cxa_guard_acquire"] fn cmp_guard_acquire(g: *mut u64) -> core::ffi::c_int;
    #[link_name = "__cxa_guard_release"] fn cmp_guard_release(g: *mut u64);
}
#[inline(always)] unsafe fn cmp_enabled() -> bool {
    let g = &*(core::ptr::addr_of_mut!(CMP_GUARD) as *const core::sync::atomic::AtomicU8);
    if g.load(core::sync::atomic::Ordering::Acquire) == 0 && cmp_guard_acquire(core::ptr::addr_of_mut!(CMP_GUARD)) != 0 {
        let v = !cmp_getenv(concat!("WP_DETERMINISTIC", "\0").as_ptr() as _).is_null();
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CMP_VALUE), v as u8);
        cmp_guard_release(core::ptr::addr_of_mut!(CMP_GUARD));
    }
    core::ptr::read_volatile(core::ptr::addr_of!(CMP_VALUE)) != 0
}
#[inline(always)] unsafe fn cmp_advance() { if cmp_enabled() { CMP_PROGRESS.fetch_add(1024, core::sync::atomic::Ordering::Relaxed); } }
#[inline(always)] unsafe fn cmp_ticks() -> u64 { CMP_PROGRESS.load(core::sync::atomic::Ordering::Relaxed) }
