//! Bounded reader for authored Pokémon standard glTF skins.
//! Geometry is decoded once, without baking joint transforms or changing its
//! neutral coordinate system. Battle time/placement and GPU entities are owned
//! by the caller. Unsupported content is an error, never a static fallback.
use std::{collections::HashSet, ops::Range, sync::OnceLock};

use bevy::prelude::{Mat4, Quat, Transform, Vec3};
use gltf::{
    accessor::{DataType, Dimensions},
    animation::{Interpolation, Property},
    mesh::{Semantic, util::ReadWeights},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::mesh::SurfaceMeshData;

const MAX_BYTES: usize = 1024 * 1024;
const MAX_VERTICES: usize = 20_000;
const MAX_INDICES: usize = 120_000;
const MAX_ACCESSORS: usize = 256;
const MAX_KEYS: usize = 257;
const MAX_DURATION: f32 = 10.0;
const ENVELOPE_HZ: f32 = 30.0;
const FLOAT_MARGIN: f32 = 0.0001;

const UNOWN_PART_COUNTS: [usize; 26] = [8, 7, 5, 5, 8, 8, 7, 8, 6, 6, 8, 6, 7, 6, 6, 6, 6, 6, 6, 6, 7, 6, 7, 8, 8, 6];
const UNOWN_DIGESTS: [&str; 26] = [
    "c73dbe28b59ab0a8a8ea462cc763acf05e7ad57d5df10794457c0b26429d08d0",
    "709f94e4172e8ecf2c3d05778bc8cf7c69e59cf652cc7906c2e003711f303a3b",
    "ea246f2878eb100b62e1f4aea35ec73dad44f9ad3ec9d1f4ac2ceb294e93ea3d",
    "f8bdf4987fd951ce491301ff87cc18e87029ca0be5f50b55eadaf27367ce1844",
    "b3791a0180b5c07e5386cfd76d68620af062be2dc3173dacd31a2014af0ac6cc",
    "e2796140938ec39927d1b7e28ce5795a960f3fbfcfd3d260362a25e0954fa700",
    "1d1c4214f8d3f0863b87939f23bb02c43d0b26b5a159f53fc90663773b3c9244",
    "ac58fb0cc0dd034c8d1f2e620ea7fe6d9a7f09bce702170c0a66605d62a1a0f6",
    "b7de7d05fdb79cc267821edc5a336048341cf8955b9b063172361b9e7d16516f",
    "d2f4bc9389a525876905ab26cec40f557679e1243cd9a7ce2bcbe94ddd27441a",
    "f6864362324b442bf98325cf5687120a4b7a16d9635a01bafe09f15b3265d8fe",
    "6fc0930cd4fd5a14c4361c50cafce9bdeb8ae85ba29dae7be405a2cec9b39e31",
    "a0ab086eee8ac95a18c8986b74c1be5482c88a53c21ac6240b1e145b349fc9fa",
    "102d136ec2c4047e2cacf80937b16c340229705f0a8231ac1b212bc023fcdff9",
    "afee279d3d7df6bedf7a697ab6e1a7cd79231929c7a92ff900f5f9da423e95aa",
    "7a7f956505eae3dafd3b99338c2d7fd4f904f6d6b9bcdb5684a18e02a58f5ec0",
    "45af59081b1f4de2664279deb2854cc0d73912cd0fd66456c8684b090f3c9195",
    "b4d44af912dd13e8380f2535a46af8d9e813f79342725b4fc56719d5119a6d77",
    "de9ea07911edecebd23e37e216f2032a05ff56cc341422f6ee5393662604ad7c",
    "24f974f50e454a54323368b22fe264a0058a75d4759ecde49cd6a0d01d8617a6",
    "6708baf987fcc42d083657b19311a6f755c77b500bb03fa3b3930ac34cdb5b62",
    "1026bb702cf9c9f5fa8d1b3f006605390b26dafc4a16943b3031489a529274bb",
    "b62999e5454a604d8a1f68300934f850fff420ecb1ccd906f748a88d1196352c",
    "22acc7dc272833b0f1e80ae84c76b538633fa6884b0d42ca8b262d1f77519a02",
    "1074672b12d5ef711208bbab10713eb1d6f232092b254c8dafcb0538bfe7d4e7",
    "c47914a33662a3304752409e05e0227748bbc7abf7ba333a1c997aa467174323",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Species {
    Chikorita,
    Bayleef,
    Meganium,
    Cyndaquil,
    Totodile,
    Gengar,
    Spearow,
    Wooper,
    Gyarados,
    Sudowoodo,
    Lapras,
    Snorlax,
    Pikachu,
    Bulbasaur,
    Charmander,
    Squirtle,
    Geodude,
    Pidgey,
    Oddish,
    Poliwag,
    Clefairy,
    Jigglypuff,
    Rattata,
    Raticate,
    Psyduck,
    Golduck,
    Zubat,
    Golbat,
    Crobat,
    Butterfree,
    Beedrill,
    Hoothoot,
    Sentret,
    Staryu,
    Voltorb,
    Onix,
    Diglett,
    Slowpoke,
    Entei,
    Raikou,
    Suicune,
    Furret,
    Noctowl,
    Ledyba,
    Ledian,
    Spinarak,
    Ariados,
    Quagsire,
    Hoppip,
    Skiploom,
    Aipom,
    Sunkern,
    Sunflora,
    Jumpluff,
    Unown(u8),
}

impl Species {
    pub const ALL: [Self; 80] = [
        Self::Chikorita,
        Self::Bayleef,
        Self::Meganium,
        Self::Cyndaquil,
        Self::Totodile,
        Self::Gengar,
        Self::Spearow,
        Self::Wooper,
        Self::Gyarados,
        Self::Sudowoodo,
        Self::Lapras,
        Self::Snorlax,
        Self::Pikachu,
        Self::Bulbasaur,
        Self::Charmander,
        Self::Squirtle,
        Self::Geodude,
        Self::Pidgey,
        Self::Oddish,
        Self::Poliwag,
        Self::Clefairy,
        Self::Jigglypuff,
        Self::Rattata,
        Self::Raticate,
        Self::Psyduck,
        Self::Golduck,
        Self::Zubat,
        Self::Golbat,
        Self::Crobat,
        Self::Butterfree,
        Self::Beedrill,
        Self::Hoothoot,
        Self::Sentret,
        Self::Staryu,
        Self::Voltorb,
        Self::Onix,
        Self::Diglett,
        Self::Slowpoke,
        Self::Entei,
        Self::Raikou,
        Self::Suicune,
        Self::Furret,
        Self::Noctowl,
        Self::Ledyba,
        Self::Ledian,
        Self::Spinarak,
        Self::Ariados,
        Self::Quagsire,
        Self::Hoppip,
        Self::Skiploom,
        Self::Aipom,
        Self::Sunkern,
        Self::Sunflora,
        Self::Jumpluff,
        Self::Unown(0),
        Self::Unown(1),
        Self::Unown(2),
        Self::Unown(3),
        Self::Unown(4),
        Self::Unown(5),
        Self::Unown(6),
        Self::Unown(7),
        Self::Unown(8),
        Self::Unown(9),
        Self::Unown(10),
        Self::Unown(11),
        Self::Unown(12),
        Self::Unown(13),
        Self::Unown(14),
        Self::Unown(15),
        Self::Unown(16),
        Self::Unown(17),
        Self::Unown(18),
        Self::Unown(19),
        Self::Unown(20),
        Self::Unown(21),
        Self::Unown(22),
        Self::Unown(23),
        Self::Unown(24),
        Self::Unown(25),

    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Chikorita => "chikorita",
            Self::Bayleef => "bayleef",
            Self::Meganium => "meganium",
            Self::Cyndaquil => "cyndaquil",
            Self::Totodile => "totodile",
            Self::Gengar => "gengar",
            Self::Spearow => "spearow",
            Self::Wooper => "wooper",
            Self::Gyarados => "gyarados",
            Self::Sudowoodo => "sudowoodo",
            Self::Lapras => "lapras",
            Self::Snorlax => "snorlax",
            Self::Pikachu => "pikachu",
            Self::Bulbasaur => "bulbasaur",
            Self::Charmander => "charmander",
            Self::Squirtle => "squirtle",
            Self::Geodude => "geodude",
            Self::Pidgey => "pidgey",
            Self::Oddish => "oddish",
            Self::Poliwag => "poliwag",
            Self::Clefairy => "clefairy",
            Self::Jigglypuff => "jigglypuff",
            Self::Rattata => "rattata",
            Self::Raticate => "raticate",
            Self::Psyduck => "psyduck",
            Self::Golduck => "golduck",
            Self::Zubat => "zubat",
            Self::Golbat => "golbat",
            Self::Crobat => "crobat",
            Self::Butterfree => "butterfree",
            Self::Beedrill => "beedrill",
            Self::Hoothoot => "hoothoot",
            Self::Sentret => "sentret",
            Self::Staryu => "staryu",
            Self::Voltorb => "voltorb",
            Self::Onix => "onix",
            Self::Diglett => "diglett",
            Self::Entei => "entei",
            Self::Raikou => "raikou",
            Self::Ledyba => "ledyba",
            Self::Ledian => "ledian",
            Self::Ariados => "ariados",
            Self::Quagsire => "quagsire",
            Self::Hoppip => "hoppip",
            Self::Skiploom => "skiploom",
            Self::Aipom => "aipom",
            Self::Sunkern => "sunkern",
            Self::Sunflora => "sunflora",
            Self::Jumpluff => "jumpluff",
            Self::Spinarak => "spinarak",
            Self::Noctowl => "noctowl",
            Self::Furret => "furret",
            Self::Suicune => "suicune",
            Self::Slowpoke => "slowpoke",
            Self::Unown(form) => ["unown_a", "unown_b", "unown_c", "unown_d", "unown_e", "unown_f", "unown_g", "unown_h", "unown_i", "unown_j", "unown_k", "unown_l", "unown_m", "unown_n", "unown_o", "unown_p", "unown_q", "unown_r", "unown_s", "unown_t", "unown_u", "unown_v", "unown_w", "unown_x", "unown_y", "unown_z"][usize::from(form)],


        }
    }

    fn joint_names(self) -> &'static [&'static str] {
        match self {
            Self::Diglett => &["root", "body"],
            Self::Unown(_) => &["root", "eye", "glyph"],
            Self::Entei => &["root", "torso", "neck", "head", "mane_left", "mane_right", "smoke_base", "smoke_tip", "blade_left", "blade_right", "fore_left", "fore_right", "rear_left", "rear_right", "tail_base", "tail_tip"],
            Self::Raikou => &["root", "torso", "neck", "head", "ear_left", "ear_right", "mane_base", "mane_middle", "mane_tip", "fore_left", "fore_right", "rear_left", "rear_right", "tail_base", "tail_bend", "tail_tip"],
            Self::Ledyba => &["root", "abdomen", "thorax", "head", "antenna_left", "antenna_right", "upper_arm_left", "upper_arm_right", "lower_arm_left", "lower_arm_right", "leg_left", "leg_right", "case_left", "case_right"],
            Self::Ledian => &["root", "abdomen", "thorax", "head", "antenna_left", "antenna_right", "upper_arm_left", "upper_arm_right", "lower_arm_left", "lower_arm_right", "leg_left", "leg_right", "case_left", "case_right", "wing_left", "wing_right"],
            Self::Ariados => &["root", "abdomen", "head", "horn", "mandible_left", "mandible_right", "leg_left_0_upper", "leg_left_0_lower", "leg_left_1_upper", "leg_left_1_lower", "leg_right_0_upper", "leg_right_0_lower", "leg_right_1_upper", "leg_right_1_lower", "raised_left_base", "raised_left_tip", "raised_right_base", "raised_right_tip"],
            Self::Quagsire => &["root", "pelvis", "torso", "head", "arm_left", "arm_right", "hand_left", "hand_right", "leg_left", "leg_right", "tail_base", "tail_middle", "tail_tip"],
            Self::Hoppip => &["root", "torso", "arm_left", "arm_right", "ear_left", "ear_right", "leaf_left", "leaf_tip_left", "leaf_right", "leaf_tip_right", "tail"],
            Self::Skiploom => &["root", "torso", "arm_left", "arm_right", "ear_left", "ear_right", "ear_tip_left", "ear_tip_right", "flower_stalk", "flower_center", "petal_0", "petal_1", "petal_2", "petal_3", "petal_4", "petal_5", "petal_6", "petal_7"],
            Self::Aipom => &["root", "torso", "neck", "head", "ear_left", "ear_right", "arm_left", "hand_left", "arm_right", "hand_right", "tail_base", "tail_middle", "tail_tip", "tail_palm", "tail_finger_0", "tail_finger_1", "tail_finger_2", "tuft_0", "tuft_1", "tuft_2"],
            Self::Sunkern => &["root", "seed_body", "sprout", "leaf_left", "leaf_tip_left", "leaf_right", "leaf_tip_right"],
            Self::Sunflora => &["root", "bulb", "stem_base", "stem_tip", "flower_disk", "leaf_left", "leaf_tip_left", "leaf_right", "leaf_tip_right", "petal_0", "petal_1", "petal_2", "petal_3", "petal_4", "petal_5", "petal_6", "petal_7", "bract_0", "bract_1", "bract_2", "bract_3", "bract_4", "bract_5", "bract_6", "bract_7"],
            Self::Jumpluff => &["root", "torso", "arm_left", "arm_right", "lower_stem_left", "lower_stem_right", "lower_seed_left", "lower_seed_right", "crown_stem", "crown_stem_tip", "crown_seed", "bract_left", "bract_right"],
            Self::Spinarak => &["root", "abdomen", "head", "horn", "mandible_left", "mandible_right", "leg_left_0_upper", "leg_left_0_lower", "leg_left_1_upper", "leg_left_1_lower", "leg_left_2_upper", "leg_left_2_lower", "leg_right_0_upper", "leg_right_0_lower", "leg_right_1_upper", "leg_right_1_lower", "leg_right_2_upper", "leg_right_2_lower"],
            Self::Noctowl => &["root", "pelvis", "chest", "neck", "head", "crown_left", "crown_right", "wing_left", "wing_right", "leg_left", "leg_right", "tail_base", "tail_middle", "tail_tip"],
            Self::Furret => &["root", "pelvis", "chest", "neck", "head", "ear_left", "ear_right", "arm_left", "arm_right", "leg_left", "leg_right", "tail_base", "tail_middle", "tail_tip"],
            Self::Suicune => &["root", "torso", "neck", "head", "crest", "mane_base", "mane_middle", "mane_tip", "fore_left", "fore_right", "rear_left", "rear_right", "ribbon_left_base", "ribbon_left_bend", "ribbon_left_tip", "ribbon_right_base", "ribbon_right_bend", "ribbon_right_tip"],
            Self::Slowpoke => &["root", "body", "head", "tail_base", "tail_tip", "ear_left", "ear_right", "fore_left", "fore_right", "rear_left", "rear_right"],
            Self::Onix => &["root", "segment_3", "segment_4", "segment_5", "segment_6", "segment_7", "segment_8", "segment_9", "segment_10", "segment_11", "segment_12", "head"],
            Self::Voltorb => &["root", "body"],
            Self::Staryu => &["root", "core", "arm_0_base", "arm_0_tip", "arm_1_base", "arm_1_tip", "arm_2_base", "arm_2_tip", "arm_3_base", "arm_3_tip", "arm_4_base", "arm_4_tip"],
            Self::Sentret => &["root", "tail_support", "tail_middle", "tail_upper", "body", "tail_curl", "tail_tip", "ear_left", "ear_right", "arm_left", "arm_right", "leg_left", "leg_right"],
            Self::Hoothoot => &["root", "body", "brow_left", "brow_tip_left", "brow_right", "brow_tip_right", "wing_left", "wing_tip_left", "wing_right", "wing_tip_right", "tail", "lower_beak"],
            Self::Butterfree => &["root", "thorax", "head", "abdomen", "upper_left", "upper_tip_left", "upper_right", "upper_tip_right", "lower_left", "lower_tip_left", "lower_right", "lower_tip_right", "antenna_left", "antenna_tip_left", "antenna_right", "antenna_tip_right", "arm_left", "arm_right"],
            Self::Beedrill => &["root", "thorax", "head", "abdomen", "upper_left", "upper_tip_left", "upper_right", "upper_tip_right", "lower_left", "lower_tip_left", "lower_right", "lower_tip_right", "antenna_left", "antenna_tip_left", "antenna_right", "antenna_tip_right", "arm_left", "arm_right", "drill_left", "drill_right"],
            Self::Zubat => &["root", "body", "wing_left", "wrist_left", "tip_left", "wing_right", "wrist_right", "tip_right", "ear_left", "ear_right"],
            Self::Golbat => &["root", "body", "wing_left", "wrist_left", "tip_left", "wing_right", "wrist_right", "tip_right", "ear_left", "ear_right"],
            Self::Crobat => &["root", "body", "wing_left", "wrist_left", "tip_left", "wing_right", "wrist_right", "tip_right", "ear_left", "ear_right", "lower_left", "lower_tip_left", "lower_right", "lower_tip_right"],
            Self::Psyduck => &["root", "torso", "head", "arm_left", "hand_left", "arm_right", "hand_right", "tail", "hair", "lower_bill"],
            Self::Golduck => &["root", "torso", "head", "arm_left", "hand_left", "arm_right", "hand_right", "tail", "crown", "lower_bill", "tail_tip"],
            Self::Raticate => &["root", "torso", "ear_left", "ear_right", "forepaw_left", "forepaw_right", "tail_base", "tail_tip"],
            Self::Rattata => &["root", "torso", "neck", "head", "ear_left", "ear_right", "tail_base", "tail_curl", "tail_tip"],
            Self::Squirtle => &["root", "torso", "head", "arm_left", "arm_right", "leg_left", "leg_right", "tail_base", "tail_curl"],
            Self::Geodude => &["root", "boulder", "arm_left", "forearm_left", "fist_left", "arm_right", "forearm_right", "fist_right"],
            Self::Pidgey => &["root", "torso", "head", "lower_beak", "wing_left", "flight_left", "wing_right", "flight_right", "tail_fan", "crown"],
            Self::Oddish => &["root", "bulb", "leaf_0_base", "leaf_0_tip", "leaf_1_base", "leaf_1_tip", "leaf_2_base", "leaf_2_tip", "leaf_3_base", "leaf_3_tip", "leaf_4_base", "leaf_4_tip"],
            Self::Poliwag => &["root", "body", "tail_base", "tail_middle", "tail_tip"],
            Self::Clefairy => &["root", "body", "arm_left", "arm_right", "ear_left", "ear_right", "wing_left", "wing_right", "forelock", "tail_base", "tail_curl"],
            Self::Jigglypuff => &["root", "body", "arm_left", "arm_right", "ear_left", "ear_right", "forelock"],
            Self::Charmander => &["root", "torso", "head", "arm_left", "arm_right", "leg_left", "leg_right", "tail_base", "tail_mid", "tail_tip", "flame"],
            Self::Bulbasaur => &["root", "torso", "head", "foreleg_left", "foreleg_right", "hindleg_left", "hindleg_right", "bulb_base", "bulb_tip"],
            Self::Pikachu => &["root", "torso", "head", "arm_left", "arm_right", "ear_left", "ear_left_tip", "ear_right", "ear_right_tip", "tail_base", "tail_tip"],
            Self::Snorlax => &["root", "torso", "head", "arm_left", "hand_left", "arm_right", "hand_right", "ear_left", "ear_right"],
            Self::Lapras => &["root", "neck_base", "neck_upper", "head", "flipper_left", "flipper_right", "rear_left", "rear_right", "tail"],
            Self::Sudowoodo => &[
                "root", "torso", "head", "arm_left", "hand_left",
                "arm_right", "hand_right", "crown_base", "crown_right_fork",
            ],
            Self::Gyarados => &[
                "root", "neck_base", "neck_upper", "head", "lower_jaw",
                "barbel_left", "barbel_right", "tail_base", "tail_tip",
            ],
            Self::Wooper => &[
                "root", "torso", "head", "leg_left", "leg_right",
                "gill_left", "gill_right", "tail_base", "tail_tip",
            ],
            Self::Chikorita => &[
                "root",
                "torso",
                "head",
                "foreleg_left",
                "foreleg_right",
                "hindleg_left",
                "hindleg_right",
                "tail_nub",
                "leaf_petiole",
                "leaf_mid_fold",
                "leaf_tip_fold",
            ],
            Self::Bayleef => &[
                "root", "pelvis", "chest", "neck_base", "head",
                "foreleg_left", "foreleg_right", "hindleg_left", "hindleg_right",
                "tail_base", "tail_tip", "crown_petiole", "crown_mid_fold",
                "crown_tip_fold", "collar_left", "collar_right",
            ],
            Self::Meganium => &[
                "root", "pelvis", "chest", "neck_base", "neck_upper", "head",
                "foreleg_left", "foreleg_right", "hindleg_left", "hindleg_right",
                "tail_base", "tail_tip", "antenna_left_base", "antenna_left_tip",
                "antenna_right_base", "antenna_right_tip", "flower_petal_1",
                "flower_petal_2", "flower_petal_3", "flower_petal_4",
                "flower_petal_5", "flower_petal_6", "lower_jaw",
            ],
            Self::Cyndaquil => &[
                "root",
                "torso",
                "head",
                "foreleg_left",
                "foreleg_right",
                "upper_left_flame_quill",
                "upper_right_flame_quill",
                "lower_left_flame_quill",
                "lower_right_flame_quill",
                "central_rear_flame_quill",
            ],
            Self::Totodile => &[
                "root",
                "torso",
                "head",
                "lower_jaw",
                "forepaw_left",
                "forepaw_right",
                "tail_base",
                "tail_tip",
            ],
            Self::Gengar => &[
                "root",
                "torso",
                "face_core",
                "shoulder_left",
                "hand_left",
                "shoulder_right",
                "hand_right",
                "ear_left",
                "ear_right",
                "dorsal_crown",
                "dorsal_left",
                "dorsal_right",
                "dorsal_lower",
                "tail_spine",
            ],
            Self::Spearow => &[
                "root",
                "torso",
                "head",
                "lower_beak",
                "wing_left",
                "flight_feathers_left",
                "wing_right",
                "flight_feathers_right",
                "tail_fan",
                "crown",
            ],
        }
    }

    fn parents(self) -> &'static [Option<usize>] {
        match self {
            Self::Diglett => &[None, Some(0)],
            Self::Unown(_) => &[None, Some(0), Some(1)],
            Self::Entei => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(1), Some(6), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(14)],
            Self::Raikou => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(1), Some(1), Some(1), Some(1), Some(1), Some(13), Some(14)],
            Self::Ledyba => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(2), Some(2), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Ledian => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(2), Some(2), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(2), Some(2)],
            Self::Ariados => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(1), Some(12), Some(1), Some(14), Some(1), Some(16)],
            Self::Quagsire => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(4), Some(5), Some(1), Some(1), Some(1), Some(10), Some(11)],
            Self::Hoppip => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(6), Some(1), Some(8), Some(1)],
            Self::Skiploom => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(4), Some(5), Some(1), Some(8), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9), Some(9)],
            Self::Aipom => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(11), Some(12), Some(13), Some(13), Some(13), Some(3), Some(3), Some(3)],
            Self::Sunkern => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(5)],
            Self::Sunflora => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(1), Some(7), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4)],
            Self::Jumpluff => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(4), Some(5), Some(1), Some(8), Some(9), Some(8), Some(8)],
            Self::Spinarak => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(1), Some(12), Some(1), Some(14), Some(1), Some(16)],
            Self::Noctowl => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(4), Some(2), Some(2), Some(1), Some(1), Some(1), Some(11), Some(12)],
            Self::Furret => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(4), Some(2), Some(2), Some(1), Some(1), Some(1), Some(11), Some(12)],
            Self::Suicune => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(1), Some(1), Some(1), Some(1), Some(1), Some(12), Some(13), Some(1), Some(15), Some(16)],
            Self::Slowpoke => &[None, Some(0), Some(1), Some(1), Some(3), Some(2), Some(2), Some(1), Some(1), Some(1), Some(1)],
            Self::Onix => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8), Some(9), Some(10)],
            Self::Voltorb => &[None, Some(0)],
            Self::Staryu => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10)],
            Self::Sentret => &[None, Some(0), Some(1), Some(2), Some(3), Some(0), Some(5), Some(4), Some(4), Some(4), Some(4), Some(4), Some(4)],
            Self::Hoothoot => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(1)],
            Self::Butterfree => &[None, Some(0), Some(1), Some(0), Some(1), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(2), Some(12), Some(2), Some(14), Some(1), Some(1)],
            Self::Beedrill => &[None, Some(0), Some(1), Some(0), Some(1), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(2), Some(12), Some(2), Some(14), Some(1), Some(1), Some(16), Some(17)],
            Self::Zubat => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(1), Some(1)],
            Self::Golbat => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(1), Some(1)],
            Self::Crobat => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(1), Some(1), Some(1), Some(10), Some(1), Some(12)],
            Self::Psyduck => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(5), Some(1), Some(2), Some(2)],
            Self::Golduck => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(1), Some(2), Some(2), Some(7)],
            Self::Raticate => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(6)],
            Self::Rattata => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7)],
            Self::Squirtle => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(7)],
            Self::Geodude => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6)],
            Self::Pidgey => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(2)],
            Self::Oddish => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10)],
            Self::Poliwag => &[None, Some(0), Some(1), Some(2), Some(3)],
            Self::Clefairy => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(9)],
            Self::Jigglypuff => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Charmander => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(7), Some(8), Some(9)],
            Self::Bulbasaur => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(7)],
            Self::Pikachu => &[None, Some(0), Some(1), Some(1), Some(1), Some(2), Some(5), Some(2), Some(7), Some(1), Some(9)],
            Self::Snorlax => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(2), Some(2)],
            Self::Lapras => &[None, Some(0), Some(1), Some(2), Some(0), Some(0), Some(0), Some(0), Some(0)],
            Self::Sudowoodo => &[
                None, Some(0), Some(1), Some(1), Some(3),
                Some(1), Some(5), Some(2), Some(7),
            ],
            Self::Gyarados => &[
                None, Some(0), Some(1), Some(2), Some(3),
                Some(3), Some(3), Some(0), Some(7),
            ],
            Self::Wooper => &[
                None, Some(0), Some(1), Some(0), Some(0),
                Some(2), Some(2), Some(1), Some(7),
            ],
            Self::Chikorita => &[
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(2),
                Some(8),
                Some(9),
            ],
            Self::Bayleef => &[
                None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(2),
                Some(1), Some(1), Some(1), Some(9), Some(4), Some(11), Some(12),
                Some(3), Some(3),
            ],
            Self::Meganium => &[
                None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(2), Some(2),
                Some(1), Some(1), Some(1), Some(10), Some(5), Some(12), Some(5),
                Some(14), Some(3), Some(3), Some(3), Some(3), Some(3), Some(3), Some(5),
            ],
            Self::Cyndaquil => &[
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
                Some(1),
            ],
            Self::Totodile => &[
                None,
                Some(0),
                Some(1),
                Some(2),
                Some(1),
                Some(1),
                Some(0),
                Some(6),
            ],
            Self::Gengar => &[
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(3),
                Some(1),
                Some(5),
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                Some(2),
                Some(1),
                Some(1),
            ],
            Self::Spearow => &[
                None,
                Some(0),
                Some(1),
                Some(2),
                Some(1),
                Some(4),
                Some(1),
                Some(6),
                Some(1),
                Some(2),
            ],
        }
    }

    fn part_count(self) -> usize {
        match self {
            Self::Diglett => 15,
            Self::Entei => 59,
            Self::Raikou => 62,
            Self::Ledyba => 36,
            Self::Ledian => 38,
            Self::Ariados => 41,
            Self::Quagsire => 22,
            Self::Hoppip => 21,
            Self::Skiploom => 24,
            Self::Aipom => 50,
            Self::Sunkern => 18,
            Self::Sunflora => 30,
            Self::Jumpluff => 25,
            Self::Spinarak => 43,
            Self::Noctowl => 52,
            Self::Furret => 33,
            Self::Suicune => 47,
            Self::Slowpoke => 40,
            Self::Unown(form) => UNOWN_PART_COUNTS[usize::from(form)],
            Self::Onix => 36,
            Self::Voltorb => 8,
            Self::Staryu => 10,
            Self::Sentret => 38,
            Self::Hoothoot => 38,
            Self::Butterfree => 59,
            Self::Beedrill => 42,
            Self::Zubat => 28,
            Self::Golbat => 32,
            Self::Crobat => 28,
            Self::Chikorita => 19,
            Self::Bayleef => 33,
            Self::Meganium => 33,
            Self::Cyndaquil => 30,
            Self::Totodile => 30,
            Self::Gengar => 27,
            Self::Spearow => 28,
            Self::Wooper => 22,
            Self::Gyarados => 40,
            Self::Sudowoodo => 33,
            Self::Lapras => 42,
            Self::Snorlax => 31,
            Self::Pikachu => 40,
            Self::Bulbasaur => 44,
            Self::Charmander => 43,
            Self::Squirtle => 45,
            Self::Geodude => 29,
            Self::Pidgey => 43,
            Self::Oddish => 19,
            Self::Poliwag => 19,
            Self::Clefairy => 35,
            Self::Jigglypuff => 22,
            Self::Rattata => 42,
            Self::Raticate => 63,
            Self::Psyduck => 28,
            Self::Golduck => 45,
        }
    }

    fn neutral_digest(self) -> &'static str {
        match self {
            Self::Diglett => "f81fd70948ece5d6ade091c74c000f7a9b03ee6aab22981524543f4a07fc38c4",
            Self::Entei => "db453e0d91aa89189660d76f63e17f62e26c4d80e750455f9fb7e066cbcc249c",
            Self::Raikou => "06890d739507ac0cd3add1a6da44b954daafcb44f897f9e96e30ea039a597c6f",
            Self::Ledyba => "38b13162e17a6677a6d3c6cc6561f870210a342cbb00e226a8688ee5d5811e2a",
            Self::Ledian => "d8b776f761bf9f6287d975cc158c3ebf2b9f06b2b67479a98ef19e881f202031",
            Self::Ariados => "b680cd4729af2101b86594f951817cb56f56a2c4e03b80209c2863345caa57f4",
            Self::Quagsire => "d97c02d58f80145da2df67f1902083c13ccc862253c75e9332e5d8b04e5e36ba",
            Self::Hoppip => "aaa5bb6c980f1ba8d3327f7f611a059193c29ae7092e35f06acab3b9181507b4",
            Self::Skiploom => "9541ae752be3d0669e5836358537f971a6bf1d42ca2d2d72db639008e0be4987",
            Self::Aipom => "5dfead35f664f46a4a4ae8d8b59a8f22345dae52cf785d5d9a200fed882a0140",
            Self::Sunkern => "2c11fc4e36f23be374beb989f959372f218b2cb292dcc91286cb8804df72ea8b",
            Self::Sunflora => "3342555c766e836e7ea17ed6fc8a931fb727e29fbb06f97eee739104ea670f70",
            Self::Jumpluff => "2de06589332b313b7e2264c47f04d319ddb334f09c1caca6632cf7a69b7659fd",
            Self::Spinarak => "b88a15aff01d0ca5d5362a8bb3bcfc8d86c389a1bc7c5d15c6d722bb194ee1d6",
            Self::Noctowl => "fc6025dcce69b1c6b5a9dd2a79254190efe7692e600171dddf3d316e29a1a713",
            Self::Furret => "23d49398710d1399a28120ee0734869065766ebb09a5d9b19b7b48edb4f9565b",
            Self::Suicune => "c3634acaa906645bb52cef6dcc37411a87b9dbd3862b908b3d4235e7c933743e",
            Self::Slowpoke => "e6b60180c57a253da129803d01cb7856b76599b67a51e6ac3f8d06b4e1e38c64",
            Self::Unown(form) => UNOWN_DIGESTS[usize::from(form)],
            Self::Onix => "c36c6f0a7b6d955e1f5937d4862220768229c6ac7e661abc7b00fd354622fa10",
            Self::Voltorb => "560b40b398d50fcbeb2944890bd8e60733c1182c83c20b824e84b8a358f4e605",
            Self::Staryu => "c8ef0c854e722452e6850b79cd7df6b66598e7b21488b69ba818c5dc80e87a67",
            Self::Sentret => "0a15f7e364cdec1b70a33e5e68df64620b7824223129e72dc9184606b8ce4485",
            Self::Hoothoot => "93fbea6a47abff0fff1f6ee6315f7ac09ea26bb1bf60a26b0d293a8ae7784555",
            Self::Butterfree => "4b94b9ecfabca7dee88ec0d5b725d50116cb1f2a139d5a2817342e53cd4ee550",
            Self::Beedrill => "998ef3cf0f5f76c6ada93622528f1806dab5361c60d1b8ecb43c991f1979b900",
            Self::Zubat => "737d2dad1c97b82b0fead248d72ac003e5e759ce90debf0c4cabb06d80c851d0",
            Self::Golbat => "db6b0c4bcb4c3f268b98ca45678c143db637a570e2cacc1867629fa2f2f558d4",
            Self::Crobat => "d01d5cfb92de35f18629800273962eb8a9719f659891ad7f397d44202ff717bd",
            Self::Psyduck => "4d28e9c51e3d5ed94fcccfe356d5cc60dc9ee2d7645332334c7bb7d6ef5b9b4c",
            Self::Golduck => "e7ab59e4de9d3d78f761557cc5b62d6e41442f54c3e25b7fffbecba18892178d",
            Self::Raticate => "f6043d52ed1588580d3e4c8f44480d8d3cb2b8b2f268e6ccd7220287d76edd0d",
            Self::Rattata => "ce7c14e7cf1694a4131d3ed3e3c709fe51f9b2ff7618089d3c40bf731aeb8a3d",
            Self::Squirtle => "222004eaea9a5593380331ab51391d1ddfdcda105fc35c11ebe53835e4e341d7",
            Self::Geodude => "90658750b47d9380750c88431617ae2860b5f5cdb8e6d1d0af8fb320124ab33d",
            Self::Pidgey => "814a75e55669f0361edc7c35026a7a0b112085cda12d4dcc485693bfd1990fb7",
            Self::Oddish => "b138445f423d89daff929c82b90cc4950b9a9534b53d767887664b65e1ada7ab",
            Self::Poliwag => "2914f4cf3bdb2b4ea6f4795e1586e3d93f4dee15f5b0050ca2981fdcc37c2f2d",
            Self::Clefairy => "0d173aea43c62408317aabf296a57c3a202a674df5464cd94b29fc04e872accc",
            Self::Jigglypuff => "be7a901fb200b5528a7ec3fbba34c9944776fdb953d1eb15b69f58cf9419664e",
            Self::Charmander => "6ae03caf697cdaa11acbbb4d2bc90af86f0db04ecf02b64c7e7342a5aed72bfa",
            Self::Bulbasaur => "2789ad2986d637dbe83e6b3b4eb899dfe9ccd06d0746c3332053e72f3bd033df",
            Self::Pikachu => "4bdd281a376a6427e9ee966c69b650d349367033a58f70614987a2e8b1065387",
            Self::Snorlax => "91058a0c15e86e5e6ff07f98ff22bafbc6cd0752f2e861fe80c74ea58a74426c",
            Self::Lapras => "8695d9198539d60d3aaaaea4daf8dfb27f129c3a57c6ef5259af9fa807ec217e",
            Self::Sudowoodo => "010b92720972b99c2bee10df2b6a074d605fce9fef79d4f8eef8c49b2e079275",
            Self::Gyarados => "555f2c6aace148f0fa0e3412cdb5912581b5f6d6f60341c6eeaecdcb7d4ac045",
            Self::Wooper => "5a306ae7d1f9b79c7ce3897c0d538b2752c7cdf85c68f050fe8a1aa0b1bb801a",
            Self::Chikorita => "f2a79d3d20fc56f5584b35c3e7655b0d0654ba12273775dc0e62cca7a8bd9774",
            Self::Bayleef => "3a47bbf1035133575ef25812194da3caec7ea9596b6cc9a5261317acec7864c0",
            Self::Meganium => "9573b5a8f0df3ae27e03195807cf2ca92cfff5067d3f83af6ea294a9d7712643",
            Self::Cyndaquil => "551051e53987f2461c2e5ed22187b44b336be64f300b469950d381b5b174b397",
            Self::Totodile => "e2e34e5ee22c9e86a70f23ec28511f2075b6d2a3f529d7996b3025288bb86a18",
            Self::Gengar => "d458acb7c2853288efef3b83c24ac453bcb409f9db01d24a96008e9a937485aa",
            Self::Spearow => "42ad1c0255af66f84d1770d9050c2d5b8fa7833a1a38180fa380665915b09fe7",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Clip {
    Idle,
    Attack,
    Hit,
}

impl Clip {
    pub const ALL: [Self; 3] = [Self::Idle, Self::Attack, Self::Hit];
    fn index(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Attack => 1,
            Self::Hit => 2,
        }
    }
    fn suffix(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Attack => "attack",
            Self::Hit => "hit",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Playback {
    Loop,
    Clamp,
}

#[derive(Debug,Clone)]
pub(crate) struct Joint {
    /// Original skin order, also the GPU JOINTS_0 indexing order.
    pub name: String,
    pub parent: Option<usize>,
    pub bind: Transform,
    pub inverse_bind: Mat4,
}

#[derive(Debug,Clone)]
pub(crate) struct Anatomy {
    pub name: String,
    pub vertices: Range<usize>,
    pub indices: Range<usize>,
    pub material: usize,
}

#[derive(Debug,Clone)]
pub(crate) struct Material {
    pub name: String,
    pub color: [f32; 4],
    pub alpha_mode: gltf::material::AlphaMode,
}

#[derive(Debug,Clone)]
pub(crate) struct SpeciesClip {
    pub name: String,
    pub duration: f32,
    pub loop_suggested: bool,
    pub cue_relative: bool,
    /// Preserve author metadata without deriving gameplay semantics from it.
    pub extras: String,
    times: Vec<f32>,
    /// Joint zero stays at identity; all other joints have authored rotations.
    rotations: Vec<Vec<Quat>>,
}

#[derive(Debug,Clone)]
pub(crate) struct SpeciesRig {
    pub species: Species,
    /// Original primitive order and f32 position/color bits. Indices are
    /// offset only to address the concatenated vertex buffer; anatomy reverses
    /// that offset exactly. This remains the physical-size/grounding source.
    pub neutral: SurfaceMeshData,
    /// Exact GLB/source normal bits for audit. `neutral.normals` applies the
    /// same one-time normalization as the prior JSON reader for shading parity.
    pub source_normals: Vec<[f32; 3]>,
    pub joint_indices: Vec<[u16; 4]>,
    pub joint_weights: Vec<[f32; 4]>,
    pub anatomy: Vec<Anatomy>,
    pub materials: Vec<Material>,
    pub joints: Vec<Joint>,
    pub clips: [SpeciesClip; 3],
    pub neutral_bounds: (Vec3, Vec3),
    /// Model-space camera/culling envelope. Never use it to renormalize height.
    pub animated_bounds: (Vec3, Vec3),
}

pub(crate) fn rig(species: Species) -> &'static SpeciesRig {
    static CHIKORITA: OnceLock<SpeciesRig> = OnceLock::new();
    static BAYLEEF: OnceLock<SpeciesRig> = OnceLock::new();
    static MEGANIUM: OnceLock<SpeciesRig> = OnceLock::new();
    static CYNDAQUIL: OnceLock<SpeciesRig> = OnceLock::new();
    static TOTODILE: OnceLock<SpeciesRig> = OnceLock::new();
    static GENGAR: OnceLock<SpeciesRig> = OnceLock::new();
    static SPEAROW: OnceLock<SpeciesRig> = OnceLock::new();
    static WOOPER: OnceLock<SpeciesRig> = OnceLock::new();
    static GYARADOS: OnceLock<SpeciesRig> = OnceLock::new();
    static SUDOWOODO: OnceLock<SpeciesRig> = OnceLock::new();
    static LAPRAS: OnceLock<SpeciesRig> = OnceLock::new();
    static SNORLAX: OnceLock<SpeciesRig> = OnceLock::new();
    static PIKACHU: OnceLock<SpeciesRig> = OnceLock::new();
    static BULBASAUR: OnceLock<SpeciesRig> = OnceLock::new();
    static CHARMANDER: OnceLock<SpeciesRig> = OnceLock::new();
    static PSYDUCK: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLDUCK: OnceLock<SpeciesRig> = OnceLock::new();
    static ZUBAT: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLBAT: OnceLock<SpeciesRig> = OnceLock::new();
    static CROBAT: OnceLock<SpeciesRig> = OnceLock::new();
    static BUTTERFREE: OnceLock<SpeciesRig> = OnceLock::new();
    static BEEDRILL: OnceLock<SpeciesRig> = OnceLock::new();
    static HOOTHOOT: OnceLock<SpeciesRig> = OnceLock::new();
    static SENTRET: OnceLock<SpeciesRig> = OnceLock::new();
    static STARYU: OnceLock<SpeciesRig> = OnceLock::new();
    static VOLTORB: OnceLock<SpeciesRig> = OnceLock::new();
    static ONIX: OnceLock<SpeciesRig> = OnceLock::new();
    static DIGLETT: OnceLock<SpeciesRig> = OnceLock::new();
            static SLOWPOKE: OnceLock<SpeciesRig> = OnceLock::new();
    static ENTEI: OnceLock<SpeciesRig> = OnceLock::new();
    static RAIKOU: OnceLock<SpeciesRig> = OnceLock::new();
    static LEDYBA: OnceLock<SpeciesRig> = OnceLock::new();
    static LEDIAN: OnceLock<SpeciesRig> = OnceLock::new();
    static ARIADOS: OnceLock<SpeciesRig> = OnceLock::new();
    static QUAGSIRE: OnceLock<SpeciesRig> = OnceLock::new();
    static HOPPIP: OnceLock<SpeciesRig> = OnceLock::new();
    static SKIPLOOM: OnceLock<SpeciesRig> = OnceLock::new();
    static AIPOM: OnceLock<SpeciesRig> = OnceLock::new();
    static SUNKERN: OnceLock<SpeciesRig> = OnceLock::new();
    static SUNFLORA: OnceLock<SpeciesRig> = OnceLock::new();
    static JUMPLUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static SPINARAK: OnceLock<SpeciesRig> = OnceLock::new();
    static NOCTOWL: OnceLock<SpeciesRig> = OnceLock::new();
    static FURRET: OnceLock<SpeciesRig> = OnceLock::new();
    static SUICUNE: OnceLock<SpeciesRig> = OnceLock::new();
    static UNOWN: [OnceLock<SpeciesRig>; 26] = [const { OnceLock::new() }; 26];


    static RATICATE: OnceLock<SpeciesRig> = OnceLock::new();
    static RATTATA: OnceLock<SpeciesRig> = OnceLock::new();
    static JIGGLYPUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static CLEFAIRY: OnceLock<SpeciesRig> = OnceLock::new();
    static POLIWAG: OnceLock<SpeciesRig> = OnceLock::new();
    static ODDISH: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEY: OnceLock<SpeciesRig> = OnceLock::new();
    static GEODUDE: OnceLock<SpeciesRig> = OnceLock::new();
    static SQUIRTLE: OnceLock<SpeciesRig> = OnceLock::new();
    let cache = match species {
        Species::Chikorita => &CHIKORITA,
        Species::Bayleef => &BAYLEEF,
        Species::Meganium => &MEGANIUM,
        Species::Cyndaquil => &CYNDAQUIL,
        Species::Totodile => &TOTODILE,
        Species::Gengar => &GENGAR,
        Species::Spearow => &SPEAROW,
        Species::Wooper => &WOOPER,
        Species::Gyarados => &GYARADOS,
        Species::Sudowoodo => &SUDOWOODO,
        Species::Lapras => &LAPRAS,
        Species::Snorlax => &SNORLAX,
        Species::Pikachu => &PIKACHU,
        Species::Bulbasaur => &BULBASAUR,
        Species::Charmander => &CHARMANDER,
        Species::Squirtle => &SQUIRTLE,
        Species::Geodude => &GEODUDE,
        Species::Pidgey => &PIDGEY,
        Species::Oddish => &ODDISH,
        Species::Poliwag => &POLIWAG,
        Species::Clefairy => &CLEFAIRY,
        Species::Jigglypuff => &JIGGLYPUFF,
        Species::Rattata => &RATTATA,
        Species::Raticate => &RATICATE,
        Species::Psyduck => &PSYDUCK,
        Species::Golduck => &GOLDUCK,
        Species::Zubat => &ZUBAT,
        Species::Golbat => &GOLBAT,
        Species::Crobat => &CROBAT,
        Species::Butterfree => &BUTTERFREE,
        Species::Beedrill => &BEEDRILL,
        Species::Hoothoot => &HOOTHOOT,
        Species::Sentret => &SENTRET,
        Species::Staryu => &STARYU,
        Species::Voltorb => &VOLTORB,
        Species::Onix => &ONIX,
        Species::Diglett => &DIGLETT,
            Species::Slowpoke => &SLOWPOKE,
        Species::Entei => &ENTEI,
        Species::Raikou => &RAIKOU,
        Species::Ledyba => &LEDYBA,
        Species::Ledian => &LEDIAN,
        Species::Ariados => &ARIADOS,
        Species::Quagsire => &QUAGSIRE,
        Species::Hoppip => &HOPPIP,
        Species::Skiploom => &SKIPLOOM,
        Species::Aipom => &AIPOM,
        Species::Sunkern => &SUNKERN,
        Species::Sunflora => &SUNFLORA,
        Species::Jumpluff => &JUMPLUFF,
        Species::Spinarak => &SPINARAK,
        Species::Noctowl => &NOCTOWL,
        Species::Furret => &FURRET,
        Species::Suicune => &SUICUNE,
        Species::Unown(form) => &UNOWN[usize::from(form)],


    };
    cache.get_or_init(|| {
        let source = match species {
            Species::Psyduck => { crate::model_storage::include_model!("models/battle_species/psyduck.glb") }
            Species::Zubat => { crate::model_storage::include_model!("models/battle_species/zubat.glb") }
            Species::Golbat => { crate::model_storage::include_model!("models/battle_species/golbat.glb") }
            Species::Butterfree => { crate::model_storage::include_model!("models/battle_species/butterfree.glb") }
            Species::Diglett => { crate::model_storage::include_model!("models/battle_species/diglett.glb") }
            Species::Unown(form) => match form {
                0 => crate::model_storage::include_model!("models/battle_species/unown_a.glb"),
                1 => crate::model_storage::include_model!("models/battle_species/unown_b.glb"),
                2 => crate::model_storage::include_model!("models/battle_species/unown_c.glb"),
                3 => crate::model_storage::include_model!("models/battle_species/unown_d.glb"),
                4 => crate::model_storage::include_model!("models/battle_species/unown_e.glb"),
                5 => crate::model_storage::include_model!("models/battle_species/unown_f.glb"),
                6 => crate::model_storage::include_model!("models/battle_species/unown_g.glb"),
                7 => crate::model_storage::include_model!("models/battle_species/unown_h.glb"),
                8 => crate::model_storage::include_model!("models/battle_species/unown_i.glb"),
                9 => crate::model_storage::include_model!("models/battle_species/unown_j.glb"),
                10 => crate::model_storage::include_model!("models/battle_species/unown_k.glb"),
                11 => crate::model_storage::include_model!("models/battle_species/unown_l.glb"),
                12 => crate::model_storage::include_model!("models/battle_species/unown_m.glb"),
                13 => crate::model_storage::include_model!("models/battle_species/unown_n.glb"),
                14 => crate::model_storage::include_model!("models/battle_species/unown_o.glb"),
                15 => crate::model_storage::include_model!("models/battle_species/unown_p.glb"),
                16 => crate::model_storage::include_model!("models/battle_species/unown_q.glb"),
                17 => crate::model_storage::include_model!("models/battle_species/unown_r.glb"),
                18 => crate::model_storage::include_model!("models/battle_species/unown_s.glb"),
                19 => crate::model_storage::include_model!("models/battle_species/unown_t.glb"),
                20 => crate::model_storage::include_model!("models/battle_species/unown_u.glb"),
                21 => crate::model_storage::include_model!("models/battle_species/unown_v.glb"),
                22 => crate::model_storage::include_model!("models/battle_species/unown_w.glb"),
                23 => crate::model_storage::include_model!("models/battle_species/unown_x.glb"),
                24 => crate::model_storage::include_model!("models/battle_species/unown_y.glb"),
                25 => crate::model_storage::include_model!("models/battle_species/unown_z.glb"),
                _ => unreachable!("Unown form must be A..Z"),
            },
            Species::Entei => { crate::model_storage::include_model!("models/battle_species/entei.glb") }
            Species::Raikou => { crate::model_storage::include_model!("models/battle_species/raikou.glb") }
            Species::Ledyba => { crate::model_storage::include_model!("models/battle_species/ledyba.glb") }
            Species::Ledian => { crate::model_storage::include_model!("models/battle_species/ledian.glb") }
            Species::Ariados => { crate::model_storage::include_model!("models/battle_species/ariados.glb") }
            Species::Quagsire => { crate::model_storage::include_model!("models/battle_species/quagsire.glb") }
            Species::Hoppip => { crate::model_storage::include_model!("models/battle_species/hoppip.glb") }
            Species::Skiploom => { crate::model_storage::include_model!("models/battle_species/skiploom.glb") }
            Species::Aipom => { crate::model_storage::include_model!("models/battle_species/aipom.glb") }
            Species::Sunkern => { crate::model_storage::include_model!("models/battle_species/sunkern.glb") }
            Species::Sunflora => { crate::model_storage::include_model!("models/battle_species/sunflora.glb") }
            Species::Jumpluff => { crate::model_storage::include_model!("models/battle_species/jumpluff.glb") }
            Species::Spinarak => { crate::model_storage::include_model!("models/battle_species/spinarak.glb") }
            Species::Noctowl => { crate::model_storage::include_model!("models/battle_species/noctowl.glb") }
            Species::Furret => { crate::model_storage::include_model!("models/battle_species/furret.glb") }
            Species::Suicune => { crate::model_storage::include_model!("models/battle_species/suicune.glb") }
            Species::Slowpoke => { crate::model_storage::include_model!("models/battle_species/slowpoke.glb") }
            Species::Onix => { crate::model_storage::include_model!("models/battle_species/onix.glb") }
            Species::Voltorb => { crate::model_storage::include_model!("models/battle_species/voltorb.glb") }
            Species::Staryu => { crate::model_storage::include_model!("models/battle_species/staryu.glb") }
            Species::Sentret => { crate::model_storage::include_model!("models/battle_species/sentret.glb") }
            Species::Hoothoot => { crate::model_storage::include_model!("models/battle_species/hoothoot.glb") }
            Species::Beedrill => { crate::model_storage::include_model!("models/battle_species/beedrill.glb") }
            Species::Crobat => { crate::model_storage::include_model!("models/battle_species/crobat.glb") }
            Species::Golduck => { crate::model_storage::include_model!("models/battle_species/golduck.glb") }
            Species::Raticate => { crate::model_storage::include_model!("models/battle_species/raticate.glb") }
            Species::Rattata => { crate::model_storage::include_model!("models/battle_species/rattata.glb") }
            Species::Jigglypuff => { crate::model_storage::include_model!("models/actor_props/icon_jigglypuff.glb") }
            Species::Clefairy => { crate::model_storage::include_model!("models/actor_props/icon_clefairy.glb") }
            Species::Poliwag => { crate::model_storage::include_model!("models/battle_species/poliwag.glb") }
            Species::Oddish => { crate::model_storage::include_model!("models/battle_species/oddish.glb") }
            Species::Pidgey => { crate::model_storage::include_model!("models/battle_species/pidgey.glb") }
            Species::Geodude => { crate::model_storage::include_model!("models/battle_species/geodude.glb") }
            Species::Squirtle => { crate::model_storage::include_model!("models/battle_species/squirtle.glb") }
            Species::Charmander => { crate::model_storage::include_model!("models/battle_species/charmander.glb") }
            Species::Bulbasaur => { crate::model_storage::include_model!("models/battle_species/bulbasaur.glb") }
            Species::Pikachu => { crate::model_storage::include_model!("models/battle_species/pikachu.glb") }
            Species::Snorlax => { crate::model_storage::include_model!("models/battle_species/snorlax.glb") }
            Species::Lapras => { crate::model_storage::include_model!("models/battle_species/lapras.glb") }
            Species::Sudowoodo => {
                crate::model_storage::include_model!("models/battle_species/sudowoodo.glb")
            }
            Species::Gyarados => {
                crate::model_storage::include_model!("models/battle_species/gyarados.glb")
            }
            Species::Wooper => {
                crate::model_storage::include_model!("models/battle_species/wooper.glb")
            }
            Species::Chikorita => {
                crate::model_storage::include_model!("models/actor_props/battle_chikorita.glb")
            }
            Species::Bayleef => {
                crate::model_storage::include_model!("models/battle_species/bayleef.glb")
            }
            Species::Meganium => {
                crate::model_storage::include_model!("models/battle_species/meganium.glb")
            }
            Species::Cyndaquil => {
                crate::model_storage::include_model!("models/actor_props/battle_cyndaquil.glb")
            }
            Species::Totodile => {
                crate::model_storage::include_model!("models/actor_props/battle_totodile.glb")
            }
            Species::Gengar => {
                crate::model_storage::include_model!("models/battle_species/gengar.glb")
            }
            Species::Spearow => {
                crate::model_storage::include_model!("models/battle_species/spearow.glb")
            }
        };
        let bytes =
            crate::model_storage::decode_bytes(source).expect("valid authored species GLB storage");
        SpeciesRig::parse(species, &bytes).expect("valid authored species skin and clips")
    })
}

pub(crate) fn for_species(species: &str) -> Option<&'static SpeciesRig> {
    if species.len() == 7 && species.as_bytes()[..6].eq_ignore_ascii_case(b"UNOWN_") {
        let letter = species.as_bytes()[6].to_ascii_lowercase();
        return letter.is_ascii_lowercase().then(|| rig(Species::Unown(letter - b'a')));
    }
    if species.eq_ignore_ascii_case("JUMPLUFF") {
        Some(rig(Species::Jumpluff))
    } else if species.eq_ignore_ascii_case("SUNKERN") {
        Some(rig(Species::Sunkern))
    } else if species.eq_ignore_ascii_case("SUNFLORA") {
        Some(rig(Species::Sunflora))
    } else if species.eq_ignore_ascii_case("AIPOM") {
        Some(rig(Species::Aipom))
    } else if species.eq_ignore_ascii_case("SKIPLOOM") {
        Some(rig(Species::Skiploom))
    } else if species.eq_ignore_ascii_case("HOPPIP") {
        Some(rig(Species::Hoppip))
    } else if species.eq_ignore_ascii_case("QUAGSIRE") {
        Some(rig(Species::Quagsire))
    } else if species.eq_ignore_ascii_case("SPINARAK") {
        Some(rig(Species::Spinarak))
    } else if species.eq_ignore_ascii_case("ARIADOS") {
        Some(rig(Species::Ariados))
    } else if species.eq_ignore_ascii_case("LEDYBA") {
        Some(rig(Species::Ledyba))
    } else if species.eq_ignore_ascii_case("LEDIAN") {
        Some(rig(Species::Ledian))
    } else if species.eq_ignore_ascii_case("NOCTOWL") {
        Some(rig(Species::Noctowl))
    } else if species.eq_ignore_ascii_case("FURRET") {
        Some(rig(Species::Furret))
    } else if species.eq_ignore_ascii_case("SUICUNE") {
        Some(rig(Species::Suicune))
    } else if species.eq_ignore_ascii_case("RAIKOU") {
        Some(rig(Species::Raikou))
    } else if species.eq_ignore_ascii_case("ENTEI") {
        Some(rig(Species::Entei))
    } else if species.eq_ignore_ascii_case("SLOWPOKE") {
        Some(rig(Species::Slowpoke))
    } else if species.eq_ignore_ascii_case("DIGLETT") {
        Some(rig(Species::Diglett))
    } else if species.eq_ignore_ascii_case("ONIX") {
        Some(rig(Species::Onix))
    } else if species.eq_ignore_ascii_case("VOLTORB") {
        Some(rig(Species::Voltorb))
    } else if species.eq_ignore_ascii_case("STARYU") {
        Some(rig(Species::Staryu))
    } else if species.eq_ignore_ascii_case("SENTRET") {
        Some(rig(Species::Sentret))
    } else if species.eq_ignore_ascii_case("HOOTHOOT") {
        Some(rig(Species::Hoothoot))
    } else if species.eq_ignore_ascii_case("BUTTERFREE") {
        Some(rig(Species::Butterfree))
    } else if species.eq_ignore_ascii_case("BEEDRILL") {
        Some(rig(Species::Beedrill))
    } else if species.eq_ignore_ascii_case("ZUBAT") {
        Some(rig(Species::Zubat))
    } else if species.eq_ignore_ascii_case("GOLBAT") {
        Some(rig(Species::Golbat))
    } else if species.eq_ignore_ascii_case("CROBAT") {
        Some(rig(Species::Crobat))
    } else if species.eq_ignore_ascii_case("PSYDUCK") {
        Some(rig(Species::Psyduck))
    } else if species.eq_ignore_ascii_case("GOLDUCK") {
        Some(rig(Species::Golduck))
    } else if species.eq_ignore_ascii_case("RATICATE") {
        Some(rig(Species::Raticate))
    } else if species.eq_ignore_ascii_case("RATTATA") {
        Some(rig(Species::Rattata))
    } else if species.eq_ignore_ascii_case("JIGGLYPUFF") {
        Some(rig(Species::Jigglypuff))
    } else if species.eq_ignore_ascii_case("CLEFAIRY") {
        Some(rig(Species::Clefairy))
    } else if species.eq_ignore_ascii_case("POLIWAG") {
        Some(rig(Species::Poliwag))
    } else if species.eq_ignore_ascii_case("ODDISH") {
        Some(rig(Species::Oddish))
    } else if species.eq_ignore_ascii_case("PIDGEY") {
        Some(rig(Species::Pidgey))
    } else if species.eq_ignore_ascii_case("GEODUDE") {
        Some(rig(Species::Geodude))
    } else if species.eq_ignore_ascii_case("SQUIRTLE") {
        Some(rig(Species::Squirtle))
    } else if species.eq_ignore_ascii_case("CHARMANDER") {
        Some(rig(Species::Charmander))
    } else if species.eq_ignore_ascii_case("BULBASAUR") {
        Some(rig(Species::Bulbasaur))
    } else if species.eq_ignore_ascii_case("PIKACHU") {
        Some(rig(Species::Pikachu))
    } else if species.eq_ignore_ascii_case("SNORLAX") {
        Some(rig(Species::Snorlax))
    } else if species.eq_ignore_ascii_case("LAPRAS") {
        Some(rig(Species::Lapras))
    } else if species.eq_ignore_ascii_case("CHIKORITA") {
        Some(rig(Species::Chikorita))
    } else if species.eq_ignore_ascii_case("BAYLEEF") {
        Some(rig(Species::Bayleef))
    } else if species.eq_ignore_ascii_case("MEGANIUM") {
        Some(rig(Species::Meganium))
    } else if species.eq_ignore_ascii_case("CYNDAQUIL") {
        Some(rig(Species::Cyndaquil))
    } else if species.eq_ignore_ascii_case("TOTODILE") {
        Some(rig(Species::Totodile))
    } else if species.eq_ignore_ascii_case("GENGAR") {
        Some(rig(Species::Gengar))
    } else if species.eq_ignore_ascii_case("SUDOWOODO") {
        Some(rig(Species::Sudowoodo))
    } else if species.eq_ignore_ascii_case("GYARADOS") {
        Some(rig(Species::Gyarados))
    } else if species.eq_ignore_ascii_case("WOOPER") {
        Some(rig(Species::Wooper))
    } else if species.eq_ignore_ascii_case("SPEAROW") {
        Some(rig(Species::Spearow))
    } else {
        None
    }
}

