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

/// One authored look per resolved human source. Never use object script names,
/// guessed species, or palette similarity to choose a character.
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
    Beauty,
    Biker,
    Bill,
    BlackBelt,
    Blaine,
    Blue,
    Brock,
    Bruno,
    BugCatcher,
    Bugsy,
    Cal,
    Captain,
    Chuck,
    Clair,
    Clerk,
    CooltrainerF,
    CooltrainerM,
    Daisy,
    Elm,
    Erika,
    Falkner,
    Fisher,
    FishingGuru,
    GameboyKid,
    Gentleman,
    Gramps,
    Granny,
    GymGuide,
    Janine,
    Jasmine,
    Karen,
    KimonoGirl,
    Koga,
    Kurt,
    KurtOutside,
    Lance,
    LinkReceptionist,
    Misty,
    Mom,
    Morty,
    Nurse,
    Oak,
    Officer,
    OldLinkReceptionist,
    Pharmacist,
    PokefanF,
    PokefanM,
    Pryce,
    Receptionist,
    Red,
    RedsMom,
    Rocker,
    Rocket,
    RocketGirl,
    Sabrina,
    Sage,
    Sailor,
    StandingYoungster,
    SuperNerd,
    Surge,
    SwimmerGirl,
    SwimmerGuy,
    Twin,
    UnusedGuy,
    Whitney,
    Will,
}

#[cfg(test)]
pub(super) const KINDS: [CharacterKind; 75] = [
    CharacterKind::Trainer,
    CharacterKind::TrainerFemale,
    CharacterKind::Rival,
    CharacterKind::Youngster,
    CharacterKind::Teacher,
    CharacterKind::Lass,
    CharacterKind::Scientist,
    CharacterKind::Outdoorsman,
    CharacterKind::Elder,
    CharacterKind::Beauty,
    CharacterKind::Biker,
    CharacterKind::Bill,
    CharacterKind::BlackBelt,
    CharacterKind::Blaine,
    CharacterKind::Blue,
    CharacterKind::Brock,
    CharacterKind::Bruno,
    CharacterKind::BugCatcher,
    CharacterKind::Bugsy,
    CharacterKind::Cal,
    CharacterKind::Captain,
    CharacterKind::Chuck,
    CharacterKind::Clair,
    CharacterKind::Clerk,
    CharacterKind::CooltrainerF,
    CharacterKind::CooltrainerM,
    CharacterKind::Daisy,
    CharacterKind::Elm,
    CharacterKind::Erika,
    CharacterKind::Falkner,
    CharacterKind::Fisher,
    CharacterKind::FishingGuru,
    CharacterKind::GameboyKid,
    CharacterKind::Gentleman,
    CharacterKind::Gramps,
    CharacterKind::Granny,
    CharacterKind::GymGuide,
    CharacterKind::Janine,
    CharacterKind::Jasmine,
    CharacterKind::Karen,
    CharacterKind::KimonoGirl,
    CharacterKind::Koga,
    CharacterKind::Kurt,
    CharacterKind::KurtOutside,
    CharacterKind::Lance,
    CharacterKind::LinkReceptionist,
    CharacterKind::Misty,
    CharacterKind::Mom,
    CharacterKind::Morty,
    CharacterKind::Nurse,
    CharacterKind::Oak,
    CharacterKind::Officer,
    CharacterKind::OldLinkReceptionist,
    CharacterKind::Pharmacist,
    CharacterKind::PokefanF,
    CharacterKind::PokefanM,
    CharacterKind::Pryce,
    CharacterKind::Receptionist,
    CharacterKind::Red,
    CharacterKind::RedsMom,
    CharacterKind::Rocker,
    CharacterKind::Rocket,
    CharacterKind::RocketGirl,
    CharacterKind::Sabrina,
    CharacterKind::Sage,
    CharacterKind::Sailor,
    CharacterKind::StandingYoungster,
    CharacterKind::SuperNerd,
    CharacterKind::Surge,
    CharacterKind::SwimmerGirl,
    CharacterKind::SwimmerGuy,
    CharacterKind::Twin,
    CharacterKind::UnusedGuy,
    CharacterKind::Whitney,
    CharacterKind::Will,
];

