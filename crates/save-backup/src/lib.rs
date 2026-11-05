// port of engine/src/platform/save_backup.cpp

use std::cmp::Ordering;
use std::ffi::{c_char, c_int, c_void};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

const K_STAMP_PATTERN: &[u8; 19] = b"0000-00-00_00-00-00";
const K_STAMP_LENGTH: usize = 19;
const K_MAXIMUM_SUFFIX_DIGITS: usize = 6;
const K_CHUNK_SIZE: usize = 64 * 1024;
const K_TITLE_TYPE: &str = "00010000";
const K_PARTIAL: &str = ".partial";
#[cfg(windows)]
const K_SYMLINK_REPARSE_TAG: u32 = 0xA000_000C;
#[cfg(windows)]
const K_JUNCTION_REPARSE_TAG: u32 = 0xA000_0003;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Result {
    Created = 0,
    Unchanged = 1,
    NoSave = 2,
    Invalid = 3,
    Failed = 4,
}

#[derive(Clone, Debug)]
pub struct Backup {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Outcome {
    pub result: Result,
    pub message: String,
    pub backup: PathBuf,
}

#[derive(Clone, Debug)]
struct Entry {
    relative: PathBuf,
    directory: bool,
    size: u64,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        path_eq(&self.relative, &other.relative)
            && self.directory == other.directory
            && self.size == other.size
    }
}

impl Eq for Entry {}

fn path_eq(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str()
            .encode_wide()
            .eq(right.as_os_str().encode_wide())
    }
    #[cfg(not(windows))]
    {
        left.as_os_str() == right.as_os_str()
    }
}

fn path_cmp(left: &Path, right: &Path) -> Ordering {
    #[cfg(windows)]
    {
        left.as_os_str()
            .encode_wide()
            .cmp(right.as_os_str().encode_wide())
    }
    #[cfg(not(windows))]
    {
        left.as_os_str().cmp(right.as_os_str())
    }
}

#[cfg(windows)]
fn is_symlink(path: &Path) -> bool {
    platform::reparse_tag(path) == Some(K_SYMLINK_REPARSE_TAG)
}

#[cfg(not(windows))]
fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_junction(path: &Path) -> bool {
    platform::reparse_tag(path) == Some(K_JUNCTION_REPARSE_TAG)
}

#[cfg(not(windows))]
fn is_junction(_path: &Path) -> bool {
    false
}

#[cfg(windows)]
fn is_symlink_directory(path: &Path) -> bool {
    use std::os::windows::fs::FileTypeExt;
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink_dir())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_symlink_directory(_path: &Path) -> bool {
    false
}

fn make_outcome(result: Result, message: String) -> Outcome {
    Outcome {
        result,
        message,
        backup: PathBuf::new(),
    }
}

