//! Complete Safari office table and square seats from the live game-corner art.
use super::*;

pub(super) fn matches(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if map != "SafariZoneMainOffice" {
        return false;
    }
    let first = &cells[p.row * g.width + p.column].source;
    if p.kind == ModelKind::ReceptionCounter && p.width == 2 && p.height == 11 {
        return p.indices(g.width).enumerate().all(|(j, i)| {
            let y = j / 2;
            let x = j % 2;
            let s = &cells[i].source;
            s.tileset_id.as_ref() == "game_corner"
                && s.metatile_id == [19, 22, 23][(y + 1) / 4]
                && s.subtile_column == x as u8
                && s.subtile_row == ((y + 1) % 4) as u8
                && s.tile_index
                    == if y == 0 {
                        [4, 5][x]
                    } else if y == 9 {
                        [20, 132][x]
                    } else if y == 10 {
                        [61, 30][x]
                    } else {
                        [20, 21][x]
                    }
        });
    }
    if p.kind == ModelKind::ReceptionCounter && p.width == 2 && p.height == 2 {
        return p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            s.tileset_id.as_ref() == "game_corner"
                && s.metatile_id == 23
                && s.subtile_column == 2 + (j % 2) as u8
                && s.subtile_row == 2 + (j / 2) as u8
                && s.tile_index == [14, 14, 30, 30][j]
        });
    }
    let art: &[u16] = match (p.kind, p.width, p.height, first.metatile_id) {
        (ModelKind::DiningTable, 4, 4, 21) => &[
            46, 13, 13, 47, 28, 12, 12, 29, 62, 80, 80, 63, 168, 78, 79, 169,
        ],
        (ModelKind::Chair, 2, 2, 6 | 25) => &[10, 11, 26, 27],
        (ModelKind::ArcadeWall, 4, 2, 2) => &[2; 8],
        (ModelKind::ArcadeWall, 4, 1, 19) => &[2; 4],
        (ModelKind::ArcadeWall, 2, 1, 19) => &[2; 2],
        (ModelKind::PottedPlant, 2, 4, 42) => &[25, 44, 41, 60, 57, 15, 36, 37],
        _ => return false,
    };
    p.indices(g.width).enumerate().all(|(j, i)| {
        let s = &cells[i].source;
        s.tileset_id.as_ref() == "game_corner"
            && s.metatile_id == first.metatile_id
            && s.subtile_column == first.subtile_column + (j % p.width) as u8
            && s.subtile_row == first.subtile_row + (j / p.width) as u8
            && s.tile_index == art[j]
    }) && if p.kind == ModelKind::DiningTable {
        first.subtile_column == 0 && first.subtile_row == 0
    } else if p.kind == ModelKind::ArcadeWall && p.width == 2 {
        first.subtile_column == 2 && first.subtile_row == 1
    } else if matches!(p.kind, ModelKind::PottedPlant | ModelKind::ArcadeWall) {
        first.subtile_column == 0 && first.subtile_row == 0
    } else {
        matches!(first.subtile_column, 0 | 2)
            && matches!(first.subtile_row, 0 | 2)
            && (first.metatile_id == 25 && first.subtile_row == 0
                || first.metatile_id == 6 && first.subtile_column == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Handle;
    use std::sync::Arc;

    fn fixture(block: u16, origin: [u8; 2], art: &[&[u16]]) -> (Vec<VisualTile>, GridGeometry) {
        let g = GridGeometry {
            width: art[0].len() + 2,
            height: art.len() + 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..g.width * g.height)
            .map(|i| {
                let x = i % g.width;
                let y = i / g.width;
                let drawn = x < art[0].len() && y < art.len();
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: Arc::from("game_corner"),
                        metatile_id: if drawn && block == 19 {
                            [19, 22, 23][(origin[1] as usize + y) / 4]
                        } else if drawn {
                            block
                        } else {
                            1
                        },
                        subtile_column: if drawn {
                            origin[0] + x as u8
                        } else {
                            (x % 4) as u8
                        },
                        subtile_row: if drawn {
                            (origin[1] + y as u8) % 4
                        } else {
                            (y % 4) as u8
                        },
                        tile_index: if drawn { art[y][x] } else { 1 },
                    },
                }
            })
            .collect();
        (tiles, g)
    }

    #[test]
    fn kanto_safari_complete_table_and_all_square_seat_phases_preserve_claims_and_footing() {
        let table: &[&[u16]] = &[
            &[46, 13, 13, 47],
            &[28, 12, 12, 29],
            &[62, 80, 80, 63],
            &[168, 78, 79, 169],
        ];
        let seat: &[&[u16]] = &[&[10, 11], &[26, 27]];
        let partition: &[&[u16]] = &[
            &[4, 5],
            &[20, 21],
            &[20, 21],
            &[20, 21],
            &[20, 21],
            &[20, 21],
            &[20, 21],
            &[20, 21],
            &[20, 21],
            &[20, 132],
            &[61, 30],
        ];
        let corner: &[&[u16]] = &[&[14, 14], &[30, 30]];
        let plant: &[&[u16]] = &[&[25, 44], &[41, 60], &[57, 15], &[36, 37]];
        let wall: &[&[u16]] = &[&[2; 4], &[2; 4]];
        let header: &[&[u16]] = &[&[2; 4]];
        let lower: &[&[u16]] = &[&[2; 2]];
        for (block, origin, art) in [
            (21, [0, 0], table),
            (6, [0, 0], seat),
            (6, [0, 2], seat),
            (25, [0, 0], seat),
            (25, [2, 0], seat),
            (19, [0, 1], partition),
            (23, [2, 2], corner),
            (42, [0, 0], plant),
            (2, [0, 0], wall),
            (19, [0, 0], header),
            (19, [2, 1], lower),
        ] {
            let (mut tiles, g) = fixture(block, origin, art);
            let refs = tiles.iter().collect::<Vec<_>>();
            let mut claims = vec![false; tiles.len()];
            let mut placements = Vec::new();
            candidates(
                "SafariZoneMainOffice",
                &refs,
                &g,
                &mut claims,
                &mut placements,
            );
            assert_eq!(placements.len(), 1);
            let p = &placements[0];
            assert_eq!(
                claims.iter().filter(|&&v| v).count(),
                art.len() * art[0].len()
            );
            let mut mesh = TerrainMeshData {
                footing_heights: vec![1.25; tiles.len()],
                ..Default::default()
            };
            assert!(super::super::append(
                &mut mesh,
                "SafariZoneMainOffice",
                &refs,
                &g,
                p,
                &mut vec![false; tiles.len()],
                [0, 0]
            ));
            assert!(mesh.solid.positions.is_empty());
            assert_eq!(mesh.footing_heights, vec![1.25; tiles.len()]);
            if p.kind == ModelKind::ReceptionCounter {
                assert_eq!(p.height_pixels, 8.);
                let top = mesh
                    .textured
                    .positions
                    .iter()
                    .map(|v| v[1])
                    .fold(0., f32::max);
                assert!(
                    (top - 8.002).abs() < 0.0001,
                    "counter stays at waist height, including its green top"
                );
            }
            let skip = p.width * p.height * 6;
            for tri in mesh.textured.indices[skip..].chunks_exact(3) {
                let q = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
                assert!(
                    (q[1] - q[0]).cross(q[2] - q[0]).dot(n) > 0.,
                    "closed fixture outward winding"
                );
            }
            assert!(!matches("GoldenrodGameCorner", p, &refs, &g));
            let mut reserved = vec![false; tiles.len()];
            reserved[0] = true;
            let mut out = Vec::new();
            candidates("SafariZoneMainOffice", &refs, &g, &mut reserved, &mut out);
            assert!(out.is_empty());
            drop(refs);
            for i in p.indices(g.width) {
                tiles[i].source.tile_index ^= 1;
                assert!(!matches(
                    "SafariZoneMainOffice",
                    p,
                    &tiles.iter().collect::<Vec<_>>(),
                    &g
                ));
                tiles[i].source.tile_index ^= 1;
            }
        }
    }
}

pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != "SafariZoneMainOffice" {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "game_corner", 1) else {
        return;
    };
    for (kind, width, height, depth_pixels, height_pixels, base_pixels) in [
        (ModelKind::DiningTable, 4, 4, 32., 10., 0.),
        (ModelKind::Chair, 2, 2, 8., 8., 0.),
        (ModelKind::PottedPlant, 2, 4, 12., 28., 0.),
        (ModelKind::ArcadeWall, 4, 2, 3., 24., 0.),
        (ModelKind::ArcadeWall, 4, 1, 3., 12., 12.),
        (ModelKind::ArcadeWall, 2, 1, 3., 12., 0.),
        (ModelKind::ReceptionCounter, 2, 11, 88., 8., 0.),
        (ModelKind::ReceptionCounter, 2, 2, 8., 8., 0.),
    ] {
        if g.width < width || g.height < height {
            continue;
        }
        for row in 0..=g.height - height {
            for column in 0..=g.width - width {
                let p = Placement {
                    kind,
                    row,
                    column,
                    width,
                    height,
                    ground,
                    depth_pixels,
                    height_pixels,
                    base_pixels,
                    footing_pixels: None,
                };
                if p.indices(g.width).any(|i| claimed[i]) || !matches(map, &p, cells, g) {
                    continue;
                }
                for i in p.indices(g.width) {
                    claimed[i] = true;
                }
                out.push(p);
            }
        }
    }
}

