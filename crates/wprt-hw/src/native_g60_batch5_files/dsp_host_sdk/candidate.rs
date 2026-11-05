// Claude G60: whole-member Rust port of hybrid dsp_host_sdk.cpp (wp::dsp host SDK glue).
// Layout constants are compiler-derived (probe.cpp -> layout.rs). C++ lifetime residue: dsp_host_lifetime.cpp.
#![allow(non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
include!("layout.rs");

#[repr(C)] pub struct RomBlob { data: *const u8, size: usize }
#[repr(C)] struct CxxVec { b: *mut u8, e: *mut u8, c: *mut u8 }
impl CxxVec { const fn new() -> Self { CxxVec { b: null_mut(), e: null_mut(), c: null_mut() } } fn len(&self) -> usize { self.e as usize - self.b as usize } }
#[repr(C)] struct PointerWrap { buf: *mut CxxVec, mode: i32, pos: usize }
#[repr(C)] #[derive(Clone, Copy)] struct MemoryWrite { address: u32, old_value: u8, new_value: u8 }
type Translated = unsafe extern "C-unwind" fn(*mut u8, *mut u8, c_int, *mut bool) -> c_int;
type FILE = c_void;

extern "C-unwind" {
    #[link_name = "_ZN3DSP11Interpreter11Interpreter4StepEv"] fn interp_step(this: *mut u8);
    #[link_name = "_ZN3DSP11Interpreter11Interpreter19ReadControlRegisterEv"] fn interp_read_control(this: *mut u8) -> u16;
    #[link_name = "_ZN3DSP11Interpreter11Interpreter20WriteControlRegisterEt"] fn interp_write_control(this: *mut u8, v: u16);
    #[link_name = "_ZN3DSP20InitInstructionTableEv"] fn init_instruction_table();
    #[link_name = "_ZN3DSP4SDSP7DoStateER11PointerWrap"] fn sdsp_do_state(this: *mut u8, p: *mut PointerWrap);
    #[link_name = "_ZN3DSP7DSPCore10InitializeERKNS_14DSPInitOptionsE"] fn core_initialize(this: *mut u8, o: *const u8) -> bool;
    #[link_name = "_ZN3DSP7DSPCore14ReadMailboxLowENS_7MailboxE"] fn core_read_low(this: *mut u8, b: c_int) -> u16;
    #[link_name = "_ZN3DSP7DSPCore15ReadMailboxHighENS_7MailboxE"] fn core_read_high(this: *mut u8, b: c_int) -> u16;
    #[link_name = "_ZN3DSP7DSPCore15WriteMailboxLowENS_7MailboxEt"] fn core_write_low(this: *mut u8, b: c_int, v: u16);
    #[link_name = "_ZN3DSP7DSPCore16WriteMailboxHighENS_7MailboxEt"] fn core_write_high(this: *mut u8, b: c_int, v: u16);
    #[link_name = "_ZN3DSP7DSPCore15CheckExceptionsEv"] fn core_check_exceptions(this: *mut u8) -> bool;
    #[link_name = "_ZN3DSP7DSPCore22CheckExternalInterruptEv"] fn core_check_external(this: *mut u8);
    #[link_name = "_ZN3DSP7DSPCore5ResetEv"] fn core_reset(this: *mut u8);
    #[link_name = "_ZN3DSP7DSPCore9ClearIRAMEv"] fn core_clear_iram(this: *mut u8);
    #[link_name = "_ZN3DSP7DSPCore9RunCyclesEi"] fn core_run_cycles(this: *mut u8, c: c_int) -> c_int;
    #[link_name = "_ZNK3DSP7DSPCore11PeekMailboxENS_7MailboxE"] fn core_peek(this: *mut u8, b: c_int) -> u32;
    #[link_name = "_ZN3DSP8Analyzer7AnalyzeERKNS_4SDSPE"] fn analyzer_analyze(this: *mut u8, s: *const u8);
    #[link_name = "_ZNSt6chrono3_V212steady_clock3nowEv"] fn steady_now() -> i64;
    #[link_name = "_ZN2wp3dsp15instruction_romEv"] fn instruction_rom() -> RomBlob;
    #[link_name = "_ZN2wp3dsp15coefficient_romEv"] fn coefficient_rom() -> RomBlob;
    #[link_name = "_Znwy"] fn op_new(n: usize) -> *mut u8;
    #[link_name = "_ZdlPvy"] fn op_delete(p: *mut u8, n: usize);
    #[link_name = "_ZSt20__throw_length_errorPKc"] fn throw_length_error(m: *const c_char) -> !;
    #[link_name = "_ZSt19__throw_logic_errorPKc"] fn throw_logic_error(m: *const c_char) -> !;
    fn __cxa_guard_acquire(g: *mut i64) -> c_int;
    fn __cxa_guard_release(g: *mut i64);
    fn wphw_dsp_rom_words(d: *const u8, n: usize, w: *mut u16, c: usize) -> bool;
    fn wphw_dsp_osd_message(m: *const c_char);
    fn wphw_dsp_bootstrap_flow(ctx: *mut c_void, op: unsafe extern "C-unwind" fn(*mut c_void, u32) -> bool) -> bool;
    fn wprt_dsp_control_ptr() -> *mut u16;
    fn wprt_dsp_initial(a: i64, b: i64, c: i64, d: i64);
    fn wprt_claude_dsp_init_options(storage: *mut u8);
    fn wprt_claude_dsp_new_core() -> *mut u8;
    fn wprt_claude_dsp_run_with_stack(n: usize, work: unsafe extern "C-unwind" fn());
}
extern "C" {
    fn getenv(n: *const c_char) -> *const c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn fopen(p: *const c_char, m: *const c_char) -> *mut FILE;
    fn fwrite(p: *const c_void, s: usize, n: usize, f: *mut FILE) -> usize;
    fn fclose(f: *mut FILE) -> c_int;
    fn fflush(f: *mut FILE) -> c_int;
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn abort() -> !;
    fn _exit(c: c_int) -> !;
    fn __acrt_iob_func(i: u32) -> *mut FILE;
    fn __mingw_fprintf(f: *mut FILE, fmt: *const c_char, ...) -> c_int;
    #[link_name = "_ZN2wp3log5writeEPKcS2_z"] fn log_write(cat: *const c_char, fmt: *const c_char, ...);
    #[link_name = "_ZN2wp8g_memoryE"] static mut g_memory: *mut u8;
    #[link_name = "_ZN2wp3dsp17g_translated_codeE"] static g_translated_code: [u8; 0];
    #[link_name = "_ZN2wp3dsp23g_translated_code_countE"] static g_translated_code_count: usize;
    #[link_name = "_ZZN2wp10comparison7enabledEvE5value"] static mut cmp_enabled_value: bool;
    #[link_name = "_ZGVZN2wp10comparison7enabledEvE5value"] static mut cmp_enabled_guard: i64;
    #[link_name = "_ZN2wp10comparison8progressE"] static cmp_progress: AtomicU64;
}

