//! Flat visual continuation beyond the host's real connected-map halo.
//!
//! The host publishes actual neighboring map data around the viewport. A
//! pitched camera can still see beyond that finite grid, however. This apron
//! continues verified ground or open coastal water beneath the real mesh.
//! It is presentation-only: it creates no walls, footing, collision, or
//! inferred topology.

use super::*;

// The real host halo already contributes neighboring source tiles. Keep a
// much larger presentation-only apron beyond it so perspective views and
// future camera yaw cannot reach the clear color in any direction.
const BACKGROUND_APRON_TILES: isize = 96;
#[cfg(test)]
const METATILE_PERIOD: usize = 4;
pub(super) const BACKGROUND_HEIGHT: f32 = -0.04;

pub(super) fn append_repeating_background_apron(
    mesh: &mut TerrainMeshData,
    geometry: &GridGeometry,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    map: &str,
) -> SurfaceMeshData {
    // Small synthetic grids are used throughout the mesher's exact-geometry
    // tests. The real optional host frame includes its connected-map halo and
    // is at least 40x38 cells.
    if geometry.width < 40 || geometry.height < 38 {
        return SurfaceMeshData::default();
    }
    let width = geometry.width as isize;
    let height = geometry.height as isize;
    let water = dominant_coastal_water_source(geometry.width, geometry.height, cells, shapes, map);
    let edges = water.map(|p| [Some(p); 4]).unwrap_or_else(||
        coastal_edge_water_sources(geometry.width, geometry.height, cells, shapes, map));
    if edges.iter().any(Option::is_some) {
        return append_coastal_background(mesh, geometry, cells, shapes, edges);
    }
    let Some((background_column, background_row)) =
        dominant_flat_source(geometry.width, geometry.height, cells, shapes)
    else {
        return SurfaceMeshData::default();
    };

    if mesh.tree_cache.is_some() {
        // Four planar strips avoid one quad per repeated tile.
        let apron = BACKGROUND_APRON_TILES;
        let mut surface = SurfaceMeshData::default();
        for [left, right, top, bottom] in [
            [-apron, width + apron, -apron, 0],
            [-apron, width + apron, height, height + apron],
            [-apron, 0, 0, height],
            [width, width + apron, 0, height],
        ] {
            append_top(
                &mut surface,
                [
                    geometry.origin_x + left as f32 * geometry.tile_width,
                    geometry.origin_x + right as f32 * geometry.tile_width,
                    geometry.origin_z + top as f32 * geometry.tile_height,
                    geometry.origin_z + bottom as f32 * geometry.tile_height,
                ],
                BACKGROUND_HEIGHT,
                (left as f32, right as f32, top as f32, bottom as f32),
            );
        }
        mesh.background = Some(RepeatingBackground {
            mesh: surface,
            source_cell: [background_column, background_row],
            source: cells[background_row * geometry.width + background_column]
                .source
                .clone(),
            texture: cells[background_row * geometry.width + background_column]
                .texture
                .clone(),
        });
        return SurfaceMeshData::default();
    }

    for row in -BACKGROUND_APRON_TILES..height + BACKGROUND_APRON_TILES {
        for column in -BACKGROUND_APRON_TILES..width + BACKGROUND_APRON_TILES {
            if (0..width).contains(&column) && (0..height).contains(&row) {
                continue;
            }

            let x0 = geometry.origin_x + column as f32 * geometry.tile_width;
            let z0 = geometry.origin_z + row as f32 * geometry.tile_height;
            append_top(
                &mut mesh.textured,
                [x0, x0 + geometry.tile_width, z0, z0 + geometry.tile_height],
                BACKGROUND_HEIGHT,
                geometry.uv(background_column, background_row),
            );
        }
    }
    SurfaceMeshData::default()
}

fn coastal_continuation_map(map: &str) -> bool {
    crate::environment_fx::coastal_map(map)
        || matches!(map, "Route12" | "Route13" | "Route14" | "Route15"
            | "CinnabarIsland" | "VermilionCity" | "PalletTown"
            | "OlivinePort" | "VermilionPort")
}

