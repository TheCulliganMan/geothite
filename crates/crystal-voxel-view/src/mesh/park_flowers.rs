//! Native flower heads and low foliage within source-complete garden beds.
use super::*;

pub(in crate::mesh) fn append_underplanting(mesh: &mut SurfaceMeshData, bounds: [f32; 4], cell: [i32; 2]) {
    let [w, e, n, s] = bounds;
    let scale = (e - w).min(s - n);
    let seed = (cell[0] as u32).wrapping_mul(0x9e3779b9)
        ^ (cell[1] as u32).wrapping_mul(0x85ebca6b);
    let phase = (seed % 997) as f32 / 997. * std::f32::consts::TAU;
    for (plant, [x, z]) in [[0.27, 0.27], [0.73, 0.27], [0.27, 0.73], [0.73, 0.73]]
        .into_iter().enumerate()
    {
        let turn = phase + plant as f32 * 2.399963;
        let center = Vec3::new(w + (e - w) * x + turn.cos() * scale * 0.009,
            scale * 0.018, n + (s - n) * z + turn.sin() * scale * 0.009);
        for leaf in 0..8 {
            let a = turn + leaf as f32 * std::f32::consts::TAU / 8.;
            let along = Vec3::new(a.cos(), 0., a.sin());
            let across = Vec3::new(-a.sin(), 0., a.cos());
            let length = scale * (0.235 + 0.014 * (a * 1.7 + phase).sin());
            let middle = center + along * length * 0.53
                + Vec3::Y * scale * (0.20 + 0.035 * (a + phase).sin());
            let tip = center + along * length
                + Vec3::Y * scale * (0.10 + 0.035 * (a * 1.3).cos());
            let left = middle - across * scale * 0.095;
            let right = middle + across * scale * 0.095;
            let ridge = middle + Vec3::Y * scale * 0.03;
            let tone = 0.92 + 0.09 * (turn * 1.3 + leaf as f32).sin();
            for p in [[center, left, ridge, center], [left, tip, ridge, left]] {
                face(mesh, p, [0.095 * tone, 0.235 * tone, 0.045 * tone, 1.]);
            }
            for p in [[tip, right, ridge, tip], [right, center, ridge, right]] {
                face(mesh, p, [0.15 * tone, 0.31 * tone, 0.055 * tone, 1.]);
            }
        }
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData, bounds: [f32; 4], base: f32, cell: [i32; 2]) {
    let [w, e, n, s] = bounds;
    let width = e - w;
    let depth = s - n;
    let scale = width.min(depth);
    let seed = (cell[0] as u32).wrapping_mul(0x9e3779b9)
        ^ (cell[1] as u32).wrapping_mul(0x85ebca6b);
    let phase = (seed % 997) as f32 / 997. * std::f32::consts::TAU;
    // Offset rosettes form one low, interlocking planting rather than five
    // identical upright bouquets. The central plant has a higher flower head.
    for plant in 0..7 {
        let angle = phase + plant as f32 * 2.399963;
        let radius = if plant == 0 { 0. } else { 0.255 };
        let center = Vec3::new((w + e) * 0.5 + angle.cos() * width * radius,
            base, (n + s) * 0.5 + angle.sin() * depth * radius);
        for leaf in 0..6 {
            let a = angle + leaf as f32 * std::f32::consts::TAU / 6.;
            let along = Vec3::new(a.cos(), 0., a.sin());
            let across = Vec3::new(-a.sin(), 0., a.cos());
            let root = center + Vec3::Y * scale * 0.025;
            let middle = root + along * scale * 0.13 + Vec3::Y * scale * 0.12;
            let tip = root + along * scale * 0.23 + Vec3::Y * scale * 0.08;
            let left = middle - across * scale * 0.055;
            let right = middle + across * scale * 0.055;
            let ridge = middle + Vec3::Y * scale * 0.025;
            for p in [[root, left, ridge, root], [left, tip, ridge, left]] {
                face(mesh, p, [0.10, 0.24, 0.085, 1.]);
            }
            for p in [[tip, right, ridge, tip], [right, root, ridge, right]] {
                face(mesh, p, [0.17, 0.32, 0.11, 1.]);
            }
        }
        // Heads reach the bed's low edging; keep basal leaves at the native
        // ground and remain below the established 0.65-tile flower envelope.
        let height = scale * (0.48 + 0.075 * (angle * 2.1).sin() +
            if plant == 0 { 0.04 } else { 0. });
        let head = center + Vec3::Y * height;
        let stem_side = Vec3::new(angle.cos(), 0., angle.sin()) * scale * 0.009;
        face(mesh, [center - stem_side, center + stem_side, head + stem_side,
            head - stem_side], [0.12, 0.26, 0.08, 1.]);
        let red = (seed.wrapping_add(plant * 17) % 5) < 2;
        let tint = if red { [0.62, 0.115, 0.145, 1.] }
            else { [0.77, 0.30, 0.40, 1.] };
        // A cupped five-petal head: raised outer lobes and a quiet warm center.
        for petal in 0..5 {
            let a = angle + petal as f32 * std::f32::consts::TAU / 5.;
            let direction = Vec3::new(a.cos(), 0., a.sin());
            let side = Vec3::new(-a.sin(), 0., a.cos());
            let middle = head + direction * scale * 0.078 + Vec3::Y * scale * 0.009;
            let rim = |segment: usize| {
                let arc = (segment % 8) as f32 * std::f32::consts::TAU / 8.;
                middle + direction * scale * 0.052 * arc.cos()
                    + side * scale * 0.048 * arc.sin()
                    + Vec3::Y * scale * (0.008 + 0.008 * arc.cos())
            };
            for segment in 0..8 {
                let tone = 0.98 + 0.035 * (segment as f32 * 0.78 + a).cos();
                face(mesh, [middle, rim(segment), rim(segment + 1), middle],
                    [tint[0] * tone, tint[1] * tone, tint[2] * tone, 1.]);
            }
        }
        let r = scale * 0.025;
        face(mesh, [head + Vec3::new(-r, 0.008 * scale, -r),
            head + Vec3::new(-r, 0.008 * scale, r),
            head + Vec3::new(r, 0.008 * scale, r),
            head + Vec3::new(r, 0.008 * scale, -r)], [0.70, 0.46, 0.12, 1.]);
    }
}

fn face(mesh: &mut SurfaceMeshData, mut vertices: [Vec3; 4], color: [f32; 4]) {
    let mut normal = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]).normalize();
    if normal.y < 0. {
        vertices.swap(0, 2);
        normal = -normal;
    }
    append_solid_quad(mesh, vertices.map(|v| v.to_array()), normal.to_array(), color);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bed_underplanting_stays_low_inside_its_cells_and_uses_world_phase() {
        let mut first = SurfaceMeshData::default();
        append_underplanting(&mut first, [0., 8., 0., 8.], [20, 40]);
        assert!(first.positions.iter().all(|p| p[0] > 0. && p[0] < 8.
            && p[2] > 0. && p[2] < 8. && p[1] > 0. && p[1] < 8. * 0.30));
        assert!(first.normals.iter().all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001));
        let mut shifted = SurfaceMeshData::default();
        append_underplanting(&mut shifted, [-16., -8., 24., 32.], [20, 40]);
        assert_eq!(first.indices, shifted.indices);
        assert_eq!(first.colors, shifted.colors);
        for (a, b) in first.positions.iter().zip(&shifted.positions) {
            assert!((a[0] - 16. - b[0]).abs() < 0.00001);
            assert_eq!(a[1], b[1]);
            assert!((a[2] + 24. - b[2]).abs() < 0.00001);
        }
        let mut neighbor = SurfaceMeshData::default();
        append_underplanting(&mut neighbor, [0., 8., 0., 8.], [21, 40]);
        assert_ne!(first.positions, neighbor.positions);
    }
    #[test]
    fn park_flower_clusters_stay_inside_native_cell_and_follow_map_phase() {
        let mut first = SurfaceMeshData::default();
        append(&mut first, [0., 8., 0., 8.], -2., [20, 40]);
        assert!(first.positions.iter().all(|p| p[0] > 0. && p[0] < 8.
            && p[2] > 0. && p[2] < 8. && p[1] >= -2. && p[1] < 3.2));
        assert!(first.normals.iter().all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001));
        let mut shifted = SurfaceMeshData::default();
        append(&mut shifted, [-16., -8., 24., 32.], -2., [20, 40]);
        assert_eq!(first.indices, shifted.indices);
        assert_eq!(first.colors, shifted.colors);
        for (a, b) in first.positions.iter().zip(&shifted.positions) {
            assert!((a[0] - 16. - b[0]).abs() < 0.00001);
            assert_eq!(a[1], b[1]);
            assert!((a[2] + 24. - b[2]).abs() < 0.00001);
        }
        let mut neighbor = SurfaceMeshData::default();
        append(&mut neighbor, [0., 8., 0., 8.], -2., [21, 40]);
        assert_ne!(first.positions, neighbor.positions);
        assert!(first.colors.iter().any(|c| c[0] > 0.58 && c[1] < 0.13));
        assert!(first.colors.iter().any(|c| c[0] > 0.72 && c[1] > 0.27 && c[2] > 0.37));
    }
}
