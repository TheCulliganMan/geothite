//! Goldenrod's authored east/west track and pedestrian crossing.
//! Complete three-cell source columns select the kit; the sidewalk remains live.
use super::*;

fn block(mesh: &mut SurfaceMeshData, b: [f32; 6], ink: [f32; 3]) {
    let [w, e, y, h, n, s] = b;
    let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
    for (p, normal) in [
        ([[w, h, n], [w, h, s], [e, h, s], [e, h, n]], [0., 1., 0.]),
        ([[w, y, s], [w, y, n], [e, y, n], [e, y, s]], [0., -1., 0.]),
        ([[w, y, s], [e, y, s], [e, h, s], [w, h, s]], [0., 0., 1.]),
        ([[e, y, n], [w, y, n], [w, h, n], [e, h, n]], [0., 0., -1.]),
        ([[w, y, n], [w, y, s], [w, h, s], [w, h, n]], [-1., 0., 0.]),
        ([[e, y, s], [e, y, n], [e, h, n], [e, h, s]], [1., 0., 0.]),
    ] {
        let shade = 0.62 + 0.38 * Vec3::from_array(normal).dot(light).max(0.);
        append_solid_quad(
            mesh,
            p,
            normal,
            [ink[0] * shade, ink[1] * shade, ink[2] * shade, 1.],
        );
    }
}

