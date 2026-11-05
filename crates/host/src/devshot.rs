// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted (Claude) dev capture of OUR OWN rendered frames (Bevy Screenshot of our window's swapchain image),
// never the screen: work/claude-ops/ownshot.txt = "name count every" -> work/private/observe/<name>/fNNNN.png.
// Works while the window is covered by other apps (it reads our GPU output, not desktop pixels).
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot,save_to_disk};
#[derive(Resource,Default)] struct Shots{name:String,left:u32,every:u32,n:u32,tick:u32}
pub fn install(app:&mut App){app.init_resource::<Shots>().add_systems(Update,shoot);}
fn shoot(mut commands:Commands,mut s:ResMut<Shots>){
    s.tick+=1;
    if s.left==0{if s.tick%20!=0{return;}
        let Some(root)=crate::scene_view::private_root()else{return;};let req=root.parent().map(|r|r.join("claude-ops/ownshot.txt"));let Some(req)=req else{return;};
        let Ok(t)=std::fs::read_to_string(&req)else{return;};let _=std::fs::remove_file(&req);let mut it=t.split_whitespace();
        let(Some(name),Some(count),Some(every))=(it.next(),it.next().and_then(|v|v.parse().ok()),it.next().and_then(|v|v.parse().ok()))else{return;};
        let _=std::fs::create_dir_all(root.join("observe").join(name));*s=Shots{name:name.to_owned(),left:count,every:std::cmp::max(1,every),n:0,tick:0};println!("[devshot] {name} x{count} every {every}");}
    if s.tick%s.every!=0{return;}
    let Some(root)=crate::scene_view::private_root()else{return;};let p=root.join("observe").join(&s.name).join(format!("f{:04}.png",s.n));
    commands.spawn(Screenshot::primary_window()).observe(save_to_disk(p));s.n+=1;s.left-=1;
}
