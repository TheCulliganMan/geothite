use super::{
    BattleAction, BattleCombatState, BattleDisableState, BattleEncoreState, BattleEscapeTrapState,
    BattleEvent, BattleSide, BattleTurnError, BattleTurnOutcome, EnemyMoveSelection, LearnedMove,
    Random, Stat, battle_state, move_data, move_data_with_effect, move_priorities, pokemon,
    resolve_battle_turn_with_enemy_ai_actions, stat_multipliers, type_categories,
    type_effectiveness_table, weather_modifiers,
};
use crate::models::pokemon_type;
use std::collections::BTreeMap;

const ENEMY: BattleSide = BattleSide::Enemy;
const PASS: &str = "BATON_PASS";

fn opponent() -> BattleCombatState {
    let player = pokemon("DITTO", 100, pokemon_type("NORMAL"), "SPLASH");
    let mut enemy = pokemon("AIPOM", 20, pokemon_type("NORMAL"), PASS);
    enemy.moves.push(LearnedMove {
        name: "SPLASH".into(),
        current_pp: 5,
        pp_ups: 0,
    });
    let mut state = battle_state(player, enemy, 0);
    let mut fainted = state.enemy.clone();
    fainted.hp = 0;
    let mut egg = state.enemy.clone();
    egg.is_egg = true;
    state.enemy_party.extend([fainted, egg]);
    state
}

fn turn(
    state: BattleCombatState,
    selected: usize,
    action: BattleAction,
) -> Result<BattleTurnOutcome, BattleTurnError> {
    turn_with_player(state, BattleAction::Move { slot: 0 }, selected, action)
}

fn turn_with_player(
    state: BattleCombatState,
    player: BattleAction,
    selected: usize,
    action: BattleAction,
) -> Result<BattleTurnOutcome, BattleTurnError> {
    let moves = BTreeMap::from([
        (
            PASS.into(),
            move_data_with_effect(PASS, pokemon_type("NORMAL"), 0, 100, PASS),
        ),
        (
            "SPLASH".into(),
            move_data_with_effect("SPLASH", pokemon_type("NORMAL"), 0, 100, "SPLASH"),
        ),
        (
            "STRUGGLE".into(),
            move_data("STRUGGLE", pokemon_type("NORMAL"), 50, 100),
        ),
        (
            "MIRROR_MOVE".into(),
            move_data_with_effect("MIRROR_MOVE", pokemon_type("FLYING"), 0, 100, "MIRROR_MOVE"),
        ),
    ]);
    resolve_battle_turn_with_enemy_ai_actions(
        state,
        player,
        &moves,
        &BTreeMap::new(),
        &move_priorities(),
        &stat_multipliers(),
        &type_categories(),
        &type_effectiveness_table(),
        &weather_modifiers(),
        &mut Random::new(31),
        &mut |_, _| {
            Ok(EnemyMoveSelection {
                slot: selected,
                ai_damage_register: 0,
            })
        },
        &mut |_, _| Ok(action.clone()),
    )
}

#[test]
fn trainer_mirror_move_can_call_a_faster_players_baton_pass_with_a_living_reserve() {
    let player = pokemon("DAKOTASKIP", 120, pokemon_type("BUG"), PASS);
    let enemy = pokemon("PRAIRONDO", 30, pokemon_type("FLYING"), "MIRROR_MOVE");
    let mut state = battle_state(player, enemy, 0);
    let spare = state.enemy_party[1].clone();
    state.enemy_party[1].hp = 0;
    state.enemy_party.extend([spare.clone(), spare]);
    state.enemy.stat_boosts.insert(Stat::Attack, 2);
    let player = BattleAction::MoveSwitch {
        slot: 0,
        party_index: 1,
    };
    let enemy = BattleAction::Move { slot: 0 };
    let mut linked = state.clone();
    linked.link_battle = true;
    assert!(matches!(
        turn_with_player(linked, player.clone(), 0, enemy.clone()),
        Err(BattleTurnError::MissingMoveSwitchTarget { side: ENEMY, .. })
    ));
    let outcome = turn_with_player(state, player, 0, enemy).expect("a copied Baton Pass resolves");
    assert!(outcome.events.contains(&BattleEvent::MirrorMoveSelected {
        side: ENEMY,
        move_name: "MIRROR_MOVE".into(),
        copied_move: PASS.into(),
    }));
    assert_eq!(outcome.state.player_party_index, 1);
    assert_eq!(outcome.state.enemy_party_index, 2);
    assert_eq!(outcome.state.enemy.stat_boosts[&Stat::Attack], 2);
    assert_eq!(outcome.state.enemy_party[2], outcome.state.enemy);
    assert_eq!(outcome.state.player_party[0].moves[0].current_pp, 4);
    assert_eq!(outcome.state.enemy_party[0].moves[0].current_pp, 4);
    assert_eq!(outcome.state.enemy_party[3].moves[0].current_pp, 5);
}

