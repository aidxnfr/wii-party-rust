// AI-assisted (Claude, G60): exact libstdc++ (GCC 15, x86_64-w64-mingw32) std::string / std::vector primitives for C++-boundary ports.
// Objects are built in place at stable addresses (SSO strings are self-referential); never move a CxxString by value after construction.
extern "C-unwind" {
    #[link_name = "_Znwy"] pub fn op_new(n: usize) -> *mut u8;
    #[link_name = "_ZdlPvy"] pub fn op_delete(p: *mut u8, n: usize);
    #[link_name = "_ZSt20__throw_length_errorPKc"] pub fn throw_length_error(w: *const u8) -> !;
    #[link_name = "_ZSt19__throw_logic_errorPKc"] pub fn throw_logic_error(w: *const u8) -> !;
    #[link_name = "_ZSt25__throw_bad_function_callv"] pub fn throw_bad_function_call() -> !;
}
extern "C" { #[link_name = "memcpy"] fn memcpy_c(d: *mut std::ffi::c_void, s: *const std::ffi::c_void, n: usize) -> *mut std::ffi::c_void; }
#[inline] pub unsafe fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8 { memcpy_c(d.cast(), s.cast(), n).cast() }
pub const STR_MAX: usize = 0x3FFF_FFFF_FFFF_FFFF;
#[repr(C)] pub struct CxxString { pub p: *mut u8, pub len: usize, pub buf: [u8; 16] }
#[repr(C)] #[derive(Clone, Copy)] pub struct RawVec { pub start: *mut u8, pub finish: *mut u8, pub eos: *mut u8 }
#[inline] pub unsafe fn local(s: *mut CxxString) -> *mut u8 { std::ptr::addr_of_mut!((*s).buf).cast() }
#[inline] pub unsafe fn is_local(s: *mut CxxString) -> bool { (*s).p == local(s) }
#[inline] pub unsafe fn cap(s: *mut CxxString) -> usize { local(s).cast::<usize>().read_unaligned() }
#[inline] pub unsafe fn set_cap(s: *mut CxxString, c: usize) { local(s).cast::<usize>().write_unaligned(c) }
#[inline] unsafe fn s_copy(d: *mut u8, s: *const u8, n: usize) { if n == 1 { *d = *s } else if n != 0 { memcpy(d, s, n); } }
/// basic_string(const char*, size_type)
pub unsafe fn str_construct(s: *mut CxxString, p: *const u8, n: usize) {
    (*s).p = local(s);
    if p.is_null() && n > 0 { throw_logic_error(b"basic_string: construction from null is not valid\0".as_ptr()) }
    if n > 15 {
        if n > STR_MAX { throw_length_error(b"basic_string::_M_create\0".as_ptr()) }
        let d = op_new(n + 1); (*s).p = d; set_cap(s, n);
    }
    s_copy((*s).p, p, n);
    (*s).len = n; *(*s).p.add(n) = 0;
}
/// ~basic_string()
pub unsafe fn str_destroy(s: *mut CxxString) { if !is_local(s) { op_delete((*s).p, cap(s) + 1) } }
/// basic_string(basic_string&&)
pub unsafe fn str_move_construct(d: *mut CxxString, s: *mut CxxString) {
    (*d).p = local(d);
    if is_local(s) { memcpy(local(d), local(s), (*s).len + 1); } else { (*d).p = (*s).p; set_cap(d, cap(s)); }
    (*d).len = (*s).len; (*s).p = local(s); (*s).len = 0; *local(s) = 0;
}
/// operator=(basic_string&&) with std::allocator (always equal)
pub unsafe fn str_move_assign(d: *mut CxxString, s: *mut CxxString) {
    if is_local(s) {
        if d != s { let n = (*s).len; if n != 0 { s_copy((*d).p, (*s).p, n) } (*d).len = n; *(*d).p.add(n) = 0; }
    } else {
        let mut data: *mut u8 = std::ptr::null_mut(); let mut c = 0usize;
        if !is_local(d) { data = (*d).p; c = cap(d); }
        (*d).p = (*s).p; (*d).len = (*s).len; set_cap(d, cap(s));
        if !data.is_null() { (*s).p = data; set_cap(s, c); } else { (*s).p = local(s); }
    }
    (*s).len = 0; *(*s).p = 0;
}
pub struct StrDrop(pub *mut CxxString);
impl Drop for StrDrop { fn drop(&mut self) { unsafe { str_destroy(self.0) } } }
/// std::vector<T> storage release (T trivially destructible or already destroyed)
pub unsafe fn vec_dealloc(v: &RawVec) { if !v.start.is_null() { op_delete(v.start, v.eos as usize - v.start as usize) } }
/// std::vector<uint8_t>(p, p + n)
pub unsafe fn vec_u8_from_range(p: *const u8, n: usize) -> RawVec {
    if n > isize::MAX as usize { throw_length_error(b"cannot create std::vector larger than max_size()\0".as_ptr()) }
    let start = if n != 0 { op_new(n) } else { std::ptr::null_mut() };
    if n > 1 { memcpy(start, p, n); } else if n == 1 { *start = *p; }
    RawVec { start, finish: start.add(n), eos: start.add(n) }
}
pub struct VecDrop(pub RawVec);
impl Drop for VecDrop { fn drop(&mut self) { unsafe { vec_dealloc(&self.0) } } }
/// std::function<R(Args...)> {_Any_data[16], _M_manager, _M_invoker}
pub unsafe fn function_check(f: *const u8) -> *const () {
    if f.add(16).cast::<usize>().read() == 0 { throw_bad_function_call() }
    f.add(24).cast::<*const ()>().read()
}
// Itanium function-local static guard (64-bit guard object, byte 0 = initialized).
extern "C-unwind" {
    pub fn __cxa_guard_acquire(g: *mut u64) -> i32;
    pub fn __cxa_guard_release(g: *mut u64);
    pub fn __cxa_guard_abort(g: *mut u64);
}
pub struct GuardAbort(pub *mut u64);
impl Drop for GuardAbort { fn drop(&mut self) { unsafe { __cxa_guard_abort(self.0) } } }
/// `static const T x = init();` : runs init exactly as GCC's guarded static initialization (abort on unwind, retry next call).
pub unsafe fn guarded_static(g: *mut u64, init: impl FnOnce()) {
    if (*(g as *const std::sync::atomic::AtomicU8)).load(std::sync::atomic::Ordering::Acquire) == 0 && __cxa_guard_acquire(g) != 0 {
        let a = GuardAbort(g); init(); std::mem::forget(a); __cxa_guard_release(g);
    }
}
/// std::vector<T>(p, p + n) for trivially copyable T of size `es`
pub unsafe fn vec_from_range(p: *const u8, n: usize, es: usize) -> RawVec {
    if n > isize::MAX as usize / es { throw_length_error(b"cannot create std::vector larger than max_size()\0".as_ptr()) }
    let bytes = n * es;
    let start = if n != 0 { op_new(bytes) } else { std::ptr::null_mut() };
    if n > 1 { memcpy(start, p, bytes); } else if n == 1 { memcpy(start, p, es); }
    RawVec { start, finish: start.add(bytes), eos: start.add(bytes) }
}
/// std::vector<T>::assign(first, first + n) (forward iterators), T trivially copyable of size `es`
pub unsafe fn vec_assign(v: *mut RawVec, p: *const u8, n: usize, es: usize) {
    let bytes = n * es;
    let capb = (*v).eos as usize - (*v).start as usize; let sizeb = (*v).finish as usize - (*v).start as usize;
    if bytes > capb {
        if n > isize::MAX as usize / es { throw_length_error(b"cannot create std::vector larger than max_size()\0".as_ptr()) }
        let tmp = if n != 0 { op_new(bytes) } else { std::ptr::null_mut() };
        if n != 0 { memcpy(tmp, p, bytes); }
        if !(*v).start.is_null() { op_delete((*v).start, capb) }
        (*v).start = tmp; (*v).finish = tmp.add(bytes); (*v).eos = tmp.add(bytes);
    } else if sizeb >= bytes {
        if n != 0 { memcpy((*v).start, p, bytes); }
        (*v).finish = (*v).start.add(bytes);
    } else {
        if sizeb != 0 { memcpy((*v).start, p, sizeb); }
        if bytes - sizeb != 0 { memcpy((*v).finish, p.add(sizeb), bytes - sizeb); }
        (*v).finish = (*v).start.add(bytes);
    }
}
extern "C-unwind" {
    #[link_name = "_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE15_M_replace_coldEPcyPKcyy"]
    pub fn str_replace_cold(this: *mut CxxString, p: *mut u8, len1: usize, s: *const u8, len2: usize, how_much: usize);
}
#[inline] pub unsafe fn str_capacity(s: *mut CxxString) -> usize { if is_local(s) { 15 } else { cap(s) } }
/// basic_string::assign(const char*, size_type) == _M_replace(0, size(), s, n) (GCC 15 inline body; _M_mutate inlined)
pub unsafe fn str_assign(s: *mut CxxString, p: *const u8, n: usize) {
    let old = (*s).len;
    if n > STR_MAX { throw_length_error(b"basic_string::_M_replace\0".as_ptr()) }
    let capacity = str_capacity(s);
    if n <= capacity {
        let d = (*s).p;
        if (p as usize) < (d as usize) || (d as usize + old) < (p as usize) { s_copy(d, p, n) } else { str_replace_cold(s, d, old, p, n, 0) }
    } else {
        let mut nc = n;
        if nc > capacity && nc < 2 * capacity { nc = 2 * capacity; if nc > STR_MAX { nc = STR_MAX; } }
        let r = op_new(nc + 1);
        if !p.is_null() && n != 0 { s_copy(r, p, n) }
        if !is_local(s) { op_delete((*s).p, cap(s) + 1) }
        (*s).p = r; set_cap(s, nc);
    }
    (*s).len = n; *(*s).p.add(n) = 0;
}
/// `[](void* c,const uint8_t* p,size_t n){ static_cast<std::string*>(c)->assign((const char*)p, n); }`
pub unsafe extern "C-unwind" fn assign_sink(c: *mut u8, p: *const u8, n: usize) { str_assign(c.cast(), p, n) }
/// std::string default-construct in place
pub unsafe fn str_init(s: *mut CxxString) { (*s).p = local(s); (*s).len = 0; *local(s) = 0; }
/// sret-returned std::string guard: destroys on unwind only
pub struct StrRet(pub *mut CxxString, pub bool);
impl Drop for StrRet { fn drop(&mut self) { if self.1 { unsafe { str_destroy(self.0) } } } }
/// std::vector<std::string> destructor
pub unsafe fn vec_str_destroy(v: *mut RawVec) { let mut e = (*v).start; while e != (*v).finish { str_destroy(e.cast()); e = e.add(32); } vec_dealloc(&*v); }
pub struct VecStrDrop(pub *mut RawVec);
impl Drop for VecStrDrop { fn drop(&mut self) { unsafe { vec_str_destroy(self.0) } } }
