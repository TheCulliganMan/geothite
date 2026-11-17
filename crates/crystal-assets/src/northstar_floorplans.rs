//! A second pass on three Gyms, the Voyageurs glacier and Granite Den.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::gym_design::GymDesign;
use crate::northstar_cave_design::Cave;
use crate::northstar_gym_design::RoomContract;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IceRooms {
    schema_version: u8,
    rooms: Vec<Cave>,
}

pub(super) fn install(
    pack: &mut CompiledGamePack,
    root: &Path,
    story: &Value,
) -> Result<Vec<(String, [u16; 2])>> {
    let mut arrivals = Vec::new();
    let mut gym_story = story.clone();
    if let Some(room_texts) = story["floorplan_revision"]["dialogue"].as_object() {
        for (room, labels) in room_texts {
            let current = gym_story["gym_overhaul"]["dialogue"][room]
                .as_object_mut()
                .context("existing Gym story")?;
            current.extend(labels.as_object().context("revised room dialogue")?.clone());
        }
    }
    let gym_story = serde_json::to_string(&gym_story)?;
    let dir = root.join("gyms/floorplans-20261117");
    for (slot, name, id) in [
        (1, "rochester", "MnRochesterGym"),
        (4, "st-paul", "MnStPaulGym"),
        (6, "brainerd", "MnBrainerdGym"),
    ] {
        let doc: GymDesign = serde_json::from_slice(&fs::read(dir.join(format!("{name}.json")))?)?;
        let [gym] = doc.gyms.as_slice() else {
            anyhow::bail!("expected one Gym per source");
        };
        let contract: RoomContract =
            serde_json::from_slice(&fs::read(dir.join(format!("{name}-contract.json")))?)?;
        ensure!(
            doc.schema_version == 1 && gym.id == id && gym.slot == slot,
            "expected {name} Gym"
        );
        crate::northstar_gym_design::validate_room(gym, &contract)?;
        crate::northstar_gym_rooms::install(pack, gym, &contract, &gym_story)
            .with_context(|| format!("{id} floorplan"))?;
        arrivals.push((gym.id.clone(), gym.arrival_core));
    }
    let source: IceRooms = serde_json::from_slice(&fs::read(
        root.join("caves/floorplans-20261117/rooms.json"),
    )?)?;
    ensure!(
        source.schema_version == 1
            && source.rooms.len() == 2
            && source
                .rooms
                .iter()
                .map(|r| r.id.as_str())
                .collect::<BTreeSet<_>>()
                == BTreeSet::from(["MnVoyageursIce1F", "MnVoyageursIceB1F"]),
        "two Voyageurs ice floors required"
    );
    let granite: Cave = serde_json::from_slice(&fs::read(
        root.join("caves/floorplans-20261117/granite-den.json"),
    )?)?;
    ensure!(
        granite.id == "MnR17Cave1F" && granite.dimensions_metatiles == [12, 9],
        "Granite Den floorplan and extent"
    );
    for cave in source.rooms.into_iter().chain([granite]) {
        let mut room = pack
            .data
            .maps
            .get(&cave.id)
            .context("installed regional cavern")?
            .clone();
        ensure!(
            room.attributes.tileset_name == cave.tileset
                && crate::northstar_cave_overhaul::preimage(&room)? == cave.preimage_sha256,
            "cavern source preimage differs: {}",
            cave.id
        );
        if cave.id != "MnR17Cave1F" {
            ensure!(
                [room.attributes.width, room.attributes.height] == cave.dimensions_metatiles,
                "retain the linked ice floors' bounds"
            );
        }
        [room.attributes.width, room.attributes.height] = cave.dimensions_metatiles;
        room.blocks = cave.blocks()?;
        crate::northstar_cave_overhaul::validate_native_art(pack, &room)?;
        for (id, tile) in &cave.actor_positions {
            let actor = room
                .objects
                .iter_mut()
                .find(|a| a.object_identifier.as_ref() == Some(id))
                .context("retained cavern actor")?;
            [actor.x, actor.y] = *tile;
        }
        for cache in &cave.hidden_items {
            let event = room
                .events
                .bg_events
                .iter_mut()
                .find(|e| e.script == cache.id && e.event_type == "BGEVENT_ITEM")
                .context("retained cavern cache")?;
            ensure!(room.script_field_pickups.iter().any(|p|p.source_script==cache.id && p.item_id.as_ref()==Some(&cache.item)), "retained cavern reward receipt");
            [event.x, event.y] = cache.tile;
        }
        if let Some(labels) = story["floorplan_revision"]["cave_dialogue"][&room.id].as_object() {
            for (label, raw) in labels {
                ensure!(
                    room.script_text_bodies.contains_key(label)
                        && room
                            .scripts
                            .get(label)
                            .is_some_and(|text| text.as_array().is_some_and(|cmds| cmds
                                .iter()
                                .all(|c| matches!(
                                    c["command"].as_str(),
                                    Some("text" | "line" | "para" | "cont" | "done")
                                )))),
                    "retained plain room text {label}"
                );
                let pages: Vec<crate::authored_dialogue::AuthoredDialoguePage> =
                    serde_json::from_value(raw.clone())?;
                room.scripts
                    .insert(label.to_owned(), crate::authored_dialogue::compile(&pages)?);
            }
        }
        crate::northstar_cave_geometry::validate(pack, &room, &cave)
            .with_context(|| format!("cavern route in {}", room.id))?;
        crate::northstar_story_revision::publish(pack, room)?;
        arrivals.push((cave.id, cave.arrival));
    }
    Ok(arrivals)
}
