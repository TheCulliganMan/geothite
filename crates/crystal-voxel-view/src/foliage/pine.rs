//! Source-seeded Johto pines: exposed limbs and separate needle-bearing sprays.
//! No cone shells, billboard crowns or retained broadleaf grove geometry.
use super::*;

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
) {
    let first = mesh.solid.positions.len();
    let interior = neighbors.iter().all(|&n| n);
    if !interior || (coordinates[0].rem_euclid(4) == 0 && coordinates[1].rem_euclid(4) == 0) {
        let v = variation(coordinates[0], coordinates[1]);
        // Three authored branch recipes: spreading, upright, and swept.
        let (crown_width, crown_height, branch_drop, courses) = [
            (1.08, 0.94, 0.025, 7),
            (0.92, 1.06, -0.005, 8),
            (1.00, 1.00, 0.065, 7),
        ][v.index];
        let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
        let offset = grove_trunk_offset(coordinates, spacing);
        let center = Vec3::new(
            (bounds[0] + bounds[1]) * 0.5 + offset.x,
            base,
            (bounds[2] + bounds[3]) * 0.5 + offset.y,
        );
        let height = tile * (5.0 + v.height * 0.9 + if interior { 1.2 } else { 0.0 }) * crown_height;
        let phase = v.turn + v.height * 5.31;
        let lean = Vec3::new(phase.cos(), 0., phase.sin()) * spacing * 0.055;
        let stem = |t: f32| center + Vec3::Y * height * t + lean * t * t;
        // A continuous fluted trunk, with a rooted flare and unequal warm fissures.
        let start = mesh.solid.positions.len() as u32;
        for row in 0..=12 {
            let t = row as f32 / 12.;
            for side in 0..=12 {
                let a = (side % 12) as f32 * std::f32::consts::TAU / 12.;
                let radial = Vec3::new(a.cos(), 0., a.sin());
                let flare = 1. + (1. - t).powi(9) * (0.6 + 0.3 * (a * 5. + phase).cos());
                let radius = spacing
                    * 0.095
                    * (1. - t * 0.94)
                    * flare
                    * (1. + 0.09 * (a * 7. + phase).sin());
                mesh.solid
                    .positions
                    .push((stem(t) + radial * radius).to_array());
                mesh.solid
                    .normals
                    .push((radial + Vec3::Y * 0.04).normalize().to_array());
                mesh.solid.uvs.push([0.; 2]);
                let grain = 0.78 + 0.22 * (a * 7. + phase + t).cos();
                mesh.solid
                    .colors
                    .push([0.20 * grain, 0.095 * grain, 0.035 * grain, 1.]);
            }
        }
        for row in 0..12 {
            for side in 0..12 {
                let a = start + row * 13 + side;
                let b = a + 13;
                mesh.solid.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
            }
        }
        let density = neighbors.iter().filter(|&&n| n).count() as f32 / 4.;
        let spread = spacing * (1.05 + density * 1.15) * crown_width;
        // Unequal staggered limbs, rather than rotationally identical whorls.
        for level in 0..courses {
            for bough in 0..4 {
                let seed = phase + level as f32 * 2.399963 + bough as f32 * 1.618034;
                if (seed * 3.7).sin() > 0.92 {
                    continue;
                }
                let t = 0.27 + level as f32 * (0.665 / courses as f32) + 0.018 * (seed * 2.3).sin();
                let angle = seed + bough as f32 * 0.18;
                let direction = Vec3::new(angle.cos(), 0., angle.sin());
                let root = stem(t);
                // Mature lower limbs must carry the crown beyond the narrow
                // source trunk plot. Taper the upper courses rather than
                // compressing every course into a pole-like silhouette.
                let reach =
                    spacing * crown_width * (0.96 - 0.70 * level as f32 / courses as f32) * (0.87 + 0.13 * (seed * 1.7).sin());
                let tip = root
                    + direction * reach
                    + Vec3::Y * height * (0.015 - branch_drop - 0.035 * (seed * 2.1).cos().abs());
                append_branch(
                    &mut mesh.solid,
                    root,
                    tip,
                    spacing * 0.026 * (1. - t * 0.65),
                );
                let across = Vec3::new(-direction.z, 0., direction.x);
                for twig in 0..6 {
                    let f = 0.22 + twig as f32 * 0.145;
                    let anchor = root.lerp(tip, f);
                    for side in [-1., 1.] {
                        let length = spread
                            * (1. - t)
                            * 0.40
                            * (0.7 + 0.3 * (seed + twig as f32).cos().abs());
                        let twig_tip = anchor + direction * length * 0.65 + across * side * length
                            - Vec3::Y * length * 0.22;
                        append_twig(
                            &mut mesh.solid,
                            anchor,
                            twig_tip,
                            spacing * 0.009 * (1. - f * 0.6),
                        );
                        spray(
                            &mut mesh.solid,
                            anchor,
                            twig_tip,
                            spread * (1. - t) * 0.32,
                            seed + twig as f32 * 1.7 + side,
                            v.tint,
                        );
                    }
                }
                spray(
                    &mut mesh.solid,
                    root.lerp(tip, 0.4),
                    tip + direction * spread * (1. - t) * 0.30,
                    spread * (1. - t) * 0.30,
                    seed,
                    v.tint,
                );
            }
        }
        // Unequal upper boughs carry lateral sprays into the leader. A few
        // tiny stem-side tufts left the whole upper crown as an exposed mast.
        for level in 0..3 {
            let t = 0.82 + level as f32 * 0.06;
            let shoots = 5 - level;
            for shoot in 0..shoots {
                let angle = phase + level as f32 * 2.399963
                    + shoot as f32 * std::f32::consts::TAU / shoots as f32;
                let direction = Vec3::new(angle.cos(), 0., angle.sin());
                let across = Vec3::new(-direction.z, 0., direction.x);
                let root = stem(t);
                let reach = spacing * (0.52 - level as f32 * 0.15)
                    * (0.86 + 0.14 * (angle * 1.7).sin());
                let tip = root
                    + direction * reach
                    + Vec3::Y * height * 0.015;
                append_twig(&mut mesh.solid, root, tip, spacing * 0.006);
                spray(
                    &mut mesh.solid,
                    root.lerp(tip, 0.25),
                    tip,
                    spacing * (0.26 - level as f32 * 0.05),
                    angle,
                    v.tint,
                );
                for side in [-1., 1.] {
                    let anchor = root.lerp(tip, 0.58);
                    let end = anchor + direction * reach * 0.25
                        + across * reach * side * 0.44;
                    append_twig(&mut mesh.solid, anchor, end, spacing * 0.004);
                    spray(&mut mesh.solid, anchor, end,
                        spacing * (0.20 - level as f32 * 0.035),
                        angle + side * 0.9, v.tint);
                }
            }
        }
        spray(
            &mut mesh.solid,
            stem(0.91),
            stem(1.02),
            spacing * 0.20,
            phase,
            v.tint,
        );
        // Fit the whole organic crown proportionally inside the existing
        // readable margin. A uniform XZ fit retains branches, folded needles
        // and their winding; a per-vertex envelope can crease narrow tips.
        // Scale about the rooted stem, preserving its authored center and base.
        let margin = spacing * 0.75;
        let limits = [
            bounds[0] - margin,
            bounds[1] + margin,
            bounds[2] - margin,
            bounds[3] + margin,
        ];
        let mut fit = 1.0_f32;
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        for p in &mesh.solid.positions[first..] {
            low = low.min(Vec3::from_array(*p));
            high = high.max(Vec3::from_array(*p));
            for (axis, origin, low, high) in [
                (0, center.x, limits[0], limits[1]),
                (2, center.z, limits[2], limits[3]),
            ] {
                let delta = p[axis] - origin;
                if delta > 0. {
                    fit = fit.min((high - origin) / delta);
                }
                if delta < 0. {
                    fit = fit.min((low - origin) / delta);
                }
            }
        }
        // Crowded interior trees get taller leaders, but their fitted crown
        // must still read as a mature tree rather than a long narrow pole.
        let span = high - low;
        let height_fit = (span.x.max(span.z) * fit / (span.y * 0.65)).min(1.);
        for i in first..mesh.solid.positions.len() {
            let p = &mut mesh.solid.positions[i];
            p[0] = center.x + (p[0] - center.x) * fit;
            p[1] = center.y + (p[1] - center.y) * height_fit;
            p[2] = center.z + (p[2] - center.z) * fit;
            let n = Vec3::from_array(mesh.solid.normals[i]);
            mesh.solid.normals[i] = Vec3::new(n.x / fit, n.y / height_fit, n.z / fit)
                .normalize()
                .to_array();
        }
    }
    crate::understory::append(&mut mesh.solid, bounds, base, tile, coordinates);
    mesh.solid
        .cutaway_ranges
        .push(first..mesh.solid.positions.len());
}

