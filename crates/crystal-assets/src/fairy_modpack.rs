//! Opt-in modern Fairy chart and earned attacks for the Northwoods mod.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::battle::damage::{MoveCategory, TypeEffectivenessEntry, TypeMultiplier};
use crystal_core::battle::turn::battle_move_effect_is_supported;
use crystal_core::models::Move;
use crystal_core::systems::learnsets::LearnsetEntry;
use serde::Deserialize;

use crate::CompiledGamePack;

/// Manifest identity of the opt-in eighteen-type ruleset.
pub const FAIRY_TYPES_MANIFEST_ID: &str = "northwoods-fairy-types-v1";
/// Manifest identity of five Fairy attacks and the physical Poison counter.
pub const FAIRY_MOVES_MANIFEST_ID: &str = "northwoods-fairy-moves-v1";

/// Install the modern Fairy chart and returning species' modern Fairy typings.
///
/// Steel also loses its old Dark/Ghost resistances. Charm, Sweet Kiss and
/// Moonlight become Fairy; their effects and every other native move stay intact.
/// Foresight exceptions, audio, graphics, stats and native learnsets are preserved.
///
/// # Errors
/// Returns an error for an invalid base, duplicate rules, missing native species
/// or moves, or a result which fails runtime verification.
pub fn build_fairy_type_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .data
            .type_categories
            .special
            .iter()
            .chain(&base.data.type_categories.physical)
            .any(|t| t == "FAIRY"),
        "Fairy types already installed"
    );
    let mut pack = base.clone();
    pack.data.type_categories.special.push("FAIRY".to_string());
    let chart = &mut pack.data.type_effectiveness.matchups;
    chart.retain(|row| {
        !(row.defender == "STEEL" && matches!(row.attacker.as_str(), "DARK" | "GHOST"))
    });
    for (attacker, defender, numerator, denominator) in [
        ("FAIRY", "FIGHTING", 2, 1),
        ("FAIRY", "DRAGON", 2, 1),
        ("FAIRY", "DARK", 2, 1),
        ("FAIRY", "FIRE", 1, 2),
        ("FAIRY", "POISON", 1, 2),
        ("FAIRY", "STEEL", 1, 2),
        ("BUG", "FAIRY", 1, 2),
        ("DARK", "FAIRY", 1, 2),
        ("FIGHTING", "FAIRY", 1, 2),
        ("DRAGON", "FAIRY", 0, 1),
        ("POISON", "FAIRY", 2, 1),
        ("STEEL", "FAIRY", 2, 1),
    ] {
        chart.push(TypeEffectivenessEntry {
            attacker: attacker.to_string(),
            defender: defender.to_string(),
            multiplier: TypeMultiplier {
                numerator,
                denominator,
            },
        });
    }
    for (number, first, second) in [
        (35, "FAIRY", "FAIRY"),
        (36, "FAIRY", "FAIRY"),
        (173, "FAIRY", "FAIRY"),
        (39, "NORMAL", "FAIRY"),
        (40, "NORMAL", "FAIRY"),
        (174, "NORMAL", "FAIRY"),
        (175, "FAIRY", "FAIRY"),
        (176, "FAIRY", "FLYING"),
        (183, "WATER", "FAIRY"),
        (184, "WATER", "FAIRY"),
        (209, "FAIRY", "FAIRY"),
        (210, "FAIRY", "FAIRY"),
        (122, "PSYCHIC_TYPE", "FAIRY"),
    ] {
        let species = pack
            .data
            .pokemon
            .values_mut()
            .find(|s| s.int_id == number)
            .with_context(|| format!("missing returning species {number}"))?;
        species.type1 = first.to_string();
        species.type2 = second.to_string();
    }
    for id in ["CHARM", "SWEET_KISS", "MOONLIGHT"] {
        pack.data
            .moves
            .get_mut(id)
            .with_context(|| format!("missing native move {id}"))?
            .move_type = "FAIRY".to_string();
    }
    finish(pack, FAIRY_TYPES_MANIFEST_ID)
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u16,
    id: String,
    moves: Vec<Definition>,
}
#[derive(Deserialize)]
struct Definition {
    id: String,
    display_name: String,
    category: MoveCategory,
    animation_donor: String,
    battle: Move,
    learners: Vec<Learner>,
}
#[derive(Deserialize)]
struct Learner {
    species_id: String,
    runtime_name: String,
    national_number: Option<u16>,
    level: u8,
}

#[cfg(test)]
fn catalog() -> Result<Catalog> {
    catalog_from_json(include_str!(
        "../../../planning/minnesota/pokemon/fairy-moves.json"
    ))
}

