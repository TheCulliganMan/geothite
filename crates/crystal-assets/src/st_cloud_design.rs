//! Adapt the authored St Cloud city and separate public station contracts.
use crate::authored_town_design::Town;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

pub(super) fn parse(source: &str, station: &str) -> Result<(Town, Value)> {
    let mut raw: Value = serde_json::from_str(source).context("St Cloud city")?;
    let mut station: Value = serde_json::from_str(station).context("St Cloud station")?;
    ensure!(
        raw["id"] == "MnStCloud"
            && raw["interiors"].as_array().context("rooms")?.len() == 4
            && station["id"] == "MnStCloudStation"
            && station["dimensions_metatiles"] == json!([10, 10])
            && station["passive_arrival"]["tile"] == json!([11, 17]),
        "St Cloud city/station identity"
    );
    station["border_block"] = json!(0x0f);
    for room in raw["interiors"].as_array_mut().context("rooms")? {
        if room["id"] == "MnStCloudStation" {
            *room = station.clone();
        }
        if room["id"] == "MnStCloudGym" {
            ensure!(
                room.get("rows").is_none() && room.get("residents").is_none(),
                "no competing city-owned Gym5 geometry or actors"
            );
            room["template"] = json!("VioletGym");
            room["rows"] = json!([]);
            room["residents"] = json!([]);
        }
        if room.get("border_block").is_none() {
            room["border_block"] = json!(0);
        }
        for actor in room["residents"].as_array().context("room residents")? {
            ensure!(
                actor["service"] != "canonical_pokemart_pending",
                "St Cloud mart must declare native service and stock"
            );
            if actor["service"] == "pokemart" {
                ensure!(
                    actor["stock"]
                        .as_array()
                        .is_some_and(|stock| !stock.is_empty()),
                    "authored St Cloud mart stock required"
                );
            }
        }
    }
    let rooms = raw["interiors"].clone();
    for door in raw["doors"].as_array_mut().context("public facades")? {
        let room = rooms
            .as_array()
            .context("rooms")?
            .iter()
            .find(|room| room["id"] == door["target"])
            .context("paired public room")?;
        door["arrival_tile"] = room["passive_arrival"]["tile"].clone();
    }
    Ok((
        serde_json::from_value(raw.clone()).context("native St Cloud contracts")?,
        raw,
    ))
}
