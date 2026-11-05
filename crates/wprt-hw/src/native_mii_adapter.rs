// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted entire owned mii_records_rust.cpp scalar/field/callback policy;typed native factories/filesystem/errors retained.
use std::ffi::{c_char,c_void,CStr};
// port of PUBLIC wp/mii.h Field (not a private class/container representation).
#[repr(C)]pub struct Field{key:*const c_char,offset:u8,bytes:u8,shift:u8,bits:u8,maximum:u8,minimum:u8}
extern "C"{fn wphw_mii_string_sdk(raw:*const c_void,p:*mut *const u8,n:*mut usize);fn wphw_mii_fields_sdk(p:*mut *const Field,n:*mut usize);fn wphw_mii_record_sdk(p:*const c_void)->*mut u8;fn wphw_mii_db_records_sdk(p:*const c_void)->*mut u8;fn wphw_mii_count_sdk(p:*const c_void)->usize;
fn wprt_mii_get(p:*const u8,offset:usize,bytes:u8,shift:u8,bits:u8)->u32;fn wprt_mii_set(p:*mut u8,offset:usize,bytes:u8,shift:u8,bits:u8,maximum:u8,minimum:u8,value:u32);fn wprt_mii_write_text(p:*mut u8,offset:usize,text:*const u8,n:usize);fn wprt_mii_occupied(p:*const u8)->bool;fn wprt_mii_crc(p:*const u8,n:usize)->u16;fn wprt_mii_remove(p:*mut u8,slot:usize);}
// port of find_field:counted stdstring equality against null-terminated source key,original field order/reference.
#[export_name="_ZN2wp3mii10find_fieldERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]pub unsafe extern "C" fn find_field(raw:*const c_void)->*const Field{let mut p=std::ptr::null();let mut n=0;wphw_mii_string_sdk(raw,&mut p,&mut n);let key=std::slice::from_raw_parts(p,n);let mut fields=std::ptr::null();let mut count=0;wphw_mii_fields_sdk(&mut fields,&mut count);for i in 0..count{let field=fields.add(i);if key==CStr::from_ptr((*field).key).to_bytes(){return field;}}std::ptr::null()}
#[export_name="_ZN2wp3mii3getERKSt5arrayIhLy74EERKNS0_5FieldE"]pub unsafe extern "C" fn get(record:*const c_void,field:*const Field)->u32{let f=&*field;wprt_mii_get(wphw_mii_record_sdk(record),f.offset as usize,f.bytes,f.shift,f.bits)}
#[export_name="_ZN2wp3mii3setERSt5arrayIhLy74EERKNS0_5FieldEj"]pub unsafe extern "C" fn set(record:*mut c_void,field:*const Field,value:u32){let f=&*field;wprt_mii_set(wphw_mii_record_sdk(record),f.offset as usize,f.bytes,f.shift,f.bits,f.maximum,f.minimum,value);}
unsafe fn write_text(record:*mut c_void,text:*const c_void,offset:usize){let mut p=std::ptr::null();let mut n=0;wphw_mii_string_sdk(text,&mut p,&mut n);wprt_mii_write_text(wphw_mii_record_sdk(record),offset,p,n);}
// port of set_name/set_creator source constants kNameOffset=0x02/kCreatorOffset=0x36.
#[export_name="_ZN2wp3mii8set_nameERSt5arrayIhLy74EERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]pub unsafe extern "C" fn set_name(record:*mut c_void,text:*const c_void){write_text(record,text,0x02)}
#[export_name="_ZN2wp3mii11set_creatorERSt5arrayIhLy74EERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]pub unsafe extern "C" fn set_creator(record:*mut c_void,text:*const c_void){write_text(record,text,0x36)}
#[export_name="_ZN2wp3mii8occupiedERKSt5arrayIhLy74EE"]pub unsafe extern "C" fn occupied(record:*const c_void)->bool{wprt_mii_occupied(wphw_mii_record_sdk(record))}
#[export_name="_ZN2wp3mii5crc16EPKhy"]pub unsafe extern "C" fn crc16(p:*const u8,n:usize)->u16{wprt_mii_crc(p,n)}
// port of count retains entire used().size() expression/temp vector lifetime in typed SDK.
#[export_name="_ZNK2wp3mii8Database5countEv"]pub unsafe extern "C" fn count(db:*const c_void)->usize{wphw_mii_count_sdk(db)}
#[export_name="_ZNK2wp3mii8Database4fullEv"]pub unsafe extern "C" fn full(db:*const c_void)->bool{count(db)>=100}
#[export_name="_ZN2wp3mii8Database6removeEy"]pub unsafe extern "C" fn remove(db:*mut c_void,slot:usize){wprt_mii_remove(wphw_mii_db_records_sdk(db),slot)}
