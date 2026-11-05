// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted host adaptation of accepted backend.rs present_pipeline/present_frame.
// The source WGSL, Unorm filtering, sampler and f64 viewport order are retained.
// Bevy displays the physical-size result 1:1 and retains its normal devshot path.
use bevy::{prelude::*,asset::RenderAssetUsages,render::{Extract,ExtractSchedule,RenderApp,RenderStartup,render_asset::RenderAssets,texture::GpuImage,renderer::{RenderDevice,RenderQueue,RenderGraph,RenderGraphSystems}}};

pub fn enabled()->bool{std::env::var_os("WPGX_HOST_SOURCE_PRESENT").is_some()}
pub fn input_format()->wgpu::TextureFormat{if enabled(){wgpu::TextureFormat::Rgba8Unorm}else{wgpu::TextureFormat::Rgba8UnormSrgb}}
#[derive(Clone)]struct Source{image:Handle<Image>,gpu:Option<wgpu::Texture>,aspect:Option<f64>,overlay:Option<crate::host_options::Overlay>}
#[derive(Resource,Clone)]struct Canvas{image:Handle<Image>,width:u32,height:u32,sources:Vec<Source>}

fn canvas_image(width:u32,height:u32)->Image{
 let mut image=Image::new_fill(wgpu::Extent3d{width,height,depth_or_array_layers:1},wgpu::TextureDimension::D2,&[0,0,0,255],wgpu::TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());
 image.texture_descriptor.usage|=wgpu::TextureUsages::RENDER_ATTACHMENT;
 image.texture_descriptor.view_formats=&[wgpu::TextureFormat::Rgba8Unorm];
 image
}
pub fn spawn(commands:&mut Commands,images:&mut Assets<Image>){
 if !enabled(){return;}
 let image=images.add(canvas_image(1,1));
 commands.spawn((ImageNode::new(image.clone()),Node{position_type:PositionType::Absolute,left:Val::Px(0.0),top:Val::Px(0.0),width:Val::Percent(100.0),height:Val::Percent(100.0),..default()}));
 commands.insert_resource(Canvas{image,width:1,height:1,sources:Vec::new()});
}
fn sync_canvas(mut canvas:ResMut<Canvas>,guests:Res<crate::Guests>,windows:Query<&Window,With<bevy::window::PrimaryWindow>>,mut images:ResMut<Assets<Image>>){
 let Ok(window)=windows.single()else{return;};let(width,height)=(window.physical_width(),window.physical_height());
 if width==0||height==0{return;}
 if canvas.width!=width||canvas.height!=height{if let Some(mut image)=images.get_mut(&canvas.image){*image=canvas_image(width,height);}canvas.width=width;canvas.height=height;}
 canvas.sources=guests.items.iter().enumerate().map(|(i,g)|{
  let(_,width,height)=crate::physical_viewport(i,guests.items.len(),window);
  Source{image:g.image.clone(),gpu:g.gpu_image.clone(),aspect:g.aspect,overlay:g.options.as_ref().and_then(|v|v.overlay(width,height))}
 }).collect();
}
fn extract_canvas(mut commands:Commands,canvas:Extract<Option<Res<Canvas>>>){if let Some(canvas)=canvas.as_ref(){commands.insert_resource((**canvas).clone());}}