fn spray(
    mesh: &mut SurfaceMeshData,
    root: Vec3,
    tip: Vec3,
    width: f32,
    phase: f32,
    tint: [f32; 3],
) {
    let axis = (tip - root).normalize();
    let side = axis
        .cross(if axis.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y })
        .normalize();
    let up = side.cross(axis).normalize();
    for pair in 0..7 {
        let t = (pair as f32 + 0.5) / 7.;
        let p = root.lerp(tip, t);
        for sign in [-1., 1.] {
            // Stagger needle planes around the twig instead of laying every
            // pair on one broad, visibly flat comb.
            let twist = (phase + pair as f32 * 1.9 + sign * 0.7).sin() * 0.6;
            let radial = side * sign * twist.cos() + up * twist.sin();
            let needle_up = up * twist.cos() - side * sign * twist.sin();
            let length = width
                * (0.48 + 0.52 * (std::f32::consts::PI * t).sin())
                * (0.86 + 0.14 * (phase + pair as f32 * 2.7 + sign).sin());
            let end = p
                + (axis * 0.48 + radial) * length
                + needle_up * length * (0.12 + 0.12 * (phase + pair as f32).sin());
            // A closed pointed bundle retains volume from every orbit. Four
            // faces replace the two open folded pairs at the same triangle
            // cost, with half as many local vertices and no broad leaf card.
            let direction = (end - p).normalize();
            let across = direction.cross(needle_up).normalize();
            let mid = p.lerp(end, 0.44);
            let points = [p, mid + across * length * 0.10
                + needle_up * length * 0.065, end,
                mid - across * length * 0.10 + needle_up * length * 0.065];
            let center = points.iter().copied().sum::<Vec3>() / 4.;
            let mut normals = [Vec3::ZERO; 4];
            let start = mesh.positions.len() as u32;
            for mut ids in [[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]] {
                let mut n = (points[ids[1]] - points[ids[0]])
                    .cross(points[ids[2]] - points[ids[0]]);
                if n.dot(points[ids[0]] - center) < 0. {
                    ids.swap(1, 2);
                    n = -n;
                }
                for i in ids { normals[i] += n; }
                mesh.indices.extend(ids.map(|i| start + i as u32));
            }
            for (point, normal) in points.into_iter().zip(normals) {
                let n = normal.normalize();
                let sun = n.dot(Vec3::new(0.45, 0.8, 0.55).normalize()).max(0.);
                let light = 0.65 + 0.35 * sun;
                let tip = 0.25 + 0.5 * sun + 0.25 * t;
                let pigment = 0.83 + 0.17 * (phase + pair as f32 * 1.3).sin();
                mesh.positions.push(point.to_array());
                mesh.normals.push(n.to_array());
                mesh.uvs.push([0.; 2]);
                mesh.colors.push([
                    (0.025 + 0.12 * tip) * tint[0] * light * pigment,
                    (0.09 + 0.22 * tip) * tint[1] * light * pigment,
                    (0.06 + 0.015 * tip) * tint[2] * light * pigment,
                    1.,
                ]);
            }
        }
    }
}

