// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine engine/src/audio/audio.cpp remaining Windows device and CRT file leaves.
// PRIVATE UNCOMPILED/UNINTEGRATED. Never records system/loopback audio;only original game mixer output.
use std::{ffi::{c_void,c_char},ptr::{null,null_mut}};
use windows::{core::{GUID,HRESULT,Interface,IUnknown_Vtbl},Win32::{Foundation::HANDLE,Media::{Audio::*,KernelStreaming::WAVE_FORMAT_EXTENSIBLE,Multimedia::WAVE_FORMAT_IEEE_FLOAT},System::{Com::*,Threading::*}}};
#[repr(C)]#[derive(Clone,Copy,Default)]pub struct Frame{pub left:f32,pub right:f32}
extern "C" {fn wprt_audio_mixer_new(rate:f64)->*mut c_void;fn wprt_audio_mixer_fill(mixer:*mut c_void,data:*mut Frame,n:u32);fn wphw_awd_start_error();fn lroundf(v:f32)->i32;}
#[link(name="kernel32")]extern "system"{#[link_name="CreateEventA"]fn create_event(attributes:*const windows::Win32::Security::SECURITY_ATTRIBUTES,manual:windows::core::BOOL,initial:windows::core::BOOL,name:windows::core::PCSTR)->HANDLE;}
#[link(name="ole32")]extern "system"{#[link_name="CoCreateInstance"]fn co_create(clsid:*const GUID,outer:*mut c_void,context:u32,iid:*const GUID,out:*mut *mut c_void)->HRESULT;}
// Official COM Interface/vtable bindings preserve original HRESULT short-circuit order and explicit Release order.
unsafe fn table<'a,T:Interface+'a>(p:&'a *mut c_void)->&'a T::Vtable{T::from_raw_borrowed(p).unwrap().vtable()}
unsafe fn release(p:&mut *mut c_void){if !p.is_null(){(table::<windows::core::IUnknown>(p).Release)(*p);*p=null_mut();}}
// port of audio.cpp is_float;the source reads SubFormat only for WAVE_FORMAT_EXTENSIBLE.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_is_float(format:*const WAVEFORMATEX)->bool{
 if (*format).wFormatTag as u32==WAVE_FORMAT_IEEE_FLOAT{return true}
 if (*format).wFormatTag as u32==WAVE_FORMAT_EXTENSIBLE{return std::ptr::addr_of!((*(format.cast::<WAVEFORMATEXTENSIBLE>())).SubFormat).read_unaligned()==GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71)}false
}
// port of audio.cpp output_thread. Native std::thread launch remains the source SDK owner.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_output_thread(){
 let _=CoInitializeEx(None,COINIT_MULTITHREADED);
 let(mut enumerator,mut device,mut client,mut render)= (null_mut(),null_mut(),null_mut(),null_mut());let mut format:*mut WAVEFORMATEX=null_mut();
 let event=create_event(null(),false.into(),false.into(),windows::core::PCSTR::null());
 let ok=co_create(&MMDeviceEnumerator,null_mut(),CLSCTX_ALL.0,&IMMDeviceEnumerator::IID,&mut enumerator).is_ok()
 &&(table::<IMMDeviceEnumerator>(&enumerator).GetDefaultAudioEndpoint)(enumerator,eRender,eConsole,&mut device).is_ok()
 &&(table::<IMMDevice>(&device).Activate)(device,&IAudioClient::IID,CLSCTX_ALL,null(),&mut client).is_ok()
 &&(table::<IAudioClient>(&client).GetMixFormat)(client,&mut format).is_ok()
 &&(table::<IAudioClient>(&client).Initialize)(client,AUDCLNT_SHAREMODE_SHARED,AUDCLNT_STREAMFLAGS_EVENTCALLBACK,400000,0,format,null()).is_ok()
 &&(table::<IAudioClient>(&client).SetEventHandle)(client,event).is_ok()
 &&(table::<IAudioClient>(&client).GetService)(client,&IAudioRenderClient::IID,&mut render).is_ok();
 let mut buffer_frames=0u32;let ok=ok&&(table::<IAudioClient>(&client).GetBufferSize)(client,&mut buffer_frames).is_ok()&&(table::<IAudioClient>(&client).Start)(client).is_ok();
 if !ok{wphw_awd_start_error();release(&mut render);release(&mut client);release(&mut device);release(&mut enumerator);return}
 // Source intentionally does not close event/free mix-format/uninitialize COM on its failure path;no new cleanup.
 let floats=wprt_awd_is_float(format);let channels=(*format).nChannels as u32;let mixer=wprt_audio_mixer_new((*format).nSamplesPerSec as f64);
 loop{
  WaitForSingleObject(event,100);let mut padding=0u32;if (table::<IAudioClient>(&client).GetCurrentPadding)(client,&mut padding).is_err(){continue}
  let available=buffer_frames.wrapping_sub(padding);if available==0{continue}
  let mut data=null_mut();if (table::<IAudioRenderClient>(&render).GetBuffer)(render,available,&mut data).is_err(){continue}
  let mut mixed=vec![Frame::default();available as usize];wprt_audio_mixer_fill(mixer,mixed.as_mut_ptr(),available);
  wprt_awd_interleave(data,mixed.as_ptr(),available,channels,floats);
  let _=(table::<IAudioRenderClient>(&render).ReleaseBuffer)(render,available,0);
 }
}
// port of audio.cpp output_thread interleave loop;no extra-channel duplication or source-absent clamping.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_interleave(data:*mut u8,mixed:*const Frame,n:u32,channels:u32,floats:bool){
 for i in 0..n{let frame=*mixed.add(i as usize);for channel in 0..channels{let value=if channel==0{frame.left}else if channel==1{frame.right}else{0.0};let at=i.wrapping_mul(channels).wrapping_add(channel) as usize;if floats{data.cast::<f32>().add(at).write(value)}else{data.cast::<i16>().add(at).write(lroundf(value*32767.0) as i16)}}}
}