// port of save_backup.cpp ascii
fn ascii(path: &Path) -> String {
    #[cfg(windows)]
    {
        path.as_os_str()
            .encode_wide()
            .map(|unit| {
                if unit < 0x80 {
                    (unit as u8) as char
                } else {
                    '?'
                }
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        path.as_os_str()
            .to_string_lossy()
            .chars()
            .map(|ch| if ch.is_ascii() { ch } else { '?' })
            .collect()
    }
}

// port of save_backup.cpp text
fn text(path: &Path) -> String {
    match path.to_str() {
        Some(value) => value.to_string(),
        None => ascii(path),
    }
}

// port of save_backup.cpp stamp_name
fn stamp_name(name: &str) -> Option<u64> {
    let bytes = name.as_bytes();
    if bytes.len() < K_STAMP_LENGTH {
        return None;
    }
    for index in 0..K_STAMP_LENGTH {
        let character = bytes[index];
        if K_STAMP_PATTERN[index] == b'0' {
            if !character.is_ascii_digit() {
                return None;
            }
        } else if character != K_STAMP_PATTERN[index] {
            return None;
        }
    }
    if bytes.len() == K_STAMP_LENGTH {
        return Some(1);
    }
    if bytes[K_STAMP_LENGTH] != b'_'
        || bytes.len() == K_STAMP_LENGTH + 1
        || bytes.len() > K_STAMP_LENGTH + 1 + K_MAXIMUM_SUFFIX_DIGITS
    {
        return None;
    }
    let mut suffix: u64 = 0;
    for &character in &bytes[K_STAMP_LENGTH + 1..] {
        if !character.is_ascii_digit() {
            return None;
        }
        suffix = suffix * 10 + (character - b'0') as u64;
    }
    Some(suffix)
}

#[cfg(windows)]
mod platform {
    use super::*;

    #[repr(C)]
    pub struct Tm {
        pub tm_sec: c_int,
        pub tm_min: c_int,
        pub tm_hour: c_int,
        pub tm_mday: c_int,
        pub tm_mon: c_int,
        pub tm_year: c_int,
        pub tm_wday: c_int,
        pub tm_yday: c_int,
        pub tm_isdst: c_int,
    }

    extern "C" {
        pub fn localtime(timer: *const i64) -> *mut Tm;
        pub fn strftime(
            output: *mut c_char,
            maximum: usize,
            format: *const c_char,
            time: *const Tm,
        ) -> usize;
    }

    #[repr(C)]
    pub struct FileTime {
        pub low: u32,
        pub high: u32,
    }

    #[repr(C)]
    pub struct Win32FindDataW {
        pub file_attributes: u32,
        pub creation_time: FileTime,
        pub last_access_time: FileTime,
        pub last_write_time: FileTime,
        pub file_size_high: u32,
        pub file_size_low: u32,
        pub reserved0: u32,
        pub reserved1: u32,
        pub file_name: [u16; 260],
        pub alternate_file_name: [u16; 14],
    }

    extern "system" {
        pub fn FormatMessageW(
            flags: u32,
            source: *const c_void,
            message_id: u32,
            language_id: u32,
            buffer: *mut u16,
            size: u32,
            arguments: *const *const c_void,
        ) -> u32;
        pub fn FindFirstFileW(
            file_name: *const u16,
            find_data: *mut Win32FindDataW,
        ) -> *mut c_void;
        pub fn FindClose(find_file: *mut c_void) -> i32;
    }

    pub fn reparse_tag(path: &Path) -> Option<u32> {
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);
        let mut data: Win32FindDataW = unsafe { std::mem::zeroed() };
        let handle = unsafe { FindFirstFileW(wide.as_ptr(), &mut data) };
        if handle as isize == -1 {
            return None;
        }
        let tag = if (data.file_attributes & 0x0000_0400) != 0 {
            Some(data.reserved0)
        } else {
            None
        };
        unsafe {
            FindClose(handle);
        }
        tag
    }
}

// port of save_backup.cpp stamp
fn stamp(now: i64) -> String {
    #[cfg(windows)]
    {
        let pointer = unsafe { platform::localtime(&now) };
        if pointer.is_null() {
            return "0000-00-00_00-00-00".to_string();
        }
        let mut buffer = [0i8; 32];
        let count = unsafe {
            platform::strftime(
                buffer.as_mut_ptr(),
                buffer.len(),
                b"%Y-%m-%d_%H-%M-%S\0".as_ptr() as *const c_char,
                pointer,
            )
        };
        if count == 0 {
            return "0000-00-00_00-00-00".to_string();
        }
        let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, count) };
        String::from_utf8_lossy(bytes).into_owned()
    }
    #[cfg(not(windows))]
    {
        let _ = now;
        "0000-00-00_00-00-00".to_string()
    }
}

// port of save_backup.cpp error_message
fn error_message(error: &io::Error) -> String {
    #[cfg(windows)]
    {
        if let Some(code) = error.raw_os_error() {
            // MinGW's std::filesystem reports many Windows failures through the
            // generic errno category; match the messages observed from the
            // pristine C++ build for the codes reachable here.
            let generic = match code {
                2 | 3 => Some("Input/output error"),
                5 | 145 => Some("Permission denied"),
                _ => None,
            };
            if let Some(message) = generic {
                return message.to_string();
            }

            let mut buffer = [0u16; 1024];
            let count = unsafe {
                platform::FormatMessageW(
                    0x0000_1000 | 0x0000_0200,
                    std::ptr::null(),
                    code as u32,
                    0,
                    buffer.as_mut_ptr(),
                    buffer.len() as u32,
                    std::ptr::null(),
                )
            };
            if count > 0 {
                let mut value = String::from_utf16_lossy(&buffer[..count as usize]);
                while value.ends_with('\r') || value.ends_with('\n') {
                    value.pop();
                }
                return value;
            }
        }
    }
    error.to_string()
}

