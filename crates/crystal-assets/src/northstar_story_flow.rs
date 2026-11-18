//! Publish the revised Minnesota investigation, trainer conversations and aftermath.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::{CompiledGamePack, LoadedCompiledGamePack};

const MANIFEST: &str = "northstar-story-flow-v1";
const COMPATIBILITY: &str = "data/northstar-save-compatibility.json";
const FLOORPLAN_RAW: &str = "2e2554784f82f6b9e53c8b1d7a24e5993f730eefb460239d3edc4bc7d558a3df";
const FLOORPLAN_HOSTED: &str = "de7379234cf30325688436bd2c846b7f00e23830f0125d762ea962ad304fddb4";

#[derive(Deserialize)]
struct Source {
    story_flow_revision: Revision,
}

#[derive(Deserialize)]
struct Revision {
    dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

/// Revises the floorplan release using the native dialogue in Minnesota's story file.
///
/// # Errors
/// Returns an error for a different parent pair, invalid pages, unexpected
/// script preimages, incompatible prior save metadata or an invalid result.
pub fn revise_northstar_story_flow(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    root: &Path,
) -> Result<CompiledGamePack> {
    let raw_id = raw.save_modpack_identity()?;
    let hosted_id = hosted.save_modpack_identity()?;
    ensure!(
        raw_id.hash() == FLOORPLAN_RAW && hosted_id.hash() == FLOORPLAN_HOSTED,
        "the story revision needs the released raw and hosted floorplans"
    );
    let source: Source = serde_json::from_slice(&fs::read(root.join("story.json"))?)?;
    let mut pack = raw.pack().clone();
    for (name, labels) in source.story_flow_revision.dialogue {
        ensure!(name.starts_with("Mn"), "expected a Minnesota map: {name}");
        let map = pack
            .data
            .maps
            .get_mut(&name)
            .with_context(|| format!("story map {name}"))?;
        for (label, pages) in labels {
            let mut program = authored_dialogue::compile(&pages)
                .with_context(|| format!("story page {name}/{label}"))?;
            if let Some(old) = map.scripts.get(&label) {
                ensure!(
                    map.script_text_bodies.contains_key(&label),
                    "cannot replace executable script {name}/{label} with speech"
                );
                let end = old
                    .as_array()
                    .and_then(|commands| commands.last())
                    .context("text terminator")?;
                ensure!(
                    matches!(end["command"].as_str(), Some("done" | "prompt")),
                    "unexpected terminator {name}/{label}"
                );
                *program
                    .as_array_mut()
                    .and_then(|cmds| cmds.last_mut())
                    .context("compiled text terminator")? = end.clone();
            }
            map.scripts.insert(label, program);
        }
    }
    crate::northstar_story_south::apply(&mut pack)?;
    crate::northstar_story_north::apply(&mut pack)?;
    crate::northstar_story_trainers::apply(&mut pack)?;
    let changed: Vec<_> = pack
        .data
        .maps
        .iter()
        .filter(|(id, map)| raw.pack().data.maps[*id].scripts != map.scripts)
        .map(|(id, _)| id.clone())
        .collect();
    for id in changed {
        let mut room = pack.data.maps[&id].clone();
        crate::ely_league_rooms::finish(&pack, &mut room)
            .with_context(|| format!("rebuild story indices for {id}"))?;
        room.script_audio_commands = crate::parse_script_audio_commands(&id, &room.scripts)?;
        // This revision edits script programs only. Keep the exact native event table,
        // including source-specific rows that the generic room publisher normalizes.
        room.map_event_section_commands
            .clone_from(&raw.pack().data.maps[&id].map_event_section_commands);
        crate::authored_town_publication::replace(&mut pack, room)
            .with_context(|| format!("publish story changes for {id}"))?;
    }
    pack.report.manifests.push(MANIFEST.into());
    let mut metadata: Value = serde_json::from_slice(
        pack.runtime_files
            .get(COMPATIBILITY)
            .context("floorplan save compatibility policy")?,
    )?;
    ensure!(
        metadata["schema_version"] == 3
            && metadata["sources"].as_array().is_some_and(|v| v.len() == 8),
        "inherited floorplan compatibility"
    );
    let sources = metadata["sources"]
        .as_array_mut()
        .context("compatible sources")?;
    sources.splice(0..0, [
        json!({"modpack_id":raw_id.id(),"modpack_hash":raw_id.hash(),"content_hash":raw.pack_content_hash()?}),
        json!({"modpack_id":hosted_id.id(),"modpack_hash":hosted_id.hash(),"content_hash":hosted.pack_content_hash()?}),
    ]);
    let id = pack.runtime_modpack_id()?;
    metadata["schema_version"] = 4.into();
    metadata["target_manifest"] = MANIFEST.into();
    metadata["target_modpack_ids"] = json!([
        id,
        format!(
            "{id}+{}+{}",
            crate::REALTIME_CLOCK_MANIFEST_ID,
            crate::PLAYER_CUSTOMIZATION_MANIFEST_ID
        )
    ]);
    pack.runtime_files
        .insert(COMPATIBILITY.into(), serde_json::to_vec(&metadata)?);
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
