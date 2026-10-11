//! A ladder laid on the existing shelf ramp, with actual rails and rungs.
//! Its bed has exactly the old continuous support plane; no foot cache writes.
use super::*;

pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    origin: [i32; 2],
    dark: bool,
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + 2. * g.tile_width;
    let s = n + 2. * g.tile_height;
    let u = g.tile_height / SOURCE_TILE_HEIGHT;
    let rise = crate::cave::CAVE_SHELF_HEIGHT * u;
    let y = |z: f32| rise * (s - z) / (s - n);
    let normal = Vec3::new(0., s - n, rise).normalize().to_array();
    let bed = [[w, rise, n], [w, 0., s], [e, 0., s], [e, rise, n]];
    let stone = if dark {
        [0.31, 0.37, 0.39, 1.]
    } else {
        [0.34, 0.345, 0.31, 1.]
    };
    append_quad_colors(
        out,
        bed,
        normal,
        bed.map(|p| cave_cliff::floor_uv(Vec3::from_array(p), g, origin, false)),
        bed.map(|p| cave_cliff::pigment(stone, Vec3::from_array(p), g, origin, false)),
    );
    let side_start = out.positions.len();
    for (x, normal) in [(w, [-1., 0., 0.]), (e, [1., 0., 0.])] {
        let mut points = [[x, 0., n], [x, 0., s], [x, 0., s], [x, rise, n]];
        if x == e {
            points.reverse();
        }
        append_solid_quad(out, points, normal, [0.32, 0.30, 0.24, 1.]);
    }
    append_solid_quad(
        out,
        [[e, 0., n], [w, 0., n], [w, rise, n], [e, rise, n]],
        [0., 0., -1.],
        [0.32, 0.30, 0.24, 1.],
    );
    for j in side_start..out.positions.len() {
        let p = Vec3::from_array(out.positions[j]);
        out.uvs[j] = cave_cliff::stone_uv(p, g, origin, false);
        out.colors[j] = cave_cliff::pigment(
            [stone[0] * 0.86, stone[1] * 0.86, stone[2] * 0.86, 1.],
            p,
            g,
            origin,
            false,
        );
    }
    // Prisms follow the ramp, so each end joins its own shelf datum.
    let mut beam = |x0: f32, x1: f32, z0: f32, z1: f32, color: [f32; 4]| {
        let low = 0.05 * u;
        let high = 0.55 * u;
        let bevel = 0.12 * u;
        let shoulder = high - bevel;
        let a = y(z0);
        let b = y(z1);
        for (points, normal, shade) in [
            (
                [
                    [x0 + bevel, y(z0 + bevel) + high, z0 + bevel],
                    [x0 + bevel, y(z1 - bevel) + high, z1 - bevel],
                    [x1 - bevel, y(z1 - bevel) + high, z1 - bevel],
                    [x1 - bevel, y(z0 + bevel) + high, z0 + bevel],
                ],
                normal,
                1.,
            ),
            (
                [
                    [x0, b + low, z1],
                    [x1, b + low, z1],
                    [x1, b + shoulder, z1],
                    [x0, b + shoulder, z1],
                ],
                [0., 0., 1.],
                0.8,
            ),
            (
                [
                    [x1, a + low, z0],
                    [x0, a + low, z0],
                    [x0, a + shoulder, z0],
                    [x1, a + shoulder, z0],
                ],
                [0., 0., -1.],
                0.7,
            ),
            (
                [
                    [x0, a + low, z0],
                    [x0, b + low, z1],
                    [x0, b + shoulder, z1],
                    [x0, a + shoulder, z0],
                ],
                [-1., 0., 0.],
                0.75,
            ),
            (
                [
                    [x1, b + low, z1],
                    [x1, a + low, z0],
                    [x1, a + shoulder, z0],
                    [x1, b + shoulder, z1],
                ],
                [1., 0., 0.],
                0.9,
            ),
        ] {
            append_solid_quad(
                out,
                points,
                normal,
                [color[0] * shade, color[1] * shade, color[2] * shade, 1.],
            );
        }
        // Worn top edges retain the source footprint and the ramp slope.
        let ring = |inset: f32, height: f32| {
            [
                Vec3::new(x0 + inset, y(z0 + inset) + height, z0 + inset),
                Vec3::new(x0 + inset, y(z1 - inset) + height, z1 - inset),
                Vec3::new(x1 - inset, y(z1 - inset) + height, z1 - inset),
                Vec3::new(x1 - inset, y(z0 + inset) + height, z0 + inset),
            ]
        };
        let outer = ring(0., shoulder);
        let inner = ring(bevel, high);
        for i in 0..4 {
            let j = (i + 1) % 4;
            let q = [outer[i], outer[j], inner[j], inner[i]];
            let normal = (q[1] - q[0]).cross(q[2] - q[0]).normalize();
            let shade = 0.72 + 0.28 * normal.dot(Vec3::new(-0.4, 0.8, 0.45).normalize()).max(0.);
            append_solid_quad(
                out,
                q.map(|p| p.to_array()),
                normal.to_array(),
                [color[0] * shade, color[1] * shade, color[2] * shade, 1.],
            );
        }
    };
    let left = w + 2. * u;
    let right = e - 2. * u;
    for x in [left, right] {
        beam(
            x - 0.4 * u,
            x + 0.4 * u,
            n + 0.25 * u,
            s - 0.25 * u,
            [0.43, 0.29, 0.13, 1.],
        );
    }
    for rung in 0..5 {
        let z = n + (rung as f32 + 0.5) * (s - n) / 5.;
        beam(
            left,
            right,
            z - 0.35 * u,
            z + 0.35 * u,
            [0.65, 0.48, 0.26, 1.],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_shelf_ladder_replaces_art_and_preserves_the_ramp_support() {
        for (block, art) in [
            (0x14, [[0x2a, 0x2b], [0x3a, 0x3b]]),
            (0x15, [[0x22, 0x23], [0x32, 0x33]]),
        ] {
            let mut t = super::super::tests::tiles(4, 4, "cave");
            for cell in &mut t {
                cell.source.tile_index = 0x16;
            }
            for y in 0..2 {
                for x in 0..2 {
                    let s = &mut t[(y + 2) * 4 + x + 2].source;
                    s.metatile_id = block;
                    s.tile_index = art[y][x];
                }
            }
            let refs = t.iter().collect::<Vec<_>>();
            let g = super::super::tests::grid(4, 4);
            let placements = resolve("MountMortar2FInside", &refs, &g, None);
            assert_eq!(placements.len(), 1);
            assert_eq!(placements[0].kind_label(), "dungeon:cave-shelf-ladder");
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0., 0., 6., 3.],
                ..Default::default()
            };
            super::super::append(&mut mesh, &g, &placements[0], &refs, [0, 0]);
            assert_eq!(mesh.footing_heights, vec![0., 0., 6., 3.]);
            let h = 6. * g.tile_height / SOURCE_TILE_HEIGHT;
            assert_eq!(mesh.solid.positions[0][1], h);
            assert_eq!(mesh.solid.positions[1][1], 0.);
            assert!(mesh.solid.uvs[..4].iter().all(|uv| (uv[1]/100000.).round()==4.));
            assert!(mesh.solid.uvs[4..16].iter().all(|uv| (uv[1]/100000.).round()==6.));
            assert!(mesh.solid.normals.iter().zip(&mesh.solid.uvs).any(|(n,uv)|
                *uv==[0.,0.] && n[0].abs()>0.1 && n[1]>0.1),"wood bevel has a real sloping face");
            assert!(mesh.solid.positions.len() > 100);
            assert!(
                mesh.solid
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            t[15].source.tile_index = 0xff;
            assert!(
                resolve(
                    "MountMortar2FInside",
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    None
                )
                .is_empty()
            );
        }
    }
}
