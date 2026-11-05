// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of pristine engine/src/input/si.cpp. Keep the original absent-device behaviour.
#![allow(non_upper_case_globals)]
const kBase:u32=0xCD006400;
const kChannels:u32=4;
const kChannelStride:u32=0x0C;
const kChannelOut:u32=0x00;
const kChannelInHigh:u32=0x04;
const kChannelInLow:u32=0x08;
const kPoll:u32=0x30;
const kComCsr:u32=0x34;
const kStatus:u32=0x38;
const kExiClockCount:u32=0x3C;
const kBuffer:u32=0x80;
const kBufferSize:u32=0x80;
const kTransferStart:u32=1 << 0;
const kChannelShift:u32=1;
const kComKeptMask:u32=(3 << 1) | (0x7F << 8) | (0x7F << 16) | (1 << 27) | (1 << 30);
const kReadStatusInterruptMask:u32=1 << 27;
const kReadStatusInterrupt:u32=1 << 28;
const kCommunicationError:u32=1 << 29;
const kTransferInterruptMask:u32=1 << 30;
const kTransferInterrupt:u32=1 << 31;
const kErrorLatch:u32=1 << 30;
const kErrorStatus:u32=1 << 31;
const kWrite:u32=1 << 31;
const kClearableErrors:u32=0x0F0F0F0F;
use std::sync::Mutex;
#[derive(Clone,Copy)]struct Channel{out:u32,in_high:u32,in_low:u32}
struct State{channels:[Channel;kChannels as usize],buffer:[u8;kBufferSize as usize],poll:u32,com:u32,status:u32,exi_clock_count:u32}
// Binding of si.cpp's zero-initialized private globals. SI service calls contain no guest control flow.
static STATE:Mutex<State>=Mutex::new(State{channels:[Channel{out:0,in_high:0,in_low:0};kChannels as usize],buffer:[0;kBufferSize as usize],poll:0,com:0,status:0,exi_clock_count:0});
// port of input/si.cpp shift_of.
fn shift_of(channel:u32)->u32{kChannels.wrapping_sub(1).wrapping_sub(channel).wrapping_mul(8)}
// port of input/si.cpp no_response_bit.
fn no_response_bit(channel:u32)->u32{1<<(shift_of(channel)+3)}
// port of input/si.cpp read_status_bit.
fn read_status_bit(channel:u32)->u32{1<<(shift_of(channel)+5)}
// port of input/si.cpp write_status_bits.
fn write_status_bits()->u32{let mut bits=0;for channel in 0..kChannels{bits|=1<<(shift_of(channel)+4);}bits}
// port of input/si.cpp update_read_status.
fn update_read_status(s:&mut State){let mut ready=false;for channel in 0..kChannels{ready=ready||(s.status&read_status_bit(channel)!=0);}s.com=if ready{s.com|kReadStatusInterrupt}else{s.com&!kReadStatusInterrupt};}
// port of input/si.cpp run_transfer: this source deliberately reports no connected device.
fn run_transfer(s:&mut State){let channel=(s.com>>kChannelShift)&3;s.com&=!kTransferStart;s.com|=kCommunicationError|kTransferInterrupt;s.status|=no_response_bit(channel);}
// port of input/si.cpp read32. Preserve big-endian buffer words and channel-read status clearing.
#[no_mangle]pub extern "C" fn wprt_si_read32(address:u32)->u32{let mut s=STATE.lock().unwrap();let offset=address.wrapping_sub(kBase);
    if offset>=kBuffer&&offset<kBuffer+kBufferSize{let index=(offset-kBuffer)as usize;unsafe{let p=s.buffer.as_ptr().add(index);return u32::from_be_bytes([*p,*p.add(1),*p.add(2),*p.add(3)]);}}
    if offset<kChannels*kChannelStride{let channel=offset/kChannelStride;return match offset%kChannelStride{kChannelOut=>s.channels[channel as usize].out,kChannelInHigh=>{s.status&=!read_status_bit(channel);update_read_status(&mut s);s.channels[channel as usize].in_high},kChannelInLow=>{s.status&=!read_status_bit(channel);update_read_status(&mut s);s.channels[channel as usize].in_low},_=>0};}
    match offset{kPoll=>s.poll,kComCsr=>s.com,kStatus=>s.status,kExiClockCount=>s.exi_clock_count,_=>0}
}
// port of input/si.cpp write32, including source write-one-to-clear and transfer-start ordering.
#[no_mangle]pub extern "C" fn wprt_si_write32(address:u32,value:u32){let mut s=STATE.lock().unwrap();let offset=address.wrapping_sub(kBase);
    if offset>=kBuffer&&offset<kBuffer+kBufferSize{let index=(offset-kBuffer)as usize;unsafe{let p=s.buffer.as_mut_ptr().add(index);*p=(value>>24)as u8;*p.add(1)=(value>>16)as u8;*p.add(2)=(value>>8)as u8;*p.add(3)=value as u8;}return;}
    if offset<kChannels*kChannelStride{let channel=(offset/kChannelStride)as usize;match offset%kChannelStride{kChannelOut=>s.channels[channel].out=value,kChannelInHigh=>s.channels[channel].in_high=value,kChannelInLow=>s.channels[channel].in_low=value,_=>{}}return;}
    match offset{kPoll=>s.poll=value,kComCsr=>{s.com=(s.com&!kComKeptMask)|(value&kComKeptMask);if value&kReadStatusInterrupt!=0{s.com&=!kReadStatusInterrupt;}if value&kTransferInterrupt!=0{s.com&=!kTransferInterrupt;}if value&kTransferStart!=0{s.com|=kTransferStart;run_transfer(&mut s);}},kStatus=>{s.status&=!(value&kClearableErrors);if value&kWrite!=0{s.status&=!(kWrite|write_status_bits());}},kExiClockCount=>s.exi_clock_count=value,_=>{}}
}
// port of input/si.cpp poll.
#[no_mangle]pub extern "C" fn wprt_si_poll(){let mut s=STATE.lock().unwrap();for channel in 0..kChannels{s.status|=no_response_bit(channel);s.channels[channel as usize].in_high|=kErrorStatus|kErrorLatch;}update_read_status(&mut s);}
// port of input/si.cpp interrupt_pending.
#[no_mangle]pub extern "C" fn wprt_si_pending()->bool{let s=STATE.lock().unwrap();((s.com&kReadStatusInterrupt!=0)&&(s.com&kReadStatusInterruptMask!=0))||((s.com&kTransferInterrupt!=0)&&(s.com&kTransferInterruptMask!=0))}
