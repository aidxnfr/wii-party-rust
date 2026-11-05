// SPDX-License-Identifier: GPL-3.0-or-later
// Private H7 source candidate. Port of engine/src/platform/mii.cpp record/text/CRC/image functions.
// Integrated source;runtime gates tracked in HANDOFF. Original native class/filesystem/random_device/mt19937 remain native.
#![allow(non_upper_case_globals)]
use std::ffi::c_void;
type Emit = unsafe extern "C" fn(*mut c_void, *const u8, usize);
const kSlots: usize = 100;
const kRecordSize: usize = 0x4A;
const kHeaderSize: usize = 4;
const kChecksumOffset: usize = 0x1F1DE;
const kDatabaseSize: usize = 0x1F1E0;
const kFileSize: usize = 0xBE6C0;
const kNameLength: usize = 10;
const kTailOffset: usize = 0x1CEC;
const kNameOffset: usize = 0x02;
const kCreatorOffset: usize = 0x36;
const kIdOffset: usize = 0x18;
const kSystemIdOffset: usize = 0x1C;
const TAIL: &[u8] = &[
    0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x52, 0x4E, 0x48, 0x44, 0xFF,
    0xFF, 0xFF, 0xFF,
];
// Field/default tables extracted programmatically from the pristine source, not transcribed.
const FIELDS: &[(&[u8], usize, u8, u8, u8, u8, u8)] = &[
    (b"gender", 0x00, 2, 14, 1, 1, 0),
    (b"birth_month", 0x00, 2, 10, 4, 12, 0),
    (b"birth_day", 0x00, 2, 5, 5, 31, 0),
    (b"favorite_color", 0x00, 2, 1, 4, 11, 0),
    (b"favorite", 0x00, 2, 0, 1, 1, 0),
    (b"height", 0x16, 1, 0, 7, 127, 0),
    (b"weight", 0x17, 1, 0, 7, 127, 0),
    (b"face_shape", 0x20, 2, 13, 3, 7, 0),
    (b"skin_color", 0x20, 2, 10, 3, 5, 0),
    (b"facial_feature", 0x20, 2, 6, 4, 11, 0),
    (b"hair_type", 0x22, 2, 9, 7, 71, 0),
    (b"hair_color", 0x22, 2, 6, 3, 7, 0),
    (b"hair_flip", 0x22, 2, 5, 1, 1, 0),
    (b"eyebrow_type", 0x24, 4, 27, 5, 23, 0),
    (b"eyebrow_rotation", 0x24, 4, 22, 4, 11, 0),
    (b"eyebrow_color", 0x24, 4, 13, 3, 7, 0),
    (b"eyebrow_size", 0x24, 4, 9, 4, 8, 0),
    (b"eyebrow_vertical", 0x24, 4, 4, 5, 18, 3),
    (b"eyebrow_spacing", 0x24, 4, 0, 4, 12, 0),
    (b"eye_type", 0x28, 4, 26, 6, 47, 0),
    (b"eye_rotation", 0x28, 4, 21, 3, 7, 0),
    (b"eye_vertical", 0x28, 4, 16, 5, 18, 0),
    (b"eye_color", 0x28, 4, 13, 3, 5, 0),
    (b"eye_size", 0x28, 4, 9, 3, 7, 0),
    (b"eye_spacing", 0x28, 4, 5, 4, 12, 0),
    (b"nose_type", 0x2C, 2, 12, 4, 11, 0),
    (b"nose_size", 0x2C, 2, 8, 4, 8, 0),
    (b"nose_vertical", 0x2C, 2, 3, 5, 18, 0),
    (b"mouth_type", 0x2E, 2, 11, 5, 23, 0),
    (b"mouth_color", 0x2E, 2, 9, 2, 2, 0),
    (b"mouth_size", 0x2E, 2, 5, 4, 8, 0),
    (b"mouth_vertical", 0x2E, 2, 0, 5, 18, 0),
    (b"glasses_type", 0x30, 2, 12, 4, 8, 0),
    (b"glasses_color", 0x30, 2, 9, 3, 5, 0),
    (b"glasses_size", 0x30, 2, 5, 3, 7, 0),
    (b"glasses_vertical", 0x30, 2, 0, 5, 20, 0),
    (b"mustache_type", 0x32, 2, 14, 2, 3, 0),
    (b"beard_type", 0x32, 2, 12, 2, 3, 0),
    (b"facial_hair_color", 0x32, 2, 9, 3, 7, 0),
    (b"mustache_size", 0x32, 2, 5, 4, 8, 0),
    (b"mustache_vertical", 0x32, 2, 0, 5, 16, 0),
    (b"mole", 0x34, 2, 15, 1, 1, 0),
    (b"mole_size", 0x34, 2, 11, 4, 8, 0),
    (b"mole_vertical", 0x34, 2, 6, 5, 30, 0),
    (b"mole_horizontal", 0x34, 2, 1, 5, 16, 0),
];
const DEFAULTS: &[(&[u8], u32)] = &[
    (b"height", 63),
    (b"weight", 63),
    (b"hair_type", 33),
    (b"hair_color", 1),
    (b"eyebrow_type", 6),
    (b"eyebrow_rotation", 6),
    (b"eyebrow_color", 1),
    (b"eyebrow_size", 4),
    (b"eyebrow_vertical", 10),
    (b"eyebrow_spacing", 2),
    (b"eye_type", 2),
    (b"eye_rotation", 4),
    (b"eye_vertical", 12),
    (b"eye_size", 4),
    (b"eye_spacing", 2),
    (b"nose_type", 1),
    (b"nose_size", 4),
    (b"nose_vertical", 9),
    (b"mouth_type", 23),
    (b"mouth_size", 4),
    (b"mouth_vertical", 13),
    (b"glasses_size", 4),
    (b"glasses_vertical", 10),
    (b"mustache_size", 4),
    (b"mustache_vertical", 10),
    (b"mole_size", 4),
    (b"mole_vertical", 20),
    (b"mole_horizontal", 2),
];
unsafe fn bytes<'a>(p: *const u8, n: usize) -> &'a [u8] {
    if n == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(p, n)
    }
}
unsafe fn record_mut<'a>(p: *mut u8) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(p, kRecordSize)
}
// port of mii.cpp to_utf16: source intentionally accepts malformed sequences without validation.
fn to_utf16(text: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let lead = text[i];
        let length = if lead < 0x80 {
            1
        } else if lead >> 5 == 6 {
            2
        } else if lead >> 4 == 14 {
            3
        } else if lead >> 3 == 30 {
            4
        } else {
            1
        };
        let mut value = if length == 1 {
            lead as u32
        } else {
            (lead & (0x7f >> length)) as u32
        };
        for k in 1..length {
            if i + k >= text.len() {
                break;
            }
            value = (value << 6) | (text[i + k] & 0x3f) as u32;
        }
        i += length;
        if value >= 0x10000 {
            value -= 0x10000;
            out.push((0xd800 + (value >> 10)) as u16);
            out.push((0xdc00 + (value & 0x3ff)) as u16);
        } else {
            out.push(value as u16)
        }
    }
    out
}
// port of mii.cpp to_utf8: source combines any following unit after a high surrogate.
fn to_utf8(text: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let mut value = text[i] as u32;
        if value >= 0xd800 && value < 0xdc00 && i + 1 < text.len() {
            i += 1;
            value = 0x10000u32
                .wrapping_add((value - 0xd800) << 10)
                .wrapping_add((text[i] as u32).wrapping_sub(0xdc00));
        }
        if value < 0x80 {
            out.push(value as u8)
        } else if value < 0x800 {
            out.push((0xc0 | (value >> 6)) as u8);
            out.push((0x80 | (value & 0x3f)) as u8)
        } else if value < 0x10000 {
            out.push((0xe0 | (value >> 12)) as u8);
            out.push((0x80 | ((value >> 6) & 0x3f)) as u8);
            out.push((0x80 | (value & 0x3f)) as u8)
        } else {
            out.push((0xf0 | (value >> 18)) as u8);
            out.push((0x80 | ((value >> 12) & 0x3f)) as u8);
            out.push((0x80 | ((value >> 6) & 0x3f)) as u8);
            out.push((0x80 | (value & 0x3f)) as u8)
        }
        i += 1;
    }
    out
}
// port of mii.cpp get/set, with the native Field supplied as individual scalars.
fn get(record: &[u8], offset: usize, n: u8, shift: u8, bits: u8) -> u32 {
    let mut value = 0u32;
    for i in 0..n as usize {
        value = (value << 8) | record[offset + i] as u32;
    }
    (value >> shift) & ((1u32 << bits) - 1)
}
fn set(
    record: &mut [u8],
    offset: usize,
    n: u8,
    shift: u8,
    bits: u8,
    maximum: u8,
    minimum: u8,
    input: u32,
) {
    let mut value = 0u32;
    for i in 0..n as usize {
        value = (value << 8) | record[offset + i] as u32;
    }
    let mask = ((1u32 << bits) - 1) << shift;
    let limit = (maximum as u32).min((1u32 << bits) - 1);
    let input = input.min(limit).max(minimum as u32);
    value = (value & !mask) | ((input << shift) & mask);
    for i in 0..n as usize {
        record[offset + i] = (value >> (8 * (n as usize - 1 - i))) as u8
    }
}
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_get(p: *const u8, o: usize, n: u8, s: u8, b: u8) -> u32 {
    get(bytes(p, kRecordSize), o, n, s, b)
}
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_set(
    p: *mut u8,
    o: usize,
    n: u8,
    s: u8,
    b: u8,
    max: u8,
    min: u8,
    v: u32,
) {
    set(record_mut(p), o, n, s, b, max, min, v)
}
// port of mii.cpp read_text/write_text; no replacement Unicode library semantics.
fn write_text(record: &mut [u8], offset: usize, text: &[u8]) {
    let mut text = to_utf16(text);
    text.truncate(kNameLength);
    for i in 0..kNameLength {
        let unit = text.get(i).copied().unwrap_or(0);
        record[offset + i * 2] = (unit >> 8) as u8;
        record[offset + i * 2 + 1] = unit as u8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_read_text(
    p: *const u8,
    offset: usize,
    c: *mut c_void,
    emit: Emit,
) {
    let p = bytes(p, kRecordSize);
    let mut text = Vec::new();
    for i in 0..kNameLength {
        let unit = ((p[offset + i * 2] as u16) << 8) | p[offset + i * 2 + 1] as u16;
        if unit == 0 {
            break;
        }
        text.push(unit);
    }
    let out = to_utf8(&text);
    emit(c, out.as_ptr(), out.len());
}
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_write_text(p: *mut u8, offset: usize, text: *const u8, n: usize) {
    write_text(record_mut(p), offset, bytes(text, n));
}
fn occupied(record: &[u8]) -> bool {
    for i in kIdOffset..kIdOffset + 4 {
        if record[i] != 0 {
            return true;
        }
    }
    for i in kNameOffset..kNameOffset + kNameLength * 2 {
        if record[i] != 0 {
            return true;
        }
    }
    false
}
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_occupied(p: *const u8) -> bool {
    occupied(bytes(p, kRecordSize))
}
// port of mii.cpp default_record; field lookups preserve the actual source table.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_default(p: *mut u8, text: *const u8, n: usize) {
    let record = record_mut(p);
    // Native blank_record() initializes at the source point before Rust borrows the output.
    write_text(record, kNameOffset, bytes(text, n));
    for &(key, value) in DEFAULTS {
        let &(_, o, n, s, b, max, min) = FIELDS.iter().find(|f| f.0 == key).unwrap();
        set(record, o, n, s, b, max, min, value);
    }
}
fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_crc(p: *const u8, n: usize) -> u16 {
    crc16(bytes(p, n))
}
// port of mii.cpp blank_image;native vector allocation/zero initialization retained.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_blank_image(p: *mut u8, n: usize) {
    // Original native vector(size,0) owns the allocation;only source header/tail writes live here.
    let image = std::slice::from_raw_parts_mut(p, n);
    image[..4].copy_from_slice(b"RNOD");
    image[kTailOffset..kTailOffset + TAIL.len()].copy_from_slice(TAIL);
}
// port of mii.cpp import_records: store-size branch has priority over raw record multiples.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_import(
    p: *const u8,
    n: usize,
    c: *mut c_void,
    push: unsafe extern "C" fn(*mut c_void, *const u8),
) {
    let data = bytes(p, n);
    let take = |o: usize| {
        let record = &data[o..o + kRecordSize];
        if occupied(record) {
            push(c, record.as_ptr());
        }
    };
    let store = kRecordSize + 2;
    if n >= kDatabaseSize && &data[..4] == b"RNOD" {
        for i in 0..kSlots {
            take(kHeaderSize + i * kRecordSize)
        }
    } else if n == kRecordSize || n == store {
        take(0)
    } else if n != 0 && n % store == 0 && n / store <= kSlots {
        for o in (0..n).step_by(store) {
            take(o)
        }
    } else if n != 0 && n % kRecordSize == 0 && n / kRecordSize <= kSlots {
        for o in (0..n).step_by(kRecordSize) {
            take(o)
        }
    }
}
// port of mii.cpp Database::parse. No native class layout guessed: member records are passed explicitly.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_parse(
    p: *const u8,
    n: usize,
    records: *mut u8,
    c: *mut c_void,
    prepare: unsafe extern "C" fn(*mut c_void) -> *const u8,
) -> bool {
    // End the input borrow before native operator= can replace/resize an aliased image_ vector.
    {
        let data = bytes(p, n);
        if n < kDatabaseSize || &data[..4] != b"RNOD" {
            return false;
        }
        let stored = u16::from_be_bytes([data[kChecksumOffset], data[kChecksumOffset + 1]]);
        if stored != crc16(&data[..kChecksumOffset]) {
            return false;
        }
    }
    let image = prepare(c);
    std::ptr::copy_nonoverlapping(image.add(kHeaderSize), records, kSlots * kRecordSize);
    true
}
// port of mii.cpp Database::bytes, original CRC update order preserved.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_bytes(p: *mut u8, n: usize, records: *const u8) {
    // Original native data=image_ allocation/copy already happened,as in the source.
    let image = std::slice::from_raw_parts_mut(p, n);
    image[kHeaderSize..kHeaderSize + kSlots * kRecordSize]
        .copy_from_slice(bytes(records, kSlots * kRecordSize));
    let crc = crc16(&image[..kChecksumOffset]);
    image[kChecksumOffset] = (crc >> 8) as u8;
    image[kChecksumOffset + 1] = crc as u8;
}

