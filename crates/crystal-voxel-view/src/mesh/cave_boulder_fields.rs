//! Adjacent complete cave rock drawings become one weathered outcrop.
//! Only their owned cells join: floor corridors, pits and height changes stay open.
use super::*;
use bevy::math::Vec2;
use std::collections::BTreeSet;

fn eligible(p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    matches!(p.form, Form::Model(Kind::CaveBoulder | Kind::DarkBoulder | Kind::IceBoulder))
        && matches!(cells[p.row*g.width+p.column].source.tileset_id.as_ref(),
            "cave" | "dark_cave" | "ice_path")
}

fn groups(placements: &[Placement], cells: &[&VisualTile], g: &GridGeometry) -> Vec<Vec<usize>> {
    let mut owner = vec![usize::MAX; g.width*g.height];
    let bases = placements.iter().map(|p| boulder_datum(p,cells,g)).collect::<Vec<_>>();
    for (i,p) in placements.iter().enumerate().filter(|(_,p)| eligible(p,cells,g)) {
        for cell in p.indices(g.width) { owner[cell] = i; }
    }
    let mut visited = vec![false; placements.len()];
    let mut result = Vec::new();
    for start in 0..placements.len() {
        if visited[start] || !eligible(&placements[start],cells,g) { continue; }
        visited[start] = true;
        let mut group = vec![start];
        let mut cursor = 0;
        while cursor < group.len() {
            let i = group[cursor]; cursor += 1;
            let p = &placements[i];
            let tileset = &cells[p.row*g.width+p.column].source.tileset_id;
            for cell in p.indices(g.width) {
                let x = cell%g.width; let z = cell/g.width;
                for (dx,dz) in [(-1,0),(1,0),(0,-1),(0,1)] {
                    let nx = x as isize+dx; let nz = z as isize+dz;
                    if nx<0 || nz<0 || nx>=g.width as isize || nz>=g.height as isize { continue; }
                    let j = owner[nz as usize*g.width+nx as usize];
                    if j == usize::MAX || visited[j] || bases[j] != bases[i] { continue; }
                    let q = &placements[j];
                    if &cells[q.row*g.width+q.column].source.tileset_id != tileset { continue; }
                    visited[j] = true; group.push(j);
                }
            }
        }
        if group.len()>1 { result.push(group); }
    }
    result
}

fn perimeter(occupied: &BTreeSet<(usize,usize)>) -> Vec<(Vec2,Vec2)> {
    let mut edges = Vec::new();
    for &(x,z) in occupied {
        let (x,z) = (x as i32,z as i32);
        for (dx,dz,a,b) in [
            (-1,0,[x,z],[x,z+1]),(1,0,[x+1,z],[x+1,z+1]),
            (0,-1,[x,z],[x+1,z]),(0,1,[x,z+1],[x+1,z+1]),
        ] {
            if x+dx<0 || z+dz<0 || !occupied.contains(&((x+dx) as usize,(z+dz) as usize)) {
                edges.push((Vec2::new(a[0] as f32,a[1] as f32),Vec2::new(b[0] as f32,b[1] as f32)));
            }
        }
    }
    edges
}

fn field_distance(point: Vec2, edges: &[(Vec2,Vec2)], origin: [i32;2]) -> f32 {
    let distance = edges.iter().map(|&(a,b)| {
        let edge = b-a;
        (point-a-edge*((point-a).dot(edge)/edge.length_squared()).clamp(0.,1.)).length()
    }).fold(f32::INFINITY,f32::min);
    let p=point+Vec2::new(origin[0] as f32,origin[1] as f32);
    // Retreat unevenly inside the owned footprint. The contact follows a
    // fractured contour instead of exposing the source rectangle as a slab.
    distance-(0.18+new_bark::meadow_noise(p.x*0.73+47.,p.y*0.73-31.)*0.12)
}

fn field_height(point: Vec2, edges: &[(Vec2,Vec2)], origin: [i32;2], rise: f32) -> f32 {
    let shoulder = (field_distance(point,edges,origin)/0.65).clamp(0.,1.);
    let shoulder = shoulder*shoulder*(3.-2.*shoulder);
    let p = point+Vec2::new(origin[0] as f32,origin[1] as f32);
    let broad = new_bark::meadow_noise(p.x*0.37+19.,p.y*0.37-7.);
    let fracture = new_bark::meadow_noise(p.x*1.1-11.,p.y*1.1+29.);
    rise*shoulder*(0.32+broad*0.24+fracture*0.20)
}

