//! World-only flight membranes beneath Ledyba's authored wing cases.
use super::*;

pub(super) fn build(rig: &mut SpeciesRig) {
    if rig.species.name() != "ledyba" { return; }
    let height = rig.neutral_bounds.1.y - rig.neutral_bounds.0.y;
    let parent = rig.joints.iter().position(|j| j.name.ends_with("/thorax")).unwrap();
    let parent_world = rig.joints[parent].inverse_bind.inverse();
    let material = rig.materials.len();
    let color = [0.86, 0.91, 0.80, 1.];
    rig.materials.push(Material { name: "Ledyba world flight membrane".into(), color,
        alpha_mode: gltf::material::AlphaMode::Opaque });
    for (side, label) in [(-1., "left"), (1., "right")] {
        let case = rig.joints.iter().find(|j| j.name.ends_with(&format!("/case_{label}"))).unwrap();
        let pivot = case.inverse_bind.inverse().transform_point3(Vec3::ZERO)
            + Vec3::new(side * height * 0.025, 0., -height * 0.035);
        let joint = rig.joints.len();
        rig.joints.push(Joint { name: format!("ledyba/wing_{label}"), parent: Some(parent),
            bind: Transform::from_translation(parent_world.inverse().transform_point3(pivot)),
            inverse_bind: Mat4::from_translation(-pivot) });
        let first_vertex = rig.neutral.positions.len();
        let first_index = rig.neutral.indices.len();
        // A shallow ribbed oval, rooted at its inner edge. Both faces have
        // explicit winding so the membrane stays visible through the stroke.
        let center = pivot + Vec3::new(side * height * 0.23, 0., -height * 0.045);
        for face in [-1., 1.] {
            let base = rig.neutral.positions.len() as u32;
            for segment in 0..=24 {
                let point = if segment == 0 { center + Vec3::Y * height * 0.012 }
                    else { let angle = (segment - 1) as f32 * TAU / 24.;
                        center + Vec3::new(side * height * 0.23 * angle.cos(),
                            0., height * 0.105 * angle.sin()) };
                rig.neutral.positions.push(point.to_array());
                rig.neutral.normals.push((Vec3::Y * face).to_array());
                rig.source_normals.push((Vec3::Y * face).to_array());
                rig.neutral.uvs.push([0., 0.]);
                rig.neutral.colors.push(color);
                rig.joint_indices.push([joint as u16, 0, 0, 0]);
                rig.joint_weights.push([1., 0., 0., 0.]);
            }
            for segment in 0..24 {
                let a = base + 1 + segment;
                let b = base + 1 + (segment + 1) % 24;
                let ring = if side * face > 0. { [base, b, a] } else { [base, a, b] };
                rig.neutral.indices.extend(ring);
            }
        }
        rig.anatomy.push(Anatomy { name: format!("Wing / world Ledyba flight membrane {label}"),
            vertices: first_vertex..rig.neutral.positions.len(),
            indices: first_index..rig.neutral.indices.len(), material });
    }
    rig.neutral_bounds = point_bounds(rig.neutral.positions.iter().copied().map(Vec3::from_array));
}

pub(super) fn open_cases(rig: &SpeciesRig, pose: &mut [Transform]) {
    if !matches!(rig.species.name(), "ledyba" | "ledian") { return; }
    for (i, joint) in rig.joints.iter().enumerate() {
        if joint.name.contains("/case_") {
            let side = if joint.name.ends_with("left") { -1. } else { 1. };
            pose[i].rotation = joint.bind.rotation
                * Quat::from_rotation_y(side * 0.6) * Quat::from_rotation_z(side * 0.8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladybird_flight_opens_cases_and_strokes_visible_membranes() {
        for name in ["LEDYBA", "LEDIAN"] {
            let canonical = crate::species_rig::for_species(name).unwrap();
            let rig = world_rig(canonical);
            assert_eq!(&rig.neutral.positions[..canonical.neutral.positions.len()],
                canonical.neutral.positions.as_slice(), "{name} changed authored body geometry");
            for v in 0..canonical.neutral.positions.len() {
                if weld_domain(canonical, v) != 0 {
                    assert_eq!(rig.joint_indices[v], canonical.joint_indices[v], "{name} welded separate case hinges");
                    assert_eq!(rig.joint_weights[v], canonical.joint_weights[v]);
                }
            }
            let wings: Vec<_> = rig.motion_limbs.iter()
                .filter(|limb| limb.levelled_flight.is_some()).collect();
            assert_eq!(wings.len(), 2, "{name} needs two visible flight membranes");
            let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
            let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
            for wing in wings {
                let tip = wing.levelled_flight.unwrap().2;
                let mut relative = Vec::new();
                for fraction in [0.25, 0.75] {
                    rig.sample_locomotion(fraction / flight_beat(rig.species.name()).0,
                        0., 1., &mut pose);
                    rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                    let pivot = gait::world(rig, &pose, wing.joint).transform_point3(Vec3::ZERO);
                    relative.push((rig.skin_point(tip, &matrices) - pivot).y);
                    for (i, joint) in rig.joints.iter().enumerate() {
                        if joint.name.contains("/case_") {
                            assert!(pose[i].rotation.angle_between(joint.bind.rotation) > 0.7);
                        }
                    }
                }
                assert!(relative[0] * relative[1] < 0., "{name} membrane never crosses shoulder");
            }
            assert!(rig.anatomy.iter().filter(|part| part.name.starts_with("Wing /"))
                .all(|part| part.vertices.len() >= 3));
        }
    }
}

// The two elytra deliberately separate at their authored centre cut. Paint
// and normal splits within either case still share a welding domain.
pub(super) fn weld_domain(rig: &SpeciesRig, vertex: usize) -> u16 {
    if !matches!(rig.species.name(), "ledyba" | "ledian") { return 0; }
    rig.joint_indices[vertex].iter().zip(rig.joint_weights[vertex])
        .find_map(|(&joint, weight)| (weight > 0. && rig.joints[joint as usize].name.contains("/case_"))
            .then_some(joint)).unwrap_or(0)
}
