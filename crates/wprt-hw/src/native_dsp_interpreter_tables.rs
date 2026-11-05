// Copyright 2018 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later
// AI-assisted entire DSPIntTables.cpp selection/initialization port.
// Masks/opcodes/function names mechanically extracted from pristine source;SDK member-function pointers remain opaque C++.
const MAIN:[(u16,u16,i32);125]=[
(0x0000,0xfffc,0), // nop
(0x0004,0xfffc,1), // dar
(0x0008,0xfffc,2), // iar
(0x000c,0xfffc,3), // subarn
(0x0010,0xfff0,4), // addarn
(0x0021,0xffff,5), // halt
(0x02d0,0xfff0,6), // ret
(0x02f0,0xfff0,7), // rti
(0x02b0,0xfff0,8), // call
(0x0270,0xfff0,9), // ifcc
(0x0290,0xfff0,10), // jcc
(0x1700,0xff10,11), // jmprcc
(0x1710,0xff10,12), // callr
(0x1200,0xff00,13), // sbclr
(0x1300,0xff00,14), // sbset
(0x1400,0xfec0,15), // lsl
(0x1440,0xfec0,16), // lsr
(0x1480,0xfec0,17), // asl
(0x14c0,0xfec0,18), // asr
(0x02ca,0xffff,19), // lsrn
(0x02cb,0xffff,20), // asrn
(0x0080,0xffe0,21), // lri
(0x00c0,0xffe0,22), // lr
(0x00e0,0xffe0,23), // sr
(0x1c00,0xfc00,24), // mrr
(0x1600,0xff00,25), // si
(0x0400,0xfe00,26), // addis
(0x0600,0xfe00,27), // cmpis
(0x0800,0xf800,28), // lris
(0x0200,0xfeff,29), // addi
(0x0220,0xfeff,30), // xori
(0x0240,0xfeff,31), // andi
(0x0260,0xfeff,32), // ori
(0x0280,0xfeff,33), // cmpi
(0x02a0,0xfeff,34), // andf
(0x02c0,0xfeff,35), // andcf
(0x0210,0xfefc,36), // ilrr
(0x0214,0xfefc,37), // ilrrd
(0x0218,0xfefc,38), // ilrri
(0x021c,0xfefc,39), // ilrrn
(0x0040,0xffe0,40), // loop
(0x0060,0xffe0,41), // bloop
(0x1000,0xff00,42), // loopi
(0x1100,0xff00,43), // bloopi
(0x1800,0xff80,44), // lrr
(0x1880,0xff80,45), // lrrd
(0x1900,0xff80,46), // lrri
(0x1980,0xff80,47), // lrrn
(0x1a00,0xff80,48), // srr
(0x1a80,0xff80,49), // srrd
(0x1b00,0xff80,50), // srri
(0x1b80,0xff80,51), // srrn
(0x2000,0xf800,52), // lrs
(0x2800,0xfe00,53), // srsh
(0x2c00,0xfc00,54), // srs
(0x3000,0xfc80,55), // xorr
(0x3400,0xfc80,56), // andr
(0x3800,0xfc80,57), // orr
(0x3c00,0xfe80,58), // andc
(0x3e00,0xfe80,59), // orc
(0x3080,0xfe80,60), // xorc
(0x3280,0xfe80,61), // notc
(0x3480,0xfc80,62), // lsrnrx
(0x3880,0xfc80,63), // asrnrx
(0x3c80,0xfe80,64), // lsrnr
(0x3e80,0xfe80,65), // asrnr
(0x4000,0xf800,66), // addr
(0x4800,0xfc00,67), // addax
(0x4c00,0xfe00,68), // add
(0x4e00,0xfe00,69), // addp
(0x5000,0xf800,70), // subr
(0x5800,0xfc00,71), // subax
(0x5c00,0xfe00,72), // sub
(0x5e00,0xfe00,73), // subp
(0x6000,0xf800,74), // movr
(0x6800,0xfc00,75), // movax
(0x6c00,0xfe00,76), // mov
(0x6e00,0xfe00,77), // movp
(0x7000,0xfc00,78), // addaxl
(0x7400,0xfe00,79), // incm
(0x7600,0xfe00,80), // inc
(0x7800,0xfe00,81), // decm
(0x7a00,0xfe00,82), // dec
(0x7c00,0xfe00,83), // neg
(0x7e00,0xfe00,84), // movnp
(0x8000,0xf700,85), // nx
(0x8100,0xf700,86), // clr
(0x8200,0xff00,87), // cmp
(0x8300,0xff00,88), // mulaxh
(0x8400,0xff00,89), // clrp
(0x8500,0xff00,90), // tstprod
(0x8600,0xfe00,91), // tstaxh
(0x8a00,0xff00,92), // srbith
(0x8b00,0xff00,92), // srbith
(0x8c00,0xff00,92), // srbith
(0x8d00,0xff00,92), // srbith
(0x8e00,0xff00,92), // srbith
(0x8f00,0xff00,92), // srbith
(0x9000,0xf700,93), // mul
(0x9100,0xf700,94), // asr16
(0x9200,0xf600,95), // mulmvz
(0x9400,0xf600,96), // mulac
(0x9600,0xf600,97), // mulmv
(0xa000,0xe700,98), // mulx
(0xa100,0xf700,99), // abs
(0xa200,0xe600,100), // mulxmvz
(0xa400,0xe600,101), // mulxac
(0xa600,0xe600,102), // mulxmv
(0xb100,0xf700,103), // tst
(0xc000,0xe700,104), // mulc
(0xc100,0xe700,105), // cmpaxh
(0xc200,0xe600,106), // mulcmvz
(0xc400,0xe600,107), // mulcac
(0xc600,0xe600,108), // mulcmv
(0xe000,0xfc00,109), // maddx
(0xe400,0xfc00,110), // msubx
(0xe800,0xfc00,111), // maddc
(0xec00,0xfc00,112), // msubc
(0xf000,0xfe00,113), // lsl16
(0xf200,0xfe00,114), // madd
(0xf400,0xfe00,115), // lsr16
(0xf600,0xfe00,116), // msub
(0xf800,0xfc00,117), // addpaxz
(0xfc00,0xfe00,118), // clrl
(0xfe00,0xfe00,119), // movpz
];
const EXT:[(u16,u16,i32);25]=[
(0x0000,0x00fc,120), // nop_ext
(0x0004,0x00fc,121), // dr
(0x0008,0x00fc,122), // ir
(0x000c,0x00fc,123), // nr
(0x0010,0x00f0,124), // mv
(0x0020,0x00e4,125), // s
(0x0024,0x00e4,126), // sn
(0x0040,0x00c4,127), // l
(0x0044,0x00c4,128), // ln
(0x0080,0x00ce,129), // ls
(0x0082,0x00ce,130), // sl
(0x0084,0x00ce,131), // lsn
(0x0086,0x00ce,132), // sln
(0x0088,0x00ce,133), // lsm
(0x008a,0x00ce,134), // slm
(0x008c,0x00ce,135), // lsnm
(0x008e,0x00ce,136), // slnm
(0x00c3,0x00cf,137), // ldax
(0x00c7,0x00cf,138), // ldaxn
(0x00cb,0x00cf,139), // ldaxm
(0x00cf,0x00cf,140), // ldaxnm
(0x00c0,0x00cc,141), // ld
(0x00c4,0x00cc,142), // ldn
(0x00c8,0x00cc,143), // ldm
(0x00cc,0x00cc,144), // ldnm
];
// Source static member-function arrays are null before initialization;SDK ID -1 denotes null,not a native pointer encoding.
static mut OP_TABLE:[i32;65536]=[-1;65536];
static mut EXT_TABLE:[i32;256]=[-1;256];
static mut INITIALIZED:bool=false;
// port of DSPTables.h FindByOpcode,first matching row in original source order.
fn find(inst:u16,rows:&[(u16,u16,i32)])->i32{for &(op,mask,id) in rows{if inst&mask==op{return id}}0}
// port of DSPIntTables.cpp GetOp;only SDK member-function identity crosses the ABI.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_it_get(inst:u16)->i32{std::ptr::addr_of!(OP_TABLE).cast::<i32>().add(inst as usize).read()}
// port of DSPIntTables.cpp GetExtOp;source7-bit extension exactly for high nibble3.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_it_get_ext(inst:u16)->i32{let index=if inst>>12==3{inst&0x7f}else{inst&0xff};std::ptr::addr_of!(EXT_TABLE).cast::<i32>().add(index as usize).read()}
// port of DSPIntTables.cpp InitInstructionTables;sourceplain(non-atomic)guard,EXTbeforeMAIN,flaglast.
// No invented synchronization/race policy;source concurrent unsynchronized initialization is not claimed.
#[no_mangle]pub unsafe extern "C" fn wprt_dsp_it_init(){if INITIALIZED{return}let ext=std::ptr::addr_of_mut!(EXT_TABLE).cast::<i32>();for i in 0..256{ext.add(i).write(find(i as u16,&EXT));}let op=std::ptr::addr_of_mut!(OP_TABLE).cast::<i32>();for i in 0..65536{op.add(i).write(find(i as u16,&MAIN));}INITIALIZED=true;}
