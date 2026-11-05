// SPDX-License-Identifier: GPL-3.0-or-later
// H8 Bevy playback transport. Samples are pulled from the guest's existing Rust Mixer.
use bevy::{prelude::*,audio::{Decodable,Source,ChannelCount,SampleRate}};
use std::{sync::Arc,time::Duration};
use cpal::traits::{DeviceTrait,HostTrait};
type Bind = unsafe extern "C" fn(u32)->bool;
type Read = unsafe extern "C" fn(*mut f32,u32)->bool;
// CPAL/WASAPI default output config reads GetMixFormat.nSamplesPerSec,as the old backend.
// Transport buffer size only, not game timing or a replacement sound asset.
const FRAMES:usize=256;
#[derive(Asset,TypePath,Clone)]
pub struct GuestAudio { library:Arc<libloading::Library>, read:Read, rate:u32 }
pub struct GuestDecoder { _library:Arc<libloading::Library>,read:Read, samples:[f32;FRAMES*2], cursor:usize, rate:u32 }
impl Decodable for GuestAudio {
    type Decoder=GuestDecoder;
    fn decoder(&self)->Self::Decoder { GuestDecoder{_library:self.library.clone(),read:self.read,samples:[0.0;FRAMES*2],cursor:FRAMES*2,rate:self.rate} }
}
impl Iterator for GuestDecoder {
    type Item=f32;
    fn next(&mut self)->Option<f32>{
        if self.cursor==self.samples.len(){unsafe{(self.read)(self.samples.as_mut_ptr(),FRAMES as u32);};self.cursor=0;}
        let sample=self.samples[self.cursor];self.cursor+=1;Some(sample)
    }
}
impl Source for GuestDecoder {
    fn current_span_len(&self)->Option<usize>{None}
    fn channels(&self)->ChannelCount{ChannelCount::new(2).unwrap()}
    fn sample_rate(&self)->SampleRate{SampleRate::new(self.rate).unwrap()}
    fn total_duration(&self)->Option<Duration>{None}
}
// Absent symbols mean an accepted legacy DLL: keep its native audio backend intact.
// Partial bindings are errors, not a guessed fallback or a second simultaneous output.
pub unsafe fn bind(library:Arc<libloading::Library>,assets:&mut Assets<GuestAudio>)->Result<Option<Handle<GuestAudio>>,String>{
    let bind=library.get::<Bind>(b"wpg_audio_bind\0");
    let read=library.get::<Read>(b"wpg_audio_read\0");
    let (bind,read)=match(bind,read){(Err(_),Err(_))=>return Ok(None),(Ok(b),Ok(r))=>(*b,*r),_=>return Err("Incomplete guest audio interface".into())};
    let device=cpal::default_host().default_output_device().ok_or("No default output device for guest mixer")?;
    let rate=device.default_output_config().map_err(|e|e.to_string())?.sample_rate();
    if !bind(rate){return Err("Guest audio must be bound once before wpg_run".into());}
    Ok(Some(assets.add(GuestAudio{library,read,rate})))
}