// port of mii.cpp Database::used (native vector<size_t> marshaling only).
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_used(
    records: *const u8,
    c: *mut c_void,
    push: unsafe extern "C" fn(*mut c_void, usize),
) {
    let records = bytes(records, kSlots * kRecordSize);
    for slot in 0..kSlots {
        if occupied(&records[slot * kRecordSize..(slot + 1) * kRecordSize]) {
            push(c, slot)
        }
    }
}
// port of mii.cpp Database::add prefix, before native random_device/mt19937 initialization.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_prepare_add(
    record: *mut u8,
    records: *const u8,
    list: *const usize,
    n: usize,
) {
    let mut system = [0x05, 0x03, 0x04, 0x05];
    if n != 0 {
        let slot = *list;
        let all = bytes(records, kSlots * kRecordSize);
        system.copy_from_slice(
            &all[slot * kRecordSize + kSystemIdOffset..slot * kRecordSize + kSystemIdOffset + 4],
        );
    }
    record_mut(record)[kSystemIdOffset..kSystemIdOffset + 4].copy_from_slice(&system);
}
// port of mii.cpp identifier_of + Database::add draw/uniqueness/slot order. Draw callback keeps original C++ RNG.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_add(
    record: *mut u8,
    records: *mut u8,
    list: *const usize,
    n: usize,
    c: *mut c_void,
    draw: unsafe extern "C" fn(*mut c_void) -> u32,
) -> usize {
    let list = if n == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(list, n)
    };
    let identifier = loop {
        let identifier = 0x80000000u32 | (draw(c) & 0x1FFFFFFFu32);
        let mut unique = true;
        {
            let all = bytes(records, kSlots * kRecordSize);
            for &slot in list {
                let o = slot * kRecordSize + kIdOffset;
                let id = u32::from_be_bytes([all[o], all[o + 1], all[o + 2], all[o + 3]]);
                if id == identifier {
                    unique = false;
                }
            }
        }
        if unique {
            break identifier;
        }
    };
    let record = record_mut(record);
    for i in 0..4 {
        record[kIdOffset + i] = (identifier >> (24 - 8 * i)) as u8;
    }
    let all = std::slice::from_raw_parts_mut(records, kSlots * kRecordSize);
    for slot in 0..kSlots {
        let target = &mut all[slot * kRecordSize..(slot + 1) * kRecordSize];
        if !occupied(target) {
            target.copy_from_slice(record);
            return slot;
        }
    }
    kSlots
}
// port of mii.cpp Database::remove bound check.
#[no_mangle]
pub unsafe extern "C" fn wprt_mii_remove(records: *mut u8, slot: usize) {
    if slot < kSlots {
        std::slice::from_raw_parts_mut(records.add(slot * kRecordSize), kRecordSize).fill(0);
    }
}