impl SpeciesRig {
    /// Call through the owner's per-species OnceLock. No load or decode work is
    /// required after the immutable rig and GPU assets have been cached.
    pub fn parse(species: Species, bytes: &[u8]) -> Result<Self, String> {
        validate_container(bytes)?;
        let gltf =
            gltf::Gltf::from_slice(bytes).map_err(|e| format!("invalid species GLB: {e}"))?;
        let binary = gltf.blob.as_deref().ok_or("missing species binary data")?;
        let joint_count = species.joint_names().len();
        if gltf.buffers().len() != 1
            || gltf.scenes().len() != 1
            || gltf.meshes().len() != 1
            || gltf.skins().len() != 1
            || gltf.nodes().len() != joint_count + 1
            || gltf.animations().len() != 3
            || gltf.materials().len() == 0
            || gltf.materials().len() > species.part_count()
            || gltf.accessors().len() > MAX_ACCESSORS
            || gltf.views().len() > MAX_ACCESSORS
            || gltf.images().len() != 0
            || gltf.textures().len() != 0
            || gltf.cameras().len() != 0
            || gltf.extensions_used().next().is_some()
            || gltf.extensions_required().next().is_some()
        {
            return Err("unsupported species scene/schema counts or resources".into());
        }
        let buffer = gltf.buffers().next().unwrap();
        if !matches!(buffer.source(), gltf::buffer::Source::Bin)
            || buffer.length() > binary.len()
            || binary.len() - buffer.length() > 3
        {
            return Err("species requires one embedded buffer".into());
        }
        validate_accessors(&gltf, buffer.length())?;
        let scene = gltf.scenes().next().unwrap();
        if scene.name() != Some(species.name())
            || scene.nodes().map(|n| n.index()).collect::<Vec<_>>() != [0, joint_count]
            || gltf.default_scene().map(|s| s.index()) != Some(0)
        {
            return Err("species requires named skeleton and mesh scene roots".into());
        }

        let materials = gltf
            .materials()
            .map(read_material)
            .collect::<Result<Vec<_>, _>>()?;
        let joints = read_joints(species, &gltf, binary)?;
        let mesh = gltf.meshes().next().unwrap();
        if mesh.name() != Some(species.name())
            || mesh.weights().is_some()
            || mesh.primitives().len() != species.part_count()
        {
            return Err("invalid species anatomy mesh".into());
        }

        let mut neutral = SurfaceMeshData::default();
        let mut source_normals = Vec::new();
        let mut joint_indices = Vec::new();
        let mut joint_weights = Vec::new();
        let mut anatomy = Vec::new();
        let mut names = HashSet::new();
        let mut digest = Sha256::new();
        for primitive in mesh.primitives() {
            validate_primitive(&primitive)?;
            let positions_count = primitive.get(&Semantic::Positions).unwrap().count();
            let indices_count = primitive.indices().unwrap().count();
            if neutral.positions.len() + positions_count > MAX_VERTICES
                || neutral.indices.len() + indices_count > MAX_INDICES
            {
                return Err("species total geometry exceeds allocation bounds".into());
            }
            let meta: PartExtras = read_extras(primitive.extras())?;
            if meta.part.is_empty() || meta.part.len() > 160 || !names.insert(meta.part.clone()) {
                return Err("species anatomy names must be bounded and unique".into());
            }
            let material = primitive
                .material()
                .index()
                .ok_or("missing species material")?;
            let reader = primitive.reader(|_| Some(binary));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or("missing positions")?
                .collect();
            let normals: Vec<_> = reader.read_normals().ok_or("missing normals")?.collect();
            let indices: Vec<_> = reader
                .read_indices()
                .ok_or("missing indices")?
                .into_u32()
                .collect();
            let ids: Vec<_> = reader
                .read_joints(0)
                .ok_or("missing joints")?
                .into_u16()
                .collect();
            let Some(ReadWeights::U8(weights)) = reader.read_weights(0) else {
                return Err("species requires exact normalized u8 weights".into());
            };
            let weights: Vec<_> = weights.collect();
            if positions.len() != positions_count
                || normals.len() != positions_count
                || ids.len() != positions_count
                || weights.len() != positions_count
                || indices.len() != indices_count
                || positions
                    .iter()
                    .flatten()
                    .chain(normals.iter().flatten())
                    .any(|v| !v.is_finite())
                || positions.iter().flatten().any(|v| v.abs() > 4.0)
                || normals
                    .iter()
                    .any(|n| !(0.99..=1.01).contains(&Vec3::from_array(*n).length_squared()))
                || indices.iter().any(|&i| i as usize >= positions_count)
                || ids.iter().zip(&weights).any(|(js, ws)| {
                    ws.iter().map(|&w| u16::from(w)).sum::<u16>() != 255
                        || js
                            .iter()
                            .zip(ws)
                            .any(|(&j, &w)| j as usize >= joint_count || (w == 0 && j != 0))
                })
            {
                return Err("invalid species geometry or skin influences".into());
            }
            // Same digest order as the surviving source/generator. No rounded
            // source JSON, duplicate neutral asset or lossy conversion needed.
            for value in positions.iter().flatten().chain(normals.iter().flatten()) {
                digest.update(value.to_le_bytes());
            }
            for value in &indices {
                digest.update(value.to_le_bytes());
            }
            for value in materials[material].color {
                digest.update(value.to_le_bytes());
            }
            let vertex_start = neutral.positions.len();
            let index_start = neutral.indices.len();
            neutral.positions.extend(positions);
            // The prior prop reader supplied neutral UVs for every vertex.
            neutral
                .uvs
                .extend(std::iter::repeat_n([0.0, 0.0], positions_count));
            neutral.normals.extend(
                normals
                    .iter()
                    .map(|n| Vec3::from_array(*n).normalize().to_array()),
            );
            source_normals.extend(normals);
            neutral.colors.extend(std::iter::repeat_n(
                materials[material].color,
                positions_count,
            ));
            neutral
                .indices
                .extend(indices.into_iter().map(|i| i + vertex_start as u32));
            joint_indices.extend(ids);
            // glTF normalized unsigned byte conversion, without reweighting.
            joint_weights.extend(weights.into_iter().map(|w| w.map(|v| f32::from(v) / 255.0)));
            anatomy.push(Anatomy {
                name: meta.part,
                vertices: vertex_start..neutral.positions.len(),
                indices: index_start..neutral.indices.len(),
                material,
            });
        }
        if format!("{:x}", digest.finalize()) != species.neutral_digest() {
            return Err("species neutral geometry differs from the reviewed source".into());
        }
        let clips = read_clips(species, &gltf, binary)?;
        let neutral_bounds = point_bounds(neutral.positions.iter().copied().map(Vec3::from_array));
        if neutral_bounds.0.y.abs() > 0.0001
            || (neutral_bounds.1 - neutral_bounds.0).min_element() < 0.01
        {
            return Err("species must be volumetric and floor rooted".into());
        }
        let mut rig = Self {
            species,
            neutral,
            source_normals,
            joint_indices,
            joint_weights,
            anatomy,
            materials,
            joints,
            clips,
            neutral_bounds,
            animated_bounds: neutral_bounds,
        };
        rig.animated_bounds = rig.compute_envelope();
        Ok(rig)
    }