fn append_field(mesh: &mut TerrainMeshData, g: &GridGeometry, placements: &[Placement],
    group: &[usize], cells: &[&VisualTile], origin: [i32;2]) {
    let first = &placements[group[0]];
    let base = boulder_datum(first,cells,g);
    let tileset = cells[first.row*g.width+first.column].source.tileset_id.as_ref();
    let frozen = tileset == "ice_path";
    let dark = tileset == "dark_cave";
    let mut occupied = BTreeSet::new();
    let mut rise = 0_f32;
    for &i in group {
        let p = &placements[i]; rise = rise.max(p.rise_pixels*g.tile_height/SOURCE_TILE_HEIGHT);
        for cell in p.indices(g.width) { occupied.insert((cell%g.width,cell/g.width)); }
    }
    let edges = perimeter(&occupied);
    let start = mesh.solid.positions.len();
    let point = |x:f32,z:f32| (Vec3::new(g.origin_x+x*g.tile_width,
        base+field_height(Vec2::new(x,z),&edges,origin,rise),g.origin_z+z*g.tile_height),
        field_distance(Vec2::new(x,z),&edges,origin));
    // Shared quarter-cell knots preserve the exact contour and avoid cracks
    // along old sprite boundaries. Large interiors get broad connected crowns.
    for &(x,z) in &occupied {
        for dz in 0..4 { for dx in 0..4 {
            let x = x as f32+dx as f32*0.25; let z = z as f32+dz as f32*0.25;
            let q = [point(x,z),point(x,z+0.25),point(x+0.25,z+0.25),point(x+0.25,z)];
            for triangle in [[q[0],q[1],q[2]],[q[0],q[2],q[3]]] {
                let mut polygon=Vec::with_capacity(4);
                let mut previous=triangle[2];
                for current in triangle {
                    if (previous.1>0.) != (current.1>0.) {
                        // Canonical edge order gives adjoining triangles the
                        // exact same floating-point contact, without cracks.
                        let (a,b)=if (previous.0.x,previous.0.z)<(current.0.x,current.0.z)
                            {(previous,current)} else {(current,previous)};
                        let mut contact=a.0.lerp(b.0,a.1/(a.1-b.1));
                        contact.y=base;
                        polygon.push(contact);
                    }
                    if current.1>0. {polygon.push(current.0);}
                    previous=current;
                }
                for i in 1..polygon.len().saturating_sub(1) {
                    cave_stones::stone_face(&mut mesh.solid,
                        [polygon[0],polygon[i],polygon[i+1]],frozen,dark,base,rise,1.);
                }
            }
        }}
    }
    // The connected body binds the field, but its exposed crowns remain
    // broken stones. A tall smooth envelope hid every individual fracture
    // and turned the whole group into a flat-looking shelf.
    for &i in group {
        let p = &placements[i];
        let (w,_,n,_) = g.bounds(p.column,p.row);
        let e = w+p.width as f32*g.tile_width;
        let s = n+p.height as f32*g.tile_height;
        let height = p.rise_pixels*g.tile_height/SOURCE_TILE_HEIGHT;
        cave_stones::append(&mut mesh.solid,
            [w+g.tile_width*0.08,e-g.tile_width*0.08,
                n+g.tile_height*0.08,s-g.tile_height*0.08],
            base+height*0.18,height*1.6,frozen,dark,
            [origin[0]+p.column as i32,origin[1]+p.row as i32]);
    }
    // Smaller embedded fractures interrupt the old sprite-center rows. Each
    // fragment stays inside an owned cell, including around concave corridors.
    for &(x,z) in &occupied {
        let seed=new_bark::meadow_noise((x as i32+origin[0]) as f32*1.31+71.,
            (z as i32+origin[1]) as f32*1.31-53.);
        if seed<0. {continue;}
        let center=Vec2::new(x as f32+0.5,z as f32+0.5);
        let (w,e,n,s)=g.bounds(x,z);
        cave_stones::append(&mut mesh.solid,
            [w+g.tile_width*0.13,e-g.tile_width*0.13,
                n+g.tile_height*0.13,s-g.tile_height*0.13],
            base+field_height(center,&edges,origin,rise)*0.55,
            rise*(0.45+seed*0.25),frozen,dark,
            [origin[0]+x as i32,origin[1]+z as i32]);
    }
    for i in start..mesh.solid.positions.len() {
        mesh.solid.uvs[i] = cave_cliff::stone_uv(Vec3::from_array(mesh.solid.positions[i]),g,origin,frozen);
    }
}

