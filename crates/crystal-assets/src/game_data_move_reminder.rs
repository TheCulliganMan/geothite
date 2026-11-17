//! Atomic move replacement and payment through the shared teaching routine.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::state::GameState;
use crystal_core::systems::move_reminder::{MOVE_REMINDER_PRICE, validate_relearning};
use crystal_core::systems::special_routines::{SpecialRoutineEffect, SpecialRoutineOutcome};

use crate::{GameDataSet, RuntimeMoveReminderCommand};

impl GameDataSet {
    pub(crate) fn relearn_party_move(
        &self,
        state: &mut GameState,
        command: RuntimeMoveReminderCommand,
        music_ids: &BTreeSet<String>,
    ) -> Result<SpecialRoutineOutcome> {
        let pokemon = state
            .storage
            .party
            .pokemon
            .get(command.party_index)
            .and_then(Option::as_ref)
            .context("move reminder party slot is empty")?;
        let hm_moves = self
            .items
            .values()
            .filter(|item| !item.consumable)
            .filter_map(|item| item.tmhm_move.clone())
            .collect();
        validate_relearning(
            &self.learnsets,
            pokemon,
            &command.move_id,
            command.forget_index,
            &hm_moves,
            state.money,
        )?;
        ensure!(
            self.moves.contains_key(&command.move_id),
            "unknown reminder move"
        );

        // Keep every failure, including the shared happiness/teaching routine,
        // outside the authoritative state. Payment and replacement commit once.
        let mut next = state.clone();
        if let Some(index) = command.forget_index {
            next.storage.party.pokemon[command.party_index]
                .as_mut()
                .expect("validated occupied party slot")
                .moves
                .remove(index);
        }
        next.money -= MOVE_REMINDER_PRICE;
        next.script_runtime
            .variables
            .insert("_party_slot".into(), command.party_index.to_string());
        next.script_runtime
            .variables
            .insert("_move".into(), command.move_id);
        let mut outcome = self.apply_special_routine(&mut next, "MoveTutor", music_ids)?;
        ensure!(
            matches!(
                &outcome.effect,
                SpecialRoutineEffect::MoveTutor { learned: true, .. }
            ),
            "move reminder did not teach the selected move"
        );
        if let Some(index) = command.forget_index {
            // MoveTutor appends. Put the learned move in the selected slot,
            // preserving every other move's order, current PP and PP Ups.
            next.storage.party.pokemon[command.party_index]
                .as_mut()
                .expect("teaching retains party slot")
                .moves[index..]
                .rotate_right(1);
            next.sync_party_from_storage();
        }
        outcome.routine = "MoveReminder".into();
        next.script_runtime.last_special_routine = Some(outcome.routine.clone());
        *state = next;
        Ok(outcome)
    }
}

#[cfg(test)]
#[path = "game_data_move_reminder_tests.rs"]
mod tests;
