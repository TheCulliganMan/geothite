//! Shore launch preparation. The ordinary movement transaction owns collision,
//! occupancy, encounters, connections, dismount and rollback.
use anyhow::Result;
use crystal_core::state::GameState;
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::world::collision::{
    Terrain, describe_collision, is_direction_blocked_leaving, sample_collision,
};
use crystal_core::world::map::Direction;
use crystal_core::world::movement::{MovementMode, StepOptions, checked_move_by_stride};
use crystal_core::world::session::OverworldSession;

use crate::GameDataSet;

impl GameDataSet {
    pub(crate) fn prepare_canoe_step(
        &self,
        state: &GameState,
        session: &mut OverworldSession,
        direction: Direction,
    ) -> Result<bool> {
        let Some(item) = self.items.get(CANOE_ITEM) else {
            return Ok(false);
        };
        if !session.map.name.starts_with("Mn")
            || !state.bag.has_item(item)
            || session.player.mode != MovementMode::Normal
            || session.player.facing != direction
            || state.flags.is_engine_flag_set("ENGINE_ALWAYS_ON_BIKE")?
        {
            return Ok(false);
        }
        let Some(target) = checked_move_by_stride(
            session.player.tile,
            direction,
            StepOptions::default().stride_tiles,
        ) else {
            return Ok(false);
        };
        let Some(sample) = sample_collision(&session.map, &session.tileset, target) else {
            return Ok(false);
        };
        // A bridge deck is land to a walker. Do not launch underneath it.
        let walking_permission = crystal_core::world::bridge::layer_permission(
            sample.permission,
            session.player.mode.traversal_state(),
        );
        if describe_collision(walking_permission).terrain != Terrain::Water
            || self
                .field_moves
                .surf
                .blocked_collisions
                .contains(&sample.permission)
        {
            return Ok(false);
        }
        let Some(current) = sample_collision(&session.map, &session.tileset, session.player.tile)
        else {
            return Ok(false);
        };
        if crystal_core::world::bridge::is_bridge(current.permission)
            || describe_collision(current.permission).terrain != Terrain::Land
            || is_direction_blocked_leaving(current.permission, direction)
            || crystal_core::world::bridge::blocks_exit(
                current.permission,
                direction,
                session.player.mode.traversal_state(),
            )
        {
            return Ok(false);
        }
        session.player.mode = MovementMode::Surf;
        Ok(true)
    }
}
