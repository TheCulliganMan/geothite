//! Repose the upright battle sculpture into a four-paw world travelling rig.
//! The canonical asset and its battle clips are never mutated.
use bevy::prelude::{Mat4, Quat, Transform, Vec3};

use crate::species_rig::{Clip, Playback, Species, SpeciesRig};

use super::gait;

pub(super) fn build(rig: &mut SpeciesRig) {
    if rig.species != Species::Furret {
        return;
    }
    let pelvis = rig
        .joints
        .iter()
        .position(|joint| joint.name.ends_with("/pelvis"))
        .expect("authored Furret pelvis");
    // The low battle body is fixed to the presentation root. During travel
    // the complete trunk must turn with the pelvis, without folding its base.
    let body = rig
        .anatomy
        .iter()
        .filter(|part| part.name.starts_with("Body /") || part.name.starts_with("Back coat /"));
    for part in body {
        for v in part.vertices.clone() {
            for (joint, weight) in rig.joint_indices[v].iter_mut().zip(rig.joint_weights[v]) {
                if *joint == 0 && weight > 0. {
                    *joint = pelvis as u16;
                }
            }
        }
    }
    // The original stationary hind paws were painted on the presentation
    // root. Carry each complete paw with its named haunch during reposing.
    for part in &rig.anatomy {
        if !(part.name.starts_with("Foot /") || part.name.starts_with("Leg /")) {
            continue;
        }
        let side: i32 = part
            .name
            .split_ascii_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        let suffix = if side < 0 { "/leg_left" } else { "/leg_right" };
        let joint = rig
            .joints
            .iter()
            .position(|joint| joint.name.ends_with(suffix))
            .unwrap();
        for v in part.vertices.clone() {
            rig.joint_indices[v] = [joint as u16, 0, 0, 0];
            rig.joint_weights[v] = [1., 0., 0., 0.];
        }
    }
    let original = rig.clone();
    let mut pose = rig
        .joints
        .iter()
        .map(|joint| joint.bind)
        .collect::<Vec<_>>();
    let index = |suffix: &str| {
        original
            .joints
            .iter()
            .position(|joint| joint.name.ends_with(suffix))
            .expect("authored Furret joint")
    };
    let pelvis = index("/pelvis");
    let neck = index("/neck");
    pose[pelvis].rotation *= Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    // Spread the turn across the chest and neck, preserving an upright face.
    pose[index("/chest")].rotation *= Quat::from_rotation_x(-40_f32.to_radians());
    pose[neck].rotation *= Quat::from_rotation_x(-50_f32.to_radians());
    pose[index("/tail_base")].rotation *= Quat::from_rotation_x(-2.22);
    pose[index("/tail_middle")].rotation *= Quat::from_rotation_x(-0.50);
    for (suffix, angle) in [
        ("/arm_left", -50_f32.to_radians()),
        ("/arm_right", -50_f32.to_radians()),
        ("/leg_left", -std::f32::consts::FRAC_PI_2),
        ("/leg_right", -std::f32::consts::FRAC_PI_2),
    ] {
        let joint = index(suffix);
        pose[joint].rotation *= Quat::from_rotation_x(angle);
        if suffix.starts_with("/arm_") {
            // The travelling shoulder sits under the chest, below the high
            // battle shoulder. Move its complete buried root before planting.
            let chest_span = pose[joint].translation.y;
            pose[joint].translation.y = chest_span * (0.060 / 0.192);
            pose[joint].translation.z += chest_span * (0.060 / 0.192);
        }
    }
    // Fit each complete paw to the same substrate. This changes the imported
    // rest posture once; the shared anatomical IK supplies actual travel.
    let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
    original.skin_matrices_into(&pose, &mut matrices).unwrap();
    let clearance = idle_clearance(&original, &pose);
    pose[pelvis].translation.y +=
        (-clearance).max(0.) + (original.neutral_bounds.1.y - original.neutral_bounds.0.y) * 0.008;
    for suffix in ["/arm_left", "/arm_right", "/leg_left", "/leg_right"] {
        let joint = index(suffix);
        original.skin_matrices_into(&pose, &mut matrices).unwrap();
        let floor = original
            .anatomy
            .iter()
            .filter(|part| part.name.starts_with("Hand /") || part.name.starts_with("Foot /"))
            .flat_map(|part| part.vertices.clone())
            .filter(|&v| {
                original.joint_indices[v]
                    .iter()
                    .zip(original.joint_weights[v])
                    .any(|(&owner, w)| owner as usize == joint && w > 0.5)
            })
            .map(|v| original.skin_point(v, &matrices).y)
            .fold(f32::INFINITY, f32::min);
        assert!(floor.is_finite(), "Furret paw has no authored surface");
        let parent = original.joints[joint].parent.unwrap();
        let rotation = gait::world(&original, &pose, parent)
            .to_scale_rotation_translation()
            .1;
        pose[joint].translation += rotation.inverse() * (Vec3::NEG_Y * floor);
    }
    original.skin_matrices_into(&pose, &mut matrices).unwrap();
    for v in 0..rig.neutral.positions.len() {
        rig.neutral.positions[v] = original.skin_point(v, &matrices).to_array();
        let normal = Vec3::from_array(original.neutral.normals[v]);
        let bent = original.joint_indices[v]
            .iter()
            .zip(original.joint_weights[v])
            .map(|(&joint, w)| matrices[joint as usize].transform_vector3(normal) * w)
            .sum::<Vec3>()
            .normalize_or_zero();
        rig.neutral.normals[v] = bent.to_array();
    }
    // Rebase both the skin and idle rotations onto the new local rest frames.
    // Every neutral skin matrix remains identity after baking the posture.
    let mut world = vec![Mat4::IDENTITY; rig.joints.len()];
    for i in 0..rig.joints.len() {
        let delta = pose[i].rotation * original.joints[i].bind.rotation.inverse();
        if i > 0 {
            for clip in &mut rig.clips {
                for key in &mut clip.rotations[i - 1] {
                    *key = (delta * *key).normalize();
                }
            }
        }
        rig.joints[i].bind = pose[i];
        world[i] = rig.joints[i].parent.map_or(pose[i].to_matrix(), |parent| {
            world[parent] * pose[i].to_matrix()
        });
        rig.joints[i].inverse_bind = world[i].inverse();
    }
    for part in &mut rig.anatomy {
        let owner = original.joint_indices[part.vertices.start][0] as usize;
        let suffix = original.joints[owner].name.rsplit('/').next().unwrap();
        if suffix.starts_with("arm_") {
            let side = if suffix.ends_with("left") {
                "left"
            } else {
                "right"
            };
            let category = if part.name.starts_with("Hand /") {
                "Paw"
            } else {
                "Foreleg"
            };
            part.name = format!("{category} / Furret front {side}");
        } else if suffix.starts_with("leg_") {
            part.name.push_str(" rear");
        }
    }
    for joint in &mut rig.joints {
        joint.name = joint.name.replace("/arm_", "/leg_front_");
    }
    rig.neutral_bounds =
        super::point_bounds(rig.neutral.positions.iter().copied().map(Vec3::from_array));
}

