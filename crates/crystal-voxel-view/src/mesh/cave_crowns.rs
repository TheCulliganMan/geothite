//! Plain cave dust also paints the interior of authored raised platforms.
//! Resolve a bounded crown from its explicit cap course, not collision flags.
use super::*;
use bevy::math::{Vec2, Vec4};

pub(super) fn enabled(map:&str) -> bool {
    matches!(map,"VictoryRoad"|"SilverCaveRoom2")
}

pub(super) fn resolve(frame: &VisualWorldFrame, cells: &[&VisualTile],
    shapes: &mut [CellShape], g: &GridGeometry) {
    // These source-verified maps reuse plain dust inside raised crowns.
    // Keep other cavern datums until their source topology is verified.
    if !enabled(frame.map_id.as_ref()) {return;}
    let within = |i:usize| {
        let x=frame.grid_origin.x+i as i32%g.width as i32;
        let z=frame.grid_origin.y+i as i32/g.width as i32;
        frame.source_map_size_core_tiles.is_none_or(|size|
            x>=0 && z>=0 && x<(size.x*2) as i32 && z<(size.y*2) as i32)
    };
    let candidate = |i:usize, shapes:&[CellShape]| {
        let s=&cells[i].source;
        within(i) && matches!(s.tileset_id.as_ref(),"cave"|"dark_cave") && s.tile_index==0x16
            && (matches!(shapes[i],CellShape::Flat)
                || matches!(shapes[i],CellShape::RaisedTop {height,solid:SolidKind::Bank}
                    if height==crate::cave::rock_height(s)))
    };
    let mut crown_blocks=std::collections::BTreeSet::new();
    let mut visited=vec![false;cells.len()];
    for start in 0..cells.len() {
        if visited[start] || !candidate(start,shapes) {continue;}
        visited[start]=true;
        let mut region=vec![start]; let mut cursor=0;
        let mut cap=false; let mut open=false;
        let tileset=&cells[start].source.tileset_id;
        while cursor<region.len() {
            let i=region[cursor]; cursor+=1;
            let s=&cells[i].source;
            // These complete north cap courses explicitly sit above their
            // two drawn south/west/east wall courses. Ceiling paint is not a
            // seed: an adjoining ceiling must never lift the cavern floor.
            cap |= matches!(s.metatile_id,0x0c|0x0d|0x0e|0x36|0x12|0x30)
                && matches!(shapes[i],CellShape::RaisedTop {height,..}
                    if height==crate::cave::rock_height(s));
            let x=i%g.width; let z=i/g.width;
            for (dx,dz) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let nx=x as isize+dx;let nz=z as isize+dz;
                if nx<0 || nz<0 || nx>=g.width as isize || nz>=g.height as isize {open=true;continue;}
                let j=nz as usize*g.width+nx as usize;
                if !within(j) {open=true;continue;}
                if !visited[j] && cells[j].source.tileset_id==*tileset && candidate(j,shapes) {
                    visited[j]=true;region.push(j);
                }
            }
        }
        if cap && !open {
            for &i in &region {
                let source=&cells[i].source;
                crown_blocks.insert((i as i32%g.width as i32-source.subtile_column as i32,
                    i as i32/g.width as i32-source.subtile_row as i32,source.tileset_id.clone()));
                shapes[i]=CellShape::RaisedTop {
                    height:crate::cave::CAVE_ROCK_HEIGHT,solid:SolidKind::Bank};
            }
        }
    }
    // Lower the matching authored rim with its crown, never the surrounding
    // ceiling geology. A source block is owned only by a proven closed crown.
    for (i,tile) in cells.iter().enumerate() {
        let s=&tile.source;
        let block=(i as i32%g.width as i32-s.subtile_column as i32,
            i as i32/g.width as i32-s.subtile_row as i32,s.tileset_id.clone());
        if !crown_blocks.contains(&block) {continue;}
        match &mut shapes[i] {
            CellShape::RaisedTop {height,..} | CellShape::LedgeBand {height,..}
                if *height==crate::cave::rock_height(s) => *height=crate::cave::CAVE_ROCK_HEIGHT,
            CellShape::LedgeBand {height,top_tile_index,..} if s.tile_index==0x24
                && matches!(s.metatile_id,0x04|0x05|0x06) => {
                    *height=crate::cave::CAVE_ROCK_HEIGHT;*top_tile_index=0x16;
                },
            _=>{},
        }
    }
    resolve_descents(cells,shapes,g);
    #[cfg(not(target_arch="wasm32"))]
    trace_stair_neighbors(frame,cells,shapes,g);
}

