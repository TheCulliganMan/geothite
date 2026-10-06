//! Original source-scoped volumetric actor assets, with no simulation state.
//!
//! Runtime source identity is published after variable-sprite and icon
//! resolution. `monster` and `icon_monster` therefore remain different keys.
//! Models are ground rooted (+Y up, front +Z) and never infer species from
//! colors, original script tokens, actor IDs, or another actor's appearance.
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
            return Err("invalid authored actor primitive".into());
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
                return Err("zero authored actor normal".into());
            }
            mesh.normals.push(normal.normalize().to_array());
        }
        mesh.indices.extend(p.indices.into_iter().map(|i| base + i));
    }
    if mesh.positions.is_empty() {
        return Err("empty authored actor model".into());
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
        return Err("authored actor must be volumetric and floor rooted".into());
    }
    Ok(mesh)
}

macro_rules! prop_mesh {
    ("battle_chikorita") => { crate::species_rig::rig(crate::species_rig::Species::Chikorita).neutral.clone() };
    ("battle_cyndaquil") => { crate::species_rig::rig(crate::species_rig::Species::Cyndaquil).neutral.clone() };
    ("battle_totodile") => { crate::species_rig::rig(crate::species_rig::Species::Totodile).neutral.clone() };
    ($label:literal) => {{
        static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
        MODEL.get_or_init(|| parse_model(crate::model_storage::include_model!(concat!("models/actor_props/", $label, ".mesh.json")))
            .expect(concat!("valid original actor asset: ", $label))).clone()
    }};
}

