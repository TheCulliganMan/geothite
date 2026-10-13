//! Earthen relief on source-verified shallow Johto jump banks.
//! Perimeters and support stay fixed; only the visible face gains detail.
use super::*;
use bevy::prelude::Vec3;

pub(in crate::mesh) fn soil_color(x: f32, z: f32, fraction: f32) -> [f32;3] {
    let grain = meadow_noise(x * 4.1 + 7., z * 4.1 - 11.) * 0.065;
    let strata = (fraction * 11.0 + meadow_noise(x * 1.9, z * 1.9) * 1.7).sin() * 0.018;
    [0.29 + grain + strata, 0.19 + grain * 0.70 + strata, 0.080 + grain * 0.35]
}

pub(in crate::mesh) fn append(
    mesh: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) -> bool {
    append_soil(mesh, p, normal, g, origin, true)
}

/// Source paths expose soil at the water drop without adding lawn plants.
pub(in crate::mesh) fn append_bare_soil(
    mesh: &mut SurfaceMeshData, p: [[f32; 3]; 4], normal: [f32; 3],
    g: &GridGeometry, origin: [i32; 2],
) -> bool {
    append_soil(mesh, p, normal, g, origin, false)
}

fn append_soil(
    mesh: &mut SurfaceMeshData, p: [[f32; 3]; 4], normal: [f32; 3],
    g: &GridGeometry, origin: [i32; 2], vegetated: bool,
) -> bool {
    let n = Vec3::from_array(normal);
    if n.y.abs() > 0.001 || n.length_squared() < 0.99 {
        return false;
    }
    let p = p.map(Vec3::from_array);
    if p[1].y <= p[0].y || p[2].y <= p[3].y {
        return false;
    }
    let width = (p[3] - p[0]).length();
    let height = (p[1].y - p[0].y).max(p[2].y - p[3].y);
    if width < g.tile_width * 0.1 || height > g.tile_height * 0.8 {
        return false;
    }
    let map = |p: Vec3| {
        Vec3::new(
            (p.x - g.origin_x) / g.tile_width + origin[0] as f32,
            p.y,
            (p.z - g.origin_z) / g.tile_height + origin[1] as f32,
        )
    };
    let anchor = p[0];
    let relative = p.map(|q| q - anchor);
    let source_anchor = map(anchor);
    let relative_map = |q: Vec3| Vec3::new(
        source_anchor.x + q.x / g.tile_width,
        source_anchor.y + q.y,
        source_anchor.z + q.z / g.tile_height,
    );
    let relative_point = |u: f32, v: f32| {
        let q = relative[0].lerp(relative[3], u)
            .lerp(relative[1].lerp(relative[2], u), v);
        // sin(PI) is not exactly zero in f32. Return the perimeter itself
        // so relief cannot move native cap corners or floor contacts.
        if u == 0. || u == 1. || v == 0. || v == 1. {
            return q;
        }
        let m = relative_map(q);
        let ripple = 0.70
            + meadow_noise(m.x * 3.7 + 23., m.z * 3.7 - 41.) * 0.5
            + meadow_noise(m.x * 9.1 + v * 3.4, m.z * 9.1 - v * 2.7) * 0.28;
        // Zero displacement around the complete perimeter seals adjacent
        // banks, ramp ends and the unchanged walking cap.
        let relief = (std::f32::consts::PI * u).sin()
            * (std::f32::consts::PI * v).sin()
            * height
            * 0.20
            * ripple;
        q + n * relief
    };
    let point = |u: f32, v: f32| {
        match (u, v) {
            (0., 0.) => p[0],
            (0., 1.) => p[1],
            (1., 1.) => p[2],
            (1., 0.) => p[3],
            _ => anchor + relative_point(u, v),
        }
    };
    let color = |m: Vec3, v: f32, face: Vec3| {
        let light = Vec3::new(0.45, 0.80, 0.55).normalize();
        let shade = (0.62 + 0.38 * v.sqrt()) * (0.70 + 0.30 * face.dot(light).max(0.));
        // Sod hangs farther over moist pockets, exposing irregular soil
        // between them. Sample world coordinates so neighboring faces share
        // the same turf boundary rather than repeating a per-tile scallop.
        let sod = meadow_noise(m.x * 2.3 + 43., m.z * 2.3 - 17.)
            + meadow_noise(m.x * 6.7, m.z * 6.7) * 0.35;
        let edge = 0.75 + sod * 0.22;
        let rim = ((v - edge) / 0.12).clamp(0., 1.);
        let rim = if vegetated { rim * rim * (3. - 2. * rim) } else { 0. };
        let soil = soil_color(m.x,m.z,v);
        let grass = [0.26, 0.39, 0.085];
        std::array::from_fn(|c| {
            if c == 3 {
                1.
            } else {
                (soil[c] * (1. - rim) + grass[c] * rim).clamp(0., 1.) * shade
            }
        })
    };
    let surface_normal = |u: f32, v: f32| {
        if u == 0. || u == 1. || v == 0. || v == 1. {
            return n;
        }
        // Different viewport translations must not change the precision of
        // these short derivatives, and hence the bank's lighting.
        let du = relative_point((u + 0.002).min(1.), v)
            - relative_point((u - 0.002).max(0.), v);
        let dv = relative_point(u, (v + 0.002).min(1.))
            - relative_point(u, (v - 0.002).max(0.));
        dv.cross(du).normalize()
    };
    const COLUMNS: usize = 20;
    const ROWS: usize = 8;
    for row in 0..ROWS {
        for col in 0..COLUMNS {
            let uv = [
                (col as f32 / COLUMNS as f32, row as f32 / ROWS as f32),
                (col as f32 / COLUMNS as f32, (row + 1) as f32 / ROWS as f32),
                (
                    (col + 1) as f32 / COLUMNS as f32,
                    (row + 1) as f32 / ROWS as f32,
                ),
                ((col + 1) as f32 / COLUMNS as f32, row as f32 / ROWS as f32),
            ];
            let q = uv.map(|(u, v)| point(u, v));
            let normals = uv.map(|(u, v)| surface_normal(u, v));
            let face = (q[1] - q[0]).cross(q[2] - q[0]).normalize();
            append_quad_colors(
                mesh,
                q.map(|p| p.to_array()),
                face.to_array(),
                [[0.; 2]; 4],
                std::array::from_fn(|i| color(
                    relative_map(relative_point(uv[i].0, uv[i].1)), uv[i].1, normals[i])),
            );
            let first = mesh.normals.len() - 4;
            for (i, vertex_normal) in normals.into_iter().enumerate() {
                mesh.normals[first + i] = vertex_normal.to_array();
            }
        }
    }
    if !vegetated { return true; }
    // Short hanging grass and exposed roots break the ruler-straight rim.
    // Basal fans grow from its unchanged rim; hanging strands lie on the face.
    let tangent = (p[3] - p[0]).normalize();
    for strand in 0..16 {
        let provisional = point((strand as f32 + 0.5) / 16., 1.);
        let m = map(provisional);
        let jitter = meadow_noise(m.x * 13.1 + 19., m.z * 13.1 - 23.) * 0.90;
        let u = (strand as f32 + 0.5 + jitter) / 16.;
        let root = point(u, 1.0);
        let m = map(root);
        let seed = meadow_noise(m.x * 7.3 + 29., m.z * 7.3 - 37.) + 0.5;
        let length = height * (0.16 + seed * 0.48);
        let root_width = width * (0.003 + seed * 0.004);
        let tuft = meadow_noise(m.x * 2.1 + 53., m.z * 2.1 - 31.) > -0.04
            && seed > 0.30;
        if tuft {
            let blades = 2 + (seed * 2.9) as usize;
            for blade in 0..blades {
                let fan = blade as f32 - (blades - 1) as f32 * 0.5;
                let blade_width = (width * (0.010 + seed * 0.012))
                    .min(width * u.min(1. - u) * 0.7);
                let side = tangent * blade_width;
                let tip_u = (u + fan * height / width * 0.09).clamp(0.008, 0.992);
                let growth = height * (0.20 + seed * 0.12)
                    * (0.82 + 0.18 * (seed * 7. + blade as f32 * 1.7).cos());
                let tip = point(tip_u, 1.) + Vec3::Y * growth + n * height * 0.12;
                let mid = root.lerp(tip, 0.55) + n * height * 0.055;
                let q = [
                    (root + side).to_array(),
                    (root - side).to_array(),
                    (mid - side * 0.6).to_array(),
                    (mid + side * 0.6).to_array(),
                ];
                append_quad_colors(
                    mesh,
                    q,
                    n.to_array(),
                    [[0.; 2]; 4],
                    [[0.14, 0.27, 0.045, 1.]; 4],
                );
                let q = [
                    (mid + side * 0.6).to_array(),
                    (mid - side * 0.6).to_array(),
                    tip.to_array(),
                    tip.to_array(),
                ];
                let first = mesh.positions.len() as u32;
                mesh.positions.extend(q[..3].iter().copied());
                mesh.normals.extend([n.to_array(); 3]);
                mesh.uvs.extend([[0.; 2]; 3]);
                mesh.colors.extend([[0.24, 0.36, 0.075, 1.]; 3]);
                mesh.indices.extend([first, first + 1, first + 2]);
            }
        }
        let exposed_root = meadow_noise(m.x * 11.7 - 13., m.z * 11.7 + 47.) > 0.18;
        if !tuft && !exposed_root { continue; }
        for section in 0..3 {
            let t0 = section as f32 / 3.;
            let t1 = (section + 1) as f32 / 3.;
            let center = |t: f32| {
                root - Vec3::Y * length * t
                    + n * (height * 0.012 + height * 0.04 * t * (1. - t))
                    + tangent * height * 0.035 * (t * 4. + seed * 6.).sin() * t
            };
            let a = center(t0);
            let b = center(t1);
            let wa = tangent * root_width * (1. - t0 * 0.85);
            let wb = tangent * root_width * (1. - t1 * 0.85);
            let roots = exposed_root && !tuft;
            let color = if roots {
                [0.16, 0.095, 0.035, 1.]
            } else {
                [0.12, 0.26, 0.048, 1.]
            };
            append_quad_colors(
                mesh,
                [
                    (a - wa).to_array(),
                    (a + wa).to_array(),
                    (b + wb).to_array(),
                    (b - wb).to_array(),
                ],
                n.to_array(),
                [[0.; 2]; 4],
                [color; 4],
            );
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relief_seals_perimeters_and_scrolls_with_the_source_bank() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = [[8., 0., 8.], [8., 3., 8.], [0., 3., 8.], [0., 0., 8.]];
        let mut a = SurfaceMeshData::default();
        assert!(append(&mut a, p, [0., 0., 1.], &g, [10, 9]));
        assert!(p.iter().all(|p| a.positions.contains(p)));
        for corner in p {
            assert!(a.positions.iter().zip(&a.normals)
                .any(|(position, normal)| *position == corner && *normal == [0., 0., 1.]),
                "native support corners retain exact positions and normals");
        }
        assert!(
            a.positions
                .iter()
                .all(|p| p[1] >= 0. && p[1] <= 4.5 && p[0] >= 0. && p[0] <= 8.)
        );
        assert!(a.positions.iter().any(|p| p[2] > 8.1));
        for ids in a.indices.chunks_exact(3) {
            let q = ids
                .iter()
                .map(|i| Vec3::from_array(a.positions[*i as usize]))
                .collect::<Vec<_>>();
            let n = Vec3::from_array(a.normals[ids[0] as usize]);
            assert!((q[1] - q[0]).cross(q[2] - q[0]).dot(n) > 0.);
        }
        let shifted_g = GridGeometry { origin_x: -8., ..g };
        let mut b = SurfaceMeshData::default();
        assert!(append(
            &mut b,
            p.map(|p| [p[0] - 8., p[1], p[2]]),
            [0., 0., 1.],
            &shifted_g,
            [10, 9]
        ));
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.normals, b.normals, "scrolling must not alter bank lighting");
        for (a, b) in a.colors.iter().zip(&b.colors) {
            assert!(a.iter().zip(b).all(|(a, b)| (*a - *b).abs() < 1e-6));
        }
        let mut floor = SurfaceMeshData::default();
        assert!(!append(
            &mut floor,
            [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]],
            [0., 1., 0.],
            &g,
            [10, 9]
        ));
        assert!(floor.positions.is_empty());
    }
}

