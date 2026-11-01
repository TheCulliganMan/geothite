//! Native transactional admission and return; no alternate movement dispatcher.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::state::{BattleMemory, GameState, OverworldMemory};
use crystal_core::systems::field_moves;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari::{self, SafariPhase, SafariSession};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;
use serde::{Deserialize, Serialize};

use crate::GameDataSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SafariAdmissionOutcome {
    Declined,
    AlreadyActive,
    Busy,
    NeedSurf,
    InsufficientMoney,
    StorageFull,
    Admitted,
}

impl GameDataSet {
    /// Confirm a nearby configured ranger's expedition transaction. A Minnesota
    /// canoe or native Surf permission, resources and the actual map transition commit
    /// together. Refusal or a failed native warp changes neither state nor map.
    ///
    /// # Errors
    /// Rejects missing/broken config, a remote ranger request, invalid storage,
    /// incompatible saved state or native transition/callback/music failures.
    /// Ordinary entry refusals return a typed outcome without charging.
    pub fn admit_safari(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        confirmed: bool,
        music_ids: &BTreeSet<String>,
    ) -> Result<SafariAdmissionOutcome> {
        let zone = self
            .safari_zone
            .as_ref()
            .context("no configured Safari expedition")?;
        zone.validate(self)?;
        if !confirmed {
            return Ok(SafariAdmissionOutcome::Declined);
        }
        if safari::session(state)
            .map_err(anyhow::Error::msg)?
            .is_some()
        {
            return Ok(SafariAdmissionOutcome::AlreadyActive);
        }
        ensure!(
            overworld.map.name == zone.admission_map,
            "Safari admission must occur at its free ranger"
        );
        validate_context(state, overworld)?;
        let (dx, dy) = overworld.player.facing.delta();
        let target = TilePosition::new(
            overworld
                .player
                .tile
                .x
                .checked_add(dx)
                .context("Safari ranger target x overflow")?,
            overworld
                .player
                .tile
                .y
                .checked_add(dy)
                .context("Safari ranger target y overflow")?,
        );
        ensure!(
            target.x >= 0
                && target.y >= 0
                && [target.x as u16, target.y as u16] == zone.admission_actor_core,
            "Safari confirmation must face the configured ranger"
        );
        let (_, ranger) = overworld
            .visible_object_at_checked(target)?
            .context("Safari admission ranger is not present")?;
        ensure!(
            ranger.script == zone.admission_script,
            "Safari admission actor changed"
        );
        if state.battle != BattleMemory::Inactive
            || state.script_runtime.active_battle_combat.is_some()
            || state.bug_contest.timer_active
            || !state.bug_contest.party_backup.is_empty()
            || state.link_session.link_mode != 0
        {
            return Ok(SafariAdmissionOutcome::Busy);
        }
        let has_surf = (0..state.storage.party.pokemon.len()).any(|index| {
            field_moves::validate_surf_field_access(&self.field_moves, state, &state.storage, index)
                .is_ok()
        });
        let has_canoe = overworld.map.name.starts_with("Mn")
            && self
                .items
                .get(field_moves::CANOE_ITEM)
                .is_some_and(|item| state.bag.has_item(item));
        if !has_surf && !has_canoe {
            return Ok(SafariAdmissionOutcome::NeedSurf);
        }
        if state.money < safari::ADMISSION_FEE {
            return Ok(SafariAdmissionOutcome::InsufficientMoney);
        }
        state
            .storage
            .validate_metadata()
            .map_err(anyhow::Error::msg)?;
        if !state
            .storage
            .has_capture_space_in_box(state.current_pc_box)
            .map_err(anyhow::Error::msg)?
        {
            return Ok(SafariAdmissionOutcome::StorageFull);
        }
        // Publish neither debit nor permit if the real map transition fails.
        let mut staged_state = state.clone();
        let mut staged_overworld = overworld.clone();
        staged_state.money -= safari::ADMISSION_FEE;
        self.transition_overworld_session(
            &mut staged_state,
            &mut staged_overworld,
            &zone.entry.map_name,
            zone.entry.tile,
            SpawnMemoryUpdate::Preserve,
            music_ids,
        )?;
        staged_overworld.player.facing = zone.entry.facing;
        self.commit_overworld_snapshot(
            &mut staged_state,
            &staged_overworld,
            SpawnMemoryUpdate::Preserve,
        );
        let session = SafariSession::new(zone.id.clone(), zone.return_to.clone())
            .map_err(anyhow::Error::msg)?;
        safari::set_session(&mut staged_state, &session).map_err(anyhow::Error::msg)?;
        self.sync_safari_gate(&staged_state, &mut staged_overworld)?;
        self.validate_saved_safari_references(&staged_state)?;
        *state = staged_state;
        *overworld = staged_overworld;
        Ok(SafariAdmissionOutcome::Admitted)
    }

