//! Quiet finger curls and loose running fists on the supplied weighted hands.
use crate::open_people::PersonRig;
use bevy::prelude::*;

struct Finger {
    node: usize,
    open: Quat,
    grip: Quat,
    thumb: bool,
    side: usize,
}

/// Cache the supplied grip's finger rotations only. The combat crouch, wrist
/// rotation and arm placement never enter the native standing/walking pose.
pub(crate) struct Hands(Vec<Finger>);

impl Hands {
    pub(crate) fn new(rig: &PersonRig) -> Self {
        let mut grip = rig.nodes.clone();
        rig.sample("Idle", 0., &mut grip);
        Self(
            rig.finger_nodes
                .iter()
                .map(|&node| {
                    let descendant = |root: usize| {
                        let mut current = Some(node);
                        while let Some(n) = current {
                            if n == root {
                                return true;
                            }
                            current = rig.parents[n];
                        }
                        false
                    };
                    let thumb = rig
                        .thumb_nodes
                        .iter()
                        .flatten()
                        .any(|&root| descendant(root));
                    let side = usize::from(
                        rig.hand_joints[1]
                            .iter()
                            .any(|&j| rig.skin_nodes[j] == node),
                    );
                    Finger {
                        node,
                        open: rig.nodes[node].rotation,
                        grip: grip[node].rotation,
                        thumb,
                        side,
                    }
                })
                .collect(),
        )
    }

    /// Activity blends relaxed fingers into a loose fist. Thumb opposition
    /// stays gentler than index-chain flexion, leaving the palm open inside.
    /// Pulse is a tiny, continuous change in tension, never a grasp event.
    pub(crate) fn sample(&self, activity: f32, pulse: f32, pose: &mut [Transform]) {
        let activity = activity.clamp(0., 1.);
        for finger in &self.0 {
            let pulse = pulse.clamp(-1., 1.) * if finger.side == 0 { 1. } else { -1. };
            let amount = if finger.thumb {
                0.18 + 0.25 * activity + pulse * (0.004 + 0.008 * activity)
            } else {
                0.22 + 0.42 * activity + pulse * (0.006 + 0.018 * activity)
            };
            pose[finger.node].rotation = finger.open.slerp(finger.grip, amount).normalize();
        }
    }
}

/// Source-authored jump, surf and seated clips retain their separate fitting.
pub(crate) fn relax(rig: &PersonRig, pose: &mut [Transform]) {
    for &node in &rig.finger_nodes {
        pose[node].rotation = rig.nodes[node]
            .rotation
            // The supplied animation closes around an absent handle. Keep a
            // small resting bend rather than the hooked half-grip; mounted
            // poses use the separate production grip solver.
            .slerp(pose[node].rotation, 0.20)
            .normalize();
    }
}

