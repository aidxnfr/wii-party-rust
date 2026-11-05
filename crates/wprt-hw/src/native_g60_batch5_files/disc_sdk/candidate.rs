// AI-assisted (Claude, G60) disc_sdk.cpp -> Rust: whole member, no C++ residue.
// wphw_disc_sort reproduces std::vector<FileEntry>::push_back growth and libstdc++ (GCC 15) std::sort exactly:
// introsort (threshold 16, depth 2*lg n, median-of-3 to first, unguarded partition, heap fallback) + final insertion sort,
// with FileEntry/std::string move-construct/move-assign/destroy semantics and by-value heap temporaries, so tie order and
// buffer ownership (observable via deallocation order) match the original.
#![allow(non_snake_case)]
#[cfg(not(all(target_os = "windows", target_env = "gnu", target_arch = "x86_64")))]
compile_error!("disc_sdk exports require pinned Windows GNU x64 metadata");
#[allow(dead_code)] mod stdabi { include!("stdabi.rs"); }
use stdabi::*;
use std::mem::MaybeUninit;
extern "C-unwind" { fn wprt_disc_mount(p: *const u8, n: usize) -> bool; }
/// bool wp::disc::mount(const std::string&)
#[export_name = "_ZN2wp4disc5mountERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn mount(p: *const CxxString) -> bool { wprt_disc_mount((*p).p, (*p).len) }
#[export_name = "wphw_disc_path_sdk"]
pub unsafe extern "C-unwind" fn wphw_disc_path_sdk(p: *const u8, n: usize, callback: unsafe extern "C-unwind" fn(*mut u8, *const u8), c: *mut u8) {
    let mut slot = MaybeUninit::<CxxString>::uninit(); let s = slot.as_mut_ptr();
    str_construct(s, p, n);
    let _g = StrDrop(s);
    callback(c, (*s).p);
}
#[repr(C)] struct Entry { offset: u64, size: u64, path: CxxString }
type E = *mut Entry;
const ESZ: usize = 48; const VMAX: usize = isize::MAX as usize / ESZ;
#[inline] unsafe fn sp(e: E) -> *mut CxxString { std::ptr::addr_of_mut!((*e).path) }
#[inline] unsafe fn mc(d: E, s: E) { (*d).offset = (*s).offset; (*d).size = (*s).size; str_move_construct(sp(d), sp(s)); }
#[inline] unsafe fn ma(d: E, s: E) { (*d).offset = (*s).offset; (*d).size = (*s).size; str_move_assign(sp(d), sp(s)); }
#[inline] unsafe fn ds(e: E) { str_destroy(sp(e)) }
#[inline] unsafe fn lt(a: E, b: E) -> bool { (*a).offset < (*b).offset }
#[inline] unsafe fn at(f: E, i: isize) -> E { f.offset(i) }
unsafe fn iter_swap(a: E, b: E) {
    let mut t = MaybeUninit::<Entry>::uninit(); let t = t.as_mut_ptr();
    mc(t, a); ma(a, b); ma(b, t); ds(t);
}
unsafe fn move_median_to_first(r: E, a: E, b: E, c: E) {
    if lt(a, b) { if lt(b, c) { iter_swap(r, b) } else if lt(a, c) { iter_swap(r, c) } else { iter_swap(r, a) } }
    else if lt(a, c) { iter_swap(r, a) } else if lt(b, c) { iter_swap(r, c) } else { iter_swap(r, b) }
}
unsafe fn unguarded_partition(mut first: E, mut last: E, pivot: E) -> E {
    loop {
        while lt(first, pivot) { first = first.add(1); }
        last = last.sub(1);
        while lt(pivot, last) { last = last.sub(1); }
        if !(first < last) { return first; }
        iter_swap(first, last);
        first = first.add(1);
    }
}
unsafe fn push_heap(first: E, mut hole: isize, top: isize, value: E) {
    let mut parent = (hole - 1) / 2;
    while hole > top && lt(at(first, parent), value) { ma(at(first, hole), at(first, parent)); hole = parent; parent = (hole - 1) / 2; }
    ma(at(first, hole), value);
}
unsafe fn adjust_heap(first: E, mut hole: isize, len: isize, value: E) {
    let top = hole; let mut child = hole;
    while child < (len - 1) / 2 {
        child = 2 * (child + 1);
        if lt(at(first, child), at(first, child - 1)) { child -= 1; }
        ma(at(first, hole), at(first, child)); hole = child;
    }
    if (len & 1) == 0 && child == (len - 2) / 2 { child = 2 * (child + 1); ma(at(first, hole), at(first, child - 1)); hole = child - 1; }
    let mut p = MaybeUninit::<Entry>::uninit(); let p = p.as_mut_ptr();
    mc(p, value); push_heap(first, hole, top, p); ds(p);
}
unsafe fn make_heap(first: E, last: E) {
    let len = last.offset_from(first); if len < 2 { return; }
    let mut parent = (len - 2) / 2;
    loop {
        let mut v = MaybeUninit::<Entry>::uninit(); let v = v.as_mut_ptr();
        mc(v, at(first, parent));
        let mut p = MaybeUninit::<Entry>::uninit(); let p = p.as_mut_ptr();
        mc(p, v); adjust_heap(first, parent, len, p); ds(p);
        ds(v);
        if parent == 0 { return; }
        parent -= 1;
    }
}
unsafe fn pop_heap(first: E, last: E, result: E) {
    let mut v = MaybeUninit::<Entry>::uninit(); let v = v.as_mut_ptr();
    mc(v, result); ma(result, first);
    let mut p = MaybeUninit::<Entry>::uninit(); let p = p.as_mut_ptr();
    mc(p, v); adjust_heap(first, 0, last.offset_from(first), p); ds(p);
    ds(v);
}
unsafe fn sort_heap(first: E, mut last: E) { while last.offset_from(first) > 1 { last = last.sub(1); pop_heap(first, last, last); } }
unsafe fn introsort_loop(first: E, mut last: E, mut depth: isize) {
    while last.offset_from(first) > 16 {
        if depth == 0 { make_heap(first, last); sort_heap(first, last); return; }
        depth -= 1;
        let mid = first.offset(last.offset_from(first) / 2);
        move_median_to_first(first, first.add(1), mid, last.sub(1));
        let cut = unguarded_partition(first.add(1), last, first);
        introsort_loop(cut, last, depth);
        last = cut;
    }
}
unsafe fn unguarded_linear_insert(mut last: E) {
    let mut v = MaybeUninit::<Entry>::uninit(); let v = v.as_mut_ptr();
    mc(v, last);
    let mut next = last.sub(1);
    while lt(v, next) { ma(last, next); last = next; next = next.sub(1); }
    ma(last, v); ds(v);
}
unsafe fn insertion_sort(first: E, last: E) {
    if first == last { return; }
    let mut i = first.add(1);
    while i != last {
        if lt(i, first) {
            let mut v = MaybeUninit::<Entry>::uninit(); let v = v.as_mut_ptr();
            mc(v, i);
            let mut s = i; let mut d = i.add(1);
            while s != first { s = s.sub(1); d = d.sub(1); ma(d, s); }
            ma(first, v); ds(v);
        } else { unguarded_linear_insert(i); }
        i = i.add(1);
    }
}
unsafe fn std_sort(first: E, last: E) {
    if first == last { return; }
    let n = last.offset_from(first) as u64;
    introsort_loop(first, last, (63 - n.leading_zeros() as isize) * 2);
    if last.offset_from(first) > 16 {
        insertion_sort(first, first.add(16));
        let mut i = first.add(16); while i != last { unguarded_linear_insert(i); i = i.add(1); }
    } else { insertion_sort(first, last); }
}
struct Files(RawVec);
impl Drop for Files { fn drop(&mut self) { unsafe { let mut e = self.0.start; while e != self.0.finish { ds(e.cast()); e = e.add(ESZ); } vec_dealloc(&self.0); } } }
struct TmpDrop(E);
impl Drop for TmpDrop { fn drop(&mut self) { unsafe { ds(self.0) } } }
struct NewStorage(*mut u8, usize, bool);
impl Drop for NewStorage { fn drop(&mut self) { if self.2 { unsafe { op_delete(self.0, self.1 * ESZ) } } } }
unsafe fn push_back(v: &mut RawVec, tmp: E) {
    if v.finish != v.eos { mc(v.finish.cast(), tmp); v.finish = v.finish.add(ESZ); return; }
    let size = (v.finish as usize - v.start as usize) / ESZ;
    if VMAX - size < 1 { throw_length_error(b"vector::_M_realloc_append\0".as_ptr()) }
    let mut n = size + if size > 1 { size } else { 1 }; if n < size || n > VMAX { n = VMAX; }
    let ns = op_new(n * ESZ);
    let g = NewStorage(ns, n, false);
    mc(ns.add(size * ESZ).cast(), tmp);
    let mut src = v.start; let mut dst = ns;
    while src != v.finish { mc(dst.cast(), src.cast()); ds(src.cast()); src = src.add(ESZ); dst = dst.add(ESZ); }
    drop(g);
    vec_dealloc(v);
    v.start = ns; v.finish = dst.add(ESZ); v.eos = ns.add(n * ESZ);
}
type Sink = unsafe extern "C-unwind" fn(*mut u8, u64, u64, *const u8, usize);
#[export_name = "wphw_disc_sort"]
pub unsafe extern "C-unwind" fn wphw_disc_sort(offsets: *const u64, sizes: *const u64, paths: *const *const u8, lengths: *const usize, n: usize, sink: Sink, c: *mut u8) {
    let mut files = Files(RawVec { start: std::ptr::null_mut(), finish: std::ptr::null_mut(), eos: std::ptr::null_mut() });
    for i in 0..n {
        let mut t = MaybeUninit::<Entry>::uninit(); let t = t.as_mut_ptr();
        (*t).offset = *offsets.add(i); (*t).size = *sizes.add(i);
        str_construct(sp(t), *paths.add(i), *lengths.add(i));
        let _g = TmpDrop(t);
        push_back(&mut files.0, t);
    }
    std_sort(files.0.start.cast(), files.0.finish.cast());
    let mut e = files.0.start;
    while e != files.0.finish { let f: E = e.cast(); sink(c, (*f).offset, (*f).size, (*f).path.p, (*f).path.len); e = e.add(ESZ); }
}
