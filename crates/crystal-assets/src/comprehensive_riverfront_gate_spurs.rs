//! Preserve the comprehensive campaign while adopting the accepted world delta.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;
use crate::northstar_story_refresh::preserve_unrevised_rows;

const MARKER: &str = "comprehensive-riverfront-gate-spurs-v1";
const ART: &str = "mn_grand_rapids_riverfront";

/// Integrates Grand Rapids riverfront, the Detroit gate and R10 rock spurs.
///
/// Retains the comprehensive campaign and all audio. Grand Rapids command caches
/// merge only the optional fisher's labels, preserving current depot gates.
/// Existing packs and saves are never changed or migrated.
///
/// # Errors
/// Rejects different input identities, stale preimages, unexpected donor fields
/// or assets, repeated integration, invalid runtime data or regional geography.
pub fn integrate_comprehensive_riverfront_gate_spurs(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, hash) in [
        (
            current,
            "2fc5397022b18b6d08fb85c5761a61c0aa956eee5680e718b9df963e8ea3eae5",
        ),
        (
            common,
            "01e986be28fb7e6fa88ffad3182b70da75541af92a8820c4f5183ee85438527a",
        ),
        (
            latest,
            "b828c463da5d05885a90a49b4f822fee5e6d7c2d5f997443143d22bf376face9",
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
    let gate = "MnR19DetroitGate";
    let raw = pack
        .data
        .map_attributes
        .get_mut(gate)
        .context("raw gate attributes")?;
    ensure!(
        raw.fishing_group.as_deref() == Some("FISHGROUP_SHORE"),
        "unreviewed raw gate fishing"
    );
    raw.fishing_group = Some("FISHGROUP_MN_SHORE".into());
    let room = pack.data.maps.get_mut(gate).context("runtime gate")?;
    ensure!(
        room.attributes.fishing_group.as_deref() == Some("FISHGROUP_SHORE"),
        "unreviewed gate fishing"
    );
    room.attributes.fishing_group = Some("FISHGROUP_MN_SHORE".into());

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
    pack.report.maps = pack.data.maps.len();
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

fn adopt_value<T: Clone + Eq>(target: &mut T, common: &T, latest: &T) -> Result<()> {
    ensure!(target == common, "accepted field preimage changed");
    target.clone_from(latest);
    Ok(())
}

fn merge_rows<T: Clone + Eq>(
    target: &mut Vec<T>,
    common: &[T],
    latest: &[T],
    changed: &BTreeSet<String>,
    label: impl Fn(&T) -> &str + Copy,
) -> Result<()> {
    if target == common {
        *target = latest.to_vec();
        return Ok(());
    }
    let selected = |rows: &[T]| {
        rows.iter()
            .filter(|row| changed.contains(label(row)))
            .cloned()
            .collect::<Vec<_>>()
    };
    ensure!(
        selected(target) == selected(common),
        "fisher cache preimage changed"
    );
    let mut revised = latest.to_vec();
    preserve_unrevised_rows(&mut revised, target, changed, label);
    *target = revised;
    Ok(())
}

fn apply_data(
    pack: &mut CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<()> {
    for name in [
        "MnDetroitLakes",
        "MnR10Cave1F",
        "MnR19C00R01",
        "MnR19DetroitGate",
    ] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            name,
        )?;
    }
    let before = common
        .data
        .maps
        .get("MnGrandRapids")
        .context("common Grand Rapids")?;
    let donor = latest
        .data
        .maps
        .get("MnGrandRapids")
        .context("accepted Grand Rapids")?;
    let city = pack
        .data
        .maps
        .get_mut("MnGrandRapids")
        .context("current Grand Rapids")?;
    adopt_value(&mut city.attributes, &before.attributes, &donor.attributes)?;
    adopt_value(&mut city.blocks, &before.blocks, &donor.blocks)?;
    adopt_value(
        &mut city.map_event_section_commands,
        &before.map_event_section_commands,
        &donor.map_event_section_commands,
    )?;
    adopt_value(&mut city.objects, &before.objects, &donor.objects)?;
    adopt_value(
        &mut city.script_map_commands,
        &before.script_map_commands,
        &donor.script_map_commands,
    )?;
    adopt_value(
        &mut city.script_object_commands,
        &before.script_object_commands,
        &donor.script_object_commands,
    )?;
    adopt_value(
        &mut city.script_runtime_commands,
        &before.script_runtime_commands,
        &donor.script_runtime_commands,
    )?;
    adopt_value(
        &mut city.scripted_trainer_battles,
        &before.scripted_trainer_battles,
        &donor.scripted_trainer_battles,
    )?;
    let changed = donor
        .scripts
        .keys()
        .chain(before.scripts.keys())
        .filter(|key| donor.scripts.get(*key) != before.scripts.get(*key))
        .cloned()
        .collect::<BTreeSet<_>>();
    ensure!(
        changed.len() == 6
            && changed
                .iter()
                .all(|label| label.starts_with("MN_EXTRA_GR_BANK_FISHER")),
        "unreviewed riverfront programs"
    );
    adopt_changed(&mut city.scripts, &before.scripts, &donor.scripts)?;
    adopt_changed(
        &mut city.script_text_bodies,
        &before.script_text_bodies,
        &donor.script_text_bodies,
    )?;
    merge_rows(
        &mut city.script_control_commands,
        &before.script_control_commands,
        &donor.script_control_commands,
        &changed,
        |row| &row.source_script,
    )?;
    merge_rows(
        &mut city.script_flag_commands,
        &before.script_flag_commands,
        &donor.script_flag_commands,
        &changed,
        |row| &row.source_script,
    )?;
    merge_rows(
        &mut city.script_text_commands,
        &before.script_text_commands,
        &donor.script_text_commands,
        &changed,
        |row| &row.source_script,
    )?;
    adopt_changed(
        &mut pack.data.map_attributes,
        &common.data.map_attributes,
        &latest.data.map_attributes,
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
    adopt_changed(
        &mut pack.data.tilesets,
        &common.data.tilesets,
        &latest.data.tilesets,
    )?;
    adopt_changed(
        &mut pack.data.pokegear_landmarks.map_to_landmark,
        &common.data.pokegear_landmarks.map_to_landmark,
        &latest.data.pokegear_landmarks.map_to_landmark,
    )?;
    adopt_changed(
        &mut pack.data.trainers.trainers,
        &common.data.trainers.trainers,
        &latest.data.trainers.trainers,
    )?;
    adopt_value(
        &mut pack.data.field_moves,
        &common.data.field_moves,
        &latest.data.field_moves,
    )?;
    Ok(())
}
