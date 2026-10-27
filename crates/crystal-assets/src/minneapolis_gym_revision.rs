//! Import the reviewed Gym3 into the northern composition before city binding.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::{CompiledGamePack, MapModule, minneapolis_gym};

const MAP: &str = "MnMinneapolisGym";
const CONSTANT: &str = "MN_MINNEAPOLIS_GYM";
const TILESET: &str = "mn_gym_3";
const MANIFEST: &str = "minnesota-reviewed-gym3-replacement-v1";
const OLD_CONTENT: &str = "158fb28276d1d82de8525964e9f14578db667b716a9c9e1dfa31bdda06ffa104";
const NEW_CONTENT: &str = "895908cb2cc75dd4b716aa1d51f5da288a41adc76e26bc11c69440e9063b67f9";
const DONOR_CONTENT: &str = "7498f5589b974e91b24333a0d89bbe7938894b12400ee8662f0a15117c0a4ede";
const TRAINERS: [&str; 3] = ["MN_GYM_3", "MN_GYM_3_TRAINER_1", "MN_GYM_3_TRAINER_2"];
const FILES: [&str; 5] = [
    "data/tilesets/mn_gym_3.json",
    "data/tilesets/mn_gym_3_metatiles.bin",
    "data/tilesets/mn_gym_3_palette_map.json",
    "gfx/tilesets/mn_gym_3.2bpp",
    "gfx/tilesets/mn_gym_3.png",
];

