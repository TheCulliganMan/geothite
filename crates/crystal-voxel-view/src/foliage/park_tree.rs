//! Spreading garden trees, distinct from the taller irregular route groves.
use super::*;

pub(super) fn crown_shape(a: f32, b: f32, phase: f32) -> (f32, Vec3) {
    // Unequal boughs overlap around a smaller inner body. Deeper saddles
    // separate their silhouettes instead of stretching one balloon of leaves.
    let direction = Vec3::new(a.sin() * b.cos(), a.cos(), a.sin() * b.sin());
    let mut mass = 0.74;
    let mut gradient = Vec3::ZERO;
    for cluster in 0..7 {
        let turn = phase + cluster as f32 * 2.399963;
        let y = if cluster < 5 {
            0.08 + 0.18 * (phase + cluster as f32 * 1.7).sin()
        } else {
            0.68 + 0.07 * (phase * 1.7 + cluster as f32).sin()
        };
        let spread = (1. - y * y).sqrt();
        let axis = Vec3::new(spread * turn.cos(), y, spread * turn.sin());
        let dot = direction.dot(axis);
        let weight = 0.30 + 0.18 *
            (0.5 + 0.5 * (phase * 1.37 + cluster as f32 * 0.91).sin());
        let lobe = weight * ((dot - 1.) * 10.).exp();
        mass += lobe;
        gradient += (axis - direction * dot) * (10. * lobe);
    }
    (mass, gradient)
}