fn catalog_from_json(json: &str) -> Result<Catalog> {
    let catalog: Catalog = serde_json::from_str(json)?;
    ensure!(
        catalog.schema_version == 1
            && catalog.id == FAIRY_MOVES_MANIFEST_ID
            && catalog.moves.len() == 6,
        "invalid Fairy move catalog"
    );
    let mut ids = BTreeSet::new();
    for (offset, definition) in catalog.moves.iter().enumerate() {
        ensure!(
            ids.insert(&definition.id)
                && definition.id == definition.battle.name
                && usize::from(definition.battle.source_index) == 265 + offset,
            "invalid Fairy move identity/index"
        );
        ensure!(
            battle_move_effect_is_supported(&definition.battle.effect),
            "unsupported Fairy move effect {} for {}",
            definition.battle.effect,
            definition.id
        );
        ensure!(
            !definition.display_name.trim().is_empty(),
            "missing move display name"
        );
        for learner in &definition.learners {
            ensure!(
                (1..=100).contains(&learner.level)
                    && (learner.species_id.starts_with("MN_") || learner.national_number.is_some()),
                "invalid Fairy learner"
            );
        }
    }
    Ok(catalog)
}

/// Add earned Fairy attacks and Poison Jab to a completed Northwoods move pack.
///
/// Existing moves, programs and learnsets are retained; new moves reuse native
/// presentation programs and execute their own declared damage/effects. Retired
/// regional species are skipped. Returning learners resolve by national number.
///
/// # Errors
/// Returns an error for a missing prerequisite, malformed catalog, duplicate
/// extension, missing donor/learnset or a result which fails verification.
pub fn build_fairy_moves_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    build_fairy_moves_modpack_from_json(
        base,
        include_str!("../../../planning/minnesota/pokemon/fairy-moves.json"),
    )
}

/// Install a validated authored move catalog without rebuilding the exporter.
///
/// This has the same prerequisites and preservation guarantees as
/// [`build_fairy_moves_modpack`], with the catalog supplied by the caller.
///
/// # Errors
/// Returns an error for malformed or incompatible catalog data, a missing
/// prerequisite or donor, duplicate content, or runtime verification failure.
pub fn build_fairy_moves_modpack_from_json(
    base: &CompiledGamePack,
    json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|id| id == FAIRY_TYPES_MANIFEST_ID),
        "install Fairy type rules first"
    );
    ensure!(
        base.data.moves.len() == 264
            && base.data.move_names.len() == 264
            && base.data.battle_animation_table.len() == 265,
        "Fairy attacks require the completed thirteen-move extension"
    );
    crate::modern_move_split_modpack::verify_modern_categories(&base.data)?;
    let mut pack = base.clone();
    for definition in catalog_from_json(json)?.moves {
        ensure!(
            !pack.data.moves.contains_key(&definition.id),
            "duplicate Fairy move"
        );
        let donor = pack
            .data
            .moves
            .get(&definition.animation_donor)
            .context("missing native animation donor")?;
        let animation = pack
            .data
            .battle_animation_table
            .get(usize::from(donor.source_index))
            .context("missing animation table slot")?
            .clone();
        // Moonblast introduces a secondary effect absent from the native catalog.
        // Its priority is ordinary; retain all existing effect overrides.
        pack.data
            .move_priorities
            .effect_priorities
            .entry(definition.battle.effect.clone())
            .or_insert(pack.data.move_priorities.base_priority);
        pack.data.battle_animation_table.push(animation);
        pack.data.move_names.push(definition.display_name);
        pack.data
            .type_categories
            .moves
            .insert(definition.id.clone(), definition.category);
        pack.data
            .moves
            .insert(definition.id.clone(), definition.battle);
        for learner in definition.learners {
            let key = if let Some(number) = learner.national_number {
                pack.data
                    .pokemon
                    .iter()
                    .find(|(_, s)| s.int_id == number)
                    .map(|(key, _)| key.clone())
                    .with_context(|| format!("missing returning learner {number}"))?
            } else {
                if !pack.data.pokemon.contains_key(&learner.runtime_name) {
                    continue;
                }
                learner.runtime_name
            };
            let learnset = pack
                .data
                .learnsets
                .get_mut(&key)
                .context("missing learner learnset")?;
            ensure!(
                !learnset.iter().any(|entry| entry.1 == definition.id),
                "duplicate learned move"
            );
            learnset.push(LearnsetEntry(learner.level, definition.id.clone()));
            learnset.sort_by_key(|entry| entry.0);
        }
    }
    pack.report.moves = pack.data.moves.len();
    finish(pack, FAIRY_MOVES_MANIFEST_ID)
}

