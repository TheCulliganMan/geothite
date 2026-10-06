//! Original exact-species battle meshes. No icon-family aliases or palette guesses.
//! Ground rooted, +Y up, front +Z. Source collections retain named anatomy.
use std::sync::OnceLock;

use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;
use serde::Deserialize;

#[derive(Deserialize)]
struct Primitive {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct Export {
    primitives: Vec<Primitive>,
}

/// Parse our original authoring format without discarding smooth normals.
/// Kept render-only so battle and world presentation can share the same art.
fn parse_model<'a>(
    json: impl Into<crate::model_storage::Source<'a>>,
) -> Result<SurfaceMeshData, String> {
    let source: Export = crate::model_storage::parse(json)?;
    let mut mesh = SurfaceMeshData::default();
    for p in source.primitives {
        let n = p.positions.len() / 3;
        if n < 3
            || p.positions.len() % 3 != 0
            || p.normals.len() != p.positions.len()
            || p.indices.is_empty()
            || p.indices.len() % 3 != 0
            || p.indices.iter().any(|&i| i as usize >= n)
            || p.positions.iter().chain(&p.normals).any(|v| !v.is_finite())
            || p.base_color
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("invalid authored species primitive".into());
        }
        let base = mesh.positions.len() as u32;
        for point in p.positions.chunks_exact(3) {
            mesh.positions.push([point[0], point[1], point[2]]);
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push(p.base_color);
        }
        for normal in p.normals.chunks_exact(3) {
            let normal = Vec3::new(normal[0], normal[1], normal[2]);
            if normal.length_squared() < 0.000_001 {
                return Err("zero authored species normal".into());
            }
            mesh.normals.push(normal.normalize().to_array());
        }
        mesh.indices.extend(p.indices.into_iter().map(|i| base + i));
    }
    if mesh.positions.is_empty() {
        return Err("empty authored species model".into());
    }
    let min = (0..3)
        .map(|a| {
            mesh.positions
                .iter()
                .map(|p| p[a])
                .fold(f32::INFINITY, f32::min)
        })
        .collect::<Vec<_>>();
    let max = (0..3)
        .map(|a| {
            mesh.positions
                .iter()
                .map(|p| p[a])
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .collect::<Vec<_>>();
    if min[1].abs() > 0.000_1 || (0..3).any(|a| max[a] - min[a] < 0.005) {
        return Err("authored species must be volumetric and floor rooted".into());
    }
    Ok(mesh)
}

// Retain one explicit identity catalogue while routing articulated species to
// their canonical GLB. The rig also exposes an exact neutral surface to callers
// that only need immutable bounds or an overworld prop.
macro_rules! species_mesh {
    ("gengar") => {
        Some(crate::species_rig::rig(crate::species_rig::Species::Gengar).neutral.clone())
    };
    ("spearow") => {
        Some(
            crate::species_rig::rig(crate::species_rig::Species::Spearow)
                .neutral
                .clone(),
        )
    };
    ("pidgeotto") => {
        Some(crate::pidgeotto_rig::rig().neutral.clone())
    };
    ($file:literal) => {{
        static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
        Some(
            MODEL
                .get_or_init(|| {
                    parse_model(crate::model_storage::include_model!(concat!(
                        "models/battle_species/",
                        $file,
                        ".mesh.json"
                    )))
                    .expect(concat!("valid original species: ", $file))
                })
                .clone(),
        )
    }};
}
macro_rules! species_models {
    ($( $id:literal => $file:tt ),+ $(,)?) => {
        pub(crate) fn supported_species() -> impl Iterator<Item = &'static str> {
            [$( $id ),+].into_iter()
        }
        pub(crate) fn mesh(species: &str) -> Option<SurfaceMeshData> {
            match species.to_ascii_uppercase().as_str() { $(
                $id => species_mesh!($file),
            )+ _ => None }
        }
    };
}
species_models! {
    "ABRA" => "abra",
    "AERODACTYL" => "aerodactyl",
    "AIPOM" => "aipom",
    "ALAKAZAM" => "alakazam",
    "AMPHAROS" => "ampharos",
    "ARBOK" => "arbok",
    "ARCANINE" => "arcanine",
    "ARIADOS" => "ariados",
    "ARTICUNO" => "articuno",
    "AZUMARILL" => "azumarill",
    "BAYLEEF" => "bayleef",
    "BEEDRILL" => "beedrill",
    "BELLOSSOM" => "bellossom",
    "BELLSPROUT" => "bellsprout",
    "BLASTOISE" => "blastoise",
    "BLISSEY" => "blissey",
    "BUTTERFREE" => "butterfree",
    "CATERPIE" => "caterpie",
    "CELEBI" => "celebi",
    "CHANSEY" => "chansey",
    "CHARIZARD" => "charizard",
    "CHARMELEON" => "charmeleon",
    "CHINCHOU" => "chinchou",
    "CLEFABLE" => "clefable",
    "CLEFFA" => "cleffa",
    "CLOYSTER" => "cloyster",
    "CORSOLA" => "corsola",
    "CROBAT" => "crobat",
    "CROCONAW" => "croconaw",
    "CUBONE" => "cubone",
    "DELIBIRD" => "delibird",
    "DEWGONG" => "dewgong",
    "DITTO" => "ditto",
    "DODRIO" => "dodrio",
    "DODUO" => "doduo",
    "DONPHAN" => "donphan",
    "DRAGONAIR" => "dragonair",
    "DRAGONITE" => "dragonite",
    "DRATINI" => "dratini",
    "DROWZEE" => "drowzee",
    "DUGTRIO" => "dugtrio",
    "DUNSPARCE" => "dunsparce",
    "EEVEE" => "eevee",
    "EKANS" => "ekans",
    "ELECTABUZZ" => "electabuzz",
    "ELECTRODE" => "electrode",
    "ELEKID" => "elekid",
    "ESPEON" => "espeon",
    "EXEGGCUTE" => "exeggcute",
    "EXEGGUTOR" => "exeggutor",
    "FARFETCH_D" => "farfetch_d",
    "FEAROW" => "fearow",
    "FERALIGATR" => "feraligatr",
    "FLAAFFY" => "flaaffy",
    "FLAREON" => "flareon",
    "FORRETRESS" => "forretress",
    "FURRET" => "furret",
    "GASTLY" => "gastly",
    "GENGAR" => "gengar",
    "GIRAFARIG" => "girafarig",
    "GLIGAR" => "gligar",
    "GLOOM" => "gloom",
    "GOLBAT" => "golbat",
    "GOLDEEN" => "goldeen",
    "GOLDUCK" => "golduck",
    "GOLEM" => "golem",
    "GRANBULL" => "granbull",
    "GRAVELER" => "graveler",
    "GRIMER" => "grimer",
    "GROWLITHE" => "growlithe",
    "HAUNTER" => "haunter",
    "HERACROSS" => "heracross",
    "HITMONCHAN" => "hitmonchan",
    "HITMONLEE" => "hitmonlee",
    "HITMONTOP" => "hitmontop",
    "HOPPIP" => "hoppip",
    "HORSEA" => "horsea",
    "HOUNDOOM" => "houndoom",
    "HOUNDOUR" => "houndour",
    "HYPNO" => "hypno",
    "IGGLYBUFF" => "igglybuff",
    "IVYSAUR" => "ivysaur",
    "JOLTEON" => "jolteon",
    "JUMPLUFF" => "jumpluff",
    "JYNX" => "jynx",
    "KABUTO" => "kabuto",
    "KABUTOPS" => "kabutops",
    "KADABRA" => "kadabra",
    "KAKUNA" => "kakuna",
    "KANGASKHAN" => "kangaskhan",
    "KINGDRA" => "kingdra",
    "KINGLER" => "kingler",
    "KOFFING" => "koffing",
    "KRABBY" => "krabby",
    "LANTURN" => "lanturn",
    "LARVITAR" => "larvitar",
    "LEDIAN" => "ledian",
    "LEDYBA" => "ledyba",
    "LICKITUNG" => "lickitung",
    "MACHAMP" => "machamp",
    "MACHOKE" => "machoke",
    "MACHOP" => "machop",
    "MAGBY" => "magby",
    "MAGCARGO" => "magcargo",
    "MAGIKARP" => "magikarp",
    "MAGMAR" => "magmar",
    "MAGNEMITE" => "magnemite",
    "MAGNETON" => "magneton",
    "MANKEY" => "mankey",
    "MANTINE" => "mantine",
    "MAREEP" => "mareep",
    "MARILL" => "marill",
    "MAROWAK" => "marowak",
    "MEGANIUM" => "meganium",
    "MEOWTH" => "meowth",
    "METAPOD" => "metapod",
    "MEW" => "mew",
    "MEWTWO" => "mewtwo",
    "MILTANK" => "miltank",
    "MISDREAVUS" => "misdreavus",
    "MOLTRES" => "moltres",
    "MR__MIME" => "mr__mime",
    "MUK" => "muk",
    "MURKROW" => "murkrow",
    "NATU" => "natu",
    "NIDOKING" => "nidoking",
    "NIDOQUEEN" => "nidoqueen",
    "NIDORAN_F" => "nidoran_f",
    "NIDORAN_M" => "nidoran_m",
    "NIDORINA" => "nidorina",
    "NIDORINO" => "nidorino",
    "NINETALES" => "ninetales",
    "NOCTOWL" => "noctowl",
    "OCTILLERY" => "octillery",
    "OMANYTE" => "omanyte",
    "OMASTAR" => "omastar",
    "PARAS" => "paras",
    "PARASECT" => "parasect",
    "PERSIAN" => "persian",
    "PHANPY" => "phanpy",
    "PICHU" => "pichu",
    "PIDGEOT" => "pidgeot",
    "PIDGEOTTO" => "pidgeotto",
    "PILOSWINE" => "piloswine",
    "PINECO" => "pineco",
    "PINSIR" => "pinsir",
    "POLITOED" => "politoed",
    "POLIWHIRL" => "poliwhirl",
    "POLIWRATH" => "poliwrath",
    "PONYTA" => "ponyta",
    "PORYGON" => "porygon",
    "PORYGON2" => "porygon2",
    "PRIMEAPE" => "primeape",
    "PSYDUCK" => "psyduck",
    "PUPITAR" => "pupitar",
    "QUAGSIRE" => "quagsire",
    "QUILAVA" => "quilava",
    "QWILFISH" => "qwilfish",
    "RAICHU" => "raichu",
    "RAPIDASH" => "rapidash",
    "RATICATE" => "raticate",
    "REMORAID" => "remoraid",
    "RHYDON" => "rhydon",
    "RHYHORN" => "rhyhorn",
    "SANDSHREW" => "sandshrew",
    "SANDSLASH" => "sandslash",
    "SCIZOR" => "scizor",
    "SCYTHER" => "scyther",
    "SEADRA" => "seadra",
    "SEAKING" => "seaking",
    "SEEL" => "seel",
    "SHELLDER" => "shellder",
    "SHUCKLE" => "shuckle",
    "SKARMORY" => "skarmory",
    "SKIPLOOM" => "skiploom",
    "SLOWBRO" => "slowbro",
    "SLOWKING" => "slowking",
    "SLUGMA" => "slugma",
    "SMEARGLE" => "smeargle",
    "SMOOCHUM" => "smoochum",
    "SNEASEL" => "sneasel",
    "SNUBBULL" => "snubbull",
    "SPEAROW" => "spearow",
    "SPINARAK" => "spinarak",
    "STANTLER" => "stantler",
    "STARMIE" => "starmie",
    "STEELIX" => "steelix",
    "SUNFLORA" => "sunflora",
    "SUNKERN" => "sunkern",
    "SWINUB" => "swinub",
    "TANGELA" => "tangela",
    "TAUROS" => "tauros",
    "TEDDIURSA" => "teddiursa",
    "TENTACOOL" => "tentacool",
    "TENTACRUEL" => "tentacruel",
    "TOGEPI" => "togepi",
    "TOGETIC" => "togetic",
    "TYPHLOSION" => "typhlosion",
    "TYRANITAR" => "tyranitar",
    "TYROGUE" => "tyrogue",
    "UMBREON" => "umbreon",
    "URSARING" => "ursaring",
    "VAPOREON" => "vaporeon",
    "VENOMOTH" => "venomoth",
    "VENONAT" => "venonat",
    "VENUSAUR" => "venusaur",
    "VICTREEBEL" => "victreebel",
    "VILEPLUME" => "vileplume",
    "VULPIX" => "vulpix",
    "WARTORTLE" => "wartortle",
    "WEEDLE" => "weedle",
    "WEEPINBELL" => "weepinbell",
    "WEEZING" => "weezing",
    "WIGGLYTUFF" => "wigglytuff",
    "WOBBUFFET" => "wobbuffet",
    "WOOPER" => "wooper",
    "XATU" => "xatu",
    "YANMA" => "yanma",
    "ZAPDOS" => "zapdos",
    "ZUBAT" => "zubat",
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_exact_species_is_grounded_and_volumetric() {
        let mut seen = HashSet::new();
        for species in supported_species() {
            assert!(seen.insert(species));
            let m = mesh(species).expect(species);
            assert_eq!(m.positions.len(), m.normals.len());
            assert_eq!(m.positions.len(), m.colors.len());
            assert!(m.indices.len() >= 60);
            assert!(m.indices.iter().all(|&i| (i as usize) < m.positions.len()));
            let lo = std::array::from_fn::<_, 3, _>(|a| {
                m.positions
                    .iter()
                    .map(|p| p[a])
                    .fold(f32::INFINITY, f32::min)
            });
            let hi = std::array::from_fn::<_, 3, _>(|a| {
                m.positions
                    .iter()
                    .map(|p| p[a])
                    .fold(f32::NEG_INFINITY, f32::max)
            });
            assert!(lo[1].abs() < 0.0001, "{species}");
            assert!((0..3).all(|a| hi[a] - lo[a] > 0.005), "{species}");
            assert!(
                m.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length_squared() - 1.0).abs() < 0.0001)
            );
        }
    }

    #[test]
    fn unknown_and_generic_families_are_explicitly_unsupported() {
        for name in [
            "",
            "MISSINGNO",
            "icon_fish",
            "monster",
            "bird",
            "gfx/pokemon/bayleef/front.png",
        ] {
            assert!(mesh(name).is_none(), "{name}");
        }
        assert!(mesh("bayleef").is_some());
    }
}
