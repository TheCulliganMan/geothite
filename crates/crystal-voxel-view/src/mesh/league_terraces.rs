//! Native Elite Four landings and the complete central stair flight.
//! Heights are presentation geometry; the authoritative map stays two-dimensional.
use super::*;
use bevy::prelude::{Vec2,UVec2};
#[cfg(test)]
use bevy::prelude::IVec2;

fn supports(map:&str)->bool {
    matches!(map,"WillsRoom"|"KogasRoom"|"BrunosRoom"|"KarensRoom")
}
fn drawing(block:u16)->Option<[[u16;4];4]> {
    Some(match block {
        0x14=>[[0x27,0x26,0x25,0x27],[0x37,0x36,0x35,0x37],
            [0x25,0x27,0x27,0x26],[0x35,0x37,0x37,0x36]],
        0x15=>[[0x10;4],[0x10;4],[0x11;4],[1;4]],
        0x16=>[[0x11;4],[1;4],[1;4],[1;4]],
        0x22=>[[0x48,0x49,0x48,0x49],[0x4a,0x4b,0x4a,0x4b],[0x11;4],[1;4]],
        0x34=>[[0x39;4],[0x39;4],[0x51;4],[0x52;4]],
        1=>[[0x53,0x54,0x59,0x53];4],
        _=>return None,
    })
}
fn expected(map:&str,x:i32,y:i32,block:u16)->bool {
    if !(0..20).contains(&x)||!(0..12).contains(&y) {return false;}
    let column=x/4;let row=y/4;
    match row {
        0=>block==0x14,
        1 if column==2=>matches!(block,0x22|0x16),
        1=>block==0x15,
        2 if column==2=>block==1,
        _=>block==if map=="KarensRoom" {0x15}else{0x34},
    }
}
fn coherent(map:&str,cells:&[&VisualTile],g:&GridGeometry,origin:[i32;2])->bool {
    if !supports(map)||cells.len()!=g.width*g.height {return false;}
    let mut stairs=0;
    for (i,tile) in cells.iter().enumerate() {
        let x=origin[0]+(i%g.width)as i32;let y=origin[1]+(i/g.width)as i32;
        if !(0..20).contains(&x)||!(0..12).contains(&y) {continue;}
        let s=&tile.source;
        if s.tileset_id.as_ref()!="elite_four_room"||!expected(map,x,y,s.metatile_id)
            ||s.subtile_column as i32!=x%4||s.subtile_row as i32!=y%4
            ||drawing(s.metatile_id).is_none_or(|a|a[y as usize%4][x as usize%4]!=s.tile_index) {
            return false;
        }
        if s.metatile_id==1 {stairs+=1;}
    }
    stairs==16
}
fn stair(p:Vec2)->bool {(8.0..12.0).contains(&p.x)&&(8.0..12.0).contains(&p.y)}
fn stair_height(y:f32)->f32 {
    (14.-((y-8.)*2.).floor()*2.).clamp(0.,14.)
}
fn level(map:&str,p:Vec2)->f32 {
    if !(0.0..20.0).contains(&p.x)||p.y<0. {return 0.;}
    if p.y<4. {32.}
    else if p.y<8. {16.}
    else if stair(p) {stair_height(p.y)}
    else if map!="KarensRoom"&&p.y<10. {16.}
    else {0.}
}
fn map_point(g:&GridGeometry,origin:[i32;2],p:[f32;3])->Vec2 {
    Vec2::new((p[0]-g.origin_x)/g.tile_width+origin[0]as f32,
        (p[2]-g.origin_z)/g.tile_height+origin[1]as f32)
}

