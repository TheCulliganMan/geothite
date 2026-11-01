//! Carry the accepted road and R17 cave changes into the comprehensive story.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;

/// Integrates the twelve reviewed road/cave entries from the accepted world.
///
/// The exact comprehensive, common ice-world and accepted R06-world identities
/// are required. Current story programs, optional maps, rewards, roster, art and
/// audio remain authoritative. Borrowed inputs and existing saves are unchanged;
/// the resulting pack has a new content identity.
///
/// # Errors
/// Rejects different inputs, stale entry preimages, changed donor assets/audio,
/// repeated integration, invalid runtime content or inconsistent geography.
pub fn integrate_comprehensive_road_successors(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    const MARKER: &str = "comprehensive-road-cave-successors-v1";
    for (pack, hash) in [
        (
            current,
            "baacf2caab1884a2cf94d97cd5f1e099fbae9de50a35fecd20a65b69d960538e",
        ),
        (
            common,
            "e964dc760a49b64510f93d1469169a5d88aa73104d12f026e8dcf4084f73bc8f",
        ),
        (
            latest,
            "41aad022dd980dc1bf1f5784fb8a8a465648d500c1119ee2ac2cda2946121ffa",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact accepted road integration inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        common.runtime_files == latest.runtime_files,
        "donor assets changed"
    );
    ensure!(
        common.audio_manifest == latest.audio_manifest,
        "donor audio changed"
    );
    ensure!(
        !current.report.manifests.iter().any(|m| m == MARKER),
        "fresh road integration required"
    );
    let mut pack = current.clone();
    for name in [
        "MnR05C00R00",
        "MnR05C00R01",
        "MnR06C00R01",
        "MnR09C00R01",
        "MnR11C00R00",
        "MnR17Cave1F",
    ] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            name,
        )?;
    }
    for name in [
        "MnR05C00R00",
        "MnR06C00R01",
        "MnR09C00R01",
        "MnR11C00R00",
        "MnR17Cave1F",
    ] {
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
            &format!("{name}_Blocks"),
        )?;
    }
    adopt_entry(
        &mut pack.data.map_scripts,
        &common.data.map_scripts,
        &latest.data.map_scripts,
        "MN_R05_RAMP_SIGNText",
    )?;
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
