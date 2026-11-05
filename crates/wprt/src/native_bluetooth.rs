// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted 1:1 port of pristine engine/src/input/bluetooth.cpp. PRIVATE, UNBUILT/UNINTEGRATED.
// Protocol constants extracted from pristine source; H5 comparison clock preserved by native adapter.
#![allow(non_upper_case_globals)]
const kVectorSize: u32 = 8;
const kControlMessage: u32 = 0;
const kBulkMessage: u32 = 1;
const kInterruptMessage: u32 = 2;
const kEndpointEvents: u8 = 0x81;
const kEndpointAclIn: u8 = 0x82;
const kEndpointAclOut: u8 = 0x02;
const kEventConnectionComplete: u8 = 0x03;
const kEventConnectionRequest: u8 = 0x04;
const kEventDisconnectionComplete: u8 = 0x05;
const kReasonRemoteTerminated: u8 = 0x13;
const kEventAuthenticationComplete: u8 = 0x06;
const kEventRemoteNameComplete: u8 = 0x07;
const kEventRemoteFeaturesComplete: u8 = 0x0B;
const kEventRemoteVersionComplete: u8 = 0x0C;
const kEventCommandComplete: u8 = 0x0E;
const kEventCommandStatus: u8 = 0x0F;
const kEventRoleChange: u8 = 0x12;
const kEventCompletedPackets: u8 = 0x13;
const kEventModeChange: u8 = 0x14;
const kEventReturnLinkKeys: u8 = 0x15;
const kEventClockOffsetComplete: u8 = 0x1C;
const kEventPacketTypeChanged: u8 = 0x1D;
const kCommandInquiry: u16 = 0x0401;
const kCommandCreateConnection: u16 = 0x0405;
const kCommandDisconnect: u16 = 0x0406;
const kCommandAcceptConnection: u16 = 0x0409;
const kCommandLinkKeyReply: u16 = 0x040B;
const kCommandLinkKeyNegativeReply: u16 = 0x040C;
const kCommandChangePacketType: u16 = 0x040F;
const kCommandAuthenticationRequest: u16 = 0x0411;
const kCommandRemoteNameRequest: u16 = 0x0419;
const kCommandReadRemoteFeatures: u16 = 0x041B;
const kCommandReadRemoteVersion: u16 = 0x041D;
const kCommandReadClockOffset: u16 = 0x041F;
const kCommandSniffMode: u16 = 0x0803;
const kCommandWriteLinkPolicy: u16 = 0x080D;
const kCommandReset: u16 = 0x0C03;
const kCommandReadStoredLinkKey: u16 = 0x0C0D;
const kCommandDeleteStoredLinkKey: u16 = 0x0C12;
const kCommandWriteScanEnable: u16 = 0x0C1A;
const kCommandWriteLinkSupervisionTimeout: u16 = 0x0C37;
const kCommandReadLocalVersion: u16 = 0x1001;
const kCommandReadLocalFeatures: u16 = 0x1003;
const kCommandReadBufferSize: u16 = 0x1005;
const kCommandReadBdAddr: u16 = 0x1009;
const kAclPacketSize: u16 = 339;
const kAclPacketCount: u16 = 10;
const kScoPacketSize: u8 = 64;
const kScoPacketCount: u16 = 0;
const kPageScanEnable: u8 = 0x02;
const kSignalChannel: u16 = 0x0001;
const kSignalReject: u8 = 0x01;
const kSignalConnectRequest: u8 = 0x02;
const kSignalConnectResponse: u8 = 0x03;
const kSignalConfigRequest: u8 = 0x04;
const kSignalConfigResponse: u8 = 0x05;
const kSignalDisconnectRequest: u8 = 0x06;
const kSignalDisconnectResponse: u8 = 0x07;
const kOptionMtu: u8 = 0x01;
const kDefaultMtu: u16 = 672;
const kRequestMtu: u16 = 185;
const kPsmSdp: u16 = 0x0001;
const kPsmHidControl: u16 = 0x0011;
const kPsmHidInterrupt: u16 = 0x0013;
const kHidSetReportOutput: u8 = 0x52;
const kHidDataOutput: u8 = 0xA2;
const kHidHandshakeSuccess: u8 = 0x00;
const kWiimoteCount: u32 = 4;
const kControllerAddress: [u8; 6] = [0xff, 0x00, 0x79, 0x19, 0x02, 0x11];
const kWiimoteClass: [u8; 3] = [0x00, 0x04, 0x48];
const kWiimoteFeatures: [u8; 8] = [0xBC, 0x02, 0x04, 0x38, 0x08, 0x00, 0x00, 0x00];
const kDevicePath: &[u8] = b"/dev/usb/oh1/57e/305";
const kWiimoteName: &[u8] = b"Nintendo RVL-CNT-01";
use crate::memory::{host, rd16, rd32, rd8};
use std::{
    cell::UnsafeCell,
    collections::{BTreeMap, VecDeque},
};
extern "C" {
    fn wphw_bt_now() -> i64;
    fn wphw_bt_deadline(now: i64) -> i64;
    fn wphw_bt_log_enabled() -> bool;
    fn wphw_bt_log(kind: u32, a: u32, b: u32, c: u32, d: u32);
    fn wphw_bt_real_present(i: u32) -> bool;
    fn wphw_bt_real_wake(i: u32) -> bool;
    fn wphw_bt_input_connected(i: u32) -> bool;
    fn wphw_bt_input_wake(i: u32) -> bool;
    fn wphw_bt_real_start();
    fn wphw_bt_real_reset(i: u32);
    fn wphw_bt_wii_reset(i: u32);
    fn wphw_bt_output(i: u32, real: bool, data: *const u8, size: u32);
    fn wphw_bt_update(i: u32, real: bool);
}
#[derive(Default, Clone, Copy)]
struct Endpoint {
    request: u32,
    data: u32,
    length: u32,
}
struct Completion {
    request: u32,
    result: i32,
}
#[derive(Default, Clone, Copy)]
struct Channel {
    psm: u16,
    remote_cid: u16,
    remote_mtu: u16,
    configuration_sent: bool,
    complete: bool,
}
#[derive(Default, PartialEq, Clone, Copy)]
enum Baseband {
    #[default]
    Inactive,
    RequestConnection,
    Pending,
    Complete,
}
#[derive(Default)]
struct Wiimote {
    baseband: Baseband,
    real: bool,
    linking: bool,
    host_ready: bool,
    link_deadline: i64,
    channels: BTreeMap<u16, Channel>,
    completed_packets: u32,
}
// port of bluetooth.cpp globals. Original single guest-thread state; pointer access keeps mutable borrows out of native callback scopes.
struct Globals {
    events: Endpoint,
    acl_in: Endpoint,
    event_queue: VecDeque<Vec<u8>>,
    acl_queue: VecDeque<Vec<u8>>,
    completions: VecDeque<Completion>,
    wiimotes: [Wiimote; 4],
    scan_enable: u8,
    started: bool,
}
impl Globals {
    const fn new() -> Self {
        Self {
            events: Endpoint {
                request: 0,
                data: 0,
                length: 0,
            },
            acl_in: Endpoint {
                request: 0,
                data: 0,
                length: 0,
            },
            event_queue: VecDeque::new(),
            acl_queue: VecDeque::new(),
            completions: VecDeque::new(),
            wiimotes: [const {
                Wiimote {
                    baseband: Baseband::Inactive,
                    real: false,
                    linking: false,
                    host_ready: false,
                    link_deadline: 0,
                    channels: BTreeMap::new(),
                    completed_packets: 0,
                }
            }; 4],
            scan_enable: 0,
            started: false,
        }
    }
}
struct Global(UnsafeCell<Globals>);
unsafe impl Sync for Global {}
static G: Global = Global(UnsafeCell::new(Globals::new()));
unsafe fn remote(i: u32) -> *mut Wiimote {
    std::ptr::addr_of_mut!((*G.0.get()).wiimotes)
        .cast::<Wiimote>()
        .add(i as usize)
}
// port of bluetooth.cpp attached/in_vector/io_vector/io_vector_size.
unsafe fn attached(i: u32) -> bool {
    wphw_bt_real_present(i) || wphw_bt_input_connected(i)
}
fn in_vector(v: u32, i: u32) -> u32 {
    rd32(v.wrapping_add(i.wrapping_mul(kVectorSize)))
}
fn io_vector(v: u32, n: u32, i: u32) -> u32 {
    rd32(v.wrapping_add(n.wrapping_add(i).wrapping_mul(kVectorSize)))
}
fn io_vector_size(v: u32, n: u32, i: u32) -> u32 {
    rd32(
        v.wrapping_add(n.wrapping_add(i).wrapping_mul(kVectorSize))
            .wrapping_add(4),
    )
}
// port of bluetooth.cpp put16/get16; protocol byte order is little endian, unlike guest memory scalars.
fn put16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}
unsafe fn get16(p: *const u8) -> u16 {
    *p as u16 | ((*p.add(1) as u16) << 8)
}
// port of bluetooth.cpp wiimote_address/connection_handle/put_address/wiimote_from_address/wiimote_from_handle.
fn wiimote_address(i: u32) -> [u8; 6] {
    [0x11, 0x02, 0x19, 0x79, 0x00, i as u8]
}
fn connection_handle(i: u32) -> u16 {
    0x100u32.wrapping_add(i) as u16
}
fn put_address(out: &mut Vec<u8>, i: u32) {
    out.extend_from_slice(&wiimote_address(i));
}
fn wiimote_from_address(a: u32) -> i32 {
    let i = rd8(a.wrapping_add(5));
    let expected = wiimote_address(i);
    for k in 0..6 {
        if rd8(a.wrapping_add(k)) != expected[k as usize] as u32 {
            return -1;
        }
    }
    if i < kWiimoteCount {
        i as i32
    } else {
        -1
    }
}
fn wiimote_from_handle(h: u16) -> i32 {
    let i = (h & 0xff) as u32;
    if h & 0x0f00 == 0x100 && i < kWiimoteCount {
        i as i32
    } else {
        -1
    }
}
// port of bluetooth.cpp deliver. Events have priority; at most one ACL delivery each call.
unsafe fn deliver() {
    let g = G.0.get();
    while (*g).events.request != 0 && !(*g).event_queue.is_empty() {
        let e = (*g).events;
        let p = (*g).event_queue.front().unwrap();
        let n = (p.len() as u32).min(e.length);
        std::ptr::copy_nonoverlapping(p.as_ptr(), host(e.data), n as usize);
        (*g).completions.push_back(Completion {
            request: e.request,
            result: n as i32,
        });
        (*g).events = Endpoint::default();
        (*g).event_queue.pop_front();
    }
    if (*g).acl_in.request != 0 && !(*g).acl_queue.is_empty() && (*g).event_queue.is_empty() {
        let e = (*g).acl_in;
        let p = (*g).acl_queue.front().unwrap();
        let n = (p.len() as u32).min(e.length);
        std::ptr::copy_nonoverlapping(p.as_ptr(), host(e.data), n as usize);
        (*g).completions.push_back(Completion {
            request: e.request,
            result: n as i32,
        });
        (*g).acl_in = Endpoint::default();
        (*g).acl_queue.pop_front();
    }
}
// port of bluetooth.cpp queue_event/event/command_complete/command_status.
unsafe fn queue_event(p: Vec<u8>) {
    if wphw_bt_log_enabled() {
        wphw_bt_log(0, p[0] as u32, p.len() as u32, 0, 0);
    }
    (*G.0.get()).event_queue.push_back(p);
    deliver();
}
unsafe fn event(code: u8, params: &[u8]) {
    let mut p = vec![code, params.len() as u8];
    p.extend_from_slice(params);
    queue_event(p);
}
unsafe fn command_complete(op: u16, params: &[u8]) {
    let mut p = vec![1];
    put16(&mut p, op);
    p.extend_from_slice(params);
    event(kEventCommandComplete, &p);
}
unsafe fn command_status(op: u16) {
    let mut p = vec![0, 1];
    put16(&mut p, op);
    event(kEventCommandStatus, &p);
}
// port of bluetooth.cpp send_acl/send_signal.
unsafe fn send_acl(i: u32, cid: u16, payload: &[u8]) {
    let mut p = Vec::new();
    put16(&mut p, connection_handle(i) | (0x2 << 12));
    put16(&mut p, payload.len().wrapping_add(4) as u16);
    put16(&mut p, payload.len() as u16);
    put16(&mut p, cid);
    p.extend_from_slice(payload);
    (*G.0.get()).acl_queue.push_back(p);
    deliver();
}
unsafe fn send_signal(i: u32, ident: u8, code: u8, data: &[u8]) {
    let mut p = vec![code, ident];
    put16(&mut p, data.len() as u16);
    p.extend_from_slice(data);
    if wphw_bt_log_enabled() {
        wphw_bt_log(1, code as u32, ident as u32, 0, 0);
    }
    send_acl(i, kSignalChannel, &p);
}
// port of bluetooth.cpp channel_with_psm/new_channel_id/channel_ready/link_channel. BTreeMap retains std::map ascending CID order.
unsafe fn channel_with_psm(i: u32, psm: u16) -> Option<(u16, Channel)> {
    (*remote(i))
        .channels
        .iter()
        .find(|(_, c)| c.psm == psm)
        .map(|(&k, &c)| (k, c))
}
unsafe fn new_channel_id(i: u32) -> u16 {
    let mut cid = 0x40u16;
    while (*remote(i)).channels.contains_key(&cid) {
        cid = cid.wrapping_add(1);
    }
    cid
}
fn channel_ready(c: &Channel) -> bool {
    c.remote_cid != 0 && c.remote_mtu != 0 && c.complete
}
unsafe fn link_channel(i: u32, psm: u16) -> bool {
    if let Some((_, c)) = channel_with_psm(i, psm) {
        return channel_ready(&c);
    }
    let cid = new_channel_id(i);
    (*remote(i)).channels.entry(cid).or_default().psm = psm;
    let mut p = Vec::new();
    put16(&mut p, psm);
    put16(&mut p, cid);
    send_signal(i, kSignalConnectRequest, kSignalConnectRequest, &p);
    false
}
// port of bluetooth.cpp send_interrupt. Called synchronously by the source's Sender closure.
unsafe fn send_interrupt(i: u32, p: &[u8]) {
    let cid = match channel_with_psm(i, kPsmHidInterrupt) {
        Some((_, c)) if c.remote_cid != 0 => c.remote_cid,
        _ => return,
    };
    send_acl(i, cid, p);
}
#[no_mangle]
pub unsafe extern "C" fn wprt_bt_send_interrupt(i: u32, p: *const u8, n: usize) {
    send_interrupt(i, std::slice::from_raw_parts(p, n));
}
// port of bluetooth.cpp signal. Original packet boundary check and signal/option parsing order retained.
unsafe fn signal(i: u32, mut p: *const u8, mut size: u32) {
    while size >= 4 {
        let code = *p;
        let ident = *p.add(1);
        let length = get16(p.add(2));
        let b = p.add(4);
        if 4 + length as u32 > size {
            break;
        }
        if wphw_bt_log_enabled() {
            wphw_bt_log(2, code as u32, ident as u32, length as u32, 0);
        }
        match code {
            kSignalConnectRequest => {
                let psm = get16(b);
                let scid = get16(b.add(2));
                let mut response = Vec::new();
                if channel_with_psm(i, psm).is_some() {
                    put16(&mut response, 0);
                    put16(&mut response, scid);
                    put16(&mut response, 0x0004);
                    put16(&mut response, 0);
                } else {
                    let cid = new_channel_id(i);
                    let c = (*remote(i)).channels.entry(cid).or_default();
                    c.psm = psm;
                    c.remote_cid = scid;
                    put16(&mut response, cid);
                    put16(&mut response, scid);
                    put16(&mut response, 0);
                    put16(&mut response, 0);
                }
                send_signal(i, ident, kSignalConnectResponse, &response);
            }
            kSignalConnectResponse => {
                let dcid = get16(b);
                let scid = get16(b.add(2));
                let result = get16(b.add(4));
                if wphw_bt_log_enabled() {
                    wphw_bt_log(
                        3,
                        dcid as u32,
                        scid as u32,
                        result as u32,
                        get16(b.add(6)) as u32,
                    );
                }
                if result == 0 {
                    if let Some(c) = (*remote(i)).channels.get_mut(&scid) {
                        c.remote_cid = dcid;
                    }
                }
            }
            kSignalConfigRequest => {
                let dcid = get16(b);
                if let Some(c) = (*remote(i)).channels.get(&dcid).copied() {
                    let mut response = Vec::new();
                    put16(&mut response, c.remote_cid);
                    put16(&mut response, 0);
                    put16(&mut response, 0);
                    let mut mtu = kDefaultMtu;
                    let mut off = 4u32;
                    while off + 2 <= length as u32 {
                        let typ = *b.add(off as usize);
                        let len = *b.add(off as usize + 1);
                        if typ == kOptionMtu && len == 2 {
                            mtu = get16(b.add(off as usize + 2));
                        }
                        response.extend_from_slice(std::slice::from_raw_parts(
                            b.add(off as usize),
                            2 + len as usize,
                        ));
                        off += 2 + len as u32;
                    }
                    (*remote(i)).channels.get_mut(&dcid).unwrap().remote_mtu = mtu;
                    send_signal(i, ident, kSignalConfigResponse, &response);
                }
            }
            kSignalConfigResponse => {
                let scid = get16(b);
                if let Some(c) = (*remote(i)).channels.get_mut(&scid) {
                    c.complete = true;
                }
            }
            kSignalDisconnectRequest => {
                let dcid = get16(b);
                let scid = get16(b.add(2));
                (*remote(i)).channels.remove(&dcid);
                let mut response = Vec::new();
                put16(&mut response, dcid);
                put16(&mut response, scid);
                send_signal(i, ident, kSignalDisconnectResponse, &response);
            }
            _ => wphw_bt_log(4, code as u32, 0, 0, 0),
        }
        p = p.add(4 + length as usize);
        size -= 4 + length as u32;
    }
}
// port of bluetooth.cpp receive_acl. No mutable Wiimote reference spans native report callbacks.
unsafe fn receive_acl(data: u32) {
    let handle = (rd8(data) | (rd8(data.wrapping_add(1)) << 8)) as u16 & 0x0fff;
    let length = (rd8(data.wrapping_add(2)) | (rd8(data.wrapping_add(3)) << 8)) as u16;
    let index = wiimote_from_handle(handle);
    if index < 0 || length < 4 {
        return;
    }
    let i = index as u32;
    (*remote(i)).completed_packets = (*remote(i)).completed_packets.wrapping_add(1);
    let p = std::slice::from_raw_parts(host(data.wrapping_add(4)), length as usize).to_vec();
    let l = get16(p.as_ptr());
    let cid = get16(p.as_ptr().add(2));
    let payload = p.as_ptr().add(4);
    let size = (l as u32).min(length as u32 - 4);
    if cid == kSignalChannel {
        signal(i, payload, size);
        return;
    }
    let c = match (*remote(i)).channels.get(&cid).copied() {
        Some(c) if size != 0 => c,
        _ => return,
    };
    if c.psm == kPsmHidControl && *payload == kHidSetReportOutput {
        send_acl(i, c.remote_cid, &[kHidHandshakeSuccess]);
    }
    let report = (c.psm == kPsmHidInterrupt && *payload == kHidDataOutput)
        || (c.psm == kPsmHidControl && *payload == kHidSetReportOutput);
    if report {
        wphw_bt_output(i, (*remote(i)).real, payload.add(1), size - 1);
    } else {
        wphw_bt_log(5, c.psm as u32, *payload as u32, 0, 0);
    }
}
// port of bluetooth.cpp execute_command; original opcode grouping, status/event ordering, link timeout and default success preserved.
unsafe fn execute_command(data: u32) {
    let opcode = (rd8(data) | (rd8(data.wrapping_add(1)) << 8)) as u16;
    let p = data.wrapping_add(3);
    if wphw_bt_log_enabled() {
        wphw_bt_log(6, opcode as u32, 0, 0, 0);
    }
    match opcode {
        kCommandReadLocalVersion => {
            let mut v = vec![0x00, 0x03];
            put16(&mut v, 0x40a7);
            v.push(0x03);
            put16(&mut v, 0x000f);
            put16(&mut v, 0x430e);
            command_complete(opcode, &v);
        }
        kCommandReadLocalFeatures => command_complete(
            opcode,
            &[0x00, 0xff, 0xff, 0x8d, 0xfe, 0x9b, 0xf9, 0x00, 0x80],
        ),
        kCommandReadBufferSize => {
            let mut v = vec![0];
            put16(&mut v, kAclPacketSize);
            v.push(kScoPacketSize);
            put16(&mut v, kAclPacketCount);
            put16(&mut v, kScoPacketCount);
            command_complete(opcode, &v);
        }
        kCommandReadBdAddr => {
            let mut v = vec![0];
            v.extend_from_slice(&kControllerAddress);
            command_complete(opcode, &v);
        }
        kCommandReadStoredLinkKey => {
            let count = if rd8(p.wrapping_add(6)) == 1 {
                kWiimoteCount as u8
            } else {
                0
            };
            let mut keys = vec![count];
            for i in 0..count as u32 {
                put_address(&mut keys, i);
                keys.extend_from_slice(&[0xa0u32.wrapping_add(i) as u8; 16]);
            }
            event(kEventReturnLinkKeys, &keys);
            let mut v = vec![0];
            put16(&mut v, 255);
            put16(&mut v, count as u16);
            command_complete(opcode, &v);
        }
        kCommandDeleteStoredLinkKey => {
            let mut v = vec![0];
            put16(&mut v, 0);
            command_complete(opcode, &v);
        }
        kCommandWriteScanEnable => {
            (*G.0.get()).scan_enable = rd8(p) as u8;
            command_complete(opcode, &[0]);
        }
        kCommandWriteLinkSupervisionTimeout => {
            let i = wiimote_from_handle((rd8(p) | (rd8(p.wrapping_add(1)) << 8)) as u16);
            if i >= 0 {
                (*remote(i as u32)).host_ready = true;
            }
            command_complete(opcode, &[0, rd8(p) as u8, rd8(p.wrapping_add(1)) as u8]);
        }
        kCommandInquiry => command_status(opcode),
        kCommandCreateConnection | kCommandAcceptConnection => {
            command_status(opcode);
            let index = wiimote_from_address(p);
            let accept = opcode == kCommandAcceptConnection;
            let role = if accept { rd8(p.wrapping_add(6)) } else { 0 };
            let ok = index >= 0
                && attached(index as u32)
                && (*G.0.get()).scan_enable & kPageScanEnable != 0;
            if ok {
                let i = index as u32;
                (*remote(i)).baseband = Baseband::Complete;
                (*remote(i)).real = wphw_bt_real_present(i);
                if accept {
                    (*remote(i)).linking = true;
                    (*remote(i)).host_ready = false;
                    (*remote(i)).link_deadline = wphw_bt_deadline(wphw_bt_now());
                    if role == 0 {
                        let mut v = vec![0];
                        put_address(&mut v, i);
                        v.push(0);
                        event(kEventRoleChange, &v);
                    }
                }
            }
            let mut body = vec![if ok { 0 } else { 0x08 }];
            put16(
                &mut body,
                connection_handle(if index < 0 { 0 } else { index as u32 }),
            );
            for k in 0..6 {
                body.push(rd8(p.wrapping_add(k)) as u8);
            }
            body.push(1);
            body.push(0);
            event(kEventConnectionComplete, &body);
        }
        kCommandDisconnect => {
            let handle = (rd8(p) | (rd8(p.wrapping_add(1)) << 8)) as u16;
            let reason = rd8(p.wrapping_add(2)) as u8;
            command_status(opcode);
            let mut body = vec![0];
            put16(&mut body, handle);
            body.push(reason);
            event(kEventDisconnectionComplete, &body);
            let index = wiimote_from_handle(handle);
            if index >= 0 {
                let i = index as u32;
                *remote(i) = Wiimote::default();
                wphw_bt_wii_reset(i);
                wphw_bt_real_reset(i);
            }
        }
        kCommandLinkKeyReply | kCommandLinkKeyNegativeReply => {
            let mut v = vec![0];
            for k in 0..6 {
                v.push(rd8(p.wrapping_add(k)) as u8);
            }
            command_complete(opcode, &v);
        }
        kCommandChangePacketType => {
            command_status(opcode);
            event(
                kEventPacketTypeChanged,
                &[
                    0,
                    rd8(p) as u8,
                    rd8(p.wrapping_add(1)) as u8,
                    rd8(p.wrapping_add(2)) as u8,
                    rd8(p.wrapping_add(3)) as u8,
                ],
            );
        }
        kCommandAuthenticationRequest => {
            command_status(opcode);
            event(
                kEventAuthenticationComplete,
                &[0, rd8(p) as u8, rd8(p.wrapping_add(1)) as u8],
            );
        }
        kCommandRemoteNameRequest => {
            command_status(opcode);
            let index = wiimote_from_address(p);
            if index >= 0 {
                let mut v = vec![0];
                put_address(&mut v, index as u32);
                let mut name = vec![0; 248];
                name[..kWiimoteName.len()].copy_from_slice(kWiimoteName);
                v.extend_from_slice(&name);
                event(kEventRemoteNameComplete, &v);
            }
        }
        kCommandReadRemoteFeatures => {
            command_status(opcode);
            let mut v = vec![0, rd8(p) as u8, rd8(p.wrapping_add(1)) as u8];
            v.extend_from_slice(&kWiimoteFeatures);
            event(kEventRemoteFeaturesComplete, &v);
        }
        kCommandReadRemoteVersion => {
            command_status(opcode);
            let mut v = vec![0, rd8(p) as u8, rd8(p.wrapping_add(1)) as u8, 2];
            put16(&mut v, 0x000f);
            put16(&mut v, 0x0229);
            event(kEventRemoteVersionComplete, &v);
        }
        kCommandReadClockOffset => {
            command_status(opcode);
            let mut v = vec![0, rd8(p) as u8, rd8(p.wrapping_add(1)) as u8];
            put16(&mut v, 0x3818);
            event(kEventClockOffsetComplete, &v);
        }
        kCommandSniffMode => {
            command_status(opcode);
            event(
                kEventModeChange,
                &[
                    0,
                    rd8(p) as u8,
                    rd8(p.wrapping_add(1)) as u8,
                    2,
                    rd8(p.wrapping_add(2)) as u8,
                    rd8(p.wrapping_add(3)) as u8,
                ],
            );
        }
        kCommandWriteLinkPolicy => command_status(opcode),
        _ => command_complete(opcode, &[0]),
    }
}
// port of bluetooth.cpp update_wiimote. Preserve polling short circuits, ascending channel order and state-before-send writes.
unsafe fn update_wiimote(i: u32) {
    let w = remote(i);
    let real = wphw_bt_real_present(i);
    if (*w).baseband != Baseband::Inactive && (!attached(i) || (*w).real != real) {
        if (*w).baseband == Baseband::Complete {
            let mut v = vec![0];
            put16(&mut v, connection_handle(i));
            v.push(kReasonRemoteTerminated);
            event(kEventDisconnectionComplete, &v);
        }
        *w = Wiimote::default();
        wphw_bt_wii_reset(i);
        wphw_bt_real_reset(i);
        return;
    }
    let wakes = if real {
        wphw_bt_real_wake(i)
    } else {
        wphw_bt_input_connected(i) && wphw_bt_input_wake(i)
    };
    if (*w).baseband == Baseband::Inactive && wakes {
        (*w).baseband = Baseband::RequestConnection;
        (*w).real = real;
    }
    if (*w).baseband == Baseband::RequestConnection
        && (*G.0.get()).scan_enable & kPageScanEnable != 0
    {
        let mut v = Vec::new();
        put_address(&mut v, i);
        v.extend_from_slice(&kWiimoteClass);
        v.push(1);
        event(kEventConnectionRequest, &v);
        (*w).baseband = Baseband::Pending;
    }
    if (*w).baseband != Baseband::Complete {
        return;
    }
    // Snapshot only keys; each channel is read and updated at its original iteration point, before send_signal.
    let keys: Vec<u16> = (*w).channels.keys().copied().collect();
    for cid in keys {
        let c = (*w).channels.get(&cid).copied().unwrap();
        if c.remote_cid != 0 && !c.configuration_sent {
            (*w).channels.get_mut(&cid).unwrap().configuration_sent = true;
            let mut v = Vec::new();
            put16(&mut v, c.remote_cid);
            put16(&mut v, 0);
            v.push(kOptionMtu);
            v.push(2);
            put16(&mut v, kRequestMtu);
            send_signal(i, kSignalConfigRequest, kSignalConfigRequest, &v);
        }
    }
    let may_link = (*w).host_ready || wphw_bt_now() >= (*w).link_deadline;
    if (*w).linking
        && may_link
        && link_channel(i, kPsmHidControl)
        && link_channel(i, kPsmHidInterrupt)
    {
        (*w).linking = false;
    }
    if let Some((_, c)) = channel_with_psm(i, kPsmHidInterrupt) {
        if channel_ready(&c) {
            let real = (*w).real;
            wphw_bt_update(i, real);
        }
    }
}
// port of bluetooth.cpp send_completed_packets, including source wrapping and reset-before-event.
unsafe fn send_completed_packets() {
    let mut body = vec![kWiimoteCount as u8];
    let mut total = 0u32;
    for i in 0..kWiimoteCount {
        put16(&mut body, connection_handle(i));
        put16(&mut body, (*remote(i)).completed_packets as u16);
        total = total.wrapping_add((*remote(i)).completed_packets);
        (*remote(i)).completed_packets = 0;
    }
    if total != 0 {
        event(kEventCompletedPackets, &body);
    }
}
// port of bluetooth.cpp start; close intentionally does not reset g_started, remotes or completion queue.
unsafe fn start() {
    let g = G.0.get();
    if (*g).started {
        return;
    }
    (*g).started = true;
    wphw_bt_real_start();
    for i in 0..kWiimoteCount {
        wphw_bt_wii_reset(i);
        if wphw_bt_input_connected(i) {
            (*remote(i)).baseband = Baseband::RequestConnection;
        }
    }
}
// port of bluetooth.cpp handles. Native std::string is flattened by explicit length (embedded NUL is preserved).
#[no_mangle]
pub unsafe extern "C" fn wprt_bt_handles(p: *const u8, n: usize) -> bool {
    std::slice::from_raw_parts(p, n) == kDevicePath
}
// port of bluetooth.cpp ioctlv; unused output_count remains unused. ios::kDeferred sourced from ios.h.
#[no_mangle]
pub unsafe extern "C" fn wprt_bt_ioctlv(
    request: u32,
    command: u32,
    input_count: u32,
    _output_count: u32,
    vectors: u32,
) -> i32 {
    start();
    match command {
        kControlMessage => {
            let data = io_vector(vectors, input_count, 0);
            let length =
                rd8(in_vector(vectors, 4)) | (rd8(in_vector(vectors, 4).wrapping_add(1)) << 8);
            execute_command(data);
            return length as i32;
        }
        kBulkMessage => {
            let ep = rd8(in_vector(vectors, 0)) as u8;
            if ep == kEndpointAclIn {
                (*G.0.get()).acl_in = Endpoint {
                    request,
                    data: io_vector(vectors, input_count, 0),
                    length: io_vector_size(vectors, input_count, 0),
                };
                deliver();
                return i32::MIN;
            }
            if ep == kEndpointAclOut {
                receive_acl(io_vector(vectors, input_count, 0));
                return rd16(in_vector(vectors, 1)) as i32;
            }
        }
        kInterruptMessage => {
            let ep = rd8(in_vector(vectors, 0)) as u8;
            if ep == kEndpointEvents {
                (*G.0.get()).events = Endpoint {
                    request,
                    data: io_vector(vectors, input_count, 0),
                    length: io_vector_size(vectors, input_count, 0),
                };
                deliver();
                return i32::MIN;
            }
        }
        _ => {}
    }
    wphw_bt_log(7, command, 0, 0, 0);
    0
}
// port of bluetooth.cpp close/update/take_completion.
#[no_mangle]
pub unsafe extern "C" fn wprt_bt_close() {
    let g = G.0.get();
    (*g).events = Endpoint::default();
    (*g).acl_in = Endpoint::default();
    (*g).event_queue.clear();
    (*g).acl_queue.clear();
    (*g).scan_enable = 0;
}
#[no_mangle]
pub unsafe extern "C" fn wprt_bt_update() {
    if (*G.0.get()).started {
        for i in 0..kWiimoteCount {
            update_wiimote(i);
        }
        send_completed_packets();
    }
    deliver();
}
#[no_mangle]
pub unsafe extern "C" fn wprt_bt_take_completion(request: *mut u32, result: *mut i32) -> bool {
    match (*G.0.get()).completions.pop_front() {
        None => false,
        Some(c) => {
            *request = c.request;
            *result = c.result;
            true
        }
    }
}
