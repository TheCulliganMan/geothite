//! Opt-in bounded menu ownership diagnostics for earned controller tests.
use crystal_core::input::GameButton;
use crystal_runtime::RuntimeShellSnapshot;

use super::VisibleShellController;

pub(super) fn record(
    controller: &VisibleShellController,
    snapshot: &RuntimeShellSnapshot,
    trainer: &str,
    step: usize,
    button: GameButton,
    preferred_partner: Option<usize>,
) {
    if std::env::var_os("CRYSTAL_EARNED_DRIVER_TRACE").is_none() || !step.is_multiple_of(32) {
        return;
    }
    eprintln!(
        "earned-driver trainer={trainer} step={step} turn={:?} button={button:?} preferred={preferred_partner:?} active={:?} hp={:?} text={:?} action={:?} move={:?} switch={:?} party_action={:?} pack_target={} shift={:?}",
        snapshot
            .battle
            .as_ref()
            .map(|battle| battle.player_turns_taken),
        snapshot
            .battle
            .as_ref()
            .and_then(|battle| battle.active_player_party_index),
        snapshot
            .party
            .slots
            .iter()
            .map(|slot| (&slot.pokemon.species.id, slot.pokemon.hp))
            .collect::<Vec<_>>(),
        snapshot.ui.text.as_ref().map(|text| text.label.as_str()),
        controller
            .shell
            .battle_action_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .battle_move_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .battle_switch_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .battle_party_action_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller.shell.battle_pack_target_mode.is_some(),
        controller
            .shell
            .battle_shift_prompt_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
    );
}

/// Keep a bounded battle-driver failure out of story-specific dialogue helpers.
pub(super) fn require_finished(controller: &mut VisibleShellController, trainer: &str) {
    let snapshot = controller
        .snapshot()
        .expect("bounded battle driver outcome");
    assert!(
        snapshot.battle.is_none(),
        "{trainer}: battle driver exhausted its input budget; turn={:?}, action={:?}, move={:?}, switch={:?}, party_action={:?}",
        snapshot
            .battle
            .as_ref()
            .map(|battle| battle.player_turns_taken),
        controller
            .shell
            .battle_action_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .battle_move_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .battle_switch_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .battle_party_action_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
    );
}