// Remaining mii.cpp load/save/MT19937 candidate. UNINTEGRATED.
// CRT stream details: GCC15.2 basic_file_stdio.cc:251-279,312-363;
// target os_defines.h:97 overrides _GLIBCXX_BUFSIZ=4096; fstream.tcc:328-483,760-804.
extern "C" {
    fn _wfopen(path:*const u16,mode:*const u16)->*mut c_void;
    fn _fileno(file:*mut c_void)->i32;
    fn _read(fd:i32,data:*mut c_void,n:u32)->i32;
    fn _write(fd:i32,data:*const c_void,n:u32)->i32;
    fn fclose(file:*mut c_void)->i32;
    fn _errno()->*mut i32;
    fn rand_s(out:*mut u32)->i32;
}
struct MiiFile(*mut c_void);
impl Drop for MiiFile{fn drop(&mut self){unsafe{fclose(self.0);}}}
unsafe fn mii_wide(path:*const u16,n:usize)->Vec<u16>{let mut v=if n==0{Vec::new()}else{std::slice::from_raw_parts(path,n).to_vec()};v.push(0);v}
type MiiImage=unsafe extern "C" fn(*mut c_void,*const u8,usize)->*const u8;
fn mii_blank()->Vec<u8>{let mut image=vec![0;kFileSize];unsafe{wprt_mii_blank_image(image.as_mut_ptr(),image.len())};image}
// port of mii.cpp Database::load. Members passed individually;native image callback only owns std::vector.
// Result -1 requests the original native ios_failure AFTER returning from Rust;errno is captured before close.
#[no_mangle]pub unsafe extern "C" fn wprt_mii_load_file(path:*const u16,n:usize,records:*mut u8,valid:*mut bool,existed:*mut bool,c:*mut c_void,image:MiiImage,error:*mut i32)->i32{
    records.write_bytes(0,kSlots*kRecordSize);
    let blank=mii_blank();image(c,blank.as_ptr(),blank.len());
    *valid=true;*existed=false;
    let path=mii_wide(path,n);let raw=_wfopen(path.as_ptr(),[b'r' as u16,b'b' as u16,0].as_ptr());
    if raw.is_null(){return 1}
    let file=MiiFile(raw);*existed=true;
    let fd=_fileno(file.0);let mut data=Vec::new();let mut buf=[0u8;4095];
    loop{
        let count=loop{let count=_read(fd,buf.as_mut_ptr().cast(),buf.len() as u32);if count != -1 || *_errno()!=4{break count}};
        if count<0{*error=*_errno();return -1}
        if count==0{break}
        data.extend_from_slice(&buf[..count as usize]);
    }
    *valid=data.len()>=kDatabaseSize && &data[..4]==b"RNOD" && u16::from_be_bytes([data[kChecksumOffset],data[kChecksumOffset+1]])==crc16(&data[..kChecksumOffset]);
    if *valid{
        let p=image(c,data.as_ptr(),data.len());
        std::ptr::copy_nonoverlapping(p.add(kHeaderSize),records,kSlots*kRecordSize);
    }else{
        let blank=mii_blank();image(c,blank.as_ptr(),blank.len());records.write_bytes(0,kSlots*kRecordSize);
    }
    // Keep file alive through parse/member updates,as the source ifstream does.
    if *valid{1}else{0}
}
#[repr(C)]pub struct MiiPath{pub p:*const u16,pub n:usize}
type MiiFs=unsafe extern "C" fn(*mut c_void,i32,*mut MiiPath)->i32;
// port of mii.cpp Database::save,with original filesystem/path API calls retained as opaque SDK leaves.
// data is Database::bytes() evaluated by the native API wrapper BEFORE any parent/temp-path work.
#[no_mangle]pub unsafe extern "C" fn wprt_mii_save_file(data:*const u8,n:usize,valid:*mut bool,existed:*mut bool,c:*mut c_void,fs:MiiFs)->bool{
    let mut temp=MiiPath{p:std::ptr::null(),n:0};
    if fs(c,0,&mut temp)<0{return false} // create_directories(parent,error),error deliberately ignored
    if fs(c,1,&mut temp)<0{return false} // temporary=path;temporary+=".tmp"
    let temporary=mii_wide(temp.p,temp.n);
    let wrote={
        let raw=_wfopen(temporary.as_ptr(),[b'w' as u16,b'b' as u16,0].as_ptr());
        if raw.is_null(){false}else{
            let file=MiiFile(raw);let fd=_fileno(file.0);let mut left=n;let mut at=data;
            loop{
                let ret=_write(fd,at.cast(),left as u32);
                if ret == -1 && *_errno()==4{continue}
                if ret == -1{break}
                left-=ret as usize;
                if left==0{break}
                at=at.add(ret as usize);
            }
            // Source checks stream write state BEFORE destructor/fclose;close errors are ignored.
            left==0
        }
    };
    if !wrote{return false}
    let rename=fs(c,2,&mut temp);if rename<0{return false}
    if rename!=0{fs(c,3,&mut temp);return false}
    *valid=true;*existed=true;true
}
// port of GCC15.2 random.cc __winxp_rand_s,_M_init default branch.
// Target os_defines.h:94 enables this branch BEFORE RDSEED/RDRAND;no invented RNG source.
#[no_mangle]pub unsafe extern "C" fn wprt_mii_entropy(out:*mut u32)->i32{rand_s(out)}
// port of GCC15.2 bits/random.tcc MT19937 seed,_M_gen_rand,operator();parameters random.h:1714-1717.
struct MiiMt{state:[u32;624],position:usize}
impl MiiMt{
    fn new(seed:u32)->Self{
        let mut out=Self{state:[0;624],position:624};out.state[0]=seed;
        for i in 1..624{let x=out.state[i-1]^(out.state[i-1]>>30);out.state[i]=x.wrapping_mul(1812433253).wrapping_add(i as u32)}out
    }
    fn next(&mut self)->u32{
        if self.position>=624{
            for k in 0..227{let y=(self.state[k]&0x80000000)|(self.state[k+1]&0x7fffffff);self.state[k]=self.state[k+397]^(y>>1)^if y&1!=0{0x9908b0df}else{0}}
            for k in 227..623{let y=(self.state[k]&0x80000000)|(self.state[k+1]&0x7fffffff);self.state[k]=self.state[k-227]^(y>>1)^if y&1!=0{0x9908b0df}else{0}}
            let y=(self.state[623]&0x80000000)|(self.state[0]&0x7fffffff);self.state[623]=self.state[396]^(y>>1)^if y&1!=0{0x9908b0df}else{0};self.position=0;
        }
        let mut z=self.state[self.position];self.position+=1;z^=z>>11;z^=(z<<7)&0x9d2c5680;z^=(z<<15)&0xefc60000;z^=z>>18;z
    }
}
// port of mii.cpp Database::add generator construction and draw boundary;prefix still precedes entropy in wrapper.
#[no_mangle]pub unsafe extern "C" fn wprt_mii_seeded_add(record:*mut u8,records:*mut u8,list:*const usize,n:usize,seed:u32)->usize{
    unsafe extern "C" fn draw(p:*mut c_void)->u32{(&mut *p.cast::<MiiMt>()).next()}
    let mut generator=MiiMt::new(seed);wprt_mii_add(record,records,list,n,(&mut generator as *mut MiiMt).cast(),draw)
}