// port of save_backup.cpp create_directories error handling
fn create_directories_error(path: &Path, error: &io::Error) -> String {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.is_dir() {
            return "Not a directory".to_string();
        }
    }
    let mut parent = path.parent();
    while let Some(value) = parent {
        if let Ok(metadata) = fs::symlink_metadata(value) {
            if !metadata.is_dir() {
                return "Not a directory".to_string();
            }
        }
        parent = value.parent();
    }
    error_message(error)
}

// port of save_backup.cpp entries
fn entries(root: &Path, out: &mut Vec<Entry>) -> bool {
    match fs::metadata(root) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return false;
            }
        }
        Err(_) => return false,
    }

    fn walk(root: &Path, directory: &Path, out: &mut Vec<Entry>) -> io::Result<()> {
        for item in fs::read_dir(directory)? {
            let item = item?;
            let path = item.path();
            let metadata = fs::metadata(&path)?;
            let relative = match path.strip_prefix(root) {
                Ok(relative) => relative.to_path_buf(),
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "path is not under root",
                    ))
                }
            };
            let is_directory = metadata.is_dir();
            // libstdc++ follows a directory symlink for is_directory(), but
            // file_size() reports the link itself (0) for a file symlink.
            let size = if is_directory {
                0
            } else if is_symlink(&path) {
                fs::symlink_metadata(&path)
                    .map(|link| link.len())
                    .unwrap_or(0)
            } else {
                metadata.len()
            };
            out.push(Entry {
                relative,
                directory: is_directory,
                size,
            });
            // libstdc++ on Windows follows a true directory symlink here (the
            // same way it follows a junction), so match that rather than
            // treating the symlink itself as a leaf.
            if is_directory {
                walk(root, &path, out)?;
            }
        }
        Ok(())
    }

    if walk(root, root, out).is_err() {
        return false;
    }
    out.sort_by(|left, right| path_cmp(&left.relative, &right.relative));
    true
}

