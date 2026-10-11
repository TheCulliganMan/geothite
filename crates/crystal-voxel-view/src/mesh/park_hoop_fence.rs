//! Open hoop rails on the native park's low gold masonry posts.
use super::*;
use crate::park::HoopFenceKind;

pub(super) fn append(mesh: &mut SurfaceMeshData, b: [f32; 4], unit: f32, kind: HoopFenceKind) {
    let [w, e, n, s] = b;
    // Opposite drawings put the rail in front of / behind its low post row.
    let z = n
        + (s - n)
            * if kind == HoopFenceKind::Light {
                0.34
            } else {
                0.70
            };
    let cx = (w + e) * 0.5;
    let radius = (e - w) * 0.34;
    let shoulder = unit * 0.54;
    let crown = unit * 1.10;
    for x in [cx - radius, cx + radius] {
        post(mesh, x, z, unit);
    }
    let mut points = vec![[cx - radius, unit * 0.27, z]];
    for i in 0..=16 {
        let a = std::f32::consts::PI * (1.0 - i as f32 / 16.0);
        points.push([
            cx + radius * a.cos(),
            shoulder + (crown - shoulder) * a.sin(),
            z,
        ]);
    }
    points.push([cx + radius, unit * 0.27, z]);
    tube_path(mesh, &points, unit * 0.055, [0.64, 0.67, 0.64, 1.]);
}

/// Side rails share the horizontal kit's scale and material, rotated around Y.
pub(in crate::mesh) fn append_side(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    unit: f32,
    east: bool,
    turn: Option<super::super::park_side_fence::source::Turn>,
) {
    let [w, e, n, s] = b;
    let plane = if east { e - unit * 0.5 } else { w + unit * 0.5 };
    let mut local = SurfaceMeshData::default();
    for i in 0..2 {
        let a = n + (s - n) * i as f32 / 2.;
        let b = n + (s - n) * (i + 1) as f32 / 2.;
        append(&mut local, [a, b, 0., unit * 2.], unit, HoopFenceKind::Dark);
    }
    let offset = mesh.positions.len() as u32;
    mesh.positions.extend(
        local
            .positions
            .into_iter()
            .map(|p| [plane - (p[2] - unit * 1.4), p[1], p[0]]),
    );
    mesh.normals
        .extend(local.normals.into_iter().map(|p| [-p[2], p[1], p[0]]));
    mesh.uvs.extend(local.uvs);
    mesh.colors.extend(local.colors);
    mesh.indices
        .extend(local.indices.into_iter().map(|i| i + offset));
    if let Some(turn) = turn {
        let x = if east { w + unit * 0.5 } else { e - unit * 0.5 };
        let z = match turn {
            super::super::park_side_fence::source::Turn::North => n + unit * 0.68,
            super::super::park_side_fence::source::Turn::South => s - unit * 0.60,
        };
        post(mesh, x, z, unit);
    }
}

fn post(mesh: &mut SurfaceMeshData, x: f32, z: f32, u: f32) {
    let w = x - u * 0.19;
    let e = x + u * 0.19;
    let n = z - u * 0.13;
    let s = z + u * 0.13;
    let h = u * 0.38;
    for (p, normal, color) in [
        (
            [[w, h, n], [w, h, s], [e, h, s], [e, h, n]],
            [0., 1., 0.],
            [0.57, 0.43, 0.19, 1.],
        ),
        (
            [[w, 0., s], [e, 0., s], [e, h, s], [w, h, s]],
            [0., 0., 1.],
            [0.44, 0.31, 0.12, 1.],
        ),
        (
            [[e, 0., n], [w, 0., n], [w, h, n], [e, h, n]],
            [0., 0., -1.],
            [0.40, 0.29, 0.14, 1.],
        ),
        (
            [[w, 0., n], [w, 0., s], [w, h, s], [w, h, n]],
            [-1., 0., 0.],
            [0.46, 0.33, 0.15, 1.],
        ),
        (
            [[e, 0., s], [e, 0., n], [e, h, n], [e, h, s]],
            [1., 0., 0.],
            [0.52, 0.39, 0.18, 1.],
        ),
    ] {
        append_quad_colors(mesh, p, normal, [[0.; 2]; 4], [color; 4]);
    }
}

fn tube_path(mesh: &mut SurfaceMeshData, points: &[[f32; 3]], r: f32, color: [f32; 4]) {
    // Shared rings at bends prevent the gaps left by independently fitted rods.
    let rings: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, &point)| {
            let before = Vec3::from_array(points[i.saturating_sub(1)]);
            let after = Vec3::from_array(points[(i + 1).min(points.len() - 1)]);
            let radial = (after - before).normalize().cross(Vec3::Z).normalize();
            let center = Vec3::from_array(point);
            std::array::from_fn::<_, 8, _>(|j| {
                let a = j as f32 * std::f32::consts::TAU / 8.;
                (center + (Vec3::Z * a.cos() + radial * a.sin()) * r).to_array()
            })
        })
        .collect();
    for pair in rings.windows(2) {
        for i in 0..8 {
            let j = (i + 1) % 8;
            let p = [pair[0][i], pair[0][j], pair[1][j], pair[1][i]];
            let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
                .cross(Vec3::from_array(p[3]) - Vec3::from_array(p[0]))
                .normalize()
                .to_array();
            append_quad_colors(mesh, p, normal, [[0.; 2]; 4], [color; 4]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn park_fences_have_open_centers_and_floor_rooted_posts() {
        for kind in [HoopFenceKind::Light, HoopFenceKind::Dark] {
            let mut m = SurfaceMeshData::default();
            append(&mut m, [20., 36., -12., 4.], 8., kind);
            assert!(m.positions.len() > 500);
            assert!(m.positions.iter().all(|p| p.iter().all(|v| v.is_finite())
                && p[1] >= 0.
                && p[0] >= 20.
                && p[0] <= 36.
                && p[2] >= -12.
                && p[2] <= 4.));
            assert!(m.positions.iter().any(|p| p[1] == 0.));
            // The center opening is unfilled, rather than a dense canopy cube.
            assert!(
                !m.positions
                    .iter()
                    .any(|p| (p[0] - 28.).abs() < 2. && p[1] > 3. && p[1] < 6.)
            );
            for t in m.indices.chunks_exact(3) {
                let a = Vec3::from_array(m.positions[t[0] as usize]);
                let b = Vec3::from_array(m.positions[t[1] as usize]);
                let c = Vec3::from_array(m.positions[t[2] as usize]);
                assert!((b - a).cross(c - a).length() > 0.0001);
            }
        }
    }
}
