//! Weather the existing closed maze stones without separating their joins.
use super::*;

fn relief(x: f32, z: f32) -> (f32, f32, f32) {
    let (a, da) = (0.193 * x + 0.317 * z).sin_cos();
    let (b, db) = (0.463 * x - 0.239 * z).sin_cos();
    let (c, dc) = (0.127 * x - 0.571 * z).sin_cos();
    (
        0.5 + 0.22 * a + 0.18 * b + 0.10 * c,
        0.22 * 0.193 * da + 0.18 * 0.463 * db + 0.10 * 0.127 * dc,
        0.22 * 0.317 * da - 0.18 * 0.239 * db - 0.10 * 0.571 * dc,
    )
}

pub(super) fn finish(mesh: &mut SurfaceMeshData, start: usize, b: [f32; 4], rise: f32) {
    let scale = (b[1] - b[0]).min(b[3] - b[2]) / 8.;
    if b.iter().any(|v| !v.is_finite())
        || !scale.is_finite()
        || scale <= 0.
        || !rise.is_finite()
        || rise <= 0.
    {
        return;
    }
    // A shared world field gives both copies of every buried join the same
    // height. The cubic ramp keeps the original floor contact and lowers the
    // coping inside its existing envelope. Its vertical derivative stays >0,
    // so the original closed parts and their overlap cannot turn inside out.
    let amplitude = (2.4 * scale).min(rise * 0.24);
    for i in start..mesh.positions.len() {
        let mut p = mesh.positions[i];
        let old_normal = Vec3::from_array(mesh.normals[i]);
        let mut normal = old_normal;
        let t = (p[1] / rise).clamp(0., 1.);
        let ramp = t * t * t;
        // Break the prototype's repeated concentric top rings. Two bounded
        // one-axis deformations keep a positive Jacobian at every point:
        // 1 - .85 * (PI / 8 + .61) > .14. The window is exactly zero on
        // every section boundary, so adjacent buried faces remain coincident.
        for (axis, kx, kz, phase) in [(0, 0.51, 0.37, 1.1), (2, -0.43, 0.61, 2.3)] {
            let u = ((p[0] - b[0]) / (b[1] - b[0])).clamp(0., 1.);
            let v = ((p[2] - b[2]) / (b[3] - b[2])).clamp(0., 1.);
            let (mut su, cu) = (std::f32::consts::PI * u).sin_cos();
            let (mut sv, cv) = (std::f32::consts::PI * v).sin_cos();
            if u == 0. || u == 1. {
                su = 0.;
            }
            if v == 0. || v == 1. {
                sv = 0.;
            }
            let window = su * sv;
            let wx = std::f32::consts::PI / (b[1] - b[0]) * cu * sv;
            let wz = std::f32::consts::PI / (b[3] - b[2]) * su * cv;
            let (f, df) = (kx * p[0] / scale + kz * p[2] / scale + phase).sin_cos();
            let jitter = 0.85 * scale;
            let gradient = Vec3::new(
                jitter * ramp * (wx * f + window * kx * df / scale),
                jitter * 3. * t * t / rise * window * f,
                jitter * ramp * (wz * f + window * kz * df / scale),
            );
            let component = normal[axis] / (1. + gradient[axis]);
            normal -= gradient * component;
            normal[axis] = component;
            p[axis] += jitter * window * f * ramp;
        }
        let (depth, dx, dz) = relief(p[0] / scale, p[2] / scale);
        let dy = 1. - 3. * amplitude * depth * t * t / rise;
        let slope_x = -amplitude * dx * ramp / scale;
        let slope_z = -amplitude * dz * ramp / scale;
        let normal = Vec3::new(
            normal.x - slope_x * normal.y / dy,
            normal.y / dy,
            normal.z - slope_z * normal.y / dy,
        )
        .normalize();
        p[1] -= amplitude * depth * ramp;
        mesh.positions[i] = p;
        mesh.normals[i] = normal.to_array();
        let old_shade = crate::interior_models::authored_face_color([1.; 4], old_normal)[0];
        let new_shade = crate::interior_models::authored_face_color([1.; 4], normal)[0];
        let grain = 1. - 0.08 * depth * ramp;
        for channel in &mut mesh.colors[i][..3] {
            *channel *= new_shade / old_shade * grain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn point(p: [f32; 3]) -> [i32; 3] {
        p.map(|v| (v * 10000.).round() as i32)
    }

    #[test]
    fn kanto_maze_weathering_preserves_closed_parts_bounds_and_floor_contact() {
        for mask in 0..16 {
            for scale in [0.5, 1., 2.] {
                for offset in [[0., 0.], [16., -24.], [-32., 40.]] {
                    let b = [
                        offset[0] * scale,
                        (offset[0] + 8.) * scale,
                        offset[1] * scale,
                        (offset[1] + 8.) * scale,
                    ];
                    let rise = 12. * scale;
                    let model = crate::dungeon_models::gym_model(4 + mask);
                    let mut mesh = SurfaceMeshData::default();
                    model.append(&mut mesh, b, 0., rise);
                    let original = mesh.clone();
                    finish(&mut mesh, 0, b, rise);
                    assert_eq!(mesh.indices, original.indices);
                    assert_eq!(mesh.uvs, original.uvs);
                    assert_eq!(mesh.cutaway_ranges, original.cutaway_ranges);
                    let mut copies = BTreeMap::new();
                    let mut heights = BTreeSet::new();
                    for (i, (before, after)) in
                        original.positions.iter().zip(&mesh.positions).enumerate()
                    {
                        assert!(after[0] >= b[0] && after[0] <= b[1]);
                        assert!(after[2] >= b[2] && after[2] <= b[3]);
                        if before[0] == b[0]
                            || before[0] == b[1]
                            || before[2] == b[2]
                            || before[2] == b[3]
                        {
                            assert_eq!(
                                [before[0], before[2]],
                                [after[0], after[2]],
                                "section outlines and joins stay fixed"
                            );
                        }
                        assert!(after[1] >= 0. && after[1] <= before[1]);
                        if before[1] == 0. {
                            assert_eq!(before, after);
                        }
                        if (before[1] - rise).abs() < 0.0001 {
                            heights.insert((after[1] / scale * 1000.).round() as i32);
                        }
                        if let Some(other) = copies.insert(point(*before), point(*after)) {
                            assert_eq!(
                                other,
                                point(*after),
                                "hard edges and closed-part copies remain coincident"
                            );
                        }
                        assert!((Vec3::from_array(mesh.normals[i]).length() - 1.).abs() < 0.0001);
                    }
                    assert!(
                        heights.len() > 4,
                        "the coping must have real nonplanar relief"
                    );
                    for t in mesh.indices.chunks_exact(3) {
                        let [a, b, c] = [t[0], t[1], t[2]]
                            .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                        assert!(
                            (b - a)
                                .cross(c - a)
                                .dot(Vec3::from_array(mesh.normals[t[0] as usize]))
                                > 0.
                        );
                    }
                    let mut cached = SurfaceMeshData::default();
                    model.append(&mut cached, b, 0., rise);
                    assert_eq!(cached, original, "cached models remain immutable");
                }
            }
        }
    }

    #[test]
    fn kanto_maze_weathering_keeps_adjacent_buried_faces_coincident() {
        for scale in [0.5, 1., 2.] {
            let mut faces = Vec::new();
            for (mask, x) in [(13, 0.), (7, 8.)] {
                let b = [x * scale, (x + 8.) * scale, -16. * scale, -8. * scale];
                let mut mesh = SurfaceMeshData::default();
                crate::dungeon_models::gym_model(4 + mask).append(&mut mesh, b, 0., 12. * scale);
                finish(&mut mesh, 0, b, 12. * scale);
                faces.push(
                    mesh.positions
                        .iter()
                        .filter(|p| (p[0] - 8. * scale).abs() < 0.0001)
                        .map(|p| point(p.map(|v| v / scale)))
                        .collect::<BTreeSet<_>>(),
                );
            }
            assert!(faces[0].len() > 4);
            assert_eq!(
                faces[0], faces[1],
                "neighboring closed faces must meet throughout their height"
            );
        }
    }
}
