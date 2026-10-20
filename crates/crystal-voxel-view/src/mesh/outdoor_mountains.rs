//! Mountain relief inside permanent outdoor rock footprints. Walking support,
//! jump banks, planted trees and cave interiors keep their authored geometry.
use super::*;
use bevy::math::Vec2;

const REACH: i32 = 6;

pub(super) fn enabled(map: &str) -> bool {
    new_bark::ordinary_lawn_map(map)
        || matches!(map, "Route10North" | "Route10South" | "CinnabarIsland" | "BattleTowerOutside")
}

/// A body reveal can see beneath the closed mineral silhouette. Continue
/// nearby authenticated dry ground there, rather than exposing the horizon.
/// This hidden base supplies presentation only, never walking support.
pub(super) fn append_reveal_floor(mesh: &mut TerrainMeshData, map: &str,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry, origin: [i32;2]) {
    append_family_reveal_floor(mesh,map,cells,shapes,g,origin,false);
}

/// Kanto canyon backing runs after terraces, so its neighboring walking
/// support is final. Volcanic rock retains its separate source paint domain.
pub(super) fn append_kanto_reveal_floor(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2]) {
    append_family_reveal_floor(mesh,map,cells,shapes,g,origin,true);
}

fn append_family_reveal_floor(mesh:&mut TerrainMeshData,map:&str,
    cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,origin:[i32;2],kanto:bool) {
    if !enabled(map) || g.width==0 || g.height==0 || cells.len()!=g.width*g.height
        || shapes.len()!=cells.len()
        || !mesh.solid.uvs.iter().any(|uv|uv[0]< -9000. && uv[1]< -30000.) {return;}
    let eligible=cells.iter().enumerate().map(|(i,cell)| {
        let owned=mesh.authored_cells.get(i).copied().flatten()==Some("dungeon:connected-rock-terrain");
        let native_border=owned && matches!(shapes[i],
            CellShape::RaisedTop{height,solid:SolidKind::Bank}|CellShape::LedgeBand{height,..}
                if height>=crate::profile::MOUNTAIN_CLIFF_HEIGHT);
        (if kanto {cell.source.tileset_id.as_ref()=="kanto"}
            else {matches!(cell.source.tileset_id.as_ref(),"johto"|"johto_modern")})
            && (permanent(&cell.source,shapes[i]) || native_border)
            && matches!(mesh.authored_cells.get(i).copied().flatten(),
                None|Some("dungeon:connected-rock-terrain"))})
        .collect::<Vec<_>>();
    let mut donors=vec![None;cells.len()];
    let mut queue=std::collections::VecDeque::new();
    for (i,cell) in cells.iter().enumerate() {
        if !eligible[i] {continue;}
        let (x,z)=((i%g.width) as isize,(i/g.width) as isize);
        let mut donor=None;
        for dz in -2isize..=2 {for dx in -2isize..=2 {
            let (nx,nz)=(x+dx,z+dz);
            if nx<0 || nz<0 || nx>=g.width as isize || nz>=g.height as isize {continue;}
            let j=nz as usize*g.width+nx as usize;
            let source=&cells[j].source;
            let ground=if kanto {matches!(source.tile_index,0x23|0x39|0x2c)
                || (source.metatile_id==0x55 && source.tile_index==0x5b)}
                else {matches!(source.tile_index,0x05|0x06)};
            let claim=mesh.authored_cells.get(j).copied().flatten();
            let owned_paving=kanto && source.metatile_id==0x55 && source.tile_index==0x5b
                && claim==Some("outdoor/kanto-brick-path");
            if source.tileset_id!=cell.source.tileset_id || !ground
                || shapes[j]!=CellShape::Flat || (claim.is_some() && !owned_paving) {continue;}
            let distance=dx*dx+dz*dz;
            if donor.is_none_or(|(d,_)|distance<d) {donor=Some((distance,j));}
        }}
        if let Some((_,ground))=donor {donors[i]=Some(ground);queue.push_back(i);}
    }
    // A wide verified rock body can be revealed several cells from its edge.
    // Continue only its authenticated neighboring floor through that connected
    // body; never cross water, another atlas, or a separately claimed object.
    while let Some(i)=queue.pop_front() {
        let (x,z)=(i%g.width,i/g.width);
        for j in [x.checked_sub(1).map(|x|z*g.width+x),
            (x+1<g.width).then_some(i+1),z.checked_sub(1).map(|z|z*g.width+x),
            (z+1<g.height).then_some(i+g.width)].into_iter().flatten() {
            if eligible[j] && donors[j].is_none()
                && cells[j].source.tileset_id==cells[i].source.tileset_id {
                donors[j]=donors[i];queue.push_back(j);
            }
        }
    }
    let mut caps=Vec::new();
    let cap_vertices=mesh.mineral_cap_ranges.as_ref().map(|ranges| {
        let mut selected=vec![false;mesh.solid.positions.len()];
        for range in ranges {selected[range.clone()].fill(true);}
        selected
    });
    for indices in mesh.solid.indices.chunks_exact(3) {
        let ids=[indices[0] as usize,indices[1] as usize,indices[2] as usize];
        if cap_vertices.as_ref().is_some_and(|selected|ids.iter().any(|&i|!selected[i])) {continue;}
        if !ids.iter().all(|&i|mesh.solid.uvs.get(i)
            .is_some_and(|uv|uv[0]< -9000. && uv[1]< -30000.)) {continue;}
        if kanto && ids.iter().any(|&i|mesh.solid.uvs[i][1]< -50000.) {continue;}
        let p=ids.map(|i|Vec3::from_array(mesh.solid.positions[i]));
        if (p[1]-p[0]).cross(p[2]-p[0]).y<=0.0001 {continue;}
        // Source UVs retain the unsculpted cell identity. The actual positions
        // retain its rounded boundary, which this backing must never exceed.
        let uv=ids.map(|i|Vec2::from_array(mesh.solid.uvs[i])).into_iter().sum::<Vec2>()/3.;
        let x=((-10000.-uv.x)/g.tile_width-origin[0] as f32).floor() as isize;
        let z=((-40000.-uv.y)/g.tile_height-origin[1] as f32).floor() as isize;
        if x<0 || z<0 || x>=g.width as isize || z>=g.height as isize {continue;}
        let cell=z as usize*g.width+x as usize;
        let Some(ground)=donors[cell] else {continue;};
        let support_cell=if kanto {ground}else{cell};
        let support=mesh.footing_heights.get(support_cell).copied().filter(|h|h.is_finite()).unwrap_or(0.);
        let seat_bias=g.tile_height*0.01/SOURCE_TILE_HEIGHT;
        let floor=if support>0.02 {support-seat_bias}else{support};
        let clearance=g.tile_height/SOURCE_TILE_HEIGHT;
        let clip_height=floor+clearance;
        // Keep a strict interior gap. Clipping at the floor itself makes its
        // edge touch descending rock, exposing a pale line in the depth pass.
        // Require one source pixel of rock above this hidden floor. A cap
        // merely seating at walking height needs no backing through its rim.
        let mut clipped=Vec::with_capacity(4);
        for edge in 0..3 {
            let (a,b)=(p[edge],p[(edge+1)%3]);
            let (inside_a,inside_b)=(a.y>=clip_height,b.y>=clip_height);
            if inside_a {clipped.push(a);}
            if inside_a!=inside_b {clipped.push(a.lerp(b,(clip_height-a.y)/(b.y-a.y)));}
        }
        for i in 1..clipped.len().saturating_sub(1) {
            let triangle=[clipped[0],clipped[i],clipped[i+1]]
                .map(|p|[p.x,floor,p.z]);
            caps.push((ground,triangle));
        }
    }
    new_bark::append_rock_reveal_caps(&mut mesh.solid,map,cells,g,origin,&caps);
}

