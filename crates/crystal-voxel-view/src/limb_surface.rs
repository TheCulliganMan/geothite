//! Connected, rounded limb contours on the shared weighted human bodies.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};

fn joints(rig: &PersonRig) -> Vec<usize> {
    let mut limbs: Vec<_> = rig.leg_joints.into_iter().flatten().collect();
    for hand in &rig.hand_joints {
        let wrist = hand
            .iter()
            .map(|&j| rig.skin_nodes[j])
            .find(|&node| {
                rig.parents[node].is_none_or(|p| !hand.iter().any(|&j| rig.skin_nodes[j] == p))
            })
            .expect("human requires a wrist");
        let forearm = rig.parents[wrist].expect("human requires a forearm");
        let arm = rig.parents[forearm].expect("human requires an upper arm");
        for node in [arm, forearm] {
            limbs.push(
                rig.skin_nodes
                    .iter()
                    .position(|&n| n == node)
                    .expect("arm must belong to the human skin"),
            );
        }
    }
    limbs
}

/// Keep the neck, torso, hands, ankle/sole rings and their attribute islands.
/// Joined refinement splits neighboring seam triangles too, so a weighted
/// midpoint cannot open a crack against an unsplit edge during a knee bend.
pub(crate) fn refine(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    let limbs = joints(rig);
    for _ in 0..2 {
        let selected = ids
            .iter()
            .zip(weights.iter())
            .map(|(ids, weights)| {
                (0..4)
                    .filter(|&slot| limbs.contains(&(ids[slot] as usize)))
                    .map(|slot| weights[slot])
                    .sum::<f32>()
                    > 0.85
            })
            .collect::<Vec<_>>();
        crate::hand_surface::smooth_selection_joined(mesh, ids, weights, &selected);
    }
}

#[cfg(any(test, feature = "people-review"))]
pub(crate) mod validation {
    use super::*;
    use crate::human_locomotion::{HumanLocomotion, world};
    use bevy::prelude::*;

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human GLBs")]
    pub(crate) fn supplied_limbs_preserve_landmarks_and_animate_without_seam_cracks() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            refine(rig, &mut mesh, &mut ids, &mut weights);
            assert!(
                mesh.positions.len() > rig.mesh.positions.len() + 300,
                "{name} refinement"
            );
            let limbs = joints(rig);
            let mut moved = 0;
            for i in 0..rig.mesh.positions.len() {
                assert_eq!(mesh.uvs[i], rig.mesh.uvs[i], "{name} UV landmark {i}");
                assert_eq!(mesh.colors[i], rig.mesh.colors[i]);
                assert_eq!(ids[i], rig.ids[i]);
                assert_eq!(weights[i], rig.weights[i]);
                let weight: f32 = (0..4)
                    .filter(|&s| limbs.contains(&(ids[i][s] as usize)))
                    .map(|s| weights[i][s])
                    .sum();
                let distance = Vec3::from_array(mesh.positions[i])
                    .distance(Vec3::from_array(rig.mesh.positions[i]));
                if weight <= 0.85 {
                    assert_eq!(
                        mesh.positions[i], rig.mesh.positions[i],
                        "{name} protected {i}"
                    );
                    assert_eq!(mesh.normals[i], rig.mesh.normals[i]);
                }
                assert!(
                    distance < 0.075,
                    "{name} limb volume changed too far at {i}: {distance}"
                );
                moved += usize::from(distance > 0.001);
            }
            assert!(moved > 30, "{name} contours were not refined");
            for i in 0..mesh.positions.len() {
                assert!(Vec3::from_array(mesh.positions[i]).is_finite());
                assert!((Vec3::from_array(mesh.normals[i]).length() - 1.).abs() < 0.001);
                assert!(weights[i].iter().all(|w| w.is_finite() && *w >= 0.));
                assert!((weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
                assert!(ids[i].iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
            }
            // Every duplicate geometric seam must carry the same skin field,
            // including new edge points. Compare deformed points, not only IDs:
            // an equivalent influence can occupy a different palette slot.
            let mut welds = std::collections::HashMap::new();
            let pairs = mesh
                .positions
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    let key = p.map(|v| (v * 100_000.).round() as i32);
                    welds.insert(key, i).map(|other| (other, i))
                })
                .collect::<Vec<_>>();
            let gait = HumanLocomotion::new(rig);
            let mut pose = rig.nodes.clone();
            let bounds = rig.locomotion_bounds();
            for run in [0., 0.5, 1.] {
                for frame in 0..=60 {
                    gait.sample(
                        rig,
                        frame as f32 / 60. * std::f32::consts::TAU,
                        run,
                        &mut pose,
                    );
                    let inverse = world(rig, &pose, rig.mesh_node).inverse();
                    let palette = rig
                        .skin_nodes
                        .iter()
                        .zip(&rig.inverse_binds)
                        .map(|(&n, bind)| inverse * world(rig, &pose, n) * *bind)
                        .collect::<Vec<_>>();
                    let posed = mesh
                        .positions
                        .iter()
                        .enumerate()
                        .map(|(i, &p)| {
                            (0..4)
                                .map(|s| {
                                    palette[ids[i][s] as usize]
                                        .transform_point3(Vec3::from_array(p))
                                        * weights[i][s]
                                })
                                .sum::<Vec3>()
                        })
                        .collect::<Vec<_>>();
                    for p in &posed {
                        assert!(
                            p.is_finite() && p.cmpge(bounds.0).all() && p.cmple(bounds.1).all(),
                            "{name} run={run} frame={frame} bounds: {p}"
                        );
                    }
                    for &(a, b) in &pairs {
                        // The supplied body already has minute seam rounding;
                        // permit that tolerance while rejecting visible tears.
                        assert!(
                            posed[a].distance(posed[b]) < 0.001,
                            "{name} run={run} frame={frame} seam {a}/{b}"
                        );
                    }
                }
            }
        }
    }
}
