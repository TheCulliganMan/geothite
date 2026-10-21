//! Round the connected torso/arm junction, rather than pinning both halves.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;
pub(crate) mod root_shape;

/// Earlier torso and sleeve sculpts deliberately pin their shared borders.
/// Finish that border as one surface before fitting the continuous skin field.
/// Geometric selection includes every copy on separate texture islands.
pub(crate) fn round(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_ARM_SURFACE_BASELINE").is_some() {
        return;
    }
    let carriers: Vec<_> = [
        crate::arm_skin::carriers(rig, 0),
        crate::arm_skin::carriers(rig, 1),
    ]
    .into_iter()
    .flatten()
    .collect();
    for _ in 0..2 {
        let selected: Vec<_> = mesh
            .positions
            .iter()
            .zip(joints.iter())
            .zip(weights.iter())
            .map(|((&p, ids), values)| {
                (0.17..0.96).contains(&p[0].abs())
                    && (1.93..2.66).contains(&p[1])
                    && (0..4)
                        .filter(|&s| carriers.contains(&(ids[s] as usize)))
                        .map(|s| values[s])
                        .sum::<f32>()
                        > 0.999
            })
            .collect();
        crate::hand_surface::round_selection_joined(mesh, joints, weights, &selected, 0.04);
    }
}