const K_IRAM_WORDS: u16 = 0x1000;
static mut G_CORE: *mut u8 = null_mut();
static mut G_CONTROL: *mut u16 = null_mut();
static G_CODE_LOADED: AtomicBool = AtomicBool::new(false);
static G_NATIVE: AtomicBool = AtomicBool::new(false);
static G_LOAD_PERCENT: AtomicU64 = AtomicU64::new(0);
static mut G_LOG: bool = false;
static mut G_FORCE_INTERPRETER: bool = false;
static mut G_VERIFY: bool = false;
static mut G_WRITE_LOG: *mut CxxVec = null_mut();
static mut G_TRANSLATED: usize = 0;
static mut G_RESTORING: bool = false;
static mut G_VERIFY_CHECKS: u64 = 0;
static mut G_SELF_TESTING: bool = false;
static mut G_IRAM_WRITTEN: bool = false;
static mut G_VERIFY_SKIPPED: u64 = 0;
static mut G_VERIFY_FAILURES: u64 = 0;
static mut G_VERIFIED_PC: [u8; K_IRAM_WORDS as usize] = [0; K_IRAM_WORDS as usize];
static SELFTEST_COUNT_STATE: AtomicU8 = AtomicU8::new(0);
static mut SELFTEST_COUNT: c_int = 0;

