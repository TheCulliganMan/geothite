//! Append the two final-starter attacks without rebuilding existing game content.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::battle::damage::MoveCategory;
use crystal_core::battle::turn::{battle_move_effect_is_supported, move_priority};
use crystal_core::models::Move;
use crystal_core::systems::learnsets::LearnsetEntry;
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::starter_signature_animations;

/// Identity of the two-move supplement after the completed 270-move catalog.
pub const NORTHWOODS_STARTER_SIGNATURES_MANIFEST_ID: &str = "northwoods-starter-signatures-v1";

const CATALOG: &str =
    include_str!("../../../planning/minnesota/pokemon/starter-signature-moves.json");
const GRAPHICS_PREFIX: &str = "gfx/battle_anims/northwoods_starter_";
const NEW_POSES: usize = 8;

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
    level: u8,
}

fn catalog_from_json(json: &str) -> Result<Catalog> {
    let catalog: Catalog =
        serde_json::from_str(json).context("decode starter signature catalog")?;
    ensure!(
        catalog.schema_version == 1
            && catalog.id == NORTHWOODS_STARTER_SIGNATURES_MANIFEST_ID
            && catalog.moves.len() == 2,
        "starter supplement requires exactly two moves and its declared manifest identity"
    );
    for (offset, definition) in catalog.moves.iter().enumerate() {
        let (id, species, learner) = match offset {
            0 => ("RIMEBRAND", "MN_276", "BOREALYNX"),
            _ => ("PADDLE_SWEEP", "MN_012", "VOYAGEAR"),
        };
        let battle = &definition.battle;
        ensure!(
            definition.id == id
                && battle.name == id
                && usize::from(battle.source_index) == 271 + offset,
            "starter signature identity/index must be {id} at {}",
            271 + offset
        );
        ensure!(
            matches!(
                definition.category,
                MoveCategory::Physical | MoveCategory::Special
            ) && (1..=255).contains(&battle.power)
                && (1..=100).contains(&battle.accuracy)
                && (1..=40).contains(&battle.pp)
                && battle.effect_chance <= 100
                && battle_move_effect_is_supported(&battle.effect),
            "invalid or unsupported starter signature battle fields for {id}"
        );
        ensure!(
            !definition.display_name.is_empty()
                && definition.display_name.trim() == definition.display_name,
            "starter signature display name is empty or padded"
        );
        ensure!(
            definition.learners.len() == 1
                && definition.learners[0].species_id == species
                && definition.learners[0].runtime_name == learner
                && definition.learners[0].level == 36,
            "{id} must be exclusive to {learner} at level36"
        );
    }
    Ok(catalog)
}

fn validate_base(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == NORTHWOODS_STARTER_SIGNATURES_MANIFEST_ID),
        "starter signature supplement already installed"
    );
    ensure!(
        base.data.moves.len() == 270
            && base.data.move_names.len() == 270
            && base.data.battle_animation_table.len() == 271
            && base
                .data
                .moves
                .values()
                .map(|mov| mov.source_index)
                .collect::<BTreeSet<_>>()
                == (1..=270).collect(),
        "starter supplement requires dense existing move indices1..270 and complete presentation tables"
    );
    crate::modern_move_split_modpack::verify_modern_categories(&base.data)?;
    ensure!(
        base.data.moves.contains_key("SLIPPER_WALTZ")
            && base.data.learnsets.get("CYPRIELLE").is_some_and(|rows| {
                rows.iter()
                    .filter(|row| row.0 == 36 && row.1 == "SLIPPER_WALTZ")
                    .count()
                    == 1
            }),
        "Cyprielle must already retain its level36 Slipper Waltz"
    );
    Ok(())
}

/// Add Rimebrand and Paddle Sweep with exclusive level36 final-starter learnsets.
///
/// All existing moves, species, maps, rules, audio, artwork, learnsets and native
/// animation programs are retained. Only two move slots and eight small attack
/// pose banks are appended. Cyprielle's existing Slipper Waltz remains unchanged.
///
/// # Errors
/// Returns an error for an invalid/incomplete base, duplicate installation,
/// missing final starter/donor, incompatible authoring or preservation failure.
pub fn build_northwoods_starter_signatures_modpack(
    base: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    build_northwoods_starter_signatures_modpack_from_json(base, CATALOG)
}

