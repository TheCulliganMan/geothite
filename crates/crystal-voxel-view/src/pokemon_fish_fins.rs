//! Complete world-fish appendages and cached folds for resting on a flank.
use super::*;

pub(super) fn build(rig: &mut SpeciesRig) {
    let tail=rig.joints.iter().position(|j|j.name.ends_with("/tail")).unwrap();
    for part in &rig.anatomy {
        let category=part.name.split(" / ").next().unwrap();
        let owner=if category.starts_with("Tail") { Some(tail) }
        else if category.starts_with("Pectoral") || category.starts_with("Whisker")
            || category.starts_with("Pelvic") {
            let side=part.name.split_ascii_whitespace().find_map(|word|word.parse::<i32>().ok());
            let prefix=if category.starts_with("Pectoral") {"pectoral"}
                else if category.starts_with("Whisker") {"barbel"} else {"pelvic"};
            let suffix=format!("/{prefix}{}",side.map_or("",|s|if s<0 {"_left"}else{"_right"}));
            rig.joints.iter().position(|j|j.name.ends_with(&suffix))
        } else if category.starts_with("Fin ray") || category.starts_with("Fin paint")
            || category.starts_with("Fin spot") {
            // These authored paint/ray collections explicitly name their bone.
            Some(part.name.split_ascii_whitespace().last().unwrap().parse::<usize>().unwrap())
        } else { None };
        if let Some(owner)=owner {
            assert!(owner<rig.joints.len());
            for v in part.vertices.clone() {
                rig.joint_indices[v]=[owner as u16,0,0,0];
                rig.joint_weights[v]=[1.,0.,0.,0.];
            }
        }
    }
    if matches!(rig.species.name(),"goldeen"|"seaking") {
        // Their two broad, curved fan lobes cannot lie flat as one rigid
        // assembly. Separate hinges share the existing caudal attachment.
        let start=rig.joints.len();
        let pivot=rig.joints[tail].inverse_bind.inverse().transform_point3(Vec3::ZERO);
        for side in ["left","right"] {
            rig.joints.push(Joint {name:format!("{}/land_tail_lobe_{side}",rig.species.name()),
                parent:Some(tail),bind:Transform::IDENTITY,
                inverse_bind:rig.joints[tail].inverse_bind});
        }
        for v in 0..rig.neutral.positions.len() {
            let belongs=rig.joint_indices[v].iter().zip(rig.joint_weights[v])
                .any(|(&owner,w)|owner as usize==tail && w>0.5);
            if belongs {
                let side=usize::from(rig.neutral.positions[v][0]>=pivot.x);
                rig.joint_indices[v]=[(start+side) as u16,0,0,0];
                rig.joint_weights[v]=[1.,0.,0.,0.];
            }
        }
    }
    for (joint,bone) in rig.joints.iter().enumerate() {
        let label=bone.name.rsplit('/').next().unwrap();
        let lobe=label.starts_with("land_tail_lobe_");
        if !(lobe || label.starts_with("pectoral_") || label.starts_with("barbel_")
            || label.starts_with("pelvic")) {continue;}
        let pivot=bone.inverse_bind.inverse().transform_point3(Vec3::ZERO);
        let points=(0..rig.neutral.positions.len()).filter(|&v|
            rig.joint_indices[v].iter().zip(rig.joint_weights[v])
                .any(|(&owner,w)|owner as usize==joint && w>0.5))
            .map(|v|Vec3::from_array(rig.neutral.positions[v])-pivot).collect::<Vec<_>>();
        assert!(!points.is_empty(),"authored fish appendage surface");
        let span=if lobe {points.iter().copied().sum::<Vec3>()/points.len() as f32}
            else {*points.iter().max_by(|a,b|a.length_squared().total_cmp(&b.length_squared())).unwrap()};
        let trailing=Quat::from_rotation_arc(span.normalize(),Vec3::NEG_Z);
        let mut best=(f32::INFINITY,trailing);
        // Roll the folded membrane onto its thinnest flank. This is a one-time
        // measurement of the actual fan, not a per-frame geometry rewrite.
        for tick in 0..256 {
            let rotation=Quat::from_rotation_z(TAU*tick as f32/256.)*trailing;
            let (lo,hi)=points.iter().map(|p|(rotation* *p).x)
                .fold((f32::INFINITY,f32::NEG_INFINITY),|(lo,hi),x|(lo.min(x),hi.max(x)));
            if hi-lo<best.0 {best=(hi-lo,rotation);}
        }
        let parent=bone.parent.map_or(Quat::IDENTITY,|p|
            rig.joints[p].inverse_bind.inverse().to_scale_rotation_translation().1);
        rig.flop_folds.push((joint,parent.inverse()*best.1*parent));
    }
}
