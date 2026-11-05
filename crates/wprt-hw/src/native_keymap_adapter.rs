// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted entire owned cpp/patched/keymap.cpp scalar/reference/callback policy port.
// SDK retains original static descriptor guard and string/vector/function/exception ownership.
use std::ffi::c_void;
type Sink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
type Codes=unsafe extern "C" fn(*const c_void,usize,*mut *const i32,*mut usize);
type Value=unsafe extern "C" fn(*mut c_void,*const u8,usize,Sink,*mut c_void);
extern "C"{
#[link_name="_ZN2wp6keymap6actionEy"]fn action(index:usize)->*const c_void;
fn wprt_keymap_key_code(p:*const u8,n:usize)->i32;fn wprt_keymap_reserved(code:i32)->bool;fn wprt_keymap_direction(index:usize)->bool;
fn wprt_keymap_has(p:*const i32,n:usize,code:i32)->bool;fn wprt_keymap_bound(raw:*const c_void,codes:Codes,code:i32)->bool;fn wprt_keymap_legacy(value:Value,raw:*mut c_void)->bool;
fn wphw_keymap_string_sdk(raw:*const c_void,p:*mut *const u8,n:*mut usize);fn wphw_keymap_codes_sdk(raw:*const c_void,index:usize,p:*mut *const i32,n:*mut usize);fn wphw_keymap_value_sdk(raw:*mut c_void,p:*const u8,n:usize,sink:Sink,context:*mut c_void);}
// port of action(Action):public enum int -> size_t,original descriptor guard/reference remains SDK.
#[export_name="_ZN2wp6keymap6actionENS0_6ActionE"]pub unsafe extern "C" fn action_enum(which:i32)->*const c_void{action(which as usize)}
// port of key_code:read SAME counted stdstring through typed SDK,no private layout.
#[export_name="_ZN2wp6keymap8key_codeERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]pub unsafe extern "C" fn key_code(raw:*const c_void)->i32{let mut p=std::ptr::null();let mut n=0;wphw_keymap_string_sdk(raw,&mut p,&mut n);wprt_keymap_key_code(p,n)}
#[export_name="_ZN2wp6keymap8reservedEi"]pub unsafe extern "C" fn reserved(code:i32)->bool{wprt_keymap_reserved(code)}
// port of Bindings::has;source of(Action) exposes the same vector bytes synchronously.
#[export_name="_ZNK2wp6keymap8Bindings3hasENS0_6ActionEi"]pub unsafe extern "C" fn has(raw:*const c_void,which:i32,code:i32)->bool{let mut p=std::ptr::null();let mut n=0;wphw_keymap_codes_sdk(raw,which as usize,&mut p,&mut n);wprt_keymap_has(p,n,code)}
#[export_name="_ZNK2wp6keymap8Bindings5boundEi"]pub unsafe extern "C" fn bound(raw:*const c_void,code:i32)->bool{wprt_keymap_bound(raw,wphw_keymap_codes_sdk,code)}
// port of legacy_directions;source const_cast affects ONLY callback context,Getter object retained unchanged.
#[export_name="_ZN2wp6keymap17legacy_directionsERKSt8functionIFNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEERKS7_EE"]pub unsafe extern "C" fn legacy(raw:*const c_void)->bool{wprt_keymap_legacy(wphw_keymap_value_sdk,raw.cast_mut())}
#[export_name="_ZN2wp6keymap9directionEy"]pub unsafe extern "C" fn direction(index:usize)->bool{wprt_keymap_direction(index)}
