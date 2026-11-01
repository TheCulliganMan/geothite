//! Native physical contracts shared by explicitly authored towns.
use crate::authored_route_design::{Door, Facing, PassiveArrival, Sign};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Town {
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
pub(crate) struct Port {
    pub route: String,
    pub side: String,
    pub gate_metatile: [u16; 2],
    pub landing_metatile: [u16; 2],
}
#[derive(Deserialize)]
pub(crate) struct Actor {
    pub id: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub facing: Facing,
    #[serde(default)]
    pub lines: Vec<String>,
    pub service: Option<String>,
    #[serde(default)]
    pub repeat_lines: Vec<String>,
    #[serde(default)]
    pub stock: Vec<String>,
    pub approach: Option<[u16; 2]>,
    pub counter_tile: Option<[u16; 2]>,
}
#[derive(Deserialize)]
pub(crate) struct Exit {
    pub index: u16,
    pub tile: [u16; 2],
    pub target: String,
    pub target_warp: i16,
}
#[derive(Deserialize)]
pub(crate) struct Pc {
    pub tile: [u16; 2],
    pub approach: [u16; 2],
}
#[derive(Deserialize)]
pub(crate) struct Room {
    pub id: String,
    pub native_constant: String,
    pub template: String,
    pub tileset: String,
    pub border_block: u8,
    pub dimensions_metatiles: [u16; 2],
    pub rows: Vec<String>,
    pub passive_arrival: PassiveArrival,
    pub exits: Vec<Exit>,
    pub residents: Vec<Actor>,
    #[serde(default)]
    pub signs: Vec<Sign>,
    pub pc: Option<Pc>,
}
