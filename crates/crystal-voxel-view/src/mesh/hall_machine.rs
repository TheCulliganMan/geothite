//! Complete native Hall-of-Fame recording machine and stepped platform.
use super::*;
use bevy::prelude::UVec2;
fn drawing(block:u16)->Option<[[u16;4];4]> {Some(match block {
 0x30=>[[82;4],[83,83,40,83],[12,13,62,56],[28,29,40,84]],
 0x31=>[[82;4],[44,45,45,46],[73,14,15,74],[72,30,31,75]],
 0x32=>[[82;4],[83,41,83,83],[57,63,12,13],[86,41,28,29]],
 0x34=>[[12,13,62,84],[28,29,76,77],[12,13,92,93],[28,29,28,29]],
 0x35=>[[88,89,90,91],[56,47,47,57],[87,71,71,71],[28,29,28,29]],
 0x36=>[[85,63,12,13],[78,79,28,29],[94,95,12,13],[28,29,28,29]],
 _=>return None,
})}
fn owned(x:i32,y:i32)->bool {
 if !(4..16).contains(&x)||!(0..8).contains(&y) {return false;}
 let block=[[0x30,0x31,0x32],[0x34,0x35,0x36]][y as usize/4][(x as usize-4)/4];
 !matches!(drawing(block).unwrap()[y as usize%4][x as usize%4],12|13|28|29|82|83)
}
fn complete(frame:&VisualWorldFrame,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if frame.map_id.as_ref()!="HallOfFame"||cells.len()!=g.width*g.height
   ||frame.source_map_size_core_tiles.is_some_and(|s|s!=UVec2::new(10,14)) {return false;}
 for y in 0..8 {for x in 4..16 {
  let lx=x-frame.grid_origin.x;let ly=y-frame.grid_origin.y;
  if lx<0||ly<0||lx as usize>=g.width||ly as usize>=g.height {return false;}
  let s=&cells[ly as usize*g.width+lx as usize].source;
  let b=[[0x30,0x31,0x32],[0x34,0x35,0x36]][y as usize/4][(x as usize-4)/4];
  if s.tileset_id.as_ref()!="ice_path"||s.metatile_id!=b||s.subtile_column!=x as u8%4
   ||s.subtile_row!=y as u8%4||s.tile_index!=drawing(b).unwrap()[y as usize%4][x as usize%4] {return false;}
 }}true
}
fn builtin_owns(frame:&VisualWorldFrame,cells:&[&VisualTile],g:&GridGeometry,
 profiles:Option<&crate::live_profiles::Document>)->bool {
 !live::resolve(cells,g.width,g.height,"HallOfFame",profiles).iter().any(|p|
  p.indices(g.width).any(|i|owned(frame.grid_origin.x+(i%g.width)as i32,frame.grid_origin.y+(i/g.width)as i32)))
}

pub(super) fn effects_supported(frame:&VisualWorldFrame,profiles:Option<&crate::live_profiles::Document>)->bool {
 if !frame.tile_size.is_finite()||frame.tile_size.min_element()<=0. {return false;}
 let g=GridGeometry {width:frame.grid_size.x as usize,height:frame.grid_size.y as usize,
  tile_width:frame.tile_size.x,tile_height:frame.tile_size.y,origin_x:0.,origin_z:0.};
 let Some(count)=g.width.checked_mul(g.height) else{return false;};
 if count==0||count!=frame.tiles.len(){return false;}
 let mut order=vec![None;count];
 for tile in &frame.tiles {
  let x=tile.column as usize;let y=tile.row as usize;
  if x>=g.width||y>=g.height||order[y*g.width+x].is_some(){return false;}
  order[y*g.width+x]=Some(tile);
 }
 let Some(cells)=order.into_iter().collect::<Option<Vec<_>>>() else{return false;};
 complete(frame,&cells,&g)&&builtin_owns(frame,&cells,&g,profiles)
}

pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
 g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
 if !complete(frame,cells,g)||!builtin_owns(frame,cells,g,profiles) {return;}
 let origin=frame.grid_origin;
 for surface in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
  surface.indices=surface.indices.chunks_exact(3).filter(|tri| {
   let c=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize])).sum::<Vec3>()/3.;
   !owned(((c.x-g.origin_x)/g.tile_width).floor()as i32+origin.x,
    ((c.z-g.origin_z)/g.tile_height).floor()as i32+origin.y)
  }).flatten().copied().collect();
 }
 for (i,h) in mesh.footing_heights.iter_mut().enumerate() {
  let x=origin.x+(i%g.width)as i32;let y=origin.y+(i/g.width)as i32;
  if owned(x,y) {
   let floor=g.uv((4+x.rem_euclid(2)-origin.x)as usize,(2+y.rem_euclid(2)-origin.y)as usize);
   append_top(&mut mesh.textured,g.bounds(i%g.width,i/g.width).into(),0.,floor);
   *h=if y==6 {2.}else{4.}*g.tile_height/8.;
   if mesh.authored_cells.len()==cells.len() {mesh.authored_cells[i]=Some("hall:recording-machine");}
  }
 }
 let sample=|x:i32,y:i32,px:f32,py:f32| {
  let uv=g.uv((x-origin.x)as usize,(y-origin.y)as usize);
  [uv.0+(uv.1-uv.0)*(px+0.5)/8.,uv.2+(uv.3-uv.2)*(py+0.5)/8.]
 };
 let body=sample(9,2,4.,4.);let dark=sample(9,2,6.,6.);
 let light=sample(9,1,4.,4.);
 let sx=g.tile_width/8.;let sy=g.tile_height/8.;
 let wx=g.origin_x-origin.x as f32*g.tile_width;let nz=g.origin_z-origin.y as f32*g.tile_height;
 let mut box_part=|b:[f32;4],bottom:f32,top:f32,paint:[f32;2]| {
  let [w,e,n,s]=[wx+b[0]*sx,wx+b[1]*sx,nz+b[2]*sy,nz+b[3]*sy];
  let lo=bottom*sy;let hi=top*sy;
  for (p,n) in [
   ([[w,hi,n],[w,hi,s],[e,hi,s],[e,hi,n]],[0.,1.,0.]),
   ([[w,lo,n],[e,lo,n],[e,lo,s],[w,lo,s]],[0.,-1.,0.]),
   ([[w,lo,s],[e,lo,s],[e,hi,s],[w,hi,s]],[0.,0.,1.]),
   ([[e,lo,n],[w,lo,n],[w,hi,n],[e,hi,n]],[0.,0.,-1.]),
   ([[w,lo,n],[w,lo,s],[w,hi,s],[w,hi,n]],[-1.,0.,0.]),
   ([[e,lo,s],[e,lo,n],[e,hi,n],[e,hi,s]],[1.,0.,0.]),
  ] {append_quad(&mut mesh.textured,p,n,[paint;4],TEXTURED_SHADE);}
 };
 // Two side wings and center landing surround a compact machine body.
 box_part([48.,64.,16.,48.],0.,4.,body);
 box_part([96.,112.,16.,48.],0.,4.,body);
 box_part([64.,96.,8.,48.],0.,4.,body);
 for step in 0..4 {box_part([64.,96.,48.+step as f32*2.,50.+step as f32*2.],0.,(4-step)as f32,dark);}
 box_part([62.,98.,18.,42.],4.,8.,dark);
 box_part([64.,96.,8.,30.],8.,24.,body);
 box_part([66.,94.,30.,38.],8.,10.,light);
 box_part([64.,68.,30.,40.],4.,13.,body);
 box_part([92.,96.,30.,40.],4.,13.,body);
 // Preserve the verified screen/frame drawing on the upright front instead
 // of reducing its control details to a uniform palette patch.
 for row in 0..2 {for col in 0..4 {
  let w=wx+(64.+col as f32*8.)*sx;let e=w+8.*sx;let front=nz+30.25*sy;
  let top=(24.-row as f32*8.)*sy;let bottom=top-8.*sy;
  let uv=g.uv((8+col-origin.x)as usize,(2+row-origin.y)as usize);
  append_quad(&mut mesh.textured,[[w,bottom,front],[e,bottom,front],
   [e,top,front],[w,top,front]],[0.,0.,1.],
   [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
 }}
 for col in 0..4 {
  let w=wx+(64.+col as f32*8.)*sx;let e=w+8.*sx;
  for (source_row,front,back,height) in [(1,8.,30.,24.05),(4,30.,38.,10.05)] {
   let uv=g.uv((8+col-origin.x)as usize,(source_row-origin.y)as usize);
   append_quad(&mut mesh.textured,[[w,height*sy,nz+front*sy],[w,height*sy,nz+back*sy],
    [e,height*sy,nz+back*sy],[e,height*sy,nz+front*sy]],[0.,1.,0.],
    [[uv.0,uv.2],[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2]],TEXTURED_SHADE);
  }
  for step in 0..4 {
   let uv=g.uv((8+col-origin.x)as usize,(6-origin.y)as usize);
   let north=nz+(48.+step as f32*2.)*sy;let south=north+2.*sy;
   let height=((4-step)as f32+0.05)*sy;
   let v0=uv.2+(uv.3-uv.2)*step as f32/4.;let v1=uv.2+(uv.3-uv.2)*(step+1)as f32/4.;
   append_quad(&mut mesh.textured,[[w,height,north],[w,height,south],[e,height,south],[e,height,north]],
    [0.,1.,0.],[[uv.0,v0],[uv.0,v1],[uv.1,v1],[uv.1,v0]],TEXTURED_SHADE);
  }
 }
 // The native side-wing borders and front landing are horizontal parts of
 // this complete drawing. Preserve their pixels on the existing platform.
 for (left,top,width,depth) in [(6,2,2,4),(12,2,2,4),(8,5,4,1)] {
  for y in top..top+depth {for x in left..left+width {
   let w=wx+x as f32*8.*sx;let e=w+8.*sx;
   let n=nz+y as f32*8.*sy;let s=n+8.*sy;
   let uv=g.uv((x-origin.x)as usize,(y-origin.y)as usize);
   append_quad(&mut mesh.textured,[[w,4.05*sy,n],[w,4.05*sy,s],
    [e,4.05*sy,s],[e,4.05*sy,n]],[0.,1.,0.],
    [[uv.0,uv.2],[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2]],TEXTURED_SHADE);
  }}
 }
}

pub(super) fn support_correction(frame:&VisualWorldFrame,cell:UVec2,foot:bevy::prelude::Vec2,resolved:&[f32])->f32 {
 if frame.map_id.as_ref()!="HallOfFame" {return 0.;}
 let start=frame.grid_origin+cell.as_ivec2();
 if start.y!=6||!(8..12).contains(&start.x) {return 0.;}
 let g=GridGeometry {width:frame.grid_size.x as usize,height:frame.grid_size.y as usize,
  tile_width:frame.tile_size.x,tile_height:frame.tile_size.y,origin_x:0.,origin_z:0.};
 let mut order=vec![None;g.width*g.height];
 for tile in &frame.tiles {
  let i=tile.row as usize*g.width+tile.column as usize;
  if i>=order.len()||order[i].is_some() {return 0.;}order[i]=Some(tile);
 }
 let Some(cells)=order.into_iter().collect::<Option<Vec<_>>>() else {return 0.;};
 if !complete(frame,&cells,&g) {return 0.;}
 let scale=frame.tile_size.y/8.;
 for x in 8..12 {
  let i=(6-frame.grid_origin.y)as usize*g.width+(x-frame.grid_origin.x)as usize;
  if resolved.get(i).is_none_or(|h|(*h-2.*scale).abs()>0.001) {return 0.;}
 }
 let size=frame.grid_size.as_vec2()*frame.tile_size;
 let y=(frame.center.y+size.y*0.5-foot.y)/frame.tile_size.y+frame.grid_origin.y as f32;
 let tread=(4.-((y-6.).clamp(0.,0.99999)*4.).floor()).clamp(1.,4.);
 (tread-2.)*scale
}

#[cfg(test)]
mod tests {
 use super::*;
 use bevy::prelude::{IVec2,Vec2};
 fn fixture()->(VisualWorldFrame,GridGeometry,TerrainMeshData) {
  let g=GridGeometry {width:20,height:12,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
  let mut tiles=Vec::new();let mut mesh=TerrainMeshData::default();
  for y in 0..12 {for x in 0..20 {
   let block=if y<8&&(4..16).contains(&x) {[[0x30,0x31,0x32],[0x34,0x35,0x36]][y/4][(x-4)/4]}else{0x2b};
   let tile=drawing(block).map_or([[12,13],[28,29]][y%2][x%2],|d|d[y%4][x%4]);
   tiles.push(VisualTile {column:x as u32,row:y as u32,texture:Default::default(),priority:false,
    animation_frames:None,source:VisualTileSource {tileset_id:"ice_path".into(),metatile_id:block,
     subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:tile}});
   append_top(&mut mesh.textured,g.bounds(x,y).into(),0.,g.uv(x,y));
  }}
  mesh.footing_heights=vec![0.;240];mesh.authored_cells=vec![None;240];
  let frame=VisualWorldFrame { source_regions: Default::default(), view_center: Vec2::ZERO, terrace_datum: None,active:true,map_id:"HallOfFame".into(),source_map_size_core_tiles:Some(UVec2::new(10,14)),
   boundary_warps:Default::default(),jump_ledges:Default::default(),terrain_revision:0,grid_origin:IVec2::ZERO,map_texture:Default::default(),
   center:Vec2::ZERO,viewport_size:Vec2::new(160.,96.),tile_size:Vec2::splat(8.),grid_size:UVec2::new(20,12),tiles,actors:Vec::new()};
  (frame,g,mesh)
 }
 #[test]
 fn native_hall_recording_balls_attach_to_the_tray_and_reject_changed_owners() {
  use crystal_render_api::{VisualActor,VisualActorId,VisualEffectId};
  let (mut frame,_,_)=fixture();
  let profiles:crate::live_profiles::Document=serde_json::from_str(include_str!("../../../../modpacks/voxel-view/profiles.json")).unwrap();
  assert!(effects_supported(&frame,Some(&profiles)));
  let centers=[(77.,32.),(82.,32.),(73.,31.),(86.,31.),(69.,29.),(89.,29.)];
  for (slot,(x,y)) in centers.into_iter().enumerate() {
   let mut actor=VisualActor{id:VisualActorId::Effect(VisualEffectId::HealMachineBall(slot as u8)),
    source_id:"effect_heal_ball".into(),texture:Default::default(),center:Vec2::new(-80.+x,48.-y),
    size:Vec2::splat(8.),flip_x:false,above_priority:false,facing:None,model_palette:None};
   let pose=crate::healing_effects::transform(&frame,&actor).unwrap();
   assert!(pose.transform_point(Vec3::new(0.,0.5,0.)).abs_diff_eq(Vec3::new(-80.+x,10.15,-48.+y+4.),0.0001),
    "the rendered sprite center, not its bottom pivot, must match the tray slot");
   assert!((pose.rotation*Vec3::Z).abs_diff_eq(Vec3::Y,0.0001),"source card must lie on the tray");
   assert_eq!(pose.scale,Vec3::new(8.,8.,1.));
   frame.center+=Vec2::new(17.,-9.);actor.center+=Vec2::new(17.,-9.);
   let moved=crate::healing_effects::transform(&frame,&actor).unwrap();
   assert!((moved.translation-pose.translation).abs_diff_eq(Vec3::new(17.,0.,9.),0.0001));
   frame.center=Vec2::ZERO;
  }
  // Reproduce the real expanded publication and the captured first ball.
  // This catches confusing raw OAM coordinates with map-space actor centers.
  let mut expanded=frame.clone();expanded.grid_origin=IVec2::new(-48,-48);
  expanded.grid_size=UVec2::new(116,114);expanded.tile_size=Vec2::splat(32.);
  expanded.viewport_size=Vec2::new(640.,576.);expanded.tiles.clear();
  for row in 0..114 {for col in 0..116 {
   let x=col as i32-48;let y=row as i32-48;
   let mut tile=if (0..20).contains(&x)&&(0..12).contains(&y) {frame.tiles[y as usize*20+x as usize].clone()}else{frame.tiles[0].clone()};
   tile.column=col;tile.row=row;expanded.tiles.push(tile);
  }}
  let mut actor=VisualActor{id:VisualActorId::Effect(VisualEffectId::HealMachineBall(0)),
   source_id:"effect_heal_ball".into(),texture:Default::default(),center:Vec2::new(-12.,160.),
   size:Vec2::splat(32.),flip_x:false,above_priority:false,facing:None,model_palette:None};
  let pose=crate::healing_effects::transform(&expanded,&actor).unwrap();
  assert!(pose.transform_point(Vec3::new(0.,0.5,0.)).abs_diff_eq(Vec3::new(-12.,40.6,-144.),0.001));
  actor.center.y-=16.*4.;assert!(crate::healing_effects::transform(&expanded,&actor).is_none(),"untranslated OAM cannot masquerade as a map-space actor");
  for index in 0..frame.tiles.len() {
   let x=index%20;let y=index/20;if y>=8||!(4..16).contains(&x){continue;}
   frame.tiles[index].source.tile_index^=128;assert!(!effects_supported(&frame,Some(&profiles)));
   frame.tiles[index].source.tile_index^=128;
  }
  let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom machine","map":"HallOfFame","tileset":"ice_path","metatile":49,"origin":[0,1],"tiles":[[44,45,45,46]],"ground":12,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
  assert!(!effects_supported(&frame,Some(&custom)));
  assert!(!crate::healing_effects::profile_allows(&frame,&custom));
  frame.tiles.reverse();assert!(effects_supported(&frame,None),"ordering is not ownership");
  frame.tiles[0]=frame.tiles[1].clone();assert!(!effects_supported(&frame,None),"duplicate cells cannot authenticate a machine");
 }

 #[test]
 fn kanto_hall_machine_has_closed_volume_native_claims_and_each_stair_tread() {
  let (frame,g,mut mesh)=fixture();let refs=frame.tiles.iter().collect::<Vec<_>>();
  finish(&mut mesh,&frame,&refs,&g,None);
  assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("hall:recording-machine")).count(),46);
  assert_eq!(mesh.footing_heights[8*20+9],0.,"player dialogue aisle remains native floor");
  assert_eq!(mesh.footing_heights[2*20+4],0.,"guard-only floor remains independent");
  assert!(mesh.textured.positions.iter().any(|p|p[1]==24.),"machine must have physical height");
  for quarter in 0..4 {
   let foot=Vec2::new(-80.+9.5*8.,48.-(6.+(quarter as f32+0.5)/4.)*8.);
   assert_eq!(crate::footing::resolved_footing_height(&frame,foot,&mesh.footing_heights),Some((4-quarter)as f32));
  }
  let mut front_area=0.;
  for tri in mesh.textured.indices.chunks_exact(3) {
   let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
   let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
   assert!((b-a).cross(c-a).dot(n)>0.,"all machine/platform faces point outward");
   if n==Vec3::Z && [a,b,c].iter().all(|p|p.z==30.25) {
    front_area+=(b-a).cross(c-a).length()*0.5;
    assert!(tri.iter().any(|&i|mesh.textured.uvs[i as usize]!=mesh.textured.uvs[tri[0]as usize]),
      "screen/frame faces must preserve spatial source detail");
   }
  }
  assert_eq!(front_area,32.*16.,"complete upright recording face must retain native detail");
  let mut platform_area=0.;
  for tri in mesh.textured.indices.chunks_exact(3) {
   let ps=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
   if mesh.textured.normals[tri[0]as usize]!=[0.,1.,0.] || !ps.iter().all(|p|(p.y-4.05).abs()<0.001) {continue;}
   let c=(ps[0]+ps[1]+ps[2])/3.;
   let wing=((48.0..64.0).contains(&c.x)||(96.0..112.0).contains(&c.x))&&(16.0..48.0).contains(&c.z);
   let landing=(64.0..96.0).contains(&c.x)&&(40.0..48.0).contains(&c.z);
   if !wing&&!landing {continue;}
   platform_area+=(ps[1]-ps[0]).cross(ps[2]-ps[0]).length()*0.5;
   let uv=g.uv((c.x/8.).floor()as usize,(c.z/8.).floor()as usize);
   assert!(tri.iter().all(|&i| {let p=mesh.textured.uvs[i as usize];p[0]>=uv.0&&p[0]<=uv.1&&p[1]>=uv.2&&p[1]<=uv.3}),
    "platform details retain their spatial source tiles");
  }
  assert_eq!(platform_area,2.*16.*32.+32.*8.,"both wings and the exposed landing retain their native borders");
 }
 #[test]
 fn kanto_hall_machine_rejects_each_altered_guard_cell_and_other_maps() {
  let (mut frame,g,baseline)=fixture();
  for y in 0..8 {for x in 4..16 {
   let i=y*20+x;frame.tiles[i].source.tile_index^=1;let mut mesh=baseline.clone();
   finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
   assert_eq!(mesh,baseline,"altered complete source must retain native art");frame.tiles[i].source.tile_index^=1;
  }}
  let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom machine","map":"HallOfFame","tileset":"ice_path","metatile":49,"origin":[0,1],"tiles":[[44,45,45,46]],"ground":12,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
  let refs=frame.tiles.iter().collect::<Vec<_>>();
  assert!(!live::resolve(&refs,g.width,g.height,"HallOfFame",Some(&custom)).is_empty(),"custom fixture must match");
  let mut mesh=baseline.clone();finish(&mut mesh,&frame,&refs,&g,Some(&custom));
  assert_eq!(mesh,baseline,"custom machine source must remain authoritative");
  frame.source_map_size_core_tiles=Some(UVec2::new(10,16));let mut mesh=baseline.clone();
  finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(mesh,baseline);
  frame.source_map_size_core_tiles=Some(UVec2::new(10,14));
  frame.map_id="IcePath1F".into();let mut mesh=baseline.clone();
  finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(mesh,baseline);
 }
 #[test]
 fn kanto_hall_terminal_uses_live_source_for_complete_existing_volume() {
  let model=crate::dungeon_models::extension_model(crate::dungeon_models::ExtensionKind::HallOfFameTerminal);
  let mut native=SurfaceMeshData::default();model.append(&mut native,[0.,16.,0.,10.],0.,22.);
  let mut painted=SurfaceMeshData::default();
  assert!(model.append_native_hall_terminal(&mut painted,[0.,16.,0.,10.],22.,|p|[p[0]as f32,p[1]as f32]));
  assert_eq!(painted.positions,native.positions);assert_eq!(painted.normals,native.normals);assert_eq!(painted.indices,native.indices);
  for uv in [[4.,4.],[4.,12.],[2.,2.],[4.,20.]] {assert!(painted.uvs.contains(&uv));}
 }
}