    pub fn clip(&self, clip: Clip) -> &SpeciesClip {
        &self.clips[clip.index()]
    }

    /// Fully overwrites caller-owned storage with local TRS, including root.
    /// Idle normally loops; attack/hit normally clamp to cue-relative seconds.
    /// Caller controls playback explicitly. Untargeted bind translations remain
    /// exact, root stays identity, and closed endpoints return exact bind TRS.
    pub fn sample_into(
        &self,
        clip: Clip,
        seconds: f32,
        playback: Playback,
        pose: &mut [Transform],
    ) -> Result<(), &'static str> {
        if !seconds.is_finite() || pose.len() != self.joints.len() {
            return Err("species sample needs finite time and exact joint storage");
        }
        let clip = self.clip(clip);
        let time = match playback {
            Playback::Loop => seconds.rem_euclid(clip.duration),
            Playback::Clamp => seconds.clamp(0.0, clip.duration),
        };
        for (local, joint) in pose.iter_mut().zip(&self.joints) {
            *local = joint.bind;
        }
        if time == 0.0 || time == clip.duration {
            return Ok(());
        }
        let upper = clip.times.partition_point(|&t| t <= time);
        let left = upper.saturating_sub(1);
        let right = upper.min(clip.times.len() - 1);
        let fraction = if left == right {
            0.0
        } else {
            ((time - clip.times[left]) / (clip.times[right] - clip.times[left])).clamp(0.0, 1.0)
        };
        for (local, keys) in pose[1..].iter_mut().zip(&clip.rotations) {
            local.rotation = keys[left]
                .normalize()
                .slerp(keys[right].normalize(), fraction)
                .normalize();
        }
        Ok(())
    }

    /// Audit/envelope utility; GPU playback should write local joint entities.
    /// Result is species-model-space skin matrices, without instance placement.
    pub fn skin_matrices_into(
        &self,
        pose: &[Transform],
        matrices: &mut [Mat4],
    ) -> Result<(), &'static str> {
        if pose.len() != self.joints.len() || matrices.len() != self.joints.len() {
            return Err("species matrices need exact joint storage");
        }
        for (i, joint) in self.joints.iter().enumerate() {
            let local = pose[i].to_matrix();
            matrices[i] = joint
                .parent
                .map_or(local, |parent| matrices[parent] * local);
        }
        for (matrix, joint) in matrices.iter_mut().zip(&self.joints) {
            *matrix *= joint.inverse_bind;
        }
        Ok(())
    }

    pub(crate) fn skin_point(&self, index: usize, matrices: &[Mat4]) -> Vec3 {
        let position = Vec3::from_array(self.neutral.positions[index]);
        self.joint_indices[index]
            .iter()
            .zip(self.joint_weights[index])
            .fold(Vec3::ZERO, |p, (&j, w)| {
                p + matrices[j as usize].transform_point3(position) * w
            })
    }

    fn compute_envelope(&self) -> (Vec3, Vec3) {
        // Hard normals and paint seams duplicate render vertices. Their skin
        // motion is identical when position, joint IDs and weights have the
        // same bits; evaluate that motion once without changing any extrema.
        let mut seen: bevy::platform::collections::HashSet<_> = Default::default();
        let vertices = (0..self.neutral.positions.len()).filter(|&i| seen.insert((
            self.neutral.positions[i].map(f32::to_bits), self.joint_indices[i],
            self.joint_weights[i].map(f32::to_bits),
        ))).collect::<Vec<_>>();
        self.compute_envelope_vertices(&vertices)
    }

    fn compute_envelope_vertices(&self, vertices: &[usize]) -> (Vec3, Vec3) {
        self.compute_adjusted_envelope(vertices, &[], &Clip::ALL)
    }

    /// A renderer can compose a fixed resting pose with the authored idle
    /// without changing the canonical battle clips or their source clock.
    pub(crate) fn idle_bounds_with_joint_offsets(&self, offsets: &[(usize, Quat)]) -> (Vec3, Vec3) {
        let vertices=(0..self.neutral.positions.len()).collect::<Vec<_>>();
        self.compute_adjusted_envelope(&vertices, offsets, &[Clip::Idle])
    }

    fn compute_adjusted_envelope(&self, vertices: &[usize], offsets: &[(usize, Quat)], clips: &[Clip]) -> (Vec3, Vec3) {
        let _trace = crate::loading_trace::span("actor.animation_envelope", || serde_json::json!({
            "species":self.species.name(),"render_vertices":self.neutral.positions.len(),
            "evaluated_vertices":vertices.len()}));
        let initial_bounds=if offsets.is_empty() {self.neutral_bounds}
            else {(Vec3::splat(f32::INFINITY),Vec3::splat(f32::NEG_INFINITY))};
        let mut bounds = initial_bounds;
        let mut pose = vec![Transform::IDENTITY; self.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; self.joints.len()];
        for &which in clips {
            let clip = self.clip(which);
            let mut angular_speed = vec![0.0_f32; self.joints.len()];
            for (joint, keys) in clip.rotations.iter().enumerate() {
                angular_speed[joint + 1] = keys
                    .windows(2)
                    .zip(clip.times.windows(2))
                    .map(|(q, t)| {
                        let a = q[0].as_dquat().normalize();
                        let mut b = q[1].as_dquat().normalize();
                        if a.dot(b) < 0.0 {
                            b = -b;
                        }
                        // 4*tan(angle/4)/dt bounds both true slerp speed and
                        // normalized-lerp's maximum speed (glam's near-zero path).
                        let speed =
                            4.0 * (a - b).length() / (a + b).length() / f64::from(t[1] - t[0]);
                        (speed * 1.01) as f32
                    })
                    .fold(0.0, f32::max);
            }
            // A rotating ancestor moves an influenced vertex at <= omega*r.
            // Triangle inequality bounds r for every possible intervening pose.
            // We sum nonnegative skin weights before taking max vertex speed.
            let mut maximum_speed = 0.0_f32;
            for &i in vertices {
                let point = Vec3::from_array(self.neutral.positions[i]);
                let mut speed = 0.0;
                for (&id, weight) in self.joint_indices[i].iter().zip(self.joint_weights[i]) {
                    if weight == 0.0 {
                        continue;
                    }
                    let mut joint = id as usize;
                    let mut radius = self.joints[joint]
                        .inverse_bind
                        .transform_point3(point)
                        .length();
                    let mut influence_speed = 0.0;
                    loop {
                        influence_speed += angular_speed[joint] * radius;
                        let Some(parent) = self.joints[joint].parent else {
                            break;
                        };
                        radius += self.joints[joint].bind.translation.length();
                        joint = parent;
                    }
                    speed += weight * influence_speed;
                }
                maximum_speed = maximum_speed.max(speed);
            }
            // Long appendages and Spearow's early wing spread need a finer cached
            // envelope to keep the conservative speed bound from enlarging the
            // whole camera box. Thin Unown glyphs need the same tighter padding.
            // This runs once when the GLB is loaded; frame
            // playback still uses the authored keys and original cue clock.
            let envelope_hz = match self.species {
                Species::Chikorita | Species::Bayleef | Species::Meganium | Species::Spearow | Species::Unown(_) => 90.0,
                _ => ENVELOPE_HZ,
            };
            let steps = (clip.duration * envelope_hz).ceil() as usize;
            let margin =
                Vec3::splat(maximum_speed * (clip.duration / steps as f32) * 0.5 + FLOAT_MARGIN);
            let mut clip_bounds = initial_bounds;
            for frame in 0..=steps {
                let time = clip.duration * (frame as f32 / steps as f32);
                self.sample_into(which, time, Playback::Clamp, &mut pose)
                    .unwrap();
                for &(joint,rotation) in offsets {
                    pose[joint].rotation=(rotation*pose[joint].rotation).normalize();
                }
                self.skin_matrices_into(&pose, &mut matrices).unwrap();
                for &i in vertices {
                    let point = self.skin_point(i, &matrices);
                    clip_bounds.0 = clip_bounds.0.min(point);
                    clip_bounds.1 = clip_bounds.1.max(point);
                }
            }
            bounds.0 = bounds.0.min(clip_bounds.0 - margin);
            bounds.1 = bounds.1.max(clip_bounds.1 + margin);
        }
        bounds
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartExtras {
    part: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClipExtras {
    loop_suggested: bool,
    cue_relative: bool,
}

fn read_extras<T: serde::de::DeserializeOwned>(extras: &gltf::json::Extras) -> Result<T, String> {
    serde_json::from_str(extras.as_ref().ok_or("missing species metadata")?.get())
        .map_err(|e| e.to_string())
}

fn point_bounds(points: impl Iterator<Item = Vec3>) -> (Vec3, Vec3) {
    points.fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(lo, hi), p| (lo.min(p), hi.max(p)),
    )
}

fn validate_container(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 28
        || bytes.len() > MAX_BYTES
        || !bytes.starts_with(b"glTF")
        || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
        || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != bytes.len()
    {
        return Err("species requires a bounded binary GLB v2".into());
    }
    let glb = gltf::binary::Glb::from_slice(bytes).map_err(|e| e.to_string())?;
    let binary = glb.bin.as_deref().ok_or("missing binary chunk")?;
    if glb.json.len() % 4 != 0
        || binary.len() % 4 != 0
        || 28 + glb.json.len() + binary.len() != bytes.len()
    {
        return Err("species requires exactly two aligned GLB chunks".into());
    }
    let json: serde_json::Value = serde_json::from_slice(&glb.json).map_err(|e| e.to_string())?;
    // Undeclared extension data may disappear in gltf builds without features.
    fn has_extensions(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(o) => o
                .iter()
                .any(|(k, v)| k == "extensions" || (k != "extras" && has_extensions(v))),
            serde_json::Value::Array(a) => a.iter().any(has_extensions),
            _ => false,
        }
    }
    if json["asset"]["version"] != "2.0"
        || json["asset"].get("minVersion").is_some_and(|v| v != "2.0")
        || has_extensions(&json)
    {
        return Err("unsupported species glTF version or extension".into());
    }
    Ok(())
}

