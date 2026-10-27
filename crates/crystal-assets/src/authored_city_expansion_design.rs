//! Explicit installed-city expansion inputs; no geography is generated here.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::authored_route_design::{Door, Pickup, RouteTrainer, Sign};
use crate::authored_town_design::Actor;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Expansion {
    pub schema_version: u16,
    pub city: String,
    pub native_module_sha256: String,
    pub old_dimensions_metatiles: [u16; 2],
    pub dimensions_metatiles: [u16; 2],
    pub translation_core: [u16; 2],
    pub arrival_core: [u16; 2],
    pub clinic_constant: String,
    #[serde(default)]
    pub clinic_recovery_core: Option<[u16; 2]>,
    pub default_mart: String,
    #[serde(default)]
    pub anchor_overrides: Vec<Anchor>,
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub spawn_overrides: Vec<Spawn>,
    #[serde(default)]
    pub residents: Vec<Actor>,
    #[serde(default)]
    pub signs: Vec<Sign>,
    #[serde(default)]
    pub doors: Vec<Door>,
    #[serde(default)]
    pub rooms: Vec<Room>,
    #[serde(default)]
    pub event_flags: Vec<String>,
    #[serde(default)]
    pub programs: BTreeMap<String, Value>,
    #[serde(default)]
    pub trainers: Vec<RouteTrainer>,
    #[serde(default)]
    pub items: Vec<Pickup>,
    #[serde(default)]
    pub text_revisions: Vec<TextRevision>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Anchor {
    pub kind: AnchorKind,
    /// Zero-based native vector index, never a new ordering of old events.
    pub index: usize,
    pub before: [u16; 2],
    pub after: [u16; 2],
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AnchorKind {
    Object,
    Background,
    Coordinate,
    Warp,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Connection {
    pub direction: String,
    pub target: String,
    pub old_offset: i32,
    pub offset: i32,
    pub town_edge: [[u16; 2]; 2],
    pub route_edge: [[u16; 2]; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Spawn {
    pub identifier: u16,
    pub before: [u16; 2],
    pub after: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Room {
    pub id: String,
    pub native_constant: String,
    pub template: String,
    pub tileset: String,
    pub border_block: u8,
    pub dimensions_metatiles: [u16; 2],
    pub rows: Vec<String>,
    pub arrival_core: [u16; 2],
    pub warps: Vec<Warp>,
    #[serde(default)]
    pub residents: Vec<Actor>,
    #[serde(default)]
    pub signs: Vec<Sign>,
    #[serde(default)]
    pub programs: BTreeMap<String, Value>,
    #[serde(default)]
    pub trainers: Vec<RouteTrainer>,
    #[serde(default)]
    pub items: Vec<Pickup>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Warp {
    pub index: u16,
    pub tile: [u16; 2],
    pub target: String,
    pub target_warp: i16,
    pub passive: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TextRevision {
    pub label: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
}
