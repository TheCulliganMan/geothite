//! Exact Mount Moon fixtures and source-complete traditional-house cushions.
use super::*;
pub(super) fn traditional_table(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if !traditional_room::floor_map(map)||p.kind!=ModelKind::LowTable
    ||p.width!=4||p.height!=4||p.base_pixels!=0.0
    ||g.width.checked_mul(g.height)!=Some(cells.len())
    ||p.column+4>g.width||p.row+4>g.height {return false;}
 let art=[[0x23,0x22,0x22,0x24],[0x42,0x15,0x15,0x43],
    [0x42,0x15,0x15,0x43],[0x33,0x32,0x32,0x34]];
 p.indices(g.width).enumerate().all(|(j,i)| {
  let x=j%4;let y=j/4;let s=&cells[i].source;
  let block=match (x<2,y<2) {(true,true)=>0x1b,(false,true)=>0x1c,
    (true,false)=>0x18,(false,false)=>0x19};
  s.tileset_id.as_ref()=="traditional_house"&&s.metatile_id==block
   &&s.subtile_column==(if x<2 {(x+2)as u8}else{(x-2)as u8})
   &&s.subtile_row==(if y<2 {(y+2)as u8}else{(y-2)as u8})
   &&s.tile_index==art[y][x]
 })
}
pub(super) fn traditional_cushion(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->bool {
 if !traditional_room::floor_map(map)||p.kind!=ModelKind::Cushion
    ||p.width!=2||p.height!=2||p.base_pixels!=0.0||g.width.checked_mul(g.height)!=Some(cells.len())
    ||p.column+2>g.width||p.row+2>g.height {return false;}
 let first=&cells[p.row*g.width+p.column].source;
 let origin=match first.metatile_id {0x0e=>[0,0],0x10|0x19=>[2,0],0x1b=>[0,2],_=>return false};
 p.indices(g.width).enumerate().all(|(j,i)| {
  let s=&cells[i].source;
  s.tileset_id.as_ref()=="traditional_house"&&s.metatile_id==first.metatile_id
   &&s.subtile_column==origin[0]+(j%2)as u8&&s.subtile_row==origin[1]+(j/2)as u8
   &&s.tile_index==[0x02,0x03,0x12,0x13][j]
 })
}
pub(super) fn append_counter(map:&str,cells:&[&VisualTile],g:&GridGeometry,claimed:&mut[bool],out:&mut Vec<Placement>) {
 if map!="MountMoonGiftShop"||g.width<2||g.height<4 {return;}
 let Some(ground)=ground_sample(cells,map,"traditional_house",17)else{return;};
 for row in 0..=g.height-4 {for column in 0..=g.width-2 {
  let p=Placement{kind:ModelKind::LabCounter,row,column,width:2,height:4,ground,depth_pixels:24.,height_pixels:8.,base_pixels:0.,footing_pixels:None};
  if p.indices(g.width).any(|i|claimed[i])||source(map,&p,cells,g).is_none() {continue;}
  for i in p.indices(g.width){claimed[i]=true;}out.push(p);
 }}
}
pub(super) fn source(map:&str,p:&Placement,cells:&[&VisualTile],g:&GridGeometry)->Option<ModelKind> {
 let traditional=traditional_room::floor_map(map)
    &&matches!(p.kind,ModelKind::PendulumClock|ModelKind::Cabinet|ModelKind::DrawerCabinet
        |ModelKind::WallTraditional|ModelKind::RadioWall);
 if map!="MountMoonGiftShop" && !(map=="MahoganyMart1F"&&p.kind==ModelKind::GiftShelf)
    &&!traditional {return None;}
 if p.base_pixels!=0.0||g.width.checked_mul(g.height)!=Some(cells.len())
    ||p.column+p.width>g.width||p.row+p.height>g.height {return None;}
 let (block,origin,art):(_,_,&[u16])=match (p.kind,p.width,p.height) {
  (ModelKind::LabCounter,2,4)=>(18,[0,0],&[35,36,66,67,66,67,51,52]),
  (ModelKind::GiftShelf,4,4)=>(2,[0,0],&[6,7,7,32,22,23,23,48,33,39,39,40,49,55,55,56]),
  (ModelKind::WallTraditional,4,4)=>(8,[0,0],&[17,17,17,17,17,17,17,17,80,80,80,80,80,80,80,80]),
  (ModelKind::RadioWall,4,4)=>(1,[0,0],&[17,17,17,17,17,17,17,17,10,11,80,80,26,27,80,80]),
  (ModelKind::PendulumClock,2,4)=>(19,[0,0],&[35,36,41,42,57,58,82,83]),
  (ModelKind::Cabinet,2,3) if cells[p.row*g.width+p.column].source.metatile_id==41
    =>(41,[0,1],&[35,36,8,9,82,83]),
  (ModelKind::Cabinet,2,3)=>(19,[2,1],&[35,36,62,63,24,25]),
  (ModelKind::DrawerCabinet,2,4)=>(41,[2,0],&[35,36,30,31,46,47,24,25]),
  (ModelKind::Cushion,2,2)=>(16,[2,0],&[2,3,18,19]),_=>return None,
 };
 p.indices(g.width).enumerate().all(|(j,i)| {
  let s=&cells[i].source;
  s.tileset_id.as_ref()=="traditional_house"&&s.metatile_id==block
   &&s.subtile_column==origin[0]+(j%p.width)as u8&&s.subtile_row==origin[1]+(j/p.width)as u8&&s.tile_index==art[j]
 }).then_some(p.kind)
}
pub(super) fn append(out:&mut SurfaceMeshData,g:&GridGeometry,p:&Placement,kind:ModelKind,model:&crate::interior_models::Model,b:[f32;4]) {
 let scale=g.tile_height/8.;let height=p.height_pixels*scale;
 if matches!(kind,ModelKind::GiftShelf|ModelKind::LabCounter|ModelKind::WallTraditional|ModelKind::RadioWall) {
  let [w,e,_,s]=b;let shelf=kind==ModelKind::GiftShelf;let counter=kind==ModelKind::LabCounter;let top_rows=if counter {3}else{2};
  let h=if shelf {16.*scale}else{height};let n=s-if shelf||counter {top_rows as f32*8.*scale}else{3.*scale};
  let first=out.positions.len();let index=out.indices.len();architecture_box(out,[w,e,0.,h,n,s],[1.;3]);
  let pixel=if shelf {[3,1]}else{[2,0]};let paint=native_source_swatches(p,g,[pixel])[0];
  out.uvs[first..].fill(paint);out.indices.drain(index+6..index+12);
  if shelf||counter {
   // Replace the plain cap with the two authored plan-view merchandise rows.
   out.indices.drain(index..index+6);
   for y in 0..top_rows {for x in 0..p.width {let uv=g.uv(p.column+x,p.row+y);
    append_top(out,[w+(e-w)*x as f32/p.width as f32,w+(e-w)*(x+1)as f32/p.width as f32,n+(s-n)*y as f32/top_rows as f32,n+(s-n)*(y+1)as f32/top_rows as f32],h,uv);}}
  }
  let rows=if counter {1}else if shelf {2}else{p.height};let offset=if shelf||counter {top_rows}else{0};
  for y in 0..rows {for x in 0..p.width {
   let uv=g.uv(p.column+x,p.row+offset+y);let left=w+(e-w)*x as f32/p.width as f32;
   let right=w+(e-w)*(x+1)as f32/p.width as f32;let high=h*(rows-y)as f32/rows as f32;let low=h*(rows-y-1)as f32/rows as f32;
   append_quad(out,[[left,low,s],[right,low,s],[right,high,s],[left,high,s]],[0.,0.,1.],
    [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
  }}
 } else {
  let swatches=native_source_swatches(p,g,if kind==ModelKind::Cushion {
   [[0,0],[3,3],[2,1],[2,0]]
  }else{[[1,0],[2,3],[2,1],[2,0]]});
  if matches!(kind,ModelKind::Cabinet|ModelKind::DrawerCabinet) {assert!(model.append_native_home_fixture(out,b,0.,height,swatches));}
  else {assert!(model.append_native_gift_fixture(out,b,0.,height,swatches));}
  if kind==ModelKind::Cushion {
   let [w,e,n,s]=b;
   for y in 0..2 {for x in 0..2 {append_top(out,
    [w+(e-w)*x as f32/2.,w+(e-w)*(x+1)as f32/2.,n+(s-n)*y as f32/2.,n+(s-n)*(y+1)as f32/2.],
    height+0.002*scale,g.uv(p.column+x,p.row+y));}}
  }
 }
}
