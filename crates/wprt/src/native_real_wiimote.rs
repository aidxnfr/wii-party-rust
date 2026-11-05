// AI-assisted port of pristine engine/src/input/real_wiimote.cpp. PRIVATE UNBUILT/UNINTEGRATED.
// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(non_upper_case_globals)]
const kNintendo: u16 = 0x057e;
const kRemote: u16 = 0x0306;
const kRemotePlus: u16 = 0x0330;
const kSlots: u32 = 4;
const kLongestReport: usize = 22;
const kQueueLimit: usize = 256;
const kHidDataInput: u8 = 0xA1;
const kStatusRequest: u8 = 0x15;
const kButtonMask: u16 = 0x9F1F;
const kReportIrEnable: u8 = 0x13;
const kReportWriteMemory: u8 = 0x16;
const kCameraModeAddress: u32 = 0xB00033;
const kRegisterSpace: u8 = 0x04;
use std::{
    collections::{BTreeSet, VecDeque},
    ffi::{c_void, CStr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, Once, OnceLock,
    },
};
type Sender = unsafe extern "C" fn(*mut c_void, *const u8, usize);
extern "C" {
    fn wphw_real_enabled() -> bool;
    fn wphw_real_mouse() -> bool;
    fn wphw_real_hid_init() -> i32;
    fn wphw_real_enumerate(vendor: u16, product: u16) -> *mut c_void;
    fn wphw_real_next(info: *mut c_void) -> *mut c_void;
    fn wphw_real_product(info: *mut c_void) -> u16;
    fn wphw_real_path(info: *mut c_void) -> *const std::ffi::c_char;
    fn wphw_real_free(info: *mut c_void);
    fn wphw_real_open(path: *const std::ffi::c_char) -> usize;
    fn wphw_real_close(d: usize);
    fn wphw_real_write(d: usize, p: *const u8, n: usize) -> i32;
    fn wphw_real_read(d: usize, p: *mut u8, n: usize, timeout: i32) -> i32;
    fn wphw_real_change_count() -> u32;
    fn wphw_real_now() -> i64;
    fn wphw_real_after_scan(now: i64) -> i64;
    fn wphw_real_sleep();
    fn wphw_real_log(kind: u32, slot: u32, product: u16, path: *const u8, n: usize);
    fn wphw_real_camera_objects(mode: u8, out: *mut u8);
}
// port of real_wiimote.cpp Remote/g_remotes/g_started. Source sequentially consistent atomics and one detached polling worker retained.
#[derive(Default)]
struct RemoteData {
    device: usize,
    input: VecDeque<Vec<u8>>,
    output: VecDeque<Vec<u8>>,
    camera_on: bool,
    camera_mode: u8,
}
struct Remote {
    // Source path is read/written only by the single polling worker; no added lock on open_new reads.
    path: std::cell::UnsafeCell<Vec<u8>>,
    mutex: Mutex<RemoteData>,
    present: AtomicBool,
    woke: AtomicBool,
}
// SAFETY: only run -> open_new/close_remote accesses path; guest callbacks use locked queues/atomics only.
unsafe impl Sync for Remote {}
impl Remote {
    fn new() -> Self {
        Self {
            path: std::cell::UnsafeCell::new(Vec::new()),
            mutex: Mutex::new(RemoteData::default()),
            present: AtomicBool::new(false),
            woke: AtomicBool::new(false),
        }
    }
}
static REMOTES: OnceLock<[Remote; 4]> = OnceLock::new();
static STARTED: Once = Once::new();
fn remotes() -> &'static [Remote; 4] {
    REMOTES.get_or_init(|| std::array::from_fn(|_| Remote::new()))
}
// port of real_wiimote.cpp payload_size.
fn payload_size(id: u8) -> usize {
    match id {
        0x20 => 6,
        0x22 => 4,
        0x30 => 2,
        0x31 => 5,
        0x32 => 10,
        0x33 => 14,
        0x21 | 0x34 | 0x35 | 0x36 | 0x37 | 0x3d | 0x3e | 0x3f => 21,
        _ => 0,
    }
}
// port of real_wiimote.cpp pressed; report buttons are big endian, 0x3d has no buttons.
fn pressed(p: &[u8]) -> bool {
    if p.len() < 3 || p[0] == 0x3d {
        return false;
    }
    ((p[1] as u16) << 8 | p[2] as u16) & kButtonMask != 0
}
// port of real_wiimote.cpp close_remote. Source intentionally keeps camera_on/camera_mode after disconnect.
unsafe fn close_remote(slot: u32) {
    let remote = &remotes()[slot as usize];
    let mut d = remote.mutex.lock().unwrap();
    if d.device != 0 {
        wphw_real_close(d.device);
    }
    wphw_real_log(0, slot, 0, std::ptr::null(), 0);
    d.device = 0;
    (*remote.path.get()).clear();
    d.input.clear();
    d.output.clear();
    remote.woke.store(false, Ordering::SeqCst);
    remote.present.store(false, Ordering::SeqCst);
}
// port of real_wiimote.cpp open_new. Enumeration object/path pointers remain native-owned until free_enumeration.
unsafe fn open_new(refused: &mut BTreeSet<Vec<u8>>) {
    let list = wphw_real_enumerate(kNintendo, 0);
    let mut info = list;
    while !info.is_null() {
        let product = wphw_real_product(info);
        let pathp = wphw_real_path(info);
        if (product == kRemote || product == kRemotePlus) && !pathp.is_null() {
            let path = CStr::from_ptr(pathp).to_bytes().to_vec();
            let mut known = false;
            let mut free_slot = kSlots;
            for i in 0..kSlots {
                let r = &remotes()[i as usize];
                if r.present.load(Ordering::SeqCst) && *r.path.get() == path {
                    known = true;
                }
                if !r.present.load(Ordering::SeqCst) && free_slot == kSlots {
                    free_slot = i;
                }
            }
            if !known && free_slot != kSlots {
                let device = wphw_real_open(pathp);
                if device != 0 {
                    let probe = [kStatusRequest, 0];
                    if wphw_real_write(device, probe.as_ptr(), probe.len()) < 0 {
                        wphw_real_close(device);
                        if refused.insert(path.clone()) {
                            wphw_real_log(1, 0, 0, path.as_ptr(), path.len());
                        }
                    } else {
                        let mut buffer = [0u8; kLongestReport];
                        while wphw_real_read(device, buffer.as_mut_ptr(), buffer.len(), 50) > 0 {}
                        refused.remove(&path);
                        let remote = &remotes()[free_slot as usize];
                        let mut d = remote.mutex.lock().unwrap();
                        d.device = device;
                        *remote.path.get() = path;
                        d.input.clear();
                        d.output.clear();
                        remote.woke.store(true, Ordering::SeqCst);
                        remote.present.store(true, Ordering::SeqCst);
                        wphw_real_log(2, free_slot, product, std::ptr::null(), 0);
                    }
                }
            }
        }
        info = wphw_real_next(info);
    }
    wphw_real_free(list);
}
// port of real_wiimote.cpp service. Queue swap under mutex; no mutex is held across HID I/O/close_remote.
unsafe fn service(slot: u32) -> bool {
    let remote = &remotes()[slot as usize];
    if !remote.present.load(Ordering::SeqCst) {
        return false;
    }
    let (pending, device) = {
        let mut d = remote.mutex.lock().unwrap();
        (std::mem::take(&mut d.output), d.device)
    };
    for p in &pending {
        if wphw_real_write(device, p.as_ptr(), p.len()) < 0 {
            close_remote(slot);
            return false;
        }
    }
    let mut busy = !pending.is_empty();
    let mut buffer = [0u8; kLongestReport];
    loop {
        let count = wphw_real_read(device, buffer.as_mut_ptr(), buffer.len(), 0);
        if count < 0 {
            close_remote(slot);
            return false;
        }
        if count == 0 {
            break;
        }
        busy = true;
        let n = payload_size(buffer[0]);
        if n == 0 || (count as usize) < n + 1 {
            continue;
        }
        let mut frame = Vec::with_capacity(n + 2);
        frame.push(kHidDataInput);
        frame.extend_from_slice(&buffer[..n + 1]);
        if pressed(&buffer[..n + 1]) {
            remote.woke.store(true, Ordering::SeqCst);
        }
        let mut d = remote.mutex.lock().unwrap();
        d.input.push_back(frame);
        if d.input.len() > kQueueLimit {
            d.input.pop_front();
        }
    }
    busy
}
// port of real_wiimote.cpp aim_with_mouse. Only all-FF IR data is replaced, using original Wii Remote camera_objects and sample(0).
unsafe fn aim_with_mouse(frame: &mut [u8], mode: u8) {
    let (offset, n) = match frame[1] {
        0x33 => (7, 12),
        0x36 => (4, 10),
        0x37 => (7, 10),
        _ => return,
    };
    if frame.len() < offset + n {
        return;
    }
    for i in 0..n {
        if frame[offset + i] != 0xff {
            return;
        }
    }
    let mut objects = [0u8; 36];
    wphw_real_camera_objects(mode, objects.as_mut_ptr());
    frame[offset..offset + n].copy_from_slice(&objects[..n]);
}
// port of real_wiimote.cpp run. Every slot is serviced even when an earlier slot is busy; clock is original host steady clock, not guest comparison time.
unsafe fn run() {
    let mut refused = BTreeSet::new();
    let mut next_scan = wphw_real_now();
    let mut changes = 0;
    loop {
        let now = wphw_real_now();
        let count = wphw_real_change_count();
        if now >= next_scan || count != changes {
            changes = count;
            open_new(&mut refused);
            next_scan = wphw_real_after_scan(now);
        }
        let mut busy = false;
        for i in 0..kSlots {
            busy = service(i) || busy;
        }
        if !busy {
            wphw_real_sleep();
        }
    }
}
// port of real_wiimote.cpp start. call_once consumes the attempt even if disabled or HID initialization fails.
#[no_mangle]
pub unsafe extern "C" fn wprt_real_start() {
    STARTED.call_once(|| {
        if !wphw_real_enabled() || wphw_real_hid_init() != 0 {
            return;
        }
        std::thread::spawn(|| unsafe { run() });
    });
}
// port of real_wiimote.cpp present/take_wake/reset.
#[no_mangle]
pub extern "C" fn wprt_real_present(slot: u32) -> bool {
    slot < kSlots && remotes()[slot as usize].present.load(Ordering::SeqCst)
}
#[no_mangle]
pub extern "C" fn wprt_real_take_wake(slot: u32) -> bool {
    slot < kSlots && remotes()[slot as usize].woke.swap(false, Ordering::SeqCst)
}
#[no_mangle]
pub extern "C" fn wprt_real_reset(slot: u32) {
    if slot >= kSlots {
        return;
    }
    remotes()[slot as usize].mutex.lock().unwrap().input.clear();
}
// port of real_wiimote.cpp output_report. Preserve register range/length tests before updating camera mode; enqueue the original complete report.
#[no_mangle]
pub unsafe extern "C" fn wprt_real_output_report(slot: u32, data: *const u8, size: u32) {
    if slot >= kSlots || size == 0 {
        return;
    }
    let remote = &remotes()[slot as usize];
    let mut d = remote.mutex.lock().unwrap();
    if !remote.present.load(Ordering::SeqCst) {
        return;
    }
    let data = std::slice::from_raw_parts(data, size as usize);
    if data[0] == kReportIrEnable && size >= 2 {
        d.camera_on = data[1] & 4 != 0;
    } else if data[0] == kReportWriteMemory && size >= 7 && data[1] & kRegisterSpace != 0 {
        let address = (data[2] as u32) << 16 | (data[3] as u32) << 8 | data[4] as u32;
        let length = data[5] as u32;
        if address <= kCameraModeAddress
            && kCameraModeAddress < address.wrapping_add(length)
            && 6 + kCameraModeAddress - address < size
        {
            d.camera_mode = data[(6 + kCameraModeAddress - address) as usize];
        }
    }
    d.output.push_back(data.to_vec());
}
// port of real_wiimote.cpp update. Callback scope is outside the queue lock; source slot-zero-only mouse substitution retained.
#[no_mangle]
pub unsafe extern "C" fn wprt_real_update(slot: u32, send: Sender, context: *mut c_void) {
    if slot >= kSlots {
        return;
    }
    let (reports, camera_on, camera_mode) = {
        let mut d = remotes()[slot as usize].mutex.lock().unwrap();
        (std::mem::take(&mut d.input), d.camera_on, d.camera_mode)
    };
    let substitute = slot == 0 && camera_on && wphw_real_mouse();
    for mut report in reports {
        if substitute {
            aim_with_mouse(&mut report, camera_mode);
        }
        send(context, report.as_ptr(), report.len());
    }
}