fn validate_accessors(gltf: &gltf::Gltf, binary_len: usize) -> Result<(), String> {
    for view in gltf.views() {
        if view.buffer().index() != 0
            || view.stride().is_some()
            || view.offset() % 4 != 0
            || view
                .offset()
                .checked_add(view.length())
                .is_none_or(|end| end > binary_len)
        {
            return Err(
                "species buffer views must be tightly packed, aligned and in bounds".into(),
            );
        }
    }
    for accessor in gltf.accessors() {
        let view = accessor
            .view()
            .ok_or("species accessor needs a buffer view")?;
        let valid_type = matches!(
            (
                accessor.data_type(),
                accessor.dimensions(),
                accessor.normalized()
            ),
            (
                DataType::F32,
                Dimensions::Scalar | Dimensions::Vec3 | Dimensions::Vec4 | Dimensions::Mat4,
                false
            ) | (DataType::U16, Dimensions::Scalar, false)
                | (DataType::U8, Dimensions::Vec4, _)
        );
        if !valid_type
            || accessor.sparse().is_some()
            || accessor.offset() % 4 != 0
            || accessor.count() == 0
            || accessor.count() > MAX_INDICES
            || accessor
                .count()
                .checked_mul(accessor.size())
                .and_then(|n| n.checked_add(accessor.offset()))
                .is_none_or(|end| end > view.length())
        {
            return Err("unsupported species accessor encoding or bounds".into());
        }
    }
    Ok(())
}

