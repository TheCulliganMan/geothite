//! Later-generation attacks and original Northwoods signatures, with real effects.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::battle::damage::MoveCategory;
use crystal_core::battle::turn::{battle_move_effect_is_supported, move_priority};
use crystal_core::models::Move;
use crystal_core::systems::learnsets::LearnsetEntry;
use serde::Deserialize;

use crate::{CompiledGamePack, GameDataSet};

/// Stable manifest identity for the authored thirteen-move extension.
pub const NORTHWOODS_MOVES_MANIFEST_ID: &str = "northwoods-moves-v1";

#[derive(Deserialize)]
struct Catalog {
    schema_version: u16,
    id: String,
    moves: Vec<MoveDefinition>,
}

#[derive(Deserialize)]
struct MoveDefinition {
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
    level: u8,
}

fn catalog() -> Result<Catalog> {
    let catalog: Catalog = serde_json::from_str(include_str!(
        "../../../planning/minnesota/pokemon/northwoods-moves.json"
    ))?;
    ensure!(
        catalog.schema_version == 1 && catalog.id == NORTHWOODS_MOVES_MANIFEST_ID,
        "unsupported Northwoods move catalog"
    );
    let mut ids = BTreeSet::new();
    for (offset, definition) in catalog.moves.iter().enumerate() {
        ensure!(ids.insert(&definition.id), "duplicate Northwoods move");
        ensure!(
            definition.id == definition.battle.name
                && usize::from(definition.battle.source_index) == offset + 252,
            "Northwoods move IDs must form a dense extension after the native catalog"
        );
        ensure!(
            battle_move_effect_is_supported(&definition.battle.effect),
            "unsupported battle effect for {}",
            definition.id
        );
        ensure!(
            !definition.display_name.is_empty()
                && definition.display_name.trim() == definition.display_name,
            "invalid move display name"
        );
        for learner in &definition.learners {
            ensure!(
                learner.species_id.starts_with("MN_")
                    && (1..=100).contains(&learner.level)
                    && learner
                        .runtime_name
                        .bytes()
                        .all(|byte| byte.is_ascii_uppercase()),
                "invalid authored learner for {}",
                definition.id
            );
        }
    }
    ensure!(
        catalog.moves.len() == 13,
        "Northwoods catalog must contain 13 moves"
    );
    Ok(catalog)
}

/// Extend a verified pack with eight imported attacks and five regional signatures.
///
/// Native move indices, data and animation programs are preserved. The new attacks
/// receive dense indices 252–264 and explicit categories; native Metronome retains
/// its byte-based 1–251 selection. Authored learners are added only for species
/// already present in `base`; no removed species are restored. Audio is unchanged.
/// The modern native move split is installed when missing. Incompatible existing
/// classifications are rejected so authored physical/special roles stay valid.
///
/// Returns an error for an invalid base, an already installed extension, a missing
/// native donor, inconsistent tables, unsupported mechanics or an invalid result.
pub fn build_northwoods_moves_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == NORTHWOODS_MOVES_MANIFEST_ID),
        "Northwoods move extension already installed"
    );
    let catalog = catalog()?;
    let mut pack = if base.data.type_categories.moves.is_empty() {
        crate::build_modern_move_split_modpack(base)?
    } else {
        base.clone()
    };
    crate::modern_move_split_modpack::verify_modern_categories(&pack.data)?;
    install_move_data(&mut pack.data, &catalog)?;
    crate::minnesota_signature_animations::install_northwoods_signature_animations(
        &mut pack.data,
        &mut pack.runtime_files,
    )?;
    pack.report.moves = pack.data.moves.len();
    pack.report
        .manifests
        .push(NORTHWOODS_MOVES_MANIFEST_ID.to_string());
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