#[cfg(any(test, feature = "people-review"))]
pub(crate) mod validation {
    use super::*;
    use crate::human_locomotion::{HumanLocomotion, world};

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human GLBs")]
    pub(crate) fn native_fingers_relax_and_close_without_wrist_or_skin_drift() {
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::get(name).unwrap();
            let hands = Hands::new(rig);
            assert_eq!(hands.0.len(), 10, "{name}: both index and thumb chains");
            let mut rest = rig.nodes.clone();
            hands.sample(0., 0., &mut rest);
            let mut running = rig.nodes.clone();
            hands.sample(1., 0., &mut running);
            let sum_curl = |pose: &[Transform]| {
                hands
                    .0
                    .iter()
                    .map(|finger| {
                        rig.nodes[finger.node]
                            .rotation
                            .angle_between(pose[finger.node].rotation)
                    })
                    .sum::<f32>()
            };
            assert!(sum_curl(&rest) > 0.4, "{name}: flat resting fingers");
            assert!(
                sum_curl(&running) > sum_curl(&rest) + 1.,
                "{name}: run lacks a loose fist"
            );
            for hand in &rig.hand_joints {
                let wrist = hand
                    .iter()
                    .map(|&j| rig.skin_nodes[j])
                    .find(|&n| {
                        rig.parents[n].is_none_or(|p| !hand.iter().any(|&j| rig.skin_nodes[j] == p))
                    })
                    .unwrap();
                // The index chain carries the connected four-finger surface.
                // Its terminal marker must move inward as the hand closes.
                let tip = hands
                    .0
                    .iter()
                    .filter(|finger| {
                        !finger.thumb && hand.iter().any(|&j| rig.skin_nodes[j] == finger.node)
                    })
                    .find_map(|finger| {
                        rig.parents.iter().enumerate().find_map(|(n, &parent)| {
                            (parent == Some(finger.node) && !rig.finger_nodes.contains(&n))
                                .then_some(n)
                        })
                    })
                    .expect("hand requires an index tip marker");
                let span = |pose: &[Transform]| {
                    world(rig, pose, wrist)
                        .transform_point3(Vec3::ZERO)
                        .distance(world(rig, pose, tip).transform_point3(Vec3::ZERO))
                };
                assert!(
                    span(&rest) < span(&rig.nodes) - 0.005,
                    "{name}: resting finger silhouette did not bend"
                );
                assert!(
                    span(&running) < span(&rest) - 0.02,
                    "{name}: running fingertips did not move toward the palm"
                );
            }
            let body = rig.refined_body();
            let fingers = rig
                .hand_joints
                .iter()
                .flatten()
                .copied()
                .collect::<std::collections::HashSet<_>>();
            let bounds = rig.locomotion_bounds();
            let gait = HumanLocomotion::new(rig);
            let mut pose = rig.nodes.clone();
            for activity in [0., 0.25, 0.5, 0.75, 1.] {
                for frame in 0..=64 {
                    let phase = frame as f32 / 64. * std::f32::consts::TAU;
                    gait.sample(rig, phase, activity, &mut pose);
                    let original = pose.clone();
                    hands.sample(activity, phase.sin(), &mut pose);
                    for (n, (a, b)) in original.iter().zip(&pose).enumerate() {
                        assert_eq!(a.translation, b.translation, "{name} finger joint position");
                        assert_eq!(a.scale, b.scale, "{name} finger bone length");
                        assert!(b.rotation.is_finite() && (b.rotation.length() - 1.).abs() < 1e-5);
                        if !rig.finger_nodes.contains(&n) {
                            assert_eq!(
                                a.rotation, b.rotation,
                                "{name} activity={activity}: wrist/body changed at {n}"
                            );
                        }
                    }
                    let inverse = world(rig, &pose, rig.mesh_node).inverse();
                    let palette = rig
                        .skin_nodes
                        .iter()
                        .zip(&rig.inverse_binds)
                        .map(|(&n, bind)| inverse * world(rig, &pose, n) * *bind)
                        .collect::<Vec<_>>();
                    for ((&p, ids), weights) in
                        body.mesh.positions.iter().zip(&body.ids).zip(&body.weights)
                    {
                        if !(0..4).any(|s| weights[s] > 0. && fingers.contains(&(ids[s] as usize)))
                        {
                            continue;
                        }
                        let point = (0..4)
                            .map(|s| {
                                palette[ids[s] as usize].transform_point3(Vec3::from_array(p))
                                    * weights[s]
                            })
                            .sum::<Vec3>();
                        assert!(
                            point.is_finite()
                                && point.cmpge(bounds.0).all()
                                && point.cmple(bounds.1).all(),
                            "{name}: hand outside visible bounds at activity={activity}, frame={frame}"
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_relaxed_fingers_preserve_wrist_body_and_skinning_envelope() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female", "skater-male", "skater-female"] {
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            assert_eq!(rig.finger_nodes.len(), 10, "{name} finger discovery");
            let mut mesh = rig.mesh.clone();
            let mut joints = rig.ids.clone();
            let mut weights = rig.weights.clone();
            crate::hand_surface::smooth(&rig, &mut mesh, &mut joints, &mut weights);
            for clip in ["Idle", "Run"] {
                for step in 0..=60 {
                    let mut pose = rig.nodes.clone();
                    rig.sample(clip, rig.duration(clip) * step as f32 / 60., &mut pose);
                    let original = pose.clone();
                    relax(&rig, &mut pose);
                    let mut changed = 0;
                    for (i, (a, b)) in original.iter().zip(&pose).enumerate() {
                        assert_eq!(a.translation, b.translation);
                        assert_eq!(a.scale, b.scale);
                        assert!(b.rotation.is_finite() && (b.rotation.length() - 1.).abs() < 1e-5);
                        if rig.finger_nodes.contains(&i) {
                            let before = rig.nodes[i].rotation.angle_between(a.rotation);
                            let after = rig.nodes[i].rotation.angle_between(b.rotation);
                            assert!(after <= before + 0.001);
                            changed += usize::from(before > 0.10 && after < before * 0.50);
                        } else {
                            assert_eq!(a.rotation, b.rotation);
                        }
                    }
                    assert!(changed >= 4, "{name} {clip} still has original grip");
                    fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                        parents[i]
                            .map(|p| world(p, pose, parents))
                            .unwrap_or(Mat4::IDENTITY)
                            * pose[i].to_matrix()
                    }
                    let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                    let skin: Vec<_> = rig
                        .skin_nodes
                        .iter()
                        .zip(&rig.inverse_binds)
                        .map(|(&i, b)| inverse * world(i, &pose, &rig.parents) * *b)
                        .collect();
                    for ((&p, j), w) in mesh.positions.iter().zip(&joints).zip(&weights) {
                        let p: Vec3 = (0..4)
                            .map(|a| {
                                skin[j[a] as usize].transform_point3(Vec3::from_array(p)) * w[a]
                            })
                            .sum();
                        assert!(p.is_finite());
                        assert!(
                            p.cmpge(rig.bounds.0 - Vec3::splat(0.02)).all()
                                && p.cmple(rig.bounds.1 + Vec3::splat(0.02)).all(),
                            "{name} {clip} phase {step} culling"
                        );
                    }
                }
            }
        }
    }
}
