// Copyright 2008 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted port of pristine engine/src/core/hollywood.cpp.
#![allow(non_upper_case_globals)]
use std::sync::Mutex;
struct State{out:u32,direction:u32}
// Binding of source g_out=0 and g_direction=kOutputPins, private to each game DLL.
static STATE:Mutex<State>=Mutex::new(State{out:0,direction:kOutputPins});
// port of core/hollywood.cpp owns, including source-owned unknown registers.
#[no_mangle]pub extern "C" fn wprt_hollywood_owns(address:u32)->bool{address==kGpioOut||address==kGpioDirection||address==kGpioIn||address==kUnknown180||address==kUnknown1CC||address==kUnknown1D0}
// port of core/hollywood.cpp read32; GPIO input remains the source's constant slot-in bit.
#[no_mangle]pub extern "C" fn wprt_hollywood_read32(address:u32)->u32{let s=STATE.lock().unwrap();match address{kGpioOut=>s.out,kGpioDirection=>s.direction,kGpioIn=>kSlotIn,_=>0}}
// port of core/hollywood.cpp write32; only Broadway-owned bits are replaced.
#[no_mangle]pub extern "C" fn wprt_hollywood_write32(address:u32,value:u32){let mut s=STATE.lock().unwrap();if address==kGpioOut{s.out=(value&kBroadwayOwned)|(s.out&!kBroadwayOwned);}else if address==kGpioDirection{s.direction=(value&kBroadwayOwned)|(s.direction&!kBroadwayOwned);}}
const kGpioOut:u32=0xCD8000C0;
const kGpioDirection:u32=0xCD8000C4;
const kGpioIn:u32=0xCD8000C8;
const kUnknown180:u32=0xCD800180;
const kUnknown1CC:u32=0xCD8001CC;
const kUnknown1D0:u32=0xCD8001D0;
const kPower:u32=0x1;
const kShutdown:u32=0x2;
const kFan:u32=0x4;
const kDcDc:u32=0x8;
const kDiSpin:u32=0x10;
const kSlotLed:u32=0x20;
const kSlotIn:u32=0x80;
const kSensorBar:u32=0x100;
const kDoEject:u32=0x200;
const kEepromSelect:u32=0x400;
const kEepromClock:u32=0x800;
const kEepromOut:u32=0x1000;
const kVideoEncoderClock:u32=0x4000;
const kVideoEncoderData:u32=0x8000;
const kDebug:u32=0xFF0000;
const kBroadwayOwned:u32=kSlotLed | kSlotIn | kSensorBar | kDoEject | kVideoEncoderClock | kVideoEncoderData;
const kOutputPins:u32=kPower | kShutdown | kFan | kDcDc | kDiSpin | kSlotLed | kSensorBar | kDoEject | kEepromSelect | kEepromClock |
                                 kEepromOut | kVideoEncoderClock | kVideoEncoderData | kDebug;
