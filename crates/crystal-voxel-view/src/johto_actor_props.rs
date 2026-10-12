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
    #[serde(default)]
    material: String,
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct Export {
    #[serde(default)]
    name: String,
    primitives: Vec<Primitive>,
}

/// Parse our original authoring format without discarding smooth normals.
/// Kept render-only so battle and world presentation can share the same art.
fn parse_model<'a>(
    json: impl Into<crate::model_storage::Source<'a>>,
) -> Result<SurfaceMeshData, String> {
    parse_model_palette(json, false)
}

fn parse_model_palette<'a>(
    json: impl Into<crate::model_storage::Source<'a>>,
    gold_gym_stone: bool,
) -> Result<SurfaceMeshData, String> {
    let source: Export = crate::model_storage::parse(json)?;
    if gold_gym_stone && source.name != "boulder" {
        return Err("gym palette requires the named boulder asset".into());
    }
    let mut mesh = SurfaceMeshData::default();
    for p in source.primitives {
        let color = if gold_gym_stone {
            let rgb = match p.material.as_str() {
                "stone" => [165, 123, 24],
                "stone_light" => [198, 148, 57],
                "stone_dark" => [57, 57, 57],
                _ => return Err("gym boulder requires named stone materials".into()),
            };
            // Verified Cianwood/Blackthorn sprite shades; PBR vertex colors are linear.
            let linear = |v: u8| {
                let v = f32::from(v) / 255.;
                if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            [linear(rgb[0]), linear(rgb[1]), linear(rgb[2]), 1.]
        } else { p.base_color };
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
            mesh.colors.push(color);
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

pub(crate) fn gold_gym_boulder_mesh() -> SurfaceMeshData {
    static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
    MODEL.get_or_init(|| parse_model_palette(
        crate::model_storage::include_model!("models/actor_props/boulder.mesh.json"), true,
    ).expect("named Cianwood boulder palette")).clone()
}

macro_rules! prop_mesh {
    ("chris_bike") => { crate::bicycle::mesh(false) };
    ("kris_bike") => { crate::bicycle::mesh(true) };
    ("battle_chikorita") => {
        crate::species_rig::rig(crate::species_rig::Species::Chikorita)
            .neutral
            .clone()
    };
    ("battle_cyndaquil") => {
        crate::species_rig::rig(crate::species_rig::Species::Cyndaquil)
            .neutral
            .clone()
    };
    ("battle_totodile") => {
        crate::species_rig::rig(crate::species_rig::Species::Totodile)
            .neutral
            .clone()
    };
    ($label:literal) => {{
        static MODEL: OnceLock<SurfaceMeshData> = OnceLock::new();
        MODEL
            .get_or_init(|| {
                parse_model(crate::model_storage::include_model!(concat!(
                    "models/actor_props/",
                    $label,
                    ".mesh.json"
                )))
                .expect(concat!("valid original actor asset: ", $label))
            })
            .clone()
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
    fn gold_gym_boulder_named_palette_preserves_geometry_and_other_assets() {
        let ordinary = mesh(PropKind::Boulder);
        let gym = gold_gym_boulder_mesh();
        assert_eq!(ordinary.positions, gym.positions);
        assert_eq!(ordinary.normals, gym.normals);
        assert_eq!(ordinary.indices, gym.indices);
        assert_eq!(ordinary.uvs, gym.uvs);
        assert_ne!(ordinary.colors, gym.colors);
        assert_eq!(mesh(PropKind::Boulder), ordinary);
        let json = include_str!("../models/actor_props/boulder.mesh.json");
        assert!(parse_model_palette(json.replace("stone_light", "unknown").as_str(), true).is_err());
        assert!(parse_model_palette(json.replace("\"boulder\"", "\"rock\"").as_str(), true).is_err());
        // These are the reviewed sprite palette's ochre body, gold highlight
        // and charcoal outline. Comparing decoded display colors catches a
        // generic gray substitution as well as double sRGB conversion.
        let display_rgb = |color: &[f32; 4]| -> [u8; 3] {
            std::array::from_fn(|i| {
                let v = color[i];
                let srgb = if v <= 0.0031308 { v * 12.92 }
                    else { 1.055 * v.powf(1. / 2.4) - 0.055 };
                (srgb * 255.).round() as u8
            })
        };
        let expected = [[165, 123, 24], [198, 148, 57], [57, 57, 57]];
        for rgb in expected {
            assert!(gym.colors.iter().any(|color| display_rgb(color) == rgb),
                "movable Gym boulders must retain every source palette shade");
        }
        for color in &gym.colors {
            assert!(color[3] == 1. && color.iter().all(|v| v.is_finite()));
            assert!(expected.contains(&display_rgb(color)),
                "movable Gym boulders must not inherit a generic stone palette");
        }
    }

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
    fn exact_species_props_share_canonical_bodies_without_icon_substitution() {
        for species in ["CHIKORITA", "CYNDAQUIL", "TOTODILE"] {
            assert_eq!(mesh(PropKind::ExactSpecies(species)),
                crate::battle_species_models::mesh(species).unwrap());
            assert!(crate::has_authored_battle_species(species));
        }
        for source in ["monster", "icon_fish", "species:UNKNOWN:icon_fish"] {
            assert!(!crate::has_authored_battle_species(source));
        }
        assert_eq!(prop_kind_for_source("species:UNKNOWN:icon_fish"),None);
        for (species, icon) in [("PIDGEY", "icon_bird"), ("RATTATA", "icon_monster"), ("STARYU", "icon_staryu")] {
            assert!(!crate::has_authored_battle_species(species));
            assert_eq!(prop_kind_for_source(&format!("species:{species}:{icon}")), None);
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
        assert_eq!(ALL_KINDS.len(), 69);
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
    fn ho_oh_uses_its_dedicated_battle_model_and_preserves_icon_routing() {
        assert_eq!(
            crate::battle_species_models::supported_species()
                .filter(|name| *name == "HO_OH")
                .count(),
            1
        );
        assert_eq!(prop_kind_for_source("icon_ho_oh"), Some(PropKind::IconHoOh));
        assert_eq!(
            prop_kind_for_source("species:HO_OH:icon_ho_oh"),
            Some(PropKind::ExactSpecies("HO_OH"))
        );
        let model = mesh(PropKind::ExactSpecies("HO_OH"));
        let battle = crate::battle_species_models::mesh("HO_OH").unwrap();
        assert_eq!(model.positions, battle.positions);
        assert_eq!(model.indices, battle.indices);
        assert_eq!(model.normals, battle.normals);
        assert_eq!(model.colors, battle.colors);
        assert_ne!(model.positions, mesh(PropKind::IconHoOh).positions);
    }

    #[test]
    fn lugia_uses_its_dedicated_battle_model_and_preserves_icon_routing() {
        assert_eq!(
            crate::battle_species_models::supported_species()
                .filter(|name| *name == "LUGIA")
                .count(),
            1
        );
        assert_eq!(prop_kind_for_source("icon_lugia"), Some(PropKind::IconLugia));
        assert_eq!(
            prop_kind_for_source("species:LUGIA:icon_lugia"),
            Some(PropKind::ExactSpecies("LUGIA"))
        );
        let model = mesh(PropKind::ExactSpecies("LUGIA"));
        let battle = crate::battle_species_models::mesh("LUGIA").unwrap();
        assert_eq!(model.positions, battle.positions);
        assert_eq!(model.indices, battle.indices);
        assert_eq!(model.normals, battle.normals);
        assert_eq!(model.colors, battle.colors);
        assert_ne!(model.positions, mesh(PropKind::IconLugia).positions);
    }

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
    fn every_canonical_species_has_a_bounded_reusable_visible_model() {
        let mut species: Vec<_> = crate::battle_species_models::supported_species()
            .collect();
        species.sort_unstable();
        species.dedup();
        assert_eq!(species.len(), 225);
        for name in species {
            let source = format!("species:{name}:icon_monster");
            assert_eq!(
                prop_kind_for_source(&source),
                Some(PropKind::ExactSpecies(name))
            );
            let model = mesh(PropKind::ExactSpecies(name));
            assert_eq!(model.positions.len(), model.normals.len(), "{name}");
            assert_eq!(model.positions.len(), model.colors.len(), "{name}");
            assert!(model.indices.len() >= 60, "{name}");
            assert!(
                model
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < model.positions.len()),
                "{name}"
            );
            assert!(
                model.positions.iter().flatten().all(|v| v.is_finite()),
                "{name}"
            );
            let lo = std::array::from_fn::<_, 3, _>(|axis| {
                model
                    .positions
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::INFINITY, f32::min)
            });
            let hi = std::array::from_fn::<_, 3, _>(|axis| {
                model
                    .positions
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::NEG_INFINITY, f32::max)
            });
            assert!(lo[1].abs() < 0.0001, "{name}");
            assert!((0..3).all(|axis| hi[axis] - lo[axis] > 0.005), "{name}");
            assert!(
                model.normals.iter().all(|n| n.iter().all(|v| v.is_finite())
                    && (n.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 0.0001),
                "{name}"
            );
        }
    }
}