/// Canonical host-published sprite family identifiers. Different source families
/// retain different costumes, hair, props and silhouettes even when roles match.
pub(super) const SOURCE_KINDS: [(&str, CharacterKind); 74] = [
    ("beauty", CharacterKind::Beauty),
    ("biker", CharacterKind::Biker),
    ("bill", CharacterKind::Bill),
    ("black_belt", CharacterKind::BlackBelt),
    ("blaine", CharacterKind::Blaine),
    ("blue", CharacterKind::Blue),
    ("brock", CharacterKind::Brock),
    ("bruno", CharacterKind::Bruno),
    ("bug_catcher", CharacterKind::BugCatcher),
    ("bugsy", CharacterKind::Bugsy),
    ("cal", CharacterKind::Cal),
    ("captain", CharacterKind::Captain),
    ("chris", CharacterKind::Trainer),
    ("chuck", CharacterKind::Chuck),
    ("clair", CharacterKind::Clair),
    ("clerk", CharacterKind::Clerk),
    ("cooltrainer_f", CharacterKind::CooltrainerF),
    ("cooltrainer_m", CharacterKind::CooltrainerM),
    ("daisy", CharacterKind::Daisy),
    ("elder", CharacterKind::Elder),
    ("elm", CharacterKind::Elm),
    ("erika", CharacterKind::Erika),
    ("falkner", CharacterKind::Falkner),
    ("fisher", CharacterKind::Fisher),
    ("fishing_guru", CharacterKind::FishingGuru),
    ("gameboy_kid", CharacterKind::GameboyKid),
    ("gentleman", CharacterKind::Gentleman),
    ("gramps", CharacterKind::Gramps),
    ("granny", CharacterKind::Granny),
    ("gym_guide", CharacterKind::GymGuide),
    ("janine", CharacterKind::Janine),
    ("jasmine", CharacterKind::Jasmine),
    ("karen", CharacterKind::Karen),
    ("kimono_girl", CharacterKind::KimonoGirl),
    ("koga", CharacterKind::Koga),
    ("kris", CharacterKind::TrainerFemale),
    ("kurt", CharacterKind::Kurt),
    ("kurt_outside", CharacterKind::KurtOutside),
    ("lance", CharacterKind::Lance),
    ("lass", CharacterKind::Lass),
    ("link_receptionist", CharacterKind::LinkReceptionist),
    ("misty", CharacterKind::Misty),
    ("mom", CharacterKind::Mom),
    ("morty", CharacterKind::Morty),
    ("nurse", CharacterKind::Nurse),
    ("oak", CharacterKind::Oak),
    ("officer", CharacterKind::Officer),
    ("old_link_receptionist", CharacterKind::OldLinkReceptionist),
    ("pharmacist", CharacterKind::Pharmacist),
    ("pokefan_f", CharacterKind::PokefanF),
    ("pokefan_m", CharacterKind::PokefanM),
    ("pryce", CharacterKind::Pryce),
    ("receptionist", CharacterKind::Receptionist),
    ("red", CharacterKind::Red),
    ("reds_mom", CharacterKind::RedsMom),
    ("rival", CharacterKind::Rival),
    ("rocker", CharacterKind::Rocker),
    ("rocket", CharacterKind::Rocket),
    ("rocket_girl", CharacterKind::RocketGirl),
    ("sabrina", CharacterKind::Sabrina),
    ("sage", CharacterKind::Sage),
    ("sailor", CharacterKind::Sailor),
    ("scientist", CharacterKind::Scientist),
    ("standing_youngster", CharacterKind::StandingYoungster),
    ("super_nerd", CharacterKind::SuperNerd),
    ("surge", CharacterKind::Surge),
    ("swimmer_girl", CharacterKind::SwimmerGirl),
    ("swimmer_guy", CharacterKind::SwimmerGuy),
    ("teacher", CharacterKind::Teacher),
    ("twin", CharacterKind::Twin),
    ("unused_guy", CharacterKind::UnusedGuy),
    ("whitney", CharacterKind::Whitney),
    ("will", CharacterKind::Will),
    ("youngster", CharacterKind::Youngster),
];

