//! Carry the accepted bank/cache delta into the comprehensive campaign review.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const MARKER: &str = "comprehensive-current-world-bank-cache-v1";
const APPROACH: &str = "MnIronDrillApproach";

/// Integrates the exact newer selected bank/cache world into the repaired review.
///
/// Adopts only the reviewed fourteen data paths. All other campaign content,
/// optional areas, cast, fishing bindings, original assets and audio survive.
/// Borrowed inputs and existing saves remain untouched; output has a new identity.
///
/// # Errors
/// Rejects different input identities, stale preimages, missing accepted fields,
/// repeated integration, invalid runtime data or inconsistent regional geography.
pub fn integrate_comprehensive_current_world(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, hash) in [
        (
            current,
            "6192149ec0b740acea79b9d503d8a4491aa949ac779ab2d0fb466f7af0b62721",
        ),
        (
            common,
            "a7f6270ac5c7cdc83fe0f7484fd8485990ceb9749a0fde803dc43cbd22b90bad",
        ),
        (
            latest,
            "762ea924b566ae78f839d72944bf5b19c9397a4730b4c6413f29bbd555a5e406",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact reviewed world integration inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    let mut pack = current.clone();
    adopt_entry(
        &mut pack.data.maps,
        &common.data.maps,
        &latest.data.maps,
        "MnR14C00R00",
    )?;
    adopt_entry(
        &mut pack.data.npcs,
        &common.data.npcs,
        &latest.data.npcs,
        APPROACH,
    )?;
    for label in ["MnIronDrillApproach_MapEvents", "MN_IRON_APPROACH_ETHER"] {
        adopt_entry(
            &mut pack.data.map_scripts,
            &common.data.map_scripts,
            &latest.data.map_scripts,
            label,
        )?;
    }
    for label in ["MnIronDrillApproach_Blocks", "MnR14C00R00_Blocks"] {
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
            label,
        )?;
    }
    adopt_entry(
        &mut pack.data.story_event_script_constants.global,
        &common.data.story_event_script_constants.global,
        &latest.data.story_event_script_constants.global,
        "EVENT_MN_IRON_APPROACH_ETHER",
    )?;
    let map = pack
        .data
        .maps
        .get_mut(APPROACH)
        .context("current approach")?;
    let before = common.data.maps.get(APPROACH).context("common approach")?;
    let revised = latest
        .data
        .maps
        .get(APPROACH)
        .context("accepted approach")?;
    adopt(
        &mut map.blocks,
        &before.blocks,
        &revised.blocks,
        "approach blocks",
    )?;
    adopt(
        &mut map.script_field_pickups,
        &before.script_field_pickups,
        &revised.script_field_pickups,
        "approach pickups",
    )?;
    adopt(
        &mut map.objects,
        &before.objects,
        &revised.objects,
        "approach objects",
    )?;
    adopt(
        &mut map.scripts,
        &before.scripts,
        &revised.scripts,
        "approach programs",
    )?;
    adopt(
        &mut map.attributes.border_block,
        &before.attributes.border_block,
        &revised.attributes.border_block,
        "approach border",
    )?;
    adopt(
        &mut map.map_event_section_commands,
        &before.map_event_section_commands,
        &revised.map_event_section_commands,
        "approach event mirror",
    )?;
    let attributes = pack
        .data
        .map_attributes
        .get_mut(APPROACH)
        .context("current raw approach attributes")?;
    adopt(
        &mut attributes.border_block,
        &common
            .data
            .map_attributes
            .get(APPROACH)
            .context("common raw approach attributes")?
            .border_block,
        &latest
            .data
            .map_attributes
            .get(APPROACH)
            .context("accepted raw approach attributes")?
            .border_block,
        "raw approach border",
    )?;
    for manifest in &latest.report.manifests {
        if !common.report.manifests.contains(manifest) && !pack.report.manifests.contains(manifest)
        {
            pack.report.manifests.push(manifest.clone());
        }
    }
    ensure!(
        !pack.report.manifests.iter().any(|m| m == MARKER),
        "fresh world integration required"
    );
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn adopt<T: Clone + Eq>(current: &mut T, common: &T, latest: &T, path: &str) -> Result<()> {
    ensure!(
        current == common && latest != common,
        "stale accepted delta preimage: {path}"
    );
    current.clone_from(latest);
    Ok(())
}

pub(super) fn adopt_entry<T: Clone + Eq>(
    current: &mut BTreeMap<String, T>,
    common: &BTreeMap<String, T>,
    latest: &BTreeMap<String, T>,
    key: &str,
) -> Result<()> {
    ensure!(
        current.get(key) == common.get(key) && latest.get(key) != common.get(key),
        "stale accepted map entry: {key}"
    );
    current.insert(
        key.into(),
        latest.get(key).context("accepted delta entry")?.clone(),
    );
    Ok(())
}

/// Carries the exact accepted Brainerd mart roof into the integrated review.
///
/// Only Brainerd's block array and its raw block mirror change. Existing
/// campaign content, collision/event metadata, assets and audio are preserved.
/// The resulting pack has a new identity; existing saves are not migrated.
///
/// # Errors
/// Rejects different inputs, stale block preimages or invalid resulting content.
pub fn integrate_comprehensive_brainerd_roof(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, hash) in [
        (
            current,
            "8fdae651b0e96af7d80f96e03f1a5091b327c062a759303701110abea81972cc",
        ),
        (
            common,
            "762ea924b566ae78f839d72944bf5b19c9397a4730b4c6413f29bbd555a5e406",
        ),
        (
            latest,
            "acbcb88bb951f3cc343ac550a4a83e423904a7b42fa4dfccfc88497d49092866",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact reviewed roof integration inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    let mut pack = current.clone();
    adopt_entry(
        &mut pack.data.map_blocks,
        &common.data.map_blocks,
        &latest.data.map_blocks,
        "MnBrainerd_Blocks",
    )?;
    adopt(
        &mut pack
            .data
            .maps
            .get_mut("MnBrainerd")
            .context("current Brainerd")?
            .blocks,
        &common
            .data
            .maps
            .get("MnBrainerd")
            .context("common Brainerd")?
            .blocks,
        &latest
            .data
            .maps
            .get("MnBrainerd")
            .context("accepted Brainerd")?
            .blocks,
        "Brainerd roof blocks",
    )?;
    for manifest in &latest.report.manifests {
        if !common.report.manifests.contains(manifest) && !pack.report.manifests.contains(manifest)
        {
            pack.report.manifests.push(manifest.clone());
        }
    }
    pack.report
        .manifests
        .push("comprehensive-brainerd-roof-integration-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