fn read_material(material: gltf::Material<'_>) -> Result<Material, String> {
    let pbr = material.pbr_metallic_roughness();
    let color = pbr.base_color_factor();
    let name = material.name().ok_or("species materials must be named")?;
    if name.is_empty()
        || name.len() > 160
        || color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || pbr.metallic_factor() != 0.0
        || pbr.roughness_factor() != 1.0
        || pbr.base_color_texture().is_some()
        || pbr.metallic_roughness_texture().is_some()
        || material.normal_texture().is_some()
        || material.occlusion_texture().is_some()
        || material.emissive_texture().is_some()
        || material.emissive_factor() != [0.0; 3]
        || material.double_sided()
        || material.alpha_cutoff().is_some()
        || material.alpha_mode() == gltf::material::AlphaMode::Mask
        || (material.alpha_mode() == gltf::material::AlphaMode::Opaque && color[3] != 1.0)
    {
        return Err(
            "species requires named, untextured metallic=0 roughness=1 RGBA materials".into(),
        );
    }
    Ok(Material {
        name: name.to_owned(),
        color,
        alpha_mode: material.alpha_mode(),
    })
}

fn validate_primitive(primitive: &gltf::Primitive<'_>) -> Result<(), String> {
    if primitive.mode() != gltf::mesh::Mode::Triangles
        || primitive.morph_targets().next().is_some()
        || primitive.attributes().len() != 4
    {
        return Err("species requires indexed POSITION/NORMAL/JOINTS_0/WEIGHTS_0 triangles".into());
    }
    let expected = [
        (Semantic::Positions, DataType::F32, Dimensions::Vec3, false),
        (Semantic::Normals, DataType::F32, Dimensions::Vec3, false),
        (Semantic::Joints(0), DataType::U8, Dimensions::Vec4, false),
        (Semantic::Weights(0), DataType::U8, Dimensions::Vec4, true),
    ];
    let count = primitive
        .get(&Semantic::Positions)
        .ok_or("missing species positions")?
        .count();
    if count < 3 || count > MAX_VERTICES {
        return Err("species primitive exceeds vertex bounds".into());
    }
    for (semantic, data_type, dimensions, normalized) in expected {
        let a = primitive
            .get(&semantic)
            .ok_or("missing species skin attribute")?;
        if a.data_type() != data_type
            || a.dimensions() != dimensions
            || a.normalized() != normalized
            || a.count() != count
        {
            return Err("invalid species skin attribute schema".into());
        }
    }
    let indices = primitive.indices().ok_or("missing species indices")?;
    if indices.data_type() != DataType::U16
        || indices.dimensions() != Dimensions::Scalar
        || indices.count() % 3 != 0
        || indices.count() > MAX_INDICES
        || indices.normalized()
    {
        return Err("invalid species triangle indices".into());
    }
    Ok(())
}

