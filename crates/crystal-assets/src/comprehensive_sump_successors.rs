//! Integrate the accepted native sump floor and R11/R13 successors.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;

/// Carries the exact accepted sump and route changes into the review game.
///
/// Uses Minnesota lake fishing for the optional flooded sump basin. Only its
/// private native art is added. Existing story,
/// optional content, roster and audio remain authoritative; saves are unchanged.
///
/// # Errors
/// Rejects different identities, stale preimages, unrelated donor art changes,
/// repeated integration, invalid runtime content or inconsistent geography.
pub fn integrate_comprehensive_sump_successors(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    const MARKER: &str = "comprehensive-sump-route-successors-v1";
    const MINE: &str = "MnIronSump";
    const ART: &str = "mn_iron_sump_rock";
    for (pack, hash) in [
        (
            current,
            "06a5a0d158c4ed49ef87c6995bd0845d17218d7f00aca35bb1eac3604b8c1b5c",
        ),
        (
            common,
            "c68542065bac76644016febec0ff6217ecf1ac446caa247a1579d3cd024a9594",
        ),
        (
            latest,
            "d1977ad46aeb9734e8c268d857e01c169a031a8abdc3a935e88a60420ce67b9c",
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
    for name in ["MnR11C00R03", "MnR13C00R02"] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            name,
        )?;
    }
    // The comprehensive fishing pass already fixed the old generic reference.
    // The accepted flooded successor uses MN_LAKE for its optional fishing bank.
    let mut mine = current.data.maps.get(MINE).context("current sump")?.clone();
    ensure!(
        mine.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_SHORE"),
        "current regional fishing preimage"
    );
    mine.attributes.fishing_group = Some("FISHGROUP_SHORE".into());
    let original = common.data.maps.get(MINE).context("common sump")?;
    ensure!(
        mine.attributes == original.attributes && mine.blocks == original.blocks,
        "unreviewed current sump terrain changes"
    );
    let donor = latest.data.maps.get(MINE).context("accepted sump")?;
    let mut expected_donor = original.clone();
    expected_donor.attributes = donor.attributes.clone();
    expected_donor.blocks = donor.blocks.clone();
    ensure!(
        &expected_donor == donor,
        "donor changed sump programs or actors"
    );
    ensure!(
        donor.attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_LAKE"),
        "accepted flooded-basin fishing reference"
    );
    // Preserve the comprehensive supporting dialogue and its cached commands.
    mine.attributes = donor.attributes.clone();
    mine.blocks = donor.blocks.clone();
    pack.data.maps.insert(MINE.into(), mine);
    let mut attributes = current
        .data
        .map_attributes
        .get(MINE)
        .context("current raw sump attributes")?
        .clone();
    ensure!(
        attributes.fishing_group.as_deref() == Some("FISHGROUP_MN_SHORE"),
        "raw regional fishing preimage"
    );
    attributes.fishing_group = Some("FISHGROUP_SHORE".into());
    ensure!(
        Some(&attributes) == common.data.map_attributes.get(MINE),
        "unreviewed raw sump changes"
    );
    pack.data.map_attributes.insert(
        MINE.into(),
        latest
            .data
            .map_attributes
            .get(MINE)
            .context("accepted raw sump attributes")?
            .clone(),
    );
    for name in ["MnIronSump_Blocks", "MnR13C00R02_Blocks"] {
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
            name,
        )?;
    }
    for name in ["MN_R11_LESTER_SIGNText", "MN_R11_PALISADE_SIGNText"] {
        adopt_entry(
            &mut pack.data.map_scripts,
            &common.data.map_scripts,
            &latest.data.map_scripts,
            name,
        )?;
    }
    // R11 retains current fields outside this one reviewed sign and its caches.
    let name = "MnR11C00R01";
    let label = "MN_R11_PALISADE_SIGNText";
    let original = common.data.maps.get(name).context("common R11 sign map")?;
    let donor = latest
        .data
        .maps
        .get(name)
        .context("accepted R11 sign map")?;
    let room = pack
        .data
        .maps
        .get_mut(name)
        .context("current R11 sign map")?;
    adopt_entry(&mut room.scripts, &original.scripts, &donor.scripts, label)?;
    adopt_entry(
        &mut room.script_text_bodies,
        &original.script_text_bodies,
        &donor.script_text_bodies,
        label,
    )?;
    ensure!(
        room.script_text_commands == original.script_text_commands
            && donor.script_text_commands != original.script_text_commands,
        "stale R11 text-command cache"
    );
    room.script_text_commands = donor.script_text_commands.clone();
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
