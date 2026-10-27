//! Adapt the existing source dossier without duplicating its geometry or actors.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_town_design::Town;

pub(super) fn parse(town: &str, depot: &str) -> Result<(Town, Value)> {
    let mut raw: Value = serde_json::from_str(town).context("St Paul dossier")?;
    let depot: Value = serde_json::from_str(depot).context("St Paul depot")?;
    ensure!(
        raw["id"] == "MnStPaul"
            && raw["interiors"].as_array().context("rooms")?.len() == 4
            && depot["id"] == "MnStPaulUnionDepot"
            && depot["dimensions_metatiles"] == json!([16, 12])
            && depot["public_arrival_core"] == json!([15, 21]),
        "St Paul city/depot identity and inventory"
    );
    let mut rooms = raw["interiors"].as_array().context("rooms")?.clone();
    for room in &mut rooms {
        room["border_block"] = json!(u8::from_str_radix(
            room["border_block"].as_str().unwrap_or("00"),
            16
        )?);
        room["exits"] = room["exit_warps"].clone();
        if room.get("source_reference").is_some() {
            room["template"] = json!("SaffronGym");
            room["rows"] = json!([]);
            room["residents"] = json!([]);
        }
        adapt_actors(&mut room["residents"])?;
    }
    let mut station = depot.clone();
    station["template"] = json!("GoldenrodMagnetTrainStation");
    station["border_block"] = json!(u8::from_str_radix(
        depot["border_block"].as_str().context("depot border")?,
        16
    )?);
    station["passive_arrival"] = json!({"index":5,"tile":[15,21]});
    station["exits"] = Value::Array(
        depot["exit_tiles_core"]
            .as_array()
            .context("depot carpets")?
            .iter()
            .enumerate()
            .map(|(i, tile)| json!({"index":i+1,"tile":tile,"target":"MnStPaul","target_warp":2}))
            .collect(),
    );
    adapt_actors(&mut station["residents"])?;
    rooms.push(station);
    raw["interiors"] = Value::Array(rooms);
    // Shared native Door.arrival_tile denotes the interior passive arrival;
    // the older city dossier uses that field for its exterior safe return.
    let source_rooms = raw["interiors"].clone();
    for door in raw["doors"].as_array_mut().context("facades")? {
        let room = source_rooms
            .as_array()
            .context("rooms")?
            .iter()
            .find(|r| r["id"] == door["target"])
            .context("paired room")?;
        door["arrival_tile"] = room["passive_arrival"]["tile"].clone();
    }
    raw["doors"]
        .as_array_mut()
        .context("facades")?
        .sort_by_key(|d| d["warp_index"].as_u64());
    Ok((
        serde_json::from_value(raw.clone()).context("native St Paul contracts")?,
        raw,
    ))
}

fn adapt_actors(actors: &mut Value) -> Result<()> {
    for actor in actors.as_array_mut().context("room actors")? {
        if let Some(approach) = actor.get("interaction_approach").cloned() {
            actor["approach"] = approach;
            actor["counter_tile"] = actor["native_counter"].clone();
        }
        if actor["service"] == "native_mart" {
            actor["service"] = json!("pokemart");
            ensure!(
                actor["stock"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty()),
                "St Paul mart requires authored stock"
            );
        }
    }
    Ok(())
}