/// Replace the old installed Gym3 with the captured canonical switch-pair room.
///
/// This explicit transition precedes Minneapolis city installation. It imports
/// only the pinned gym module, its tileset/five art files and three trainers.
/// Unrelated trainer numbers, existing event numbers, group/map identity and
/// review-hall destinations remain stable; new puzzle events receive unused
/// numbers. Obsolete puzzle flags are retained for save compatibility, but no
/// saved player state is migrated or awarded. Unrelated maps, scripts, art,
/// audio, HOME/recovery and story catalogs remain unchanged.
///
/// # Errors
/// Returns an error for an invalid pack, an unsupported old/donor generation,
/// shared owned labels/assets, missing trainer identities, conflicting field
/// behavior, inconsistent split payloads or failed final runtime verification.
/// Inputs remain unchanged on failure. Existing saves inside the old gym need
/// separate position/terrain-override migration review.
pub fn replace_installed_minneapolis_gym(
    base: &CompiledGamePack,
    canonical: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_compiled_game_pack_for_runtime(canonical)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST)
            && !base.data.maps.contains_key("MnMinneapolis"),
        "Gym3 replacement must precede city installation and run only once"
    );
    ensure!(
        canonical.identity.content_hash == DONOR_CONTENT,
        "canonical Gym3 donor differs from the captured reviewed pack"
    );
    let old = base.data.maps.get(MAP).context("installed old Gym3")?;
    let donor = canonical.data.maps.get(MAP).context("canonical Gym3")?;
    ensure!(
        minneapolis_gym::content_hash(old)? == OLD_CONTENT
            && minneapolis_gym::content_hash(donor)? == NEW_CONTENT,
        "exact old and reviewed Gym3 module generations required"
    );
    let original_metadata = base
        .data
        .runtime_map_metadata
        .get(CONSTANT)
        .context("original gym metadata")?;
    ensure!(
        old.events.warps.len() == 3
            && old.events.warps.iter().all(|w| {
                w.target_map_constant == "MN_GYM_REVIEW_HALL" && w.target_warp_id == 9
            })
            && original_metadata.group_id == 63
            && original_metadata.map_id == 3,
        "original full-circuit Gym3 identity and review-hall returns required"
    );
    ensure!(
        !base
            .data
            .maps
            .values()
            .any(|m| m.id != MAP && m.attributes.tileset_name == TILESET)
            && !base.data.field_moves.cut.replacements.contains_key(TILESET)
            && !base
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key(TILESET)
            && !canonical
                .data
                .field_moves
                .cut
                .replacements
                .contains_key(TILESET)
            && !canonical
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key(TILESET),
        "Gym3 tileset must be private with no field-move replacement behavior"
    );
    let old_labels = labels(old)?;
    let new_labels = labels(donor)?;
    for (name, other) in &base.data.maps {
        if name != MAP {
            ensure!(
                labels(other)?.is_disjoint(&old_labels) && labels(other)?.is_disjoint(&new_labels),
                "Gym3 owns a script label shared with {name}"
            );
        }
    }
    for label in new_labels.difference(&old_labels) {
        ensure!(
            !base.data.map_scripts.contains_key(label),
            "new Gym3 label occupied: {label}"
        );
    }
    validate_split(base, old)?;
    validate_split(canonical, donor)?;

    let mut result = base.clone();
    for name in TRAINERS {
        let previous = base
            .data
            .trainers
            .trainers
            .get(name)
            .context("old gym trainer")?;
        let reviewed = canonical
            .data
            .trainers
            .trainers
            .get(name)
            .context("reviewed gym trainer")?;
        let previous_number = *base
            .data
            .story_event_script_constants
            .global
            .get(name)
            .context("original trainer number")?;
        let class_number = base
            .data
            .story_event_script_constants
            .global
            .get(&reviewed.trainer_class)
            .context("native reviewed trainer class")?;
        ensure!(
            Some(class_number)
                == canonical
                    .data
                    .story_event_script_constants
                    .global
                    .get(&reviewed.trainer_class),
            "native trainer class number differs: {name}"
        );
        let number = if previous.trainer_class == reviewed.trainer_class {
            previous_number
        } else {
            // Trainer IDs are local to their class. The older room used a
            // different roster; reusing its numbers can select an unrelated
            // native trainer in the reviewed class.
            base.data
                .trainers
                .trainers
                .values()
                .chain(result.data.trainers.trainers.values())
                .filter(|trainer| trainer.trainer_class == reviewed.trainer_class)
                .filter_map(|trainer| {
                    result
                        .data
                        .story_event_script_constants
                        .global
                        .get(&trainer.trainer_id)
                })
                .copied()
                .max()
                .context("reviewed trainer class roster")?
                .checked_add(1)
                .context("Gym3 trainer allocation overflow")?
        };
        u16::try_from(number).context("Gym3 trainer number exceeds native range")?;
        result
            .data
            .story_event_script_constants
            .global
            .insert(name.into(), number);
        result
            .data
            .trainers
            .trainers
            .insert(name.into(), reviewed.clone());
    }
    for name in canonical
        .data
        .story_event_script_constants
        .global
        .keys()
        .filter(|name| name.starts_with("EVENT_MN_GYM_3_"))
    {
        if !result
            .data
            .story_event_script_constants
            .global
            .contains_key(name)
        {
            let next = result
                .data
                .story_event_script_constants
                .global
                .iter()
                .filter(|(key, _)| key.starts_with("EVENT_"))
                .map(|(_, value)| *value)
                .max()
                .context("native event table")?
                .checked_add(1)
                .context("Gym3 event allocation overflow")?;
            u16::try_from(next).context("Gym3 event exceeds native flag range")?;
            result
                .data
                .story_event_script_constants
                .global
                .insert(name.clone(), next);
        }
    }
    result.data.tilesets.insert(
        TILESET.into(),
        canonical
            .data
            .tilesets
            .get(TILESET)
            .context("reviewed tileset")?
            .clone(),
    );
    for path in FILES {
        ensure!(
            base.runtime_files.contains_key(path),
            "old owned gym art missing: {path}"
        );
        result.runtime_files.insert(
            path.into(),
            canonical
                .runtime_files
                .get(path)
                .context("reviewed owned gym art")?
                .clone(),
        );
    }

    let mut gym = donor.clone();
    for (warp, previous) in gym.events.warps.iter_mut().zip(&old.events.warps) {
        warp.target_map_constant = previous.target_map_constant.clone();
        warp.target_map = previous.target_map.clone();
        warp.target_warp_id = previous.target_warp_id;
    }
    crate::gym_scripts::finish(&result, &mut gym)?;
    for label in old_labels {
        result.data.map_scripts.remove(&label);
    }
    let blocks = gym
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    result.data.map_blocks.insert(
        gym.attributes.blocks_label.clone().context("gym blocks")?,
        crate::encode_base64_bytes(&blocks),
    );
    result
        .data
        .map_attributes
        .insert(MAP.into(), gym.attributes.clone());
    result
        .data
        .map_dimensions
        .insert(CONSTANT.into(), json!({"width":5,"height":9}));
    let metadata = result
        .data
        .runtime_map_metadata
        .get_mut(CONSTANT)
        .context("gym metadata")?;
    metadata.width = 5;
    metadata.height = 9;
    crate::insert_compiled_map_split_payloads(
        &mut result.data.map_scripts,
        &mut result.data.npcs,
        MAP,
        gym.attributes
            .map_scripts_label
            .as_deref()
            .context("gym script section")?,
        gym.attributes
            .map_events_label
            .as_deref()
            .context("gym event section")?,
        &gym,
    )?;
    validate_split(&result, &gym)?;
    ensure!(
        minneapolis_gym::content_hash(&gym)? == NEW_CONTENT,
        "imported gym content drift"
    );
    result.data.maps.insert(MAP.into(), gym);
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn labels(module: &MapModule) -> Result<BTreeSet<String>> {
    Ok(module
        .scripts
        .keys()
        .chain(module.script_text_bodies.keys())
        .cloned()
        .chain([
            module
                .attributes
                .map_scripts_label
                .clone()
                .context("map script label")?,
            module
                .attributes
                .map_events_label
                .clone()
                .context("map event label")?,
        ])
        .collect())
}

fn validate_split(pack: &CompiledGamePack, expected: &MapModule) -> Result<()> {
    let mut uncached = pack.data.clone();
    uncached.maps.remove(MAP);
    let mut assembled = uncached.assemble_map_module_from_compiled_payloads(MAP)?;
    // The production assembler includes raw transport sections in `scripts`;
    // generated cached modules keep those sections in their dedicated fields.
    for label in [
        expected.attributes.map_scripts_label.as_ref(),
        expected.attributes.map_events_label.as_ref(),
    ] {
        let label = label.context("owned gym transport")?;
        if !expected.scripts.contains_key(label) {
            assembled.scripts.remove(label);
        }
    }
    ensure!(
        assembled == *expected,
        "uncached native Gym3 assembly differs from its split payloads"
    );
    Ok(())
}