/// Inspect source evidence before changing a visible stair join. Report once
/// per process and bound both stair count and each neighborhood; normal play
/// never writes diagnostics unless the existing surface trace is requested.
#[cfg(not(target_arch="wasm32"))]
fn trace_stair_neighbors(frame:&VisualWorldFrame,cells:&[&VisualTile],
    shapes:&[CellShape],g:&GridGeometry) {
    if std::env::var_os("CRYSTAL_CAVE_SURFACE_TRACE").is_none() {return;}
    static REPORTED:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
    if REPORTED.swap(true,std::sync::atomic::Ordering::Relaxed) {return;}
    for (i,_) in cells.iter().enumerate().filter(|(_,tile)| {
        let s=&tile.source;
        matches!(s.metatile_id,0x12|0x30) && s.subtile_row==2
            && s.subtile_column==if s.metatile_id==0x12 {2} else {0}
            && s.tile_index==0x36
    }).take(8) {
        let column=i%g.width;let row=i/g.width;
        eprintln!("cave_stair map={} cell={},{} admitted={}",frame.map_id,
            frame.grid_origin.x+column as i32,frame.grid_origin.y+row as i32,
            matches!(shapes[i],CellShape::RampNorth {..}));
        for z in row.saturating_sub(2)..(row+4).min(g.height) {
            for x in column.saturating_sub(2)..(column+4).min(g.width) {
                let j=z*g.width+x;let s=&cells[j].source;
                eprintln!("cave_stair neighbor={},{} tileset={} block={:02x} glyph={:02x} subtile={},{} raw={:?} resolved={:?}",
                    frame.grid_origin.x+x as i32,frame.grid_origin.y+z as i32,
                    s.tileset_id,s.metatile_id,s.tile_index,s.subtile_column,s.subtile_row,
                    crate::cave::cave_shape(s),shapes[j]);
            }
        }
    }
}

fn resolve_descents(cells:&[&VisualTile],shapes:&mut [CellShape],g:&GridGeometry) {
    for row in 1..g.height.saturating_sub(2) {
        for column in 0..g.width.saturating_sub(1) {
            let s=&cells[row*g.width+column].source;
            if !matches!(s.metatile_id,0x12|0x30) || s.subtile_row!=2
                || s.subtile_column!=if s.metatile_id==0x12 {2} else {0} {continue;}
            // A complete paired stair picture and both landings are required.
            // Ordinary ladders/warps and reused isolated strips never join.
            let complete=(0..2).all(|dz|(0..2).all(|dx| {
                let q=&cells[(row+dz)*g.width+column+dx].source;
                q.tileset_id==s.tileset_id && q.metatile_id==s.metatile_id
                    && q.subtile_row==2+dz as u8
                    && q.subtile_column==s.subtile_column+dx as u8
                    && q.tile_index==0x36+dx as u16
            }));
            let height=crate::cave::CAVE_ROCK_HEIGHT;
            let upper=(row-1..row).all(|z|(column..column+2).all(|x|
                matches!(shapes[z*g.width+x],CellShape::RaisedTop {height:h,..} if h==height)
                && cells[z*g.width+x].source.tile_index==0x16));
            let low=shapes[(row+2)*g.width+column].surface_height(SOURCE_TILE_HEIGHT);
            let lower=low<=crate::cave::CAVE_SHELF_HEIGHT && (column..column+2).all(|x| {
                let i=(row+2)*g.width+x;
                matches!(cells[i].source.tile_index,0x01|0x14|0x16)
                    && cells[i].source.tileset_id==s.tileset_id
                    && matches!(shapes[i],CellShape::Flat|CellShape::PlaneAt {..}
                        |CellShape::RaisedTop {solid:SolidKind::Bank,..})
                    && shapes[i].surface_height(SOURCE_TILE_HEIGHT)==low
            });
            if !complete || !upper || !lower {continue;}
            // Preserve the crown approach. Only the native two-by-two stair
            // picture descends; its upper and lower edges meet both landings.
            for dz in 0..2 {for dx in 0..2 {
                shapes[(row+dz)*g.width+column+dx]=CellShape::RampNorth {
                    north_height:low+(height-low)*(2-dz) as f32/2.,
                    south_height:low+(height-low)*(1-dz) as f32/2.,
                };
            }}
        }
    }
}

