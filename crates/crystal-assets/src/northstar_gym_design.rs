//! Explicit geometry and encounter contracts for the eight-room revision.

use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::gym_design::Gym;

pub(crate) const MAPS: [&str; 8] = [
    "MnRochesterGym",
    "MnMankatoGym",
    "MnMinneapolisGym",
    "MnStPaulGym",
    "MnStCloudGym",
    "MnBrainerdGym",
    "MnBemidjiGym",
    "MnDuluthGym",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Revision {
    pub schema_version: u8,
    pub rooms: Vec<Gym>,
    pub contracts: BTreeMap<String, RoomContract>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoomContract {
    pub sightlines: BTreeMap<String, Sightline>,
    pub court: Court,
    #[serde(default)]
    pub victory_opens_gates: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Sightline {
    pub direction: String,
    pub radius: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Court {
    #[serde(default)]
    pub open: bool,
    pub all: Vec<String>,
    pub any: Vec<String>,
}

impl Revision {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == 1 && self.rooms.len() == 8,
            "eight gym designs required"
        );
        ensure!(self.contracts.len() == 8, "eight gym contracts required");
        for (index, gym) in self.rooms.iter().enumerate() {
            ensure!(
                gym.id == MAPS[index] && usize::from(gym.slot) == index + 1,
                "ordered gym identities"
            );
            let contract = self
                .contracts
                .get(&gym.id)
                .ok_or_else(|| anyhow::anyhow!("missing gym contract"))?;
            validate_room(gym, contract)?;
        }
        Ok(())
    }
}

pub(crate) fn validate_room(gym: &Gym, contract: &RoomContract) -> Result<()> {
    let [width, height] = gym.dimensions_metatiles;
    ensure!(
        (5..=16).contains(&width) && (5..=16).contains(&height),
        "bounded native gym extent"
    );
    ensure!(
        gym.rows.len() == usize::from(height)
            && gym
                .rows
                .iter()
                .all(|row| row.chars().count() == usize::from(width)
                    && row.chars().all(|glyph| gym.art.contains_key(&glyph)))
            && gym.art.contains_key(&'#'),
        "complete native geometry recipe: {}",
        gym.id
    );
    ensure!(
        contract.sightlines.len() == gym.trainers.len(),
        "every trainer needs a sightline contract"
    );
    for trainer in &gym.trainers {
        let sight = contract
            .sightlines
            .get(&trainer.id)
            .ok_or_else(|| anyhow::anyhow!("missing trainer sightline"))?;
        ensure!(
            matches!(sight.direction.as_str(), "up" | "down" | "left" | "right")
                && (1..=3).contains(&sight.radius),
            "bounded cardinal trainer sightline"
        );
    }
    ensure!(
        contract.court.open == (contract.court.all.is_empty() && contract.court.any.is_empty()),
        "declare either an open court or explicit court requirements"
    );
    ensure!(
        gym.stations.is_empty() && gym.switch_puzzle.is_none() && gym.pads.is_empty(),
        "revision uses physical routes and controls, not old quizzes or ordered guesses"
    );
    Ok(())
}