/// These native corner drawings contain mineral courses, not planted trees.
/// Match their actual ink so custom planted art keeps the existing flat seat.
pub(super) fn native_rock_corner(source: &VisualTileSource) -> bool {
    if !matches!(source.tileset_id.as_ref(), "johto" | "johto_modern")
        || source.subtile_column >= 4 || source.subtile_row >= 4 {return false;}
    let drawing = match source.metatile_id {
        0x6a => [0x3c,0x2b,0x2c,0x2c,0x2b,0x3b,0x3c,0x3c,
            0x3b,0x3b,0x3c,0x3c,0x3b,0x3b,0x3c,0x3c],
        0x6c => [0x3b,0x3b,0x3c,0x3c,0x3b,0x3b,0x3c,0x3c,
            0x3b,0x4b,0x4c,0x4c,0x4b,0x4c,0x4c,0x4c],
        _ => return false,
    };
    source.tile_index == drawing[source.subtile_row as usize*4+source.subtile_column as usize]
}

pub(super) fn permanent(source: &VisualTileSource, shape: CellShape) -> bool {
    // The complete native entrance producer owns these two-by-two mouths.
    // Leave a real opening in the relief field above their warp footing.
    if source.metatile_id == 0x73
        && source.subtile_row >= 2
        && ((source.tileset_id.as_ref() == "johto" && source.subtile_column < 2)
            || (source.tileset_id.as_ref() == "johto_modern" && source.subtile_column >= 2))
    {
        return false;
    }
    if source.tileset_id.as_ref() == "kanto"
        && source.metatile_id == 0x06
        && source.subtile_row >= 2
        && source.subtile_column < 2
    {
        return false;
    }
    let family = match source.tileset_id.as_ref() {
        "johto" | "johto_modern" => {
            source.metatile_id == 0x0a
                || ((0x68..=0x73).contains(&source.metatile_id)
                    && (!matches!(source.metatile_id, 0x6a | 0x6c)
                        || native_rock_corner(source)))
        }
        "kanto" => crate::kanto_cliff::kanto_cliff_shape(source).is_some_and(|native| {
            native.surface_height(SOURCE_TILE_HEIGHT) >= crate::kanto_cliff::KANTO_CLIFF_HEIGHT
        }),
        "battle_tower_outside" => crate::battle_tower::cliff_source(source),
        _ => false,
    };
    family
        && matches!(shape, CellShape::RaisedTop { solid: SolidKind::Bank, height }
        | CellShape::LedgeBand { height, .. } if height >= crate::profile::MOUNTAIN_CLIFF_HEIGHT)
}