// port of save_backup.cpp files_equal
fn read_chunk(file: &mut File, buffer: &mut [u8]) -> io::Result<(usize, bool)> {
    let mut count = 0;
    while count < buffer.len() {
        match file.read(&mut buffer[count..]) {
            Ok(0) => return Ok((count, true)),
            Ok(amount) => count += amount,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok((count, false))
}

// port of save_backup.cpp files_equal
fn files_equal(first: &Path, second: &Path) -> bool {
    let mut first_file = match File::open(first) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut second_file = match File::open(second) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut first_buffer = vec![0u8; K_CHUNK_SIZE];
    let mut second_buffer = vec![0u8; K_CHUNK_SIZE];
    loop {
        let (first_count, first_eof) = match read_chunk(&mut first_file, &mut first_buffer) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let (second_count, second_eof) = match read_chunk(&mut second_file, &mut second_buffer) {
            Ok(value) => value,
            Err(_) => return false,
        };
        if first_count != second_count
            || first_buffer[..first_count] != second_buffer[..second_count]
        {
            return false;
        }
        if first_count == 0 || first_eof || second_eof {
            return first_eof == second_eof;
        }
    }
}

// port of save_backup.cpp copy_tree
fn copy_tree(from: &Path, to: &Path) -> bool {
    if let Some(parent) = to.parent() {
        if fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    copy_recursive(from, to).is_ok()
}

fn copy_recursive(from: &Path, to: &Path) -> io::Result<()> {
    let metadata = fs::metadata(from)?;
    if metadata.is_dir() {
        fs::create_dir_all(to)?;
        for item in fs::read_dir(from)? {
            let item = item?;
            copy_recursive(&item.path(), &to.join(item.file_name()))?;
        }
    } else {
        fs::copy(from, to)?;
    }
    Ok(())
}

// port of save_backup.cpp remove_all_path
fn remove_all_path(path: &Path) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return,
    };
    if metadata.is_dir() {
        remove_directory_tree(path);
    } else if metadata.file_type().is_symlink() {
        let junction = is_junction(path);
        let directory_target = !is_symlink(path)
            && fs::metadata(path)
                .map(|followed| followed.is_dir())
                .unwrap_or(false);
        if junction && directory_target {
            // libstdc++'s remove_all sees a junction as a directory: it empties
            // the target through the junction, then removes the junction itself.
            remove_directory_tree(path);
        } else if junction {
            let _ = fs::remove_dir(path);
        } else if is_symlink_directory(path) {
            // libstdc++'s remove_all treats a true directory symlink like a
            // junction on Windows: empty the directory reached through the
            // link, then unlink the link itself.
            remove_directory_tree(path);
            let _ = fs::remove_dir(path);
        } else {
            let _ = fs::remove_file(path);
        }
    } else {
        let _ = fs::remove_file(path);
    }
}

fn remove_directory_tree(path: &Path) {
    if let Ok(items) = fs::read_dir(path) {
        for item in items.flatten() {
            remove_all_path(&item.path());
        }
    }
    let _ = fs::remove_dir(path);
}

// port of save_backup.cpp remove_partials
fn remove_partials(backups_root: &Path) {
    let items = match fs::read_dir(backups_root) {
        Ok(items) => items,
        Err(_) => return,
    };
    let suffix_length = K_PARTIAL.len();
    let mut stale: Vec<PathBuf> = Vec::new();
    for item in items.flatten() {
        let name = ascii(&PathBuf::from(item.file_name()));
        if name.len() > suffix_length
            && name.ends_with(K_PARTIAL)
            && stamp_name(&name[..name.len() - suffix_length]).is_some()
        {
            stale.push(item.path());
        }
    }
    for path in stale {
        remove_all_path(&path);
    }
}

// port of save_backup.cpp unique_name
fn unique_name(backups_root: &Path, base: &str) -> String {
    let mut name = base.to_string();
    let mut number = 2;
    while backups_root.join(&name).exists()
        || backups_root
            .join(format!("{}{}", name, K_PARTIAL))
            .exists()
    {
        name = format!("{}_{}", base, number);
        number += 1;
    }
    name
}

// port of save_backup.cpp save_relative
pub fn save_relative(title_low: u32) -> PathBuf {
    PathBuf::from("title")
        .join(K_TITLE_TYPE)
        .join(format!("{:08x}", title_low))
        .join("data")
}

// port of save_backup.cpp backups_beside
pub fn backups_beside(nand_root: &Path) -> PathBuf {
    fn has_filename(path: &Path) -> bool {
        let value = path.as_os_str().to_string_lossy();
        let drive_only = value.len() == 2
            && value.as_bytes()[0].is_ascii_alphabetic()
            && value.as_bytes()[1] == b':';
        !value.is_empty()
            && !value.ends_with('/')
            && !value.ends_with('\\')
            && !drive_only
    }

    fn parent_path(path: &Path) -> PathBuf {
        // std::filesystem::path::parent_path() returns the root itself for a
        // root path; Rust's Path::parent() returns None there.
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    }

    let mut root = nand_root.to_path_buf();
    if !has_filename(&root) {
        root = parent_path(&root);
    }
    parent_path(&root).join("backups")
}

// port of save_backup.cpp saved_relative
pub fn saved_relative(backup: &Backup) -> PathBuf {
    let titles = backup.path.join("title").join(K_TITLE_TYPE);
    let mut names: Vec<PathBuf> = Vec::new();
    if let Ok(items) = fs::read_dir(&titles) {
        for item in items.flatten() {
            let path = item.path();
            let has_data = fs::metadata(path.join("data"))
                .map(|metadata| metadata.is_dir())
                .unwrap_or(false);
            if has_data {
                names.push(PathBuf::from(item.file_name()));
            }
        }
    }
    if names.is_empty() {
        return PathBuf::new();
    }
    names.sort_by(|left, right| path_cmp(left, right));
    PathBuf::from("title")
        .join(K_TITLE_TYPE)
        .join(&names[0])
        .join("data")
}

// port of save_backup.cpp list
pub fn list(backups_root: &Path) -> Vec<Backup> {
    let mut found: Vec<(String, u64, Backup)> = Vec::new();
    if let Ok(items) = fs::read_dir(backups_root) {
        for item in items.flatten() {
            let name = ascii(&PathBuf::from(item.file_name()));
            if let Some(suffix) = stamp_name(&name) {
                let path = item.path();
                let is_directory = fs::metadata(&path)
                    .map(|metadata| metadata.is_dir())
                    .unwrap_or(false);
                if is_directory {
                    found.push((
                        name[..K_STAMP_LENGTH].to_string(),
                        suffix,
                        Backup {
                            name: name.clone(),
                            path,
                        },
                    ));
                }
            }
        }
    }
    found.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
    });
    found
        .into_iter()
        .map(|(_, _, backup)| backup)
        .collect()
}