/// A land-heavy coastal frame can still have a complete open-water edge.
/// Keep the other edges dry; neither an inland pond nor a still drawing is
/// evidence that the landscape beyond the source grid should become sea.
fn coastal_edge_water_sources(width: usize, height: usize, cells: &[&VisualTile],
    shapes: &[CellShape], map: &str) -> [Option<(usize, usize)>; 4] {
    if !coastal_continuation_map(map) { return [None; 4]; }
    std::array::from_fn(|side| {
        let length = if side < 2 { width } else { height };
        let index = |j| match side {
            0 => j, 1 => (height-1)*width+j,
            2 => j*width, _ => j*width+width-1,
        };
        let wet = |i: usize| !cells[i].priority && shapes[i] == CellShape::Water
            && crate::cave_water::role_for_map(cells[i], map) == [255, 0];
        if (0..length).filter(|&j| wet(index(j))).count()*2 <= length { return None; }
        (0..length).map(index).find(|&i| wet(i)).map(|i| (i%width,i/width))
    })
}

fn append_coastal_background(mesh: &mut TerrainMeshData, g: &GridGeometry,
    cells: &[&VisualTile], shapes: &[CellShape], edges: [Option<(usize, usize)>; 4])
    -> SurfaceMeshData {
    let (width,height,apron) = (g.width as isize,g.height as isize,BACKGROUND_APRON_TILES);
    let dry = (!edges.iter().all(Option::is_some)).then(||
        dominant_flat_source(g.width,g.height,cells,shapes)).flatten();
    let mut ground = SurfaceMeshData::default();
    let mut water = SurfaceMeshData::default();
    let mut append = |bounds: [isize;4], donor: Option<(usize,usize)>| {
        let [left,right,top,bottom] = bounds;
        let positions = [g.origin_x+left as f32*g.tile_width,
            g.origin_x+right as f32*g.tile_width,g.origin_z+top as f32*g.tile_height,
            g.origin_z+bottom as f32*g.tile_height];
        if let Some((x,z)) = donor {
            // Native live-water identity, continuous world-space optics.
            let (u0,u1,v0,v1) = g.uv(x,z);
            append_top(&mut water,positions,CellShape::Water.surface_height(g.tile_height),
                ((u0+u1)*0.5,(u0+u1)*0.5,(v0+v1)*0.5,(v0+v1)*0.5));
        } else if dry.is_some() {
            append_top(&mut ground,positions,BACKGROUND_HEIGHT,
                (left as f32,right as f32,top as f32,bottom as f32));
        }
    };
    if edges.iter().all(Option::is_some) {
        for (bounds,donor) in [([-apron,width+apron,-apron,0],edges[0]),
            ([-apron,width+apron,height,height+apron],edges[1]),
            ([-apron,0,0,height],edges[2]),([width,width+apron,0,height],edges[3])] {
            append(bounds,donor);
        }
    } else {
        // Eight disjoint sectors close the corners without laying a dry strip
        // beyond a sea edge. All vertices stay outside the finite source grid.
        let xs = [-apron,0,width,width+apron];
        let zs = [-apron,0,height,height+apron];
        for z in 0..3 { for x in 0..3 {
            if x==1 && z==1 { continue; }
            let horizontal = match z {0=>edges[0],2=>edges[1],_=>None};
            let vertical = match x {0=>edges[2],2=>edges[3],_=>None};
            append([xs[x],xs[x+1],zs[z],zs[z+1]],horizontal.or(vertical));
        }}
    }
    if let Some((x,z)) = dry && !ground.positions.is_empty() {
        mesh.background = Some(RepeatingBackground {mesh:ground, source_cell:[x,z],
            source:cells[z*g.width+x].source.clone(),texture:cells[z*g.width+x].texture.clone()});
    }
    water
}

