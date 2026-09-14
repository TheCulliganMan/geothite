//! Continuous authored north walls for the four player-house layouts.

use super::*;

const PLAYERS_HOUSE_1F_COURSE: &[u16] = &[0x07, 0x0f, 0x11, 0x05, 0x0a];
const ELMS_HOUSE_COURSE: &[u16] = &[0x10, 0x01, 0x11, 0x1b];
const REDS_HOUSE_1F_COURSE: &[u16] = &[0x1b, 0x11, 0x01, 0x0a];
const COPYCATS_HOUSE_1F_COURSE: &[u16] = &[0x1b, 0x23, 0x11, 0x01];
const REDS_HOUSE_2F_COURSE: &[u16] = &[0x10, 0x20, 0x1b, 0x0b];
const COPYCATS_HOUSE_2F_COURSE: &[u16] = &[0x1b, 0x0b, 0x02, 0x20, 0x04];
const COURSE_VARIANTS: [&[u16]; 6] = [
    PLAYERS_HOUSE_1F_COURSE,
    ELMS_HOUSE_COURSE,
    REDS_HOUSE_1F_COURSE,
    COPYCATS_HOUSE_1F_COURSE,
    REDS_HOUSE_2F_COURSE,
    COPYCATS_HOUSE_2F_COURSE,
];
const COURSE_HEIGHT: usize = 4;

pub(super) fn append_north_wall_courses(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    claimed: &mut [bool],
) -> Result<(), TerrainMeshError> {
    for (column, row, width) in course_origins(cells, geometry) {
        append_course(mesh, cells, geometry, claimed, column, row, width)?;
    }
    append_partitions(mesh, cells, geometry, claimed)?;
    Ok(())
}

pub(super) fn course_origins(
    cells: &[&VisualTile],
    geometry: &GridGeometry,
) -> Vec<(usize, usize, usize)> {
    let mut origins = Vec::new();
    for row in 0..geometry.height.saturating_sub(COURSE_HEIGHT - 1) {
        for column in 0..geometry.width {
            let matched_width = COURSE_VARIANTS.iter().find_map(|variant| {
                let width = variant.len() * 4;
                if column + width > geometry.width {
                    return None;
                }
                variant
                    .iter()
                    .enumerate()
                    .all(|(block, expected)| {
                        (0..4).all(|local_row| {
                            (0..4).all(|local_column| {
                                let tile = cells[(row + local_row) * geometry.width
                                    + column
                                    + block * 4
                                    + local_column];
                                tile.source.tileset_id.as_ref() == "players_house"
                                    && tile.source.metatile_id == *expected
                                    && usize::from(tile.source.subtile_column) == local_column
                                    && usize::from(tile.source.subtile_row) == local_row
                            })
                        })
                    })
                    .then_some(width)
            });
            if let Some(width) = matched_width {
                origins.push((column, row, width));
            }
        }
    }
    origins
}

