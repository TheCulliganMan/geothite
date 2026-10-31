//! Integrate the accepted ventilation floor and seven North Shore bridges.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;

/// Carries the exact accepted ventilation and route changes into the review game.
///
/// Uses the accepted Minnesota lake fishing reference and guarded native bridge
/// art updates. Existing story,
/// optional content, roster and audio remain authoritative; saves are unchanged.
///
/// # Errors
/// Rejects different identities, stale preimages, unrelated donor art changes,
/// repeated integration, invalid runtime content or inconsistent geography.
pub fn integrate_comprehensive_vent_bridges(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    const MARKER: &str = "comprehensive-vent-bridges-v1";
    const MINE: &str = "MnIronVentilation";
    const ART: &str = "mn_iron_vent_rock";
    for (pack, hash) in [
        (
            current,
            "74a0fbdaee301653502d0d4e93eac4e24600dcc34617b88e3e860a76565bbe6e",
        ),
        (
            common,
            "d1977ad46aeb9734e8c268d857e01c169a031a8abdc3a935e88a60420ce67b9c",
        ),
        (
            latest,
            "c021bf20b9ff9f6354a6e92a9ed8035eb91533a91a4c590819902431a7bf3734",
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
    let mut pack = current.clone();
    let mut changed = 0;
    for (name, bytes) in &latest.runtime_files {
        if common.runtime_files.get(name) == Some(bytes) {
            continue;
        }
        ensure!(
            name.contains(ART)
                || name == "data/tilesets/mn_north_shore.json"
                || name == "data/tilesets/mn_north_shore_metatiles.bin",
            "unrelated donor art: {name}"
        );
        ensure!(
            current.runtime_files.get(name) == common.runtime_files.get(name),
            "current native art preimage changed: {name}"
        );
        pack.runtime_files.insert(name.clone(), bytes.clone());
        changed += 1;
    }
    ensure!(changed == 7, "exact seven native art changes required");
    ensure!(
        common
            .runtime_files
            .keys()
            .all(|name| latest.runtime_files.contains_key(name)),
        "donor removed native art"
    );
    for name in [ART, "mn_north_shore"] {
        adopt_entry(
            &mut pack.data.tilesets,
            &common.data.tilesets,
            &latest.data.tilesets,
            name,
        )?;
    }
    for name in ["MnR11C00R00", "MnR11C00R02", "MnR11C00R03"] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            name,
        )?;
    }
    let name = "MnR11C00R01";
    let original = common.data.maps.get(name).context("common bridge map")?;
    let donor = latest.data.maps.get(name).context("accepted bridge map")?;
    let room = pack.data.maps.get_mut(name).context("current bridge map")?;
    ensure!(
        room.blocks == original.blocks,
        "current bridge terrain changed"
    );
    let mut expected = original.clone();
    expected.blocks = donor.blocks.clone();
    ensure!(&expected == donor, "donor changed bridge map scripts");
    room.blocks = donor.blocks.clone();
    // The comprehensive fishing pass already fixed the old generic reference.
    // The accepted dry ventilation successor uses the Minnesota lake reference.
    let mut mine = current
        .data
        .maps
        .get(MINE)
        .context("current ventilation")?
        .clone();
    ensure!(
        mine.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_SHORE"),
        "current regional fishing preimage"
    );
    mine.attributes.fishing_group = Some("FISHGROUP_SHORE".into());
    let original = common.data.maps.get(MINE).context("common ventilation")?;
    ensure!(
        mine.attributes == original.attributes && mine.blocks == original.blocks,
        "unreviewed current ventilation terrain changes"
    );
    let donor = latest.data.maps.get(MINE).context("accepted ventilation")?;
    let mut expected_donor = original.clone();
    expected_donor.attributes = donor.attributes.clone();
    expected_donor.blocks = donor.blocks.clone();
    ensure!(
        &expected_donor == donor,
        "donor changed ventilation programs or actors"
    );
    ensure!(
        donor.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_LAKE"),
        "accepted ventilation fishing reference"
    );
    // Preserve the comprehensive supporting dialogue and its cached commands.
    mine.attributes = donor.attributes.clone();
    mine.blocks = donor.blocks.clone();
    pack.data.maps.insert(MINE.into(), mine);
    let mut attributes = current
        .data
        .map_attributes
        .get(MINE)
        .context("current raw ventilation attributes")?
        .clone();
    ensure!(
        attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_SHORE"),
        "raw regional fishing preimage"
    );
    attributes.fishing_group = Some("FISHGROUP_SHORE".into());
    ensure!(
        Some(&attributes) == common.data.map_attributes.get(MINE),
        "unreviewed raw ventilation changes"
    );
    pack.data.map_attributes.insert(
        MINE.into(),
        latest
            .data
            .map_attributes
            .get(MINE)
            .context("accepted raw ventilation attributes")?
            .clone(),
    );
    for name in [
        "MnIronVentilation_Blocks",
        "MnR11C00R00_Blocks",
        "MnR11C00R01_Blocks",
        "MnR11C00R02_Blocks",
        "MnR11C00R03_Blocks",
    ] {
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
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