macro_rules! authored_props {
    ($( $variant:ident => $label:tt ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub(crate) enum PropKind { $( $variant ),+, ExactSpecies(&'static str) }
        pub(crate) const ALL_KINDS: &[PropKind] = &[$( PropKind::$variant ),+];
        impl PropKind {
            pub(crate) fn label(self) -> &'static str {
                match self { $( Self::$variant => $label ),+, Self::ExactSpecies(species) => species }
            }
        }
        pub(crate) fn mesh(kind: PropKind) -> SurfaceMeshData {
            match kind { $(
                PropKind::$variant => prop_mesh!($label)
            ),+ ,
                PropKind::ExactSpecies(species) => crate::battle_species_models::mesh(species)
                    .or_else(|| battle_species_kind(species).map(mesh))
                    .expect("verified exact visible species"),
            }
        }
    };
}
authored_props! {
    PokeBall => "poke_ball",
    Boulder => "boulder",
    Rock => "rock",
    FruitTree => "fruit_tree",
    Paper => "paper",
    Pokedex => "pokedex",
    Famicom => "famicom",
    Snes => "snes",
    N64 => "n64",
    VirtualBoy => "virtual_boy",
    GoldTrophy => "gold_trophy",
    SilverTrophy => "silver_trophy",
    Bird => "bird",
    Fairy => "fairy",
    Monster => "monster",
    Dragon => "dragon",
    Slowpoke => "slowpoke",
    Sudowoodo => "sudowoodo",
    Entei => "entei",
    Raikou => "raikou",
    Suicune => "suicune",
    BigSnorlax => "big_snorlax",
    BigLapras => "big_lapras",
    BigOnix => "big_onix",
    Surf => "surf",
    SurfingPikachu => "surfing_pikachu",
    IconBat => "icon_bat",
    IconBigmon => "icon_bigmon",
    IconBird => "icon_bird",
    IconBlob => "icon_blob",
    IconBug => "icon_bug",
    IconBulbasaur => "icon_bulbasaur",
    IconCaterpillar => "icon_caterpillar",
    IconCharmander => "icon_charmander",
    IconClefairy => "icon_clefairy",
    IconDiglett => "icon_diglett",
    IconEgg => "icon_egg",
    IconEquine => "icon_equine",
    IconFighter => "icon_fighter",
    IconFish => "icon_fish",
    IconFox => "icon_fox",
    IconGeodude => "icon_geodude",
    IconGhost => "icon_ghost",
    IconGyarados => "icon_gyarados",
    IconHoOh => "icon_ho_oh",
    IconHumanshape => "icon_humanshape",
    IconJellyfish => "icon_jellyfish",
    IconJigglypuff => "icon_jigglypuff",
    IconLapras => "icon_lapras",
    IconLugia => "icon_lugia",
    IconMonster => "icon_monster",
    IconMoth => "icon_moth",
    IconOddish => "icon_oddish",
    IconPikachu => "icon_pikachu",
    IconPoliwag => "icon_poliwag",
    IconSerpent => "icon_serpent",
    IconShell => "icon_shell",
    IconSlowpoke => "icon_slowpoke",
    IconSnorlax => "icon_snorlax",
    ChrisBike => "chris_bike",
    KrisBike => "kris_bike",
    BattleChikorita => "battle_chikorita",
    BattleCyndaquil => "battle_cyndaquil",
    BattleTotodile => "battle_totodile",
    BattlePidgey => "battle_pidgey",
    BattleRattata => "battle_rattata",
    BattleSentret => "battle_sentret",
    BattleHoothoot => "battle_hoothoot",
    IconSquirtle => "icon_squirtle",
    IconStaryu => "icon_staryu",
    IconSudowoodo => "icon_sudowoodo",
    IconUnown => "icon_unown",
    IconVoltorb => "icon_voltorb",
}

/// Exact resolved source/path handling. In particular, gfx/icons/monster.png
/// cannot accidentally claim the unrelated sprite family's representation.
pub(crate) fn prop_kind_for_source(source: &str) -> Option<PropKind> {
    // The publisher supplies only the currently resolved visible species and
    // actual bitmap source. Never recover a species from a shared icon family.
    if let Some(identity) = source.strip_prefix("species:") {
        let (species, art) = identity.split_once(':')?;
        if art.contains(':') || art.contains('/') {
            return None;
        }
        let known_art = art == species.to_ascii_lowercase() || prop_kind_for_source(art).is_some();
        if !known_art {
            return None;
        }
        let exact = crate::battle_species_models::supported_species()
            .chain(EXACT_SHARED_SPECIES.iter().copied())
            .find(|candidate| *candidate == species)?;
        return Some(PropKind::ExactSpecies(exact));
    }
    let source = if let Some(path) = source.strip_prefix("gfx/sprites/") {
        let stem = path.strip_suffix(".png")?;
        if stem.starts_with("icon_") {
            return None;
        }
        stem.to_owned()
    } else if let Some(path) = source.strip_prefix("gfx/icons/") {
        format!("icon_{}", path.strip_suffix(".png")?)
    } else {
        source.to_owned()
    };
    if source.contains('/') || source.starts_with("battle_") {
        return None;
    }
    ALL_KINDS
        .iter()
        .copied()
        .find(|kind| kind.label() == source)
}

pub(crate) const EXACT_SHARED_SPECIES: &[&str] = &[
    "CHIKORITA",
    "CYNDAQUIL",
    "TOTODILE",
    "PIDGEY",
    "RATTATA",
    "SENTRET",
    "HOOTHOOT",
    "ENTEI",
    "RAIKOU",
    "SUICUNE",
    "ONIX",
    "BULBASAUR",
    "CHARMANDER",
    "CLEFAIRY",
    "DIGLETT",
    "GEODUDE",
    "GYARADOS",
    "HO_OH",
    "JIGGLYPUFF",
    "LAPRAS",
    "LUGIA",
    "ODDISH",
    "PIKACHU",
    "POLIWAG",
    "SLOWPOKE",
    "SNORLAX",
    "SQUIRTLE",
    "STARYU",
    "SUDOWOODO",
    "UNOWN",
    "VOLTORB",
];

/// These are exact battle species, separately authored from generic icon
/// families. An unsupported species returns None rather than a false match.
pub(crate) fn battle_species_kind(species: &str) -> Option<PropKind> {
    match species.to_ascii_uppercase().as_str() {
        "CHIKORITA" => Some(PropKind::BattleChikorita),
        "CYNDAQUIL" => Some(PropKind::BattleCyndaquil),
        "TOTODILE" => Some(PropKind::BattleTotodile),
        "PIDGEY" => Some(PropKind::BattlePidgey),
        "RATTATA" => Some(PropKind::BattleRattata),
        "SENTRET" => Some(PropKind::BattleSentret),
        "HOOTHOOT" => Some(PropKind::BattleHoothoot),
        // Sprite/icon families with exact species names have their own actual
        // recognizable species meshes. Do not map generic bigmon/fish/etc.
        "ENTEI" => Some(PropKind::Entei),
        "RAIKOU" => Some(PropKind::Raikou),
        "SUICUNE" => Some(PropKind::Suicune),
        "ONIX" => Some(PropKind::BigOnix),
        "BULBASAUR" => Some(PropKind::IconBulbasaur),
        "CHARMANDER" => Some(PropKind::IconCharmander),
        "CLEFAIRY" => Some(PropKind::IconClefairy),
        "DIGLETT" => Some(PropKind::IconDiglett),
        "GEODUDE" => Some(PropKind::IconGeodude),
        "GYARADOS" => Some(PropKind::IconGyarados),
        "HO_OH" => Some(PropKind::IconHoOh),
        "JIGGLYPUFF" => Some(PropKind::IconJigglypuff),
        "LAPRAS" => Some(PropKind::IconLapras),
        "LUGIA" => Some(PropKind::IconLugia),
        "ODDISH" => Some(PropKind::IconOddish),
        "PIKACHU" => Some(PropKind::IconPikachu),
        "POLIWAG" => Some(PropKind::IconPoliwag),
        "SLOWPOKE" => Some(PropKind::IconSlowpoke),
        "SNORLAX" => Some(PropKind::IconSnorlax),
        "SQUIRTLE" => Some(PropKind::IconSquirtle),
        "STARYU" => Some(PropKind::IconStaryu),
        "SUDOWOODO" => Some(PropKind::IconSudowoodo),
        "UNOWN" => Some(PropKind::IconUnown),
        "VOLTORB" => Some(PropKind::IconVoltorb),
        _ => None,
    }
}

pub(crate) fn battle_species_mesh(species: &str) -> Option<SurfaceMeshData> {
    battle_species_kind(species).map(mesh)
}

pub(crate) fn kind_labels() -> impl Iterator<Item = &'static str> {
    ALL_KINDS.iter().copied().map(PropKind::label)
}

/// Compatibility entry point for the previously supported orchard actor.
pub(super) fn fruit_tree() -> SurfaceMeshData {
    mesh(PropKind::FruitTree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn exact_source_families_preserve_icon_namespace_and_reject_guesses() {
        assert_eq!(prop_kind_for_source("poke_ball"), Some(PropKind::PokeBall));
        assert_eq!(
            prop_kind_for_source("gfx/sprites/monster.png"),
            Some(PropKind::Monster)
        );
        assert_eq!(
            prop_kind_for_source("gfx/icons/monster.png"),
            Some(PropKind::IconMonster)
        );
        assert_ne!(
            prop_kind_for_source("monster"),
            prop_kind_for_source("icon_monster")
        );
        for bad in [
            "fruit_tree_picked",
            "monster_extra",
            "SPRITE_MONSTER",
            "pikachu",
            "remote_player",
            "battle_chikorita",
            "gfx/sprites/icon_monster.png/extra",
            "gfx/sprites/icon_monster.png",
            "path/poke_ball",
        ] {
            assert_eq!(prop_kind_for_source(bad), None, "{bad}");
        }
        for kind in ALL_KINDS
            .iter()
            .filter(|kind| !kind.label().starts_with("battle_"))
        {
            assert_eq!(prop_kind_for_source(kind.label()), Some(*kind));
        }
    }

    #[test]
    fn battle_species_never_use_generic_family_aliases() {
        assert_eq!(
            battle_species_kind("CHIKORITA"),
            Some(PropKind::BattleChikorita)
        );
        assert_eq!(
            battle_species_kind("RATTATA"),
            Some(PropKind::BattleRattata)
        );
        assert_eq!(battle_species_kind("PIDGEY"), Some(PropKind::BattlePidgey));
        for missing in [
            "EEVEE",
            "CHARIZARD",
            "MAGIKARP",
            "NIDORAN_M",
            "monster",
            "icon_fish",
        ] {
            assert_eq!(battle_species_kind(missing), None);
        }
    }

    #[test]
    fn every_authored_actor_is_finite_volumetric_and_floor_rooted() {
        let mut labels = HashSet::new();
        for &kind in ALL_KINDS {
            assert!(labels.insert(kind.label()));
            let model = mesh(kind);
            assert!(model.positions.len() >= 12, "{}", kind.label());
            assert_eq!(model.positions.len(), model.normals.len());
            assert_eq!(model.positions.len(), model.colors.len());
            assert_eq!(model.positions.len(), model.uvs.len());
            let min_y = model
                .positions
                .iter()
                .map(|p| p[1])
                .fold(f32::INFINITY, f32::min);
            assert!(min_y.abs() < 0.0001, "{}: {min_y}", kind.label());
            assert!(model
                .indices
                .iter()
                .all(|&i| (i as usize) < model.positions.len()));
            for n in model.normals {
                assert!((Vec3::from_array(n).length() - 1.0).abs() < 0.0001);
            }
        }
        assert_eq!(ALL_KINDS.len(), 73);
    }

    #[test]
    fn malformed_or_unanchored_exports_fail_closed() {
        assert!(parse_model(r#"{"primitives":[]}"#).is_err());
        assert!(parse_model(r#"{"primitives":[{"positions":[0,0,0],"normals":[0,1,0],"indices":[0,1,2],"base_color":[1,1,1,1]}]}"#).is_err());
    }
}

#[cfg(test)]
mod exact_visible_species_tests {
    use super::*;
    #[test]
    fn published_amphy_identity_reuses_exact_ampharos_mesh_and_keeps_monsters_generic() {
        let kind = prop_kind_for_source("species:AMPHAROS:monster").unwrap();
        assert_eq!(kind, PropKind::ExactSpecies("AMPHAROS"));
        assert_eq!(prop_kind_for_source("monster"), Some(PropKind::Monster));
        assert_eq!(
            prop_kind_for_source("icon_monster"),
            Some(PropKind::IconMonster)
        );
        let model = mesh(kind);
        let exact = crate::battle_species_models::mesh("AMPHAROS").unwrap();
        assert_eq!(model.positions, exact.positions);
        assert_eq!(model.indices, exact.indices);
        assert_eq!(model.normals, exact.normals);
        assert_eq!(model.colors, exact.colors);
        let min_y = model
            .positions
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min);
        assert!(min_y.abs() < 0.0001);
    }

    #[test]
    fn shared_icon_families_do_not_erase_current_species() {
        for (species, art) in [
            ("MAGIKARP", "icon_fish"),
            ("GENGAR", "icon_ghost"),
            ("GRIMER", "icon_blob"),
            ("WEEDLE", "icon_caterpillar"),
            ("MACHOP", "icon_humanshape"),
            ("TENTACOOL", "icon_jellyfish"),
            ("SHELLDER", "icon_shell"),
            ("STARYU", "icon_staryu"),
        ] {
            let kind = prop_kind_for_source(&format!("species:{species}:{art}")).unwrap();
            assert_eq!(kind, PropKind::ExactSpecies(species));
            assert_ne!(kind, prop_kind_for_source(art).unwrap());
            let sculpt = mesh(kind);
            assert!(!sculpt.indices.is_empty());
            assert!(sculpt.positions.iter().any(|p| p[2] > 0.01));
        }
        for unknown in [
            "species:MISSING:icon_fish",
            "species:MAGIKARP:new_art",
            "species:MAGIKARP:gfx/icons/fish.png",
            "species:MAGIKARP:species:GENGAR:icon_ghost",
        ] {
            assert_eq!(prop_kind_for_source(unknown), None, "{unknown}");
        }
    }
    #[test]
    fn every_normal_species_has_a_bounded_reusable_visible_model() {
        let mut species: Vec<_> = crate::battle_species_models::supported_species()
            .chain(EXACT_SHARED_SPECIES.iter().copied())
            .collect();
        species.sort_unstable();
        species.dedup();
        assert_eq!(species.len(), 251);
        for name in species {
            let source = format!("species:{name}:icon_monster");
            assert_eq!(
                prop_kind_for_source(&source),
                Some(PropKind::ExactSpecies(name))
            );
        }
    }
}
