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
    let mut front_columns=Vec::new();
    if map!="KarensRoom" {
        let donor_row=7-origin[1];
        for absolute_x in (0..8).chain(12..20) {
            let x=absolute_x-origin[0];if x<0 || x as usize>=g.width {continue;}
            if donor_row<0 || donor_row as usize>=g.height {return;}
            let x=x as usize;let donor=donor_row as usize*g.width+x;let source=&cells[donor].source;
            if source.metatile_id!=0x15 || source.tile_index!=1 {return;}
            for absolute_y in 8..10 {
                let y=absolute_y-origin[1];if y<0 || y as usize>=g.height {return;}
                if mesh.authored_cells.get(y as usize*g.width+x).is_some_and(Option::is_some) {return;}
            }
            front_columns.push((x,donor));
        }
    }
    // Require the complete closed/open source block, including its paving donor.
    // A mixed or clipped gate cannot safely replace the original drawing.
    let mut gate_cells=Vec::new();
    for y in 4..8 {for x in 8..12 {
        let local_x=x-origin[0];let local_y=y-origin[1];
        if local_x<0 || local_y<0 || local_x as usize>=g.width || local_y as usize>=g.height {return;}
        gate_cells.push(local_y as usize*g.width+local_x as usize);
    }}
    let gate_block=cells[gate_cells[0]].source.metatile_id;
    if gate_cells.iter().any(|&i|cells[i].source.metatile_id!=gate_block) {return;}
    let closed_gate=gate_block==0x22;
    if closed_gate && gate_cells[..8].iter().any(|&i|
        mesh.authored_cells.get(i).is_some_and(Option::is_some)) {return;}
    let scale=g.tile_height/8.;
    for surface in [&mut mesh.textured,&mut mesh.solid,
        &mut mesh.animated_textured,&mut mesh.animated_solid] {
        let mut retained=Vec::with_capacity(surface.indices.len());
        let mut lifts:Vec<Option<f32>>=vec![None;surface.positions.len()];
        for tri in surface.indices.chunks_exact(3) {
            let center=tri.iter().map(|&i|Vec3::from_array(surface.positions[i as usize]))
                .sum::<Vec3>()/3.;
            let p=map_point(g,origin,center.to_array());
            // Only the authored stair drawing is replaced. The paving below
            // the terrace face stays horizontal and must meet its base.
            if stair(p) {continue;}
            if closed_gate && (8.0..12.0).contains(&p.x) && (4.0..6.0).contains(&p.y)
                && tri.iter().all(|&i|surface.positions[i as usize][1].abs()<0.00001
                    && surface.normals[i as usize][1]>0.9) {continue;}
            if map!="KarensRoom" && (8.0..10.0).contains(&p.y)
                && (0.0..20.0).contains(&p.x) && !(8.0..12.0).contains(&p.x)
                && tri.iter().all(|&i|surface.positions[i as usize][1].abs()<0.00001
                    && surface.normals[i as usize][1]>0.9) {continue;}
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
    if closed_gate {
        // Native two-row door art faces the landing; a one-pixel slab closes
        // its top, rear and jambs. Its base and head meet the adjoining wall.
        for column in 0..4 {
            let x=(8-origin[0])as usize+column;let y=(4-origin[1])as usize;
            let (w,e,n,_)=g.bounds(x,y);let front=n+2.*g.tile_height;
            let back=front-scale;let low=16.*scale;let high=32.*scale;
            let donor=g.uv(x,(7-origin[1])as usize);
            for row in 0..2 {
                let uv=g.uv(x,y+row);let top=high-row as f32*8.*scale;let bottom=top-8.*scale;
                append_quad(&mut mesh.textured,[[w,bottom,front],[e,bottom,front],
                    [e,top,front],[w,top,front]],[0.,0.,1.],
                    [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
                let north=n+row as f32*g.tile_height;
                append_top(&mut mesh.textured,[w,e,north,north+g.tile_height],low,donor);
                if mesh.authored_cells.len()==cells.len() {
                    mesh.authored_cells[(y+row)*g.width+x]=Some("league:native-north-door");
                }
            }
            let face=g.uv(x,y);let paint=[face.0+(face.1-face.0)*2.5/8.,face.2+(face.3-face.2)*2.5/8.];
            for (points,normal) in [
                ([[e,low,back],[w,low,back],[w,high,back],[e,high,back]],[0.,0.,-1.]),
                ([[w,high,back],[w,high,front],[e,high,front],[e,high,back]],[0.,1.,0.]),
                ([[w,low,back],[e,low,back],[e,low,front],[w,low,front]],[0.,-1.,0.]),
                ([[w,low,back],[w,low,front],[w,high,front],[w,high,back]],[-1.,0.,0.]),
                ([[e,low,front],[e,low,back],[e,high,back],[e,high,front]],[1.,0.,0.]),
            ] {append_quad(&mut mesh.textured,points,normal,[paint;4],TEXTURED_SHADE);}
        }
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
            let neighbor=if row<2 {16.*scale}else{0.};
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
    if map=="KarensRoom" {
        // The folded lower wall occupies only the front five pixels of its
        // sixteen-pixel source drawing. Cap the space behind it at the upper
        // landing datum, meeting the model's actual top shoulder. Keep its
        // native blue floor and leave the wall's front/top finish independent.
        let mut profile=SurfaceMeshData::default();
        crate::dungeon_models::extension_model(crate::dungeon_models::ExtensionKind::LeagueWall)
            .append(&mut profile,[0.,32.,0.,5.],0.,16.);
        let shoulder=profile.positions.iter().filter(|p|(p[1]-16.).abs()<0.00001)
            .map(|p|p[2]).fold(f32::INFINITY,f32::min);
        if shoulder.is_finite() && (0.0..5.0).contains(&shoulder) {
            for absolute_x in (0..8).chain(12..20) {
                let x=absolute_x-origin[0];let donor_y=7-origin[1];
                if x<0 || donor_y<0 || x as usize>=g.width || donor_y as usize>=g.height {continue;}
                let x=x as usize;let donor_y=donor_y as usize;
                let donor=&cells[donor_y*g.width+x].source;
                if donor.metatile_id!=0x15 || donor.tile_index!=1 {continue;}
                let uv=g.uv(x,donor_y);
                for absolute_y in 8..10 {
                    let y=absolute_y-origin[1];if y<0 || y as usize>=g.height {continue;}
                    let (w,e,n,s)=g.bounds(x,y as usize);
                    let end=s.min(g.origin_z+(80.-5.+shoulder-origin[1]as f32*8.)*scale);
                    if end<=n {continue;}
                    let v=uv.2+(uv.3-uv.2)*(end-n)/(s-n);
                    append_quad(&mut mesh.textured,[[w,16.*scale,n],[w,16.*scale,end],
                        [e,16.*scale,end],[e,16.*scale,n]],[0.,1.,0.],
                        [[uv.0,uv.2],[uv.0,v],[uv.1,v],[uv.1,uv.2]],TEXTURED_SHADE);
                }
            }
        }
    } else {
        for (x,donor) in front_columns {
            let first=(8-origin[1])as usize;
            let (w,e,n,_)=g.bounds(x,first);let s=n+2.*g.tile_height;let height=16.*scale;
            let floor=g.uv(donor%g.width,donor/g.width);let face=g.uv(x,first);
            let paint=[face.0+(face.1-face.0)*2.5/8.,face.2+(face.3-face.2)*2.5/8.];
            for row in 0..2 {
                let uv=g.uv(x,first+row);let top=height-row as f32*8.*scale;let bottom=top-8.*scale;
                append_quad(&mut mesh.textured,[[w,bottom,s],[e,bottom,s],[e,top,s],[w,top,s]],
                    [0.,0.,1.],[[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]],TEXTURED_SHADE);
                let north=n+row as f32*g.tile_height;let south=north+g.tile_height;
                append_top(&mut mesh.textured,[w,e,north,south],height,floor);
                if mesh.authored_cells.len()==cells.len() {
                    mesh.authored_cells[(first+row)*g.width+x]=Some("league:native-terrace-front");
                }
            }
            let absolute_x=x as i32+origin[0];
            for (points,normal) in [
                ([[e,0.,n],[w,0.,n],[w,height,n],[e,height,n]],[0.,0.,-1.]),
                ([[w,0.,n],[w,0.,s],[w,height,s],[w,height,n]],[-1.,0.,0.]),
                ([[e,0.,s],[e,0.,n],[e,height,n],[e,height,s]],[1.,0.,0.]),
                ([[w,0.,n],[e,0.,n],[e,0.,s],[w,0.,s]],[0.,-1.,0.]),
            ] {
                // The flight supplies the exposed side above each tread.
                // Do not stack a second blue face over its native gold edge.
                if (absolute_x==7 && normal[0]>0.) || (absolute_x==12 && normal[0]<0.) {continue;}
                append_quad(&mut mesh.textured,points,normal,[paint;4],TEXTURED_SHADE);
            }
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
        let frame=VisualWorldFrame { view_center: Vec2::ZERO, terrace_datum: None,active:true,map_id:map.into(),source_map_size_core_tiles:Some(UVec2::new(10,18)),
            boundary_warps:Default::default(),jump_ledges:Default::default(),terrain_revision:0,grid_origin:IVec2::ZERO,
            map_texture:Default::default(),center:Vec2::ZERO,viewport_size:Vec2::new(160.,128.),
            tile_size:Vec2::splat(8.),grid_size:UVec2::new(20,16),tiles,actors:Vec::new()};
        (frame,g,mesh)
    }
    #[test]
    fn kanto_league_north_door_is_closed_native_volume_and_disappears_when_open() {
        for map in ["WillsRoom","KogasRoom","BrunosRoom","KarensRoom"] {
            for open in [false,true] {
                let (mut frame,g,mut mesh)=fixture(map);
                if open {for y in 4..8 {for x in 8..12 {
                    let s=&mut frame.tiles[y*20+x].source;s.metatile_id=0x16;
                    s.tile_index=drawing(0x16).unwrap()[y%4][x%4];
                }}}
                finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
                assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("league:native-north-door")).count(),
                    if open {0}else{8});
                let mut front_area=0.;let mut volume=0.;let mut backing=0.;
                for tri in mesh.textured.indices.chunks_exact(3) {
                    let points=tri;
                    let [a,b,c]=[points[0],points[1],points[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                    let n=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                    let center=(a+b+c)/3.;let area=(b-a).cross(c-a).length()*0.5;
                    if (64.0..96.0).contains(&center.x) && [a,b,c].iter().all(|p|p.z==48.) && n==Vec3::Z {
                        front_area+=area;
                        let uv=tri.iter().map(|&i|Vec2::from_array(mesh.textured.uvs[i as usize])).sum::<Vec2>()/3.;
                        let source=&frame.tiles[(uv.y*16.)as usize*20+(uv.x*20.)as usize].source;
                        assert_eq!(source.metatile_id,0x22);assert!(source.subtile_row<2);
                    }
                    if (64.0..96.0).contains(&center.x) && (32.0..48.0).contains(&center.z)
                        && n==Vec3::Y && [a,b,c].iter().all(|p|p.y==16.) {backing+=area;}
                    if !open && (64.0..=96.0).contains(&center.x) && (47.0..=48.0).contains(&center.z)
                        && [a,b,c].iter().all(|p|(16.0..=32.0).contains(&p.y)) {
                        volume+=a.dot(b.cross(c))/6.;
                    }
                }
                assert_eq!(front_area,if open {0.}else{512.});
                assert!((backing-512.).abs()<0.001,"{map}: blue backing survives the fold/open state");
                if !open {assert!((volume-512.).abs()<0.01,"closed door volume {volume}");}
                assert_eq!(mesh.footing_heights[6*20+9],16.);
            }
            let (frame,g,mut baseline)=fixture(map);baseline.authored_cells[4*20+8]=Some("reserved");
            let mut mesh=baseline.clone();finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(mesh,baseline,"an existing gate owner protects the complete assembly");
        }
    }
    #[test]
    fn kanto_league_terrace_front_keeps_wall_art_vertical_and_blue_landing_on_top() {
        for map in ["WillsRoom","KogasRoom","BrunosRoom"] {
            let (frame,g,mut mesh)=fixture(map);
            finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
            let mut front=0.;let mut cap=0.;
            for tri in mesh.textured.indices.chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let center=points.iter().copied().sum::<Vec3>()/3.;
                let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
                let area=(points[1]-points[0]).cross(points[2]-points[0]);
                assert!(area.dot(normal)>0.);
                if (64.0..96.0).contains(&center.x) {continue;}
                let is_front=normal==Vec3::Z && points.iter().all(|p|p.z==80. && (0.0..=16.0).contains(&p.y));
                let is_cap=normal==Vec3::Y && (64.0..80.0).contains(&center.z) && points.iter().all(|p|p.y==16.);
                if !is_front && !is_cap {continue;}
                let uv=tri.iter().map(|&i|Vec2::from_array(mesh.textured.uvs[i as usize])).sum::<Vec2>()/3.;
                let s=&frame.tiles[(uv.y*g.height as f32)as usize*g.width+(uv.x*g.width as f32)as usize].source;
                if is_front {
                    front+=area.length()*0.5;assert_eq!((s.metatile_id,s.tile_index),(0x34,0x39));
                    assert!(s.subtile_row<2,"each original blue wall row folds exactly once");
                }else{
                    cap+=area.length()*0.5;assert_eq!((s.metatile_id,s.tile_index),(0x15,1));
                }
            }
            assert_eq!((front,cap),(128.*16.,128.*16.));
            assert_eq!(mesh.authored_cells.iter().filter(|&&v|v==Some("league:native-terrace-front")).count(),32);
            for y in 8..10 {for x in (0..8).chain(12..20) {assert_eq!(mesh.footing_heights[y*20+x],16.);}}
            let (frame,g,mut owned)=fixture(map);owned.authored_cells[8*20]=Some("custom:front");
            let before=owned.clone();finish(&mut owned,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(owned,before,"existing source ownership protects the complete assembly");
            let mut changed=frame.clone();changed.tiles[7*20].source.tile_index^=1;
            let (_,_,mut out)=fixture(map);let before=out.clone();
            finish(&mut out,&changed,&changed.tiles.iter().collect::<Vec<_>>(),&g,None);
            assert_eq!(out,before,"a changed floor donor never invents a cap finish");
            let custom:crate::live_profiles::Document=serde_json::from_value(serde_json::json!({"objects":[{
                "name":"Custom terrace","map":map,"tileset":"elite_four_room","metatile":52,"origin":[0,0],
                "tiles":[[57]],"ground":82,"top_pixels":0,"depth_pixels":8}]})).unwrap();
            let (_,_,mut out)=fixture(map);let before=out.clone();
            finish(&mut out,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,Some(&custom));
            assert_eq!(out,before,"live terrace overrides remain authoritative");
        }
    }
    #[test]
    fn kanto_karen_landing_cap_meets_wall_shoulder_and_closes_stair_sides() {
        let (frame,g,mut mesh)=fixture("KarensRoom");
        finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
        let mut cap_area=0.;let mut cap_end=0f32;let mut stair_side_area=0.;
        for tri in mesh.textured.indices.chunks_exact(3) {
            let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
            let center=points.iter().copied().sum::<Vec3>()/3.;
            let normal=Vec3::from_array(mesh.textured.normals[tri[0]as usize]);
            let area=(points[1]-points[0]).cross(points[2]-points[0]);
            assert!(area.dot(normal)>0.);
            if (64.0..80.0).contains(&center.z) && !(64.0..96.0).contains(&center.x)
                && points.iter().all(|p|p.y==16.) {
                cap_area+=area.length()*0.5;
                cap_end=points.iter().map(|p|p.z).fold(cap_end,f32::max);
                let uv=tri.iter().map(|&i|Vec2::from_array(mesh.textured.uvs[i as usize])).sum::<Vec2>()/3.;
                let source=&frame.tiles[(uv.y*g.height as f32)as usize*g.width+(uv.x*g.width as f32)as usize].source;
                assert_eq!((source.metatile_id,source.tile_index),(0x15,1));
            }
            if (64.0..80.0).contains(&center.z) && normal.x.abs()==1.
                && points.iter().any(|p|p.y==16.) {
                stair_side_area+=area.length()*0.5;
            }
        }
        assert!((75.3..75.4).contains(&cap_end),"cap meets the existing model's top shoulder");
        assert!((128.*11.3..128.*11.4).contains(&cap_area));
        assert_eq!(stair_side_area,2.*(4.*2.+4.*4.+4.*6.+4.*8.));
        for x in (0..8).chain(12..20) {
            assert_eq!(mesh.footing_heights[8*20+x],0.,"blocked wall cells do not acquire walk support");
        }
        let (mut altered,g,baseline)=fixture("KarensRoom");
        altered.tiles[8*20].source.tile_index^=1;
        let mut out=baseline.clone();
        finish(&mut out,&altered,&altered.tiles.iter().collect::<Vec<_>>(),&g,None);
        assert_eq!(out,baseline,"edited lower-wall art keeps the entire original assembly");
        let custom:crate::live_profiles::Document=serde_json::from_str(r#"{"objects":[{"name":"Custom lower wall","map":"KarensRoom","tileset":"elite_four_room","metatile":21,"origin":[0,0],"tiles":[[16]],"ground":1,"top_pixels":0,"depth_pixels":8}]}"#).unwrap();
        let (frame,g,baseline)=fixture("KarensRoom");let mut out=baseline.clone();
        finish(&mut out,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,Some(&custom));
        assert_eq!(out,baseline,"custom walls retain authority over their landing cap");
    }
    #[test]
    fn kanto_league_authored_wall_backing_closes_the_upper_landing() {
        for map in ["WillsRoom","KogasRoom","BrunosRoom","KarensRoom"] {
            let (frame,g,_)=fixture(map);let cells=frame.tiles.iter().collect::<Vec<_>>();
            let placements=super::super::modeled_dungeons::resolve(map,&cells,&g,None);
            let walls=placements.iter().filter(|p|p.kind_label()=="special:league-wall").collect::<Vec<_>>();
            assert_eq!(walls.len(),if map=="KarensRoom" {8}else{4});
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;cells.len()],
                authored_cells:vec![None;cells.len()],..Default::default()};
            for p in walls {super::super::modeled_dungeons::append(&mut mesh,&g,p,&cells,[0,0]);}
            finish(&mut mesh,&frame,&cells,&g,None);
            let mut area=0.;
            for tri in mesh.textured.indices.chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let center=points.iter().copied().sum::<Vec3>()/3.;
                if !(32.0..48.0).contains(&center.z) || (64.0..96.0).contains(&center.x)
                    || !points.iter().all(|p|p.y==16.)
                    || !tri.iter().all(|&i|mesh.textured.normals[i as usize][1]>0.9) {continue;}
                area+=(points[1]-points[0]).cross(points[2]-points[0]).length()*0.5;
                let uv=tri.iter().map(|&i|Vec2::from_array(mesh.textured.uvs[i as usize])).sum::<Vec2>()/3.;
                let sample=(uv.y*g.height as f32)as usize*g.width+(uv.x*g.width as f32)as usize;
                assert_eq!(cells[sample].source.tile_index,1,
                    "the wall's backing uses actual native blue paving, not black void");
            }
            assert_eq!(area,128.*16.,"{map}: every folded upper-wall source cell keeps its landing backing");
        }
    }
    #[test]
    fn kanto_league_terraces_preserve_paving_below_the_raised_front() {
        for map in ["WillsRoom","KogasRoom","BrunosRoom"] {
            let (frame,g,mut mesh)=fixture(map);
            finish(&mut mesh,&frame,&frame.tiles.iter().collect::<Vec<_>>(),&g,None);
            let mut floor_area=0.;
            for tri in mesh.textured.indices.chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                let center=points.iter().copied().sum::<Vec3>()/3.;
                if (80.0..88.0).contains(&center.z) && !(64.0..96.0).contains(&center.x)
                    && points.iter().all(|p|p.y==0.) {
                    assert!(tri.iter().all(|&i|mesh.textured.normals[i as usize]==[0.,1.,0.]));
                    floor_area+=(points[1]-points[0]).cross(points[2]-points[0]).length()*0.5;
                    for &i in tri {
                        let p=mesh.textured.positions[i as usize];
                        let uv=mesh.textured.uvs[i as usize];
                        assert!((uv[0]-p[0]/160.).abs()<0.00001);
                        assert!((uv[1]-p[2]/128.).abs()<0.00001,
                            "the horizontal paving keeps its original native phase");
                    }
                }
            }
            assert_eq!(floor_area,128.*8.,"{map}: paving must cover the whole row outside the flight");
            for x in (0..8).chain(12..20) {
                assert_eq!(mesh.footing_heights[10*20+x],0.);
                assert_eq!(mesh.authored_cells[10*20+x],None,
                    "preserved paving is not a newly claimed object");
            }
        }
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