fn dominant_flat_source(
    width: usize,
    height: usize,
    cells: &[&VisualTile],
    shapes: &[CellShape],
) -> Option<(usize, usize)> {
    let mut counts: HashMap<(u16, u8, u8, u16), (usize, usize, usize)> = HashMap::new();
    for row in 0..height {
        for column in 0..width {
            let index = row * width + column;
            if cells[index].priority
                || !matches!(
                    shapes[index],
                    CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
                )
            {
                continue;
            }
            let source = &cells[index].source;
            // Object masking changes its classification to Flat but retains
            // the original atlas pixels. That is never a ground sample.
            if source.tile_index == u16::MAX || super::kanto_shore::source_bank(source)
                || !matches!(new_bark::ground_material(source), Some(new_bark::GroundMaterial::Lawn
                    | new_bark::GroundMaterial::Path | new_bark::GroundMaterial::Pavers)) {
                // Native shore corners are flat drawing cells, not a ground
                // field. Repeating one stamps tiny island icons over the horizon.
                continue;
            }
            let key = (
                source.metatile_id,
                source.subtile_column,
                source.subtile_row,
                source.tile_index,
            );
            let entry = counts.entry(key).or_insert((0, column, row));
            entry.0 += 1;
        }
    }
    counts
        .into_values()
        // Prefer the earliest source on ties, independently of HashMap order.
        .max_by_key(|(count, column, row)| (*count, std::cmp::Reverse(row * width + column)))
        .map(|(_, column, row)| (column, row))
}

fn dominant_coastal_water_source(
    width: usize, height: usize, cells: &[&VisualTile], shapes: &[CellShape], map: &str,
) -> Option<(usize, usize)> {
    if !coastal_continuation_map(map) {
        return None;
    }
    let wet = |i: usize| !cells[i].priority && shapes[i] == CellShape::Water
        && crate::cave_water::role_for_map(cells[i], map) == [255, 0];
    // An isolated pond or an unverified still drawing cannot turn the distant
    // landscape into ocean. The frame and its outer edge must both be wet.
    if (0..cells.len()).filter(|&i| wet(i)).count() * 2 <= cells.len() { return None; }
    let edge = |i: usize| i < width || i >= width * (height - 1)
        || i % width == 0 || i % width == width - 1;
    let edge_count = 2 * width + 2 * height - 4;
    if (0..cells.len()).filter(|&i| edge(i) && wet(i)).count() * 2 < edge_count { return None; }
    (0..cells.len()).find(|&i| wet(i)).map(|i| (i % width, i / width))
}

/// Publish after source-cell fitting and before the shared final waterline fit.
/// These faces never acquire support, collision or source-map ownership.
pub(super) fn finish_water_apron(mesh: &mut TerrainMeshData, water: SurfaceMeshData) {
    for base in (0..water.positions.len()).step_by(4) {
        append_quad_colors(&mut mesh.textured,
            water.positions[base..base + 4].try_into().unwrap(),
            water.normals[base],water.uvs[base..base + 4].try_into().unwrap(),
            water.colors[base..base + 4].try_into().unwrap());
    }
}

#[cfg(test)]
fn nearest_flat_source(
    nominal_column: usize,
    nominal_row: usize,
    width: usize,
    height: usize,
    cells: &[&VisualTile],
    shapes: &[CellShape],
) -> Option<(usize, usize)> {
    let is_flat = |column: usize, row: usize| {
        let index = row * width + column;
        !cells[index].priority
            && matches!(
                shapes[index],
                CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
            )
    };
    if is_flat(nominal_column, nominal_row) {
        return Some((nominal_column, nominal_row));
    }

    // Search outward for actual ground art. This deliberately refuses tree,
    // building, rock, water, ledge, and cutout artwork rather than flattening
    // an object into the distant floor.
    for radius in 1..width.max(height) {
        let min_column = nominal_column.saturating_sub(radius);
        let max_column = (nominal_column + radius).min(width - 1);
        let min_row = nominal_row.saturating_sub(radius);
        let max_row = (nominal_row + radius).min(height - 1);
        for column in min_column..=max_column {
            for row in [min_row, max_row] {
                if is_flat(column, row) {
                    return Some((column, row));
                }
            }
        }
        for row in min_row.saturating_add(1)..max_row {
            for column in [min_column, max_column] {
                if is_flat(column, row) {
                    return Some((column, row));
                }
            }
        }
    }
    None
}

