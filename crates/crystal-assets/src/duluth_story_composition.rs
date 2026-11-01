//! Carry the accepted Duluth stairs across the released guard-text revision.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, MapModule};

const CITY: &str = "MnDuluth";
const ART: &str = "mn_duluth_expanded";
const MANIFEST: &str = "minnesota-duluth-cliff-stairs-v1";
const FILES: [&str; 5] = [
    "data/tilesets/mn_duluth_expanded_metatiles.bin",
    "data/tilesets/mn_duluth_expanded.json",
    "data/tilesets/mn_duluth_expanded_palette_map.json",
    "gfx/tilesets/mn_duluth_expanded.2bpp",
    "gfx/tilesets/mn_duluth_expanded.png",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    current_content_hash: String,
    accepted_base_content_hash: String,
    accepted_child_content_hash: String,
}

/// Composes the accepted stair terrain/art while preserving released guard text.
/// All three pack identities must match the explicit source contract. The
/// accepted pair may change only Duluth's blocks, its tileset and five native
/// art mirrors. Current spatial data and art must retain the accepted preimage;
/// only four guard messages and waitbutton-to-promptbutton boundaries may differ.
/// Every other instruction, including forced movement, must remain identical.
/// Every other map, program, catalog and audio program remains current.
/// Borrowed inputs remain unchanged and no files are written.
///
/// # Errors
/// Rejects stale inputs, duplicate installation, uncontracted donor changes,
/// spatial/art conflicts, non-presentation guard edits or invalid native data.
pub fn compose_duluth_story_terrain(
    current: &CompiledGamePack,
    before: &CompiledGamePack,
    accepted: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    let contract: Contract = serde_json::from_str(source)?;
    ensure!(
        contract.schema_version == 1,
        "Duluth composition contract version"
    );
    for (pack, hash) in [
        (current, &contract.current_content_hash),
        (before, &contract.accepted_base_content_hash),
        (accepted, &contract.accepted_child_content_hash),
    ] {
        crate::verify_compiled_game_pack_for_runtime(pack)?;
        crate::validate_compact_superior_geography(pack)?;
        let identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        ensure!(
            &identity.content_hash == hash,
            "Duluth composition input differs"
        );
    }
    ensure!(
        !current.report.manifests.iter().any(|id| id == MANIFEST),
        "Duluth stairs already composed"
    );
    let old = before
        .data
        .maps
        .get(CITY)
        .context("accepted Duluth preimage")?;
    let new = accepted
        .data
        .maps
        .get(CITY)
        .context("accepted Duluth result")?;
    let installed = current.data.maps.get(CITY).context("released Duluth")?;
    let mut expected = old.clone();
    expected.blocks.clone_from(&new.blocks);
    ensure!(
        expected == *new
            && old
                .blocks
                .iter()
                .zip(&new.blocks)
                .filter(|(a, b)| a != b)
                .count()
                == 29,
        "accepted Duluth revision must change only the reviewed 29 cells"
    );
    presentation_difference(old, installed)?;
    let mut report = before.report.clone();
    report.manifests.push(MANIFEST.into());
    ensure!(
        accepted.report == report,
        "accepted Duluth publication bookkeeping differs"
    );
    ensure!(
        without_delta(accepted, before)? == *before,
        "accepted Duluth revision contains unrelated content changes"
    );
    ensure!(
        current.data.tilesets.get(ART) == before.data.tilesets.get(ART)
            && current.data.tilesets.contains_key(ART),
        "current Duluth art preimage differs"
    );
    for path in FILES {
        ensure!(
            current.runtime_files.get(path) == before.runtime_files.get(path)
                && current.runtime_files.contains_key(path),
            "current Duluth art differs: {path}"
        );
    }
    let mut result = current.clone();
    let mut city = installed.clone();
    city.blocks.clone_from(&new.blocks);
    result.data.maps.insert(CITY.into(), city);
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("Duluth block mirror")?;
    result.data.map_blocks.insert(
        label.clone(),
        accepted
            .data
            .map_blocks
            .get(label)
            .context("accepted Duluth block mirror")?
            .clone(),
    );
    result.data.tilesets.insert(
        ART.into(),
        accepted
            .data
            .tilesets
            .get(ART)
            .context("accepted Duluth art")?
            .clone(),
    );
    for path in FILES {
        result.runtime_files.insert(
            path.into(),
            accepted
                .runtime_files
                .get(path)
                .with_context(|| format!("accepted Duluth art mirror {path}"))?
                .clone(),
        );
    }
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    ensure!(
        without_delta(&result, current)? == *current,
        "Duluth composition changed unrelated content"
    );
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}