fn column(mesh: &mut SurfaceMeshData, b: [f32; 4], crossing: bool, seed: u32) {
    let [w, e, n, s] = b;
    let u = (s - n) / 3.;
    append_solid_quad(
        mesh,
        [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]],
        [0., 1., 0.],
        [0.47, 0.44, 0.36, 1.],
    );
    if crossing {
        // Flush card panels retain the authored pedestrian opening. Thin slots
        // separate the rail heads, without placing sleepers across the walkway.
        for (a, b) in [(0., 0.36), (0.64, 2.36), (2.64, 3.)] {
            let count = if b - a > 1. { 4 } else { 1 };
            for i in 0..count {
                let z0 = n + u * (a + (b - a) * i as f32 / count as f32) + u * 0.014;
                let z1 = n + u * (a + (b - a) * (i + 1) as f32 / count as f32) - u * 0.014;
                block(mesh, [w, e, 0., u * 0.14, z0, z1], [0.70, 0.68, 0.59]);
            }
        }
    } else {
        // Sparse, stable stone chips and timber grain replace the yellow pixel
        // strip. Small variation is seeded in map coordinates, not the camera.
        for i in 0..6u32 {
            let h = seed.wrapping_add(i.wrapping_mul(0x9e3779b9));
            let x = w + (e - w) * (0.09 + (h % 79) as f32 / 100.);
            let z = n + u * (0.09 + ((h >> 8) % 278) as f32 / 100.);
            let dx = (e - w) * 0.035;
            let dz = u * 0.035;
            let tone = ((h >> 16) % 7) as f32 * 0.012;
            block(
                mesh,
                [x - dx, x + dx, 0., u * 0.025, z - dz, z + dz],
                [0.53 + tone, 0.50 + tone, 0.41 + tone],
            );
        }
        let center = (w + e) * 0.5;
        let half = (e - w) * 0.13;
        let tone = (seed % 11) as f32 * 0.006;
        block(
            mesh,
            [
                center - half,
                center + half,
                0.,
                u * 0.06,
                n + u * 0.18,
                s - u * 0.18,
            ],
            [0.38 + tone, 0.29 + tone, 0.22 + tone],
        );
        block(
            mesh,
            [
                center - half * 0.6,
                center - half * 0.45,
                u * 0.06,
                u * 0.065,
                n + u * 0.22,
                s - u * 0.22,
            ],
            [0.30, 0.24, 0.18],
        );
    }
    // Two low rail webs and wider pale heads continue exactly across columns.
    for z in [n + u * 0.5, n + u * 2.5] {
        block(
            mesh,
            [w, e, u * 0.06, u * 0.11, z - u * 0.035, z + u * 0.035],
            [0.28, 0.32, 0.33],
        );
        block(
            mesh,
            [w, e, u * 0.11, u * 0.14, z - u * 0.065, z + u * 0.065],
            [0.65, 0.69, 0.68],
        );
        if !crossing {
            let cx = (w + e) * 0.5;
            for dz in [-0.13, 0.13] {
                block(
                    mesh,
                    [
                        cx - u * 0.045,
                        cx + u * 0.045,
                        u * 0.06,
                        u * 0.085,
                        z + u * dz - u * 0.035,
                        z + u * dz + u * 0.035,
                    ],
                    [0.30, 0.33, 0.31],
                );
            }
        }
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    claimed: &mut [bool],
) {
    if map != "GoldenrodCity" {
        return;
    }
    for row in 0..g.height.saturating_sub(2) {
        for col in 0..g.width {
            let first = &cells[row * g.width + col].source;
            if first.tileset_id.as_ref() != "johto_modern"
                || first.subtile_row != 0
                || first.subtile_column >= 4
                || !matches!(first.metatile_id, 0x7e | 0x7f)
            {
                continue;
            }
            let crossing = first.metatile_id == 0x7f;
            if !(0..3).all(|y| {
                let i = (row + y) * g.width + col;
                let s = &cells[i].source;
                !claimed[i]
                    && s.tileset_id == first.tileset_id
                    && s.metatile_id == first.metatile_id
                    && s.subtile_column == first.subtile_column
                    && s.subtile_row == y as u8
                    && s.tile_index
                        == if y == 1 {
                            if crossing { 0x06 } else { 0x46 }
                        } else {
                            0x36
                        }
            }) {
                continue;
            }
            let (w, e, n, _) = g.bounds(col, row);
            let (_, _, _, s) = g.bounds(col, row + 2);
            let seed = (col as i32 + origin[0]) as u32;
            let seed = seed.wrapping_mul(0x85ebca6b)
                ^ ((row as i32 + origin[1]) as u32).wrapping_mul(0xc2b2ae35);
            column(&mut mesh.solid, [w, e, n, s], crossing, seed);
            for y in 0..3 {
                let i = (row + y) * g.width + col;
                claimed[i] = true;
                mesh.authored_cells[i] = Some("goldenrod/railway");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn track_and_crossing_require_the_exact_source_column_and_keep_sidewalk() {
        let g = GridGeometry {
            width: 1,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        for crossing in [false, true] {
            let mut tiles: Vec<_> = (0..4)
                .map(|y| VisualTile {
                    column: 0,
                    row: y,
                    source: VisualTileSource {
                        tileset_id: Arc::from("johto_modern"),
                        metatile_id: if crossing { 0x7f } else { 0x7e },
                        subtile_column: 2,
                        subtile_row: y as u8,
                        tile_index: match y {
                            0 | 2 => 0x36,
                            1 => {
                                if crossing {
                                    0x06
                                } else {
                                    0x46
                                }
                            }
                            _ => 0x06,
                        },
                    },
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                })
                .collect();
            let build = |tiles: &[VisualTile], map: &str, occupied: bool| {
                let mut mesh = TerrainMeshData {
                    authored_cells: vec![None; 4],
                    footing_heights: vec![0.; 4],
                    ..Default::default()
                };
                let mut claimed = vec![false; 4];
                claimed[1] = occupied;
                append(
                    &mut mesh,
                    map,
                    &tiles.iter().collect::<Vec<_>>(),
                    &g,
                    [28, 24],
                    &mut claimed,
                );
                (mesh, claimed)
            };
            let (mesh, claimed) = build(&tiles, "GoldenrodCity", false);
            assert_eq!(claimed, [true, true, true, false]);
            assert_eq!(mesh.authored_cells[3], None);
            assert_eq!(mesh.footing_heights, [0.; 4]);
            assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
                && p[0] <= 8.
                && p[2] >= 0.
                && p[2] <= 24.
                && p[1] >= 0.
                && p[1] <= 1.12));
            assert_eq!(mesh, build(&tiles, "GoldenrodCity", false).0);
            assert!(build(&tiles, "Route34", false).0.solid.positions.is_empty());
            assert!(
                build(&tiles, "GoldenrodCity", true)
                    .0
                    .solid
                    .positions
                    .is_empty()
            );
            tiles[2].source.tile_index = 0;
            assert!(
                build(&tiles, "GoldenrodCity", false)
                    .0
                    .solid
                    .positions
                    .is_empty()
            );
        }
    }
}
