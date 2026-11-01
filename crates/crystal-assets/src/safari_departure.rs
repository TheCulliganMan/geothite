//! End a permit at the real destination of native outward travel.

use anyhow::{Result, ensure};
use crystal_core::state::GameState;
use crystal_core::systems::safari::{self, SafariPhase};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;

use crate::GameDataSet;

impl GameDataSet {
    pub(crate) fn safari_departure_for_transition(
        &self,
        state: &GameState,
        source: &OverworldSession,
        destination: &str,
        tile: TilePosition,
    ) -> Result<bool> {
        let Some(zone) = &self.safari_zone else {
            return Ok(false);
        };
        let active = safari::session(state)?;
        if zone.maps.contains_key(destination) && active.is_none() {
            ensure!(
                zone.gate_court.contains(destination, tile),
                "inactive expedition cannot arrive beyond the free gate court"
            );
        }
        if !zone.maps.contains_key(&source.map.name) || zone.maps.contains_key(destination) {
            return Ok(false);
        }
        let Some(active) = active else {
            return Ok(false);
        };
        // The explicit ranger/exhaustion return owns its existing exact-arrival
        // settlement; do not clear the permit ahead of complete_return.
        if matches!(active.phase, SafariPhase::Ending(_))
            && destination == active.return_to.map_name
            && tile == active.return_to.tile
        {
            return Ok(false);
        }
        ensure!(
            matches!(
                active.phase,
                SafariPhase::Exploring | SafariPhase::Ending(_)
            ),
            "Safari capture or pending result owns outward travel"
        );
        Ok(true)
    }

    pub(crate) fn finish_safari_departure(
        &self,
        state: &mut GameState,
        destination: &OverworldSession,
    ) -> Result<()> {
        safari::complete_departure(
            state,
            &destination.map.name,
            destination.player.tile,
            destination.player.facing,
        )?;
        self.validate_saved_safari_references(state)
    }
}