#[cfg(test)]
fn repeated_edge_coordinate(coordinate: isize, length: usize) -> usize {
    let period = length.min(METATILE_PERIOD);
    debug_assert!(period > 0);
    if coordinate < 0 {
        coordinate.rem_euclid(period as isize) as usize
    } else if coordinate >= length as isize {
        length - period + (coordinate - length as isize) as usize % period
    } else {
        coordinate as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec2;

    fn mixed_coast(side: usize, map: &str) -> (VisualWorldFrame, Vec<CellShape>) {
        let mut frame = test_frame_with_grid(40,48);
        frame.map_id = map.into();
        let next = Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData);
        let mut shapes = vec![CellShape::Flat;40*48];
        for tile in &mut frame.tiles {
            let (x,z) = (tile.column,tile.row);
            let water = match side {0=>z<12,1=>z>=36,2=>x<12,_=>x>=28};
            tile.source = super::super::tests::source_with_tile(if water {0x35} else {1},
                (x%4) as u8,(z%4) as u8,if water {0x14} else {5});
            if water {
                shapes[z as usize*40+x as usize] = CellShape::Water;
                tile.animation_frames = Some(vec![tile.texture.clone(),next.clone()].into());
            }
        }
        (frame,shapes)
    }

    #[test]
    fn mixed_coasts_continue_only_proven_sea_edges_and_close_their_outer_corners() {
        for map in ["Route12","Route13","Route14","Route15"] {
            for side in 0..4 { for scale in [1.,4.] { for instanced in [false,true] {
                let (frame,shapes) = mixed_coast(side,map);
                let before = frame.clone();
                let cells = frame.tiles.iter().collect::<Vec<_>>();
                let g = GridGeometry {width:40,height:48,tile_width:8.*scale,tile_height:8.*scale,
                    origin_x:-160.*scale,origin_z:-192.*scale};
                assert!(dominant_coastal_water_source(40,48,&cells,&shapes,map).is_none());
                let edges = coastal_edge_water_sources(40,48,&cells,&shapes,map);
                assert_eq!(edges.map(|s|s.is_some()),std::array::from_fn(|s|s==side));
                let mut mesh = TerrainMeshData {tree_cache:instanced.then(HashMap::new),
                    footing_heights:shapes.iter().map(|s|s.surface_height(g.tile_height)).collect(),
                    authored_cells:vec![None;cells.len()],..Default::default()};
                let mut control = mesh.clone();
                let water = append_repeating_background_apron(&mut mesh,&g,&cells,&shapes,map);
                assert_eq!(water.quad_count(),3,"sea-facing strip and its two corners");
                let ground = &mesh.background.as_ref().unwrap().mesh;
                assert_eq!(ground.quad_count(),5);
                assert!(ground.positions.iter().all(|p|p[1]==BACKGROUND_HEIGHT));
                let xs = [-96.,0.,40.,136.];
                let zs = [-96.,0.,48.,144.];
                for z in 0..3 { for x in 0..3 {
                    if x==1 && z==1 {continue;}
                    let point = [g.origin_x+(xs[x]+xs[x+1])*0.5*g.tile_width,
                        g.origin_z+(zs[z]+zs[z+1])*0.5*g.tile_height];
                    let covers = |s: &SurfaceMeshData| s.positions.chunks_exact(4).filter(|p| {
                        let min = |axis| p.iter().map(|p|p[axis]).fold(f32::INFINITY,f32::min);
                        let max = |axis| p.iter().map(|p|p[axis]).fold(f32::NEG_INFINITY,f32::max);
                        point[0]>min(0) && point[0]<max(0) && point[1]>min(2) && point[1]<max(2)
                    }).count();
                    let is_sea = match side {0=>z==0,1=>z==2,2=>x==0,_=>x==2};
                    assert_eq!(covers(&water),usize::from(is_sea));
                    assert_eq!(covers(ground),usize::from(!is_sea));
                }}
                assert_eq!(mesh.footing_heights,control.footing_heights);
                assert_eq!(mesh.authored_cells,control.authored_cells);
                let (u0,u1,v0,v1) = g.uv(edges[side].unwrap().0,edges[side].unwrap().1);
                assert!(water.uvs.iter().all(|uv|*uv==[(u0+u1)*0.5,(v0+v1)*0.5]));
                finish_water_apron(&mut mesh,water);
                super::super::outdoor_water::finish(&mut mesh,map,&cells,&g);
                super::super::outdoor_water::finish(&mut control,map,&cells,&g);
                assert_eq!(mesh.footing_heights,control.footing_heights);
                assert_eq!(mesh.authored_cells,control.authored_cells);
                assert!(mesh.textured.positions.iter().all(|p|p[1]==-g.tile_height*0.25));
                for tri in mesh.textured.indices.chunks_exact(3) {
                    let p = [tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(mesh.textured.positions[i as usize]));
                    assert!((p[1]-p[0]).cross(p[2]-p[0]).y>0.);
                }
                let domains = crate::terrain_batches::separate_authentic_domains(mesh.textured,
                    Some(crate::terrain_batches::AuthenticBounds {min:Vec2::new(g.origin_x,g.origin_z),
                        max:Vec2::new(g.origin_x+40.*g.tile_width,g.origin_z+48.*g.tile_height)}));
                assert!(!domains.is_empty());
                assert!(domains.iter().all(|(domain,_)|
                    *domain==crate::terrain_batches::AuthenticDomain::Outside));
                assert_eq!(frame,before);
            }}}}
    }

    #[test]
    fn a_coastal_name_alone_cannot_turn_ponds_or_still_drawings_into_open_sea() {
        let (mut frame,mut shapes) = mixed_coast(3,"Route14");
        let select = |frame: &VisualWorldFrame,shapes: &[CellShape]| {
            coastal_edge_water_sources(40,48,&frame.tiles.iter().collect::<Vec<_>>(),shapes,&frame.map_id)
        };
        frame.map_id="NationalPark".into();
        assert_eq!(select(&frame,&shapes),[None;4]);
        frame.map_id="Route14".into();
        let original = frame.clone();
        for mode in 0..3 {
            frame=original.clone();
            for tile in &mut frame.tiles {
                match mode {0=>tile.animation_frames=None,1=>tile.animation_frames=Some(vec![tile.texture.clone()].into()),
                    _=>tile.priority=true}
            }
            assert_eq!(select(&frame,&shapes),[None;4]);
        }
        frame=original;
        for z in 0..48 {shapes[z*40+39]=CellShape::Flat;}
        assert_eq!(select(&frame,&shapes),[None;4],"closed dry boundary retains land");
    }

    #[test]
    fn flat_prop_art_cannot_supply_the_distant_ground() {
        let mut frame = test_frame_with_grid(40, 40);
        let shapes = vec![CellShape::Flat; 1600];
        for tile in &mut frame.tiles {
            tile.source = super::super::tests::source_with_tile(0x3a, 3, 3, 0x58);
        }
        assert_eq!(dominant_flat_source(40, 40,
            &frame.tiles.iter().collect::<Vec<_>>(), &shapes), None);
        frame.tiles[799].source = super::super::tests::source_with_tile(1, 0, 0, 6);
        assert_eq!(dominant_flat_source(40, 40,
            &frame.tiles.iter().collect::<Vec<_>>(), &shapes), Some((39, 19)));
    }

    #[test]
    fn coastal_continuation_uses_live_water_without_inventing_a_bank_or_support() {
        let mut frame = test_frame_with_grid(40, 40);
        frame.map_id = "Route40".into();
        let next = Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData);
        for tile in &mut frame.tiles {
            tile.source = super::super::tests::source_with_tile(0x35, 0, 0, 0x14);
            tile.animation_frames = Some(vec![tile.texture.clone(), next.clone()].into());
        }
        let shapes = vec![CellShape::Water; 1600];
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let g = GridGeometry { width: 40, height: 40, tile_width: 8., tile_height: 8.,
            origin_x: -160., origin_z: -160. };
        // Both builder modes must carry four faces through the same material;
        // source-cell terrain fitting must never map their outside vertices.
        for instanced in [false, true] {
            let mut mesh = TerrainMeshData { tree_cache: instanced.then(HashMap::new),
                footing_heights: vec![crate::profile::WATER_HEIGHT; 1600],
                authored_cells: vec![None; 1600], ..Default::default() };
            let support = mesh.footing_heights.clone();
            let water = append_repeating_background_apron(&mut mesh, &g, &cells, &shapes, &frame.map_id);
            assert_eq!(water.quad_count(), 4);
            new_bark::polish_world_surfaces_from_frame(&mut mesh, &frame, &cells, &g, &shapes);
            finish_water_apron(&mut mesh, water);
            assert!(mesh.background.is_none());
            assert_eq!(mesh.textured.quad_count(), 4);
            assert!(mesh.solid.positions.is_empty(), "no bank walls or land planes");
            assert_eq!(mesh.footing_heights, support);
            assert_eq!(mesh.authored_cells, vec![None; 1600]);
            let (u0, u1, v0, v1) = g.uv(0, 0);
            assert!(mesh.textured.uvs.iter().all(|uv|
                *uv == [(u0 + u1) * 0.5, (v0 + v1) * 0.5]));
            assert!(mesh.textured.positions.iter().all(|p|p[1] == crate::profile::WATER_HEIGHT));
            assert!(mesh.textured.positions.iter().all(|p|
                p[0] <= -160. || p[0] >= 160. || p[2] <= -160. || p[2] >= 160.));
            let domains = crate::terrain_batches::separate_authentic_domains(mesh.textured,
                Some(crate::terrain_batches::AuthenticBounds {
                    min: Vec2::splat(-160.), max: Vec2::splat(160.),
                }));
            assert!(!domains.is_empty());
            assert!(domains.iter().all(|(domain, _)|
                *domain == crate::terrain_batches::AuthenticDomain::Outside),
                "ocean continuation must never become an authenticated battle surface");
        }
        assert_eq!(dominant_coastal_water_source(40, 40, &cells, &shapes, "NationalPark"), None);
        frame.tiles[0].animation_frames = None;
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        assert_ne!(dominant_coastal_water_source(40, 40, &cells, &shapes, "Route40"), Some((0, 0)));
        // Even a large enclosed basin provides no evidence of open sea beyond
        // dry outer source cells.
        for tile in &mut frame.tiles {
            if tile.row == 0 || tile.row == 39 || tile.column == 0 || tile.column == 39 {
                tile.animation_frames = None;
            }
        }
        assert_eq!(dominant_coastal_water_source(40, 40,
            &frame.tiles.iter().collect::<Vec<_>>(), &shapes, "Route40"), None);
        for tile in &mut frame.tiles { tile.animation_frames = None; }
        assert_eq!(dominant_coastal_water_source(40, 40,
            &frame.tiles.iter().collect::<Vec<_>>(), &shapes, "Route40"), None);
    }

    #[test]
    fn masked_tree_art_cannot_become_the_dominant_ground_tile() {
        let mut frame = test_frame_with_grid(40, 40);
        for tile in &mut frame.tiles {
            tile.source = super::super::tests::source_with_tile(u16::MAX, 0, 0, u16::MAX);
        }
        frame.tiles[799].source = super::super::tests::source_with_tile(2, 0, 0, 5);
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let shapes = vec![CellShape::Flat; cells.len()];
        assert_eq!(dominant_flat_source(40, 40, &cells, &shapes), Some((39, 19)));
        frame.tiles[799].source.tile_index = u16::MAX;
        assert_eq!(dominant_flat_source(40, 40,
            &frame.tiles.iter().collect::<Vec<_>>(), &shapes), None);
    }

    #[test]
    fn flat_native_shore_corners_cannot_become_repeated_horizon_art() {
        let mut frame=test_frame_with_grid(40,40);
        let shapes=vec![CellShape::Flat;1600];
        for (block,x,y,tile) in [(0x2d,3,0,0x33),(0x1d,0,1,0x33),(0x1e,3,1,0x54)] {
            for t in &mut frame.tiles {
                t.source=super::super::tests::source_with_tile(block,x,y,tile);
                t.source.tileset_id="kanto".into();
            }
            let cells=frame.tiles.iter().collect::<Vec<_>>();
            assert_eq!(dominant_flat_source(40,40,&cells,&shapes),None,
                "a flat bank corner is not repeatable ground");
            frame.tiles[799].source=super::super::tests::source_with_tile(0x31,0,0,0x39);
            frame.tiles[799].source.tileset_id="kanto".into();
            let before=frame.clone();
            let cells=frame.tiles.iter().collect::<Vec<_>>();
            assert_eq!(dominant_flat_source(40,40,&cells,&shapes),Some((39,19)));
            assert_eq!(frame,before,"donor selection never rewrites source scenery");
        }
    }

    #[test]
    fn dominant_source_ties_are_stable_and_retain_ground_identity() {
        let mut frame = test_frame_with_grid(40, 40);
        for (i, tile) in frame.tiles.iter_mut().enumerate() {
            tile.source =
                super::super::tests::source_with_tile(0, 0, 0, if i % 2 == 0 { 5 } else { 6 });
        }
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes = vec![CellShape::Flat; cells.len()];
        let g = GridGeometry {
            width: 40,
            height: 40,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        for _ in 0..16 {
            let mut mesh = TerrainMeshData {
                tree_cache: Some(HashMap::new()),
                ..Default::default()
            };
            append_repeating_background_apron(&mut mesh, &g, &cells, &shapes, "test");
            assert_eq!(mesh.background.unwrap().source, cells[0].source);
        }
    }

    #[test]
    fn repeats_the_nearest_outer_metatile_course() {
        assert_eq!(repeated_edge_coordinate(-1, 44), 3);
        assert_eq!(repeated_edge_coordinate(-4, 44), 0);
        assert_eq!(repeated_edge_coordinate(44, 44), 40);
        assert_eq!(repeated_edge_coordinate(47, 44), 43);
        assert_eq!(repeated_edge_coordinate(48, 44), 40);
    }

    #[test]
    fn raised_objects_are_never_flattened_into_the_background() {
        let mut shapes = vec![CellShape::Flat; 16];
        shapes[0] = CellShape::Water;
        shapes[1] = CellShape::RaisedTop {
            height: 16.0,
            solid: SolidKind::Tree,
        };
        let frame = test_frame_with_grid(4, 4);
        let cells: Vec<_> = frame.tiles.iter().collect();

        assert_eq!(
            nearest_flat_source(0, 0, 4, 4, &cells, &shapes),
            Some((0, 1))
        );
        assert_ne!(
            nearest_flat_source(0, 0, 4, 4, &cells, &shapes),
            Some((1, 0))
        );
    }

    #[test]
    fn apron_is_flat_and_extends_every_grid_edge_without_solid_geometry() {
        let geometry = GridGeometry {
            width: 40,
            height: 40,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -16.0,
            origin_z: -16.0,
        };
        let mut mesh = TerrainMeshData::default();
        let shapes = vec![CellShape::Flat; geometry.width * geometry.height];
        let mut frame = test_frame_with_grid(geometry.width, geometry.height);
        for tile in &mut frame.tiles {
            tile.source = super::super::tests::source_with_tile(1,
                (tile.column % 4) as u8, (tile.row % 4) as u8, 5);
        }
        let cells: Vec<_> = frame.tiles.iter().collect();

        append_repeating_background_apron(&mut mesh, &geometry, &cells, &shapes, "test");

        let extended_width = geometry.width + BACKGROUND_APRON_TILES as usize * 2;
        let extended_height = geometry.height + BACKGROUND_APRON_TILES as usize * 2;
        let expected_quads = extended_width * extended_height - geometry.width * geometry.height;
        assert_eq!(mesh.textured.positions.len(), expected_quads * 4);
        assert!(
            mesh.textured
                .positions
                .iter()
                .all(|position| position[1] == BACKGROUND_HEIGHT)
        );
        assert!(mesh.solid.positions.is_empty());
        let mut compact = TerrainMeshData {
            tree_cache: Some(HashMap::new()),
            ..Default::default()
        };
        append_repeating_background_apron(&mut compact, &geometry, &cells, &shapes, "test");
        assert!(compact.textured.positions.is_empty());
        let background = compact.background.as_ref().unwrap();
        assert_eq!(background.mesh.quad_count(), 4);
        let mut area = 0.0;
        for (positions, uvs) in background
            .mesh
            .positions
            .chunks_exact(4)
            .zip(background.mesh.uvs.chunks_exact(4))
        {
            for (p, uv) in positions.iter().zip(uvs) {
                assert_eq!(p[0], geometry.origin_x + uv[0] * geometry.tile_width);
                assert_eq!(p[2], geometry.origin_z + uv[1] * geometry.tile_height);
                assert_eq!(p[1], BACKGROUND_HEIGHT);
            }
            area += (uvs[3][0] - uvs[0][0]) * (uvs[1][1] - uvs[0][1]);
            // A repeat sampler must read the same source texel at every source-pixel center,
            // including negative coordinates to the north and west of the map.
            for row in (uvs[0][1] as i32)..(uvs[1][1] as i32) {
                for column in (uvs[0][0] as i32)..(uvs[3][0] as i32) {
                    assert!(
                        !(column >= 0
                            && column < geometry.width as i32
                            && row >= 0
                            && row < geometry.height as i32)
                    );
                    for pixel in 0..8 {
                        assert_eq!(
                            ((column as f32 + (pixel as f32 + 0.5) / 8.0).rem_euclid(1.0) * 8.0)
                                .floor() as usize,
                            pixel
                        );
                        assert_eq!(
                            ((row as f32 + (pixel as f32 + 0.5) / 8.0).rem_euclid(1.0) * 8.0)
                                .floor() as usize,
                            pixel
                        );
                    }
                }
            }
        }
        assert_eq!(area as usize, expected_quads);

        let min_x = mesh
            .textured
            .positions
            .iter()
            .map(|position| position[0])
            .fold(f32::INFINITY, f32::min);
        let min_z = mesh
            .textured
            .positions
            .iter()
            .map(|position| position[2])
            .fold(f32::INFINITY, f32::min);
        assert_eq!(
            min_x,
            geometry.origin_x - BACKGROUND_APRON_TILES as f32 * geometry.tile_width
        );
        assert_eq!(
            min_z,
            geometry.origin_z - BACKGROUND_APRON_TILES as f32 * geometry.tile_height
        );
    }

    fn test_frame_with_grid(width: usize, height: usize) -> VisualWorldFrame {
        use std::sync::Arc;

        use bevy::prelude::{Handle, UVec2, Vec2};

        let mut frame = VisualWorldFrame { source_regions: Default::default(), view_center: Vec2::ZERO, terrace_datum: None,
            active: true,
            grid_origin: bevy::prelude::IVec2::ZERO,
            map_id: Arc::from("test"),
            source_map_size_core_tiles: None,
            jump_ledges: Default::default(),
            boundary_warps: Default::default(),
            map_texture: Handle::default(),
            center: Vec2::ZERO,
            viewport_size: Vec2::new(width as f32 * 8.0, height as f32 * 8.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(width as u32, height as u32),
            tiles: Vec::new(),
            actors: Vec::new(),
            terrain_revision: 0,
        };
        frame.tiles = (0..height)
            .flat_map(|row| {
                (0..width).map(move |column| VisualTile {
                    animation_frames: None,
                    column: column as u32,
                    row: row as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from("test"),
                        metatile_id: 0,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: 0,
                    },
                    texture: Handle::default(),
                    priority: false,
                })
            })
            .collect();
        frame
    }
}
