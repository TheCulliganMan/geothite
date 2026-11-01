//! Frozen hand-authored Rainy River terrain and native activity contracts.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounterTable;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_route_design::{Door, Interior, Pickup, Resident, RouteTrainer};
use crate::authored_route_layout::Layout;

pub(crate) const OUTDOOR: &str = "MnRainyRiverIcefields";
pub(crate) const HUT: &str = "MnRainyRiverWarmingHut";
pub(crate) const OUTDOOR_CONSTANT: &str = "MN_RAINY_RIVER_ICEFIELDS";
pub(crate) const HUT_CONSTANT: &str = "MN_RAINY_RIVER_WARMING_HUT";
pub(crate) const NEIGHBOR: &str = "MnR20C00R00";
pub(crate) const GROUP: &str = "MINNESOTA_RAINY_RIVER_ICE";
pub(crate) const GROUP_CONSTANT: &str = "MAPGROUP_MINNESOTA_RAINY_RIVER_ICE";
pub(crate) const GROUP_ID: u16 = 84;
pub(crate) const ALIAS: &str = "mn_winter_outdoor";
pub(crate) const NEIGHBOR_HASH: &str =
    "9002397e9453ccfddbf992afa99d1912ca50e660530ce0c56bb5cc63c8de8ed4";
pub(crate) const SOURCE_BLOCKS: &str =
    "718fcae45a4ade9bef073c5577c6e40ba1aa28a4ba6e1441b3abac55fc1ec057";
const LAYOUT_HASH: &str = "0e474ee47f0f3fdea0fcd3c32f0f5683d1f206222ce74457214f29f7017b1c78";
const DOSSIER_HASH: &str = "9f795c5da670c0db655906d7d046fd4c6862805b790209a40f3731e7d4f42019";

#[derive(Deserialize)]
pub(crate) struct Region {
    pub id: String,
    pub map_constant: String,
    pub dimensions_metatiles: [u16; 2],
    pub tileset: String,
    pub environment: String,
    pub music: String,
    pub arrival_core: [u16; 2],
    pub trainers: Vec<RouteTrainer>,
    pub residents: Vec<Resident>,
    pub items: Vec<Pickup>,
    pub doors: Vec<Door>,
    pub interiors: Vec<Interior>,
    pub encounters: Encounters,
}

#[derive(Deserialize)]
pub(crate) struct Encounters {
    pub grass_rates: BTreeMap<String, u8>,
    pub grass: WildEncounterTable,
    pub water_rate: Option<u8>,
    pub water: Option<WildEncounterTable>,
}

pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn parse(layout_json: &str, dossier_json: &str) -> Result<(Layout, Region)> {
    ensure!(
        hash(layout_json.as_bytes()) == LAYOUT_HASH
            && hash(dossier_json.as_bytes()) == DOSSIER_HASH,
        "exact frozen Rainy River source inputs required"
    );
    let raw: Value = serde_json::from_str(layout_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    let region: Region = serde_json::from_str(dossier_json)?;
    ensure!(
        layout.schema_version == 2
            && layout.id == OUTDOOR
            && layout.dimensions_metatiles == [32, 24]
            && layout.compiled_tileset == ALIAS
            && raw["source_pixels_modified"] == false
            && raw
                .get("derived_blocks")
                .is_none_or(|v| v.as_array().is_some_and(Vec::is_empty))
            && region.id == OUTDOOR
            && region.map_constant == OUTDOOR_CONSTANT
            && region.dimensions_metatiles == [32, 24]
            && region.tileset == ALIAS
            && region.environment == "ROUTE"
            && region.music == "MUSIC_LAKE_OF_RAGE"
            && region.arrival_core == [60, 44]
            && region.trainers.len() == 2
            && region.residents.len() == 1
            && region.items.len() == 1
            && region.doors.len() == 1
            && region.interiors.len() == 1,
        "bounded Rainy River terrain/activity contract"
    );
    let room = region.interiors.first().context("warming shelter")?;
    let door = region.doors.first().context("warming shelter door")?;
    ensure!(
        room.id == HUT
            && room.template == "VioletNicknameSpeechHouse"
            && room.dimensions_metatiles == [5, 4]
            && room.exits == [[5, 7]]
            && room.clear_arrival == Some([5, 6])
            && room
                .passive_arrival
                .as_ref()
                .is_some_and(|a| a.index == 2 && a.tile == [5, 6])
            && room.residents.len() == 1
            && room.items.is_empty()
            && room.signs.is_empty()
            && room.keeper_tile.is_none()
            && door.warp_index == 1
            && door.return_warp_index == Some(2)
            && door.tile == [9, 27]
            && door.arrival_tile == [9, 28]
            && door.return_tile == [9, 28]
            && door.target == HUT
            && door.target_warp == 2,
        "exact active and passive shelter pairing required"
    );
    ensure!(
        region.encounters.water.is_none() && region.encounters.water_rate.is_none(),
        "frozen backwater has ICE, not native Surf Water"
    );
    for actor in region.residents.iter().chain(&room.residents) {
        ensure!(
            !actor.lines.is_empty()
                && actor
                    .lines
                    .iter()
                    .all(|s| !s.is_empty() && s.chars().count() <= 18),
            "resident dialogue must fit native text"
        );
    }
    Ok((layout, region))
}
