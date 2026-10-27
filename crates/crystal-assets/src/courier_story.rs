//! St Paul's courier confrontation and separately acknowledged shipping record.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const BASE: &str = "f98c5446760288142f6e56727f910b585d9dc67cbd9eaec5674c4bc542960821";
const MANIFEST: &str = "minnesota-courier-story-v1";
pub(crate) const DEPOT: &str = "MnStPaulUnionDepot";
pub(crate) const WIN: &str = "EVENT_MN_COURIER_DEFEATED";
pub(crate) const READ: &str = "EVENT_MN_DEEP_BORE_MANIFEST_READ";
pub(crate) const TRAINER: &str = "MN_LODE_COURIER";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Chapter {
    schema_version: u8,
    manifest: String,
    base_content_hash: String,
    pub town: String,
    pub route: String,
    pub offset: i32,
    pub mouth_metatile: [u16; 2],
    pub town_edge: [[u16; 2]; 2],
    pub route_edge: [[u16; 2]; 2],
    pub courier: Actor,
    pub record: Record,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Actor {
    pub map: String,
    pub id: String,
    pub tile: [u16; 2],
    pub approach: [u16; 2],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub id: String,
    pub tile: [u16; 2],
    pub approach: [u16; 2],
}

/// Connects Minneapolis/R04 and binds the existing Union Depot courier.
///
/// Current alone permits eastward departure. The courier separately requires
/// Current and the recovered ICE dispatch copy; an actual win records defeat.
/// Reading every manifest page records its own clue without a bag item or road
/// permission. The canonical Magnemite23/Drowzee22 party, native trainer money
/// and blackout recovery use the production engine. All other maps, encounters,
/// gyms, story constants, art, audio, HOME and recovery are preserved.
///
/// # Errors
/// Returns an error for stale/duplicate inputs, altered chapter contracts,
/// occupied anchors, invalid trainer identities, unsafe seams or failed script,
/// storage or runtime verification. The borrowed input stays unchanged.
pub fn build_courier_story_modpack(
    base: &CompiledGamePack,
    chapter_json: &str,
    story_json: &str,
    route_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let chapter: Chapter = serde_json::from_str(chapter_json)?;
    ensure!(
        chapter.schema_version == 1
            && chapter.manifest == MANIFEST
            && chapter.base_content_hash == BASE
            && base.identity.content_hash == BASE
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact River Valley story input required; duplicate/stale courier chapter"
    );
    let story: Value = serde_json::from_str(story_json)?;
    let mut pack = base.clone();
    crate::courier_trainer::install(&mut pack, &story)?;
    let mut depot = base
        .data
        .maps
        .get(DEPOT)
        .context("installed Union Depot")?
        .clone();
    crate::courier_events::install(&pack, &mut depot, &chapter, &story)?;
    let mut maps = crate::courier_connections::install(base, &chapter, route_json)?;
    maps.push(depot);
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect();
    for mut map in maps {
        authoring::finish(&pack, &mut map)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_variable_commands =
            crate::parse_script_variable_commands(&map.id, &map.scripts)?;
        map.script_movements =
            crate::parse_script_movements(&map.id, &map.scripts, &map.script_object_commands)?;
        map.trainer_scripts = crate::parse_trainer_scripts(&map.id, &map.scripts)?;
        map.scripted_trainer_battles =
            crate::parse_scripted_trainer_battles(&map.id, &map.scripts)?;
        map.script_map_commands = crate::parse_script_map_commands(&map.id, &map.scripts, &names)?;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    crate::courier_connections::verify(&pack)?;
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
