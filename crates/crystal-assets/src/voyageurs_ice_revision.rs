//! Revise both existing ice puzzles without reinstalling their game programs.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{Design, Room, geometry};
use crate::CompiledGamePack;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    rooms: Vec<Binding>,
    purpose: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: String,
    native_module_sha256: String,
}

/// Reshapes the installed two-floor Voyageurs ice dungeon using native terrain.
/// Exact dimensions, art, actors, programs, rewards and every warp are retained;
/// only the two block arrays/payloads and derived pack identity change.
///
/// Native sliding validates the destinations and return routes for every local
/// pickup-body combination. Initial/collected item positions cannot create a
/// one-way ice trap. This validates mechanics, not earned regional access or
/// visual quality; production input and native image review remain necessary.
///
/// # Errors
/// Rejects stale complete modules, source/geometry drift, changed retained
/// fields, unsafe ice routes/perimeters or invalid resulting packs.
pub fn revise_voyageurs_ice_turns(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Design = serde_json::from_str(before)?;
    let new: Design = serde_json::from_str(after)?;
    let contract: Contract = serde_json::from_str(contract)?;
    let mut old_value: Value = serde_json::from_str(before)?;
    let mut new_value: Value = serde_json::from_str(after)?;
    for value in [&mut old_value, &mut new_value] {
        for room in value["rooms"].as_array_mut().context("two ice sources")? {
            let fields = room.as_object_mut().context("ice room source")?;
            for field in ["rows", "purpose", "review_viewpoints"] {
                fields.remove(field);
            }
        }
    }
    ensure!(
        old_value == new_value
            && old.schema_version == 1
            && old.tileset == "ice_path"
            && old.id == "minnesota-voyageurs-ice-v1"
            && old
                .rooms
                .iter()
                .map(|r| r.id.as_str())
                .eq(["MnVoyageursIce1F", "MnVoyageursIceB1F"])
            && new.rooms.len() == 2
            && contract.schema_version == 1
            && contract.rooms.len() == 2
            && !contract.purpose.is_empty(),
        "ice revision changes retained authoring fields"
    );
    let mut result = base.clone();
    for ((old_room, new_room), binding) in old.rooms.iter().zip(&new.rooms).zip(&contract.rooms) {
        ensure!(
            old_room.id == new_room.id
                && binding.id == old_room.id
                && matches!(
                    old_room.id.as_str(),
                    "MnVoyageursIce1F" | "MnVoyageursIceB1F"
                ),
            "both installed Voyageurs ice rooms required"
        );
        let original = base
            .data
            .maps
            .get(&old_room.id)
            .context("installed ice room")?;
        ensure!(
            binding.native_module_sha256
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
                && original.attributes.tileset_name == "ice_path"
                && original.attributes.border_block == old.border_block
                && [original.attributes.width, original.attributes.height]
                    == old_room.dimensions_metatiles
                && original.blocks == blocks(old_room)?,
            "complete ice room preimage differs: {}",
            old_room.id
        );
        let mut revised = original.clone();
        revised.blocks = blocks(new_room)?;
        ensure!(
            revised.blocks != original.blocks,
            "both ice floors must change"
        );
        let mut projection = new_value["rooms"].clone();
        let room_index = if old_room.id == "MnVoyageursIce1F" {
            0
        } else {
            1
        };
        projection[room_index]["rows"] = json!(new_room.rows);
        projection[room_index]["residents"] = json!(
            original
                .objects
                .iter()
                .filter(|o| o.object_type != "OBJECTTYPE_ITEMBALL")
                .map(|o| json!({"id":o.script,"tile":[o.x,o.y],"sprite":o.sprite,
                "lines":["Native program retained."]}))
                .collect::<Vec<_>>()
        );
        let geometry_room: Room = serde_json::from_value(projection[room_index].clone())?;
        let pickups = revised
            .objects
            .iter()
            .filter(|o| o.object_type == "OBJECTTYPE_ITEMBALL")
            .map(|o| o.script.clone())
            .collect::<Vec<_>>();
        ensure!(pickups.len() <= 3, "bounded ice pickup combinations");
        for mask in 0..(1_usize << pickups.len()) {
            let mut state = revised.clone();
            state.objects.retain(|o| {
                !pickups
                    .iter()
                    .enumerate()
                    .any(|(bit, id)| o.script == *id && mask & (1 << bit) != 0)
            });
            geometry::validate(base, &state, &geometry_room)?;
            validate_current_returns(base, &state, new_room.arrival)?;
        }
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("ice blocks label")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "shared ice block payload"
        );
        let bytes = revised
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            crate::decode_base64_bytes(
                base.data
                    .map_blocks
                    .get(label)
                    .context("ice block payload")?
            )?
            .iter()
            .copied()
            .map(u16::from)
            .eq(original.blocks.iter().copied()),
            "ice split payload preimage differs"
        );
        result
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
        result.data.maps.insert(new_room.id.clone(), revised);
    }
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn blocks(room: &Room) -> Result<Vec<u16>> {
    let [width, height] = room.dimensions_metatiles;
    ensure!(
        room.rows.len() == usize::from(height),
        "complete ice room height"
    );
    let mut blocks = Vec::new();
    for row in &room.rows {
        let cells = row.split_whitespace().collect::<Vec<_>>();
        ensure!(cells.len() == usize::from(width), "complete ice room width");
        for cell in cells {
            blocks.push(u16::from_str_radix(cell, 16).context("native ice metatile")?);
        }
    }
    Ok(blocks)
}