// The longer tail must clear the floor while the stationary idle still plays.
// Fit its entire sampled idle envelope before planting the four separate paws.
fn idle_clearance(original: &SpeciesRig, rest: &[Transform]) -> f32 {
    let mut pose = rest.to_vec();
    let mut matrices = vec![Mat4::IDENTITY; original.joints.len()];
    let mut floor = f32::INFINITY;
    for &time in &original.clips[0].times {
        original
            .sample_into(Clip::Idle, time, Playback::Clamp, &mut pose)
            .expect("authored Furret idle has a finite clock");
        for (i, joint) in original.joints.iter().enumerate() {
            pose[i].rotation = rest[i].rotation * joint.bind.rotation.inverse() * pose[i].rotation;
            pose[i].translation = rest[i].translation;
        }
        original
            .skin_matrices_into(&pose, &mut matrices)
            .expect("authored Furret repose retains its skeleton");
        let surfaces = original.anatomy.iter().filter(|part| {
            part.name.starts_with("Body /")
                || part.name.starts_with("Back coat /")
                || part.name.starts_with("Tail /")
                || part.name.starts_with("Tail tuft /")
        });
        for part in surfaces {
            for v in part.vertices.clone() {
                floor = floor.min(original.skin_point(v, &matrices).y);
            }
        }
    }
    floor
}

