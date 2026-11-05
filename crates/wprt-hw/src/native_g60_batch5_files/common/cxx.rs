// Claude G60 shared libstdc++ (GCC 15.2 mingw-w64, _GLIBCXX_USE_CXX11_ABI=1) object model for SDK shim ports.
// std::string out-of-line members (_M_replace/_M_create/_M_dispose) are the library's extern-template instantiations;
// inline/template algorithms (constructors, moves, vector growth/relocation, std::function) are reproduced here.
#[repr(C)] pub struct CxxString { pub p: *mut u8, pub n: usize, pub u: [u64; 2] }
#[repr(C)] pub struct CxxVec { pub b: *mut u8, pub e: *mut u8, pub c: *mut u8 }
#[repr(C)] pub struct CxxFunction { pub data: [u64; 2], pub manager: usize, pub invoker: usize }
#[repr(C)] pub struct CxxTypeInfo { pub vptr: *const u8, pub name: *const u8 }
unsafe impl Sync for CxxTypeInfo {}
extern "C-unwind" {
    #[link_name = "_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEyyPKcy"] pub fn cxx_s_replace(s: *mut CxxString, pos: usize, n1: usize, p: *const u8, n2: usize) -> *mut CxxString;
    #[link_name = "_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERyy"] pub fn cxx_s_create(s: *mut CxxString, cap: *mut usize, old: usize) -> *mut u8;
    #[link_name = "_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv"] pub fn cxx_s_dispose(s: *mut CxxString);
    #[link_name = "_Znwy"] pub fn cxx_new(n: usize) -> *mut u8;
    #[link_name = "_ZdlPvy"] pub fn cxx_delete(p: *mut u8, n: usize);
    #[link_name = "_ZSt20__throw_length_errorPKc"] pub fn cxx_throw_length_error(m: *const core::ffi::c_char) -> !;
    #[link_name = "_ZSt19__throw_logic_errorPKc"] pub fn cxx_throw_logic_error(m: *const core::ffi::c_char) -> !;
    #[link_name = "_ZSt25__throw_bad_function_callv"] pub fn cxx_throw_bad_function_call() -> !;
}
extern "C" { #[link_name = "_ZTVN10__cxxabiv117__class_type_infoE"] pub static CXX_CLASS_TYPE_INFO_VTABLE: [usize; 3]; }
pub const CXX_STRING_MAX: usize = 0x3FFF_FFFF_FFFF_FFFF;
#[inline] pub unsafe fn s_local(s: *mut CxxString) -> *mut u8 { core::ptr::addr_of_mut!((*s).u) as *mut u8 }
#[inline] pub unsafe fn s_init(s: *mut CxxString) { (*s).p = s_local(s); (*s).n = 0; *s_local(s) = 0; }
#[inline] pub unsafe fn s_copy(d: *mut u8, src: *const u8, n: usize) { if n == 1 { *d = *src; } else if n != 0 { core::ptr::copy_nonoverlapping(src, d, n); } }
/// basic_string(const char* s, size_type n): null check + _M_construct(s, s+n, forward_iterator_tag).
pub unsafe fn s_construct(s: *mut CxxString, p: *const u8, n: usize) {
    (*s).p = s_local(s);
    if p.is_null() && n > 0 { cxx_throw_logic_error(concat!("basic_string: construction from null is not valid", "\0").as_ptr() as _); }
    if n > 15 { let mut c = n; let q = cxx_s_create(s, &mut c, 0); (*s).p = q; (*s).u[0] = c as u64; }
    s_copy((*s).p, p, n);
    (*s).n = n; *(*s).p.add(n) = 0;
}
/// basic_string(const char* s): null check, traits length, then as above.
pub unsafe fn s_construct_cstr(s: *mut CxxString, p: *const u8) {
    if p.is_null() { (*s).p = s_local(s); cxx_throw_logic_error(concat!("basic_string: construction from null is not valid", "\0").as_ptr() as _); }
    let mut n = 0; while *p.add(n) != 0 { n += 1; }
    s_construct(s, p, n);
}
/// copy constructor: _M_construct<true>(str.data(), str.length()).
pub unsafe fn s_copy_construct(s: *mut CxxString, src: *const CxxString) {
    let n = (*src).n;
    if n > 15 { (*s).p = s_local(s); let mut c = n; let q = cxx_s_create(s, &mut c, 0); (*s).p = q; (*s).u[0] = c as u64; } else { (*s).p = s_local(s); }
    s_copy((*s).p, (*src).p, n + 1);
    (*s).n = n;
}
/// move constructor.
pub unsafe fn s_move_construct(s: *mut CxxString, src: *mut CxxString) {
    (*s).p = s_local(s);
    if (*src).p == s_local(src) { s_copy(s_local(s), s_local(src), (*src).n + 1); } else { (*s).p = (*src).p; (*s).u[0] = (*src).u[0]; }
    (*s).n = (*src).n;
    (*src).p = s_local(src); (*src).n = 0; *s_local(src) = 0;
}
extern "C-unwind" { #[link_name = "_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE15_M_replace_coldEPcyPKcyy"] fn cxx_s_replace_cold_c(s: *mut CxxString, p: *mut u8, len1: usize, src: *const u8, len2: usize, how_much: usize); }
// std::string::assign(const char*, n) as inlined by GCC 15.2 -O2 (_M_replace fast paths + _M_create, SIZED dispose) - matches pristine objs that only reference _M_replace_cold.
pub unsafe fn s_assign(s: *mut CxxString, src: *const u8, n: usize) {
    const MAX: usize = 0x7FFF_FFFF_FFFF_FFFE;
    if n > MAX { cxx_throw_length_error(concat!("basic_string::_M_replace", "\0").as_ptr() as _); }
    let local = (*s).p == s_local(s);
    let cap = if local { 15 } else { (*s).u[0] as usize };
    if n <= cap {
        let p = (*s).p;
        if (src as usize) < (p as usize) || (p as usize) + (*s).n < (src as usize) { s_copy(p, src, n); }
        else { cxx_s_replace_cold_c(s, p, (*s).n, src, n, 0); }
    } else {
        let (alloc, newcap) = if n < cap * 2 { if cap * 2 <= MAX { (cap * 2 + 1, cap * 2) } else { (MAX + 1, MAX) } } else { (n + 1, n) };
        let q = cxx_new(alloc);
        if !src.is_null() && n != 0 { core::ptr::copy_nonoverlapping(src, q, n); }
        if !local { cxx_delete((*s).p, cap + 1); }
        (*s).p = q; (*s).u[0] = newcap as u64;
    }
    (*s).n = n; *(*s).p.add(n) = 0;
}
#[inline] pub unsafe fn s_dispose(s: *mut CxxString) { cxx_s_dispose(s); }
#[inline] pub unsafe fn v_len(v: *const CxxVec, size: usize) -> usize { ((*v).e as usize - (*v).b as usize) / size }
#[inline] pub unsafe fn v_init(v: *mut CxxVec) { (*v).b = core::ptr::null_mut(); (*v).e = core::ptr::null_mut(); (*v).c = core::ptr::null_mut(); }
#[inline] pub unsafe fn v_free(v: *mut CxxVec) { if !(*v).b.is_null() { cxx_delete((*v).b, (*v).c as usize - (*v).b as usize); } }
/// _M_realloc_append growth: returns (new storage, new capacity elements) after _M_check_len(1).
pub unsafe fn v_grow(v: *mut CxxVec, size: usize, msg: *const core::ffi::c_char) -> (*mut u8, usize) {
    let n = v_len(v, size); let max = (isize::MAX as usize) / size;
    if max - n < 1 { cxx_throw_length_error(msg); }
    let mut len = n + if n > 1 { n } else { 1 };
    if len < n || len > max { len = max; }
    (cxx_new(len * size), len)
}
/// finish _M_realloc_append after the new element was constructed at new + n*size; `relocate(dst, src)` moves one element.
pub unsafe fn v_finish(v: *mut CxxVec, size: usize, new: *mut u8, len: usize, relocate: Option<unsafe fn(*mut u8, *mut u8)>) {
    let n = v_len(v, size);
    match relocate { None => { if n > 0 { core::ptr::copy((*v).b, new, n * size); } } Some(f) => { for i in 0..n { f(new.add(i * size), (*v).b.add(i * size)); } } }
    v_free(v);
    (*v).b = new; (*v).e = new.add((n + 1) * size); (*v).c = new.add(len * size);
}
pub unsafe fn relocate_string(d: *mut u8, s: *mut u8) { s_move_construct(d as *mut CxxString, s as *mut CxxString); }
/// vector<std::string>::emplace_back(const char*&, size_t&).
pub unsafe fn v_emplace_string(v: *mut CxxVec, p: *const u8, n: usize) {
    if (*v).e != (*v).c { s_construct((*v).e as *mut CxxString, p, n); (*v).e = (*v).e.add(32); return; }
    let (new, len) = v_grow(v, 32, concat!("vector::_M_realloc_append", "\0").as_ptr() as _);
    let nn = v_len(v, 32);
    struct G(*mut u8, usize, bool); impl Drop for G { fn drop(&mut self) { if self.2 { unsafe { cxx_delete(self.0, self.1) } } } }
    let mut g = G(new, len * 32, true);
    s_construct(new.add(nn * 32) as *mut CxxString, p, n);
    g.2 = false;
    v_finish(v, 32, new, len, Some(relocate_string));
}
/// vector<T>::push_back for trivially copyable T.
pub unsafe fn v_push_pod(v: *mut CxxVec, size: usize, item: *const u8) {
    if (*v).e != (*v).c { core::ptr::copy_nonoverlapping(item, (*v).e, size); (*v).e = (*v).e.add(size); return; }
    let (new, len) = v_grow(v, size, concat!("vector::_M_realloc_append", "\0").as_ptr() as _);
    core::ptr::copy_nonoverlapping(item, new.add(v_len(v, size) * size), size);
    v_finish(v, size, new, len, None);
}
pub unsafe fn v_destroy_strings(v: *mut CxxVec) { let n = v_len(v, 32); for i in 0..n { s_dispose((*v).b.add(i * 32) as *mut CxxString); } v_free(v); }
/// std::function<R(A...)> invoke: empty -> __throw_bad_function_call.
#[inline] pub unsafe fn f_check(f: *const CxxFunction) { if (*f).manager == 0 { cxx_throw_bad_function_call(); } }
/// std::function destructor (_Function_base::~_Function_base).
pub unsafe fn f_destroy(f: *mut CxxFunction) { if (*f).manager != 0 { let m: unsafe extern "C-unwind" fn(*mut CxxFunction, *const CxxFunction, i32) -> bool = core::mem::transmute((*f).manager); m(f, f, 3); } }