fn read_joints(species: Species, gltf: &gltf::Gltf, binary: &[u8]) -> Result<Vec<Joint>, String> {
    let count = species.joint_names().len();
    let nodes: Vec<_> = gltf.nodes().collect();
    let skin = gltf.skins().next().unwrap();
    if skin.name() != Some(species.name())
        || skin.skeleton().map(|n| n.index()) != Some(0)
        || skin.joints().map(|n| n.index()).collect::<Vec<_>>() != (0..count).collect::<Vec<_>>()
    {
        return Err("species skin must preserve named authored joint order".into());
    }
    let accessor = skin
        .inverse_bind_matrices()
        .ok_or("missing inverse bind matrices")?;
    if accessor.data_type() != DataType::F32
        || accessor.dimensions() != Dimensions::Mat4
        || accessor.count() != count
    {
        return Err("invalid inverse bind matrix schema".into());
    }
    let inverse: Vec<_> = skin
        .reader(|_| Some(binary))
        .read_inverse_bind_matrices()
        .ok_or("missing inverse binds")?
        .map(|m| Mat4::from_cols_array_2d(&m))
        .collect();
    if inverse.len() != count {
        return Err("truncated species inverse binds".into());
    }
    let mut parents = vec![None; count + 1];
    for node in &nodes {
        if node.weights().is_some() || node.camera().is_some() {
            return Err("species node morphs and cameras are unsupported".into());
        }
        for child in node.children() {
            if child.index() <= node.index()
                || parents[child.index()].replace(node.index()).is_some()
            {
                return Err("species joint hierarchy is cyclic, shared or out of order".into());
            }
        }
    }
    if parents[..count] != *species.parents()
        || parents[count].is_some()
        || nodes[count].children().next().is_some()
        || nodes[count].mesh().map(|m| m.index()) != Some(0)
        || nodes[count].skin().map(|s| s.index()) != Some(0)
        || nodes[count].name() != Some(species.name())
        || bind_transform(&nodes[count])? != Transform::IDENTITY
    {
        return Err("species requires exact authored hierarchy and identity mesh root".into());
    }
    let mut joints = Vec::with_capacity(count);
    let mut globals = vec![Mat4::IDENTITY; count];
    for i in 0..count {
        let name = format!("{}/{}", species.name(), species.joint_names()[i]);
        if nodes[i].name() != Some(name.as_str())
            || nodes[i].mesh().is_some()
            || nodes[i].skin().is_some()
        {
            return Err(
                "species joint name or geometry ownership differs from the authored rig".into(),
            );
        }
        let bind = bind_transform(&nodes[i])?;
        if i == 0 && bind != Transform::IDENTITY {
            return Err("species root must stay identity".into());
        }
        let local = bind.to_matrix();
        globals[i] = parents[i].map_or(local, |p| globals[p] * local);
        if !inverse[i].is_finite()
            || (globals[i] * inverse[i])
                .to_cols_array()
                .iter()
                .zip(Mat4::IDENTITY.to_cols_array())
                .any(|(a, b)| (a - b).abs() > 0.000002)
        {
            return Err("species inverse bind does not cancel its neutral joint transform".into());
        }
        joints.push(Joint {
            name,
            parent: parents[i],
            bind,
            inverse_bind: inverse[i],
        });
    }
    Ok(joints)
}

