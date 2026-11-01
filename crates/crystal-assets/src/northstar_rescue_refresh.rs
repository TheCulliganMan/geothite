//! Bounded rescue-evidence correction for an already revised current game.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const MARKER: &str = "northstar-rescue-evidence-refresh-v1";
const OFFICE: &str = "MnMinneapolisDispatchOffice";
const HOME: &str = "MnMinneapolisDispatcherHouse";

/// Refreshes the holding enclosure and dispatch-evidence handoff in place.
/// Ellis supplies directions after rescue; the office desk grants the dispatch
/// note only after holding power, enforcer victory and rescue. Existing staged
/// dialogue and every outdoor map remain intact. A new content identity is
/// derived; saves are never migrated and the borrowed pack is unchanged.
///
/// # Errors
/// Rejects missing current-story markers, duplicate refresh, changed guarded
/// scene anchors, unrelated map or original-asset changes, invalid runtime data,
/// and inconsistent world geometry.
pub fn refresh_northstar_rescue_evidence(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|m| m == "northstar-dialogue-overhaul-v1"),
        "installed current dialogue required"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MARKER),
        "rescue evidence refresh already installed"
    );
    let mut pack = base.clone();
    let mut office = pack
        .data
        .maps
        .get(OFFICE)
        .context("reviewed holding office")?
        .clone();
    crate::northstar_holding_fence::install(&mut pack, &mut office)?;
    crate::northstar_story_revision::publish(&mut pack, office)?;
    crate::northstar_dispatch_revision::install(&mut pack)?;
    ensure!(
        base.data.maps.len() == pack.data.maps.len(),
        "rescue refresh may not add maps"
    );
    for (name, original) in &base.data.maps {
        if name != OFFICE && name != HOME {
            ensure!(
                pack.data.maps.get(name) == Some(original),
                "unrelated rescue refresh map: {name}"
            );
        }
    }
    ensure!(
        pack.data.story_event_script_constants == base.data.story_event_script_constants,
        "rescue refresh may not alter event identities"
    );
    ensure!(
        pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_compression == base.audio_compression,
        "rescue refresh may not alter audio"
    );
    for (name, original) in &base.runtime_files {
        ensure!(
            pack.runtime_files.get(name) == Some(original),
            "rescue refresh changed original asset: {name}"
        );
    }
    for name in pack
        .runtime_files
        .keys()
        .filter(|name| !base.runtime_files.contains_key(*name))
    {
        ensure!(
            name.contains("mn_northstar_holding"),
            "unrelated added rescue asset: {name}"
        );
    }
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