// port of audio.cpp write_wav_header. Source native-endian u32 words and seek/write/seek/flush order.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_wav_header(file:*mut libc::FILE,frames:u32,rate:u32){
 let bytes=frames.wrapping_mul(4);let values=[0x46464952,36u32.wrapping_add(bytes),0x45564157,0x20746D66,16,0x00020001,rate,rate.wrapping_mul(4),0x00100004,0x61746164,bytes];
 libc::fseek(file,0,libc::SEEK_SET);libc::fwrite(values.as_ptr().cast(),4,11,file);libc::fseek(file,0,libc::SEEK_END);libc::fflush(file);
}
// port of original Mixer constructor dump-open leaf;returns original FILE*,does not add a close/destructor.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_dump_open(rate:u32)->usize{
 let path=libc::getenv(b"WP_DUMP_OUTPUT\0".as_ptr().cast());if path.is_null(){return 0}let file=libc::fopen(path,b"wb\0".as_ptr().cast());if !file.is_null(){wprt_awd_wav_header(file,0,rate)}file as usize
}
// port of original Mixer::dump clamp/lround conversion. Original std::clamp preserves NaN and signed zero.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_dump_pcm(mixed:*const Frame,n:u32,out:*mut i16){
 for i in 0..n{let f=*mixed.add(i as usize);for (c,v) in [f.left,f.right].into_iter().enumerate(){let v=if v < -1.0{-1.0}else if 1.0 < v{1.0}else{v};out.add(i as usize*2+c).write(lroundf(v*32767.0) as i16)}}
}
// port of original Mixer dump leaf;source count/wrapping/header order and ignored fwrite result.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_mixer_dump(file:usize,mixed:*const Frame,n:u32,frames:*mut u32,rate:u32){
 let mut samples=vec![0i16;n.wrapping_mul(2) as usize];wprt_awd_dump_pcm(mixed,n,samples.as_mut_ptr());libc::fwrite(samples.as_ptr().cast(),4,n as usize,file as *mut libc::FILE);*frames=(*frames).wrapping_add(n);wprt_awd_wav_header(file as *mut libc::FILE,*frames,rate);
}
static SOURCE_PATH:std::sync::OnceLock<usize>=std::sync::OnceLock::new();
static mut SOURCE_DUMP:*mut libc::FILE=null_mut();static mut SOURCE_FRAMES:u32=0;
// port of audio.cpp push optional source dump;called under the accepted original queue mutex.
#[no_mangle]pub unsafe extern "C" fn wprt_awd_push_dump(frames:*const i16,count:usize,rate:u32){
 let path=*SOURCE_PATH.get_or_init(||libc::getenv(b"WP_DUMP_AUDIO\0".as_ptr().cast()) as usize) as *const c_char;
 if !path.is_null()&&SOURCE_DUMP.is_null(){SOURCE_DUMP=libc::fopen(path,b"wb\0".as_ptr().cast());if !SOURCE_DUMP.is_null(){wprt_awd_wav_header(SOURCE_DUMP,0,rate)}}
 if !SOURCE_DUMP.is_null(){libc::fwrite(frames.cast(),4,count,SOURCE_DUMP);SOURCE_FRAMES=SOURCE_FRAMES.wrapping_add(count as u32);if ((SOURCE_FRAMES%16000) as usize)<count{wprt_awd_wav_header(SOURCE_DUMP,SOURCE_FRAMES,rate)}}
}