macro_rules! c { ($s:expr) => { concat!($s, "\0").as_ptr() as *const c_char }; }
#[inline] unsafe fn rd<T: Copy>(p: *const u8, o: usize) -> T { (p.add(o) as *const T).read_unaligned() }
#[inline] unsafe fn wr<T>(p: *mut u8, o: usize, v: T) { (p.add(o) as *mut T).write_unaligned(v) }
#[inline] unsafe fn state_of(core: *mut u8) -> *mut u8 { core.add(CORE_DSP) }
#[inline] unsafe fn interp_of(core: *mut u8) -> *mut u8 { rd::<*mut u8>(core, CORE_INTERP) }
#[inline] unsafe fn flags_of(state: *mut u8) -> *const u8 { state.add(S_ANALYZER + A_FLAGS) }
#[inline] unsafe fn translated() -> Translated { core::mem::transmute::<usize, Translated>(G_TRANSLATED) }
#[inline] unsafe fn stderr() -> *mut FILE { __acrt_iob_func(2) }

unsafe fn comparison_enabled() -> bool {
    let g = addr_of_mut!(cmp_enabled_guard);
    if (*(g as *const AtomicU8)).load(Ordering::Acquire) == 0 && __cxa_guard_acquire(g) != 0 {
        cmp_enabled_value = !getenv(c!("WP_DETERMINISTIC")).is_null();
        __cxa_guard_release(g);
    }
    cmp_enabled_value
}
unsafe fn comparison_now() -> i64 {
    if !comparison_enabled() { return steady_now(); }
    let value = cmp_progress.load(Ordering::Relaxed);
    let ns = (value / 60750000).wrapping_mul(1000000000).wrapping_add((value % 60750000).wrapping_mul(1000000000) / 60750000);
    ns as i64
}

#[no_mangle] pub unsafe extern "C-unwind" fn wprt_claude_dsp_host_static_init() {
    G_CONTROL = wprt_dsp_control_ptr();
    let a = comparison_now(); let b = comparison_now(); let cc = comparison_now(); let d = comparison_now();
    wprt_dsp_initial(a, b, cc, d);
    G_LOG = !getenv(c!("WP_LOG_DSP")).is_null();
    G_FORCE_INTERPRETER = !getenv(c!("WP_DSP_INTERPRETER")).is_null();
    G_VERIFY = !getenv(c!("WP_DSP_VERIFY")).is_null();
}

