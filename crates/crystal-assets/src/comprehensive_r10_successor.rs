//! Preserve the campaign while adopting the accepted Route 10 successor.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;

/// Adopts five reviewed Route 10 entries after the haulage integration.
///
/// Requires the completed haulage-stage marker, exact common/latest world
/// identities and unchanged current preimages for all five entries. All other
/// current content, assets and audio are preserved. Existing saves are unchanged.
///
/// # Errors
/// Rejects an incorrect or repeated stage, different world inputs, stale Route10
/// preimages, donor asset/audio changes or invalid resulting gameplay/geography.
pub fn integrate_comprehensive_r10_successor(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    const MARKER: &str = "comprehensive-r10-current-successor-v1";
    ensure!(
        current
            .report
            .manifests
            .iter()
            .any(|m| m == "comprehensive-haulage-route-successors-v1")
            && !current.report.manifests.iter().any(|m| m == MARKER),
        "completed haulage stage and fresh R10 integration required"
    );
    for (pack, hash) in [
        (
            common,
            "66bf8c7f11c17ce5c8eee4a69f7dc07c28b79dcfc67d336dd2c9b2baf9b79d4c",
        ),
        (
            latest,
            "c68542065bac76644016febec0ff6217ecf1ac446caa247a1579d3cd024a9594",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact accepted R10 world inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    crate::verify_compiled_game_pack_for_runtime(current)?;
    ensure!(
        common.runtime_files == latest.runtime_files,
        "donor art changed"
    );
    ensure!(
        common.audio_manifest == latest.audio_manifest,
        "donor audio changed"
    );
    let mut pack = current.clone();
    adopt_entry(
        &mut pack.data.maps,
        &common.data.maps,
        &latest.data.maps,
        "MnR10C00R00",
    )?;
    adopt_entry(
        &mut pack.data.map_blocks,
        &common.data.map_blocks,
        &latest.data.map_blocks,
        "MnR10C00R00_Blocks",
    )?;
    for name in [
        "MN_R10_RAMP_SIGNText",
        "MN_R10_ROAD_KEEPERText",
        "MN_R10_T01Won",
    ] {
        adopt_entry(
            &mut pack.data.map_scripts,
            &common.data.map_scripts,
            &latest.data.map_scripts,
            name,
        )?;
    }
    for manifest in &latest.report.manifests {
        if !common.report.manifests.contains(manifest) && !pack.report.manifests.contains(manifest)
        {
            pack.report.manifests.push(manifest.clone());
        }
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
