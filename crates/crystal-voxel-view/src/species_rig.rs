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
    /// Runtime articulation of an existing named source mesh.
    Procedural(&'static str),
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
    Horsea,
    Seadra,
    Kingdra,
    Goldeen,
    Seaking,
    Magikarp,
    Larvitar,
    Pupitar,
    Tyranitar,
    Quilava,
    Typhlosion,
    Charmeleon,
    Charizard,
    Graveler,
    Golem,
    Omanyte,
    Omastar,
    Aerodactyl,
    Dratini,
    Dragonair,
    Dragonite,
    Clefable,
    Wigglytuff,
    Sandshrew,
    Sandslash,
    Politoed,
    Poliwrath,
    Poliwhirl,
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
    Bellsprout,
    Weepinbell,
    Victreebel,
    Gloom,
    Vileplume,
    Raichu,
    Ekans,
    Arbok,
    Steelix,
    Pineco,
    Forretress,
    Kabuto,
    Kabutops,
    Electrode,
    Onix,
    Diglett,
    Dugtrio,
    Tentacool,
    Jynx,
    Mankey,
    Primeape,
    Meowth,
    Persian,
    Doduo,
    Dodrio,
    Pidgeotto,
    FarfetchD,
    Venusaur,
    Ivysaur,
    Exeggutor,
    Exeggcute,
    Fearow,
    Pidgeot,
    Tentacruel,
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
    Bellossom,
    Qwilfish,
    Wobbuffet,
    Dunsparce,
    Shuckle,
    Gligar,
    Yanma,
    Murkrow,
    Misdreavus,
    Teddiursa,
    Ursaring,
    Snubbull,
    Granbull,
    Houndour,
    Houndoom,
    Miltank,
    Tauros,
    Delibird,
    Stantler,
    Girafarig,
    Phanpy,
    Marill,
    Natu,
    Remoraid,
    Octillery,
    Swinub,
    Piloswine,
    Sneasel,
    Skarmory,
    Smeargle,
    Moltres,
    Zapdos,
    Articuno,
    Mew,
    Mewtwo,
    Heracross,
    Pinsir,
    Scizor,
    Scyther,
    Starmie,
    Gastly,
    Haunter,
    Eevee,
    Vulpix,
    Ninetales,
    Venonat,
    Venomoth,
    Paras,
    Parasect,
    Metapod,
    Kakuna,
    Caterpie,
    Weedle,
    Krabby,
    Kingler,
    Shellder,
    Cloyster,
    Grimer,
    Muk,
    Ditto,
    Seel,
    Dewgong,
    Drowzee,
    Hypno,
    Abra,
    Alakazam,
    Kadabra,
    Koffing,
    Weezing,
    Wartortle,
    Blastoise,
    Croconaw,
    Feraligatr,
    Cubone,
    Marowak,
    Magnemite,
    Magneton,
    Tangela,
    Kangaskhan,
    Ponyta,
    Rapidash,
    Machop,
    Machoke,
    Machamp,
    Rhyhorn,
    Rhydon,
    Umbreon,
    Jolteon,
    Flareon,
    Espeon,
    Vaporeon,
    Celebi,
    HoOh,
    Lugia,
    Corsola,
    Mantine,
    Porygon,
    Porygon2,
    Chinchou,
    Lanturn,
    Slugma,
    Magcargo,
    Togepi,
    Togetic,
    Hitmonlee,
    Hitmonchan,
    Hitmontop,
    Tyrogue,
    Smoochum,
    Elekid,
    Magby,
    Magmar,
    Electabuzz,
    Slowking,
    Slowbro,
    Lickitung,
    MrMime,
    Chansey,
    Blissey,
    Growlithe,
    Arcanine,
    NidoranF,
    NidoranM,
    Nidorina,
    Nidorino,
    Nidoqueen,
    Nidoking,
    Pichu,
    Cleffa,
    Igglybuff,
    Mareep,
    Flaaffy,
    Ampharos,
    Xatu,
    Azumarill,
    Donphan,
    Sunkern,
    Sunflora,
    Jumpluff,
    Unown(u8),
}