pub(in crate::mesh) fn append_all(mesh: &mut TerrainMeshData, g: &GridGeometry, placements: &[Placement],
    cells: &[&VisualTile], origin: [i32;2]) {
    let fields = groups(placements,cells,g);
    let mut merged = vec![false;placements.len()];
    for group in &fields { for &i in group { merged[i] = true; } }
    for (i,p) in placements.iter().enumerate() {
        append_internal(mesh,g,p,cells,origin,!merged[i]);
    }
    for group in fields { append_field(mesh,g,placements,&group,cells,origin); }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(x:usize,z:usize) -> Placement {
        Placement { column:x,row:z,width:2,height:2,ground:63,
            form:Form::Model(Kind::CaveBoulder),rise_pixels:8.,depth_pixels:10.,front_rows:2. }
    }

    fn fixture() -> (GridGeometry,Vec<VisualTile>) {
        let g = GridGeometry {width:8,height:8,tile_width:8.,tile_height:8.,origin_x:-32.,origin_z:-32.};
        let mut tiles = super::super::tests::tiles(8,8,"cave");
        for t in &mut tiles {t.source.tile_index=0x16;}
        (g,tiles)
    }

    #[test]
    fn cave_boulder_fields_join_only_edge_neighbors_on_the_same_floor_and_material() {
        let (g,mut tiles) = fixture();
        let p = [placement(0,0),placement(2,0),placement(2,2),placement(5,0),placement(4,4)];
        let refs = tiles.iter().collect::<Vec<_>>();
        assert_eq!(groups(&p,&refs,&g),vec![vec![0,1,2]]);
        // Even physically adjacent complete rocks cannot bridge floor tiers.
        for z in 0..4 {for x in 0..4 {tiles[z*8+x].source.tile_index=0x01;}}
        let tiered = [placement(0,0),placement(4,0)];
        // Make the first footprint touch the second while retaining a proven
        // local upper-floor metatile behind it.
        let tiered = [Placement {width:4,..tiered[0].clone()},tiered[1].clone()];
        let refs = tiles.iter().collect::<Vec<_>>();
        assert_ne!(boulder_datum(&tiered[0],&refs,&g),boulder_datum(&tiered[1],&refs,&g));
        assert!(groups(&tiered,&refs,&g).is_empty());
        for t in &mut tiles {t.source.tile_index=0x16;}
        for z in 0..2 {for x in 2..4 {tiles[z*8+x].source.tileset_id="dark_cave".into();}}
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(groups(&p[..2],&refs,&g).is_empty());
    }

    #[test]
    fn cave_boulder_field_is_continuous_and_leaves_its_concave_floor_corridor_open() {
        let (g,tiles) = fixture(); let refs = tiles.iter().collect::<Vec<_>>();
        let p = [placement(0,0),placement(2,0),placement(0,2)];
        let mut mesh = TerrainMeshData {footing_heights:vec![0.;64],..Default::default()};
        append_all(&mut mesh,&g,&p,&refs,[-20,30]);
        assert_eq!(mesh.footing_heights,vec![0.;64]);
        assert!(mesh.textured.positions.is_empty());
        for v in &mesh.solid.positions {
            assert!(v.iter().all(|c| c.is_finite()));
            let x=(v[0]-g.origin_x)/g.tile_width; let z=(v[2]-g.origin_z)/g.tile_height;
            assert!((0. ..=4.).contains(&x) && (0. ..=4.).contains(&z));
            assert!(!(x>2. && z>2.),"filled the unowned concave floor corridor");
            if x==0. || z==0. || x==4. || z==4. || (x==2. && z>=2.) || (z==2. && x>=2.) {
                assert_eq!(v[1],0.,"field perimeter must meet the existing floor");
            }
        }
        assert!(mesh.solid.positions.iter().any(|v| v[1]==0. && {
            let x=(v[0]-g.origin_x)/g.tile_width*4.;
            let z=(v[2]-g.origin_z)/g.tile_height*4.;
            (x-x.round()).abs()>0.001 || (z-z.round()).abs()>0.001
        }), "field contact must retreat from the rectangular source grid");
        assert!(mesh.solid.normals.iter().all(|n|n.iter().all(|v|v.is_finite())));
        assert!(mesh.solid.normals.iter().any(|n|n[1].abs()<0.5),
            "rock field needs exposed fracture faces, not only a smooth roof");
        assert!(mesh.solid.positions.iter().any(|v|
            v[0]==g.origin_x+2.*g.tile_width && v[2]==g.origin_z+g.tile_height && v[1]>1.5),
            "old boulder seam must become an elevated connected shoulder");
        let occupied = BTreeSet::from([(0,0),(1,0),(2,0),(3,0),(0,1),(1,1),(2,1),(3,1)]);
        let edges = perimeter(&occupied);
        let point = Vec2::new(2.,1.);
        let shifted = edges.iter().map(|&(a,b)|(a-Vec2::X,b-Vec2::X)).collect::<Vec<_>>();
        assert_eq!(field_height(point,&edges,[-20,30],8.),
            field_height(point-Vec2::X,&shifted,[-19,30],8.));
    }
}
