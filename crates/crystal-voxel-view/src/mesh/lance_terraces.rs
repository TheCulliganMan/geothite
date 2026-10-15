//! Lance's own native champions-room platform and three broad stair courses.
use super::*;
use bevy::prelude::{Vec2,UVec2};
fn drawing(block:u16)->Option<[[u16;4];4]> {Some(match block {
 0x28=>[[202,176,176,176],[203,177,221,177],[204,178,222,178],[201,192,192,192]],
 0x2c=>[[176,176,193,194],[221,177,195,196],[222,178,197,198],[192,192,197,198]],
 0x2e=>[[176;4],[180,181,182,183],[184,185,186,187],[188,189,190,191]],
 0x0b=>[[176;4],[180,213,214,183],[215,216,216,217],[218,219,219,220]],
 0x2d=>[[193,194,176,176],[195,196,177,221],[197,198,178,222],[197,198,192,192]],
 0x29=>[[176,176,176,206],[177,221,177,207],[178,222,178,208],[192,192,192,205]],
 0x2f=>[[129,93,136,137],[130,93,136,137],[130,93,136,137],[128,93,136,137]],
 0x32=>[[210,210,199,200],[210,211,211,211],[211,210,211,211],[211,211,210,211]],
 0x15=>[[209,163,164,209],[165,166,167,168],[169,170,171,172],[173,174,175,212]],
 0x17=>[[199,200,210,210],[211;4],[211,211,210,211],[211,211,211,210]],
 0x30=>[[139,140,93,133],[139,140,93,134],[139,140,93,135],[139,140,93,132]],
 0x3a=>[[129,93,136,137],[130,93,136,137],[131,93,136,144],[128,93,141,142]],
 0x3e=>[[161,211,211,211],[162,210,211,210],[145;4],[142;4]],
 0x3d=>[[211;4],[59,60,60,61],[148;4],[147;4]],
 0x3f=>[[210,211,211,161],[210,210,211,162],[145;4],[142;4]],
 0x3b=>[[139,140,93,133],[139,140,93,134],[146,140,93,135],[142,143,93,132]],
 _=>return None,
})}
fn block(x:i32,y:i32)->u16 {
 [[0x28,0x2c,0x2e,0x2d,0x29],[0x2f,0x32,0x15,0x17,0x30],
  [0x3a,0x3e,0x3d,0x3f,0x3b]][y as usize/4][x as usize/4]
}
fn complete(frame:&VisualWorldFrame,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if frame.map_id.as_ref()!="LancesRoom"||cells.len()!=g.width*g.height
  ||frame.source_map_size_core_tiles.is_some_and(|s|s!=UVec2::new(10,24)) {return false;}
 for y in 0..12 {for x in 0..20 {
  let lx=x-frame.grid_origin.x;let ly=y-frame.grid_origin.y;
  if lx<0||ly<0||lx as usize>=g.width||ly as usize>=g.height {return false;}
  let s=&cells[ly as usize*g.width+lx as usize].source;let b=block(x,y);
  if s.tileset_id.as_ref()!="champions_room"||s.subtile_column!=x as u8%4||s.subtile_row!=y as u8%4
   ||(s.metatile_id!=b&&!(b==0x2e&&s.metatile_id==0x0b))
   ||drawing(s.metatile_id).is_none_or(|d|d[y as usize%4][x as usize%4]!=s.tile_index) {return false;}
 }}true
}
fn stair(p:Vec2)->bool {(2.0..18.0).contains(&p.x)&&(9.0..12.0).contains(&p.y)}
fn height(p:Vec2)->f32 {
 if !(0.0..20.0).contains(&p.x)||p.y<0.||p.y>=12. {return 0.;}
 if stair(p) {16.*(11.-p.y.floor())/3.}else{16.}
}
pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,cells:&[&VisualTile],
 g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
 if !complete(frame,cells,g) {return;}
 let origin=frame.grid_origin;
 if live::resolve(cells,g.width,g.height,"LancesRoom",profiles).iter().any(|p|
  p.indices(g.width).any(|i| {
   let x=origin.x+(i%g.width)as i32;let y=origin.y+(i/g.width)as i32;
   (0..20).contains(&x)&&(0..12).contains(&y)
  })) {return;}
 let scale=g.tile_height/8.;
 for surface in [&mut mesh.textured,&mut mesh.solid,&mut mesh.animated_textured,&mut mesh.animated_solid] {
  let mut rebuilt=SurfaceMeshData::default();
  let mut owners=vec![None;surface.positions.len()];
  for (group,range) in surface.cutaway_ranges.iter().enumerate() {for i in range.clone() {owners[i]=Some(group);}}
  let mut triangles=vec![Vec::new();surface.cutaway_ranges.len()+1];
  for tri in surface.indices.chunks_exact(3) {
   let c=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize])).sum::<Vec3>()/3.;
   let p=Vec2::new((c.x-g.origin_x)/g.tile_width+origin.x as f32,(c.z-g.origin_z)/g.tile_height+origin.y as f32);
   if stair(p) {continue;}
   let owner=owners[tri[0]as usize];
   debug_assert!(tri.iter().all(|&i|owners[i as usize]==owner));
   triangles[owner.map_or(0,|i|i+1)].push((tri.to_vec(),height(p)*scale));
  }
  for (group,tris) in triangles.into_iter().enumerate() {
   let start=rebuilt.positions.len();let mut mapping=std::collections::HashMap::new();
   for (tri,lift) in tris {for old in tri {
    let i=*mapping.entry((old,lift.to_bits())).or_insert_with(|| {
     let i=rebuilt.positions.len()as u32;let mut p=surface.positions[old as usize];p[1]+=lift;
     rebuilt.positions.push(p);rebuilt.normals.push(surface.normals[old as usize]);
     rebuilt.uvs.push(surface.uvs[old as usize]);rebuilt.colors.push(surface.colors[old as usize]);i
    });rebuilt.indices.push(i);
   }}
   if group>0 {rebuilt.cutaway_ranges.push(start..rebuilt.positions.len());}
  }
  *surface=rebuilt;
 }
 for (i,h) in mesh.footing_heights.iter_mut().enumerate() {
  let p=Vec2::new(origin.x as f32+(i%g.width)as f32+0.5,origin.y as f32+(i/g.width)as f32+0.5);
  *h+=height(p)*scale;
 }
 for y in 9..12 {for x in 2..18 {
  let lx=(x-origin.x)as usize;let ly=(y-origin.y)as usize;
  let (w,e,n,s)=g.bounds(lx,ly);let uv=g.uv(lx,ly);
  let top=height(Vec2::new(x as f32+0.5,y as f32+0.5))*scale;
  let previous=16.*(12-y)as f32/3.*scale;
  append_quad(&mut mesh.textured,[[w,top,n],[w,top,s],[e,top,s],[e,top,n]],
   [0.,1.,0.],[[uv.0,uv.2],[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2]],TEXTURED_SHADE);
  append_quad(&mut mesh.textured,[[w,top,n],[e,top,n],[e,previous,n],[w,previous,n]],
   [0.,0.,1.],[[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
  if x==2||x==17 {
   let plane=if x==2 {w}else{e};
   let points=if x==2 {[[plane,top,s],[plane,top,n],[plane,16.*scale,n],[plane,16.*scale,s]]}
    else {[[plane,top,n],[plane,top,s],[plane,16.*scale,s],[plane,16.*scale,n]]};
   append_quad(&mut mesh.textured,points,[if x==2 {1.}else{-1.},0.,0.],
    [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
  }
  let i=ly*g.width+lx;
  if mesh.authored_cells.len()==cells.len() {mesh.authored_cells[i]=Some("lance:terrace-stair");}
 }}
 for x in (0..2).chain(18..20) {
  let lx=(x-origin.x)as usize;let ly=(11-origin.y)as usize;
  let (w,e,_,s)=g.bounds(lx,ly);let uv=g.uv(lx,ly);
  append_quad(&mut mesh.textured,[[w,0.,s],[e,0.,s],[e,16.*scale,s],[w,16.*scale,s]],
   [0.,0.,1.],[[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 use bevy::prelude::IVec2;
 fn fixture()->(VisualWorldFrame,GridGeometry,TerrainMeshData) {
  let g=GridGeometry {width:20,height:16,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
  let mut mesh=TerrainMeshData::default();let mut tiles=Vec::new();
  for y in 0..16 {for x in 0..20 {
   let b=if y<12 {block(x as i32,y as i32)}else{0x31};
   let tile=if y<12 {drawing(b).unwrap()[y%4][x%4]}else{59};
   tiles.push(VisualTile {column:x as u32,row:y as u32,texture:Default::default(),priority:false,
    animation_frames:None,source:VisualTileSource {tileset_id:"champions_room".into(),metatile_id:b,
     subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:tile}});
   append_top(&mut mesh.textured,g.bounds(x,y).into(),0.,g.uv(x,y));
  }}
  mesh.footing_heights=vec![0.;320];mesh.authored_cells=vec![None;320];
  mesh.textured.cutaway_ranges=vec![0..4];
  let frame=VisualWorldFrame {active:true,map_id:"LancesRoom".into(),source_map_size_core_tiles:Some(UVec2::new(10,24)),
   jump_ledges:Default::default(),terrain_revision:0,grid_origin:IVec2::ZERO,map_texture:Default::default(),
   center:Vec2::ZERO,viewport_size:Vec2::new(160.,128.),tile_size:Vec2::splat(8.),grid_size:UVec2::new(20,16),tiles,actors:Vec::new()};
  (frame,g,mesh)
 }
 #[test]
 fn kanto_lance_platform_has_three_broad_steps_and_preserves_fade_owners() {
  let (mut frame,g,baseline)=fixture();
  for open in [false,true] {
   if open {for y in 0..4 {for x in 8..12 {
    let source=&mut frame.tiles[y*20+x].source;source.metatile_id=0x0b;source.tile_index=drawing(0x0b).unwrap()[y][x%4];
   }}}
   let mut mesh=baseline.clone();finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
   assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("lance:terrace-stair")).count(),48);
   assert_eq!(mesh.textured.cutaway_ranges.len(),1);
   assert_eq!(mesh.textured.cutaway_ranges[0].len(),4,"complete original fade owner retained");
   assert_eq!(mesh.footing_heights[7*20+9],16.,"upper trainer/player landing");
   assert_eq!(mesh.footing_heights[13*20+9],0.,"lower carpet datum unchanged");
   for row in 9..12 {
    let foot=Vec2::new(-80.+9.5*8.,64.-(row as f32+0.5)*8.);
    assert_eq!(crate::footing::resolved_footing_height(&frame,foot,&mesh.footing_heights),Some(16.*(11-row)as f32/3.));
   }
   for tri in mesh.textured.indices.chunks_exact(3) {
    let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
    let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
    assert!((b-a).cross(c-a).dot(n)>0.,"all native treads and closures point outward");
   }
  }
 }
 #[test]
 fn kanto_lance_terraces_require_all_native_source_cells_and_yield_to_customization() {
  let (mut frame,g,baseline)=fixture();
  for y in 0..12 {for x in 0..20 {
   let i=y*20+x;frame.tiles[i].source.tile_index^=1;let mut mesh=baseline.clone();
   finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
   assert_eq!(mesh,baseline,"no partial lift after changing native source");frame.tiles[i].source.tile_index^=1;
  }}
  let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom stair course","map":"LancesRoom","tileset":"champions_room","metatile":61,"origin":[0,0],"tiles":[[211,211,211,211]],"ground":211,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
  let refs=frame.tiles.iter().collect::<Vec<_>>();
  assert!(!live::resolve(&refs,g.width,g.height,"LancesRoom",Some(&custom)).is_empty());
  let mut mesh=baseline.clone();finish(&mut mesh,&frame,&refs,&g,Some(&custom));assert_eq!(mesh,baseline);
  frame.source_map_size_core_tiles=Some(UVec2::new(10,18));let mut mesh=baseline.clone();
  finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(mesh,baseline);
 }
}
