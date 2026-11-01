//! Add original Minnesota species to an already verified local game pack.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::models::{
    FrontpicAnimCommand, FrontpicAnimProgram, PokemonSpecies, RuntimePokedexEntry,
};
use crystal_core::systems::evolution::{EvolutionEntry, EvolutionTable};
use crystal_core::systems::learnsets::LearnsetEntry;

use crate::minnesota_pokemon_graphics;
use crate::minnesota_pokemon_profiles::{self, Member, Profile, Roster};
use crate::{CompiledGamePack, ModpackAudioKind};

/// Fully authored profiles and reviewed native graphics, independent of a base pack.
/// Loading does not generate, export, publish or modify source assets.
pub struct MinnesotaPokemonCatalog {
    roster: Roster,
    profiles: BTreeMap<String, Profile>,
    graphics: BTreeMap<String, BTreeMap<String, Vec<u8>>>,
}

impl MinnesotaPokemonCatalog {
    /// Load canonical identities, complete profile batches and reviewed native assets.
    ///
    /// `profiles` may be one JSON batch file or a directory of JSON batches.
    /// Returns errors for incomplete/duplicate profiles, identity conflicts, malformed
    /// profiles, stale visual reviews, missing art, or PNG/index/2bpp disagreement.
    pub fn load(pokemon_root: &Path, native_root: &Path, profiles: &Path) -> Result<Self> {
        let mut roster: Roster =
            serde_json::from_slice(&fs::read(pokemon_root.join("roster.json"))?)?;
        let mut profiles = minnesota_pokemon_profiles::load_profiles(profiles)?;
        let mut ids = BTreeSet::new();
        let mut tokens = BTreeSet::new();
        let mut families = BTreeSet::new();
        for family in &roster.families {
            ensure!(
                families.insert(&family.id),
                "duplicate family {}",
                family.id
            );
            ensure!(
                (1..=3).contains(&family.members.len()),
                "invalid family size"
            );
            let family_ids: BTreeSet<_> = family
                .members
                .iter()
                .map(|member| member.id.as_str())
                .collect();
            for member in &family.members {
                ensure!(
                    ids.insert(member.id.clone()),
                    "duplicate regional species {}",
                    member.id
                );
                ensure!(
                    !member.name.is_empty()
                        && member.name.len() <= 10
                        && member.name.bytes().all(|byte| byte.is_ascii_alphabetic()),
                    "species name must contain1..10 ASCII letters"
                );
                ensure!(
                    tokens.insert(member.name.to_ascii_uppercase()),
                    "duplicate runtime species name {}",
                    member.name
                );
                ensure!(
                    (1..=2).contains(&member.types.len()),
                    "species needs one or two types"
                );
                ensure!(
                    member.types.iter().all(|kind| [
                        "NORMAL", "FIRE", "WATER", "GRASS", "ELECTRIC", "ICE", "FIGHTING",
                        "POISON", "GROUND", "FLYING", "PSYCHIC", "BUG", "ROCK", "GHOST", "DRAGON",
                        "DARK", "STEEL", "FAIRY"
                    ]
                    .contains(&kind.as_str())),
                    "unknown canonical species type"
                );
                if let Some(target) = &member.evolves_to {
                    ensure!(
                        family_ids.contains(target.as_str()),
                        "evolution target outside family closure"
                    );
                }
                let profile = profiles
                    .get(&member.id)
                    .with_context(|| format!("missing profile {}", member.id))?;
                ensure!(
                    member.evolves_to.is_some() == profile.evolution.is_some(),
                    "evolution profile disagrees with canonical family for {}",
                    member.id
                );
                runtime_number(member)?;
            }
        }
        ensure!(
            ids == profiles.keys().cloned().collect(),
            "profile coverage differs from canonical species IDs"
        );
        crate::minnesota_pokemon_curation::apply(pokemon_root, &mut roster, &mut profiles)?;
        let mut graphics = BTreeMap::new();
        for family in &roster.families {
            graphics.insert(
                family.id.clone(),
                minnesota_pokemon_graphics::load_family_graphics(
                    pokemon_root,
                    native_root,
                    family,
                    &profiles,
                )
                .with_context(|| format!("native graphics {}", family.id))?,
            );
        }
        Ok(Self {
            roster,
            profiles,
            graphics,
        })
    }

    /// Return stable canonical family IDs for complete evolutionary family packs.
    pub fn family_ids(&self) -> Vec<&str> {
        self.roster
            .families
            .iter()
            .map(|family| family.id.as_str())
            .collect()
    }