#[test]
fn trainer_baton_pass_resolves_at_move_speed_and_transfers_the_field_state() {
    let mut state = opponent();
    state.enemy.stat_boosts.insert(Stat::Attack, 3);
    state.enemy.focus_energy = true;
    state.enemy.perish_song_turns = 3;
    state.enemy_substitute_hp = 14;
    state.enemy_disable = Some(BattleDisableState {
        move_name: "SPLASH".into(),
        turns_remaining: 3,
    });
    state.player_escape_trap = Some(BattleEscapeTrapState {
        source: ENEMY,
        move_name: "MEAN_LOOK".into(),
    });
    let trap = state.player_escape_trap.clone();
    let ally_attack = state.enemy_party[1].attack;
    let outcome = turn(
        state,
        0,
        BattleAction::MoveSwitch {
            slot: 0,
            party_index: 1,
        },
    )
    .expect("the trainer can pass to its conscious reserve");
    let enemy_switched = outcome
        .events
        .iter()
        .position(|e| {
            matches!(
                e,
                BattleEvent::Switched {
                    side: ENEMY,
                    party_index: 1
                }
            )
        })
        .expect("incoming ally");
    let player_acted = outcome
        .events
        .iter()
        .position(|e| {
            matches!(
                e,
                BattleEvent::MoveUsed {
                    side: BattleSide::Player,
                    ..
                }
            )
        })
        .expect("faster player acted");
    assert!(player_acted < enemy_switched);
    assert!(outcome.events[..enemy_switched].iter().any(|e| matches!(
        e,
        BattleEvent::BatonPassed {
            side: ENEMY,
            party_index: 1,
            ..
        }
    )));
    let state = outcome.state;
    assert_eq!(state.enemy_party_index, 1);
    assert_eq!(state.enemy_party[0].moves[0].current_pp, 4);
    assert_eq!(state.enemy_party[1], state.enemy);
    assert_eq!(state.enemy.stat_boosts[&Stat::Attack], 3);
    assert_eq!(
        state.enemy_loaded_stats.attack,
        super::apply_stage(&stat_multipliers(), ally_attack, 3).expect("attack +3")
    );
    assert_eq!(state.enemy_substitute_hp, 14);
    assert!(state.enemy.focus_energy);
    assert_eq!(state.enemy.perish_song_turns, 2);
    assert_eq!(state.player_escape_trap, trap);
    assert!(state.enemy_disable.is_none());
    assert!(state.player_last_counter_move.is_none() && state.enemy_last_counter_move.is_none());
}

#[test]
fn trainer_baton_pass_with_only_eggs_or_fainted_allies_uses_a_turn_and_fails() {
    let mut state = opponent();
    state.enemy_party[1].hp = 0;
    state.enemy_encore = Some(BattleEncoreState {
        move_name: PASS.into(),
        turns_remaining: 3,
    });
    state.enemy_substitute_hp = 14;
    let outcome =
        turn(state, 0, BattleAction::Move { slot: 0 }).expect("failed move finishes the turn");
    assert!(outcome.events.contains(&BattleEvent::BatonPassFailed {
        side: ENEMY,
        move_name: PASS.into()
    }));
    assert!(
        !outcome
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::Switched { .. }))
    );
    assert_eq!(outcome.state.enemy_party_index, 0);
    assert_eq!(outcome.state.enemy.moves[0].current_pp, 4);
    assert_eq!(outcome.state.enemy_party[0], outcome.state.enemy);
    assert_eq!(outcome.state.enemy_substitute_hp, 14);
}

#[test]
fn trainer_baton_pass_refuses_substituted_moves_and_invalid_destinations() {
    let state = opponent();
    for (selected, slot, party_index) in [
        (0, 1, 1),
        (1, 1, 1),
        (0, 0, 0),
        (0, 0, 2),
        (0, 0, 3),
        (0, 0, 4),
    ] {
        let action = BattleAction::MoveSwitch { slot, party_index };
        let error =
            turn(state.clone(), selected, action.clone()).expect_err("illegal post-order switch");
        assert_eq!(
            error,
            BattleTurnError::InvalidEnemyPostOrderAction {
                selected_move_slot: selected,
                action
            }
        );
    }
    let action = BattleAction::MoveSwitch {
        slot: 0,
        party_index: 1,
    };
    let mut depleted = state.clone();
    for learned in &mut depleted.enemy.moves {
        learned.current_pp = 0;
    }
    let mut disabled = state.clone();
    disabled.enemy_disable = Some(BattleDisableState {
        move_name: PASS.into(),
        turns_remaining: 2,
    });
    let mut encored = state;
    encored.enemy_encore = Some(BattleEncoreState {
        move_name: "SPLASH".into(),
        turns_remaining: 2,
    });
    for state in [depleted, disabled, encored] {
        assert_eq!(
            turn(state, 0, action.clone()).expect_err("unable to use the selected move"),
            BattleTurnError::InvalidEnemyPostOrderAction {
                selected_move_slot: 0,
                action: action.clone()
            }
        );
    }
}
