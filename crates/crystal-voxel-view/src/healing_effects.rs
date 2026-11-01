//! Pack-backed healing art placed on the native clinic enclosure's front.
use bevy::prelude::*;
use crystal_render_api::{VisualActor,VisualActorId,VisualEffectId,VisualWorldFrame};
pub fn native_clinic_healing_map(map:&str)->bool {
 crate::mesh::native_clinic_map(map)
}
fn fixture(frame:&VisualWorldFrame)->Option<(usize,usize)> {
 if !native_clinic_healing_map(&frame.map_id)||frame.grid_size.x<4||frame.grid_size.y<4 {return None;}
 let width=frame.grid_size.x as usize;
 for tile in &frame.tiles {
  let source=&tile.source;
  if source.tileset_id.as_ref()!="pokecenter"||source.metatile_id!=1||source.subtile_column!=0||source.subtile_row!=0 {continue;}
  let col=tile.column as usize;let row=tile.row as usize;
  if col+4>width||row+4>frame.grid_size.y as usize {continue;}
  if (0..16).all(|j|frame.tiles.get((row+j/4)*width+col+j%4).is_some_and(|t| {
   let s=&t.source;
   s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==1
    &&s.subtile_column==(j%4)as u8&&s.subtile_row==(j/4)as u8
    &&s.tile_index==[28,29,30,31,44,45,46,47,60,61,61,63,76,77,78,79][j]
  })) {return Some((col,row));}
 }
 None
}
pub fn native_clinic_healing_supported(frame:&VisualWorldFrame)->bool {fixture(frame).is_some()}
pub(super) fn profile_allows(frame:&VisualWorldFrame,profiles:&crate::live_profiles::Document)->bool {
 if frame.map_id.as_ref()=="HallOfFame" {return crate::mesh::hall_healing_effects_supported(frame,Some(profiles));}
 fixture(frame).is_some_and(|(column,row)|crate::mesh::native_healing_owner_allows(frame,profiles,column,row))
}
pub(super) fn is_healing(actor:&VisualActor)->bool {
 matches!(actor.id,VisualActorId::Effect(VisualEffectId::HealMachineBall(_)|VisualEffectId::HealMachineLamp(_)))
}
pub(super) fn transform(frame:&VisualWorldFrame,actor:&VisualActor)->Option<Transform> {
 if frame.map_id.as_ref()=="HallOfFame" {return hall_transform(frame,actor);}
 let (col,row)=fixture(frame)?;
 if !frame.tile_size.is_finite()||frame.tile_size.min_element()<=0.||!actor.center.is_finite() {return None;}
 let scale=frame.tile_size/8.;
 let origin=frame.center+Vec2::new(-(frame.grid_size.x as f32)*frame.tile_size.x*0.5,
  frame.grid_size.y as f32*frame.tile_size.y*0.5)
  +Vec2::new(col as f32*frame.tile_size.x,-(row as f32)*frame.tile_size.y);
 let local=Vec2::new(actor.center.x-origin.x,origin.y-actor.center.y)/scale;
 if local.x<0.||local.x>32.||local.y<0.||local.y>32. {return None;}
 let ball=matches!(actor.id,VisualActorId::Effect(VisualEffectId::HealMachineBall(_)));
 let inset=frame.tile_size.x*0.045+0.5*scale.x;
 let width=4.*frame.tile_size.x-2.*inset;let height=30.*scale.y;
 let card_size=Vec2::new(width*actor.size.x/(32.*scale.x),height*actor.size.y/(32.*scale.y));
 let center_y=height*(1.-local.y/32.);
 let south=-origin.y+4.*frame.tile_size.y;
 let z=south-if ball {0.15*scale.y}else{0.}+0.025*scale.y;
 Some(Transform::from_xyz(origin.x+inset+width*local.x/32.,center_y-card_size.y*0.5,z)
  .with_scale(Vec3::new(if actor.flip_x {-card_size.x}else{card_size.x},card_size.y,1.)))
}

fn hall_transform(frame:&VisualWorldFrame,actor:&VisualActor)->Option<Transform> {
 if !crate::mesh::hall_healing_effects_supported(frame,None)
  ||!matches!(actor.id,VisualActorId::Effect(VisualEffectId::HealMachineBall(0..=5)))
  ||!actor.center.is_finite()||!actor.size.is_finite()||actor.size.min_element()<=0. {return None;}
 let scale=frame.tile_size/8.;
 let origin=frame.center+Vec2::new(
  -(frame.grid_size.x as f32)*frame.tile_size.x*0.5-frame.grid_origin.x as f32*frame.tile_size.x,
  (frame.grid_size.y as f32)*frame.tile_size.y*0.5+frame.grid_origin.y as f32*frame.tile_size.y);
 let local=Vec2::new(actor.center.x-origin.x,origin.y-actor.center.y)/scale;
 let VisualActorId::Effect(VisualEffectId::HealMachineBall(slot))=actor.id else{return None;};
 // Publication already applies the source camera offset: Hall's player row3
 // shifts OAM centers sixteen pixels north into these actual map positions.
 let &(x,y)=[(77.,32.),(82.,32.),(73.,31.),(86.,31.),(69.,29.),(89.,29.)].get(slot as usize)?;
 if !local.abs_diff_eq(Vec2::new(x,y),0.01){return None;}
 // Fold the three nested pairs four pixels south onto the physical tray.
 // Actor cards pivot at their bottom edge. After folding, that edge is
 // south of the sprite center by half its height.
 Some(Transform::from_xyz(origin.x+local.x*scale.x,10.15*scale.y,-origin.y+(local.y+4.)*scale.y+actor.size.y*0.5)
  .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
  .with_scale(Vec3::new(if actor.flip_x {-actor.size.x}else{actor.size.x},actor.size.y,1.)))
}