fn install_move_data(data: &mut GameDataSet, catalog: &Catalog) -> Result<()> {
    ensure!(
        data.moves.len() == 251
            && data.move_names.len() == 251
            && data.battle_animation_table.len() == 252,
        "Northwoods extensions require complete native move/name/animation tables"
    );
    ensure!(
        data.moves
            .values()
            .map(|mov| mov.source_index)
            .collect::<BTreeSet<_>>()
            == (1..=251).collect(),
        "native move indices must remain exactly 1–251"
    );
    crate::modern_move_split_modpack::verify_modern_categories(data)?;
    for definition in &catalog.moves {
        ensure!(
            !data.moves.contains_key(&definition.id),
            "duplicate move {}",
            definition.id
        );
        let donor = data
            .moves
            .get(&definition.animation_donor)
            .with_context(|| format!("missing animation donor {}", definition.animation_donor))?;
        let label = data
            .battle_animation_table
            .get(usize::from(donor.source_index))
            .context("native animation donor is outside the presentation table")?
            .clone();
        let priority = move_priority(&definition.battle, &data.move_priorities)
            .map_err(|error| anyhow::anyhow!("move priority {}: {error:?}", definition.id))?;
        let expected_priority = data.move_priorities.base_priority
            + i8::from(definition.battle.effect == "PRIORITY_HIT");
        ensure!(
            priority == expected_priority,
            "unexpected priority for {}",
            definition.id
        );
        data.move_names.push(definition.display_name.clone());
        data.battle_animation_table.push(label);
        data.type_categories
            .moves
            .insert(definition.id.clone(), definition.category);
        data.moves
            .insert(definition.id.clone(), definition.battle.clone());
        for learner in &definition.learners {
            if !data.pokemon.contains_key(&learner.runtime_name) {
                continue;
            }
            let learnset = data
                .learnsets
                .get_mut(&learner.runtime_name)
                .with_context(|| format!("missing learnset for {}", learner.runtime_name))?;
            ensure!(
                !learnset.iter().any(|entry| entry.1 == definition.id),
                "duplicate authored move for {}",
                learner.runtime_name
            );
            learnset.push(LearnsetEntry(learner.level, definition.id.clone()));
            learnset.sort_by_key(|entry| entry.0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn northwoods_catalog_has_unique_supported_effects_and_dense_presentation_indices() {
        let catalog = catalog().expect("authored move catalog");
        assert_eq!(catalog.moves.len(), 13);
        assert_eq!(catalog.moves[0].battle.source_index, 252);
        assert_eq!(catalog.moves[12].battle.source_index, 264);
        for definition in &catalog.moves {
            if definition.battle.effect.ends_with("DOWN_HIT") {
                assert!(definition.battle.stat.is_some());
                assert_eq!(definition.battle.amount, Some(-1));
            }
        }
    }

    #[test]
    #[ignore = "requires the completed external Minnesota pack"]
    fn northwoods_extension_preserves_native_moves_and_installs_real_learnsets() {
        let path = std::env::var_os("CRYSTAL_NORTHWOODS_TEST_PACK")
            .expect("set CRYSTAL_NORTHWOODS_TEST_PACK to the completed Minnesota pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified base");
        let extended = build_northwoods_moves_modpack(&base).expect("verified extension");
        assert_eq!(extended.data.moves.len(), 264);
        assert_eq!(extended.data.move_names.len(), 264);
        assert_eq!(extended.data.battle_animation_table.len(), 265);
        for (id, mov) in &base.data.moves {
            assert_eq!(&extended.data.moves[id], mov, "native move {id}");
        }
        assert_eq!(extended.compiled_audio, base.compiled_audio);
        assert_eq!(extended.audio_manifest, base.audio_manifest);
        assert_eq!(extended.data.pokemon, base.data.pokemon);
        for (id, species) in &base.data.pokemon {
            if species.int_id <= 251 {
                assert_eq!(
                    extended.data.learnsets.get(id),
                    base.data.learnsets.get(id),
                    "returning learnset {id}"
                );
            }
        }
        assert_eq!(extended.data.maps, base.data.maps);
        assert_eq!(extended.data.move_names[..251], base.data.move_names);
        assert_eq!(
            extended.data.battle_animation_table[..252],
            base.data.battle_animation_table
        );
        for (path, bytes) in &base.runtime_files {
            assert_eq!(&extended.runtime_files[path], bytes, "native asset {path}");
        }
        for (id, category) in &base.data.type_categories.moves {
            assert_eq!(
                &extended.data.type_categories.moves[id], category,
                "native category {id}"
            );
        }
        for definition in &catalog().expect("catalog").moves {
            for learner in &definition.learners {
                if base.data.pokemon.contains_key(&learner.runtime_name) {
                    assert!(
                        extended.data.learnsets[&learner.runtime_name]
                            .iter()
                            .any(|entry| entry.0 == learner.level && entry.1 == definition.id)
                    );
                }
            }
        }
    }
}