/// Apply the same focused supplement from caller-supplied authoring JSON.
///
/// # Errors
/// Returns the same errors as [`build_northwoods_starter_signatures_modpack`],
/// including rejection of unsupported mechanics, indices or non-final learners.
pub fn build_northwoods_starter_signatures_modpack_from_json(
    base: &CompiledGamePack,
    json: &str,
) -> Result<CompiledGamePack> {
    let catalog = catalog_from_json(json)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    validate_base(base)?;
    let mut pack = base.clone();
    for definition in &catalog.moves {
        ensure!(
            !pack.data.moves.contains_key(&definition.id),
            "duplicate starter move"
        );
        ensure!(
            base.data
                .type_categories
                .physical
                .iter()
                .chain(&base.data.type_categories.special)
                .any(|kind| kind == &definition.battle.move_type)
                && base.data.moves.values().any(|existing| {
                    existing.effect == definition.battle.effect
                        && existing.stat == definition.battle.stat
                        && existing.amount == definition.battle.amount
                }),
            "starter signatures must reuse existing types and native effect/stat mechanics"
        );
        let donor = pack
            .data
            .moves
            .get(&definition.animation_donor)
            .with_context(|| format!("missing donor {}", definition.animation_donor))?;
        let animation = pack
            .data
            .battle_animation_table
            .get(usize::from(donor.source_index))
            .context("missing donor presentation slot")?
            .clone();
        let priority = move_priority(&definition.battle, &pack.data.move_priorities)
            .map_err(|error| anyhow::anyhow!("starter priority {}: {error:?}", definition.id))?;
        ensure!(
            priority == pack.data.move_priorities.base_priority,
            "starter attack priority changed"
        );
        pack.data
            .moves
            .insert(definition.id.clone(), definition.battle.clone());
        pack.data
            .type_categories
            .moves
            .insert(definition.id.clone(), definition.category);
        pack.data.move_names.push(definition.display_name.clone());
        pack.data.battle_animation_table.push(animation);
        let learner = &definition.learners[0];
        let species = pack
            .data
            .pokemon
            .get(&learner.runtime_name)
            .with_context(|| format!("missing final starter {}", learner.runtime_name))?;
        ensure!(
            species.int_id > 251,
            "starter learner must be a regional final evolution"
        );
        let rows = pack
            .data
            .learnsets
            .get_mut(&learner.runtime_name)
            .context("missing starter learnset")?;
        ensure!(
            !rows.iter().any(|row| row.1 == definition.id),
            "duplicate learned signature"
        );
        let position = rows
            .iter()
            .position(|row| row.0 > learner.level)
            .unwrap_or(rows.len());
        rows.insert(
            position,
            LearnsetEntry(learner.level, definition.id.clone()),
        );
    }
    starter_signature_animations::install_starter_signature_animations(
        &mut pack.data,
        &mut pack.runtime_files,
    )?;
    pack.report.moves = pack.data.moves.len();
    pack.report
        .manifests
        .push(NORTHWOODS_STARTER_SIGNATURES_MANIFEST_ID.to_string());
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
    verify_preservation(base, &pack, &catalog)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

/// Verify that an installed supplement changes only the declared starter attacks.
///
/// This also checks every original runtime file byte, complete audio storage and
/// metadata, existing animation bundle entries, and the restored whole dataset.
///
/// # Errors
/// Returns an error for unexpected moves, learners, programs, graphics, report
/// changes or any alteration to previously present game content.
pub fn verify_northwoods_starter_signatures_preservation(
    base: &CompiledGamePack,
    installed: &CompiledGamePack,
) -> Result<()> {
    validate_base(base)?;
    verify_preservation(base, installed, &catalog_from_json(CATALOG)?)
}

fn verify_preservation(
    base: &CompiledGamePack,
    installed: &CompiledGamePack,
    catalog: &Catalog,
) -> Result<()> {
    ensure!(
        installed.format_version == base.format_version
            && installed.compiled_audio == base.compiled_audio
            && installed.audio_manifest == base.audio_manifest
            && installed.audio_compression == base.audio_compression,
        "starter supplement changed existing audio or storage format"
    );
    ensure!(
        installed.runtime_files.len() == base.runtime_files.len() + NEW_POSES,
        "unexpected starter graphics count"
    );
    for (path, bytes) in &base.runtime_files {
        ensure!(
            installed.runtime_files.get(path) == Some(bytes),
            "existing runtime file changed: {path}"
        );
    }
    for (path, bytes) in &installed.runtime_files {
        if !base.runtime_files.contains_key(path) {
            ensure!(
                path.starts_with(GRAPHICS_PREFIX) && path.ends_with(".2bpp") && bytes.len() == 96,
                "unexpected added runtime file: {path}"
            );
        }
    }
    let mut report = installed.report.clone();
    ensure!(
        report.moves == base.report.moves + 2,
        "incorrect move report count"
    );
    report.moves = base.report.moves;
    ensure!(
        report.manifests.pop().as_deref() == Some(NORTHWOODS_STARTER_SIGNATURES_MANIFEST_ID),
        "missing starter report identity"
    );
    ensure!(
        report == base.report,
        "starter supplement changed unrelated compile report data"
    );
    let mut restored = installed.data.clone();
    ensure!(
        restored.moves.len() == base.data.moves.len() + 2
            && restored.move_names.len() == base.data.move_names.len() + 2
            && restored.battle_animation_table.len() == base.data.battle_animation_table.len() + 2
            && restored.battle_animations.len() == base.data.battle_animations.len() + 2,
        "starter supplement added unexpected move or program records"
    );
    for (offset, definition) in catalog.moves.iter().enumerate() {
        ensure!(
            restored.moves.remove(&definition.id).as_ref() == Some(&definition.battle),
            "installed starter move facts differ"
        );
        ensure!(
            restored.type_categories.moves.remove(&definition.id) == Some(definition.category),
            "installed starter category differs"
        );
        ensure!(
            restored.move_names[base.data.move_names.len() + offset] == definition.display_name,
            "installed starter display name differs"
        );
        let label = starter_signature_animations::animation_label(&definition.id);
        ensure!(
            restored.battle_animation_table[base.data.battle_animation_table.len() + offset]
                == label,
            "starter animation slot differs"
        );
        let (_, commands) = starter_signature_animations::programs()
            .into_iter()
            .find(|(id, _)| *id == definition.id)
            .context("missing authored starter program")?;
        ensure!(
            restored.battle_animations.remove(&label) == Some(commands),
            "installed starter program differs"
        );
        let learner = &definition.learners[0];
        let mut found = 0;
        for (name, rows) in &mut restored.learnsets {
            for row in rows.iter().filter(|row| row.1 == definition.id) {
                ensure!(
                    name == &learner.runtime_name && row.0 == learner.level,
                    "starter move has an unauthorized learner or level"
                );
                found += 1;
            }
            rows.retain(|row| row.1 != definition.id);
        }
        ensure!(
            found == 1,
            "starter signature must have exactly one learned row"
        );
    }
    restored.move_names.truncate(base.data.move_names.len());
    restored
        .battle_animation_table
        .truncate(base.data.battle_animation_table.len());
    verify_animation_bundle(&base.data.battle_anim_bundle, &restored.battle_anim_bundle)?;
    restored
        .battle_anim_bundle
        .clone_from(&base.data.battle_anim_bundle);
    ensure!(
        restored == base.data,
        "starter supplement changed unrelated game data"
    );
    Ok(())
}

fn verify_animation_bundle(base: &str, installed: &str) -> Result<()> {
    let original: Value = serde_json::from_str(base).context("decode original animation bundle")?;
    let mut restored: Value =
        serde_json::from_str(installed).context("decode starter animation bundle")?;
    for (section, prefix) in [
        ("objects", "BATTLE_ANIM_OBJ_MN_STARTER_"),
        ("gfx_table", "BATTLE_ANIM_GFX_MN_STARTER_"),
        ("gfx_sources", "NorthwoodsStarter"),
    ] {
        let before = original[section]
            .as_object()
            .context("original animation section is not an object")?;
        let after = restored[section]
            .as_object_mut()
            .context("starter animation section is not an object")?;
        let added: Vec<_> = after
            .keys()
            .filter(|key| !before.contains_key(*key))
            .cloned()
            .collect();
        ensure!(
            added.len() == NEW_POSES && added.iter().all(|key| key.starts_with(prefix)),
            "unexpected starter animation bundle additions in {section}"
        );
        for key in added {
            after.remove(&key);
        }
    }
    ensure!(
        restored == original,
        "starter supplement changed an existing animation bundle entry"
    );
    Ok(())
}