pub(super) fn kind_for_source(source: &str) -> Option<CharacterKind> {
    SOURCE_KINDS
        .iter()
        .find_map(|&(name, kind)| (name == source).then_some(kind))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Geometry {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometryLibrary {
    version: u32,
    id: String,
    geometries: Vec<Geometry>,
}

fn validate_geometry(positions: &[f32], normals: &[f32], indices: &[u32]) -> Result<(), String> {
    let count = positions.len() / 3;
    if count == 0
        || positions.len() % 3 != 0
        || normals.len() != positions.len()
        || indices.is_empty()
        || indices.len() % 3 != 0
        || indices.iter().any(|&i| i as usize >= count)
        || positions.iter().chain(normals).any(|v| !v.is_finite())
    {
        return Err("invalid articulated mesh geometry".into());
    }
    for n in normals.chunks_exact(3) {
        if Vec3::new(n[0], n[1], n[2]).length_squared() < 0.5 {
            return Err("invalid character normal".into());
        }
    }
    Ok(())
}

impl GeometryLibrary {
    fn parse<'a>(json: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let library: Self = crate::model_storage::parse(json)?;
        if library.version != 1
            || library.id.len() != 64
            || !library
                .id
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            || library.geometries.is_empty()
        {
            return Err("unsupported character geometry library".into());
        }
        for geometry in &library.geometries {
            validate_geometry(&geometry.positions, &geometry.normals, &geometry.indices)?;
        }
        Ok(library)
    }
}

fn geometry_library() -> &'static GeometryLibrary {
    static LIBRARY: OnceLock<GeometryLibrary> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        GeometryLibrary::parse(crate::model_storage::include_model!(
            "models/johto_characters/shared.geometry.json"
        ))
        .expect("authored shared character geometry must be valid")
    })
}

