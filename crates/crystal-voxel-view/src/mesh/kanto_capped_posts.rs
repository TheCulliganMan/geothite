//! One closed post for each complete, narrow native Kanto two-row drawing.
//!
//! A cap cannot consume the neighboring post, and a missing shaft rejects the
//! entire drawing. The classifier, source art, block phase and live paving
//! must all agree. These decorative meshes never alter collision or footing.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Placement {
    pub(super) rect: [usize; 4],
    sample: usize,
}

fn source_band(source: &VisualTileSource, row: u8) -> Option<CellShape> {
    if source.subtile_column >= 4
        || source.subtile_row >= 4
        || source.tile_index != [0x0e, 0x55][usize::from(row)]
    {
        return None;
    }
    let shape = crate::kanto_post::kanto_post_shape(source)?;
    matches!(
        shape,
        CellShape::FacadeBand {
            band_from_top,
            band_count: 2,
            ground_tile_index: 0x23,
            solid: SolidKind::Fence,
            ..
        } if band_from_top == row
    )
    .then_some(shape)
}

pub(super) fn resolve(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
) -> Vec<Placement> {
    if g.width == 0 || cells.len() != g.width * g.height || shapes.len() != cells.len() {
        return Vec::new();
    }
    // The preferred native paving must belong to this atlas and zero datum.
    // Identically numbered connected-map and elevated samples are rejected.
    let preferred = cells.iter().zip(shapes).position(|(cell, shape)| {
        cell.source.tileset_id.as_ref() == "kanto"
            && cell.source.tile_index == 0x23
            && matches!(shape, CellShape::Flat | CellShape::PlaneAt { height: 0.0 })
    });
    let mut placements = Vec::new();
    for y in 0..g.height.saturating_sub(1) {
        for x in 0..g.width {
            let index = y * g.width + x;
            let first = &cells[index].source;
            let Some(cap_shape) = source_band(first, 0) else {
                continue;
            };
            let second = &cells[index + g.width].source;
            let Some(shaft_shape) = source_band(second, 1) else {
                continue;
            };
            if first.tileset_id != second.tileset_id
                || first.metatile_id != second.metatile_id
                || first.subtile_column != second.subtile_column
                || first.subtile_row + 1 != second.subtile_row
            {
                continue;
            }
            let Some(sample) = preferred.or_else(|| {
                // Blocks 56/77 author two native dotted-paving rows above their
                // post course. Some complete scenes contain no tile 23. Only
                // those exact cells in THIS block can supply the alternative;
                // no global guess, unrelated floor or map allowlist is used.
                if !matches!(first.metatile_id, 0x56 | 0x77) || first.subtile_row != 2 || y < 2 {
                    return None;
                }
                (0..2)
                    .all(|row| {
                        let index = (y - 2 + row) * g.width + x;
                        let backing = &cells[index].source;
                        backing.tileset_id == first.tileset_id
                            && backing.metatile_id == first.metatile_id
                            && backing.subtile_column == first.subtile_column
                            && usize::from(backing.subtile_row) == row
                            && backing.tile_index == 0x39
                            && matches!(
                                shapes[index],
                                CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
                            )
                    })
                    .then_some((y - 1) * g.width + x)
            }) else {
                continue;
            };
            let authored_bands =
                shapes[index] == cap_shape && shapes[index + g.width] == shaft_shape;
            // build_scene downgrades bands to Flat when their preferred ground
            // is absent. Admit that state only after proving the exact native
            // backing above; other custom shape changes still keep fallback.
            let missing_preferred_ground = preferred.is_none()
                && matches!(shapes[index], CellShape::Flat)
                && matches!(shapes[index + g.width], CellShape::Flat);
            if !authored_bands && !missing_preferred_ground {
                continue;
            }
            placements.push(Placement {
                rect: [x, y, 1, 2],
                sample,
            });
        }
    }
    placements
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
) {
    for p in resolve(cells, shapes, g) {
        if !clear(claimed, g, p.rect) {
            continue;
        }
        floor(mesh, cells, shapes, g, p.rect, p.sample, claimed);
        let [west, east, _, south] = bounds(g, p.rect);
        // Preserve the individual narrow silhouette and the source course's
        // southern footing. Cap overhang stays inside its own native column;
        // two neighboring caps retain a 0.28-cell gap, with no joining rail.
        let fitted = [
            west + g.tile_width * 0.14,
            east - g.tile_width * 0.14,
            south - g.tile_height * 0.68,
            south - g.tile_height * 0.02,
        ];
        let m = model(Kind::KantoCappedPost);
        m.append_fitted(
            &mut mesh.solid,
            fitted,
            0.0,
            g.tile_height * 1.55 / (m.max[1] - m.min[1]),
            None,
        );
        mark_authored_rect(mesh, g, p.rect, Kind::KantoCappedPost.label());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;
    use std::sync::Arc;

    fn fixture(block: u16, column: u8, top: u8) -> (Vec<VisualTile>, Vec<CellShape>, GridGeometry) {
        let mut tiles = Vec::new();
        let mut shapes = Vec::new();
        for row in 0..2 {
            for x in 0..2 {
                let source = VisualTileSource {
                    tileset_id: Arc::from("kanto"),
                    metatile_id: if x == 0 { block } else { 0x01 },
                    subtile_column: if x == 0 { column } else { 0 },
                    subtile_row: if x == 0 { top + row } else { 0 },
                    tile_index: if x == 0 {
                        [0x0e, 0x55][row as usize]
                    } else {
                        0x23
                    },
                };
                shapes.push(if x == 0 {
                    crate::kanto_post::kanto_post_shape(&source).unwrap_or(CellShape::Flat)
                } else {
                    CellShape::Flat
                });
                tiles.push(VisualTile {
                    animation_frames: None,
                    column: x,
                    row: u32::from(row),
                    source,
                    texture: Handle::default(),
                    priority: false,
                });
            }
        }
        (
            tiles,
            shapes,
            GridGeometry {
                width: 2,
                height: 2,
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: -24.0,
                origin_z: 40.0,
            },
        )
    }

    fn phases() -> Vec<(u16, u8, u8)> {
        let mut result = Vec::new();
        for (block, columns, tops) in [
            (0x1b, 0..2, vec![0, 2]),
            (0x5f, 2..4, vec![0, 2]),
            (0x56, 0..2, vec![2]),
            (0x77, 0..4, vec![2]),
        ] {
            for column in columns {
                for &top in &tops {
                    result.push((block, column, top));
                }
            }
        }
        result
    }

    #[test]
    fn all_four_source_blocks_and_fourteen_courses_append_whole_posts() {
        assert_eq!(phases().len(), 14);
        for (block, column, top) in phases() {
            let (tiles, shapes, g) = fixture(block, column, top);
            let cells = tiles.iter().collect::<Vec<_>>();
            let before = tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>();
            assert_eq!(resolve(&cells, &shapes, &g).len(), 1);
            // There is intentionally no map allowlist, including connected-map
            // and dynamically assembled visual grids carrying these sources.
            for map in ["Route13", "PalletTown", "UnlistedConnectedScene"] {
                assert_eq!(preferred_cells(map, &cells, &g), [true, false, true, false]);
            }
            let mut mesh = TerrainMeshData {
                footing_heights: vec![3.25; 4],
                authored_cells: vec![None; 4],
                ..Default::default()
            };
            let mut claimed = [false; 4];
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(claimed, [true, false, true, false]);
            assert_eq!(mesh.footing_heights, [3.25; 4]);
            assert_eq!(mesh.textured.indices.len(), 12);
            assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.0));
            assert_eq!(
                mesh.solid.indices.len(),
                model(Kind::KantoCappedPost).surface.indices.len()
            );
            assert_eq!(
                mesh.authored_cells,
                [
                    Some(Kind::KantoCappedPost.label()),
                    None,
                    Some(Kind::KantoCappedPost.label()),
                    None
                ]
            );
            assert_eq!(
                before,
                tiles.iter().map(|t| t.source.clone()).collect::<Vec<_>>()
            );
            let count = mesh.solid.indices.len();
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(
                mesh.solid.indices.len(),
                count,
                "a second append cannot duplicate the post"
            );
        }
    }

    #[test]
    fn changed_cropped_mixed_and_shifted_posts_keep_both_source_cells() {
        for (block, column, top) in phases() {
            for change in 0..11 {
                let (mut tiles, mut shapes, mut g) = fixture(block, column, top);
                match change {
                    0 => tiles[0].source.tile_index ^= 1,
                    1 => tiles[2].source.tile_index ^= 1,
                    2 => tiles[2].source.tileset_id = Arc::from("johto"),
                    3 => tiles[2].source.metatile_id ^= 1,
                    4 => tiles[2].source.subtile_column ^= 1,
                    5 => tiles[2].source.subtile_row ^= 2,
                    6 => tiles[0].source.subtile_column = 4,
                    7 => shapes[2] = CellShape::Flat,
                    8 => {
                        g.height = 1;
                        tiles.truncate(2);
                        shapes.truncate(2);
                    }
                    9 => {
                        g.height = 1;
                        tiles.drain(..2);
                        shapes.drain(..2);
                    }
                    10 => {
                        tiles[0].source.subtile_row += 4;
                        tiles[2].source.subtile_row += 4;
                    }
                    _ => unreachable!(),
                }
                let cells = tiles.iter().collect::<Vec<_>>();
                assert!(
                    resolve(&cells, &shapes, &g).is_empty(),
                    "{block:x}/{column}/{top}, mutation {change}"
                );
                let mut mesh = TerrainMeshData::default();
                let mut claimed = vec![false; cells.len()];
                append(&mut mesh, &cells, &shapes, &g, &mut claimed);
                assert!(!claimed.iter().any(|v| *v));
                assert!(mesh.solid.indices.is_empty());
                assert!(mesh.textured.indices.is_empty());
            }
        }
        // Two valid family members still cannot donate each other's shaft.
        let (mut tiles, shapes, g) = fixture(0x1b, 0, 2);
        tiles[2].source.metatile_id = 0x56;
        assert!(resolve(&tiles.iter().collect::<Vec<_>>(), &shapes, &g).is_empty());
    }

    #[test]
    fn matching_art_in_unrelated_quadrants_is_not_a_post() {
        for block in [0x1b, 0x5f, 0x56, 0x77, 0x01, 0x13, 0x29] {
            for column in 0..4 {
                for top in [0, 2] {
                    let (tiles, shapes, g) = fixture(block, column, top);
                    assert_eq!(
                        resolve(&tiles.iter().collect::<Vec<_>>(), &shapes, &g).len(),
                        usize::from(phases().contains(&(block, column, top)))
                    );
                }
            }
        }
    }

    #[test]
    fn live_native_zero_height_paving_and_full_claim_availability_are_required() {
        for change in 0..5 {
            let (mut tiles, mut shapes, g) = fixture(0x77, 0, 2);
            for index in [1, 3] {
                match change {
                    0 => tiles[index].source.tileset_id = Arc::from("johto"),
                    1 => tiles[index].source.tile_index = 0x11,
                    2 => shapes[index] = CellShape::Water,
                    3 => {
                        shapes[index] = CellShape::RaisedTop {
                            height: 8.0,
                            solid: SolidKind::Bank,
                        }
                    }
                    4 => shapes[index] = CellShape::PlaneAt { height: 1.0 },
                    _ => unreachable!(),
                }
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            assert!(resolve(&cells, &shapes, &g).is_empty());
            if change < 2 {
                assert!(!preferred_cells("Route13", &cells, &g).iter().any(|v| *v));
            }
        }
        let (tiles, shapes, g) = fixture(0x77, 0, 2);
        let cells = tiles.iter().collect::<Vec<_>>();
        for occupied in [0, 2] {
            let mut claimed = [false; 4];
            claimed[occupied] = true;
            let before = claimed;
            let mut mesh = TerrainMeshData::default();
            append(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(
                claimed, before,
                "an existing claim rejects the entire object"
            );
            assert!(mesh.solid.indices.is_empty());
            assert!(mesh.textured.indices.is_empty());
        }
    }

    #[test]
    fn separate_posts_and_an_open_course_gap_preserve_spacing_at_any_tile_scale() {
        for tile_height in [8.0, 16.0] {
            let (base_tiles, _, mut g) = fixture(0x77, 0, 2);
            g.width = 5;
            g.tile_height = tile_height;
            let mut tiles = Vec::new();
            let mut shapes = Vec::new();
            for y in 0..2 {
                for x in 0..5 {
                    let is_post = matches!(x, 0 | 1 | 3);
                    let mut tile = base_tiles[y * 2 + usize::from(!is_post)].clone();
                    tile.column = x as u32;
                    tile.row = y as u32;
                    if is_post {
                        tile.source.subtile_column = x as u8;
                    }
                    shapes.push(if is_post {
                        crate::kanto_post::kanto_post_shape(&tile.source).unwrap()
                    } else {
                        CellShape::Flat
                    });
                    tiles.push(tile);
                }
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(&cells, &shapes, &g);
            assert_eq!(
                placements.iter().map(|p| p.rect).collect::<Vec<_>>(),
                [[0, 0, 1, 2], [1, 0, 1, 2], [3, 0, 1, 2]]
            );
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 10],
                ..Default::default()
            };
            let mut claimed = [false; 10];
            append(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(
                claimed,
                [
                    true, true, false, true, false, true, true, false, true, false
                ]
            );
            let m = model(Kind::KantoCappedPost);
            for (positions, x) in mesh
                .solid
                .positions
                .chunks_exact(m.surface.positions.len())
                .zip([0, 1, 3])
            {
                let west = g.origin_x + x as f32 * g.tile_width;
                let south = g.origin_z + 2.0 * g.tile_height;
                assert!(
                    positions.iter().all(|v| v[0] >= west + g.tile_width * 0.139
                        && v[0] <= west + g.tile_width * 0.861)
                );
                assert!(
                    positions
                        .iter()
                        .all(|v| v[2] >= south - g.tile_height * 0.681
                            && v[2] <= south - g.tile_height * 0.019)
                );
                assert!(
                    positions
                        .iter()
                        .all(|v| v[1] >= 0.0 && v[1] <= tile_height * 1.551)
                );
                assert!(
                    positions
                        .iter()
                        .any(|v| (v[1] - tile_height * 1.55).abs() < 0.001)
                );
                assert!(positions.iter().any(|v| v[1] == 0.0));
            }
        }
    }

    #[test]
    fn missing_preferred_paving_uses_only_the_complete_same_block_native_backing() {
        for block in [0x56, 0x77] {
            let (mut tiles, mut shapes, mut g) = fixture(block, 0, 2);
            for index in [1, 3] {
                tiles[index].source.tile_index = 0x11;
            }
            let mut upper = Vec::new();
            for row in 0..2 {
                for column in 0..2 {
                    let mut tile = tiles[0].clone();
                    tile.column = column;
                    tile.row = row;
                    tile.source.subtile_row = row as u8;
                    tile.source.subtile_column = column as u8;
                    tile.source.tile_index = 0x39;
                    upper.push(tile);
                }
            }
            for tile in &mut tiles {
                tile.row += 2;
            }
            upper.extend(tiles);
            tiles = upper;
            let mut upper_shapes = vec![CellShape::Flat; 4];
            upper_shapes.append(&mut shapes);
            shapes = upper_shapes;
            g.height = 4;
            let cells = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(&cells, &shapes, &g);
            assert_eq!(placements.len(), 1);
            assert_eq!(placements[0].sample, 2);
            assert_eq!(
                preferred_cells("UnlistedConnectedScene", &cells, &g),
                [false, false, false, false, true, false, true, false]
            );
            // Exercise the production missing-paving facade downgrade too.
            shapes[4] = CellShape::Flat;
            shapes[6] = CellShape::Flat;
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 8],
                ..Default::default()
            };
            let mut claimed = [false; 8];
            append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
            assert_eq!(
                claimed,
                [false, false, false, false, true, false, true, false]
            );
            assert_eq!(
                mesh.solid.indices.len(),
                model(Kind::KantoCappedPost).surface.indices.len()
            );
            let uv = g.uv(0, 1);
            assert!(
                mesh.textured
                    .uvs
                    .iter()
                    .all(|p| (p[0] == uv.0 || p[0] == uv.1) && (p[1] == uv.2 || p[1] == uv.3))
            );
            for index in [0, 2] {
                for mutation in 0..6 {
                    let mut changed = tiles.clone();
                    let mut changed_shapes = shapes.clone();
                    match mutation {
                        0 => changed[index].source.tile_index = 0x11,
                        1 => changed[index].source.metatile_id = 0x31,
                        2 => changed[index].source.tileset_id = Arc::from("johto"),
                        3 => changed[index].source.subtile_column = 1,
                        4 => changed[index].source.subtile_row ^= 1,
                        5 => changed_shapes[index] = CellShape::Water,
                        _ => unreachable!(),
                    }
                    assert!(
                        resolve(&changed.iter().collect::<Vec<_>>(), &changed_shapes, &g)
                            .is_empty()
                    );
                }
            }
        }
    }

    #[test]
    fn capped_post_is_one_closed_sculpt_with_a_narrow_shaft_and_rounded_crown() {
        let m = model(Kind::KantoCappedPost);
        assert_eq!(m.surface.indices.len() / 3, 288);
        assert_eq!(m.min[1], 0.0);
        assert!(m.max[1] > 2.0 * (m.max[0] - m.min[0]));
        let shaft = m
            .surface
            .positions
            .iter()
            .filter(|v| (0.15..=0.805).contains(&v[1]))
            .map(|v| v[0].abs())
            .fold(0.0_f32, f32::max);
        let cap = m
            .surface
            .positions
            .iter()
            .filter(|v| v[1] > 0.90)
            .map(|v| v[0].abs())
            .fold(0.0_f32, f32::max);
        assert!(cap > shaft * 1.25, "the pale cap overhangs the slim shaft");
        let vertex =
            |i: u32| m.surface.positions[i as usize].map(|v| (v * 1_000_000.0).round() as i32);
        let mut edges = std::collections::BTreeMap::new();
        let mut signed_volume = 0.0;
        for tri in m.surface.indices.chunks_exact(3) {
            let a = Vec3::from_array(m.surface.positions[tri[0] as usize]);
            let b = Vec3::from_array(m.surface.positions[tri[1] as usize]);
            let c = Vec3::from_array(m.surface.positions[tri[2] as usize]);
            assert!((b - a).cross(c - a).length_squared() > 0.0000000001);
            signed_volume += a.dot(b.cross(c)) / 6.0;
            for [from, to] in [[tri[0], tri[1]], [tri[1], tri[2]], [tri[2], tri[0]]] {
                let (a, b) = (vertex(from), vertex(to));
                let (key, direction) = if a < b {
                    ((a, b), 1_i32)
                } else {
                    ((b, a), -1_i32)
                };
                let entry = edges.entry(key).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += direction;
            }
        }
        assert!(signed_volume > 0.17);
        assert!(
            edges
                .values()
                .all(|&(count, balance)| count == 2 && balance == 0)
        );
        for (axis, sign) in [
            (0, -1.0),
            (0, 1.0),
            (1, -1.0),
            (1, 1.0),
            (2, -1.0),
            (2, 1.0),
        ] {
            assert!(m.surface.normals.iter().any(|v| v[axis] * sign > 0.9));
        }
    }
}
