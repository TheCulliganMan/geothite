//! Explicitly bounded cave extensions to the exact gym-overhaul migration.

use anyhow::Result;
use crystal_assets::GameDataSet;
use crystal_core::state::GameState;
use serde::Deserialize;

use crate::room_save_migration::{ArrivalSurface, RoomReentry};

pub(crate) const TARGET_MANIFEST: &str = "northstar-cave-overhaul-v1";
pub(crate) const CAVES: [&str; 8] = [
    "MnR01Cave1F",
    "MnR10Cave1F",
    "MnR12Cave1F",
    "MnVoyageursIce1F",
    "MnVoyageursIceB1F",
    "MnR13Cave1F",
    "MnR17Cave1F",
    "MnR20Cave1F",
];

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct CaveReentry(RoomReentry);

impl CaveReentry {
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        // Retain support for the earlier local review pack's exact five rooms.
        self.0
            .validate_metadata("minnesota-eight-caves-v1", &CAVES)
            .or_else(|_| {
                self.0
                    .validate_metadata("minnesota-five-caves-v1", &CAVES[..5])
            })
    }

    pub(crate) fn validate_arrivals(&self, data: &GameDataSet) -> Result<()> {
        self.0.validate_arrivals(data, ArrivalSurface::Floor)
    }

    pub(crate) fn apply(&self, state: &mut GameState) -> Result<()> {
        self.0.apply(state)
    }
}
