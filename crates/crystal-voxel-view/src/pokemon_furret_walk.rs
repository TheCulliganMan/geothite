//! Repose the upright battle sculpture into a four-paw world travelling rig.
//! The canonical asset and its battle clips are never mutated.
use super::*;

pub(super) fn build(rig: &mut SpeciesRig) {
    if rig.species != Species::Furret { return; }
    // The original stationary hind paws were painted on the presentation
    // root. Carry each complete paw with its named haunch during reposing.
    for part in &rig.anatomy {
        if !(part.name.starts_with("Foot /") || part.name.starts_with("Leg /")) { continue; }
        let side: i32 = part.name.split_ascii_whitespace().last().unwrap().parse().unwrap();
        let suffix = if side < 0 { "/leg_left" } else { "/leg_right" };
        let joint = rig.joints.iter().position(|joint|joint.name.ends_with(suffix)).unwrap();
        for v in part.vertices.clone() {
            rig.joint_indices[v] = [joint as u16,0,0,0];
            rig.joint_weights[v] = [1.,0.,0.,0.];
        }
    }
    let original = rig.clone();
    let mut pose = rig.joints.iter().map(|joint| joint.bind).collect::<Vec<_>>();
    let index = |suffix: &str| original.joints.iter()
        .position(|joint| joint.name.ends_with(suffix)).expect("authored Furret joint");
    let pelvis = index("/pelvis");
    let neck = index("/neck");
    pose[pelvis].rotation *= Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    pose[neck].rotation *= Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    pose[index("/tail_base")].rotation *= Quat::from_rotation_x(-2.22);
    pose[index("/tail_middle")].rotation *= Quat::from_rotation_x(-0.50);
    for suffix in ["/arm_left", "/arm_right", "/leg_left", "/leg_right"] {
        pose[index(suffix)].rotation *= Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    }
    // Fit each complete paw to the same substrate. This changes the imported
    // rest posture once; the shared anatomical IK supplies actual travel.
    let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
    original.skin_matrices_into(&pose,&mut matrices).unwrap();
    let belly = original.anatomy.iter().filter(|part|part.name.starts_with("Body /"))
        .flat_map(|part|part.vertices.clone()).map(|v|original.skin_point(v,&matrices).y)
        .fold(f32::INFINITY,f32::min);
    pose[pelvis].translation.y += (-belly).max(0.)
        + (original.neutral_bounds.1.y-original.neutral_bounds.0.y)*0.008;
    for suffix in ["/arm_left", "/arm_right", "/leg_left", "/leg_right"] {
        let joint = index(suffix);
        original.skin_matrices_into(&pose, &mut matrices).unwrap();
        let floor = original.anatomy.iter()
            .filter(|part| part.name.starts_with("Hand /") || part.name.starts_with("Foot /"))
            .flat_map(|part| part.vertices.clone())
            .filter(|&v| original.joint_indices[v].iter().zip(original.joint_weights[v])
                .any(|(&owner,w)| owner as usize == joint && w > 0.5))
            .map(|v| original.skin_point(v,&matrices).y).fold(f32::INFINITY,f32::min);
        assert!(floor.is_finite(), "Furret paw has no authored surface");
        let parent = original.joints[joint].parent.unwrap();
        let rotation = gait::world(&original,&pose,parent).to_scale_rotation_translation().1;
        pose[joint].translation += rotation.inverse() * (Vec3::NEG_Y * floor);
    }
    original.skin_matrices_into(&pose,&mut matrices).unwrap();
    for v in 0..rig.neutral.positions.len() {
        rig.neutral.positions[v] = original.skin_point(v,&matrices).to_array();
        let normal = Vec3::from_array(original.neutral.normals[v]);
        let bent = original.joint_indices[v].iter().zip(original.joint_weights[v])
            .map(|(&joint,w)|matrices[joint as usize].transform_vector3(normal)*w)
            .sum::<Vec3>().normalize_or_zero();
        rig.neutral.normals[v] = bent.to_array();
    }
    // Rebase both the skin and idle rotations onto the new local rest frames.
    // Every neutral skin matrix remains identity after baking the posture.
    let mut world = vec![Mat4::IDENTITY; rig.joints.len()];
    for i in 0..rig.joints.len() {
        let delta = pose[i].rotation * original.joints[i].bind.rotation.inverse();
        if i > 0 {
            for clip in &mut rig.clips {
                for key in &mut clip.rotations[i-1] { *key = (delta * *key).normalize(); }
            }
        }
        rig.joints[i].bind = pose[i];
        world[i] = rig.joints[i].parent.map_or(pose[i].to_matrix(),
            |parent| world[parent] * pose[i].to_matrix());
        rig.joints[i].inverse_bind = world[i].inverse();
    }
    for part in &mut rig.anatomy {
        let owner = original.joint_indices[part.vertices.start][0] as usize;
        let suffix = original.joints[owner].name.rsplit('/').next().unwrap();
        if suffix.starts_with("arm_") {
            let side = if suffix.ends_with("left") { "left" } else { "right" };
            let category = if part.name.starts_with("Hand /") { "Paw" } else { "Foreleg" };
            part.name = format!("{category} / Furret front {side}");
        } else if suffix.starts_with("leg_") {
            part.name.push_str(" rear");
        }
    }
    for joint in &mut rig.joints {
        joint.name = joint.name.replace("/arm_", "/leg_front_");
    }
    rig.neutral_bounds = point_bounds(rig.neutral.positions.iter().copied().map(Vec3::from_array));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn furret_world_travel_uses_four_paws_and_a_low_body_without_rewriting_battle() {
        let canonical = crate::species_rig::for_species("FURRET").unwrap();
        let before = canonical.neutral.positions.clone();
        let rig = world_rig(canonical);
        assert_eq!(canonical.neutral.positions,before);
        assert_eq!(rig.neutral.indices,canonical.neutral.indices);
        assert_eq!(rig.neutral.colors,canonical.neutral.colors);
        assert_eq!(rig.walk_legs.len(),4);
        let height = rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
        assert!(height < (canonical.neutral_bounds.1.y-canonical.neutral_bounds.0.y)*0.65);
        assert!(rig.neutral_bounds.1.z-rig.neutral_bounds.0.z > height*1.5);
        let paws:Vec<_> = rig.anatomy.iter().filter(|p|p.name.starts_with("Paw /")
            || p.name.starts_with("Foot /")).collect();
        assert_eq!(paws.len(),4);
        for paw in &paws {
            let floor = paw.vertices.clone().map(|v|rig.neutral.positions[v][1])
                .fold(f32::INFINITY,f32::min);
            assert!(floor.abs()<0.00001,"{} is not planted: {floor}",paw.name);
        }
        let mut pose=vec![Transform::IDENTITY;rig.joints.len()];
        let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
        for tick in 0..64 {
            rig.sample_locomotion(0.2,tick as f32*TAU/64.,1.,&mut pose);
            gait::assert_supports(rig,&pose);
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            let floor=(0..rig.neutral.positions.len()).map(|v|rig.skin_point(v,&matrices).y)
                .fold(f32::INFINITY,f32::min);
            assert!(floor>=-0.00001 && floor<height*0.01,"Furret lost floor contact: {floor}");
        }
    }
}