    /// Build an aggregate extension or one complete evolutionary family in memory.
    ///
    /// `family` selects a canonical family ID; `None` includes the entire catalog.
    /// Every output retains all base species, maps, runtime files and audio bytes.
    /// Northwoods always uses the modern physical/special split. It is installed
    /// when missing; incompatible existing categories are an error.
    /// Returns errors for an invalid base, duplicate species/numbers/assets, unknown
    /// moves/items/icons/cries/footprints, incomplete evolution closure or invalid
    /// runtime data. This method never writes a pack file.
    pub fn build(&self, base: &CompiledGamePack, family: Option<&str>) -> Result<CompiledGamePack> {
        crate::verify_compiled_game_pack_for_runtime(base)?;
        if let Some(id) = family {
            ensure!(
                self.roster
                    .families
                    .iter()
                    .any(|candidate| candidate.id == id),
                "unknown Minnesota family {id}"
            );
        }
        let manifest_id = family.map_or_else(
            || "minnesota-pokemon-v1".to_string(),
            |id| format!("minnesota-{}-v1", id.to_ascii_lowercase()),
        );
        ensure!(
            !base.report.manifests.contains(&manifest_id),
            "Minnesota species overlay already installed"
        );
        let mut pack = if base.data.type_categories.moves.is_empty() {
            crate::build_modern_move_split_modpack(base)?
        } else {
            base.clone()
        };
        crate::modern_move_split_modpack::verify_modern_categories(&pack.data)?;
        let tokens: BTreeMap<_, _> = self
            .roster
            .families
            .iter()
            .flat_map(|family| &family.members)
            .map(|member| (member.id.as_str(), member.name.to_ascii_uppercase()))
            .collect();
        let mut numbers: BTreeSet<_> = base
            .data
            .pokemon
            .values()
            .map(|species| species.int_id)
            .collect();
        for selected in self
            .roster
            .families
            .iter()
            .filter(|selected| family.is_none_or(|id| selected.id == id))
        {
            for member in &selected.members {
                let profile = &self.profiles[&member.id];
                let token = &tokens[member.id.as_str()];
                validate_base_references(base, profile)?;
                let number = runtime_number(member)?;
                ensure!(
                    numbers.insert(number),
                    "duplicate global Pokédex number {number}"
                );
                let species = PokemonSpecies {
                    id: token.clone(),
                    int_id: number,
                    base_stats: profile.base_stats,
                    type1: runtime_type(&member.types[0]).to_string(),
                    type2: runtime_type(member.types.get(1).unwrap_or(&member.types[0]))
                        .to_string(),
                    catch_rate: profile.catch_rate,
                    base_exp: profile.base_exp,
                    item1: profile.held_items[0].clone(),
                    item2: profile.held_items[1].clone(),
                    gender_ratio: profile.gender_ratio,
                    unknown1: 100,
                    step_cycles_to_hatch: profile.hatch_cycles,
                    unknown2: 5,
                    growth_rate: profile.growth_rate.clone(),
                    egg_group1: profile.egg_groups[0].clone(),
                    egg_group2: profile.egg_groups[1].clone(),
                    tmhm_learnset: profile.tmhm_moves.clone(),
                    ability: "NONE".to_string(),
                    pic_size: 0x77,
                    front_pic: 0,
                    back_pic: 0,
                    weight: profile.weight_tenths_lb,
                };
                species
                    .validate_compiled_shape()
                    .map_err(anyhow::Error::msg)?;
                crate::insert_keyed_pokemon_species(
                    &mut pack.data.pokemon,
                    token.clone(),
                    species,
                )?;
                crate::merge_learnsets(
                    &mut pack.data.learnsets,
                    BTreeMap::from([(
                        token.clone(),
                        profile
                            .level_moves
                            .iter()
                            .map(|(level, name)| LearnsetEntry(*level, name.clone()))
                            .collect(),
                    )]),
                )?;
                let entries = match (&profile.evolution, &member.evolves_to) {
                    (Some(evolution), Some(target)) => vec![EvolutionEntry {
                        method: evolution.method.clone(),
                        species: tokens[target.as_str()].clone(),
                        level: evolution.level,
                        item: evolution.item.clone(),
                        held_item: evolution.held_item.clone(),
                        happiness: evolution.happiness.clone(),
                        stat_ratio: evolution.stat_ratio.clone(),
                    }],
                    (None, None) => Vec::new(),
                    _ => anyhow::bail!("inconsistent evolution profile"),
                };
                crate::merge_evolution_table(
                    &mut pack.data.evolutions,
                    &EvolutionTable(BTreeMap::from([(token.clone(), entries)])),
                )?;
                ensure!(
                    pack.data
                        .egg_moves
                        .insert(token.clone(), serde_json::to_value(&profile.egg_moves)?)
                        .is_none(),
                    "duplicate egg-move entry"
                );
                crate::insert_keyed_pokedex_entry(
                    &mut pack.data.pokedex_entries,
                    token.clone(),
                    RuntimePokedexEntry {
                        species: token.clone(),
                        classification: profile.classification.clone(),
                        height_digits: profile.height_digits()?,
                        weight_digits: profile.weight_tenths_lb,
                        pages: profile.dex_pages.clone(),
                    },
                )?;
                crate::merge_menu_icons(
                    &mut pack.data.menu_icons,
                    BTreeMap::from([(token.clone(), profile.menu_icon.clone())]),
                )?;
                crate::merge_pokemon_cry_entries(
                    &mut pack.data.pokemon_cries,
                    BTreeMap::from([(token.clone(), profile.cry.clone())]),
                )?;
                let animation = static_animation();
                crate::merge_frontpic_anim_entries(
                    &mut pack.data.pokemon_frontpic_anim,
                    BTreeMap::from([
                        (token.clone(), animation.clone()),
                        (format!("{token}_IDLE"), animation),
                    ]),
                )?;
                pack.data
                    .capture_rules
                    .heavy_ball_modifiers
                    .insert(token.clone(), heavy_ball_modifier(profile.weight_tenths_lb));
                let source = format!(
                    "gfx/footprints/{}.png",
                    profile.footprint_species.to_ascii_lowercase()
                );
                let target = format!("gfx/footprints/{}.png", token.to_ascii_lowercase());
                ensure!(
                    pack.runtime_files
                        .insert(target.clone(), base.runtime_files[&source].clone())
                        .is_none(),
                    "duplicate footprint asset {target}"
                );
            }
            for (key, bytes) in &self.graphics[&selected.id] {
                ensure!(
                    !pack.runtime_files.contains_key(key),
                    "duplicate Minnesota runtime asset {key}"
                );
                pack.runtime_files.insert(key.clone(), bytes.clone());
            }
        }
        let species_count = pack.data.pokemon.len();
        let species_ids = pack.data.pokemon.keys().cloned().collect();
        let item_ids = pack.data.items.keys().cloned().collect();
        let evolution_issues = crystal_core::systems::evolution::evolution_table_issues(
            &pack.data.evolutions,
            &species_ids,
            &item_ids,
        );
        ensure!(
            evolution_issues.is_empty(),
            "invalid evolution closure: {evolution_issues:?}"
        );
        let last_rating = pack
            .data
            .oak_ratings
            .last_mut()
            .context("base lacks Oak rating table")?;
        last_rating.caught_count_limit = last_rating.caught_count_limit.max(species_count);
        pack.report.manifests.push(manifest_id);
        pack.report.pokemon = species_count;
        pack.identity = if pack.audio_compression.is_some() {
            crate::derive_compiled_game_pack_identity_from_manifest(
                pack.format_version,
                &pack.data,
                &pack.audio_manifest,
                &pack.runtime_files,
                &pack.report,
            )?
        } else {
            crate::derive_compiled_game_pack_identity(
                pack.format_version,
                &pack.data,
                &pack.compiled_audio,
                &pack.runtime_files,
                &pack.report,
            )?
        };
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
        Ok(pack)
    }
}