    /// Commit the real safe return after the controller releases capture/result
    /// input. A failed transition retains the ending permit for a safe retry.
    ///
    /// # Errors
    /// Requires an ending expedition, unchanged config and safe native arrival.
    /// Does not close dialogue, acknowledge a result or discard a caught Pokémon.
    pub fn return_from_safari(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        music_ids: &BTreeSet<String>,
    ) -> Result<()> {
        self.validate_saved_safari_references(state)?;
        validate_context(state, overworld)?;
        let session = safari::session(state)
            .map_err(anyhow::Error::msg)?
            .context("no Safari return pending")?;
        ensure!(
            matches!(session.phase, SafariPhase::Ending(_)),
            "Safari result must finish before return"
        );
        let mut staged_state = state.clone();
        let mut staged_overworld = overworld.clone();
        self.transition_overworld_session(
            &mut staged_state,
            &mut staged_overworld,
            &session.return_to.map_name,
            session.return_to.tile,
            SpawnMemoryUpdate::Preserve,
            music_ids,
        )?;
        staged_overworld.player.facing = session.return_to.facing;
        self.commit_overworld_snapshot(
            &mut staged_state,
            &staged_overworld,
            SpawnMemoryUpdate::Preserve,
        );
        safari::complete_return(&mut staged_state).map_err(anyhow::Error::msg)?;
        self.validate_saved_safari_references(&staged_state)?;
        *state = staged_state;
        *overworld = staged_overworld;
        Ok(())
    }
}

impl GameDataSet {
    pub(crate) fn sync_safari_gate(
        &self,
        state: &GameState,
        overworld: &mut OverworldSession,
    ) -> Result<()> {
        let Some(zone) = &self.safari_zone else {
            return Ok(());
        };
        if overworld.map.name != zone.gate_court.map_name {
            return Ok(());
        }
        let saved = safari::session(state).map_err(anyhow::Error::msg)?;
        if let Some(active) = &saved {
            ensure!(
                active.zone_id == zone.id,
                "Safari guard cannot accept a different zone's permit"
            );
        }
        let open = saved.is_some();
        for guard in &zone.gate_court.guard_identifiers {
            overworld.clear_loaded_roster_visibility_override(guard);
            if open {
                overworld.hidden_object_identifiers.insert(guard.clone());
                overworld.delete_loaded_object_struct_for_disappear(guard);
            } else {
                overworld.hidden_object_identifiers.remove(guard);
                overworld
                    .copy_object_struct_for_appear(guard)
                    .context("restore native Safari permit guard")?;
            }
            overworld.shown_object_identifiers.remove(guard);
        }
        Ok(())
    }
}

fn validate_context(state: &GameState, overworld: &OverworldSession) -> Result<()> {
    ensure!(
        matches!(&state.overworld, OverworldMemory::Active { map_name, tile, facing, mode }
        if *map_name == overworld.map.name && *tile == overworld.player.tile
            && *facing == overworld.player.facing && *mode == overworld.player.mode),
        "Safari native map context disagrees with authoritative state"
    );
    Ok(())
}