fn bind_transform(node: &gltf::Node<'_>) -> Result<Transform, String> {
    let gltf::scene::Transform::Decomposed {
        translation,
        rotation,
        scale,
    } = node.transform()
    else {
        return Err("species requires translation-only TRS bind nodes".into());
    };
    if translation.iter().any(|v| !v.is_finite() || v.abs() > 4.0)
        || rotation != [0.0, 0.0, 0.0, 1.0]
        || scale != [1.0; 3]
    {
        return Err("species requires finite translation-only bind transforms".into());
    }
    Ok(Transform::from_translation(Vec3::from_array(translation)))
}

fn read_clips(
    species: Species,
    gltf: &gltf::Gltf,
    binary: &[u8],
) -> Result<[SpeciesClip; 3], String> {
    let count = species.joint_names().len();
    let mut clips = Vec::new();
    for (animation, kind) in gltf.animations().zip(Clip::ALL) {
        let name = format!("{}.{}", species.name(), kind.suffix());
        let metadata: ClipExtras = read_extras(animation.extras())?;
        if animation.name() != Some(name.as_str())
            || animation.channels().count() != count - 1
            || animation.samplers().count() != count - 1
            || metadata.loop_suggested != (kind == Clip::Idle)
            || metadata.cue_relative != (kind != Clip::Idle)
        {
            return Err("species requires authored idle/attack/hit clip semantics".into());
        }
        let mut times = Vec::new();
        let mut rotations = vec![Vec::new(); count - 1];
        for channel in animation.channels() {
            let target = channel.target().node().index();
            if target == 0
                || target >= count
                || !rotations[target - 1].is_empty()
                || channel.target().property() != Property::Rotation
                || channel.sampler().interpolation() != Interpolation::Linear
            {
                return Err(
                    "species clips require one LINEAR rotation channel per articulated joint"
                        .into(),
                );
            }
            let input = channel.sampler().input();
            let output = channel.sampler().output();
            if input.data_type() != DataType::F32
                || input.dimensions() != Dimensions::Scalar
                || input.count() < 2
                || input.count() > MAX_KEYS
                || output.count() != input.count()
                || output.data_type() != DataType::F32
                || output.dimensions() != Dimensions::Vec4
            {
                return Err("invalid species animation accessor schema".into());
            }
            let reader = channel.reader(|_| Some(binary));
            let channel_times: Vec<_> = reader
                .read_inputs()
                .ok_or("missing species key times")?
                .collect();
            if channel_times.len() != input.count()
                || channel_times[0] != 0.0
                || channel_times
                    .iter()
                    .any(|t| !t.is_finite() || *t < 0.0 || *t > MAX_DURATION)
                || channel_times.windows(2).any(|t| t[1] - t[0] < 0.0001)
            {
                return Err(
                    "species key times must be finite and strictly increasing from zero".into(),
                );
            }
            if times.is_empty() {
                times = channel_times;
            } else if times != channel_times {
                return Err("species clip channels must share one authored clock".into());
            }
            let Some(gltf::animation::util::ReadOutputs::Rotations(values)) = reader.read_outputs()
            else {
                return Err("missing species rotation keys".into());
            };
            let keys: Vec<_> = values.into_f32().map(Quat::from_array).collect();
            if keys.len() != times.len()
                || keys
                    .iter()
                    .any(|q| !q.is_finite() || (q.length_squared() - 1.0).abs() > 0.000002)
                || keys.first() != Some(&Quat::IDENTITY)
                || keys.last() != Some(&Quat::IDENTITY)
            {
                return Err(
                    "species rotation keys must be finite unit quaternions with neutral endpoints"
                        .into(),
                );
            }
            rotations[target - 1] = keys;
        }
        if rotations.iter().any(Vec::is_empty) {
            return Err("missing species articulated joint channel".into());
        }
        clips.push(SpeciesClip {
            name,
            duration: *times.last().ok_or("empty species clip")?,
            times,
            rotations,
            loop_suggested: metadata.loop_suggested,
            cue_relative: metadata.cue_relative,
            extras: animation.extras().as_ref().unwrap().get().to_owned(),
        });
    }
    clips
        .try_into()
        .map_err(|_| "expected three species clips".into())
}

#[cfg(test)]
#[path = "species_rig_tests.rs"]
mod tests;
