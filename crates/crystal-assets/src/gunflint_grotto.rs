//! An expanded native spring circuit with preserved camp care and cache state.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::{self, command as c};

#[path = "gunflint_grotto_geometry.rs"]
mod geometry;

const ROOM: &str = "MnR12Cave1F";
const CACHE: &str = "MN_R12_CAVE_CACHE";
const DISCOVERY: &str = "MN_R12_GROTTO_REVIVE";
const CLAIMED: &str = "EVENT_MN_R12_GROTTO_REVIVE";
const MARKER: &str = "minnesota-gunflint-spring-grotto-v1";

#[cfg(test)]
#[path = "gunflint_grotto_tests.rs"]
mod tests;

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u16,
    palette_group: String,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self, dimensions: [u16; 2]) -> Result<Vec<u16>> {
        ensure!(
            self.schema_version == 1
                && self.id == ROOM
                && self.tileset == "cave"
                && self.dimensions_metatiles == dimensions
                && self.border_block == 46
                && self.palette_group == "nite"
                && self.rows.len() == usize::from(dimensions[1]),
            "complete native night-cave source required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(
                words.len() == usize::from(dimensions[0]),
                "complete cave columns required"
            );
            for word in words {
                let block = u16::from_str_radix(word, 16).context("native cave metatile")?;
                ensure!(block < 64, "original native cave art required");
                blocks.push(block);
            }
        }
        Ok(blocks)
    }
}

/// Expands the Gunflint refuge into a hooked spring grotto with two dry circuits,
/// a deeper Max Ether recess and a Surf-only hidden Revive on a blocking pillar.
/// Retains the original mouth, exterior pair, free rest, cache claim flag, quiet
/// walking floor, lake wildlife, original programs, native art and audio.
/// Existing saved terrain overrides are not migrated.
///
/// # Errors
/// Rejects stale complete-map inputs, unsupported terrain, changed original
/// programs or anchors, stranded ground, unsafe water, dry discovery bypasses,
/// occupied identities, foreign wildlife or an invalid resulting pack.
pub fn rebuild_gunflint_grotto(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Terrain = serde_json::from_str(before_json)?;
    let after: Terrain = serde_json::from_str(after_json)?;
    let source: Value = serde_json::from_str(after_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let old = base.data.maps.get(ROOM).context("current spring refuge")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(old)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&old.events.warps)?
            && old.attributes.width == 16
            && old.attributes.height == 12
            && old.attributes.tileset_name == "cave"
            && old.attributes.border_block == 46
            && old.attributes.time_of_day.as_deref() == Some("nite")
            && old.blocks == before.blocks([16, 12])?
            && old.events.warps.len() == 1
            && old.events.warps[0].x == 17
            && old.events.warps[0].y == 23
            && old.events.coord_events.is_empty()
            && old.events.bg_events.is_empty()
            && old
                .objects
                .iter()
                .map(|o| (o.script.as_str(), o.x, o.y))
                .eq([("MN_R12_OTTA", 5, 5), (CACHE, 27, 5)])
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete current refuge preimage and original anchors required"
    );
    ensure!(
        source["moved_cache"]["after"] == json!([35, 3])
            && source["hidden_item"]["id"] == DISCOVERY
            && source["hidden_item"]["tile"] == json!([28, 16])
            && source["hidden_item"]["item"] == "REVIVE"
            && source["hidden_item"]["quantity"] == 1
            && source["hidden_item"]["flag"] == CLAIMED
            && !base.data.map_scripts.contains_key(DISCOVERY),
        "reviewed cache and pillar discovery required"
    );
    let mut pack = base.clone();
    let mut room = old.clone();
    room.attributes.width = 20;
    room.attributes.height = 14;
    room.blocks = after.blocks([20, 14])?;
    let cache = room
        .objects
        .iter_mut()
        .find(|o| o.script == CACHE)
        .context("native Max Ether")?;
    cache.x = 35;
    cache.y = 3;
    geometry::validate(&pack, &room)?;
    crate::northstar_lair_authoring::flag(&mut pack, CLAIMED)?;
    room.scripts.insert(
        DISCOVERY.into(),
        json!([c("hiddenitem", &["REVIVE", CLAIMED])]),
    );
    room.events.bg_events.push(BackgroundEvent {
        x: 28,
        y: 16,
        event_type: "BGEVENT_ITEM".into(),
        script: DISCOVERY.into(),
    });
    room.script_field_pickups =
        crate::parse_script_field_pickups(ROOM, &room.scripts, &room.objects)?;
    authoring::refresh_event_section(&mut room);
    for (label, body) in &old.scripts {
        ensure!(
            room.scripts.get(label) == Some(body),
            "original cave program changed: {label}"
        );
    }
    validate_wildlife(&pack)?;
    crate::authored_town_publication::replace(&mut pack, room)?;
    for (id, map) in &base.data.maps {
        if id != ROOM {
            ensure!(
                pack.data.maps.get(id) == Some(map),
                "unrelated map changed: {id}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.data.wild_encounters == base.data.wild_encounters
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "all original wildlife, native art and audio must remain exact"
    );
    pack.report.manifests.push(MARKER.into());
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

fn validate_wildlife(pack: &CompiledGamePack) -> Result<()> {
    let roster = crate::northstar_regional_roster::RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )?;
    let lake = pack
        .data
        .wild_encounters
        .get(ROOM)
        .context("native spring wildlife")?;
    ensure!(
        lake.grass_rates.is_none() && lake.grass.is_none() && lake.water_rate == Some(5),
        "quiet refuge floor and active original spring required"
    );
    for table in lake
        .water
        .iter()
        .chain(lake.zones.iter().map(|z| &z.grass))
        .chain(lake.swarm_overrides.values().map(|s| &s.grass))
    {
        for slot in table.morning.iter().chain(&table.day).chain(&table.night) {
            ensure!(
                roster.allowed.contains(&slot.species),
                "spring wildlife must remain regional"
            );
        }
    }
    Ok(())
}
