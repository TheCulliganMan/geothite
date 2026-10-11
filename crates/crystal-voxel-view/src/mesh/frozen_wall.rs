//! Connected Ice Path enclosure from complete authenticated rock-mass art.
//! Ground, sliding ice, little boulders and presentation support are untouched.
use super::*;

const HEIGHT: f32 = 48.0;

pub(in crate::mesh) fn frozen_point(p: Vec3, g: &GridGeometry, origin: [i32; 2]) -> Vec3 {
    let strength = (p.y / g.tile_height / 2.0).clamp(0.0, 1.0);
    if strength == 0.0 {
        return p;
    }
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let y = p.y / g.tile_height;
    let noise = new_bark::meadow_noise;
    // Broad unequal crowns sit above the shaft's smaller frozen fractures.
    // Apply this only above the authored low shelves; cap and vertical faces
    // share the field, including the continuous outside rock enclosure.
    let crown = ((y - 2.5) / 3.5).clamp(0., 1.);
    let crest = cave_cliff::slab(x * 0.22 + 53., z * 0.22 - 17.) * 1.8 * crown;
    // Frozen fractures run mostly vertically, unlike the sedimentary beds in
    // Union Cave. Both faces sample this field at a shared corner.
    let crust = ((HEIGHT / SOURCE_TILE_HEIGHT - y) / 1.4).clamp(0.0, 1.0);
    let fissure = (cave_cliff::slab(x * 0.75 + y * 0.045, z * 0.75).abs() - 0.14)
        * 0.65
        * crust
        * crust
        * (3.0 - 2.0 * crust);
    p + Vec3::new(
        (noise(x * 0.3 + y * 0.06, z * 0.3) * 0.45
            + cave_cliff::slab(x * 0.65 + y * 0.38, z * 0.65) * 0.65
            + fissure)
            * g.tile_width,
        (noise(x * 0.3, z * 0.3) * 0.80 + cave_cliff::slab(x * 0.40, z * 0.40) * 1.25 - 0.10 + crest)
            * g.tile_height,
        (noise(z * 0.3 - y * 0.06 + 37.0, x * 0.3) * 0.45
            + cave_cliff::slab(z * 0.65 - y * 0.38 + 37., x * 0.65) * 0.65
            - fissure)
            * g.tile_height,
    ) * strength
}

// Chip only a proven outside corner of the connected enclosure. Interior
// joins are untouched; no viewport edge or incomplete drawing gains a skirt.
fn enclosure_point(p: Vec3, cells: &[&VisualTile], g: &GridGeometry, origin: [i32; 2]) -> Vec3 {
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let corner = bevy::prelude::Vec2::new((x / 4.).round() * 4., (z / 4.).round() * 4.);
    let cx = corner.x as i32 - origin[0];
    let cz = corner.y as i32 - origin[1];
    let mut occupied = None;
    for (dx, dz) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
        let (column, row) = (cx + dx, cz + dz);
        if column < 0 || row < 0 || column >= g.width as i32 || row >= g.height as i32 {
            return frozen_point(p, g, origin);
        }
        let i = row as usize * g.width + column as usize;
        if complete_at(cells, g, i) {
            if occupied.is_some() {
                return frozen_point(p, g, origin);
            }
            occupied = Some(bevy::prelude::Vec2::new(
                if dx < 0 { -1. } else { 1. },
                if dz < 0 { -1. } else { 1. },
            ));
        } else if cells[i].source.tileset_id.as_ref() != "ice_path"
            || !matches!(shape_for_source(&cells[i].source), CellShape::Flat)
        {
            return frozen_point(p, g, origin);
        }
    }
    let Some(sign) = occupied else {
        return frozen_point(p, g, origin);
    };
    let world = bevy::prelude::Vec2::new(x, z);
    let local = (world - corner) * sign;
    // Broad, unequal corner breaks give each complete mass a less rectangular
    // crown. Keep the field in source-map coordinates so the shape stays put
    // when the player scrolls. The upper break widens above the lower shaft.
    let variation = new_bark::meadow_noise(corner.x * 0.5 + 71., corner.y * 0.5 - 19.) + 0.5;
    let crown = ((p.y / g.tile_height - 2.5) / 3.5).clamp(0., 1.);
    let radius = 1.05 + variation * 0.35 + crown * (0.35 + variation * 0.05);
    let mut rounded = p;
    if local.min_element() >= -0.02 && local.max_element() < radius && local.x + local.y < radius {
        // Compress rather than collapse the cap onto a diagonal line. Its
        // positive area keeps the fractured crown closed, with no folded
        // slivers over the corner. Broad faces retain their mineral relief.
        let next = world + sign * (radius - local.x - local.y) * 0.4;
        let rise = (p.y / g.tile_height / 1.4).clamp(0., 1.);
        let rise = rise * rise * (3. - 2. * rise);
        rounded.x += (next.x - world.x) * g.tile_width * rise;
        rounded.z += (next.y - world.y) * g.tile_height * rise;
    }
    frozen_point(rounded, g, origin)
}

