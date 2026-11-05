// AI-assisted integration adapter for the prepared source port. NOT INTEGRATED.
// Windows native paths cross as counted UTF-16, preserving unpaired surrogates.

use crate::{
    adopt_root_files as rust_adopt_root_files, back_up as rust_back_up,
    backups_beside as rust_backups_beside, list as rust_list, restore as rust_restore,
    rotate as rust_rotate, same_contents as rust_same_contents, save_relative as rust_save_relative,
    saved_relative as rust_saved_relative, validate as rust_validate, Backup, Outcome,
};
use std::any::Any;
use std::ffi::{c_char, c_int, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::ptr;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

#[repr(C)] pub struct WpSavesPath { pub count:usize, pub items:*mut u16 }
fn path_to_c(path:&Path)->WpSavesPath {
 let units:Box<[u16]>=path.as_os_str().encode_wide().collect::<Vec<_>>().into_boxed_slice();
 let count=units.len(); let items=Box::into_raw(units) as *mut u16; WpSavesPath{count,items}
}
#[no_mangle] pub unsafe extern "C" fn wpsaves_free_path(value:*mut WpSavesPath){
 if value.is_null(){return} let p=&mut *value;
 if !p.items.is_null(){drop(Box::from_raw(ptr::slice_from_raw_parts_mut(p.items,p.count)));}
 p.items=ptr::null_mut();p.count=0;
}

// catch_unwind cannot stop a panic compiled with panic=abort. Refuse to build
// the C ABI static library in that configuration rather than silently letting a
// Rust panic terminate the host process across the C boundary.
#[cfg(panic = "abort")]
compile_error!(
    "wprt_save_backup C ABI requires panic=unwind; remove panic=abort from the \
     integrated profile or the catch_unwind guards are ineffective"
);

#[repr(C)]
pub struct WpSavesOutcome {
    pub result: i32,
    pub message: *mut c_char,
    pub backup: WpSavesPath,
}

#[repr(C)]
pub struct WpSavesBackup {
    pub name: *mut c_char,
    pub path: WpSavesPath,
}

#[repr(C)]
pub struct WpSavesList {
    pub count: usize,
    pub items: *mut WpSavesBackup,
}

#[repr(C)]
pub struct WpSavesStrings {
    pub count: usize,
    pub items: *mut *mut c_char,
}

fn c_string(value: &str) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

unsafe fn input_string(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }
    CStr::from_ptr(value).to_string_lossy().into_owned()
}

unsafe fn input_path(value: *const WpSavesPath) -> PathBuf {
 if value.is_null(){return PathBuf::new()}
 let p=&*value;let units=if p.count==0{&[]}else{std::slice::from_raw_parts(p.items,p.count)};
 PathBuf::from(OsString::from_wide(units))
}

fn outcome_to_c(outcome: Outcome) -> WpSavesOutcome {
    WpSavesOutcome {
        result: outcome.result as i32,
        message: c_string(&outcome.message),
        backup: path_to_c(&outcome.backup),
    }
}

fn list_to_c(backups: Vec<Backup>) -> WpSavesList {
    let mut items = Vec::with_capacity(backups.len());
    for backup in &backups {
        items.push(WpSavesBackup {
            name: c_string(&backup.name),
            path: path_to_c(&backup.path),
        });
    }
    if items.is_empty() {
        return WpSavesList {
            count: 0,
            items: ptr::null_mut(),
        };
    }
    let mut boxed = items.into_boxed_slice();
    let count = boxed.len();
    let pointer = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    WpSavesList {
        count,
        items: pointer,
    }
}

fn strings_to_c(values: Vec<String>) -> WpSavesStrings {
    let mut items: Vec<*mut c_char> = values.iter().map(|value| c_string(value)).collect();
    if items.is_empty() {
        return WpSavesStrings {
            count: 0,
            items: ptr::null_mut(),
        };
    }
    let mut items = items.into_boxed_slice();
    let count = items.len();
    let pointer = items.as_mut_ptr();
    std::mem::forget(items);
    WpSavesStrings {
        count,
        items: pointer,
    }
}

fn empty_list() -> WpSavesList {
    WpSavesList {
        count: 0,
        items: ptr::null_mut(),
    }
}

fn empty_strings() -> WpSavesStrings {
    WpSavesStrings {
        count: 0,
        items: ptr::null_mut(),
    }
}

fn panic_text(payload: Box<dyn Any + Send>) -> String {
    if let Some(value) = payload.downcast_ref::<&str>() {
        (*value).to_string()
    } else if let Some(value) = payload.downcast_ref::<String>() {
        value.clone()
    } else {
        "Rust panic".to_string()
    }
}

fn failed_outcome(payload: Box<dyn Any + Send>) -> WpSavesOutcome {
    WpSavesOutcome {
        result: crate::Result::Failed as i32,
        message: c_string(&panic_text(payload)),
        backup: path_to_c(Path::new("")),
    }
}

