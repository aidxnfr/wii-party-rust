// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/ios/nand.cpp. Windows filesystem/CRT primitives retain native semantics.
use crate::{memory::{host,rd8,rd32},native_settings};
use std::{collections::BTreeMap,ffi::{c_char,c_void},sync::{Mutex,OnceLock}};
// Constants extracted from pristine nand.cpp/nand.h and installed MinGW winnt.h.
#[allow(non_upper_case_globals)]const kModeRead:u32=1;
#[allow(non_upper_case_globals)]const kModeWrite:u32=2;
#[allow(non_upper_case_globals)]const kSysconfSize:usize=0x4000;
#[allow(non_upper_case_globals)]const kTypeBigArray:u8=1;
#[allow(non_upper_case_globals)]const kTypeSmallArray:u8=2;
#[allow(non_upper_case_globals)]const kTypeByte:u8=3;
#[allow(non_upper_case_globals)]const kTypeLong:u8=5;
#[allow(non_upper_case_globals)]const kTypeBool:u8=7;
#[allow(non_upper_case_globals)]const kLanguageEnglish:u8=1;
#[allow(non_upper_case_globals)]const kLanguageGerman:u8=2;
#[allow(non_upper_case_globals)]const kLanguageFrench:u8=3;
#[allow(non_upper_case_globals)]const kLanguageSpanish:u8=4;
#[allow(non_upper_case_globals)]const kLanguageItalian:u8=5;
#[allow(non_upper_case_globals)]const kLanguageDutch:u8=6;
#[allow(non_upper_case_globals)]const kGameIdAddress:u32=0x80000000;
#[allow(non_upper_case_globals)]const kDefaultBackups:i32=5;
#[allow(non_upper_case_globals)]const kNotFound:i32=-106;
#[allow(non_upper_case_globals)]const kExists:i32=-105;
#[allow(non_upper_case_globals)]const kInvalid:i32=-101;
const LANG_GERMAN:u16=0x07;
const LANG_FRENCH:u16=0x0c;
const LANG_SPANISH:u16=0x0a;
const LANG_ITALIAN:u16=0x10;
const LANG_DUTCH:u16=0x13;
type Path=Vec<u16>;
type WideSink=unsafe extern "C" fn(*mut c_void,*const u16,usize);
type ByteSink=unsafe extern "C" fn(*mut c_void,*const u8,usize);
extern "C"{
 fn wphw_nand_path_bytes(p:*const u8,n:usize,sink:WideSink,context:*mut c_void);
 fn wphw_nand_path_join(a:*const u16,an:usize,b:*const u16,bn:usize,sink:WideSink,context:*mut c_void);
 fn wphw_nand_path_components(p:*const u16,n:usize,relative:bool,sink:WideSink,context:*mut c_void);
 fn wphw_nand_path_string(p:*const u16,n:usize,generic:bool,sink:ByteSink,context:*mut c_void);
 fn wphw_nand_parent(p:*const u16,n:usize,sink:WideSink,context:*mut c_void);
 fn wphw_nand_lower(c:u16)->u16;
 fn wphw_nand_fs_query(operation:u32,p:*const u16,n:usize)->bool;
 fn wphw_nand_fs_create(p:*const u16,n:usize)->bool;
 fn wphw_nand_fs_remove(p:*const u16,n:usize)->u64;
 fn wphw_nand_fs_rename(a:*const u16,an:usize,b:*const u16,bn:usize)->bool;
 fn wphw_nand_save_relative(title:u32,sink:WideSink,context:*mut c_void);
 fn wphw_nand_backups_beside(p:*const u16,n:usize,sink:WideSink,context:*mut c_void);
 fn wphw_nand_adopt(root:*const u16,rn:usize,save:*const u16,sn:usize,sink:ByteSink,context:*mut c_void);
 fn wphw_nand_back_up(root:*const u16,rn:usize,save:*const u16,sn:usize,backups:*const u16,bn:usize,keep:i32,sink:ByteSink,context:*mut c_void)->i32;
 fn wphw_nand_steady_ns()->i64;
 fn wphw_nand_language()->u16;
 fn wphw_nand_log_backup(moment:*const c_char,result:i32,message:*const u8,n:usize);
 fn wphw_nand_log_adopt(name:*const u8,n:usize,save:*const u16,sn:usize);
}
unsafe extern "C" fn wide(context:*mut c_void,p:*const u16,n:usize){*context.cast::<Path>()=std::slice::from_raw_parts(p,n).to_vec();}
unsafe extern "C" fn bytes(context:*mut c_void,p:*const u8,n:usize){*context.cast::<Vec<u8>>()=std::slice::from_raw_parts(p,n).to_vec();}
unsafe extern "C" fn component(context:*mut c_void,p:*const u16,n:usize){(*context.cast::<Vec<Path>>()).push(std::slice::from_raw_parts(p,n).to_vec());}
// Native path primitives transport wide Windows path values; Rust owns all persistent state and decisions.
unsafe fn path_bytes(p:&[u8])->Path{let mut out=Vec::new();wphw_nand_path_bytes(p.as_ptr(),p.len(),wide,(&mut out as *mut Path).cast());out}
unsafe fn join(a:&[u16],b:&[u16])->Path{let mut out=Vec::new();wphw_nand_path_join(a.as_ptr(),a.len(),b.as_ptr(),b.len(),wide,(&mut out as *mut Path).cast());out}
unsafe fn components(p:&[u16],relative:bool)->Vec<Path>{let mut out=Vec::new();wphw_nand_path_components(p.as_ptr(),p.len(),relative,component,(&mut out as *mut Vec<Path>).cast());out}
unsafe fn path_string(p:&[u16])->Vec<u8>{let mut out=Vec::new();wphw_nand_path_string(p.as_ptr(),p.len(),false,bytes,(&mut out as *mut Vec<u8>).cast());out}
unsafe fn parent(p:&[u16])->Path{let mut out=Vec::new();wphw_nand_parent(p.as_ptr(),p.len(),wide,(&mut out as *mut Path).cast());out}
fn terminated(p:&[u8])->Vec<u8>{let mut out=p.to_vec();out.push(0);out}
unsafe fn fopen(path:&[u16],mode:*const c_char)->*mut libc::FILE{let p=terminated(&path_string(path));libc::fopen(p.as_ptr().cast(),mode)}
unsafe fn setting(key:*const c_char,var:*const c_char)->Vec<u8>{let mut out=Vec::new();native_settings::wprt_settings_text(key,var,bytes,(&mut out as *mut Vec<u8>).cast());out}
struct OpenFile{file:usize,path:Path,written:bool}
#[derive(Default)]struct State{root:Path,files:BTreeMap<i32,OpenFile>,next_handle:i32,save:Path,backups:Path,backup_pending:bool,backed_up:bool,last_backup:i64}
// port of nand.cpp namespace globals. next_handle starts at1; mount does not reset existing handles/pending state.
fn state()->&'static Mutex<State>{static S:OnceLock<Mutex<State>>=OnceLock::new();S.get_or_init(||Mutex::new(State{next_handle:1,..State::default()}))}
// port of nand.cpp host_path: relative_path iteration skips literal dot/dotdot components, not canonicalization.
unsafe fn host_path(s:&State,p:&[u8])->Path{let mut relative=Vec::new();for part in components(&path_bytes(p),true){if part!=[b'.' as u16]&&part!=[b'.' as u16,b'.' as u16]{relative=join(&relative,&part);}}join(&s.root,&relative)}
// port of nand.cpp backups_to_keep: preserve native long parsing, trailing-byte check and clamp.
unsafe fn backups_to_keep()->i32{let value=setting(c"saves.backups".as_ptr(),c"WP_SAVE_BACKUPS".as_ptr());let input=terminated(&value);let mut end=std::ptr::null_mut();let number=libc::strtol(input.as_ptr().cast(),&mut end,10);if value.is_empty()||*end!=0{return kDefaultBackups;}number.clamp(0,1000)as i32}
// port of nand.cpp back_up_save. Native helper returns original saves::Result enum and message, not decisions.
unsafe fn back_up_save(s:&mut State,moment:*const c_char){let keep=backups_to_keep();if keep==0||s.save.is_empty(){return;}s.backup_pending=false;let mut message=Vec::<u8>::new();let result=wphw_nand_back_up(s.root.as_ptr(),s.root.len(),s.save.as_ptr(),s.save.len(),s.backups.as_ptr(),s.backups.len(),keep,bytes,(&mut message as *mut Vec<u8>).cast());if result!=2&&result!=1{s.backed_up=true;s.last_backup=wphw_nand_steady_ns();}wphw_nand_log_backup(moment,result,message.as_ptr(),message.len());}
// port of nand.cpp same_element and inside_save; native towlower retains installed Windows locale semantics.
unsafe fn same_element(a:&[u16],b:&[u16])->bool{a.len()==b.len()&&a.iter().zip(b).all(|(&x,&y)|wphw_nand_lower(x)==wphw_nand_lower(y))}
unsafe fn inside_save(s:&State,path:&[u16])->bool{if s.save.is_empty(){return false;}let folder=components(&join(&s.root,&s.save),false);let parts=components(path,false);if parts.len()<folder.len(){return false;}folder.iter().zip(&parts).all(|(a,b)|same_element(a,b))}
// port of nand.cpp save_changed; test writing before sampling the rate-limit clock, as in source short circuit.
unsafe fn save_changed(s:&mut State){if backups_to_keep()==0{return;}let writing=s.files.values().any(|f|f.written&&inside_save(s,&f.path));if writing||(s.backed_up&&wphw_nand_steady_ns().wrapping_sub(s.last_backup)<60_000_000_000){s.backup_pending=true;return;}back_up_save(s,c"after a write".as_ptr());}
// port of nand.cpp append16 / add_item.
fn append16(out:&mut Vec<u8>,value:u16){out.push((value>>8)as u8);out.push(value as u8);}
fn add_item(items:&mut Vec<Vec<u8>>,kind:u8,name:&[u8],data:&[u8]){let mut item=Vec::new();item.push(((kind as usize)<<5|name.len().wrapping_sub(1))as u8);item.extend_from_slice(name);if kind==kTypeBigArray{append16(&mut item,data.len().wrapping_sub(1)as u16);}else if kind==kTypeSmallArray{item.push(data.len().wrapping_sub(1)as u8);}item.extend_from_slice(data);items.push(item);}
// port of nand.cpp paired_remotes. System bootstrap values are copied from source, not game assets.
fn paired_remotes()->Vec<u8>{let device_size=6+0x40;let registered=10;let active=5;let remotes=4u8;let name=b"Nintendo RVL-CNT-01";let mut data=vec![0;1+device_size*(registered+active+1)];data[0]=remotes;for i in 0..remotes{let address=[i,0x00,0x79,0x19,0x02,0x11];for slot in [1+device_size*i as usize,1+device_size*(registered+i as usize)]{data[slot..slot+6].copy_from_slice(&address);data[slot+6..slot+6+name.len()].copy_from_slice(name);}}data}
// port of nand.cpp pal60 / console_area / console_language (Windows branch).
unsafe fn pal60()->u8{native_settings::wprt_settings_flag(c"system.pal60".as_ptr(),c"WP_PAL60".as_ptr())as u8}
fn console_area()->(Vec<u8>,Vec<u8>){match rd8(kGameIdAddress+3)as u8{b'E'=>(b"USA\0".to_vec(),b"LU\0\0".to_vec()),b'J'=>(b"JPN\0".to_vec(),b"LJ\0\0".to_vec()),b'K'=>(b"KOR\0".to_vec(),b"LKH\0".to_vec()),_=>(b"EUR\0".to_vec(),b"LEH\0".to_vec())}}
unsafe fn console_language()->u8{let names:[(&[u8],u8);6]=[(b"en",kLanguageEnglish),(b"de",kLanguageGerman),(b"fr",kLanguageFrench),(b"es",kLanguageSpanish),(b"it",kLanguageItalian),(b"nl",kLanguageDutch)];let value=setting(c"system.language".as_ptr(),c"WP_LANGUAGE".as_ptr());for (code,language)in names{if value==code{return language;}}match wphw_nand_language(){LANG_GERMAN=>kLanguageGerman,LANG_FRENCH=>kLanguageFrench,LANG_SPANISH=>kLanguageSpanish,LANG_ITALIAN=>kLanguageItalian,LANG_DUTCH=>kLanguageDutch,_=>kLanguageEnglish}}
// port of nand.cpp default_sysconf; ordered item calls copied programmatically from source.
unsafe fn default_sysconf()->Vec<u8>{let mut items=Vec::new();
add_item(&mut items,kTypeBigArray,b"BT.DINF",&paired_remotes());
add_item(&mut items,kTypeBigArray,b"BT.CDIF",&vec![0;0x205]);
add_item(&mut items,kTypeLong,b"BT.SENS",&[0, 0, 0, 3]);
add_item(&mut items,kTypeByte,b"BT.BAR",&[1]);
add_item(&mut items,kTypeByte,b"BT.SPKV",&[0x58]);
add_item(&mut items,kTypeByte,b"BT.MOT",&[1]);
add_item(&mut items,kTypeSmallArray,b"IPL.NIK",&[0, b'w', 0, b'i', 0, b'i', 0, b'p', 0, b'a', 0, b'r', 0, b't', 0, b'y']);
add_item(&mut items,kTypeByte,b"IPL.LNG",&[console_language()]);
let mut address=vec![0;0x1008];address[0]=0x6c;
add_item(&mut items,kTypeBigArray,b"IPL.SADR",&address);
let mut parental=vec![0;0x4a];parental[1]=0x04;parental[2]=0x14;
add_item(&mut items,kTypeSmallArray,b"IPL.PC",&parental);
add_item(&mut items,kTypeLong,b"IPL.CB",&[0, 0, 0, 0]);
add_item(&mut items,kTypeByte,b"IPL.AR",&[1]);
add_item(&mut items,kTypeByte,b"IPL.SSV",&[1]);
add_item(&mut items,kTypeBool,b"IPL.CD",&[0]);
add_item(&mut items,kTypeBool,b"IPL.CD2",&[0]);
add_item(&mut items,kTypeBool,b"IPL.EULA",&[1]);
add_item(&mut items,kTypeByte,b"IPL.UPT",&[2]);
add_item(&mut items,kTypeByte,b"IPL.PGS",&[0]);
add_item(&mut items,kTypeByte,b"IPL.E60",&[pal60()]);
add_item(&mut items,kTypeByte,b"IPL.DH",&[0]);
add_item(&mut items,kTypeLong,b"IPL.INC",&[0, 0, 0, 8]);
add_item(&mut items,kTypeLong,b"IPL.FRC",&[0, 0, 0, 0x28]);
add_item(&mut items,kTypeSmallArray,b"IPL.IDL",&[0, 1]);
add_item(&mut items,kTypeByte,b"IPL.SND",&[1]);
let (area_area,area_code)=console_area();
add_item(&mut items,kTypeSmallArray,b"IPL.AREA",&area_area);
add_item(&mut items,kTypeSmallArray,b"IPL.CODE",&area_code);
add_item(&mut items,kTypeLong,b"NET.WCFG",&[0, 0, 0, 1]);
add_item(&mut items,kTypeLong,b"NET.CTPC",&[0, 0, 0, 0]);
add_item(&mut items,kTypeByte,b"WWW.RST",&[0]);
add_item(&mut items,kTypeBool,b"MPLS.MOVIE",&[1]);
let mut file=b"SCv0".to_vec();append16(&mut file,items.len()as u16);let mut position=4+2+2*(items.len()+1);for item in &items{append16(&mut file,position as u16);position+=item.len();}append16(&mut file,position as u16);for item in items{file.extend_from_slice(&item);}file.resize(kSysconfSize,0);file[kSysconfSize-4..].copy_from_slice(b"SCed");file}
// port of nand.cpp read16 / valid_sysconf.
fn read16(data:&[u8],offset:usize)->u16{((data[offset]as u16)<<8)|data[offset+1]as u16}
unsafe fn valid_sysconf(path:&[u16])->bool{let file=fopen(path,c"rb".as_ptr());if file.is_null(){return false;}let mut data=vec![0;kSysconfSize];let count=libc::fread(data.as_mut_ptr().cast(),1,data.len(),file);libc::fclose(file);if count!=kSysconfSize||&data[..4]!=b"SCv0"||&data[kSysconfSize-4..]!=b"SCed"{return false;}let entries=read16(&data,4)as usize;let table_end=6+2*(entries+1);if table_end>kSysconfSize-4{return false;}let mut previous=table_end;for i in 0..=entries{let offset=read16(&data,6+2*i)as usize;if offset<previous||offset>kSysconfSize-4{return false;}previous=offset;}for name in [b"BT.DINF".as_slice(),b"BT.CDIF",b"IPL.SADR"]{if !data.windows(name.len()).any(|w|w==name){return false;}}true}
// port of nand.cpp set_byte_item / set_array_item: first matching item returns even when unchanged.
unsafe fn set_byte_item(data:&[u8],count:usize,name:&[u8],value:u8,file:*mut libc::FILE){let length=name.len();for i in 1..count.saturating_sub(length){if data[i-1]==(((kTypeByte as usize)<<5)|(length-1))as u8&&&data[i..i+length]==name{if data[i+length]!=value{libc::fseek(file,(i+length)as libc::c_long,libc::SEEK_SET);libc::fwrite((&value as *const u8).cast(),1,1,file);}return;}}}
unsafe fn set_array_item(data:&[u8],count:usize,name:&[u8],value:&[u8],file:*mut libc::FILE){let length=name.len();let mut i=1;while i+length+1+value.len()<=count{if data[i-1]==(((kTypeSmallArray as usize)<<5)|(length-1))as u8&&&data[i..i+length]==name{let start=i+length+1;if data[i+length]as usize+1==value.len()&&&data[start..start+value.len()]!=value{libc::fseek(file,start as libc::c_long,libc::SEEK_SET);libc::fwrite(value.as_ptr().cast(),1,value.len(),file);}return;}i+=1;}}
// port of nand.cpp apply_settings / ensure_sysconf.
unsafe fn apply_settings(path:&[u16]){let file=fopen(path,c"r+b".as_ptr());if file.is_null(){return;}let mut data=vec![0;kSysconfSize];let count=libc::fread(data.as_mut_ptr().cast(),1,data.len(),file);set_byte_item(&data,count,b"IPL.LNG",console_language(),file);set_byte_item(&data,count,b"IPL.E60",pal60(),file);let(area,code)=console_area();set_array_item(&data,count,b"IPL.AREA",&area,file);set_array_item(&data,count,b"IPL.CODE",&code,file);libc::fclose(file);}
unsafe fn ensure_sysconf(s:&State){let path=host_path(s,b"/shared2/sys/SYSCONF");if wphw_nand_fs_query(0,path.as_ptr(),path.len())&&valid_sysconf(&path){apply_settings(&path);return;}let folder=parent(&path);wphw_nand_fs_create(folder.as_ptr(),folder.len());let data=default_sysconf();let file=fopen(&path,c"wb".as_ptr());if !file.is_null(){libc::fwrite(data.as_ptr().cast(),1,data.len(),file);libc::fclose(file);}}
// port of nand.cpp mount. Native helpers preserve saves::* operations and filesystem representation only.
unsafe extern "C" fn adopted(context:*mut c_void,p:*const u8,n:usize){let save=&*context.cast::<Path>();wphw_nand_log_adopt(p,n,save.as_ptr(),save.len());}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_mount(p:*const u8,n:usize)->bool{let mut s=state().lock().unwrap();s.root=path_bytes(std::slice::from_raw_parts(p,n));if !wphw_nand_fs_create(s.root.as_ptr(),s.root.len()){return false;}ensure_sysconf(&s);wphw_nand_save_relative(rd32(kGameIdAddress),wide,(&mut s.save as *mut Path).cast());let folder=join(&s.root,&s.save);wphw_nand_fs_create(folder.as_ptr(),folder.len());wphw_nand_adopt(s.root.as_ptr(),s.root.len(),s.save.as_ptr(),s.save.len(),adopted,(&s.save as *const Path).cast_mut().cast());wphw_nand_backups_beside(s.root.as_ptr(),s.root.len(),wide,(&mut s.backups as *mut Path).cast());s.backed_up=false;back_up_save(&mut s,c"start".as_ptr());true}
// port of nand.cpp host_directory.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_host_directory(p:*const u8,n:usize,sink:ByteSink,context:*mut c_void){let s=state().lock().unwrap();let path=host_path(&s,std::slice::from_raw_parts(p,n));wphw_nand_path_string(path.as_ptr(),path.len(),false,sink,context);}
// port of nand.cpp widescreen: retain first raw name match, without interpreting/revalidating its item type.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_widescreen()->bool{let s=state().lock().unwrap();let path=host_path(&s,b"/shared2/sys/SYSCONF");let file=fopen(&path,c"rb".as_ptr());if file.is_null(){return false;}let mut data=vec![0;kSysconfSize];let count=libc::fread(data.as_mut_ptr().cast(),1,data.len(),file);libc::fclose(file);let name=b"IPL.AR";let mut i=0;while i+name.len()<count{if &data[i..i+name.len()]==name{return data[i+name.len()]==1;}i+=1;}false}
// port of nand.cpp exists / open.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_exists(p:*const u8,n:usize)->bool{let s=state().lock().unwrap();let target=host_path(&s,std::slice::from_raw_parts(p,n));wphw_nand_fs_query(0,target.as_ptr(),target.len())}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_open(p:*const u8,n:usize,mode:u32)->i32{let mut s=state().lock().unwrap();let target=host_path(&s,std::slice::from_raw_parts(p,n));if !wphw_nand_fs_query(1,target.as_ptr(),target.len()){return kNotFound;}let file=fopen(&target,if mode&kModeWrite!=0{c"r+b".as_ptr()}else{c"rb".as_ptr()});if file.is_null(){return kNotFound;}let handle=s.next_handle;s.next_handle=s.next_handle.wrapping_add(1);s.files.insert(handle,OpenFile{file:file as usize,path:target,written:false});handle}
// port of nand.cpp read / write / seek. CRT seek uses the source's long and SEEK_* values.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_read(handle:i32,destination:u32,length:u32)->i32{let s=state().lock().unwrap();let Some(f)=s.files.get(&handle)else{return kInvalid;};libc::fread(host(destination).cast(),1,length as usize,f.file as *mut libc::FILE)as i32}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_write(handle:i32,source:u32,length:u32)->i32{let mut s=state().lock().unwrap();let Some(f)=s.files.get_mut(&handle)else{return kInvalid;};let written=libc::fwrite(host(source).cast(),1,length as usize,f.file as *mut libc::FILE)as i32;libc::fflush(f.file as *mut libc::FILE);f.written=true;written}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_seek(handle:i32,offset:i32,whence:i32)->i32{let s=state().lock().unwrap();let Some(f)=s.files.get(&handle)else{return kInvalid;};let origins=[libc::SEEK_SET,libc::SEEK_CUR,libc::SEEK_END];if whence<0||whence>2||libc::fseek(f.file as *mut libc::FILE,offset as libc::c_long,origins[whence as usize])!=0{return kInvalid;}libc::ftell(f.file as *mut libc::FILE)as i32}
// port of nand.cpp close; remove current file before checking other open writers and pending backups.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_close(handle:i32){let mut s=state().lock().unwrap();let Some(f)=s.files.get(&handle)else{return;};libc::fclose(f.file as *mut libc::FILE);let saved=f.written&&inside_save(&s,&f.path);s.files.remove(&handle);if saved||s.backup_pending{save_changed(&mut s);}}
// port of nand.cpp create_file / create_directory / remove / rename.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_create_file(p:*const u8,n:usize)->i32{let s=state().lock().unwrap();let target=host_path(&s,std::slice::from_raw_parts(p,n));if wphw_nand_fs_query(0,target.as_ptr(),target.len()){return kExists;}let folder=parent(&target);wphw_nand_fs_create(folder.as_ptr(),folder.len());let file=fopen(&target,c"wb".as_ptr());if file.is_null(){return kInvalid;}libc::fclose(file);0}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_create_directory(p:*const u8,n:usize)->i32{let s=state().lock().unwrap();let target=host_path(&s,std::slice::from_raw_parts(p,n));if wphw_nand_fs_query(0,target.as_ptr(),target.len()){return kExists;}if wphw_nand_fs_create(target.as_ptr(),target.len()){0}else{kInvalid}}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_remove(p:*const u8,n:usize)->i32{let s=state().lock().unwrap();let path=host_path(&s,std::slice::from_raw_parts(p,n));if wphw_nand_fs_remove(path.as_ptr(),path.len())>0{0}else{kNotFound}}
#[no_mangle]pub unsafe extern "C" fn wprt_nand_rename(p:*const u8,n:usize,q:*const u8,m:usize)->i32{let mut s=state().lock().unwrap();let target=host_path(&s,std::slice::from_raw_parts(q,m));let from=host_path(&s,std::slice::from_raw_parts(p,n));if !wphw_nand_fs_rename(from.as_ptr(),from.len(),target.as_ptr(),target.len()){return kNotFound;}if inside_save(&s,&target){save_changed(&mut s);}0}
// port of nand.cpp console_is_spanish.
#[no_mangle]pub unsafe extern "C" fn wprt_nand_console_is_spanish()->bool{console_language()==kLanguageSpanish}