struct Relief {
    width: usize,
    height: usize,
    peaks: Vec<f32>,
}

impl Relief {
    fn new(
        cells: &[&VisualTile],
        shapes: &[CellShape],
        g: &GridGeometry,
        origin: [i32; 2],
    ) -> Self {
        let rock: Vec<_> = cells
            .iter()
            .zip(shapes)
            .map(|(cell, &shape)| permanent(&cell.source, shape))
            .collect();
        let mut peaks = vec![0.; (g.width + 1) * (g.height + 1)];
        for z in 0..=g.height {
            for x in 0..=g.width {
                // Distance to the nearest real opening, including paths, stairs,
                // portals and tree seats. A viewport cut is not a mountain edge.
                let mut distance = REACH as f32;
                for dz in -REACH..REACH {
                    for dx in -REACH..REACH {
                        let (cx, cz) = (x as i32 + dx, z as i32 + dz);
                        if cx < 0 || cz < 0 || cx >= g.width as i32 || cz >= g.height as i32 {
                            continue;
                        }
                        if rock[cz as usize * g.width + cx as usize] {
                            continue;
                        }
                        let px = (-(dx as f32) - 1.).max(dx as f32).max(0.);
                        let pz = (-(dz as f32) - 1.).max(dz as f32).max(0.);
                        distance = distance.min(px.hypot(pz));
                    }
                }
                // Large intersecting planar beds form unequal ridges rather than
                // a cone repeated at every metatile. World coordinates preserve
                // the formation through connected-map origins and camera crops.
                let wx = x as f32 + origin[0] as f32;
                let wz = z as f32 + origin[1] as f32;
                let bed = super::cave_cliff::slab(wx * 0.105 + 17., wz * 0.105 - 31.);
                let ridge = 2.5 + bed.abs() * 3.;
                peaks[z * (g.width + 1) + x] = distance.min(ridge) * 1.85 * g.tile_height;
            }
        }
        Self {
            width: g.width,
            height: g.height,
            peaks,
        }
    }

