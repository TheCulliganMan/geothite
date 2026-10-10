//! Goldenrod's complete ornamental garden drawing, distinct from animated flowers.
//! Source identity and ownership select art; navigation remains in the controller.
use super::*;
use crate::new_bark_models::{ModelKind, model};

const DRAWING: [[u16; 4]; 4] = [
    [0xd4, 0xd5, 0x2f, 0xde],
    [0xd6, 0xd7, 0xde, 0x2f],
    [0xd4, 0xd5, 0x2f, 0xde],
    [0xd6, 0xd7, 0x2f, 0x2f],
];

fn box_faces(mesh: &mut SurfaceMeshData, b: [f32; 4], y: f32, height: f32, color: [f32; 4]) {
    let [w, e, n, s] = b;
    for (p, normal) in [
        (
            [
                [w, y + height, n],
                [w, y + height, s],
                [e, y + height, s],
                [e, y + height, n],
            ],
            [0., 1., 0.],
        ),
        (
            [[w, y, s], [e, y, s], [e, y + height, s], [w, y + height, s]],
            [0., 0., 1.],
        ),
        (
            [[e, y, n], [w, y, n], [w, y + height, n], [e, y + height, n]],
            [0., 0., -1.],
        ),
        (
            [[w, y, n], [w, y, s], [w, y + height, s], [w, y + height, n]],
            [-1., 0., 0.],
        ),
        (
            [[e, y, s], [e, y, n], [e, y + height, n], [e, y + height, s]],
            [1., 0., 0.],
        ),
    ] {
        append_solid_quad(mesh, p, normal, color);
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
    for row in 0..g.height.saturating_sub(3) {
        for col in 0..g.width.saturating_sub(3) {
            if !(0..4).all(|y| {
                (0..4).all(|x| {
                    let i = (row + y) * g.width + col + x;
                    let s = &cells[i].source;
                    !claimed[i]
                        && s.tileset_id.as_ref() == "johto_modern"
                        && s.metatile_id == 0x66
                        && s.subtile_column == x as u8
                        && s.subtile_row == y as u8
                        && s.tile_index == DRAWING[y][x]
                })
            }) {
                continue;
            }
            // The right-hand half is authored ornamental paving. Keep its full
            // original acreage, replacing the flat mosaic with warm paper stone.
            for y in 0..4 {
                for x in 0..4 {
                    let i = (row + y) * g.width + col + x;
                    let (w, e, n, s) = g.bounds(col + x, row + y);
                    append_solid_quad(
                        &mut mesh.solid,
                        [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]],
                        [0., 1., 0.],
                        [0.70, 0.68, 0.59, 1.],
                    );
                    claimed[i] = true;
                    mesh.authored_cells[i] = Some("goldenrod/garden");
                }
            }
            for y in [0, 2] {
                let (w, _, n, _) = g.bounds(col, row + y);
                let u = g.tile_width.min(g.tile_height);
                let plot = [
                    w + u * 0.13,
                    w + g.tile_width * 2. - u * 0.13,
                    n + u * 0.13,
                    n + g.tile_height * 2. - u * 0.13,
                ];
                box_faces(&mut mesh.solid, plot, 0., u * 0.20, [0.48, 0.39, 0.29, 1.]);
                let soil = [
                    plot[0] + u * 0.09,
                    plot[1] - u * 0.09,
                    plot[2] + u * 0.09,
                    plot[3] - u * 0.09,
                ];
                box_faces(
                    &mut mesh.solid,
                    soil,
                    u * 0.20,
                    u * 0.015,
                    [0.22, 0.28, 0.13, 1.],
                );
                // Map coordinates retain variety while the camera scrolls.
                let seed = ((col as i32 + origin[0]).wrapping_mul(73856093)
                    ^ (row as i32 + y as i32 + origin[1]).wrapping_mul(19349663))
                    as u32;
                let phase = (seed % 997) as f32 / 997.;
                let flower = model(ModelKind::Flowers);
                let first = mesh.solid.positions.len();
                flower.append_fitted(
                    &mut mesh.solid,
                    soil,
                    u * 0.215,
                    u * (0.9 + phase * 0.3) / (flower.max[1] - flower.min[1]),
                    None,
                );
                if phase > 0.5 {
                    let cx = (soil[0] + soil[1]) * 0.5;
                    let cz = (soil[2] + soil[3]) * 0.5;
                    for i in first..mesh.solid.positions.len() {
                        let p = &mut mesh.solid.positions[i];
                        p[0] = 2. * cx - p[0];
                        p[2] = 2. * cz - p[2];
                        let n = &mut mesh.solid.normals[i];
                        let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
                        let before = 0.62 + 0.38 * Vec3::from_array(*n).dot(light).max(0.);
                        n[0] = -n[0];
                        n[2] = -n[2];
                        let after = 0.62 + 0.38 * Vec3::from_array(*n).dot(light).max(0.);
                        for channel in 0..3 {
                            mesh.solid.colors[i][channel] *= after / before;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn only_complete_unclaimed_garden_drawings_replace_source_art() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut tiles: Vec<_> = (0..16)
            .map(|i| VisualTile {
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("johto_modern"),
                    metatile_id: 0x66,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: DRAWING[i / 4][i % 4],
                },
                texture: Handle::default(),
                animation_frames: None,
                priority: false,
            })
            .collect();
        let build = |tiles: &[VisualTile], map: &str, occupied: bool| {
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 16],
                ..Default::default()
            };
            let mut claimed = vec![false; 16];
            claimed[0] = occupied;
            append(
                &mut mesh,
                map,
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                [40, 10],
                &mut claimed,
            );
            (mesh, claimed)
        };
        let (mesh, claimed) = build(&tiles, "GoldenrodCity", false);
        assert!(claimed.iter().all(|x| *x));
        assert!(mesh.solid.positions.len() > 1000);
        assert!(mesh.solid.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 32.
            && p[2] >= 0.
            && p[2] <= 32.
            && p[1] >= 0.
            && p[1] < 12.));
        assert!(
            build(&tiles, "GoldenrodCity", true)
                .0
                .solid
                .positions
                .is_empty()
        );
        assert!(build(&tiles, "Route34", false).0.solid.positions.is_empty());
        tiles[15].source.tile_index = 0;
        assert!(
            build(&tiles, "GoldenrodCity", false)
                .0
                .solid
                .positions
                .is_empty()
        );
    }
}
