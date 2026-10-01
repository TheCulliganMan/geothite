//! Original authored rigid-limb character assets. Joint pivots survive export;
//! vertex normals remain smooth rather than being flattened triangle-by-triangle.
use std::sync::OnceLock;

use bevy::prelude::Transform;

#[path = "human_glb.rs"]
pub(super) mod human_glb;

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

pub(super) struct Joint {
    pub parent: Option<usize>,
    pub bind: Transform,
    pub mesh: SurfaceMeshData,
}

pub(super) struct CharacterRig {
    pub joints: [Joint; JOINT_COUNT],
    /// The original named glTF scene and animation data remain available. The
    /// gameplay controller selects locomotion clips; importing one never starts it.
    pub scene_name: String,
    pub animations: Vec<human_glb::NodeClip>,
    locomotion: Option<[usize; 2]>,
}

impl CharacterRig {
    /// Catalog loading resolves these exact scene-scoped names once. Gameplay
    /// never searches or allocates a clip name while posing a frame.
    pub fn locomotion_clip(&self, run: bool) -> &human_glb::NodeClip {
        &self.animations
            [self.locomotion.expect("human rig requires walk/run clips")[usize::from(run)]]
    }

    fn require_locomotion(self) -> Result<Self, String> {
        if self.locomotion.is_none() {
            return Err(format!(
                "human scene {} requires named walk/run clips",
                self.scene_name
            ));
        }
        for run in [false, true] {
            let clip = self.locomotion_clip(run);
            if clip.duration <= 0.0 {
                return Err(format!(
                    "human scene {} has a zero-duration locomotion clip",
                    self.scene_name
                ));
            }
            for joint in 0..JOINT_COUNT {
                if joint == EYES {
                    continue;
                }
                let present = clip.channels.iter().any(|channel| {
                    channel.joint == joint
                        && match (&channel.values, joint) {
                            (human_glb::NodeValues::Translations(_), PELVIS) => true,
                            (human_glb::NodeValues::Rotations(_), joint) if joint != PELVIS => true,
                            _ => false,
                        }
                });
                if !present {
                    return Err(format!(
                        "human scene {} locomotion clip is missing {} motion",
                        self.scene_name, NAMES[joint]
                    ));
                }
            }
        }
        Ok(self)
    }
}

fn catalog() -> &'static human_glb::Catalog {
    static CATALOG: OnceLock<human_glb::Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let bytes = crate::model_storage::decode_bytes(crate::model_storage::include_model!(
            "models/johto_characters/catalog.glb"
        ))
        .expect("authored human GLB storage must be valid");
        human_glb::Catalog::parse(&bytes).expect("authored human GLB catalog must be valid")
    })
}

