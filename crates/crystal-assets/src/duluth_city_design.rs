//! Typed source contracts for the authored Duluth exterior and ordinary rooms.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_route_design::{Door, Facing, PassiveArrival, Sign};
use crate::authored_route_layout::Layout;

#[derive(Deserialize)]
pub(super) struct Town {
    pub id: String,
    pub native_constant: String,
    pub compiled_tileset: String,
    pub dimensions_metatiles: [u16; 2],
    pub ports: Vec<Port>,
    pub residents: Vec<Actor>,
    pub signs: Vec<Sign>,
    pub doors: Vec<Door>,
    pub interiors: Vec<Room>,
}

#[derive(Deserialize)]
pub(super) struct Port {
    pub route: String,
    pub side: String,
    pub gate_metatile: [u16; 2],
    pub landing_metatile: [u16; 2],
}

#[derive(Deserialize)]
pub(super) struct Actor {
    pub id: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub facing: Facing,
    pub lines: Vec<String>,
    pub service: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub sets: Vec<String>,
    #[serde(default)]
    pub repeat_lines: Vec<String>,
    #[serde(default)]
    pub stock: Vec<String>,
    pub approach: Option<[u16; 2]>,
    pub counter_tile: Option<[u16; 2]>,
}

#[derive(Deserialize)]
pub(super) struct Room {
    pub id: String,
    pub native_constant: String,
    pub dimensions_metatiles: [u16; 2],
    pub tileset: String,
    #[serde(default)]
    pub rows: Vec<String>,
    pub passive_arrival: PassiveArrival,
    pub exits: Vec<Exit>,
    #[serde(default)]
    pub residents: Vec<Actor>,
    #[serde(default)]
    pub signs: Vec<Sign>,
    pub source_reference: Option<GymReference>,
    pub pc: Option<Pc>,
}

#[derive(Deserialize)]
pub(super) struct Exit {
    pub index: u16,
    pub tile: [u16; 2],
    pub target: String,
    pub target_warp: i16,
}

#[derive(Deserialize)]
pub(super) struct GymReference {
    pub path: String,
    pub gym_id: String,
    pub room_sha256: String,
    pub symbol_blocks: std::collections::BTreeMap<char, String>,
}

#[derive(Deserialize)]
pub(super) struct Pc {
    pub tile: [u16; 2],
    pub approach: [u16; 2],
}

pub(super) fn parse(town_json: &str, layout_json: &str) -> Result<(Town, Vec<u16>)> {
    let town: Town = serde_json::from_str(town_json).context("Duluth town source")?;
    // Reuse the native typed stroke compiler without extending the route-only
    // Layout::parse allowlist. This city validates its own reserved identity.
    let layout: Layout = serde_json::from_str(layout_json).context("Duluth typed layout")?;
    let raw: serde_json::Value = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnDuluth"
            && town.native_constant == "MN_DULUTH"
            && town.dimensions_metatiles == [32, 24]
            && town.compiled_tileset == "mn_duluth"
            && layout.id == town.id
            && layout.schema_version == 2
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset
            && raw["tileset"] == "johto_modern"
            && raw["shore_fitting"] == "johto_8_neighbors"
            && raw["reject_cliff_cuts"] == true
            && raw["source_pixels_modified"] == false
            && raw["derived_blocks"].as_array().is_some_and(Vec::is_empty),
        "retain complete 32x24 Duluth and unchanged native johto_modern art"
    );
    ensure!(
        town.ports.len() == 2,
        "retain both Duluth route reservations"
    );
    for (port, (route, side, gate, landing)) in town.ports.iter().zip([
        ("MN_R10", "W", [1, 12], [3, 12]),
        ("MN_R11", "N", [16, 1], [16, 3]),
    ]) {
        ensure!(
            port.route == route
                && port.side == side
                && port.gate_metatile == gate
                && port.landing_metatile == landing,
            "changed Duluth {route} port reservation"
        );
    }
    ensure!(
        town.doors.len() == 7 && town.interiors.len() == 7,
        "all seven Duluth door/room pairs required"
    );
    ensure!(
        town.doors
            .iter()
            .map(|d| &d.target)
            .collect::<std::collections::BTreeSet<_>>()
            == town.interiors.iter().map(|r| &r.id).collect(),
        "Duluth needs exactly one facade for every authored room"
    );
    for (room, (name, constant, dimensions)) in town.interiors.iter().zip([
        ("MnDuluthPokecenter1F", "MN_DULUTH_POKECENTER_1F", [10, 8]),
        ("MnDuluthMart", "MN_DULUTH_MART", [6, 4]),
        ("MnDuluthGym", "MN_DULUTH_GYM", [7, 10]),
        (
            "MnDuluthHarborWorkshop",
            "MN_DULUTH_HARBOR_WORKSHOP",
            [8, 6],
        ),
        ("MnDuluthHillHouse", "MN_DULUTH_HILL_HOUSE", [5, 4]),
        ("MnDuluthHarborHouse", "MN_DULUTH_HARBOR_HOUSE", [5, 4]),
        ("MnDuluthHistoryHouse", "MN_DULUTH_HISTORY_HOUSE", [6, 5]),
    ]) {
        ensure!(
            room.id == name
                && room.native_constant == constant
                && room.dimensions_metatiles == dimensions,
            "changed Duluth room identity/dimensions: {name}"
        );
    }
    Ok((
        town,
        layout.compile().context("native Duluth terrain strokes")?,
    ))
}
