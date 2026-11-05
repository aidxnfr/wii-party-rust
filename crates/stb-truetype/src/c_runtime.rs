use std;
use std::alloc;
use std::mem;

pub trait One {
    fn one() -> Self;
}

impl One for i32 {
    fn one() -> Self {
        1
    }
}

impl One for u32 {
    fn one() -> Self {
        1
    }
}

impl One for u8 {
    fn one() -> Self {
        1
    }
}

impl One for u16 {
    fn one() -> Self {
        1
    }
}

pub unsafe fn postInc<T: std::ops::AddAssign + One + Copy>(mut a: *mut T) -> T {
    let mut result: T = *a;
    *a += One::one();
    return result;
}

pub unsafe fn preInc<T: std::ops::AddAssign + One + Copy>(mut a: *mut T) -> T {
    *a += One::one();
    return *a;
}

pub unsafe fn postDec<T: std::ops::SubAssign + One + Copy>(mut a: *mut T) -> T {
    let mut result: T = *a;
    *a -= One::one();
    return result;
}

pub unsafe fn preDec<T: std::ops::SubAssign + One + Copy>(mut a: *mut T) -> T {
    *a -= One::one();
    return *a;
}

pub unsafe fn preIncPtr<T>(mut a: *mut *mut T) -> *mut T {
    *a = (*a).offset(1);
    return *a;
}

pub unsafe fn preDecPtr<T>(mut a: *mut *mut T) -> *mut T {
    *a = (*a).offset(-1);
    return *a;
}

pub unsafe fn postIncPtr<T>(mut a: *mut *mut T) -> *mut T {
    let mut result: *mut T = *a;
    *a = (*a).offset(1);
    return result;
}

pub unsafe fn postDecPtr<T>(mut a: *mut *mut T) -> *mut T {
    let mut result: *mut T = *a;
    *a = (*a).offset(-1);
    return result;
}

pub unsafe fn preIncConstPtr<T>(mut a: *mut *const T) -> *const T {
    *a = (*a).offset(1);
    return *a;
}

pub unsafe fn preDecConstPtr<T>(mut a: *mut *const T) -> *const T {
    *a = (*a).offset(-1);
    return *a;
}

pub unsafe fn postIncConstPtr<T>(mut a: *mut *const T) -> *const T {
    let mut result: *const T = *a;
    *a = (*a).offset(1);
    return result;
}

pub unsafe fn postDecConstPtr<T>(mut a: *mut *const T) -> *const T {
    let mut result: *const T = *a;
    *a = (*a).offset(-1);
    return result;
}

pub unsafe fn memcpy(src: *mut u8, dest: *mut u8, count: u64) {
    std::ptr::copy_nonoverlapping(dest, src, count as usize);
}

pub unsafe fn memset(src: *mut u8, value: i32, count: u64) {
    std::ptr::write_bytes(src, value as u8, count as usize);
}

// Exact CRT allocation family used by pristine stb_truetype.h defaults.
extern "C" {
 #[link_name="malloc"] fn crt_malloc(n:usize)->*mut u8;
 #[link_name="realloc"] fn crt_realloc(p:*mut u8,n:usize)->*mut u8;
 #[link_name="free"] fn crt_free(p:*mut u8);
 #[link_name="sqrt"] fn crt_sqrt(v:f64)->f64;
}
pub unsafe fn malloc(n:u64)->*mut u8 {crt_malloc(n as usize)}
pub unsafe fn realloc<T>(p:*mut T,n:u64)->*mut u8 {crt_realloc(p.cast(),n as usize)}
pub unsafe fn free<T>(p:*mut T) {crt_free(p.cast())}
// C sqrt accepts a promoted double; source casts at the destination.
pub fn sqrt64(x:f32)->f64 {unsafe {crt_sqrt(x as f64)}}
pub fn _lrotl(x: u32, y: i32) -> u32 {
    return (x << y) | (x >> (32 - y));
}

pub fn abs(x: i32) -> i32 {
    return i32::abs(x);
}

pub fn pow(x: f32, p: f32) -> f32 {
    return x.powf(p);
}

pub fn fabs(x: f32) -> f32 {
    return f32::abs(x);
}

pub fn fmod(x: f32, y: f32) -> f32 {
    return x % y;
}

pub unsafe fn strlen(str: *mut u8) -> i32 {
    let mut ptr = str;
    let mut result = 0;

    while *ptr != 0 {
        ptr = ptr.offset(1);
        result += 1;
    }

    return result;
}

pub fn sqrt(x: f32) -> f32 {
    return sqrt64(x) as f32;
}

pub fn acos(x: f32) -> f32 {
    return f32::acos(x);
}

pub fn cos(x: f32) -> f32 {
    return f32::cos(x);
}

pub fn floor(x: f32) -> f32 {
    return f32::floor(x);
}

pub fn ceil(x: f32) -> f32 {
    return f32::ceil(x);
}
