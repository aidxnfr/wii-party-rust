// SPDX-License-Identifier: GPL-3.0-or-later
// Claude G60: comparison_frame.cpp.obj = hybrid/review/oracle-all-src/comparison_frame_parallel.cpp (cmp-fast, -std=c++17 -O3)
// -> whole Rust, no residue. Same FNV-1a over the same bytes, same CRT formatted line (__mingw_fprintf, same format), same
// flush/order/drain/dump/limit/exit behaviour, same function-local-static retry-on-throw semantics; worker/writer threads,
// mutex and condition variables are Rust std (no C++ object crosses). Buffers use C++ operator new so failures still throw bad_alloc.
#![allow(dead_code, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_ulong, c_ulonglong, c_void};
use core::ptr::null_mut;
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};
include!("../common/cmp.rs");

type File = c_void;
extern "C" {
    #[link_name = "_ZN2wp8g_memoryE"] static g_memory: *mut u8;
    fn getenv(n: *const c_char) -> *mut c_char;
    fn fopen(p: *const c_char, m: *const c_char) -> *mut File;
    fn fflush(f: *mut File) -> c_int;
    fn fwrite(p: *const c_void, s: usize, n: usize, f: *mut File) -> usize;
    fn fclose(f: *mut File) -> c_int;
    fn strtoul(s: *const c_char, e: *mut *mut c_char, b: c_int) -> c_ulong;
    fn strtoull(s: *const c_char, e: *mut *mut c_char, b: c_int) -> c_ulonglong;
    fn _exit(c: c_int) -> !;
    fn atexit(f: extern "C" fn()) -> c_int;
    fn __acrt_iob_func(i: u32) -> *mut File;
    fn __mingw_fprintf(f: *mut File, fmt: *const c_char, ...) -> c_int;
    fn __mingw_snprintf(b: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
}
extern "C-unwind" {
    #[link_name = "_Znwy"] fn cxx_new(n: usize) -> *mut u8;
    #[link_name = "_ZdlPvy"] fn cxx_delete(p: *mut u8, n: usize);
    #[link_name = "_ZSt17__throw_bad_allocv"] fn cxx_throw_bad_alloc() -> !;
    #[link_name = "_ZSt20__throw_length_errorPKc"] fn cxx_throw_length_error(m: *const c_char) -> !;
    #[link_name = "_ZSt20__throw_system_errori"] fn cxx_throw_system_error(e: c_int) -> !;
}
macro_rules! cs { ($s:expr) => { concat!($s, "\0").as_ptr() as *const c_char }; }

const K_CPU_BYTES: usize = 4828; // offsetof(Cpu,spr)+sizeof(Cpu::spr)
const K_MEM2_BEGIN: u32 = 0x9000_0000;
const K_MEM1_BYTES: usize = 0x180_0000;
const CPU_LR: usize = 648;
const CPU_MSR: usize = 664;

#[inline] unsafe fn host(a: u32) -> *mut u8 {
    if (a & 0xF000_0000) == 0xE000_0000 { g_memory.add(0x1400_0000 + (a & 0x3FFF) as usize) } else { g_memory.add((a & 0x1FFF_FFFF) as usize) }
}
#[inline] unsafe fn rd32(a: u32) -> u32 { u32::from_be(core::ptr::read_unaligned(g_memory.add((a & 0x1FFF_FFFF) as usize) as *const u32)) }
fn hash(p: *const u8, n: usize) -> u64 {
    let mut v: u64 = 0xcbf2_9ce4_8422_2325;
    for i in 0..n { v = (v ^ unsafe { *p.add(i) } as u64).wrapping_mul(0x0000_0100_0000_01b3); }
    v
}

