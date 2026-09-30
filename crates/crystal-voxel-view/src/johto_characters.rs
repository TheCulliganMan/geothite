//! Original authored rigid-limb character assets. Joint pivots survive export;
//! vertex normals remain smooth rather than being flattened triangle-by-triangle.
use std::sync::OnceLock;

use bevy::prelude::{Transform, Vec3};
use serde::Deserialize;

use crate::mesh::SurfaceMeshData;

pub(super) const JOINT_COUNT: usize = 16;
pub(super) const PELVIS: usize = 0;
pub(super) const TORSO: usize = 1;
pub(super) const HEAD: usize = 2;
pub(super) const EYES: usize = 3;
pub(super) const UPPER_ARM_L: usize = 4;
pub(super) const FOREARM_L: usize = 5;
pub(super) const HAND_L: usize = 6;
pub(super) const UPPER_ARM_R: usize = 7;
pub(super) const FOREARM_R: usize = 8;
pub(super) const HAND_R: usize = 9;
pub(super) const THIGH_L: usize = 10;
pub(super) const SHIN_L: usize = 11;
pub(super) const SHOE_L: usize = 12;
pub(super) const THIGH_R: usize = 13;
pub(super) const SHIN_R: usize = 14;
pub(super) const SHOE_R: usize = 15;
const NAMES: [&str; JOINT_COUNT] = [
    "pelvis",
    "torso",
    "head",
    "eyes",
    "upper_arm_l",
    "forearm_l",
    "hand_l",
    "upper_arm_r",
    "forearm_r",
    "hand_r",
    "thigh_l",
    "shin_l",
    "shoe_l",
    "thigh_r",
    "shin_r",
    "shoe_r",
];
const PARENTS: [Option<usize>; JOINT_COUNT] = [
    None,
    Some(PELVIS),
    Some(TORSO),
    Some(HEAD),
    Some(TORSO),
    Some(UPPER_ARM_L),
    Some(FOREARM_L),
    Some(TORSO),
    Some(UPPER_ARM_R),
    Some(FOREARM_R),
    Some(PELVIS),
    Some(THIGH_L),
    Some(SHIN_L),
    Some(PELVIS),
    Some(THIGH_R),
    Some(SHIN_R),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum CharacterKind {
    Trainer,
    TrainerFemale,
    Rival,
    Youngster,
    Teacher,
    Lass,
    Scientist,
    Outdoorsman,
    Elder,
}

pub(super) const KINDS: [CharacterKind; 9] = [
    CharacterKind::Trainer,
    CharacterKind::TrainerFemale,
    CharacterKind::Rival,
    CharacterKind::Youngster,
    CharacterKind::Teacher,
    CharacterKind::Lass,
    CharacterKind::Scientist,
    CharacterKind::Outdoorsman,
    CharacterKind::Elder,
];

#[derive(Deserialize)]
struct Primitive {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct JointExport {
    name: String,
    parent: Option<usize>,
    translation: [f32; 3],
    primitives: Vec<Primitive>,
}
#[derive(Deserialize)]
struct RigExport {
    version: u32,
    joints: Vec<JointExport>,
}

pub(super) struct Joint {
    pub parent: Option<usize>,
    pub bind: Transform,
    pub mesh: SurfaceMeshData,
}
pub(super) struct CharacterRig {
    pub joints: [Joint; JOINT_COUNT],
}

impl CharacterRig {
    fn parse(json: &str) -> Result<Self, String> {
        let export: RigExport = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if export.version != 1 || export.joints.len() != JOINT_COUNT {
            return Err("unsupported articulated character schema".into());
        }
        let mut joints = Vec::with_capacity(JOINT_COUNT);
        for (index, source) in export.joints.into_iter().enumerate() {
            if source.name != NAMES[index]
                || source.parent != PARENTS[index]
                || source.translation.iter().any(|v| !v.is_finite())
            {
                return Err("invalid character joint or parent".into());
            }
            let mut surface = SurfaceMeshData::default();
            for p in source.primitives {
                let count = p.positions.len() / 3;
                if count == 0
                    || p.positions.len() % 3 != 0
                    || p.normals.len() != p.positions.len()
                    || p.indices.is_empty()
                    || p.indices.len() % 3 != 0
                    || p.indices.iter().any(|&i| i as usize >= count)
                    || p.positions.iter().chain(&p.normals).any(|v| !v.is_finite())
                    || p.base_color
                        .iter()
                        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                {
                    return Err("invalid articulated mesh primitive".into());
                }
                let base = surface.positions.len() as u32;
                surface
                    .positions
                    .extend(p.positions.chunks_exact(3).map(|p| [p[0], p[1], p[2]]));
                for n in p.normals.chunks_exact(3) {
                    let normal = Vec3::new(n[0], n[1], n[2]);
                    if normal.length_squared() < 0.5 {
                        return Err("invalid character normal".into());
                    }
                    surface.normals.push(normal.normalize().to_array());
                }
                surface.uvs.extend(std::iter::repeat_n([0.0, 0.0], count));
                surface
                    .colors
                    .extend(std::iter::repeat_n(p.base_color, count));
                surface
                    .indices
                    .extend(p.indices.into_iter().map(|i| base + i));
            }
            if surface.indices.is_empty() {
                return Err("character joint must own real geometry".into());
            }
            joints.push(Joint {
                parent: source.parent,
                bind: Transform::from_translation(Vec3::from_array(source.translation)),
                mesh: surface,
            });
        }
        Ok(Self {
            joints: joints.try_into().map_err(|_| "invalid joint count")?,
        })
    }
}

pub(super) fn rig(kind: CharacterKind) -> &'static CharacterRig {
    static RIGS: OnceLock<[CharacterRig; 9]> = OnceLock::new();
    let rigs = RIGS.get_or_init(|| {
        [
            include_str!("../models/johto_characters/trainer.rig.json"),
            include_str!("../models/johto_characters/trainer_female.rig.json"),
            include_str!("../models/johto_characters/rival.rig.json"),
            include_str!("../models/johto_characters/youngster.rig.json"),
            include_str!("../models/johto_characters/teacher.rig.json"),
            include_str!("../models/johto_characters/lass.rig.json"),
            include_str!("../models/johto_characters/scientist.rig.json"),
            include_str!("../models/johto_characters/outdoorsman.rig.json"),
            include_str!("../models/johto_characters/elder.rig.json"),
        ]
        .map(|json| CharacterRig::parse(json).expect("authored character rig must be valid"))
    });
    &rigs[KINDS.iter().position(|&k| k == kind).unwrap()]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_character_has_real_articulated_geometry_and_valid_normals() {
        for kind in KINDS {
            let model = rig(kind);
            let mut world = [Vec3::ZERO; JOINT_COUNT];
            for (i, joint) in model.joints.iter().enumerate() {
                world[i] =
                    joint.bind.translation + joint.parent.map(|p| world[p]).unwrap_or(Vec3::ZERO);
                assert!(joint.mesh.indices.len() >= 3);
                assert_eq!(joint.mesh.positions.len(), joint.mesh.normals.len());
                assert_eq!(joint.mesh.positions.len(), joint.mesh.colors.len());
                for normal in &joint.mesh.normals {
                    assert!((Vec3::from_array(*normal).length() - 1.0).abs() < 0.001);
                }
                for &index in &joint.mesh.indices {
                    assert!((index as usize) < joint.mesh.positions.len());
                }
            }
            // Asset bind locations and runtime two-link foot solver agree.
            for (thigh, shin, shoe) in [(THIGH_L, SHIN_L, SHOE_L), (THIGH_R, SHIN_R, SHOE_R)] {
                assert!((world[thigh].y - 0.66).abs() < 0.0001);
                assert!((world[shin].y - 0.39).abs() < 0.0001);
                assert!((world[shoe].y - 0.12).abs() < 0.0001);
                let floor = model.joints[shoe]
                    .mesh
                    .positions
                    .iter()
                    .map(|p| p[1] + world[shoe].y)
                    .fold(f32::INFINITY, f32::min);
                assert!((0.0..0.006).contains(&floor), "{kind:?} floor: {floor}");
            }
        }
    }
    #[test]
    fn rig_parser_rejects_missing_or_cyclic_joints() {
        assert!(CharacterRig::parse(r#"{"version":1,"joints":[]}"#).is_err());
        let bad = include_str!("../models/johto_characters/trainer.rig.json").replacen(
            "\"parent\":null",
            "\"parent\":0",
            1,
        );
        assert!(CharacterRig::parse(&bad).is_err());
    }
}
