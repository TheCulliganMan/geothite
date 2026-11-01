//! Reshape the installed Border Pines shelter without reinstalling its scripts.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_cave_revision;
use crate::authored_route_design::Interior;
use crate::authored_route_modpack;

const ROOM: &str = "MnR17Cave1F";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    map: String,
    native_module_sha256: String,
    purpose: String,
}

/// Replaces the shelter's terrain while retaining its original Revive, lichen
/// reader, programs, mouths, dimensions, art, audio and all other content.
/// The complete installed module is bound by the supplied contract. Native
/// geometry includes both actual bodies; source-only previews are insufficient
/// evidence of gameplay or visual acceptance.
///
/// # Errors
/// Rejects stale sources/modules, changed retained fields, unsafe terrain or
/// actor access, shared block payloads and invalid resulting packs.
pub fn revise_r17_talus_loop(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Interior = serde_json::from_str(before)?;
    let mut design: Value = serde_json::from_str(after)?;
    let new: Interior = serde_json::from_value(design.clone())?;
    let contract: Contract = serde_json::from_str(contract)?;
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("installed talus shelter")?;
    ensure!(
        contract.schema_version == 1
            && contract.map == ROOM
            && !contract.purpose.is_empty()
            && contract.native_module_sha256
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && old.id == ROOM
            && old.dimensions_metatiles == [7, 8]
            && original.attributes.width == 7
            && original.attributes.height == 8
            && original.attributes.tileset_name == "cave"
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && original.attributes.border_block == 0x2e
            && original.blocks == authored_cave_revision::blocks(&old)?
            && original.events.warps.len() == 1
            && original.events.bg_events.is_empty()
            && original.events.coord_events.is_empty()
            && original
                .objects
                .iter()
                .map(|o| o.script.as_str())
                .eq(["MN_R17_ITEM_04", "MN_INTERIOR_R17_LICHEN_READER"]),
        "complete installed R17 shelter preimage differs"
    );
    let mut old_value: Value = serde_json::from_str(before)?;
    let mut new_value = design.clone();
    for value in [&mut old_value, &mut new_value] {
        for field in ["rows", "purpose", "candidate_status", "review_contract"] {
            value
                .as_object_mut()
                .context("cave source object")?
                .remove(field);
        }
    }
    ensure!(
        old_value == new_value,
        "retained shelter source field changed"
    );
    ensure!(
        old.exits == vec![[9, 13]] && old.clear_arrival == Some([9, 12]),
        "original mouth required"
    );
    let mut room = original.clone();
    room.blocks = authored_cave_revision::blocks(&new)?;
    ensure!(room.blocks != original.blocks, "shelter terrain unchanged");
    // The lichen reader belongs to a later installed overlay. Project its exact
    // native body into geometry only; retain the original program byte for byte.
    design["residents"]
        .as_array_mut()
        .context("source residents")?
        .push(json!({
            "id":"MN_INTERIOR_R17_LICHEN_READER", "sprite":"SPRITE_SCIENTIST",
            "tile":[6,10], "facing":"south", "lines":["Retain native text."]
        }));
    let geometry: Interior = serde_json::from_value(design)?;
    authored_route_modpack::validate_room(base, &room, &geometry)?;
    authored_cave_revision::validate_terrain(base, &room, &geometry)?;
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("native block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(id, map)| id == ROOM || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared shelter block payload"
    );
    ensure!(
        crate::decode_base64_bytes(&base.data.map_blocks[label])?
            .into_iter()
            .map(u16::from)
            .eq(original.blocks.iter().copied()),
        "split shelter preimage differs"
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
