// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted complete hybrid/cpp/patched/padmap.cpp adapter policy port.
// Source public padmap.h:Physical enum17 booleans,Mapping array7;C++ caller comparison checks named fields.
use std::ffi::c_char;
#[repr(C)]pub struct Pressed{pub held:[bool;17],pub nintendo_layout:bool}
#[repr(C)]pub struct Mapping{pub values:[i32;7]}
extern "C"{fn wprt_padmap_action_name(index:usize)->*const c_char;fn wprt_padmap_action_button(index:usize)->u32;fn wprt_padmap_physical_name(physical:i32)->*const c_char;fn wprt_padmap_current_mapping(values:*mut i32);fn wprt_padmap_buttons(held:*const u8,nintendo:bool,sideways:bool,mapping:*const i32)->u32;}
// port of wp::padmap action_name/action_button/physical_name exact scalar forwards.
#[export_name="_ZN2wp6padmap11action_nameEy"]pub unsafe extern "C" fn action_name(index:usize)->*const c_char{wprt_padmap_action_name(index)}
#[export_name="_ZN2wp6padmap13action_buttonEy"]pub unsafe extern "C" fn action_button(index:usize)->u32{wprt_padmap_action_button(index)}
#[export_name="_ZN2wp6padmap13physical_nameEi"]pub unsafe extern "C" fn physical_name(physical:i32)->*const c_char{wprt_padmap_physical_name(physical)}
// port of wp::padmap current_mapping:zero transport,callback,copy each field into public aggregate.
#[export_name="_ZN2wp6padmap15current_mappingEv"]pub unsafe extern "C" fn current_mapping()->Mapping{let mut values=[0i32;7];wprt_padmap_current_mapping(values.as_mut_ptr());let mut out=Mapping{values:[0;7]};for i in 0..7{out.values[i]=values[i];}out}
// port of wp::padmap buttons;std::array public field transport only,no container/heap representations.
#[export_name="_ZN2wp6padmap7buttonsERKNS0_7PressedEbRKSt5arrayIiLy7EE"]pub unsafe extern "C" fn buttons(pressed:*const Pressed,sideways:bool,mapping:*const Mapping)->u32{let p=&*pressed;let mapping=&*mapping;let mut held=[0u8;17];let mut values=[0i32;7];for i in 0..17{held[i]=if p.held[i]{1}else{0};}for i in 0..7{values[i]=mapping.values[i];}wprt_padmap_buttons(held.as_ptr(),p.nintendo_layout,sideways,values.as_ptr())}
