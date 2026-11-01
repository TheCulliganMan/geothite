// Destination collision for connected-map movement.

impl GameDataSet {
    fn blocked_connection_edge_target(
        &self,
        state: &GameState,
        session: &OverworldSession,
        direction: Direction,
        force_step_after_turn: bool,
    ) -> Result<Option<TilePosition>> {
        if session.player.facing != direction && !force_step_after_turn {
            return Ok(None);
        }
        let Some(target) = checked_move_by_stride(
            session.player.tile,
            direction,
            StepOptions::default().stride_tiles,
        ) else {
            return Ok(None);
        };
        let mut probe = session.clone();
        probe.player.tile = target;
        let Some(trigger) = probe
            .check_connection_checked()
            .with_context(|| format!("check connection on {}", probe.map.name))?
        else {
            return Ok(None);
        };
        if self.connection_trigger_has_destination(state, session, direction, &trigger)? {
            Ok(None)
        } else {
            Ok(Some(target))
        }
    }

    fn connection_trigger_has_destination(
        &self,
        state: &GameState,
        session: &OverworldSession,
        direction: Direction,
        trigger: &ConnectionTrigger,
    ) -> Result<bool> {
        let target_attributes = self
            .map_attributes
            .get(&trigger.connection.target_map)
            .with_context(|| {
                format!(
                    "connection target '{}' missing attributes (referenced by {})",
                    trigger.connection.target_map, trigger.map_name
                )
            })?;
        if !connection_destination_tile_in_bounds(
            trigger.tile,
            &trigger.connection.direction,
            trigger.connection.offset,
            target_attributes,
        )? {
            return Ok(false);
        }
        let destination = self.resolve_connection_transition(trigger)?.destination;
        let module = self.map_module(&destination.map_name)?;
        let tileset = self.tileset_collision(&module.attributes.tileset_name)?;
        let mut map = OverworldMapData::from_attributes(
            &destination.map_name,
            &module.attributes,
            module.blocks.clone(),
        );
        // Connections must sample the visible neighbor, including saved Cut or
        // puzzle edits, before committing an off-map step on the source side.
        if let Some(overrides) = state.map_block_overrides.get(&destination.map_name) {
            self.validate_saved_block_overrides(&destination.map_name, overrides)?;
            for (&(x, y), &block) in overrides {
                map.metatile_ids
                    [usize::from(y) * usize::from(module.attributes.width) + usize::from(x)] =
                    block;
            }
        }
        Ok(can_enter_tile(
            &map,
            &tileset,
            destination.tile,
            direction,
            session.player.mode.traversal_state(),
        ))
    }
}
