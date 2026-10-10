//! World-anchored paper brick streets, limestone flags and flat curb courses.
use super::*;

pub(super) fn borders(materials: &[Option<GroundMaterial>], g: &GridGeometry) -> Vec<u8> {
    (0..materials.len())
        .map(|i| {
            if materials[i] != Some(GroundMaterial::Path) {
                return 0;
            }
            let (x, z) = (i % g.width, i / g.width);
            [
                (1, x.checked_sub(1).map(|x| z * g.width + x)),
                (2, (x + 1 < g.width).then_some(i + 1)),
                (4, z.checked_sub(1).map(|z| z * g.width + x)),
                (8, (z + 1 < g.height).then_some(i + g.width)),
            ]
            .into_iter()
            .filter_map(|(bit, j)| {
                j.filter(|j| materials[*j] == Some(GroundMaterial::Pavers))
                    .map(|_| bit)
            })
            .fold(0, |flags, bit| flags | bit)
        })
        .collect()
}

fn face(mesh: &mut SurfaceMeshData, b: [f32; 4], y: f32, color: [f32; 3]) {
    let [w, e, n, s] = b;
    if e <= w || s <= n {
        return;
    }
    append_solid_quad(
        mesh,
        [[w, y, n], [w, y, s], [e, y, s], [e, y, n]],
        [0., 1., 0.],
        [color[0], color[1], color[2], 1.],
    );
}
fn clipped(mesh: &mut SurfaceMeshData, b: [f32; 4], clip: [f32; 4], y: f32, c: [f32; 3]) {
    face(
        mesh,
        [
            b[0].max(clip[0]),
            b[1].min(clip[1]),
            b[2].max(clip[2]),
            b[3].min(clip[3]),
        ],
        y,
        c,
    );
}

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    g: &GridGeometry,
    origin: [i32; 2],
    material: GroundMaterial,
    flags: u8,
) {
    let [w, e, n, s] = face_bounds(&p);
    let y = p[0][1];
    let cell_x = ((w - g.origin_x) / g.tile_width).round() as i32 + origin[0];
    let cell_z = ((n - g.origin_z) / g.tile_height).round() as i32 + origin[1];
    let curb = g.tile_width.min(g.tile_height) * 0.12;
    let [pw, pe, pn, ps] = [1, 2, 4, 8].map(|bit| if flags & bit != 0 { curb } else { 0. });
    let clip = [w + pw, e - pe, n + pn, s - ps];
    let c = [0.62, 0.60, 0.51];
    // Each curb strip is a disjoint part of the original plane. No lifted
    // decal, changed walking datum, or coplanar face overlap is introduced.
    face(mesh, [w, e, n, n + pn], y, c);
    face(mesh, [w, e, s - ps, s], y, c);
    face(mesh, [w, w + pw, n + pn, s - ps], y, c);
    face(mesh, [e - pe, e, n + pn, s - ps], y, c);
    let road = material == GroundMaterial::Pavers;
    let rows = if road { 4 } else { 2 };
    let columns = if road { 2 } else { 1 };
    let bw = g.tile_width / columns as f32;
    let bh = g.tile_height / rows as f32;
    let gap = g.tile_width.min(g.tile_height) * 0.011;
    let edge = gap * 0.8;
    for row in 0..rows {
        let world_row = cell_z * rows + row;
        let offset = if world_row.rem_euclid(2) == 0 {
            0.
        } else {
            bw * 0.5
        };
        let zn = n + row as f32 * bh;
        for col in -1..columns {
            let xw = w + col as f32 * bw + offset;
            let xe = xw + bw;
            let variation = (lattice(cell_x * columns + col, world_row) - 0.5) * 0.045;
            let base = if road {
                [0.57, 0.43, 0.35]
            } else {
                [0.72, 0.69, 0.60]
            };
            let color = base.map(|c| c + variation);
            let seam = if road {
                [0.42, 0.38, 0.32]
            } else {
                [0.56, 0.55, 0.48]
            };
            // Seven non-overlapping strips partition a complete stone, clipped
            // only at this source face. A clipped half-stone gains no false seam.
            for (b, c) in [
                ([xw, xw + gap, zn, zn + bh], seam),
                ([xe - gap, xe, zn, zn + bh], seam),
                ([xw + gap, xe - gap, zn, zn + gap], seam),
                ([xw + gap, xe - gap, zn + bh - gap, zn + bh], seam),
                (
                    [xw + gap, xe - gap, zn + gap, zn + gap + edge],
                    color.map(|c| c + 0.035),
                ),
                (
                    [xw + gap, xe - gap, zn + gap + edge, zn + bh - gap - edge],
                    color,
                ),
                (
                    [xw + gap, xe - gap, zn + bh - gap - edge, zn + bh - gap],
                    color.map(|c| c - 0.025),
                ),
            ] {
                clipped(mesh, b, clip, y, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paving_partitions_original_plane_and_keeps_world_phase() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let positions = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        for material in [GroundMaterial::Path, GroundMaterial::Pavers] {
            for flags in [0, 1, 2, 4, 8, 15] {
                let mut m = SurfaceMeshData::default();
                append(&mut m, positions, &g, [37, 29], material, flags);
                let area: f32 = m
                    .positions
                    .chunks_exact(4)
                    .map(|p| (p[2][0] - p[0][0]) * (p[2][2] - p[0][2]))
                    .sum();
                assert!((area - 64.).abs() < 0.001);
                // Equal total area alone could hide an overlap plus a hole.
                // Probe interior points, including narrow mortar/curb strips.
                for z in 0..41 {
                    for x in 0..41 {
                        let px = (x as f32 + 0.37) * 8. / 41.;
                        let pz = (z as f32 + 0.37) * 8. / 41.;
                        let owners = m
                            .positions
                            .chunks_exact(4)
                            .filter(|q| {
                                px >= q[0][0] && px < q[2][0] && pz >= q[0][2] && pz < q[2][2]
                            })
                            .count();
                        assert_eq!(owners, 1, "paving overlap/hole at {px},{pz}");
                    }
                }

                assert!(m.positions.iter().all(|p| p[1] == 0.
                    && p[0] >= 0.
                    && p[0] <= 8.
                    && p[2] >= 0.
                    && p[2] <= 8.));
                assert!(m.normals.iter().all(|n| *n == [0., 1., 0.]));
                let shifted_g = GridGeometry {
                    origin_x: -80.,
                    origin_z: 40.,
                    ..g
                };
                let shifted = positions.map(|p| [p[0] - 80., p[1], p[2] + 40.]);
                let mut same = SurfaceMeshData::default();
                append(&mut same, shifted, &shifted_g, [37, 29], material, flags);
                assert_eq!(m.colors, same.colors);
                assert_eq!(m.indices, same.indices);
                assert!(
                    m.positions
                        .iter()
                        .zip(same.positions)
                        .all(|(a, b)| (a[0] - 80. - b[0]).abs() < 0.0001
                            && (a[2] + 40. - b[2]).abs() < 0.0001)
                );
            }
        }
    }
    #[test]
    fn curb_bands_require_the_real_path_brick_boundary() {
        let g = GridGeometry {
            width: 3,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut m = vec![Some(GroundMaterial::Lawn); 9];
        m[4] = Some(GroundMaterial::Path);
        assert_eq!(borders(&m, &g), vec![0; 9]);
        for i in [1, 3, 5, 7] {
            m[i] = Some(GroundMaterial::Pavers);
        }
        assert_eq!(borders(&m, &g)[4], 15);
        assert!(
            borders(&m, &g)
                .iter()
                .enumerate()
                .all(|(i, b)| i == 4 || *b == 0)
        );
    }
}
