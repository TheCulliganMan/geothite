//! Prairie Road's real badge gate and Mankato's acknowledged survey notebook.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-prairie-story-v1";
const BASE: &str = "09dbcb239adf58805875b478ed837924f0084aa8ee85a215982231dabd752cae";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Chapter {
    schema_version: u8,
    manifest: String,
    base_content_hash: String,
    links: Vec<Link>,
    notebook: Notebook,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Link {
    pub town: String,
    pub town_direction: String,
    pub route_direction: String,
    pub offset: i32,
    pub mouth_metatile: [u16; 2],
    pub town_edge: [[u16; 2]; 2],
    pub route_edge: [[u16; 2]; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Notebook {
    pub map: String,
    pub id: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub approach: [u16; 2],
    pub milestone: String,
    pub hint_actor: String,
}

/// Connects Rochester, Prairie Farm Road and Mankato on the reviewed story pack.
///
/// Only Care is required to leave Rochester westward. Return travel is free.
/// Reading both notebook pages records MS02; the optional clue grants no item,
/// money or badge and does not gate the second gym. R03 remains unconnected.
/// Existing story, gyms, route terrain/encounters, art, audio and recovery stay
/// intact. Installing the overlay awards no player progress.
///
/// # Errors
/// Returns an error for stale or duplicated input, changed chapter contracts,
/// missing native flags, unsafe geometry, invalid scripts or native seam/pack
/// verification failure. The borrowed input remains unchanged on failure.
pub fn build_prairie_story_modpack(
    base: &CompiledGamePack,
    chapter_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let chapter: Chapter = serde_json::from_str(chapter_json).context("prairie chapter")?;
    ensure!(
        chapter.schema_version == 1
            && chapter.manifest == MANIFEST
            && chapter.base_content_hash == BASE
            && base.identity.content_hash == BASE
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact reviewed regional story input required; duplicate or stale chapter"
    );
    let mut maps = ["MnRochester", "MnR02C00R00", "MnMankato"]
        .into_iter()
        .map(|name| {
            Ok((
                name.to_owned(),
                base.data
                    .maps
                    .get(name)
                    .with_context(|| format!("installed {name}"))?
                    .clone(),
            ))
        })
        .collect::<Result<BTreeMap<String, MapModule>>>()?;
    let mut pack = base.clone();
    crate::prairie_notebook::install(
        &mut pack,
        maps.get_mut("MnMankato").context("Mankato")?,
        &chapter.notebook,
        &serde_json::from_str(story_json)?,
    )?;
    crate::prairie_story_connections::install(base, &mut maps, &chapter.links)?;
    for (_, mut map) in maps {
        authoring::finish(&pack, &mut map)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_variable_commands =
            crate::parse_script_variable_commands(&map.id, &map.scripts)?;
        map.script_movements =
            crate::parse_script_movements(&map.id, &map.scripts, &map.script_object_commands)?;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    crate::prairie_story_connections::verify(&pack, &chapter.links)?;
    pack.report.manifests.push(MANIFEST.into());
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
