// SPDX-License-Identifier: GPL-3.0-or-later
// Private G60 whole exi_sdk.cpp + padmap_sdk.cpp + png_sdk.cpp port.
// CANDIDATE: existing three pure differentials byte-exact; live quick gate pending. GNU x64 layout/return ABI from exact accepted object disassembly.
// Native libstdc++ allocator/string implementation remains an explicit dependency.
use std::ffi::{c_char,c_void};
#[repr(C)] pub union StringStorage { local:[u8;16], capacity:usize }
#[repr(C)] pub struct GnuString { data:*mut u8, len:usize, storage:StringStorage }
#[repr(C)] pub struct GnuVector { begin:*mut u8, end:*mut u8, cap:*mut u8 }
type Sink=unsafe extern "C-unwind" fn(*mut c_void,*const u8,usize);
extern "C-unwind" {
 fn wprt_exi_mount(p:*const c_char,n:usize);
 fn wprt_padmap_setting_key(i:usize,sink:Sink,context:*mut c_void);
 fn wprt_padmap_physical_from_name(p:*const u8,n:usize)->i32;
 fn wphw_png_encode_adapter(raw:*const c_void,width:u32,height:u32,out:*mut c_void);
 #[link_name="_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_replaceEyyPKcy"]
 fn string_replace(s:*mut GnuString,pos:usize,old:usize,p:*const u8,n:usize)->*mut GnuString;
 #[link_name="_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE10_M_disposeEv"]
 fn string_dispose(s:*mut GnuString);
 #[link_name="_Znwy"] fn cpp_new(n:usize)->*mut u8;
 #[link_name="_ZdlPvy"] fn cpp_delete(p:*mut u8,n:usize);
 #[link_name="_ZSt20__throw_length_errorPKc"] fn length_error(s:*const c_char)->!;
 fn memcpy(d:*mut c_void,s:*const c_void,n:usize)->*mut c_void;
 fn memmove(d:*mut c_void,s:*const c_void,n:usize)->*mut c_void;
}
struct StringGuard(*mut GnuString);
impl Drop for StringGuard { fn drop(&mut self){unsafe{string_dispose(self.0)}} }
struct VectorGuard(*mut GnuVector);
impl Drop for VectorGuard { fn drop(&mut self){unsafe{let v=&*self.0;if !v.begin.is_null(){cpp_delete(v.begin,(v.cap as usize).wrapping_sub(v.begin as usize));}}} }
unsafe extern "C-unwind" fn string_sink(c:*mut c_void,p:*const u8,n:usize){
 let s=c.cast::<GnuString>();string_replace(s,0,(*s).len,p,n);
}
// exi_sdk.cpp entire namespace member; counted bytes including embedded NUL are preserved.
#[export_name="_ZN2wp3exi5mountERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn exi_mount(s:*const GnuString){wprt_exi_mount((*s).data.cast(),(*s).len)}
// Accepted padmap object .text20: hidden return RCX, index RDX, RAX returns original RCX.
// Initializes data=object+16, length=0, local[0]=0 before callback; cleanup only on unwind.
#[export_name="_ZN2wp6padmap11setting_keyB5cxx11Ey"]
pub unsafe extern "C-unwind" fn padmap_setting_key(out:*mut GnuString,i:usize)->*mut GnuString{
 std::ptr::addr_of_mut!((*out).data).write(std::ptr::addr_of_mut!((*out).storage).cast());
 std::ptr::addr_of_mut!((*out).len).write(0);
 std::ptr::addr_of_mut!((*out).storage).cast::<u8>().write(0);
 let guard=StringGuard(out);wprt_padmap_setting_key(i,string_sink,out.cast());std::mem::forget(guard);out
}
#[export_name="_ZN2wp6padmap18physical_from_nameERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE"]
pub unsafe extern "C-unwind" fn physical_from_name(s:*const GnuString)->i32{wprt_padmap_physical_from_name((*s).data,(*s).len)}
// png_sdk.cpp .text0/.text20: vector fields at 0/8/16; input u32 span length=(end-begin)/4.
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_png_vector_sdk(raw:*const GnuVector,p:*mut *const u32,n:*mut usize){
 p.write((*raw).begin.cast());n.write(((*raw).end as usize).wrapping_sub((*raw).begin as usize)/4)
}
#[no_mangle] pub unsafe extern "C-unwind" fn wphw_png_receive_sdk(c:*mut GnuVector,p:*const u8,n:usize){
 let v=&mut *c;let cap=(v.cap as usize).wrapping_sub(v.begin as usize);let len=(v.end as usize).wrapping_sub(v.begin as usize);
 if n>cap{
  // Exact length guard/message/operator new/sized delete from accepted .text20-14c + relocations.
  if n>isize::MAX as usize{length_error(b"cannot create std::vector larger than max_size()\0".as_ptr().cast())}
  let fresh=cpp_new(n);memcpy(fresh.cast(),p.cast(),n);
  if !v.begin.is_null(){cpp_delete(v.begin,cap)}
  v.begin=fresh;v.end=fresh.add(n);v.cap=v.end;
 }else if n<=len{
  if n==1{v.begin.write(p.read())}else if n>1{memmove(v.begin.cast(),p.cast(),n);}
  v.end=v.begin.wrapping_add(n);
 }else{
  if len==1{v.begin.write(p.read())}else if len>1{memmove(v.begin.cast(),p.cast(),len);}
  memcpy(v.end.cast(),p.add(len).cast(),n-len);v.end=v.begin.add(n);
 }
}
#[export_name="_ZN2wp5video10encode_pngERKSt6vectorIjSaIjEEjj"]
pub unsafe extern "C-unwind" fn encode_png(out:*mut GnuVector,raw:*const c_void,width:u32,height:u32)->*mut GnuVector{
 std::ptr::addr_of_mut!((*out).begin).write(std::ptr::null_mut());
 std::ptr::addr_of_mut!((*out).end).write(std::ptr::null_mut());
 std::ptr::addr_of_mut!((*out).cap).write(std::ptr::null_mut());
 let guard=VectorGuard(out);wphw_png_encode_adapter(raw,width,height,out.cast());std::mem::forget(guard);out
}
