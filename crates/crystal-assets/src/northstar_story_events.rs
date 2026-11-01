//! Incremental native publication of the character-driven Northstar revision.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

/// Publishes the first playable chapter of the event-driven story revision.
///
/// Adds Reed's acted field-station encounter/battle, Tamarack's egg-care offer
/// after the Rochester errand, and the optional return conversations. Source
/// plans for the two lairs are retained as plans; this opening publisher does
/// not claim their implementation or grant their future results.
///
/// # Errors
/// Returns an error for an incompatible reviewed parent, invalid source,
/// occupied native identities, unsafe actor paths, or pack verification failure.
/// The borrowed input, original art and all original audio are retained.
pub fn build_northstar_story_events(
    base: &CompiledGamePack,
    story: &str,
    events: &str,
) -> Result<CompiledGamePack> {
    let source: Value = serde_json::from_str(events).context("event-driven story source")?;
    ensure!(
        source["parent_dialogue_hash"] == base.identity()?.content_hash
            || crate::northstar_current_story::is_composing(base),
        "event story parent differs"
    );
    let mut pack = crate::build_northstar_returns(base, story)?;
    crate::northstar_story_opening::install(&mut pack, &source)?;
    pack.report
        .manifests
        .push("minnesota-northstar-events-opening-v2".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