#[inline] unsafe fn host(address: u32) -> *mut u8 {
    if (address & 0xF000_0000) == 0xE000_0000 { return g_memory.add(0x1400_0000 + (address & (0x4000 - 1)) as usize); }
    g_memory.add((address & 0x1FFF_FFFF) as usize)
}
unsafe fn vec_free(v: &mut CxxVec) { if !v.b.is_null() { op_delete(v.b, v.c as usize - v.b as usize); } }
unsafe fn push_write(v: *mut CxxVec, w: MemoryWrite) {
    let v = &mut *v;
    if v.e != v.c { (v.e as *mut MemoryWrite).write(w); v.e = v.e.add(8); return; }
    let n = v.len() / 8; let max = (isize::MAX as usize) / 8;
    if max - n < 1 { throw_length_error(c!("vector::_M_realloc_append")); }
    let mut len = n + if n > 1 { n } else { 1 };
    if len < n || len > max { len = max; }
    let new = op_new(len * 8);
    (new.add(n * 8) as *mut MemoryWrite).write(w);
    if n > 0 { core::ptr::copy(v.b, new, n * 8); }
    if !v.b.is_null() { op_delete(v.b, v.c as usize - v.b as usize); }
    v.b = new; v.e = new.add((n + 1) * 8); v.c = new.add(len * 8);
}
unsafe fn writes(v: &CxxVec) -> &[MemoryWrite] { if v.b.is_null() { &[] } else { core::slice::from_raw_parts(v.b as *const MemoryWrite, v.len() / 8) } }
unsafe fn record_write(address: u32, value: u8) { if !G_WRITE_LOG.is_null() { let old = *host(address); push_write(G_WRITE_LOG, MemoryWrite { address, old_value: old, new_value: value }); } }
unsafe fn undo_writes(log: &CxxVec) { for w in writes(log).iter().rev() { *host(w.address) = w.old_value; } }
unsafe fn redo_writes(log: &CxxVec) { for w in writes(log) { *host(w.address) = w.new_value; } }
unsafe fn same_writes(a: &CxxVec, b: &CxxVec) -> bool {
    let (a, b) = (writes(a), writes(b));
    if a.len() != b.len() { return false; }
    for i in 0..a.len() { if a[i].address != b[i].address || a[i].new_value != b[i].new_value { return false; } }
    true
}
unsafe fn save_state() -> CxxVec {
    let mut buffer = CxxVec::new();
    let mut writer = PointerWrap { buf: &mut buffer, mode: PW_WRITE, pos: 0 };
    sdsp_do_state(state_of(G_CORE), &mut writer);
    buffer
}
unsafe fn load_state(buffer: &mut CxxVec) {
    let t = G_TRANSLATED;
    let mut reader = PointerWrap { buf: buffer, mode: PW_READ, pos: 0 };
    G_RESTORING = true;
    sdsp_do_state(state_of(G_CORE), &mut reader);
    G_RESTORING = false;
    G_TRANSLATED = t;
}
unsafe fn report_mismatch(start_pc: u16, instructions: c_int, t: &CxxVec, i: &CxxVec) {
    let skip = STEP_SKIP; let mut first = 0usize; let n = t.len();
    while first < n && (*t.b.add(first) == *i.b.add(first) || (first >= skip && first < skip + 8)) { first += 1; }
    let (a, b) = (t.b as *const u8, i.b as *const u8);
    let pc_a: u16 = rd(a, REGS_SIZE); let pc_b: u16 = rd(b, REGS_SIZE);
    __mingw_fprintf(stderr(), c!("DSP verify: mismatch after %d instructions from pc %04x, first differing byte %zu of %zu\n"), instructions, start_pc as c_int, first, n);
    for (fmt, r, pc) in [(c!("  translated  pc %04x sr %04x ac0 %016llx ac1 %016llx ax0 %08x ax1 %08x prod %016llx ar %04x %04x %04x %04x\n"), a, pc_a), (c!("  interpreter pc %04x sr %04x ac0 %016llx ac1 %016llx ax0 %08x ax1 %08x prod %016llx ar %04x %04x %04x %04x\n"), b, pc_b)] {
        __mingw_fprintf(stderr(), fmt, pc as c_int, rd::<u16>(r, R_SR) as c_int, rd::<u64>(r, R_AC), rd::<u64>(r, R_AC + AC_STRIDE), rd::<u32>(r, R_AX), rd::<u32>(r, R_AX + AX_STRIDE), rd::<u64>(r, R_PROD),
            rd::<u16>(r, R_AR) as c_int, rd::<u16>(r, R_AR + 2) as c_int, rd::<u16>(r, R_AR + 4) as c_int, rd::<u16>(r, R_AR + 6) as c_int);
    }
}
unsafe fn same_state(a: &CxxVec, b: &CxxVec) -> bool {
    if a.len() != b.len() { return false; }
    let skip = STEP_SKIP;
    memcmp(a.b as _, b.b as _, skip) == 0 && memcmp(a.b.add(skip + 8) as _, b.b.add(skip + 8) as _, a.len().wrapping_sub(skip).wrapping_sub(8)) == 0
}
unsafe fn coverage_percent(covered: &mut c_int, total: &mut c_int) -> c_int {
    *covered = 0; *total = 0;
    let f = flags_of(state_of(G_CORE));
    for address in 0..K_IRAM_WORDS { if *f.add(address as usize) & 1 != 0 { *total += 1; *covered += (G_VERIFIED_PC[address as usize] != 0) as c_int; } }
    if *total != 0 { *covered * 100 / *total } else { 0 }
}
unsafe fn run_translated_verified(state: *mut u8, left: c_int, idle: *mut bool) -> c_int {
    let mut before = save_state();
    let start_pc: u16 = rd(state, S_PC);
    let mut tw = CxxVec::new(); let mut iw = CxxVec::new();
    G_WRITE_LOG = &mut tw; G_IRAM_WRITTEN = false;
    let remaining = translated()(interp_of(G_CORE), state, left, idle);
    G_WRITE_LOG = null_mut();
    let executed = left.wrapping_sub(remaining);
    let mut tr = save_state();
    undo_writes(&tw); load_state(&mut before);
    G_WRITE_LOG = &mut iw;
    let mut i = 0; while i < executed { let pc: u16 = rd(state, S_PC); if pc < K_IRAM_WORDS { G_VERIFIED_PC[pc as usize] = 1; } interp_step(interp_of(G_CORE)); i += 1; }
    G_WRITE_LOG = null_mut();
    let mut it = save_state();
    undo_writes(&iw); redo_writes(&tw);
    G_VERIFY_CHECKS = G_VERIFY_CHECKS.wrapping_add(1);
    if G_IRAM_WRITTEN {
        G_VERIFY_SKIPPED = G_VERIFY_SKIPPED.wrapping_add(1);
        if !G_SELF_TESTING { __mingw_fprintf(stderr(), c!("DSP verify: the microcode overwrote its own instruction memory at pc %04x; not comparable\n"), start_pc as c_int); }
    } else if !same_writes(&tw, &iw) {
        G_VERIFY_FAILURES = G_VERIFY_FAILURES.wrapping_add(1);
        if G_VERIFY_FAILURES <= 20 { __mingw_fprintf(stderr(), c!("DSP verify: memory writes differ after %d instructions from pc %04x (%zu vs %zu bytes)\n"), executed, start_pc as c_int, tw.len() / 8, iw.len() / 8); }
    } else if !same_state(&tr, &it) {
        G_VERIFY_FAILURES = G_VERIFY_FAILURES.wrapping_add(1);
        if G_VERIFY_FAILURES <= 20 {
            report_mismatch(start_pc, executed, &tr, &it);
            undo_writes(&tw);
            let mut k: c_int = 1;
            while k <= left {
                let mut log_t = CxxVec::new(); let mut log_i = CxxVec::new();
                load_state(&mut before);
                let mut unused = false;
                G_WRITE_LOG = &mut log_t;
                let left_k = translated()(interp_of(G_CORE), state, k, &mut unused);
                G_WRITE_LOG = null_mut();
                let pc_t: u16 = rd(state, S_PC);
                let mut t_k = save_state();
                undo_writes(&log_t); load_state(&mut before);
                G_WRITE_LOG = &mut log_i;
                let mut pc_last: u16 = rd(state, S_PC);
                let n = k.wrapping_sub(left_k);
                let mut j = 0; while j < n { pc_last = rd(state, S_PC); interp_step(interp_of(G_CORE)); j += 1; }
                G_WRITE_LOG = null_mut();
                let mut i_k = save_state();
                undo_writes(&log_i);
                let diverged = !same_state(&t_k, &i_k);
                if diverged {
                    __mingw_fprintf(stderr(), c!("  first divergence with budget %d: counted %d, translated stopped at %04x, interpreter last executed %04x and is at %04x, exceptions %02x\n"),
                        k, n, pc_t as c_int, pc_last as c_int, rd::<u16>(state, S_PC) as c_int, rd::<u8>(state, S_EXC) as c_int);
                }
                vec_free(&mut i_k); vec_free(&mut t_k); vec_free(&mut log_i); vec_free(&mut log_t);
                if diverged { break; }
                k += 1;
            }
            redo_writes(&tw);
        }
    }
    if G_VERIFY_CHECKS % 20000 == 0 {
        let (mut covered, mut total) = (0, 0);
        let percent = coverage_percent(&mut covered, &mut total);
        __mingw_fprintf(stderr(), c!("DSP verify: %llu runs checked, %llu mismatches, %d of %d instructions covered (%d%%)\n"), G_VERIFY_CHECKS, G_VERIFY_FAILURES, covered, total, percent);
    }
    load_state(&mut tr);
    vec_free(&mut it); vec_free(&mut tr); vec_free(&mut iw); vec_free(&mut tw); vec_free(&mut before);
    remaining
}

