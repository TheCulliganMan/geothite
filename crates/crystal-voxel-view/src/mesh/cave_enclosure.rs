//! Solid geology outside the actual source map, never a flat ground apron.
use super::*;

fn ordered_tiles<'a>(frame: &'a VisualWorldFrame, g: &GridGeometry) -> Option<Vec<&'a VisualTile>> {
    let mut tiles = vec![None; g.width * g.height];
    for t in &frame.tiles {
        let x = t.column as usize;
        let z = t.row as usize;
        if x >= g.width || z >= g.height || tiles[z * g.width + x].replace(t).is_some() {
            return None;
        }
    }
    tiles.into_iter().collect()
}

// Complete only a perimeter corner whose empty half has enclosing rock on
// both of its outside edges. Interior diagonal room openings stay authored.
fn sealed_corners(
    frame: &VisualWorldFrame,
    g: &GridGeometry,
    selected: &[usize],
) -> Vec<DiagonalCaveCornerPlacement> {
    let Some(size) = frame.source_map_size_core_tiles else {
        return Vec::new();
    };
    let Some(cells) = ordered_tiles(frame, g) else {
        return Vec::new();
    };
    let mut fill = vec![false; cells.len()];
    for &i in selected {
        fill[i] = true;
    }
    let solid = |x: usize, z: usize| {
        let i = z * g.width + x;
        let source_x = x as i32 + frame.grid_origin.x;
        let source_z = z as i32 + frame.grid_origin.y;
        source_x < 0
            || source_z < 0
            || source_x >= size.x as i32 * 2
            || source_z >= size.y as i32 * 2
            || fill[i]
            || shape_for_source(&cells[i].source).surface_height(g.tile_height)
                >= 32. * g.tile_height / SOURCE_TILE_HEIGHT
    };
    diagonal_cave_corner_placements(&cells, g)
        .into_iter()
        .filter(|p| {
            let x = p.column;
            let z = p.row;
            let sx = x as i32 + frame.grid_origin.x;
            let sz = z as i32 + frame.grid_origin.y;
            let perimeter = sx <= 4
                || sz <= 4
                || sx + 2 >= size.x as i32 * 2 - 4
                || sz + 2 >= size.y as i32 * 2 - 4;
            if !perimeter || z == 0 || !solid(x, z - 1) || !solid(x + 1, z - 1) {
                return false;
            }
            match p.corner {
                crate::cave::DiagonalCorner::SouthEast => {
                    x > 0 && solid(x - 1, z) && solid(x - 1, z + 1)
                }
                crate::cave::DiagonalCorner::SouthWest => {
                    x + 2 < g.width && solid(x + 2, z) && solid(x + 2, z + 1)
                }
            }
        })
        .collect()
}

