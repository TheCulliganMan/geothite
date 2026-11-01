//! Integrate the accepted native haulage floor and R07/R08 successors.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;

/// Carries the exact accepted haulage and route changes into the review game.
///
/// Resolves the dry haulage floor's documented unused fishing reference to the
/// Minnesota lake group. Only its private native art is added. Existing story,
/// optional content, roster and audio remain authoritative; saves are unchanged.
///
/// # Errors
/// Rejects different identities, stale preimages, unrelated donor art changes,
/// repeated integration, invalid runtime content or inconsistent geography.
pub fn integrate_comprehensive_haulage_successors(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    const MARKER: &str = "comprehensive-haulage-route-successors-v1";
    const MINE: &str = "MnIronHaulage";
    const ART: &str = "mn_iron_haulage_rock";
    for (pack, hash) in [
        (
            current,
            "8e66a4740371ad2502ee83edc5b2e3011b0d1186423ceb31d0af047abe8a82f8",
        ),
        (
            common,
            "41aad022dd980dc1bf1f5784fb8a8a465648d500c1119ee2ac2cda2946121ffa",
        ),
        (
            latest,
            "66bf8c7f11c17ce5c8eee4a69f7dc07c28b79dcfc67d336dd2c9b2baf9b79d4c",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact accepted integration inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        !current.report.manifests.iter().any(|m| m == MARKER),
        "fresh integration required"
    );
    ensure!(
        common.audio_manifest == latest.audio_manifest,
        "donor audio changed"
    );
    for (name, bytes) in &common.runtime_files {
        ensure!(
            latest.runtime_files.get(name) == Some(bytes),
            "existing donor art changed: {name}"
        );
    }
    let additions = latest
        .runtime_files
        .keys()
        .filter(|name| !common.runtime_files.contains_key(*name))
        .collect::<Vec<_>>();
    ensure!(
        additions.len() == 5,
        "exact five private native art files required"
    );
    let mut pack = current.clone();
    for name in additions {
        ensure!(
            name.contains(ART) && !current.runtime_files.contains_key(name),
            "unrelated or existing art: {name}"
        );
        pack.runtime_files
            .insert(name.clone(), latest.runtime_files[name].clone());
    }
    adopt_entry(
        &mut pack.data.tilesets,
        &common.data.tilesets,
        &latest.data.tilesets,
        ART,
    )?;
    for name in ["MnR07C00R00", "MnR07C00R01", "MnR08C00R00", "MnR08C00R01"] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            name,
        )?;
    }
    // The comprehensive fishing pass already fixed the old generic reference.
    // The accepted dry-floor successor deliberately selects MN_LAKE instead.
    let mut mine = current
        .data
        .maps
        .get(MINE)
        .context("current haulage")?
        .clone();
    ensure!(
        mine.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_SHORE"),
        "current regional fishing preimage"
    );
    mine.attributes.fishing_group = Some("FISHGROUP_SHORE".into());
    let original = common.data.maps.get(MINE).context("common haulage")?;
    ensure!(
        mine.attributes == original.attributes && mine.blocks == original.blocks,
        "unreviewed current haulage terrain changes"
    );
    let donor = latest.data.maps.get(MINE).context("accepted haulage")?;
    let mut expected_donor = original.clone();
    expected_donor.attributes = donor.attributes.clone();
    expected_donor.blocks = donor.blocks.clone();
    ensure!(
        &expected_donor == donor,
        "donor changed haulage programs or actors"
    );
    ensure!(
        donor.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_LAKE"),
        "accepted dry-floor fishing reference"
    );
    // Preserve the comprehensive supporting dialogue and its cached commands.
    mine.attributes = donor.attributes.clone();
    mine.blocks = donor.blocks.clone();
    pack.data.maps.insert(MINE.into(), mine);
    let mut attributes = current
        .data
        .map_attributes
        .get(MINE)
        .context("current raw haulage attributes")?
        .clone();
    ensure!(
        attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_SHORE"),
        "raw regional fishing preimage"
    );
    attributes.fishing_group = Some("FISHGROUP_SHORE".into());
    ensure!(
        Some(&attributes) == common.data.map_attributes.get(MINE),
        "unreviewed raw haulage changes"
    );
    pack.data.map_attributes.insert(
        MINE.into(),
        latest
            .data
            .map_attributes
            .get(MINE)
            .context("accepted raw haulage attributes")?
            .clone(),
    );
    for name in [
        "MnIronHaulage_Blocks",
        "MnR07C00R00_Blocks",
        "MnR08C00R01_Blocks",
    ] {
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
            name,
        )?;
    }
    for name in [
        "MN_R07_DROP_SIGNText",
        "MN_R07_T02Won",
        "MN_R08_BIRCH_WALKERText",
        "MN_R08_SIGN_03Text",
        "MN_R08_T07Won",
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