// port of save_backup.cpp validate
pub fn validate(save: &Path, problem: &mut String) -> bool {
    let mut files = Vec::new();
    if !entries(save, &mut files) {
        *problem = format!("cannot list {}", text(save));
        return false;
    }
    for entry in &files {
        if entry.directory {
            continue;
        }
        let path = save.join(&entry.relative);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                *problem = format!("{} cannot be opened", text(&entry.relative));
                return false;
            }
        };
        let mut buffer = vec![0u8; K_CHUNK_SIZE];
        let mut count: u64 = 0;
        let mut bad = false;
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(amount) => count += amount as u64,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    bad = true;
                    break;
                }
            }
        }
        if bad || count != entry.size {
            *problem = format!(
                "{} gave {} of its {} bytes",
                text(&entry.relative),
                count,
                entry.size
            );
            return false;
        }
    }
    true
}

// port of save_backup.cpp same_contents
pub fn same_contents(first: &Path, second: &Path) -> bool {
    let mut first_entries = Vec::new();
    let mut second_entries = Vec::new();
    if !entries(first, &mut first_entries) || !entries(second, &mut second_entries) {
        return false;
    }
    if first_entries != second_entries {
        return false;
    }
    for entry in &first_entries {
        if entry.directory {
            continue;
        }
        if !files_equal(&first.join(&entry.relative), &second.join(&entry.relative)) {
            return false;
        }
    }
    true
}

// port of save_backup.cpp back_up_unguarded
fn back_up_unguarded(
    nand_root: &Path,
    relative: &Path,
    backups_root: &Path,
    keep: i32,
    now: i64,
) -> Outcome {
    let save = nand_root.join(relative);
    if !fs::metadata(&save)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
    {
        return make_outcome(Result::NoSave, format!("no save folder at {}", text(&save)));
    }

    let mut problem = String::new();
    if !validate(&save, &mut problem) {
        return make_outcome(
            Result::Invalid,
            format!("the save in {} failed the check: {}", text(&save), problem),
        );
    }

    let backups = list(backups_root);
    if let Some(front) = backups.first() {
        if same_contents(&save, &front.path.join(relative)) {
            rotate(backups_root, keep);
            let mut outcome = make_outcome(
                Result::Unchanged,
                format!("the save matches the newest backup {}", front.name),
            );
            outcome.backup = front.path.clone();
            return outcome;
        }
    }

    if let Err(error) = fs::create_dir_all(backups_root) {
        return make_outcome(
            Result::Failed,
            format!(
                "cannot create {}: {}",
                text(backups_root),
                create_directories_error(backups_root, &error)
            ),
        );
    }

    remove_partials(backups_root);

    let name = unique_name(backups_root, &stamp(now));
    let partial = backups_root.join(format!("{}{}", name, K_PARTIAL));
    let target = backups_root.join(&name);

    if !copy_tree(&save, &partial.join(relative))
        || !same_contents(&save, &partial.join(relative))
    {
        remove_all_path(&partial);
        return make_outcome(
            Result::Failed,
            format!("cannot copy the save to {}", text(&partial)),
        );
    }

    if let Err(error) = fs::rename(&partial, &target) {
        let message = format!("cannot rename {}: {}", text(&partial), error_message(&error));
        remove_all_path(&partial);
        return make_outcome(Result::Failed, message);
    }

    rotate(backups_root, keep);
    let mut outcome = make_outcome(
        Result::Created,
        format!("backed up the save to {}", text(&target)),
    );
    outcome.backup = target;
    outcome
}

// port of save_backup.cpp back_up
pub fn back_up(
    nand_root: &Path,
    relative: &Path,
    backups_root: &Path,
    keep: i32,
    now: i64,
) -> Outcome {
    back_up_unguarded(nand_root, relative, backups_root, keep, now)
}