impl Species {
    pub const ALL: [Self; 276] = [
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
        Self::Horsea,
        Self::Seadra,
        Self::Kingdra,
        Self::Goldeen,
        Self::Seaking,
        Self::Magikarp,
        Self::Larvitar,
        Self::Pupitar,
        Self::Tyranitar,
        Self::Quilava,
        Self::Typhlosion,
        Self::Charmeleon,
        Self::Charizard,
        Self::Graveler,
        Self::Golem,
        Self::Omanyte,
        Self::Omastar,
        Self::Aerodactyl,
        Self::Dratini,
        Self::Dragonair,
        Self::Dragonite,
        Self::Clefable,
        Self::Wigglytuff,
        Self::Sandshrew,
        Self::Sandslash,
        Self::Politoed,
        Self::Poliwrath,
        Self::Poliwhirl,
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
        Self::Bellsprout,
        Self::Weepinbell,
        Self::Victreebel,
        Self::Gloom,
        Self::Vileplume,
        Self::Raichu,
        Self::Ekans,
        Self::Arbok,
        Self::Steelix,
        Self::Pineco,
        Self::Forretress,
        Self::Kabuto,
        Self::Kabutops,
        Self::Electrode,
        Self::Onix,
        Self::Diglett,
        Self::Dugtrio,
        Self::Tentacool,
        Self::Jynx,
        Self::Mankey,
        Self::Primeape,
        Self::Meowth,
        Self::Persian,
        Self::Doduo,
        Self::Dodrio,
        Self::Pidgeotto,
        Self::FarfetchD,
        Self::Venusaur,
        Self::Ivysaur,
        Self::Exeggutor,
        Self::Exeggcute,
        Self::Fearow,
        Self::Pidgeot,
        Self::Tentacruel,
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
        Self::Bellossom,
        Self::Qwilfish,
        Self::Wobbuffet,
        Self::Dunsparce,
        Self::Shuckle,
        Self::Gligar,
        Self::Yanma,
        Self::Murkrow,
        Self::Misdreavus,
        Self::Teddiursa,
        Self::Ursaring,
        Self::Snubbull,
        Self::Granbull,
        Self::Houndour,
        Self::Houndoom,
        Self::Miltank,
        Self::Tauros,
        Self::Delibird,
        Self::Stantler,
        Self::Girafarig,
        Self::Phanpy,
        Self::Marill,
        Self::Natu,
        Self::Remoraid,
        Self::Octillery,
        Self::Swinub,
        Self::Piloswine,
        Self::Sneasel,
        Self::Skarmory,
        Self::Smeargle,
        Self::Moltres,
        Self::Zapdos,
        Self::Articuno,
        Self::Mew,
        Self::Mewtwo,
        Self::Heracross,
        Self::Pinsir,
        Self::Scizor,
        Self::Scyther,
        Self::Starmie,
        Self::Gastly,
        Self::Haunter,
        Self::Eevee,
        Self::Vulpix,
        Self::Ninetales,
        Self::Venonat,
        Self::Venomoth,
        Self::Paras,
        Self::Parasect,
        Self::Metapod,
        Self::Kakuna,
        Self::Caterpie,
        Self::Weedle,
        Self::Krabby,
        Self::Kingler,
        Self::Shellder,
        Self::Cloyster,
        Self::Grimer,
        Self::Muk,
        Self::Ditto,
        Self::Seel,
        Self::Dewgong,
        Self::Drowzee,
        Self::Hypno,
        Self::Abra,
        Self::Alakazam,
        Self::Kadabra,
        Self::Koffing,
        Self::Weezing,
        Self::Wartortle,
        Self::Blastoise,
        Self::Croconaw,
        Self::Feraligatr,
        Self::Cubone,
        Self::Marowak,
        Self::Magnemite,
        Self::Magneton,
        Self::Tangela,
        Self::Kangaskhan,
        Self::Ponyta,
        Self::Rapidash,
        Self::Machop,
        Self::Machoke,
        Self::Machamp,
        Self::Rhyhorn,
        Self::Rhydon,
        Self::Umbreon,
        Self::Jolteon,
        Self::Flareon,
        Self::Espeon,
        Self::Vaporeon,
        Self::Celebi,
        Self::HoOh,
        Self::Lugia,
        Self::Corsola,
        Self::Mantine,
        Self::Porygon,
        Self::Porygon2,
        Self::Chinchou,
        Self::Lanturn,
        Self::Slugma,
        Self::Magcargo,
        Self::Togepi,
        Self::Togetic,
        Self::Hitmonlee,
        Self::Hitmonchan,
        Self::Hitmontop,
        Self::Tyrogue,
        Self::Smoochum,
        Self::Elekid,
        Self::Magby,
        Self::Magmar,
        Self::Electabuzz,
        Self::Slowking,
        Self::Slowbro,
        Self::Lickitung,
        Self::MrMime,
        Self::Chansey,
        Self::Blissey,
        Self::Growlithe,
        Self::Arcanine,
        Self::NidoranF,
        Self::NidoranM,
        Self::Nidorina,
        Self::Nidorino,
        Self::Nidoqueen,
        Self::Nidoking,
        Self::Pichu,
        Self::Cleffa,
        Self::Igglybuff,
        Self::Mareep,
        Self::Flaaffy,
        Self::Ampharos,
        Self::Xatu,
        Self::Azumarill,
        Self::Donphan,
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
            Self::Procedural(name) => name,
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
            Self::Horsea => "horsea",
            Self::Seadra => "seadra",
            Self::Kingdra => "kingdra",
            Self::Goldeen => "goldeen",
            Self::Seaking => "seaking",
            Self::Magikarp => "magikarp",
            Self::Larvitar => "larvitar",
            Self::Pupitar => "pupitar",
            Self::Tyranitar => "tyranitar",
            Self::Quilava => "quilava",
            Self::Typhlosion => "typhlosion",
            Self::Charmeleon => "charmeleon",
            Self::Charizard => "charizard",
            Self::Graveler => "graveler",
            Self::Golem => "golem",
            Self::Omanyte => "omanyte",
            Self::Omastar => "omastar",
            Self::Aerodactyl => "aerodactyl",
            Self::Dratini => "dratini",
            Self::Dragonair => "dragonair",
            Self::Dragonite => "dragonite",
            Self::Clefable => "clefable",
            Self::Wigglytuff => "wigglytuff",
            Self::Sandshrew => "sandshrew",
            Self::Sandslash => "sandslash",
            Self::Politoed => "politoed",
            Self::Poliwrath => "poliwrath",
            Self::Poliwhirl => "poliwhirl",
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
            Self::Bellsprout => "bellsprout",
            Self::Weepinbell => "weepinbell",
            Self::Victreebel => "victreebel",
            Self::Gloom => "gloom",
            Self::Vileplume => "vileplume",
            Self::Raichu => "raichu",
            Self::Ekans => "ekans",
            Self::Arbok => "arbok",
            Self::Steelix => "steelix",
            Self::Pineco => "pineco",
            Self::Forretress => "forretress",
            Self::Kabuto => "kabuto",
            Self::Kabutops => "kabutops",
            Self::Electrode => "electrode",
            Self::Onix => "onix",
            Self::Diglett => "diglett",
            Self::Dugtrio => "dugtrio",
            Self::Tentacruel => "tentacruel",
            Self::Tentacool => "tentacool",
            Self::Jynx => "jynx",
            Self::Primeape => "primeape",
            Self::Mankey => "mankey",
            Self::Meowth => "meowth",
            Self::Persian => "persian",
            Self::Doduo => "doduo",
            Self::Dodrio => "dodrio",
            Self::Pidgeotto => "pidgeotto",
            Self::FarfetchD => "farfetch_d",
            Self::Venusaur => "venusaur",
            Self::Ivysaur => "ivysaur",
            Self::Exeggutor => "exeggutor",
            Self::Exeggcute => "exeggcute",
            Self::Fearow => "fearow",
            Self::Pidgeot => "pidgeot",
            Self::Entei => "entei",
            Self::Raikou => "raikou",
            Self::Ledyba => "ledyba",
            Self::Ledian => "ledian",
            Self::Ariados => "ariados",
            Self::Quagsire => "quagsire",
            Self::Hoppip => "hoppip",
            Self::Skiploom => "skiploom",
            Self::Aipom => "aipom",
            Self::Bellossom => "bellossom",
            Self::Qwilfish => "qwilfish",
            Self::Wobbuffet => "wobbuffet",
            Self::Dunsparce => "dunsparce",
            Self::Shuckle => "shuckle",
            Self::Gligar => "gligar",
            Self::Yanma => "yanma",
            Self::Murkrow => "murkrow",
            Self::Misdreavus => "misdreavus",
            Self::Teddiursa => "teddiursa",
            Self::Ursaring => "ursaring",
            Self::Snubbull => "snubbull",
            Self::Granbull => "granbull",
            Self::Houndour => "houndour",
            Self::Houndoom => "houndoom",
            Self::Miltank => "miltank",
            Self::Tauros => "tauros",
            Self::Delibird => "delibird",
            Self::Stantler => "stantler",
            Self::Girafarig => "girafarig",
            Self::Donphan => "donphan",
            Self::Phanpy => "phanpy",
            Self::Azumarill => "azumarill",
            Self::Marill => "marill",
            Self::Xatu => "xatu",
            Self::Natu => "natu",
            Self::Ampharos => "ampharos",
            Self::Flaaffy => "flaaffy",
            Self::Togepi => "togepi",
            Self::Togetic => "togetic",
            Self::Remoraid => "remoraid",
            Self::Octillery => "octillery",
            Self::Swinub => "swinub",
            Self::Piloswine => "piloswine",
            Self::Sneasel => "sneasel",
            Self::Skarmory => "skarmory",
            Self::Smeargle => "smeargle",
            Self::Moltres => "moltres",
            Self::Zapdos => "zapdos",
            Self::Articuno => "articuno",
            Self::Mew => "mew",
            Self::Mewtwo => "mewtwo",
            Self::Heracross => "heracross",
            Self::Pinsir => "pinsir",
            Self::Scizor => "scizor",
            Self::Scyther => "scyther",
            Self::Starmie => "starmie",
            Self::Gastly => "gastly",
            Self::Haunter => "haunter",
            Self::Eevee => "eevee",
            Self::Vulpix => "vulpix",
            Self::Ninetales => "ninetales",
            Self::Venonat => "venonat",
            Self::Venomoth => "venomoth",
            Self::Paras => "paras",
            Self::Parasect => "parasect",
            Self::Metapod => "metapod",
            Self::Kakuna => "kakuna",
            Self::Caterpie => "caterpie",
            Self::Weedle => "weedle",
            Self::Rhydon => "rhydon",
            Self::Drowzee => "drowzee",
            Self::Hypno => "hypno",
            Self::Abra => "abra",
            Self::Alakazam => "alakazam",
            Self::Kadabra => "kadabra",
            Self::Koffing => "koffing",
            Self::Weezing => "weezing",
            Self::Wartortle => "wartortle",
            Self::Blastoise => "blastoise",
            Self::Croconaw => "croconaw",
            Self::Feraligatr => "feraligatr",
            Self::Cubone => "cubone",
            Self::Marowak => "marowak",
            Self::Magnemite => "magnemite",
            Self::Magneton => "magneton",
            Self::Tangela => "tangela",
            Self::Kangaskhan => "kangaskhan",
            Self::Ponyta => "ponyta",
            Self::Rapidash => "rapidash",
            Self::Machop => "machop",
            Self::Machoke => "machoke",
            Self::Machamp => "machamp",
            Self::Rhyhorn => "rhyhorn",
            Self::Dewgong => "dewgong",
            Self::Seel => "seel",
            Self::Ditto => "ditto",
            Self::Muk => "muk",
            Self::Grimer => "grimer",
            Self::Cloyster => "cloyster",
            Self::Shellder => "shellder",
            Self::Kingler => "kingler",
            Self::Krabby => "krabby",
            Self::Umbreon => "umbreon",
            Self::Jolteon => "jolteon",
            Self::Flareon => "flareon",
            Self::Espeon => "espeon",
            Self::Vaporeon => "vaporeon",
            Self::Celebi => "celebi",
            Self::HoOh => "ho_oh",
            Self::Lugia => "lugia",
            Self::Corsola => "corsola",
            Self::Mantine => "mantine",
            Self::Porygon => "porygon",
            Self::Porygon2 => "porygon2",
            Self::Chinchou => "chinchou",
            Self::Lanturn => "lanturn",
            Self::Slugma => "slugma",
            Self::Magcargo => "magcargo",
            Self::Hitmonlee => "hitmonlee",
            Self::Hitmonchan => "hitmonchan",
            Self::Hitmontop => "hitmontop",
            Self::Tyrogue => "tyrogue",
            Self::Smoochum => "smoochum",
            Self::Elekid => "elekid",
            Self::Magby => "magby",
            Self::Magmar => "magmar",
            Self::Electabuzz => "electabuzz",
            Self::Slowking => "slowking",
            Self::Slowbro => "slowbro",
            Self::Lickitung => "lickitung",
            Self::MrMime => "mr__mime",
            Self::Chansey => "chansey",
            Self::Blissey => "blissey",
            Self::Growlithe => "growlithe",
            Self::Arcanine => "arcanine",
            Self::NidoranF => "nidoran_f",
            Self::NidoranM => "nidoran_m",
            Self::Nidorina => "nidorina",
            Self::Nidorino => "nidorino",
            Self::Nidoking => "nidoking",
            Self::Nidoqueen => "nidoqueen",
            Self::Pichu => "pichu",
            Self::Cleffa => "cleffa",
            Self::Igglybuff => "igglybuff",
            Self::Mareep => "mareep",
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
            Self::Procedural(_) => unreachable!("procedural rigs do not use the GLB schema"),
            Self::Diglett => &["root", "body"],
            Self::Dugtrio => &["root", "mole_0", "mole_1", "mole_2"],
            Self::Tentacruel => &["root", "bell", "tentacle_0_base", "tentacle_0_bend", "tentacle_1_base", "tentacle_1_bend", "tentacle_2_base", "tentacle_2_bend", "tentacle_3_base", "tentacle_3_bend", "tentacle_4_base", "tentacle_4_bend", "tentacle_5_base", "tentacle_5_bend", "tentacle_6_base", "tentacle_6_bend", "tentacle_7_base", "tentacle_7_bend", "tentacle_8_base", "tentacle_8_bend", "tentacle_9_base", "tentacle_9_bend", "arm_-1", "arm_1"],
            Self::Tentacool => &["root", "bell", "tentacle_0_base", "tentacle_0_bend", "tentacle_1_base", "tentacle_1_bend"],
            Self::Jynx => &["root", "torso", "neck", "head", "hair_left", "hair_right", "arm_left", "elbow_left", "hand_left", "arm_right", "elbow_right", "hand_right", "finger_left_0", "finger_left_1", "finger_left_2", "finger_left_3", "finger_left_4", "finger_right_0", "finger_right_1", "finger_right_2", "finger_right_3", "finger_right_4"],
            Self::Primeape => &["root", "body", "arm_left", "hand_left", "arm_right", "hand_right", "ear_left", "ear_right", "leg_left", "leg_right"],
            Self::Mankey => &["root", "body", "arm_left", "hand_left", "arm_right", "hand_right", "ear_left", "ear_right", "leg_left", "leg_right", "tail_base", "tail_tip"],
            Self::Meowth => &["root", "body", "neck", "head", "ear_left", "ear_right", "arm_left", "hand_left", "arm_right", "hand_right", "leg_-1", "leg_1", "tail_base", "tail_mid", "tail_tip"],
            Self::Persian => &["root", "body", "neck", "head", "ear_left", "ear_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1", "tail_base", "tail_mid", "tail_tip"],
            Self::Doduo => &["root", "body", "neck_base_0", "neck_tip_0", "head_0", "jaw_0", "neck_base_1", "neck_tip_1", "head_1", "jaw_1", "leg_left", "leg_right"],
            Self::Dodrio => &["root", "body", "neck_base_0", "neck_tip_0", "head_0", "jaw_0", "neck_base_1", "neck_tip_1", "head_1", "jaw_1", "neck_base_2", "neck_tip_2", "head_2", "jaw_2", "leg_left", "leg_right", "tail_0", "tail_1", "tail_2"],
            Self::Pidgeotto => &["root", "body", "head", "wing_left", "wrist_left", "wing_right", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_outer", "tail_middle", "tail_center", "crest"],
            Self::FarfetchD => &["root", "body", "head", "wing_raised", "wing_wrist_raised", "wing_holding", "wing_elbow_holding", "grip", "leek_base", "leek_tip", "leek_leaf_0", "leek_leaf_1", "leek_leaf_2", "tail_0", "tail_1", "tail_2", "crest_0", "crest_1", "crest_2", "leg_left", "leg_right"],
            Self::Venusaur => &["root", "torso", "head", "foreleg_left", "foreleg_right", "hindleg_left", "hindleg_right", "plant_base", "flower_crown", "leaf_base_0", "leaf_tip_0", "leaf_base_1", "leaf_tip_1", "leaf_base_2", "leaf_tip_2", "leaf_base_3", "leaf_tip_3", "leaf_base_4", "leaf_tip_4", "leaf_base_5", "leaf_tip_5", "petal_base_0", "petal_tip_0", "petal_base_1", "petal_tip_1", "petal_base_2", "petal_tip_2", "petal_base_3", "petal_tip_3", "petal_base_4", "petal_tip_4", "flower_heart"],
            Self::Ivysaur => &["root", "torso", "head", "foreleg_left", "foreleg_right", "hindleg_left", "hindleg_right", "plant_base", "bud_tip", "leaf_base_0", "leaf_tip_0", "leaf_base_1", "leaf_tip_1", "leaf_base_2", "leaf_tip_2", "leaf_base_3", "leaf_tip_3", "leaf_base_4", "leaf_tip_4", "leaf_base_5", "leaf_tip_5"],
            Self::Exeggutor => &["root", "trunk_base", "crown_core", "head_0", "head_1", "head_2", "leg_left", "leg_right", "frond_base_0", "frond_tip_0", "frond_base_1", "frond_tip_1", "frond_base_2", "frond_tip_2", "frond_base_3", "frond_tip_3", "frond_base_4", "frond_tip_4", "frond_base_5", "frond_tip_5", "frond_base_6", "frond_tip_6", "frond_base_7", "frond_tip_7", "frond_base_8", "frond_tip_8", "frond_base_9", "frond_tip_9", "frond_base_10", "frond_tip_10"],
            Self::Exeggcute => &["root", "egg_0", "egg_1", "egg_2", "egg_3", "egg_4", "egg_5"],
            Self::Fearow => &["root", "body", "neck_base", "neck_tip", "head", "lower_bill", "wing_left", "wrist_left", "wing_right", "wrist_right", "tail_0", "tail_1", "tail_2", "tail_3", "tail_4", "leg_left", "leg_right", "crown", "mane"],
            Self::Pidgeot => &["root", "body", "neck_base", "neck_tip", "head", "lower_bill", "wing_left", "wrist_left", "wing_right", "wrist_right", "tail_0", "tail_1", "tail_2", "tail_3", "tail_4", "leg_left", "leg_right", "crest_-1", "crest_0", "crest_1"],
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
            Self::Bellossom => &["root", "torso", "neck", "head", "arm_right", "arm_tip_right", "arm_left", "arm_tip_left", "flower_left", "flower_right", "under_0", "inner_0", "outer_0", "under_1", "inner_1", "outer_1", "under_2", "inner_2", "outer_2", "under_3", "inner_3", "outer_3", "under_4", "inner_4", "outer_4"],
            Self::Qwilfish => &["root", "body", "fin_right", "fin_left", "tail_base", "tail_bend", "tail_fan"],
            Self::Wobbuffet => &["root", "torso", "salute_shoulder", "salute_elbow", "salute_hand", "upright_shoulder", "upright_elbow", "upright_hand", "tail_base", "tail_bend", "tail_paddle"],
            Self::Dunsparce => &["root", "body", "wing_left", "wing_right", "tail_base", "tail_bend", "drill"],
            Self::Shuckle => &["root", "limb_right_rear_limb", "limb_left_front_limb", "limb_right_front_limb", "limb_left_rear_limb", "neck_base", "neck_bend", "head"],
            Self::Gligar => &["root", "torso", "head", "ear_left", "ear_right", "tongue", "arm_left", "pincer_left", "arm_right", "pincer_right", "tail_base", "tail_middle", "tail_tip", "venom_bulb", "stinger"],
            Self::Yanma => &["root", "thorax", "head", "wing_left_front", "wing_left_rear", "wing_right_front", "wing_right_rear", "leg_left_0", "leg_left_1", "leg_left_2", "leg_right_0", "leg_right_1", "leg_right_2", "abdomen_base", "abdomen_bend", "tail_cap"],
            Self::Murkrow => &["root", "torso", "head", "lower_bill", "wing_left", "wing_right", "tail_base", "broom_fan", "crest_center", "crest_left", "crest_right"],
            Self::Misdreavus => &["root", "body", "head", "lock_crown", "lock_left_upper", "lock_left_lower", "lock_right_upper", "lock_right_lower", "lock_rear", "curl_crown", "curl_left_upper", "curl_left_lower", "curl_right_upper", "curl_right_lower", "curl_rear"],
            Self::Teddiursa => &["root", "torso", "head", "arm_left", "hand_left", "arm_right", "hand_right", "tail", "ear_left", "ear_right", "leg_left", "leg_right"],
            Self::Ursaring => &["root", "torso", "head", "arm_left", "hand_left", "arm_right", "hand_right", "tail", "ear_left", "ear_right", "leg_left", "leg_right"],
            Self::Snubbull => &["root", "torso", "head", "jaw", "arm_left", "hand_left", "arm_right", "hand_right", "tail", "ear_left", "ear_right", "leg_left", "leg_right"],
            Self::Granbull => &["root", "torso", "head", "jaw", "arm_left", "hand_left", "arm_right", "hand_right", "tail", "ear_left", "ear_right", "leg_left", "leg_right"],
            Self::Houndour => &["root", "torso", "neck", "head", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear", "tail_base", "tail_bend", "tail_tip", "ear_left", "ear_right"],
            Self::Houndoom => &["root", "torso", "neck", "head", "jaw", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear", "tail_base", "tail_bend", "tail_tip"],
            Self::Miltank => &["root", "torso", "head", "arm_left", "hand_left", "arm_right", "hand_right", "ear_left", "ear_right", "tail_base", "tail_bend", "tail_ball", "leg_left", "leg_right"],
            Self::Tauros => &["root", "torso", "mane", "head", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear", "tail_0_base", "tail_0_bend", "tail_0_tuft", "tail_1_base", "tail_1_bend", "tail_1_tuft", "tail_2_base", "tail_2_bend", "tail_2_tuft"],
            Self::Delibird => &["root", "torso", "head", "lower_bill", "wing_left", "held_wing_and_sack", "crest_left", "crest_right"],
            Self::Stantler => &["root", "torso", "neck", "head", "ear_left", "ear_right", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear", "tail_base", "tail_curl"],
            Self::Girafarig => &["root", "torso", "neck", "head", "ear_left", "ear_right", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear", "tail_base", "tail_bend", "tail_head"],
            Self::Donphan => &["root", "torso", "head", "trunk_base", "trunk_tip", "ear_left", "ear_right", "tail_base", "tail_tip", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear"],
            Self::Phanpy => &["root", "torso", "head", "trunk_base", "trunk_tip", "ear_left", "ear_right", "tail_base", "tail_tip", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear"],
            Self::Azumarill => &["root", "torso", "head", "ear_left", "ear_left_tip", "ear_right", "ear_right_tip", "arm_left", "arm_right", "tail_base", "tail_bend", "tail_ball"],
            Self::Marill => &["root", "torso", "head", "ear_left", "ear_left_tip", "ear_right", "ear_right_tip", "arm_left", "arm_right", "tail_base", "tail_bend", "tail_ball"],
            Self::Xatu => &["root", "torso", "head", "wing_left", "wing_right", "streamer_left", "streamer_left_tip", "streamer_right", "streamer_right_tip", "ankle_left", "ankle_right"],
            Self::Natu => &["root", "torso", "head", "wing_left", "wing_right", "crest", "tail_left", "tail_right", "ankle_left", "ankle_right"],
            Self::Ampharos => &["root", "torso", "neck", "head", "arm_left", "arm_right", "tail_base", "tail_bend", "tail_beacon"],
            Self::Flaaffy => &["root", "torso", "neck", "head", "ear_left", "ear_right", "arm_left", "arm_right", "tail_base", "tail_bend", "tail_beacon", "leg_left", "leg_right"],
            Self::Togepi => &["root", "shell", "head", "arm_left", "arm_right", "leg_left", "leg_right"],
            Self::Togetic => &["root", "torso", "neck", "head", "wing_left", "wing_right", "arm_left", "arm_right", "leg_left", "leg_right"],
            Self::Remoraid => &["root", "body", "tail", "dorsal", "pelvic", "pectoral_left", "pectoral_right"],
            Self::Octillery => &["root", "mantle", "siphon", "tentacle_0", "tentacle_1", "tentacle_2", "tentacle_3", "tentacle_4", "tentacle_5", "tentacle_6", "tentacle_7"],
            Self::Swinub => &["root", "body", "head", "fringe", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear"],
            Self::Piloswine => &["root", "body", "head", "ear_left", "ear_right", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear"],
            Self::Sneasel => &["root", "torso", "neck", "head", "ear", "plume", "tail", "arm_left", "wrist_left", "arm_right", "wrist_right", "leg_left", "leg_right"],
            Self::Skarmory => &["root", "torso", "neck", "head", "wing_left", "wing_right", "tail", "leg_left", "leg_right", "mandible"],
            Self::Smeargle => &["root", "torso", "neck", "head", "ear_left", "ear_right", "beret", "arm_left", "wrist_left", "arm_right", "brush_hold", "leg_left", "leg_right"],
            Self::Moltres => &["root", "torso", "neck", "head", "wing_left", "wing_right", "fire_left", "fire_right", "crown", "tail", "leg_left", "leg_right"],
            Self::Zapdos => &["root", "torso", "neck", "head", "wing_left", "wing_right", "tail", "leg_left", "leg_right"],
            Self::Articuno => &["root", "torso", "neck", "head", "wing_left", "wing_right", "tail_root", "tail_curve", "tail_tip", "leg_left", "leg_right"],
            Self::Mew => &["root", "torso", "neck", "head", "arm_left", "arm_right", "leg_left", "leg_right", "tail_root", "tail_curve", "tail_tip", "ear_left", "ear_right"],
            Self::Mewtwo => &["root", "torso", "neck", "head", "upper_left", "forearm_left", "hand_left", "upper_right", "forearm_right", "hand_right", "hip_left", "knee_left", "hip_right", "knee_right", "tail_root", "tail_curve", "tail_tip"],
            Self::Heracross => &["root", "body", "thorax", "head", "arm_left", "forearm_left", "arm_right", "forearm_right", "antenna_left", "antenna_right", "leg_left", "leg_right"],
            Self::Pinsir => &["root", "abdomen", "carapace", "pincer_left", "pincer_right", "arm_left", "forearm_left", "hand_left", "arm_right", "forearm_right", "hand_right", "leg_left", "leg_right"],
            Self::Scizor => &["root", "thorax", "abdomen", "neck", "head", "arm_left", "forearm_left", "pincer_left", "jaw_left", "arm_right", "forearm_right", "pincer_right", "jaw_right", "wing_left", "wing_right", "leg_left", "leg_right"],
            Self::Scyther => &["root", "thorax", "abdomen", "neck", "head", "arm_left", "scythe_left", "arm_right", "scythe_right", "wing_left", "wing_right", "leg_left", "leg_right"],
            Self::Starmie => &["root", "core", "front_layer", "front_arm_0_base", "front_arm_0_tip", "front_arm_1_base", "front_arm_1_tip", "front_arm_2_base", "front_arm_2_tip", "front_arm_3_base", "front_arm_3_tip", "front_arm_4_base", "front_arm_4_tip", "back_layer", "back_arm_0_base", "back_arm_0_tip", "back_arm_1_base", "back_arm_1_tip", "back_arm_2_base", "back_arm_2_tip", "back_arm_3_base", "back_arm_3_tip", "back_arm_4_base", "back_arm_4_tip"],
            Self::Gastly => &["root", "face_core", "gas_halo", "gas_sector_0", "gas_sector_1", "gas_sector_2", "gas_sector_3", "gas_sector_4", "gas_sector_5", "gas_sector_6", "gas_sector_7"],
            Self::Haunter => &["root", "face_core", "ear_left", "ear_right", "shroud_left", "shroud_right", "hem_left", "hem_right", "hand_left", "finger_-1_0", "finger_-1_1", "finger_-1_2", "hand_right", "finger_1_0", "finger_1_1", "finger_1_2"],
            Self::Eevee => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_base", "tail_middle", "tail_tip", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Vulpix => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_fan", "tail_0", "tail_1", "tail_2", "tail_3", "tail_4", "tail_5", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Ninetales => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_fan", "tail_0", "tail_1", "tail_2", "tail_3", "tail_4", "tail_5", "tail_6", "tail_7", "tail_8", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Venonat => &["root", "body", "antenna_left", "antenna_right", "arm_left", "arm_right", "hand_left", "hand_right", "leg_left", "leg_right"],
            Self::Venomoth => &["root", "thorax", "head", "abdomen", "wing_left_front", "wing_left_rear", "wing_right_front", "wing_right_rear", "antenna_center", "antenna_left", "antenna_right", "leg_-1_0", "leg_-1_1", "leg_-1_2", "leg_1_0", "leg_1_1", "leg_1_2"],
            Self::Paras => &["root", "body", "head", "claw_left", "claw_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1", "fungus_-1", "fungus_1"],
            Self::Parasect => &["root", "body", "head", "claw_left", "claw_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1", "fungus_0"],
            Self::Metapod => &["root", "shell", "head", "tail_fold"],
            Self::Kakuna => &["root", "shell", "head", "arm_fold_left", "arm_fold_right"],
            Self::Caterpie => &["root", "segment_0", "segment_1", "segment_2", "segment_3", "segment_4", "segment_5", "segment_6", "segment_7", "head", "horn_left", "horn_right"],
            Self::Weedle => &["root", "segment_0", "segment_1", "segment_2", "segment_3", "segment_4", "segment_5", "segment_6", "segment_7", "head", "head_sting", "tail_sting"],
            Self::Rhydon => &["root", "body", "head", "jaw", "arm_left", "hand_left", "arm_right", "hand_right", "leg_left", "leg_right", "tail_base", "tail_tip", "ear_left", "ear_right", "drill"],
            Self::Drowzee => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "ear_left", "ear_right"],
            Self::Hypno => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "ear_left", "ear_right", "pendulum"],
            Self::Abra => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "ear_left", "ear_right", "tail_base", "tail_tip"],
            Self::Alakazam => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "ear_left", "ear_right", "moustache_left", "moustache_right", "spoon_left", "spoon_right"],
            Self::Kadabra => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "ear_left", "ear_right", "moustache_left", "moustache_right", "tail_base", "tail_tip", "spoon"],
            Self::Koffing => &["root", "body", "gas_cloud_0", "gas_lobe_0_0", "gas_lobe_0_1", "gas_lobe_0_2", "gas_lobe_0_3", "gas_cloud_1", "gas_lobe_1_0", "gas_lobe_1_1", "gas_lobe_1_2", "gas_lobe_1_3"],
            Self::Weezing => &["root", "body", "small_orb", "rear_orb", "gas_cloud_0", "gas_lobe_0_0", "gas_lobe_0_1", "gas_lobe_0_2", "gas_lobe_0_3", "gas_cloud_1", "gas_lobe_1_0", "gas_lobe_1_1", "gas_lobe_1_2", "gas_lobe_1_3"],
            Self::Wartortle => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "feather_left", "feather_right", "tail_base", "tail_tip"],
            Self::Blastoise => &["root", "body", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "barrel_left", "barrel_right", "tail_base", "tail_tip"],
            Self::Croconaw => &["root", "body", "head", "jaw", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "tail", "tail_tip"],
            Self::Feraligatr => &["root", "body", "head", "jaw", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "tail", "tail_tip"],
            Self::Cubone => &["root", "body", "neck", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "tail", "tail_tip"],
            Self::Marowak => &["root", "body", "neck", "head", "arm_left", "elbow_left", "wrist_left", "arm_right", "elbow_right", "wrist_right", "leg_left", "leg_right", "tail", "tail_tip"],
            Self::Magnemite => &["root", "body_0", "magnet_left_0", "magnet_right_0", "crown_0", "face_bolt_0_0", "face_bolt_0_1"],
            Self::Magneton => &["root", "body_0", "magnet_left_0", "magnet_right_0", "crown_0", "face_bolt_0_0", "face_bolt_0_1", "body_1", "magnet_left_1", "magnet_right_1", "crown_1", "face_bolt_1_0", "face_bolt_1_1", "body_2", "magnet_left_2", "magnet_right_2", "crown_2", "face_bolt_2_0", "face_bolt_2_1"],
            Self::Tangela => &["root", "body", "vine_00", "vine_01", "vine_02", "vine_03", "vine_04", "vine_05", "vine_06", "vine_07", "vine_08", "vine_09", "vine_10", "vine_11", "vine_12", "vine_13", "vine_14", "vine_15", "vine_16", "vine_17", "vine_18", "vine_19", "vine_20", "vine_21", "vine_22", "vine_23", "vine_24", "vine_25", "vine_26", "vine_27", "vine_28", "vine_29", "vine_30", "vine_31", "vine_32", "vine_33", "vine_34", "vine_35", "vine_36"],
            Self::Kangaskhan => &["root", "body", "head", "jaw", "ear_left", "ear_right", "arm_left", "wrist_left", "arm_right", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip", "baby_body", "baby_head", "baby_left_grip", "baby_right_grip", "baby_ear_left", "baby_ear_right"],
            Self::Ponyta => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_stem", "tail_fire", "leg_-1_front", "leg_-1_rear", "leg_1_front", "leg_1_rear", "mane_fire", "crown_fire"],
            Self::Rapidash => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_stem", "tail_fire", "leg_-1_front", "leg_-1_rear", "leg_1_front", "leg_1_rear", "mane_fire", "crown_fire", "jaw"],
            Self::Machop => &["root", "body", "neck", "head", "leg_left", "leg_right", "arm_upper_left", "elbow_upper_left", "wrist_upper_left", "arm_upper_right", "elbow_upper_right", "wrist_upper_right", "tail"],
            Self::Machoke => &["root", "body", "neck", "head", "leg_left", "leg_right", "arm_upper_left", "elbow_upper_left", "wrist_upper_left", "arm_upper_right", "elbow_upper_right", "wrist_upper_right"],
            Self::Machamp => &["root", "body", "neck", "head", "leg_left", "leg_right", "arm_upper_left", "elbow_upper_left", "wrist_upper_left", "arm_upper_right", "elbow_upper_right", "wrist_upper_right", "arm_lower_left", "elbow_lower_left", "wrist_lower_left", "arm_lower_right", "elbow_lower_right", "wrist_lower_right"],
            Self::Rhyhorn => &["root", "body", "head", "tail", "leg_-1_rear", "leg_-1_front", "leg_1_rear", "leg_1_front", "jaw", "ear_left", "ear_right"],
            Self::Dewgong => &["root", "body", "head", "tail_base", "flipper_left", "flipper_right", "tail_tip", "fluke_left", "fluke_right"],
            Self::Seel => &["root", "body", "head", "tail_base", "flipper_left", "flipper_right", "tail_tip", "fluke_left", "fluke_right", "tongue"],
            Self::Ditto => &["root", "body", "head", "lobe_left", "lobe_right"],
            Self::Muk => &["root", "body", "head", "arm_left", "hand_left", "arm_right", "hand_right", "finger_left_0", "finger_left_1", "finger_left_2", "finger_right_0", "finger_right_1", "finger_right_2"],
            Self::Grimer => &["root", "body", "head", "arm_left", "hand_left", "arm_right", "hand_right", "finger_left_0", "finger_left_1", "finger_left_2", "finger_right_0", "finger_right_1", "finger_right_2"],
            Self::Cloyster => &["root", "body", "valve_left", "valve_right", "mantle"],
            Self::Shellder => &["root", "body", "upper_valve", "lower_valve", "tongue_base", "tongue_fold"],
            Self::Kingler => &["root", "body", "arm_left", "wrist_left", "jaw_left", "arm_right", "wrist_right", "jaw_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Krabby => &["root", "body", "arm_left", "wrist_left", "jaw_left", "arm_right", "wrist_right", "jaw_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Umbreon => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_base", "tail_middle", "tail_tip", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Jolteon => &["root", "body", "neck", "head", "ear_left", "ear_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1", "neck_ruff"],
            Self::Flareon => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_base", "tail_middle", "tail_tip", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1", "neck_ruff"],
            Self::Espeon => &["root", "body", "neck", "head", "ear_left", "ear_right", "tail_base", "tail_middle", "tail_hub", "fork_left", "fork_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Vaporeon => &["root", "body", "neck", "head", "head_fin_left", "head_fin_right", "tail_base", "tail_middle", "tail_tip", "fluke_left", "fluke_right", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1", "neck_frill"],
            Self::Celebi => &["root", "torso", "head", "arm_left", "arm_right", "wing_left", "wing_right", "antenna_left", "antenna_right", "leg_left", "leg_right"],
            Self::HoOh => &["root", "torso", "neck", "head", "wing_left", "wing_right", "tail", "mandible", "leg_left", "leg_right"],
            Self::Lugia => &["root", "torso", "neck", "head", "wing_left", "palm_left", "wing_right", "palm_right", "mandible", "tail_root", "tail_tip", "leg_left", "leg_right"],
            Self::Corsola => &["root", "body", "coral_left", "coral_right", "coral_rear", "coral_rear_right", "coral_rear_left", "arm_left", "arm_right"],
            Self::Mantine => &["root", "body", "wing_left_base", "wing_left_tip", "wing_right_base", "wing_right_tip", "cephalic_left", "cephalic_right", "tail_base", "tail_tip", "remora", "remora_tail", "remora_tail_fin"],
            Self::Porygon => &["root", "hull", "neck", "head", "tail", "paddle_left", "paddle_right"],
            Self::Porygon2 => &["root", "torso", "neck", "head", "tail_base", "tail_tip"],
            Self::Chinchou => &["root", "body", "fin_left", "fin_right", "antenna_left_base", "antenna_left_tip", "antenna_right_base", "antenna_right_tip", "pelvic_left", "pelvic_right"],
            Self::Lanturn => &["root", "body", "fin_left", "fin_right", "beacon_base", "beacon_tip", "branch_base", "branch_tip", "pelvic_left", "pelvic_right", "tail_base", "tail_tip"],
            Self::Slugma => &["root", "mantle", "neck", "eye_left", "eye_right"],
            Self::Magcargo => &["root", "mantle", "neck", "eye_left", "eye_right", "shell"],
            Self::Hitmonlee => &["root", "torso", "kick_hip", "kick_knee", "support_leg", "upper_body", "arm_left", "wrist_left", "arm_right", "wrist_right"],
            Self::Hitmonchan => &["root", "torso", "neck", "head", "shoulder_left", "shoulder_right", "arm_left", "wrist_left", "arm_right", "wrist_right", "leg_left", "leg_right"],
            Self::Hitmontop => &["root", "helmet", "neck", "torso", "arm_left", "arm_right", "leg_left", "leg_right", "tail_base", "tail_bend", "tail_ball"],
            Self::Tyrogue => &["root", "torso", "neck", "head", "ear_left", "ear_right", "arm_left", "wrist_left", "arm_right", "wrist_right", "leg_left", "leg_right"],
            Self::Smoochum => &["root", "torso", "neck", "head", "hair_left", "hair_right", "hair_tufts", "arm_left", "arm_right", "leg_left", "leg_right"],
            Self::Elekid => &["root", "torso", "head", "plug_left", "plug_right", "arm_left", "arm_right", "leg_left", "leg_right"],
            Self::Magby => &["root", "torso", "neck", "head", "crown", "arm_left", "arm_right", "tail_base", "tail_tip", "leg_left", "leg_right"],
            Self::Magmar => &["root","body","neck","head","arm_left","elbow_left","wrist_left","arm_right","elbow_right","wrist_right","leg_left","leg_right","crown_left","crown_right","tail_base","tail_tip","fire_tip"],
            Self::Electabuzz => &["root","body","neck","head","arm_left","elbow_left","wrist_left","arm_right","elbow_right","wrist_right","leg_left","leg_right","antenna_left","antenna_right","crest_left","crest_right","tail_base","tail_mid","tail_tip"],
            Self::Slowking => &["root","body","neck","head","arm_left","elbow_left","wrist_left","arm_right","elbow_right","wrist_right","leg_left","leg_right","tail_base","tail_tip","crown","ruff_left","ruff_right","ruff_center"],
            Self::Slowbro => &["root","body","neck","head","arm_left","elbow_left","wrist_left","arm_right","elbow_right","wrist_right","leg_left","leg_right","tail_base","tail_tip","shell","ear_left","ear_right","shell_jaw"],
            Self::Lickitung => &["root","body","neck","head","arm_left","elbow_left","wrist_left","arm_right","elbow_right","wrist_right","leg_left","leg_right","tail_base","tail_mid","tail_tip","tongue_base","tongue_mid","tongue_tip"],
            Self::MrMime => &["root","body","neck","head","arm_left","elbow_left","wrist_left","arm_right","elbow_right","wrist_right","leg_left","leg_right","knee_left","knee_right","fan_left","fan_right","finger_left_0","finger_left_1","finger_left_2","finger_left_3","finger_left_4","finger_right_0","finger_right_1","finger_right_2","finger_right_3","finger_right_4"],
            Self::Chansey => &["root","body","head","arm_left","elbow_left","arm_right","elbow_right","leg_left","leg_right","ribbon_left_0","ribbon_left_1","ribbon_left_2","ribbon_right_0","ribbon_right_1","ribbon_right_2","tail_base","tail_tip"],
            Self::Blissey => &["root","body","head","arm_left","elbow_left","arm_right","elbow_right","leg_left","leg_right","crown_left","curl_left","crown_right","curl_right","frill_left","frill_right","skirt_0","skirt_1","skirt_2","skirt_3","skirt_4","skirt_5","skirt_6","skirt_7","skirt_8","skirt_9"],
            Self::Growlithe => &["root","body","neck","head","ear_left","ear_right","jaw","fore_left","fore_right","rear_left","rear_right","tail_base","tail_mid","tail_tip","mane","forelock","ruff_left","ruff_right"],
            Self::Arcanine => &["root","body","neck","head","ear_left","ear_right","jaw","fore_left","fore_right","rear_left","rear_right","tail_base","tail_mid","tail_tip","mane","forelock","ruff_left","ruff_right"],
            Self::NidoranF => &["root","body","head","ear_left","ear_right","fore_left","fore_right","rear_left","rear_right","tail_base","tail_tip"],
            Self::NidoranM => &["root","body","head","ear_left","ear_right","fore_left","fore_right","rear_left","rear_right","tail_base","tail_tip"],
            Self::Nidorina => &["root","body","head","ear_left","ear_right","arm_left","forearm_left","wrist_left","arm_right","forearm_right","wrist_right","leg_left","leg_right","tail_base","tail_tip"],
            Self::Nidorino => &["root","body","head","ear_left","ear_right","jaw","horn","fore_left","fore_right","rear_left","rear_right","tail_base","tail_tip"],
            Self::Nidoking => &["root", "body", "head", "ear_left", "ear_right", "horn", "lower_jaw", "tongue", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_mid", "tail_tip"],
            Self::Nidoqueen => &["root", "body", "head", "ear_left", "ear_right", "horn", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_mid", "tail_tip"],
            Self::Pichu => &["root", "torso", "head", "ear_left", "ear_right", "arm_left", "arm_right", "tail", "leg_left", "leg_right"],
            Self::Cleffa => &["root", "torso", "head", "ear_left", "ear_right", "tuft", "arm_left", "arm_right", "tail"],
            Self::Igglybuff => &["root", "torso", "head", "crown", "arm_left", "arm_right"],
            Self::Mareep => &["root", "torso", "neck", "head", "ear_left", "ear_right", "tail_base", "tail_bend", "tail_beacon", "leg_left_front", "leg_left_rear", "leg_right_front", "leg_right_rear"],
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
            Self::Bellsprout => &["root", "body", "upper_stem", "head", "leaf_left", "leaf_right", "leaf_tip_left", "leaf_tip_right", "leg_left", "leg_right"],
            Self::Weepinbell => &["root", "body", "mouth", "leaf_left", "leaf_right", "leaf_tip_left", "leaf_tip_right", "hook", "hook_tip"],
            Self::Victreebel => &["root", "body", "mouth", "leaf_left", "leaf_right", "leaf_tip_left", "leaf_tip_right", "hood", "vine", "vine_tip"],
            Self::Gloom => &["root", "body", "crown", "arm_left", "arm_right", "leg_left", "leg_right", "sepal_0", "sepal_1", "sepal_2", "sepal_3"],
            Self::Vileplume => &["root", "body", "crown", "arm_left", "arm_right", "leg_left", "leg_right", "petal_0", "petal_1", "petal_2", "petal_3", "petal_4"],
            Self::Raichu => &["root", "body", "head", "arm_left", "arm_right", "ear_left", "ear_right", "ear_left_tip", "ear_right_tip", "tail_base", "tail_tip", "leg_left", "leg_right"],
            Self::Ekans => &["root", "body", "head", "tail_base", "tail_tip"],
            Self::Arbok => &["root", "body", "head", "tail_base", "tail_tip", "hood_left", "hood_right", "jaw"],
            Self::Steelix => &["root", "segment_3", "segment_4", "segment_5", "segment_6", "segment_7", "segment_8", "segment_9", "head", "jaw"],
            Self::Pineco => &["root", "body", "whorl_low", "whorl_middle", "whorl_high", "crown", "stem"],
            Self::Forretress => &["root", "body", "upper_shell", "port_-1_0", "port_-1_1", "port_1_0", "port_1_1"],
            Self::Kabuto => &["root", "body", "leg_-1_0", "leg_-1_1", "leg_1_0", "leg_1_1"],
            Self::Kabutops => &["root", "body", "head", "arm_left", "arm_right", "elbow_left", "elbow_right", "blade_left", "blade_right", "leg_left", "leg_right", "tail_base", "tail_tip"],
            Self::Electrode => &["root", "body"],
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
            Self::Horsea => &["root", "body", "head", "fin_left", "fin_right", "head_fin_left", "head_fin_right", "tail_base", "tail_middle", "tail_tip"],
            Self::Seadra => &["root", "body", "head", "fin_left", "fin_right", "head_fin_left", "head_fin_right", "tail_base", "tail_middle", "tail_tip"],
            Self::Kingdra => &["root", "body", "head", "fin_left", "fin_right", "head_fin_left", "head_fin_right", "tail_base", "tail_middle", "tail_tip", "crown_left", "crown_right"],
            Self::Goldeen => &["root", "body", "tail", "dorsal", "pectoral_left", "pectoral_right"],
            Self::Seaking => &["root", "body", "tail", "dorsal", "pectoral_left", "pectoral_right", "pelvic_left", "pelvic_right"],
            Self::Magikarp => &["root", "body", "tail", "dorsal", "pectoral_left", "pectoral_right", "pelvic", "barbel_left", "barbel_right"],
            Self::Larvitar => &["root", "body", "head", "arm_left", "arm_right", "wrist_left", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip"],
            Self::Pupitar => &["root", "body", "head", "tail_fold"],
            Self::Tyranitar => &["root", "body", "head", "jaw", "arm_left", "arm_right", "forearm_left", "forearm_right", "wrist_left", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip"],
            Self::Quilava => &["root", "body", "head", "foreleg_left", "foreleg_right", "hindleg_left", "hindleg_right", "fire_neck_0", "fire_neck_1", "fire_neck_2", "fire_rump_0", "fire_rump_1", "fire_rump_2"],
            Self::Typhlosion => &["root", "body", "head", "jaw", "arm_left", "arm_right", "wrist_left", "wrist_right", "leg_left", "leg_right", "fire_mantle_0", "fire_mantle_1", "fire_mantle_2", "fire_mantle_3", "fire_mantle_4", "fire_mantle_5", "fire_mantle_6"],
            Self::Charmeleon => &["root", "body", "head", "jaw", "arm_left", "arm_right", "wrist_left", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip", "flame"],
            Self::Charizard => &["root", "body", "head", "jaw", "arm_left", "arm_right", "wrist_left", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip", "flame", "wing_left", "wing_right"],
            Self::Graveler => &["root", "body", "arm_upper_left", "arm_upper_right", "arm_lower_left", "arm_lower_right", "wrist_upper_left", "wrist_upper_right", "wrist_lower_left", "wrist_lower_right"],
            Self::Golem => &["root", "body", "head", "jaw", "arm_left", "arm_right", "forearm_left", "forearm_right", "wrist_left", "wrist_right", "leg_left", "leg_right"],
            Self::Omanyte => &["root", "body", "shell", "eye_left", "eye_right", "crawl_tentacle_0", "crawl_tentacle_1", "crawl_tentacle_2", "crawl_tentacle_3", "crawl_tentacle_4", "crawl_tentacle_5", "crawl_tentacle_6", "crawl_tentacle_7", "crawl_tentacle_8", "crawl_tentacle_9"],
            Self::Omastar => &["root", "body", "shell", "eye_left", "eye_right", "crawl_tentacle_0", "crawl_tentacle_1", "crawl_tentacle_2", "crawl_tentacle_3", "crawl_tentacle_4", "crawl_tentacle_5", "crawl_tentacle_6", "crawl_tentacle_7", "crawl_tentacle_8", "crawl_tentacle_9", "beak_left_lower", "beak_left_upper", "beak_right_lower", "beak_right_upper"],
            Self::Aerodactyl => &["root", "body", "head", "jaw", "wing_left", "wing_right", "leg_left", "leg_right", "tail"],
            Self::Dratini => &["root", "body", "head", "ear_left", "ear_right", "tail_base", "tail_tip"],
            Self::Dragonair => &["root", "body", "head", "ear_left", "ear_right", "tail_base", "tail_tip"],
            Self::Dragonite => &["root", "body", "head", "antenna_left", "antenna_right", "wing_left", "wing_right", "arm_left", "arm_right", "wrist_left", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip"],
            Self::Clefable => &["root", "body", "ear_left", "ear_right", "arm_left", "arm_right", "tuft", "wing_left", "wing_right", "tail_base", "tail_tip"],
            Self::Wigglytuff => &["root", "body", "ear_left", "ear_right", "arm_left", "arm_right", "tuft"],
            Self::Sandshrew => &["root", "body", "head", "ear_left", "ear_right", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip"],
            Self::Sandslash => &["root", "body", "head", "ear_left", "ear_right", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right", "tail_base", "tail_tip", "quills_lower", "quills_upper"],
            Self::Politoed => &["root", "body", "head", "eye_left", "eye_right", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right", "crest_base", "crest_tip"],
            Self::Poliwrath => &["root", "body", "eye_left", "eye_right", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right"],
            Self::Poliwhirl => &["root", "body", "eye_left", "eye_right", "arm_left", "forearm_left", "wrist_left", "arm_right", "forearm_right", "wrist_right", "leg_left", "leg_right"],
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
                "hindleg_left",
                "hindleg_right",
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
            Self::Procedural(_) => unreachable!("procedural rigs do not use the GLB schema"),
            Self::Diglett => &[None, Some(0)],
            Self::Dugtrio => &[None, Some(0), Some(0), Some(0)],
            Self::Tentacruel => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(1), Some(12), Some(1), Some(14), Some(1), Some(16), Some(1), Some(18), Some(1), Some(20), Some(1), Some(1)],
            Self::Tentacool => &[None, Some(0), Some(1), Some(2), Some(1), Some(4)],
            Self::Jynx => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(1), Some(9), Some(10), Some(8), Some(8), Some(8), Some(8), Some(8), Some(11), Some(11), Some(11), Some(11), Some(11)],
            Self::Primeape => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(1), Some(0), Some(0)],
            Self::Mankey => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(1), Some(0), Some(0), Some(1), Some(10)],
            Self::Meowth => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(1), Some(8), Some(0), Some(0), Some(1), Some(12), Some(13)],
            Self::Persian => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(0), Some(0), Some(0), Some(0), Some(1), Some(10), Some(11)],
            Self::Doduo => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(1), Some(6), Some(7), Some(8), Some(0), Some(0)],
            Self::Dodrio => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(1), Some(6), Some(7), Some(8), Some(1), Some(10), Some(11), Some(12), Some(0), Some(0), Some(1), Some(1), Some(1)],
            Self::Pidgeotto => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(0), Some(0), Some(1), Some(9), Some(9), Some(9), Some(2)],
            Self::FarfetchD => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(6), Some(7), Some(8), Some(9), Some(9), Some(9), Some(1), Some(1), Some(1), Some(2), Some(2), Some(2), Some(0), Some(0)],
            Self::Venusaur => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(7), Some(7), Some(9), Some(7), Some(11), Some(7), Some(13), Some(7), Some(15), Some(7), Some(17), Some(7), Some(19), Some(8), Some(21), Some(8), Some(23), Some(8), Some(25), Some(8), Some(27), Some(8), Some(29), Some(8)],
            Self::Ivysaur => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(7), Some(7), Some(9), Some(7), Some(11), Some(7), Some(13), Some(7), Some(15), Some(7), Some(17), Some(7), Some(19)],
            Self::Exeggutor => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(0), Some(0), Some(2), Some(8), Some(2), Some(10), Some(2), Some(12), Some(2), Some(14), Some(2), Some(16), Some(2), Some(18), Some(2), Some(20), Some(2), Some(22), Some(2), Some(24), Some(2), Some(26), Some(2), Some(28)],
            Self::Exeggcute => &[None, Some(0), Some(0), Some(0), Some(1), Some(2), Some(3)],
            Self::Fearow => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(1), Some(1), Some(1), Some(1), Some(0), Some(0), Some(4), Some(2)],
            Self::Pidgeot => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(1), Some(6), Some(1), Some(8), Some(1), Some(1), Some(1), Some(1), Some(1), Some(0), Some(0), Some(4), Some(4), Some(4)],
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
            Self::Bellossom => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(3), Some(3), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Qwilfish => &[None, Some(0), Some(1), Some(1), Some(1), Some(4), Some(5)],
            Self::Wobbuffet => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(0), Some(8), Some(9)],
            Self::Dunsparce => &[None, Some(0), Some(1), Some(1), Some(1), Some(4), Some(5)],
            Self::Shuckle => &[None, Some(0), Some(0), Some(0), Some(0), Some(0), Some(5), Some(6)],
            Self::Gligar => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(1), Some(6), Some(1), Some(8), Some(1), Some(10), Some(11), Some(12), Some(13)],
            Self::Yanma => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(1), Some(13), Some(14)],
            Self::Murkrow => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(6), Some(2), Some(2), Some(2)],
            Self::Misdreavus => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8)],
            Self::Teddiursa => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(1), Some(2), Some(2), Some(0), Some(0)],
            Self::Ursaring => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(1), Some(2), Some(2), Some(0), Some(0)],
            Self::Snubbull => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(2), Some(2), Some(0), Some(0)],
            Self::Granbull => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(1), Some(2), Some(2), Some(0), Some(0)],
            Self::Houndour => &[None, Some(0), Some(1), Some(2), Some(0), Some(0), Some(0), Some(0), Some(1), Some(8), Some(9), Some(3), Some(3)],
            Self::Houndoom => &[None, Some(0), Some(1), Some(2), Some(3), Some(0), Some(0), Some(0), Some(0), Some(1), Some(9), Some(10)],
            Self::Miltank => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(2), Some(2), Some(1), Some(9), Some(10), Some(0), Some(0)],
            Self::Tauros => &[None, Some(0), Some(1), Some(2), Some(0), Some(0), Some(0), Some(0), Some(1), Some(8), Some(9), Some(1), Some(11), Some(12), Some(1), Some(14), Some(15)],
            Self::Delibird => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(2), Some(2)],
            Self::Stantler => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(0), Some(0), Some(0), Some(0), Some(1), Some(10)],
            Self::Girafarig => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(0), Some(0), Some(0), Some(0), Some(1), Some(10), Some(11)],
            Self::Donphan => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(2), Some(1), Some(7), Some(0), Some(0), Some(0), Some(0)],
            Self::Phanpy => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(2), Some(1), Some(7), Some(1), Some(0), Some(0), Some(0)],
            Self::Azumarill => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(5), Some(1), Some(1), Some(1), Some(9), Some(10)],
            Self::Marill => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(5), Some(1), Some(1), Some(1), Some(9), Some(10)],
            Self::Xatu => &[None, Some(0), Some(1), Some(1), Some(1), Some(2), Some(5), Some(2), Some(7), Some(0), Some(0)],
            Self::Natu => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(1), Some(1), Some(0), Some(0)],
            Self::Ampharos => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(6), Some(7)],
            Self::Flaaffy => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(1), Some(1), Some(8), Some(9), Some(0), Some(0)],
            Self::Togepi => &[None, Some(0), Some(1), Some(1), Some(1), Some(0), Some(0)],
            Self::Togetic => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(1), Some(0), Some(0)],
            Self::Remoraid => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Octillery => &[None, Some(0), Some(1), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0)],
            Self::Swinub => &[None, Some(0), Some(1), Some(1), Some(0), Some(0), Some(0), Some(0)],
            Self::Piloswine => &[None, Some(0), Some(1), Some(1), Some(1), Some(0), Some(0), Some(0), Some(0)],
            Self::Sneasel => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(1), Some(7), Some(1), Some(9), Some(0), Some(0)],
            Self::Skarmory => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(0), Some(0), Some(3)],
            Self::Smeargle => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(3), Some(1), Some(7), Some(1), Some(9), Some(0), Some(0)],
            Self::Moltres => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(4), Some(5), Some(3), Some(1), Some(0), Some(0)],
            Self::Zapdos => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(0), Some(0)],
            Self::Articuno => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(6), Some(7), Some(0), Some(0)],
            Self::Mew => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(1), Some(1), Some(8), Some(9), Some(3), Some(3)],
            Self::Mewtwo => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(5), Some(1), Some(7), Some(8), Some(1), Some(10), Some(1), Some(12), Some(1), Some(14), Some(15)],
            Self::Heracross => &[None, Some(0), Some(1), Some(2), Some(2), Some(4), Some(2), Some(6), Some(3), Some(3), Some(1), Some(1)],
            Self::Pinsir => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(5), Some(6), Some(2), Some(8), Some(9), Some(1), Some(1)],
            Self::Scizor => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(6), Some(7), Some(1), Some(9), Some(10), Some(11), Some(1), Some(1), Some(1), Some(1)],
            Self::Scyther => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(1), Some(7), Some(1), Some(1), Some(2), Some(2)],
            Self::Starmie => &[None, Some(0), Some(1), Some(2), Some(3), Some(2), Some(5), Some(2), Some(7), Some(2), Some(9), Some(2), Some(11), Some(1), Some(13), Some(14), Some(13), Some(16), Some(13), Some(18), Some(13), Some(20), Some(13), Some(22)],
            Self::Gastly => &[None, Some(0), Some(0), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2)],
            Self::Haunter => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(0), Some(8), Some(8), Some(8), Some(0), Some(12), Some(12), Some(12)],
            Self::Eevee => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(0), Some(0), Some(0), Some(0)],
            Self::Vulpix => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(0), Some(0), Some(0), Some(0)],
            Self::Ninetales => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(6), Some(0), Some(0), Some(0), Some(0)],
            Self::Venonat => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(4), Some(5), Some(0), Some(0)],
            Self::Venomoth => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(2), Some(2), Some(2), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Paras => &[None, Some(0), Some(1), Some(1), Some(1), Some(0), Some(0), Some(0), Some(0), Some(1), Some(1)],
            Self::Parasect => &[None, Some(0), Some(1), Some(1), Some(1), Some(0), Some(0), Some(0), Some(0), Some(1)],
            Self::Metapod => &[None, Some(0), Some(1), Some(0)],
            Self::Kakuna => &[None, Some(0), Some(1), Some(1), Some(1)],
            Self::Caterpie => &[None, Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(8), Some(9), Some(9)],
            Self::Weedle => &[None, Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), Some(8), Some(9), Some(1)],
            Self::Rhydon => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(0), Some(0), Some(1), Some(10), Some(2), Some(2), Some(2)],
            Self::Drowzee => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(2), Some(2)],
            Self::Hypno => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(2), Some(2), Some(8)],
            Self::Abra => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(2), Some(2), Some(1), Some(13)],
            Self::Alakazam => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(2), Some(2), Some(2), Some(2), Some(5), Some(8)],
            Self::Kadabra => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(2), Some(2), Some(2), Some(2), Some(1), Some(15), Some(8)],
            Self::Koffing => &[None, Some(0), Some(0), Some(2), Some(2), Some(2), Some(2), Some(0), Some(7), Some(7), Some(7), Some(7)],
            Self::Weezing => &[None, Some(0), Some(1), Some(1), Some(0), Some(4), Some(4), Some(4), Some(4), Some(0), Some(9), Some(9), Some(9), Some(9)],
            Self::Wartortle => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(2), Some(2), Some(1), Some(13)],
            Self::Blastoise => &[None, Some(0), Some(1), Some(1), Some(3), Some(4), Some(1), Some(6), Some(7), Some(0), Some(0), Some(1), Some(1), Some(1), Some(13)],
            Self::Croconaw => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(5), Some(1), Some(7), Some(8), Some(0), Some(0), Some(1), Some(12)],
            Self::Feraligatr => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(5), Some(1), Some(7), Some(8), Some(0), Some(0), Some(1), Some(12)],
            Self::Cubone => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(5), Some(1), Some(7), Some(8), Some(0), Some(0), Some(1), Some(12)],
            Self::Marowak => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(5), Some(1), Some(7), Some(8), Some(0), Some(0), Some(1), Some(12)],
            Self::Magnemite => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Magneton => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(0), Some(7), Some(7), Some(7), Some(7), Some(7), Some(0), Some(13), Some(13), Some(13), Some(13), Some(13)],
            Self::Tangela => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Kangaskhan => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(1), Some(6), Some(1), Some(8), Some(0), Some(0), Some(1), Some(12), Some(1), Some(14), Some(15), Some(15), Some(15), Some(15)],
            Self::Ponyta => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(0), Some(0), Some(0), Some(0), Some(2), Some(3)],
            Self::Rapidash => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(0), Some(0), Some(0), Some(0), Some(2), Some(3), Some(3)],
            Self::Machop => &[None, Some(0), Some(1), Some(2), Some(0), Some(0), Some(1), Some(6), Some(7), Some(1), Some(9), Some(10), Some(1)],
            Self::Machoke => &[None, Some(0), Some(1), Some(2), Some(0), Some(0), Some(1), Some(6), Some(7), Some(1), Some(9), Some(10)],
            Self::Machamp => &[None, Some(0), Some(1), Some(2), Some(0), Some(0), Some(1), Some(6), Some(7), Some(1), Some(9), Some(10), Some(1), Some(12), Some(13), Some(1), Some(15), Some(16)],
            Self::Rhyhorn => &[None, Some(0), Some(1), Some(1), Some(0), Some(0), Some(0), Some(0), Some(2), Some(2), Some(2)],
            Self::Dewgong => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(3), Some(6), Some(6)],
            Self::Seel => &[None, Some(0), Some(1), Some(1), Some(0), Some(0), Some(3), Some(6), Some(6), Some(2)],
            Self::Ditto => &[None, Some(0), Some(1), Some(2), Some(2)],
            Self::Muk => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(4), Some(4), Some(4), Some(6), Some(6), Some(6)],
            Self::Grimer => &[None, Some(0), Some(1), Some(1), Some(3), Some(1), Some(5), Some(4), Some(4), Some(4), Some(6), Some(6), Some(6)],
            Self::Cloyster => &[None, Some(0), Some(1), Some(1), Some(1)],
            Self::Shellder => &[None, Some(0), Some(1), Some(1), Some(1), Some(4)],
            Self::Kingler => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(0), Some(0), Some(0), Some(0)],
            Self::Krabby => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(5), Some(6), Some(0), Some(0), Some(0), Some(0)],
            Self::Umbreon => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(0), Some(0), Some(0), Some(0)],
            Self::Jolteon => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(0), Some(0), Some(0), Some(0), Some(2)],
            Self::Flareon => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(0), Some(0), Some(0), Some(0), Some(2)],
            Self::Espeon => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(8), Some(8), Some(0), Some(0), Some(0), Some(0)],
            Self::Vaporeon => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(8), Some(8), Some(0), Some(0), Some(0), Some(0), Some(2)],
            Self::Celebi => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(2), Some(2), Some(0), Some(0)],
            Self::HoOh => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(1), Some(3), Some(0), Some(0)],
            Self::Lugia => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(6), Some(3), Some(1), Some(9), Some(0), Some(0)],
            Self::Corsola => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1), Some(1)],
            Self::Mantine => &[None, Some(0), Some(1), Some(2), Some(1), Some(4), Some(1), Some(1), Some(1), Some(8), Some(4), Some(10), Some(11)],
            Self::Porygon => &[None, Some(0), Some(1), Some(2), Some(1), Some(0), Some(0)],
            Self::Porygon2 => &[None, Some(0), Some(1), Some(2), Some(1), Some(4)],
            Self::Chinchou => &[None, Some(0), Some(1), Some(1), Some(1), Some(4), Some(1), Some(6), Some(0), Some(0)],
            Self::Lanturn => &[None, Some(0), Some(1), Some(1), Some(1), Some(4), Some(4), Some(6), Some(0), Some(0), Some(1), Some(10)],
            Self::Slugma => &[None, Some(0), Some(1), Some(2), Some(2)],
            Self::Magcargo => &[None, Some(0), Some(1), Some(2), Some(2), Some(1)],
            Self::Hitmonlee => &[None, Some(0), Some(0), Some(2), Some(0), Some(1), Some(5), Some(6), Some(5), Some(8)],
            Self::Hitmonchan => &[None, Some(0), Some(1), Some(2), Some(1), Some(1), Some(4), Some(6), Some(5), Some(8), Some(0), Some(0)],
            Self::Hitmontop => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(3), Some(3), Some(3), Some(8), Some(9)],
            Self::Tyrogue => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(1), Some(8), Some(0), Some(0)],
            Self::Smoochum => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(3), Some(1), Some(1), Some(0), Some(0)],
            Self::Elekid => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(2), Some(0), Some(0)],
            Self::Magby => &[None, Some(0), Some(1), Some(2), Some(3), Some(1), Some(1), Some(1), Some(7), Some(0), Some(0)],
            Self::Magmar => &[None,Some(0),Some(1),Some(2),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(0),Some(0),Some(3),Some(3),Some(1),Some(14),Some(15)],
            Self::Electabuzz => &[None,Some(0),Some(1),Some(2),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(0),Some(0),Some(3),Some(3),Some(3),Some(3),Some(1),Some(16),Some(17)],
            Self::Slowking => &[None,Some(0),Some(1),Some(2),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(0),Some(0),Some(1),Some(12),Some(3),Some(2),Some(2),Some(2)],
            Self::Slowbro => &[None,Some(0),Some(1),Some(2),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(0),Some(0),Some(1),Some(12),Some(13),Some(3),Some(3),Some(14)],
            Self::Lickitung => &[None,Some(0),Some(1),Some(2),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(0),Some(0),Some(1),Some(12),Some(13),Some(3),Some(15),Some(16)],
            Self::MrMime => &[None,Some(0),Some(1),Some(2),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(1),Some(1),Some(10),Some(11),Some(3),Some(3),Some(6),Some(6),Some(6),Some(6),Some(6),Some(9),Some(9),Some(9),Some(9),Some(9)],
            Self::Chansey => &[None,Some(0),Some(1),Some(1),Some(3),Some(1),Some(5),Some(0),Some(0),Some(2),Some(2),Some(2),Some(2),Some(2),Some(2),Some(1),Some(15)],
            Self::Blissey => &[None,Some(0),Some(1),Some(1),Some(3),Some(1),Some(5),Some(0),Some(0),Some(2),Some(9),Some(2),Some(11),Some(3),Some(5),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1)],
            Self::Growlithe => &[None,Some(0),Some(1),Some(2),Some(3),Some(3),Some(3),Some(1),Some(1),Some(1),Some(1),Some(1),Some(11),Some(12),Some(2),Some(3),Some(14),Some(14)],
            Self::Arcanine => &[None,Some(0),Some(1),Some(2),Some(3),Some(3),Some(3),Some(1),Some(1),Some(1),Some(1),Some(1),Some(11),Some(12),Some(2),Some(3),Some(14),Some(14)],
            Self::NidoranF => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(1),Some(1),Some(1),Some(1),Some(9)],
            Self::NidoranM => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(1),Some(1),Some(1),Some(1),Some(9)],
            Self::Nidorina => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(5),Some(6),Some(1),Some(8),Some(9),Some(1),Some(1),Some(1),Some(13)],
            Self::Nidorino => &[None,Some(0),Some(1),Some(2),Some(2),Some(2),Some(2),Some(1),Some(1),Some(1),Some(1),Some(1),Some(11)],
            Self::Nidoking => &[None,Some(0),Some(1),Some(2),Some(2),Some(2),Some(2),Some(6),Some(1),Some(8),Some(9),Some(1),Some(11),Some(12),Some(1),Some(1),Some(1),Some(16),Some(17)],
            Self::Nidoqueen => &[None,Some(0),Some(1),Some(2),Some(2),Some(2),Some(1),Some(6),Some(7),Some(1),Some(9),Some(10),Some(1),Some(1),Some(1),Some(14),Some(15)],
            Self::Pichu => &[None, Some(0), Some(1), Some(2), Some(2), Some(1), Some(1), Some(1), Some(0), Some(0)],
            Self::Cleffa => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(2), Some(2), Some(1)],
            Self::Igglybuff => &[None, Some(0), Some(1), Some(2), Some(2), Some(2)],
            Self::Mareep => &[None, Some(0), Some(1), Some(2), Some(3), Some(3), Some(1), Some(6), Some(7), Some(0), Some(0), Some(0), Some(0)],
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
            Self::Bellsprout => &[None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(4), Some(5), Some(0), Some(0)],
            Self::Weepinbell => &[None, Some(0), Some(1), Some(1), Some(1), Some(3), Some(4), Some(1), Some(7)],
            Self::Victreebel => &[None, Some(0), Some(1), Some(1), Some(1), Some(3), Some(4), Some(2), Some(1), Some(8)],
            Self::Gloom => &[None, Some(0), Some(1), Some(1), Some(1), Some(0), Some(0), Some(2), Some(2), Some(2), Some(2)],
            Self::Vileplume => &[None, Some(0), Some(1), Some(1), Some(1), Some(0), Some(0), Some(2), Some(2), Some(2), Some(2), Some(2)],
            Self::Raichu => &[None, Some(0), Some(1), Some(1), Some(1), Some(2), Some(2), Some(5), Some(6), Some(1), Some(9), Some(0), Some(0)],
            Self::Ekans => &[None, Some(0), Some(1), Some(0), Some(3)],
            Self::Arbok => &[None, Some(0), Some(1), Some(0), Some(3), Some(1), Some(1), Some(2)],
            Self::Steelix => &[None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8)],
            Self::Pineco => &[None, Some(0), Some(1), Some(1), Some(1), Some(1), Some(5)],
            Self::Forretress => &[None, Some(0), Some(0), Some(1), Some(1), Some(1), Some(1)],
            Self::Kabuto => &[None, Some(0), Some(0), Some(0), Some(0), Some(0)],
            Self::Kabutops => &[None, Some(0), Some(1), Some(1), Some(1), Some(3), Some(4), Some(5), Some(6), Some(0), Some(0), Some(1), Some(11)],
            Self::Electrode => &[None, Some(0)],
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
            Self::Horsea => &[None,Some(0),Some(1),Some(1),Some(1),Some(2),Some(2),Some(1),Some(7),Some(8)],
            Self::Seadra => &[None,Some(0),Some(1),Some(1),Some(1),Some(2),Some(2),Some(1),Some(7),Some(8)],
            Self::Kingdra => &[None,Some(0),Some(1),Some(1),Some(1),Some(2),Some(2),Some(1),Some(7),Some(8),Some(2),Some(2)],
            Self::Goldeen => &[None,Some(0),Some(1),Some(1),Some(1),Some(1)],
            Self::Seaking => &[None,Some(0),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1)],
            Self::Magikarp => &[None,Some(0),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1)],
            Self::Larvitar => &[None,Some(0),Some(1),Some(1),Some(1),Some(3),Some(4),Some(0),Some(0),Some(1),Some(9)],
            Self::Pupitar => &[None,Some(0),Some(1),Some(0)],
            Self::Tyranitar => &[None,Some(0),Some(1),Some(2),Some(1),Some(1),Some(4),Some(5),Some(6),Some(7),Some(0),Some(0),Some(1),Some(12)],
            Self::Quilava => &[None,Some(0),Some(1),Some(0),Some(0),Some(0),Some(0),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1)],
            Self::Typhlosion => &[None,Some(0),Some(1),Some(2),Some(1),Some(1),Some(4),Some(5),Some(0),Some(0),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1)],
            Self::Charmeleon => &[None,Some(0),Some(1),Some(2),Some(1),Some(1),Some(4),Some(5),Some(0),Some(0),Some(1),Some(10),Some(11)],
            Self::Charizard => &[None,Some(0),Some(1),Some(2),Some(1),Some(1),Some(4),Some(5),Some(0),Some(0),Some(1),Some(10),Some(11),Some(1),Some(1)],
            Self::Graveler => &[None,Some(0),Some(1),Some(1),Some(1),Some(1),Some(2),Some(3),Some(4),Some(5)],
            Self::Golem => &[None,Some(0),Some(1),Some(1),Some(1),Some(1),Some(4),Some(5),Some(6),Some(7),Some(0),Some(0)],
            Self::Omanyte => &[None,Some(0),Some(1),Some(1),Some(1),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0)],
            Self::Omastar => &[None,Some(0),Some(1),Some(1),Some(1),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(0),Some(1),Some(1),Some(1),Some(1)],
            Self::Aerodactyl => &[None,Some(0),Some(1),Some(2),Some(1),Some(1),Some(0),Some(0),Some(1)],
            Self::Dratini => &[None,Some(0),Some(1),Some(2),Some(2),Some(0),Some(5)],
            Self::Dragonair => &[None,Some(0),Some(1),Some(2),Some(2),Some(0),Some(5)],
            Self::Dragonite => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(1),Some(1),Some(1),Some(7),Some(8),Some(0),Some(0),Some(1),Some(13)],
            Self::Clefable => &[None,Some(0),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(1),Some(9)],
            Self::Wigglytuff => &[None,Some(0),Some(1),Some(1),Some(1),Some(1),Some(1)],
            Self::Sandshrew => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(5),Some(6),Some(1),Some(8),Some(9),Some(1),Some(1),Some(1),Some(13)],
            Self::Sandslash => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(5),Some(6),Some(1),Some(8),Some(9),Some(1),Some(1),Some(1),Some(13),Some(1),Some(1)],
            Self::Politoed => &[None,Some(0),Some(1),Some(2),Some(2),Some(1),Some(5),Some(6),Some(1),Some(8),Some(9),Some(1),Some(1),Some(2),Some(13)],
            Self::Poliwrath => &[None,Some(0),Some(1),Some(1),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(1),Some(1)],
            Self::Poliwhirl => &[None,Some(0),Some(1),Some(1),Some(1),Some(4),Some(5),Some(1),Some(7),Some(8),Some(1),Some(1)],
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
            Self::Procedural(_) => unreachable!("procedural rigs do not use the GLB schema"),
            Self::Diglett => 15,
            Self::Dugtrio => 33,
            Self::Tentacruel => 23,
            Self::Tentacool => 13,
            Self::Jynx => 35,
            Self::Primeape => 45,
            Self::Mankey => 38,
            Self::Meowth => 39,
            Self::Persian => 36,
            Self::Doduo => 19,
            Self::Dodrio => 32,
            Self::Pidgeotto => 34,
            Self::FarfetchD => 21,
            Self::Venusaur => 29,
            Self::Ivysaur => 26,
            Self::Exeggutor => 24,
            Self::Exeggcute => 17,
            Self::Fearow => 21,
            Self::Pidgeot => 25,
            Self::Entei => 59,
            Self::Raikou => 62,
            Self::Ledyba => 36,
            Self::Ledian => 38,
            Self::Ariados => 41,
            Self::Quagsire => 22,
            Self::Hoppip => 21,
            Self::Skiploom => 24,
            Self::Aipom => 50,
            Self::Bellossom => 45,
            Self::Qwilfish => 36,
            Self::Wobbuffet => 19,
            Self::Dunsparce => 19,
            Self::Shuckle => 27,
            Self::Gligar => 32,
            Self::Yanma => 58,
            Self::Murkrow => 62,
            Self::Misdreavus => 41,
            Self::Teddiursa => 37,
            Self::Ursaring => 48,
            Self::Snubbull => 39,
            Self::Granbull => 42,
            Self::Houndour => 44,
            Self::Houndoom => 59,
            Self::Miltank => 45,
            Self::Tauros => 42,
            Self::Delibird => 28,
            Self::Stantler => 51,
            Self::Girafarig => 57,
            Self::Donphan => 49,
            Self::Phanpy => 32,
            Self::Azumarill => 25,
            Self::Marill => 19,
            Self::Xatu => 49,
            Self::Natu => 46,
            Self::Ampharos => 26,
            Self::Flaaffy => 33,
            Self::Togepi => 27,
            Self::Togetic => 37,
            Self::Remoraid => 18,
            Self::Octillery => 52,
            Self::Swinub => 22,
            Self::Piloswine => 22,
            Self::Sneasel => 38,
            Self::Skarmory => 42,
            Self::Smeargle => 43,
            Self::Moltres => 46,
            Self::Zapdos => 54,
            Self::Articuno => 49,
            Self::Mew => 40,
            Self::Mewtwo => 52,
            Self::Heracross => 47,
            Self::Pinsir => 53,
            Self::Scizor => 47,
            Self::Scyther => 40,
            Self::Starmie => 23,
            Self::Gastly => 19,
            Self::Haunter => 23,
            Self::Eevee => 47,
            Self::Vulpix => 48,
            Self::Ninetales => 46,
            Self::Venonat => 33,
            Self::Venomoth => 53,
            Self::Paras => 56,
            Self::Parasect => 36,
            Self::Metapod => 15,
            Self::Kakuna => 14,
            Self::Caterpie => 33,
            Self::Weedle => 24,
            Self::Rhydon => 48,
            Self::Drowzee => 43,
            Self::Hypno => 48,
            Self::Abra => 43,
            Self::Alakazam => 53,
            Self::Kadabra => 38,
            Self::Koffing => 23,
            Self::Weezing => 35,
            Self::Wartortle => 37,
            Self::Blastoise => 42,
            Self::Croconaw => 28,
            Self::Feraligatr => 41,
            Self::Cubone => 45,
            Self::Marowak => 52,
            Self::Magnemite => 9,
            Self::Magneton => 27,
            Self::Tangela => 10,
            Self::Kangaskhan => 46,
            Self::Ponyta => 40,
            Self::Rapidash => 47,
            Self::Machop => 32,
            Self::Machoke => 41,
            Self::Machamp => 51,
            Self::Rhyhorn => 46,
            Self::Dewgong => 22,
            Self::Seel => 27,
            Self::Ditto => 8,
            Self::Muk => 26,
            Self::Grimer => 26,
            Self::Cloyster => 24,
            Self::Shellder => 19,
            Self::Kingler => 36,
            Self::Krabby => 31,
            Self::Umbreon => 46,
            Self::Jolteon => 50,
            Self::Flareon => 46,
            Self::Espeon => 44,
            Self::Vaporeon => 57,
            Self::Celebi => 38,
            Self::HoOh => 38,
            Self::Lugia => 34,
            Self::Corsola => 37,
            Self::Mantine => 32,
            Self::Porygon => 16,
            Self::Porygon2 => 17,
            Self::Chinchou => 21,
            Self::Lanturn => 29,
            Self::Slugma => 12,
            Self::Magcargo => 22,
            Self::Hitmonlee => 33,
            Self::Hitmonchan => 36,
            Self::Hitmontop => 35,
            Self::Tyrogue => 40,
            Self::Smoochum => 40,
            Self::Elekid => 32,
            Self::Magby => 38,
            Self::Magmar => 39,
            Self::Electabuzz => 41,
            Self::Slowking => 33,
            Self::Slowbro => 38,
            Self::Lickitung => 26,
            Self::MrMime => 34,
            Self::Chansey => 16,
            Self::Blissey => 19,
            Self::Growlithe => 38,
            Self::Arcanine => 46,
            Self::NidoranF => 40,
            Self::NidoranM => 36,
            Self::Nidorina => 35,
            Self::Nidorino => 39,
            Self::Nidoking => 44,
            Self::Nidoqueen => 39,
            Self::Pichu => 24,
            Self::Cleffa => 17,
            Self::Igglybuff => 19,
            Self::Mareep => 26,
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
            Self::Bellsprout => 12,
            Self::Weepinbell => 12,
            Self::Victreebel => 16,
            Self::Gloom => 18,
            Self::Vileplume => 23,
            Self::Raichu => 25,
            Self::Ekans => 16,
            Self::Arbok => 25,
            Self::Steelix => 32,
            Self::Pineco => 13,
            Self::Forretress => 18,
            Self::Kabuto => 20,
            Self::Kabutops => 27,
            Self::Electrode => 10,
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
            Self::Cyndaquil => 34,
            Self::Totodile => 30,
            Self::Gengar => 27,
            Self::Spearow => 28,
            Self::Wooper => 22,
            Self::Gyarados => 40,
            Self::Sudowoodo => 33,
            Self::Lapras => 49,
            Self::Snorlax => 31,
            Self::Pikachu => 40,
            Self::Bulbasaur => 44,
            Self::Charmander => 43,
            Self::Squirtle => 45,
            Self::Geodude => 29,
            Self::Pidgey => 43,
            Self::Oddish => 19,
            Self::Poliwag => 19,
            Self::Horsea => 24,
            Self::Seadra => 30,
            Self::Kingdra => 35,
            Self::Goldeen => 28,
            Self::Seaking => 29,
            Self::Magikarp => 25,
            Self::Larvitar => 20,
            Self::Pupitar => 9,
            Self::Tyranitar => 34,
            Self::Quilava => 39,
            Self::Typhlosion => 45,
            Self::Charmeleon => 29,
            Self::Charizard => 38,
            Self::Graveler => 22,
            Self::Golem => 29,
            Self::Omanyte => 20,
            Self::Omastar => 26,
            Self::Aerodactyl => 27,
            Self::Dratini => 13,
            Self::Dragonair => 17,
            Self::Dragonite => 34,
            Self::Clefable => 32,
            Self::Wigglytuff => 24,
            Self::Sandshrew => 33,
            Self::Sandslash => 30,
            Self::Politoed => 32,
            Self::Poliwrath => 27,
            Self::Poliwhirl => 25,
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
            Self::Procedural(_) => unreachable!("procedural rigs do not use the GLB schema"),
            Self::Diglett => "f81fd70948ece5d6ade091c74c000f7a9b03ee6aab22981524543f4a07fc38c4",
            Self::Dugtrio => "b3b89e9de63fe28defb7ee03d3cdb8a7efac2c97c9db3e9aae24d2f7bd5f790c",
            Self::Tentacruel => "c21b730c9a3ea4b05d3a7031fff2c30ace66889c6948516b36fe2698a225117b",
            Self::Tentacool => "acb31c9286cd89fc4218aec6bf9706b51fad8a1d0a967ac0e1a158be4b9a1798",
            Self::Jynx => "d2fe39c6dbd02071e663680a8656da8d7e9caf6cc24daf6fb3a2c612b76a0921",
            Self::Primeape => "45c0c8e59be21979e46ab5fe5c7ffbeae3c3ec9ad54858740b918aac26ee564f",
            Self::Mankey => "13470b899782dfb2cb9de04a9579f5d061061c75dbbb95cfe913ed0e01500e52",
            Self::Meowth => "c517d3dbd7ce7b9ebfecbbd4e0d02ded0530254c2a6bef293e92f112d3affdb8",
            Self::Persian => "d0cd25924eb6b4a3c97ab852a94cf24b53e6f65d54dd04ee3b0584804c1fc69b",
            Self::Doduo => "94090facb05decd9d763ba5fc85c0ad67866d7b05745eb98dcae903ab934f190",
            Self::Dodrio => "21ca4a7aeec485dac57a7c5f2a54692259c76b131b22661383cd2e84ef7dd811",
            Self::Pidgeotto => "a9a41d37c7ee99cc44e3be2513069b47edd87bbb73c47abc5e5bf02890f10650",
            Self::FarfetchD => "37528f1600a372ff12c162bd213c8e2fe9099e224aa8028582fa00a94fa76a3c",
            Self::Venusaur => "68b82bacf260770f0ef9db47eb72c1547c3d962acf4dcd15d365cd7a87185673",
            Self::Ivysaur => "05ae76219415afb128f34a59f243f170231d578d3e99a2959d55419aaff5bfda",
            Self::Exeggutor => "22e3a7b6c82b33b797f06d8e8d1e143dfbea7063adfe20f3c345da215d7053ea",
            Self::Exeggcute => "627e7883a07875496f6221bad5ca019ee5a4904ed8071cc21d714d0d60a3df7b",
            Self::Fearow => "9d317f1778d94a75d2c959dd71d7eacfe45293d7171419b9782fdc04ea114f1a",
            Self::Pidgeot => "4290723e8ee8dd56c9e7aba24fc9e65aafe69384cc58f960f2cffdc2f84b7241",
            Self::Entei => "db453e0d91aa89189660d76f63e17f62e26c4d80e750455f9fb7e066cbcc249c",
            Self::Raikou => "06890d739507ac0cd3add1a6da44b954daafcb44f897f9e96e30ea039a597c6f",
            Self::Ledyba => "38b13162e17a6677a6d3c6cc6561f870210a342cbb00e226a8688ee5d5811e2a",
            Self::Ledian => "d8b776f761bf9f6287d975cc158c3ebf2b9f06b2b67479a98ef19e881f202031",
            Self::Ariados => "b680cd4729af2101b86594f951817cb56f56a2c4e03b80209c2863345caa57f4",
            Self::Quagsire => "d97c02d58f80145da2df67f1902083c13ccc862253c75e9332e5d8b04e5e36ba",
            Self::Hoppip => "aaa5bb6c980f1ba8d3327f7f611a059193c29ae7092e35f06acab3b9181507b4",
            Self::Skiploom => "9541ae752be3d0669e5836358537f971a6bf1d42ca2d2d72db639008e0be4987",
            Self::Aipom => "5dfead35f664f46a4a4ae8d8b59a8f22345dae52cf785d5d9a200fed882a0140",
            Self::Bellossom => "21bc079e44b6abd8d1012172d7fb1c909e679d18d143ceafc613241861785f57",
            Self::Qwilfish => "84580db6a0528140ae8fb939c5808b542d23f52df996fb2eb072b8352d1dd6d1",
            Self::Wobbuffet => "4afb5da2de8fcfaaa070c632289d645dc8b67e7790a2f57b19dc3dc3e5b20a09",
            Self::Dunsparce => "97c7da81fb99f3363b601244efa047a94585dd0fa8c4b6faad88f5537b4fde4f",
            Self::Shuckle => "2a7a21948c474962c403086b8e508fc560f2c001c1448311316a0dbb46bc2206",
            Self::Gligar => "9d3742959ab097a791488894bd78c071e6ace9c2dcb6c41b7dbd7e81ced1c10d",
            Self::Yanma => "85a1c4865714a1dd738973cce3244a9b4708e155890fe20061208182eed8114b",
            Self::Murkrow => "1eb3ca435d7fe293a9371188874d4c5e7007e7f2ad87cddd52b1e3202fbbf132",
            Self::Misdreavus => "65520ddfee63c5e0fa3d39643396f88dfe2b9e2fbefd7bc24355cc2491b59935",
            Self::Teddiursa => "38a7cdab920c315dac0608ae661e9f9408e9a3b2709000393b88439a3dafbe3b",
            Self::Ursaring => "3edc0d94b47fe39e7234b958db6c16a2a8254ae548ad21262b4d5994473260a0",
            Self::Snubbull => "70065daa04c182094aef38bc6772adf416da67efaf666cc9bffe28b7b2854412",
            Self::Granbull => "386b9e7d3742f158136ec0acd0401377fefafc9dc1cf76afbdebbebb5467276e",
            Self::Houndour => "f35d0751435b77708a37e8d0c8f5ece071f3de0b65c8fb2586c3775b5d9c1ce6",
            Self::Houndoom => "2d2d89e4bd3ce410dfbbeee0d49659d1c6b78e4f9d0e143e14fa02b54bfde878",
            Self::Miltank => "cab41187fb7632ada815e8d5ef241e6ed3e56af5ba964d6195e44cf5a09e851c",
            Self::Tauros => "9c7a80af7788de4b71e74f6217e1abc0eced54d06e9a6280dbd6fabd57da2edc",
            Self::Delibird => "a1665f55e8b724b62c20970d1f7b663dd06a46707a6c9764531d71e7c6bd1d4d",
            Self::Stantler => "7508b413c731950b19f25b4f91fd096dedd8d449e8158fa2439e06ae8c98d843",
            Self::Girafarig => "6c3c3a187cdce79b3adbe6a17a77a5214cfc55b7c5b0dfed356b2dc46d3bcb4c",
            Self::Donphan => "7a95df265d655ae2c8f7f273c5e42040da683449abe331b9c667955a295f70ca",
            Self::Phanpy => "4f4d743119d23709e102ad164adb17c52d236d9dca587c4ef10b3176e6fab0f4",
            Self::Azumarill => "19aba9a5fabbfd5f187d24b28f9c10fc291c0312046b6142dfa73d3e11219333",
            Self::Marill => "39c4a5f6e20db9f7c4aeddef3353131a3dda11ddd1befd021dfcb5aa1e88fbe7",
            Self::Xatu => "5196df1ef836e954c4904ec06cf3448e3a701eba946050d5a38a477d116e785b",
            Self::Natu => "42b91ce42d9cbfb2367f3640cdd1641c5da72aedb65da5cc41b412d9d8b09fd1",
            Self::Ampharos => "415ea89d9f82cd86b510cb8d455cf9625022765f8475c4cea4dc76f50b0cc765",
            Self::Flaaffy => "eaabe519c0cb6edfa85b9b9ddc5a9b1ca5f5a4fa3bbdedfe948dc57bb68bb8d6",
            Self::Togepi => "975cb4f931bddf2545df3621a30a743f70bb3662f218c61eb5ae90879d34b53b",
            Self::Togetic => "5971f35977b196a3a2a62472e147b9b0743a53573c66770e9b4bdbc892dea912",
            Self::Remoraid => "02f2a0f1061633dd1b6772fc886c1ccce95ddc5237d63a8e3ca29a280a6d0514",
            Self::Octillery => "11c7a88eb486c7a59ae19980866dcfc9a9d0f25510be9472abfaa419a8a2c36f",
            Self::Swinub => "99b43c8174c27d71d246f211d8ca6a0ce2764cb9d6e60cc2153b1fe25ffc0591",
            Self::Piloswine => "b6b3ef74640643f89b461d55f276157aad0cfb2f7a39d17214b9a9eb8d3ea0b9",
            Self::Sneasel => "0509cb264ed9206131887afeee6bb84b852aecfbf0d0743e4c46f08652d0f19a",
            Self::Skarmory => "4a314b8b28c7023ad5044af9003e534b194def610a87be4865b89bde70c1e997",
            Self::Smeargle => "e87a6aa122e9560a34741eb17c164e9ae34f2da00990129878ef18676460b595",
            Self::Moltres => "bc2c5af3c85515bb49c4773acaf16f96232104ac36d08b59681ba4eba064bc33",
            Self::Zapdos => "75512d822cf34c6097ea7c8421a159f7bad0291a985a4ae20ab5ae2642ba041f",
            Self::Articuno => "e38af1c0bb7982e268d4de9841a4722458a42d6d8f23e5abb82d697d7f0b377c",
            Self::Mew => "36d3fc53223d21ac492d3d93879589a130773ba3d884c34a65f6173b3980d37e",
            Self::Mewtwo => "ef903cf6d3fbd2c2b7ed2d8777e1d91726c56940e30f77aaa767e5e50eceb521",
            Self::Heracross => "a33ddd1481ba4cb3435132daf36f3c5770e6b8155e24afc627142a32a82d923d",
            Self::Pinsir => "347418889561cb58a6f9b3f05d441ca8f6bbe2a7b8fb6c4df81ec1a24ea01d84",
            Self::Scizor => "c53bc5cf81b225ae9057e6bb3e1f854cf3903411a8c2b30e59b402b565002a73",
            Self::Scyther => "549a4c15bda5cddc1bb41177a7aac6881f06e09d2e1d245940677f0beb3ea908",
            Self::Starmie => "d9e6623d83cb207c09a01879bc85d5124e8a3a87fcdb6bc93ca79084369a6897",
            Self::Gastly => "ed3adabb2b737302e42e576b7a43375e42045406d494e4beaed7acee1546ce32",
            Self::Haunter => "de932449c4d842c05d74073ea292ea5b1a81cc4e565110568e11bef78b047f73",
            Self::Eevee => "dbb03e378a70bab065c75f1b0dd241eb71f6da0bdf126ba5f2289df74407d633",
            Self::Vulpix => "94d20844527dddce0a41783dddcff88ca7f08abff9b6d2e7aa86aa25dcaa6817",
            Self::Ninetales => "d3c5cac139c338a855ec83edbb1e3df15cbfddb68f7affd4a59f6564301173c3",
            Self::Venonat => "b1425a1645ef672eb73a014ed63285503c3f184ddecb3f782c466bea7163d4f3",
            Self::Venomoth => "7670a960d16e7c38c3e79a8ffe12ba570ca202b6e77bb25f1d7f21d17dcb5323",
            Self::Paras => "9cf3e0d314c5f90ef570f927288a43e2dd32e7421e7d6218202fc1807ec9d82f",
            Self::Parasect => "ee0b479c17022294ba5cbc8084b4d2312aeb028868773ecd51113da09416838b",
            Self::Metapod => "6b3fdcfefd6a88f8417852ca46a6d144a9d4d35c5e5e712b3c42c8b61576b36f",
            Self::Kakuna => "2174b2bfbb87197fe326bd5fd82a05a06dc5539f943618d45a086dcb10d1f805",
            Self::Caterpie => "527fcc82fab88435eb538b1b93614b03f896566ecefafc9d0f6e05e5cb419931",
            Self::Weedle => "ef69247a874d0f1cec8f314d3885cd93ddaa5356fd45ce850dda1f75a0a162ea",
            Self::Rhydon => "c0131e43f7235c93fc5a645ab0c3e69f63dc8c2f8d8aa36e0134cc89ab22b4f7",
            Self::Drowzee => "1789c22bcf0b67308e90a00a00965c00c6e42b8f5bde57c782bd0d930c4e0859",
            Self::Hypno => "7e70103e4d8495bf4b78adeb41eca93860c7bde563ec30811f43098380997915",
            Self::Abra => "0f62090ca5311bb42f9f3a8375859f0336df8bfe1e76752e34588c9deb257eb9",
            Self::Alakazam => "9798ee114b47db333f11ac0dfddb761373146d62684df9051613f8585fdb4f5c",
            Self::Kadabra => "d027eeb4ab22721959745884fec132d2073f9ed1671cb3088292166a42360f58",
            Self::Koffing => "ba5c5ad08e422850b832e4cb9412e18bcecbc53b62530186b11fa4f879b957de",
            Self::Weezing => "b655fa7a75abd024c672927149600b45231e833200036b2c7bc97ed9ddba0c70",
            Self::Wartortle => "8f76359639251ae9161bf295201020e8e4bafafe252c9f286fd3812218eb0a58",
            Self::Blastoise => "0e4dfa76fc3b7fd8a1a2360552fa7c2663e821fd88a442bb899b32dd6e859548",
            Self::Croconaw => "5e931c599dd42d7e2ebbf57fa8174ae66023d02dea8875b09730c1efc1a1697f",
            Self::Feraligatr => "d80bb181fae2556e57b3ef65e60fb42f4e816f7008c5e72ca1db8d1979719cad",
            Self::Cubone => "5bd078b289b719850ffbd5621c43b25276736ac2bc919b1e60460c9e84814d77",
            Self::Marowak => "a72e2b8c3dd27fc1f022bc51c29880bc5cf789e205311bfe86d4436d6ef30c0a",
            Self::Magnemite => "fcd9ef30d31caba9b51c6fc627b4bf0b68a20edf961777c9477acdb7b96904ce",
            Self::Magneton => "432c67fcf83876db5de5b5e9c0dc9ae27df2a6180760e7a0325e394dfccb7f8f",
            Self::Tangela => "9122c50cf61e0829b7d88147233af06cdfc8e5ba7ee98d917c864bc2b918b416",
            Self::Kangaskhan => "d05bf3560699741a45d687e8c44745f0ba667d37fa551a0c1d7b16ef90cffc94",
            Self::Ponyta => "586c58b6e344ad355eed3521ceeb585a241b17255a01a5ae26bb2fb4f3b185ce",
            Self::Rapidash => "c56d7e051ad01b2f418fca26b70baa6f9c682bc2048ad11e790fee0401ec156f",
            Self::Machop => "6847f6ddd211164ff5bdcffb5dc94b104dd87fdd3955610b53b611d728b5d71f",
            Self::Machoke => "4b0b6d1f8f12e160831ba3ec746729bf11fcceffffec0fa4c9ef8932bd8313ae",
            Self::Machamp => "c70bd3ba26597bfadc1fafae6d459a9334b41bd63b22f022578d5a6db4793ca6",
            Self::Rhyhorn => "e6ad87610f6a2b99b8e1780fddffabfb97166fdb1c8d50278ca6e765dcbc0188",
            Self::Dewgong => "0efac101ffd20a33f75f7f67954878fad79f1323bf867e348cab34588f58a7ff",
            Self::Seel => "e1b358039ce8dba65f99bfde6d01438da48fa17f8532ea107117e290e7fae852",
            Self::Ditto => "1a3fc7b87714f7c4c5b82db35cecc4fb7bc3d3679f52f0ccf2336727626327e8",
            Self::Muk => "373608073b4024110aaaf4e23d345ca1c07c2fd6a2725f72fe985893224ca8c3",
            Self::Grimer => "01ccaddd323bf25f60a019cbc0642316ec52af5c7cd650ecc625d54d607c3a03",
            Self::Cloyster => "714eb4a74d131c5a8b303d1fde4af07a30dd0c739fa0741176cee1efbb336d74",
            Self::Shellder => "9603bdedd0b70757c615251f8ea5ea6b258a33c53c3ce6ab434f2f441edd95e9",
            Self::Kingler => "77ef337fdce8469cd785876b536b3c92953ebd6a1b026f7b12001c2a592cfc36",
            Self::Krabby => "5f8f12d336b23cecec7ad64a1ef4ab89f387324289654f1fdbc722df900ff5ef",
            Self::Umbreon => "c627b4d60cb817a8e18596f759e06d97b6b57497c9af7badaea772b215b77e03",
            Self::Jolteon => "3e154fb857ffb4531d84ef1cf35b3ec944ab4d59bbb5b0da316722aa0f6af022",
            Self::Flareon => "2bdb14269978d1eaed0163530f5ecd9a966b45b5501db0927542151ec15268b5",
            Self::Espeon => "051135bb83bc3ca1a337c80500fa056725f8b3fa59436c1b5267d195d4d5eb4e",
            Self::Vaporeon => "0d92fd28cf7c31c03b49e55538d760de684e22797e9a32a7ccfba31790e84a81",
            Self::Celebi => "94a8a577208273651ba7294a48345f701b30142a260b5fc2e920b21e252837fb",
            Self::HoOh => "4e246f222c484ed35240a4bd22ce63b024b482e85b707d9968dbea9da3fac218",
            Self::Lugia => "8bfe309e8e03304e4547f34346df87add36f0706bed92b5cc7b589b57cf4001b",
            Self::Corsola => "70a819e196fadc5fade144ed07e3483aa3a404f9c1a373f572975adb9d9f65d2",
            Self::Mantine => "712a8adc15f3cc0fde8aab46912528e87185c6ea8c953ec4283a9f00db1dd7b7",
            Self::Porygon => "d2c2a841701ee9ed8d38fdc4c27b87b5e1f5544ec665ebba0372b4f1fba4abb7",
            Self::Porygon2 => "449d2c9aa5b3d054b2a96818696b9e677a72369d926bfa80e39093976bcb08ac",
            Self::Chinchou => "0924cdb724dca3b1b63d2893ac161eda23672ff9052be7728c7d84b216c52828",
            Self::Lanturn => "50ecc25097080930e9c2f4d69a37ea5388ae765b0c4289cfd4b91e6395944019",
            Self::Slugma => "035d33384e2d2375f1a6ad51fee8bc97efe63d2449179ac3fcc79f83522d8ade",
            Self::Magcargo => "47cde235b8a7adf55ab9c3c7eeaeb86d824ec44959fe3bf46e7764a8b74cbe5b",
            Self::Hitmonlee => "ac4facbaf5bc5ea34f73342afaa5bc98e8ffa962e8704c7c7d34da37f8ffd0ac",
            Self::Hitmonchan => "59c5d015861346b850e7aca45fa3056246a8d8bc950ce5d9793a0035d7856daf",
            Self::Hitmontop => "554e1edaafe9860ab6aa4b5c306a67549b3f8e7c2773e3adefad4901867a3de3",
            Self::Tyrogue => "6f08d0c81d0c633e32fd55a565109a56759eeb7250212e0e4dfef93812dc71a2",
            Self::Smoochum => "90026857a3c50573313f78f6fbd0fd12ed25f7cdae53bbd869a6a0f5e2c792d2",
            Self::Elekid => "5087eaed56db849c823c7e069ef7879375ebb39c6d3f68c84de469b858ad076a",
            Self::Magby => "aab2245cffeaa6b35e780fc79dee5fa83f7d6b482103151f3e1cfd6acec8c60b",
            Self::Magmar => "3463aa43d0d0b9d5d979c0a55071283a3ef1a1893d053a2ffa97e5361dd0c83d",
            Self::Electabuzz => "b043a49e3f3885b12adb6401eee1999cb9dcaa956952ca39e300cabc97bc0c34",
            Self::Slowking => "c66499f5fae82d1d849da07acc4b7d5e48ce75270cac872bdee5681bb1f5320b",
            Self::Slowbro => "33741cd0059c20dfaee09820fe9898e00eee22e21fce8e153eda4985ed38fc14",
            Self::Lickitung => "972f51bfd45847d1dcda324c713ac66537d2eb19a6a9407ae395279d9bc99156",
            Self::MrMime => "f2409770cbce156b43cca909213fbcdcbda84094b050618144544ef3e3eeb81d",
            Self::Chansey => "9b0d87c52e08fad76e7ee66f2983e075382f8a3da37b989e4cc025be0061021e",
            Self::Blissey => "316186bb6e6a3a5c2eac8d4040d19e52536c3d642bdb23d6dba0288056f62a0e",
            Self::Growlithe => "93977ea316f368e22fd7a1061f565e63ca5fd01264a654a6b628f2d65961ab99",
            Self::Arcanine => "b8b53169922d6cb2ea77f87da6f84f68bb50279d875021a4bec7b884a9382b7e",
            Self::NidoranF => "8bd3d7aefc6dd2bb59d701de719be620443ce0c1fc4aaaeb4bcdecc0a9c03802",
            Self::NidoranM => "5d11703a3c1639cd8682d31e6d0466e48540efb084afadfbc04d29dfde753c81",
            Self::Nidorina => "85c4a8e56049919dc4ca944a15f2711111399da8b3542cf1d4e38fffdd3f9b5e",
            Self::Nidorino => "6a11581fdc1fd23d94214c2430bf1c33f8bd92bccc9e94e7abbffdf8f9ccd688",
            Self::Nidoking => "0730055ac57991e822b343db39eb9020d0416bbd179956f8e49ee721f937c346",
            Self::Nidoqueen => "5b9950cf9bb193673825a3db9214c65d3f2eb037367486e563ec310be35d6132",
            Self::Pichu => "45b03dd0a788ebf6714334c7c25ab472bab589aa9d5b70e2512ad105b7e992be",
            Self::Cleffa => "25d7809325a90fc9b89a9451ee3785256001f63b95cfb0a1568e7a3d7bd45725",
            Self::Igglybuff => "de4ed97b7ae946d32303e70a839c3880f24b1a01e0036b72be5aa78bc97010a4",
            Self::Mareep => "058c61bce99e43bc24f2bf17a0cabe8bb62a0a6475fcefcd081d34cf9f580c8d",
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
            Self::Bellsprout => "34c7d0c2b0acffea6b36f16c6ec988186e5bb2c1b5906c6781c805600ed1ae32",
            Self::Weepinbell => "2f702734e363aaa15b2bc88e89e5ca88b6ad31a7517a6980bdae11926eb3c9ab",
            Self::Victreebel => "efb9f3f09ee0e691fc9a3acd3675e9c23b86e86ff0ecbaccdc97411cd2958923",
            Self::Gloom => "1d5f919f3795501e874632168a05d657bb5bddab837adffb6c017924a16cdcad",
            Self::Vileplume => "0df615733458f9a0fa2a45bce3bf68a390fd8c8da2cb0fcd329a9138411acb3c",
            Self::Raichu => "0352f76630694336e563c1def6366e9da3d479fd90b43e219c0d57c0fe89c4f0",
            Self::Ekans => "eaf2ebad196f288b8c3827336f2ca7c90dd56070ced67699b45e13427dfbb6e4",
            Self::Arbok => "2db1012c5b11414aee4a5b75c1517805dcf4be91989a2f854a74c6df71830790",
            Self::Steelix => "1868650cc107f68412c8bd5614b9e2ae7db2b6f35b5db834e3213dc97ec556d0",
            Self::Pineco => "57da153652ef565881c64f8a0874e22e182a0c1ce7eee3985311a145835167f8",
            Self::Forretress => "5dbc71683b7f28a531a606b07e72f102a64dd20fa39fdd3938c2848429d19b5a",
            Self::Kabuto => "46446b487607972de83a305147cd15a15e66dc7e9b15d3dda2c226c112864fa6",
            Self::Kabutops => "425d7b4409372617d60635989f3d8993b5733b4c91754397bb39c056a27ce7b6",
            Self::Electrode => "cc8828f83e653417b3a084b8e0ca908deb1f3a5fe4acfea67e831e87050e7571",
            Self::Staryu => "c8ef0c854e722452e6850b79cd7df6b66598e7b21488b69ba818c5dc80e87a67",
            Self::Sentret => "0a15f7e364cdec1b70a33e5e68df64620b7824223129e72dc9184606b8ce4485",
            Self::Hoothoot => "93fbea6a47abff0fff1f6ee6315f7ac09ea26bb1bf60a26b0d293a8ae7784555",
            Self::Butterfree => "4b94b9ecfabca7dee88ec0d5b725d50116cb1f2a139d5a2817342e53cd4ee550",
            Self::Beedrill => "998ef3cf0f5f76c6ada93622528f1806dab5361c60d1b8ecb43c991f1979b900",
            Self::Zubat => "737d2dad1c97b82b0fead248d72ac003e5e759ce90debf0c4cabb06d80c851d0",
            Self::Golbat => "2b4111963c9a7224224eacf4a70ec2359fd0e36d6426c7a992543e66e763379a",
            Self::Crobat => "87f89cd4cc0e7d68b7f29292721ea068514dcdc982722d76fddcb8d23f69ced4",
            Self::Psyduck => "4d28e9c51e3d5ed94fcccfe356d5cc60dc9ee2d7645332334c7bb7d6ef5b9b4c",
            Self::Golduck => "e7ab59e4de9d3d78f761557cc5b62d6e41442f54c3e25b7fffbecba18892178d",
            Self::Raticate => "f6043d52ed1588580d3e4c8f44480d8d3cb2b8b2f268e6ccd7220287d76edd0d",
            Self::Rattata => "ce7c14e7cf1694a4131d3ed3e3c709fe51f9b2ff7618089d3c40bf731aeb8a3d",
            Self::Squirtle => "222004eaea9a5593380331ab51391d1ddfdcda105fc35c11ebe53835e4e341d7",
            Self::Geodude => "90658750b47d9380750c88431617ae2860b5f5cdb8e6d1d0af8fb320124ab33d",
            Self::Pidgey => "814a75e55669f0361edc7c35026a7a0b112085cda12d4dcc485693bfd1990fb7",
            Self::Oddish => "b9c42e2be84290dfd4b02c68370d07d6f235fb3f7ea34826d677a1faaef29384",
            Self::Poliwag => "2914f4cf3bdb2b4ea6f4795e1586e3d93f4dee15f5b0050ca2981fdcc37c2f2d",
            Self::Horsea => "9ced391e14c1462d6aec4e5fbb3cb7ea1882fe08cde7b7f9ab6f62a02c08c133",
            Self::Seadra => "77563b2fa80988f87c72a2e83f27d69fcd5eb992cbc3f8585d30819ed651aa89",
            Self::Kingdra => "630e46aff5bc1a55ef3e52dbacfb4617e6be6b2a5381389b8ee019795bfda69e",
            Self::Goldeen => "248b6d97497e6c5d260216233c6692c8d2636c2f99573ee5302025acc47bba31",
            Self::Seaking => "598ef7f5ae399ebef8d5bed97e398152862673a02de191b4cade5bb6c2ea9b57",
            Self::Magikarp => "75e6d1ec0cd9b364d8c982351adf6acd23f477048cbea91a461d53daf622aabc",
            Self::Larvitar => "61cd43cca9f55febba20701c2ce9244510d85dd4a826f4476c903f0815bb9706",
            Self::Pupitar => "b972d03ec14adb898e754bb68511dad4e7f7f131aee538229c455b77ecc3a492",
            Self::Tyranitar => "ab12485cb05e61b1722d768f92af6cc5151592507139262e80a58ce2c360db33",
            Self::Quilava => "a93592c6a5759ecb4c8dec112b2a655d5283c6514da75bfdb4ba66f4b8ce7650",
            Self::Typhlosion => "ed5bd1e628a7763fd30dce90c6d0e7fa6c09a8c8d5d93bb35eebafe28ae81402",
            Self::Charmeleon => "5c7062be45d9caa717b32a0dd665c207e4b6e563add9971868dee91edc343007",
            Self::Charizard => "20ca8700dd7dbe8c6db55ee12f00bae240ee62f1a39255d3aa06dec9d01ef05b",
            Self::Graveler => "6c318d8d37db542a1b5cf1901ab815c6ff9674aba735a626ae390a3c8b676049",
            Self::Golem => "9b16cb2ddc7577b4f64ae437f97e894b2154f75f5626f1dab4fcb1564a0b9671",
            Self::Omanyte => "9df10630971b62ea925b0d03f556b92a04dcfe75d25ad66bbe7f0c7965f86246",
            Self::Omastar => "2df6740a96834863a7812b4ca47f7fbf799992bb8eab0541f445325d864fb4ee",
            Self::Aerodactyl => "09cc8fb6a4257c7daabf5eb7a77b4bea649eaf9ec6f5b0235cc8cedce0a7503a",
            Self::Dratini => "53985a5d2655309199dcc14faf9c3e554d04e0fb273d8bdfcaa67b3cc64a75f7",
            Self::Dragonair => "3effe7b64485d15be8a2b9d78c14ad80dd74dcab79c10b169f9145f13391efc2",
            Self::Dragonite => "8fe63ca60d85a0dc97632a7f182a368a91d0f8fdba3fba2f9da92eb2d3524c3f",
            Self::Clefable => "2dcfdca000ef1ccc8fa751fe5e95753104aa8097685d5200ff18cb6dbcf1c3e8",
            Self::Wigglytuff => "ba8c7ae0e3fadd9d0a8f91ae55d2ab67ae4e2aed3432ea20df0254265ccf7614",
            Self::Sandshrew => "2d57a940df4c3b341cf18d234d61cf58c391b2175e1bde7022a7afccc1386b8f",
            Self::Sandslash => "fddac7741a7fbab34202801708542b62ccd5cd3a901a46bc37209062e627301d",
            Self::Politoed => "8d04df99bf27b790d08bd554c5cbc4762bcc345acd6cba1fea600867cc5c8c44",
            Self::Poliwrath => "3628cbf3a335a904aebdfa48f9ecc48d68033f9449107546b6fe256e7364d09e",
            Self::Poliwhirl => "331c07cd646e2dd21412c2243507420d69862a69a0acaac3086eb3cafbbe4394",
            Self::Clefairy => "0d173aea43c62408317aabf296a57c3a202a674df5464cd94b29fc04e872accc",
            Self::Jigglypuff => "be7a901fb200b5528a7ec3fbba34c9944776fdb953d1eb15b69f58cf9419664e",
            Self::Charmander => "6ae03caf697cdaa11acbbb4d2bc90af86f0db04ecf02b64c7e7342a5aed72bfa",
            Self::Bulbasaur => "2789ad2986d637dbe83e6b3b4eb899dfe9ccd06d0746c3332053e72f3bd033df",
            Self::Pikachu => "4bdd281a376a6427e9ee966c69b650d349367033a58f70614987a2e8b1065387",
            Self::Snorlax => "91058a0c15e86e5e6ff07f98ff22bafbc6cd0752f2e861fe80c74ea58a74426c",
            Self::Lapras => "7a083c865c869d1fa5f832195c3ed2ec042b1267773294cc39fe47cd3c6b789b",
            Self::Sudowoodo => "010b92720972b99c2bee10df2b6a074d605fce9fef79d4f8eef8c49b2e079275",
            Self::Gyarados => "555f2c6aace148f0fa0e3412cdb5912581b5f6d6f60341c6eeaecdcb7d4ac045",
            Self::Wooper => "d23893a24e7dd83f9711b2309ece7d7b24094ea0184ffa86237d9ab059e0bf88",
            Self::Chikorita => "0e77b09b2db90bbc252933d54d4fa3b7948e464e2d2c27cbd38f81efe43a5142",
            Self::Bayleef => "4f0f8b38c600b13d5180cbf39719a5b4620b2e5eed525e79bd8100ad03762891",
            Self::Meganium => "a2305f2aca111722d9bf59710f5be879cc4bc09f2a7f4d0c9736f5b87bad339c",
            Self::Cyndaquil => "343dc219a6151db3baa8b52459f911b33746aa2cac61ad77497efd434bd4d9ea",
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
    pub hover_clearance: f32,
    pub motion_limbs: Vec<motion::MotionLimb>,
    pub crawl_segments: Vec<motion::CrawlSegment>,
    pub walk_legs: Vec<motion::WalkLeg>,
    pub walk_cycle: f32,
    pub ground_vertices: Vec<usize>,
    /// Parent-local trailing folds used only by stranded world fish.
    pub flop_folds: Vec<(usize, Quat)>,
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
    static BELLSPROUT: OnceLock<SpeciesRig> = OnceLock::new();
    static WEEPINBELL: OnceLock<SpeciesRig> = OnceLock::new();
    static VICTREEBEL: OnceLock<SpeciesRig> = OnceLock::new();
    static GLOOM: OnceLock<SpeciesRig> = OnceLock::new();
    static VILEPLUME: OnceLock<SpeciesRig> = OnceLock::new();
    static RAICHU: OnceLock<SpeciesRig> = OnceLock::new();
    static EKANS: OnceLock<SpeciesRig> = OnceLock::new();
    static ARBOK: OnceLock<SpeciesRig> = OnceLock::new();
    static STEELIX: OnceLock<SpeciesRig> = OnceLock::new();
    static PINECO: OnceLock<SpeciesRig> = OnceLock::new();
    static FORRETRESS: OnceLock<SpeciesRig> = OnceLock::new();
    static KABUTO: OnceLock<SpeciesRig> = OnceLock::new();
    static KABUTOPS: OnceLock<SpeciesRig> = OnceLock::new();
    static ELECTRODE: OnceLock<SpeciesRig> = OnceLock::new();
    static ONIX: OnceLock<SpeciesRig> = OnceLock::new();
    static DIGLETT: OnceLock<SpeciesRig> = OnceLock::new();
    static DUGTRIO: OnceLock<SpeciesRig> = OnceLock::new();
    static TENTACRUEL: OnceLock<SpeciesRig> = OnceLock::new();
    static TENTACOOL: OnceLock<SpeciesRig> = OnceLock::new();
    static JYNX: OnceLock<SpeciesRig> = OnceLock::new();
    static PRIMEAPE: OnceLock<SpeciesRig> = OnceLock::new();
    static MANKEY: OnceLock<SpeciesRig> = OnceLock::new();
    static MEOWTH: OnceLock<SpeciesRig> = OnceLock::new();
    static PERSIAN: OnceLock<SpeciesRig> = OnceLock::new();
    static DODUO: OnceLock<SpeciesRig> = OnceLock::new();
    static DODRIO: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEOTTO: OnceLock<SpeciesRig> = OnceLock::new();
    static FARFETCH_D: OnceLock<SpeciesRig> = OnceLock::new();
    static VENUSAUR: OnceLock<SpeciesRig> = OnceLock::new();
    static IVYSAUR: OnceLock<SpeciesRig> = OnceLock::new();
    static EXEGGUTOR: OnceLock<SpeciesRig> = OnceLock::new();
    static EXEGGCUTE: OnceLock<SpeciesRig> = OnceLock::new();
    static FEAROW: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEOT: OnceLock<SpeciesRig> = OnceLock::new();
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
    static BELLOSSOM: OnceLock<SpeciesRig> = OnceLock::new();
    static QWILFISH: OnceLock<SpeciesRig> = OnceLock::new();
    static WOBBUFFET: OnceLock<SpeciesRig> = OnceLock::new();
    static DUNSPARCE: OnceLock<SpeciesRig> = OnceLock::new();
    static SHUCKLE: OnceLock<SpeciesRig> = OnceLock::new();
    static GLIGAR: OnceLock<SpeciesRig> = OnceLock::new();
    static YANMA: OnceLock<SpeciesRig> = OnceLock::new();
    static MURKROW: OnceLock<SpeciesRig> = OnceLock::new();
    static MISDREAVUS: OnceLock<SpeciesRig> = OnceLock::new();
    static TEDDIURSA: OnceLock<SpeciesRig> = OnceLock::new();
    static URSARING: OnceLock<SpeciesRig> = OnceLock::new();
    static SNUBBULL: OnceLock<SpeciesRig> = OnceLock::new();
    static GRANBULL: OnceLock<SpeciesRig> = OnceLock::new();
    static HOUNDOUR: OnceLock<SpeciesRig> = OnceLock::new();
    static HOUNDOOM: OnceLock<SpeciesRig> = OnceLock::new();
    static MILTANK: OnceLock<SpeciesRig> = OnceLock::new();
    static TAUROS: OnceLock<SpeciesRig> = OnceLock::new();
    static DELIBIRD: OnceLock<SpeciesRig> = OnceLock::new();
    static STANTLER: OnceLock<SpeciesRig> = OnceLock::new();
    static GIRAFARIG: OnceLock<SpeciesRig> = OnceLock::new();
    static DONPHAN: OnceLock<SpeciesRig> = OnceLock::new();
    static PHANPY: OnceLock<SpeciesRig> = OnceLock::new();
    static AZUMARILL: OnceLock<SpeciesRig> = OnceLock::new();
    static MARILL: OnceLock<SpeciesRig> = OnceLock::new();
    static XATU: OnceLock<SpeciesRig> = OnceLock::new();
    static NATU: OnceLock<SpeciesRig> = OnceLock::new();
    static AMPHAROS: OnceLock<SpeciesRig> = OnceLock::new();
    static FLAAFFY: OnceLock<SpeciesRig> = OnceLock::new();
    static TOGEPI: OnceLock<SpeciesRig> = OnceLock::new();
    static TOGETIC: OnceLock<SpeciesRig> = OnceLock::new();
    static REMORAID: OnceLock<SpeciesRig> = OnceLock::new();
    static OCTILLERY: OnceLock<SpeciesRig> = OnceLock::new();
    static SWINUB: OnceLock<SpeciesRig> = OnceLock::new();
    static PILOSWINE: OnceLock<SpeciesRig> = OnceLock::new();
    static SNEASEL: OnceLock<SpeciesRig> = OnceLock::new();
    static SKARMORY: OnceLock<SpeciesRig> = OnceLock::new();
    static SMEARGLE: OnceLock<SpeciesRig> = OnceLock::new();
    static MOLTRES: OnceLock<SpeciesRig> = OnceLock::new();
    static ZAPDOS: OnceLock<SpeciesRig> = OnceLock::new();
    static ARTICUNO: OnceLock<SpeciesRig> = OnceLock::new();
    static MEW: OnceLock<SpeciesRig> = OnceLock::new();
    static MEWTWO: OnceLock<SpeciesRig> = OnceLock::new();
    static HERACROSS: OnceLock<SpeciesRig> = OnceLock::new();
    static PINSIR: OnceLock<SpeciesRig> = OnceLock::new();
    static SCIZOR: OnceLock<SpeciesRig> = OnceLock::new();
    static SCYTHER: OnceLock<SpeciesRig> = OnceLock::new();
    static STARMIE: OnceLock<SpeciesRig> = OnceLock::new();
    static GASTLY: OnceLock<SpeciesRig> = OnceLock::new();
    static HAUNTER: OnceLock<SpeciesRig> = OnceLock::new();
    static EEVEE: OnceLock<SpeciesRig> = OnceLock::new();
    static VULPIX: OnceLock<SpeciesRig> = OnceLock::new();
    static NINETALES: OnceLock<SpeciesRig> = OnceLock::new();
    static VENONAT: OnceLock<SpeciesRig> = OnceLock::new();
    static VENOMOTH: OnceLock<SpeciesRig> = OnceLock::new();
    static PARAS: OnceLock<SpeciesRig> = OnceLock::new();
    static PARASECT: OnceLock<SpeciesRig> = OnceLock::new();
    static METAPOD: OnceLock<SpeciesRig> = OnceLock::new();
    static KAKUNA: OnceLock<SpeciesRig> = OnceLock::new();
    static CATERPIE: OnceLock<SpeciesRig> = OnceLock::new();
    static WEEDLE: OnceLock<SpeciesRig> = OnceLock::new();
    static RHYDON: OnceLock<SpeciesRig> = OnceLock::new();
    static DROWZEE: OnceLock<SpeciesRig> = OnceLock::new();
    static HYPNO: OnceLock<SpeciesRig> = OnceLock::new();
    static ABRA: OnceLock<SpeciesRig> = OnceLock::new();
    static ALAKAZAM: OnceLock<SpeciesRig> = OnceLock::new();
    static KADABRA: OnceLock<SpeciesRig> = OnceLock::new();
    static KOFFING: OnceLock<SpeciesRig> = OnceLock::new();
    static WEEZING: OnceLock<SpeciesRig> = OnceLock::new();
    static WARTORTLE: OnceLock<SpeciesRig> = OnceLock::new();
    static BLASTOISE: OnceLock<SpeciesRig> = OnceLock::new();
    static CROCONAW: OnceLock<SpeciesRig> = OnceLock::new();
    static FERALIGATR: OnceLock<SpeciesRig> = OnceLock::new();
    static CUBONE: OnceLock<SpeciesRig> = OnceLock::new();
    static MAROWAK: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGNEMITE: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGNETON: OnceLock<SpeciesRig> = OnceLock::new();
    static TANGELA: OnceLock<SpeciesRig> = OnceLock::new();
    static KANGASKHAN: OnceLock<SpeciesRig> = OnceLock::new();
    static PONYTA: OnceLock<SpeciesRig> = OnceLock::new();
    static RAPIDASH: OnceLock<SpeciesRig> = OnceLock::new();
    static MACHOP: OnceLock<SpeciesRig> = OnceLock::new();
    static MACHOKE: OnceLock<SpeciesRig> = OnceLock::new();
    static MACHAMP: OnceLock<SpeciesRig> = OnceLock::new();
    static RHYHORN: OnceLock<SpeciesRig> = OnceLock::new();
    static DEWGONG: OnceLock<SpeciesRig> = OnceLock::new();
    static SEEL: OnceLock<SpeciesRig> = OnceLock::new();
    static DITTO: OnceLock<SpeciesRig> = OnceLock::new();
    static MUK: OnceLock<SpeciesRig> = OnceLock::new();
    static GRIMER: OnceLock<SpeciesRig> = OnceLock::new();
    static CLOYSTER: OnceLock<SpeciesRig> = OnceLock::new();
    static SHELLDER: OnceLock<SpeciesRig> = OnceLock::new();
    static KINGLER: OnceLock<SpeciesRig> = OnceLock::new();
    static KRABBY: OnceLock<SpeciesRig> = OnceLock::new();
    static UMBREON: OnceLock<SpeciesRig> = OnceLock::new();
    static JOLTEON: OnceLock<SpeciesRig> = OnceLock::new();
    static FLAREON: OnceLock<SpeciesRig> = OnceLock::new();
    static ESPEON: OnceLock<SpeciesRig> = OnceLock::new();
    static VAPOREON: OnceLock<SpeciesRig> = OnceLock::new();
    static CELEBI: OnceLock<SpeciesRig> = OnceLock::new();
    static HO_OH: OnceLock<SpeciesRig> = OnceLock::new();
    static LUGIA: OnceLock<SpeciesRig> = OnceLock::new();
    static CORSOLA: OnceLock<SpeciesRig> = OnceLock::new();
    static MANTINE: OnceLock<SpeciesRig> = OnceLock::new();
    static PORYGON: OnceLock<SpeciesRig> = OnceLock::new();
    static PORYGON2: OnceLock<SpeciesRig> = OnceLock::new();
    static CHINCHOU: OnceLock<SpeciesRig> = OnceLock::new();
    static LANTURN: OnceLock<SpeciesRig> = OnceLock::new();
    static SLUGMA: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGCARGO: OnceLock<SpeciesRig> = OnceLock::new();
    static HITMONLEE: OnceLock<SpeciesRig> = OnceLock::new();
    static HITMONCHAN: OnceLock<SpeciesRig> = OnceLock::new();
    static HITMONTOP: OnceLock<SpeciesRig> = OnceLock::new();
    static TYROGUE: OnceLock<SpeciesRig> = OnceLock::new();
    static SMOOCHUM: OnceLock<SpeciesRig> = OnceLock::new();
    static ELEKID: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGBY: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGMAR: OnceLock<SpeciesRig> = OnceLock::new();
    static ELECTABUZZ: OnceLock<SpeciesRig> = OnceLock::new();
    static SLOWKING: OnceLock<SpeciesRig> = OnceLock::new();
    static SLOWBRO: OnceLock<SpeciesRig> = OnceLock::new();
    static LICKITUNG: OnceLock<SpeciesRig> = OnceLock::new();
    static MR_MIME: OnceLock<SpeciesRig> = OnceLock::new();
    static CHANSEY: OnceLock<SpeciesRig> = OnceLock::new();
    static BLISSEY: OnceLock<SpeciesRig> = OnceLock::new();
    static GROWLITHE: OnceLock<SpeciesRig> = OnceLock::new();
    static ARCANINE: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORAN_F: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORAN_M: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORINA: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDORINO: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDOKING: OnceLock<SpeciesRig> = OnceLock::new();
    static NIDOQUEEN: OnceLock<SpeciesRig> = OnceLock::new();
    static PICHU: OnceLock<SpeciesRig> = OnceLock::new();
    static CLEFFA: OnceLock<SpeciesRig> = OnceLock::new();
    static IGGLYBUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static MAREEP: OnceLock<SpeciesRig> = OnceLock::new();
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
    static HORSEA: OnceLock<SpeciesRig> = OnceLock::new();
    static SEADRA: OnceLock<SpeciesRig> = OnceLock::new();
    static KINGDRA: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLDEEN: OnceLock<SpeciesRig> = OnceLock::new();
    static SEAKING: OnceLock<SpeciesRig> = OnceLock::new();
    static MAGIKARP: OnceLock<SpeciesRig> = OnceLock::new();
    static LARVITAR: OnceLock<SpeciesRig> = OnceLock::new();
    static PUPITAR: OnceLock<SpeciesRig> = OnceLock::new();
    static TYRANITAR: OnceLock<SpeciesRig> = OnceLock::new();
    static QUILAVA: OnceLock<SpeciesRig> = OnceLock::new();
    static TYPHLOSION: OnceLock<SpeciesRig> = OnceLock::new();
    static CHARMELEON: OnceLock<SpeciesRig> = OnceLock::new();
    static CHARIZARD: OnceLock<SpeciesRig> = OnceLock::new();
    static GRAVELER: OnceLock<SpeciesRig> = OnceLock::new();
    static GOLEM: OnceLock<SpeciesRig> = OnceLock::new();
    static OMANYTE: OnceLock<SpeciesRig> = OnceLock::new();
    static OMASTAR: OnceLock<SpeciesRig> = OnceLock::new();
    static AERODACTYL: OnceLock<SpeciesRig> = OnceLock::new();
    static DRATINI: OnceLock<SpeciesRig> = OnceLock::new();
    static DRAGONAIR: OnceLock<SpeciesRig> = OnceLock::new();
    static DRAGONITE: OnceLock<SpeciesRig> = OnceLock::new();
    static CLEFABLE: OnceLock<SpeciesRig> = OnceLock::new();
    static WIGGLYTUFF: OnceLock<SpeciesRig> = OnceLock::new();
    static SANDSHREW: OnceLock<SpeciesRig> = OnceLock::new();
    static SANDSLASH: OnceLock<SpeciesRig> = OnceLock::new();
    static POLITOED: OnceLock<SpeciesRig> = OnceLock::new();
    static POLIWRATH: OnceLock<SpeciesRig> = OnceLock::new();
    static POLIWHIRL: OnceLock<SpeciesRig> = OnceLock::new();
    static ODDISH: OnceLock<SpeciesRig> = OnceLock::new();
    static PIDGEY: OnceLock<SpeciesRig> = OnceLock::new();
    static GEODUDE: OnceLock<SpeciesRig> = OnceLock::new();
    static SQUIRTLE: OnceLock<SpeciesRig> = OnceLock::new();
    let cache = match species {
        Species::Procedural(name) => return crate::battle_species_models::rig(name).expect("catalogued species rig"),
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
        Species::Horsea => &HORSEA,
        Species::Seadra => &SEADRA,
        Species::Kingdra => &KINGDRA,
        Species::Goldeen => &GOLDEEN,
        Species::Seaking => &SEAKING,
        Species::Magikarp => &MAGIKARP,
        Species::Larvitar => &LARVITAR,
        Species::Pupitar => &PUPITAR,
        Species::Tyranitar => &TYRANITAR,
        Species::Quilava => &QUILAVA,
        Species::Typhlosion => &TYPHLOSION,
        Species::Charmeleon => &CHARMELEON,
        Species::Charizard => &CHARIZARD,
        Species::Graveler => &GRAVELER,
        Species::Golem => &GOLEM,
        Species::Omanyte => &OMANYTE,
        Species::Omastar => &OMASTAR,
        Species::Aerodactyl => &AERODACTYL,
        Species::Dratini => &DRATINI,
        Species::Dragonair => &DRAGONAIR,
        Species::Dragonite => &DRAGONITE,
        Species::Clefable => &CLEFABLE,
        Species::Wigglytuff => &WIGGLYTUFF,
        Species::Sandshrew => &SANDSHREW,
        Species::Sandslash => &SANDSLASH,
        Species::Politoed => &POLITOED,
        Species::Poliwrath => &POLIWRATH,
        Species::Poliwhirl => &POLIWHIRL,
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
        Species::Bellsprout => &BELLSPROUT,
        Species::Weepinbell => &WEEPINBELL,
        Species::Victreebel => &VICTREEBEL,
        Species::Gloom => &GLOOM,
        Species::Vileplume => &VILEPLUME,
        Species::Raichu => &RAICHU,
        Species::Ekans => &EKANS,
        Species::Arbok => &ARBOK,
        Species::Steelix => &STEELIX,
        Species::Pineco => &PINECO,
        Species::Forretress => &FORRETRESS,
        Species::Kabuto => &KABUTO,
        Species::Kabutops => &KABUTOPS,
        Species::Electrode => &ELECTRODE,
        Species::Onix => &ONIX,
        Species::Diglett => &DIGLETT,
        Species::Dugtrio => &DUGTRIO,
        Species::Tentacruel => &TENTACRUEL,
        Species::Tentacool => &TENTACOOL,
        Species::Jynx => &JYNX,
        Species::Primeape => &PRIMEAPE,
        Species::Mankey => &MANKEY,
        Species::Meowth => &MEOWTH,
        Species::Persian => &PERSIAN,
        Species::Doduo => &DODUO,
        Species::Dodrio => &DODRIO,
        Species::Pidgeotto => &PIDGEOTTO,
        Species::FarfetchD => &FARFETCH_D,
        Species::Venusaur => &VENUSAUR,
        Species::Ivysaur => &IVYSAUR,
        Species::Exeggutor => &EXEGGUTOR,
        Species::Exeggcute => &EXEGGCUTE,
        Species::Fearow => &FEAROW,
        Species::Pidgeot => &PIDGEOT,
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
        Species::Bellossom => &BELLOSSOM,
        Species::Qwilfish => &QWILFISH,
        Species::Wobbuffet => &WOBBUFFET,
        Species::Dunsparce => &DUNSPARCE,
        Species::Shuckle => &SHUCKLE,
        Species::Gligar => &GLIGAR,
        Species::Yanma => &YANMA,
        Species::Murkrow => &MURKROW,
        Species::Misdreavus => &MISDREAVUS,
        Species::Teddiursa => &TEDDIURSA,
        Species::Ursaring => &URSARING,
        Species::Snubbull => &SNUBBULL,
        Species::Granbull => &GRANBULL,
        Species::Houndour => &HOUNDOUR,
        Species::Houndoom => &HOUNDOOM,
        Species::Miltank => &MILTANK,
        Species::Tauros => &TAUROS,
        Species::Delibird => &DELIBIRD,
        Species::Stantler => &STANTLER,
        Species::Girafarig => &GIRAFARIG,
        Species::Donphan => &DONPHAN,
        Species::Phanpy => &PHANPY,
        Species::Azumarill => &AZUMARILL,
        Species::Marill => &MARILL,
        Species::Xatu => &XATU,
        Species::Natu => &NATU,
        Species::Ampharos => &AMPHAROS,
        Species::Flaaffy => &FLAAFFY,
        Species::Togepi => &TOGEPI,
        Species::Togetic => &TOGETIC,
        Species::Remoraid => &REMORAID,
        Species::Octillery => &OCTILLERY,
        Species::Swinub => &SWINUB,
        Species::Piloswine => &PILOSWINE,
        Species::Sneasel => &SNEASEL,
        Species::Skarmory => &SKARMORY,
        Species::Smeargle => &SMEARGLE,
        Species::Moltres => &MOLTRES,
        Species::Zapdos => &ZAPDOS,
        Species::Articuno => &ARTICUNO,
        Species::Mew => &MEW,
        Species::Mewtwo => &MEWTWO,
        Species::Heracross => &HERACROSS,
        Species::Pinsir => &PINSIR,
        Species::Scizor => &SCIZOR,
        Species::Scyther => &SCYTHER,
        Species::Starmie => &STARMIE,
        Species::Gastly => &GASTLY,
        Species::Haunter => &HAUNTER,
        Species::Eevee => &EEVEE,
        Species::Vulpix => &VULPIX,
        Species::Ninetales => &NINETALES,
        Species::Venonat => &VENONAT,
        Species::Venomoth => &VENOMOTH,
        Species::Paras => &PARAS,
        Species::Parasect => &PARASECT,
        Species::Metapod => &METAPOD,
        Species::Kakuna => &KAKUNA,
        Species::Caterpie => &CATERPIE,
        Species::Weedle => &WEEDLE,
        Species::Rhydon => &RHYDON,
        Species::Drowzee => &DROWZEE,
        Species::Hypno => &HYPNO,
        Species::Abra => &ABRA,
        Species::Alakazam => &ALAKAZAM,
        Species::Kadabra => &KADABRA,
        Species::Koffing => &KOFFING,
        Species::Weezing => &WEEZING,
        Species::Wartortle => &WARTORTLE,
        Species::Blastoise => &BLASTOISE,
        Species::Croconaw => &CROCONAW,
        Species::Feraligatr => &FERALIGATR,
        Species::Cubone => &CUBONE,
        Species::Marowak => &MAROWAK,
        Species::Magnemite => &MAGNEMITE,
        Species::Magneton => &MAGNETON,
        Species::Tangela => &TANGELA,
        Species::Kangaskhan => &KANGASKHAN,
        Species::Ponyta => &PONYTA,
        Species::Rapidash => &RAPIDASH,
        Species::Machop => &MACHOP,
        Species::Machoke => &MACHOKE,
        Species::Machamp => &MACHAMP,
        Species::Rhyhorn => &RHYHORN,
        Species::Dewgong => &DEWGONG,
        Species::Seel => &SEEL,
        Species::Ditto => &DITTO,
        Species::Muk => &MUK,
        Species::Grimer => &GRIMER,
        Species::Cloyster => &CLOYSTER,
        Species::Shellder => &SHELLDER,
        Species::Kingler => &KINGLER,
        Species::Krabby => &KRABBY,
        Species::Umbreon => &UMBREON,
        Species::Jolteon => &JOLTEON,
        Species::Flareon => &FLAREON,
        Species::Espeon => &ESPEON,
        Species::Vaporeon => &VAPOREON,
        Species::Celebi => &CELEBI,
        Species::HoOh => &HO_OH,
        Species::Lugia => &LUGIA,
        Species::Corsola => &CORSOLA,
        Species::Mantine => &MANTINE,
        Species::Porygon => &PORYGON,
        Species::Porygon2 => &PORYGON2,
        Species::Chinchou => &CHINCHOU,
        Species::Lanturn => &LANTURN,
        Species::Slugma => &SLUGMA,
        Species::Magcargo => &MAGCARGO,
        Species::Hitmonlee => &HITMONLEE,
        Species::Hitmonchan => &HITMONCHAN,
        Species::Hitmontop => &HITMONTOP,
        Species::Tyrogue => &TYROGUE,
        Species::Smoochum => &SMOOCHUM,
        Species::Elekid => &ELEKID,
        Species::Magby => &MAGBY,
        Species::Magmar => &MAGMAR,
        Species::Electabuzz => &ELECTABUZZ,
        Species::Slowking => &SLOWKING,
        Species::Slowbro => &SLOWBRO,
        Species::Lickitung => &LICKITUNG,
        Species::MrMime => &MR_MIME,
        Species::Chansey => &CHANSEY,
        Species::Blissey => &BLISSEY,
        Species::Growlithe => &GROWLITHE,
        Species::Arcanine => &ARCANINE,
        Species::NidoranF => &NIDORAN_F,
        Species::NidoranM => &NIDORAN_M,
        Species::Nidorina => &NIDORINA,
        Species::Nidorino => &NIDORINO,
        Species::Nidoking => &NIDOKING,
        Species::Nidoqueen => &NIDOQUEEN,
        Species::Pichu => &PICHU,
        Species::Cleffa => &CLEFFA,
        Species::Igglybuff => &IGGLYBUFF,
        Species::Mareep => &MAREEP,
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
            Species::Procedural(_) => unreachable!(),
            Species::Psyduck => { crate::model_storage::include_model!("models/battle_species/psyduck.glb") }
            Species::Zubat => { crate::model_storage::include_model!("models/battle_species/zubat.glb") }
            Species::Golbat => { crate::model_storage::include_model!("models/battle_species/golbat.glb") }
            Species::Butterfree => { crate::model_storage::include_model!("models/battle_species/butterfree.glb") }
            Species::Diglett => { crate::model_storage::include_model!("models/battle_species/diglett.glb") }
            Species::Dugtrio => { crate::model_storage::include_model!("models/battle_species/dugtrio.glb") }
            Species::Tentacruel => { crate::model_storage::include_model!("models/battle_species/tentacruel.glb") }
            Species::Tentacool => { crate::model_storage::include_model!("models/battle_species/tentacool.glb") }
            Species::Jynx => { crate::model_storage::include_model!("models/battle_species/jynx.glb") }
            Species::Primeape => { crate::model_storage::include_model!("models/battle_species/primeape.glb") }
            Species::Mankey => { crate::model_storage::include_model!("models/battle_species/mankey.glb") }
            Species::Meowth => { crate::model_storage::include_model!("models/battle_species/meowth.glb") }
            Species::Persian => { crate::model_storage::include_model!("models/battle_species/persian.glb") }
            Species::Doduo => { crate::model_storage::include_model!("models/battle_species/doduo.glb") }
            Species::Dodrio => { crate::model_storage::include_model!("models/battle_species/dodrio.glb") }
            Species::Pidgeotto => { crate::model_storage::include_model!("models/battle_species/pidgeotto.glb") }
            Species::FarfetchD => { crate::model_storage::include_model!("models/battle_species/farfetch_d.glb") }
            Species::Venusaur => { crate::model_storage::include_model!("models/battle_species/venusaur.glb") }
            Species::Ivysaur => { crate::model_storage::include_model!("models/battle_species/ivysaur.glb") }
            Species::Exeggutor => { crate::model_storage::include_model!("models/battle_species/exeggutor.glb") }
            Species::Exeggcute => { crate::model_storage::include_model!("models/battle_species/exeggcute.glb") }
            Species::Fearow => { crate::model_storage::include_model!("models/battle_species/fearow.glb") }
            Species::Pidgeot => { crate::model_storage::include_model!("models/battle_species/pidgeot.glb") }
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
            Species::Bellossom => { crate::model_storage::include_model!("models/battle_species/bellossom.glb") }
            Species::Qwilfish => { crate::model_storage::include_model!("models/battle_species/qwilfish.glb") }
            Species::Wobbuffet => { crate::model_storage::include_model!("models/battle_species/wobbuffet.glb") }
            Species::Dunsparce => { crate::model_storage::include_model!("models/battle_species/dunsparce.glb") }
            Species::Shuckle => { crate::model_storage::include_model!("models/battle_species/shuckle.glb") }
            Species::Gligar => { crate::model_storage::include_model!("models/battle_species/gligar.glb") }
            Species::Yanma => { crate::model_storage::include_model!("models/battle_species/yanma.glb") }
            Species::Murkrow => { crate::model_storage::include_model!("models/battle_species/murkrow.glb") }
            Species::Misdreavus => { crate::model_storage::include_model!("models/battle_species/misdreavus.glb") }
            Species::Teddiursa => { crate::model_storage::include_model!("models/battle_species/teddiursa.glb") }
            Species::Ursaring => { crate::model_storage::include_model!("models/battle_species/ursaring.glb") }
            Species::Snubbull => { crate::model_storage::include_model!("models/battle_species/snubbull.glb") }
            Species::Granbull => { crate::model_storage::include_model!("models/battle_species/granbull.glb") }
            Species::Houndour => { crate::model_storage::include_model!("models/battle_species/houndour.glb") }
            Species::Houndoom => { crate::model_storage::include_model!("models/battle_species/houndoom.glb") }
            Species::Miltank => { crate::model_storage::include_model!("models/battle_species/miltank.glb") }
            Species::Tauros => { crate::model_storage::include_model!("models/battle_species/tauros.glb") }
            Species::Delibird => { crate::model_storage::include_model!("models/battle_species/delibird.glb") }
            Species::Stantler => { crate::model_storage::include_model!("models/battle_species/stantler.glb") }
            Species::Girafarig => { crate::model_storage::include_model!("models/battle_species/girafarig.glb") }
            Species::Donphan => { crate::model_storage::include_model!("models/battle_species/donphan.glb") }
            Species::Phanpy => { crate::model_storage::include_model!("models/battle_species/phanpy.glb") }
            Species::Azumarill => { crate::model_storage::include_model!("models/battle_species/azumarill.glb") }
            Species::Marill => { crate::model_storage::include_model!("models/battle_species/marill.glb") }
            Species::Xatu => { crate::model_storage::include_model!("models/battle_species/xatu.glb") }
            Species::Natu => { crate::model_storage::include_model!("models/battle_species/natu.glb") }
            Species::Ampharos => { crate::model_storage::include_model!("models/battle_species/ampharos.glb") }
            Species::Flaaffy => { crate::model_storage::include_model!("models/battle_species/flaaffy.glb") }
            Species::Togepi => { crate::model_storage::include_model!("models/battle_species/togepi.glb") }
            Species::Togetic => { crate::model_storage::include_model!("models/battle_species/togetic.glb") }
            Species::Remoraid => { crate::model_storage::include_model!("models/battle_species/remoraid.glb") }
            Species::Octillery => { crate::model_storage::include_model!("models/battle_species/octillery.glb") }
            Species::Swinub => { crate::model_storage::include_model!("models/battle_species/swinub.glb") }
            Species::Piloswine => { crate::model_storage::include_model!("models/battle_species/piloswine.glb") }
            Species::Sneasel => { crate::model_storage::include_model!("models/battle_species/sneasel.glb") }
            Species::Skarmory => { crate::model_storage::include_model!("models/battle_species/skarmory.glb") }
            Species::Smeargle => { crate::model_storage::include_model!("models/battle_species/smeargle.glb") }
            Species::Moltres => { crate::model_storage::include_model!("models/battle_species/moltres.glb") }
            Species::Zapdos => { crate::model_storage::include_model!("models/battle_species/zapdos.glb") }
            Species::Articuno => { crate::model_storage::include_model!("models/battle_species/articuno.glb") }
            Species::Mew => { crate::model_storage::include_model!("models/battle_species/mew.glb") }
            Species::Mewtwo => { crate::model_storage::include_model!("models/battle_species/mewtwo.glb") }
            Species::Heracross => { crate::model_storage::include_model!("models/battle_species/heracross.glb") }
            Species::Pinsir => { crate::model_storage::include_model!("models/battle_species/pinsir.glb") }
            Species::Scizor => { crate::model_storage::include_model!("models/battle_species/scizor.glb") }
            Species::Scyther => { crate::model_storage::include_model!("models/battle_species/scyther.glb") }
            Species::Starmie => { crate::model_storage::include_model!("models/battle_species/starmie.glb") }
            Species::Gastly => { crate::model_storage::include_model!("models/battle_species/gastly.glb") }
            Species::Haunter => { crate::model_storage::include_model!("models/battle_species/haunter.glb") }
            Species::Eevee => { crate::model_storage::include_model!("models/battle_species/eevee.glb") }
            Species::Vulpix => { crate::model_storage::include_model!("models/battle_species/vulpix.glb") }
            Species::Ninetales => { crate::model_storage::include_model!("models/battle_species/ninetales.glb") }
            Species::Venonat => { crate::model_storage::include_model!("models/battle_species/venonat.glb") }
            Species::Venomoth => { crate::model_storage::include_model!("models/battle_species/venomoth.glb") }
            Species::Paras => { crate::model_storage::include_model!("models/battle_species/paras.glb") }
            Species::Parasect => { crate::model_storage::include_model!("models/battle_species/parasect.glb") }
            Species::Metapod => { crate::model_storage::include_model!("models/battle_species/metapod.glb") }
            Species::Kakuna => { crate::model_storage::include_model!("models/battle_species/kakuna.glb") }
            Species::Caterpie => { crate::model_storage::include_model!("models/battle_species/caterpie.glb") }
            Species::Weedle => { crate::model_storage::include_model!("models/battle_species/weedle.glb") }
            Species::Rhydon => { crate::model_storage::include_model!("models/battle_species/rhydon.glb") }
            Species::Drowzee => { crate::model_storage::include_model!("models/battle_species/drowzee.glb") }
            Species::Hypno => { crate::model_storage::include_model!("models/battle_species/hypno.glb") }
            Species::Abra => { crate::model_storage::include_model!("models/battle_species/abra.glb") }
            Species::Alakazam => { crate::model_storage::include_model!("models/battle_species/alakazam.glb") }
            Species::Kadabra => { crate::model_storage::include_model!("models/battle_species/kadabra.glb") }
            Species::Koffing => { crate::model_storage::include_model!("models/battle_species/koffing.glb") }
            Species::Weezing => { crate::model_storage::include_model!("models/battle_species/weezing.glb") }
            Species::Wartortle => { crate::model_storage::include_model!("models/battle_species/wartortle.glb") }
            Species::Blastoise => { crate::model_storage::include_model!("models/battle_species/blastoise.glb") }
            Species::Croconaw => { crate::model_storage::include_model!("models/battle_species/croconaw.glb") }
            Species::Feraligatr => { crate::model_storage::include_model!("models/battle_species/feraligatr.glb") }
            Species::Cubone => { crate::model_storage::include_model!("models/battle_species/cubone.glb") }
            Species::Marowak => { crate::model_storage::include_model!("models/battle_species/marowak.glb") }
            Species::Magnemite => { crate::model_storage::include_model!("models/battle_species/magnemite.glb") }
            Species::Magneton => { crate::model_storage::include_model!("models/battle_species/magneton.glb") }
            Species::Tangela => { crate::model_storage::include_model!("models/battle_species/tangela.glb") }
            Species::Kangaskhan => { crate::model_storage::include_model!("models/battle_species/kangaskhan.glb") }
            Species::Ponyta => { crate::model_storage::include_model!("models/battle_species/ponyta.glb") }
            Species::Rapidash => { crate::model_storage::include_model!("models/battle_species/rapidash.glb") }
            Species::Machop => { crate::model_storage::include_model!("models/battle_species/machop.glb") }
            Species::Machoke => { crate::model_storage::include_model!("models/battle_species/machoke.glb") }
            Species::Machamp => { crate::model_storage::include_model!("models/battle_species/machamp.glb") }
            Species::Rhyhorn => { crate::model_storage::include_model!("models/battle_species/rhyhorn.glb") }
            Species::Dewgong => { crate::model_storage::include_model!("models/battle_species/dewgong.glb") }
            Species::Seel => { crate::model_storage::include_model!("models/battle_species/seel.glb") }
            Species::Ditto => { crate::model_storage::include_model!("models/battle_species/ditto.glb") }
            Species::Muk => { crate::model_storage::include_model!("models/battle_species/muk.glb") }
            Species::Grimer => { crate::model_storage::include_model!("models/battle_species/grimer.glb") }
            Species::Cloyster => { crate::model_storage::include_model!("models/battle_species/cloyster.glb") }
            Species::Shellder => { crate::model_storage::include_model!("models/battle_species/shellder.glb") }
            Species::Kingler => { crate::model_storage::include_model!("models/battle_species/kingler.glb") }
            Species::Krabby => { crate::model_storage::include_model!("models/battle_species/krabby.glb") }
            Species::Umbreon => { crate::model_storage::include_model!("models/battle_species/umbreon.glb") }
            Species::Jolteon => { crate::model_storage::include_model!("models/battle_species/jolteon.glb") }
            Species::Flareon => { crate::model_storage::include_model!("models/battle_species/flareon.glb") }
            Species::Espeon => { crate::model_storage::include_model!("models/battle_species/espeon.glb") }
            Species::Vaporeon => { crate::model_storage::include_model!("models/battle_species/vaporeon.glb") }
            Species::Celebi => { crate::model_storage::include_model!("models/battle_species/celebi.glb") }
            Species::HoOh => { crate::model_storage::include_model!("models/battle_species/ho_oh.glb") }
            Species::Lugia => { crate::model_storage::include_model!("models/battle_species/lugia.glb") }
            Species::Corsola => { crate::model_storage::include_model!("models/battle_species/corsola.glb") }
            Species::Mantine => { crate::model_storage::include_model!("models/battle_species/mantine.glb") }
            Species::Porygon => { crate::model_storage::include_model!("models/battle_species/porygon.glb") }
            Species::Porygon2 => { crate::model_storage::include_model!("models/battle_species/porygon2.glb") }
            Species::Chinchou => { crate::model_storage::include_model!("models/battle_species/chinchou.glb") }
            Species::Lanturn => { crate::model_storage::include_model!("models/battle_species/lanturn.glb") }
            Species::Slugma => { crate::model_storage::include_model!("models/battle_species/slugma.glb") }
            Species::Magcargo => { crate::model_storage::include_model!("models/battle_species/magcargo.glb") }
            Species::Hitmonlee => { crate::model_storage::include_model!("models/battle_species/hitmonlee.glb") }
            Species::Hitmonchan => { crate::model_storage::include_model!("models/battle_species/hitmonchan.glb") }
            Species::Hitmontop => { crate::model_storage::include_model!("models/battle_species/hitmontop.glb") }
            Species::Tyrogue => { crate::model_storage::include_model!("models/battle_species/tyrogue.glb") }
            Species::Smoochum => { crate::model_storage::include_model!("models/battle_species/smoochum.glb") }
            Species::Elekid => { crate::model_storage::include_model!("models/battle_species/elekid.glb") }
            Species::Magby => { crate::model_storage::include_model!("models/battle_species/magby.glb") }
            Species::Magmar => { crate::model_storage::include_model!("models/battle_species/magmar.glb") }
            Species::Electabuzz => { crate::model_storage::include_model!("models/battle_species/electabuzz.glb") }
            Species::Slowking => { crate::model_storage::include_model!("models/battle_species/slowking.glb") }
            Species::Slowbro => { crate::model_storage::include_model!("models/battle_species/slowbro.glb") }
            Species::Lickitung => { crate::model_storage::include_model!("models/battle_species/lickitung.glb") }
            Species::MrMime => { crate::model_storage::include_model!("models/battle_species/mr__mime.glb") }
            Species::Chansey => { crate::model_storage::include_model!("models/battle_species/chansey.glb") }
            Species::Blissey => { crate::model_storage::include_model!("models/battle_species/blissey.glb") }
            Species::Growlithe => { crate::model_storage::include_model!("models/battle_species/growlithe.glb") }
            Species::Arcanine => { crate::model_storage::include_model!("models/battle_species/arcanine.glb") }
            Species::NidoranF => { crate::model_storage::include_model!("models/battle_species/nidoran_f.glb") }
            Species::NidoranM => { crate::model_storage::include_model!("models/battle_species/nidoran_m.glb") }
            Species::Nidorina => { crate::model_storage::include_model!("models/battle_species/nidorina.glb") }
            Species::Nidorino => { crate::model_storage::include_model!("models/battle_species/nidorino.glb") }
            Species::Nidoking => { crate::model_storage::include_model!("models/battle_species/nidoking.glb") }
            Species::Nidoqueen => { crate::model_storage::include_model!("models/battle_species/nidoqueen.glb") }
            Species::Pichu => { crate::model_storage::include_model!("models/battle_species/pichu.glb") }
            Species::Cleffa => { crate::model_storage::include_model!("models/battle_species/cleffa.glb") }
            Species::Igglybuff => { crate::model_storage::include_model!("models/battle_species/igglybuff.glb") }
            Species::Mareep => { crate::model_storage::include_model!("models/battle_species/mareep.glb") }
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
            Species::Bellsprout => { crate::model_storage::include_model!("models/battle_species/bellsprout.glb") }
            Species::Weepinbell => { crate::model_storage::include_model!("models/battle_species/weepinbell.glb") }
            Species::Victreebel => { crate::model_storage::include_model!("models/battle_species/victreebel.glb") }
            Species::Gloom => { crate::model_storage::include_model!("models/battle_species/gloom.glb") }
            Species::Vileplume => { crate::model_storage::include_model!("models/battle_species/vileplume.glb") }
            Species::Raichu => { crate::model_storage::include_model!("models/battle_species/raichu.glb") }
            Species::Ekans => { crate::model_storage::include_model!("models/battle_species/ekans.glb") }
            Species::Arbok => { crate::model_storage::include_model!("models/battle_species/arbok.glb") }
            Species::Steelix => { crate::model_storage::include_model!("models/battle_species/steelix.glb") }
            Species::Pineco => { crate::model_storage::include_model!("models/battle_species/pineco.glb") }
            Species::Forretress => { crate::model_storage::include_model!("models/battle_species/forretress.glb") }
            Species::Kabuto => { crate::model_storage::include_model!("models/battle_species/kabuto.glb") }
            Species::Kabutops => { crate::model_storage::include_model!("models/battle_species/kabutops.glb") }
            Species::Electrode => { crate::model_storage::include_model!("models/battle_species/electrode.glb") }
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
            Species::Horsea => { crate::model_storage::include_model!("models/battle_species/horsea.glb") }
            Species::Seadra => { crate::model_storage::include_model!("models/battle_species/seadra.glb") }
            Species::Kingdra => { crate::model_storage::include_model!("models/battle_species/kingdra.glb") }
            Species::Goldeen => { crate::model_storage::include_model!("models/battle_species/goldeen.glb") }
            Species::Seaking => { crate::model_storage::include_model!("models/battle_species/seaking.glb") }
            Species::Magikarp => { crate::model_storage::include_model!("models/battle_species/magikarp.glb") }
            Species::Larvitar => { crate::model_storage::include_model!("models/battle_species/larvitar.glb") }
            Species::Pupitar => { crate::model_storage::include_model!("models/battle_species/pupitar.glb") }
            Species::Tyranitar => { crate::model_storage::include_model!("models/battle_species/tyranitar.glb") }
            Species::Quilava => { crate::model_storage::include_model!("models/battle_species/quilava.glb") }
            Species::Typhlosion => { crate::model_storage::include_model!("models/battle_species/typhlosion.glb") }
            Species::Charmeleon => { crate::model_storage::include_model!("models/battle_species/charmeleon.glb") }
            Species::Charizard => { crate::model_storage::include_model!("models/battle_species/charizard.glb") }
            Species::Graveler => { crate::model_storage::include_model!("models/battle_species/graveler.glb") }
            Species::Golem => { crate::model_storage::include_model!("models/battle_species/golem.glb") }
            Species::Omanyte => { crate::model_storage::include_model!("models/battle_species/omanyte.glb") }
            Species::Omastar => { crate::model_storage::include_model!("models/battle_species/omastar.glb") }
            Species::Aerodactyl => { crate::model_storage::include_model!("models/battle_species/aerodactyl.glb") }
            Species::Dratini => { crate::model_storage::include_model!("models/battle_species/dratini.glb") }
            Species::Dragonair => { crate::model_storage::include_model!("models/battle_species/dragonair.glb") }
            Species::Dragonite => { crate::model_storage::include_model!("models/battle_species/dragonite.glb") }
            Species::Clefable => { crate::model_storage::include_model!("models/battle_species/clefable.glb") }
            Species::Wigglytuff => { crate::model_storage::include_model!("models/battle_species/wigglytuff.glb") }
            Species::Sandshrew => { crate::model_storage::include_model!("models/battle_species/sandshrew.glb") }
            Species::Sandslash => { crate::model_storage::include_model!("models/battle_species/sandslash.glb") }
            Species::Politoed => { crate::model_storage::include_model!("models/battle_species/politoed.glb") }
            Species::Poliwrath => { crate::model_storage::include_model!("models/battle_species/poliwrath.glb") }
            Species::Poliwhirl => { crate::model_storage::include_model!("models/battle_species/poliwhirl.glb") }
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
    authored_for_species(species).or_else(|| crate::battle_species_models::rig(species))
}