/// std::vector<uint8_t> storage model (operator new/sized delete, _M_default_append growth, value-initialised tail).
struct Bytes { b: *mut u8, n: usize, c: usize }
impl Bytes {
    const fn new() -> Self { Bytes { b: null_mut(), n: 0, c: 0 } }
    unsafe fn resize(&mut self, n: usize) {
        if n > self.n {
            let add = n - self.n;
            if self.c - self.n >= add { core::ptr::write_bytes(self.b.add(self.n), 0, add); self.n = n; return; }
            const MAX: usize = isize::MAX as usize;
            if MAX - self.n < add { cxx_throw_length_error(cs!("vector::_M_default_append")); }
            let mut len = self.n + core::cmp::max(self.n, add);
            if len > MAX { len = MAX; }
            let q = cxx_new(len);
            core::ptr::write_bytes(q.add(self.n), 0, add);
            if self.n != 0 { core::ptr::copy_nonoverlapping(self.b, q, self.n); }
            if !self.b.is_null() { cxx_delete(self.b, self.c); }
            self.b = q; self.n = n; self.c = len;
        } else if n < self.n { self.n = n; }
    }
}
struct Slot { mem1: Bytes, mem2: Bytes, cpu: Bytes, mem2_end: u32, serial: u64, h1: u64, h2: u64, hc: u64 }
struct State { free_slots: VecDeque<usize>, pending: VecDeque<usize>, in_order: VecDeque<usize>, hashed: Vec<bool>, queued: u64, written: u64 }
struct Pipeline {
    output: *mut File,
    lock: Mutex<State>,
    work_ready: Condvar, slot_free: Condvar, hashed: Condvar, drained: Condvar,
    slots: *mut Slot, nslots: usize,
}
unsafe impl Sync for Pipeline {}
unsafe impl Send for Pipeline {}
fn reserve_or_throw<T>(v: &mut VecDeque<T>, n: usize) { if v.try_reserve_exact(n).is_err() { unsafe { cxx_throw_bad_alloc() } } }
impl Pipeline {
    fn g(&self) -> MutexGuard<'_, State> { match self.lock.lock() { Ok(g) => g, Err(p) => p.into_inner() } }
    unsafe fn slot(&self, i: usize) -> *mut Slot { self.slots.add(i) }
    unsafe fn construct(out: *mut File, workers: u32) -> *mut Pipeline {
        let nslots = workers.wrapping_add(2) as usize; // unsigned arithmetic as in slots(workers+2)
        let p = cxx_new(core::mem::size_of::<Pipeline>()) as *mut Pipeline;
        let slots = if nslots == 0 { null_mut() } else { cxx_new(nslots * core::mem::size_of::<Slot>()) as *mut Slot };
        for i in 0..nslots { slots.add(i).write(Slot { mem1: Bytes::new(), mem2: Bytes::new(), cpu: Bytes::new(), mem2_end: 0, serial: 0, h1: 0, h2: 0, hc: 0 }); }
        let mut st = State { free_slots: VecDeque::new(), pending: VecDeque::new(), in_order: VecDeque::new(), hashed: Vec::new(), queued: 0, written: 0 };
        reserve_or_throw(&mut st.free_slots, nslots); reserve_or_throw(&mut st.pending, nslots); reserve_or_throw(&mut st.in_order, nslots);
        if st.hashed.try_reserve_exact(nslots).is_err() { cxx_throw_bad_alloc(); }
        st.hashed.resize(nslots, false);
        for i in 0..nslots { let s = &mut *slots.add(i); s.mem1.resize(K_MEM1_BYTES); s.cpu.resize(K_CPU_BYTES); st.free_slots.push_back(i); }
        p.write(Pipeline { output: out, lock: Mutex::new(st), work_ready: Condvar::new(), slot_free: Condvar::new(), hashed: Condvar::new(), drained: Condvar::new(), slots, nslots });
        let pa = p as usize;
        for _ in 0..workers { spawn(move || unsafe { (*(pa as *const Pipeline)).worker() }); }
        spawn(move || unsafe { (*(pa as *const Pipeline)).writer() });
        p
    }
    unsafe fn worker(&self) -> ! {
        loop {
            let i = { let mut g = self.g(); while g.pending.is_empty() { g = self.work_ready.wait(g).unwrap_or_else(|e| e.into_inner()); } g.pending.pop_front().unwrap() };
            let s = self.slot(i);
            (*s).h1 = hash((*s).mem1.b, K_MEM1_BYTES);
            (*s).h2 = hash((*s).mem2.b, ((*s).mem2_end.wrapping_sub(K_MEM2_BEGIN)) as usize);
            (*s).hc = hash((*s).cpu.b, K_CPU_BYTES);
            { let mut g = self.g(); g.hashed[i] = true; }
            self.hashed.notify_all();
        }
    }
    unsafe fn writer(&self) -> ! {
        loop {
            let i = { let mut g = self.g(); while !(!g.in_order.is_empty() && g.hashed[*g.in_order.front().unwrap()]) { g = self.hashed.wait(g).unwrap_or_else(|e| e.into_inner()); } *g.in_order.front().unwrap() };
            let s = self.slot(i);
            __mingw_fprintf(self.output, cs!("%llu %016llx %08x %08x %016llx %zu %016llx\n"),
                (*s).serial as c_ulonglong, (*s).h1 as c_ulonglong, K_MEM2_BEGIN, (*s).mem2_end, (*s).h2 as c_ulonglong, K_CPU_BYTES, (*s).hc as c_ulonglong);
            fflush(self.output);
            { let mut g = self.g(); g.in_order.pop_front(); g.hashed[i] = false; g.free_slots.push_back(i); g.written += 1; }
            self.slot_free.notify_one(); self.drained.notify_all();
        }
    }
    unsafe fn push(&self, serial: u64, mem2_end: u32, cpu: *const u8) {
        let i = { let mut g = self.g(); while g.free_slots.is_empty() { g = self.slot_free.wait(g).unwrap_or_else(|e| e.into_inner()); } g.free_slots.pop_front().unwrap() };
        let s = self.slot(i);
        core::ptr::copy_nonoverlapping(host(0x8000_0000), (*s).mem1.b, K_MEM1_BYTES);
        let n2 = mem2_end.wrapping_sub(K_MEM2_BEGIN) as usize;
        (*s).mem2.resize(n2);
        if n2 != 0 { core::ptr::copy_nonoverlapping(host(K_MEM2_BEGIN), (*s).mem2.b, n2); }
        core::ptr::copy_nonoverlapping(cpu, (*s).cpu.b, K_CPU_BYTES);
        (*s).mem2_end = mem2_end; (*s).serial = serial;
        { let mut g = self.g(); g.pending.push_back(i); g.in_order.push_back(i); g.queued += 1; }
        self.work_ready.notify_one();
    }
    fn drain(&self) { let mut g = self.g(); while g.written != g.queued { g = self.drained.wait(g).unwrap_or_else(|e| e.into_inner()); } }
}
/// std::thread(f).detach(): construction failure throws std::system_error like pthread_create failure in libstdc++.
fn spawn<F: FnOnce() + Send + 'static>(f: F) {
    match std::thread::Builder::new().spawn(f) { Ok(h) => drop(h), Err(e) => unsafe { cxx_throw_system_error(e.raw_os_error().unwrap_or(11)) } }
}