fn paint_box(out: &mut SurfaceMeshData, b: [f32; 6], uv: [f32; 2]) {
    let first = out.positions.len();
    architecture_box(out, b, [1.; 3]);
    out.uvs[first..].fill(uv);
}

pub(super) fn append(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement, b: [f32; 4]) {
    let scale = g.tile_height / 8.;
    let [w, e, n, s] = b;
    if p.kind == ModelKind::ArcadeWall {
        let index = out.indices.len();
        let base = p.base_pixels * scale;
        let height = p.height_pixels * scale;
        paint_box(
            out,
            [w, e, base, base + height, n, s],
            native_source_swatches(p, g, [[3, 0]])[0],
        );
        out.indices.drain(index + 6..index + 12);
        for y in 0..p.height {
            for x in 0..p.width {
                let uv = g.uv(p.column + x, p.row + y);
                let left = w + (e - w) * x as f32 / p.width as f32;
                let right = w + (e - w) * (x + 1) as f32 / p.width as f32;
                let low = base + (p.height - 1 - y) as f32 * height / p.height as f32;
                let high = base + (p.height - y) as f32 * height / p.height as f32;
                append_quad(
                    out,
                    [
                        [left, low, s],
                        [right, low, s],
                        [right, high, s],
                        [left, high, s],
                    ],
                    [0., 0., 1.],
                    [[uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2]],
                    TEXTURED_SHADE,
                );
            }
        }
        return;
    }
    if p.kind == ModelKind::PottedPlant {
        let cx = (w + e) * 0.5;
        let cz = (n + s) * 0.5;
        let plant = model(ModelKind::PottedPlant);
        assert!(plant.append_native_safari_plant(
            out,
            [
                cx - 6. * scale,
                cx + 6. * scale,
                cz - 6. * scale,
                cz + 6. * scale
            ],
            28. * scale,
            native_source_swatches(p, g, [[2, 3], [4, 3], [2, 3], [5, 24], [2, 24], [3, 24]])
        ));
        return;
    }
    if p.kind == ModelKind::ReceptionCounter {
        let long = p.height == 11;
        let height = p.height_pixels * scale;
        let [green, edge] = native_source_swatches(
            p,
            g,
            if long {
                [[10, 11], [9, 80]]
            } else {
                [[3, 3], [1, 8]]
            },
        );
        let (w, e, n, s) = if long {
            let cx = (w + e) * 0.5;
            (cx - 4. * scale, cx + 4. * scale, n, s)
        } else {
            (w - 4. * scale, e, n, s)
        };
        paint_box(out, [w, e, 0., height, n, s], green);
        // Gold source trim encloses the green face; no cream backing slab.
        paint_box(out, [w, e, 0., scale, n, s], edge);
        paint_box(out, [w, e, height - scale, height, n, s], edge);
        append_top(
            out,
            [w + scale, e - scale, n + scale, s - scale],
            height + 0.002 * scale,
            (green[0], green[0], green[1], green[1]),
        );
        if long {
            paint_box(out, [w, w + scale, scale, height - scale, n, s], edge);
            paint_box(out, [e - scale, e, scale, height - scale, n, s], edge);
        }
        return;
    }
    if p.kind == ModelKind::Chair {
        // The square back/seat occupies8px, half the full16px source drawing.
        let cx = (w + e) * 0.5;
        let cz = (n + s) * 0.5;
        let r = 4. * scale;
        let [seat, frame, leg] = native_source_swatches(p, g, [[3, 7], [1, 3], [3, 3]]);
        paint_box(
            out,
            [cx - r, cx + r, 4. * scale, 5. * scale, cz - r, cz + r],
            seat,
        );
        paint_box(
            out,
            [
                cx - r,
                cx + r,
                5. * scale,
                8. * scale,
                cz - r,
                cz - r + scale,
            ],
            frame,
        );
        for x in [cx - r, cx + r - scale] {
            for z in [cz - r, cz + r - scale] {
                paint_box(out, [x, x + scale, 0., 4. * scale, z, z + scale], leg);
            }
        }
        return;
    }
    let [pink, edge, leg, mat] = native_source_swatches(p, g, [[8, 3], [8, 0], [8, 0], [0, 0]]);
    append_top(out, b, 0.02 * scale, (mat[0], mat[0], mat[1], mat[1]));
    let margin = scale;
    let cut = 4. * scale;
    let (w, e, n, s) = (w + margin, e - margin, n + margin, s - margin);
    let ring = [
        [w + cut, n],
        [e - cut, n],
        [e, n + cut],
        [e, s - cut],
        [e - cut, s],
        [w + cut, s],
        [w, s - cut],
        [w, n + cut],
    ];
    let h = 10. * scale;
    let low = 9. * scale;
    let center = [(w + e) * 0.5, h, (n + s) * 0.5];
    for j in 0..8 {
        let a = ring[j];
        let b = ring[(j + 1) % 8];
        let first = out.positions.len() as u32;
        out.positions
            .extend_from_slice(&[center, [b[0], h, b[1]], [a[0], h, a[1]]]);
        out.normals.extend_from_slice(&[[0., 1., 0.]; 3]);
        out.uvs.extend_from_slice(&[pink; 3]);
        out.colors.extend_from_slice(&[TEXTURED_SHADE; 3]);
        out.indices
            .extend_from_slice(&[first, first + 1, first + 2]);
        let normal = Vec3::new(b[1] - a[1], 0., a[0] - b[0])
            .normalize()
            .to_array();
        append_quad(
            out,
            [
                [b[0], low, b[1]],
                [a[0], low, a[1]],
                [a[0], h, a[1]],
                [b[0], h, b[1]],
            ],
            normal,
            [edge; 4],
            TEXTURED_SHADE,
        );
    }
    for x in [w + 5. * scale, e - 6. * scale] {
        for z in [n + 5. * scale, s - 6. * scale] {
            paint_box(out, [x, x + scale, 0., low, z, z + scale], leg);
        }
    }
    // Seal the underside so the chamfered top stays closed in a walking view.
    for j in 0..8 {
        let a = ring[j];
        let b = ring[(j + 1) % 8];
        let first = out.positions.len() as u32;
        out.positions.extend_from_slice(&[
            [(w + e) * 0.5, low, (n + s) * 0.5],
            [a[0], low, a[1]],
            [b[0], low, b[1]],
        ]);
        out.normals.extend_from_slice(&[[0., -1., 0.]; 3]);
        out.uvs.extend_from_slice(&[edge; 3]);
        out.colors.extend_from_slice(&[TEXTURED_SHADE; 3]);
        out.indices
            .extend_from_slice(&[first, first + 1, first + 2]);
    }
}