/// Three shallow stone treads per source row, joined across both stair columns.
pub(super) fn append_steps(mesh:&mut SurfaceMeshData,g:&GridGeometry,
    [x0,x1,z0,z1]:[f32;4],north:f32,south:f32) {
    for step in 0..3 {
        let za=z0+(z1-z0)*step as f32/3.;
        let zb=z0+(z1-z0)*(step+1) as f32/3.;
        let top=north-(north-south)*step as f32/3.;
        let bottom=north-(north-south)*(step+1) as f32/3.;
        let bevel=(zb-za)*0.12;
        let lip=(top-bottom)*0.12;
        for (positions,normal,base) in [
            ([[x0,top,za],[x0,top,zb-bevel],[x1,top,zb-bevel],[x1,top,za]],
                [0.,1.,0.],[0.40,0.40,0.38,1.]),
            ([[x0,top,zb-bevel],[x0,top-lip,zb],[x1,top-lip,zb],[x1,top,zb-bevel]],
                Vec3::new(0.,bevel,lip).normalize().to_array(),[0.38,0.38,0.36,1.]),
            ([[x0,bottom,zb],[x1,bottom,zb],[x1,top-lip,zb],[x0,top-lip,zb]],
                [0.,0.,1.],[0.34,0.345,0.31,1.]),
        ] {
            let uv=positions.map(|p|cave_cliff::floor_uv(Vec3::from_array(p),g,[0,0],false));
            append_quad(mesh,positions,normal,uv,base);
        }
    }
}

/// Replace legacy borrowed atlas caps only inside proven stair cells. This is
/// the final surface pass because earlier bank joins can restore those caps.
#[derive(Clone, Copy)]
struct StairVertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    color: Vec4,
    cutaway: bool,
}

impl StairVertex {
    fn interpolate(self, other:Self,t:f32)->Self {
        Self {position:self.position.lerp(other.position,t),
            normal:self.normal.lerp(other.normal,t).normalize_or_zero(),
            uv:self.uv.lerp(other.uv,t),color:self.color.lerp(other.color,t),
            cutaway:self.cutaway || other.cutaway}
    }
}

fn stair_half_plane(polygon:&[StairVertex],axis:usize,edge:f32,sign:f32)->Vec<StairVertex> {
    let mut result=Vec::new();
    let Some(&last)=polygon.last() else {return result;};
    let mut previous=last;
    let mut before=(previous.position[axis]-edge)*sign;
    for &current in polygon {
        let distance=(current.position[axis]-edge)*sign;
        if (before>=0.) != (distance>=0.) {
            result.push(previous.interpolate(current,before/(before-distance)));
        }
        if distance>=0. {result.push(current);}
        previous=current;before=distance;
    }
    result
}

/// Subtract the opening from each face, retaining its actual outside polygon.
/// A triangle's center does not establish whether it overlaps a stair socket.
fn trim_stair_openings(surface:&mut SurfaceMeshData,openings:&[[f32;4]]) {
    let indices=std::mem::take(&mut surface.indices);
    for triangle in indices.chunks_exact(3) {
        let points=[0,1,2].map(|i|surface.positions[triangle[i] as usize]);
        if openings.iter().all(|&[w,e,n,s]| {
            [(0,w,1.),(0,e,-1.),(2,n,1.),(2,s,-1.)].into_iter().any(|(axis,edge,sign)|
                points.iter().all(|p|(p[axis]-edge)*sign<=0.)
                    && points.iter().any(|p|(p[axis]-edge)*sign<0.))
        }) {
            surface.indices.extend_from_slice(triangle);continue;
        }
        let mut polygons=vec![triangle.iter().map(|&i| {
            let i=i as usize;
            StairVertex {position:Vec3::from_array(surface.positions[i]),
                normal:Vec3::from_array(surface.normals[i]),uv:Vec2::from_array(surface.uvs[i]),
                color:Vec4::from_array(surface.colors[i]),
                cutaway:surface.cutaway_ranges.iter().any(|r|r.contains(&i))}
        }).collect::<Vec<_>>()];
        for &[w,e,n,s] in openings {
            let mut retained=Vec::new();
            for polygon in polygons {
                let mut inside=polygon;
                for (axis,edge,sign) in [(0,w,1.),(0,e,-1.),(2,n,1.),(2,s,-1.)] {
                    // Coplanar boundary faces belong to the replacement cheek.
                    if !inside.iter().all(|p|(p.position[axis]-edge).abs()<0.00001) {
                        let outside=stair_half_plane(&inside,axis,edge,-sign);
                        if outside.len()>=3 {retained.push(outside);}
                    }
                    inside=stair_half_plane(&inside,axis,edge,sign);
                    if inside.len()<3 {break;}
                }
            }
            polygons=retained;
        }
        for polygon in polygons {
            for i in 1..polygon.len()-1 {
                let vertices=[polygon[0],polygon[i],polygon[i+1]];
                if (vertices[1].position-vertices[0].position)
                    .cross(vertices[2].position-vertices[0].position).length_squared()<0.00000001 {
                    continue;
                }
                let base=surface.positions.len();
                for vertex in vertices {
                    surface.positions.push(vertex.position.to_array());
                    surface.normals.push(vertex.normal.to_array());
                    surface.uvs.push(vertex.uv.to_array());surface.colors.push(vertex.color.to_array());
                }
                surface.indices.extend([base as u32,base as u32+1,base as u32+2]);
                if vertices.iter().any(|v|v.cutaway) {surface.cutaway_ranges.push(base..base+3);}
            }
        }
    }
}

