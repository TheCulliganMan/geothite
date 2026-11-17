//! A bounded adventure revision on the selected integrated Minnesota campaign.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const MANIFEST: &str = "northstar-adventure-v1";

/// Adds authored discoveries, optional rematches and current campaign writing.
///
/// The input must be the selected integrated v03 pack. This operation preserves
/// existing audio and non-Minnesota maps, does not select the result, and never
/// reads or migrates player saves. Hidden-item placement is checked against the
/// actual map collision and native event tables without running the game.
///
/// # Errors
/// Rejects a different parent, invalid source, unstaged dialogue, conflicting
/// event identities, disconnected discoveries or invalid compiled content.
pub fn improve_northstar_adventure(
    base: &CompiledGamePack,
    story: &str,
    discoveries: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        base.identity()?.content_hash
            == "cd81b377a9d08285dddba8568547e573827860f1ea41630e2301be4750296313",
        "adventure revision requires the selected integrated v03 campaign"
    );
    let (mut candidate, deferred) = crate::northstar_story_revision::refresh_installed_story(
        base,
        story,
        crate::northstar_story_revision::InvestigationAccess::PreserveBadgeChecks,
    )
    .context("refresh canonical campaign writing")?;
    ensure!(
        deferred.is_empty(),
        "unstaged canonical dialogue:\n{}",
        deferred.join("\n")
    );
    crate::northstar_presentation::install(&mut candidate, story)
        .context("accepted native trainer identities")?;
    crate::northstar_battle_balance::install(&mut candidate).context("campaign battle balance")?;
    crate::northstar_postgame_rematches::install(&mut candidate, story)
        .context("optional gym rematches")?;
    crate::northstar_hidden_items::install(&mut candidate, discoveries)
        .context("hidden discoveries")?;
    candidate.report.manifests.push(MANIFEST.into());
    ensure!(
        candidate.data.maps.len() == base.data.maps.len(),
        "adventure revision changed the map set"
    );
    for (id, original) in &base.data.maps {
        let revised = candidate.data.maps.get(id).context("preserved map")?;
        ensure!(
            original.attributes == revised.attributes && original.blocks == revised.blocks,
            "adventure revision changed terrain: {id}"
        );
        if !id.starts_with("Mn") {
            ensure!(
                original == revised,
                "adventure revision changed non-Minnesota map {id}"
            );
        }
    }
    ensure!(
        candidate.data.audio == base.data.audio
            && candidate.audio_manifest == base.audio_manifest
            && candidate.compiled_audio == base.compiled_audio
            && candidate.audio_compression == base.audio_compression,
        "adventure revision changed existing audio"
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
