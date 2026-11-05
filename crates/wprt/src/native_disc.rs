// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/ios/disc.cpp. Native std::sort preserves equal-offset ordering.
use crate::memory::host;
use std::{ffi::{c_void,CStr},sync::{Mutex,OnceLock}};
// Constants copied programmatically from pristine disc.cpp.
#[allow(non_upper_case_globals)]const kEntrySize:usize=12;
#[allow(non_upper_case_globals)]const kOffsetShift:u32=2;
#[allow(non_upper_case_globals)]const kBootOffset:u64=0;
#[allow(non_upper_case_globals)]const kBi2Offset:u64=0x440;
#[allow(non_upper_case_globals)]const kApploaderOffset:u64=0x2440;
#[allow(non_upper_case_globals)]const kDolOffsetField:u32=0x420;
#[allow(non_upper_case_globals)]const kFstOffsetField:u32=0x424;
type EntrySink=unsafe extern "C" fn(*mut c_void,u64,u64,*const u8,usize);
extern "C"{
 fn wphw_disc_sort(offsets:*const u64,sizes:*const u64,paths:*const *const u8,lengths:*const usize,n:usize,sink:EntrySink,context:*mut c_void);
 fn wphw_disc_clock()->i64;fn wphw_disc_spent(began:i64)->f64;
 fn wphw_disc_stderr(offset:u64,length:u32,destination:u32,path:*const u8,n:usize,found:bool);
 fn wphw_disc_log(count:u32,path:*const u8,n:usize,found:bool,spent:f64);
}
struct FileEntry{offset:u64,size:u64,path:Vec<u8>}
#[derive(Default)]struct State{files:Vec<FileEntry>,root:Vec<u8>}
// port of disc.cpp namespace globals, per-DLL and cleared on mount exactly as source.
fn state()->&'static Mutex<State>{static S:OnceLock<Mutex<State>>=OnceLock::new();S.get_or_init(Default::default)}
fn terminated(p:&[u8])->Vec<u8>{let mut out=p.to_vec();out.push(0);out}
// port of disc.cpp be32 / read_file / file_size.
fn be32(data:&[u8],offset:usize)->u32{((data[offset]as u32)<<24)|((data[offset+1]as u32)<<16)|((data[offset+2]as u32)<<8)|data[offset+3]as u32}
unsafe fn read_file(path:&[u8],data:&mut Vec<u8>)->bool{let p=terminated(path);let file=libc::fopen(p.as_ptr().cast(),c"rb".as_ptr());if file.is_null(){return false;}let mut buffer=[0u8;65536];loop{let count=libc::fread(buffer.as_mut_ptr().cast(),1,buffer.len(),file);if count==0{break;}data.extend_from_slice(&buffer[..count]);}libc::fclose(file);true}
unsafe fn file_size(path:&[u8])->u64{let p=terminated(path);let file=libc::fopen(p.as_ptr().cast(),c"rb".as_ptr());if file.is_null(){return 0;}libc::fseek(file,0,libc::SEEK_END);let size=libc::ftell(file)as u64;libc::fclose(file);size}
// port of disc.cpp add_system_file.
unsafe fn add_system_file(s:&mut State,name:&[u8],offset:u64){let mut path=s.root.clone();path.extend_from_slice(b"/sys/");path.extend_from_slice(name);s.files.push(FileEntry{offset,size:file_size(&path),path});}
// port of disc.cpp add_files. Valid FST names use the original NUL-terminated byte read, not Unicode decoding.
unsafe fn add_files(s:&mut State,fst:&[u8])->bool{if fst.len()<kEntrySize{return false;}let count=be32(fst,8);let names=count as usize*kEntrySize;if names>fst.len(){return false;}let mut directories=vec![(Vec::<u8>::new(),count)];for index in 1..count{while directories.len()>1&&index>=directories.last().unwrap().1{directories.pop();}let entry=index as usize*kEntrySize;let is_directory=fst[entry]!=0;let name_offset=be32(fst,entry)&0x00ffffff;let name=CStr::from_ptr(fst.as_ptr().add(names+name_offset as usize).cast()).to_bytes();let mut parent=Vec::new();for (directory,_)in &directories{parent.extend_from_slice(directory);}if is_directory{let mut directory=name.to_vec();directory.push(b'/');directories.push((directory,be32(fst,entry+8)));}else{let offset=(be32(fst,entry+4)as u64)<<kOffsetShift;let mut path=s.root.clone();path.extend_from_slice(b"/files/");path.extend_from_slice(&parent);path.extend_from_slice(name);s.files.push(FileEntry{offset,size:be32(fst,entry+8)as u64,path});}}true}
unsafe extern "C" fn sorted_entry(context:*mut c_void,offset:u64,size:u64,path:*const u8,n:usize){(*context.cast::<Vec<FileEntry>>()).push(FileEntry{offset,size,path:std::slice::from_raw_parts(path,n).to_vec()});}
// port of disc.cpp mount std::sort call. Keep native algorithm/type/order for equal offsets; no guessed tie breaker.
unsafe fn sort_files(files:&mut Vec<FileEntry>){let offsets:Vec<_>=files.iter().map(|f|f.offset).collect();let sizes:Vec<_>=files.iter().map(|f|f.size).collect();let paths:Vec<_>=files.iter().map(|f|f.path.as_ptr()).collect();let lengths:Vec<_>=files.iter().map(|f|f.path.len()).collect();let mut out=Vec::new();wphw_disc_sort(offsets.as_ptr(),sizes.as_ptr(),paths.as_ptr(),lengths.as_ptr(),files.len(),sorted_entry,(&mut out as *mut Vec<FileEntry>).cast());*files=out;}
// port of disc.cpp mount; source short-circuit read order and failed-mount partial state are preserved.
#[no_mangle]pub unsafe extern "C" fn wprt_disc_mount(p:*const u8,n:usize)->bool{let mut s=state().lock().unwrap();s.root=std::slice::from_raw_parts(p,n).to_vec();s.files.clear();let mut boot=Vec::new();let mut fst=Vec::new();let mut boot_path=s.root.clone();boot_path.extend_from_slice(b"/sys/boot.bin");let mut fst_path=s.root.clone();fst_path.extend_from_slice(b"/sys/fst.bin");if !read_file(&boot_path,&mut boot)||!read_file(&fst_path,&mut fst)||boot.len()<0x440{return false;}add_system_file(&mut s,b"boot.bin",kBootOffset);add_system_file(&mut s,b"bi2.bin",kBi2Offset);add_system_file(&mut s,b"apploader.img",kApploaderOffset);add_system_file(&mut s,b"main.dol",(be32(&boot,kDolOffsetField as usize)as u64)<<kOffsetShift);add_system_file(&mut s,b"fst.bin",(be32(&boot,kFstOffsetField as usize)as u64)<<kOffsetShift);if !add_files(&mut s,&fst){return false;}sort_files(&mut s.files);true}
// port of disc.cpp read: upper_bound then previous entry, full zero-fill, overlapping spans and native long seek.
#[no_mangle]pub unsafe extern "C" fn wprt_disc_read(offset:u64,length:u32,destination:u32){static LOG:OnceLock<bool>=OnceLock::new();let logging=*LOG.get_or_init(||!libc::getenv(c"WP_LOG_DISC".as_ptr()).is_null());let output=host(destination);std::ptr::write_bytes(output,0,length as usize);let end=offset.wrapping_add(length as u64);let s=state().lock().unwrap();let mut first=0;let mut count=s.files.len();while count>0{let step=count/2;let i=first+step;if offset>=s.files[i].offset{first=i+1;count-=step+1;}else{count=step;}}let mut index=if first!=0{first-1}else{first};while index<s.files.len()&&s.files[index].offset<end{let f=&s.files[index];let file_end=f.offset.wrapping_add(f.size);if file_end<=offset{index+=1;continue;}let start=offset.max(f.offset);let stop=end.min(file_end);let began=wphw_disc_clock();let p=terminated(&f.path);let file=libc::fopen(p.as_ptr().cast(),c"rb".as_ptr());if logging{wphw_disc_stderr(offset,length,destination,f.path.as_ptr(),f.path.len(),!file.is_null());}if !file.is_null(){libc::fseek(file,start.wrapping_sub(f.offset)as libc::c_long,libc::SEEK_SET);libc::fread(output.add(start.wrapping_sub(offset)as usize).cast(),1,stop.wrapping_sub(start)as usize,file);libc::fclose(file);}let spent=wphw_disc_spent(began);wphw_disc_log(stop.wrapping_sub(start)as u32,f.path.as_ptr(),f.path.len(),!file.is_null(),spent);index+=1;}}
