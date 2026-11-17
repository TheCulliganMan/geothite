//! The five-cave extension to the exact gym-overhaul release migration.

use anyhow::Result;
use crystal_assets::GameDataSet;
use crystal_core::state::GameState;
use serde::Deserialize;

use crate::room_save_migration::{ArrivalSurface, RoomReentry};

pub(crate) const TARGET_MANIFEST: &str = "northstar-cave-overhaul-v1";
pub(crate) const CAVES: [&str; 5] = [
    "MnR01Cave1F",
    "MnR10Cave1F",
    "MnR12Cave1F",
    "MnVoyageursIce1F",
    "MnVoyageursIceB1F",
];

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct CaveReentry(RoomReentry);

impl CaveReentry {
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        self.0.validate_metadata("minnesota-five-caves-v1", &CAVES)
    }

    pub(crate) fn validate_arrivals(&self, data: &GameDataSet) -> Result<()> {
        self.0.validate_arrivals(data, ArrivalSurface::Floor)
    }

    pub(crate) fn apply(&self, state: &mut GameState) -> Result<()> {
        self.0.apply(state)
    }
}
