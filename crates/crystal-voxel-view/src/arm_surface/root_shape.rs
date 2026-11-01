//! Fair the connected shoulder surface while retaining its authored boundaries.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::math::DVec3;
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn key(p: [f32; 3]) -> [i32; 3] {
    p.map(|v| (v * 100_000.).round() as i32)
}

fn fair(points: &mut [DVec3], faces: &[[usize; 3]], amount: &[f32], strength: f32) {
    let mut neighbors = vec![BTreeMap::<usize, f64>::new(); points.len()];
    for &f in faces {
        let p = f.map(|i| points[i]);
        let area = (p[1] - p[0]).cross(p[2] - p[0]).length();
        if area < 1e-14 {
            continue;
        }
        for c in 0..3 {
            let a = f[(c + 1) % 3];
            let b = f[(c + 2) % 3];
            let cotangent = (points[a] - p[c]).dot(points[b] - p[c]) / area;
            for (a, b) in [(a, b), (b, a)] {
                if amount[a] > 0. {
                    *neighbors[a].entry(b).or_default() += cotangent;
                }
            }
        }
    }
    let denominators: Vec<_> = neighbors
        .iter()
        .map(|n| n.values().map(|w| w.abs()).sum::<f64>())
        .collect();
    let active: Vec<_> = amount
        .iter()
        .enumerate()
        .filter_map(|(i, &a)| (a > 0. && denominators[i] > 1e-10).then_some(i))
        .collect();
    let mut next = points.to_vec();
    // Paired positive/negative Laplacian steps filter local creases while
    // retaining the broad shoulder mass. Fixed boundary copies remain pinned.
    for _ in 0..32 {
        for step in [0.4_f32, -0.42] {
            for &i in &active {
                // Cotangent weights reproduce a plane even when its triangles
                // have very different widths. A plain neighbor mean slides
                // narrow painted triangles sideways and can turn them over.
                let laplacian = neighbors[i]
                    .iter()
                    .map(|(&j, &w)| (points[j] - points[i]) * w)
                    .sum::<DVec3>()
                    / denominators[i];
                next[i] = points[i] + laplacian * f64::from(step * strength * amount[i]);
            }
            for &i in &active {
                points[i] = next[i];
            }
        }
    }
}

pub(crate) fn finish(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &[[u16; 4]],
    weights: &[[f32; 4]],
) {
    let strength = 1_f32;
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    let strength = std::env::var("CRYSTAL_PEOPLE_REVIEW_ROOT_FAIRING")
        .ok()
        .map_or(strength, |v| {
            let value: f32 = v.parse().expect("shoulder fairing must be a number");
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            value
        });
    finish_with_strength(rig, mesh, ids, weights, strength);
}