// Fill the shallow strip behind a perimeter wall in its own cross-section.
// A distant entrance can connect this strip to interior floor, so global
// flood connectivity cannot identify the outside of the enclosing rock.
fn outer_ground_cells(frame: &VisualWorldFrame, g: &GridGeometry) -> Vec<usize> {
    let Some(size) = frame.source_map_size_core_tiles else {
        return Vec::new();
    };
    let width = size.x as i32 * 2;
    let height = size.y as i32 * 2;
    let Some(cells) = ordered_tiles(frame, g) else {
        return Vec::new();
    };
    let tiles: Vec<_> = cells.iter().copied().map(Some).collect();
    // A complete diagonal corner is a tall rock model even though its
    // individual source drawings also occur on the six-pixel shelf datum.
    // Use only validated quartets; never infer a wall from an isolated glyph.
    let mut corners = vec![false; tiles.len()];
    if let Some(cells) = tiles.iter().copied().collect::<Option<Vec<_>>>() {
        for corner in diagonal_cave_corner_placements(&cells, g) {
            for dz in 0..2 {
                for dx in 0..2 {
                    corners[(corner.row + dz) * g.width + corner.column + dx] = true;
                }
            }
        }
    }
    let index = |x: i32, z: i32| {
        let x = x - frame.grid_origin.x;
        let z = z - frame.grid_origin.y;
        (x >= 0 && z >= 0 && x < g.width as i32 && z < g.height as i32)
            .then(|| z as usize * g.width + x as usize)
    };
    let occupied: Vec<_> = frame
        .actors
        .iter()
        .map(|a| {
            [
                ((a.center.x - frame.center.x - g.origin_x) / g.tile_width).floor() as i32,
                ((frame.center.y - a.center.y - g.origin_z) / g.tile_height).floor() as i32,
            ]
        })
        .collect();
    let mut selected = vec![false; tiles.len()];
    let mut trace = |x: i32, z: i32, dx: i32, dz: i32| {
        let mut strip = Vec::new();
        // Union 1F's rounded perimeter ends sit up to two full source
        // blocks inward. Beyond the nearest block, require the exact outer
        // wall family and verified flat ground behind it.
        for depth in 0..12 {
            let Some(i) = index(x + dx * depth, z + dz * depth) else {
                break;
            };
            let Some(t) = tiles[i] else {
                break;
            };
            let local = [(i % g.width) as i32, (i / g.width) as i32];
            if occupied
                .iter()
                .any(|a| (a[0] - local[0]).abs() <= 1 && (a[1] - local[1]).abs() <= 1)
            {
                break;
            }
            // At recessed wall ends the source perimeter meets a shallow
            // shelf instead of a tall bank. Only the uninterrupted blank
            // outside strip is filled; interior uses of $09 stay floor.
            if matches!(t.source.tileset_id.as_ref(), "cave" | "dark_cave")
                && matches!(t.source.metatile_id, 0x04 | 0x05 | 0x23)
                && !strip.is_empty()
                && strip
                    .iter()
                    .all(|&j: &usize| tiles[j].unwrap().source.metatile_id == 0x09)
            {
                for i in strip {
                    selected[i] = true;
                }
                break;
            }
            let shape = shape_for_source(&t.source);
            let corner = corners[i]
                && strip.iter().all(|&j: &usize| {
                    matches!(
                        tiles[j].unwrap().source.metatile_id,
                        0x09 | 0x08 | 0x0a | 0x0c | 0x0d | 0x0e | 0x10 | 0x11 | 0x35
                    )
                });
            if corner
                || shape.surface_height(g.tile_height) >= 32. * g.tile_height / SOURCE_TILE_HEIGHT
            {
                if corner
                    || depth < 4
                    || matches!(t.source.tileset_id.as_ref(), "cave" | "dark_cave")
                        && matches!(
                            t.source.metatile_id,
                            0x05 | 0x08 | 0x0a | 0x0c | 0x0d | 0x0e
                        )
                {
                    for i in strip {
                        selected[i] = true;
                    }
                }
                break;
            }
            let ground = match t.source.tileset_id.as_ref() {
                "cave" | "dark_cave" => t.source.tile_index == 0x16,
                "ice_path" => t.source.tile_index == crate::ice_path::CAVE_GROUND_TILE,
                _ => false,
            };
            if !ground || !matches!(shape, CellShape::Flat | CellShape::PlaneAt { height: 0. }) {
                break;
            }
            strip.push(i);
        }
    };
    for z in 0..height {
        trace(0, z, 1, 0);
        trace(width - 1, z, -1, 0);
    }
    for x in 0..width {
        trace(x, 0, 0, 1);
        trace(x, height - 1, 0, -1);
    }
    selected
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| s.then_some(i))
        .collect()
}

fn point(p: Vec3, g: &GridGeometry, origin: [i32; 2], ice: bool, corners: &cave_cliff::RockCorners) -> Vec3 {
    if ice {
        modeled_dungeons::frozen_wall::frozen_point(p, g, origin)
    } else {
        corners.point(p, g, origin)
    }
}

pub(super) fn void_cells(frame:&VisualWorldFrame,g:&GridGeometry)->Vec<usize> {
    let Some(size)=frame.source_map_size_core_tiles else{return Vec::new();};
    let Some(cells)=ordered_tiles(frame,g) else{return Vec::new();};
    let mut result=Vec::new();
    for row in 0..g.height.saturating_sub(3) {for col in 0..g.width.saturating_sub(3) {
        let sx=col as i32+frame.grid_origin.x;let sz=row as i32+frame.grid_origin.y;
        if sx<0 || sz<0 || sx+4>size.x as i32*2 || sz+4>size.y as i32*2 {continue;}
        if (0..16).all(|j| {
            let s=&cells[(row+j/4)*g.width+col+j%4].source;
            matches!(s.tileset_id.as_ref(),"cave"|"dark_cave") && s.metatile_id==0x01
                && s.tile_index==0x10 && s.subtile_column as usize==j%4 && s.subtile_row as usize==j/4
        }) {result.extend((0..16).map(|j|(row+j/4)*g.width+col+j%4));}
    }}
    result
}

