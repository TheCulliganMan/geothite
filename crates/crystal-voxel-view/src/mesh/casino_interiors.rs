// Whole-source Game Corner architecture. Rectangles describe physical pieces,
// leaving the checker floor, entrance and service access in the native layout.
fn casino_drawing(block: u16) -> Option<[u16; 16]> {
    Some(match block {
        0x02 | 0x0d => [
            2, 2, 2, 2, 2, 2, 2, 2, 1, 0x10, 1, 0x10, 0x11, 0x12, 0x11, 0x12,
        ],
        0x04 => [
            0x16, 0x17, 0x17, 0x18, 0x26, 0x27, 0x27, 0x28, 0x36, 0x37, 0x37, 0x38, 0x11, 0x12,
            0x11, 0x12,
        ],
        0x09 => [
            1, 0x10, 1, 0x10, 0x11, 0x12, 0x11, 0x12, 0x0e, 0x0e, 0x0e, 0x0e, 0x1e, 0x1e, 0x1e,
            0x1e,
        ],
        0x10 => [
            0x4d, 0x4d, 0x4d, 0x4d, 0xc4, 0x51, 0x4d, 0x4d, 0x0e, 0x0e, 0x0e, 0x0e, 0x1e, 0x1e,
            0x1e, 0x1e,
        ],
        0x11 => [
            0x87, 0x88, 0x4d, 0x4d, 0x97, 0x98, 0x4d, 0x4d, 0x0e, 0x0e, 0x0e, 0x0e, 0x1e, 0x1e,
            0x1e, 0x1e,
        ],
        0x13 => [
            2, 2, 2, 2, 4, 5, 2, 2, 0x14, 0x15, 1, 0x10, 0x14, 0x15, 0x11, 0x12,
        ],
        0x14 => [
            0x14, 0x15, 1, 0x10, 0x14, 0x15, 0x11, 0x12, 0x14, 0x84, 0x0e, 0x0e, 0x3d, 0x1e, 0x1e,
            0x1e,
        ],
        0x1b => [
            1, 0x10, 0x85, 0x86, 0x11, 0x12, 0x85, 0x86, 0x0e, 0x0e, 0x94, 0x86, 0x1e, 0x1e, 0x1e,
            0x1f,
        ],
        0x1c => [
            2, 2, 2, 2, 2, 2, 0x95, 0x96, 1, 0x10, 0x85, 0x86, 0x11, 0x12, 0x85, 0x86,
        ],
        0x27 => [
            1, 0x10, 1, 0x10, 0x34, 0x35, 0x11, 0x12, 0x40, 0x41, 0x0e, 0x0e, 0x1e, 0x1e, 0x1e,
            0x1e,
        ],
        0x28 => [
            2, 2, 6, 7, 2, 2, 8, 9, 1, 0x10, 1, 0x10, 0x11, 0x12, 0x11, 0x12,
        ],
        0x2b => [
            2, 2, 2, 2, 2, 2, 0xa4, 0xa5, 1, 0x10, 0xb4, 0xb5, 0x11, 0x12, 0x32, 0x33,
        ],
        0x2f => [
            2, 2, 2, 2, 2, 2, 0x89, 0x8a, 0x0a, 0x0b, 0x8c, 0x8d, 0x1a, 0x1b, 0x99, 0x9a,
        ],
        0x30 => [
            2, 2, 2, 2, 0x8a, 0x8b, 2, 2, 0x8d, 0x8e, 0x0a, 0x0b, 0x9b, 0x9c, 0x1a, 0x1b,
        ],
        _ => return None,
    })
}

