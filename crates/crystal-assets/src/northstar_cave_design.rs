//! Explicit native terrain and discovery contracts for eight installed caves.

use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use serde::Deserialize;

pub(crate) const MAPS: [&str; 8] = [
    "MnR01Cave1F",
    "MnR10Cave1F",
    "MnR12Cave1F",
    "MnVoyageursIce1F",
    "MnVoyageursIceB1F",
    "MnR13Cave1F",
    "MnR17Cave1F",
    "MnR20Cave1F",
];
pub(crate) const ARRIVALS: [[u16; 2]; 8] = [
    [9, 10],
    [4, 13],
    [17, 22],
    [5, 18],
    [5, 18],
    [7, 14],
    [9, 12],
    [5, 20],
];
pub(crate) const TRAINERS: [&str; 3] = [
    "MN_CAVE_DRIFTLESS_ROOKIE",
    "MN_CAVE_ORE_CLIMBER",
    "MN_CAVE_GUNFLINT_ANGLER",
];
pub(crate) const EVENTS: [&str; 12] = [
    "EVENT_MN_CAVE_DRIFTLESS_ROOKIE",
    "EVENT_MN_CAVE_ORE_CLIMBER",
    "EVENT_MN_CAVE_GUNFLINT_ANGLER",
    "EVENT_MN_CAVE_DRIFTLESS_REPEL",
    "EVENT_MN_CAVE_ORE_NUGGET",
    "EVENT_MN_CAVE_SPRING_ETHER",
    "EVENT_MN_CAVE_GUNFLINT_ELIXER",
    "EVENT_MN_CAVE_ICE_FULL_HEAL",
    "EVENT_MN_CAVE_ICE_PP_UP",
    "EVENT_MN_CAVE_CEDAR_MAX_REPEL",
    "EVENT_MN_CAVE_R20_UPPER_CACHE",
    "EVENT_MN_CAVE_R20_SEEP_CACHE",
];
pub(crate) const MANIFEST: &str = "northstar-cave-overhaul-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Revision {
    pub schema_version: u8,
    pub id: String,
    pub rooms: Vec<Cave>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cave {
    pub id: String,
    pub dimensions_metatiles: [u16; 2],
    pub tileset: String,
    pub preimage_sha256: String,
    pub arrival: [u16; 2],
    pub rows: Vec<String>,
    pub actor_positions: BTreeMap<String, [u16; 2]>,
    #[serde(default)]
    pub retained_sightlines: BTreeMap<String, CaveSightline>,
    pub trainers: Vec<Trainer>,
    pub hidden_items: Vec<HiddenItem>,
    pub waypoints: Vec<[u16; 2]>,
    pub optional_water_waypoints: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Trainer {
    pub id: String,
    pub name: String,
    pub trainer_class: String,
    pub sprite: String,
    pub tile: [u16; 2],
    pub direction: String,
    pub radius: u16,
    pub party: Vec<Member>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaveSightline {
    pub direction: String,
    pub radius: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Member {
    pub species: String,
    pub level: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HiddenItem {
    pub id: String,
    pub tile: [u16; 2],
    pub item: String,
    pub landmark: String,
    #[serde(default)]
    pub target_surface: HiddenItemSurface,
    #[serde(default)]
    pub requires_surf: bool,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HiddenItemSurface {
    #[default]
    Floor,
    Rock,
}

impl Revision {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1 && self.id == MANIFEST && self.rooms.len() == MAPS.len(),
            "eight-cave revision required"
        );
        let mut ids = std::collections::BTreeSet::new();
        for ((room, expected), arrival) in self.rooms.iter().zip(MAPS).zip(ARRIVALS) {
            ensure!(room.arrival == arrival, "approved cave migration arrival");
            ensure!(room.id == expected, "ordered cave identities required");
            ensure!(
                matches!(room.tileset.as_str(), "cave" | "ice_path")
                    && room
                        .dimensions_metatiles
                        .iter()
                        .all(|n| (1..=32).contains(n))
                    && room.preimage_sha256.len() == 64
                    && room.preimage_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
                "bounded cave dimensions, native tileset and exact preimage"
            );
            room.blocks()?;
            for id in room
                .trainers
                .iter()
                .map(|trainer| &trainer.id)
                .chain(room.hidden_items.iter().map(|item| &item.id))
            {
                ensure!(
                    id.starts_with("MN_CAVE_")
                        && id.len() <= 40
                        && id
                            .bytes()
                            .all(|b| b.is_ascii_uppercase() || b == b'_' || b.is_ascii_digit())
                        && ids.insert(id),
                    "distinct cave event identity: {id}"
                );
            }
            for trainer in &room.trainers {
                ensure!(
                    (1..=3).contains(&trainer.radius)
                        && matches!(trainer.direction.as_str(), "up" | "right" | "down" | "left")
                        && (1..=3).contains(&trainer.party.len()),
                    "bounded native sightline and party"
                );
            }
            for (id, sight) in &room.retained_sightlines {
                ensure!(
                    !id.is_empty()
                        && !room.trainers.iter().any(|trainer| &trainer.id == id)
                        && (1..=3).contains(&sight.radius)
                        && matches!(sight.direction.as_str(), "up" | "right" | "down" | "left"),
                    "bounded retained native cave sightline"
                );
            }
            ensure!(
                room.hidden_items
                    .iter()
                    .all(|item| !item.landmark.trim().is_empty())
                    && room.waypoints.len() >= 3,
                "exploration landmarks and explicit route contracts required"
            );
        }
        let trainers = self
            .rooms
            .iter()
            .flat_map(|room| &room.trainers)
            .map(|trainer| trainer.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        ensure!(
            trainers == TRAINERS.into_iter().collect(),
            "declared cave trainer identities"
        );
        let events = ids
            .into_iter()
            .map(|id| format!("EVENT_{id}"))
            .collect::<std::collections::BTreeSet<_>>();
        ensure!(
            events == EVENTS.into_iter().map(str::to_owned).collect(),
            "declared cave receipt identities"
        );
        Ok(())
    }
}

impl Cave {
    pub fn blocks(&self) -> Result<Vec<u16>> {
        let [width, height] = self.dimensions_metatiles;
        ensure!(self.rows.len() == usize::from(height), "complete cave rows");
        let mut blocks = Vec::new();
        for row in &self.rows {
            let values = row.split_whitespace().collect::<Vec<_>>();
            ensure!(
                values.len() == usize::from(width),
                "complete cave row width"
            );
            for value in values {
                blocks.push(u16::from(u8::from_str_radix(value, 16)?));
            }
        }
        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::Revision;

    const SOURCE: &str =
        include_str!("../../../planning/minnesota/caves/continuation-20261117/rooms.json");

    #[test]
    fn cave_design_rejects_truncated_geometry_and_duplicate_rewards() {
        let mut revision: Revision = serde_json::from_str(SOURCE).expect("authored cave source");
        revision.validate().expect("valid authored caves");
        revision.rooms[0].rows.pop();
        assert!(revision.validate().is_err());
        let mut revision: Revision = serde_json::from_str(SOURCE).expect("authored cave source");
        revision.rooms[1].hidden_items[0].id = revision.rooms[0].hidden_items[0].id.clone();
        assert!(revision.validate().is_err());
    }
}
