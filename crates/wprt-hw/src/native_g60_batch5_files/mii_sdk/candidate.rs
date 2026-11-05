// Claude G60: whole-member Rust port of hybrid mii_sdk.cpp (wp::mii API owners over the wprt_mii_* Rust adapter).
// Residue (residue.cpp, verbatim C++): kFields dynamic initializer/atexit owner, Database::save exception-sink callback.
#![allow(non_upper_case_globals, clippy::missing_safety_doc)]
use core::ffi::{c_char, c_int, c_void};
include!("../common/cxx.rs");

const K_RECORD: usize = 0x4A;
const K_FILE: usize = 0xBE6C0;
const OFF_IMAGE: usize = 0x1CE8; // std::array<Record,100> records_ precedes image_
const OFF_VALID: usize = 0x1D00;
const OFF_EXISTED: usize = 0x1D01;

#[repr(C)] pub struct MiiPath { p: *const u16, n: usize }
type MiiEmit = unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize);
extern "C-unwind" {
    fn wprt_mii_load_file(p: *const u16, n: usize, records: *mut u8, valid: *mut bool, existed: *mut bool, ctx: *mut c_void, cb: unsafe extern "C-unwind" fn(*mut c_void, *const u8, usize) -> *const u8, err: *mut c_int) -> c_int;
    fn wprt_mii_entropy(seed: *mut u32) -> c_int;
    fn wprt_mii_seeded_add(rec: *mut u8, records: *mut u8, list: *const usize, n: usize, seed: u32) -> usize;
    fn wprt_mii_read_text(rec: *const u8, offset: usize, ctx: *mut c_void, emit: MiiEmit);
    fn wprt_mii_default(rec: *mut u8, name: *const u8, n: usize);
    fn wprt_mii_blank_image(p: *mut u8, n: usize);
    fn wprt_mii_import(p: *const u8, n: usize, ctx: *mut c_void, cb: unsafe extern "C-unwind" fn(*mut c_void, *const u8));
    fn wprt_mii_parse(p: *const u8, n: usize, records: *mut u8, ctx: *mut c_void, cb: unsafe extern "C-unwind" fn(*mut c_void) -> *const u8) -> bool;
    fn wprt_mii_bytes(p: *mut u8, n: usize, records: *const u8);
    fn wprt_mii_used(records: *const u8, ctx: *mut c_void, cb: unsafe extern "C-unwind" fn(*mut c_void, usize));
    fn wprt_mii_prepare_add(rec: *mut u8, records: *const u8, list: *const usize, n: usize);
    fn wprt_claude_mii_kfields() -> *const CxxVec;
    fn wprt_claude_mii_save_io(p: *const u8, n: usize, valid: *mut bool, existed: *mut bool, path: *const c_void) -> bool;
    #[link_name = "_ZSt19__throw_ios_failurePKci"] fn cxx_throw_ios_failure(m: *const c_char, e: c_int) -> !;
    #[link_name = "_ZSt21__throw_runtime_errorPKc"] fn cxx_throw_runtime_error(m: *const c_char) -> !;
}
extern "C" { fn memset(p: *mut c_void, c: c_int, n: usize) -> *mut c_void; }

// Landing-pad owners (C++ destructors on unwind).
struct VecGuard(*mut CxxVec); impl Drop for VecGuard { fn drop(&mut self) { unsafe { v_free(self.0) } } }
struct StrGuard(*mut CxxString); impl Drop for StrGuard { fn drop(&mut self) { unsafe { s_dispose(self.0) } } }