#[cfg(test)]
mod directions {
    use super::*;
    #[test]
    fn each_cardinal_bank_has_outward_winding_and_keeps_its_support_corners() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: -16.,
        };
        for turn in 0..4 {
            let rotate =
                bevy::prelude::Quat::from_rotation_y(turn as f32 * std::f32::consts::FRAC_PI_2);
            let p = [[8., 0., 8.], [8., 3., 8.], [0., 3., 8.], [0., 0., 8.]]
                .map(|p| (rotate * Vec3::from_array(p)).to_array());
            let n = (rotate * Vec3::Z).to_array();
            let mut mesh = SurfaceMeshData::default();
            assert!(append(&mut mesh, p, n, &g, [0, 0]));
            for corner in p {
                assert!(mesh.positions.iter().zip(&mesh.normals).any(|(q, m)| {
                    Vec3::from_array(*q).distance(Vec3::from_array(corner)) < 1e-5
                        && Vec3::from_array(*m).distance(Vec3::from_array(n)) < 1e-5
                }));
            }
            for ids in mesh.indices.chunks_exact(3) {
                let q = ids
                    .iter()
                    .map(|i| Vec3::from_array(mesh.positions[*i as usize]))
                    .collect::<Vec<_>>();
                let n = Vec3::from_array(mesh.normals[ids[0] as usize]);
                assert!((q[1] - q[0]).cross(q[2] - q[0]).dot(n) > 0.);
            }
        }
    }
}