pub(crate) fn append_park_tree(
    mesh: &mut TerrainMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
) {
    let v = variation(coordinates[0], coordinates[1]);
    let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
    let center = Vec3::new(
        (bounds[0] + bounds[1]) * 0.5,
        base,
        (bounds[2] + bounds[3]) * 0.5,
    );
    let height = tile * (4.8 + v.height * 0.5);
    let width = spacing * (0.92 + v.width * 0.12);
    let phase = v.turn + v.height * 5.31;
    let radius = spacing * 0.11;
    let first = mesh.solid.positions.len();
    let start = first as u32;
    let trunk_point = |t: f32, angle: f32| {
        // Root flare settles quickly into a full stem. Tapering throughout
        // the exposed height made every mature tree read as a long cone.
        let flare = 1. + (1. - t).powi(8) * (0.8 + 0.3 * (angle * 5. + phase).cos());
        let r = radius * (1. - 0.22 * t) * flare
            * (1. + 0.055 * (angle * 7. + t).cos());
        let bend = Vec3::new(phase.cos(), 0., phase.sin()) * spacing * 0.055 * t * t;
        Vec3::new(angle.cos() * r, height * 0.52 * t, angle.sin() * r) + bend
    };
    // Continuous fluting and a rooted flare, with restrained warm bark.
    for row in 0..=8 {
        let t = row as f32 / 8.;
        for side in 0..=20 {
            let angle = std::f32::consts::TAU * (side % 20) as f32 / 20.;
            let p = center + trunk_point(t, angle);
            // Light the fluted, flared surface itself. A cylindrical normal
            // flattened the roots and hid the trunk's taper at every angle.
            let vertical = trunk_point(t + 0.001, angle) - trunk_point(t - 0.001, angle);
            let around = trunk_point(t, angle + 0.002) - trunk_point(t, angle - 0.002);
            let normal = vertical.cross(around).normalize();
            let shade = 0.68 + 0.32 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
            let grain = 0.94 + 0.06 * (angle * 7. + t * 1.2).cos();
            mesh.solid.positions.push(p.to_array());
            mesh.solid.normals.push(normal.to_array());
            mesh.solid.uvs.push([0.; 2]);
            mesh.solid.colors.push([
                0.30 * shade * grain,
                0.205 * shade * grain,
                0.105 * shade * grain,
                1.,
            ]);
        }
    }
    for row in 0..8 {
        for side in 0..20 {
            let a = start + row * 21 + side;
            let b = a + 21;
            mesh.solid.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    // Broad, tapering buttress roots settle the tall park stems into the
    // lawn. Keep their complete footprint inside the authored tree plot.
    for root in 0..6 {
        let angle = phase + root as f32 * std::f32::consts::TAU / 6.;
        let reach = spacing * (0.20 + 0.025 * (angle * 3.7).sin());
        append_root(&mut mesh.solid, center, angle, radius, reach, tile);
    }
    for branch in 0..5 {
        let angle = phase + branch as f32 * 2.399963;
        let level = 0.15 + branch as f32 * 0.04;
        let t = level / 0.52;
        let root = center + Vec3::Y * height * level
            + Vec3::new(phase.cos(), 0., phase.sin()) * spacing * 0.055 * t * t;
        let tip = center
            + Vec3::new(
                angle.cos() * width * 0.32,
                height * (0.63 + 0.02 * angle.sin()),
                angle.sin() * width * 0.32,
            );
        append_spreading_bough(&mut mesh.solid, root, tip, radius * 0.65);
        let fork = spreading_bough_center(root, tip, 0.68);
        for direction in [-1., 1.] {
            let turn = angle + direction * 0.35;
            let end = center
                + Vec3::new(
                    turn.cos() * width * 0.40,
                    height * 0.69,
                    turn.sin() * width * 0.40,
                );
            append_branch(&mut mesh.solid, fork, end, radius * 0.16);
        }
    }
    append_understory(&mut mesh.solid, center, spacing, tile, phase);
    let crown_first = mesh.solid.colors.len();
    // Separate overlapping boughs let light reach saddles between branches.
    // A single radial body, even with lobes, kept the entire tree looking
    // like a solid green cap. The lower boughs surround the authored forks;
    // an offset leader supplies the higher, asymmetric crown.
    for bough in 0..6 {
        let turn = phase + bough as f32 * 2.399963;
        let leader = bough == 5;
        let reach = width * if leader { 0.065 } else {
            0.25 + 0.025 * (turn * 1.7).sin()
        };
        let level = if leader { 0.71 } else {
            0.55 + 0.035 * (turn * 0.9 + phase).sin()
        };
        let crown = center + Vec3::new(turn.cos() * reach, height * level,
            turn.sin() * reach);
        let span = width * if leader { 0.33 } else {
            0.31 + 0.025 * (turn + phase).cos()
        };
        let brightness = if leader { 1.04 } else {
            0.91 + 0.07 * (turn * 1.3).sin()
        };
        town_crown(&mut mesh.solid, crown,
            Vec3::new(span, height * if leader { 0.16 } else { 0.20 }, span * 0.92),
            turn + v.height * 0.7,
            v.tint.map(|c| c * brightness), CrownStyle::Park);
    }
    // Park trees have a cooler mature-leaf palette than the route groves.
    // Keep sun/shade variation, but temper the uniformly yellow-green crown
    // seen against the light limestone and enrich its shaded blue-green body.
    for color in &mut mesh.solid.colors[crown_first..] {
        color[0] *= 0.85;
        color[1] *= 0.88;
        color[2] *= 1.20;
    }
    mesh.solid
        .cutaway_ranges
        .push(first..mesh.solid.positions.len());
}

fn append_understory(mesh: &mut SurfaceMeshData, center: Vec3, spacing: f32, tile: f32, phase: f32) {
    // Low rosettes sit beside the buttress roots, inside the original tree
    // plot. They add planted depth without spreading over the promenade.
    for plant in 0..12 {
        let angle = phase + plant as f32 * 2.399963;
        let radius = spacing * (0.16 + (plant % 3) as f32 * 0.05
            + 0.025 * (angle * 2.3).sin());
        let root = center + Vec3::new(angle.cos() * radius, tile * 0.0015,
            angle.sin() * radius);
        for leaf in 0..6 {
            let a = angle + leaf as f32 * std::f32::consts::TAU / 6.;
            let along = Vec3::new(a.cos(), 0., a.sin());
            let across = Vec3::new(-a.sin(), 0., a.cos());
            let length = tile * (0.40 + 0.045 * (a * 1.7 + phase).sin());
            let middle = root + along * length * 0.55 + Vec3::Y * tile * 0.30;
            let tip = root + along * length + Vec3::Y * tile * 0.18;
            let left = middle - across * tile * 0.090;
            let right = middle + across * tile * 0.090;
            let ridge = middle + Vec3::Y * tile * 0.045;
            for mut p in [[root, left, ridge], [left, tip, ridge],
                [tip, right, ridge], [right, root, ridge]] {
                let mut normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
                if normal.y < 0. { p.swap(1, 2); normal = -normal; }
                let shade = 0.72 + 0.28 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
                let first = mesh.positions.len() as u32;
                mesh.positions.extend(p.map(|p| p.to_array()));
                mesh.normals.extend([normal.to_array(); 3]);
                mesh.uvs.extend([[0.; 2]; 3]);
                mesh.colors.extend([[0.12 * shade, 0.25 * shade, 0.075 * shade, 1.]; 3]);
                mesh.indices.extend([first, first + 1, first + 2]);
            }
        }
    }
}

fn spreading_bough_center(root: Vec3, tip: Vec3, t: f32) -> Vec3 {
    let horizontal = Vec3::new(tip.x - root.x, 0., tip.z - root.z);
    let control = root + horizontal * 1.05 + Vec3::Y * (tip.y - root.y) * 0.12;
    root.lerp(control, t).lerp(control.lerp(tip, t), t)
}

fn append_spreading_bough(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, radius: f32) {
    let horizontal = Vec3::new(tip.x - root.x, 0., tip.z - root.z);
    let side = horizontal.normalize().cross(Vec3::Y);
    // Mature garden limbs spread below the crown before turning upward.
    // An upward bow hid the entire fork inside its leaf mass.
    let control = root + horizontal * 1.05 + Vec3::Y * (tip.y - root.y) * 0.12;
    let point = |t: f32, angle: f32| {
        let center = spreading_bough_center(root, tip, t);
        let tangent = ((control - root) * (1. - t) + (tip - control) * t).normalize();
        let radial = side * angle.cos() + side.cross(tangent) * angle.sin();
        center + radial * radius * (1. - 0.70 * t)
    };
    let first = mesh.positions.len() as u32;
    for row in 0..=8 {
        let t = row as f32 / 8.;
        for ring in 0..=10 {
            let angle = std::f32::consts::TAU * (ring % 10) as f32 / 10.;
            let p = point(t, angle);
            let along = point(t + 0.001, angle) - point(t - 0.001, angle);
            let around = point(t, angle + 0.001) - point(t, angle - 0.001);
            let normal = along.cross(around).normalize();
            let light = 0.68 + 0.32 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
            mesh.positions.push(p.to_array());
            mesh.normals.push(normal.to_array());
            mesh.uvs.push([0.; 2]);
            mesh.colors.push([0.30 * light, 0.205 * light, 0.105 * light, 1.]);
        }
    }
    for row in 0..8 { for ring in 0..10 {
        let a = first + row * 11 + ring;
        let b = a + 11;
        mesh.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
    }}
}

fn append_root(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    angle: f32,
    trunk_radius: f32,
    reach: f32,
    tile: f32,
) {
    let along = Vec3::new(angle.cos(), 0., angle.sin());
    let across = Vec3::new(-angle.sin(), 0., angle.cos());
    let ring = |step: usize| {
        let t = step as f32 / 5.;
        let distance = trunk_radius * 0.60 * (1. - t) + reach * t;
        let bend = (t * std::f32::consts::PI).sin() * tile * 0.12;
        let c = center + along * distance + across * bend;
        let width = trunk_radius * 0.46 * (1. - t).powf(1.3) + tile * 0.045;
        let crest = tile * (0.82 * (1. - t).powi(2) + 0.025);
        [c - across * width, c + Vec3::Y * crest, c + across * width]
    };
    let face = |mesh: &mut SurfaceMeshData, p: [Vec3; 4]| {
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        let light = 0.72 + 0.28 * n.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
        let start = mesh.positions.len() as u32;
        mesh.positions.extend(p.map(|v| v.to_array()));
        mesh.normals.extend([n.to_array(); 4]);
        mesh.uvs.extend([[0.; 2]; 4]);
        mesh.colors.extend([[0.30 * light, 0.205 * light, 0.105 * light, 1.]; 4]);
        mesh.indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
    };
    for step in 0..5 {
        let a = ring(step);
        let b = ring(step + 1);
        face(mesh, [a[0], a[1], b[1], b[0]]);
        face(mesh, [a[1], a[2], b[2], b[1]]);
        face(mesh, [a[2], a[0], b[0], b[2]]);
    }
    let a = ring(0);
    let b = ring(5);
    face(mesh, [a[2], a[1], a[0], a[0]]);
    face(mesh, [b[0], b[1], b[2], b[2]]);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spreading_boughs_keep_outward_faces_and_closed_ring_seams() {
        for angle in [0.0_f32, 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            append_spreading_bough(&mut mesh, Vec3::new(0., 6., 0.),
                Vec3::new(angle.cos() * 11., 26., angle.sin() * 11.), 2.3);
            for row in 0..=8 {
                assert_eq!(mesh.positions[row * 11], mesh.positions[row * 11 + 10]);
                assert_eq!(mesh.normals[row * 11], mesh.normals[row * 11 + 10]);
            }
            for ids in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [ids[0], ids[1], ids[2]];
                let face = (Vec3::from_array(mesh.positions[b as usize]) - Vec3::from_array(mesh.positions[a as usize]))
                    .cross(Vec3::from_array(mesh.positions[c as usize]) - Vec3::from_array(mesh.positions[a as usize]));
                let normal = ids.iter().map(|&i| Vec3::from_array(mesh.normals[i as usize])).sum::<Vec3>();
                assert!(face.length_squared() > 1e-6 && face.dot(normal) > 0.);
            }
        }
    }
    #[test]
    fn park_cluster_leaves_keep_outward_faces() {
        for phase in [0., 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            town_crown(&mut mesh, Vec3::ZERO, Vec3::new(17., 15., 16.),
                phase, [1.; 3], CrownStyle::Park);
            for ids in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [ids[0], ids[1], ids[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let normal = ids.iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum::<Vec3>();
                let face = (b - a).cross(c - a);
                assert!(face.length_squared() > 1e-10);
                assert!(face.dot(normal) > 0.);
            }
        }
    }
    #[test]
    fn park_boughs_have_real_light_gaps_without_losing_the_leaf_mass() {
        let sun = Vec3::new(0.45, 0.80, 0.55).normalize();
        let side = sun.cross(Vec3::Y).normalize();
        let up = side.cross(sun);
        for phase in [0., 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            town_crown(&mut mesh, Vec3::ZERO, Vec3::new(17., 15., 16.),
                phase, [1.; 3], CrownStyle::Park);
            let mut hits = 0;
            for row in 0..16 { for column in 0..16 {
                let origin = sun * 60. + side * (column as f32 - 7.5) * 0.75
                    + up * (row as f32 - 7.5) * 0.75;
                let blocked = mesh.indices.chunks_exact(3).any(|ids| {
                    let [a, b, c] = [ids[0], ids[1], ids[2]]
                        .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                    let e1 = b - a;
                    let e2 = c - a;
                    let p = (-sun).cross(e2);
                    let det = e1.dot(p);
                    if det.abs() < 1e-6 { return false; }
                    let offset = origin - a;
                    let u = offset.dot(p) / det;
                    let q = offset.cross(e1);
                    let v = (-sun).dot(q) / det;
                    u >= 0. && v >= 0. && u + v <= 1. && e2.dot(q) / det > 0.
                });
                hits += usize::from(blocked);
            }}
            assert!(hits > 128, "the crown must retain a substantial leaf mass");
            assert!(hits < 256, "real light must pass between leaves, not hit a closed core");
        }
    }
    #[test]
    fn park_cluster_normals_follow_boughs_and_keep_the_closed_seam() {
        let radius = Vec3::new(17., 15., 16.);
        for phase in [0., 1.3, 3.8] {
            let point = |a: f32, b: f32| {
                let (lobe, _) = crown_shape(a, b, phase);
                Vec3::new(a.sin() * b.cos(), a.cos(), a.sin() * b.sin()) * radius * lobe
            };
            for row in 1..12 {
                let a = std::f32::consts::PI * row as f32 / 12.;
                for side in 0..24 {
                    let b = std::f32::consts::TAU * side as f32 / 24.;
                    let (lobe, gradient) = crown_shape(a, b, phase);
                    let direction = Vec3::new(a.sin() * b.cos(), a.cos(), a.sin() * b.sin());
                    let normal = crown_normal(direction, radius, lobe, gradient);
                    let da = point(a + 0.0005, b) - point(a - 0.0005, b);
                    let db = point(a, b + 0.0005) - point(a, b - 0.0005);
                    assert!(normal.dot(db.cross(da).normalize()) > 0.999);
                }
                assert!((point(a, 0.) - point(a, std::f32::consts::TAU)).length() < 0.0001);
            }
        }
    }
    #[test]
    fn park_root_planting_stays_low_inside_the_tree_plot() {
        for phase in [0., 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            append_understory(&mut mesh, Vec3::new(16., 3., 16.), 32., 8., phase);
            assert!(mesh.positions.iter().all(|p| p[0] > 0. && p[0] < 32.
                && p[2] > 0. && p[2] < 32. && p[1] >= 3. && p[1] < 3. + 8. * 0.40));
            for ids in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [ids[0], ids[1], ids[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let normal = Vec3::from_array(mesh.normals[ids[0] as usize]);
                let face = (b - a).cross(c - a);
                assert!(face.length_squared() > 0.000001);
                assert!(face.normalize().dot(normal) > 0.999);
                assert!((normal.length() - 1.).abs() < 0.00001);
            }
        }
    }

    #[test]
    fn park_oval_leaves_keep_outward_winding_through_curvature() {
        for direction in [Vec3::X, Vec3::Y, Vec3::NEG_Z,
            Vec3::new(0.4, 0.7, -0.3).normalize()] {
            let u = direction.cross(if direction.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y })
                .normalize();
            let v = direction.cross(u);
            for size in [0.25, 1.4] {
                let (p, normals) = park_leaf(Vec3::ZERO, u, v, direction, size);
                assert!(normals.iter().all(|n| n.is_finite()
                    && (n.length() - 1.).abs() < 0.00001));
                for edge in 0..8 {
                    let a = edge + 1;
                    let b = (edge + 1) % 8 + 1;
                    let face = (p[a] - p[0]).cross(p[b] - p[0]);
                    assert!(face.length_squared() > 0.0000000001);
                    assert!(face.dot(normals[0] + normals[a] + normals[b]) > 0.);
                }
            }
        }
    }

    #[test]
    fn park_trunk_flare_normals_and_duplicate_seam_follow_the_sculpted_surface() {
        let mut mesh = TerrainMeshData::default();
        append_park_tree(&mut mesh, [-16., 16., -16., 16.], 3., 8., [4, 8]);
        for row in 0..=8 {
            let start = row * 21;
            assert_eq!(mesh.solid.positions[start], mesh.solid.positions[start + 20]);
            assert_eq!(mesh.solid.normals[start], mesh.solid.normals[start + 20]);
        }
        let base = &mesh.solid.normals[..20];
        let upper = &mesh.solid.normals[8 * 21..8 * 21 + 20];
        assert!(base.iter().all(|n| n[1] > 0.20), "root flare needs upward-facing normals");
        assert!(upper.iter().any(|n| n[1] < 0.05), "upper trunk lighting must include its lean");
    }
    #[test]
    fn park_buttress_roots_are_closed_above_the_authored_floor() {
        for angle in [0., 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            append_root(&mut mesh, Vec3::new(0., 3., 0.), angle, 2.72, 7.2, 8.);
            assert_eq!(mesh.positions.len(), 17 * 4);
            assert!(mesh.positions.iter().all(|p| p[1] >= 3.
                && p[0].abs() < 8. && p[2].abs() < 8.));
            assert!(mesh.positions.iter().any(|p| p[1] == 3.));
            for (p, n) in mesh.positions.chunks_exact(4).zip(mesh.normals.chunks_exact(4)) {
                let [a, b, c] = [p[0], p[1], p[2]].map(Vec3::from_array);
                let cross = (b-a).cross(c-a);
                assert!(cross.length_squared() > 0.000001);
                assert!(cross.normalize().dot(Vec3::from_array(n[0])) > 0.999);
            }
        }
    }
    #[test]
    fn park_trees_are_rooted_varied_and_keep_one_cutaway_owner() {
        let mut previous = None;
        for coordinates in [[0, 0], [4, 8], [-12, 20]] {
            let mut mesh = TerrainMeshData::default();
            append_park_tree(&mut mesh, [-16., 16., -16., 16.], 3., 8., coordinates);
            let s = &mesh.solid;
            assert_eq!(s.cutaway_ranges, vec![0..s.positions.len()]);
            assert!(
                s.positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite()) && p[1] >= 3.)
            );
            assert!(s.positions.iter().any(|p| p[1] == 3.));
            assert!(
                s.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
            );
            assert!(s.indices.iter().all(|&i| (i as usize) < s.positions.len()));
            if let Some(p) = previous {
                assert_ne!(s.positions, p);
            }
            previous = Some(s.positions.clone());
        }
    }
}