// Optional fields keep direct f32 deserialization identical to the original
// inline schema. Version checks below reject mixed inline/reference primitives.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Primitive {
    positions: Option<Vec<f32>>,
    normals: Option<Vec<f32>>,
    indices: Option<Vec<u32>>,
    geometry: Option<usize>,
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
    geometry_library: Option<String>,
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
    fn parse<'a>(json: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let export: RigExport = crate::model_storage::parse(json)?;
        let library = (export.version == 2).then(geometry_library);
        Self::from_export(export, library)
    }

    fn from_export(export: RigExport, library: Option<&GeometryLibrary>) -> Result<Self, String> {
        if !matches!(export.version, 1 | 2) || export.joints.len() != JOINT_COUNT {
            return Err("unsupported articulated character schema".into());
        }
        match (export.version, export.geometry_library.as_deref(), library) {
            (1, None, _) => (),
            (2, Some(id), Some(library)) if id == library.id.as_str() => (),
            _ => return Err("character geometry library identity mismatch".into()),
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
            for primitive in source.primitives {
                let (positions, normals, indices) = match (
                    export.version,
                    &primitive.positions,
                    &primitive.normals,
                    &primitive.indices,
                    primitive.geometry,
                ) {
                    (1, Some(positions), Some(normals), Some(indices), None) => {
                        validate_geometry(positions, normals, indices)?;
                        (positions, normals, indices)
                    }
                    (2, None, None, None, Some(index)) => {
                        let geometry = library
                            .and_then(|library| library.geometries.get(index))
                            .ok_or_else(|| "invalid character geometry reference".to_string())?;
                        (&geometry.positions, &geometry.normals, &geometry.indices)
                    }
                    _ => return Err("primitive does not match character schema version".into()),
                };
                let color = primitive.base_color;
                if color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                {
                    return Err("invalid articulated mesh material".into());
                }
                let count = positions.len() / 3;
                let base = surface.positions.len() as u32;
                surface
                    .positions
                    .extend(positions.chunks_exact(3).map(|p| [p[0], p[1], p[2]]));
                for n in normals.chunks_exact(3) {
                    // Keep exactly the v1 normalization path. Shared storage
                    // changes no resulting vertex, normal, color, or index bit.
                    let normal = Vec3::new(n[0], n[1], n[2]);
                    surface.normals.push(normal.normalize().to_array());
                }
                surface.uvs.extend(std::iter::repeat_n([0.0, 0.0], count));
                surface.colors.extend(std::iter::repeat_n(color, count));
                surface.indices.extend(indices.iter().map(|&i| base + i));
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

/// A family is decoded only when it is used. A new town must not eagerly parse
/// all 75 detailed rigs; immutable geometry is decoded once, and each assembled
/// family has its own bounded process-lifetime cache.
pub(super) fn rig(kind: CharacterKind) -> &'static CharacterRig {
    match kind {
        CharacterKind::Trainer => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/trainer.rig.json"
                ))
                .expect("authored trainer rig must be valid")
            })
        }
        CharacterKind::TrainerFemale => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/trainer_female.rig.json"
                ))
                .expect("authored trainer_female rig must be valid")
            })
        }
        CharacterKind::Rival => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/rival.rig.json"
                ))
                .expect("authored rival rig must be valid")
            })
        }
        CharacterKind::Youngster => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/youngster.rig.json"
                ))
                .expect("authored youngster rig must be valid")
            })
        }
        CharacterKind::Teacher => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/teacher.rig.json"
                ))
                .expect("authored teacher rig must be valid")
            })
        }
        CharacterKind::Lass => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/lass.rig.json"
                ))
                .expect("authored lass rig must be valid")
            })
        }
        CharacterKind::Scientist => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/scientist.rig.json"
                ))
                .expect("authored scientist rig must be valid")
            })
        }
        CharacterKind::Outdoorsman => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/outdoorsman.rig.json"
                ))
                .expect("authored outdoorsman rig must be valid")
            })
        }
        CharacterKind::Elder => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/elder.rig.json"
                ))
                .expect("authored elder rig must be valid")
            })
        }
        CharacterKind::Beauty => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/beauty.rig.json"
                ))
                .expect("authored beauty rig must be valid")
            })
        }
        CharacterKind::Biker => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/biker.rig.json"
                ))
                .expect("authored biker rig must be valid")
            })
        }
        CharacterKind::Bill => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/bill.rig.json"
                ))
                .expect("authored bill rig must be valid")
            })
        }
        CharacterKind::BlackBelt => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/black_belt.rig.json"
                ))
                .expect("authored black_belt rig must be valid")
            })
        }
        CharacterKind::Blaine => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/blaine.rig.json"
                ))
                .expect("authored blaine rig must be valid")
            })
        }
        CharacterKind::Blue => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/blue.rig.json"
                ))
                .expect("authored blue rig must be valid")
            })
        }
        CharacterKind::Brock => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/brock.rig.json"
                ))
                .expect("authored brock rig must be valid")
            })
        }
        CharacterKind::Bruno => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/bruno.rig.json"
                ))
                .expect("authored bruno rig must be valid")
            })
        }
        CharacterKind::BugCatcher => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/bug_catcher.rig.json"
                ))
                .expect("authored bug_catcher rig must be valid")
            })
        }
        CharacterKind::Bugsy => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/bugsy.rig.json"
                ))
                .expect("authored bugsy rig must be valid")
            })
        }
        CharacterKind::Cal => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/cal.rig.json"
                ))
                .expect("authored cal rig must be valid")
            })
        }
        CharacterKind::Captain => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/captain.rig.json"
                ))
                .expect("authored captain rig must be valid")
            })
        }
        CharacterKind::Chuck => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/chuck.rig.json"
                ))
                .expect("authored chuck rig must be valid")
            })
        }
        CharacterKind::Clair => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/clair.rig.json"
                ))
                .expect("authored clair rig must be valid")
            })
        }
        CharacterKind::Clerk => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/clerk.rig.json"
                ))
                .expect("authored clerk rig must be valid")
            })
        }
        CharacterKind::CooltrainerF => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/cooltrainer_f.rig.json"
                ))
                .expect("authored cooltrainer_f rig must be valid")
            })
        }
        CharacterKind::CooltrainerM => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/cooltrainer_m.rig.json"
                ))
                .expect("authored cooltrainer_m rig must be valid")
            })
        }
        CharacterKind::Daisy => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/daisy.rig.json"
                ))
                .expect("authored daisy rig must be valid")
            })
        }
        CharacterKind::Elm => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/elm.rig.json"
                ))
                .expect("authored elm rig must be valid")
            })
        }
        CharacterKind::Erika => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/erika.rig.json"
                ))
                .expect("authored erika rig must be valid")
            })
        }
        CharacterKind::Falkner => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/falkner.rig.json"
                ))
                .expect("authored falkner rig must be valid")
            })
        }
        CharacterKind::Fisher => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/fisher.rig.json"
                ))
                .expect("authored fisher rig must be valid")
            })
        }
        CharacterKind::FishingGuru => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/fishing_guru.rig.json"
                ))
                .expect("authored fishing_guru rig must be valid")
            })
        }
        CharacterKind::GameboyKid => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/gameboy_kid.rig.json"
                ))
                .expect("authored gameboy_kid rig must be valid")
            })
        }
        CharacterKind::Gentleman => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/gentleman.rig.json"
                ))
                .expect("authored gentleman rig must be valid")
            })
        }
        CharacterKind::Gramps => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/gramps.rig.json"
                ))
                .expect("authored gramps rig must be valid")
            })
        }
        CharacterKind::Granny => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/granny.rig.json"
                ))
                .expect("authored granny rig must be valid")
            })
        }
        CharacterKind::GymGuide => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/gym_guide.rig.json"
                ))
                .expect("authored gym_guide rig must be valid")
            })
        }
        CharacterKind::Janine => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/janine.rig.json"
                ))
                .expect("authored janine rig must be valid")
            })
        }
        CharacterKind::Jasmine => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/jasmine.rig.json"
                ))
                .expect("authored jasmine rig must be valid")
            })
        }
        CharacterKind::Karen => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/karen.rig.json"
                ))
                .expect("authored karen rig must be valid")
            })
        }
        CharacterKind::KimonoGirl => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/kimono_girl.rig.json"
                ))
                .expect("authored kimono_girl rig must be valid")
            })
        }
        CharacterKind::Koga => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/koga.rig.json"
                ))
                .expect("authored koga rig must be valid")
            })
        }
        CharacterKind::Kurt => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/kurt.rig.json"
                ))
                .expect("authored kurt rig must be valid")
            })
        }
        CharacterKind::KurtOutside => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/kurt_outside.rig.json"
                ))
                .expect("authored kurt_outside rig must be valid")
            })
        }
        CharacterKind::Lance => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/lance.rig.json"
                ))
                .expect("authored lance rig must be valid")
            })
        }
        CharacterKind::LinkReceptionist => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/link_receptionist.rig.json"
                ))
                .expect("authored link_receptionist rig must be valid")
            })
        }
        CharacterKind::Misty => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/misty.rig.json"
                ))
                .expect("authored misty rig must be valid")
            })
        }
        CharacterKind::Mom => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/mom.rig.json"
                ))
                .expect("authored mom rig must be valid")
            })
        }
        CharacterKind::Morty => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/morty.rig.json"
                ))
                .expect("authored morty rig must be valid")
            })
        }
        CharacterKind::Nurse => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/nurse.rig.json"
                ))
                .expect("authored nurse rig must be valid")
            })
        }
        CharacterKind::Oak => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/oak.rig.json"
                ))
                .expect("authored oak rig must be valid")
            })
        }
        CharacterKind::Officer => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/officer.rig.json"
                ))
                .expect("authored officer rig must be valid")
            })
        }
        CharacterKind::OldLinkReceptionist => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/old_link_receptionist.rig.json"
                ))
                .expect("authored old_link_receptionist rig must be valid")
            })
        }
        CharacterKind::Pharmacist => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/pharmacist.rig.json"
                ))
                .expect("authored pharmacist rig must be valid")
            })
        }
        CharacterKind::PokefanF => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/pokefan_f.rig.json"
                ))
                .expect("authored pokefan_f rig must be valid")
            })
        }
        CharacterKind::PokefanM => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/pokefan_m.rig.json"
                ))
                .expect("authored pokefan_m rig must be valid")
            })
        }
        CharacterKind::Pryce => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/pryce.rig.json"
                ))
                .expect("authored pryce rig must be valid")
            })
        }
        CharacterKind::Receptionist => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/receptionist.rig.json"
                ))
                .expect("authored receptionist rig must be valid")
            })
        }
        CharacterKind::Red => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/red.rig.json"
                ))
                .expect("authored red rig must be valid")
            })
        }
        CharacterKind::RedsMom => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/reds_mom.rig.json"
                ))
                .expect("authored reds_mom rig must be valid")
            })
        }
        CharacterKind::Rocker => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/rocker.rig.json"
                ))
                .expect("authored rocker rig must be valid")
            })
        }
        CharacterKind::Rocket => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/rocket.rig.json"
                ))
                .expect("authored rocket rig must be valid")
            })
        }
        CharacterKind::RocketGirl => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/rocket_girl.rig.json"
                ))
                .expect("authored rocket_girl rig must be valid")
            })
        }
        CharacterKind::Sabrina => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/sabrina.rig.json"
                ))
                .expect("authored sabrina rig must be valid")
            })
        }
        CharacterKind::Sage => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/sage.rig.json"
                ))
                .expect("authored sage rig must be valid")
            })
        }
        CharacterKind::Sailor => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/sailor.rig.json"
                ))
                .expect("authored sailor rig must be valid")
            })
        }
        CharacterKind::StandingYoungster => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/standing_youngster.rig.json"
                ))
                .expect("authored standing_youngster rig must be valid")
            })
        }
        CharacterKind::SuperNerd => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/super_nerd.rig.json"
                ))
                .expect("authored super_nerd rig must be valid")
            })
        }
        CharacterKind::Surge => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/surge.rig.json"
                ))
                .expect("authored surge rig must be valid")
            })
        }
        CharacterKind::SwimmerGirl => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/swimmer_girl.rig.json"
                ))
                .expect("authored swimmer_girl rig must be valid")
            })
        }
        CharacterKind::SwimmerGuy => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/swimmer_guy.rig.json"
                ))
                .expect("authored swimmer_guy rig must be valid")
            })
        }
        CharacterKind::Twin => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/twin.rig.json"
                ))
                .expect("authored twin rig must be valid")
            })
        }
        CharacterKind::UnusedGuy => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/unused_guy.rig.json"
                ))
                .expect("authored unused_guy rig must be valid")
            })
        }
        CharacterKind::Whitney => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/whitney.rig.json"
                ))
                .expect("authored whitney rig must be valid")
            })
        }
        CharacterKind::Will => {
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                CharacterRig::parse(crate::model_storage::include_model!(
                    "models/johto_characters/will.rig.json"
                ))
                .expect("authored will rig must be valid")
            })
        }
    }
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
    fn human_sources_keep_exact_identity_instead_of_role_aliases() {
        use std::collections::HashSet;
        let mut source_names = HashSet::new();
        let mut source_kinds = HashSet::new();
        for (source, kind) in SOURCE_KINDS {
            assert!(source_names.insert(source), "duplicate source {source}");
            assert!(
                source_kinds.insert(kind),
                "source {source} collapses into another family"
            );
            assert_eq!(kind_for_source(source), Some(kind));
        }
        assert_eq!(source_names.len(), 74);
        for unknown in [
            "remote_player",
            "surf",
            "chris_bike",
            "kris_bike",
            "nurse_hat",
            "NURSE",
            "sprite_nurse",
            "lyra",
            "schoolboy",
            "fisherman",
            "",
        ] {
            assert_eq!(kind_for_source(unknown), None, "must not guess {unknown}");
        }
        assert_eq!(kind_for_source("chris"), Some(CharacterKind::Trainer));
        assert_eq!(kind_for_source("kris"), Some(CharacterKind::TrainerFemale));
        assert_ne!(kind_for_source("nurse"), kind_for_source("scientist"));
        assert_ne!(kind_for_source("mom"), kind_for_source("granny"));
        assert_ne!(kind_for_source("fisher"), kind_for_source("sailor"));
        assert_ne!(kind_for_source("rocket"), kind_for_source("officer"));
        assert_ne!(kind_for_source("kurt"), kind_for_source("kurt_outside"));
    }

    #[test]
    fn shared_rig_and_legacy_rig_produce_identical_runtime_bits() {
        let compact_json =
            crate::model_storage::include_model!("models/johto_characters/trainer.rig.json");
        let library_json =
            crate::model_storage::include_model!("models/johto_characters/shared.geometry.json");
        let mut expanded: serde_json::Value = crate::model_storage::parse(compact_json).unwrap();
        let library: serde_json::Value = crate::model_storage::parse(library_json).unwrap();
        expanded["version"] = serde_json::json!(1);
        expanded.as_object_mut().unwrap().remove("geometry_library");
        for joint in expanded["joints"].as_array_mut().unwrap() {
            for primitive in joint["primitives"].as_array_mut().unwrap() {
                let color = primitive["base_color"].clone();
                let index = primitive["geometry"].as_u64().unwrap() as usize;
                *primitive = library["geometries"][index].clone();
                primitive
                    .as_object_mut()
                    .unwrap()
                    .insert("base_color".into(), color);
            }
        }
        let compact = CharacterRig::parse(compact_json).unwrap();
        let inline = CharacterRig::parse(&serde_json::to_string(&expanded).unwrap()).unwrap();
        for (a, b) in compact.joints.iter().zip(&inline.joints) {
            assert_eq!(a.parent, b.parent);
            assert_eq!(
                a.bind.translation.to_array().map(f32::to_bits),
                b.bind.translation.to_array().map(f32::to_bits)
            );
            for (a, b) in [&a.mesh.positions, &a.mesh.normals]
                .into_iter()
                .zip([&b.mesh.positions, &b.mesh.normals])
            {
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    assert_eq!(a.map(f32::to_bits), b.map(f32::to_bits));
                }
            }
            assert_eq!(a.mesh.colors.len(), b.mesh.colors.len());
            for (a, b) in a.mesh.colors.iter().zip(&b.mesh.colors) {
                assert_eq!(a.map(f32::to_bits), b.map(f32::to_bits));
            }
            assert_eq!(a.mesh.indices, b.mesh.indices);
            assert_eq!(a.mesh.uvs, b.mesh.uvs);
        }
    }

    #[test]
    fn shared_rig_rejects_invalid_references_and_library_mismatch() {
        let original: serde_json::Value = crate::model_storage::parse(
            crate::model_storage::include_model!("models/johto_characters/trainer.rig.json"),
        )
        .unwrap();
        let mut invalid = original.clone();
        invalid["joints"][0]["primitives"][0]["geometry"] = serde_json::json!(u64::MAX);
        assert!(CharacterRig::parse(&invalid.to_string()).is_err());
        invalid = original.clone();
        invalid["geometry_library"] = serde_json::json!("wrong library");
        assert!(CharacterRig::parse(&invalid.to_string()).is_err());
        invalid = original.clone();
        invalid.as_object_mut().unwrap().remove("geometry_library");
        assert!(CharacterRig::parse(&invalid.to_string()).is_err());
        invalid = original.clone();
        invalid["joints"][0]["primitives"][0]["positions"] = serde_json::json!([0.0, 0.0, 0.0]);
        assert!(CharacterRig::parse(&invalid.to_string()).is_err());
        invalid = original.clone();
        invalid["version"] = serde_json::json!(1);
        assert!(CharacterRig::parse(&invalid.to_string()).is_err());
        invalid = original;
        invalid["joints"][0]["primitives"][0]["base_color"] =
            serde_json::json!([2.0, 0.0, 0.0, 1.0]);
        assert!(CharacterRig::parse(&invalid.to_string()).is_err());
    }

    #[test]
    fn shared_geometry_library_rejects_invalid_meshes() {
        assert!(GeometryLibrary::parse(r#"{"version":1,"id":"bad","geometries":[]}"#).is_err());
        let mut library: serde_json::Value = crate::model_storage::parse(
            crate::model_storage::include_model!("models/johto_characters/shared.geometry.json"),
        )
        .unwrap();
        library["geometries"][0]["indices"][0] = serde_json::json!(u32::MAX);
        assert!(GeometryLibrary::parse(&library.to_string()).is_err());
    }

    #[test]
    fn rig_parser_rejects_missing_or_cyclic_joints() {
        assert!(CharacterRig::parse(r#"{"version":1,"joints":[]}"#).is_err());
        let mut bad: serde_json::Value = crate::model_storage::parse(
            crate::model_storage::include_model!("models/johto_characters/trainer.rig.json"),
        )
        .unwrap();
        assert!(bad["joints"][0]["parent"].is_null());
        bad["joints"][0]["parent"] = serde_json::json!(0);
        assert!(CharacterRig::parse(&bad.to_string()).is_err());
    }
}