// vector<unsigned char>::_M_fill_insert(end(), n, x) as used by resize(n_new, 0) when growing.
unsafe fn u8_fill_append(v: *mut CxxVec, n: usize, x: u8) {
    if n == 0 { return; }
    let size = (*v).e as usize - (*v).b as usize;
    if ((*v).c as usize - (*v).e as usize) >= n { memset((*v).e as *mut c_void, x as c_int, n); (*v).e = (*v).e.add(n); return; }
    let max = isize::MAX as usize;
    if max - size < n { cxx_throw_length_error(concat!("vector::_M_fill_insert", "\0").as_ptr() as _); }
    let mut len = size + if size > n { size } else { n };
    if len < size || len > max { len = max; }
    let new = cxx_new(len);
    memset(new.add(size) as *mut c_void, x as c_int, n);
    if size > 0 { core::ptr::copy_nonoverlapping((*v).b, new, size); }
    v_free(v);
    (*v).b = new; (*v).e = new.add(size + n); (*v).c = new.add(len);
}
unsafe fn u8_resize_up(v: *mut CxxVec, n: usize) { let size = (*v).e as usize - (*v).b as usize; if size < n { u8_fill_append(v, n - size, 0); } }
// vector<unsigned char>::assign(first, first+n) (forward-iterator _M_assign_aux).
unsafe fn u8_assign(v: *mut CxxVec, p: *const u8, n: usize) {
    let cap = (*v).c as usize - (*v).b as usize;
    let size = (*v).e as usize - (*v).b as usize;
    if n > cap {
        if n > isize::MAX as usize { cxx_throw_length_error(concat!("cannot create std::vector larger than max_size()", "\0").as_ptr() as _); }
        let tmp = cxx_new(n);
        core::ptr::copy_nonoverlapping(p, tmp, n);
        v_free(v);
        (*v).b = tmp; (*v).e = tmp.add(n); (*v).c = tmp.add(n);
    } else if size >= n {
        if n > 0 { core::ptr::copy(p, (*v).b, n); }
        (*v).e = (*v).b.add(n);
    } else {
        if size > 0 { core::ptr::copy(p, (*v).b, size); }
        core::ptr::copy(p.add(size), (*v).e, n - size);
        (*v).e = (*v).b.add(n);
    }
}
// vector<unsigned char>::operator=(const vector&).
unsafe fn u8_copy_assign(v: *mut CxxVec, x: *const CxxVec) {
    if v as *const CxxVec == x { return; }
    let xlen = (*x).e as usize - (*x).b as usize;
    let cap = (*v).c as usize - (*v).b as usize;
    let size = (*v).e as usize - (*v).b as usize;
    if xlen > cap {
        let tmp = cxx_new(xlen);
        core::ptr::copy_nonoverlapping((*x).b, tmp, xlen);
        v_free(v);
        (*v).b = tmp; (*v).c = tmp.add(xlen);
    } else if size >= xlen {
        if xlen > 0 { core::ptr::copy((*x).b, (*v).b, xlen); }
    } else {
        if size > 0 { core::ptr::copy((*x).b, (*v).b, size); }
        core::ptr::copy((*x).b.add(size), (*v).e, xlen - size);
    }
    (*v).e = (*v).b.add(xlen);
}