fn finish(mut pack: CompiledGamePack, manifest: &str) -> Result<CompiledGamePack> {
    ensure!(
        !pack.report.manifests.iter().any(|id| id == manifest),
        "Fairy overlay already installed"
    );
    pack.report.manifests.push(manifest.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_fairy_attacks_have_dense_supported_effects_and_physical_counter() {
        let catalog = catalog().expect("authored catalog");
        assert_eq!(
            catalog
                .moves
                .iter()
                .filter(|m| m.battle.move_type == "FAIRY")
                .count(),
            5
        );
        assert_eq!(catalog.moves[5].category, MoveCategory::Physical);
        assert_eq!(catalog.moves[5].battle.move_type, "POISON");
    }

    #[test]
    #[ignore = "requires external native pack"]
    fn fairy_rules_preserve_unaffected_native_content() {
        let path = std::env::var_os("CRYSTAL_FAIRY_BASE_PACK").expect("native base path");
        let base = crate::read_verified_compiled_game_pack(path).expect("base");
        let result = build_fairy_type_modpack(&base).expect("Fairy overlay");
        assert_eq!(base.runtime_files, result.runtime_files);
        assert_eq!(base.audio_manifest, result.audio_manifest);
        assert_eq!(base.compiled_audio, result.compiled_audio);
        assert_eq!(base.data.learnsets, result.data.learnsets);
        assert_eq!(
            base.data.type_effectiveness.foresight_matchups,
            result.data.type_effectiveness.foresight_matchups
        );
        assert_eq!(
            result
                .data
                .type_effectiveness
                .matchups
                .iter()
                .filter(|e| e.attacker == "FAIRY" || e.defender == "FAIRY")
                .count(),
            12
        );
        assert_eq!(
            result
                .data
                .pokemon
                .values()
                .filter(|s| s.type1 == "FAIRY" || s.type2 == "FAIRY")
                .count(),
            13
        );
        let changed_types: BTreeSet<_> = base
            .data
            .pokemon
            .iter()
            .filter(|(id, old)| {
                let new = &result.data.pokemon[*id];
                old.type1 != new.type1 || old.type2 != new.type2
            })
            .map(|(_, old)| old.int_id)
            .collect();
        assert_eq!(
            changed_types,
            [35, 36, 39, 40, 122, 173, 174, 175, 176, 183, 184, 209, 210]
                .into_iter()
                .collect()
        );
        let retained_chart: Vec<_> = result
            .data
            .type_effectiveness
            .matchups
            .iter()
            .filter(|row| row.attacker != "FAIRY" && row.defender != "FAIRY")
            .cloned()
            .collect();
        let expected_chart: Vec<_> = base
            .data
            .type_effectiveness
            .matchups
            .iter()
            .filter(|row| {
                !(row.defender == "STEEL" && matches!(row.attacker.as_str(), "DARK" | "GHOST"))
            })
            .cloned()
            .collect();
        assert_eq!(retained_chart, expected_chart);
        for (id, old) in &base.data.moves {
            let mut expected = old.clone();
            if matches!(id.as_str(), "CHARM" | "SWEET_KISS" | "MOONLIGHT") {
                expected.move_type = "FAIRY".to_string();
            }
            assert_eq!(&result.data.moves[id], &expected);
        }
        for (id, old) in &base.data.pokemon {
            let mut expected = old.clone();
            expected.type1 = result.data.pokemon[id].type1.clone();
            expected.type2 = result.data.pokemon[id].type2.clone();
            assert_eq!(
                &result.data.pokemon[id], &expected,
                "native stats/data {id}"
            );
        }
        assert!(build_fairy_type_modpack(&result).is_err());
    }
    #[test]
    #[ignore = "requires completed Northwoods pack with Fairy type rules"]
    fn fairy_moves_preserve_existing_content_and_install_earned_counterplay() {
        let path = std::env::var_os("CRYSTAL_FAIRY_MOVES_BASE_PACK").expect("move base path");
        let base = crate::read_verified_compiled_game_pack(path).expect("base");
        let result = build_fairy_moves_modpack(&base).expect("Fairy moves");
        assert_eq!(result.data.moves.len(), 270);
        assert_eq!(base.runtime_files, result.runtime_files);
        assert_eq!(base.audio_manifest, result.audio_manifest);
        assert_eq!(base.compiled_audio, result.compiled_audio);
        assert_eq!(base.data.pokemon, result.data.pokemon);
        assert_eq!(base.data.type_effectiveness, result.data.type_effectiveness);
        for (id, mov) in &base.data.moves {
            assert_eq!(&result.data.moves[id], mov);
        }
        assert_eq!(&result.data.move_names[..264], &base.data.move_names[..]);
        assert_eq!(
            &result.data.battle_animation_table[..265],
            &base.data.battle_animation_table[..]
        );
        let catalog = catalog().expect("catalog");
        let ids: BTreeSet<_> = catalog.moves.iter().map(|m| m.id.as_str()).collect();
        for (id, entries) in &result.data.learnsets {
            let retained: Vec<_> = entries
                .iter()
                .filter(|entry| !ids.contains(entry.1.as_str()))
                .cloned()
                .collect();
            assert_eq!(
                &retained, &base.data.learnsets[id],
                "original learnset {id}"
            );
        }
        for definition in catalog.moves {
            let donor = base.data.moves[&definition.animation_donor].source_index;
            assert_eq!(
                result.data.battle_animation_table[usize::from(definition.battle.source_index)],
                base.data.battle_animation_table[usize::from(donor)]
            );
            for learner in definition.learners {
                let key = if let Some(number) = learner.national_number {
                    result
                        .data
                        .pokemon
                        .iter()
                        .find(|(_, s)| s.int_id == number)
                        .map(|(id, _)| id.as_str())
                        .expect("native learner")
                } else {
                    learner.runtime_name.as_str()
                };
                if let Some(entries) = result.data.learnsets.get(key) {
                    assert!(
                        entries
                            .iter()
                            .any(|entry| entry.0 == learner.level && entry.1 == definition.id)
                    );
                }
            }
        }
        assert!(build_fairy_moves_modpack(&result).is_err());
    }
}