fn runtime_number(member: &Member) -> Result<u16> {
    member
        .id
        .strip_prefix("MN_")
        .context("invalid regional species prefix")?
        .parse::<u16>()?
        .checked_add(251)
        .context("global species number overflow")
}

fn runtime_type(token: &str) -> &str {
    if token == "PSYCHIC" {
        "PSYCHIC_TYPE"
    } else {
        token
    }
}

fn validate_base_references(base: &CompiledGamePack, profile: &Profile) -> Result<()> {
    let data = base.data();
    ensure!(
        data.growth_rates.contains_key(&profile.growth_rate),
        "unknown growth curve {}",
        profile.growth_rate
    );
    let egg_groups: BTreeSet<_> = data
        .pokemon
        .values()
        .flat_map(|species| [&species.egg_group1, &species.egg_group2])
        .collect();
    ensure!(
        profile
            .egg_groups
            .iter()
            .all(|group| egg_groups.contains(group)),
        "unknown egg group"
    );
    let tmhm: BTreeSet<_> = data
        .pokemon
        .values()
        .flat_map(|species| &species.tmhm_learnset)
        .collect();
    ensure!(
        profile.tmhm_moves.iter().all(|name| tmhm.contains(name)),
        "unknown TM/HM-compatible move"
    );
    for name in profile
        .level_moves
        .iter()
        .map(|(_, name)| name)
        .chain(&profile.tmhm_moves)
        .chain(&profile.egg_moves)
    {
        ensure!(
            data.moves.contains_key(name),
            "unknown move {name} in {}",
            profile.id
        );
    }
    ensure!(
        profile
            .level_moves
            .iter()
            .any(|(level, name)| *level == 1 && data.moves[name].power > 0),
        "{} needs a damaging level1 move",
        profile.id
    );
    ensure!(
        data.menu_icons
            .values()
            .any(|icon| *icon == profile.menu_icon),
        "unknown explicit menu icon {}",
        profile.menu_icon
    );
    let icon = profile
        .menu_icon
        .strip_prefix("ICON_")
        .context("menu icon must have ICON_ prefix")?
        .to_ascii_lowercase();
    ensure!(
        base.runtime_files
            .contains_key(&format!("gfx/icons/{icon}.png")),
        "menu icon PNG missing"
    );
    ensure!(
        data.audio
            .iter()
            .any(|asset| asset.kind == ModpackAudioKind::Cry && asset.id == profile.cry.cry),
        "unknown cry audio {}",
        profile.cry.cry
    );
    ensure!(
        data.pokemon.contains_key(&profile.footprint_species)
            && base.runtime_files.contains_key(&format!(
                "gfx/footprints/{}.png",
                profile.footprint_species.to_ascii_lowercase()
            )),
        "unknown footprint donor {}",
        profile.footprint_species
    );
    for item in profile.held_items.iter().flatten() {
        ensure!(data.items.contains_key(item), "unknown held item {item}");
    }
    if let Some(evolution) = &profile.evolution {
        for item in evolution.item.iter().chain(&evolution.held_item) {
            ensure!(
                (evolution.method == "TRADE"
                    && evolution.held_item.as_ref() == Some(item)
                    && item == crystal_core::systems::evolution::TRADE_ANY_ITEM)
                    || data.items.contains_key(item),
                "unknown evolution item {item}"
            );
        }
    }
    Ok(())
}