pub(super) fn finish(mesh:&mut TerrainMeshData,frame:&VisualWorldFrame,
    cells:&[&VisualTile],g:&GridGeometry,profiles:Option<&crate::live_profiles::Document>) {
    let origin=frame.grid_origin.to_array();let map=frame.map_id.as_ref();
    if frame.source_map_size_core_tiles.is_some_and(|s|s!=UVec2::new(10,18)) {return;}
    if !coherent(map,cells,g,origin) {return;}
    // Protect customized floor, stairs and architecture as one complete assembly.
    if live::resolve(cells,g.width,g.height,map,profiles).iter().any(|p|
        p.indices(g.width).any(|i| {
            let x=origin[0]+(i%g.width)as i32;let y=origin[1]+(i/g.width)as i32;
            (0..20).contains(&x)&&(0..12).contains(&y)
        })) {return;}
    let scale=g.tile_height/8.;
    for surface in [&mut mesh.textured,&mut mesh.solid,
        &mut mesh.animated_textured,&mut mesh.animated_solid] {
        let mut retained=Vec::with_capacity(surface.indices.len());
        let mut lifts:Vec<Option<f32>>=vec![None;surface.positions.len()];
        for tri in surface.indices.chunks_exact(3) {
            let center=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize]))
                .sum::<Vec3>()/3.;
            let p=map_point(g,origin,center.to_array());
            if stair(p)||(map!="KarensRoom"&&p.y>=10.&&p.y<11.
                &&(0.0..20.0).contains(&p.x)) {continue;}
            let h=level(map,p)*scale;
            for &i in tri {
                let slot=&mut lifts[i as usize];
                // Source cells use separate corners at each level seam.
                if let Some(previous)=slot {debug_assert!((*previous-h).abs()<0.001);}
                *slot=Some(h);
            }
            retained.extend_from_slice(tri);
        }
        surface.indices=retained;
        for (position,lift) in surface.positions.iter_mut().zip(lifts) {
            if let Some(h)=lift {position[1]+=h;}
        }
    }
    for (i,h) in mesh.footing_heights.iter_mut().enumerate() {
        let p=Vec2::new(origin[0]as f32+(i%g.width)as f32+0.5,
            origin[1]as f32+(i/g.width)as f32+0.5);
        *h+=if stair(p) {(16.-(p.y-8.)*4.)*scale}else{level(map,p)*scale};
    }
    // Two treads per native source row. Atlas UVs stay at their source phase.
    for row in 0..4 {for column in 0..4 {
        let x=(8-origin[0])as usize+column;let y=(8-origin[1])as usize+row;
        let (w,e,n,s)=g.bounds(x,y);let uv=g.uv(x,y);
        for half in 0..2 {
            let north=n+(s-n)*half as f32/2.;let south=n+(s-n)*(half+1)as f32/2.;
            let height=(14.-(row*2+half)as f32*2.)*scale;
            let v0=uv.2+(uv.3-uv.2)*half as f32/2.;
            let v1=uv.2+(uv.3-uv.2)*(half+1)as f32/2.;
            append_quad(&mut mesh.textured,[[w,height,north],[w,height,south],
                [e,height,south],[e,height,north]],[0.,1.,0.],
                [[uv.0,v0],[uv.0,v1],[uv.1,v1],[uv.1,v0]],TEXTURED_SHADE);
            append_quad(&mut mesh.textured,[[w,height,north],[e,height,north],
                [e,height+2.*scale,north],[w,height+2.*scale,north]],[0.,0.,1.],
                [[uv.0,v1],[uv.1,v1],[uv.1,v0],[uv.0,v0]],TEXTURED_SHADE);
            let neighbor=if map!="KarensRoom"&&row<2 {16.*scale}else{0.};
            if neighbor!=height && (column==0||column==3) {
                let plane=if column==0 {w}else{e};
                let low=neighbor.min(height);let high=neighbor.max(height);
                let east=(column==3)==(height>neighbor);
                let points=if east {[[plane,low,south],[plane,low,north],
                    [plane,high,north],[plane,high,south]]}
                    else {[[plane,low,north],[plane,low,south],
                    [plane,high,south],[plane,high,north]]};
                append_quad(&mut mesh.textured,points,[if east {1.}else{-1.},0.,0.],
                    [[uv.0,v1],[uv.1,v1],[uv.1,v0],[uv.0,v0]],TEXTURED_SHADE);
            }
        }
        let i=y*g.width+x;
        if mesh.authored_cells.len()==cells.len() {mesh.authored_cells[i]=Some("league:terrace-stair");}
    }}
    // Close the northern terrace's exposed front. Its two native front
    // courses supply the paint; leaving only a raised lid made the cap float.
    for absolute_x in 0..20 {
        let x=(absolute_x-origin[0])as usize;
        let boundary_y=(4-origin[1])as usize;
        if x>=g.width||boundary_y>=g.height {continue;}
        let (w,e,n,_)=g.bounds(x,boundary_y);
        for row in 2..4 {
            let source_y=(row-origin[1])as usize;
            if source_y>=g.height {continue;}
            let uv=g.uv(x,source_y);
            let top=(32.-(row-2)as f32*8.)*scale;let bottom=top-8.*scale;
            append_quad(&mut mesh.textured,[[w,bottom,n],[e,bottom,n],
                [e,top,n],[w,top,n]],[0.,0.,1.],
                [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
        }
    }
    if map!="KarensRoom" {
        for absolute_x in (0..8).chain(12..20) {
            let x=(absolute_x-origin[0])as usize;let y=(10-origin[1])as usize;
            if x>=g.width||y>=g.height {continue;}
            let (w,e,n,_)=g.bounds(x,y);let uv=g.uv(x,y);
            append_quad(&mut mesh.textured,[[w,0.,n],[e,0.,n],
                [e,16.*scale,n],[w,16.*scale,n]],[0.,0.,1.],
                [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
        }
    }
}

pub(super) fn support_correction(frame:&VisualWorldFrame,cell:UVec2,foot:Vec2,resolved:&[f32])->f32 {
    if !supports(frame.map_id.as_ref())
        ||frame.source_map_size_core_tiles.is_some_and(|s|s!=UVec2::new(10,18)) {return 0.;}
    let g=GridGeometry {width:frame.grid_size.x as usize,height:frame.grid_size.y as usize,
        tile_width:frame.tile_size.x,tile_height:frame.tile_size.y,origin_x:0.,origin_z:0.};
    let mut ordered=vec![None;g.width*g.height];
    for tile in &frame.tiles {
        let i=tile.row as usize*g.width+tile.column as usize;
        if i>=ordered.len()||ordered[i].is_some() {return 0.;}
        ordered[i]=Some(tile);
    }
    let Some(cells)=ordered.into_iter().collect::<Option<Vec<_>>>() else {return 0.;};
    if !coherent(frame.map_id.as_ref(),&cells,&g,frame.grid_origin.to_array()) {return 0.;}
    let start=frame.grid_origin.as_vec2()+cell.as_vec2();
    if !stair(start+Vec2::splat(0.5)) {return 0.;}
    let scale=frame.tile_size.y/8.;
    for y in 8..12 {for x in 8..12 {
        let i=(y-frame.grid_origin.y)as usize*g.width+(x-frame.grid_origin.x)as usize;
        let expected=(14.-(y-8)as f32*4.)*scale;
        if resolved.get(i).is_none_or(|h|(*h-expected).abs()>0.001) {return 0.;}
    }}
    let center=(14.-(start.y-8.)*4.)*scale;
    let size=frame.grid_size.as_vec2()*frame.tile_size;
    let y=(frame.center.y+size.y*0.5-foot.y)/frame.tile_size.y+frame.grid_origin.y as f32;
    stair_height(y.clamp(start.y,start.y+0.99999))*scale-center
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(map:&str)->(VisualWorldFrame,GridGeometry,TerrainMeshData) {
        let g=GridGeometry {width:20,height:16,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut tiles=Vec::new();let mut mesh=TerrainMeshData::default();
        for y in 0..16 {for x in 0..20 {
            let block=if y<4 {0x14}else if y<8 {if x/4==2 {0x22}else{0x15}}
                else if y<12 {if x/4==2 {1}else if map=="KarensRoom" {0x15}else{0x34}}
                else {0x38};
            let tile=drawing(block).map_or(0x52,|a|a[y%4][x%4]);
            tiles.push(VisualTile {column:x as u32,row:y as u32,texture:Default::default(),
                priority:false,animation_frames:None,source:VisualTileSource {
                    tileset_id:"elite_four_room".into(),metatile_id:block,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:tile}});
            append_top(&mut mesh.textured,g.bounds(x,y).into(),0.,g.uv(x,y));
        }}
        mesh.footing_heights=vec![0.;320];mesh.authored_cells=vec![None;320];
        let frame=VisualWorldFrame {active:true,map_id:map.into(),source_map_size_core_tiles:Some(UVec2::new(10,18)),
            boundary_warps:Default::default(),jump_ledges:Default::default(),terrain_revision:0,grid_origin:IVec2::ZERO,
            map_texture:Default::default(),center:Vec2::ZERO,viewport_size:Vec2::new(160.,128.),
            tile_size:Vec2::splat(8.),grid_size:UVec2::new(20,16),tiles,actors:Vec::new()};
        (frame,g,mesh)
    }
    #[test]
    fn kanto_league_terraces_build_steps_and_sample_each_actual_tread() {
        for map in ["WillsRoom","KogasRoom","BrunosRoom","KarensRoom"] {
            let (frame,g,mut mesh)=fixture(map);let refs=frame.tiles.iter().collect::<Vec<_>>();
            finish(&mut mesh,&frame,&refs,&g,None);
            assert_eq!(mesh.footing_heights[6*20+9],16.);
            assert_eq!(mesh.footing_heights[2*20+9],32.);
            assert_eq!(mesh.footing_heights[13*20+9],0.);
            assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("league:terrace-stair")).count(),16);
            for row in 8..12 {for half in 0..2 {
                let y=row as f32+(half as f32+0.5)/2.;
                let foot=Vec2::new(-80.+9.5*8.,64.-y*8.);
                let h=crate::footing::resolved_footing_height(&frame,foot,&mesh.footing_heights).unwrap();
                assert_eq!(h,14.-((row-8)*2+half)as f32*2.);
            }}
            let mut northern_front_area=0.;
            for tri in mesh.textured.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                assert!((b-a).cross(c-a).dot(n)>0.,"terrace and stair winding");
                if n==Vec3::Z && [a,b,c].iter().all(|p|p.z==32. && (16.0..=32.0).contains(&p.y)) {
                    northern_front_area+=(b-a).cross(c-a).length()*0.5;
                }
            }
            assert_eq!(northern_front_area,160.*16.,"complete northern terrace front must close its lid");
        }
    }
    #[test]
    fn kanto_league_terraces_reject_altered_stairs_and_custom_floor_without_partial_lift() {
        let (mut frame,g,mesh)=fixture("WillsRoom");
        for row in 8..12 {for column in 8..12 {
            let i=row*20+column;frame.tiles[i].source.tile_index^=1;
            let mut out=mesh.clone();finish(&mut out,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(out,mesh);frame.tiles[i].source.tile_index^=1;
        }}
        let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom stair","map":"WillsRoom","tileset":"elite_four_room","metatile":1,"origin":[0,0],"tiles":[[83]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        let mut out=mesh.clone();finish(&mut out,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,Some(&custom));
        assert_eq!(out,mesh);
        let foot=Vec2::new(-80.+9.5*8.,64.-11.25*8.);
        assert_eq!(crate::footing::resolved_footing_height(&frame,foot,&out.footing_heights),Some(0.));
        frame.map_id="BlackthornGym1F".into();let mut out=mesh.clone();
        finish(&mut out,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(out,mesh);
        frame.map_id="WillsRoom".into();frame.source_map_size_core_tiles=Some(UVec2::new(11,18));
        let mut out=mesh.clone();
        finish(&mut out,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);assert_eq!(out,mesh);
    }
}