/// Only the document and binary buffer are shared eagerly. Geometry and clip
/// samples are decoded once per used family, never instantiated for all scenes.
pub(super) fn rig(kind: CharacterKind) -> &'static CharacterRig {
    macro_rules! scene {
        ($name:literal) => {{
            static MODEL: OnceLock<CharacterRig> = OnceLock::new();
            MODEL.get_or_init(|| {
                catalog()
                    .rig($name)
                    .and_then(CharacterRig::require_locomotion)
                    .expect(concat!("invalid human scene: ", $name))
            })
        }};
    }
    match kind {
        CharacterKind::Trainer => scene!("trainer"),
        CharacterKind::TrainerFemale => scene!("trainer_female"),
        CharacterKind::Rival => scene!("rival"),
        CharacterKind::Youngster => scene!("youngster"),
        CharacterKind::Teacher => scene!("teacher"),
        CharacterKind::Lass => scene!("lass"),
        CharacterKind::Scientist => scene!("scientist"),
        CharacterKind::Outdoorsman => scene!("outdoorsman"),
        CharacterKind::Elder => scene!("elder"),
        CharacterKind::Beauty => scene!("beauty"),
        CharacterKind::Biker => scene!("biker"),
        CharacterKind::Bill => scene!("bill"),
        CharacterKind::BlackBelt => scene!("black_belt"),
        CharacterKind::Blaine => scene!("blaine"),
        CharacterKind::Blue => scene!("blue"),
        CharacterKind::Brock => scene!("brock"),
        CharacterKind::Bruno => scene!("bruno"),
        CharacterKind::BugCatcher => scene!("bug_catcher"),
        CharacterKind::Bugsy => scene!("bugsy"),
        CharacterKind::Cal => scene!("cal"),
        CharacterKind::Captain => scene!("captain"),
        CharacterKind::Chuck => scene!("chuck"),
        CharacterKind::Clair => scene!("clair"),
        CharacterKind::Clerk => scene!("clerk"),
        CharacterKind::CooltrainerF => scene!("cooltrainer_f"),
        CharacterKind::CooltrainerM => scene!("cooltrainer_m"),
        CharacterKind::Daisy => scene!("daisy"),
        CharacterKind::Elm => scene!("elm"),
        CharacterKind::Erika => scene!("erika"),
        CharacterKind::Falkner => scene!("falkner"),
        CharacterKind::Fisher => scene!("fisher"),
        CharacterKind::FishingGuru => scene!("fishing_guru"),
        CharacterKind::GameboyKid => scene!("gameboy_kid"),
        CharacterKind::Gentleman => scene!("gentleman"),
        CharacterKind::Gramps => scene!("gramps"),
        CharacterKind::Granny => scene!("granny"),
        CharacterKind::GymGuide => scene!("gym_guide"),
        CharacterKind::Janine => scene!("janine"),
        CharacterKind::Jasmine => scene!("jasmine"),
        CharacterKind::Karen => scene!("karen"),
        CharacterKind::KimonoGirl => scene!("kimono_girl"),
        CharacterKind::Koga => scene!("koga"),
        CharacterKind::Kurt => scene!("kurt"),
        CharacterKind::KurtOutside => scene!("kurt_outside"),
        CharacterKind::Lance => scene!("lance"),
        CharacterKind::LinkReceptionist => scene!("link_receptionist"),
        CharacterKind::Misty => scene!("misty"),
        CharacterKind::Mom => scene!("mom"),
        CharacterKind::Morty => scene!("morty"),
        CharacterKind::Nurse => scene!("nurse"),
        CharacterKind::Oak => scene!("oak"),
        CharacterKind::Officer => scene!("officer"),
        CharacterKind::OldLinkReceptionist => scene!("old_link_receptionist"),
        CharacterKind::Pharmacist => scene!("pharmacist"),
        CharacterKind::PokefanF => scene!("pokefan_f"),
        CharacterKind::PokefanM => scene!("pokefan_m"),
        CharacterKind::Pryce => scene!("pryce"),
        CharacterKind::Receptionist => scene!("receptionist"),
        CharacterKind::Red => scene!("red"),
        CharacterKind::RedsMom => scene!("reds_mom"),
        CharacterKind::Rocker => scene!("rocker"),
        CharacterKind::Rocket => scene!("rocket"),
        CharacterKind::RocketGirl => scene!("rocket_girl"),
        CharacterKind::Sabrina => scene!("sabrina"),
        CharacterKind::Sage => scene!("sage"),
        CharacterKind::Sailor => scene!("sailor"),
        CharacterKind::StandingYoungster => scene!("standing_youngster"),
        CharacterKind::SuperNerd => scene!("super_nerd"),
        CharacterKind::Surge => scene!("surge"),
        CharacterKind::SwimmerGirl => scene!("swimmer_girl"),
        CharacterKind::SwimmerGuy => scene!("swimmer_guy"),
        CharacterKind::Twin => scene!("twin"),
        CharacterKind::UnusedGuy => scene!("unused_guy"),
        CharacterKind::Whitney => scene!("whitney"),
        CharacterKind::Will => scene!("will"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;
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
    fn catalog_retains_every_named_scene_and_opt_in_locomotion_clip() {
        use std::collections::HashSet;
        let names: HashSet<_> = catalog().scene_names().collect();
        assert_eq!(names.len(), KINDS.len());
        for kind in KINDS {
            let model = rig(kind);
            assert!(names.contains(model.scene_name.as_str()));
            assert_eq!(model.animations.len(), 2);
            for movement in ["walk", "run"] {
                let expected = format!("{}.{movement}", model.scene_name);
                let clip = model
                    .animations
                    .iter()
                    .find(|clip| clip.name.as_deref() == Some(expected.as_str()))
                    .unwrap();
                assert!(!clip.channels.is_empty());
                assert!(clip.duration > 0.0);
                let bind = model.joints.each_ref().map(|joint| joint.bind);
                let animated = clip
                    .sample(&bind, clip.start + clip.duration * 0.25)
                    .unwrap();
                assert_ne!(animated, bind, "{expected} must contain real articulation");
                assert_eq!(model.joints.each_ref().map(|joint| joint.bind), bind);
            }
            assert!(std::ptr::eq(model, rig(kind)));
        }
    }
    // Frozen from the pre-GLB Rust parser at 62b6ae6, after every field of all
    // 75 rigs passed a bit-for-bit comparison with this GLB reader. These hashes
    // cover parent/bind, vertices, normalized normals, linear RGBA, UVs and
    // indices, without retaining a second geometry representation in the repo.
    const LEGACY_RUNTIME_GEOMETRY_SHA256: &[(&str, &str)] = &[
        (
            "trainer",
            "b8101eac3ecdbb679f343fb585fe148004722386519ecaa4cc0984a22a64a9ab",
        ),
        (
            "trainer_female",
            "1544a2644c196c5b3d6d4d5ba9173cfa9e9dfd46b2d8d7d98d8d02bc572c8a0c",
        ),
        (
            "rival",
            "df669a3f1ec48b34359716a41f8686d3641d9acf0971f71518be853ca33bb9fe",
        ),
        (
            "youngster",
            "e9a31a482ebf1ad144ae3e6e1d3be892ce7e1c06a651f4d18b2874823f63a756",
        ),
        (
            "teacher",
            "a66582d5b91c26d4a36117d8c707e57149f5e6d2a1b0525830d0f84b819d44b2",
        ),
        (
            "lass",
            "2f779a76cdd2db65c3f17042df747fb92257bf5de1928ed4343a1e8ee22f562b",
        ),
        (
            "scientist",
            "1467b85dcf5ddf1282d619d62fbd41f330eba10d5d61ad090eea4fd1253aff12",
        ),
        (
            "outdoorsman",
            "7c655b62944806dff0fc6fbcd7d0b695866d54abe5a96168661c6c84bcaaccab",
        ),
        (
            "elder",
            "7948599f6b3e50ff4894c5d968846e65aa52b0326a15f19d53358455e692cba6",
        ),
        (
            "beauty",
            "0a85fd9080854c88d732837a972cffadddf8b87963964844892e85fd83c3a8cb",
        ),
        (
            "biker",
            "4816b241fa1b493bfad419053f1549ce82fe7a04efbafc449facb82b0c7a5f99",
        ),
        (
            "bill",
            "80de423cde2c19b4c79ab23731982f5f5ae6b3d5aef70232ed8b849008fa5a8c",
        ),
        (
            "black_belt",
            "d7310eda0f9265619b59da46ae1f2513bd014e8a819f14d7732c9ed6fb0b1857",
        ),
        (
            "blaine",
            "f00693e1ea5de28f8626ba2f5f389062ae2e7cdc5beab606de5f9d7bb2772d83",
        ),
        (
            "blue",
            "cc91da9de658d5f05071a3db7d228fd6d64952be14701211ab633e997e9c07dd",
        ),
        (
            "brock",
            "af4bacf386bb63a4eed20c4654877a1411ad9a414e04ca11675dc66a8723b9f9",
        ),
        (
            "bruno",
            "9973bb74308cbc1b95feff0f0d5f6ffad181882ff7b94fef7385efb3814c48d2",
        ),
        (
            "bug_catcher",
            "707e3365535bc30798068684f36d1c564a82addccc7fa66ea5048be8cb3eae13",
        ),
        (
            "bugsy",
            "3bbb9976244f3ad3205ee89fa4d94e3530ab2562246f4ed72a4ca032026c82eb",
        ),
        (
            "cal",
            "c87291d4331fb03c4a13d1e6f09f1efd7a26f5fec3a41e044be5d848afb27842",
        ),
        (
            "captain",
            "bd849886179b6174910514dae3c277c91034f844fe9ea7598223eecd5f6529af",
        ),
        (
            "chuck",
            "aad46c48198838b7bfad705592b37275073db390d2dd7f716dc0f68b2176f109",
        ),
        (
            "clair",
            "9e6c0407574c244b07524b627dc2136f1a66e580b97525472a5fbb0f96cd26ec",
        ),
        (
            "clerk",
            "70f7422da78376ffb56983799fc00cde991eee6bb4b701fdb2372ba5d4c09f9d",
        ),
        (
            "cooltrainer_f",
            "acf90a2b6ad7cb29109f5c0a75ef2de960e5c8185cdf0d66ea2c2076e6051dbe",
        ),
        (
            "cooltrainer_m",
            "757f0d11bcc57458aae40ba0d3a615c278f63f2084bdeecd4f6cfe5c1243962c",
        ),
        (
            "daisy",
            "40b7a9af2a5aa5df71af10a6cb21e3306b3ec16a971f0b66d5ac3659971c163e",
        ),
        (
            "elm",
            "fb99269ca9374eeee500f6994d4f4ca1d0ccc6f7fef7f0e69a6ae7fbeb64140a",
        ),
        (
            "erika",
            "80842d314cd4c70c6a83c9f5bbf50724d981a7b0c3a455c0cae57829cde2af98",
        ),
        (
            "falkner",
            "b77ead3f26284cec5b1061c203e978ddc74a958753b481388961b68e7f1b53a2",
        ),
        (
            "fisher",
            "4264f1de133bb4b4b6e5fe40584779b2b5648641fbae9f3e103026123cb64265",
        ),
        (
            "fishing_guru",
            "18557f28c5c8360fde6c438d2a6cae17834dec20ad41648cbef5d8d6853e6bba",
        ),
        (
            "gameboy_kid",
            "d4572fe71df5abe56e52ccdd3fda2f6442c80bc5dd30b7ab182bf4f8443c1c10",
        ),
        (
            "gentleman",
            "ce4ee0f8c38bd8d56941e0d5c6755a9ab3a256d407e587f848dee0c20f43b871",
        ),
        (
            "gramps",
            "fa7d1935efc2bfca268b91122350fc3e6c15fcedd95f8524371d12c576e03bf1",
        ),
        (
            "granny",
            "af7aa02d599311ac19d12e7026cb5921d042b4322865ae404b9a8a9cc5d185c5",
        ),
        (
            "gym_guide",
            "c053c04b3ba93ba6094f11de6d019c073bc06e20b9cc3c7f2736e915894dd404",
        ),
        (
            "janine",
            "a1af2352b8a8a6155bd55b4c7750916afeb41bfa2d0788a965f5771abeeafc3c",
        ),
        (
            "jasmine",
            "6fbf68e728a9f3b6e3a74e9514f44b3b7c95892df3c602e19acadf211fc4d466",
        ),
        (
            "karen",
            "7845f10cf89f3ccffc70b445da9fa7abdade717ed7582af792981daa590ea847",
        ),
        (
            "kimono_girl",
            "fcb27e4c3e8a781f2fc290028473fc55e81309d68c6f1831496b55b9f834dcf2",
        ),
        (
            "koga",
            "50c8ef20d7e23d919efe2fe8b7f662e1bd44f7b64c42f44c105c60d9c342ac43",
        ),
        (
            "kurt",
            "9a9d2531e27d3930eb8a3d6755a4e081666f471cd586fa98a8a64aa851463b4c",
        ),
        (
            "kurt_outside",
            "fad9fadc635603685cea79f76defc8ad7958688c67ec1fba928c867554f3d5f7",
        ),
        (
            "lance",
            "efffe631dcfcd5449d61d60add446acfbe2f1a50cfad732d016201de8af16807",
        ),
        (
            "link_receptionist",
            "a4c0bd8a7fed77e19d788d0e1ba0372ebc3ef6d38a5cb78292041546ac5d4c1e",
        ),
        (
            "misty",
            "660c6a25b12897bbcd1b4f19b774d0399b75906f03765bfaf8319a17a3b6f26e",
        ),
        (
            "mom",
            "61d89da5f6bbe056a69bc4c6f449670939ebaed9671e16a8e791fa09b077818c",
        ),
        (
            "morty",
            "bfa5bc8be2e6172ebf121494058dc3a899390f20180004dd5fc80e4a3c01d38a",
        ),
        (
            "nurse",
            "642b30e13210d521b915d1a109aec83090cb10c650282f41c1a791831ff1e908",
        ),
        (
            "oak",
            "e7849509c5adaf976e37b72895465f489da2e470991c6251aafa72e36ab352d6",
        ),
        (
            "officer",
            "41a29bd1a5cb9fe726d49aeed215b62bc59e3cad15967d0688481a5138a657aa",
        ),
        (
            "old_link_receptionist",
            "aec0005c36cb776c422ac7641234152cc047691527c1b4c52f63cced0031ade0",
        ),
        (
            "pharmacist",
            "5df5746a9c55968d82b62ebbc6235e6f6f9480f78149a0d5b54d26a274a37cde",
        ),
        (
            "pokefan_f",
            "efb86eb92c02d79b8fde3e84d289c45503e8a2a613c16d19c78d5d517b92e678",
        ),
        (
            "pokefan_m",
            "ba23cf55739f0ae5e0d79018a98f1b9576bf04b07bc2b6a6d37a30e730b60534",
        ),
        (
            "pryce",
            "434a78b578378be56c6cfafb964752708b41d119548abd646296d92ef8e7bea3",
        ),
        (
            "receptionist",
            "9294aea60c90e9f39b93557cddf323ffb444ef5bbebc5b75516d966e3fe0fae9",
        ),
        (
            "red",
            "100c7cbae0128995eb09a8dcb98d878379a0c70c327b79e202965ecad2093810",
        ),
        (
            "reds_mom",
            "129b268803f99042026960a61b1820333d16921fa290091a8468101bdc8d1ba6",
        ),
        (
            "rocker",
            "dd6feaf9cfc0c746ff5ac2a98b0746bea21490af00031e4aa6d8368094494688",
        ),
        (
            "rocket",
            "69f292c34f13bb4faeaa37501173fc0c01956280f78f4f4be828d3b46d255368",
        ),
        (
            "rocket_girl",
            "d449931b105e7d9681d57735501ee83820ba811ddb1ce08d9928341cfa93357d",
        ),
        (
            "sabrina",
            "ec331cbddd5b54b42dcd1ad184fe7ae7f739395d407d1527a5f83e48669e6748",
        ),
        (
            "sage",
            "269fa4a68b742ef2d566d23afc50fe8ce67961064f6a67c4c26165b85810701d",
        ),
        (
            "sailor",
            "e3b426c32858ba99547ace2bb69585eda8bf4433d7fa145d49f23b9152f32afe",
        ),
        (
            "standing_youngster",
            "4447bd52d6960b9299ec77198e2831b82d0ed73cc9ec4f57011a20103a00c061",
        ),
        (
            "super_nerd",
            "0645b6628700b3a0a79b0c77cf3f2ac5adb8ad10dbff418ac5895ef6299a6364",
        ),
        (
            "surge",
            "974f7b11543fa6f94a0f339f5cebce384987d6009fe155d9b1f5d839a58ab272",
        ),
        (
            "swimmer_girl",
            "750a3ada8b6ba831c7a193acf7abfaaa1a576a0609dbc8ec68869cafc1c423e3",
        ),
        (
            "swimmer_guy",
            "71dacda91acde1c56db55f1705055d0dadd4cda705c33e3e51cad26c9f555002",
        ),
        (
            "twin",
            "824a88b00d1d1f3a6665d245ca75a6287d7e20a73dc328e46a7eb59fe362467e",
        ),
        (
            "unused_guy",
            "158fbced7c6938522e3faf82cd7c2109871ec4c4fc1367265e55caba81200b93",
        ),
        (
            "whitney",
            "44a8ff217508bea6aeae9b1b75552f64d909d051dbfeeecaac2da8f5a0ba3d9b",
        ),
        (
            "will",
            "9d0b66b7b33c83fdca79e38ecf700612f580abf5252d65880215d5db48c61231",
        ),
    ];

    #[test]
    fn all_75_glb_scenes_preserve_original_runtime_geometry_bits() {
        use sha2::{Digest, Sha256};
        assert_eq!(LEGACY_RUNTIME_GEOMETRY_SHA256.len(), KINDS.len());
        for kind in KINDS {
            let model = rig(kind);
            let expected = LEGACY_RUNTIME_GEOMETRY_SHA256
                .iter()
                .find(|(name, _)| *name == model.scene_name)
                .expect("every canonical scene must have a migration baseline")
                .1;
            let mut hash = Sha256::new();
            for joint in &model.joints {
                hash.update(
                    joint
                        .parent
                        .map(|i| i as u32)
                        .unwrap_or(u32::MAX)
                        .to_le_bytes(),
                );
                for value in joint
                    .bind
                    .translation
                    .to_array()
                    .into_iter()
                    .chain(joint.bind.rotation.to_array())
                    .chain(joint.bind.scale.to_array())
                {
                    hash.update(value.to_bits().to_le_bytes());
                }
                for values in [&joint.mesh.positions, &joint.mesh.normals] {
                    hash.update((values.len() as u64).to_le_bytes());
                    for value in values.iter().flatten() {
                        hash.update(value.to_bits().to_le_bytes());
                    }
                }
                hash.update((joint.mesh.colors.len() as u64).to_le_bytes());
                for value in joint.mesh.colors.iter().flatten() {
                    hash.update(value.to_bits().to_le_bytes());
                }
                hash.update((joint.mesh.uvs.len() as u64).to_le_bytes());
                for value in joint.mesh.uvs.iter().flatten() {
                    hash.update(value.to_bits().to_le_bytes());
                }
                hash.update((joint.mesh.indices.len() as u64).to_le_bytes());
                for value in &joint.mesh.indices {
                    hash.update(value.to_le_bytes());
                }
            }
            assert_eq!(
                format!("{:x}", hash.finalize()),
                expected,
                "{} runtime geometry changed",
                model.scene_name
            );
        }
    }
}