fn static_animation() -> FrontpicAnimProgram {
    FrontpicAnimProgram {
        commands: vec![
            FrontpicAnimCommand {
                kind: "frame".to_string(),
                frame: Some(0),
                duration: Some(8),
                ..FrontpicAnimCommand::default()
            },
            FrontpicAnimCommand {
                kind: "endanim".to_string(),
                ..FrontpicAnimCommand::default()
            },
        ],
    }
}

fn heavy_ball_modifier(weight: u16) -> i16 {
    let half = weight >> 1;
    let fraction = half >> 4;
    let high = (half.wrapping_sub(fraction).wrapping_sub(fraction >> 1) >> 8) as u8;
    match high {
        0..=3 => -20,
        4..=7 => 0,
        8..=11 => 20,
        12..=15 => 30,
        _ => 40,
    }
}

#[cfg(test)]
mod tests {
    use super::{heavy_ball_modifier, runtime_number, runtime_type};
    use crate::minnesota_pokemon_profiles::Member;

    #[test]
    fn regional_numbers_extend_beyond_byte_range_and_match_capture_weight_quirks() {
        let member = Member {
            id: "MN_266".to_string(),
            name: "Tipulord".to_string(),
            types: vec!["WATER".to_string()],
            evolves_to: None,
        };
        assert_eq!(runtime_number(&member).expect("valid regional ID"), 517);
        assert_eq!(runtime_type("PSYCHIC"), "PSYCHIC_TYPE");
        assert_eq!(heavy_ball_modifier(2257), -20);
        assert_eq!(heavy_ball_modifier(2258), 0);
        assert_eq!(heavy_ball_modifier(8820), 30);
        assert_eq!(heavy_ball_modifier(10140), 40);
    }
}

#[cfg(test)]
#[path = "minnesota_pokemon_modpack_tests.rs"]
mod catalog_tests;

#[cfg(test)]
#[path = "minnesota_pokemon_authored_tests.rs"]
mod authored_tests;