static mut G_PIPELINE: *mut Pipeline = null_mut();
static mut OUTPUT_DONE: bool = false; static mut OUTPUT: *mut File = null_mut();
static mut WORKERS_DONE: bool = false; static mut WORKERS: u32 = 0;
static mut PIPE_DONE: bool = false; static mut PIPE: *mut Pipeline = null_mut();
static mut SERIAL: u64 = 0;
extern "C" fn drain_at_exit() { unsafe { (*G_PIPELINE).drain(); } }

unsafe fn frame(cpu: *const u8) {
    if !cmp_enabled() { return; }
    if !OUTPUT_DONE { let p = getenv(cs!("WP_COMPARE_LOG")); OUTPUT = if !p.is_null() { fopen(getenv(cs!("WP_COMPARE_LOG")), cs!("wb")) } else { null_mut() }; OUTPUT_DONE = true; }
    if OUTPUT.is_null() { return; }
    if !WORKERS_DONE { let t = getenv(cs!("WP_COMPARE_THREADS")); WORKERS = if !t.is_null() { strtoul(t, null_mut(), 10) as u32 } else { 4 }; WORKERS_DONE = true; }
    if !PIPE_DONE {
        // initialiser may throw (bad_alloc/system_error): flag stays clear so the next call retries, like an aborted guard.
        PIPE = if WORKERS == 0 { null_mut() } else { let p = Pipeline::construct(OUTPUT, WORKERS); G_PIPELINE = p; atexit(drain_at_exit); p };
        PIPE_DONE = true;
    }
    let pipeline = PIPE;
    let mem2_begin = K_MEM2_BEGIN;
    let mem2_end = rd32(0x8000_3120);
    if mem2_end < mem2_begin || mem2_end > 0x9400_0000 {
        if !pipeline.is_null() { (*pipeline).drain(); }
        let m = "invalid MEM2 comparison bound\n"; fwrite(m.as_ptr() as _, 1, m.len(), __acrt_iob_func(2)); _exit(4);
    }
    SERIAL = SERIAL.wrapping_add(1);
    let serial = SERIAL;
    if !pipeline.is_null() { (*pipeline).push(serial, mem2_end, cpu); }
    else {
        __mingw_fprintf(OUTPUT, cs!("%llu %016llx %08x %08x %016llx %zu %016llx\n"), serial as c_ulonglong, hash(host(0x8000_0000), 0x180_0000) as c_ulonglong,
            mem2_begin, mem2_end, hash(host(mem2_begin), (mem2_end - mem2_begin) as usize) as c_ulonglong, K_CPU_BYTES, hash(cpu, K_CPU_BYTES) as c_ulonglong);
        fflush(OUTPUT);
    }
    let dump_frame = getenv(cs!("WP_COMPARE_DUMP_FRAME"));
    let prefix = getenv(cs!("WP_COMPARE_DUMP_PREFIX"));
    if !dump_frame.is_null() && !prefix.is_null() && serial == strtoull(dump_frame, null_mut(), 10) {
        if !pipeline.is_null() { (*pipeline).drain(); }
        let save = |suffix: *const c_char, data: *const u8, length: usize| {
            let mut path = [0 as c_char; 1024];
            __mingw_snprintf(path.as_mut_ptr(), 1024, cs!("%s%s"), prefix, suffix);
            let f = fopen(path.as_ptr(), cs!("wb"));
            if !f.is_null() { fwrite(data as _, 1, length, f); fclose(f); }
        };
        save(cs!(".mem1"), host(0x8000_0000), 0x180_0000);
        save(cs!(".cpu"), cpu, K_CPU_BYTES);
        __mingw_fprintf(__acrt_iob_func(2), cs!("comparison snapshot frame %llu progress %llu lr %08x msr %08x\n"), serial as c_ulonglong, cmp_ticks() as c_ulonglong,
            core::ptr::read_unaligned(cpu.add(CPU_LR) as *const u32), core::ptr::read_unaligned(cpu.add(CPU_MSR) as *const u32));
    }
    let limit = getenv(cs!("WP_COMPARE_MAX_FRAMES"));
    if !limit.is_null() && serial >= strtoull(limit, null_mut(), 10) { if !pipeline.is_null() { (*pipeline).drain(); } _exit(0); }
}
#[export_name = "_ZN2wp10comparison5frameERKNS_3CpuE"] pub unsafe extern "C-unwind" fn comparison_frame(cpu: *const c_void) { frame(cpu as *const u8) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_comparison_frame(cpu: *const c_void) { frame(cpu as *const u8) }