pub(crate) fn authored_for_species(species: &str) -> Option<&'static SpeciesRig> {
    if species.len() == 7 && species.as_bytes()[..6].eq_ignore_ascii_case(b"UNOWN_") {
        let letter = species.as_bytes()[6].to_ascii_lowercase();
        return letter.is_ascii_lowercase().then(|| rig(Species::Unown(letter - b'a')));
    }
    if species.eq_ignore_ascii_case("JUMPLUFF") {
        Some(rig(Species::Jumpluff))
    } else if species.eq_ignore_ascii_case("TEDDIURSA") {
        Some(rig(Species::Teddiursa))
    } else if species.eq_ignore_ascii_case("SNUBBULL") {
        Some(rig(Species::Snubbull))
    } else if species.eq_ignore_ascii_case("HOUNDOUR") {
        Some(rig(Species::Houndour))
    } else if species.eq_ignore_ascii_case("MILTANK") {
        Some(rig(Species::Miltank))
    } else if species.eq_ignore_ascii_case("TOGEPI") {
        Some(rig(Species::Togepi))
    } else if species.eq_ignore_ascii_case("TOGETIC") {
        Some(rig(Species::Togetic))
    } else if species.eq_ignore_ascii_case("REMORAID") {
        Some(rig(Species::Remoraid))
    } else if species.eq_ignore_ascii_case("OCTILLERY") {
        Some(rig(Species::Octillery))
    } else if species.eq_ignore_ascii_case("SWINUB") {
        Some(rig(Species::Swinub))
    } else if species.eq_ignore_ascii_case("PILOSWINE") {
        Some(rig(Species::Piloswine))
    } else if species.eq_ignore_ascii_case("SNEASEL") {
        Some(rig(Species::Sneasel))
    } else if species.eq_ignore_ascii_case("SKARMORY") {
        Some(rig(Species::Skarmory))
    } else if species.eq_ignore_ascii_case("SMEARGLE") {
        Some(rig(Species::Smeargle))
    } else if species.eq_ignore_ascii_case("SCYTHER") {
        Some(rig(Species::Scyther))
    } else if species.eq_ignore_ascii_case("STARMIE") {
        Some(rig(Species::Starmie))
    } else if species.eq_ignore_ascii_case("GASTLY") {
        Some(rig(Species::Gastly))
    } else if species.eq_ignore_ascii_case("HAUNTER") {
        Some(rig(Species::Haunter))
    } else if species.eq_ignore_ascii_case("EEVEE") {
        Some(rig(Species::Eevee))
    } else if species.eq_ignore_ascii_case("JOLTEON") {
        Some(rig(Species::Jolteon))
    } else if species.eq_ignore_ascii_case("FLAREON") {
        Some(rig(Species::Flareon))
    } else if species.eq_ignore_ascii_case("ESPEON") {
        Some(rig(Species::Espeon))
    } else if species.eq_ignore_ascii_case("UMBREON") {
        Some(rig(Species::Umbreon))
    } else if species.eq_ignore_ascii_case("VULPIX") {
        Some(rig(Species::Vulpix))
    } else if species.eq_ignore_ascii_case("PARAS") {
        Some(rig(Species::Paras))
    } else if species.eq_ignore_ascii_case("METAPOD") {
        Some(rig(Species::Metapod))
    } else if species.eq_ignore_ascii_case("CATERPIE") {
        Some(rig(Species::Caterpie))
    } else if species.eq_ignore_ascii_case("KRABBY") {
        Some(rig(Species::Krabby))
    } else if species.eq_ignore_ascii_case("KINGLER") {
        Some(rig(Species::Kingler))
    } else if species.eq_ignore_ascii_case("SHELLDER") {
        Some(rig(Species::Shellder))
    } else if species.eq_ignore_ascii_case("CLOYSTER") {
        Some(rig(Species::Cloyster))
    } else if species.eq_ignore_ascii_case("GRIMER") {
        Some(rig(Species::Grimer))
    } else if species.eq_ignore_ascii_case("MUK") {
        Some(rig(Species::Muk))
    } else if species.eq_ignore_ascii_case("DITTO") {
        Some(rig(Species::Ditto))
    } else if species.eq_ignore_ascii_case("SEEL") {
        Some(rig(Species::Seel))
    } else if species.eq_ignore_ascii_case("DEWGONG") {
        Some(rig(Species::Dewgong))
    } else if species.eq_ignore_ascii_case("DROWZEE") {
        Some(rig(Species::Drowzee))
    } else if species.eq_ignore_ascii_case("HYPNO") {
        Some(rig(Species::Hypno))
    } else if species.eq_ignore_ascii_case("ABRA") {
        Some(rig(Species::Abra))
    } else if species.eq_ignore_ascii_case("ALAKAZAM") {
        Some(rig(Species::Alakazam))
    } else if species.eq_ignore_ascii_case("KADABRA") {
        Some(rig(Species::Kadabra))
    } else if species.eq_ignore_ascii_case("KOFFING") {
        Some(rig(Species::Koffing))
    } else if species.eq_ignore_ascii_case("WEEZING") {
        Some(rig(Species::Weezing))
    } else if species.eq_ignore_ascii_case("WARTORTLE") {
        Some(rig(Species::Wartortle))
    } else if species.eq_ignore_ascii_case("BLASTOISE") {
        Some(rig(Species::Blastoise))
    } else if species.eq_ignore_ascii_case("CROCONAW") {
        Some(rig(Species::Croconaw))
    } else if species.eq_ignore_ascii_case("FERALIGATR") {
        Some(rig(Species::Feraligatr))
    } else if species.eq_ignore_ascii_case("CUBONE") {
        Some(rig(Species::Cubone))
    } else if species.eq_ignore_ascii_case("MAROWAK") {
        Some(rig(Species::Marowak))
    } else if species.eq_ignore_ascii_case("MAGNEMITE") {
        Some(rig(Species::Magnemite))
    } else if species.eq_ignore_ascii_case("MAGNETON") {
        Some(rig(Species::Magneton))
    } else if species.eq_ignore_ascii_case("TANGELA") {
        Some(rig(Species::Tangela))
    } else if species.eq_ignore_ascii_case("KANGASKHAN") {
        Some(rig(Species::Kangaskhan))
    } else if species.eq_ignore_ascii_case("PONYTA") {
        Some(rig(Species::Ponyta))
    } else if species.eq_ignore_ascii_case("RAPIDASH") {
        Some(rig(Species::Rapidash))
    } else if species.eq_ignore_ascii_case("MACHOP") {
        Some(rig(Species::Machop))
    } else if species.eq_ignore_ascii_case("MACHOKE") {
        Some(rig(Species::Machoke))
    } else if species.eq_ignore_ascii_case("MACHAMP") {
        Some(rig(Species::Machamp))
    } else if species.eq_ignore_ascii_case("RHYHORN") {
        Some(rig(Species::Rhyhorn))
    } else if species.eq_ignore_ascii_case("RHYDON") {
        Some(rig(Species::Rhydon))
    } else if species.eq_ignore_ascii_case("WEEDLE") {
        Some(rig(Species::Weedle))
    } else if species.eq_ignore_ascii_case("KAKUNA") {
        Some(rig(Species::Kakuna))
    } else if species.eq_ignore_ascii_case("PARASECT") {
        Some(rig(Species::Parasect))
    } else if species.eq_ignore_ascii_case("VENOMOTH") {
        Some(rig(Species::Venomoth))
    } else if species.eq_ignore_ascii_case("VENONAT") {
        Some(rig(Species::Venonat))
    } else if species.eq_ignore_ascii_case("NINETALES") {
        Some(rig(Species::Ninetales))
    } else if species.eq_ignore_ascii_case("VAPOREON") {
        Some(rig(Species::Vaporeon))
    } else if species.eq_ignore_ascii_case("SCIZOR") {
        Some(rig(Species::Scizor))
    } else if species.eq_ignore_ascii_case("PINSIR") {
        Some(rig(Species::Pinsir))
    } else if species.eq_ignore_ascii_case("HERACROSS") {
        Some(rig(Species::Heracross))
    } else if species.eq_ignore_ascii_case("MEWTWO") {
        Some(rig(Species::Mewtwo))
    } else if species.eq_ignore_ascii_case("MEW") {
        Some(rig(Species::Mew))
    } else if species.eq_ignore_ascii_case("CELEBI") {
        Some(rig(Species::Celebi))
    } else if species.eq_ignore_ascii_case("HO_OH") || species.eq_ignore_ascii_case("HO-OH") {
        Some(rig(Species::HoOh))
    } else if species.eq_ignore_ascii_case("LUGIA") {
        Some(rig(Species::Lugia))
    } else if species.eq_ignore_ascii_case("ARTICUNO") {
        Some(rig(Species::Articuno))
    } else if species.eq_ignore_ascii_case("ZAPDOS") {
        Some(rig(Species::Zapdos))
    } else if species.eq_ignore_ascii_case("MOLTRES") {
        Some(rig(Species::Moltres))
    } else if species.eq_ignore_ascii_case("CORSOLA") {
        Some(rig(Species::Corsola))
    } else if species.eq_ignore_ascii_case("MANTINE") {
        Some(rig(Species::Mantine))
    } else if species.eq_ignore_ascii_case("PORYGON") {
        Some(rig(Species::Porygon))
    } else if species.eq_ignore_ascii_case("PORYGON2") {
        Some(rig(Species::Porygon2))
    } else if species.eq_ignore_ascii_case("CHINCHOU") {
        Some(rig(Species::Chinchou))
    } else if species.eq_ignore_ascii_case("LANTURN") {
        Some(rig(Species::Lanturn))
    } else if species.eq_ignore_ascii_case("SLUGMA") {
        Some(rig(Species::Slugma))
    } else if species.eq_ignore_ascii_case("MAGCARGO") {
        Some(rig(Species::Magcargo))
    } else if species.eq_ignore_ascii_case("HITMONLEE") {
        Some(rig(Species::Hitmonlee))
    } else if species.eq_ignore_ascii_case("HITMONCHAN") {
        Some(rig(Species::Hitmonchan))
    } else if species.eq_ignore_ascii_case("HITMONTOP") {
        Some(rig(Species::Hitmontop))
    } else if species.eq_ignore_ascii_case("TYROGUE") {
        Some(rig(Species::Tyrogue))
    } else if species.eq_ignore_ascii_case("SMOOCHUM") {
        Some(rig(Species::Smoochum))
    } else if species.eq_ignore_ascii_case("ELEKID") {
        Some(rig(Species::Elekid))
    } else if species.eq_ignore_ascii_case("MAGBY") {
        Some(rig(Species::Magby))
    } else if species.eq_ignore_ascii_case("MAGMAR") {
        Some(rig(Species::Magmar))
    } else if species.eq_ignore_ascii_case("ELECTABUZZ") {
        Some(rig(Species::Electabuzz))
    } else if species.eq_ignore_ascii_case("SLOWKING") {
        Some(rig(Species::Slowking))
    } else if species.eq_ignore_ascii_case("SLOWBRO") {
        Some(rig(Species::Slowbro))
    } else if species.eq_ignore_ascii_case("LICKITUNG") {
        Some(rig(Species::Lickitung))
    } else if species.eq_ignore_ascii_case("MR__MIME") {
        Some(rig(Species::MrMime))
    } else if species.eq_ignore_ascii_case("CHANSEY") {
        Some(rig(Species::Chansey))
    } else if species.eq_ignore_ascii_case("BLISSEY") {
        Some(rig(Species::Blissey))
    } else if species.eq_ignore_ascii_case("GROWLITHE") {
        Some(rig(Species::Growlithe))
    } else if species.eq_ignore_ascii_case("ARCANINE") {
        Some(rig(Species::Arcanine))
    } else if species.eq_ignore_ascii_case("NIDORAN_F") {
        Some(rig(Species::NidoranF))
    } else if species.eq_ignore_ascii_case("NIDORAN_M") {
        Some(rig(Species::NidoranM))
    } else if species.eq_ignore_ascii_case("NIDORINA") {
        Some(rig(Species::Nidorina))
    } else if species.eq_ignore_ascii_case("NIDORINO") {
        Some(rig(Species::Nidorino))
    } else if species.eq_ignore_ascii_case("NIDOKING") {
        Some(rig(Species::Nidoking))
    } else if species.eq_ignore_ascii_case("NIDOQUEEN") {
        Some(rig(Species::Nidoqueen))
    } else if species.eq_ignore_ascii_case("PICHU") {
        Some(rig(Species::Pichu))
    } else if species.eq_ignore_ascii_case("CLEFFA") {
        Some(rig(Species::Cleffa))
    } else if species.eq_ignore_ascii_case("IGGLYBUFF") {
        Some(rig(Species::Igglybuff))
    } else if species.eq_ignore_ascii_case("MAREEP") {
        Some(rig(Species::Mareep))
    } else if species.eq_ignore_ascii_case("FLAAFFY") {
        Some(rig(Species::Flaaffy))
    } else if species.eq_ignore_ascii_case("AMPHAROS") {
        Some(rig(Species::Ampharos))
    } else if species.eq_ignore_ascii_case("NATU") {
        Some(rig(Species::Natu))
    } else if species.eq_ignore_ascii_case("XATU") {
        Some(rig(Species::Xatu))
    } else if species.eq_ignore_ascii_case("MARILL") {
        Some(rig(Species::Marill))
    } else if species.eq_ignore_ascii_case("AZUMARILL") {
        Some(rig(Species::Azumarill))
    } else if species.eq_ignore_ascii_case("PHANPY") {
        Some(rig(Species::Phanpy))
    } else if species.eq_ignore_ascii_case("DONPHAN") {
        Some(rig(Species::Donphan))
    } else if species.eq_ignore_ascii_case("GIRAFARIG") {
        Some(rig(Species::Girafarig))
    } else if species.eq_ignore_ascii_case("STANTLER") {
        Some(rig(Species::Stantler))
    } else if species.eq_ignore_ascii_case("DELIBIRD") {
        Some(rig(Species::Delibird))
    } else if species.eq_ignore_ascii_case("TAUROS") {
        Some(rig(Species::Tauros))
    } else if species.eq_ignore_ascii_case("HOUNDOOM") {
        Some(rig(Species::Houndoom))
    } else if species.eq_ignore_ascii_case("GRANBULL") {
        Some(rig(Species::Granbull))
    } else if species.eq_ignore_ascii_case("URSARING") {
        Some(rig(Species::Ursaring))
    } else if species.eq_ignore_ascii_case("MISDREAVUS") {
        Some(rig(Species::Misdreavus))
    } else if species.eq_ignore_ascii_case("MURKROW") {
        Some(rig(Species::Murkrow))
    } else if species.eq_ignore_ascii_case("YANMA") {
        Some(rig(Species::Yanma))
    } else if species.eq_ignore_ascii_case("GLIGAR") {
        Some(rig(Species::Gligar))
    } else if species.eq_ignore_ascii_case("SHUCKLE") {
        Some(rig(Species::Shuckle))
    } else if species.eq_ignore_ascii_case("DUNSPARCE") {
        Some(rig(Species::Dunsparce))
    } else if species.eq_ignore_ascii_case("WOBBUFFET") {
        Some(rig(Species::Wobbuffet))
    } else if species.eq_ignore_ascii_case("QWILFISH") {
        Some(rig(Species::Qwilfish))
    } else if species.eq_ignore_ascii_case("BELLOSSOM") {
        Some(rig(Species::Bellossom))
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
    } else if species.eq_ignore_ascii_case("DUGTRIO") {
        Some(rig(Species::Dugtrio))
    } else if species.eq_ignore_ascii_case("TENTACRUEL") {
        Some(rig(Species::Tentacruel))
    } else if species.eq_ignore_ascii_case("TENTACOOL") {
        Some(rig(Species::Tentacool))
    } else if species.eq_ignore_ascii_case("JYNX") {
        Some(rig(Species::Jynx))
    } else if species.eq_ignore_ascii_case("PRIMEAPE") {
        Some(rig(Species::Primeape))
    } else if species.eq_ignore_ascii_case("MEOWTH") {
        Some(rig(Species::Meowth))
    } else if species.eq_ignore_ascii_case("DODUO") {
        Some(rig(Species::Doduo))
    } else if species.eq_ignore_ascii_case("PIDGEOT") {
        Some(rig(Species::Pidgeot))
    } else if species.eq_ignore_ascii_case("FEAROW") {
        Some(rig(Species::Fearow))
    } else if species.eq_ignore_ascii_case("EXEGGCUTE") {
        Some(rig(Species::Exeggcute))
    } else if species.eq_ignore_ascii_case("EXEGGUTOR") {
        Some(rig(Species::Exeggutor))
    } else if species.eq_ignore_ascii_case("IVYSAUR") {
        Some(rig(Species::Ivysaur))
    } else if species.eq_ignore_ascii_case("VENUSAUR") {
        Some(rig(Species::Venusaur))
    } else if species.eq_ignore_ascii_case("FARFETCH_D") {
        Some(rig(Species::FarfetchD))
    } else if species.eq_ignore_ascii_case("PIDGEOTTO") {
        Some(rig(Species::Pidgeotto))
    } else if species.eq_ignore_ascii_case("DODRIO") {
        Some(rig(Species::Dodrio))
    } else if species.eq_ignore_ascii_case("PERSIAN") {
        Some(rig(Species::Persian))
    } else if species.eq_ignore_ascii_case("MANKEY") {
        Some(rig(Species::Mankey))
    } else if species.eq_ignore_ascii_case("ONIX") {
        Some(rig(Species::Onix))
    } else if species.eq_ignore_ascii_case("ELECTRODE") {
        Some(rig(Species::Electrode))
    } else if species.eq_ignore_ascii_case("KABUTO") {
        Some(rig(Species::Kabuto))
    } else if species.eq_ignore_ascii_case("KABUTOPS") {
        Some(rig(Species::Kabutops))
    } else if species.eq_ignore_ascii_case("PINECO") {
        Some(rig(Species::Pineco))
    } else if species.eq_ignore_ascii_case("FORRETRESS") {
        Some(rig(Species::Forretress))
    } else if species.eq_ignore_ascii_case("BELLSPROUT") {
        Some(rig(Species::Bellsprout))
    } else if species.eq_ignore_ascii_case("WEEPINBELL") {
        Some(rig(Species::Weepinbell))
    } else if species.eq_ignore_ascii_case("VICTREEBEL") {
        Some(rig(Species::Victreebel))
    } else if species.eq_ignore_ascii_case("GLOOM") {
        Some(rig(Species::Gloom))
    } else if species.eq_ignore_ascii_case("VILEPLUME") {
        Some(rig(Species::Vileplume))
    } else if species.eq_ignore_ascii_case("STEELIX") {
        Some(rig(Species::Steelix))
    } else if species.eq_ignore_ascii_case("ARBOK") {
        Some(rig(Species::Arbok))
    } else if species.eq_ignore_ascii_case("EKANS") {
        Some(rig(Species::Ekans))
    } else if species.eq_ignore_ascii_case("RAICHU") {
        Some(rig(Species::Raichu))
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
    } else if species.eq_ignore_ascii_case("HORSEA") {
        Some(rig(Species::Horsea))
    } else if species.eq_ignore_ascii_case("SEADRA") {
        Some(rig(Species::Seadra))
    } else if species.eq_ignore_ascii_case("KINGDRA") {
        Some(rig(Species::Kingdra))
    } else if species.eq_ignore_ascii_case("GOLDEEN") {
        Some(rig(Species::Goldeen))
    } else if species.eq_ignore_ascii_case("SEAKING") {
        Some(rig(Species::Seaking))
    } else if species.eq_ignore_ascii_case("MAGIKARP") {
        Some(rig(Species::Magikarp))
    } else if species.eq_ignore_ascii_case("LARVITAR") {
        Some(rig(Species::Larvitar))
    } else if species.eq_ignore_ascii_case("PUPITAR") {
        Some(rig(Species::Pupitar))
    } else if species.eq_ignore_ascii_case("TYRANITAR") {
        Some(rig(Species::Tyranitar))
    } else if species.eq_ignore_ascii_case("QUILAVA") {
        Some(rig(Species::Quilava))
    } else if species.eq_ignore_ascii_case("TYPHLOSION") {
        Some(rig(Species::Typhlosion))
    } else if species.eq_ignore_ascii_case("CHARMELEON") {
        Some(rig(Species::Charmeleon))
    } else if species.eq_ignore_ascii_case("CHARIZARD") {
        Some(rig(Species::Charizard))
    } else if species.eq_ignore_ascii_case("GRAVELER") {
        Some(rig(Species::Graveler))
    } else if species.eq_ignore_ascii_case("GOLEM") {
        Some(rig(Species::Golem))
    } else if species.eq_ignore_ascii_case("OMANYTE") {
        Some(rig(Species::Omanyte))
    } else if species.eq_ignore_ascii_case("OMASTAR") {
        Some(rig(Species::Omastar))
    } else if species.eq_ignore_ascii_case("AERODACTYL") {
        Some(rig(Species::Aerodactyl))
    } else if species.eq_ignore_ascii_case("DRATINI") {
        Some(rig(Species::Dratini))
    } else if species.eq_ignore_ascii_case("DRAGONAIR") {
        Some(rig(Species::Dragonair))
    } else if species.eq_ignore_ascii_case("DRAGONITE") {
        Some(rig(Species::Dragonite))
    } else if species.eq_ignore_ascii_case("CLEFABLE") {
        Some(rig(Species::Clefable))
    } else if species.eq_ignore_ascii_case("WIGGLYTUFF") {
        Some(rig(Species::Wigglytuff))
    } else if species.eq_ignore_ascii_case("SANDSHREW") {
        Some(rig(Species::Sandshrew))
    } else if species.eq_ignore_ascii_case("SANDSLASH") {
        Some(rig(Species::Sandslash))
    } else if species.eq_ignore_ascii_case("POLITOED") {
        Some(rig(Species::Politoed))
    } else if species.eq_ignore_ascii_case("POLIWRATH") {
        Some(rig(Species::Poliwrath))
    } else if species.eq_ignore_ascii_case("POLIWHIRL") {
        Some(rig(Species::Poliwhirl))
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
        if matches!(species, Species::Procedural(_)) {
            return Err("procedural species use their existing named source mesh".into());
        }
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
            hover_clearance: 0.,motion_limbs: Vec::new(),crawl_segments: Vec::new(),walk_legs: Vec::new(),walk_cycle: 0.72,ground_vertices: Vec::new(),flop_folds: Vec::new(),
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
            let base_hz: f32 = match self.species {
                Species::Chikorita | Species::Bayleef | Species::Meganium | Species::Spearow | Species::Unown(_) => 90.0,
                _ => ENVELOPE_HZ,
            };
            // Small bodies otherwise inherit too much padding from the fastest
            // appendage. Pick only the extra cached samples their speed needs,
            // instead of raising every species to a fixed high rate. Bound the
            // added diameter to 9% of the smallest neutral span when the 120 Hz
            // preparation cap permits it. The analytic margin below remains
            // conservative even at the cap; playback keeps its original clock.
            let margin_budget = (self.neutral_bounds.1 - self.neutral_bounds.0)
                .min_element() * 0.045 - FLOAT_MARGIN;
            let tight_hz = if margin_budget > 0.0 {
                (maximum_speed / (2.0 * margin_budget)).ceil().min(120.0)
            } else { base_hz };
            let envelope_hz = base_hz.max(tight_hz);
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

#[path = "pokemon_motion.rs"]
pub(crate) mod motion;