#[cfg(test)]
mod tests {
    use super::super::world_rig;
    use super::*;
    use std::f32::consts::TAU;
    #[test]
    fn furret_world_travel_uses_four_paws_and_a_low_body_without_rewriting_battle() {
        let canonical = crate::species_rig::for_species("FURRET").unwrap();
        let before = canonical.neutral.positions.clone();
        let rig = world_rig(canonical);
        assert_eq!(canonical.neutral.positions, before);
        assert_eq!(rig.neutral.indices, canonical.neutral.indices);
        assert_eq!(rig.neutral.colors, canonical.neutral.colors);
        assert_eq!(rig.walk_legs.len(), 4);
        let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
        assert!(height < (canonical.neutral_bounds.1.y - canonical.neutral_bounds.0.y) * 0.65);
        assert!(rig.neutral_bounds.1.z - rig.neutral_bounds.0.z > height * 1.5);
        let paws: Vec<_> = rig
            .anatomy
            .iter()
            .filter(|p| p.name.starts_with("Paw /") || p.name.starts_with("Foot /"))
            .collect();
        assert_eq!(paws.len(), 4);
        for paw in &paws {
            let floor = paw
                .vertices
                .clone()
                .map(|v| rig.neutral.positions[v][1])
                .fold(f32::INFINITY, f32::min);
            assert!(
                floor.abs() < 0.00001,
                "{} is not planted: {floor}",
                paw.name
            );
        }
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        for tick in 0..64 {
            rig.sample_locomotion(0.2, tick as f32 * TAU / 64., 1., &mut pose);
            gait::assert_supports(rig, &pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            let floor = (0..rig.neutral.positions.len())
                .map(|v| rig.skin_point(v, &matrices).y)
                .fold(f32::INFINITY, f32::min);
            assert!(
                floor >= -0.00001 && floor < height * 0.01,
                "Furret lost floor contact: {floor}"
            );
        }
    }

    #[test]
    fn furret_world_repose_keeps_the_brown_back_coat_on_its_actual_body() {
        let canonical = crate::species_rig::for_species("FURRET").expect("authored Furret");
        let rig = world_rig(canonical);
        let coat = canonical
            .anatomy
            .iter()
            .find(|part| part.name.starts_with("Back coat /"))
            .expect("Furret has a brown body section");
        let body: Vec<_> = canonical
            .anatomy
            .iter()
            .filter(|part| part.name.starts_with("Body /"))
            .flat_map(|part| part.vertices.clone())
            .collect();
        let seams: Vec<_> = coat
            .vertices
            .clone()
            .filter_map(|v| {
                body.iter()
                    .copied()
                    .find(|&other| {
                        canonical.neutral.positions[other] == canonical.neutral.positions[v]
                    })
                    .map(|other| (v, other))
            })
            .collect();
        let unique: std::collections::HashSet<_> = seams
            .iter()
            .map(|&(v, _)| canonical.neutral.positions[v].map(f32::to_bits))
            .collect();
        assert!(
            unique.len() > 30,
            "the brown coat must share the body's actual material seams"
        );
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        for tick in 0..32 {
            let time = tick as f32 / 8.;
            let weight = [0., 0.5, 1.][tick % 3];
            rig.sample_locomotion(time, tick as f32 * TAU / 8., weight, &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices)
                .expect("finite Furret travel pose");
            for &(v, other) in &seams {
                assert!(
                    rig.skin_point(v, &matrices)
                        .distance(rig.skin_point(other, &matrices))
                        < 0.000001,
                    "Furret coat opens at time {time}, weight {weight}, vertices {v}/{other}"
                );
            }
        }
    }

    #[test]
    fn furret_world_forelimb_root_caps_remain_inside_its_chest() {
        let canonical = crate::species_rig::for_species("FURRET").expect("authored Furret");
        let rig = world_rig(canonical);
        let body: Vec<Vec<[usize; 3]>> = canonical
            .anatomy
            .iter()
            .filter(|part| part.name.starts_with("Body /") || part.name.starts_with("Back coat /"))
            .map(|part| {
                canonical
                    .neutral
                    .indices
                    .chunks_exact(3)
                    .filter(|tri| tri.iter().all(|&v| part.vertices.contains(&(v as usize))))
                    .map(|tri| [tri[0] as usize, tri[1] as usize, tri[2] as usize])
                    .collect()
            })
            .collect();
        let arms = canonical
            .anatomy
            .iter()
            .filter(|part| part.name.starts_with("Arm /"));
        let mut caps = Vec::new();
        for arm in arms {
            let joint = canonical.joint_indices[arm.vertices.start][0] as usize;
            let pivot = canonical.joints[joint]
                .inverse_bind
                .inverse()
                .transform_point3(Vec3::ZERO);
            let center = arm
                .vertices
                .clone()
                .map(|v| Vec3::from_array(canonical.neutral.positions[v]))
                .sum::<Vec3>()
                / arm.vertices.len() as f32;
            let mut cap = std::collections::HashSet::new();
            let triangles = canonical
                .neutral
                .indices
                .chunks_exact(3)
                .filter(|tri| tri.iter().all(|&v| arm.vertices.contains(&(v as usize))));
            for tri in triangles {
                let [a, b, c] = [tri[0], tri[1], tri[2]]
                    .map(|v| Vec3::from_array(canonical.neutral.positions[v as usize]));
                let normal = (b - a).cross(c - a).normalize();
                if normal.dot(a - pivot).abs() < 0.00001 && normal.dot(center - pivot) < -0.01 {
                    cap.extend(tri.iter().map(|&v| v as usize));
                }
            }
            assert_eq!(
                cap.len(),
                18,
                "each complete Furret forelimb root is checked"
            );
            caps.extend(cap);
        }
        assert_eq!(caps.len(), 36);
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        for tick in 0..32 {
            let time = tick as f32 / 8.;
            let weight = [0., 0.5, 1.][tick % 3];
            rig.sample_locomotion(time, tick as f32 * TAU / 8., weight, &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices)
                .expect("finite Furret travel pose");
            let points: Vec<_> = (0..rig.neutral.positions.len())
                .map(|v| rig.skin_point(v, &matrices))
                .collect();
            for &v in &caps {
                assert!(
                    body.iter()
                        .any(|triangles| contains_point(triangles, &points, points[v])),
                    "Furret forelimb root {v} leaves its chest at time {time}, weight {weight}"
                );
            }
        }
    }

    fn contains_point(triangles: &[[usize; 3]], points: &[Vec3], point: Vec3) -> bool {
        let mut hits: Vec<_> = triangles
            .iter()
            .filter_map(|tri| {
                let [a, b, c] = tri.map(|v| points[v]);
                let ab = (b - a).truncate();
                let ac = (c - a).truncate();
                let delta = (point - a).truncate();
                let det = ab.perp_dot(ac);
                if det.abs() < 0.0000000001 {
                    return None;
                }
                let u = delta.perp_dot(ac) / det;
                let w = ab.perp_dot(delta) / det;
                if u < -0.000001 || w < -0.000001 || u + w > 1.000001 {
                    return None;
                }
                Some(a.z + (b.z - a.z) * u + (c.z - a.z) * w)
            })
            .collect();
        hits.sort_by(f32::total_cmp);
        hits.dedup_by(|a, b| (*a - *b).abs() < 0.0000001);
        hits.iter().filter(|&&z| z > point.z + 0.00000001).count() % 2 == 1
    }
}
