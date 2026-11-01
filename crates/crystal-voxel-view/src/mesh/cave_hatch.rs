//! Upright cave ladders rooted on the authored walking surface.
use super::*;

fn beam(out: &mut SurfaceMeshData, a: Vec3, b: Vec3, width: f32, depth: f32, color: [f32; 4]) {
    let axis = (b - a).normalize();
    let u = if axis.y.abs() > 0.9 {
        Vec3::X
    } else {
        axis.cross(Vec3::Y).normalize()
    };
    let v = axis.cross(u).normalize();
    let ring = [
        [-0.5, -0.25],
        [-0.25, -0.5],
        [0.25, -0.5],
        [0.5, -0.25],
        [0.5, 0.25],
        [0.25, 0.5],
        [-0.25, 0.5],
        [-0.5, 0.25],
    ];
    let point = |center: Vec3, i: usize| center + u * ring[i][0] * width + v * ring[i][1] * depth;
    for i in 0..8 {
        let j = (i + 1) % 8;
        let p = [point(a, i), point(a, j), point(b, j), point(b, i)];
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        let shade = 0.84 + 0.16 * normal.dot(Vec3::new(-0.4, 0.7, 0.6)).max(0.);
        append_solid_quad(
            out,
            p.map(|p| p.to_array()),
            normal.to_array(),
            [color[0] * shade, color[1] * shade, color[2] * shade, 1.],
        );
        for (normal, p) in [
            (-axis, [a, point(a, j), point(a, i)]),
            (axis, [b, point(b, i), point(b, j)]),
        ] {
            let first = out.positions.len() as u32;
            out.positions.extend(p.map(|p| p.to_array()));
            out.normals.extend([normal.to_array(); 3]);
            out.uvs.extend([[0.; 2]; 3]);
            out.colors
                .extend([[color[0] * 0.68, color[1] * 0.68, color[2] * 0.68, color[3]]; 3]);
            out.indices.extend([first, first + 1, first + 2]);
        }
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    dark: bool,
    ice: bool,
    _origin: [i32; 2],
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let unit = g.tile_height / SOURCE_TILE_HEIGHT;
    let z = |y: f32| n + (9. - 4. * y / 27.) * unit;
    let rails = if dark {
        [0.34, 0.25, 0.15, 1.]
    } else if ice {
        [0.47, 0.34, 0.20, 1.]
    } else {
        [0.44, 0.29, 0.12, 1.]
    };
    let rungs = [0.60, 0.43, 0.23, 1.];
    let left = w + 3.2 * unit;
    let right = w + 12.8 * unit;
    for x in [left, right] {
        beam(
            &mut mesh.solid,
            Vec3::new(x, 0.14 * unit, z(0.14)),
            Vec3::new(x, 26.86 * unit, z(26.86)),
            1.1 * unit,
            1.4 * unit,
            rails,
        );
    }
    for i in 0..9 {
        let y = 2.5 + i as f32 * 3.;
        beam(
            &mut mesh.solid,
            Vec3::new(left, y * unit, z(y)),
            Vec3::new(right, y * unit, z(y)),
            0.9 * unit,
            0.8 * unit,
            rungs.map(|channel| channel * if i % 3 == 0 { 0.90 } else { 1.0 }),
        );
    }
    for y in [3.0, 23.0] {
        for x in [left, right] {
            beam(
                &mut mesh.solid,
                Vec3::new(x, y * unit, z(y) - 0.7 * unit),
                Vec3::new(x, y * unit, z(y) + 0.8 * unit),
                0.6 * unit,
                0.6 * unit,
                [0.21, 0.19, 0.16, 1.],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beveled_ladder_beams_have_closed_caps_and_outward_faces() {
        for (a, b) in [
            (Vec3::new(3.2, 0.14, 9.), Vec3::new(3.2, 26.86, 2.)),
            (Vec3::new(3.2, 2.5, 8.5), Vec3::new(12.8, 2.5, 8.5)),
        ] {
            let mut mesh = SurfaceMeshData::default();
            beam(&mut mesh, a, b, 1.1, 1.4, [0.4, 0.3, 0.2, 1.]);
            assert_eq!(mesh.indices.len(), 96);
            let center = (a + b) * 0.5;
            for triangle in mesh.indices.chunks_exact(3) {
                let p = [triangle[0], triangle[1], triangle[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let geometric = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
                let authored = Vec3::from_array(mesh.normals[triangle[0] as usize]);
                assert!(geometric.dot(authored) > 0.999);
                assert!(authored.dot((p[0] + p[1] + p[2]) / 3. - center) > 0.);
            }
        }
    }
}

/// Downward rails disappear into a lined shaft; only the grab handles rise
/// above the lip. The gameplay support and warp remain at the source datum.
pub(super) fn append_shaft(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    datum: f32,
    origin: [i32; 2],
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + 2. * g.tile_width;
    let s = n + 2. * g.tile_height;
    let u = g.tile_height / SOURCE_TILE_HEIGHT;
    // A chipped eight-sided opening follows the source hole rather than
    // exposing a perfect rectangular floor-cell cutout.
    let contour = [
        (3.2, 2.0),
        (8.0, 1.7),
        (13.0, 2.3),
        (14.1, 8.0),
        (12.6, 14.0),
        (7.8, 14.4),
        (2.4, 13.0),
        (1.7, 7.5),
    ];
    let inner: [[f32; 2]; 8] = contour.map(|(x, z)| [w + x * u, n + z * u]);
    let outer = [
        [w, n],
        [w + 8. * u, n],
        [e, n],
        [e, n + 8. * u],
        [e, s],
        [w + 8. * u, s],
        [w, s],
        [w, n + 8. * u],
    ];
    let edge = datum - 0.75 * u;
    for i in 0..8 {
        let j = (i + 1) % 8;
        let q = [
            [outer[i][0], datum, outer[i][1]],
            [inner[i][0], edge, inner[i][1]],
            [inner[j][0], edge, inner[j][1]],
            [outer[j][0], datum, outer[j][1]],
        ];
        let v = q.map(Vec3::from_array);
        let normal = (v[1] - v[0]).cross(v[2] - v[0]).normalize();
        let uv = q.map(|point| {
            let mut uv = cave_cliff::floor_uv(Vec3::from_array(point), g, origin, false);
            uv[1] += 2_600_000.;
            uv
        });
        append_quad_colors(
            &mut mesh.solid,
            q,
            normal.to_array(),
            uv,
            [
                [0.402, 0.402, 0.402, 1.],
                [0.22, 0.22, 0.22, 1.],
                [0.22, 0.22, 0.22, 1.],
                [0.402, 0.402, 0.402, 1.],
            ],
        );
    }
    let lining_start = mesh.solid.positions.len();
    let depth = datum - 13. * u;
    // Close the unseen bottom with a fan, keeping the entire shaft below
    // authored support. No cap or foundation crosses the opening at datum.
    for i in 0..8 {
        let j = (i + 1) % 8;
        let a = inner[i];
        let b = inner[j];
        let first = mesh.solid.positions.len() as u32;
        mesh.solid.positions.extend([
            [w + 8. * u, depth, n + 8. * u],
            [b[0], depth, b[1]],
            [a[0], depth, a[1]],
        ]);
        mesh.solid.normals.extend([[0., 1., 0.]; 3]);
        mesh.solid.colors.extend([[0.009, 0.011, 0.016, 1.]; 3]);
        mesh.solid.uvs.extend([[0.; 2]; 3]);
        mesh.solid.indices.extend([first, first + 1, first + 2]);
        for k in 0..4 {
            let top = edge + (depth - edge) * k as f32 / 4.;
            let bottom = edge + (depth - edge) * (k + 1) as f32 / 4.;
            let q = [
                [a[0], top, a[1]],
                [a[0], bottom, a[1]],
                [b[0], bottom, b[1]],
                [b[0], top, b[1]],
            ];
            let v = q.map(Vec3::from_array);
            let normal = (v[1] - v[0]).cross(v[2] - v[0]).normalize();
            let shade = |k: usize| {
                let gain = (1. - k as f32 / 4.).powi(2);
                [
                    0.018 + 0.30 * gain,
                    0.021 + 0.29 * gain,
                    0.028 + 0.27 * gain,
                    1.,
                ]
            };
            append_quad_colors(
                &mut mesh.solid,
                q,
                normal.to_array(),
                [[0.; 2]; 4],
                [shade(k), shade(k + 1), shade(k + 1), shade(k)],
            );
        }
    }
    for i in lining_start..mesh.solid.positions.len() {
        let mut uv =
            cave_cliff::stone_uv(Vec3::from_array(mesh.solid.positions[i]), g, origin, false);
        uv[1] += 2_000_000.;
        mesh.solid.uvs[i] = uv;
    }
    let left = w + 4. * u;
    let right = e - 4. * u;
    let rail_z = n + 5.8 * u;
    for x in [left, right] {
        beam(
            &mut mesh.solid,
            Vec3::new(x, depth + u, rail_z),
            Vec3::new(x, datum + 5. * u, n + 1.2 * u),
            1.1 * u,
            1.3 * u,
            [0.36, 0.27, 0.17, 1.],
        );
    }
    for i in 0..5 {
        let y = depth + (2. + i as f32 * 3.) * u;
        let z = rail_z - (y - depth - u) / (17. * u) * 4.6 * u;
        let gain = 0.20 + 0.80 * ((y - depth) / (datum - depth)).clamp(0., 1.);
        beam(
            &mut mesh.solid,
            Vec3::new(left, y, z),
            Vec3::new(right, y, z),
            0.9 * u,
            0.8 * u,
            [0.56 * gain, 0.40 * gain, 0.22 * gain, 1.],
        );
    }
}

/// A stone mouth at the authored outdoor walk-off, with a recessed sky-facing
/// reveal. Its arch stays within the source footprint and leaves the approach open.
pub(super) fn append_portal(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    origin: [i32; 2],
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + 2. * g.tile_width;
    let s = n + 2. * g.tile_height;
    let u = g.tile_height / SOURCE_TILE_HEIGHT;
    let datum = crate::cave::CAVE_SHELF_HEIGHT * u;
    new_bark::cave_finish::append(
        &mut mesh.solid,
        [[w, datum, n], [w, datum, s], [e, datum, s], [e, datum, n]],
        [0., 1., 0.],
        &cells[p.ground].source,
        g,
        origin,
    );
    // A connected rock cross-section, with a broad brow and tapered cheeks.
    // The inner contour is a traversable opening; the outer contour is rock,
    // not a rectangular lintel balanced on two independent boxes.
    let inner = [
        (2.2, 0.),
        (2.6, 12.8),
        (3.4, 18.6),
        (5.4, 20.1),
        (8.3, 22.4),
        (11.2, 20.3),
        (12.8, 17.2),
        (14.2, 13.5),
        (13.8, 0.),
    ];
    let outer = [
        (0., 0.),
        (0.3, 17.0),
        (0.8, 22.3),
        (3.5, 25.3),
        (7.4, 27.0),
        (11.4, 26.1),
        (15.2, 22.0),
        (16., 16.5),
        (16., 0.),
    ];
    let front_depth = [12., 11., 9.5, 8.0, 7.6, 8.8, 10.2, 12.0, 12.];
    let point = |xy: (f32, f32), z: f32| [w + xy.0 * u, datum + xy.1 * u, n + z * u];
    let front = inner.map(|xy| point(xy, 9.0));
    let back = inner.map(|xy| point((8. + (xy.0 - 8.) * 0.86, xy.1 * 0.96), 0.4));
    let outer_front: [[f32; 3]; 9] = std::array::from_fn(|i| point(outer[i], front_depth[i]));
    let outer_back = outer.map(|xy| point(xy, 0.));
    let stone = |out: &mut SurfaceMeshData, q: [[f32; 3]; 4], color: [f32; 4], lining: bool| {
        let v = q.map(Vec3::from_array);
        let normal = (v[1] - v[0]).cross(v[2] - v[0]).normalize();
        let first = out.positions.len();
        append_solid_quad(out, q, normal.to_array(), color);
        for i in first..out.positions.len() {
            let at = Vec3::from_array(out.positions[i]);
            let mut uv = cave_cliff::stone_uv(at, g, origin, false);
            // Portal rock has its own forward-pass body aperture. Ordinary
            // low stones remain opaque, and the doorway's support is intact.
            uv[1] += if lining { 2_800_000. } else { 2_600_000. };
            out.uvs[i] = uv;
            out.colors[i] = cave_cliff::pigment(out.colors[i], at, g, origin, false);
        }
    };
    for i in 0..8 {
        let j = i + 1;
        // Beveled front rock and a real tunnel reveal, all within the native
        // footprint. The open bottom has no threshold across the walking path.
        stone(
            &mut mesh.solid,
            [outer_front[i], front[i], front[j], outer_front[j]],
            [0.40, 0.39, 0.35, 1.],
            false,
        );
        stone(
            &mut mesh.solid,
            [outer_front[j], outer_back[j], outer_back[i], outer_front[i]],
            [0.37, 0.36, 0.32, 1.],
            false,
        );
        stone(
            &mut mesh.solid,
            [outer_back[j], back[j], back[i], outer_back[i]],
            [0.30, 0.29, 0.26, 1.],
            false,
        );
        let q = [front[i], back[i], back[j], front[j]];
        let v = q.map(Vec3::from_array);
        let normal = (v[1] - v[0]).cross(v[2] - v[0]).normalize();
        let first = mesh.solid.positions.len();
        append_quad_colors(
            &mut mesh.solid,
            q,
            normal.to_array(),
            [[0.; 2]; 4],
            [
                [0.30, 0.30, 0.30, 1.],
                [0.09, 0.09, 0.09, 1.],
                [0.09, 0.09, 0.09, 1.],
                [0.30, 0.30, 0.30, 1.],
            ],
        );
        for k in first..mesh.solid.positions.len() {
            let mut uv =
                cave_cliff::stone_uv(Vec3::from_array(mesh.solid.positions[k]), g, origin, false);
            uv[1] += 2_800_000.;
            mesh.solid.uvs[k] = uv;
        }
    }
    // The bright distant opening follows the vault silhouette. It is set
    // behind the shaded lining, rather than filling the mouth like a door.
    for i in 1..7 {
        let left = back[i];
        let right = back[i + 1];
        let first = mesh.solid.positions.len();
        append_solid_quad(
            &mut mesh.solid,
            [
                [left[0], datum, left[2]],
                [right[0], datum, right[2]],
                right,
                left,
            ],
            [0., 0., 1.],
            [0.65, 0.65, 0.65, 1.],
        );
        for k in first..mesh.solid.positions.len() {
            let mut uv =
                cave_cliff::stone_uv(Vec3::from_array(mesh.solid.positions[k]), g, origin, false);
            uv[1] += 2_200_000.;
            mesh.solid.uvs[k] = uv;
        }
    }
}

#[cfg(test)]
mod portal_tests {
    use super::*;

    #[test]
    fn outdoor_exit_is_an_open_arch_with_authored_floor_and_no_new_support() {
        let g = super::super::tests::grid(4, 4);
        let mut tiles = super::super::tests::tiles(4, 4, "cave");
        for tile in &mut tiles {
            tile.source.metatile_id = 0x02;
            tile.source.tile_index = 0x16;
        }
        for i in [10, 11, 14, 15] {
            tiles[i].source.metatile_id = 0x24;
            tiles[i].source.tile_index = 0x01;
        }
        let cells = tiles.iter().collect::<Vec<_>>();
        let placements = resolve("UnionCave1F", &cells, &g, None);
        let p = placements
            .iter()
            .find(|p| matches!(p.form, Form::CavePortal))
            .unwrap();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![6.; 16],
            ..Default::default()
        };
        super::super::append(&mut mesh, &g, p, &cells, [0, 0]);
        assert_eq!(mesh.footing_heights, vec![6.; 16]);
        assert!(!p.cuts_foundation());
        let categories = mesh.solid.uvs.iter().map(|uv| (uv[1] / 100000.).round() as u32)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(categories.contains(&32) && categories.contains(&34),
            "portal shell and lining must expose the forward body aperture");
        assert!(categories.contains(&28), "sky opening keeps its source light role");
        assert!(mesh.solid.positions.iter().all(|p| p[0] >= 16.
            && p[0] <= 32.
            && p[2] >= 16.
            && p[2] <= 32.
            && p[1] >= 6.));
        // The source approach has no vertical surface blocking its center.
        assert!(!mesh.solid.positions.chunks_exact(4).any(|face| {
            face.iter().all(|p| p[2] >= 24. && p[1] > 6. && p[1] < 20.)
                && face.iter().any(|p| p[0] < 24.)
                && face.iter().any(|p| p[0] > 24.)
        }));
        tiles[10].source.tile_index = 0xff;
        assert!(
            !resolve("UnionCave1F", &tiles.iter().collect::<Vec<_>>(), &g, None)
                .iter()
                .any(|p| matches!(p.form, Form::CavePortal))
        );
    }
}
