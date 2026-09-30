#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::prelude::{Handle, Image, UVec2, Vec2};
    use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldFrame};

    use super::*;

    #[test]
    fn complete_rock_formation_folds_side_strips_inside_its_plot() {
        let geometry = GridGeometry {
            width: 16,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = BuildingPlacement {
            column: 0,
            row: 0,
            width: 16,
            height: 8,
            roof_rows: 6,
            ground_tile_index: 0,
        };
        let mut mesh = TerrainMeshData::default();
        append_kanto_cliff_cap(
            &mut mesh,
            &geometry,
            placement,
            &[],
            48,
            128,
            0.0,
            0.0,
            crate::cave::MOUND_FACE_HEIGHT,
            1.0,
            1.0,
        );
        assert!(mesh.textured.positions.iter().all(|p| p[0] >= 16.0
            && p[0] <= 112.0
            && p[2] >= 0.0
            && p[2] <= 48.0
            && p[1] <= 16.0));
        assert!(
            mesh.textured
                .normals
                .iter()
                .all(|n| n[1] == 0.0 || n[1] == 1.0)
        );
    }

    fn source(metatile_id: u16, subtile_column: u8, subtile_row: u8) -> VisualTileSource {
        source_with_tile(metatile_id, subtile_column, subtile_row, 0x06)
    }

    pub(super) fn source_with_tile(
        metatile_id: u16,
        subtile_column: u8,
        subtile_row: u8,
        tile_index: u16,
    ) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("johto"),
            metatile_id,
            subtile_column,
            subtile_row,
            tile_index,
        }
    }

    fn source_for_tileset(
        tileset_id: &str,
        metatile_id: u16,
        subtile_column: u8,
        subtile_row: u8,
        tile_index: u16,
    ) -> VisualTileSource {
        let mut source = source_with_tile(metatile_id, subtile_column, subtile_row, tile_index);
        source.tileset_id = Arc::from(tileset_id);
        source
    }

    pub(super) fn frame(
        width: u32,
        height: u32,
        sources: Vec<VisualTileSource>,
    ) -> VisualWorldFrame {
        assert_eq!(sources.len(), (width * height) as usize);
        VisualWorldFrame {
            active: true,
            // Generic mesher fixtures must not opt into a named map's
            // authored scenery/material override. Map-specific tests opt in.
            map_id: Arc::from("UnmodeledTestMap"),
            terrain_revision: 1,
            grid_origin: bevy::prelude::IVec2::ZERO,
            map_texture: Handle::<Image>::weak_from_u128(1),
            center: Vec2::ZERO,
            viewport_size: Vec2::new(width as f32 * 8.0, height as f32 * 8.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(width, height),
            tiles: sources
                .into_iter()
                .enumerate()
                .map(|(index, source)| VisualTile {
                    column: index as u32 % width,
                    row: index as u32 / width,
                    source,
                    texture: Handle::<Image>::weak_from_u128(index as u128 + 10),
                    priority: false,
                })
                .collect(),
            actors: Vec::new(),
        }
    }

    fn flat_source() -> VisualTileSource {
        source(0x01, 0, 0)
    }

    #[test]
    fn viewport_edge_has_no_generated_skirt() {
        let mesh = build_terrain_mesh(&frame(1, 1, vec![flat_source()]))
            .expect("one unknown cell should remain a flat surface");
        assert_eq!(mesh.textured.quad_count(), 1);
        assert_eq!(mesh.solid.quad_count(), 0);
    }

    #[test]
    fn player_bedroom_wall_stays_compact_and_keeps_stairwell_open() {
        let mut sources = Vec::new();
        for row in 0..2 {
            for column in 0..16 {
                sources.push(source_for_tileset(
                    "players_room",
                    0x04,
                    (column % 4) as u8,
                    row as u8,
                    0x01,
                ));
            }
        }
        for row in 0..2 {
            for col in 0..2 {
                sources[row * 16 + 14 + col] = source_for_tileset(
                    "players_room",
                    0x1f,
                    (col + 2) as u8,
                    row as u8,
                    [[0x40, 0x41], [0x50, 0x51]][row][col],
                );
            }
        }
        let frame = frame(16, 2, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes = vec![CellShape::Flat; cells.len()];
        let geometry = GridGeometry {
            width: 16,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; cells.len()];
        append_player_room_wall(&mut mesh, &cells, &shapes, &geometry, &mut claimed)
            .expect("complete player-room wall course should mesh");

        let max_height = mesh
            .textured
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(max_height, 32.0);
        assert_eq!(mesh.textured.quad_count(), 14 * 4);
        assert!(
            mesh.textured.positions.iter().all(|p| p[0] <= 112.0),
            "wall must not cover the descending flight"
        );
    }

    #[test]
    fn traditional_gift_shop_shelf_uses_two_top_rows_and_two_front_rows() {
        let drawing = [
            [0x06, 0x07, 0x07, 0x20],
            [0x16, 0x17, 0x17, 0x30],
            [0x21, 0x27, 0x27, 0x28],
            [0x31, 0x37, 0x37, 0x38],
        ];
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                sources.push(source_for_tileset(
                    "traditional_house",
                    0x02,
                    column,
                    row,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
        }
        // The dedicated mesher needs one faithful floor sample outside the
        // shelf drawing. Use a fifth column without changing shelf topology.
        let mut expanded = Vec::new();
        for row in 0..4 {
            expanded.extend(
                sources[usize::from(row) * 4..usize::from(row) * 4 + 4]
                    .iter()
                    .cloned(),
            );
            expanded.push(source_for_tileset("traditional_house", 0x20, 0, row, 0x50));
        }
        let shelf_frame = frame(5, 4, expanded);
        let cells = shelf_frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 5,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = traditional_gift_shop_shelf_placements("MahoganyMart1F", &cells, &geometry)
            .into_iter()
            .next()
            .expect("complete shelf placement");
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; cells.len()];
        let mut samples = TerrainImageSamples::default();
        for tile in &shelf_frame.tiles {
            let mut rgba = [210, 210, 210, 255].repeat(64);
            if tile.source.subtile_row >= 2 {
                // A small framed display field exercises the same measured
                // relief path as the live shelf art.
                for y in 2..6 {
                    for x in 2..6 {
                        let offset = (y * 8 + x) * 4;
                        let value = if x == 2 || x == 5 || y == 2 || y == 5 {
                            0
                        } else {
                            120
                        };
                        rgba[offset..offset + 3].fill(value);
                    }
                }
            }
            samples
                .pixels
                .insert(tile.texture.id(), TileImageSample::Rgba(rgba));
        }
        append_traditional_gift_shop_shelf(
            &mut mesh,
            &samples,
            &cells,
            &vec![CellShape::Flat; cells.len()],
            &geometry,
            placement,
            &mut claimed,
        )
        .expect("gift-shop shelf should mesh");
        let max_height = mesh
            .textured
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(max_height, 16.0);
        assert!(mesh.textured.quad_count() > 32);
        assert!(
            mesh.textured
                .positions
                .iter()
                .any(|position| position[2] < 16.0),
            "enclosed merchandise fields sit behind their proud frames"
        );
        assert_eq!(claimed.iter().filter(|claimed| **claimed).count(), 16);
    }

    #[test]
    fn cave_keeps_a_continuous_faithful_floor_below_raised_geometry() {
        let mesh = build_terrain_mesh(&frame(
            1,
            1,
            vec![source_for_tileset("cave", 0x01, 0, 0, 0x16)],
        ))
        .expect("cave floor should mesh");
        assert_eq!(
            mesh.textured.quad_count(),
            2,
            "one faithful cave underlay plus the visible authored cell"
        );
        assert_eq!(mesh.solid.quad_count(), 0);
    }

    #[test]
    fn game_corner_stool_is_recognized_only_as_a_complete_two_by_two_drawing() {
        let sources = vec![
            source_for_tileset("game_corner", 0x05, 2, 0, 0x0a),
            source_for_tileset("game_corner", 0x05, 3, 0, 0x0b),
            source_for_tileset("game_corner", 0x05, 2, 1, 0x1a),
            source_for_tileset("game_corner", 0x05, 3, 1, 0x1b),
        ];
        let complete = frame(2, 2, sources.clone());
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            casino_stool_placements(&cells, &geometry),
            vec![CasinoStoolPlacement { column: 0, row: 0 }]
        );

        let mut incomplete_sources = sources;
        incomplete_sources[3].tile_index = 0x01;
        let incomplete = frame(2, 2, incomplete_sources);
        let incomplete_cells: Vec<_> = incomplete.tiles.iter().collect();
        assert!(casino_stool_placements(&incomplete_cells, &geometry).is_empty());
    }

    #[test]
    fn adjacent_house_stool_is_not_mistaken_for_a_partial_table() {
        let sources = vec![
            source_for_tileset("house", 0x01, 0, 2, 0x02),
            source_for_tileset("house", 0x01, 1, 2, 0x03),
            source_for_tileset("house", 0x01, 2, 2, 0x26),
            source_for_tileset("house", 0x01, 3, 2, 0x27),
            source_for_tileset("house", 0x01, 0, 3, 0x12),
            source_for_tileset("house", 0x01, 1, 3, 0x13),
            source_for_tileset("house", 0x01, 2, 3, 0x36),
            source_for_tileset("house", 0x01, 3, 3, 0x2f),
        ];
        let frame = frame(4, 2, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            house_furniture_placements(&cells, &geometry),
            vec![HouseFurniturePlacement {
                column: 0,
                row: 0,
                kind: crate::house::FurnitureKind::Stool,
            }]
        );
        assert!(house_table_placements(&cells, &geometry).is_empty());
    }

    #[test]
    fn ordinary_house_table_is_one_complete_four_by_four_drawing() {
        let drawing = [
            [
                (0x01, 2, 2, 0x26),
                (0x01, 3, 2, 0x27),
                (0x02, 0, 2, 0x27),
                (0x02, 1, 2, 0x29),
            ],
            [
                (0x01, 2, 3, 0x36),
                (0x01, 3, 3, 0x2f),
                (0x02, 0, 3, 0x2f),
                (0x02, 1, 3, 0x39),
            ],
            [
                (0x0c, 2, 0, 0x05),
                (0x0c, 3, 0, 0x2f),
                (0x0d, 0, 0, 0x2f),
                (0x0d, 1, 0, 0x15),
            ],
            [
                (0x0c, 2, 1, 0x3c),
                (0x0c, 3, 1, 0x3a),
                (0x0d, 0, 1, 0x3a),
                (0x0d, 1, 1, 0x3b),
            ],
        ];
        let sources = drawing
            .into_iter()
            .flat_map(|tiles| {
                tiles
                    .into_iter()
                    .map(|(metatile, subtile_column, subtile_row, tile)| {
                        source_for_tileset("house", metatile, subtile_column, subtile_row, tile)
                    })
            })
            .collect();
        let complete = frame(4, 4, sources);
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            house_table_placements(&cells, &geometry),
            vec![HouseTablePlacement {
                column: 0,
                row: 0,
                ground_tile_index: crate::house::HOUSE_FLOOR_TILE,
                height_pixels: 6.0,
            }]
        );
        let coverage = audit_cell_coverage_on_map("BillsHouse", &complete.tiles, 4, 4)
            .expect("canonical shared-house table coverage");
        assert!(
            coverage
                .iter()
                .all(|kind| *kind == CellCoverageKind::Raised)
        );
    }

    #[test]
    fn player_family_table_is_one_complete_four_by_four_drawing() {
        let drawing = [
            [0x23, 0x22, 0x22, 0x24],
            [0x25, 0x15, 0x15, 0x35],
            [0x25, 0x15, 0x15, 0x35],
            [0x33, 0x32, 0x32, 0x34],
        ];
        let sources = drawing
            .into_iter()
            .enumerate()
            .flat_map(|(row, tiles)| {
                tiles.into_iter().enumerate().map(move |(column, tile)| {
                    source_for_tileset("players_house", 0x08, column as u8, row as u8, tile)
                })
            })
            .collect();
        let complete = frame(4, 4, sources);
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            house_table_placements(&cells, &geometry),
            vec![HouseTablePlacement {
                column: 0,
                row: 0,
                ground_tile_index: crate::house::HOUSE_FLOOR_TILE,
                height_pixels: 6.0,
            }]
        );
    }

    #[test]
    fn traditional_low_table_is_one_complete_four_by_four_drawing() {
        let drawing = [
            [0x23, 0x22, 0x22, 0x24],
            [0x42, 0x15, 0x15, 0x43],
            [0x42, 0x15, 0x15, 0x43],
            [0x33, 0x32, 0x32, 0x34],
        ];
        let sources = drawing
            .into_iter()
            .enumerate()
            .flat_map(|(row, tiles)| {
                tiles.into_iter().enumerate().map(move |(column, tile)| {
                    source_for_tileset("traditional_house", 0x1c, column as u8, row as u8, tile)
                })
            })
            .collect();
        let complete = frame(4, 4, sources);
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            house_table_placements(&cells, &geometry),
            vec![HouseTablePlacement {
                column: 0,
                row: 0,
                ground_tile_index: crate::house::TRADITIONAL_HOUSE_FLOOR_TILE,
                height_pixels: 4.0,
            }]
        );
    }

    #[test]
    fn dark_cave_diagonal_is_one_complete_corner_not_four_tile_boxes() {
        let sources = vec![
            source_for_tileset("dark_cave", 0x10, 2, 2, 0x0a),
            source_for_tileset("dark_cave", 0x10, 3, 2, 0x26),
            source_for_tileset("dark_cave", 0x10, 2, 3, 0x17),
            source_for_tileset("dark_cave", 0x10, 3, 3, 0x0a),
        ];
        let complete = frame(2, 2, sources.clone());
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            diagonal_cave_corner_placements(&cells, &geometry),
            vec![DiagonalCaveCornerPlacement {
                column: 0,
                row: 0,
                corner: crate::cave::DiagonalCorner::SouthEast,
            }]
        );

        let mut incomplete_sources = sources;
        incomplete_sources[3].tile_index = 0x16;
        let incomplete = frame(2, 2, incomplete_sources);
        let incomplete_cells: Vec<_> = incomplete.tiles.iter().collect();
        assert!(diagonal_cave_corner_placements(&incomplete_cells, &geometry).is_empty());
    }

    #[test]
    fn mixed_block_35_reuses_the_complete_diagonal_below_its_loose_rock() {
        let sources = vec![
            source_for_tileset("dark_cave", 0x35, 2, 2, 0x0a),
            source_for_tileset("dark_cave", 0x35, 3, 2, 0x26),
            source_for_tileset("dark_cave", 0x35, 2, 3, 0x17),
            source_for_tileset("dark_cave", 0x35, 3, 3, 0x0a),
        ];
        let frame = frame(2, 2, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(
            diagonal_cave_corner_placements(&cells, &geometry),
            vec![DiagonalCaveCornerPlacement {
                column: 0,
                row: 0,
                corner: crate::cave::DiagonalCorner::SouthEast,
            }]
        );
    }

    #[test]
    fn cave_diagonal_corner_is_a_closed_rock_prism() {
        let complete = frame(
            3,
            2,
            vec![
                source_for_tileset("cave", 0x10, 2, 2, 0x0a),
                source_for_tileset("cave", 0x10, 3, 2, 0x26),
                source_for_tileset("cave", 0x01, 0, 0, 0x16),
                source_for_tileset("cave", 0x10, 2, 3, 0x17),
                source_for_tileset("cave", 0x10, 3, 3, 0x0a),
                source_for_tileset("cave", 0x01, 1, 0, 0x16),
            ],
        );
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 3,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = diagonal_cave_corner_placements(&cells, &geometry)[0];
        let mut mesh = TerrainMeshData::default();
        let shapes = vec![CellShape::Flat; cells.len()];
        let mut claimed = vec![false; cells.len()];

        append_diagonal_cave_corner(
            &mut mesh,
            &cells,
            &shapes,
            &geometry,
            placement,
            &mut claimed,
        )
        .expect("a complete cave corner with a ground sample should mesh");

        assert_eq!(
            mesh.solid.quad_count(),
            0,
            "diagonal closures must use live cave art, not solid-color fins"
        );
        assert_eq!(
            mesh.textured.indices.len() / 3,
            16,
            "ground, cap, diagonal face, and two live edge-strip closures"
        );
        assert_eq!(&claimed[..2], &[true, true]);
        assert_eq!(&claimed[3..5], &[true, true]);
    }

    #[test]
    fn coverage_auditor_reports_game_corner_stools_as_cutout_cards() {
        let sources = vec![
            source_for_tileset("game_corner", 0x05, 2, 0, 0x0a),
            source_for_tileset("game_corner", 0x05, 3, 0, 0x0b),
            source_for_tileset("game_corner", 0x05, 2, 1, 0x1a),
            source_for_tileset("game_corner", 0x05, 3, 1, 0x1b),
        ];
        let frame = frame(2, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("GoldenrodGameCorner", &frame.tiles, 2, 2)
                .expect("complete casino stool should audit"),
            vec![CellCoverageKind::Cutout; 4]
        );
    }

    #[test]
    fn coverage_auditor_reports_game_corner_machine_banks_as_individual_cards() {
        let drawing = [[0xa0, 0xa1], [0x90, 0x91]];
        let mut sources = Vec::new();
        for source_row in 0..4_u8 {
            for source_column in 0..4_u8 {
                let tile = drawing[usize::from(source_row % 2)][usize::from(source_column % 2)]
                    + 2 * u16::from(source_column / 2);
                sources.push(source_for_tileset(
                    "game_corner",
                    0x07,
                    source_column,
                    source_row,
                    tile,
                ));
            }
        }
        for column in 0..4_u8 {
            sources.push(source_for_tileset("game_corner", 0x01, column, 0, 0x01));
        }
        let frame = frame(4, 5, sources);
        let mut expected = vec![CellCoverageKind::Cutout; 16];
        expected.extend([CellCoverageKind::Flat; 4]);
        assert_eq!(
            audit_cell_coverage_on_map("GoldenrodGameCorner", &frame.tiles, 4, 5)
                .expect("complete machine bank should audit"),
            expected
        );
    }

    #[test]
    fn coverage_auditor_reports_center_healing_console_as_one_cutout_group() {
        let sources = vec![
            source_for_tileset("pokecenter", 0x02, 0, 2, 0x0a),
            source_for_tileset("pokecenter", 0x02, 1, 2, 0x0b),
            source_for_tileset("pokecenter", 0x02, 0, 3, 0x1a),
            source_for_tileset("pokecenter", 0x02, 1, 3, 0x1b),
        ];
        let frame = frame(2, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("PewterPokecenter1F", &frame.tiles, 2, 2)
                .expect("complete healing console should audit"),
            vec![CellCoverageKind::Cutout; 4]
        );
    }

    #[test]
    fn coverage_auditor_reports_player_house_bookcases_as_cutout_geometry() {
        let drawing = [
            [0x0e, 0x0f, 0x0e, 0x0f],
            [0x1e, 0x1f, 0x2e, 0x2f],
            [0x2e, 0x2f, 0x08, 0x09],
            [0x18, 0x19, 0x3a, 0x3b],
        ];
        let mut sources = Vec::new();
        for row in 0..4_u8 {
            for column in 0..4_u8 {
                sources.push(source_for_tileset(
                    "players_house",
                    0x1b,
                    column,
                    row,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
        }
        let frame = frame(4, 4, sources);
        assert_eq!(
            audit_cell_coverage_on_map("CopycatsHouse1F", &frame.tiles, 4, 4)
                .expect("complete paired player-house bookcases should audit"),
            vec![CellCoverageKind::Cutout; 16]
        );
    }

    #[test]
    fn coverage_auditor_reports_player_bedroom_fixture_bank_as_cutout_geometry() {
        let drawing = [[0x05, 0x06], [0x15, 0x16], [0x25, 0x26], [0x35, 0x36]];
        let mut sources = Vec::new();
        for row in 0..4_u8 {
            for column in 0..2_u8 {
                sources.push(source_for_tileset(
                    "players_room",
                    0x03,
                    column + 2,
                    row,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
        }
        let frame = frame(2, 4, sources);
        assert_eq!(
            audit_cell_coverage_on_map("PlayersHouse2F", &frame.tiles, 2, 4)
                .expect("complete player-room fixture bank should audit"),
            vec![CellCoverageKind::Cutout; 8]
        );
    }

    #[test]
    fn coverage_auditor_reports_complete_player_house_wall_course_as_facade() {
        let blocks = [0x07, 0x0f, 0x11, 0x05, 0x0a];
        let mut sources = Vec::new();
        for row in 0..4_u8 {
            for block in blocks {
                for column in 0..4_u8 {
                    let tile = if block == 0x0a && column >= 2 && row < 2 {
                        [[0x4c, 0x4d], [0x5c, 0x5d]][usize::from(row)][usize::from(column - 2)]
                    } else {
                        0x11
                    };
                    sources.push(source_for_tileset(
                        "players_house",
                        block,
                        column as u8,
                        row,
                        tile,
                    ));
                }
            }
        }
        let frame = frame(20, 4, sources);
        let coverage = audit_cell_coverage_on_map("PlayersHouse1F", &frame.tiles, 20, 4)
            .expect("complete player-house wall course should audit");
        for row in 0..4 {
            for column in 0..20 {
                let expected = if (18..20).contains(&column) && row < 2 {
                    CellCoverageKind::Ramp
                } else {
                    CellCoverageKind::Facade
                };
                assert_eq!(coverage[row * 20 + column], expected);
            }
        }
    }

    #[test]
    fn coverage_auditor_reports_wise_trio_divider_as_one_cutout_group() {
        let mut sources = Vec::new();
        for row in 0..2_u8 {
            for column in 0..4_u8 {
                sources.push(source_for_tileset(
                    "traditional_house",
                    0x28,
                    column,
                    row,
                    if row == 0 { 0x40 } else { 0x41 },
                ));
            }
        }
        let frame = frame(4, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("WiseTriosRoom", &frame.tiles, 4, 2)
                .expect("complete Wise Trio divider should audit"),
            vec![CellCoverageKind::Cutout; 8]
        );
    }

    #[test]
    fn coverage_auditor_reports_mr_pokemon_work_counter_as_one_raised_surface() {
        let drawing = [[0x02, 0x03, 0x04, 0x05], [0x12, 0x13, 0x14, 0x15]];
        let mut sources = Vec::new();
        for row in 0..2_u8 {
            for column in 0..4_u8 {
                sources.push(source_for_tileset(
                    "facility",
                    0x28,
                    column,
                    row,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
        }
        let frame = frame(4, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("MrPokemonsHouse", &frame.tiles, 4, 2)
                .expect("complete work counter should audit"),
            vec![CellCoverageKind::Raised; 8]
        );
    }

    #[test]
    fn trainer_house_basement_stair_uses_eight_treads_over_faithful_floor() {
        let sources = vec![
            source_for_tileset("facility", 0x04, 2, 0, 0x10),
            source_for_tileset("facility", 0x04, 3, 0, 0x11),
            source_for_tileset("facility", 0x04, 2, 1, 0x20),
            source_for_tileset("facility", 0x04, 3, 1, 0x21),
            source_for_tileset("facility", 0x00, 0, 0, 0x26),
            source_for_tileset("facility", 0x00, 1, 0, 0x26),
        ];
        let frame = frame(2, 3, sources);
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 2,
            height: 3,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; 6];
        append_house_stairs(
            &mut mesh,
            "TrainerHouseB1F",
            &cells,
            &geometry,
            &mut claimed,
        );
        assert_eq!(&claimed[..4], &[true; 4]);
        assert_eq!(
            mesh.textured
                .normals
                .chunks_exact(4)
                .filter(|face| face[0] == [0.0, 1.0, 0.0])
                .count(),
            20,
            "four faithful floor cells plus eight two-row stair treads"
        );
    }

    #[test]
    fn ice_mass_exposed_sides_continue_native_texture_courses() {
        let frame = frame(
            4,
            4,
            (0..16)
                .map(|i| {
                    source_for_tileset(
                        "ice_path",
                        0,
                        (i % 4) as u8,
                        (i / 4) as u8,
                        crate::ice_path::CAVE_GROUND_TILE,
                    )
                })
                .collect(),
        );
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData::default();
        append_ice_path_closed_rock_mass(&mut mesh, &cells, &geometry, 0, 0, &mut [false; 16])
            .unwrap();
        assert!(
            mesh.solid.positions.is_empty(),
            "ice sides must not use the generic brown rock color"
        );
        for normal in [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, -1.0]] {
            assert!(mesh.textured.normals.contains(&normal));
        }
    }

    #[test]
    fn kitchen_cooktop_folds_above_a_closed_eight_pixel_cabinet() {
        let frame = frame(
            2,
            2,
            vec![
                source_for_tileset("players_house", 0x07, 0, 2, 0x50),
                source_for_tileset("players_house", 0x07, 1, 2, 0x51),
                source_for_tileset("players_house", 0x07, 0, 3, 0x52),
                source_for_tileset("players_house", 0x07, 1, 3, 0x53),
            ],
        );
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = players_house_upright_fixture_placements(&cells, &geometry).remove(0);
        let mut mesh = TerrainMeshData::default();
        let mut claimed = [false; 4];
        append_kitchen_fixture(&mut mesh, &geometry, placement, &mut claimed);
        assert_eq!(claimed, [true; 4]);
        assert!(
            mesh.textured
                .positions
                .iter()
                .all(|p| p[1] >= 0.0 && p[1] <= 8.0)
        );
        let mut area = [0.0; 3];
        for (vertices, normals) in mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
        {
            let a = bevy::prelude::Vec3::from_array(vertices[1])
                - bevy::prelude::Vec3::from_array(vertices[0]);
            let b = bevy::prelude::Vec3::from_array(vertices[3])
                - bevy::prelude::Vec3::from_array(vertices[0]);
            let cross = a.cross(b);
            assert!(cross.dot(bevy::prelude::Vec3::from_array(normals[0])) > 0.0);
            for axis in 0..3 {
                area[axis] += cross[axis];
            }
        }
        assert_eq!(area, [0.0; 3], "cabinet shell must close");
        for (normals, uvs) in mesh
            .textured
            .normals
            .chunks_exact(4)
            .zip(mesh.textured.uvs.chunks_exact(4))
        {
            if normals[0] == [0.0, 1.0, 0.0] {
                assert!(
                    uvs.iter().all(|uv| uv[1] <= 0.5),
                    "burners use upper source band"
                );
            }
            if normals[0] == [0.0, 0.0, 1.0] {
                assert!(
                    uvs.iter().all(|uv| uv[1] >= 0.5),
                    "cabinet uses lower source band"
                );
            }
        }
    }

    #[test]
    fn house_stair_sides_are_closed_and_use_source_paint() {
        let frame = frame(
            2,
            2,
            vec![
                source_for_tileset("players_house", 0x0a, 2, 0, 0x4c),
                source_for_tileset("players_house", 0x0a, 3, 0, 0x4d),
                source_for_tileset("players_house", 0x0a, 2, 1, 0x5c),
                source_for_tileset("players_house", 0x0a, 3, 1, 0x5d),
            ],
        );
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData::default();
        append_house_stairs(
            &mut mesh,
            "PlayersHouse1F",
            &cells,
            &geometry,
            &mut [false; 4],
        );
        for normal in [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ] {
            assert!(
                mesh.textured.normals.contains(&normal),
                "missing painted stair face {normal:?}"
            );
        }
        assert!(
            mesh.solid.positions.is_empty(),
            "stair paint must come from the authored drawing"
        );
    }

    #[test]
    fn player_bed_is_one_level_mattress_not_a_sloped_card() {
        let drawing = [[0x03, 0x04], [0x13, 0x14], [0x23, 0x24], [0x33, 0x34]];
        let mut sources = Vec::new();
        for row in 0..4_u8 {
            for column in 0..2_u8 {
                sources.push(source_for_tileset(
                    "players_room",
                    0x1b,
                    column,
                    row,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
            sources.push(source_for_tileset("players_room", 0x01, 3, row, 0x01));
        }
        let frame = frame(3, 4, sources);
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 3,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = player_bed_placements(&cells, &geometry)
            .into_iter()
            .next()
            .expect("complete bedroom bed placement");
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; cells.len()];
        append_player_bed_card(
            &mut mesh,
            &cells,
            &vec![CellShape::Flat; cells.len()],
            &geometry,
            placement,
            &mut claimed,
        )
        .expect("bed should mesh");
        let raised_heights = mesh
            .textured
            .positions
            .iter()
            .map(|position| position[1])
            .filter(|height| *height > 0.0)
            .collect::<Vec<_>>();
        assert!(!raised_heights.is_empty());
        assert!(raised_heights.iter().all(|height| *height == 7.0));
        for row in 0..4 {
            for column in 0..2 {
                assert_eq!(mesh.footing_heights[row * 3 + column], 7.0);
            }
        }
        assert_eq!(mesh.textured.quad_count(), 18);
        assert_eq!(
            mesh.solid.quad_count(),
            3,
            "only the hidden head and narrow flanks use neutral structure; the visible foot is source art"
        );
        assert_eq!(claimed.iter().filter(|claimed| **claimed).count(), 8);
    }

    #[test]
    fn player_family_table_exports_its_six_pixel_visual_support() {
        let drawing = [
            [0x23, 0x22, 0x22, 0x24],
            [0x25, 0x15, 0x15, 0x35],
            [0x25, 0x15, 0x15, 0x35],
            [0x33, 0x32, 0x32, 0x34],
        ];
        let mut sources = Vec::new();
        for row in 0..4_u8 {
            for column in 0..4_u8 {
                sources.push(source_for_tileset(
                    "players_house",
                    0x25,
                    column,
                    row,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
            sources.push(source_for_tileset("players_house", 0x03, 0, row, 0x01));
        }
        let frame = frame(5, 4, sources);
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 5,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = house_table_placements(&cells, &geometry)
            .into_iter()
            .next()
            .expect("complete player-family table placement");
        let mut mesh = TerrainMeshData::default();
        let mut claimed = vec![false; cells.len()];
        append_house_table(
            &mut mesh,
            &cells,
            &vec![CellShape::Flat; cells.len()],
            &geometry,
            placement,
            &mut claimed,
        )
        .expect("table should mesh");
        for row in 0..4 {
            for column in 0..4 {
                assert_eq!(mesh.footing_heights[row * 5 + column], 6.0);
            }
        }
        assert_eq!(mesh.footing_heights[4], 0.0);
    }

    #[test]
    fn coverage_auditor_reports_ice_path_boulders_as_props_not_trees() {
        let sources = vec![
            source_for_tileset("ice_path", 0x1a, 2, 0, 0x82),
            source_for_tileset("ice_path", 0x1a, 3, 0, 0x83),
            source_for_tileset("ice_path", 0x1a, 2, 1, 0x92),
            source_for_tileset("ice_path", 0x1a, 3, 1, 0x93),
        ];
        let frame = frame(2, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("IcePath1F", &frame.tiles, 2, 2)
                .expect("complete Ice Path boulder should audit"),
            vec![CellCoverageKind::Cutout; 4]
        );
    }

    #[test]
    fn coverage_auditor_reports_ice_path_edge_rocks_as_complete_props() {
        let sources = vec![
            source_for_tileset("ice_path", 0x14, 0, 0, 0xc4),
            source_for_tileset("ice_path", 0x14, 1, 0, 0xc5),
            source_for_tileset("ice_path", 0x14, 0, 1, 0xd4),
            source_for_tileset("ice_path", 0x14, 1, 1, 0xd5),
        ];
        let frame = frame(2, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("IcePathB2FBlackthornSide", &frame.tiles, 2, 2)
                .expect("complete Ice Path edge rock should audit"),
            vec![CellCoverageKind::Cutout; 4]
        );
    }

    #[test]
    fn coverage_auditor_reports_complete_ice_mass_as_one_raised_platform() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                sources.push(source_for_tileset(
                    "ice_path",
                    0x19,
                    column,
                    row,
                    0x84 + u16::from(row) * 0x10 + u16::from(column),
                ));
            }
        }
        let frame = frame(4, 4, sources);
        assert_eq!(
            audit_cell_coverage_on_map("IcePath1F", &frame.tiles, 4, 4)
                .expect("complete Ice Path mass should audit"),
            vec![CellCoverageKind::Raised; 16]
        );
    }

    #[test]
    fn coverage_auditor_reports_each_complete_cave_rock_as_a_prop() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                let drawing = [[0x0c, 0x0d], [0x1c, 0x1d]];
                sources.push(source_for_tileset(
                    "dark_cave",
                    0x1d,
                    column,
                    row,
                    drawing[usize::from(row % 2)][usize::from(column % 2)],
                ));
            }
        }
        let frame = frame(4, 4, sources);
        assert_eq!(
            audit_cell_coverage_on_map("DarkCaveBlackthornEntrance", &frame.tiles, 4, 4)
                .expect("four complete cave rocks should audit"),
            vec![CellCoverageKind::Cutout; 16]
        );
    }

    #[test]
    fn coverage_auditor_folds_the_complete_barred_cave_shelf() {
        let drawing = [[0x0e, 0x0f], [0x1e, 0x1f]];
        let mut sources = Vec::new();
        for row in 0..2 {
            for column in 0..2 {
                sources.push(source_for_tileset(
                    "dark_cave",
                    0x13,
                    column + 2,
                    row + 2,
                    drawing[usize::from(row)][usize::from(column)],
                ));
            }
        }
        let frame = frame(2, 2, sources);
        assert_eq!(
            audit_cell_coverage_on_map("RockTunnel1F", &frame.tiles, 2, 2)
                .expect("complete barred cave shelf should audit"),
            vec![CellCoverageKind::Ledge; 4]
        );
    }

    #[test]
    fn coverage_auditor_keeps_mirrored_quarter_rock_topology() {
        for (metatile, drawing) in [
            (0x12, [[0x26, 0x26, 0x36, 0x37], [0x26, 0x26, 0x36, 0x37]]),
            (0x30, [[0x36, 0x37, 0x26, 0x26], [0x36, 0x37, 0x26, 0x26]]),
        ] {
            let sources = drawing
                .into_iter()
                .enumerate()
                .flat_map(|(local_row, tiles)| {
                    tiles.into_iter().enumerate().map(move |(column, tile)| {
                        source_for_tileset(
                            "cave",
                            metatile,
                            column as u8,
                            local_row as u8 + 2,
                            tile,
                        )
                    })
                })
                .collect();
            let frame = frame(4, 2, sources);
            let coverage = audit_cell_coverage_on_map("MountMortar1FOutside", &frame.tiles, 4, 2)
                .expect("complete mirrored cave quarter should audit");
            assert_eq!(
                coverage
                    .iter()
                    .filter(|kind| **kind == CellCoverageKind::Raised)
                    .count(),
                4
            );
            assert_eq!(
                coverage
                    .iter()
                    .filter(|kind| **kind == CellCoverageKind::Ledge)
                    .count(),
                4
            );
        }
    }

    #[test]
    fn coverage_auditor_keeps_lateral_hop_edges_one_course_high() {
        for (metatile, drawing) in [
            (
                0x3b,
                [
                    [0x15, 0x01, 0x01, 0x01],
                    [0x15, 0x01, 0x01, 0x01],
                    [0x15, 0x01, 0x01, 0x01],
                    [0x15, 0x01, 0x01, 0x01],
                ],
            ),
            (
                0x3c,
                [
                    [0x01, 0x01, 0x01, 0x17],
                    [0x01, 0x01, 0x01, 0x17],
                    [0x01, 0x01, 0x01, 0x17],
                    [0x01, 0x01, 0x01, 0x17],
                ],
            ),
        ] {
            let sources = drawing
                .into_iter()
                .enumerate()
                .flat_map(|(row, tiles)| {
                    tiles.into_iter().enumerate().map(move |(column, tile)| {
                        source_for_tileset("dark_cave", metatile, column as u8, row as u8, tile)
                    })
                })
                .collect();
            let frame = frame(4, 4, sources);
            let coverage = audit_cell_coverage_on_map("WhirlIslandB2F", &frame.tiles, 4, 4)
                .expect("complete lateral cave hop edge should audit");
            assert_eq!(
                coverage
                    .iter()
                    .filter(|kind| **kind == CellCoverageKind::Ledge)
                    .count(),
                4
            );
            assert_eq!(
                coverage
                    .iter()
                    .filter(|kind| **kind == CellCoverageKind::Raised)
                    .count(),
                12
            );
        }
    }

    #[test]
    fn coverage_auditor_raises_only_the_underground_boundary_halves() {
        for (metatile, boundary_columns) in [(0x0c, 0_usize..2), (0x0e, 2_usize..4)] {
            let mut sources = Vec::new();
            for row in 0..4 {
                for column in 0..4 {
                    let boundary = boundary_columns.contains(&column);
                    sources.push(source_for_tileset(
                        "underground",
                        metatile,
                        column as u8,
                        row as u8,
                        if boundary {
                            0x0c + (column % 2) as u16
                        } else {
                            0x10
                        },
                    ));
                }
            }
            let mut frame = frame(4, 4, sources);
            frame.map_id = Arc::from("GoldenrodDeptStoreB1F");
            let coverage = audit_cell_coverage_on_map("GoldenrodDeptStoreB1F", &frame.tiles, 4, 4)
                .expect("complete underground boundary coverage");
            for row in 0..4 {
                for column in 0..4 {
                    assert_eq!(
                        coverage[row * 4 + column],
                        if boundary_columns.contains(&column) {
                            CellCoverageKind::Raised
                        } else {
                            CellCoverageKind::Flat
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn complete_cave_rock_is_a_voxel_hull_on_the_ground_datum() {
        let complete = frame(
            3,
            2,
            vec![
                source_for_tileset("cave", 0x1d, 0, 0, 0x0c),
                source_for_tileset("cave", 0x1d, 1, 0, 0x0d),
                source_for_tileset("cave", 0x01, 0, 0, 0x01),
                source_for_tileset("cave", 0x1d, 0, 1, 0x1c),
                source_for_tileset("cave", 0x1d, 1, 1, 0x1d),
                source_for_tileset("cave", 0x01, 1, 0, 0x01),
            ],
        );
        let cells: Vec<_> = complete.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 3,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = cave_small_rock_placements(&cells, &geometry)[0];
        assert_eq!(placement.width, 2);
        assert_eq!(placement.height, 2);
        assert_eq!(placement.base_height, 0.0);
        assert!(
            placement.rounded,
            "round rock artwork must occupy a voxel hull"
        );
    }

    #[test]
    fn kanto_path_bollards_are_independent_thin_objects() {
        let mut sources = Vec::new();
        for origin_row in [0, 2] {
            for row in 0..2 {
                for column in 0..2 {
                    sources.push(source_for_tileset(
                        "kanto",
                        0x29,
                        (2 + column) as u8,
                        (origin_row + row) as u8,
                        0x24,
                    ));
                }
            }
        }
        let frame = frame(2, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 2,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        let placements = kanto_round_path_barrier_placements(&cells, &geometry);
        assert_eq!(placements.len(), 2);
        assert_eq!(placements[0].column, 0);
        assert_eq!(placements[0].row, 0);
        assert_eq!(placements[1].column, 0);
        assert_eq!(placements[1].row, 2);
        assert!(placements.iter().all(|placement| !placement.rounded));
        assert!(
            placements
                .iter()
                .all(|placement| placement.card_thickness == 1.0)
        );
    }

    #[test]
    fn mixed_block_34_keeps_two_loose_rocks_off_the_pale_course() {
        let drawing = [
            [0x0c, 0x0d, 0x0c, 0x0d],
            [0x1c, 0x1d, 0x1c, 0x1d],
            [0x26, 0x26, 0x26, 0x26],
            [0x26, 0x26, 0x26, 0x26],
        ];
        let sources = drawing
            .into_iter()
            .enumerate()
            .flat_map(|(row, tiles)| {
                tiles.into_iter().enumerate().map(move |(column, tile)| {
                    source_for_tileset("dark_cave", 0x34, column as u8, row as u8, tile)
                })
            })
            .collect();
        let frame = frame(4, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        let placements = cave_small_rock_placements(&cells, &geometry);
        assert_eq!(placements.len(), 2);
        assert!(
            placements
                .iter()
                .all(|placement| placement.base_height == 0.0)
        );
        assert!(placements.iter().all(|placement| placement.rounded));
        assert_eq!(
            placements
                .iter()
                .map(|placement| (placement.column, placement.row))
                .collect::<Vec<_>>(),
            vec![(0, 0), (2, 0)]
        );
    }

    #[test]
    fn grouped_park_fountain_is_one_continuous_shell_not_pixel_spikes() {
        let geometry = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -32.0,
            origin_z: -32.0,
        };
        let mut mesh = TerrainMeshData::default();
        append_park_fountain(
            &mut mesh,
            &geometry,
            FountainPlacement { column: 2, row: 2 },
        );

        assert_eq!(mesh.solid.quad_count(), 24, "one side per oval segment");
        assert_eq!(
            mesh.textured.quad_count(),
            24,
            "one triangle-fan segment per top"
        );
        let distinct_top_heights = mesh
            .textured
            .positions
            .iter()
            .map(|position| position[1].to_bits())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(distinct_top_heights.len(), 1);
        assert!(
            mesh.solid
                .positions
                .chunks_exact(4)
                .all(|quad| quad[0][1] == quad[3][1] && quad[1][1] == quad[2][1])
        );
    }

    #[test]
    fn shoreline_drop_uses_cropped_ground_art_instead_of_a_solid_wall() {
        let ground = source_for_tileset("kanto", 0x01, 0, 0, 0x2c);
        let water = source_for_tileset("kanto", 0x15, 0, 0, 0x14);
        let mesh = build_terrain_mesh(&frame(2, 1, vec![ground, water]))
            .expect("authored water edge should mesh");

        assert_eq!(mesh.textured.quad_count(), 3);
        assert_eq!(mesh.solid.quad_count(), 0);
        let shoreline_uvs = &mesh.textured.uvs[8..12];
        let min_v = shoreline_uvs
            .iter()
            .map(|uv| uv[1])
            .fold(f32::INFINITY, f32::min);
        let max_v = shoreline_uvs
            .iter()
            .map(|uv| uv[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_v - min_v - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn waterfall_uses_each_source_row_once_on_one_bounded_cave_slope() {
        let mut sources = Vec::new();
        for row in 0..2 {
            for column in 0..3 {
                sources.push(source_for_tileset(
                    "cave",
                    if column < 2 { 0x2c } else { 0x01 },
                    column as u8,
                    row as u8,
                    if column < 2 { 0x40 } else { 0x14 },
                ));
            }
        }
        let mesh = build_terrain_mesh(&frame(3, 2, sources))
            .expect("waterfall and its authored water replacement should mesh");

        assert_eq!(mesh.textured.quad_count(), 11);
        let waterfall_vertices = &mesh.textured.positions[7 * 4..11 * 4];
        let min_y = waterfall_vertices
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = waterfall_vertices
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_z = waterfall_vertices
            .iter()
            .map(|position| position[2])
            .fold(f32::INFINITY, f32::min);
        let max_z = waterfall_vertices
            .iter()
            .map(|position| position[2])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - min_y - crate::cave::CAVE_ROCK_HEIGHT).abs() < f32::EPSILON);
        assert!((max_z - min_z - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cianwood_buoy_relief_keeps_its_base_on_recessed_water() {
        let buoy = source_with_tile(0x34, 0, 0, 0x58);
        let water = source_with_tile(0x34, 2, 0, 0x14);
        let mesh = build_terrain_mesh(&frame(2, 1, vec![buoy, water]))
            .expect("buoy relief should resolve its authored water base");

        assert_eq!(mesh.textured.quad_count(), 2);
        assert!(
            mesh.textured.positions.iter().all(|position| {
                (position[1] - crate::profile::WATER_HEIGHT).abs() < f32::EPSILON
            })
        );
        assert_eq!(mesh.solid.quad_count(), 0);
    }

    #[test]
    fn shore_transition_keeps_its_exact_rock_cap_art() {
        let ground = flat_source();
        let shore = source_with_tile(0x54, 0, 0, 0x4c);
        let water = source_with_tile(0x54, 1, 0, 0x14);
        let mesh = build_terrain_mesh(&frame(3, 1, vec![ground, shore, water]))
            .expect("authored shoreline should mesh");
        let shore_top = &mesh.textured.uvs[4..8];
        let min_u = shore_top
            .iter()
            .map(|uv| uv[0])
            .fold(f32::INFINITY, f32::min);
        let max_u = shore_top
            .iter()
            .map(|uv| uv[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((min_u - 1.0 / 3.0).abs() < f32::EPSILON);
        assert!((max_u - 2.0 / 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn port_ship_keeps_water_holes_and_emits_only_native_height_side_bands() {
        let ship = source_for_tileset("port", 0x18, 0, 2, 0x2b);
        let water = source_for_tileset("port", 0x18, 1, 2, 0x14);
        let mesh = build_terrain_mesh(&frame(2, 1, vec![ship, water]))
            .expect("ship edge beside open port water should mesh");
        assert!(
            mesh.textured.positions[0..4]
                .iter()
                .all(|position| position[1] == crate::port::SHIP_HEIGHT)
        );
        assert!(
            mesh.textured.positions[4..8]
                .iter()
                .all(|position| position[1] == crate::profile::WATER_HEIGHT)
        );
        let side_quads: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(_, normal)| normal[0] != [0.0, 1.0, 0.0])
            .map(|(positions, _)| positions)
            .collect();
        assert!(!side_quads.is_empty());
        assert!(side_quads.iter().all(|quad| {
            let min = quad
                .iter()
                .map(|position| position[1])
                .fold(f32::INFINITY, f32::min);
            let max = quad
                .iter()
                .map(|position| position[1])
                .fold(f32::NEG_INFINITY, f32::max);
            max - min <= SOURCE_TILE_HEIGHT
        }));
    }

    #[test]
    fn flower_frames_change_only_the_separate_animated_mesh() {
        let frame = frame(
            2,
            1,
            vec![
                source_with_tile(0x03, 0, 0, 0x03),
                source_with_tile(0x01, 0, 0, 0x05),
            ],
        );
        let ground = [160, 200, 120, 255].repeat(64);
        let build = |pixel: usize| {
            let mut samples = TerrainImageSamples::default();
            let mut pixels = ground.clone();
            pixels[pixel * 4..pixel * 4 + 4].copy_from_slice(&[20, 0, 10, 255]);
            samples
                .pixels
                .insert(frame.tiles[0].texture.id(), TileImageSample::Rgba(pixels));
            samples.pixels.insert(
                frame.tiles[1].texture.id(),
                TileImageSample::Rgba(ground.clone()),
            );
            build_terrain_mesh_with_samples(&frame, &samples).unwrap()
        };
        let first = build(19);
        let second = build(20);
        assert_eq!(first.textured, second.textured);
        assert_eq!(first.solid, second.solid);
        assert_eq!(first.footing_heights, second.footing_heights);
        assert!(!first.animated_textured.positions.is_empty());
        assert_ne!(
            first.animated_textured.positions,
            second.animated_textured.positions
        );
    }

    #[test]
    fn flower_mask_subtracts_dark_paletted_ground_instead_of_standing_a_full_card() {
        let flower_source = source_with_tile(0x03, 0, 0, 0x03);
        let ground_source = source_with_tile(0x01, 0, 0, 0x05);
        let frame = frame(2, 1, vec![flower_source, ground_source]);
        let flower = &frame.tiles[0];
        let ground = &frame.tiles[1];
        let ground_pixel = [16, 80, 96, 255];
        let mut ground_rgba = ground_pixel.repeat(64);
        let mut flower_rgba = ground_rgba.clone();
        for (x, y) in [(3, 3), (4, 3), (3, 4), (4, 4)] {
            let offset = (y * 8 + x) * 4;
            flower_rgba[offset..offset + 4].copy_from_slice(&[112, 0, 96, 255]);
        }
        let mut images = TerrainImageSamples::default();
        images
            .pixels
            .insert(flower.texture.id(), TileImageSample::Rgba(flower_rgba));
        images.pixels.insert(
            ground.texture.id(),
            TileImageSample::Rgba(std::mem::take(&mut ground_rgba)),
        );

        let removable = decorative_cutout_mask(&images, flower, ground, SolidKind::Flower)
            .expect("paletted flower mask should resolve");
        assert_eq!(removable.iter().filter(|pixel| !**pixel).count(), 4);
        assert!(!removable[3 * 8 + 3]);
        assert!(removable[0]);
    }

    #[test]
    fn flowers_stand_behind_actors_on_the_same_ground_cell() {
        assert_eq!(upright_plane_z(SolidKind::Flower, 8.0, 16.0), 8.0);
        assert_eq!(upright_plane_z(SolidKind::CutTree, 8.0, 16.0), 12.0);
        assert_eq!(upright_plane_z(SolidKind::Prop, 8.0, 16.0), 12.0);
    }

    #[test]
    fn fence_posts_are_deeper_than_sign_plates() {
        assert_eq!(upright_depth(SolidKind::Prop), 2.0);
        assert_eq!(upright_depth(SolidKind::Fence), 6.0);
    }

    #[test]
    fn grass_tile_rows_stand_on_distinct_depth_planes() {
        let mut textured = SurfaceMeshData::default();
        let mut solid = SurfaceMeshData::default();
        let mut removable = [true; 64];
        removable[3 * 8 + 3] = false;
        for plane_z in [4.0, 12.0] {
            append_masked_upright_hull(
                &mut textured,
                &mut solid,
                &removable,
                [0.0, 8.0, 0.0, 8.0, plane_z],
                [0.0, 1.0, 0.0, 1.0],
                SolidKind::Grass,
            )
            .expect("grass tuft should mesh");
        }
        assert!(
            textured
                .positions
                .iter()
                .any(|position| (position[2] - 4.0).abs() < f32::EPSILON)
        );
        assert!(
            textured
                .positions
                .iter()
                .any(|position| (position[2] - 12.0).abs() < f32::EPSILON)
        );
        assert_eq!(upright_depth(SolidKind::Grass), 2.0);
    }

    #[test]
    fn johto_transition_drawing_folds_only_authored_south_face_bands() {
        let mut sources = Vec::new();
        for row in 0..5 {
            for column in 0..4 {
                sources.push(if row < 4 {
                    source_with_tile(
                        0x72,
                        column as u8,
                        row as u8,
                        if row < 2 { 0x3c } else { 0x4c },
                    )
                } else {
                    flat_source()
                });
            }
        }
        let mesh =
            build_terrain_mesh(&frame(4, 5, sources)).expect("authored mountain ledge should mesh");
        let south_faces: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .zip(mesh.textured.uvs.chunks_exact(4))
            .filter(|((_, normal), _)| normal[0][2] > 0.5)
            .map(|((positions, _), uvs)| (positions, uvs))
            .collect();
        assert!(!south_faces.is_empty());
        assert!(south_faces.iter().all(|(positions, _)| {
            let min = positions
                .iter()
                .map(|position| position[1])
                .fold(f32::INFINITY, f32::min);
            let max = positions
                .iter()
                .map(|position| position[1])
                .fold(f32::NEG_INFINITY, f32::max);
            max - min <= SOURCE_TILE_HEIGHT
        }));
    }

    #[test]
    fn blackthorn_transition_corner_folds_connected_south_and_east_faces() {
        let mut sources = Vec::new();
        for row in 0..5 {
            for column in 0..5 {
                sources.push(if row < 4 && column < 4 {
                    source_with_tile(
                        0x6d,
                        column as u8,
                        row as u8,
                        match (column >= 2, row >= 2) {
                            (_, true) => 0x4c,
                            (true, false) => 0x3d,
                            _ => 0x3c,
                        },
                    )
                } else {
                    flat_source()
                });
            }
        }
        let mesh = build_terrain_mesh(&frame(5, 5, sources))
            .expect("authored mountain corner should mesh");
        let south_faces = mesh
            .textured
            .normals
            .chunks_exact(4)
            .filter(|normal| normal[0][2] > 0.5)
            .count();
        let east_faces = mesh
            .textured
            .normals
            .chunks_exact(4)
            .filter(|normal| normal[0][0] > 0.5)
            .count();
        assert!(south_faces > 0);
        assert!(east_faces > 0);
    }

    #[test]
    fn transition_run_uses_authored_directional_wall_courses() {
        let sources = vec![
            source_with_tile(0x69, 2, 0, 0x3d),
            source_with_tile(0x69, 3, 0, 0x3d),
            flat_source(),
            source_with_tile(0x70, 0, 0, 0x3c),
            flat_source(),
        ];
        let mesh = build_terrain_mesh(&frame(5, 1, sources))
            .expect("connected plateau side should use authored wall art");
        let textured_east_courses = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(positions, normals)| {
                normals[0][0] > 0.5 && positions.iter().all(|position| position[0] > 8.0)
            })
            .count();
        let solid_east_faces = mesh
            .solid
            .positions
            .chunks_exact(4)
            .zip(mesh.solid.normals.chunks_exact(4))
            .filter(|(positions, normals)| {
                normals[0][0] > 0.5 && positions.iter().all(|position| position[0] > 8.0)
            })
            .count();
        assert_eq!(textured_east_courses, 2);
        assert_eq!(solid_east_faces, 0);
    }

    #[test]
    fn transition_bank_run_folds_native_front_bands() {
        let sources = vec![
            source_with_tile(0x70, 0, 0, 0x3c),
            flat_source(),
            source_with_tile(0x72, 0, 1, 0x4b),
            flat_source(),
            source_with_tile(0x72, 0, 2, 0x4c),
            flat_source(),
            flat_source(),
            flat_source(),
        ];
        let mesh = build_terrain_mesh(&frame(2, 4, sources))
            .expect("bank column should fold its own source courses");
        let mut v_ranges: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .zip(mesh.textured.uvs.chunks_exact(4))
            .filter(|((_, normals), _)| normals[0][0] > 0.5)
            .map(|(_, uvs)| {
                let min = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
                let max = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
                ((min * 100.0).round() as i32, (max * 100.0).round() as i32)
            })
            .collect();
        v_ranges.sort_unstable();
        v_ranges.dedup();
        assert!(!v_ranges.is_empty());
        assert!(v_ranges.iter().all(|(min, max)| max - min <= 25));
    }

    #[test]
    fn johto_rock_platform_folds_native_courses_without_flared_feet() {
        let tile_indices = [
            0x2b, 0x2c, 0x2c, 0x2d, 0x3b, 0x3c, 0x3c, 0x3d, 0x3b, 0x3c, 0x3c, 0x3d, 0x4b, 0x4c,
            0x4c, 0x4d,
        ];
        let mut sources = vec![flat_source(); 36];
        for (index, tile_index) in tile_indices.into_iter().enumerate() {
            let local_column = index % 4;
            let local_row = index / 4;
            sources[(local_row + 1) * 6 + local_column + 1] =
                source_with_tile(0x0a, local_column as u8, local_row as u8, tile_index);
        }
        let mesh = build_terrain_mesh(&frame(6, 6, sources)).expect("rock platform meshes");
        let raised_tops = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(positions, normals)| {
                normals[0] == [0.0, 1.0, 0.0]
                    && positions.iter().all(|position| {
                        (position[1] - crate::profile::MOUNTAIN_LEDGE_HEIGHT).abs() < f32::EPSILON
                    })
            })
            .count();
        assert_eq!(
            raised_tops, 24,
            "twelve square cells and four three-triangle corner fans rise together"
        );

        let mut raised_top_source_rows: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .zip(mesh.textured.uvs.chunks_exact(4))
            .filter(|((positions, normals), _)| {
                normals[0] == [0.0, 1.0, 0.0]
                    && positions.iter().all(|position| {
                        (position[1] - crate::profile::MOUNTAIN_LEDGE_HEIGHT).abs() < f32::EPSILON
                    })
            })
            .map(|(_, uvs)| {
                (uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min) * 6.0).round() as i32
            })
            .collect();
        raised_top_source_rows.sort_unstable();
        assert_eq!(
            raised_top_source_rows,
            vec![
                1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
            ],
            "the cap preserves the complete authored 4x4 drawing"
        );

        assert!(
            mesh.textured
                .normals
                .iter()
                .all(|normal| normal[1] == 0.0 || normal[1] == 1.0),
            "rock courses must not become inclined trapezoid walls"
        );
        let wall_positions: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(_, normals)| normals[0][1] == 0.0)
            .flat_map(|(positions, _)| positions.iter())
            .collect();
        assert!(!wall_positions.is_empty());
        let caps: Vec<_> = mesh
            .textured
            .positions
            .iter()
            .filter(|p| p[1] == crate::profile::MOUNTAIN_LEDGE_HEIGHT)
            .collect();
        for axis in [0, 2] {
            let low = caps.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min);
            let high = caps
                .iter()
                .map(|p| p[axis])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                wall_positions
                    .iter()
                    .all(|p| p[axis] >= low && p[axis] <= high),
                "walls may not flare outside the authored platform"
            );
        }
    }

    #[test]
    fn johto_modern_uses_the_same_authored_rock_platform_hull() {
        let tile_indices = [
            0x2b, 0x2c, 0x2c, 0x2d, 0x3b, 0x3c, 0x3c, 0x3d, 0x3b, 0x3c, 0x3c, 0x3d, 0x4b, 0x4c,
            0x4c, 0x4d,
        ];
        let mut sources = vec![flat_source(); 36];
        for (index, tile_index) in tile_indices.into_iter().enumerate() {
            let local_column = index % 4;
            let local_row = index / 4;
            let mut source =
                source_with_tile(0x0a, local_column as u8, local_row as u8, tile_index);
            source.tileset_id = Arc::from("johto_modern");
            sources[(local_row + 1) * 6 + local_column + 1] = source;
        }
        let mesh =
            build_terrain_mesh(&frame(6, 6, sources)).expect("Johto Modern rock platform meshes");

        let raised_caps = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(positions, normals)| {
                normals[0] == [0.0, 1.0, 0.0]
                    && positions.iter().all(|position| {
                        (position[1] - crate::profile::MOUNTAIN_LEDGE_HEIGHT).abs() < f32::EPSILON
                    })
            })
            .count();
        assert_eq!(raised_caps, 24);
        assert!(
            mesh.textured
                .normals
                .chunks_exact(4)
                .all(|normals| normals[0][1] == 0.0 || normals[0][1] == 1.0)
        );
    }

    #[test]
    fn adjacent_rock_platforms_remain_one_universal_height() {
        let tile_indices = [
            0x2b, 0x2c, 0x2c, 0x2d, 0x3b, 0x3c, 0x3c, 0x3d, 0x3b, 0x3c, 0x3c, 0x3d, 0x4b, 0x4c,
            0x4c, 0x4d,
        ];
        let mut sources = vec![flat_source(); 60];
        for platform_row in 0..2 {
            for (index, tile_index) in tile_indices.into_iter().enumerate() {
                let local_column = index % 4;
                let local_row = index / 4;
                let row = 1 + platform_row * 4 + local_row;
                sources[row * 6 + local_column + 1] =
                    source_with_tile(0x0a, local_column as u8, local_row as u8, tile_index);
            }
        }
        let mesh = build_terrain_mesh(&frame(6, 10, sources)).expect("joined rocks mesh");
        let cap_faces_at = |height: f32| {
            mesh.textured
                .positions
                .chunks_exact(4)
                .zip(mesh.textured.normals.chunks_exact(4))
                .filter(|(positions, normals)| {
                    normals[0] == [0.0, 1.0, 0.0]
                        && positions
                            .iter()
                            .all(|position| (position[1] - height).abs() < f32::EPSILON)
                })
                .count()
        };
        assert_eq!(
            cap_faces_at(crate::profile::MOUNTAIN_LEDGE_HEIGHT),
            40,
            "the shared seam removes the two touching outer-corner fans"
        );
        assert_eq!(cap_faces_at(crate::profile::MOUNTAIN_LEDGE_HEIGHT * 2.0), 0);
        assert_eq!(
            mesh.footing_heights[2 * 6 + 2],
            crate::profile::MOUNTAIN_LEDGE_HEIGHT,
            "the northern copy remains on the universal platform course"
        );
        assert_eq!(
            mesh.footing_heights[6 * 6 + 2],
            crate::profile::MOUNTAIN_LEDGE_HEIGHT,
            "the southern copy remains on the same platform course"
        );
    }

    #[test]
    fn jump_ledge_does_not_invent_a_plateau_in_neighboring_flat_ground() {
        let mut sources = vec![flat_source(); 5];
        for row in 0..3 {
            sources[row + 1] = source_with_tile(0x57, 1, row as u8, 0x05);
        }
        sources[4] = source_with_tile(0x57, 1, 3, 0x4c);

        let mesh = build_terrain_mesh(&frame(1, 5, sources)).expect("exact jump ledge meshes");
        assert_eq!(
            mesh.footing_heights[0], 0.0,
            "ordinary ground north of the exact ledge drawing must remain flat"
        );
        assert_eq!(
            mesh.footing_heights[1],
            crate::profile::JUMP_LEDGE_HEIGHT,
            "the authored ledge cap retains its single level"
        );
    }

    #[test]
    fn cave_doorway_art_is_not_repeated_on_additional_cliff_tiers() {
        let frame = frame(
            2,
            2,
            vec![
                source_with_tile(0x73, 0, 0, 0x57),
                source_with_tile(0x72, 0, 0, 0x4c),
                source_with_tile(0x73, 0, 1, 0x3c),
                source_with_tile(0x72, 0, 1, 0x3c),
            ],
        );
        let cells: Vec<_> = frame.tiles.iter().collect();
        let band = |band_from_top| CellShape::LedgeBand {
            face: LedgeFace::South,
            plane_subtile: 2,
            band_from_top,
            band_count: 2,
            top_tile_index: 0x3c,
            height: crate::profile::MOUNTAIN_CLIFF_HEIGHT * 2.0,
        };
        let shapes = vec![band(1), band(1), band(0), band(0)];
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let run = BankColumnRun { north: 0, front: 1 };
        assert_eq!(
            authored_bank_face_cell(&cells, &shapes, &geometry, run, 0, Direction::South, 0,),
            Some(0),
            "the authored lowest tier keeps its one doorway"
        );
        assert_eq!(
            authored_bank_face_cell(&cells, &shapes, &geometry, run, 0, Direction::South, 2,),
            Some(1),
            "the repeated upper tier borrows ordinary rock art"
        );
    }

    #[test]
    fn bank_side_ao_darkens_the_ground_crease_and_inside_corner() {
        let shades = side_ao_shades(16.0, 0.0, 0.0, 8.0, true, 1.0);
        assert!(shades[0] < shades[1]);
        assert!(shades[0] < shades[3]);
        assert!(shades[3] < 1.0);
        assert_eq!(shades[2], 1.0);
    }

    #[test]
    fn traditional_roof_rises_to_middle_ridge_and_closes_at_both_eaves() {
        assert_eq!(gabled_roof_height(32.0, 16.0, 0.0, 24.0), 32.0);
        assert_eq!(gabled_roof_height(32.0, 16.0, 12.0, 24.0), 48.0);
        assert_eq!(gabled_roof_height(32.0, 16.0, 24.0, 24.0), 32.0);
    }

    #[test]
    fn explicit_ledge_face_is_not_covered_by_a_generic_solid_side() {
        let sources = vec![
            source_with_tile(0x69, 2, 0, 0x3c),
            source_with_tile(0x69, 3, 0, 0x3d),
            flat_source(),
        ];
        let mesh =
            build_terrain_mesh(&frame(3, 1, sources)).expect("authored east ledge should mesh");
        let generic_faces = mesh
            .solid
            .positions
            .chunks_exact(4)
            .zip(mesh.solid.normals.chunks_exact(4))
            .filter(|(positions, normals)| {
                normals[0] == [1.0, 0.0, 0.0] && positions.iter().all(|position| position[0] == 4.0)
            })
            .count();
        assert_eq!(generic_faces, 0);
    }

    #[test]
    fn compact_facade_bands_are_native_height_and_share_a_plane() {
        let mut sources = Vec::new();
        for row in 0..4 {
            sources.push(source(0x14, 0, row));
            sources.push(flat_source());
        }
        let mesh = build_terrain_mesh(&frame(2, 4, sources)).expect("compact house should mesh");

        let vertical_faces: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(_, normals)| normals[0][1] == 0.0)
            .map(|(positions, _)| positions)
            .collect();
        assert_eq!(vertical_faces.len(), 2);
        for face in &vertical_faces {
            let min_y = face
                .iter()
                .map(|vertex| vertex[1])
                .fold(f32::INFINITY, f32::min);
            let max_y = face
                .iter()
                .map(|vertex| vertex[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(max_y - min_y <= 8.0, "source art was stretched vertically");
        }
        assert_eq!(vertical_faces[0][0][2], vertical_faces[1][0][2]);
    }

    #[test]
    fn generated_sides_are_not_in_the_textured_material_domain() {
        let mesh = build_terrain_mesh(&frame(2, 1, vec![source(0x18, 0, 0), flat_source()]))
            .expect("raised roof beside ground should mesh");

        assert_eq!(
            mesh.textured
                .normals
                .chunks_exact(4)
                .filter(|face| face[0][1] == 0.0)
                .count(),
            0
        );
        assert!(mesh.solid.quad_count() > 0);
        assert!(mesh.solid.uvs.iter().all(|uv| *uv == [0.0, 0.0]));
    }

    #[test]
    fn complete_large_house_is_detected_as_one_authored_placement() {
        let metatiles = [[0x18, 0x19], [0x16, 0x1e]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..8 {
                sources.push(source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                ));
            }
        }
        let frame = frame(8, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -32.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 8,
                height: 8,
                roof_rows: 4,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn forest_entrance_is_one_complete_three_by_three_block_structure() {
        for lower_left in [0x24, 0x25] {
            let metatiles = [
                [0x1d, 0x1e, 0x1f],
                [0x21, 0x22, 0x23],
                [lower_left, 0x26, 0x27],
            ];
            let mut sources = Vec::new();
            for row in 0..12 {
                for column in 0..12 {
                    sources.push(source_for_tileset(
                        "forest",
                        metatiles[row / 4][column / 4],
                        (column % 4) as u8,
                        (row % 4) as u8,
                        0x20,
                    ));
                }
            }
            let frame = frame(12, 12, sources);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let geometry = GridGeometry {
                width: 12,
                height: 12,
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: -48.0,
                origin_z: -48.0,
            };
            assert_eq!(
                outdoor_building_placements(&cells, &geometry),
                vec![BuildingPlacement {
                    column: 0,
                    row: 0,
                    width: 12,
                    height: 12,
                    roof_rows: 4,
                    ground_tile_index: 0x05,
                }]
            );
        }
    }

    #[test]
    fn traditional_three_block_house_is_one_authored_placement() {
        let metatiles = [[0x2c, 0x2a, 0x2d], [0x26, 0x27, 0x2f]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                sources.push(source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                ));
            }
        }
        let frame = frame(12, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -48.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 2,
                width: 12,
                height: 6,
                roof_rows: 4,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn burned_tower_exterior_is_one_authored_roof_and_facade() {
        let metatiles = [[0x20, 0x21], [0x37, 0x3b]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..8 {
                sources.push(source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                ));
            }
        }
        let frame = frame(8, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -32.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 8,
                height: 8,
                roof_rows: 4,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn complete_modern_city_building_is_detected_outside_new_bark() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..8 {
                let mut tile = source(
                    if column < 4 { 0x12 } else { 0x13 },
                    (column % 4) as u8,
                    row as u8,
                );
                tile.tileset_id = Arc::from("johto_modern");
                sources.push(tile);
            }
        }
        let mut frame = frame(8, 4, sources);
        frame.map_id = Arc::from("GoldenrodCity");
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -32.0,
            origin_z: -16.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![
                BuildingPlacement {
                    column: 0,
                    row: 0,
                    width: 4,
                    height: 4,
                    roof_rows: 2,
                    ground_tile_index: 0x06,
                },
                BuildingPlacement {
                    column: 4,
                    row: 0,
                    width: 4,
                    height: 4,
                    roof_rows: 2,
                    ground_tile_index: 0x06,
                },
            ]
        );
    }

    #[test]
    fn goldenrod_department_store_claims_every_storey_as_one_building() {
        let metatiles = [
            [0x18, 0x1f, 0x19],
            [0x27, 0x23, 0x28],
            [0x27, 0x23, 0x28],
            [0x10, 0x17, 0x33],
        ];
        let mut sources = Vec::new();
        for row in 0..16 {
            for column in 0..12 {
                let mut tile = source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                );
                tile.tileset_id = Arc::from("johto_modern");
                sources.push(tile);
            }
        }
        let frame = frame(12, 16, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 16,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 16,
                roof_rows: 2,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn goldenrod_game_corner_facade_is_not_stamped_as_a_second_building() {
        let metatiles = [[0x18, 0x1f, 0x19], [0x10, 0x17, 0x11]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                let mut tile = source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                );
                tile.tileset_id = Arc::from("johto_modern");
                sources.push(tile);
            }
        }
        let frame = frame(12, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 8,
                roof_rows: 4,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn goldenrod_radio_tower_claims_its_complete_landmark_drawing() {
        let metatiles = [[0x25, 0x26], [0x29, 0x2a], [0x2d, 0x2e]];
        let mut sources = Vec::new();
        for row in 0..12 {
            for column in 0..8 {
                let mut tile = source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                );
                tile.tileset_id = Arc::from("johto_modern");
                sources.push(tile);
            }
        }
        let frame = frame(8, 12, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 12,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 8,
                height: 12,
                roof_rows: 2,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn route34_daycare_is_one_complete_modern_building() {
        let metatiles = [[0x18, 0x1f, 0x19], [0x1a, 0x2c, 0x11]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                let mut tile = source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                );
                tile.tileset_id = Arc::from("johto_modern");
                sources.push(tile);
            }
        }
        let mut frame = frame(12, 8, sources);
        frame.map_id = Arc::from("Route34");
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -48.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 8,
                roof_rows: 4,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn blackthorn_closed_mound_is_one_object_not_six_cliff_blocks() {
        let metatiles = [[0x6a, 0x70, 0x6b], [0x6c, 0x72, 0x6d]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                sources.push(source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                ));
            }
        }
        let frame = frame(12, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(johto_closed_mound_origins(&cells, 12, 8), vec![(0, 0)]);
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 8,
                roof_rows: 6,
                ground_tile_index: 0x01,
            }]
        );
    }

    #[test]
    fn ice_path_closed_plateau_reuses_the_complete_mound_object() {
        let metatiles = [[0x04, 0x05, 0x06], [0x0c, 0x0d, 0x0e]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                let mut tile = source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                );
                tile.tileset_id = Arc::from("ice_path");
                tile.tile_index = 0x9a;
                sources.push(tile);
            }
        }
        let frame = frame(12, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(ice_path_plateau_origins(&cells, 12, 8), vec![(0, 0, 12)]);
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 8,
                roof_rows: 6,
                ground_tile_index: 0x9a,
            }]
        );

        let mut wrong_sources = frame.tiles.clone();
        for row in 4..8 {
            for column in 8..12 {
                wrong_sources[row * 12 + column].source.metatile_id = 0x1f;
            }
        }
        let wrong_cells: Vec<_> = wrong_sources.iter().collect();
        assert!(ice_path_plateau_origins(&wrong_cells, 12, 8).is_empty());
    }

    #[test]
    fn ice_path_two_block_island_is_one_rock_formation() {
        let metatiles = [[0x04, 0x06], [0x10, 0x3a]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..8 {
                let mut tile = source(
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                );
                tile.tileset_id = Arc::from("ice_path");
                tile.tile_index = 0x9a;
                sources.push(tile);
            }
        }
        let frame = frame(8, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(ice_path_plateau_origins(&cells, 8, 8), vec![(0, 0, 8)]);
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 8,
                height: 8,
                roof_rows: 6,
                ground_tile_index: 0x9a,
            }]
        );
    }

    #[test]
    fn ice_path_long_plateau_keeps_transition_blocks_inside_one_object() {
        let north = [0x04, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x06];
        let south = [0x09, 0x10, 0x0d, 0x3e, 0x0d, 0x3e, 0x0d, 0x12, 0x0e];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..36 {
                let metatile = if row < 4 {
                    north[column / 4]
                } else {
                    south[column / 4]
                };
                let mut tile = source(metatile, (column % 4) as u8, (row % 4) as u8);
                tile.tileset_id = Arc::from("ice_path");
                tile.tile_index = 0x9a;
                sources.push(tile);
            }
        }
        let frame = frame(36, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 36,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(ice_path_plateau_origins(&cells, 36, 8), vec![(0, 0, 36)]);
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 36,
                height: 8,
                roof_rows: 6,
                ground_tile_index: 0x9a,
            }]
        );
    }

    #[test]
    fn olivine_lighthouse_is_detected_as_one_tall_door_anchored_tower() {
        let block_rows: [[u16; 2]; 7] = [
            [0x08, 0x09],
            [0x7e, 0x7f],
            [0x13, 0x0f],
            [0x13, 0x0f],
            [0x13, 0x0f],
            [0x13, 0x0f],
            [0x1a, 0x11],
        ];
        let mut sources = Vec::new();
        for block_row in block_rows {
            for subtile_row in 0..4 {
                for block in block_row {
                    for subtile_column in 0..4 {
                        sources.push(source(block, subtile_column, subtile_row));
                    }
                }
            }
        }
        let frame = frame(8, 28, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 28,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 8,
                height: 28,
                roof_rows: 8,
                ground_tile_index: 0x06,
            }]
        );
    }

    #[test]
    fn kanto_connected_roof_and_facade_courses_form_one_building() {
        let metatiles = [[0x20, 0x54, 0x21], [0x37, 0x3a, 0x7e]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..12 {
                sources.push(source_for_tileset(
                    "kanto",
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                    if column == 11 && row == 7 { 0x2c } else { 0x30 },
                ));
            }
        }
        let frame = frame(12, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -48.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 8,
                roof_rows: 4,
                ground_tile_index: KANTO_GROUND_TILE_INDEX,
            }]
        );
    }

    #[test]
    fn celadon_department_store_claims_its_roof_and_every_window_course() {
        let metatiles = [
            [0x20, 0x54, 0x21],
            [0x68, 0x7f, 0x69],
            [0x68, 0x7f, 0x69],
            [0x37, 0x3a, 0x73],
        ];
        let mut sources = Vec::new();
        for row in 0..16 {
            for column in 0..12 {
                sources.push(source_for_tileset(
                    "kanto",
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                    0x30,
                ));
            }
        }
        let frame = frame(12, 16, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 12,
            height: 16,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 12,
                height: 16,
                roof_rows: 4,
                ground_tile_index: KANTO_GROUND_TILE_INDEX,
            }]
        );
        assert_eq!(building_facade_height_scale(true), 2.0);
        assert_eq!(
            (16 - 4) as f32 * 8.0 * building_facade_height_scale(true),
            192.0,
            "Celadon's twelve facade rows must exceed the Game Boy viewport before the cap"
        );
        assert_eq!(
            building_facade_height_scale(false),
            1.0,
            "ordinary buildings retain native course height"
        );
    }

    #[test]
    fn saffron_silph_claims_its_cap_before_all_four_window_courses() {
        let metatiles = [
            [0x20, 0x54, 0x54, 0x21],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x37, 0x3a, 0x7d, 0x7e],
        ];
        let mut sources = Vec::new();
        for row in 0..24 {
            for column in 0..16 {
                sources.push(source_for_tileset(
                    "kanto",
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                    0x30,
                ));
            }
        }
        let frame = frame(16, 24, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 16,
            height: 24,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 16,
                height: 24,
                roof_rows: 4,
                ground_tile_index: KANTO_GROUND_TILE_INDEX,
            }]
        );
    }

    #[test]
    fn route_2_cliff_mound_claims_all_four_blocks_without_a_center_hole() {
        let metatiles = [[0x3e, 0x3f, 0x3f, 0x3b], [0x24, 0x06, 0x57, 0x25]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..16 {
                sources.push(source_for_tileset(
                    "kanto",
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                    0x11,
                ));
            }
        }
        let frame = frame(16, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 16,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -64.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 16,
                height: 8,
                roof_rows: 6,
                ground_tile_index: KANTO_GROUND_TILE_INDEX,
            }]
        );
    }

    #[test]
    fn kanto_rounded_center_cap_and_facade_form_one_building() {
        let metatiles = [[0x68, 0x7f, 0x7f, 0x69], [0x37, 0x3a, 0x3a, 0x73]];
        let mut sources = Vec::new();
        for row in 0..8 {
            for column in 0..16 {
                sources.push(source_for_tileset(
                    "kanto",
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                    if column == 15 && row == 7 { 0x2c } else { 0x30 },
                ));
            }
        }
        let frame = frame(16, 8, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 16,
            height: 8,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -64.0,
            origin_z: -32.0,
        };

        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 16,
                height: 8,
                roof_rows: 4,
                ground_tile_index: KANTO_GROUND_TILE_INDEX,
            }]
        );
    }

    #[test]
    fn saffron_silph_claims_repeated_storeys_before_its_facade() {
        let metatiles = [
            [0x68, 0x7f, 0x7f, 0x69],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x68, 0x7f, 0x7f, 0x69],
            [0x37, 0x3a, 0x7d, 0x7e],
        ];
        let mut sources = Vec::new();
        for row in 0..20 {
            for column in 0..16 {
                sources.push(source_for_tileset(
                    "kanto",
                    metatiles[row / 4][column / 4],
                    (column % 4) as u8,
                    (row % 4) as u8,
                    0x30,
                ));
            }
        }
        let frame = frame(16, 20, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 16,
            height: 20,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            outdoor_building_placements(&cells, &geometry),
            vec![BuildingPlacement {
                column: 0,
                row: 0,
                width: 16,
                height: 20,
                roof_rows: 4,
                ground_tile_index: KANTO_GROUND_TILE_INDEX,
            }]
        );
    }

    #[test]
    fn kanto_unknown_adjacency_does_not_invent_a_building() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..8 {
                sources.push(source_for_tileset(
                    "kanto",
                    if column < 4 { 0x20 } else { 0x22 },
                    (column % 4) as u8,
                    row as u8,
                    0x30,
                ));
            }
        }
        let frame = frame(8, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -32.0,
            origin_z: -16.0,
        };
        assert!(outdoor_building_placements(&cells, &geometry).is_empty());
    }

    #[test]
    fn recognized_building_without_ground_evidence_reports_missing_source() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..8 {
                sources.push(source_for_tileset(
                    "kanto",
                    if column < 4 { 0x02 } else { 0x03 },
                    (column % 4) as u8,
                    row as u8,
                    0x30,
                ));
            }
        }
        let frame = frame(8, 4, sources);
        let mut samples = TerrainImageSamples::default();
        for tile in &frame.tiles {
            samples.pixels.insert(
                tile.texture.id(),
                TileImageSample::Rgba([90, 80, 70, 255].repeat(64)),
            );
        }
        assert!(matches!(
            build_terrain_mesh_with_samples(&frame, &samples),
            Err(TerrainMeshError::MissingGroundSample { .. })
        ));
    }

    #[test]
    fn modern_buildings_keep_pixel_facades_and_source_colored_side_courses() {
        let sources = (0..40)
            .map(|i| {
                source_for_tileset(
                    "johto_modern",
                    if i < 32 { 0x18 } else { 0x01 },
                    (i % 4) as u8,
                    ((i / 8) % 4) as u8,
                    if i < 32 { 0x20 } else { 0x06 },
                )
            })
            .collect();
        let frame = frame(8, 5, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 8,
            height: 5,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut images = TerrainImageSamples::default();
        for tile in &frame.tiles {
            let rgba = (0..64)
                .flat_map(|p| {
                    let shade = [0, 70, 140, 220][(p / 8) % 4];
                    [shade, shade, shade, 255]
                })
                .collect();
            images
                .pixels
                .insert(tile.texture.id(), TileImageSample::Rgba(rgba));
        }
        let mut mesh = TerrainMeshData::default();
        append_pixel_building(
            &mut mesh,
            &images,
            &cells,
            &geometry,
            "GoldenrodCity",
            BuildingPlacement {
                column: 0,
                row: 0,
                width: 8,
                height: 4,
                roof_rows: 2,
                ground_tile_index: 0x06,
            },
            &mut [false; 40],
        )
        .unwrap();
        let rear_wall_uvs: Vec<_> = mesh
            .textured
            .normals
            .chunks_exact(4)
            .zip(mesh.textured.uvs.chunks_exact(4))
            .filter(|(n, uv)| n[0] == [0.0, 0.0, -1.0] && uv[0][1] > 0.4)
            .map(|(_, uv)| uv)
            .collect();
        assert!(!rear_wall_uvs.is_empty(), "rear wall must remain closed");
        assert!(
            rear_wall_uvs
                .iter()
                .all(|uvs| uvs.iter().all(|uv| uv[0] <= 1.0 / 64.0)),
            "rear walls must sample siding, not the facade's doors and windows"
        );
        let side_courses = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(p, n)| n[0][0].abs() == 1.0 && p.iter().all(|v| v[1] <= 16.0))
            .count();
        assert!(
            side_courses > 16,
            "modern walls must carry source texel courses, not plain gray sides"
        );
        let front_pixels = mesh
            .textured
            .normals
            .chunks_exact(4)
            .filter(|n| n[0] == [0.0, 0.0, 1.0])
            .count();
        assert!(
            front_pixels >= 64 * 16,
            "modern facades must retain per-pixel geometry"
        );
    }

    #[test]
    fn roof_slab_uses_the_drawn_column_silhouette_instead_of_a_generic_gable() {
        assert_eq!(roof_slab_height(32.0, 4, 0, 1.0), 36.0);
        assert_eq!(roof_slab_height(32.0, 4, 2, 1.0), 34.0);
        assert_eq!(roof_slab_height(32.0, 4, 4, 1.0), 32.0);
        assert_eq!(roof_slab_height(32.0, 4, 12, 1.0), 32.0);
    }

    #[test]
    fn building_facade_stays_on_the_original_south_door_edge() {
        assert_eq!(building_facade_plane_z(-72.0, 8, 1.0), -64.0);
        assert_eq!(building_facade_plane_z(-72.0, 32, 1.0), -40.0);
    }

    #[test]
    fn building_roof_profile_keeps_a_gable_but_rejects_one_pixel_noise() {
        let width = 9;
        let rows = 5;
        let mut inside = vec![false; width * rows];
        for (x, top) in [4, 3, 2, 1, 0, 1, 2, 3, 4].into_iter().enumerate() {
            for y in top..rows {
                inside[y * width + x] = true;
            }
        }
        // A decorative pixel above the otherwise two-pixel-inset shoulder
        // must not become a physical chimney/spike.
        inside[2] = true;
        assert_eq!(
            measured_roof_profile(&inside, width, rows),
            vec![3, 3, 1, 1, 1, 1, 2, 3, 3]
        );
    }

    #[test]
    fn deeper_roof_keeps_unique_rims_and_repeats_only_middle_courses() {
        let rows: Vec<_> = (0..48)
            .map(|depth| roof_source_row(depth, 48, 32))
            .collect();
        assert_eq!(&rows[..4], &[0, 1, 2, 3]);
        assert_eq!(&rows[44..], &[28, 29, 30, 31]);
        assert!(rows[4..44].iter().all(|row| (4..28).contains(row)));
        assert_eq!(rows[4], rows[28]);
    }

    #[test]
    fn small_enclosed_facade_pane_recesses_but_siding_stays_flush() {
        let width = 32;
        let height = 16;
        let mut inside = vec![true; width * height];
        let mut luminance = vec![300_u16; width * height];
        // A black frame encloses a small 6x6 bright pane.
        for y in 3..11 {
            for x in 3..11 {
                if x == 3 || x == 10 || y == 3 || y == 10 {
                    luminance[y * width + x] = 0;
                }
            }
        }
        // Outside art is not eligible even if it has a non-black shade.
        inside[15 * width + 31] = false;
        let recessed = facade_recess_mask(&inside, &luminance, width, height, 0, 0);

        assert!(recessed[6 * width + 6]);
        assert!(
            !recessed[3 * width + 3],
            "the proud black frame stays flush"
        );
        assert!(
            !recessed[1 * width + 20],
            "the broad connected siding course must not become one deep panel"
        );
        assert!(!recessed[15 * width + 31]);
    }

    #[test]
    fn house_bookcase_recess_starts_below_the_lid_and_keeps_frame_proud() {
        let width = 16;
        let height = 32;
        let mut inside = vec![true; width * height];
        let mut luminance = vec![300_u16; width * height];
        // A dark rectangular frame around one shelf opening. The opening is
        // deliberately below source row 9, where the authored lid folds.
        for y in 11..18 {
            for x in 3..13 {
                if x == 3 || x == 12 || y == 11 || y == 17 {
                    luminance[y * width + x] = 0;
                }
            }
        }
        // Transparent pixels are outside the drawing and cannot be panels.
        inside[31 * width] = false;

        let recessed = facade_recess_mask(&inside, &luminance, width, height, 9, 0);

        assert!(
            recessed[14 * width + 8],
            "the enclosed shelf field recesses"
        );
        assert!(
            !recessed[11 * width + 8],
            "the dark shelf frame stays proud"
        );
        assert!(
            recessed[..9 * width].iter().all(|pixel| !pixel),
            "the cabinet lid is outside the facade relief pass"
        );
        assert!(
            !recessed[31 * width],
            "transparent background never recesses"
        );
    }

    #[test]
    fn facade_side_course_ignores_outline_and_uses_dominant_row_paint() {
        let inside = vec![true; 8];
        // Black outline at both edges, one window pixel, and a repeated
        // siding course across the rest of the authored row.
        let luminance = vec![10, 80, 80, 200, 80, 80, 80, 10];
        assert_eq!(
            facade_side_course_x(&inside, &luminance, 8, 0, 10, false),
            1
        );
        assert_eq!(facade_side_course_x(&inside, &luminance, 8, 0, 10, true), 6);
    }

    #[test]
    fn tree_art_forms_a_closed_pixel_voxel_hull() {
        let mut sources = Vec::new();
        for row in 0..4 {
            sources.push(source_with_tile(0x05, 0, row, 0x1e + row as u16 * 0x10));
            sources.push(source_with_tile(0x01, 0, 0, 0x05));
        }
        let mut sources = sources;
        for row in 0..4 {
            sources[row * 2 + 1] = source_with_tile(0x05, 1, row as u8, 0x1f + row as u16 * 0x10);
        }
        sources.extend((0..2).map(|_| source_with_tile(0x01, 0, 0, 0x05)));
        let frame = frame(2, 5, sources);
        let mut samples = TerrainImageSamples::default();
        for tile in &frame.tiles {
            let rgba = if tile.source.metatile_id == 0x05 {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            };
            samples
                .pixels
                .insert(tile.texture.id(), TileImageSample::Rgba(rgba.repeat(64)));
        }
        let mesh = build_terrain_mesh_with_samples(&frame, &samples)
            .expect("tree group should mesh as an upright 2D card");
        let upright = mesh
            .textured
            .normals
            .chunks_exact(4)
            .filter(|face| face[0] == [0.0, 0.0, 1.0])
            .count();
        let backs = mesh
            .textured
            .normals
            .chunks_exact(4)
            .filter(|face| face[0] == [0.0, 0.0, -1.0])
            .count();
        assert!(upright > 0);
        assert!(backs > 0, "tree hulls must have back faces");
        let mut front_depths: Vec<_> = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(_, normals)| normals[0] == [0.0, 0.0, 1.0])
            .map(|(positions, _)| (positions[0][2] * 1000.0).round() as i32)
            .collect();
        front_depths.sort_unstable();
        front_depths.dedup();
        assert!(
            front_depths.len() > 1,
            "tree crowns must occupy multiple depth voxels"
        );
    }

    #[test]
    fn house_furniture_has_solid_depth_while_signs_keep_pixel_thickness() {
        let drawing = [
            [0x11, 0x11, 0x11, 0x11],
            [0x06, 0x07, 0x11, 0x11],
            [0x16, 0x17, 0x0e, 0x0f],
            [0x08, 0x09, 0x3a, 0x3b],
        ];
        let house_tiles = (0..4)
            .flat_map(|row| {
                (0..4).map(move |column| {
                    source_for_tileset(
                        "players_house",
                        0x11,
                        column as u8,
                        row as u8,
                        drawing[row][column],
                    )
                })
            })
            .collect::<Vec<_>>();
        let frame = frame(4, 4, house_tiles);
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let fixtures = players_house_upright_fixture_placements(&cells, &geometry);
        assert_eq!(fixtures.len(), 2);
        assert!(fixtures.iter().all(|fixture| fixture.card_thickness == 8.0));

        let generic = grouped_flat_card_placements(
            &cells,
            &geometry,
            0x11,
            false,
            crate::players_house::upright_fixture_local,
        );
        assert!(generic.iter().all(|fixture| fixture.card_thickness == 1.0));
    }

    #[test]
    fn trainer_house_open_book_keeps_the_complete_page_drawing() {
        let book = [
            source_for_tileset("house", 0x14, 2, 3, 0x46),
            source_for_tileset("house", 0x14, 3, 3, 0x47),
            source_for_tileset("house", 0x18, 2, 0, 0x56),
            source_for_tileset("house", 0x18, 3, 0, 0x57),
        ];
        let frame = frame(2, 2, book.into());
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };

        let placements = house_open_book_placements(&cells, &geometry);
        assert_eq!(placements.len(), 1);
        assert_eq!((placements[0].column, placements[0].row), (0, 0));
        assert_eq!((placements[0].width, placements[0].height), (2, 2));
        assert!(
            !placements[0].outline_mask,
            "the light pages touch the drawing boundary, so a dark-outline mask drops them"
        );
    }

    #[test]
    fn runtime_repeated_scenery_has_a_fixed_geometry_budget() {
        let width = 65;
        let height = 64;
        let sources = (0..height)
            .flat_map(|row| {
                (0..width).map(move |column| {
                    if column < 64 {
                        source_with_tile(
                            0x05,
                            (column % 4) as u8,
                            (row % 4) as u8,
                            0x20 + (row % 4) as u16,
                        )
                    } else {
                        source_with_tile(0x01, 0, 0, 0x05)
                    }
                })
            })
            .collect();
        let frame = frame(width, height, sources);
        let mut samples = TerrainImageSamples::default();
        for tile in &frame.tiles {
            let mut rgba = [255, 255, 255, 255].repeat(64);
            if tile.source.metatile_id == 0x05 {
                for y in 0..8 {
                    for x in 1..7 {
                        rgba[(y * 8 + x) * 4..(y * 8 + x + 1) * 4]
                            .copy_from_slice(&[0, 80, 0, 255]);
                    }
                }
            }
            samples
                .pixels
                .insert(tile.texture.id(), TileImageSample::Rgba(rgba));
        }
        let mesh = build_instanced_terrain_mesh_with_samples(&frame, &samples).unwrap();
        assert_eq!(mesh.tree_instances.len(), 1);
        assert_eq!(
            mesh.tree_instances[0].origins.len(),
            512,
            "every tree remains present"
        );
        let bytes =
            |surface: &SurfaceMeshData| surface.positions.len() * 48 + surface.indices.len() * 4;
        let total = bytes(&mesh.textured)
            + bytes(&mesh.solid)
            + mesh
                .tree_instances
                .iter()
                .map(|group| bytes(&group.mesh))
                .sum::<usize>()
            + bytes(&mesh.background.as_ref().unwrap().mesh);
        assert!(
            total < 2 * 1024 * 1024,
            "512 repeated trees exceeded the 2 MiB geometry budget: {total}"
        );
    }

    #[test]
    fn repeated_tree_metatile_becomes_two_complete_two_tile_tall_sprites() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..5 {
                sources.push(if column < 4 {
                    source_with_tile(0x05, column as u8, row as u8, 0x20 + row as u16)
                } else {
                    source_with_tile(0x01, 0, 0, 0x05)
                });
            }
        }
        let frame = frame(5, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 5,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -20.0,
            origin_z: -16.0,
        };
        assert_eq!(
            complete_tree_placements(&cells, &geometry),
            vec![
                TreePlacement {
                    column: 0,
                    row: 0,
                    width: 2,
                    height: 4,
                    ground_tile_index: 0x05,
                    ground_metatile_id: None,
                    base_height: 0.0,
                    rounded: true,
                    outline_mask: true,
                    remove_all_ground: false,
                    card_thickness: 0.0,
                },
                TreePlacement {
                    column: 2,
                    row: 0,
                    width: 2,
                    height: 4,
                    ground_tile_index: 0x05,
                    ground_metatile_id: None,
                    base_height: 0.0,
                    rounded: true,
                    outline_mask: true,
                    remove_all_ground: false,
                    card_thickness: 0.0,
                },
            ]
        );

        let mut samples = TerrainImageSamples::default();
        for tile in &frame.tiles {
            let rgba = if tile.source.metatile_id == 0x05 {
                // Model the authored drawing: boundary-connected light
                // ground surrounds a darker canopy that continues through
                // all four source rows. Painting the corners canopy-colored
                // would correctly classify the entire synthetic image as
                // background and would not represent a real tree sprite.
                let mut rgba = [255, 255, 255, 255].repeat(64);
                for y in 0..SOURCE_TILE_PIXELS {
                    for x in 1..SOURCE_TILE_PIXELS - 1 {
                        let offset = (y * SOURCE_TILE_PIXELS + x) * 4;
                        rgba[offset..offset + 4].copy_from_slice(&[
                            0,
                            if x > 1 && x < 6 && y % 3 == 0 {
                                120
                            } else {
                                80
                            },
                            0,
                            255,
                        ]);
                    }
                }
                rgba
            } else {
                [255, 255, 255, 255].repeat(64)
            };
            samples
                .pixels
                .insert(tile.texture.id(), TileImageSample::Rgba(rgba));
        }
        let mesh = build_terrain_mesh_with_samples(&frame, &samples)
            .expect("complete repeated tree drawing should mesh as upright sprites");
        let instanced = build_instanced_terrain_mesh_with_samples(&frame, &samples).unwrap();
        assert_eq!(
            instanced.tree_instances.len(),
            1,
            "identical painted hulls share one mesh"
        );
        assert_eq!(instanced.tree_instances[0].origins.len(), 2);
        assert!(instanced.textured.positions.len() < mesh.textured.positions.len() / 4);
        // Expand instances only in the test and compare every position, normal,
        // shade, and sampled source texel against the original complete mesh.
        let face_signatures = |surface: &SurfaceMeshData, offset: [f32; 3]| {
            surface
                .positions
                .iter()
                .zip(&surface.normals)
                .zip(&surface.colors)
                .zip(&surface.uvs)
                .map(|(((position, normal), color), uv)| {
                    let x = (uv[0] * frame.grid_size.x as f32 * 8.0).floor() as usize;
                    let y = (uv[1] * frame.grid_size.y as f32 * 8.0).floor() as usize;
                    let tile = &frame.tiles[(y.min(31) / 8) * 5 + x.min(39) / 8];
                    let TileImageSample::Rgba(bytes) = &samples.pixels[&tile.texture.id()] else {
                        panic!()
                    };
                    let pixel = ((y % 8) * 8 + x % 8) * 4;
                    let mut key = Vec::new();
                    key.extend(position.iter().zip(offset).map(|(p, o)| (p + o).to_bits()));
                    key.extend(normal.map(f32::to_bits));
                    key.extend(color.map(f32::to_bits));
                    key.extend(bytes[pixel..pixel + 4].iter().map(|b| u32::from(*b)));
                    key
                })
                .collect::<Vec<_>>()
        };
        let mut expected = face_signatures(&mesh.textured, [0.0; 3]);
        let mut actual = face_signatures(&instanced.textured, [0.0; 3]);
        for group in &instanced.tree_instances {
            for &origin in &group.origins {
                actual.extend(face_signatures(&group.mesh, origin));
            }
        }
        expected.sort();
        actual.sort();
        assert_eq!(
            actual, expected,
            "instancing must preserve the complete painted geometry"
        );
        let mut recolored = samples.clone();
        let TileImageSample::Rgba(pixels) = recolored
            .pixels
            .get_mut(&frame.tiles[2].texture.id())
            .unwrap()
        else {
            panic!()
        };
        pixels[4..8].copy_from_slice(&[0, 0, 80, 255]);
        let distinct = build_instanced_terrain_mesh_with_samples(&frame, &recolored).unwrap();
        assert_eq!(
            distinct.tree_instances.len(),
            2,
            "different palettes must never share a prototype"
        );

        let is_tree_card_normal = |normal: [f32; 3]| normal == [0.0, 0.0, 1.0];
        let (min_y, max_y) = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(_, normals)| is_tree_card_normal(normals[0]))
            .flat_map(|(positions, _)| positions)
            .fold(
                (f32::INFINITY, f32::NEG_INFINITY),
                |(min, max), position| (min.min(position[1]), max.max(position[1])),
            );
        let expected_world_y_span = 32.0;
        let normal_samples: Vec<_> = mesh
            .textured
            .normals
            .chunks_exact(4)
            .map(|quad| quad[0])
            .collect();
        assert!(
            (max_y - min_y - expected_world_y_span).abs() < 0.001,
            "headbutt trees are two map tiles tall: measured {}..{} (span {}), expected {}; normals {:?}",
            min_y,
            max_y,
            max_y - min_y,
            expected_world_y_span,
            normal_samples,
        );
        assert!(!normal_samples.is_empty());
        let (min_z, max_z) = mesh
            .textured
            .positions
            .chunks_exact(4)
            .zip(mesh.textured.normals.chunks_exact(4))
            .filter(|(_, normals)| is_tree_card_normal(normals[0]))
            .flat_map(|(positions, _)| positions)
            .fold(
                (f32::INFINITY, f32::NEG_INFINITY),
                |(min, max), position| (min.min(position[2]), max.max(position[2])),
            );
        assert!(max_z > min_z, "isolated canopies must have real depth");
    }

    #[test]
    fn headbutt_tree_block_becomes_four_independent_two_by_two_cards() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..5 {
                sources.push(if column < 4 {
                    source_for_tileset(
                        "johto",
                        0x61,
                        column,
                        row,
                        if row % 2 == 0 {
                            if column % 2 == 0 { 0x1e } else { 0x1f }
                        } else if column % 2 == 0 {
                            0x3e
                        } else {
                            0x3f
                        },
                    )
                } else {
                    source_with_tile(0x01, 0, 0, 0x05)
                });
            }
        }
        let frame = frame(5, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 5,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placements = complete_tree_placements(&cells, &geometry);
        assert_eq!(placements.len(), 4);
        assert_eq!(
            placements
                .iter()
                .map(|placement| (
                    placement.column,
                    placement.row,
                    placement.width,
                    placement.height
                ))
                .collect::<Vec<_>>(),
            vec![(0, 0, 2, 2), (2, 0, 2, 2), (0, 2, 2, 2), (2, 2, 2, 2)]
        );
    }

    #[test]
    fn mixed_headbutt_block_claims_two_trees_but_not_its_ground_rows() {
        let mut sources = Vec::new();
        for row in 0..2 {
            for column in 0..5 {
                sources.push(if column < 4 {
                    source_for_tileset(
                        "johto",
                        0x5d,
                        column,
                        row,
                        if row == 0 {
                            if column % 2 == 0 { 0x1e } else { 0x1f }
                        } else if column % 2 == 0 {
                            0x3e
                        } else {
                            0x3f
                        },
                    )
                } else {
                    source_with_tile(0x01, 0, 0, 0x05)
                });
            }
        }
        let frame = frame(5, 2, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 5,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        assert_eq!(
            complete_tree_placements(&cells, &geometry)
                .iter()
                .map(|placement| (
                    placement.column,
                    placement.row,
                    placement.width,
                    placement.height
                ))
                .collect::<Vec<_>>(),
            vec![(0, 0, 2, 2), (2, 0, 2, 2)]
        );
    }

    #[test]
    fn battle_tower_border_metatile_becomes_two_complete_upright_trees() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                let mut source = source_with_tile(
                    0x05,
                    column as u8,
                    row as u8,
                    [0x1e, 0x13, 0x13, 0x3e][row] + (column % 2) as u16,
                );
                source.tileset_id = Arc::from("battle_tower_outside");
                sources.push(source);
            }
        }
        let frame = frame(4, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -16.0,
            origin_z: -16.0,
        };
        assert_eq!(
            complete_tree_placements(&cells, &geometry),
            vec![
                TreePlacement {
                    column: 0,
                    row: 0,
                    width: 2,
                    height: 4,
                    ground_tile_index: 0x06,
                    ground_metatile_id: None,
                    base_height: 0.0,
                    rounded: true,
                    outline_mask: true,
                    remove_all_ground: false,
                    card_thickness: 0.0,
                },
                TreePlacement {
                    column: 2,
                    row: 0,
                    width: 2,
                    height: 4,
                    ground_tile_index: 0x06,
                    ground_metatile_id: None,
                    base_height: 0.0,
                    rounded: true,
                    outline_mask: true,
                    remove_all_ground: false,
                    card_thickness: 0.0,
                },
            ]
        );
    }

    #[test]
    fn forest_sources_reuse_grouped_tree_placements() {
        let forest_source = |metatile_id, column, row, tile_index| {
            let mut source = source_with_tile(metatile_id, column, row, tile_index);
            source.tileset_id = Arc::from("forest");
            source
        };
        let scattered = frame(
            2,
            2,
            vec![
                forest_source(0x08, 0, 0, 0x26),
                forest_source(0x08, 1, 0, 0x27),
                forest_source(0x08, 0, 1, 0x36),
                forest_source(0x08, 1, 1, 0x37),
            ],
        );
        let scattered_cells: Vec<_> = scattered.tiles.iter().collect();
        let scattered_geometry = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -8.0,
            origin_z: -8.0,
        };
        assert_eq!(
            complete_tree_placements(&scattered_cells, &scattered_geometry),
            vec![TreePlacement {
                column: 0,
                row: 0,
                width: 2,
                height: 2,
                ground_tile_index: 0x05,
                ground_metatile_id: None,
                base_height: 0.0,
                rounded: true,
                outline_mask: true,
                remove_all_ground: false,
                card_thickness: 0.0,
            }]
        );

        let mut border_sources = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                border_sources.push(forest_source(
                    0x05,
                    column,
                    row,
                    0x0c + u16::from(column) + u16::from(row) * 0x10,
                ));
            }
        }
        let border = frame(4, 4, border_sources);
        let border_cells: Vec<_> = border.tiles.iter().collect();
        let border_geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -16.0,
            origin_z: -16.0,
        };
        assert_eq!(
            complete_tree_placements(&border_cells, &border_geometry),
            vec![TreePlacement {
                column: 0,
                row: 0,
                width: 4,
                height: 4,
                ground_tile_index: 0x05,
                ground_metatile_id: None,
                base_height: 0.0,
                rounded: true,
                outline_mask: true,
                remove_all_ground: false,
                card_thickness: 0.0,
            }]
        );
    }

    #[test]
    fn saffron_gym_planter_stays_a_flat_masked_card() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..2 {
                let mut source = source_with_tile(0x36, column, row, 0x20);
                source.tileset_id = Arc::from("underground");
                sources.push(source);
            }
        }
        let frame = frame(2, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 2,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -8.0,
            origin_z: -16.0,
        };

        let placements = saffron_gym_planter_placements(&cells, &geometry);
        assert_eq!(placements.len(), 1);
        assert_eq!((placements[0].width, placements[0].height), (2, 4));
        assert!(!placements[0].rounded);
        assert!(placements[0].outline_mask);
    }

    #[test]
    fn mountain_corner_art_is_not_misclassified_as_a_tree() {
        let mut sources = Vec::new();
        for row in 0..4 {
            for column in 0..4 {
                sources.push(source_with_tile(
                    0x6a,
                    column as u8,
                    row as u8,
                    if column < 2 { 0x20 + row as u16 } else { 0x3c },
                ));
            }
        }
        let frame = frame(4, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -16.0,
            origin_z: -16.0,
        };
        assert!(complete_tree_placements(&cells, &geometry).is_empty());
    }

    #[test]
    fn clipped_profile_without_ground_evidence_stays_flat() {
        let mut tree = source_with_tile(0x32, 0, 0, 0x40);
        tree.tileset_id = Arc::from("kanto");
        let mesh = build_terrain_mesh(&frame(1, 1, vec![tree]))
            .expect("missing profile evidence should preserve the flat baseline");

        assert_eq!(mesh.textured.quad_count(), 1);
        assert_eq!(mesh.solid.quad_count(), 0);
        assert!(
            mesh.textured
                .positions
                .iter()
                .all(|position| position[1] == 0.0)
        );
    }

    #[test]
    fn grouped_mask_does_not_treat_internal_tile_seams_as_an_outer_boundary() {
        let width = SOURCE_TILE_PIXELS * 2;
        let height = SOURCE_TILE_PIXELS * 2;
        let mut equals_ground = vec![true; width * height];
        for y in 3..=12 {
            for x in 3..=12 {
                if x == 3 || x == 12 || y == 3 || y == 12 {
                    equals_ground[y * width + x] = false;
                }
            }
        }

        let removable = boundary_connected_mask(width, height, &equals_ground);
        assert!(removable[0], "open background must be removed");
        assert!(
            !removable[8 * width + 8],
            "enclosed face crossing the x=8/y=8 tile seams must remain"
        );
        assert!(!removable[3 * width + 3], "outline must remain");
    }

    #[test]
    fn shuffled_explicit_coordinates_preserve_source_cell_uvs() {
        let mut frame = frame(2, 1, vec![flat_source(), flat_source()]);
        frame.tiles.swap(0, 1);

        let mesh = build_terrain_mesh(&frame).expect("tile vector order is not spatial order");
        assert_eq!(mesh.textured.uvs[0], [0.0, 0.0]);
        assert_eq!(mesh.textured.uvs[4], [0.5, 0.0]);
    }
}