pub struct SourcePresentPlugin;
impl Plugin for SourcePresentPlugin{
 fn build(&self,app:&mut App){
  if !enabled(){return;}
  app.add_systems(Update,sync_canvas.after(crate::layout));
  app.sub_app_mut(RenderApp).add_systems(ExtractSchedule,extract_canvas).add_systems(RenderStartup,setup).add_systems(RenderGraph,draw.in_set(RenderGraphSystems::Begin));
 }
}
#[derive(Resource)]struct Pipeline{layout:wgpu::BindGroupLayout,pipeline:wgpu::RenderPipeline,overlay:wgpu::RenderPipeline,sampler:wgpu::Sampler}
fn setup(mut commands:Commands,device:Res<RenderDevice>){
 let device=device.wgpu_device();
 let shader=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("source present"),source:wgpu::ShaderSource::Wgsl(include_str!("host_source_present.wgsl").into())});
 let layout=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:Some("source present"),entries:&[
  wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Texture{sample_type:wgpu::TextureSampleType::Float{filterable:true},view_dimension:wgpu::TextureViewDimension::D2,multisampled:false},count:None},
  wgpu::BindGroupLayoutEntry{binding:1,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),count:None}]});
 let pipeline_layout=device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:Some("source present"),bind_group_layouts:&[Some(&layout)],immediate_size:0});
 // port of backend.rs present_pipeline, fixed to the source Unorm canvas target.
 let pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:Some("source present"),layout:Some(&pipeline_layout),
  vertex:wgpu::VertexState{module:&shader,entry_point:Some("vertex_main"),compilation_options:Default::default(),buffers:&[]},
  primitive:wgpu::PrimitiveState{topology:wgpu::PrimitiveTopology::TriangleList,cull_mode:None,unclipped_depth:true,..Default::default()},depth_stencil:None,multisample:Default::default(),
  fragment:Some(wgpu::FragmentState{module:&shader,entry_point:Some("present_main"),compilation_options:Default::default(),targets:&[Some(wgpu::ColorTargetState{format:wgpu::TextureFormat::Rgba8Unorm,blend:None,write_mask:wgpu::ColorWrites::ALL})]}),multiview_mask:None,cache:None});
 // port of backend.rs create_overlay_pipeline: SRC_ALPHA/INV_SRC_ALPHA; destination alpha retained.
 let overlay=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:Some("source options overlay"),layout:Some(&pipeline_layout),
  vertex:wgpu::VertexState{module:&shader,entry_point:Some("vertex_main"),compilation_options:Default::default(),buffers:&[]},
  primitive:wgpu::PrimitiveState{topology:wgpu::PrimitiveTopology::TriangleList,cull_mode:None,unclipped_depth:true,..Default::default()},depth_stencil:None,multisample:Default::default(),
  fragment:Some(wgpu::FragmentState{module:&shader,entry_point:Some("overlay_main"),compilation_options:Default::default(),targets:&[Some(wgpu::ColorTargetState{format:wgpu::TextureFormat::Rgba8Unorm,
   blend:Some(wgpu::BlendState{color:wgpu::BlendComponent{src_factor:wgpu::BlendFactor::SrcAlpha,dst_factor:wgpu::BlendFactor::OneMinusSrcAlpha,operation:wgpu::BlendOperation::Add},
    alpha:wgpu::BlendComponent{src_factor:wgpu::BlendFactor::Zero,dst_factor:wgpu::BlendFactor::One,operation:wgpu::BlendOperation::Add}}),write_mask:wgpu::ColorWrites::ALL})]}),multiview_mask:None,cache:None});
 // port of backend.rs linear_clamp, including the source LOD range.
 let sampler=device.create_sampler(&wgpu::SamplerDescriptor{label:Some("source linear clamp"),address_mode_u:wgpu::AddressMode::ClampToEdge,address_mode_v:wgpu::AddressMode::ClampToEdge,address_mode_w:wgpu::AddressMode::ClampToEdge,mag_filter:wgpu::FilterMode::Linear,min_filter:wgpu::FilterMode::Linear,mipmap_filter:wgpu::MipmapFilterMode::Linear,lod_min_clamp:0.0,lod_max_clamp:32.0,compare:None,anisotropy_clamp:1,border_color:None});
 commands.insert_resource(Pipeline{layout,pipeline,overlay,sampler});
}
fn draw(canvas:Option<Res<Canvas>>,pipeline:Res<Pipeline>,images:Res<RenderAssets<GpuImage>>,device:Res<RenderDevice>,queue:Res<RenderQueue>){
 let Some(canvas)=canvas else{return;};let Some(target)=images.get(&canvas.image)else{return;};
 // The underlying texture permits Unorm writes and Bevy's normal Srgb UI read view.
 // Scaling has already happened numerically in source Unorm space; UI is physical 1:1.
 let view=target.texture.create_view(&wgpu::TextureViewDescriptor{format:Some(wgpu::TextureFormat::Rgba8Unorm),..Default::default()});
 let device=device.wgpu_device();let mut encoder=device.create_command_encoder(&Default::default());
 {
  let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:Some("source present canvas"),color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:&view,depth_slice:None,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color{r:0.0,g:0.0,b:0.0,a:1.0}),store:wgpu::StoreOp::Store}})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None,multiview_mask:None});
  for(index,source)in canvas.sources.iter().enumerate(){
   let Some(aspect)=source.aspect else{continue;};
   let source_view=if let Some(texture)=&source.gpu{texture.create_view(&Default::default())}else{
    let Some(image)=images.get(&source.image)else{continue;};(*image.texture_view).clone()
   };
   let count=canvas.sources.len()as u64;let left=canvas.width as u64*index as u64/count;let right=canvas.width as u64*(index+1)as u64/count;let client_width=(right-left)as f64;let client_height=canvas.height as f64;
   if client_width<=0.0||client_height<=0.0{continue;}
   // port of present_frame fitting/order. Tile offset is the existing host mashup adapter.
   let mut width=client_width;let mut height=width/aspect;if height>client_height{height=client_height;width=height*aspect;}
   let bind=device.create_bind_group(&wgpu::BindGroupDescriptor{label:Some("source frame"),layout:&pipeline.layout,entries:&[wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(&source_view)},wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::Sampler(&pipeline.sampler)}]});
   pass.set_viewport((left as f64+(client_width-width)/2.0)as f32,((client_height-height)/2.0)as f32,width as f32,height as f32,0.0,1.0);
   pass.set_pipeline(&pipeline.pipeline);pass.set_bind_group(0,&bind,&[]);pass.draw(0..3,0..1);
   // port of backend.rs present_frame/draw_overlay: after game image, using exact options::place.
   if let Some(overlay)=&source.overlay{let rect=overlay.rect;if rect.width>0&&rect.height>0{
    let view=overlay.texture.create_view(&Default::default());
    let bind=device.create_bind_group(&wgpu::BindGroupDescriptor{label:Some("source options overlay"),layout:&pipeline.layout,entries:&[
     wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(&view)},wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::Sampler(&pipeline.sampler)}]});
    pass.set_viewport((left as i32+rect.x)as f32,rect.y as f32,rect.width as f32,rect.height as f32,0.0,1.0);
    pass.set_pipeline(&pipeline.overlay);pass.set_bind_group(0,&bind,&[]);pass.draw(0..3,0..1);
   }}
  }
 }
 queue.submit([encoder.finish()]);
}
