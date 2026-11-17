//! Pure transaction tests; no pack, controller, renderer, or game input.

use std::collections::BTreeSet;

use crystal_core::models::{BaseStats, Dv, Item, LearnedMove, Move, Pokemon, PokemonSpecies};
use crystal_core::state::GameState;
use crystal_core::systems::learnsets::LearnsetEntry;
use crystal_core::systems::special_routines::{
    HappinessChangeEntry, HappinessData, HappinessServiceOutcome, SpecialRoutineEffect,
};

use crate::{GameDataSet, RuntimeMoveReminderCommand, SpecialRoutineRule};

fn fixture(full: bool) -> (GameDataSet, GameState) {
    let species = PokemonSpecies::new_for_tests("TESTMON", BaseStats::new(50, 50, 50, 50, 50, 50));
    let mut pokemon = Pokemon::new_for_tests(species.clone(), 12, Dv::default());
    pokemon.moves = if full {
        vec!["TACKLE", "SURF", "BITE", "TAIL_WHIP"]
    } else {
        vec!["TACKLE"]
    }
    .into_iter()
    .enumerate()
    .map(|(index, name)| LearnedMove {
        name: name.into(),
        current_pp: 2 + index as u8,
        pp_ups: index as u8,
    })
    .collect();
    let mut state = GameState {
        money: 2000,
        ..GameState::default()
    };
    state.storage.party.pokemon[0] = Some(pokemon.clone());
    state.storage.party.pokemon[1] = Some(pokemon);
    state.sync_party_from_storage();
    let mut data = GameDataSet::default();
    data.pokemon.insert(species.id.clone(), species);
    data.learnsets.insert(
        "TESTMON".into(),
        vec![
            LearnsetEntry(1, "TACKLE".into()),
            LearnsetEntry(5, "GROWL".into()),
            LearnsetEntry(13, "SCREECH".into()),
        ],
    );
    for (index, name) in ["TACKLE", "GROWL", "SURF", "BITE", "TAIL_WHIP", "SCREECH"]
        .iter()
        .enumerate()
    {
        data.moves.insert(
            (*name).into(),
            Move {
                source_index: index as u16 + 1,
                name: (*name).into(),
                move_type: "NORMAL".into(),
                power: 40,
                accuracy: 100,
                pp: 20,
                effect: "EFFECT_NORMAL_HIT".into(),
                effect_chance: 0,
                stat: None,
                amount: None,
            },
        );
    }
    data.special_routines
        .insert("MoveTutor".into(), SpecialRoutineRule::default());
    data.happiness_data = Some(HappinessData {
        changes: [(
            5,
            HappinessChangeEntry {
                code: "HAPPINESS_LEARNMOVE".into(),
                low: 1,
                mid: 1,
                high: 0,
            },
        )]
        .into(),
        services: [(
            "OlderHaircutBrother".into(),
            vec![HappinessServiceOutcome {
                roll_weight: 1,
                script_value: 1,
                change_code: 5,
            }],
        )]
        .into(),
    });
    data.items.insert("HM_03".into(), hm_surf());
    (data, state)
}

fn hm_surf() -> Item {
    Item {
        name: "HM_03".into(),
        description: "Surf".into(),
        effect: "NONE".into(),
        status_heals: Vec::new(),
        revive_hp_percent: None,
        party_revive_hp_percent: None,
        pp_restore_scope: None,
        pp_restore_points: None,
        pp_up_stages: None,
        vitamin_stat: None,
        vitamin_stat_exp: None,
        vitamin_max_stat_exp: None,
        rare_candy_level_gain: None,
        battle_stat_boost_stat: None,
        battle_stat_boost_stages: None,
        battle_escape_mode: None,
        battle_capture_ball: None,
        battle_focus_energy: None,
        battle_stat_drop_guard: None,
        confusion_heal: None,
        confusion_inflict_turns: None,
        repel_steps: None,
        escape_rope_mode: None,
        price: 0,
        held_effect: "HELD_NONE".into(),
        parameter: 0,
        property: String::new(),
        pocket: "TM_HM".into(),
        field_menu: "ITEMMENU_NOUSE".into(),
        field_usable: false,
        battle_menu: "ITEMMENU_NOUSE".into(),
        battle_usable: false,
        script_name: "HM_03".into(),
        consumable: false,
        tmhm_index: Some(3),
        tmhm_move: Some("SURF".into()),
    }
}