// std::string::assign(const char*, n) as inlined by GCC 15.2 -O2 (_M_replace fast paths + _M_create, sized dispose);
// aliasing sources take the library's _M_replace_cold exactly like the inline code.
extern "C-unwind" { #[link_name = "_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE15_M_replace_coldEPcyPKcyy"] fn cxx_s_replace_cold(s: *mut CxxString, p: *mut u8, len1: usize, src: *const u8, len2: usize, how_much: usize); }
unsafe fn assign_inline(s: *mut CxxString, src: *const u8, n: usize) {
    const MAX: usize = 0x7FFF_FFFF_FFFF_FFFE;
    if n > MAX { cxx_throw_length_error(concat!("basic_string::_M_replace", "\0").as_ptr() as _); }
    let local = (*s).p == s_local(s);
    let cap = if local { 15 } else { (*s).u[0] as usize };
    if n <= cap {
        let p = (*s).p;
        if (src as usize) < (p as usize) || (p as usize) + (*s).n < (src as usize) { s_copy(p, src, n); }
        else { cxx_s_replace_cold(s, p, (*s).n, src, n, 0); }
    } else {
        let (alloc, newcap) = if n < cap * 2 { if cap * 2 <= MAX { (cap * 2 + 1, cap * 2) } else { (MAX + 1, MAX) } } else { (n + 1, n) };
        let q = cxx_new(alloc);
        if !src.is_null() && n != 0 { core::ptr::copy_nonoverlapping(src, q, n); }
        if !local { cxx_delete((*s).p, cap + 1); }
        (*s).p = q; (*s).u[0] = newcap as u64;
    }
    (*s).n = n; *(*s).p.add(n) = 0;
}
unsafe extern "C-unwind" fn mii_receive_text(ctx: *mut c_void, data: *const u8, n: usize) { assign_inline(ctx as *mut CxxString, data, n); }
unsafe extern "C-unwind" fn mii_receive_record(ctx: *mut c_void, data: *const u8) {
    let mut record = [0u8; K_RECORD];
    core::ptr::copy_nonoverlapping(data, record.as_mut_ptr(), K_RECORD);
    v_push_pod(ctx as *mut CxxVec, K_RECORD, record.as_ptr());
}
unsafe extern "C-unwind" fn used_sink(ctx: *mut c_void, slot: usize) { v_push_pod(ctx as *mut CxxVec, 8, &slot as *const usize as *const u8); }
unsafe extern "C-unwind" fn load_sink(c: *mut c_void, p: *const u8, n: usize) -> *const u8 {
    let image = c as *mut CxxVec; u8_assign(image, p, n); u8_resize_up(image, K_FILE); (*image).b
}
#[repr(C)] struct ParseContext { input: *const CxxVec, image: *mut CxxVec }
unsafe extern "C-unwind" fn parse_sink(raw: *mut c_void) -> *const u8 {
    let c = &*(raw as *const ParseContext); u8_copy_assign(c.image, c.input); u8_resize_up(c.image, K_FILE); (*c.image).b
}

unsafe fn read_text(ret: *mut CxxString, record: *const u8, offset: usize) -> *mut CxxString {
    s_init(ret); let g = StrGuard(ret); wprt_mii_read_text(record, offset, ret as *mut c_void, mii_receive_text); core::mem::forget(g); ret
}
unsafe fn used_into(list: *mut CxxVec, this: *const u8) { v_init(list); let g = VecGuard(list); wprt_mii_used(this, list as *mut c_void, used_sink); core::mem::forget(g); }

#[export_name = "_ZN2wp3mii6fieldsEv"]
pub unsafe extern "C-unwind" fn fields() -> *const CxxVec { wprt_claude_mii_kfields() }
#[export_name = "_ZN2wp3mii4nameB5cxx11ERKSt5arrayIhLy74EE"]
pub unsafe extern "C-unwind" fn name(ret: *mut CxxString, record: *const u8) -> *mut CxxString { read_text(ret, record, 0x02) }
#[export_name = "_ZN2wp3mii7creatorB5cxx11ERKSt5arrayIhLy74EE"]
pub unsafe extern "C-unwind" fn creator(ret: *mut CxxString, record: *const u8) -> *mut CxxString { read_text(ret, record, 0x36) }
#[export_name = "_ZN2wp3mii12blank_recordEv"]
pub unsafe extern "C-unwind" fn blank_record(ret: *mut u8) -> *mut u8 { memset(ret as *mut c_void, 0, K_RECORD); ret }
#[export_name = "_ZN2wp3mii14default_recordERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn default_record(ret: *mut u8, utf8_name: *const CxxString) -> *mut u8 {
    memset(ret as *mut c_void, 0, K_RECORD); wprt_mii_default(ret, (*utf8_name).p, (*utf8_name).n); ret
}
#[export_name = "_ZN2wp3mii14import_recordsERKSt6vectorIhSaIhEE"]
pub unsafe extern "C-unwind" fn import_records(ret: *mut CxxVec, data: *const CxxVec) -> *mut CxxVec {
    v_init(ret); let g = VecGuard(ret);
    wprt_mii_import((*data).b, (*data).e as usize - (*data).b as usize, ret as *mut c_void, mii_receive_record);
    core::mem::forget(g); ret
}
#[export_name = "_ZN2wp3mii8DatabaseC2Ev"]
pub unsafe extern "C-unwind" fn database_ctor(this: *mut u8) {
    memset(this as *mut c_void, 0, OFF_IMAGE);
    let image = this.add(OFF_IMAGE) as *mut CxxVec;
    let p = cxx_new(K_FILE);
    (*image).b = p; (*image).c = p.add(K_FILE);
    memset(p as *mut c_void, 0, K_FILE);
    (*image).e = p.add(K_FILE);
    let g = VecGuard(image);
    wprt_mii_blank_image(p, K_FILE);
    core::mem::forget(g);
    *this.add(OFF_VALID) = 1; *this.add(OFF_EXISTED) = 0;
}
#[export_name = "_ZN2wp3mii8DatabaseC1Ev"]
pub unsafe extern "C-unwind" fn database_ctor1(this: *mut u8) { database_ctor(this) }
#[export_name = "_ZN2wp3mii8Database5parseERKSt6vectorIhSaIhEE"]
pub unsafe extern "C-unwind" fn database_parse(this: *mut u8, data: *const CxxVec) -> bool {
    let mut context = ParseContext { input: data, image: this.add(OFF_IMAGE) as *mut CxxVec };
    wprt_mii_parse((*data).b, (*data).e as usize - (*data).b as usize, this, &mut context as *mut ParseContext as *mut c_void, parse_sink)
}
#[export_name = "_ZN2wp3mii8Database4loadERKNSt10filesystem7__cxx114pathE"]
pub unsafe extern "C-unwind" fn database_load(this: *mut u8, path: *const c_void) -> bool {
    // path.native(): std::wstring _M_pathname is the first member.
    let p = *(path as *const *const u16); let n = *(path as *const usize).add(1);
    let mut error: c_int = 0;
    let result = wprt_mii_load_file(p, n, this, this.add(OFF_VALID) as *mut bool, this.add(OFF_EXISTED) as *mut bool, this.add(OFF_IMAGE) as *mut c_void, load_sink, &mut error);
    if result < 0 { cxx_throw_ios_failure(concat!("basic_filebuf::underflow error reading the file", "\0").as_ptr() as _, error); }
    result != 0
}
#[export_name = "_ZNK2wp3mii8Database5bytesEv"]
pub unsafe extern "C-unwind" fn database_bytes(ret: *mut CxxVec, this: *const u8) -> *mut CxxVec {
    let image = this.add(OFF_IMAGE) as *const CxxVec;
    let n = (*image).e as usize - (*image).b as usize;
    let p = if n != 0 { cxx_new(n) } else { core::ptr::null_mut() };
    (*ret).b = p; (*ret).c = p.wrapping_add(n); (*ret).e = p;
    if n > 0 { core::ptr::copy_nonoverlapping((*image).b, p, n); }
    (*ret).e = p.wrapping_add(n);
    let g = VecGuard(ret);
    wprt_mii_bytes(p, n, this);
    core::mem::forget(g); ret
}
#[export_name = "_ZN2wp3mii8Database4saveERKNSt10filesystem7__cxx114pathE"]
pub unsafe extern "C-unwind" fn database_save(this: *mut u8, path: *const c_void) -> bool {
    let mut data = CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() };
    database_bytes(&mut data, this);
    let g = VecGuard(&mut data);
    let result = wprt_claude_mii_save_io(data.b, data.e as usize - data.b as usize, this.add(OFF_VALID) as *mut bool, this.add(OFF_EXISTED) as *mut bool, path);
    drop(g);
    result
}
#[export_name = "_ZNK2wp3mii8Database4usedEv"]
pub unsafe extern "C-unwind" fn database_used(ret: *mut CxxVec, this: *const u8) -> *mut CxxVec { used_into(ret, this); ret }
#[export_name = "_ZN2wp3mii8Database3addESt5arrayIhLy74EE"]
pub unsafe extern "C-unwind" fn database_add(this: *mut u8, record: *mut u8) -> usize {
    let mut list = CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() };
    used_into(&mut list, this);
    let g = VecGuard(&mut list);
    wprt_mii_prepare_add(record, this, list.b as *const usize, v_len(&list, 8));
    let mut seed: u32 = 0;
    if wprt_mii_entropy(&mut seed) != 0 { cxx_throw_runtime_error(concat!("random_device: rand_s failed", "\0").as_ptr() as _); }
    let r = wprt_mii_seeded_add(record, this, list.b as *const usize, v_len(&list, 8), seed);
    drop(g);
    r
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_mii_string_sdk(raw: *const c_void, p: *mut *const u8, n: *mut usize) { let s = raw as *const CxxString; *p = (*s).p; *n = (*s).n; }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_mii_fields_sdk(p: *mut *const u8, n: *mut usize) { let f = fields(); *p = (*f).b; *n = ((*f).e as usize - (*f).b as usize) / 16; }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_mii_record_sdk(p: *const c_void) -> *mut u8 { p as *mut u8 }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_mii_db_records_sdk(p: *const c_void) -> *mut u8 { p as *mut u8 }
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_mii_count_sdk(p: *const c_void) -> usize {
    let mut list = CxxVec { b: core::ptr::null_mut(), e: core::ptr::null_mut(), c: core::ptr::null_mut() };
    used_into(&mut list, p as *const u8);
    let n = v_len(&list, 8); v_free(&mut list); n
}