pub(super) fn build(frame: &VisualWorldFrame, g: &GridGeometry) -> SurfaceMeshData {
    let mut out = SurfaceMeshData::default();
    let Some(size) = frame.source_map_size_core_tiles else {
        return out;
    };
    let Some(source) = frame.tiles.first().map(|t| &t.source) else {
        return out;
    };
    let ice = source.tileset_id.as_ref() == "ice_path" && frame.map_id.as_ref() != "HallOfFame";
    if !crate::interior_cutaway::is_cave(frame) {
        return out;
    }
    let origin = [frame.grid_origin.x, frame.grid_origin.y];
    let ordered = ordered_tiles(frame, g);
    let corners = cave_cliff::RockCorners::new(ordered.as_deref().unwrap_or(&[]),g);
    let w = g.origin_x - origin[0] as f32 * g.tile_width;
    let n = g.origin_z - origin[1] as f32 * g.tile_height;
    let e = w + size.x as f32 * 2. * g.tile_width;
    let s = n + size.y as f32 * 2. * g.tile_height;
    let margin = 40.;
    let outer_w = w.min(g.origin_x) - margin * g.tile_width;
    let outer_n = n.min(g.origin_z) - margin * g.tile_height;
    let outer_e = e.max(g.origin_x + g.width as f32 * g.tile_width) + margin * g.tile_width;
    let outer_s = s.max(g.origin_z + g.height as f32 * g.tile_height) + margin * g.tile_height;
    let height = if ice {
        48.
    } else {
        crate::cave::rock_height(source)
    } * g.tile_height
        / SOURCE_TILE_HEIGHT;
    let tint = if ice {
        [0.62, 0.75, 0.77, 1.]
    } else if source.tileset_id.as_ref() == "dark_cave" {
        [0.31, 0.37, 0.39, 1.]
    } else {
        [0.34, 0.345, 0.31, 1.]
    };
    for [left, right, top, bottom] in [
        [outer_w, outer_e, outer_n, n],
        [outer_w, outer_e, s, outer_s],
        [outer_w, w, n, s],
        [e, outer_e, n, s],
    ] {
        // The cutaway can reveal the interior of this outer rock mass. Keep
        // a mineral foundation under it instead of exposing the clear void.
        // These rectangles lie strictly outside the authored map, so no
        // playable floor, opening, water or actor datum changes.
        let foundation = [Vec3::new(left,0.,top),Vec3::new(left,0.,bottom),
            Vec3::new(right,0.,bottom),Vec3::new(right,0.,top)];
        append_quad_colors(&mut out,foundation.map(|p|p.to_array()),[0.,1.,0.],
            foundation.map(|p|cave_cliff::stone_uv(p,g,origin,ice)),
            foundation.map(|p|cave_cliff::pigment(tint,p,g,origin,ice)));
        // Match the wall's quarter-tile knots along every map-facing edge.
        // Coarsen only beyond the joint so distant rock does not need the
        // full density of the playable enclosure.
        let axis = |from: f32, to: f32, low: f32, high: f32, tile: f32| {
            let mut coordinates = vec![from];
            let mut p = from;
            while p < to - tile * 0.001 {
                let fine = p >= low - 2. * tile && p <= high + 2. * tile;
                p = (p + tile * if fine { 0.25 } else { 1. }).min(to);
                coordinates.push(p);
            }
            coordinates
        };
        let xs = axis(left, right, w, e, g.tile_width);
        let zs = axis(top, bottom, n, s, g.tile_height);
        let columns = xs.len() - 1;
        let rows = zs.len() - 1;
        let at = |x: usize, z: usize| point(Vec3::new(xs[x], height, zs[z]), g, origin, ice, &corners);
        for z in 0..rows {
            for x in 0..columns {
                let p = [at(x, z), at(x, z + 1), at(x + 1, z + 1), at(x + 1, z)];
                let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
                let colors = p.map(|p| cave_cliff::pigment(tint, p, g, origin, ice));
                append_quad_colors(
                    &mut out,
                    p.map(|p| p.to_array()),
                    normal.to_array(),
                    p.map(|p| cave_cliff::rock_uv(p, g, origin, ice)),
                    colors,
                );
            }
        }
    }
    let voids=void_cells(frame,g);
    for &i in &voids {
        let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
        let base=[Vec3::new(w,0.,n),Vec3::new(w,0.,s),Vec3::new(e,0.,s),Vec3::new(e,0.,n)];
        append_quad_colors(&mut out,base.map(|p|p.to_array()),[0.,1.,0.],
            base.map(|p|cave_cliff::stone_uv(p,g,origin,ice)),
            base.map(|p|cave_cliff::pigment(tint,p,g,origin,ice)));
    }
    let mut selected = outer_ground_cells(frame, g);
    selected.extend(voids);selected.sort_unstable();selected.dedup();
    let mut filled = vec![false; g.width * g.height];
    for &i in &selected {
        filled[i] = true;
    }
    // The outer cap also needs a room-facing cross-section at the real map
    // boundary. Otherwise a low boundary tile exposes the black underside
    // of a floating cap. A viewport edge is never evidence of this boundary.
    if let Some(cells) = ordered.as_ref() {
        for (i, tile) in cells.iter().enumerate() {
            let sx = i as i32 % g.width as i32 + origin[0];
            let sz = i as i32 / g.width as i32 + origin[1];
            if sx < 0 || sz < 0 || sx >= size.x as i32 * 2 || sz >= size.y as i32 * 2 {
                continue;
            }
            if ice && modeled_dungeons::frozen_wall::complete_at(cells,g,i) {
                continue;
            }
            let low = if filled[i] {
                height
            } else {
                shape_for_source(&tile.source).surface_height(g.tile_height)
            };
            if low >= height {
                continue;
            }
            let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
            for (boundary, neighbor, face) in [
                (
                    sz == 0,
                    i.checked_sub(g.width),
                    [[w, low, n], [e, low, n], [e, height, n], [w, height, n]],
                ),
                (
                    sx == size.x as i32 * 2 - 1,
                    (i%g.width+1<g.width).then_some(i+1),
                    [[e, low, n], [e, low, s], [e, height, s], [e, height, n]],
                ),
                (
                    sz == size.y as i32 * 2 - 1,
                    (i/g.width+1<g.height).then_some(i+g.width),
                    [[e, low, s], [w, low, s], [w, height, s], [e, height, s]],
                ),
                (
                    sx == 0,
                    (i%g.width>0).then(||i-1),
                    [[w, low, s], [w, low, n], [w, height, n], [w, height, s]],
                ),
            ] {
                if boundary {
                    // A complete frozen mass in the retained source halo
                    // already supplies this room-facing boundary wall too.
                    if ice && neighbor.is_some_and(|j|
                        modeled_dungeons::frozen_wall::complete_at(cells,g,j)) {
                        continue;
                    }
                    cave_cliff::quad_with_point(
                        &mut out,
                        face,
                        tint,
                        g,
                        origin,
                        |p, g, origin| point(p, g, origin, ice, &corners),
                        ice,
                    );
                }
            }
        }
    }
    for &index in &selected {
        let (w, e, n, s) = g.bounds(index % g.width, index / g.width);
        cave_cliff::quad_with_point(
            &mut out,
            [
                [w, height, n],
                [w, height, s],
                [e, height, s],
                [e, height, n],
            ],
            tint,
            g,
            origin,
            |p, g, origin| point(p, g, origin, ice, &corners),
            ice,
        );
    }
    // Close the newly filled exterior cap at changes in height. These faces
    // belong to presentation-only outer geology, never actor footing.
    if let Some(cells) = ordered {
        let mut corner_cells = vec![false; cells.len()];
        for corner in diagonal_cave_corner_placements(&cells, g) {
            for dz in 0..2 {
                for dx in 0..2 {
                    corner_cells[(corner.row + dz) * g.width + corner.column + dx] = true;
                }
            }
        }
        for &i in &selected {
            let x = i % g.width;
            let z = i / g.width;
            let (w, e, n, s) = g.bounds(x, z);
            for (side, neighbor) in [
                z.checked_sub(1).map(|z| z * g.width + x),
                (x + 1 < g.width).then_some(i + 1),
                (z + 1 < g.height).then_some(i + g.width),
                x.checked_sub(1).map(|x| z * g.width + x),
            ]
            .into_iter()
            .enumerate()
            {
                let Some(j) = neighbor else { continue };
                if filled[j] {
                    continue;
                }
                let sx = (j % g.width) as i32 + origin[0];
                let sz = (j / g.width) as i32 + origin[1];
                if sx < 0 || sz < 0 || sx >= size.x as i32 * 2 || sz >= size.y as i32 * 2 {
                    continue;
                }
                if corner_cells[j] {
                    continue;
                }
                // Complete frozen masses are modeled at the same enclosing
                // datum as this outer cap. Their raw sixteen-pixel source
                // shape is not the rendered wall height. Closing above that
                // raw height duplicates the existing wall on the same plane,
                // producing interleaved pale and blue triangles.
                if ice && modeled_dungeons::frozen_wall::complete_at(&cells,g,j) {
                    continue;
                }
                let low = shape_for_source(&cells[j].source).surface_height(g.tile_height);
                if low >= height {
                    continue;
                }
                let p = match side {
                    0 => [[e, low, n], [w, low, n], [w, height, n], [e, height, n]],
                    1 => [[e, low, s], [e, low, n], [e, height, n], [e, height, s]],
                    2 => [[w, low, s], [e, low, s], [e, height, s], [w, height, s]],
                    _ => [[w, low, n], [w, low, s], [w, height, s], [w, height, n]],
                };
                cave_cliff::quad_with_point(
                    &mut out,
                    p,
                    tint,
                    g,
                    origin,
                    |p, g, origin| point(p, g, origin, ice, &corners),
                    ice,
                );
            }
        }
    }
    for corner in sealed_corners(frame, g, &selected) {
        let (w, _, n, _) = g.bounds(corner.column, corner.row);
        let (_, e, _, s) = g.bounds(corner.column + 1, corner.row + 1);
        let mut p = match corner.corner {
            crate::cave::DiagonalCorner::SouthEast => {
                [[w, height, n], [w, height, s], [e, height, n]]
            }
            crate::cave::DiagonalCorner::SouthWest => {
                [[w, height, n], [e, height, s], [e, height, n]]
            }
        };
        if (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
            .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
            .y
            < 0.
        {
            p.swap(1, 2)
        }
        cave_cliff::cap_triangle_with_point(&mut out, p, tint, g, origin,
            |p,g,o| if ice { cave_cliff::rock_point(p,g,o) } else { corners.point(p,g,o) });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2, UVec2, Vec2};
    #[test]
    fn complete_frozen_halo_already_closes_the_low_map_boundary() {
        let g=GridGeometry {width:12,height:12,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut frame=VisualWorldFrame {map_id:"IcePathB2FBlackthornSide".into(),
            grid_origin:IVec2::splat(-4),source_map_size_core_tiles:Some(UVec2::splat(2)),
            ..Default::default()};
        frame.tiles=(0..144).map(|i| {
            let (x,z)=(i%12,i/12);let ground=(4..8).contains(&x)&&(4..8).contains(&z);
            VisualTile {column:x as u32,row:z as u32,texture:Default::default(),
                animation_frames:None,priority:false,
                source:VisualTileSource {tileset_id:"ice_path".into(),metatile_id:if ground {2}else{0x19},
                    subtile_column:(x%4) as u8,subtile_row:(z%4) as u8,
                    tile_index:if ground {crate::ice_path::CAVE_GROUND_TILE}
                        else {0x84+(x%4) as u16+(z%4) as u16*0x10}}}
        }).collect();
        let closed=build(&frame,&g);
        let mut unknown=frame.clone();unknown.tiles.truncate(1);
        let outside=build(&unknown,&g);
        assert_eq!(closed.positions,outside.positions,
            "the retained native halo wall must not be drawn twice");
        assert_eq!(closed.indices,outside.indices);
        frame.tiles[3*12+6].source.tile_index=crate::ice_path::CAVE_GROUND_TILE;
        assert!(build(&frame,&g).positions.len()>outside.positions.len(),
            "a clipped or changed halo cannot prove an existing enclosing wall");
    }
    #[test]
    fn complete_frozen_boundary_does_not_gain_a_duplicate_outer_wall() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut frame=VisualWorldFrame {map_id:"IcePathB2FBlackthornSide".into(),
            source_map_size_core_tiles:Some(UVec2::splat(2)),..Default::default()};
        frame.tiles=(0..16).map(|i|VisualTile {column:(i%4) as u32,row:(i/4) as u32,
            texture:Default::default(),animation_frames:None,priority:false,
            source:VisualTileSource {tileset_id:"ice_path".into(),metatile_id:0x19,
                subtile_column:(i%4) as u8,subtile_row:(i/4) as u8,
                tile_index:0x84+(i%4) as u16+(i/4) as u16*0x10}}).collect();
        let closed=build(&frame,&g);
        let mut halo=frame.clone();halo.tiles.truncate(1);
        let outside=build(&halo,&g);
        assert_eq!(closed.positions,outside.positions,
            "the complete native frozen wall already closes the outer cap");
        assert_eq!(closed.indices,outside.indices);
        frame.tiles[0].source.metatile_id=2;
        frame.tiles[0].source.tile_index=crate::ice_path::CAVE_GROUND_TILE;
        assert!(build(&frame,&g).positions.len()>outside.positions.len(),
            "unknown boundary art still needs its enclosing cross-section");
    }
    #[test]
    fn complete_authored_void_blocks_have_rock_and_an_opaque_base() {
        let g=GridGeometry{width:6,height:6,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut frame=VisualWorldFrame{map_id:"SilverCaveRoom3".into(),
            source_map_size_core_tiles:Some(UVec2::splat(3)),..Default::default()};
        frame.tiles=(0..36).map(|i| {
            let (x,z)=(i%6,i/6);let void=(1..5).contains(&x)&&(1..5).contains(&z);
            VisualTile{column:x as u32,row:z as u32,texture:Default::default(),animation_frames:None,priority:false,
                source:VisualTileSource{tileset_id:"cave".into(),metatile_id:if void {1}else{9},
                    tile_index:if void {0x10}else{0x16},subtile_column:if void {(x-1) as u8}else{0},
                    subtile_row:if void {(z-1) as u8}else{0}}}
        }).collect();
        let selected=void_cells(&frame,&g);assert_eq!(selected.len(),16);
        frame.active=true;frame.tile_size=Vec2::splat(8.);
        frame.grid_size=UVec2::splat(6);frame.viewport_size=Vec2::splat(48.);
        let mut images=Assets::<Image>::default();frame.map_texture=images.add(Image::default());
        for tile in &mut frame.tiles {tile.texture=frame.map_texture.clone();}
        let scene=build_terrain_mesh(&frame).unwrap();
        assert_eq!(scene.footing_heights,vec![0.;36],
            "exterior rock is presentation geometry, never a new actor datum");
        assert_eq!(scene.authored_cells.iter().filter(|&&cell|cell==Some("cave:exterior-rock")).count(),16);
        for ids in scene.textured.indices.chunks_exact(3) {
            let uv=ids.iter().fold(Vec2::ZERO,|sum,&i|sum+Vec2::from_array(scene.textured.uvs[i as usize]))/3.;
            let sampled=(uv.y*6.).floor() as usize*6+(uv.x*6.).floor() as usize;
            assert!(!selected.contains(&sampled),"the old black plane remains over the mineral base");
        }
        let out=build(&frame,&g);
        assert!(out.positions.iter().zip(&out.uvs).filter(|(p,uv)|
            p[0]>=8. && p[0]<=40. && p[2]>=8. && p[2]<=40. && (uv[1]/100000.).round()==6.)
            .any(|(p,_)|p[1]==0.),"authored void must retain mineral rock below the cutaway");
        assert!(out.positions.iter().any(|p|p[0]>8. && p[0]<40. && p[2]>8. && p[2]<40. && p[1]>24.));
        frame.tiles[8].source.tile_index=0x16;assert!(void_cells(&frame,&g).is_empty());
        frame.tiles[8].source.tile_index=0x10;frame.source_map_size_core_tiles=None;
        assert!(void_cells(&frame,&g).is_empty());
    }
    #[test]
    fn outer_cap_closes_to_boundary_floor_but_never_to_a_viewport_edge() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        for (tileset, floor) in [("cave", 0x16), ("ice_path", 0x19)] {
            let mut frame = VisualWorldFrame::default();
            frame.source_map_size_core_tiles = Some(UVec2::ONE);
            frame.tiles = (0..4)
                .map(|i| VisualTile {
                    column: i % 2,
                    row: i / 2,
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: tileset.into(),
                        metatile_id: 0x02,
                        subtile_column: (i % 2) as u8,
                        subtile_row: (i / 2) as u8,
                        tile_index: floor,
                    },
                })
                .collect();
            let out = build(&frame, &g);
            for corner in [[0., 0., 0.], [16., 0., 0.], [0., 0., 16.], [16., 0., 16.]] {
                assert!(
                    out.positions.contains(&corner),
                    "missing rock base {tileset}: {corner:?}"
                );
            }
            assert!(out.normals.iter().all(|n| Vec3::from_array(*n).is_finite()));
            assert!(
                frame
                    .tiles
                    .iter()
                    .all(|t| shape_for_source(&t.source).surface_height(8.) == 0.)
            );
            frame.grid_origin = IVec2::splat(10);
            frame.source_map_size_core_tiles = Some(UVec2::splat(50));
            let interior = build(&frame, &g);
            assert!(
                !interior.positions.iter().zip(&interior.normals)
                    .any(|(p,n)|p[1]==0. && n[1].abs()<0.99),
                "viewport became a wall"
            );
            frame.grid_origin = IVec2::ZERO;
            frame.source_map_size_core_tiles = Some(UVec2::ONE);
            frame.tiles.pop();
            let unknown=build(&frame,&g);
            assert!(
                !unknown.positions.iter().zip(&unknown.normals)
                    .any(|(p,n)|p[1]==0. && n[1].abs()<0.99),
                "unknown boundary inferred"
            );
        }
    }
    #[test]
    fn perimeter_cross_sections_preserve_openings_even_when_outside_floor_connects() {
        let g = GridGeometry {
            width: 12,
            height: 12,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut frame = VisualWorldFrame::default();
        frame.source_map_size_core_tiles = Some(UVec2::new(6, 6));
        for row in 0..12 {
            for column in 0..12 {
                let wall = column == 2;
                frame.tiles.push(VisualTile {
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: "cave".into(),
                        metatile_id: if wall { 0x2e } else { 0 },
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: if wall { 0x26 } else { 0x16 },
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let cells = outer_ground_cells(&frame, &g);
        assert_eq!(cells.len(), 24);
        assert!(cells.iter().all(|i| i % 12 < 2));
        frame.tiles.reverse();
        assert_eq!(outer_ground_cells(&frame, &g), cells);
        frame.tiles.reverse();
        // Only the opening stays floor; outside rock continues beside intact walls.
        frame.tiles[6 * 12 + 2].source.metatile_id = 0;
        frame.tiles[6 * 12 + 2].source.tile_index = 0x16;
        let cells = outer_ground_cells(&frame, &g);
        assert_eq!(cells.len(), 22);
        assert!(!cells.contains(&(6 * 12)) && !cells.contains(&(6 * 12 + 1)));
        // A shallow walking ledge never establishes an enclosing rock wall.
        for t in &mut frame.tiles {
            if t.column == 2 {
                t.source.metatile_id = 0x3b;
                t.source.subtile_column = 0;
                t.source.tile_index = 0x15;
            }
        }
        assert!(outer_ground_cells(&frame, &g).is_empty());
    }
    #[test]
    fn recessed_outer_wall_ends_close_only_authored_blank_continuation() {
        let g = GridGeometry {
            width: 24,
            height: 12,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut frame = VisualWorldFrame::default();
        frame.source_map_size_core_tiles = Some(UVec2::new(12, 6));
        for row in 0..12 {
            for column in 0..24 {
                frame.tiles.push(VisualTile {
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: "cave".into(),
                        metatile_id: if column == 8 { 0x0d } else { 0 },
                        subtile_column: (column % 4) as u8,
                        subtile_row: 0,
                        tile_index: 0x16,
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let cells = outer_ground_cells(&frame, &g);
        assert_eq!(cells.len(), 96);
        assert!(cells.iter().all(|i| i % 24 < 8));
        // A distant interior rock mass is not evidence of an outside wall.
        for t in &mut frame.tiles {
            if t.column == 8 {
                t.source.metatile_id = 0x2e;
                t.source.tile_index = 0x26;
            }
        }
        assert!(outer_ground_cells(&frame, &g).is_empty());
    }

    #[test]
    fn recessed_shelf_closes_exterior_blank_floor_without_raising_interior_floor() {
        let g = GridGeometry {
            width: 12,
            height: 12,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut frame = VisualWorldFrame::default();
        frame.source_map_size_core_tiles = Some(UVec2::new(6, 6));
        for row in 0..12 {
            for column in 0..12 {
                frame.tiles.push(VisualTile {
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: "cave".into(),
                        metatile_id: if column < 4 || (6..8).contains(&column) {
                            0x09
                        } else if column == 4 {
                            0x05
                        } else {
                            0
                        },
                        subtile_column: (column % 4) as u8,
                        subtile_row: 1,
                        tile_index: 0x16,
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let before: Vec<_> = frame
            .tiles
            .iter()
            .map(|t| shape_for_source(&t.source).surface_height(8.))
            .collect();
        let selected = outer_ground_cells(&frame, &g);
        assert_eq!(selected.len(), 48);
        assert!(selected.iter().all(|i| i % 12 < 4));
        let out = build(&frame, &g);
        assert!(out.positions.contains(&[32., 0., 8.]));
        assert!(out.normals.iter().all(|n| Vec3::from_array(*n).is_finite()));
        assert_eq!(
            before,
            frame
                .tiles
                .iter()
                .map(|t| shape_for_source(&t.source).surface_height(8.))
                .collect::<Vec<_>>()
        );
        assert_eq!(before[6], 0.);
    }

    #[test]
    fn complete_diagonal_corner_closes_only_its_verified_outer_ground() {
        let g = GridGeometry {
            width: 24,
            height: 12,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut frame = VisualWorldFrame::default();
        frame.source_map_size_core_tiles = Some(UVec2::new(12, 6));
        for row in 0..12 {
            for column in 0..24 {
                let corner = (6..8).contains(&column) && (2..4).contains(&row);
                frame.tiles.push(VisualTile {
                    column,
                    row,
                    source: VisualTileSource {
                        tileset_id: "cave".into(),
                        metatile_id: if corner {
                            0x10
                        } else if column < 6 && (2..4).contains(&row) {
                            0x10
                        } else {
                            0
                        },
                        subtile_column: if corner {
                            (column - 4) as u8
                        } else {
                            (column % 4) as u8
                        },
                        subtile_row: if corner { row as u8 } else { (row % 4) as u8 },
                        tile_index: if corner {
                            [[0x0a, 0x26], [0x17, 0x0a]][(row - 2) as usize][(column - 6) as usize]
                        } else {
                            0x16
                        },
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let cells = outer_ground_cells(&frame, &g);
        assert_eq!(cells.len(), 12);
        assert!(
            cells
                .iter()
                .all(|&i| i % 24 < 6 && (2..4).contains(&(i / 24)))
        );
        // Room-side floor and the diagonal opening remain at their own datum.
        assert!(!cells.contains(&(2 * 24 + 6)));
        assert!(sealed_corners(&frame, &g, &cells).is_empty());
        // Both outer sides must be solid; a single adjoining bank is not
        // evidence to close a genuine diagonal opening.
        for x in 6..8 {
            frame.tiles[24 + x].source.metatile_id = 0x2e;
            frame.tiles[24 + x].source.tile_index = 0x26;
        }
        assert_eq!(sealed_corners(&frame, &g, &cells).len(), 1);
        frame.grid_origin = IVec2::new(20, 20);
        frame.source_map_size_core_tiles = Some(UVec2::new(50, 50));
        assert!(sealed_corners(&frame, &g, &cells).is_empty());
        frame.grid_origin = IVec2::ZERO;
        frame.source_map_size_core_tiles = Some(UVec2::new(12, 6));
        for x in 6..8 {
            frame.tiles[24 + x].source.metatile_id = 0;
            frame.tiles[24 + x].source.tile_index = 0x16;
        }
        frame.tiles[3 * 24 + 7].source.tile_index = 0x16;
        assert!(outer_ground_cells(&frame, &g).is_empty());
    }

    #[test]
    fn enclosure_requires_real_bounds_and_leaves_the_playable_map_open() {
        let g = GridGeometry {
            width: 40,
            height: 38,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -160.,
            origin_z: -152.,
        };
        let mut frame = VisualWorldFrame::default();
        frame.map_id = "UnionCaveB2F".into();
        frame.grid_origin = IVec2::new(-8, -8);
        frame.tiles.push(VisualTile {
            column: 0,
            row: 0,
            source: VisualTileSource {
                tileset_id: "cave".into(),
                metatile_id: 0,
                subtile_column: 0,
                subtile_row: 0,
                tile_index: 0x16,
            },
            texture: Handle::default(),
            animation_frames: None,
            priority: false,
        });
        assert!(build(&frame, &g).positions.is_empty());
        frame.source_map_size_core_tiles = Some(UVec2::new(12, 11));
        let out = build(&frame, &g);
        assert!(!out.indices.is_empty());
        let w = -96.;
        let n = -88.;
        let e = 96.;
        let s = 88.;
        // Relief can cross an edge by less than a tile, but no interior floor
        // or actor support is generated by this presentation-only rock.
        assert!(
            out.positions
                .iter()
                .all(|p| p[0] <= w + 8. || p[0] >= e - 8. || p[2] <= n + 8. || p[2] >= s - 8.)
        );
        assert!(out.positions.iter().zip(&out.uvs).all(|(p,uv)|
            if (uv[1]/100000.).round()==6. {p[1]==0. && (p[0]<=w || p[0]>=e || p[2]<=n || p[2]>=s)}
            else {p[1]>24.}));
        assert_eq!(out.uvs.iter().filter(|uv|(uv[1]/100000.).round()==6.).count(),16,
            "all four exterior rock regions need an opaque foundation");
        assert!(out.normals.iter().all(|n| Vec3::from_array(*n).is_finite()));
        frame.tiles[0].source.tileset_id = "johto".into();
        assert!(build(&frame, &g).positions.is_empty());
        frame.map_id = "DragonsDenB1F".into();
        let den = build(&frame, &g);
        assert!(!den.indices.is_empty());
        assert!(den.positions.iter().all(|p| p[0] <= w + 8. || p[0] >= e - 8.
            || p[2] <= n + 8. || p[2] >= s - 8.));
        assert!(den.positions.iter().zip(&den.uvs).all(|(p,uv)|
            if (uv[1]/100000.).round()==6. {p[1]==0.} else {p[1]>24.}));
        frame.map_id="IcePath1F".into();frame.tiles[0].source.tileset_id="ice_path".into();
        let ice=build(&frame,&g);
        assert_eq!(ice.positions.iter().zip(&ice.uvs)
            .filter(|(p,uv)|p[1]==0. && (uv[1]/100000.).round()==7.).count(),16);
    }
}
