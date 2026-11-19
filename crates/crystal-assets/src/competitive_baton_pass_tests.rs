//! Drive the same two AI callbacks and turn resolver used by ordinary field trainers.
use std::cell::Cell;
use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::{CompiledGamePack, GameDataSet, read_verified_compiled_game_pack};
use crystal_core::battle::start::materialize_trainer_party;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleEvent, BattleSide, BattleTurnOutcome,
    resolve_battle_turn_with_enemy_ai_actions,
};
use crystal_core::models::{Dv, LearnedMove, Stat};
use crystal_core::random::BattleRandomSource;

fn data() -> &'static GameDataSet {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK")
            .expect("external Minnesota battle pack");
        read_verified_compiled_game_pack(path).expect("load Minnesota battle fixture pack")
    })
    .data()
}

struct Random(u32);

impl BattleRandomSource for Random {
    fn battle_random_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 24) as u8
    }
}

fn trainer_state(
    trainer_id: &str,
    active: usize,
    player_species: &str,
    player_moves: [&str; 4],
) -> BattleCombatState {
    let data = data();
    assert!(data.competitive_battles);
    let trainer = &data.trainers.trainers[trainer_id];
    let mut party = materialize_trainer_party(
        trainer,
        &data.pokemon,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
    )
    .expect("materialize authored party");
    assert_eq!(party[active].species.id, "DAKOTASKIP");
    assert_eq!(party[active].moves[3].name, "BATON_PASS");
    party[active].stat_boosts.insert(Stat::Speed, 2);
    let mut player = data
        .create_pokemon(player_species, 60, Dv::from_non_hp(15, 15, 15, 15))
        .expect("create opposing Pokémon");
    player.moves = player_moves
        .iter()
        .map(|id| LearnedMove {
            name: (*id).into(),
            current_pp: data.moves[*id].pp,
            pp_ups: 0,
        })
        .collect();
    let mut state = BattleCombatState::new(player.clone(), party[active].clone())
        .with_parties(vec![player], party)
        .with_party_indices(0, active);
    state.player_used_moves = player_moves.into_iter().map(str::to_owned).collect();
    state
}

fn trainer_turn(
    state: BattleCombatState,
    trainer_id: &str,
    player_slot: usize,
) -> (BattleAction, BattleTurnOutcome) {
    let data = data();
    let trainer = &data.trainers.trainers[trainer_id];
    let slot = Cell::new(None);
    let mut chosen = None;
    let mut used = BTreeSet::new();
    let outcome = resolve_battle_turn_with_enemy_ai_actions(
        state,
        BattleAction::Move { slot: player_slot },
        &data.moves,
        &data.items,
        &data.move_priorities,
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
        &mut Random(42),
        &mut |state, rng| {
            let selected = data
                .select_trainer_enemy_move_for_class(
                    state,
                    &trainer.trainer_class,
                    trainer.ai_move_flags,
                    rng,
                )
                .expect("choose move before order");
            slot.set(Some(selected.slot));
            Ok(selected)
        },
        &mut |state, rng| {
            let action = data
                .select_trainer_post_order_action_for_class(
                    state,
                    trainer_id,
                    &trainer.trainer_class,
                    "BATTLETYPE_NORMAL",
                    trainer.ai_item_switch_flags,
                    &mut used,
                    slot.get().expect("move preselected"),
                    rng,
                )
                .expect("choose the action for the preselected move");
            chosen = Some(action.clone());
            Ok(action)
        },
    )
    .expect("resolve ordinary trainer AI turn");
    (chosen.expect("trainer's final action"), outcome)
}

fn assert_pass(state: BattleCombatState, trainer_id: &str, expected_target: usize) {
    let outgoing = state.enemy_party_index;
    let initial_pp = state.enemy.moves[3].current_pp;
    let (action, outcome) = trainer_turn(state, trainer_id, 0);
    assert_eq!(
        action,
        BattleAction::MoveSwitch {
            slot: 3,
            party_index: expected_target
        }
    );
    assert!(
        outcome
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::BatonPassed {
        side: BattleSide::Enemy, party_index, ..
    } if *party_index == expected_target))
    );
    assert_eq!(outcome.state.enemy_party_index, expected_target);
    assert_eq!(outcome.state.enemy.stat_boosts.get(&Stat::Speed), Some(&2));
    assert_eq!(
        outcome.state.enemy_party[outgoing].moves[3].current_pp,
        initial_pp - 1
    );
}

#[test]
#[ignore = "requires an external CRYSTAL_MINNESOTA_BATTLE_TEST_PACK"]
fn saven_rematch_baton_pass_selects_the_ground_recipient_against_electric() {
    let mut state = trainer_state(
        "MN_GYM_5_REMATCH",
        0,
        "PIPULSE",
        ["THUNDERBOLT", "SPARK", "THUNDERSHOCK", "THUNDER_WAVE"],
    );
    assert_eq!(state.enemy_party.len(), 5);
    assert_eq!(state.enemy_party[3].species.id, "PRAIRONDO");
    // The faster lead is one hit from fainting, with its Agility boost intact.
    state.enemy.hp = 1;
    state.enemy_party[0].hp = 1;
    assert_pass(state, "MN_GYM_5_REMATCH", 3);
}

#[test]
#[ignore = "requires an external CRYSTAL_MINNESOTA_BATTLE_TEST_PACK"]
fn fern_last_listed_baton_pass_returns_to_the_living_earlier_partner() {
    let mut state = trainer_state(
        "MN_LEAGUE_1",
        3,
        "AGATOT",
        ["ROCK_SLIDE", "ROCK_THROW", "ANCIENTPOWER", "SANDSTORM"],
    );
    assert_eq!(state.enemy_party.len(), 4);
    assert_eq!(state.enemy_party[2].species.id, "TRAPNACK");
    state.enemy_party[0].hp = 0;
    state.enemy_party[1].hp = 0;
    state.enemy_party[3].hp = 1;
    state.enemy.hp = 1;
    assert_pass(state, "MN_LEAGUE_1", 2);
}

#[test]
#[ignore = "requires an external CRYSTAL_MINNESOTA_BATTLE_TEST_PACK"]
fn lone_enemy_uses_another_move_or_spends_the_forced_baton_pass_turn() {
    let mut state = trainer_state(
        "MN_LEAGUE_1",
        3,
        "AGATOT",
        ["ROCK_SLIDE", "ROCK_THROW", "ANCIENTPOWER", "SANDSTORM"],
    );
    state.enemy_party[0].hp = 0;
    state.enemy_party[1].is_egg = true;
    state.enemy_party[2].hp = 0;
    let (ordinary, _) = trainer_turn(state.clone(), "MN_LEAGUE_1", 3);
    assert!(matches!(ordinary, BattleAction::Move { slot } if slot != 3));
    for m in &mut state.enemy.moves[..3] {
        m.current_pp = 0;
    }
    state.enemy_party[3] = state.enemy.clone();
    let pp = state.enemy.moves[3].current_pp;
    let (forced, outcome) = trainer_turn(state, "MN_LEAGUE_1", 3);
    assert_eq!(forced, BattleAction::Move { slot: 3 });
    assert_eq!(outcome.state.enemy_party_index, 3);
    assert_eq!(outcome.state.enemy.moves[3].current_pp, pp - 1);
    assert!(
        !outcome
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::BatonPassed { .. }))
    );
}
