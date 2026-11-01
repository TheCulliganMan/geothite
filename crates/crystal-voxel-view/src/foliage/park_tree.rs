//! Mature spreading park trees, distinct from the irregular route groves.
use super::*;

const TRUNK_ROWS: usize = 12;
const TRUNK_SIDES: usize = 32;

pub(super) fn dimensions(spacing: f32, tile: f32, coordinates: [i32; 2]) -> (f32, f32) {
    let v = variation(coordinates[0], coordinates[1]);
    let maturity = (v.height - 0.90) / 0.70;
    (tile * (10.5 + maturity * 4.0) * [0.96, 1.04, 1.0][v.index],
        spacing * (2.0 + maturity * 0.40) * (0.94 + v.width * 0.06)
            * [1.04, 0.94, 1.0][v.index])
}

pub(super) fn phase(coordinates: [i32; 2]) -> f32 {
    let v = variation(coordinates[0], coordinates[1]);
    v.turn + v.height * 5.31
}

fn bark_color(normal: Vec3, angle: f32, along: f32, phase: f32) -> [f32; 4] {
    // Long, slightly wandering fissures follow each stem instead of painting
    // a uniform ochre cylinder. The actual fluting supplies the broad relief.
    let grain = angle * 9. + along * 1.8 + 0.35 * (along * 7. + phase).sin();
    let groove = (0.5 + 0.5 * grain.cos()).powi(6);
    let ridge = 0.5 + 0.5 * (angle * 5. - along + phase).sin();
    let light = 0.82 + 0.18 * normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.);
    let shade = light * (1. - groove * 0.25);
    [
        (0.205 + ridge * 0.045) * shade,
        (0.123 + ridge * 0.032) * shade,
        (0.059 + ridge * 0.014) * shade,
        1.,
    ]
}

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
    // The four-cell source drawing owns the planting plot, not the full
    // mature crown. Fitting the entire tree to that plot left the park with
    // waist-high forks and crowns barely taller than its visitors.
    let maturity = (v.height - 0.90) / 0.70;
    let (height, width) = dimensions(spacing, tile, coordinates);
    let phase = phase(coordinates);
    let radius = spacing * (0.12 + maturity * 0.018);
    let first = mesh.solid.positions.len();
    let start = first as u32;
    // Continue the tapering leader into the upper foliage. Ending the stem
    // below that crown exposed a flat, sawn-off cap in reverse low views.
    let stem_height = height * 0.82;
    let stem_center = |t: f32| {
        Vec3::Y * stem_height * t
            + Vec3::new(phase.cos(), 0., phase.sin()) * spacing * 0.055 * t * t
    };
    let trunk_point = |t: f32, angle: f32| {
        // The broad rooted flare settles quickly; the stem then narrows
        // gradually into the spreading forks inside the crown.
        let flare = 1. + (1. - t).powi(9) * (0.9 + 0.3 * (angle * 5. + phase).cos());
        let taper = 1. - 0.36 * t - 0.34 * t * t;
        let r = radius * taper * flare * (1. + 0.075 * (angle * 7. + t * 1.5 + phase).cos());
        stem_center(t) + Vec3::new(angle.cos() * r, 0., angle.sin() * r)
    };
    // Continuous fluting and a rooted flare, with restrained warm bark.
    for row in 0..=TRUNK_ROWS {
        let t = row as f32 / TRUNK_ROWS as f32;
        for side in 0..=TRUNK_SIDES {
            let angle = std::f32::consts::TAU * (side % TRUNK_SIDES) as f32 / TRUNK_SIDES as f32;
            let p = center + trunk_point(t, angle);
            // Light the fluted, flared surface itself. A cylindrical normal
            // flattened the roots and hid the trunk's taper at every angle.
            let vertical = trunk_point(t + 0.001, angle) - trunk_point(t - 0.001, angle);
            let around = trunk_point(t, angle + 0.002) - trunk_point(t, angle - 0.002);
            let normal = vertical.cross(around).normalize();
            mesh.solid.positions.push(p.to_array());
            mesh.solid.normals.push(normal.to_array());
            mesh.solid.uvs.push([0.; 2]);
            mesh.solid.colors.push(bark_color(normal, angle, t, phase));
        }
    }
    for row in 0..TRUNK_ROWS {
        for side in 0..TRUNK_SIDES {
            let a = start + (row * (TRUNK_SIDES + 1) + side) as u32;
            let b = a + (TRUNK_SIDES + 1) as u32;
            mesh.solid.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let top_axis = (stem_center(1.001) - stem_center(0.999)).normalize();
    append_bark_cap(
        &mut mesh.solid,
        first..first + TRUNK_SIDES,
        center,
        Vec3::NEG_Y,
        0.,
        phase,
    );
    let top_ring = first + TRUNK_ROWS * (TRUNK_SIDES + 1);
    append_bark_cap(
        &mut mesh.solid,
        top_ring..top_ring + TRUNK_SIDES,
        center + stem_center(1.) + top_axis * radius * 0.08,
        top_axis,
        1.,
        phase,
    );
    // Broad, tapering buttress roots settle the tall park stems into the
    // lawn. Keep their complete footprint inside the authored tree plot.
    for root in 0..6 {
        let angle = phase + root as f32 * std::f32::consts::TAU / 6.;
        let reach = spacing * (0.20 + 0.025 * (angle * 3.7).sin());
        append_root(&mut mesh.solid, center, angle, radius, reach, tile);
    }
    append_understory(&mut mesh.solid, center, spacing, tile, phase);
    // Each spreading limb owns its leaf mass. Aligning both from the same
    // bough removes the repeated bare forks poking through unrelated crowns.
    // Lower, broad lobes and a modest offset leader make a mature umbrella.
    let boughs = [6, 7, 5][v.index];
    for bough in 0..boughs {
        let turn = phase + bough as f32 * 2.399963;
        let leader = bough == boughs - 1;
        let reach = width
            * if leader {
                0.065
            } else {
                0.26 + 0.030 * (turn * 1.7).sin()
            };
        let level = if leader {
            0.82
        } else {
            0.65 + 0.030 * (turn * 0.9 + phase).sin()
        };
        let crown = center + Vec3::new(turn.cos() * reach, height * level, turn.sin() * reach);
        let span = width
            * if leader {
                0.31
            } else {
                0.31 + 0.025 * (turn + phase).cos()
            };
        if !leader {
            let fork_level = 0.43 + 0.11 * (0.5 + 0.5 * (turn * 1.4 + phase).sin());
            let root = center + stem_center(height * fork_level / stem_height);
            let tip = crown
                + Vec3::new(
                    turn.cos() * span * 0.18,
                    height * 0.025,
                    turn.sin() * span * 0.18,
                );
            let limb_radius = radius * (0.46 + 0.10 * (turn + phase).sin());
            append_spreading_bough(&mut mesh.solid, root, tip, limb_radius, turn);
            let fork = spreading_bough_center(root, tip, 0.66);
            for side in [-1., 1.] {
                let angle = turn + side * (0.52 + 0.08 * turn.sin());
                let end = crown
                    + Vec3::new(
                        angle.cos() * span * 0.56,
                        height * (0.045 + 0.015 * side * turn.cos()),
                        angle.sin() * span * 0.56,
                    );
                append_spreading_bough(&mut mesh.solid, fork, end, limb_radius * 0.28, turn + side);
            }
        }
        let brightness = if leader {
            1.04
        } else {
            0.91 + 0.07 * (turn * 1.3).sin()
        };
        let crown_first = mesh.solid.colors.len();
        town_crown(
            &mut mesh.solid,
            crown,
            Vec3::new(span, height * if leader { 0.15 } else { 0.16 }, span * 0.92),
            turn + v.height * 0.7,
            v.tint.map(|c| c * brightness),
        );
        // Shade retains a readable blue-green body beneath olive new growth.
        for color in &mut mesh.solid.colors[crown_first..] {
            color[0] *= 0.85;
            color[1] *= 0.88;
            color[2] *= 1.20;
        }
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

fn spreading_bough_control(root: Vec3, tip: Vec3) -> Vec3 {
    let horizontal = Vec3::new(tip.x - root.x, 0., tip.z - root.z);
    root + horizontal * 0.70 + Vec3::Y * (tip.y - root.y) * 0.36
}

fn spreading_bough_center(root: Vec3, tip: Vec3, t: f32) -> Vec3 {
    let control = spreading_bough_control(root, tip);
    root.lerp(control, t).lerp(control.lerp(tip, t), t)
}

fn append_spreading_bough(
    mesh: &mut SurfaceMeshData,
    root: Vec3,
    tip: Vec3,
    radius: f32,
    phase: f32,
) {
    let horizontal = Vec3::new(tip.x - root.x, 0., tip.z - root.z);
    let side = horizontal.normalize().cross(Vec3::Y);
    // Limbs rise into their own foliage with a visible fork near the trunk.
    // Flat, oversized elbows formed a repeated bare rack below every crown.
    let control = spreading_bough_control(root, tip);
    let point = |t: f32, angle: f32| {
        let center = spreading_bough_center(root, tip, t);
        let tangent = ((control - root) * (1. - t) + (tip - control) * t).normalize();
        let radial = side * angle.cos() + side.cross(tangent) * angle.sin();
        // Finish as a fine shoot inside the leaves. A quarter-width end
        // remained visibly sawn off wherever a low view exposed the join.
        center
            + radial * radius * (1. - 0.98 * t) * (1. + 0.045 * (angle * 5. + t * 2. + phase).cos())
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
            mesh.positions.push(p.to_array());
            mesh.normals.push(normal.to_array());
            mesh.uvs.push([0.; 2]);
            mesh.colors.push(bark_color(normal, angle, t, phase));
        }
    }
    for row in 0..8 {
        for ring in 0..10 {
            let a = first + row * 11 + ring;
            let b = a + 11;
            mesh.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let root_axis = (control - root).normalize();
    let tip_axis = (tip - control).normalize();
    let root_ring = first as usize;
    let tip_ring = root_ring + 8 * 11;
    append_bark_cap(mesh, root_ring..root_ring + 10, root, -root_axis, 0., phase);
    append_bark_cap(
        mesh,
        tip_ring..tip_ring + 10,
        tip + tip_axis * radius * 0.08,
        tip_axis,
        1.,
        phase,
    );
}

fn append_bark_cap(
    mesh: &mut SurfaceMeshData,
    ring: std::ops::Range<usize>,
    center: Vec3,
    outward: Vec3,
    along: f32,
    phase: f32,
) {
    // The floor end is flat and the tip is gently raised. Give the cap its
    // own normals so closing a flared root never changes its side lighting.
    let mut points = vec![center];
    points.extend(mesh.positions[ring].iter().copied().map(Vec3::from_array));
    let sides = points.len() - 1;
    let mut normals = vec![Vec3::ZERO; points.len()];
    let mut faces = Vec::with_capacity(sides);
    for side in 0..sides {
        let mut face = [0, side + 1, (side + 1) % sides + 1];
        let mut normal = (points[face[1]] - center).cross(points[face[2]] - center);
        if normal.dot(outward) < 0. {
            face.swap(1, 2);
            normal = -normal;
        }
        for i in face {
            normals[i] += normal;
        }
        faces.push(face);
    }
    let first = mesh.positions.len() as u32;
    for (i, (point, normal)) in points.into_iter().zip(normals).enumerate() {
        let normal = normal.normalize();
        mesh.positions.push(point.to_array());
        mesh.normals.push(normal.to_array());
        mesh.uvs.push([0.; 2]);
        mesh.colors.push(bark_color(
            normal,
            std::f32::consts::TAU * i.saturating_sub(1) as f32 / sides as f32,
            along,
            phase,
        ));
    }
    for face in faces {
        mesh.indices.extend(face.map(|i| first + i as u32));
    }
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
        let start = mesh.positions.len() as u32;
        mesh.positions.extend(p.map(|v| v.to_array()));
        mesh.normals.extend([n.to_array(); 4]);
        mesh.uvs.extend([[0.; 2]; 4]);
        mesh.colors.extend([bark_color(n, angle, 0., angle); 4]);
        mesh.indices
            .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
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

    fn assert_closed_component(mesh: &SurfaceMeshData, range: std::ops::Range<usize>) {
        let mut points = std::collections::HashMap::new();
        let mut edges = std::collections::HashMap::<(usize, usize), (usize, i32)>::new();
        let origin = Vec3::from_array(mesh.positions[range.start]).as_dvec3();
        let mut volume = 0.0_f64;
        let mut faces = 0;
        for face in mesh.indices.chunks_exact(3) {
            if !face.iter().all(|i| range.contains(&(*i as usize))) {
                continue;
            }
            faces += 1;
            let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let [a, b, c] = p.map(|p| p.as_dvec3() - origin);
            volume += a.dot(b.cross(c));
            let ids = p.map(|p| {
                let bits = p.to_array().map(|v| if v == 0. { 0 } else { v.to_bits() });
                let next = points.len();
                *points.entry(bits).or_insert(next)
            });
            for (a, b) in [(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0])] {
                assert_ne!(a, b, "a closed surface cannot contain a collapsed edge");
                let key = if a < b { (a, b) } else { (b, a) };
                let entry = edges.entry(key).or_default();
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            faces > 0 && volume > 0.,
            "a closed surface must enclose positive volume"
        );
        assert!(
            edges
                .values()
                .all(|&(count, direction)| count == 2 && direction == 0),
            "a surface has an open edge or inconsistent winding"
        );
    }

    #[test]
    fn spreading_boughs_keep_outward_faces_and_closed_ring_seams() {
        for angle in [0.0_f32, 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            append_spreading_bough(
                &mut mesh,
                Vec3::new(0., 6., 0.),
                Vec3::new(angle.cos() * 11., 26., angle.sin() * 11.),
                2.3,
                angle,
            );
            for row in 0..=8 {
                assert_eq!(mesh.positions[row * 11], mesh.positions[row * 11 + 10]);
                assert_eq!(mesh.normals[row * 11], mesh.normals[row * 11 + 10]);
            }
            for ids in mesh.indices.chunks_exact(3) {
                let [a, b, c] = [ids[0], ids[1], ids[2]];
                let face = (Vec3::from_array(mesh.positions[b as usize])
                    - Vec3::from_array(mesh.positions[a as usize]))
                .cross(
                    Vec3::from_array(mesh.positions[c as usize])
                        - Vec3::from_array(mesh.positions[a as usize]),
                );
                let normal = ids
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum::<Vec3>();
                assert!(face.length_squared() > 1e-6 && face.dot(normal) > 0.);
            }
            assert_closed_component(&mesh, 0..mesh.positions.len());
        }
    }
    #[test]
    fn park_cluster_leaves_keep_outward_faces() {
        for phase in [0., 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            town_crown(&mut mesh, Vec3::ZERO, Vec3::new(17., 15., 16.),
                phase, [1.; 3]);
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
    fn park_boughs_keep_dense_interiors_and_broken_leaf_edges() {
        let sun = Vec3::new(0.45, 0.80, 0.55).normalize();
        for phase in [0., 1.3, 3.8] {
            let mut mesh = SurfaceMeshData::default();
            town_crown(
                &mut mesh,
                Vec3::ZERO,
                Vec3::new(17., 15., 16.),
                phase,
                [1.; 3],
            );
            for view in [
                sun,
                Vec3::X,
                Vec3::Z,
                Vec3::NEG_Y,
                Vec3::new(0.4, -0.9, 0.1).normalize(),
            ] {
                let side = view
                    .cross(if view.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y })
                    .normalize();
                let up = side.cross(view);
                let coverage = |spacing: f32| {
                    let mut hits = 0;
                    for row in 0..16 {
                        for column in 0..16 {
                            let origin = view * 60.
                                + side * (column as f32 - 7.5) * spacing
                                + up * (row as f32 - 7.5) * spacing;
                            let blocked = mesh.indices.chunks_exact(3).any(|ids| {
                                let [a, b, c] = [ids[0], ids[1], ids[2]]
                                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                                let e1 = b - a;
                                let e2 = c - a;
                                let p = (-view).cross(e2);
                                let det = e1.dot(p);
                                // Match the production material's back-face
                                // culling; hidden far-side leaves cannot fill
                                // a hole in the visible canopy.
                                if det <= 1e-6 {
                                    return false;
                                }
                                let offset = origin - a;
                                let u = offset.dot(p) / det;
                                let q = offset.cross(e1);
                                let v = (-view).dot(q) / det;
                                u >= 0. && v >= 0. && u + v <= 1. && e2.dot(q) / det > 0.
                            });
                            hits += usize::from(blocked);
                        }
                    }
                    hits
                };
                // Dense overlapping leaves should block the interior. Sample the
                // complete bough footprint separately for its broken outer edge,
                // rather than demanding pinholes through the middle of a crown.
                let inner = coverage(0.75);
                assert!(
                    inner > 224,
                    "the crown interior is too porous from {view:?}: {inner}/256"
                );
                let edge = coverage(1.8);
                assert!(
                    edge > 128 && edge < 256,
                    "the bough needs substantial foliage and open leaf edges from {view:?}: {edge}/256"
                );
            }
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
        for direction in [
            Vec3::X,
            Vec3::Y,
            Vec3::NEG_Z,
            Vec3::new(0.4, 0.7, -0.3).normalize(),
        ] {
            let u = direction
                .cross(if direction.y.abs() > 0.9 {
                    Vec3::Z
                } else {
                    Vec3::Y
                })
                .normalize();
            let v = direction.cross(u);
            for size in [0.25, 1.4] {
                let (p, normals) = park_leaf(Vec3::ZERO, u, v, direction, size);
                assert!(
                    normals
                        .iter()
                        .all(|n| n.is_finite() && (n.length() - 1.).abs() < 0.00001)
                );
                let mut leaf = SurfaceMeshData::default();
                leaf.positions.extend(p.map(|p| p.to_array()));
                for [a, b, c] in PARK_LEAF_FACES {
                    let face = (p[b] - p[a]).cross(p[c] - p[a]);
                    assert!(face.length_squared() > 0.0000000001);
                    assert!(face.dot(normals[a] + normals[b] + normals[c]) > 0.);
                    leaf.indices.extend([a, b, c].map(|i| i as u32));
                }
                assert_closed_component(&leaf, 0..leaf.positions.len());
                assert!(
                    normals[0].dot(direction) > 0.99 && normals[PARK_LEAF_BACK].dot(direction) < -0.99,
                    "the leaf needs distinct upper and lower shading"
                );
                for edge in 1..=PARK_LEAF_EDGES {
                    assert_eq!(p[edge], p[PARK_LEAF_BACK + edge]);
                    assert!(normals[edge].dot(direction) > 0.95);
                    assert!(
                        normals[PARK_LEAF_BACK + edge].dot(direction) < -0.95,
                        "averaging both sides at the rim makes a thin leaf look inflated"
                    );
                }
                for view in [
                    direction,
                    -direction,
                    u,
                    -u,
                    v,
                    -v,
                    (direction + u).normalize(),
                    (v - direction).normalize(),
                ] {
                    // These rays deliberately meet shared fan edges. Evaluate
                    // the actual f32 vertices in f64 so intersection roundoff
                    // cannot reject both of two adjoining front faces.
                    let origin = (view * size * 4.).as_dvec3();
                    let ray = (-view).as_dvec3();
                    let hit = PARK_LEAF_FACES.iter().any(|&[a, b, c]| {
                        let [a, b, c] = [a, b, c].map(|i| p[i].as_dvec3());
                        let e1 = b - a;
                        let e2 = c - a;
                        let cross = ray.cross(e2);
                        let det = e1.dot(cross);
                        if det <= 1e-8 {
                            return false;
                        }
                        let offset = origin - a;
                        let u = offset.dot(cross) / det;
                        let cross = offset.cross(e1);
                        let v = ray.dot(cross) / det;
                        u >= 0. && v >= 0. && u + v <= 1. && e2.dot(cross) / det > 0.
                    });
                    assert!(
                        hit,
                        "the leaf disappeared with back-face culling from {view:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn park_trunk_flare_normals_and_duplicate_seam_follow_the_sculpted_surface() {
        let mut mesh = TerrainMeshData::default();
        append_park_tree(&mut mesh, [-16., 16., -16., 16.], 3., 8., [4, 8]);
        for row in 0..=TRUNK_ROWS {
            let start = row * (TRUNK_SIDES + 1);
            assert_eq!(
                mesh.solid.positions[start],
                mesh.solid.positions[start + TRUNK_SIDES]
            );
            assert_eq!(
                mesh.solid.normals[start],
                mesh.solid.normals[start + TRUNK_SIDES]
            );
        }
        let base = &mesh.solid.normals[..TRUNK_SIDES];
        let upper = &mesh.solid.normals
            [TRUNK_ROWS * (TRUNK_SIDES + 1)..TRUNK_ROWS * (TRUNK_SIDES + 1) + TRUNK_SIDES];
        assert!(
            base.iter().all(|n| n[1] > 0.20),
            "root flare needs upward-facing normals"
        );
        assert!(
            upper.iter().any(|n| n[1] < 0.05),
            "upper trunk lighting must include its lean"
        );
        let trunk_end = (TRUNK_ROWS + 3) * (TRUNK_SIDES + 1);
        assert_closed_component(&mesh.solid, 0..trunk_end);
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
        let visitor_height = 8. * 3.;
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
            let top = s.positions.iter().map(|p| p[1] - 3.)
                .fold(0.0_f32, f32::max);
            assert!(top > visitor_height * 3., "park trees must tower over visitors");
            assert!(s.positions.iter().filter(|p| p[0].abs() > 16. || p[2].abs() > 16.)
                .all(|p| p[1] - 3. > visitor_height),
                "spreading boughs must clear people on adjacent paths");
            if let Some(p) = previous {
                assert_ne!(s.positions, p);
            }
            previous = Some(s.positions.clone());
        }
    }
}
