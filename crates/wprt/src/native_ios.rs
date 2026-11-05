// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/ios/ios.cpp; native leaves retain filesystem/BT/disc operations.
use crate::memory::{host,rd8,rd32,wr32};
use std::{collections::{BTreeMap,BTreeSet,VecDeque},ffi::{c_void,CStr},sync::{Mutex,OnceLock}};
// Constants copied programmatically from pristine ios.cpp and nand.h.
#[allow(non_upper_case_globals)]const kTitleIdHigh:u32=0x00010000;
#[allow(non_upper_case_globals)]const kGameIdAddress:u32=0x80000000;
#[allow(non_upper_case_globals)]const kEsGetTitleId:u32=0x20;
#[allow(non_upper_case_globals)]const kEsGetConsumption:u32=0x16;
#[allow(non_upper_case_globals)]const kEsGetDiscTicketView:u32=0x1B;
#[allow(non_upper_case_globals)]const kEsGetTitleDir:u32=0x1D;
#[allow(non_upper_case_globals)]const kStmEventHook:u32=0x1000;
#[allow(non_upper_case_globals)]const kStmReleaseEventHook:u32=0x3002;
#[allow(non_upper_case_globals)]const kStmVideoDimming:u32=0x5001;
#[allow(non_upper_case_globals)]const kKdSuspendScheduler:u32=0x01;
#[allow(non_upper_case_globals)]const kKdSetRtcCounter:u32=0x17;
#[allow(non_upper_case_globals)]const kIpcExists:i32=-2;
#[allow(non_upper_case_globals)]const kIpcNotFound:i32=-6;
#[allow(non_upper_case_globals)]const kEventHookDevice:&[u8]=b"/dev/stm/eventhook";
#[allow(non_upper_case_globals)]const kStmDevice:&[u8]=b"/dev/stm/immediate";
#[allow(non_upper_case_globals)]const kVectorSize:u32=8;
#[allow(non_upper_case_globals)]const kOpen:u32=1;
#[allow(non_upper_case_globals)]const kClose:u32=2;
#[allow(non_upper_case_globals)]const kRead:u32=3;
#[allow(non_upper_case_globals)]const kWrite:u32=4;
#[allow(non_upper_case_globals)]const kSeek:u32=5;
#[allow(non_upper_case_globals)]const kIoctl:u32=6;
#[allow(non_upper_case_globals)]const kIoctlv:u32=7;
#[allow(non_upper_case_globals)]const kDiInquiry:u32=0x12;
#[allow(non_upper_case_globals)]const kDiRead:u32=0x71;
#[allow(non_upper_case_globals)]const kDiUnencryptedRead:u32=0x8D;
#[allow(non_upper_case_globals)]const kDiReportKey:u32=0xA4;
#[allow(non_upper_case_globals)]const kDiRequestError:u32=0xE0;
#[allow(non_upper_case_globals)]const kDiClearCoverInterrupt:u32=0x86;
#[allow(non_upper_case_globals)]const kDiSuccess:u32=1;
#[allow(non_upper_case_globals)]const kDiError:u32=2;
#[allow(non_upper_case_globals)]const kDiErrorOutOfRange:u32=0x00052100;
#[allow(non_upper_case_globals)]const kDiErrorInvalidField:u32=0x00053100;
#[allow(non_upper_case_globals)]const kDiscSize:u64=0x118240000;
#[allow(non_upper_case_globals)]const kDriveInfoSize:u32=0x20;
#[allow(non_upper_case_globals)]const kFileSystem:&[u8]=b"/dev/fs";
#[allow(non_upper_case_globals)]const kFsCreateDirectory:u32=0x03;
#[allow(non_upper_case_globals)]const kFsGetAttributes:u32=0x06;
#[allow(non_upper_case_globals)]const kFsDelete:u32=0x07;
#[allow(non_upper_case_globals)]const kFsRename:u32=0x08;
#[allow(non_upper_case_globals)]const kFsCreateFile:u32=0x09;
#[allow(non_upper_case_globals)]const kFsGetFileStats:u32=0x0B;
#[allow(non_upper_case_globals)]const kFsReadDirectory:u32=0x04;
#[allow(non_upper_case_globals)]const kFsGetUsage:u32=0x0C;
#[allow(non_upper_case_globals)]const kFsClusterSize:u64=0x4000;
#[allow(non_upper_case_globals)]const kIpcOverheadTicks:u64=2700;
#[allow(non_upper_case_globals)]const kDefaultReplyTicks:u64=4000;
#[allow(non_upper_case_globals)]const kSuperblockWriteTicks:u64=3170000;
#[allow(non_upper_case_globals)]const kClusterWriteTicks:u64=300000;
#[allow(non_upper_case_globals)]const kClusterReadTicks:u64=115000;
#[allow(non_upper_case_globals)]const kFreeClusterCheckTicks:u64=1000;
#[allow(non_upper_case_globals)]const kInvalidPathTicks:u64=300;
#[allow(non_upper_case_globals)]const kLookupTicks:u64=680;
#[allow(non_upper_case_globals)]const kSplitLookupTicks:u64=1000;
#[allow(non_upper_case_globals)]const kSplitComponentTicks:u64=340;
#[allow(non_upper_case_globals)]const kFsPathOffset:u32=6;
#[allow(non_upper_case_globals)]const kFsPathSize:u32=64;
#[allow(non_upper_case_globals)]const kFsNameSize:u32=13;
#[allow(non_upper_case_globals)]const kNandInvalid:i32=-101;
type Entry=unsafe extern "C" fn(*mut c_void,*const c_void);
type Usage=unsafe extern "C" fn(*mut c_void,bool,u64);
extern "C" {
 fn wphw_ios_nand_open(p:*const u8,n:usize,mode:u32)->i32;
 fn wphw_ios_nand_seek(fd:i32,offset:i32,whence:i32)->i32;
 fn wphw_ios_nand_read(fd:i32,address:u32,size:u32)->i32;
 fn wphw_ios_nand_write(fd:i32,address:u32,size:u32)->i32;
 fn wphw_ios_nand_close(fd:i32);
 fn wphw_ios_nand_path(operation:u32,p:*const u8,n:usize,q:*const u8,m:usize)->i32;
 fn wphw_ios_directory(p:*const u8,n:usize,sink:Entry,context:*mut c_void);
 fn wphw_ios_copy_name(entry:*const c_void,address:u32);
 fn wphw_ios_usage(p:*const u8,n:usize,sink:Usage,context:*mut c_void)->i32;
 fn wphw_ios_bt_handles(p:*const u8,n:usize)->bool;
 fn wphw_ios_bt_close();fn wphw_ios_bt_update();
 fn wphw_ios_bt_completion(request:*mut u32,result:*mut i32)->bool;
 fn wphw_ios_bt_ioctlv(request:u32,command:u32,inputs:u32,outputs:u32,vectors:u32)->i32;
 fn wphw_ios_disc_read(offset:u64,length:u32,destination:u32);
 fn wphw_ios_format_once(kind:u32,p:*const u8,n:usize,command:u32,out:*mut u8);
 fn wphw_ios_print_once(p:*const u8);
 fn wphw_ios_log_open(p:*const u8,n:usize,result:i32,failed:bool);
 fn wphw_ios_log_request(request:u32);
 fn wphw_ios_log_unknown(command:u32,p:*const u8,n:usize);
 fn wphw_ios_title_path(high:u32,low:u32,out:*mut u8);
}
#[derive(Clone)]struct Device{path:Vec<u8>,file:i32,superblock_flush:bool}
struct State{devices:BTreeMap<i32,Device>,completions:VecDeque<(u32,i32)>,event_hook:u32,kd_rtc:u32,next_descriptor:i32,di_last_error:u32,cache_descriptor:i32,cache_chain:u32,dirty_cache:bool,seen:BTreeSet<Vec<u8>>}
// port of ios.cpp namespace globals. This mutex serializes native entry points; guest control stays in Rust.
fn state()->&'static Mutex<State>{static S:OnceLock<Mutex<State>>=OnceLock::new();S.get_or_init(||Mutex::new(State{devices:BTreeMap::new(),completions:VecDeque::new(),event_hook:0,kd_rtc:0,next_descriptor:1,di_last_error:0,cache_descriptor:-1,cache_chain:0,dirty_cache:false,seen:BTreeSet::new()}))}
// port of ios.cpp read_string: source reads the byte before testing its limit.
fn read_string(mut address:u32,limit:usize)->Vec<u8>{let mut s=Vec::new();loop{let c=rd8(address)as u8;if c==0||s.len()>=limit{break;}s.push(c);address=address.wrapping_add(1);}s}
// port of ios.cpp lookup_ticks / memcpy_ticks.
fn lookup_ticks(path:&[u8],split:bool)->u64{let n=path.iter().filter(|&&c|c==b'/').count()as u64;if n==0{return 0;}if path.last()==Some(&b'/'){return kInvalidPathTicks;}if split{kSplitLookupTicks+kSplitComponentTicks*n}else{kLookupTicks*n}}
fn memcpy_ticks(size:u32)->u64{(0.636f64*size as f64+150.0)as u64}
// port of ios.cpp log_once; native leaf only performs original snprintf/fprintf formatting.
fn log_once(s:&mut State,kind:u32,path:&[u8],command:u32){let mut text=[0u8;160];unsafe{wphw_ios_format_once(kind,path.as_ptr(),path.len(),command,text.as_mut_ptr());let key=CStr::from_ptr(text.as_ptr().cast()).to_bytes().to_vec();if s.seen.insert(key){wphw_ios_print_once(text.as_ptr());}}}
// port of ios.cpp has_cache / flush_cache / populate_cache.
fn has_cache(s:&State,fd:i32,offset:u32)->bool{s.cache_descriptor==fd&&s.cache_chain==offset/(kFsClusterSize as u32)}
fn flush_cache(s:&mut State)->u64{if s.cache_descriptor<0||!s.dirty_cache{return 0;}s.dirty_cache=false;if let Some(d)=s.devices.get_mut(&s.cache_descriptor){d.superblock_flush=true;}kClusterWriteTicks}
fn populate_cache(s:&mut State,fd:i32,offset:u32,size:u32)->u64{if has_cache(s,fd,offset){return 0;}let mut ticks=flush_cache(s);if (offset as u64%kFsClusterSize!=0||offset!=size)&&offset<size{ticks+=kClusterReadTicks;}s.cache_descriptor=fd;s.cache_chain=(offset as u64/kFsClusterSize)as u32;ticks}
// port of ios.cpp read_write_ticks; preserve shared-cache and superblock side effects in source order.
unsafe fn read_write_ticks(s:&mut State,fd:i32,write:bool,size:u32)->u64{let file=s.devices[&fd].file;let position=wphw_ios_nand_seek(file,0,1);let end=wphw_ios_nand_seek(file,0,2);wphw_ios_nand_seek(file,position,0);let mut offset=position as u32;let file_size=end as u32;let mut count=size;if !write&&count.wrapping_add(offset)>file_size{count=if file_size>offset{file_size-offset}else{0};}let mut ticks=0;while count!=0{let length;if !has_cache(s,fd,offset)&&count as u64>=kFsClusterSize&&offset as u64%kFsClusterSize==0{ticks+=if write{kClusterWriteTicks}else{kClusterReadTicks};length=kFsClusterSize as u32;if write{s.devices.get_mut(&fd).unwrap().superblock_flush=true;}}else{ticks+=populate_cache(s,fd,offset,file_size);let start=offset.wrapping_sub(s.cache_chain.wrapping_mul(kFsClusterSize as u32));length=(kFsClusterSize as u32).wrapping_sub(start).min(count);ticks+=memcpy_ticks(length);if write{ticks+=kFreeClusterCheckTicks;}s.dirty_cache=write;if write&&offset.wrapping_add(length)as u64%kFsClusterSize==0{ticks+=flush_cache(s);}}offset=offset.wrapping_add(length);count=count.wrapping_sub(length);}ticks}
// port of ios.cpp device_name.
fn device_name(s:&State,fd:i32)->Vec<u8>{s.devices.get(&fd).map(|d|d.path.clone()).unwrap_or_else(||b"?".to_vec())}
// port of ios.cpp di_command.
unsafe fn di_command(s:&mut State,command:u32,input:u32,output:u32)->i32{match command{
 kDiInquiry=>{std::ptr::write_bytes(host(output),0,kDriveInfoSize as usize);kDiSuccess as i32},
 kDiClearCoverInterrupt=>kDiSuccess as i32,
 kDiRead=>{wphw_ios_disc_read((rd32(input.wrapping_add(8))as u64)<<2,rd32(input.wrapping_add(4)),output);kDiSuccess as i32},
 kDiUnencryptedRead=>{let end=(rd32(input.wrapping_add(8))as u64+rd32(input.wrapping_add(4))as u64/4)*4;if end>kDiscSize{s.di_last_error=kDiErrorOutOfRange;kDiError as i32}else{kDiSuccess as i32}},
 kDiReportKey=>{s.di_last_error=kDiErrorInvalidField;kDiError as i32},
 kDiRequestError=>{wr32(output,s.di_last_error);s.di_last_error=0;kDiSuccess as i32},
 _=>{log_once(s,0,b"/dev/di",command);kDiSuccess as i32}
}}
// port of ios.cpp fs_command. Native path operation dispatch uses the original FS command constants.
unsafe fn fs_command(s:&mut State,command:u32,input:u32,output:u32,ticks:&mut u64)->i32{match command{
 kFsCreateDirectory=>{let p=read_string(input.wrapping_add(kFsPathOffset),kFsPathSize as usize);let result=wphw_ios_nand_path(command,p.as_ptr(),p.len(),std::ptr::null(),0);if result==0{*ticks+=kSuperblockWriteTicks;}result},
 kFsCreateFile=>{*ticks+=kSuperblockWriteTicks;let p=read_string(input.wrapping_add(kFsPathOffset),kFsPathSize as usize);wphw_ios_nand_path(command,p.as_ptr(),p.len(),std::ptr::null(),0)},
 kFsDelete=>{*ticks+=kSuperblockWriteTicks;let p=read_string(input,kFsPathSize as usize);wphw_ios_nand_path(command,p.as_ptr(),p.len(),std::ptr::null(),0)},
 kFsRename=>{*ticks+=kSuperblockWriteTicks;let p=read_string(input,kFsPathSize as usize);let q=read_string(input.wrapping_add(kFsPathSize),kFsPathSize as usize);wphw_ios_nand_path(command,p.as_ptr(),p.len(),q.as_ptr(),q.len())},
 kFsGetAttributes=>{let p=read_string(input,kFsPathSize as usize);*ticks+=lookup_ticks(&p,true);std::ptr::write_bytes(host(output),0,12);wphw_ios_nand_path(command,p.as_ptr(),p.len(),std::ptr::null(),0)},
 _=>{log_once(s,0,b"/dev/fs",command);0}
}}
struct Directory{names:u32,maximum:u32,count:u32}
// port of ios.cpp fs_vector_command directory loop body; keep resize-to-12 including zero padding.
unsafe extern "C" fn directory_entry(context:*mut c_void,entry:*const c_void){let d=&mut*context.cast::<Directory>();if d.names!=0&&(d.maximum==0||d.count<d.maximum){wphw_ios_copy_name(entry,d.names.wrapping_add(d.count.wrapping_mul(kFsNameSize)));}d.count=d.count.wrapping_add(1);}
struct UsageCount{clusters:u32,inodes:u32}
// port of ios.cpp fs_vector_command usage loop body; native filesystem iterator/order/errors retained.
unsafe extern "C" fn usage_entry(context:*mut c_void,regular:bool,size:u64){let c=&mut*context.cast::<UsageCount>();c.inodes=c.inodes.wrapping_add(1);if regular{c.clusters=c.clusters.wrapping_add((size.wrapping_add(kFsClusterSize-1)/kFsClusterSize)as u32);}}
// port of ios.cpp fs_vector_command.
unsafe fn fs_vector_command(s:&mut State,command:u32,inputs:u32,vectors:u32)->i32{if command==kFsReadDirectory{let path=read_string(rd32(vectors),kFsPathSize as usize);let mut d=Directory{names:rd32(vectors.wrapping_add(inputs.wrapping_mul(kVectorSize))),maximum:if inputs>1{rd32(rd32(vectors.wrapping_add(kVectorSize)))}else{0},count:0};let count=rd32(vectors.wrapping_add(inputs.wrapping_add(1).wrapping_mul(kVectorSize)));wphw_ios_directory(path.as_ptr(),path.len(),directory_entry,(&mut d as *mut Directory).cast());wr32(count,d.count);return 0;}if command==kFsGetUsage{let path=read_string(rd32(vectors),kFsPathSize as usize);let mut c=UsageCount{clusters:0,inodes:1};let result=wphw_ios_usage(path.as_ptr(),path.len(),usage_entry,(&mut c as *mut UsageCount).cast());if result!=0{return result;}wr32(rd32(vectors.wrapping_add(inputs.wrapping_mul(kVectorSize))),c.clusters);wr32(rd32(vectors.wrapping_add(inputs.wrapping_add(1).wrapping_mul(kVectorSize))),c.inodes);return 0;}log_once(s,1,b"/dev/fs",command);0}
// port of ios.cpp open.
unsafe fn open(s:&mut State,path:&[u8],mode:u32)->i32{let mut device=Device{path:path.to_vec(),file:-1,superblock_flush:false};if !path.starts_with(b"/dev/"){let file=wphw_ios_nand_open(path.as_ptr(),path.len(),mode);if file<0{wphw_ios_log_open(path.as_ptr(),path.len(),file,true);return file;}device.file=file;}wphw_ios_log_open(path.as_ptr(),path.len(),0,false);let descriptor=s.next_descriptor;s.next_descriptor=s.next_descriptor.wrapping_add(1);s.devices.insert(descriptor,device);descriptor}
// port of ios.cpp ioctl; input/output sizes are intentionally unused by the original.
unsafe fn ioctl(s:&mut State,fd:i32,command:u32,input:u32,output:u32)->i32{let path=device_name(s,fd);if path==b"/dev/di"{return di_command(s,command,input,output);}if let Some(d)=s.devices.get(&fd){if d.file>=0&&command==kFsGetFileStats{let position=wphw_ios_nand_seek(d.file,0,1);let size=wphw_ios_nand_seek(d.file,0,2);wphw_ios_nand_seek(d.file,position,0);wr32(output,size as u32);wr32(output.wrapping_add(4),position as u32);return 0;}}if path==kStmDevice&&command==kStmVideoDimming{return 0;}if path==b"/dev/net/kd/request"&&command==kKdSuspendScheduler{wr32(output,0);return 0;}if path==b"/dev/net/kd/time"&&command==kKdSetRtcCounter{s.kd_rtc=rd32(input);wr32(output,0);return 0;}log_once(s,0,&path,command);0}
// port of ios.cpp ioctlv; output count intentionally unused by the original.
unsafe fn ioctlv(s:&mut State,fd:i32,command:u32,inputs:u32,vectors:u32)->i32{let path=device_name(s,fd);let out_slot=vectors.wrapping_add(inputs.wrapping_mul(kVectorSize));if path==b"/dev/es"&&command==kEsGetTitleDir{let input=rd32(vectors);let mut text=[0u8;32];wphw_ios_title_path(rd32(input),rd32(input.wrapping_add(4)),text.as_mut_ptr());let n=CStr::from_ptr(text.as_ptr().cast()).to_bytes_with_nul().len();std::ptr::copy_nonoverlapping(text.as_ptr(),host(rd32(out_slot)),n);return 0;}if path==b"/dev/es"&&command==kEsGetTitleId{let output=rd32(out_slot);wr32(output,kTitleIdHigh);wr32(output.wrapping_add(4),rd32(kGameIdAddress));return 0;}if path==b"/dev/es"&&command==kEsGetConsumption{wr32(rd32(vectors.wrapping_add(inputs.wrapping_add(1).wrapping_mul(kVectorSize))),0);return 0;}if path==b"/dev/es"&&command==kEsGetDiscTicketView{std::ptr::write_bytes(host(rd32(out_slot)),0,rd32(out_slot.wrapping_add(4))as usize);return 0;}if path==b"/dev/fs"{return fs_vector_command(s,command,inputs,vectors);}log_once(s,1,&path,command);0}
// port of ios.cpp send, including early deferred returns which deliberately skip request+4 writeback.
#[no_mangle]pub unsafe extern "C" fn wprt_ios_send(request:u32,ticks:*mut u64)->i32{let mut guard=state().lock().unwrap();let s=&mut*guard;let command=rd32(request);let fd=rd32(request.wrapping_add(8))as i32;static LOG:OnceLock<bool>=OnceLock::new();if *LOG.get_or_init(||!libc::getenv(c"WP_LOG_IOS".as_ptr()).is_null()){wphw_ios_log_request(request);}let device=s.devices.get(&fd).cloned();
#[cfg(feature="idle-nap")] if command==kOpen||!device.as_ref().is_some_and(|d|wphw_ios_bt_handles(d.path.as_ptr(),d.path.len())){crate::interrupts::note_io_activity();}
let file=device.as_ref().is_some_and(|d|d.file>=0);let fs=device.as_ref().is_some_and(|d|d.path==kFileSystem);*ticks=if file||fs{kIpcOverheadTicks}else{kDefaultReplyTicks};let mut result=0;match command{
 kOpen=>{let path=read_string(rd32(request.wrapping_add(12)),256);*ticks=if !path.starts_with(b"/dev/"){kIpcOverheadTicks+lookup_ticks(&path,false)}else if path==kFileSystem{kIpcOverheadTicks}else{kDefaultReplyTicks};result=open(s,&path,rd32(request.wrapping_add(16)));},
 kClose=>{if let Some(d)=device{if file{if s.cache_descriptor==fd{*ticks+=flush_cache(s);s.cache_descriptor=-1;}if s.devices[&fd].superblock_flush{*ticks+=kSuperblockWriteTicks;}}if wphw_ios_bt_handles(d.path.as_ptr(),d.path.len()){wphw_ios_bt_close();}wphw_ios_nand_close(d.file);s.devices.remove(&fd);}},
 kRead|kWrite=>{let write=command==kWrite;let size=rd32(request.wrapping_add(16));if file{*ticks+=read_write_ticks(s,fd,write,size);}result=if let Some(d)=device{if write{wphw_ios_nand_write(d.file,rd32(request.wrapping_add(12)),size)}else{wphw_ios_nand_read(d.file,rd32(request.wrapping_add(12)),size)}}else{kNandInvalid};},
 kSeek=>{result=if let Some(d)=device{wphw_ios_nand_seek(d.file,rd32(request.wrapping_add(12))as i32,rd32(request.wrapping_add(16))as i32)}else{kNandInvalid};},
 kIoctl=>{if device.as_ref().is_some_and(|d|d.path==kEventHookDevice)&&rd32(request.wrapping_add(12))==kStmEventHook{if s.event_hook!=0{result=kIpcExists;}else{s.event_hook=request;return i32::MIN;}}else if device.as_ref().is_some_and(|d|d.path==kStmDevice)&&rd32(request.wrapping_add(12))==kStmReleaseEventHook{if s.event_hook==0{result=kIpcNotFound;}else{wr32(rd32(s.event_hook.wrapping_add(24)),0);s.completions.push_back((s.event_hook,0));s.event_hook=0;}}else{result=if fs{fs_command(s,rd32(request.wrapping_add(12)),rd32(request.wrapping_add(16)),rd32(request.wrapping_add(24)),&mut*ticks)}else{ioctl(s,fd,rd32(request.wrapping_add(12)),rd32(request.wrapping_add(16)),rd32(request.wrapping_add(24)))};}},
 kIoctlv=>{let command=rd32(request.wrapping_add(12));let inputs=rd32(request.wrapping_add(16));let outputs=rd32(request.wrapping_add(20));let vectors=rd32(request.wrapping_add(24));if let Some(d)=device.filter(|d|wphw_ios_bt_handles(d.path.as_ptr(),d.path.len())){let _=d;result=wphw_ios_bt_ioctlv(request,command,inputs,outputs,vectors);if result==i32::MIN{return result;}}else{result=ioctlv(s,fd,command,inputs,vectors);}},
 _=>{let path=device_name(s,fd);wphw_ios_log_unknown(command,path.as_ptr(),path.len());}
}wr32(request.wrapping_add(4),result as u32);result}
// port of ios.cpp public entry points; no guest-control unwind is introduced through a native leaf.
#[no_mangle]pub unsafe extern "C" fn wprt_ios_update(){wphw_ios_bt_update();}
#[no_mangle]pub unsafe extern "C" fn wprt_ios_take_completion(request:*mut u32,result:*mut i32)->bool{let mut s=state().lock().unwrap();if let Some((r,v))=s.completions.pop_front(){*request=r;*result=v;}else if !wphw_ios_bt_completion(request,result){return false;}wr32((*request).wrapping_add(4),*result as u32);true}
#[no_mangle]pub unsafe extern "C" fn wprt_ios_open(p:*const u8,n:usize,mode:u32)->i32{open(&mut state().lock().unwrap(),std::slice::from_raw_parts(p,n),mode)}
#[no_mangle]pub unsafe extern "C" fn wprt_ios_ioctl(fd:i32,command:u32,input:u32,_input_size:u32,output:u32,_output_size:u32)->i32{ioctl(&mut state().lock().unwrap(),fd,command,input,output)}
#[no_mangle]pub unsafe extern "C" fn wprt_ios_ioctlv(fd:i32,command:u32,inputs:u32,_outputs:u32,vectors:u32)->i32{ioctlv(&mut state().lock().unwrap(),fd,command,inputs,vectors)}