/// Refined chest transitions need normals from the finished connected surface,
/// including corners pinned by an earlier, separate garment operation.
pub(crate) fn finish_normals(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    joints: &[[u16; 4]],
    weights: &[[f32; 4]],
) {
    use std::collections::{HashMap, HashSet};
    let carriers: Vec<_> = [
        crate::arm_skin::carriers(rig, 0),
        crate::arm_skin::carriers(rig, 1),
    ]
    .into_iter()
    .flatten()
    .collect();
    let key = |p: [f32; 3]| p.map(|v| (v * 100_000.).round() as i32);
    let minimum_x=0_f32;
    #[cfg(all(feature="people-review",not(target_arch="wasm32")))]
    let minimum_x=if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_SPINE_REFINEMENT_BASELINE").is_some() {0.08} else {minimum_x};
    let mut referenced = vec![false; mesh.positions.len()];
    for &i in &mesh.indices {
        referenced[i as usize] = true;
    }
    let selected: Vec<_> = mesh
        .positions
        .iter()
        .zip(joints)
        .zip(weights)
        .enumerate()
        .map(|(i, ((&p, j), w))| {
            referenced[i]
                && (minimum_x..1.04).contains(&p[0].abs())
                && (1.78..2.68).contains(&p[1])
                && (0..4)
                    .filter(|&s| carriers.contains(&(j[s] as usize)))
                    .map(|s| w[s])
                    .sum::<f32>()
                    > 0.999
        })
        .collect();
    let needed: HashSet<_> = mesh
        .positions
        .iter()
        .zip(&selected)
        .filter_map(|(&p, &yes)| yes.then_some(key(p)))
        .collect();
    let mut normals = HashMap::<[i32; 3], Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        if !p.iter().any(|p| needed.contains(&key(p.to_array()))) {
            continue;
        }
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        for corner in 0..3 {
            let k = key(p[corner].to_array());
            if !needed.contains(&k) {
                continue;
            }
            let a = (p[(corner + 1) % 3] - p[corner]).normalize_or_zero();
            let b = (p[(corner + 2) % 3] - p[corner]).normalize_or_zero();
            *normals.entry(k).or_default() += normal * a.dot(b).clamp(-1., 1.).acos();
        }
    }
    for i in 0..mesh.positions.len() {
        if selected[i] {
            let normal = normals[&key(mesh.positions[i])];
            if normal.length_squared() > 1e-12 {
                mesh.normals[i] = normal.normalize().to_array();
            }
        }
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::collections::BTreeMap;

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human rigs")]
    pub(crate) fn supplied_joined_arm_roots_round_without_opening_texture_seams() {
        dense_painted_panels_round_without_uniform_subdivision();
        let key = |p: [f32; 3]| p.map(|v| (v * 100_000.).round() as i32);
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).unwrap();
            let body = rig.refined_body();
            let mut mesh = body.mesh.clone();
            let mut ids = body.ids.clone();
            let mut weights = body.weights.clone();
            round(rig, &mut mesh, &mut ids, &mut weights);
            let old = body.mesh.positions.len();
            assert!(mesh.positions.len() > old);
            assert_eq!(&mesh.uvs[..old], body.mesh.uvs.as_slice());
            assert_eq!(&ids[..old], body.ids.as_slice());
            assert_eq!(&weights[..old], body.weights.as_slice());
            let mut root_changes = [0; 2];
            let mut moved = BTreeMap::new();
            for i in 0..old {
                let before = body.mesh.positions[i];
                let after = Vec3::from_array(mesh.positions[i]);
                let delta = after.distance(Vec3::from_array(before));
                assert!(delta < 0.06, "{name} arm root loses its authored mass");
                if (0.30..0.48).contains(&before[0].abs())
                    && (2.05..2.50).contains(&before[1])
                    && delta > 0.0001
                {
                    root_changes[usize::from(before[0] < 0.)] += 1;
                }
                if before[0].abs() <= 0.17
                    || before[0].abs() >= 0.96
                    || before[1] <= 1.93
                    || before[1] >= 2.66
                {
                    assert_eq!(mesh.positions[i], before, "unrelated body point moved");
                    assert_eq!(mesh.normals[i], body.mesh.normals[i]);
                }
                if let Some(previous) = moved.insert(key(before), after) {
                    assert!(previous.distance(after) < 1e-5, "UV copies separated");
                }
            }
            assert!(
                root_changes.iter().all(|&n| n > 20),
                "{name} retains its pinned arm-root rings: {root_changes:?}"
            );
            let mut edges = BTreeMap::new();
            for face in mesh.indices.chunks_exact(3) {
                for e in 0..3 {
                    let a = key(mesh.positions[face[e] as usize]);
                    let b = key(mesh.positions[face[(e + 1) % 3] as usize]);
                    *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
            for ((a, b), count) in edges {
                if [a, b].iter().all(|p| {
                    (195_000..265_000).contains(&p[1]) && (20_000..90_000).contains(&p[0].abs())
                }) {
                    assert_eq!(count, 2, "{name} arm root has an open/nonmanifold edge");
                }
            }
            for i in 0..mesh.positions.len() {
                assert!(Vec3::from_array(mesh.positions[i]).is_finite());
                let n = Vec3::from_array(mesh.normals[i]);
                assert!(n.is_finite() && (n.length() - 1.).abs() < 1e-4);
                assert!(weights[i].iter().all(|&w| w >= 0. && w.is_finite()));
                assert!((weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
                assert!(
                    ids[i]
                        .iter()
                        .all(|&j| usize::from(j) < rig.skin_nodes.len())
                );
            }
            // The old arm-only bounds omitted this long chest transition.
            // Its lower corner must join refinement even below y=2.0.
            let [_, arm, shoulder, upper, chest] = crate::arm_skin::ancestry(rig, 1);
            let points = vec![
                [-0.27108687, 2.1776874, 0.19109596],
                [-0.2707549, 2.164733, 0.19237849],
                [-0.12723586, 1.9690177, 0.24660161],
            ];
            let mut transition = SurfaceMeshData {
                positions: points.clone(),
                normals: vec![[0., 0., 1.]; 3],
                uvs: vec![[0.3, 0.7]; 3],
                colors: vec![[1.; 4]; 3],
                indices: vec![0, 1, 2],
                ..Default::default()
            };
            let mut ids = vec![[chest as u16, upper as u16, shoulder as u16, arm as u16]; 3];
            let mut weights = vec![[0.1, 0.4, 0.3, 0.2]; 3];
            crate::arm_skin::refine(rig, &mut transition, &mut ids, &mut weights);
            assert!(
                transition.indices.len() > 3,
                "long chest-to-arm transition was skipped"
            );
            assert_eq!(&transition.positions[..3], points.as_slice());
        }
    }

    #[cfg_attr(test, test)]
    pub(crate) fn dense_painted_panels_round_without_uniform_subdivision() {
        let mut positions = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                positions.push([
                    x as f32 * 0.01,
                    y as f32 * 0.01,
                    if x == 1 && y == 1 { 0.006 } else { 0. },
                ]);
            }
        }
        positions.push(positions[4]); // a separate painting island at the crease
        let mut indices = Vec::new();
        for y in 0..2 {
            for x in 0..2 {
                let a = y * 3 + x;
                indices.extend([a, a + 1, a + 3, a + 1, a + 4, a + 3]);
            }
        }
        for i in &mut indices[12..] {
            if *i == 4 {
                *i = 9;
            }
        }
        let mut mesh = SurfaceMeshData {
            positions,
            normals: vec![[0., 0., 1.]; 10],
            uvs: vec![[0.3, 0.7]; 10],
            colors: vec![[1.; 4]; 10],
            indices,
            ..Default::default()
        };
        mesh.uvs[9] = [0.8, 0.2];
        let before = mesh.clone();
        let mut ids = vec![[0, 0, 0, 0]; 10];
        let mut weights = vec![[1., 0., 0., 0.]; 10];
        crate::hand_surface::round_selection_joined(
            &mut mesh,
            &mut ids,
            &mut weights,
            &[true; 10],
            0.04,
        );
        assert_eq!(
            mesh.positions.len(),
            10,
            "dense painting was uniformly subdivided"
        );
        assert_eq!(mesh.indices, before.indices);
        assert_eq!(mesh.uvs, before.uvs);
        assert!(
            mesh.positions[4][2] < 0.005,
            "interior crease did not round"
        );
        assert_eq!(
            mesh.positions[4], mesh.positions[9],
            "painting islands separated"
        );
        for i in [0, 1, 2, 3, 5, 6, 7, 8] {
            assert_eq!(mesh.positions[i], before.positions[i]);
        }
        for n in mesh.normals {
            assert!((Vec3::from_array(n).length() - 1.).abs() < 1e-5);
        }
    }
}