fn validate_current_returns(
    pack: &CompiledGamePack,
    map: &crate::MapModule,
    arrival: [u16; 2],
) -> Result<()> {
    use crystal_core::world::map::Direction;
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let states = geometry::reachable(pack, map, arrival, &bodies)?;
    let mut returning = BTreeSet::from([arrival]);
    loop {
        let before = returning.len();
        for &at in &states {
            for direction in [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ] {
                if returning.contains(&geometry::slide(pack, map, at, direction, &bodies)?) {
                    returning.insert(at);
                }
            }
        }
        if returning.len() == before {
            break;
        }
    }
    ensure!(
        states == returning,
        "ice trap with current pickup bodies in {}",
        map.id
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_ICE_TURNS_PARENT and authored sources at runtime"]
    fn ice_turns_preserve_programs_warps_and_reject_stale_or_open_sources() {
        let path = std::env::var_os("CRYSTAL_ICE_TURNS_PARENT").expect("current ice parent");
        let base = crate::read_verified_compiled_game_pack(PathBuf::from(path)).expect("parent");
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/ice-caves/turning-chambers-v01");
        let read = |name| std::fs::read_to_string(root.join(name)).expect("ice source");
        let before = read("before.json");
        let after = read("after.json");
        let contract = read("contract.json");
        let new = super::revise_voyageurs_ice_turns(&base, &before, &after, &contract)
            .expect("native two-floor turning chambers");
        for (name, old) in &base.data.maps {
            let mut expected = serde_json::to_value(old).expect("native map");
            if matches!(name.as_str(), "MnVoyageursIce1F" | "MnVoyageursIceB1F") {
                expected["blocks"] =
                    serde_json::to_value(&new.data.maps[name].blocks).expect("rows");
            }
            assert_eq!(
                expected,
                serde_json::to_value(&new.data.maps[name]).expect("child map")
            );
        }
        let mut stale: serde_json::Value = serde_json::from_str(&contract).expect("contract");
        stale["rooms"][0]["native_module_sha256"] = "0".into();
        assert!(
            super::revise_voyageurs_ice_turns(&base, &before, &after, &stale.to_string()).is_err()
        );
        let mut unsafe_source: serde_json::Value = serde_json::from_str(&after).expect("source");
        unsafe_source["rooms"][0]["rows"][0] = "1f 3b 3b 3b 3b 3b 3b 3b 3b 3b 3b 3b 3b 3b".into();
        assert!(
            super::revise_voyageurs_ice_turns(
                &base,
                &before,
                &unsafe_source.to_string(),
                &contract
            )
            .is_err()
        );
    }
}