// Thin supporting twigs need fewer rings than the visible primary limbs.
fn append_twig(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, radius: f32) {
    let axis = (tip - root).normalize();
    let side = axis
        .cross(if axis.z.abs() > 0.9 { Vec3::Y } else { Vec3::Z })
        .normalize();
    let up = side.cross(axis).normalize();
    let start = mesh.positions.len() as u32;
    let rings: [Vec3; 7] = std::array::from_fn(|ring| {
        let a = (ring % 6) as f32 * std::f32::consts::TAU / 6.;
        side * a.cos() + up * a.sin()
    });
    for row in 0..=2 {
        let t = row as f32 / 2.;
        for ring in 0..=6 {
            let normal = rings[ring];
            mesh.positions
                .push((root.lerp(tip, t) + normal * radius * (1. - 0.75 * t)).to_array());
            mesh.normals.push(normal.to_array());
            mesh.uvs.push([0.; 2]);
            mesh.colors.push([0.20, 0.095, 0.035, 1.]);
        }
    }
    for row in 0..2 {
        for ring in 0..6 {
            let a = start + row * 7 + ring;
            let b = a + 7;
            mesh.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn needle_tufts_are_closed_slender_bundles_with_unchanged_triangle_budget() {
        for phase in [0.0, 0.7, 2.9] {
            for axis in [Vec3::X, Vec3::Y, Vec3::new(1., -0.3, 0.5)] {
                let mut mesh = SurfaceMeshData::default();
                spray(&mut mesh, Vec3::ZERO, axis * 2., 1., phase, [1.; 3]);
                assert_eq!(mesh.indices.len(), 7 * 2 * 4 * 3);
                assert_eq!(mesh.positions.len(), 7 * 2 * 4);
                for (bundle, faces) in mesh.positions.chunks_exact(4)
                    .zip(mesh.indices.chunks_exact(12)) {
                    let p: Vec<_> = bundle.iter().map(|p| Vec3::from_array(*p)).collect();
                    assert!(p[0].distance(p[2]) / p[1].distance(p[3]) > 4.,
                        "wide foliage blade survived");
                    let center = p.iter().copied().sum::<Vec3>() / 4.;
                    let mut edges = std::collections::BTreeMap::new();
                    for face in faces.chunks_exact(3) {
                        let ids: [usize; 3] = std::array::from_fn(|i| face[i] as usize % 4);
                        let normal = (p[ids[1]]-p[ids[0]]).cross(p[ids[2]]-p[ids[0]]);
                        assert!(normal.dot(p[ids[0]]-center) > 0.,
                            "needle bundle must have finite outward volume");
                        for (a,b) in [(ids[0],ids[1]),(ids[1],ids[2]),(ids[2],ids[0])] {
                            *edges.entry((a.min(b),a.max(b))).or_insert(0) += 1;
                        }
                    }
                    assert!(edges.values().all(|&count| count == 2),
                        "needle bundle must close every edge");
                }
            }
        }
        for neighbors in [[false; 4], [true; 4]] {
            let mut tree = TerrainMeshData::default();
            append(&mut tree, [0., 32., 0., 32.], 3., 16., [4, 8], neighbors);
            assert!(
                tree.solid.positions.len() < u16::MAX as usize,
                "a source tree must retain its compact local index range"
            );
        }
    }

    #[test]
    fn pines_keep_source_roots_stable_cutaways_and_finite_outward_geometry() {
        for coordinates in [[0, 0], [4, 8], [-12, 4], [6, 8]] {
            for neighbors in [[false; 4], [true; 4], [true, false, true, false]] {
                let mut a = TerrainMeshData::default();
                append(&mut a, [0., 32., 0., 32.], 3., 16., coordinates, neighbors);
                let mut b = TerrainMeshData::default();
                append(&mut b, [0., 32., 0., 32.], 3., 16., coordinates, neighbors);
                assert_eq!(a.solid, b.solid);
                assert_eq!(a.solid.cutaway_ranges, vec![0..a.solid.positions.len()]);
                assert!(
                    a.solid
                        .positions
                        .iter()
                        .all(|p| Vec3::from_array(*p).is_finite() && p[1] >= 3.)
                );
                assert!(
                    a.solid
                        .normals
                        .iter()
                        .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
                );
                for tri in a.solid.indices.chunks_exact(3) {
                    let [p, q, r] = std::array::from_fn(|j| {
                        Vec3::from_array(a.solid.positions[tri[j] as usize])
                    });
                    let n: Vec3 = tri
                        .iter()
                        .map(|&i| Vec3::from_array(a.solid.normals[i as usize]))
                        .sum();
                    assert!((q - p).cross(r - p).dot(n) > 0.);
                }
                if !neighbors.iter().all(|&n| n)
                    || coordinates == [4, 8]
                    || coordinates == [0, 0]
                    || coordinates == [-12, 4]
                {
                    assert!(
                        a.solid.positions[..169]
                            .iter()
                            .all(|p| p[0] > 0. && p[0] < 32. && p[2] > 0. && p[2] < 32.)
                    );
                    assert!(a.solid.positions.iter().any(|p| p[1] > 3. + 16. * 5.));
                    let (low, high) = a.solid.positions.iter().fold(
                        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                        |(low, high), &p| {
                            (low.min(Vec3::from_array(p)), high.max(Vec3::from_array(p)))
                        },
                    );
                    let span = high - low;
                    assert!(
                        span.x.max(span.z) > span.y * 0.60,
                        "mature pines need a full canopy rather than a bare pole silhouette: {coordinates:?}, {neighbors:?}, span {span:?}"
                    );
                }
            }
        }
    }
}