pub(super) fn finish_stairs(mesh:&mut TerrainMeshData,cells:&[&VisualTile],
    shapes:&[CellShape],g:&GridGeometry) {
    let stair=|i:usize|matches!(shapes[i],CellShape::RampNorth {..})
        && matches!(cells[i].source.metatile_id,0x12|0x30)
        && matches!(cells[i].source.tile_index,0x36|0x37);
    let openings=shapes.iter().enumerate().filter(|&(i,_)|
        stair(i) && cells[i].source.tile_index==0x36).map(|(i,_)| {
            let (w,e,n,s)=g.bounds(i%g.width,i/g.width);[w,e+g.tile_width,n,s]
        }).collect::<Vec<_>>();
    if openings.is_empty() {return;}
    for surface in [&mut mesh.textured,&mut mesh.solid] {trim_stair_openings(surface,&openings);}
    for (i,shape) in shapes.iter().enumerate() {
        if !stair(i) || cells[i].source.tile_index!=0x36 {continue;}
        let CellShape::RampNorth {north_height,south_height}=*shape else {continue;};
        let column=i%g.width;let row=i/g.width;
        let (w,e,n,s)=g.bounds(column,row);
        let scale=g.tile_height/SOURCE_TILE_HEIGHT;
        let north=north_height*scale;let south=south_height*scale;
        let east=e+g.tile_width;
        // Each tread spans both source columns, with no internal divider.
        append_steps(&mut mesh.solid,g,[w,east,n,s],north,south);
        for (x,neighbor,normal) in [
            (w,column.checked_sub(1).map(|x|row*g.width+x),[-1.,0.,0.]),
            (east,(column+2<g.width).then_some(i+2),[1.,0.,0.]),
        ] {
            let Some(neighbor)=neighbor else {continue;};
            if stair(neighbor) {continue;}
            let base=shapes[neighbor].surface_height(g.tile_height);
            for step in 0..3 {
                let za=n+(s-n)*step as f32/3.;let zb=n+(s-n)*(step+1) as f32/3.;
                let top=north-(north-south)*step as f32/3.;
                let bevel=(zb-za)*0.12;let lip=(north-south)/3.*0.12;
                let outward=if base>top {[ -normal[0],0.,0. ]} else {normal};
                for mut points in [
                    [[x,base,za],[x,base,zb-bevel],[x,top,zb-bevel],[x,top,za]],
                    [[x,base,zb-bevel],[x,base,zb],[x,top-lip,zb],[x,top,zb-bevel]],
                ] {
                    let p=points.map(Vec3::from_array);
                    let first=(p[1]-p[0]).cross(p[2]-p[0]);
                    let second=(p[2]-p[0]).cross(p[3]-p[0]);
                    if first.length_squared()+second.length_squared()<0.000001 {continue;}
                    if (first+second).dot(Vec3::from_array(outward))<0. {
                        points=[points[0],points[3],points[2],points[1]];
                    }
                    append_quad(&mut mesh.solid,points,outward,
                        points.map(|p|cave_cliff::rock_uv(Vec3::from_array(p),g,[0,0],false)),
                        [0.34,0.345,0.31,1.]);
                }
            }
        }
    }
}

