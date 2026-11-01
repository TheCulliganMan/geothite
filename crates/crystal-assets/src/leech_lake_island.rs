//! A native pine island and separate optional Whirlpool rock cache.
use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounterData;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

#[path = "leech_lake_island_geometry.rs"]
mod geometry;
#[path = "leech_lake_island_scripts.rs"]
mod scripts;

const LAKE: &str = "MnLeechLake";
const MARKER: &str = "minnesota-leech-pine-island-v1";
const TRAINER: &str = "MN_EXTRA_LEECH_ISLAND_NATURALIST";
const ITEM: &str = "MN_LEECH_ISLAND_WATER";
const CLAIMED: &str = "EVENT_MN_LEECH_ISLAND_WATER";
const EDDY: &str = "EVENT_MN_LEECH_EDDY_CACHE";
const HINT: &str = "MN_LEECH_SHORE_FISHERText";

#[cfg(test)]
#[path = "leech_lake_island_tests.rs"]
mod tests;

#[derive(Deserialize)]
struct Island {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    trainer_source: Value,
    encounters: WildEncounterData,
    fisher_hint: Vec<String>,
}

/// Adds a Surf-accessible pine island, regional wildlife, an optional naturalist,
/// a once-only Mystic Water and a separate Whirlpool-controlled hidden PP Up.
/// Public roads, services, original stories and all other maps remain intact.
/// Art and audio bytes are retained; native Whirlpool animation metadata is
/// registered from the supplied pack's original strips. Saves are not migrated.
///
/// # Errors
/// Rejects stale native town preimages, invalid terrain or traversal contracts,
/// foreign or prematurely evolved wildlife, occupied identities, changed native
/// forced-movement source, unrelated content changes or invalid compiled data.
pub fn build_leech_lake_island(
    base: &CompiledGamePack,
    source_json: &str,
    contract_json: &str,
    forced_movement_source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let source: Island = serde_json::from_str(source_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base.data.maps.get(LAKE).context("current lake town")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && source.schema_version == 1
            && source.id == LAKE
            && source.tileset == "mn_headwaters"
            && source.dimensions_metatiles == [32, 24]
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "unmodified current town and fresh island source required"
    );
    let mut pack = base.clone();
    crate::minnesota_field_use_art::install(&mut pack)?;
    scripts::native_current(&mut pack, forced_movement_source)?;
    for flag in [CLAIMED, EDDY] {
        crate::northstar_lair_authoring::flag(&mut pack, flag)?;
    }
    let mut lake = original.clone();
    authoring::blocks(&mut lake, 32, 24, &source.rows.join("\n"))?;
    geometry::validate(&pack, original, &lake)?;
    scripts::discoveries(&pack, &mut lake, &source.fisher_hint)?;
    crate::authored_town_publication::replace(&mut pack, lake)?;
    let encounters = scripts::wildlife(
        &pack,
        source.encounters,
        base.data.wild_encounters.get(LAKE),
    )?;
    pack.data.wild_encounters.insert(LAKE.into(), encounters);
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    let before_trainer = pack.data.maps[LAKE].clone();
    let trainer = &source.trainer_source["trainers"];
    ensure!(
        trainer.as_array().is_some_and(|t| t.len() == 1)
            && trainer[0]["id"] == TRAINER
            && trainer[0]["map"] == LAKE
            && trainer[0]["tile"] == serde_json::json!([53, 30])
            && trainer[0]["approach"] == serde_json::json!([52, 30])
            && trainer[0]["optional_battle"] == true,
        "reviewed optional island naturalist required"
    );
    pack = crate::minnesota_trainers::build_trainers(
        &pack,
        &serde_json::to_string(&source.trainer_source)?,
        MARKER,
        base.data.minnesota_trainer_scaling,
    )?;
    // The standard trainer author refreshes runtime metadata. Retain every
    // pre-existing executable row verbatim and add only this trainer's rows.
    let lake = scripts::preserve_programs(&before_trainer, &pack.data.maps[LAKE])?;
    crate::authored_town_publication::replace(&mut pack, lake)?;
    for (name, map) in &base.data.maps {
        if name != LAKE {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated map changed: {name}"
            );
        }
    }
    for (name, bytes) in &base.runtime_files {
        ensure!(
            pack.runtime_files.get(name) == Some(bytes),
            "original asset changed: {name}"
        );
    }
    ensure!(
        pack.runtime_files.len() == base.runtime_files.len() + 1
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "only native lake animation metadata may extend original art/audio"
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
