//! Independent blue offering vases below Soul House memorials.
use super::*;
use crate::interior_models::{ModelKind,selected_model};
fn drawing(block:u16)->Option<[[u16;4];4]> {Some(match block {
 0x28=>[[55,40,55,40],[56,63,56,78],[1,79,1,1],[1;4]],
 0x2a=>[[55,40,55,40],[56,78,56,63],[1,1,1,79],[1;4]],
 _=>return None,
})}
pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
 g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
 if frame.map_id.as_ref()!="SoulHouse"||cells.len()!=g.width*g.height {return;}
 let Some(model)=selected_model(ModelKind::FlowerStand).filter(|m|m.has_native_display_flower_materials()) else {return;};
 let mut custom=vec![false;cells.len()];
 static CANONICAL:std::sync::OnceLock<crate::live_profiles::Document>=std::sync::OnceLock::new();
 let canonical=CANONICAL.get_or_init(||serde_json::from_str(include_str!("../../../../modpacks/voxel-view/profiles.json")).expect("bundled profiles"));
 let customized=profiles.map(|active|crate::live_profiles::Document {
  objects:active.objects.iter().filter(|o|!canonical.objects.contains(o)).cloned().collect(),atmosphere:None,
 });
 for p in live::resolve(cells,g.width,g.height,"SoulHouse",customized.as_ref()) {for i in p.indices(g.width) {custom[i]=true;}}
 let mut offerings=Vec::new();
 for (i,tile) in cells.iter().enumerate() {
  let s=&tile.source;let Some(art)=drawing(s.metatile_id) else {continue;};
  let column=if s.metatile_id==0x28 {1}else{3};
  if s.tileset_id.as_ref()!="house"||s.subtile_row!=2||s.subtile_column!=column||s.tile_index!=79 {continue;}
  let x=i%g.width;let y=i/g.width;
  if x<column as usize||y<2 {continue;}
  let bx=x-column as usize;let by=y-2;
  if bx+4>g.width||by+4>g.height {continue;}
  if !(0..4).all(|dy|(0..4).all(|dx| {
   let j=(by+dy)*g.width+bx+dx;let t=&cells[j].source;
   !custom[j]&&t.tileset_id.as_ref()=="house"&&t.metatile_id==s.metatile_id
    &&t.subtile_column==dx as u8&&t.subtile_row==dy as u8&&t.tile_index==art[dy][dx]
  })) {continue;}
  offerings.push((i,(by+3)*g.width+bx+column as usize));
 }
 if offerings.is_empty() {return;}
 let owned=offerings.iter().map(|&(i,_)|i).collect::<std::collections::HashSet<_>>();
 for surface in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
  surface.indices=surface.indices.chunks_exact(3).filter(|tri| {
   // Remove only the original floor-facing offering drawing. The adjacent
   // memorial's upright south face can lie on this cell boundary and must stay.
   if !tri.iter().all(|&i|surface.positions[i as usize][1].abs()<0.001
    &&surface.normals[i as usize][1]>0.99) {return true;}
   let p=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize])).sum::<Vec3>()/3.;
   let x=((p.x-g.origin_x)/g.tile_width).floor()as i32;let y=((p.z-g.origin_z)/g.tile_height).floor()as i32;
   x<0||y<0||x as usize>=g.width||y as usize>=g.height||!owned.contains(&(y as usize*g.width+x as usize))
  }).flatten().copied().collect();
 }
 for (i,ground) in offerings {
  let x=i%g.width;let y=i/g.width;let (w,e,n,s)=g.bounds(x,y);
  modeled_interiors::native_house_material_floor_cell(&mut mesh.textured,[w,e,n,s],0.,
   [frame.grid_origin.x+x as i32,frame.grid_origin.y+y as i32],g.uv(ground%g.width,ground/g.width),false);
  let uv=g.uv(x,y);let sample=|px:f32,py:f32|
   [uv.0+(uv.1-uv.0)*(px+0.5)/8.,uv.2+(uv.3-uv.2)*(py+0.5)/8.];
  let blue=sample(3.,5.);let pale=sample(1.,0.);let dark=sample(3.,6.);let ink=sample(0.,0.);
  let start=mesh.textured.positions.len();
  assert!(model.append_native_display_flower(&mut mesh.textured,
   [w+(e-w)/8.,e-(e-w)/8.,n+(s-n)/8.,s-(s-n)/8.],0.,8.*g.tile_height/8.,
   [pale,blue,ink,blue,pale,pale,blue,blue,ink,ink,dark,pale]));
  mesh.textured.cutaway_ranges.push(start..mesh.textured.positions.len());
  if mesh.authored_cells.len()==cells.len() {mesh.authored_cells[i]=Some("soul:offering-vase");}
  // Collision and cached native footing stay authoritative; this is scenery.
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 use bevy::prelude::{IVec2,UVec2,Vec2};
 fn fixture(block:u16)->(VisualWorldFrame,GridGeometry,TerrainMeshData) {
  let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
  let art=drawing(block).unwrap();let mut mesh=TerrainMeshData::default();let mut tiles=Vec::new();
  for y in 0..4 {for x in 0..4 {
   tiles.push(VisualTile {column:x as u32,row:y as u32,texture:Default::default(),priority:false,
    animation_frames:None,source:VisualTileSource {tileset_id:"house".into(),metatile_id:block,
     subtile_column:x as u8,subtile_row:y as u8,tile_index:art[y][x]}});
   append_top(&mut mesh.textured,g.bounds(x,y).into(),0.,g.uv(x,y));
  }}
  // A memorial's real upright face lies on the offering's north boundary.
  append_quad(&mut mesh.textured,[[8.,0.,16.],[16.,0.,16.],[16.,12.,16.],[8.,12.,16.]],
   [0.,0.,1.],[[0.,0.],[1.,0.],[1.,1.],[0.,1.]],TEXTURED_SHADE);
  mesh.footing_heights=vec![1.25;16];mesh.authored_cells=vec![None;16];
  let frame=VisualWorldFrame {active:true,map_id:"SoulHouse".into(),source_map_size_core_tiles:Some(UVec2::new(10,8)),
   boundary_warps:Default::default(),jump_ledges:Default::default(),terrain_revision:0,grid_origin:IVec2::ZERO,map_texture:Default::default(),
   center:Vec2::ZERO,viewport_size:Vec2::splat(32.),tile_size:Vec2::splat(8.),grid_size:UVec2::splat(4),tiles,actors:Vec::new()};
  (frame,g,mesh)
 }
 #[test]
 fn kanto_soul_offerings_keep_memorial_faces_footing_and_own_blue_palette() {
  for block in [0x28,0x2a] {
   let (frame,g,mut mesh)=fixture(block);let baseline=mesh.clone();
   let profiles:crate::live_profiles::Document=serde_json::from_str(include_str!("../../../../modpacks/voxel-view/profiles.json")).unwrap();
   finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,Some(&profiles));
   assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("soul:offering-vase")).count(),1);
   assert!(mesh.footing_heights.iter().all(|&h|h==1.25));
   assert!(mesh.textured.indices.chunks_exact(3).any(|tri|tri==&baseline.textured.indices[96..99]),"memorial face retained");
   let (opaque,groups)=crate::occluder_fade::prepare(mesh.textured.clone());
   assert_eq!(groups.len(),1,"one complete vase fade object");
   assert!(!opaque.indices.is_empty(),"floor backing remains opaque");
   let range=&mesh.textured.cutaway_ranges[0];
   let floor_area=mesh.textured.indices.chunks_exact(3).filter(|tri|tri.iter().all(|i|!range.contains(&(*i as usize))))
    .map(|tri| {
     let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
     (b-a).cross(c-a).y*0.5
    }).sum::<f32>();
   assert!((floor_area-16.*64.).abs()<0.001,"opaque backing retains the complete floor footprint");
   assert!(mesh.textured.positions[range.clone()].iter().all(|p|p[1]>=0.&&p[1]<=8.001));
   let uv=g.uv(if block==0x28 {1}else{3},2);
   for pixel in [[3.,5.],[1.,0.],[3.,6.],[0.,0.]] {
    assert!(mesh.textured.uvs[range.clone()].contains(&[uv.0+(uv.1-uv.0)*(pixel[0]+0.5)/8.,uv.2+(uv.3-uv.2)*(pixel[1]+0.5)/8.]));
   }
  }
 }
 #[test]
 fn kanto_soul_offerings_reject_all_guard_mutations_and_matching_custom_profiles() {
  let (mut frame,g,baseline)=fixture(0x28);
  for i in 0..16 {
   frame.tiles[i].source.tile_index^=1;let mut mesh=baseline.clone();
   finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(mesh,baseline);
   frame.tiles[i].source.tile_index^=1;
  }
  let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom offering","map":"SoulHouse","tileset":"house","metatile":40,"origin":[1,2],"tiles":[[79]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
  let refs=frame.tiles.iter().collect::<Vec<_>>();assert!(!live::resolve(&refs,4,4,"SoulHouse",Some(&custom)).is_empty());
  let mut mesh=baseline.clone();finish(&mut mesh,&frame,&refs,&g,Some(&custom));assert_eq!(mesh,baseline);
  frame.map_id="OtherHouse".into();let mut mesh=baseline.clone();
  finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(mesh,baseline);
 }
}
