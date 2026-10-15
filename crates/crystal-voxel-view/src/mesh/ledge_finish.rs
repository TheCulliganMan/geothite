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
    // Keep the source face planar. Dense displaced subdivisions and added
    // fringe became upright spike curtains after terrain shaping in battles.
    let colors = p.map(|point| {
        let x = (point.x - g.origin_x) / g.tile_width + origin[0] as f32;
        let z = (point.z - g.origin_z) / g.tile_height + origin[1] as f32;
        let v = ((point.y - p[0].y.min(p[3].y)) / height).clamp(0., 1.);
        let soil = soil_color(x, z, v);
        let sod = if vegetated && v > 0.9 { 0.55 } else { 0. };
        let grass = [0.26, 0.39, 0.085];
        std::array::from_fn(|c| if c == 3 { 1. } else {
            soil[c] * (1. - sod) + grass[c] * sod
        })
    });
    append_quad_colors(mesh, p.map(|p| p.to_array()), normal, [[0.;2];4], colors);

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn soil_face_seals_perimeters_and_scrolls_with_the_source_bank() {
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
        assert!(a.positions.iter().all(|p| p[2] == 8.),
            "ledge finish must keep the source face planar");
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
