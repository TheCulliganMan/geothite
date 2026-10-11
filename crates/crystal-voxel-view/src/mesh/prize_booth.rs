// Whole, map-scoped source course. The three native blocks describe one
// blue service wall with two vendor displays and a sign, not house windows.
const PRIZE_BOOTH_FINGERPRINT: u64 = 0xed6b02ffaf1309d1;
fn prize_booth_matches(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    column: usize,
    row: usize,
) -> bool {
    if map != "CeladonGameCornerPrizeRoom" || column + 12 > g.width || row + 4 > g.height {
        return false;
    }
    let mut hash = 0xcbf29ce484222325u64;
    for y in 0..4 {
        for x in 0..12 {
            let source = &cells[(row + y) * g.width + column + x].source;
            if source.tileset_id.as_ref() != "game_corner" {
                return false;
            }
            for byte in [
                (source.metatile_id & 255) as u8,
                (source.metatile_id >> 8) as u8,
                source.subtile_column,
                source.subtile_row,
                (source.tile_index & 255) as u8,
                (source.tile_index >> 8) as u8,
            ] {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
    }
    hash == PRIZE_BOOTH_FINGERPRINT
}
fn append_prize_booth_candidate(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if g.width < 12 || g.height < 4 {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "game_corner", 0x01) else {
        return;
    };
    for row in 0..=g.height - 4 {
        for column in 0..=g.width - 12 {
            if !prize_booth_matches(map, cells, g, column, row) {
                continue;
            }
            let p = Placement {
                kind: ModelKind::PrizeBooth,
                column,
                row,
                width: 12,
                height: 4,
                ground,
                depth_pixels: 8.,
                height_pixels: 32.,
                base_pixels: 0.,
                footing_pixels: None,
            };
            if p.indices(g.width).any(|i| claimed[i]) {
                continue;
            }
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    }
}
fn append_prize_booth_art(
    mesh: &mut TerrainMeshData,
    _cells: &[&VisualTile],
    g: &GridGeometry,
    p: &Placement,
) {
    let kx = g.tile_width / 8.;
    let k = g.tile_height / 8.;
    let west = g.origin_x + p.column as f32 * g.tile_width;
    let north = g.origin_z + (p.row + 4) as f32 * g.tile_height - 8. * k;
    let mut face = |tile: [usize; 2], b: [f32; 4], depth: f32| {
        let (u0, u1, v0, v1) = g.uv(p.column + tile[0], p.row + tile[1]);
        let [x0, x1, y0, y1] = b;
        let z = north + depth * k;
        append_quad(
            &mut mesh.textured,
            [
                [west + x0 * kx, y0 * k, z],
                [west + x1 * kx, y0 * k, z],
                [west + x1 * kx, y1 * k, z],
                [west + x0 * kx, y1 * k, z],
            ],
            [0., 0., 1.],
            [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
            [1.; 4],
        );
    };
    for row in 0..2 {
        for col in 0..2 {
            face(
                [col, row],
                [
                    2. + col as f32 * 6.,
                    8. + col as f32 * 6.,
                    23. - row as f32 * 6.,
                    29. - row as f32 * 6.,
                ],
                2.95,
            );
        }
    }
    for source_x in [4, 8] {
        for col in 0..2 {
            let x = source_x as f32 * 8. + 1. + col as f32 * 7.;
            face([source_x + col, 1], [x, x + 7., 16., 24.], 2.95);
        }
    }
}
#[cfg(test)]
mod prize_booth_tests {
    use super::*;
    use std::sync::Arc;
    fn fixture() -> (Vec<VisualTile>, GridGeometry) {
        let g = GridGeometry {
            width: 12,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..72)
            .map(|i| {
                let x = i % 12;
                let y = i / 12;
                let tile_index = match (x, y) {
                    (0, 0) => 0x87,
                    (1, 0) => 0x88,
                    (0, 1) => 0x97,
                    (1, 1) => 0x98,
                    (4 | 8, 1) => 0xc4,
                    (5 | 9, 1) => 0x51,
                    (_, 0 | 1) => 0x4d,
                    (_, 2) => 0x0e,
                    (_, 3) => 0x1e,
                    (_, 4) => {
                        if x % 2 == 0 {
                            1
                        } else {
                            0x10
                        }
                    }
                    _ => {
                        if x % 2 == 0 {
                            0x11
                        } else {
                            0x12
                        }
                    }
                };
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: Arc::from("game_corner"),
                        metatile_id: if y < 4 {
                            if x < 4 { 0x11 } else { 0x10 }
                        } else {
                            1
                        },
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index,
                    },
                }
            })
            .collect();
        (tiles, g)
    }
    #[test]
    fn prize_booth_complete_course_rejects_every_changed_identity_and_reserved_cell() {
        let (tiles, g) = fixture();
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(prize_booth_matches(
            "CeladonGameCornerPrizeRoom",
            &refs,
            &g,
            0,
            0
        ));
        assert!(!prize_booth_matches("CeladonGameCorner", &refs, &g, 0, 0));
        for i in 0..48 {
            for mutation in 0..5 {
                let mut changed = tiles.clone();
                let s = &mut changed[i].source;
                match mutation {
                    0 => s.tile_index ^= 1,
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column ^= 1,
                    3 => s.subtile_row ^= 1,
                    _ => s.tileset_id = Arc::from("unknown"),
                }
                assert!(!prize_booth_matches(
                    "CeladonGameCornerPrizeRoom",
                    &changed.iter().collect::<Vec<_>>(),
                    &g,
                    0,
                    0
                ));
                let resolved = resolve(
                    "CeladonGameCornerPrizeRoom",
                    &changed.iter().collect::<Vec<_>>(),
                    &g,
                    None,
                );
                assert!(
                    !resolved
                        .iter()
                        .any(|p| matches!(p.kind, ModelKind::PrizeBooth | ModelKind::WindowWall))
                );
            }
        }
        for i in 0..48 {
            let mut claimed = vec![false; 72];
            claimed[i] = true;
            let before = claimed.clone();
            let mut out = vec![];
            append_prize_booth_candidate(
                "CeladonGameCornerPrizeRoom",
                &refs,
                &g,
                &mut claimed,
                &mut out,
            );
            assert!(out.is_empty());
            assert_eq!(claimed, before);
        }
        let clipped = GridGeometry { width: 11, ..g };
        assert!(!prize_booth_matches(
            "CeladonGameCornerPrizeRoom",
            &refs,
            &clipped,
            0,
            0
        ));
    }
    #[test]
    fn prize_booth_takes_precedence_and_keeps_live_sign_and_two_displays() {
        let (tiles, g) = fixture();
        let refs = tiles.iter().collect::<Vec<_>>();
        let placements = resolve("CeladonGameCornerPrizeRoom", &refs, &g, None);
        let booth = placements
            .iter()
            .find(|p| p.kind == ModelKind::PrizeBooth)
            .unwrap();
        assert_eq!(booth.indices(g.width).count(), 48);
        assert!(!placements.iter().any(|p| p.kind == ModelKind::WindowWall));
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 72],
            ..Default::default()
        };
        let mut claimed = vec![false; 72];
        assert!(append(&mut mesh, &refs, &g, booth, &mut claimed, [0, 0]));
        assert_eq!(mesh.footing_heights, vec![0.; 72]);
        // Source floor underlay owns 48 quads, then four sign and four display quads.
        assert_eq!(mesh.textured.positions.len(), (48 + 8) * 4);
        let volume: f32 = mesh
            .solid
            .indices
            .chunks_exact(3)
            .map(|tri| {
                let [a, b, c] = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                a.dot(b.cross(c)) / 6.
            })
            .sum();
        assert!(volume > 0.);
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[0] >= 0. && p[0] <= 96.001 && p[2] >= 23.999 && p[2] <= 32.001)
        );
    }
}
