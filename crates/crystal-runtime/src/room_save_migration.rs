//! Private mechanics shared by explicitly allowlisted room migrations.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::collision::{
    PlayerTraversalState, can_enter_tile, permissions, sample_collision,
};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::object_tile_position_checked;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoomReentry {
    policy: String,
    maps: Vec<RoomArrival>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomArrival {
    map_id: String,
    arrival: TilePosition,
    facing: Direction,
}

#[derive(Clone, Copy)]
pub(crate) enum ArrivalSurface {
    Walkable,
    Floor,
}

impl RoomReentry {
    pub(crate) fn arrival(&self, map_name: &str) -> Option<(TilePosition, Direction)> {
        self.maps
            .iter()
            .find(|entry| entry.map_id == map_name)
            .map(|entry| (entry.arrival, entry.facing))
    }

    pub(crate) fn validate_metadata(&self, policy: &str, allowed_maps: &[&str]) -> Result<()> {
        ensure!(self.policy == policy, "unsupported room reentry policy");
        ensure!(
            self.maps.len() == allowed_maps.len(),
            "incomplete room arrival list for {policy}"
        );
        let mut maps = BTreeSet::new();
        for entry in &self.maps {
            ensure!(
                allowed_maps.contains(&entry.map_id.as_str()) && maps.insert(&entry.map_id),
                "room reentry maps must be the distinct approved rooms for {policy}"
            );
            ensure!(
                entry.arrival.x >= 0 && entry.arrival.y >= 0,
                "room arrival coordinates must be nonnegative"
            );
        }
        Ok(())
    }

    // Source overrides, event flags and callbacks never legitimize an arrival.
    // Even normally hidden authored actors must not share its base tile.
    pub(crate) fn validate_arrivals(
        &self,
        data: &GameDataSet,
        surface: ArrivalSurface,
    ) -> Result<()> {
        for entry in &self.maps {
            let session = data
                .overworld_session(&entry.map_id, entry.arrival, 0)
                .with_context(|| format!("invalid base room arrival for {}", entry.map_id))?;
            let walkable = [
                Direction::Down,
                Direction::Up,
                Direction::Left,
                Direction::Right,
            ]
            .into_iter()
            .any(|direction| {
                can_enter_tile(
                    &session.map,
                    &session.tileset,
                    entry.arrival,
                    direction,
                    PlayerTraversalState::Walk,
                )
            });
            ensure!(
                walkable,
                "room arrival must be walkable without connection or callback repair: {}",
                entry.map_id
            );
            if matches!(surface, ArrivalSurface::Floor) {
                ensure!(
                    sample_collision(&session.map, &session.tileset, entry.arrival)
                        .is_some_and(|sample| sample.permission == permissions::FLOOR),
                    "cave arrival must be ordinary dry FLOOR: {}",
                    entry.map_id
                );
            }
            ensure!(
                session
                    .objects
                    .iter()
                    .all(|actor| object_tile_position_checked(actor) != Some(entry.arrival)),
                "room arrival overlaps an authored actor: {}",
                entry.map_id
            );
        }
        Ok(())
    }

    // Only a copy with checked identity/checksum and an idle checkpoint reaches
    // this method. The caller must fully validate the target state before write.
    pub(crate) fn apply(
        &self,
        state: &mut GameState,
        selected_rooms: Option<&[String]>,
    ) -> Result<()> {
        let selected = self
            .maps
            .iter()
            .filter(|entry| selected_rooms.is_none_or(|rooms| rooms.contains(&entry.map_id)));
        for entry in selected.clone() {
            if let Some(memory) = state.map_object_overrides.get(&entry.map_id) {
                ensure!(
                    !memory.player_hidden && !memory.object_structs.player_fixed_facing,
                    "finish hidden or fixed-facing room player state before upgrading: {}",
                    entry.map_id
                );
            }
        }
        let arrival = match &state.overworld {
            OverworldMemory::Active { map_name, mode, .. } => {
                let arrival = selected.clone().find(|entry| entry.map_id == *map_name);
                ensure!(
                    arrival.is_none() || *mode == MovementMode::Normal,
                    "room upgrade requires normal walking movement"
                );
                arrival
            }
            OverworldMemory::Inactive => {
                anyhow::bail!("room upgrade requires an overworld checkpoint")
            }
        };
        for entry in selected {
            state.map_block_overrides.remove(&entry.map_id);
            state.map_object_overrides.remove(&entry.map_id);
        }
        if let Some(arrival) = arrival
            && let OverworldMemory::Active { tile, facing, .. } = &mut state.overworld
        {
            *tile = arrival.arrival;
            *facing = arrival.facing;
        }
        Ok(())
    }
}