fn append_casino_candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    use ModelKind::*;
    if !crate::casino::is_game_corner_map(map) {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "game_corner", 1) else {
        return;
    };
    for (index, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        if s.tileset_id.as_ref() != "game_corner" || s.subtile_column != 0 || s.subtile_row != 0 {
            continue;
        }
        // The Prize Room's complete booth owns these three blocks together.
        // If that course is changed or reserved, do not turn the remaining
        // individual blocks into unrelated house windows and counters.
        if map == "CeladonGameCornerPrizeRoom" && matches!(s.metatile_id, 0x10 | 0x11) {
            continue;
        }
        let Some(drawing) = casino_drawing(s.metatile_id) else {
            continue;
        };
        let (column, row) = (index % g.width, index / g.width);
        if column + 4 > g.width || row + 4 > g.height {
            continue;
        }
        if !(0..4).all(|y| {
            (0..4).all(|x| {
                let t = &cells[(row + y) * g.width + column + x].source;
                t.tileset_id == s.tileset_id
                    && t.metatile_id == s.metatile_id
                    && t.subtile_column as usize == x
                    && t.subtile_row as usize == y
                    && t.tile_index == drawing[y * 4 + x]
            })
        }) {
            continue;
        }
        let mut piece = |kind, rect: [usize; 4], depth, rise, base| {
            let [x, y, w, h] = rect;
            let p = Placement {
                kind,
                column: column + x,
                row: row + y,
                width: w,
                height: h,
                ground,
                depth_pixels: depth,
                height_pixels: rise,
                base_pixels: base,
                footing_pixels: None,
            };
            if p.indices(g.width).any(|i| claimed[i]) {
                return;
            }
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            out.push(p);
        };
        match s.metatile_id {
            0x02 | 0x0d => piece(ArcadeWall, [0, 0, 4, 2], 3.0, 28.0, 0.0),
            0x04 => piece(RetailShelf, [0, 0, 4, 3], 10.0, 28.0, 0.0),
            0x09 => piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0),
            0x10 | 0x11 => {
                piece(WindowWall, [0, 0, 4, 2], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0);
            }
            0x13 => {
                piece(ArcadeWall, [0, 0, 2, 1], 3.0, 28.0, 0.0);
                piece(ArcadeWall, [2, 0, 2, 2], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [0, 1, 2, 3], 24.0, 8.0, 0.0);
            }
            0x14 | 0x1b => {
                piece(
                    ReceptionCounter,
                    [if s.metatile_id == 0x14 { 0 } else { 2 }, 0, 2, 2],
                    16.0,
                    8.0,
                    0.0,
                );
                piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0);
            }
            0x1c => {
                piece(ArcadeWall, [0, 0, 4, 2], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [2, 2, 2, 2], 16.0, 8.0, 0.0);
            }
            0x27 => {
                piece(LabWorkstation, [0, 1, 2, 1], 8.0, 20.0, 0.0);
                piece(ReceptionCounter, [0, 2, 4, 2], 16.0, 8.0, 0.0);
            }
            0x28 => {
                piece(ArcadeWall, [0, 0, 2, 2], 3.0, 28.0, 0.0);
                piece(PictureFrame, [2, 0, 2, 2], 2.0, 14.0, 12.0);
            }
            0x2b => {
                piece(ArcadeWall, [0, 0, 2, 2], 3.0, 28.0, 0.0);
                piece(ArcadeWall, [2, 0, 2, 1], 3.0, 28.0, 0.0);
                piece(LabWorkstation, [2, 1, 2, 3], 12.0, 22.0, 0.0);
            }
            0x2f | 0x30 => {
                let table_x = if s.metatile_id == 0x2f { 2 } else { 0 };
                piece(ArcadeWall, [0, 0, 4, 1], 3.0, 28.0, 0.0);
                piece(ArcadeWall, [2 - table_x, 1, 2, 1], 3.0, 28.0, 0.0);
                piece(ReceptionCounter, [table_x, 1, 2, 3], 24.0, 10.0, 0.0);
                piece(ArcadeStool, [2 - table_x, 2, 2, 2], 12.0, 7.0, 0.0);
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod casino_interior_tests {
    use super::*;
    use std::sync::Arc;
    fn fixture(block: u16) -> (Vec<VisualTile>, GridGeometry) {
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut tiles: Vec<VisualTile> = (0..36)
            .map(|i| VisualTile {
                column: (i % 6) as u32,
                row: (i / 6) as u32,
                texture: Handle::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: Arc::from("game_corner"),
                    metatile_id: 1,
                    subtile_column: (i % 6 % 4) as u8,
                    subtile_row: (i / 6 % 4) as u8,
                    tile_index: 1,
                },
            })
            .collect();
        let drawing = casino_drawing(block).unwrap();
        for y in 0..4 {
            for x in 0..4 {
                let s = &mut tiles[y * 6 + x].source;
                s.metatile_id = block;
                s.subtile_column = x as u8;
                s.subtile_row = y as u8;
                s.tile_index = drawing[y * 4 + x];
            }
        }
        (tiles, g)
    }
    #[test]
    fn arcade_wall_uses_complete_native_course_without_acoustic_panel_fallback() {
        for map in ["GoldenrodGameCorner", "CeladonGameCorner"] {
            for block in [2, 0x0d] {
                let (tiles, g) = fixture(block);
                let refs = tiles.iter().collect::<Vec<_>>();
                let resolved = resolve(map, &refs, &g, None);
                assert_eq!(
                    resolved
                        .iter()
                        .filter(|p| p.kind == ModelKind::ArcadeWall)
                        .count(),
                    1
                );
                assert!(!resolved.iter().any(|p| p.kind == ModelKind::WallAcoustic));
                for y in 0..4 {
                    for x in 0..4 {
                        for mutation in 0..5 {
                            let mut changed = tiles.clone();
                            let s = &mut changed[y * g.width + x].source;
                            match mutation {
                                0 => s.metatile_id ^= 0x80,
                                1 => s.tile_index ^= 0x80,
                                2 => s.subtile_column ^= 0x80,
                                3 => s.subtile_row ^= 0x80,
                                _ => s.tileset_id = Arc::from("unknown"),
                            }
                            let resolved =
                                resolve(map, &changed.iter().collect::<Vec<_>>(), &g, None);
                            assert!(!resolved.iter().any(|p| matches!(
                                p.kind,
                                ModelKind::ArcadeWall | ModelKind::WallAcoustic
                            )));
                        }
                    }
                }
                for i in (0..2).flat_map(|y| (0..4).map(move |x| y * g.width + x)) {
                    let mut claimed = vec![false; 36];
                    claimed[i] = true;
                    let before = claimed.clone();
                    let mut out = vec![];
                    append_casino_candidates(map, &refs, &g, &mut claimed, &mut out);
                    assert!(out.is_empty());
                    assert_eq!(claimed, before);
                }
            }
        }
    }
    #[test]
    fn casino_architecture_requires_complete_source_and_preserves_support() {
        for block in [
            2, 4, 9, 0x0d, 0x10, 0x11, 0x13, 0x14, 0x1b, 0x1c, 0x27, 0x28, 0x2b, 0x2f, 0x30,
        ] {
            let (mut t, g) = fixture(block);
            let mut claimed = vec![false; 36];
            let mut p = Vec::new();
            append_casino_candidates(
                "GoldenrodGameCorner",
                &t.iter().collect::<Vec<_>>(),
                &g,
                &mut claimed,
                &mut p,
            );
            assert!(!p.is_empty(), "block {block:x}");
            assert!(!claimed[4] && !claimed[24], "adjacent floor claimed");
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.0; 36],
                authored_cells: vec![None; 36],
                ..Default::default()
            };
            let mut drawn = vec![false; 36];
            for piece in &p {
                assert!(append(
                    &mut mesh,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    piece,
                    &mut drawn,
                    [0, 0]
                ));
            }
            assert_eq!(drawn, claimed);
            assert_eq!(mesh.footing_heights, vec![0.0; 36]);
            assert!(!mesh.solid.positions.is_empty());
            t[0].source.tile_index ^= 0x80;
            let mut bad = Vec::new();
            append_casino_candidates(
                "GoldenrodGameCorner",
                &t.iter().collect::<Vec<_>>(),
                &g,
                &mut vec![false; 36],
                &mut bad,
            );
            assert!(bad.is_empty());
        }
    }
    #[test]
    fn casino_custom_claims_and_unrelated_maps_retain_ownership() {
        let (t, g) = fixture(9);
        let cells = t.iter().collect::<Vec<_>>();
        let mut claimed = vec![false; 36];
        claimed[12] = true;
        let mut p = Vec::new();
        append_casino_candidates("CeladonGameCorner", &cells, &g, &mut claimed, &mut p);
        assert!(p.is_empty());
        append_casino_candidates("VermilionGym", &cells, &g, &mut vec![false; 36], &mut p);
        assert!(p.is_empty());
    }
}