struct Mt { mt: [u32; 624], i: usize }
impl Mt {
    fn new(seed: u32) -> Mt { let mut mt = [0u32; 624]; mt[0] = seed; for i in 1..624 { mt[i] = 1812433253u32.wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_add(i as u32); } Mt { mt, i: 624 } }
    fn gen(&mut self) {
        for k in 0..624 { let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % 624] & 0x7fff_ffff); self.mt[k] = self.mt[(k + 397) % 624] ^ (y >> 1) ^ if y & 1 != 0 { 0x9908_b0df } else { 0 }; }
        self.i = 0;
    }
    fn next(&mut self) -> u32 { if self.i >= 624 { self.gen(); } let mut y = self.mt[self.i]; self.i += 1; y ^= y >> 11; y ^= (y << 7) & 0x9d2c_5680; y ^= (y << 15) & 0xefc6_0000; y ^ (y >> 18) }
}
unsafe fn randomize_state(state: *mut u8, random: &mut Mt, pc: u16) {
    for i in 0..4 {
        let v = if random.next() % 8 == 0 { 0x1000 | (random.next() % 0x800) } else { random.next() % 0x1000 }; wr(state, R_AR + 2 * i, v as u16);
        let v = if random.next() % 4 == 0 { random.next() % 0x10000 } else { (random.next() % 9).wrapping_sub(4) }; wr(state, R_IX + 2 * i, v as u16);
        let v = if random.next() % 2 == 0 { 0xFFFF } else { random.next() % 0x10000 }; wr(state, R_WR + 2 * i, v as u16);
    }
    for i in 0..2 {
        let hi = random.next() as u64; let lo = random.next() as u64;
        let mut value = (hi << 32) | lo;
        if random.next() % 4 == 0 { value = if random.next() % 3 == 0 { 0 } else if random.next() % 2 == 0 { 0x7FFF0000 } else { 0xFFFFFF8000000000 }; }
        wr(state, R_AC + AC_STRIDE * i, (((value as i64) << 24) >> 24) as u64);
        wr(state, R_AX + AX_STRIDE * i, random.next());
    }
    wr(state, R_PROD + P_L, random.next() as u16);
    wr(state, R_PROD + P_M, random.next() as u16);
    wr(state, R_PROD + P_H, (random.next() & 0xFF) as u16);
    wr(state, R_PROD + P_M2, random.next() as u16);
    wr(state, R_SR, (random.next() & SR_MASK as u32) as u16);
    wr(state, R_CR, (random.next() % 0xFF) as u16);
    if random.next() % 2 == 0 { wr(state, R_ST + 4, pc.wrapping_add((random.next() % 4) as u16)); wr(state, R_ST + 6, (random.next() % 3) as u16); }
    let dram: *mut u16 = rd(state, S_DRAM);
    for i in 0..DRAM_WORDS as usize { *dram.add(i) = random.next() as u16; }
    wr(state, S_PC, pc);
    wr(state, S_EXC, 0u8);
}
unsafe fn run_self_test(trials: c_int) -> ! {
    let state = state_of(G_CORE);
    let f = flags_of(state);
    let mut original = save_state();
    let control = *G_CONTROL;
    let mut random = Mt::new(0x5eed);
    let budgets: [c_int; 4] = [1, 2, 3, 8];
    G_SELF_TESTING = true;
    let (cb, fb, sb) = (G_VERIFY_CHECKS, G_VERIFY_FAILURES, G_VERIFY_SKIPPED);
    let mut instructions: c_int = 0;
    for address in 0..K_IRAM_WORDS {
        if *f.add(address as usize) & 1 == 0 { continue; }
        instructions += 1;
        for _ in 0..trials { for budget in budgets { load_state(&mut original); randomize_state(state, &mut random, address); let mut idle = false; run_translated_verified(state, budget, &mut idle); } }
    }
    load_state(&mut original);
    *G_CONTROL = control;
    G_SELF_TESTING = false;
    let checks = G_VERIFY_CHECKS.wrapping_sub(cb); let failures = G_VERIFY_FAILURES.wrapping_sub(fb); let skipped = G_VERIFY_SKIPPED.wrapping_sub(sb);
    __mingw_fprintf(stderr(), c!("DSP self test: %d instructions, %llu random runs, %llu mismatches, %llu runs skipped because they overwrote the instruction memory\n"), instructions, checks, failures, skipped);
    log_write(c!("audio"), c!("DSP self test: %d instructions, %llu random runs, %llu mismatches, %llu skipped"), instructions, checks, failures, skipped);
    fflush(stderr());
    _exit(if failures == 0 { 0 } else { 1 });
}
unsafe extern "C-unwind" fn selftest_work() { run_self_test(SELFTEST_COUNT); }