// port of save_backup.cpp restore_unguarded
fn restore_unguarded(
    nand_root: &Path,
    backups_root: &Path,
    backup: &Backup,
    keep: i32,
    now: i64,
) -> Outcome {
    let relative = saved_relative(backup);
    if relative.as_os_str().is_empty() {
        return make_outcome(
            Result::Failed,
            format!("the backup {} holds no save", backup.name),
        );
    }

    let source = backup.path.join(&relative);
    let mut problem = String::new();
    if !validate(&source, &mut problem) {
        return make_outcome(
            Result::Invalid,
            format!(
                "the backup {} failed the check: {}",
                backup.name, problem
            ),
        );
    }

    let live = nand_root.join(&relative);
    let present = live.exists();
    if present && same_contents(&live, &source) {
        return make_outcome(
            Result::Unchanged,
            format!("the save already matches the backup {}", backup.name),
        );
    }

    let mut kept: Option<PathBuf> = None;
    if present {
        let safety = back_up_unguarded(nand_root, &relative, backups_root, 0, now);
        if safety.result == Result::Invalid {
            let damaged = format!("damaged-{}", stamp(now));
            kept = Some(
                backups_root
                    .join(unique_name(backups_root, &damaged))
                    .join(&relative),
            );
        } else if safety.result != Result::Created && safety.result != Result::Unchanged {
            return make_outcome(
                Result::Failed,
                format!(
                    "the current save could not be backed up first: {}",
                    safety.message
                ),
            );
        }
    }

    let mut staging_os = live.as_os_str().to_os_string();
    staging_os.push(".restoring");
    let staging = PathBuf::from(staging_os);
    remove_all_path(&staging);
    if !copy_tree(&source, &staging) || !same_contents(&source, &staging) {
        remove_all_path(&staging);
        return make_outcome(
            Result::Failed,
            format!(
                "cannot copy the backup {} into {}",
                backup.name,
                text(&staging)
            ),
        );
    }

    let old = if let Some(value) = &kept {
        value.clone()
    } else {
        let mut old_os = live.as_os_str().to_os_string();
        old_os.push(".replaced");
        PathBuf::from(old_os)
    };

    if present {
        if kept.is_none() {
            remove_all_path(&old);
        }
        if let Some(parent) = old.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(error) = fs::rename(&live, &old) {
            let message = format!(
                "cannot move the current save aside: {}",
                error_message(&error)
            );
            remove_all_path(&staging);
            return make_outcome(Result::Failed, message);
        }
    }

    if let Err(error) = fs::rename(&staging, &live) {
        let message = format!("cannot put the backup in place: {}", error_message(&error));
        if present {
            let _ = fs::rename(&old, &live);
        }
        remove_all_path(&staging);
        return make_outcome(Result::Failed, message);
    }

    if present && kept.is_none() {
        remove_all_path(&old);
    }

    rotate(backups_root, keep);

    let mut message = format!("restored the backup {}", backup.name);
    if let Some(value) = &kept {
        message.push_str(&format!(
            "; the damaged save was moved to {}",
            text(value)
        ));
    }
    make_outcome(Result::Created, message)
}

// port of save_backup.cpp restore
pub fn restore(
    nand_root: &Path,
    backups_root: &Path,
    backup: &Backup,
    keep: i32,
    now: i64,
) -> Outcome {
    restore_unguarded(nand_root, backups_root, backup, keep, now)
}

// port of save_backup.cpp rotate
pub fn rotate(backups_root: &Path, keep: i32) {
    if keep <= 0 {
        return;
    }
    let backups = list(backups_root);
    let keep = keep as usize;
    for index in keep..backups.len() {
        remove_all_path(&backups[index].path);
    }
}

// port of save_backup.cpp adopt_root_files
pub fn adopt_root_files(nand_root: &Path, relative: &Path) -> Vec<String> {
    let mut moved = Vec::new();
    let target = nand_root.join(relative);
    let items = match fs::read_dir(nand_root) {
        Ok(items) => items,
        Err(_) => return moved,
    };
    for item in items.flatten() {
        let path = item.path();
        let is_regular = fs::metadata(&path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false);
        if !is_regular {
            continue;
        }
        let destination = target.join(item.file_name());
        if destination.exists() {
            continue;
        }
        let _ = fs::create_dir_all(&target);
        if fs::rename(&path, &destination).is_ok() {
            moved.push(item.file_name().to_string_lossy().into_owned());
        }
    }
    moved
}

pub mod c_api;