pub(super) fn finish_sides(mesh:&mut TerrainMeshData,shapes:&[CellShape],
    g:&GridGeometry,origin:[i32;2]) {
    for i in 0..mesh.solid.positions.len() {
        let p=Vec3::from_array(mesh.solid.positions[i]);
        if mesh.solid.uvs[i][0]>=100000. && (200000. ..300000.).contains(&mesh.solid.uvs[i][1]) {
            mesh.solid.uvs[i]=cave_cliff::rock_uv(p,g,origin,false);
            continue;
        }
        if mesh.solid.uvs[i][0]>=100000. && (400000. ..500000.).contains(&mesh.solid.uvs[i][1]) {
            mesh.solid.uvs[i]=cave_cliff::floor_uv(p,g,origin,false);
            continue;
        }
        if mesh.solid.uvs[i]!=[0.;2] || mesh.solid.normals[i][0].abs()<0.99 {continue;}
        let x=((p.x-g.origin_x)/g.tile_width).round() as isize;
        let z=((p.z-g.origin_z)/g.tile_height).floor() as isize;
        if [x-1,x].into_iter().any(|x|[z-1,z].into_iter().any(|z|
            x>=0 && z>=0 && x<g.width as isize && z<g.height as isize
                && matches!(shapes[z as usize*g.width+x as usize],CellShape::RampNorth {..}))) {
            mesh.solid.uvs[i]=cave_cliff::rock_uv(p,g,origin,false);
            mesh.solid.colors[i]=cave_cliff::pigment([0.34,0.345,0.31,1.],p,g,origin,false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn exported_cave_frame(map:&str,block_width:usize,block_height:usize)->VisualWorldFrame {
        let root=std::path::PathBuf::from(std::env::var_os("CRYSTAL_CAVE_EXPORT_ROOT")
            .expect("set CRYSTAL_CAVE_EXPORT_ROOT"));
        let blocks=std::fs::read(root.join(format!("vendor/pokecrystal/maps/{map}.blk"))).unwrap();
        let art=std::fs::read(root.join("apps/web/assets/data/tilesets/cave_metatiles.bin")).unwrap();
        assert_eq!(blocks.len(),block_width*block_height);
        let width=block_width*4;let height=block_height*4;
        let mut sources=Vec::new();
        for z in 0..height {for x in 0..width {
            let block=blocks[(z/4)*block_width+x/4] as u16;
            sources.push(source(block,(x%4) as u8,(z%4) as u8,
                art[block as usize*16+(z%4)*4+x%4] as u16));
        }}
        let mut frame=super::super::tests::frame(width as u32,height as u32,sources);
        frame.map_id=map.into();
        frame.source_map_size_core_tiles=Some(bevy::math::UVec2::new((block_width*2) as u32,(block_height*2) as u32));
        frame
    }

    #[test]
    #[ignore = "requires CRYSTAL_CAVE_EXPORT_ROOT pointing to a compatible external export"]
    fn victory_source_keeps_the_full_tile_approach_at_the_reviewed_stair() {
        let frame=exported_cave_frame("VictoryRoad",10,36);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:40,height:144,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut shapes=cells.iter().map(|tile|shape_for_source_on_map(&frame.map_id,&tile.source)).collect::<Vec<_>>();
        resolve(&frame,&cells,&mut shapes,&g);
        // The reviewed player at core (5,35) has core (5,36), two source
        // rows, between them and the authored stair at source (10,74).
        for z in 72..74 {for x in 10..12 {
            let i=z*g.width+x;
            assert_eq!(cells[i].source.metatile_id,0x12);
            assert_eq!(cells[i].source.tile_index,0x16);
            assert_eq!(shapes[i],CellShape::RaisedTop {height:16.,solid:SolidKind::Bank});
        }}
        assert_eq!(shapes[74*g.width+10],CellShape::RampNorth {north_height:16.,south_height:11.});
        assert_eq!(shapes[75*g.width+10],CellShape::RampNorth {north_height:11.,south_height:6.});
        assert_eq!(shapes[76*g.width+10].surface_height(8.),6.);
    }
    /// External source stays external; this authenticates real art instead of
    /// manufacturing a stair fixture or embedding exported game data in Git.
    #[test]
    #[ignore = "requires CRYSTAL_CAVE_EXPORT_ROOT pointing to a compatible external export"]
    fn silver_source_stair_neighborhoods_preserve_landings_and_approaches() {
        let frame=exported_cave_frame("SilverCaveRoom2",15,18);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:60,height:72,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let raw=cells.iter().map(|tile|shape_for_source_on_map(&frame.map_id,&tile.source)).collect::<Vec<_>>();
        let mut shapes=raw.clone();resolve(&frame,&cells,&mut shapes,&g);
        let mut anchors=0;let mut admitted=0;
        for (i,tile) in cells.iter().enumerate() {
            let s=&tile.source;
            if s.metatile_id!=0x12 || s.subtile_row!=2 || s.subtile_column!=2 {continue;}
            anchors+=1;let x=i%g.width;let z=i/g.width;
            eprintln!("silver_source stair={},{} raw={:?} resolved={:?} left={:?} right={:?}",
                x,z,raw[i],shapes[i],shapes[i-1],shapes[i+2]);
            let CellShape::RampNorth {north_height,south_height}=shapes[i] else {continue;};
            admitted+=1;
            assert_eq!(north_height,shapes[i-g.width].surface_height(8.));
            assert_eq!(shapes[i],shapes[i+1]);
            let CellShape::RampNorth {north_height:next,south_height:bottom}=shapes[i+g.width] else {panic!("missing lower stair row");};
            assert_eq!(south_height,next);
            assert_eq!(bottom,shapes[i+2*g.width].surface_height(8.));
            // The full approach row must remain a level crown, not a trench.
            assert!(matches!(shapes[i-g.width],CellShape::RaisedTop {height:16.,..}));
        }
        assert_eq!(anchors,6,"authenticate the complete source stair inventory");
        assert_eq!(admitted,4,"three dust-floor stairs and one shelf landing descend; two raised continuations stay unchanged");
        eprintln!("silver_source authored_stairs={anchors} admitted_descents={admitted}");
    }
    #[test]
    fn cave_stair_socket_trims_crossing_faces_without_leaving_shards_or_holes() {
        let mut surface=SurfaceMeshData::default();
        append_solid_quad(&mut surface,[[0.,16.,0.],[0.,16.,24.],[24.,16.,24.],[24.,16.,0.]],
            [0.,1.,0.],[0.4,0.3,0.2,1.]);
        surface.cutaway_ranges.push(0..4);
        trim_stair_openings(&mut surface,&[[8.,16.,8.,16.]]);
        let mut area=0.;
        for triangle in surface.indices.chunks_exact(3) {
            let p=[0,1,2].map(|i|Vec3::from_array(surface.positions[triangle[i] as usize]));
            let cross=(p[1]-p[0]).cross(p[2]-p[0]);
            assert!(cross.y>0.);area+=cross.y*0.5;
            // Every retained triangle lies wholly outside at least one socket plane.
            assert!([(0,8.,1.),(0,16.,-1.),(2,8.,1.),(2,16.,-1.)].into_iter()
                .any(|(axis,edge,sign)|p.iter().all(|p|(p[axis]-edge)*sign<=0.00001)));
            for &i in triangle {assert!(surface.cutaway_ranges.iter().any(|r|r.contains(&(i as usize))));}
        }
        assert!((area-(24.*24.-8.*8.)).abs()<0.001,"outside rock must retain its whole area");
        // A face on the exact socket boundary is replaced, not left doubled.
        let mut wall=SurfaceMeshData::default();
        append_solid_quad(&mut wall,[[8.,0.,8.],[8.,0.,16.],[8.,16.,16.],[8.,16.,8.]],
            [-1.,0.,0.],[1.;4]);
        trim_stair_openings(&mut wall,&[[8.,16.,8.,16.]]);
        assert!(wall.indices.is_empty());
    }
    fn source(block:u16,column:u8,row:u8,glyph:u16) -> VisualTileSource {
        let mut s=super::super::tests::source_with_tile(block,column,row,glyph);
        s.tileset_id="cave".into();s
    }
    #[test]
    fn cave_crown_stairs_stay_inside_the_source_footprint() {
        let g=GridGeometry {width:2,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=SurfaceMeshData::default();
        append_steps(&mut mesh,&g,[0.,8.,0.,8.],16.,11.);
        append_steps(&mut mesh,&g,[0.,8.,8.,16.],11.,6.);
        assert_eq!(mesh.positions.len(),72);
        assert!(mesh.positions.iter().all(|p|p.iter().all(|v|v.is_finite())
            && (0. ..=8.).contains(&p[0]) && (6. ..=16.).contains(&p[1])
            && (0. ..=16.).contains(&p[2])));
        assert!(mesh.positions.iter().any(|p|*p==[0.,16.,0.]));
        assert!(mesh.positions.iter().any(|p|*p==[0.,6.,16.]));
        assert_eq!(mesh.normals.iter().filter(|n|n[1]==1.).count(),24);
        for triangle in mesh.indices.chunks_exact(3) {
            let [a,b,c]=[triangle[0],triangle[1],triangle[2]]
                .map(|i|Vec3::from_array(mesh.positions[i as usize]));
            let normal=Vec3::from_array(mesh.normals[triangle[0] as usize]);
            assert!((b-a).cross(c-a).dot(normal)>0.,"stair face must face its declared outward normal");
        }
    }
    #[test]
    fn cave_crown_stair_finish_removes_duplicate_caps_and_preserves_the_gap() {
        let frame=super::super::tests::frame(4,3,vec![source(0x05,0,0,0x16);12]);
        let mut tiles=frame.tiles;
        tiles[9].source=source(0x12,2,2,0x36);
        tiles[10].source=source(0x12,3,2,0x37);
        let cells=tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:4,height:3,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut shapes=vec![CellShape::RaisedTop {height:16.,solid:SolidKind::Bank};12];
        shapes[9]=CellShape::RampNorth {north_height:16.,south_height:11.};shapes[10]=shapes[9];
        shapes[11]=CellShape::PlaneAt {height:6.};
        let mut mesh=TerrainMeshData::default();
        append_solid_quad(&mut mesh.textured,[[8.,16.,24.],[8.,16.,16.],[24.,16.,16.],[24.,16.,24.]],
            [0.,1.,0.],[1.;4]);
        append_solid_quad(&mut mesh.textured,[[8.,16.,16.],[8.,16.,8.],[24.,16.,8.],[24.,16.,16.]],
            [0.,1.,0.],[1.;4]);
        let gap=mesh.textured.indices[6..].to_vec();
        finish_stairs(&mut mesh,&cells,&shapes,&g);
        assert_eq!(mesh.textured.indices,gap);
        assert!(!mesh.solid.positions.is_empty());
        assert!(mesh.solid.positions.iter().all(|p|p[2]>=16. && p[2]<=24.));
        assert!(mesh.solid.positions.chunks_exact(4).zip(mesh.solid.normals.chunks_exact(4))
            .filter(|(_,n)|n[0]==[0.,1.,0.])
            .all(|(p,_)|p.iter().any(|p|p[0]==8.) && p.iter().any(|p|p[0]==24.)));
        assert!(mesh.solid.positions.iter().zip(&mesh.solid.normals)
            .filter(|(_,n)|n[0].abs()>0.99).all(|(p,_)|p[0]==8. || p[0]==24.));
        for t in mesh.solid.indices.chunks_exact(3) {
            let [a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
            assert!((b-a).cross(c-a).dot(Vec3::from_array(mesh.solid.normals[t[0] as usize]))>=0.);
        }
    }
    #[test]
    fn cave_crown_ramp_side_finish_keeps_geometry_and_unrelated_faces() {
        let g=GridGeometry {width:3,height:3,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut shapes=vec![CellShape::Flat;9];
        shapes[4]=CellShape::RampNorth {north_height:30.,south_height:24.};
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;9],..Default::default()};
        for x in [8.,0.] {
            append_solid_quad(&mut mesh.solid,
                [[x,24.,16.],[x,30.,8.],[x,40.,8.],[x,40.,16.]],
                [-1.,0.,0.],[0.5,0.3,0.1,1.]);
        }
        let before=mesh.clone();
        finish_sides(&mut mesh,&shapes,&g,[-20,30]);
        assert_eq!(mesh.solid.positions,before.solid.positions);
        assert_eq!(mesh.solid.normals,before.solid.normals);
        assert_eq!(mesh.solid.indices,before.solid.indices);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert!(mesh.solid.uvs[..4].iter().all(|uv|uv[0]>90000.));
        assert_eq!(mesh.solid.uvs[4..],before.solid.uvs[4..]);
        assert_eq!(mesh.solid.colors[4..],before.solid.colors[4..]);
    }
    #[test]
    fn cave_crown_descent_connects_both_landings_and_rejects_incomplete_art() {
        let frame=super::super::tests::frame(4,9,vec![source(0x05,0,0,0x16);36]);
        let mut tiles=frame.tiles;
        let g=GridGeometry {width:4,height:9,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut original=vec![CellShape::Flat;36];
        for z in 1..5 {for x in 1..3 {
            original[z*4+x]=CellShape::RaisedTop {height:16.,solid:SolidKind::Bank};
        }}
        for z in 0..2 {for x in 0..2 {
            tiles[(5+z)*4+1+x].source=source(0x12,2+x as u8,2+z as u8,0x36+x as u16);
        }}
        let refs=tiles.iter().collect::<Vec<_>>();
        let mut shapes=original.clone();resolve_descents(&refs,&mut shapes,&g);
        assert_eq!(shapes[4*4+1],original[4*4+1]);
        let mut previous=16.;
        for z in 5..7 {
            let CellShape::RampNorth {north_height,south_height}=shapes[z*4+1] else {panic!("missing connected descent");};
            assert_eq!(north_height,previous);assert!(south_height<north_height);
            assert_eq!(shapes[z*4+1],shapes[z*4+2]);previous=south_height;
        }
        assert_eq!(previous,0.);assert_eq!(shapes[7*4+1],CellShape::Flat);
        let mut shelf=original.clone();
        for x in 1..3 {shelf[7*4+x]=CellShape::RaisedTop {height:6.,solid:SolidKind::Bank};}
        let refs=tiles.iter().collect::<Vec<_>>();
        resolve_descents(&refs,&mut shelf,&g);
        assert_eq!(shelf[6*4+1],CellShape::RampNorth {north_height:11.,south_height:6.});
        // Authored lower dust is glyph 0x14 in SilverCaveRoom2. Its exact
        // flat support is as valid a landing as ordinary floor paint.
        for x in 1..3 {tiles[7*4+x].source.tile_index=0x14;}
        let refs=tiles.iter().collect::<Vec<_>>();
        let mut dust=original.clone();resolve_descents(&refs,&mut dust,&g);
        assert_eq!(dust[5*4+1],CellShape::RampNorth {north_height:16.,south_height:8.});
        assert_eq!(dust[6*4+1],CellShape::RampNorth {north_height:8.,south_height:0.});
        assert_eq!(dust[7*4+1],CellShape::Flat);
        tiles[6*4+2].source.tile_index=0x16;
        let refs=tiles.iter().collect::<Vec<_>>();
        let mut shapes=original.clone();resolve_descents(&refs,&mut shapes,&g);
        assert_eq!(shapes,original);
    }
    #[test]
    fn cave_crown_inherits_its_authored_cap_but_the_lower_floor_and_ceiling_do_not() {
        let mut frame=super::super::tests::frame(7,7,
            vec![source(0x01,0,0,0x16);49]);
        frame.map_id="VictoryRoad".into();
        // A closed source wall course isolates a three-by-three crown.
        for z in 1..6 {for x in 1..6 {
            let t=&mut frame.tiles[z*7+x];
            if x==1 || x==5 || z==1 || z==5 {t.source.tile_index=0x24;}
        }}
        frame.tiles[4*7+3].source=source(0x0d,1,0,0x16);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:7,height:7,tile_width:8.,tile_height:8.,origin_x:-28.,origin_z:-28.};
        let mut shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        resolve(&frame,&cells,&mut shapes,&g);
        for z in 2..5 {for x in 2..5 {assert_eq!(shapes[z*7+x].surface_height(8.),16.);}}
        assert_eq!(shapes[0],CellShape::Flat);
        let mut silver=frame.clone();silver.map_id="SilverCaveRoom2".into();
        let cells=silver.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        resolve(&silver,&cells,&mut shapes,&g);
        assert_eq!(shapes[3*7+3].surface_height(8.),16.);
        assert_eq!(shapes[4*7+3].surface_height(8.),16.);
        assert_eq!(shapes[0],CellShape::Flat);
        let mut union=frame.clone();union.map_id="UnionCave1F".into();
        let cells=union.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let original=shapes.clone();resolve(&union,&cells,&mut shapes,&g);
        assert_eq!(shapes,original);
        // An unbounded or clipped field cannot acquire a tier from one cap.
        let mut clipped=frame.clone();clipped.tiles[3*7+1].source.tile_index=0x16;
        let cells=clipped.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        resolve(&clipped,&cells,&mut shapes,&g);
        assert_eq!(shapes[3*7+3],CellShape::Flat);
        // A ceiling mass adjacent to a floor supplies no walking-crown proof.
        clipped.tiles[4*7+3].source=source(0x2e,1,0,0x26);
        let cells=clipped.tiles.iter().collect::<Vec<_>>();
        let mut shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        resolve(&clipped,&cells,&mut shapes,&g);
        assert_eq!(shapes[3*7+3],CellShape::Flat);
    }
}
