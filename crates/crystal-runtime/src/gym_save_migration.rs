//! The eight-gym policy for one exact release migration.

use anyhow::Result;
use crystal_assets::GameDataSet;
use crystal_core::state::GameState;
use serde::Deserialize;

use crate::room_save_migration::{ArrivalSurface, RoomReentry};

pub(crate) const TARGET_MANIFEST: &str = "northstar-gym-overhaul-v1";
pub(crate) const GYMS: [&str; 8] = [
    "MnRochesterGym",
    "MnMankatoGym",
    "MnMinneapolisGym",
    "MnStPaulGym",
    "MnStCloudGym",
    "MnBrainerdGym",
    "MnBemidjiGym",
    "MnDuluthGym",
];

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct GymReentry(RoomReentry);

impl GymReentry {
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        self.0.validate_metadata("minnesota-eight-gyms-v1", &GYMS)
    }

    pub(crate) fn validate_arrivals(&self, data: &GameDataSet) -> Result<()> {
        self.0.validate_arrivals(data, ArrivalSurface::Walkable)
    }

    pub(crate) fn apply(
        &self,
        state: &mut GameState,
        selected_rooms: Option<&[String]>,
    ) -> Result<()> {
        self.0.apply(state, selected_rooms)
    }
}