fn complete_at(cells: &[&VisualTile], g: &GridGeometry, index: usize) -> bool {
    let Some((x, y)) = crate::ice_path::rock_mass_local(&cells[index].source) else {
        return false;
    };
    let Some(column) = (index % g.width).checked_sub(x as usize) else {
        return false;
    };
    let Some(row) = (index / g.width).checked_sub(y as usize) else {
        return false;
    };
    if column + 4 > g.width || row + 4 > g.height {
        return false;
    }
    let anchor = &cells[row * g.width + column].source;
    (0..4).all(|dy| {
        (0..4).all(|dx| {
            let source = &cells[(row + dy) * g.width + column + dx].source;
            source.metatile_id == anchor.metatile_id
                && crate::ice_path::rock_mass_local(source) == Some((dx as u8, dy as u8))
        })
    })
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
    origin: [i32; 2],
) {
    let h = HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    for index in p.indices(g.width) {
        let x = index % g.width;
        let y = index / g.width;
        let (w, e, n, s) = g.bounds(x, y);
        cave_cliff::quad_with_point(
            out,
            [[w, h, n], [w, h, s], [e, h, s], [e, h, n]],
            [0.62, 0.75, 0.77, 1.0],
            g,
            origin,
            |p, g, origin| enclosure_point(p, cells, g, origin),
            true,
        );
        for (side, (neighbor, points)) in [
            (
                y.checked_sub(1).map(|r| r * g.width + x),
                [[e, 0., n], [w, 0., n], [w, h, n], [e, h, n]],
            ),
            (
                (x + 1 < g.width).then_some(index + 1),
                [[e, 0., s], [e, 0., n], [e, h, n], [e, h, s]],
            ),
            (
                (y + 1 < g.height).then_some(index + g.width),
                [[w, 0., s], [e, 0., s], [e, h, s], [w, h, s]],
            ),
            (
                x.checked_sub(1).map(|c| y * g.width + c),
                [[w, 0., n], [w, 0., s], [w, h, s], [w, h, n]],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            // An unknown source halo is a continuation, not a new wall face.
            let Some(neighbor) = neighbor else {
                continue;
            };
            if complete_at(cells, g, neighbor) {
                continue;
            }
            cave_cliff::quad_with_point(
                out,
                points,
                [0.17, 0.35, 0.43, 1.0],
                g,
                origin,
                |p, g, origin| enclosure_point(p, cells, g, origin),
                true,
            );
            let source = &cells[neighbor].source;
            // Plain dry floor only. Sliding ice, authored objects, openings
            // and incomplete drawings cannot acquire this wall-foot talus.
            if source.tileset_id.as_ref() == "ice_path"
                && source.metatile_id == 0x02
                && source.tile_index == crate::ice_path::CAVE_GROUND_TILE
                && matches!(shape_for_source(source), CellShape::Flat)
            {
                cave_stones::edge_debris(
                    out,
                    [w, e, n, s],
                    0.,
                    h,
                    side,
                    false,
                    true,
                    g,
                    origin,
                    [origin[0] + x as i32, origin[1] + y as i32],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn irregular_frozen_crests_keep_vertical_order_and_enclosing_height() {
        let g = GridGeometry { width: 2, height: 2, tile_width: 8., tile_height: 8., origin_x: 0., origin_z: 0. };
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for z in 0..48 {
            for x in 0..48 {
                let p = Vec3::new(x as f32 * 4., 48., z as f32 * 4.);
                let top = frozen_point(p, &g, [0, 0]);
                min = min.min(top.y);
                max = max.max(top.y);
                assert!(top.y > 32., "crest must retain an enclosing wall at {p:?}");
                // No vertical folds from ground through the broad broken cap.
                let mut previous = frozen_point(Vec3::new(p.x, 0., p.z), &g, [0,0]).y;
                for y in 1..=96 {
                    let next = frozen_point(Vec3::new(p.x, y as f32 * 0.5, p.z), &g, [0,0]).y;
                    assert!(next > previous, "folded wall height at {p:?}, {y}");
                    previous = next;
                }
            }
        }
        assert!(max - min > 12., "crowns need broad physical height variation");
    }
    #[test]
    fn fractured_frozen_caps_keep_outward_winding_across_the_map() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let step = 1.;
        for y in [16., 32., 48.] {
            for z in 0..128 {
                for x in 0..128 {
                    let p = Vec3::new(x as f32 * step, y, z as f32 * step);
                    let a = frozen_point(p, &g, [10, 20]);
                    let b = frozen_point(p + Vec3::Z * step, &g, [10, 20]);
                    let c = frozen_point(p + Vec3::X * step, &g, [10, 20]);
                    assert!((b - a).cross(c - a).y > 0., "fold at {p:?}");
                }
            }
        }
    }
    #[test]
    fn frozen_surface_is_geometry_and_keeps_ground_exact_when_scrolling() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let floor = Vec3::new(4., 0., 3.);
        assert_eq!(frozen_point(floor, &g, [10, 20]), floor);
        let original = Vec3::new(4., 48., 3.);
        let actual = frozen_point(original, &g, [10, 20]);
        assert!((actual - original).length() > 1.0);
        let mut shifted = g;
        shifted.origin_x = -8.;
        assert!(
            (frozen_point(original - Vec3::X * 8., &shifted, [10, 20]) + Vec3::X * 8. - actual)
                .length()
                < 1e-5
        );
    }
    #[test]
    fn complete_neighbors_join_but_corrupt_or_missing_groups_keep_the_boundary() {
        let g = GridGeometry {
            width: 10,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut tiles = (0..60)
            .map(|index| VisualTile {
                column: (index % 10) as u32,
                row: (index / 10) as u32,
                animation_frames: None,
                priority: false,
                texture: Handle::default(),
                source: VisualTileSource {
                    tileset_id: "ice_path".into(),
                    metatile_id: 0,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: 0x19,
                },
            })
            .collect::<Vec<_>>();
        let art = [
            [0x84, 0x85, 0x86, 0x87],
            [0x94, 0x95, 0x96, 0x97],
            [0xa4, 0xa5, 0xa6, 0xa7],
            [0xb4, 0xb5, 0xb6, 0xb7],
        ];
        for column in [1, 5] {
            for y in 0..4 {
                for x in 0..4 {
                    tiles[(y + 1) * 10 + column + x].source = VisualTileSource {
                        tileset_id: "ice_path".into(),
                        metatile_id: 0x19,
                        subtile_column: x as u8,
                        subtile_row: y as u8,
                        tile_index: art[y][x],
                    };
                }
            }
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        // Sample the complete crown, including its widened corner breaks.
        // The old full diagonal projection could collapse the cap locally.
        for z in 0..32 {
            for x in 0..64 {
                let p = Vec3::new(8. + x as f32, 48., 8. + z as f32);
                let a = enclosure_point(p, &refs, &g, [3, 3]);
                let b = enclosure_point(p + Vec3::Z * 0.05, &refs, &g, [3, 3]);
                let c = enclosure_point(p + Vec3::X * 0.05, &refs, &g, [3, 3]);
                assert!((b - a).cross(c - a).y > 0., "folded crown at {p:?}");
            }
        }
        let corner = Vec3::new(8., 48., 8.);
        let rounded = enclosure_point(corner, &refs, &g, [3, 3]);
        assert!((rounded - frozen_point(corner, &g, [3, 3])).length() > 1.);
        let floor = Vec3::new(8., 0., 8.);
        assert_eq!(enclosure_point(floor, &refs, &g, [3, 3]), floor);
        let join = Vec3::new(40., 48., 8.);
        assert_eq!(
            enclosure_point(join, &refs, &g, [3, 3]),
            frozen_point(join, &g, [3, 3])
        );
        let groups = resolve("IcePathB1F", &refs, &g, None);
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|p| matches!(p.form, Form::FrozenWall)));
        let mut joined = SurfaceMeshData::default();
        append(&mut joined, &refs, &g, &groups[0], [10, 20]);
        assert!(joined.positions.iter().any(|p| p[1] > 40.0));
        // Talus belongs only on authenticated dry floor. It must not raise
        // the walking datum or change the enclosing mass itself.
        for tile in &mut tiles {
            if tile.source.metatile_id == 0 {
                tile.source.metatile_id = 0x02;
            }
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut dry = SurfaceMeshData::default();
        append(&mut dry, &refs, &g, &groups[0], [10, 20]);
        let stone = |uv: &[f32; 2]| (uv[1] / 100000.).round() == 7.;
        let fragments: Vec<_> = dry
            .positions
            .iter()
            .zip(&dry.uvs)
            .filter(|(_, uv)| stone(uv))
            .map(|(p, _)| *p)
            .collect();
        assert!(!fragments.is_empty());
        assert_eq!(
            fragments.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min),
            0.
        );
        assert!(fragments.iter().all(|p| p[1] >= 0. && p[1] < 4.));
        assert!(dry.normals.iter().all(|n| n.iter().all(|v| v.is_finite())
            && (Vec3::from_array(*n).length() - 1.).abs() < 1e-4));
        let wall: Vec<_> = dry
            .positions
            .iter()
            .zip(&dry.uvs)
            .filter(|(_, uv)| !stone(uv))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(wall, joined.positions);
        for (metatile, tile_index) in [(0x1f, 0xc6), (0x07, 0x19), (0x02, 0xff)] {
            for tile in &mut tiles {
                if crate::ice_path::rock_mass_local(&tile.source).is_none() {
                    tile.source.metatile_id = metatile;
                    tile.source.tile_index = tile_index;
                }
            }
            let refs = tiles.iter().collect::<Vec<_>>();
            let mut excluded = SurfaceMeshData::default();
            append(&mut excluded, &refs, &g, &groups[0], [10, 20]);
            assert!(
                !excluded.uvs.iter().any(stone),
                "talus on {metatile:x}/{tile_index:x}"
            );
        }
        for tile in &mut tiles {
            if crate::ice_path::rock_mass_local(&tile.source).is_none() {
                tile.source.metatile_id = 0;
                tile.source.tile_index = crate::ice_path::CAVE_GROUND_TILE;
            }
        }
        tiles[15].source.metatile_id = 0;
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut isolated = SurfaceMeshData::default();
        append(&mut isolated, &refs, &g, &groups[0], [10, 20]);
        assert!(isolated.indices.len() > joined.indices.len());
        assert_eq!(resolve("IcePathB1F", &refs, &g, None).len(), 1);
        let elsewhere = resolve("HallOfFame", &refs, &g, None);
        assert!(
            elsewhere
                .iter()
                .all(|p| !matches!(p.form, Form::FrozenWall))
        );
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![16.0; 60];
        super::super::append(&mut mesh, &g, &groups[0], &refs, [10, 20]);
        assert_eq!(mesh.footing_heights, vec![16.0; 60]);
    }
}