#[cfg(test)]
mod tests {
 use super::*;
 use crystal_render_api::{VisualTile,VisualTileSource};
 fn frame()->VisualWorldFrame {
  let art=[28,29,30,31,44,45,46,47,60,61,61,63,76,77,78,79];
  VisualWorldFrame{map_id:"PewterPokecenter1F".into(),grid_size:UVec2::splat(4),tile_size:Vec2::splat(8.),
   tiles:(0..16).map(|i|VisualTile{column:(i%4)as u32,row:(i/4)as u32,
    texture:default(),animation_frames:None,priority:false,source:VisualTileSource{
     tileset_id:"pokecenter".into(),metatile_id:1,subtile_column:(i%4)as u8,
     subtile_row:(i/4)as u8,tile_index:art[i]}}).collect(),..default()}
 }
 fn actor(id:VisualEffectId,p:Vec2)->VisualActor {
  VisualActor{id:VisualActorId::Effect(id),source_id:"effect_heal_ball".into(),model_palette:None,
   texture:default(),center:Vec2::new(-16.,16.)+Vec2::new(p.x,-p.y),size:Vec2::splat(8.),
   flip_x:false,above_priority:false,facing:None}
 }
 #[test]
 fn kanto_healing_effects_follow_native_enclosure_face_and_source_centers() {
  let mut f=frame();let points=[(12.,10.),(20.,10.),(12.,15.),(20.,15.),(12.,20.),(20.,20.)];
  for (slot,(x,y)) in points.into_iter().enumerate() {
   let a=actor(VisualEffectId::HealMachineBall(slot as u8),Vec2::new(x,y));
   let pose=transform(&f,&a).unwrap();
   assert!((pose.translation.z-15.875).abs()<0.001);
   assert!((pose.translation.y+pose.scale.y*0.5-30.*(1.-y/32.)).abs()<0.001);
   assert!((pose.translation.x-(-16.+0.86+30.28*x/32.)).abs()<0.001);
   assert_eq!(pose.rotation,Quat::IDENTITY);
   let mut moved=a.clone();f.center+=Vec2::new(17.,-9.);moved.center+=Vec2::new(17.,-9.);
   let next=transform(&f,&moved).unwrap();
   assert!((next.translation-pose.translation-Vec3::new(17.,0.,9.)).length()<0.001);
   f.center=Vec2::ZERO;
  }
  for (slot,x) in [14.,18.].into_iter().enumerate() {
   let a=actor(VisualEffectId::HealMachineLamp(slot as u8),Vec2::new(x,4.));
   let pose=transform(&f,&a).unwrap();assert!((pose.translation.z-16.025).abs()<0.001);
   assert!((pose.translation.y+pose.scale.y*0.5-26.25).abs()<0.001);
  }
 }
 #[test]
 fn kanto_healing_effects_yield_to_changed_and_overlapping_live_fixtures() {
  let mut f=frame();f.grid_size.y=5;
  for column in 0..4 {f.tiles.push(VisualTile{column,row:4,texture:default(),animation_frames:None,priority:false,
   source:VisualTileSource{tileset_id:"pokecenter".into(),metatile_id:4,subtile_column:column as u8,subtile_row:0,tile_index:17}});}
  let mut profiles:crate::live_profiles::Document=serde_json::from_str(include_str!("../../../modpacks/voxel-view/profiles.json")).unwrap();
  assert!(profile_allows(&f,&profiles));
  let index=profiles.objects.iter().position(|o|o.name=="Shared Pokecenter healing machine").unwrap();
  profiles.objects[index].depth_pixels+=1.;assert!(!profile_allows(&f,&profiles));
  profiles.objects[index].depth_pixels-=1.;
  let mut custom=profiles.objects[index].clone();custom.name="User healing cabinet".into();custom.depth_pixels+=1.;
  profiles.objects.push(custom);assert!(!profile_allows(&f,&profiles));
 }
 #[test]
 fn native_healing_effects_reject_partial_machine_and_unreviewed_clinics() {
  let mut f=frame();let a=actor(VisualEffectId::HealMachineBall(0),Vec2::new(12.,10.));
  for i in 0..16 {f.tiles[i].source.tile_index^=1;assert!(transform(&f,&a).is_none());f.tiles[i].source.tile_index^=1;}
  f.map_id="CherrygrovePokecenter1F".into();assert!(transform(&f,&a).is_some());
  f.map_id="UnreviewedPokecenter1F".into();assert!(transform(&f,&a).is_none());
 }
}
