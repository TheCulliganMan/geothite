//! Eight-room overlay on the exact published refinement, with bounded save entry.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::northstar_gym_design::{MAPS, Revision};
use crate::{CompiledGamePack, LoadedCompiledGamePack};

const MANIFEST: &str = "northstar-gym-overhaul-v1";
const COMPATIBILITY: &str = "data/northstar-save-compatibility.json";
const PARENT_CONTENT: &str = "a9f23e3bc1dadad74c377d02c23df61dcf9ed70c1b1d5e3918805593f37a9527";
const PARENT_FILE: &str = "5b173130deed7dc9c2af97b4bc2509e4f54f848c207361ff5278e21abce513ad";
const HOSTED_CONTENT: &str = "5eba128f5fd1871d87a58bfca28833476591708c6a6ad6d04075ed29a6f087e4";
const HOSTED_FILE: &str = "5d82fb89116f82856f8cc1cef77ceac018d2863f756ad8e7bbb7bbf589cc6c70";

/// Redesign the eight Gyms without replaying stale regional builders.
///
/// Exact refinement source identities retain their checksum-bound save lineage.
/// Geometry migration is performed separately by the runtime only on safe saves.
/// This builder never reads or writes player saves.
///
/// # Errors
/// Rejects different parents, incomplete designs, invalid native geometry or
/// parties, missing canonical dialogue, and changes outside the declared overlay.
pub fn overhaul_northstar_gyms(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    design: &str,
    story: &str,
    pokemon: &str,
    caves: Option<&str>,
) -> Result<CompiledGamePack> {
    let base = raw.pack();
    ensure!(
        raw.pack_content_hash()? == PARENT_CONTENT
            && raw.save_modpack_identity()?.hash() == PARENT_FILE,
        "Gym overhaul requires exact selected refinement v02"
    );
    ensure!(
        hosted.pack_content_hash()? == HOSTED_CONTENT
            && hosted.save_modpack_identity()?.hash() == HOSTED_FILE,
        "Gym overhaul requires exact published refinement v02"
    );
    let revision: Revision = serde_json::from_str(design)?;
    revision.validate()?;
    let mut candidate = base.clone();
    crate::northstar_gym_teams::install(&mut candidate, pokemon)?;
    for gym in &revision.rooms {
        crate::northstar_gym_rooms::install(
            &mut candidate,
            gym,
            &revision.contracts[&gym.id],
            story,
        )
        .with_context(|| format!("redesigned {}", gym.id))?;
    }
    if let Some(caves) = caves {
        crate::northstar_cave_overhaul::install(&mut candidate, caves, story)?;
    }
    crate::northstar_shore_roster_repair::repair_existing(&mut candidate, story)?;
    candidate.report.manifests.push(MANIFEST.into());
    install_compatibility(&mut candidate, raw, hosted, &revision, caves.is_some())?;
    preserve_unrelated(base, &candidate, caves.is_some())?;
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

fn install_compatibility(
    pack: &mut CompiledGamePack,
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    revision: &Revision,
    caves: bool,
) -> Result<()> {
    let old: Value = serde_json::from_slice(
        raw.pack()
            .runtime_files
            .get(COMPATIBILITY)
            .context("refinement save lineage")?,
    )?;
    let mut sources = Vec::new();
    for loaded in [raw, hosted] {
        let identity = loaded.save_modpack_identity()?;
        sources.push(
            json!({"modpack_id":identity.id(), "modpack_hash":identity.hash(),
            "content_hash":loaded.pack_content_hash()?}),
        );
    }
    ensure!(
        old["schema_version"] == 1 && old["target_manifest"] == "northstar-refinement-v1",
        "approved refinement lineage schema"
    );
    let previous = old["sources"].as_array().context("refinement sources")?;
    ensure!(previous.len() == 4, "four exact earlier releases");
    sources.extend(previous.iter().cloned());
    let raw_id = pack.runtime_modpack_id()?;
    let hosted_id = format!(
        "{raw_id}+{}+{}",
        crate::REALTIME_CLOCK_MANIFEST_ID,
        crate::PLAYER_CUSTOMIZATION_MANIFEST_ID
    );
    let maps: Vec<_> = revision.rooms.iter().map(|gym| json!({
        "map_id":gym.id, "arrival":{"x":gym.arrival_core[0],"y":gym.arrival_core[1]}, "facing":"down",
    })).collect();
    let mut metadata = json!({
        "schema_version":2, "target_manifest":MANIFEST, "target_modpack_ids":[raw_id,hosted_id],
        "sources":sources, "gym_reentry":{"policy":"minnesota-eight-gyms-v1","maps":maps},
    });
    if caves {
        let maps: Vec<_> = crate::northstar_cave_design::MAPS
            .into_iter()
            .zip(crate::northstar_cave_design::ARRIVALS)
            .map(|(map, [x, y])| {
                json!({
                    "map_id":map,"arrival":{"x":x,"y":y},"facing":"down",
                })
            })
            .collect();
        metadata["cave_reentry"] = json!({"policy":"minnesota-eight-caves-v1","maps":maps});
    }
    pack.runtime_files
        .insert(COMPATIBILITY.into(), serde_json::to_vec(&metadata)?);
    Ok(())
}

fn preserve_unrelated(
    before: &CompiledGamePack,
    after: &CompiledGamePack,
    caves: bool,
) -> Result<()> {
    ensure!(
        before.data.maps.len() == after.data.maps.len(),
        "preserve map set"
    );
    // Restore the allowed maps through their production publication adapter so
    // the split scripts/NPC/block/dimension catalogs are checked as well.
    let mut restored = after.clone();
    for id in MAPS.into_iter().chain(["MnWinona", "MnLeechLake"]).chain(
        crate::northstar_cave_design::MAPS
            .into_iter()
            .filter(|_| caves),
    ) {
        crate::authored_town_publication::replace(&mut restored, before.data.maps[id].clone())?;
    }
    let mut checked = restored.data;
    if caves {
        for id in crate::northstar_cave_design::TRAINERS {
            ensure!(
                !before.data.trainers.trainers.contains_key(id),
                "new cave trainer identity"
            );
            checked
                .trainers
                .trainers
                .remove(id)
                .context("installed new cave trainer")?;
        }
    }
    for slot in 1..=8 {
        let tileset = format!("mn_gym_{slot}");
        checked.tilesets.insert(
            tileset,
            before.data.tilesets[&format!("mn_gym_{slot}")].clone(),
        );
        for id in std::iter::once(format!("MN_GYM_{slot}")).chain(
            (1..=if slot >= 7 { 3 } else { 2 })
                .map(|index| format!("MN_GYM_{slot}_TRAINER_{index}")),
        ) {
            checked
                .trainers
                .trainers
                .insert(id.clone(), before.data.trainers.trainers[&id].clone());
        }
    }
    // Only the garden's private Cut table is replaced. Existing original tables
    // and every global event/trainer numeric identity must remain stable.
    checked.field_moves.cut.replacements.insert(
        "mn_gym_7".into(),
        before.data.field_moves.cut.replacements["mn_gym_7"].clone(),
    );
    for (name, number) in &before.data.story_event_script_constants.global {
        ensure!(
            after.data.story_event_script_constants.global.get(name) == Some(number),
            "saved constant changed: {name}"
        );
    }
    for name in after
        .data
        .story_event_script_constants
        .global
        .keys()
        .filter(|name| {
            !before
                .data
                .story_event_script_constants
                .global
                .contains_key(*name)
        })
    {
        ensure!(
            (name.starts_with("EVENT_MN_GYM_") && name.contains("_V2_"))
                || (caves
                    && (crate::northstar_cave_design::EVENTS.contains(&name.as_str())
                        || crate::northstar_cave_design::TRAINERS.contains(&name.as_str()))),
            "undeclared new constant: {name}"
        );
    }
    checked.story_event_script_constants.global =
        before.data.story_event_script_constants.global.clone();
    ensure!(checked == before.data, "unrelated game data changed");
    let mutable: BTreeSet<_> = (1..=8)
        .flat_map(|slot| {
            [
                format!("data/tilesets/mn_gym_{slot}.json"),
                format!("data/tilesets/mn_gym_{slot}_metatiles.bin"),
                format!("data/tilesets/mn_gym_{slot}_palette_map.json"),
                format!("gfx/tilesets/mn_gym_{slot}.png"),
                format!("gfx/tilesets/mn_gym_{slot}.2bpp"),
            ]
        })
        .chain([COMPATIBILITY.into()])
        .collect();
    ensure!(
        before.runtime_files.keys().eq(after.runtime_files.keys()),
        "runtime asset set changed"
    );
    for (path, bytes) in &before.runtime_files {
        ensure!(
            mutable.contains(path) || after.runtime_files.get(path) == Some(bytes),
            "unrelated asset changed: {path}"
        );
    }
    ensure!(
        before.data.audio == after.data.audio
            && before.audio_manifest == after.audio_manifest
            && before.compiled_audio == after.compiled_audio
            && before.audio_compression == after.audio_compression,
        "existing audio PCM, frames, loops and storage must remain exact"
    );
    Ok(())
}