#[no_mangle]
pub extern "C" fn wpsaves_save_relative(title_low: u32) -> WpSavesPath {
    let value = catch_unwind(AssertUnwindSafe(|| {
        rust_save_relative(title_low)
    }))
    .unwrap_or_default();
    path_to_c(&value)
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_backups_beside(nand_root: *const WpSavesPath) -> WpSavesPath {
    let root = input_path(nand_root);
    let value = catch_unwind(AssertUnwindSafe(|| {
        rust_backups_beside(&root)
    }))
    .unwrap_or_default();
    path_to_c(&value)
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_saved_relative(
    backup_name: *const c_char,
    backup_path: *const WpSavesPath,
) -> WpSavesPath {
    let backup = Backup {
        name: input_string(backup_name),
        path: input_path(backup_path),
    };
    let value = catch_unwind(AssertUnwindSafe(|| {
        rust_saved_relative(&backup)
    }))
    .unwrap_or_default();
    path_to_c(&value)
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_list(backups_root: *const WpSavesPath) -> WpSavesList {
    let root = input_path(backups_root);
    catch_unwind(AssertUnwindSafe(|| list_to_c(rust_list(&root)))).unwrap_or_else(|_| empty_list())
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_validate(
    save: *const WpSavesPath,
    problem: *mut *mut c_char,
) -> c_int {
    let save = input_path(save);
    let (valid, text) = catch_unwind(AssertUnwindSafe(|| {
        let mut text = String::new();
        let valid = rust_validate(&save, &mut text);
        (valid, text)
    }))
    .unwrap_or_else(|payload| (false, panic_text(payload)));
    if !problem.is_null() {
        *problem = c_string(&text);
    }
    if valid {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_same_contents(
    first: *const WpSavesPath,
    second: *const WpSavesPath,
) -> c_int {
    let first = input_path(first);
    let second = input_path(second);
    if catch_unwind(AssertUnwindSafe(|| {
        rust_same_contents(&first, &second)
    }))
    .unwrap_or(false)
    {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_back_up(
    nand_root: *const WpSavesPath,
    relative: *const WpSavesPath,
    backups_root: *const WpSavesPath,
    keep: i32,
    now: i64,
) -> WpSavesOutcome {
    let nand_root = input_path(nand_root);
    let relative = input_path(relative);
    let backups_root = input_path(backups_root);
    catch_unwind(AssertUnwindSafe(|| {
        outcome_to_c(rust_back_up(
            &nand_root,
            &relative,
            &backups_root,
            keep,
            now,
        ))
    }))
    .unwrap_or_else(failed_outcome)
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_restore(
    nand_root: *const WpSavesPath,
    backups_root: *const WpSavesPath,
    backup_name: *const c_char,
    backup_path: *const WpSavesPath,
    keep: i32,
    now: i64,
) -> WpSavesOutcome {
    let backup = Backup {
        name: input_string(backup_name),
        path: input_path(backup_path),
    };
    let nand_root = input_path(nand_root);
    let backups_root = input_path(backups_root);
    catch_unwind(AssertUnwindSafe(|| {
        outcome_to_c(rust_restore(
            &nand_root,
            &backups_root,
            &backup,
            keep,
            now,
        ))
    }))
    .unwrap_or_else(failed_outcome)
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_rotate(backups_root: *const WpSavesPath, keep: i32) {
    let backups_root = input_path(backups_root);
    let _ = catch_unwind(AssertUnwindSafe(|| {
        rust_rotate(&backups_root, keep);
    }));
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_adopt_root_files(
    nand_root: *const WpSavesPath,
    relative: *const WpSavesPath,
) -> WpSavesStrings {
    let nand_root = input_path(nand_root);
    let relative = input_path(relative);
    catch_unwind(AssertUnwindSafe(|| {
        strings_to_c(rust_adopt_root_files(&nand_root, &relative))
    }))
    .unwrap_or_else(|_| empty_strings())
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_free_outcome(value: *mut WpSavesOutcome) {
    if value.is_null() {
        return;
    }
    let outcome = &mut *value;
    wpsaves_free_string(outcome.message);
    wpsaves_free_path(&mut outcome.backup);
    outcome.message = ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_free_list(value: *mut WpSavesList) {
    if value.is_null() {
        return;
    }
    let list = &mut *value;
    if !list.items.is_null() {
        let items = std::slice::from_raw_parts_mut(list.items, list.count);
        for item in items.iter_mut() {
            wpsaves_free_string(item.name);
            wpsaves_free_path(&mut item.path);
        }
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            list.items,
            list.count,
        )));
    }
    list.count = 0;
    list.items = ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn wpsaves_free_strings(value: *mut WpSavesStrings) {
    if value.is_null() {
        return;
    }
    let strings = &mut *value;
    if !strings.items.is_null() {
        let items = std::slice::from_raw_parts_mut(strings.items, strings.count);
        for item in items.iter_mut() {
            wpsaves_free_string(*item);
        }
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            strings.items,
            strings.count,
        )));
    }
    strings.count = 0;
    strings.items = ptr::null_mut();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_fallback_is_a_failed_outcome() {
        let mut outcome = catch_unwind(AssertUnwindSafe(|| -> WpSavesOutcome {
            panic!("boom");
        }))
        .unwrap_or_else(failed_outcome);
        assert_eq!(outcome.result, crate::Result::Failed as i32);
        let message = unsafe { CStr::from_ptr(outcome.message) }
            .to_string_lossy()
            .into_owned();
        assert_eq!(message, "boom");
        unsafe { wpsaves_free_outcome(&mut outcome) };
        assert!(outcome.message.is_null());
        assert!(outcome.backup.is_null());
    }

    #[test]
    fn empty_fallbacks_are_safe() {
        let list = empty_list();
        assert_eq!(list.count, 0);
        assert!(list.items.is_null());
        let strings = empty_strings();
        assert_eq!(strings.count, 0);
        assert!(strings.items.is_null());
    }
}
