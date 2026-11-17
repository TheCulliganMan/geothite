//! Native floorplan and encounter revision over the selected Gym and cave pack.
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::northstar_iron_encounters::Encounters;
use crate::northstar_iron_terrain::{Relocation, Terrain};
use crate::{CompiledGamePack, LoadedCompiledGamePack};

const MANIFEST: &str = "northstar-iron-workings-revision-v1";
const COMPATIBILITY: &str = "data/northstar-save-compatibility.json";
const ROOMS: [&str; 6] = [
    "MnIronDrillApproach",
    "MnIronHaulage",
    "MnIronSump",
    "MnIronVentilation",
    "MnIronPumpGallery",
    "MnIronDrillCore",
];

/// Builds the new floorplans and mine from the exact v04 locally selected parents.
/// The source root is the repository's Minnesota planning directory.
///
/// # Errors
/// Rejects a mismatched parent, unreadable room recipes, unreachable landmarks,
/// broken trainer approaches, or a change to an inherited event identity.
pub fn revise_iron_workings(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    root: &Path,
) -> Result<CompiledGamePack> {
    ensure!(
        raw.save_modpack_identity()?.hash()
            == "11b81fcc2e06bab49de4b95f3a4211bb270872fcafbbeed5fbfad9021f143834"
            && raw.pack_content_hash()?
                == "1cd664f35594c693a87a68e25c2430a0ad24a814cd1959bf198678dd340c8149"
            && hosted.save_modpack_identity()?.hash()
                == "c0a19e3336c77920a119c7996601aca7f3962cc9b56eca84087ca1d29b1e335a"
            && hosted.pack_content_hash()?
                == "fd71e9f9231a031d5931caf559faeee0fa2287a8cb6e6686ccd3d156f8850e41",
        "expected the exact v04 Gym and cave raw/hosted pair"
    );
    let dir = root.join("lairs/iron-revision-20261117");
    let terrain = Terrain::parse(&fs::read_to_string(dir.join("terrain.json"))?)?;
    let encounters = Encounters::parse(
        &fs::read_to_string(dir.join("discoveries.json"))?,
        &fs::read_to_string(dir.join("sightlines.json"))?,
    )?;
    let story: Value = serde_json::from_slice(&fs::read(root.join("story.json"))?)?;
    let mut pack = raw.pack().clone();
    let mut moves = Vec::new();
    for id in ROOMS {
        let mut room = raw
            .pack()
            .data
            .maps
            .get(id)
            .context("installed Iron Range map")?
            .clone();
        moves.extend(
            terrain
                .apply(&pack, &mut room)
                .with_context(|| format!("floorplan for {id}"))?,
        );
        encounters
            .install(&mut pack, &mut room)
            .with_context(|| format!("encounters in {id}"))?;
        crate::northstar_iron_dialogue::install(
            &pack,
            &mut room,
            &story["iron_workings_revision"]["dialogue"],
        )?;
        crate::northstar_story_revision::publish(&mut pack, room)?;
    }
    let arrivals = crate::northstar_floorplans::install(&mut pack, root, &story)?;
    crate::northstar_regional_guidance::install(&mut pack, &story)?;
    let sightlines = fs::read_to_string(root.join("npcs/regional-sight-20261117.json"))?;
    crate::northstar_regional_sight::install(&mut pack, &sightlines, &story)?;
    let route_source = fs::read_to_string(root.join("routes/exploration-20261117/routes.json"))?;
    crate::northstar_route_exploration::install(&mut pack, &route_source)?;
    // Reinstalling private Gym graphics republishes their existing markers.
    // Keep their original positions: their order is part of the public save ID.
    let mut installed = pack.report.manifests.clone();
    let mut inherited = raw.pack().report.manifests.clone();
    installed.sort();
    inherited.sort();
    ensure!(
        installed == inherited,
        "private floorplan art changed inherited manifests"
    );
    pack.report
        .manifests
        .clone_from(&raw.pack().report.manifests);
    pack.report.manifests.push(MANIFEST.into());
    install_compatibility(&mut pack, raw, hosted, &moves, &arrivals)?;
    for (id, value) in &raw.pack().data.story_event_script_constants.global {
        ensure!(
            pack.data.story_event_script_constants.global.get(id) == Some(value),
            "saved event changed: {id}"
        );
    }
    ensure!(
        pack.data.trainers == raw.pack().data.trainers
            && pack.data.wild_encounters == raw.pack().data.wild_encounters
            && pack.audio_manifest == raw.pack().audio_manifest,
        "retained parties, encounters and native audio"
    );
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

fn install_compatibility(
    pack: &mut CompiledGamePack,
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    moves: &[Relocation],
    arrivals: &[(String, [u16; 2])],
) -> Result<()> {
    let mut metadata: Value = serde_json::from_slice(
        raw.pack()
            .runtime_files
            .get(COMPATIBILITY)
            .context("v04 compatibility metadata")?,
    )?;
    ensure!(
        metadata["schema_version"] == 2
            && metadata["sources"].as_array().is_some_and(|s| s.len() == 6),
        "original v04 save policy"
    );
    let mut sources = Vec::new();
    for parent in [raw, hosted] {
        let id = parent.save_modpack_identity()?;
        sources.push(json!({"modpack_id":id.id(),"modpack_hash":id.hash(),"content_hash":parent.pack_content_hash()?}));
    }
    sources.extend(
        metadata["sources"]
            .as_array()
            .context("previous releases")?
            .iter()
            .cloned(),
    );
    let raw_id = pack.runtime_modpack_id()?;
    let hosted_id = format!(
        "{raw_id}+{}+{}",
        crate::REALTIME_CLOCK_MANIFEST_ID,
        crate::PLAYER_CUSTOMIZATION_MANIFEST_ID
    );
    metadata["schema_version"] = 3.into();
    metadata["target_manifest"] = MANIFEST.into();
    metadata["target_modpack_ids"] = json!([raw_id, hosted_id]);
    metadata["sources"] = sources.into();
    metadata["iron_relocations"] = serde_json::to_value(moves)?;
    metadata["layout_reentry"] = json!(arrivals.iter().map(|(id, _)| id).collect::<Vec<_>>());
    for (id, [x, y]) in arrivals {
        let policy = if id.ends_with("Gym") {
            "gym_reentry"
        } else {
            "cave_reentry"
        };
        let entry = metadata[policy]["maps"]
            .as_array_mut()
            .context("inherited room reentry list")?
            .iter_mut()
            .find(|row| row["map_id"] == id.as_str())
            .context("known room reentry policy")?;
        entry["arrival"] = json!({"x":x,"y":y});
    }
    pack.runtime_files
        .insert(COMPATIBILITY.into(), serde_json::to_vec(&metadata)?);
    Ok(())
}

#[cfg(test)]
#[path = "northstar_floorplan_tests.rs"]
mod tests;
