//! Native idle healing enclosure with an empty recessed bay.
use super::*;
pub(super) fn healing_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 kanto_native_floor_map(map)&&map.ends_with("Pokecenter1F")&&p.kind==ModelKind::HealingMachine
  &&p.width==4&&p.height==4&&p.indices(g.width).enumerate().all(|(j,i)| {
   let s=&cells[i].source;
   s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==1&&s.subtile_column==j as u8%4&&s.subtile_row==j as u8/4
    &&s.tile_index==[[28,29,30,31],[44,45,46,47],[60,61,61,63],[76,77,78,79]][j/4][j%4]
  })
}
pub(super) fn pc_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 kanto_native_floor_map(map)&&map.ends_with("Pokecenter1F")&&p.kind==ModelKind::Computer
  &&p.width==2&&p.height==3&&p.indices(g.width).enumerate().all(|(j,i)| {
   let s=&cells[i].source;
   s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==8&&s.subtile_column==2+j as u8%2&&s.subtile_row==1+j as u8/2
    &&s.tile_index==[[32,33],[48,49],[64,65]][j/2][j%2]
  })
}
pub(super) fn console_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 kanto_native_floor_map(map)&&map.ends_with("Pokecenter1F")&&p.kind==ModelKind::HealingConsole
  &&p.width==2&&p.height==2&&p.indices(g.width).enumerate().all(|(j,i)| {
   let s=&cells[i].source;
   s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==2&&s.subtile_column==j as u8%2&&s.subtile_row==2+j as u8/2
    &&s.tile_index==[[10,11],[26,27]][j/2][j%2]
  })
}
// The upper blue drawing belongs to the control bank, not a plain wall panel.
pub(super) fn cabinet_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 kanto_native_floor_map(map)&&map.ends_with("Pokecenter1F")&&p.kind==ModelKind::WallClinical
  &&p.width==2&&p.height==2&&p.indices(g.width).enumerate().all(|(j,i)| {
   let s=&cells[i].source;
   s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==2&&s.subtile_column==j as u8%2&&s.subtile_row==j as u8/2
    &&s.tile_index==[[4,5],[20,21]][j/2][j%2]
  })
}
pub(super) fn append_cabinet(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32) {
 let first=out.positions.len();
 append_terminal(out,g,p,b,height);
 // Align with the 13px lower console; artwork rows are vertical courses.
 for vertex in &mut out.positions[first..] {vertex[1]+=13.*g.tile_height/8.;}
}
pub(super) fn append_mirrored_wall(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut [bool],out:&mut Vec<Placement>) {
 if !kanto_native_floor_map(map)||!map.ends_with("Pokecenter1F") {return;}
 let Some(ground)=ground_sample(cells,map,"pokecenter",17) else{return;};
 if g.width<4||g.height<4 {return;}
 for row in 0..=g.height-4 {for column in 0..=g.width-4 {
  let complete=(0..16).all(|j| {
   let i=(row+j/4)*g.width+column+j%4;let s=&cells[i].source;
   !claimed[i]&&s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==42
    &&s.subtile_column==(j%4)as u8&&s.subtile_row==(j/4)as u8
    &&s.tile_index==[2,2,2,2,15,2,2,2,15,1,1,1,15,17,17,17][j]
  });
  if !complete {continue;}
  for offset in [0,2] {
   let p=Placement{kind:ModelKind::WallClinical,column:column+offset,row,width:2,height:2,
    ground,depth_pixels:3.,height_pixels:32.,base_pixels:0.,footing_pixels:None};
   for i in p.indices(g.width) {claimed[i]=true;}
   out.push(p);
  }
 }}
}
pub(super) fn append_pc_headers(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut [bool],out:&mut Vec<Placement>) {
 let pcs=out.iter().filter(|p|pc_source(map,p,cells,g)).cloned().collect::<Vec<_>>();
 for pc in pcs {
  if pc.row==0 {continue;}
  let p=Placement{kind:ModelKind::WallClinical,row:pc.row-1,height:1,
   depth_pixels:3.,height_pixels:16.,base_pixels:16.,footing_pixels:None,..pc};
  if p.indices(g.width).any(|i|claimed[i])||!wall_source(map,&p,cells,g) {continue;}
  for i in p.indices(g.width) {claimed[i]=true;}
  out.push(p);
 }
}
pub(super) fn wall_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if !kanto_native_floor_map(map)||!map.ends_with("Pokecenter1F")||p.kind!=ModelKind::WallClinical
  ||p.width!=2||!matches!(p.height,1|2) {return false;}
 let origin=&cells[p.row*g.width+p.column].source;
 let column=origin.subtile_column;
 if p.height==1 {
  return origin.metatile_id==8&&column==2&&p.base_pixels==16.&&p.height_pixels==16.
   &&p.indices(g.width).enumerate().all(|(j,i)| {
    let s=&cells[i].source;
    s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==8&&s.subtile_column==2+j as u8&&s.subtile_row==0&&s.tile_index==2
   });
 }
 let expected=match (origin.metatile_id,column) {
  (2,2)|(3,0)|(8,0)|(19,0)|(19,2)|(42,2)=>[2,2,2,2],
  (3,2)=>[2,2,2,15],
  (42,0)=>[2,2,15,2],
  _=>return false,
 };
 p.indices(g.width).enumerate().all(|(j,i)| {
  let s=&cells[i].source;
  s.tileset_id.as_ref()=="pokecenter"&&s.metatile_id==origin.metatile_id
   &&s.subtile_column==column+j as u8%2&&s.subtile_row==j as u8/2&&s.tile_index==expected[j]
 })
}
pub(super) fn append_wall(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32) {
 let [w,e,n,s]=b;let first=out.positions.len();let indices=out.indices.len();
 let base=p.base_pixels*g.tile_height/8.;
 architecture_box(out,[w,e,base,base+height,n,s],[1.;3]);
 let uv=g.uv(p.column,p.row);
 let paint=[uv.0+(uv.1-uv.0)*4.5/8.,uv.2+(uv.3-uv.2)*1.5/8.];
 out.uvs[first..].fill(paint);
 // The body is closed; replace its front rather than layering a second face.
 out.indices.drain(indices+6..indices+12);
 for y in 0..p.height {for x in 0..2 {
  let uv=g.uv(p.column+x,p.row+y);let left=w+(e-w)*x as f32/2.;let right=w+(e-w)*(x+1)as f32/2.;
  let high=base+height*(p.height-y)as f32/p.height as f32;let low=base+height*(p.height-1-y)as f32/p.height as f32;
  append_quad(out,[[right,low,s],[right,high,s],[left,high,s],[left,low,s]],[0.,0.,1.],
   [[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],TEXTURED_SHADE);
 }}
}
fn triangle(out:&mut SurfaceMeshData,p:[[f32;3];3],normal:[f32;3],uv:[f32;2]) {
 let first=out.positions.len()as u32;out.positions.extend(p);out.normals.extend([normal;3]);
 out.uvs.extend([uv;3]);out.colors.extend([TEXTURED_SHADE;3]);out.indices.extend([first,first+1,first+2]);
}
pub(super) fn append_healing(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32) {
 append_enclosure(out,g,p,b,height,true);
}
pub(super) fn append_terminal(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32) {
 append_enclosure(out,g,p,b,height,false);
}
fn append_enclosure(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32,bay:bool) {
 let [w,e,n,s]=b;let scale=g.tile_height/8.;let bevel=0.5*scale;
 let uv=g.uv(p.column,p.row+if bay {2}else{p.height-1});
 let paint=[uv.0+(uv.1-uv.0)*2.5/8.,uv.2+(uv.3-uv.2)*4.5/8.];
 let points=[[w+bevel,n],[e-bevel,n],[e,n+bevel],[e,s-bevel],
  [e-bevel,s],[w+bevel,s],[w,s-bevel],[w,n+bevel]];
 let center=[(w+e)*0.5,(n+s)*0.5];
 for i in 0..8 {
  let a=points[i];let b=points[(i+1)%8];
  triangle(out,[[center[0],height,center[1]],[b[0],height,b[1]],[a[0],height,a[1]]],[0.,1.,0.],paint);
  triangle(out,[[center[0],0.,center[1]],[a[0],0.,a[1]],[b[0],0.,b[1]]],[0.,-1.,0.],paint);
  if i!=4 {
   let normal=Vec3::new(b[1]-a[1],0.,a[0]-b[0]).normalize().to_array();
   append_quad(out,[[a[0],0.,a[1]],[a[0],height,a[1]],[b[0],height,b[1]],[b[0],0.,b[1]]],normal,[paint;4],TEXTURED_SHADE);
  }
 }
 let left=w+bevel;let right=e-bevel;let width=right-left;
 let recessed=s-1.5*scale;
 for row in 0..p.height {for col in 0..p.width {
  let x0=left+width*col as f32/p.width as f32;let x1=left+width*(col+1)as f32/p.width as f32;
  let top=height*(p.height-row)as f32/p.height as f32;let bottom=height*(p.height-1-row)as f32/p.height as f32;
  let z=if bay&&(1..3).contains(&col)&&(1..3).contains(&row) {recessed}else{s};
  let uv=g.uv(p.column+col,p.row+row);
  append_quad(out,[[x0,bottom,z],[x1,bottom,z],[x1,top,z],[x0,top,z]],[0.,0.,1.],
   [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
 }}
 if !bay {return;}
 let x0=left+width*0.25;let x1=left+width*0.75;let low=height*0.25;let high=height*0.75;
 for (q,normal) in [
  ([[x0,low,s],[x0,low,recessed],[x0,high,recessed],[x0,high,s]],[1.,0.,0.]),
  ([[x1,low,recessed],[x1,low,s],[x1,high,s],[x1,high,recessed]],[-1.,0.,0.]),
  ([[x0,low,recessed],[x0,low,s],[x1,low,s],[x1,low,recessed]],[0.,1.,0.]),
  ([[x0,high,s],[x0,high,recessed],[x1,high,recessed],[x1,high,s]],[0.,-1.,0.]),
 ] {append_quad(out,q,normal,[paint;4],TEXTURED_SHADE);}
}