fn presentation_difference(before: &MapModule, current: &MapModule) -> Result<()> {
    ensure!(
        before.scripts.keys().eq(current.scripts.keys()),
        "Duluth program inventory differs"
    );
    for (label, program) in &current.scripts {
        if before.scripts.get(label) == Some(program) {
            continue;
        }
        ensure!(
            [
                "MnDuluthRangeCheckBadges",
                "MnDuluthRangeCheckEvidence",
                "MnDuluthShoreCheckBadges",
                "MnDuluthShoreCheckEvidence",
                "MnDuluthRangeCheckBadgesText",
                "MnDuluthRangeCheckEvidenceText",
                "MnDuluthShoreCheckBadgesText",
                "MnDuluthShoreCheckEvidenceText",
            ]
            .contains(&label.as_str()),
            "unreviewed Duluth program differs: {label}"
        );
        let old = before
            .scripts
            .get(label)
            .context("old guard presentation")?;
        if label.ends_with("Text") {
            for value in [old, program] {
                ensure!(
                    value
                        .as_array()
                        .context("guard text program")?
                        .iter()
                        .all(|row| matches!(
                            row["command"].as_str(),
                            Some("text" | "line" | "para" | "cont" | "done" | "prompt")
                        )),
                    "guard body contains executable changes: {label}"
                );
            }
        } else {
            let mut normalized = program.clone();
            let old_rows = old.as_array().context("old guard wrapper")?;
            let rows = normalized.as_array_mut().context("current guard wrapper")?;
            ensure!(
                rows.len() == old_rows.len(),
                "guard wrapper length differs: {label}"
            );
            for (original, row) in old_rows.iter().zip(rows) {
                if *original == *row {
                    continue;
                }
                ensure!(
                    original["command"] == "waitbutton" && row["command"] == "promptbutton",
                    "guard instruction differs beyond its text boundary: {label}"
                );
                row["command"] = serde_json::Value::String("waitbutton".into());
                ensure!(
                    *row == *original,
                    "guard operands or movement differ: {label}"
                );
            }
            ensure!(normalized == *old, "guard wrapper differs: {label}");
        }
    }

    let mut normalized = current.clone();
    normalized.scripts.clone_from(&before.scripts);
    normalized
        .script_control_commands
        .clone_from(&before.script_control_commands);
    normalized
        .script_text_commands
        .clone_from(&before.script_text_commands);
    normalized
        .script_text_bodies
        .clone_from(&before.script_text_bodies);
    ensure!(
        normalized == *before,
        "current Duluth spatial or executable metadata differs"
    );
    Ok(())
}

fn without_delta(pack: &CompiledGamePack, original: &CompiledGamePack) -> Result<CompiledGamePack> {
    let old = original
        .data
        .maps
        .get(CITY)
        .context("Duluth residual preimage")?;
    let mut expected = old.clone();
    expected.blocks.clone_from(
        &pack
            .data
            .maps
            .get(CITY)
            .context("Duluth residual result")?
            .blocks,
    );
    ensure!(
        pack.data.maps.get(CITY) == Some(&expected),
        "Duluth nonterrain fields changed"
    );
    let mut result = pack.clone();
    result.data.maps.insert(CITY.into(), old.clone());
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("Duluth residual block label")?;
    result.data.map_blocks.insert(
        label.clone(),
        original
            .data
            .map_blocks
            .get(label)
            .context("Duluth residual blocks")?
            .clone(),
    );
    result.data.tilesets.insert(
        ART.into(),
        original
            .data
            .tilesets
            .get(ART)
            .context("Duluth residual tileset")?
            .clone(),
    );
    for path in FILES {
        result.runtime_files.insert(
            path.into(),
            original
                .runtime_files
                .get(path)
                .with_context(|| format!("Duluth residual art {path}"))?
                .clone(),
        );
    }
    result.identity = original.identity.clone();
    result.report.clone_from(&original.report);
    Ok(result)
}

#[cfg(test)]
#[path = "duluth_story_composition_tests.rs"]
mod tests;