fn finish_with_strength(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &[[u16; 4]],
    weights: &[[f32; 4]],
    strength: f32,
) {
    if strength == 0. {
        return;
    }
    let arms = [
        crate::arm_skin::ancestry(rig, 0),
        crate::arm_skin::ancestry(rig, 1),
    ];
    let origins = arms.map(|a| {
        rig.inverse_binds[a[1]]
            .inverse()
            .transform_point3(Vec3::ZERO)
    });
    let carriers: Vec<_> = [
        crate::arm_skin::carriers(rig, 0),
        crate::arm_skin::carriers(rig, 1),
    ]
    .into_iter()
    .flatten()
    .collect();
    let units = origins[0].distance(
        rig.inverse_binds[arms[0][0]]
            .inverse()
            .transform_point3(Vec3::ZERO),
    ) / 0.47;
    let mut lookup = HashMap::new();
    let mut points = Vec::new();
    let mut amount = Vec::<f32>::new();
    let mut record = vec![None; mesh.positions.len()];
    for &i in &mesh.indices {
        let i = i as usize;
        if record[i].is_some() {
            continue;
        }
        let p = Vec3::from_array(mesh.positions[i]);
        let side = usize::from(p.x < 0.);
        let origin = origins[side];
        let attached = (0..4)
            .filter(|&s| carriers.contains(&(ids[i][s] as usize)))
            .map(|s| weights[i][s])
            .sum::<f32>()
            > 0.999;
        let x = (p.x.abs() - origin.x.abs()) / units;
        let y = (p.y - origin.y) / units;
        let fade = if attached {
            smooth((x + 0.25) / 0.08)
                * smooth((0.30 - x) / 0.08)
                * smooth((y + 0.39) / 0.10)
                * smooth((0.32 - y) / 0.09)
        } else {
            0.
        };
        let n = *lookup.entry(key(p.to_array())).or_insert_with(|| {
            let n = points.len();
            points.push(p.as_dvec3());
            amount.push(fade);
            n
        });
        amount[n] = amount[n].min(fade);
        record[i] = Some(n);
    }
    let faces: Vec<_> = mesh
        .indices
        .chunks_exact(3)
        .map(|f| [f[0], f[1], f[2]].map(|i| record[i as usize].unwrap()))
        .collect();
    let original = points.clone();
    fair(&mut points, &faces, &amount, strength);
    for (&p, &q) in original.iter().zip(&points) {
        assert!(
            q.is_finite() && p.distance(q) < f64::from(0.04 * units),
            "shoulder fairing changed the authored mass"
        );
    }
    for i in 0..mesh.positions.len() {
        if let Some(&n) = lookup.get(&key(mesh.positions[i])) {
            if amount[n] > 0. && points[n] != original[n] {
                mesh.positions[i] = points[n].as_vec3().to_array();
            }
        }
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod validation {
    use super::*;
    #[cfg_attr(test, test)]
    pub(crate) fn thin_obtuse_planar_triangles_keep_their_positions() {
        let origin = DVec3::new(-0.3, 2.4, 0.1);
        let u = DVec3::new(0.4, 0.2, -0.1);
        let v = DVec3::new(0.1, 0.6, 0.3);
        let mut points = vec![[-1., -1.], [1., -1.], [1., 1.], [-1., 1.], [0.9998, 0.3]]
            .into_iter()
            .map(|p| origin + u * p[0] + v * p[1])
            .collect::<Vec<_>>();
        let original = points.clone();
        let faces = [[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
        let plain_mean = original[..4].iter().copied().sum::<DVec3>() / 4.;
        assert!(
            plain_mean.distance(original[4]) > 0.3,
            "fixture must expose tangential neighbor drift"
        );
        fair(&mut points, &faces, &[0., 0., 0., 0., 1.], 1.);
        assert_eq!(&points[..4], &original[..4], "boundary was not pinned");
        assert!(
            points[4].distance(original[4]) < 1e-12,
            "uneven planar triangles were dragged sideways"
        );
        for f in faces {
            let before = f.map(|i| original[i]);
            let after = f.map(|i| points[i]);
            let a = (before[1] - before[0]).cross(before[2] - before[0]);
            let b = (after[1] - after[0]).cross(after[2] - after[0]);
            assert!(a.dot(b) > 0. && (a.length() - b.length()).abs() < 1e-12);
        }
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human rigs")]
    pub(crate) fn supplied_shoulders_keep_their_art_skin_boundaries_and_winding() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).expect("supplied shoulder rig");
            let body = rig.refined_body();
            let mut mesh = body.mesh.clone();
            let mut ids = body.ids.clone();
            let mut weights = body.weights.clone();
            super::super::round(rig, &mut mesh, &mut ids, &mut weights);
            let original = mesh.clone();
            finish_with_strength(rig, &mut mesh, &ids, &weights, 1.);
            assert_eq!(mesh.indices, original.indices);
            assert_eq!(mesh.normals, original.normals);
            assert_eq!(mesh.uvs, original.uvs);
            assert_eq!(mesh.colors, original.colors);
            assert_eq!(mesh.cutaway_ranges, original.cutaway_ranges);
            let mut changes = [0; 2];
            let mut copies = HashMap::<[i32; 3], Vec3>::new();
            let mut maximum = 0_f32;
            for (&before, &after) in original.positions.iter().zip(&mesh.positions) {
                let p = Vec3::from_array(before);
                let q = Vec3::from_array(after);
                assert!(q.is_finite());
                maximum = maximum.max(p.distance(q));
                if p.distance(q) > 0.00001 {
                    changes[usize::from(p.x < 0.)] += 1;
                }
                if p.x.abs() < 0.08 || p.x.abs() > 0.70 || p.y < 1.90 || p.y > 2.75 {
                    assert_eq!(before, after, "{name}: unrelated anatomy moved");
                }
                if let Some(previous) = copies.insert(key(before), q) {
                    assert!(
                        previous.distance(q) < 0.00001,
                        "{name}: shoulder material copies separated"
                    );
                }
            }
            assert!(
                changes.iter().all(|&n| n > 20),
                "{name}: shoulder contour did not change"
            );
            assert!(
                maximum < 0.04,
                "{name}: shoulder mass displaced by {maximum}"
            );
            for face in mesh.indices.chunks_exact(3) {
                let f = [face[0], face[1], face[2]].map(|i| i as usize);
                if f.iter()
                    .all(|&i| mesh.positions[i] == original.positions[i])
                {
                    continue;
                }
                let a = f.map(|i| Vec3::from_array(original.positions[i]));
                let b = f.map(|i| Vec3::from_array(mesh.positions[i]));
                let a = (a[1] - a[0]).cross(a[2] - a[0]);
                let b = (b[1] - b[0]).cross(b[2] - b[0]);
                assert!(
                    a.dot(b) > 0.,
                    "{name}: shoulder fairing reversed a bind face"
                );
                assert!(
                    b.length() / a.length() > 0.20,
                    "{name}: shoulder fairing collapsed a bind face"
                );
            }
            let mut baseline = original.clone();
            let mut baseline_ids = ids.clone();
            let mut baseline_weights = weights.clone();
            crate::arm_skin::refine(rig, &mut baseline, &mut baseline_ids, &mut baseline_weights);
            let mut reference = original.positions.clone();
            crate::arm_skin::refine_reference(
                rig,
                &mut mesh,
                &mut ids,
                &mut weights,
                &mut reference,
            );
            assert_eq!(
                reference, baseline.positions,
                "{name}: reference refinement diverged"
            );
            assert_eq!(
                mesh.indices, baseline.indices,
                "{name}: shaping changed tessellation decisions"
            );
            assert_eq!(mesh.uvs, baseline.uvs);
            assert_eq!(mesh.colors, baseline.colors);
            assert_eq!(mesh.normals, baseline.normals);
            assert_eq!(mesh.cutaway_ranges, baseline.cutaway_ranges);
            assert_eq!(ids, baseline_ids);
            assert_eq!(weights, baseline_weights);

            // Actual upper-back fan: its center corner is close to the spine.
            // The preceding all-corners x > .08 condition skipped this face.
            let [_, arm, shoulder, upper, chest] = crate::arm_skin::ancestry(rig, 1);
            let points = vec![
                [-0.1298, 2.2092, -0.1752],
                [-0.1291, 2.2185, -0.1764],
                [0.0002412, 2.3609, -0.2062],
            ];
            let mut fan = SurfaceMeshData {
                positions: points.clone(),
                normals: vec![[0., 0., -1.]; 3],
                uvs: vec![[0.3, 0.7]; 3],
                colors: vec![[1.; 4]; 3],
                indices: vec![0, 1, 2],
                ..Default::default()
            };
            let mut fan_ids = vec![[upper as u16, chest as u16, shoulder as u16, arm as u16]; 3];
            let mut fan_weights = vec![[0.45, 0.45, 0.10, 0.]; 3];
            fan_weights[2] = [1., 0., 0., 0.];
            crate::arm_skin::refine(rig, &mut fan, &mut fan_ids, &mut fan_weights);
            assert!(
                fan.indices.len() > 12,
                "{name}: spine-to-shoulder fan stayed coarse"
            );
            assert_eq!(&fan.positions[..3], points.as_slice());
        }
    }

    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn validate() {
        thin_obtuse_planar_triangles_keep_their_positions();
        supplied_shoulders_keep_their_art_skin_boundaries_and_winding();
        eprintln!(
            "Shoulder shape checks: tilted needle-triangle plane retained; both supplied rigs preserve artwork, topology, skin, fixed anatomy, joined material copies and positive bind winding."
        );
    }
}
