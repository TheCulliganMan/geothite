// Coordinated corridor finishes. World coordinates keep repeated objects stable
// across camera grids; rotations preserve the verified rectangular footprint.
fn furnishing_seed(world: [i32; 2]) -> u32 {
    let mut seed = (world[0] as u32).wrapping_mul(0x9e3779b9)
        ^ (world[1] as u32).wrapping_mul(0x85ebca6b);
    seed ^= seed >> 16;
    seed = seed.wrapping_mul(0x7feb352d);
    seed ^ (seed >> 15)
}

fn turn_counter_chair(mesh: &mut SurfaceMeshData, first: usize,
    center: [f32; 2], half: [f32; 2], direction: f32) {
    if direction == 0.0 { return; }
    // Quarter-turn in the fitted rectangle: left-side seats face east, the
    // merchant's seat faces west. Preserve the source claim and ground plane.
    let ratio = half[0] / half[1];
    for (p, n) in mesh.positions[first..].iter_mut().zip(&mut mesh.normals[first..]) {
        let x = p[0] - center[0]; let z = p[2] - center[1];
        p[0] = center[0] + direction * z * ratio;
        p[2] = center[1] - direction * x / ratio;
        *n = Vec3::new(direction * n[2] / ratio, n[1], -direction * n[0] * ratio)
            .normalize().to_array();
    }
}

fn vary_gate_furnishing(mesh: &mut SurfaceMeshData, first: usize, kind: ModelKind,
    world: [i32; 2], center: [f32; 2]) {
    let seed = furnishing_seed(world);
    let plant = matches!(kind, ModelKind::PottedPlant | ModelKind::LongPlanter);
    if !plant && kind != ModelKind::Chair { return; }
    if plant && seed & 1 != 0 {
        for (p, n) in mesh.positions[first..].iter_mut().zip(&mut mesh.normals[first..]) {
            p[0] = 2.0 * center[0] - p[0];
            p[2] = 2.0 * center[1] - p[2];
            n[0] = -n[0];
            n[2] = -n[2];
        }
    }
    for color in &mut mesh.colors[first..] {
        let tint = if plant && color[1] > color[0] * 1.08 && color[1] > color[2] * 1.08 {
            [[0.91, 0.98, 0.91], [1.0, 0.96, 0.86], [0.86, 0.94, 1.0]][(seed >> 1) as usize % 3]
        } else if !plant {
            [[1.0, 1.0, 1.0], [0.88, 0.91, 0.90], [0.95, 0.90, 0.83]][(seed >> 1) as usize % 3]
        } else { [1.0; 3] };
        for axis in 0..3 { color[axis] *= tint[axis]; }
    }
}

#[cfg(test)]
mod furnishing_variation_tests {
    use super::*;
    #[test]
    fn opposed_counter_seats_face_each_other_inside_source_bounds() {
        for direction in [-1.0, 1.0] {
            let mut mesh = SurfaceMeshData { positions: vec![[0., 1., 3.]],
                normals: vec![[0., 0., 1.]], ..Default::default() };
            turn_counter_chair(&mut mesh, 0, [0., 0.], [2., 3.], direction);
            assert_eq!(mesh.positions[0], [direction * 2., 1., 0.]);
            assert_eq!(mesh.normals[0], [direction, 0., 0.]);
        }
    }
    #[test]
    fn plant_variation_keeps_grounding_bounds_and_material_parts() {
        let original = SurfaceMeshData {
            positions: vec![[0., 0., 0.], [2., 3., 4.], [1., 2., 1.]],
            normals: vec![[1., 0., 0.]; 3],
            colors: vec![[0.5, 0.7, 0.4, 1.], [0.6, 0.4, 0.3, 1.], [0.5, 0.7, 0.4, 1.]],
            ..Default::default()
        };
        let mut variants = std::collections::HashSet::new();
        for x in 0..16 {
            let mut mesh = original.clone();
            vary_gate_furnishing(&mut mesh, 0, ModelKind::LongPlanter, [x, 8], [1., 2.]);
            assert_eq!(mesh.colors[1], original.colors[1]); // Pot material stays authored.
            assert_eq!(mesh.positions.iter().map(|p| p[1]).collect::<Vec<_>>(), vec![0., 3., 2.]);
            assert!(mesh.positions.iter().all(|p| (0.0..=2.0).contains(&p[0]) && (0.0..=4.0).contains(&p[2])));
            assert!(mesh.normals.iter().all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.001));
            let mut same = original.clone();
            vary_gate_furnishing(&mut same, 0, ModelKind::LongPlanter, [x, 8], [1., 2.]);
            assert_eq!(mesh.positions, same.positions);
            assert_eq!(mesh.colors, same.colors);
            variants.insert(mesh.colors[0].map(f32::to_bits));
        }
        assert_eq!(variants.len(), 3);
    }
}