fn command(move_id: &str, forget_index: Option<usize>) -> RuntimeMoveReminderCommand {
    RuntimeMoveReminderCommand {
        party_index: 0,
        move_id: move_id.into(),
        forget_index,
    }
}

fn assert_rejected_without_mutation(
    data: &GameDataSet,
    mut state: GameState,
    choice: RuntimeMoveReminderCommand,
) {
    let before = serde_json::to_vec(&state).expect("snapshot original state");
    assert!(
        data.relearn_party_move(&mut state, choice, &BTreeSet::new())
            .is_err()
    );
    assert_eq!(
        serde_json::to_vec(&state).expect("snapshot rejected state"),
        before
    );
}

#[test]
fn invalid_choices_and_funds_preserve_every_state_byte() {
    let (data, state) = fixture(false);
    for choice in [
        command("TACKLE", None),
        command("SCREECH", None),
        command("NOT_A_MOVE", None),
        command("GROWL", Some(0)),
    ] {
        assert_rejected_without_mutation(&data, state.clone(), choice);
    }
    let mut poor = state.clone();
    poor.money = 499;
    assert_rejected_without_mutation(&data, poor, command("GROWL", None));
    let mut egg = state;
    egg.storage.party.pokemon[0].as_mut().expect("party").is_egg = true;
    egg.sync_party_from_storage();
    assert_rejected_without_mutation(&data, egg, command("GROWL", None));
    let (data, state) = fixture(true);
    for replacement in [None, Some(1), Some(4)] {
        assert_rejected_without_mutation(&data, state.clone(), command("GROWL", replacement));
    }
}

#[test]
fn failure_after_shared_teaching_starts_rolls_back_moves_and_fee() {
    let (mut data, state) = fixture(true);
    // Keep the table structurally nonempty so MoveTutor appends the move before
    // failing to find its actual happiness change. The authoritative state must
    // still retain the original four moves, PP, happiness, money and scripts.
    let changes = &mut data.happiness_data.as_mut().expect("happiness").changes;
    let entry = changes.remove(&5).expect("learn-move change");
    changes.insert(1, entry);
    assert_rejected_without_mutation(&data, state, command("GROWL", Some(0)));
}

#[test]
fn success_commits_one_fee_and_preserves_unselected_slots_and_move_data() {
    for full in [false, true] {
        let (data, mut state) = fixture(full);
        let original_party = state.storage.party.clone();
        let original_moves = original_party.pokemon[0]
            .as_ref()
            .expect("party")
            .moves
            .clone();
        let replacement = full.then_some(2);
        let outcome = data
            .relearn_party_move(&mut state, command("GROWL", replacement), &BTreeSet::new())
            .expect("learn eligible move");
        assert_eq!(state.money, 1500);
        assert_eq!(outcome.routine, "MoveReminder");
        assert!(matches!(
            outcome.effect,
            SpecialRoutineEffect::MoveTutor { learned: true, .. }
        ));
        let mut expected_moves = original_moves;
        let learned = LearnedMove {
            name: "GROWL".into(),
            current_pp: 20,
            pp_ups: 0,
        };
        if let Some(index) = replacement {
            expected_moves[index] = learned;
        } else {
            expected_moves.push(learned);
        }
        assert_eq!(
            state.storage.party.pokemon[0]
                .as_ref()
                .expect("party")
                .moves,
            expected_moves
        );
        assert_eq!(
            state.storage.party.pokemon[1..],
            original_party.pokemon[1..]
        );
        assert_eq!(state.script_runtime.script_value.as_deref(), Some("0"));
        let before_retry = serde_json::to_vec(&state).expect("successful snapshot");
        assert!(
            data.relearn_party_move(&mut state, command("GROWL", replacement), &BTreeSet::new())
                .is_err()
        );
        assert_eq!(
            serde_json::to_vec(&state).expect("rejected repeat"),
            before_retry
        );
    }
}