#[export_name = "_ZN3DSP4Host14OSD_AddMessageENSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEEj"]
pub unsafe extern "C-unwind" fn osd_add_message(message: *const *const c_char, _duration: u32) { wphw_dsp_osd_message(*message); }

#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_step_sdk() { interp_step(interp_of(G_CORE)); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_run_cycles_sdk(cycles: c_int) { core_run_cycles(G_CORE, cycles); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_idle_skip_sdk() -> bool { let s = state_of(G_CORE); *flags_of(s).add(rd::<u16>(s, S_PC) as usize) & 2 != 0 }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_host_record_write_sdk(address: u32, value: u8) { record_write(address, value); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_host_analyze_sdk(core: *mut c_void) { let core = core as *mut u8; core_clear_iram(core); let s = state_of(core); analyzer_analyze(s.add(S_ANALYZER), s); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_host_existing_selftest_sdk() {
    let trials = getenv(c!("WP_DSP_SELFTEST"));
    if !trials.is_null() && G_TRANSLATED != 0 {
        if SELFTEST_COUNT_STATE.load(Ordering::Acquire) == 0 { let v = atoi(trials); SELFTEST_COUNT = if v > 1 { v } else { 1 }; SELFTEST_COUNT_STATE.store(1, Ordering::Release); }
        wprt_claude_dsp_run_with_stack(64usize << 20, selftest_work);
    }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_clock_sdk() -> i64 { comparison_now() }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_present_sdk() -> bool { !G_CORE.is_null() }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_logging_sdk() -> bool { G_LOG }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_verifying_sdk() -> bool { G_VERIFY }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_translated_present_sdk() -> bool { G_TRANSLATED != 0 }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_state_control_sdk() -> u16 { rd(state_of(G_CORE), S_CONTROL) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_pc_sdk() -> u16 { rd(state_of(G_CORE), S_PC) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_state_sdk() -> *mut c_void { state_of(G_CORE) as *mut c_void }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_translated_direct_sdk(raw: *mut c_void, left: c_int, idle: *mut bool) -> c_int { let t = translated(); t(interp_of(G_CORE), raw as *mut u8, left, idle) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_translated_verified_sdk(raw: *mut c_void, left: c_int, idle: *mut bool) -> c_int { run_translated_verified(raw as *mut u8, left, idle) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_mail_peek_sdk(b: c_int) -> u32 { core_peek(G_CORE, b) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_mail_high_sdk(b: c_int) -> u16 { core_read_high(G_CORE, b) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_mail_low_sdk(b: c_int) -> u16 { core_read_low(G_CORE, b) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_write_mail_high_sdk(v: u16) { core_write_high(G_CORE, MAILBOX_CPU, v) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_write_mail_low_sdk(v: u16) { core_write_low(G_CORE, MAILBOX_CPU, v) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_control_read_sdk() -> u16 { interp_read_control(interp_of(G_CORE)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_control_write_sdk(v: u16) { interp_write_control(interp_of(G_CORE), v) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_external_sdk() { core_check_external(G_CORE) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_exceptions_sdk() { core_check_exceptions(G_CORE); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_load_store_sdk(value: f64) { G_LOAD_PERCENT.store(value.to_bits(), Ordering::SeqCst) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_loaded_sdk() -> bool { G_CODE_LOADED.load(Ordering::SeqCst) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_native_sdk() -> bool { G_NATIVE.load(Ordering::SeqCst) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_load_sdk() -> f64 { f64::from_bits(G_LOAD_PERCENT.load(Ordering::SeqCst)) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_dram_sdk() -> *const u16 { rd(state_of(G_CORE), S_DRAM) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_iram_sdk() -> *const u16 { rd(state_of(G_CORE), S_IRAM) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_write_log_sdk() -> *mut c_void { G_WRITE_LOG as *mut c_void }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_self_testing_sdk() -> bool { G_SELF_TESTING }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_restoring_sdk() -> bool { G_RESTORING }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_iram_written_sdk() { G_IRAM_WRITTEN = true; }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_crc_sdk(raw: *mut c_void, crc: u32) { wr(state_of(raw as *mut u8), S_CRC, crc) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_clear_translated_sdk() { G_TRANSLATED = 0; }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_count_sdk() -> usize { g_translated_code_count }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_forced_sdk() -> bool { G_FORCE_INTERPRETER }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_crc_at_sdk(index: usize) -> u32 { rd(g_translated_code.as_ptr(), index * TC_SIZE) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_select_sdk(index: usize) { G_TRANSLATED = rd(g_translated_code.as_ptr(), index * TC_SIZE + TC_FN); }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_loaded_store_sdk() { G_CODE_LOADED.store(true, Ordering::SeqCst) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_native_store_sdk(v: bool) { G_NATIVE.store(v, Ordering::SeqCst) }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_dump_sdk(directory: *const c_char, pointer: *const u8, size: usize, crc: u32) {
    if directory.is_null() { throw_logic_error(c!("basic_string: construction from null is not valid")); }
    let n = strlen(directory);
    let mut path: Vec<u8> = Vec::with_capacity(n + 32);
    path.extend_from_slice(core::slice::from_raw_parts(directory as *const u8, n));
    path.extend_from_slice(b"/dsp_"); path.extend_from_slice(crc.to_string().as_bytes()); path.extend_from_slice(b".bin\0");
    let file = fopen(path.as_ptr() as *const c_char, c!("wb"));
    if !file.is_null() { fwrite(pointer as *const c_void, 1, size, file); fclose(file); }
}
#[repr(C, align(8))] struct BootstrapContext { options: [u8; OPT_SIZE], core: *mut u8 }
unsafe extern "C-unwind" fn bootstrap_operation(raw: *mut c_void, operation: u32) -> bool {
    let c = &mut *(raw as *mut BootstrapContext);
    let o = c.options.as_mut_ptr();
    match operation {
        0 => { let r = instruction_rom(); wphw_dsp_rom_words(r.data, r.size, o.add(OPT_IROM) as *mut u16, IROM_WORDS) }
        1 => { let r = coefficient_rom(); wphw_dsp_rom_words(r.data, r.size, o.add(OPT_COEF) as *mut u16, COEF_WORDS) }
        2 => { c.core = wprt_claude_dsp_new_core(); true }
        3 => core_initialize(c.core, o),
        4 => { core_reset(c.core); true }
        5 => { init_instruction_table(); true }
        6 => { G_CORE = c.core; true }
        _ => abort(),
    }
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_dsp_bootstrap_sdk() -> bool {
    let mut context = core::mem::MaybeUninit::<BootstrapContext>::uninit();
    let p = context.as_mut_ptr();
    wprt_claude_dsp_init_options((*p).options.as_mut_ptr());
    (*p).core = null_mut();
    wr((*p).options.as_mut_ptr(), OPT_CORE_TYPE, CORE_TYPE_INTERP);
    wphw_dsp_bootstrap_flow(p as *mut c_void, bootstrap_operation)
}
