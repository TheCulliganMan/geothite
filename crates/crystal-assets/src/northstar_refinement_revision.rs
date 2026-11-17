//! Checked refinement of the published Minnesota adventure and its save lineage.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::json;

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const PARENT: &str = "d4ce7998673f393e69771a36c3575878060f121102f6db840ddf197c583979dd";
const MANIFEST: &str = "northstar-refinement-v1";
const COMPATIBILITY_FILE: &str = "data/northstar-save-compatibility.json";
const SOURCES: [(&str, &str); 4] = [
    (
        "80dc44ba48e3347b3e760095c295d10748386b9e8ab2a985350f043862236baa",
        "5254da0a14f9bf6acdb6378ee9d7d3f890d4f6eab13e5334ff079033212c4858",
    ),
    (
        PARENT,
        "36da6928a3aa0b0d75c9ad9a7abe353a9d79a7d3d1327eed2dc29d4fc4adb181",
    ),
    (
        "b5f8411db557d73a0a9c755243415a33342537f619e200f544233dffc8027c02",
        "8498ef8d685b32d468876dc46da069ab394f97dd9da3da73b4c40ac9afecec0e",
    ),
    (
        "cd81b377a9d08285dddba8568547e573827860f1ea41630e2301be4750296313",
        "6d4cca148a5e277442a714fd1474d84bd315a1c63aea0ff80dd2a1003437e3a7",
    ),
];

/// Exact prior release eligible for checked state migration into this revision.
/// Construct this only with [`northstar_refinement_save_source`].
#[derive(Serialize)]
pub struct NorthstarRefinementSaveSource {
    modpack_id: String,
    modpack_hash: String,
    content_hash: String,
}

/// Verify a known prior release's save identity and unchanged map/event contract.
///
/// # Errors
/// Rejects unknown bytes or identities, changed geometry, reordered original
/// actors, or reassigned saved event constants. This does not migrate a save.
pub fn northstar_refinement_save_source(
    adventure: &CompiledGamePack,
    loaded: &LoadedCompiledGamePack,
) -> Result<NorthstarRefinementSaveSource> {
    ensure!(
        adventure.identity()?.content_hash == PARENT,
        "expected adventure parent"
    );
    let identity = loaded.save_modpack_identity()?;
    let content_hash = loaded.pack_content_hash()?;
    ensure!(
        SOURCES
            .iter()
            .any(|(content, bytes)| *content == content_hash && *bytes == identity.hash()),
        "save compatibility source is not an exact approved release"
    );
    preserve_saved_map_contract(loaded.pack(), adventure)?;
    Ok(NorthstarRefinementSaveSource {
        modpack_id: identity.id().to_owned(),
        modpack_hash: identity.hash().to_owned(),
        content_hash,
    })
}

