//! Connected survey-cave rock spurs without reinstalling existing programs.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_cave_revision;
use crate::authored_route_design::Interior;
use crate::authored_route_modpack;

const ROOM: &str = "MnR10Cave1F";

/// Reshapes the survey cave's broad northern hall into a connected bedrock spur
/// and a bent supply hollow. Preserves all five actors, optional battles,
/// rewards, original programs/events, stairs, drops, spring, exterior mouth,
/// dimensions, native art/audio and every other map. Only the complete block
/// array, its uniquely owned encoded payload and derived identity change.
///
/// # Errors
/// Rejects stale native/source preimages, changed retained source fields,
/// original event anchors, stairs or water/drop blocks; disconnected or unsafe
/// geometry; shared or inconsistent payloads; and invalid resulting packs.
pub fn revise_r10_survey_spurs(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Interior = serde_json::from_str(before_json)?;
    let new: Interior = serde_json::from_str(after_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base.data.maps.get(ROOM).context("installed survey cave")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && old.id == ROOM
            && old.dimensions_metatiles == [24, 18]
            && original.attributes.width == 24
            && original.attributes.height == 18
            && original.attributes.tileset_name == "cave"
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && original.attributes.border_block == 0x2e
            && original.blocks == authored_cave_revision::blocks(&old)?,
        "complete installed survey cave preimage differs"
    );
    let mut before: Value = serde_json::from_str(before_json)?;
    let mut after: Value = serde_json::from_str(after_json)?;
    for source in [&mut before, &mut after] {
        for field in ["rows", "purpose", "candidate_status", "review_contract"] {
            source
                .as_object_mut()
                .context("complete survey source")?
                .remove(field);
        }
    }
    ensure!(before == after, "retained survey source field changed");
    ensure!(
        original.events.warps.len() == 1
            && original.events.warps[0].x == 3
            && original.events.warps[0].y == 13
            && original.events.coord_events.is_empty()
            && original.events.bg_events.is_empty()
            && original.objects.len() == 5
            && old.exits == vec![[3, 13]]
            && old.clear_arrival == Some([3, 12]),
        "original survey mouth and five actor anchors required"
    );
    for (id, x, y) in [
        ("MN_R10_GALLERY_READER", 9, 5),
        ("MN_R10_ITEM_04", 29, 23),
        ("MN_R10_ESCAPE_ROPE", 39, 5),
        ("MN_EXTRA_R10_SURVEY", 18, 15),
        ("MN_EXTRA_R10_SPRING", 33, 24),
    ] {
        ensure!(
            original
                .objects
                .iter()
                .any(|o| o.script == id && o.x == x && o.y == y),
            "original survey anchor changed: {id}"
        );
    }
    let mut room = original.clone();
    room.blocks = authored_cave_revision::blocks(&new)?;
    ensure!(room.blocks != original.blocks, "survey terrain unchanged");
    let collision = &base.data.tilesets["cave"].collision;
    for (index, block) in original.blocks.iter().enumerate() {
        let protected = matches!(block, 0x30 | 0x36)
            || collision[&format!("{block:02x}")]
                .iter()
                .any(|kind| matches!(kind.as_str(), "WATER" | "LADDER" | "HOP_DOWN"));
        ensure!(
            !protected || room.blocks[index] == *block,
            "original stair, mouth, water and drop blocks must remain"
        );
    }
    // The existing optional opponents and supply ball are already installed.
    // Project their bodies into geometry only; never regenerate their programs.
    let mut geometry: Value = serde_json::from_str(after_json)?;
    for (id, sprite, tile) in [
        ("MN_EXTRA_R10_SURVEY", "SPRITE_POKEFAN_M", [18, 15]),
        ("MN_EXTRA_R10_SPRING", "SPRITE_FISHER", [33, 24]),
    ] {
        geometry["residents"]
            .as_array_mut()
            .context("survey residents")?
            .push(json!({"id":id,"sprite":sprite,"tile":tile,"facing":"south","lines":["Retain native text."]}));
    }
    geometry["items"]
        .as_array_mut()
        .context("survey caches")?
        .push(json!({"id":"MN_R10_ESCAPE_ROPE","item":"ESCAPE_ROPE","quantity":1,"tile":[39,5],"flag":"EVENT_MN_R10_ESCAPE_ROPE"}));
    let geometry: Interior = serde_json::from_value(geometry)?;
    authored_route_modpack::validate_room(base, &room, &geometry)?;
    authored_cave_revision::validate_terrain(base, &room, &geometry)?;
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("survey block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(id, map)| id == ROOM || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared survey block payload"
    );
    ensure!(
        crate::decode_base64_bytes(&base.data.map_blocks[label])?
            .into_iter()
            .map(u16::from)
            .eq(original.blocks.iter().copied()),
        "survey split preimage differs"
    );
    let bytes = room
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut pack = base.clone();
    pack.data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    pack.data.maps.insert(ROOM.into(), room);
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
