//! Native idle healing enclosure with an empty recessed bay.
use super::*;

// Indigo's two stock cabinets share one complete block. The upper course is
// the visible roof; the remaining three courses are upright recessed shelves.
fn stock_block(map:&str,column:usize,row:usize,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if map!="IndigoPlateauPokecenter1F" || g.width.checked_mul(g.height)!=Some(cells.len())
  || column+4>g.width || row+4>g.height {return false;}
 (0..16).all(|j| {
  let s=&cells[(row+j/4)*g.width+column+j%4].source;
  s.tileset_id.as_ref()=="pokecenter" && s.metatile_id==0x2c
   && s.subtile_column==(j%4)as u8 && s.subtile_row==(j/4)as u8
   && s.tile_index==[[0x5c,0x5d,0x5c,0x5d],[0x5e,0x5f,0x5e,0x5f],
    [0x5e,0x5f,0x5e,0x5f],[0x2a,0x2b,0x2a,0x2b]][j/4][j%4]
 })
}
pub(super) fn stock_source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if p.kind!=ModelKind::RetailShelf || p.width!=2 || p.height!=4 || p.base_pixels!=0.
  || p.column>=g.width || p.row>=g.height || g.width.checked_mul(g.height)!=Some(cells.len()) {return false;}
 let offset=usize::from(cells[p.row*g.width+p.column].source.subtile_column);
 matches!(offset,0|2) && p.column>=offset
  && stock_block(map,p.column-offset,p.row,cells,g)
}
pub(super) fn append_stock_candidates(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
 if map!="IndigoPlateauPokecenter1F" || g.width<4 || g.height<4 {return;}
 let Some(ground)=ground_sample(cells,map,"pokecenter",0x11) else{return;};
 for row in 0..=g.height-4 {for column in 0..=g.width-4 {
  if !stock_block(map,column,row,cells,g)
   || (0..16).any(|j|claimed[(row+j/4)*g.width+column+j%4]) {continue;}
  for offset in [0,2] {
   let p=Placement{kind:ModelKind::RetailShelf,column:column+offset,row,width:2,height:4,
    ground,depth_pixels:8.,height_pixels:32.,base_pixels:0.,footing_pixels:None};
   for i in p.indices(g.width) {claimed[i]=true;}
   out.push(p);
  }
 }}
}
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
 append_healing_controls(out,g,p,b,height);
}
pub(super) fn append_terminal(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32) {
 append_enclosure(out,g,p,b,height,false);
}
// A closed, chamfered plate faces south. Its side faces give small source
// controls actual relief without substituting blocks for the apparatus.
fn front_plate(out:&mut SurfaceMeshData,rect:[f32;4],z:[f32;2],radius:f32,paint:[f32;2]) {
 let [w,e,low,high]=rect;let [rear,front]=z;
 let r=radius.min((e-w)*0.5).min((high-low)*0.5);
 let points=[[w+r,low],[e-r,low],[e,low+r],[e,high-r],
  [e-r,high],[w+r,high],[w,high-r],[w,low+r]];
 let c=[(w+e)*0.5,(low+high)*0.5];
 for i in 0..8 {
  let a=points[i];let b=points[(i+1)%8];
  triangle(out,[[c[0],c[1],front],[a[0],a[1],front],[b[0],b[1],front]],[0.,0.,1.],paint);
  triangle(out,[[c[0],c[1],rear],[b[0],b[1],rear],[a[0],a[1],rear]],[0.,0.,-1.],paint);
  let normal=Vec3::new(b[1]-a[1],a[0]-b[0],0.).normalize().to_array();
  append_quad(out,[[a[0],a[1],rear],[b[0],b[1],rear],[b[0],b[1],front],[a[0],a[1],front]],
   normal,[paint;4],TEXTURED_SHADE);
 }
}
fn append_healing_controls(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,b:[f32;4],height:f32) {
 let [w,e,_,south]=b;let scale=g.tile_height/8.;
 let left=w+0.5*scale;let right=e-0.5*scale;let width=right-left;
 let x=|pixel:f32|left+width*pixel/32.;let y=|pixel:f32|height*(1.-pixel/32.);
 let [metal,pale,red]=native_source_swatches(p,g,[[2,13],[3,13],[15,27]]);
 let back=south-0.05*scale;
 // The narrow frame stays outside the bay; the original source ball cards
 // and all sixteen live front-art cells keep their existing presentation.
 let rail=0.45*scale;let x0=x(8.);let x1=x(24.);let low=y(24.);let high=y(8.);
 for rect in [[x0-rail,x0,low-rail,high+rail],[x1,x1+rail,low-rail,high+rail],
  [x0,x1,low-rail,low],[x0,x1,high,high+rail]] {
  front_plate(out,rect,[back,south+0.45*scale],0.15*scale,metal);
 }
 // Four source side controls have a metal rim and a cream push surface.
 for source_x in [3.,29.] {for source_y in [12.,20.] {
  let cx=x(source_x);let cy=y(source_y);let half=0.85*scale;
  front_plate(out,[cx-half,cx+half,cy-half,cy+half],
   [back,south+0.45*scale],0.3*scale,metal);
  let half=0.5*scale;
  front_plate(out,[cx-half,cx+half,cy-half,cy+half],
   [south+0.4*scale,south+0.7*scale],0.2*scale,pale);
 }}
 // Retain the source's red service cross as a shallow raised badge.
 let cx=x(16.);let cy=y(28.);let h=1.5*scale;let bar=0.45*scale;
 for rect in [[cx-h,cx+h,cy-bar,cy+bar],[cx-bar,cx+bar,cy-h,cy+h]] {
  front_plate(out,rect,[back,south+0.35*scale],0.15*scale,red);
 }
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
 let recessed=s-if bay {4.}else{1.5}*scale;
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