/// Build the complete refinement while preserving saved geography and receipts.
///
/// Each supplied save source must already be checked against the selected
/// adventure. The resulting metadata authorizes only this exact raw/hosted
/// manifest identity; the runtime must still validate each individual saved state.
///
/// # Errors
/// Rejects a different parent, incomplete compatibility lineage, invalid authored
/// content, changed original map contracts, audio or non-Minnesota content.
pub fn refine_northstar_adventure(
    base: &CompiledGamePack,
    story: &str,
    pokemon: &str,
    discoveries: &str,
    mut sources: Vec<NorthstarRefinementSaveSource>,
) -> Result<CompiledGamePack> {
    ensure!(
        base.identity()?.content_hash == PARENT,
        "refinement requires adventure v01"
    );
    ensure!(
        sources.len() == SOURCES.len()
            && sources
                .iter()
                .map(|source| source.content_hash.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(content, _)| *content).collect(),
        "exact adventure/integrated raw and hosted save sources required"
    );
    sources.sort_by_key(|source| {
        SOURCES
            .iter()
            .position(|(hash, _)| *hash == source.content_hash)
    });
    let (mut candidate, deferred) = crate::northstar_story_revision::refresh_installed_story(
        base,
        story,
        crate::northstar_story_revision::InvestigationAccess::PreserveBadgeChecks,
    )?;
    ensure!(
        deferred.is_empty(),
        "unstaged canonical dialogue: {}",
        deferred.join("\n")
    );
    crate::northstar_refinement_battles::install(&mut candidate, story, pokemon)?;
    crate::northstar_reed_postgame::install(&mut candidate, story, pokemon)?;
    crate::northstar_museum_activities::install(&mut candidate, story)?;
    crate::northstar_restoration_aftermath::install(&mut candidate, story)?;
    crate::northstar_refinement_discoveries::install(&mut candidate, story, discoveries)?;
    candidate.report.manifests.push(MANIFEST.into());
    preserve_saved_map_contract(base, &candidate)?;
    for (id, original) in &base.data.maps {
        if !id.starts_with("Mn") {
            ensure!(
                candidate.data.maps.get(id) == Some(original),
                "non-Minnesota map changed: {id}"
            );
        }
    }
    ensure!(
        candidate.data.audio == base.data.audio
            && candidate.audio_manifest == base.audio_manifest
            && candidate.compiled_audio == base.compiled_audio
            && candidate.audio_compression == base.audio_compression,
        "refinement changed existing audio"
    );
    for (path, bytes) in &base.runtime_files {
        ensure!(
            candidate.runtime_files.get(path) == Some(bytes),
            "existing runtime file changed: {path}"
        );
    }
    ensure!(
        !candidate.data.server_clock
            && !candidate.data.player_customization
            && !candidate.runtime_files.contains_key(COMPATIBILITY_FILE),
        "expected unwrapped parent without compatibility metadata"
    );
    let raw_id = candidate.runtime_modpack_id()?;
    // These are the existing server wrappers, in their production order.
    let hosted_id = format!(
        "{raw_id}+{}+{}",
        crate::REALTIME_CLOCK_MANIFEST_ID,
        crate::PLAYER_CUSTOMIZATION_MANIFEST_ID
    );
    candidate.runtime_files.insert(
        COMPATIBILITY_FILE.into(),
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "target_manifest": MANIFEST,
            "target_modpack_ids": [raw_id, hosted_id],
            "sources": sources
        }))?,
    );
    candidate.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        candidate.format_version,
        &candidate.data,
        &candidate.audio_manifest,
        &candidate.runtime_files,
        &candidate.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&candidate)?;
    Ok(candidate)
}

fn preserve_saved_map_contract(before: &CompiledGamePack, after: &CompiledGamePack) -> Result<()> {
    ensure!(
        before.data.maps.len() == after.data.maps.len(),
        "map set changed"
    );
    for (id, old) in &before.data.maps {
        let new = after.data.maps.get(id).context("preserved map")?;
        ensure!(
            old.attributes == new.attributes
                && old.blocks == new.blocks
                && old.events.warps == new.events.warps,
            "save-compatible geometry changed: {id}"
        );
        ensure!(
            new.objects.len() >= old.objects.len(),
            "original actor removed: {id}"
        );
        for (old, new) in old.objects.iter().zip(&new.objects) {
            ensure!(
                old.object_identifier == new.object_identifier && old.x == new.x && old.y == new.y,
                "original actor reordered or relocated: {id}"
            );
        }
    }
    for (name, value) in &before.data.story_event_script_constants.global {
        ensure!(
            after.data.story_event_script_constants.global.get(name) == Some(value),
            "saved event constant reassigned: {name}"
        );
    }
    ensure!(
        before.data.pokemon.keys().eq(after.data.pokemon.keys())
            && before.data.moves.keys().eq(after.data.moves.keys())
            && before.data.items.keys().eq(after.data.items.keys()),
        "saved Pokemon, move or item identities changed"
    );
    Ok(())
}