    fn height(&self, x: f32, z: f32) -> f32 {
        if x < 0. || z < 0. || x > self.width as f32 || z > self.height as f32 {
            return 0.;
        }
        let cx = (x.floor() as usize).min(self.width - 1);
        let cz = (z.floor() as usize).min(self.height - 1);
        let (u, v) = (x - cx as f32, z - cz as f32);
        let i = cz * (self.width + 1) + cx;
        let [a, b, c, d] = [
            self.peaks[i],
            self.peaks[i + 1],
            self.peaks[i + self.width + 1],
            self.peaks[i + self.width + 2],
        ];
        // Follow the cap mesh's northwest/southeast diagonal so a mineral
        // bed remains one plane instead of acquiring extra crossing creases.
        if u <= v {
            a + (c - a) * v + (d - c) * u
        } else {
            a + (b - a) * u + (d - b) * v
        }
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    if !enabled(map)
        || g.width == 0
        || g.height == 0
        || !cells
            .iter()
            .zip(shapes)
            .any(|(c, &s)| permanent(&c.source, s)
                || super::terrace_datum::mountain_ponds::plateau(&c.source))
    {
        return;
    }
    let relief = Relief::new(cells, shapes, g, origin);
    for (i, (position, uv)) in mesh
        .solid
        .positions
        .iter_mut()
        .zip(&mesh.solid.uvs)
        .enumerate()
    {
        if uv[0] >= -9000. {
            continue;
        }
        // Stone UVs retain the original source point through rim sculpting.
        // Use that point on both wall and cap to keep their seams joined.
        let x = (-10000. - uv[0]) / g.tile_width - origin[0] as f32;
        let tag = if uv[1] < -30000. { 40000. } else { 20000. };
        let z = (-tag - uv[1]) / g.tile_height - origin[1] as f32;
        // Folded native cliff courses can extend their source footprint past
        // the exposed wall plane. Anchor every mineral toe at the real ground
        // instead of lifting the entire wall along with the mountain cap.
        let support = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        let rise = relief.height(x, z) * (position[1] / support).clamp(0., 1.);
        position[1] += rise;
        if map == "CinnabarIsland" {
            if let Some(color)=mesh.solid.colors.get_mut(i) {
                *color=[0.20,0.205,0.215,color[3]];
            }
        } else if rise > 0.
            && let Some(color) = mesh.solid.colors.get_mut(i)
        {
            let grain =
                super::cave_cliff::slab((x + origin[0] as f32) * 1.7, (z + origin[1] as f32) * 1.7);
            let brown = [
                0.34 + grain * 0.08,
                0.215 + grain * 0.055,
                0.105 + grain * 0.03,
            ];
            let weight = (rise / g.tile_height).min(1.);
            for c in 0..3 {
                color[c] += (brown[c] - color[c]) * weight;
            }
        }
    }
    if map == "CinnabarIsland" {
        // Keep the same rock geometry and world-registered texture field, but
        // retain the island's volcanic pigment through the shared shader.
        // Otherwise the canyon finish paints warm sediment over this gray.
        for uv in &mut mesh.solid.uvs {
            if uv[0] < -9000. {
                uv[1] -= if uv[1] < -30000. { 20000. } else { 40000. };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descending_caps_keep_their_hidden_floor_strictly_inside_the_rock() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        let p=[[48.8,15.5,48.8],[48.8,18.,55.2],[55.2,18.,55.2],[55.2,15.5,48.8]];
        append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
            p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before=mesh.clone();
        append_reveal_floor(&mut mesh,"CianwoodCity",&cells,&shapes,&g,[0,0]);
        let floor=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .filter(|(_,uv)|uv[0]>87000. && uv[0]<90000.).collect::<Vec<_>>();
        assert!(!floor.is_empty());
        for (p,_) in floor {
            let rock=15.5+(p[2]-48.8)/(55.2-48.8)*2.5;
            assert!(rock>=16.9899,"the backing needs one source pixel inside the descending cap: {p:?}");
            assert!((p[1]-15.99).abs()<0.0001,"the actual hidden-floor height stays unchanged");
        }
        assert_eq!(&mesh.solid.positions[..before.solid.positions.len()],&before.solid.positions);
        assert_eq!(mesh.footing_heights,before.footing_heights);
    }

    #[test]
    fn weathered_upward_wall_facets_cannot_project_hidden_floor_shelves() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        let cap=[[48.8,32.,48.8],[48.8,32.,55.2],[55.2,32.,55.2],[55.2,32.,48.8]];
        append_quad_colors(&mut mesh.solid,cap,[0.,1.,0.],
            cap.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        let wall=[[56.5,20.,49.],[57.5,20.,49.],[57.5,32.,48.8],[56.5,32.,48.8]];
        let n=(Vec3::from_array(wall[1])-Vec3::from_array(wall[0]))
            .cross(Vec3::from_array(wall[3])-Vec3::from_array(wall[0])).normalize();
        assert!(n.y>0.,"fixture must reproduce an upward-facing wall facet");
        append_quad_colors(&mut mesh.solid,wall,n.to_array(),
            wall.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        outdoor_stone::retain_mineral_cap_ownership(&mut mesh);
        let before=mesh.clone();
        append_reveal_floor(&mut mesh,"CianwoodCity",&cells,&shapes,&g,[0,0]);
        let floor=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .filter(|(_,uv)|uv[0]>87000. && uv[0]<90000.).collect::<Vec<_>>();
        assert_eq!(floor.len(),6,"only the true cap supplies its two backing triangles");
        assert!(floor.iter().all(|(p,_)|p[0]>=48.8 && p[0]<=55.2 && p[2]>=48.8 && p[2]<=55.2),
            "a vertical wall cannot extend the hidden floor beyond the original cap");
        assert_eq!(&mesh.solid.positions[..before.solid.positions.len()],&before.solid.positions);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert_eq!(mesh.authored_cells,before.authored_cells);
    }

    #[test]
    fn native_corner_courses_continue_the_mountain_but_custom_planted_ink_keeps_its_seat() {
        for family in ["johto", "johto_modern"] {
            for (block,column,row,tile) in [(0x6a,1,1,0x3b),(0x6c,2,2,0x4c)] {
                let (mut frame,g)=fixture(family,0x0a);
                let index=6*g.width+6;
                let mut source=super::super::tests::source_with_tile(block,column,row,tile);
                source.tileset_id=family.into();
                assert!(native_rock_corner(&source));
                frame.tiles[index].source=source.clone();
                let cells=frame.tiles.iter().collect::<Vec<_>>();
                let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
                let relief=Relief::new(&cells,&shapes,&g,[0,0]);
                assert!(relief.height(6.5,6.5)>g.tile_height*2.,
                    "a native corner cannot cut a flat shelf into a continuous mountain");
                let mut mesh=TerrainMeshData {footing_heights:vec![0.;cells.len()],
                    authored_cells:vec![None;cells.len()],..Default::default()};
                mesh.solid.positions=vec![[52.,16.,52.]];
                mesh.solid.uvs=vec![[-10052.,-40052.]];
                let before=mesh.clone();
                finish(&mut mesh,"CianwoodCity",&cells,&shapes,&g,[0,0]);
                assert!(mesh.solid.positions[0][1]>16.);
                assert_eq!(mesh.footing_heights,before.footing_heights);
                assert_eq!(mesh.authored_cells,before.authored_cells);
                source.tile_index=0x20;
                assert!(!native_rock_corner(&source));
                assert!(!permanent(&source,shapes[index]),"custom planted art retains its exclusion");
            }
        }
    }

    #[test]
    fn reveal_floor_stays_inside_native_rock_without_changing_support_or_covering_water() {
        let (mut frame,g)=fixture("johto",0x0a);
        frame.tiles[6*12+6].source.metatile_id=0x6c;
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        mesh.authored_cells[6*12+6]=Some("dungeon:connected-rock-terrain");
        mesh.authored_cells[5*12+5]=Some("custom");
        // The actual mineral cap is inset from its source tile. Reusing raw
        // source rectangles would pass ownership checks but expose shelves.
        for z in 2..10 {for x in 2..10 {
            let (w,e,n,s)=g.bounds(x,z);
            let p=[[w+0.8,32.,n+0.8],[w+0.8,32.,s-0.8],
                [e-0.8,32.,s-0.8],[e-0.8,32.,n+0.8]];
            append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
                p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        }}
        let original=mesh.clone();
        append_reveal_floor(&mut mesh,"BlackthornCity",&cells,&shapes,&g,[0,0]);
        assert!(mesh.solid.indices.len()>original.solid.indices.len(),
            "reveal needs real ground behind the mineral face");
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
        assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions);
        assert_eq!(&mesh.solid.normals[..original.solid.normals.len()],&original.solid.normals);
        assert_eq!(&mesh.solid.indices[..original.solid.indices.len()],&original.solid.indices);
        let mut backed=std::collections::BTreeSet::new();
        for tri in mesh.solid.indices[original.solid.indices.len()..].chunks_exact(3) {
            let p=tri.iter().map(|&i|Vec3::from_array(mesh.solid.positions[i as usize])).sum::<Vec3>()/3.;
            assert!((p.y-15.99).abs()<0.0001);
            let (x,z)=((p.x/g.tile_width).floor() as usize,(p.z/g.tile_height).floor() as usize);
            let i=z*g.width+x;
            backed.insert(i);
            assert!(permanent(&cells[i].source,shapes[i]) || i==6*12+6,
                "hidden floor cannot extend onto open ground or water");
            let (w,e,n,s)=g.bounds(x,z);
            for &v in tri {
                let p=mesh.solid.positions[v as usize];
                assert!(p[0]>=w+0.799 && p[0]<=e-0.799 && p[2]>=n+0.799 && p[2]<=s-0.799,
                    "backing protrudes beyond the sculpted cap: {p:?}");
            }
        }
        assert!(backed.contains(&(6*12+6)),"a reveal at the center of a wide native formation needs backing too");
        assert!(!backed.contains(&(5*12+5)),"a custom object retains its floor ownership");
        let mut wet=frame.clone();
        for tile in &mut wet.tiles {
            if !permanent(&tile.source,shape_for_source(&tile.source)) {
                tile.source=super::super::tests::source_with_tile(0x34,0,0,0x14);
            }
        }
        let wet_cells=wet.tiles.iter().collect::<Vec<_>>();
        let wet_shapes=wet_cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut retained=original.clone();
        append_reveal_floor(&mut retained,"BlackthornCity",&wet_cells,&wet_shapes,&g,[0,0]);
        assert_eq!(retained.solid,original.solid,"water is never a dry-floor donor");
        let mut foreign=frame.clone();
        for tile in &mut foreign.tiles {
            if !permanent(&tile.source,shape_for_source(&tile.source)) {
                tile.source.tileset_id="johto_modern".into();
            }
        }
        let cells=foreign.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut retained=original.clone();
        append_reveal_floor(&mut retained,"BlackthornCity",&cells,&shapes,&g,[0,0]);
        assert_eq!(retained.solid,original.solid,"a connected atlas cannot supply this rock's ground pigment");
    }

    #[test]
    fn production_reveal_backing_uses_the_final_raised_cap() {
        let (frame,g)=fixture("johto",0x0a);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],
            footing_heights:vec![16.;cells.len()],..Default::default()};
        let p=[[48.8,8.,48.8],[48.8,8.,55.2],[55.2,8.,55.2],[55.2,8.,48.8]];
        append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
            p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        let support=mesh.footing_heights.clone();
        outdoor_stone::finish_rock_geometry(&mut mesh,"BlackthornCity",&cells,&shapes,&g,[0,0]);
        assert!(mesh.solid.positions[..4].iter().all(|p|p[1]>16.),
            "fixture must reproduce relief rising above its walking support");
        let floor=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .filter(|(_,uv)|(87000. ..90000.).contains(&uv[0])).collect::<Vec<_>>();
        assert!(!floor.is_empty(),"backing must be clipped against final relief, not the low initial cap");
        assert!(floor.iter().all(|(p,_)|(p[1]-15.99).abs()<0.0001));
        assert_eq!(mesh.footing_heights,support);
    }

    #[test]
    fn kanto_reveal_floor_uses_final_dry_support_inside_the_existing_cap_only() {
        let (mut frame,g)=fixture("kanto",0x2c);
        for tile in &mut frame.tiles {
            if tile.source.metatile_id==0x2c {tile.source.tile_index=0x11;}
            else {tile.source.tileset_id="kanto".into();tile.source.metatile_id=0x55;tile.source.tile_index=0x5b;}
        }
        let blocked=5*g.width+5;let wet=6*g.width+6;
        frame.tiles[wet].source.metatile_id=0x43;frame.tiles[wet].source.tile_index=0x14;
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData {footing_heights:vec![7.;cells.len()],
            authored_cells:vec![Some("outdoor/kanto-brick-path");cells.len()],..Default::default()};
        for z in 2..10 {for x in 2..10 {
            let i=z*g.width+x;
            mesh.footing_heights[i]=16.;mesh.authored_cells[i]=Some("dungeon:connected-rock-terrain");
            let (w,e,n,s)=g.bounds(x,z);
            let p=[[w+0.8,32.,n+0.8],[w+0.8,32.,s-0.8],[e-0.8,32.,s-0.8],[e-0.8,32.,n+0.8]];
            append_quad_colors(&mut mesh.solid,p,[0.,1.,0.],
                p.map(|p|[-10000.-p[0],-40000.-p[2]]),[[0.3,0.2,0.1,1.];4]);
        }}
        mesh.authored_cells[blocked]=Some("custom");mesh.authored_cells[wet]=None;
        let original=mesh.clone();
        append_kanto_reveal_floor(&mut mesh,"Route22",&cells,&shapes,&g,[0,0]);
        assert!(mesh.solid.positions.len()>original.solid.positions.len());
        assert_eq!(mesh.footing_heights,original.footing_heights);
        assert_eq!(mesh.authored_cells,original.authored_cells);
        assert_eq!(&mesh.solid.positions[..original.solid.positions.len()],&original.solid.positions);
        assert_eq!(&mesh.solid.indices[..original.solid.indices.len()],&original.solid.indices);
        for tri in mesh.solid.indices[original.solid.indices.len()..].chunks_exact(3) {
            let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.solid.positions[i as usize]));
            assert!((p[1]-p[0]).cross(p[2]-p[0]).y>0.);
            for point in p {
                assert!((point.y-6.99).abs()<0.0001,"use the adjacent road, not the cliff's 16-unit standing datum");
                let x=(point.x/8.).floor()as usize;let z=(point.z/8.).floor()as usize;
                let cell=z*g.width+x;assert!(cell!=blocked&&cell!=wet);
                assert!(point.x>=x as f32*8.+0.799 && point.x<=(x+1)as f32*8.-0.799);
                assert!(point.z>=z as f32*8.+0.799 && point.z<=(z+1)as f32*8.-0.799);
            }
        }
        for fault in 0..3 {
            let mut source=frame.clone();let mut retained=original.clone();
            for (i,tile) in source.tiles.iter_mut().enumerate() {
                if (2..10).contains(&(i%g.width)) && (2..10).contains(&(i/g.width)) {continue;}
                match fault {
                    0=>{tile.source.metatile_id=0x43;tile.source.tile_index=0x14;},
                    1=>retained.authored_cells[i]=Some("custom"),
                    _=>tile.source.tileset_id="johto_modern".into(),
                }
            }
            let refs=source.tiles.iter().collect::<Vec<_>>();
            let shapes=refs.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
            append_kanto_reveal_floor(&mut retained,"Route22",&refs,&shapes,&g,[0,0]);
            assert_eq!(retained.solid,original.solid,"missing, wet or separately owned ground cannot supply backing");
        }
        let mut volcanic=original.clone();
        for uv in &mut volcanic.solid.uvs {uv[1]-=20000.;}
        let before=volcanic.solid.clone();
        append_kanto_reveal_floor(&mut volcanic,"CinnabarIsland",&cells,&shapes,&g,[0,0]);
        assert_eq!(volcanic.solid,before,"volcanic paint retains its separate domain");
    }

    #[test]
    fn volcanic_finish_preserves_relief_footing_and_texture_registration() {
        let (mut frame, g) = fixture("kanto", 0x2c);
        for tile in &mut frame.tiles {
            if tile.source.metatile_id == 0x2c { tile.source.tile_index = 0x11; }
        }
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect::<Vec<_>>();
        let mut original = TerrainMeshData::default();
        original.footing_heights = vec![7.; cells.len()];
        original.solid.positions = vec![[48.,16.,48.];3];
        original.solid.normals = vec![[0.,1.,0.];3];
        original.solid.colors = vec![[0.4,0.3,0.2,1.];3];
        original.solid.uvs = vec![[-10048.,-40048.],[-10048.,-20048.],crate::scenery_finish::SOURCE_PAINT_UV];
        let mut island = original.clone();
        let mut mainland = original.clone();
        finish(&mut island,"CinnabarIsland",&cells,&shapes,&g,[0,0]);
        finish(&mut mainland,"LavenderTown",&cells,&shapes,&g,[0,0]);
        assert_eq!(island.solid.positions,mainland.solid.positions);
        assert_eq!(island.footing_heights,original.footing_heights);
        assert_eq!(mainland.solid.uvs,original.solid.uvs);
        for i in 0..2 {
            assert_eq!(island.solid.uvs[i],[-10048.,-60048.]);
            assert!(island.solid.colors[i][2]>island.solid.colors[i][0],"volcanic stone keeps its cool ash tint");
        }
        assert_eq!(island.solid.uvs[2],original.solid.uvs[2]);
        assert_eq!(island.solid.colors[2],original.solid.colors[2]);
    }

    fn fixture(tileset: &str, metatile: u16) -> (VisualWorldFrame, GridGeometry) {
        let mut source = super::super::tests::source_with_tile(metatile, 1, 1, 0x3c);
        source.tileset_id = tileset.into();
        let flat = super::super::tests::source_with_tile(1, 0, 0, 5);
        let mut sources = vec![flat; 144];
        for z in 2..10 {
            for x in 2..10 {
                sources[z * 12 + x] = source.clone();
            }
        }
        (
            super::super::tests::frame(12, 12, sources),
            GridGeometry {
                width: 12,
                height: 12,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }

    #[test]
    fn kanto_mountains_retain_their_ridges_without_changing_walk_support_or_water() {
        let (mut frame,g)=fixture("kanto",0x2c);
        for tile in &mut frame.tiles {if tile.source.metatile_id==0x2c {tile.source.tile_index=0x11;}}
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let field=Relief::new(&cells,&shapes,&g,[0,0]);
        assert!(field.height(6.,6.)>g.tile_height*2.,"native mountain masses must retain their relief");
        let mut mesh=TerrainMeshData::default();
        mesh.solid.positions=vec![[48.,16.,48.],[40.,16.,48.],[48.,-2.,48.]];
        mesh.solid.uvs=vec![[-10048.,-40048.],[-10040.,-40048.],[0.5,0.5]];
        mesh.solid.colors=vec![[0.4,0.3,0.2,1.];3];
        mesh.footing_heights=vec![16.;144];
        let before=mesh.clone();
        finish(&mut mesh,"CinnabarIsland",&cells,&shapes,&g,[0,0]);
        assert!(mesh.solid.positions[..2].iter().zip(&before.solid.positions).all(|(a,b)|a[1]>b[1]));
        assert_eq!(mesh.solid.positions[2],before.solid.positions[2],"water stays on its level plane");
        assert_eq!(mesh.solid.colors[2],before.solid.colors[2]);
        assert_eq!(mesh.solid.uvs[2],before.solid.uvs[2]);
        assert_eq!(mesh.footing_heights,before.footing_heights);
        assert!(mesh.solid.colors[..2].iter().all(|c|c[2]>c[0]));
    }

    #[test]
    fn permanent_rock_grows_ridges_inside_its_footprint() {
        let (frame, g) = fixture("johto", 0x0a);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        assert!(field.height(6., 6.) > g.tile_height * 2.,
            "wide permanent rock should retain substantial relief");
        for z in 0..12 { for x in 0..12 {
            let h = field.height(x as f32, z as f32);
            for (dx, dz) in [(1., 0.), (0., 1.)] {
                let adjacent = field.height(x as f32 + dx, z as f32 + dz);
                assert!((adjacent-h).abs() <= g.tile_height * 1.85 + 0.001,
                    "the restored ridge must retain its bounded slope: {h} -> {adjacent}");
            }
        }}
        for x in 0..=12 {
            for z in [0., 2., 10., 12.] {
                assert_eq!(
                    field.height(x as f32, z),
                    0.,
                    "mountain spreads over its source edge"
                );
            }
        }
        // Shared cap/wall samples are independent of previous rock relief.
        let mut mesh = TerrainMeshData::default();
        mesh.solid.positions = vec![
            [48., 16., 48.],
            [47.9, 16., 48.1],
            [16., 0., 48.],
            [48., 0., 48.],
        ];
        mesh.solid.uvs = vec![
            [-10048., -20048.],
            [-10048., -40048.],
            [-10016., -20048.],
            [-10048., -20048.],
        ];
        mesh.solid.colors = vec![[0.1, 0.1, 0.1, 1.]; 4];
        mesh.footing_heights = vec![16.; 144];
        let support = mesh.footing_heights.clone();
        let original = mesh.solid.clone();
        finish(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
        assert_eq!(mesh.solid.positions[0][1], mesh.solid.positions[1][1]);
        assert_eq!(
            mesh.solid.colors[0], mesh.solid.colors[1],
            "cap and wall must share the same mineral pigment"
        );
        assert_eq!(mesh.solid.positions[2], [16., 0., 48.]);
        assert_eq!(
            mesh.solid.positions[3],
            [48., 0., 48.],
            "the mountain toe must stay grounded"
        );
        assert_eq!(mesh.footing_heights, support);
        assert_eq!(mesh.solid.uvs, original.uvs);
        assert_eq!(mesh.solid.indices, original.indices);
        for (before, after) in original.positions.iter().zip(&mesh.solid.positions) {
            assert_eq!(
                [before[0], before[2]],
                [after[0], after[2]],
                "mountain detail cannot change the horizontal footprint"
            );
        }
    }

    #[test]
    fn broad_summits_vary_without_repeating_a_distance_plateau() {
        let (seed, g) = fixture("johto", 0x0a);
        let rock = seed.tiles[6 * 12 + 6].source.clone();
        let flat = seed.tiles[0].source.clone();
        let frame = super::super::tests::frame(32, 32, (0..1024).map(|i| {
            if (2..30).contains(&(i % 32)) && (2..30).contains(&(i / 32)) {
                rock.clone()
            } else { flat.clone() }
        }).collect());
        let g = GridGeometry { width: 32, height: 32, ..g };
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        // All four samples have the same distance to the rectangular edge.
        // Compare across a whole formation: neighboring tiles belong to one
        // broad ridge, rather than four unrelated miniature summits.
        let heights = [(6., 6.), (26., 6.), (6., 26.), (26., 26.)]
            .map(|(x, z)| field.height(x, z));
        let low = heights.into_iter().fold(f32::INFINITY, f32::min);
        let high = heights.into_iter().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            high - low > g.tile_height * 0.5,
            "summits repeat: {heights:?}"
        );
        assert!(enabled("Route10South") && enabled("Route10North") && enabled("CinnabarIsland"));
    }

    #[test]
    fn caves_jump_ledges_and_tree_seats_are_unchanged() {
        let mut jump = super::super::tests::source_with_tile(0x07, 1, 1, 0x2c);
        jump.tileset_id = "kanto".into();
        assert!(
            !permanent(
                &jump,
                CellShape::RaisedTop {
                    height: 48.,
                    solid: SolidKind::Bank
                }
            ),
            "a resolved upper terrace cannot turn a jump bank into a mountain"
        );
        for (tileset, block, map) in [
            ("johto", 0x0a, "DragonsDenB1F"),
            ("kanto", 0x2c, "RockTunnel1F"),
            ("johto", 0x0a, "UndergroundPath"),
            ("cave", 0x0a, "MountMoon"),
            ("dark_cave", 0x0a, "DarkCaveVioletEntrance"),
            ("johto", 0x4b, "Route26"),
            ("johto", 0x6a, "Route26"),
            ("johto_modern", 0x6c, "Route33"),
            ("kanto", 0x07, "Route1"),
        ] {
            let (frame, g) = fixture(tileset, block);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let mut mesh = TerrainMeshData::default();
            mesh.solid.positions = vec![[48., 16., 48.]];
            mesh.solid.uvs = vec![[-10048., -20048.]];
            let before = mesh.clone();
            finish(&mut mesh, map, &cells, &shapes, &g, [0, 0]);
            assert_eq!(mesh, before, "{tileset}/{block:02x} in {map}");
        }
    }

    #[test]
    fn mountain_openings_and_world_origin_survive_relief() {
        let (mut frame, g) = fixture("johto", 0x0a);
        for z in 5..7 {
            for x in 5..7 {
                frame.tiles[z * 12 + x].source =
                    super::super::tests::source_with_tile(0x73, (x - 5) as u8, (z - 3) as u8, 0x46);
            }
        }
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        assert_eq!(field.height(5.5, 5.5), 0., "cave entrance must stay open");
        // A cropped viewport with a translated origin samples the same mass.
        let original = &cells;
        let cropped: Vec<_> = (1..12)
            .flat_map(|z| (1..12).map(move |x| original[z * 12 + x]))
            .collect();
        let cropped_shapes: Vec<_> = cropped
            .iter()
            .map(|c| shape_for_source(&c.source))
            .collect();
        let cg = GridGeometry {
            width: 11,
            height: 11,
            ..g
        };
        let shifted = Relief::new(&cropped, &cropped_shapes, &cg, [1, 1]);
        for (x, z) in [(3.5, 3.5), (7.5, 7.5), (8.25, 4.5)] {
            assert!((field.height(x, z) - shifted.height(x - 1., z - 1.)).abs() < 0.0001);
        }
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_route26_has_mountains_and_preserves_every_open_tile() {
        check_external_footprints(&["Route26"]);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_lavender_cinnabar_and_johto_keep_their_rock_footprints() {
        check_external_footprints(&[
            "Route10South",
            "LavenderTown",
            "CinnabarIsland",
            "CianwoodCity",
            "Route29",
        ]);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_named_outdoor_sites_keep_rock_relief_out_of_open_footprints() {
        check_external_footprints(&["RuinsOfAlphOutside","LakeOfRage","SilverCaveOutside","MountMoonSquare"]);
    }

    fn check_external_footprints(maps: &[&str]) {
        let path = std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        for &map in maps {
            assert!(enabled(map));
            let module = &pack.data().maps[map];
            let tileset = &module.attributes.tileset_name;
            let layout = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let width = module.attributes.width as usize * 4;
            let height = module.attributes.height as usize * 4;
            let sources = (0..height)
                .flat_map(|z| {
                    (0..width).map(move |x| {
                        let block = module.blocks[z / 4 * (width / 4) + x / 4];
                        VisualTileSource {
                            tileset_id: tileset.as_str().into(),
                            metatile_id: block,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (z % 4) as u8,
                            tile_index: layout[block as usize * 16 + z % 4 * 4 + x % 4] as u16,
                        }
                    })
                })
                .collect();
            let frame = super::super::tests::frame(width as u32, height as u32, sources);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let g = GridGeometry {
                width,
                height,
                tile_width: 16.,
                tile_height: 16.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let field = Relief::new(&cells, &shapes, &g, [0, 0]);
            let mut peaks = 0;
            for (i, cell) in cells.iter().enumerate() {
                let x = (i % width) as f32;
                let z = (i / width) as f32;
                if permanent(&cell.source, shapes[i]) {
                    peaks += usize::from(field.height(x + 0.5, z + 0.5) > g.tile_height * 2.);
                } else {
                    for (dx, dz) in [(0., 0.), (0.5, 0.5), (1., 1.)] {
                        assert_eq!(
                            field.height(x + dx, z + dz),
                            0.,
                            "{map}: relief enters open tile {x},{z}"
                        );
                    }
                }
            }
            if map == "Route26" || map == "CianwoodCity" {
                assert!(
                    peaks > 100,
                    "{map} should contain substantial mountain relief: {peaks}"
                );
            }
        }
    }
}
