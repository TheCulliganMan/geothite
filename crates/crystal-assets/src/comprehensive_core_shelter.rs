//! Preserve the comprehensive campaign while adopting the accepted world delta.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;
use crate::northstar_story_refresh::preserve_unrevised_rows;

const MARKER: &str = "comprehensive-core-shelter-v1";
const ART: &str = "mn_iron_core_rock";

/// Integrates the accepted pump/core, route guidance, Game Corner and shelter.
///
/// Preserves the current campaign, all regional cries and other audio, and
/// unrelated native assets. The four fishing mirrors use the accepted regional
/// lake binding. Minneapolis retains its newer badge-gate text commands.
/// Existing packs and saves are never changed or migrated.
///
/// # Errors
/// Rejects different input identities, stale preimages, unexpected donor fields
/// or assets, repeated integration, invalid runtime data or regional geography.
pub fn integrate_comprehensive_core_shelter(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, hash) in [
        (
            current,
            "e0814d35357b488a07959c080746420e30cdaf9d3e1c8d117d97ea96b72611c8",
        ),
        (
            common,
            "c021bf20b9ff9f6354a6e92a9ed8035eb91533a91a4c590819902431a7bf3734",
        ),
        (
            latest,
            "01e986be28fb7e6fa88ffad3182b70da75541af92a8820c4f5183ee85438527a",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact reviewed integration inputs required"
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
    // Reconstruct the entire donor data first: no unhandled donor change may be
    // silently omitted while assembling a campaign successor.
    let mut expected = common.clone();
    apply_data(&mut expected, common, latest)?;
    ensure!(
        expected.data == latest.data,
        "unhandled accepted-world data change"
    );
    drop(expected);
    let mut pack = current.clone();
    apply_data(&mut pack, common, latest)?;
    let expected_art = [
        format!("data/tilesets/{ART}.json"),
        format!("data/tilesets/{ART}_metatiles.bin"),
        format!("data/tilesets/{ART}_palette_map.json"),
        format!("gfx/tilesets/{ART}.2bpp"),
        format!("gfx/tilesets/{ART}.png"),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let changed_art = common
        .runtime_files
        .keys()
        .chain(latest.runtime_files.keys())
        .filter(|name| common.runtime_files.get(*name) != latest.runtime_files.get(*name))
        .cloned()
        .collect::<BTreeSet<_>>();
    ensure!(changed_art == expected_art, "unreviewed native art delta");
    for name in changed_art {
        ensure!(
            current.runtime_files.get(&name) == common.runtime_files.get(&name),
            "native art preimage changed: {name}"
        );
        pack.runtime_files.insert(
            name.clone(),
            latest
                .runtime_files
                .get(&name)
                .context("accepted art")?
                .clone(),
        );
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

fn adopt_changed<T: Clone + Eq>(
    target: &mut BTreeMap<String, T>,
    common: &BTreeMap<String, T>,
    latest: &BTreeMap<String, T>,
) -> Result<()> {
    let keys = common
        .keys()
        .chain(latest.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in keys {
        if common.get(&key) == latest.get(&key) {
            continue;
        }
        ensure!(
            target.get(&key) == common.get(&key),
            "stale accepted entry: {key}"
        );
        if let Some(value) = latest.get(&key) {
            target.insert(key, value.clone());
        } else {
            target.remove(&key);
        }
    }
    Ok(())
}

fn apply_data(
    pack: &mut CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<()> {
    for name in [
        "MnMinneapolisCornerHome",
        "MnR12Cave1F",
        "MnR15C00R00",
        "MnR17C00R01",
    ] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            name,
        )?;
    }
    for name in ["MnIronDrillCore", "MnIronPumpGallery"] {
        let before = common.data.maps.get(name).context("common mine room")?;
        let donor = latest.data.maps.get(name).context("accepted mine room")?;
        let room = pack.data.maps.get_mut(name).context("current mine room")?;
        let mut attributes = room.attributes.clone();
        normalize_fishing(&mut attributes.fishing_group)?;
        ensure!(
            attributes == before.attributes && room.blocks == before.blocks,
            "mine terrain preimage changed: {name}"
        );
        room.attributes = donor.attributes.clone();
        room.blocks.clone_from(&donor.blocks);
        if name == "MnIronDrillCore" {
            ensure!(
                room.script_movements == before.script_movements,
                "drill movement preimage changed"
            );
            room.script_movements.clone_from(&donor.script_movements);
            for label in ["MnNDrillWatchFrom19", "MnNDrillWatchFrom20"] {
                adopt_entry(&mut room.scripts, &before.scripts, &donor.scripts, label)?;
            }
        }
        let raw = pack
            .data
            .map_attributes
            .get_mut(name)
            .context("raw mine attributes")?;
        let mut normalized = raw.clone();
        normalize_fishing(&mut normalized.fishing_group)?;
        ensure!(
            Some(&normalized) == common.data.map_attributes.get(name),
            "raw mine preimage changed"
        );
        raw.clone_from(
            latest
                .data
                .map_attributes
                .get(name)
                .context("accepted raw mine attributes")?,
        );
    }
    let name = "MnMinneapolis";
    let before = common.data.maps.get(name).context("common city")?;
    let donor = latest.data.maps.get(name).context("accepted city")?;
    let city = pack.data.maps.get_mut(name).context("current city")?;
    ensure!(
        city.events == before.events
            && city.map_event_section_commands == before.map_event_section_commands,
        "city entrance preimage changed"
    );
    city.events.clone_from(&donor.events);
    city.map_event_section_commands
        .clone_from(&donor.map_event_section_commands);
    let changed = donor
        .scripts
        .keys()
        .chain(before.scripts.keys())
        .filter(|key| donor.scripts.get(*key) != before.scripts.get(*key))
        .cloned()
        .collect::<BTreeSet<_>>();
    ensure!(
        changed == BTreeSet::from(["MnCornerSign".into(), "MnCornerSignText".into()]),
        "unreviewed city programs"
    );
    adopt_changed(&mut city.scripts, &before.scripts, &donor.scripts)?;
    adopt_changed(
        &mut city.script_text_bodies,
        &before.script_text_bodies,
        &donor.script_text_bodies,
    )?;
    if city.script_text_commands == before.script_text_commands {
        city.script_text_commands
            .clone_from(&donor.script_text_commands);
    } else {
        let current_rows = city
            .script_text_commands
            .iter()
            .filter(|row| changed.contains(&row.source_script))
            .collect::<Vec<_>>();
        let before_rows = before
            .script_text_commands
            .iter()
            .filter(|row| changed.contains(&row.source_script))
            .collect::<Vec<_>>();
        ensure!(
            current_rows == before_rows,
            "city sign cache preimage changed"
        );
        let mut revised = donor.script_text_commands.clone();
        preserve_unrevised_rows(&mut revised, &city.script_text_commands, &changed, |row| {
            &row.source_script
        });
        city.script_text_commands = revised;
    }
    let name = "MnR16C00R00";
    let before = common
        .data
        .maps
        .get(name)
        .context("common earth-bank route")?;
    let donor = latest
        .data
        .maps
        .get(name)
        .context("accepted earth-bank route")?;
    let room = pack
        .data
        .maps
        .get_mut(name)
        .context("current earth-bank route")?;
    ensure!(
        room.scripts == before.scripts && room.script_text_bodies == before.script_text_bodies,
        "earth-bank guidance preimage changed"
    );
    room.scripts.clone_from(&donor.scripts);
    room.script_text_bodies
        .clone_from(&donor.script_text_bodies);
    adopt_entry(
        &mut pack.data.map_attributes,
        &common.data.map_attributes,
        &latest.data.map_attributes,
        "MnMinneapolisCornerHome",
    )?;
    adopt_changed(
        &mut pack.data.map_blocks,
        &common.data.map_blocks,
        &latest.data.map_blocks,
    )?;
    adopt_changed(
        &mut pack.data.map_dimensions,
        &common.data.map_dimensions,
        &latest.data.map_dimensions,
    )?;
    adopt_changed(
        &mut pack.data.map_scripts,
        &common.data.map_scripts,
        &latest.data.map_scripts,
    )?;
    adopt_changed(&mut pack.data.npcs, &common.data.npcs, &latest.data.npcs)?;
    adopt_changed(
        &mut pack.data.runtime_map_metadata,
        &common.data.runtime_map_metadata,
        &latest.data.runtime_map_metadata,
    )?;
    adopt_changed(
        &mut pack.data.story_event_script_constants.global,
        &common.data.story_event_script_constants.global,
        &latest.data.story_event_script_constants.global,
    )?;
    adopt_entry(
        &mut pack.data.tilesets,
        &common.data.tilesets,
        &latest.data.tilesets,
        ART,
    )?;
    Ok(())
}

fn normalize_fishing(group: &mut Option<String>) -> Result<()> {
    ensure!(
        matches!(
            group.as_deref(),
            Some("FISHGROUP_SHORE" | "FISHGROUP_MN_SHORE")
        ),
        "unreviewed mine fishing preimage"
    );
    *group = Some("FISHGROUP_SHORE".into());
    Ok(())
}
