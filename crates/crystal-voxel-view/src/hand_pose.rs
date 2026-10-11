//! Relax the supplied grip in visible Idle/Run poses only.
use crate::open_people::PersonRig;
use bevy::prelude::*;

pub(crate) fn relax(rig: &PersonRig, pose: &mut [Transform]) {
    for &node in &rig.finger_nodes {
        pose[node].rotation = rig.nodes[node]
            .rotation
            .slerp(pose[node].rotation, 0.40)
            .normalize();
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
