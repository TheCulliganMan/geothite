//! Remove imported combat-idle roll without moving the steering contacts.
use super::{joint_world, rotate_world, wrist_node};
use bevy::prelude::*;

pub(crate) fn align(rig: &crate::open_people::PersonRig, pose: &mut [Transform], strength: f32) {
    if strength == 0. {
        return;
    }
    for side in 0..2 {
        let wrist = wrist_node(rig, side);
        let lower = rig.parents[wrist].expect("mounted wrist requires a forearm");
        let upper = rig.parents[lower].expect("mounted forearm requires an upper arm");
        let elbow = joint_world(rig, pose, lower);
        let target = elbow.transform_point3(Vec3::ZERO);
        let orientation = elbow.to_scale_rotation_translation().1;
        let old = pose[upper].rotation;
        pose[upper].rotation = rig.nodes[upper].rotation;
        let origin = joint_world(rig, pose, upper).transform_point3(Vec3::ZERO);
        let from = joint_world(rig, pose, lower).transform_point3(Vec3::ZERO) - origin;
        let to = target - origin;
        rotate_world(
            rig,
            pose,
            upper,
            Quat::from_rotation_arc(from.normalize(), to.normalize()),
        );
        pose[upper].rotation = old.slerp(pose[upper].rotation, strength).normalize();
        let current = joint_world(rig, pose, lower)
            .to_scale_rotation_translation()
            .1;
        rotate_world(rig, pose, lower, orientation * current.inverse());
    }
}