fn append_course(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    claimed: &mut [bool],
    column: usize,
    row: usize,
    width: usize,
) -> Result<(), TerrainMeshError> {
    let ground_index = cells
        .iter()
        .position(|tile| {
            tile.source.tileset_id.as_ref() == "players_house" && tile.source.tile_index == 0x01
        })
        .ok_or(TerrainMeshError::MissingGroundSample {
            column: column as u32,
            row: row as u32,
            tile_index: 0x01,
        })?;
    let ground_uv = geometry.uv(ground_index % geometry.width, ground_index / geometry.width);
    let plane_z = geometry.origin_z + (row + COURSE_HEIGHT - 1) as f32 * geometry.tile_height;
    for local_row in 0..COURSE_HEIGHT {
        let band_bottom = (COURSE_HEIGHT - local_row - 1) as f32 * geometry.tile_height;
        let band_top = band_bottom + geometry.tile_height;
        for local_column in 0..width {
            let source_column = column + local_column;
            let source_row = row + local_row;
            let index = source_row * geometry.width + source_column;
            // Stair art belongs to the solid flight pass, not the wall.
            if crate::players_house::stair_shape(&cells[index].source).is_some() {
                continue;
            }
            let fixture = crate::players_house::upright_fixture_local(&cells[index].source)
                .is_some()
                || crate::players_house::tv_local(&cells[index].source).is_some();
            // Architectural backing must not own the cabinet/TV source cells:
            // claiming them here suppresses the later solid-object pass.
            claimed[index] = !fixture;
            let (x0, x1, z0, z1) = geometry.bounds(source_column, source_row);
            append_top(&mut mesh.textured, [x0, x1, z0, z1], 0.0, ground_uv);
            let stair_column = (row..row + COURSE_HEIGHT).any(|scan_row| {
                crate::players_house::stair_local(
                    &cells[scan_row * geometry.width + source_column].source,
                )
                .is_some()
            });
            if stair_column
                || (cells[index].source.metatile_id == 0x0f
                    && cells[index].source.subtile_column >= 2)
            {
                continue;
            }
            let wall_source = if fixture {
                cells
                    .iter()
                    .position(|tile| {
                        tile.source.tileset_id.as_ref() == "players_house"
                            && tile.source.tile_index == 0x11
                    })
                    .ok_or(TerrainMeshError::MissingGroundSample {
                        column: source_column as u32,
                        row: source_row as u32,
                        tile_index: 0x11,
                    })?
            } else {
                index
            };
            let (u0, u1, v0, v1) =
                geometry.uv(wall_source % geometry.width, wall_source / geometry.width);
            append_quad(
                &mut mesh.textured,
                [
                    [x1, band_bottom, plane_z],
                    [x1, band_top, plane_z],
                    [x0, band_top, plane_z],
                    [x0, band_bottom, plane_z],
                ],
                [0.0, 0.0, 1.0],
                [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
                TEXTURED_SHADE,
            );
        }
    }
    Ok(())
}

// The divider is drawn in plan for six rows, with two wall bands at its
// south end. Folding that complete drawing avoids a tall front-facing card.
fn append_partitions(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
) -> Result<(), TerrainMeshError> {
    const DRAWING: [[u16; 2]; 8] = [
        [0x25, 0x35],
        [0x25, 0x35],
        [0x25, 0x35],
        [0x25, 0x35],
        [0x25, 0x35],
        [0x33, 0x34],
        [0x11, 0x11],
        [0x11, 0x11],
    ];
    for row in 0..g.height.saturating_sub(7) {
        for col in 0..g.width.saturating_sub(1) {
            let complete = (0..8).all(|r| {
                (0..2).all(|c| {
                    let src = &cells[(row + r) * g.width + col + c].source;
                    src.tileset_id.as_ref() == "players_house"
                        && src.metatile_id == (if r < 4 { 0x0f } else { 0x16 })
                        && src.subtile_column == (c + 2) as u8
                        && src.subtile_row == (r % 4) as u8
                        && src.tile_index == DRAWING[r][c]
                })
            });
            if !complete {
                continue;
            }
            let ground = cells
                .iter()
                .position(|t| {
                    t.source.tileset_id.as_ref() == "players_house"
                        && t.source.tile_index == crate::players_house::FLOOR_TILE
                })
                .ok_or(TerrainMeshError::MissingGroundSample {
                    column: col as u32,
                    row: row as u32,
                    tile_index: crate::players_house::FLOOR_TILE,
                })?;
            let h = 2.0 * g.tile_height;
            let front = g.origin_z + (row + 6) as f32 * g.tile_height;
            for r in 0..8 {
                for c in 0..2 {
                    claimed[(row + r) * g.width + col + c] = true;
                    let (x0, x1, z0, z1) = g.bounds(col + c, row + r);
                    append_top(
                        &mut mesh.textured,
                        [x0, x1, z0, z1],
                        0.0,
                        g.uv(ground % g.width, ground / g.width),
                    );
                    if r < 6 {
                        append_top(
                            &mut mesh.textured,
                            [x0, x1, z0, z1],
                            h,
                            g.uv(col + c, row + r),
                        );
                    }
                }
            }
            for band in 0..2 {
                let top = h - band as f32 * g.tile_height;
                let bottom = top - g.tile_height;
                for c in 0..2 {
                    let (x0, x1, _, _) = g.bounds(col + c, row);
                    let (u0, u1, v0, v1) = g.uv(col + c, row + 6 + band);
                    append_quad(
                        &mut mesh.textured,
                        [
                            [x1, bottom, front],
                            [x1, top, front],
                            [x0, top, front],
                            [x0, bottom, front],
                        ],
                        [0.0, 0.0, 1.0],
                        [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
                        TEXTURED_SHADE,
                    );
                    let north = g.origin_z + row as f32 * g.tile_height;
                    append_quad(
                        &mut mesh.textured,
                        [
                            [x0, bottom, north],
                            [x0, top, north],
                            [x1, top, north],
                            [x1, bottom, north],
                        ],
                        [0.0, 0.0, -1.0],
                        [[u0, v1], [u0, v0], [u1, v0], [u1, v1]],
                        [0.68, 0.68, 0.68, 1.0],
                    );
                }
                for r in 0..6 {
                    for east in [false, true] {
                        let x = g.origin_x + (col + if east { 2 } else { 0 }) as f32 * g.tile_width;
                        let z0 = g.origin_z + (row + r) as f32 * g.tile_height;
                        let z1 = z0 + g.tile_height;
                        let (a, b, n) = if east {
                            (z0, z1, [1.0, 0.0, 0.0])
                        } else {
                            (z1, z0, [-1.0, 0.0, 0.0])
                        };
                        let (u0, u1, v0, v1) = g.uv(col + usize::from(east), row + 6 + band);
                        append_quad(
                            &mut mesh.textured,
                            [[x, bottom, a], [x, top, a], [x, top, b], [x, bottom, b]],
                            n,
                            [[u0, v1], [u0, v0], [u1, v0], [u1, v1]],
                            [0.78, 0.78, 0.78, 1.0],
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Handle;
    use std::sync::Arc;

    fn tile(column: u32, row: u32) -> VisualTile {
        VisualTile {
            column,
            row,
            source: VisualTileSource {
                tileset_id: Arc::from("players_house"),
                metatile_id: 0,
                subtile_column: (column % 4) as u8,
                subtile_row: (row % 4) as u8,
                tile_index: 0x11,
            },
            texture: Handle::default(),
            priority: false,
        }
    }

    #[test]
    fn room_partition_folds_its_plan_and_front_into_one_solid_wall() {
        let mut tiles = (0..8)
            .flat_map(|r| (0..3).map(move |c| tile(c, r)))
            .collect::<Vec<_>>();
        for r in 0..8 {
            for c in 0..2 {
                let src = &mut tiles[r * 3 + c].source;
                src.metatile_id = if r < 4 { 0x0f } else { 0x16 };
                src.subtile_column = (c + 2) as u8;
                src.subtile_row = (r % 4) as u8;
                src.tile_index = if r < 5 {
                    [0x25, 0x35][c]
                } else if r == 5 {
                    [0x33, 0x34][c]
                } else {
                    0x11
                };
            }
        }
        tiles[2].source.tile_index = 0x01;
        let cells = tiles.iter().collect::<Vec<_>>();
        let g = GridGeometry {
            width: 3,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 24];
        append_partitions(&mut mesh, &cells, &g, &mut claimed).unwrap();
        assert_eq!(claimed.iter().filter(|v| **v).count(), 16);
        let raised_tops = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(p, n)| n[0] == [0.0, 1.0, 0.0] && p[0][1] > 0.0)
            .collect::<Vec<_>>();
        assert_eq!(raised_tops.len(), 12);
        assert!(
            raised_tops
                .iter()
                .all(|(p, _)| p.iter().all(|v| v[1] == 16.0 && v[2] <= 48.0))
        );
        assert!(mesh.textured.positions.iter().all(|p| p[1] <= 16.0));
    }

    #[test]
    fn player_house_courses_include_their_complete_wall_widths() {
        assert_eq!(COURSE_VARIANTS.len(), 6);
        assert_eq!(PLAYERS_HOUSE_1F_COURSE, &[0x07, 0x0f, 0x11, 0x05, 0x0a]);
        assert_eq!(PLAYERS_HOUSE_1F_COURSE.len() * 4, 20);
        assert_eq!(COPYCATS_HOUSE_2F_COURSE.len() * 4, 20);
        assert_eq!(REDS_HOUSE_2F_COURSE.len() * 4, 16);
    }

    #[test]
    fn player_house_course_leaves_stairs_and_furniture_to_their_volume_renderers() {
        let mut tiles = (0..4)
            .flat_map(|row| (0..20).map(move |column| tile(column, row)))
            .collect::<Vec<_>>();
        let blocks = PLAYERS_HOUSE_1F_COURSE;
        for (block_column, block) in blocks.iter().enumerate() {
            for local_row in 0..4 {
                for local_column in 0..4 {
                    let tile = &mut tiles[local_row * 20 + block_column * 4 + local_column];
                    tile.source.tileset_id = Arc::from("players_house");
                    tile.source.metatile_id = *block;
                    tile.source.subtile_column = local_column as u8;
                    tile.source.subtile_row = local_row as u8;
                }
            }
        }
        for (local_row, drawing) in [[0x0a, 0x0b], [0x1a, 0x1b]].into_iter().enumerate() {
            for (local_column, tile_index) in drawing.into_iter().enumerate() {
                tiles[(local_row + 1) * 20 + 4 + local_column]
                    .source
                    .tile_index = tile_index;
            }
        }
        for (row, drawing) in [
            [0x06, 0x07, 0x11, 0x11],
            [0x16, 0x17, 0x0e, 0x0f],
            [0x08, 0x09, 0x3a, 0x3b],
        ]
        .into_iter()
        .enumerate()
        {
            for (column, index) in drawing.into_iter().enumerate() {
                tiles[(row + 1) * 20 + 8 + column].source.tile_index = index;
            }
        }
        for (local_row, drawing) in [[0x4c, 0x4d], [0x5c, 0x5d]].iter().enumerate() {
            for (local_column, tile) in drawing.iter().enumerate() {
                tiles[local_row * 20 + 18 + local_column].source.tile_index = *tile;
            }
        }
        tiles[17].source.tile_index = 0x01;
        let cells = tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 20,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 80];
        append_north_wall_courses(&mut mesh, &cells, &geometry, &mut claimed).expect("wall course");
        for row in 1..=2 {
            for column in 4..=5 {
                assert!(!claimed[row * 20 + column]);
            }
        }
        for row in 1..=3 {
            for column in 8..12 {
                assert!(
                    !claimed[row * 20 + column],
                    "wall must not suppress the TV/cabinet volume"
                );
            }
        }
        for (positions, normals) in mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
        {
            if normals[0] == [0.0, 0.0, 1.0] {
                assert!(
                    positions.iter().all(|p| p[0] <= 18.0 * 8.0),
                    "wall must not occlude the stair opening"
                );
            }
        }
        assert!(claimed[0]);
        assert!(claimed[3 * 20 + 19]);
    }
}
