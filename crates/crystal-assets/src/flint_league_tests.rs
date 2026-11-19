//! Flint's authored party and an interruptible Rollout against a regional opponent.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::{
    CompiledGamePack, GameDataSet, TrainerBattleProfile, read_verified_compiled_game_pack,
};
use crystal_core::battle::start::materialize_trainer_party;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleEvent, BattleSide, BattleTurnInput, BattleTurnOutcome,
    resolve_battle_turn_with_items,
};
use crystal_core::models::{Dv, LearnedMove, Stat};
use crystal_core::random::{BattleRandomSource, Random};

/// Hits a 90%-accurate move, avoids criticals, and gives the same damage variation each turn.
struct OrdinaryHit;

impl BattleRandomSource for OrdinaryHit {
    fn battle_random_byte(&mut self) -> u8 {
        0xc1
    }
}

fn turn(
    data: &GameDataSet,
    state: BattleCombatState,
    player: usize,
    enemy: usize,
) -> BattleTurnOutcome {
    resolve_battle_turn_with_items(
        state,
        BattleTurnInput {
            player: BattleAction::Move { slot: player },
            enemy: BattleAction::Move { slot: enemy },
        },
        &data.moves,
        &data.items,
        &data.move_priorities,
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
        &mut OrdinaryHit,
    )
    .expect("Flint's battle turn")
}

fn rollout_hit(outcome: &BattleTurnOutcome, power: u16, curled: bool) -> u16 {
    assert!(
        outcome
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::RolloutPower {
        side: BattleSide::Enemy, power: actual, defense_curled, ..
    } if *actual == power && *defense_curled == curled))
    );
    outcome
        .events
        .iter()
        .find_map(|e| match e {
            BattleEvent::Damage {
                side: BattleSide::Enemy,
                move_name,
                damage,
                ..
            } if move_name == "ROLLOUT" => Some(*damage),
            _ => None,
        })
        .expect("Rollout damaged Latchquill")
}

fn fixture() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        read_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                        "../../content-packs/minnesota-mothlight-league-and-fair-v01.crystalpack",
                    )
                }),
        )
        .expect("load the League review pack")
    })
}

#[test]
#[ignore = "requires a Minnesota northern-trails or league-and-fair external pack"]
fn flint_league_ace_builds_rollout_and_protect_breaks_the_lock() {
    let pack = fixture();
    let data = pack.data();
    assert!(data.competitive_battles);
    for (id, previous_class, move_flags, item_flags) in [
        ("MN_LEAGUE_1", "COOLTRAINERF", 979, 68),
        ("MN_LEAGUE_2", "HIKER", 265, 68),
        ("MN_LEAGUE_3", "SAGE", 903, 68),
        ("MN_LEAGUE_4", "COOLTRAINERM", 979, 68),
    ] {
        let trainer = &data.trainers.trainers[id];
        assert_eq!(
            TrainerBattleProfile::for_class(&trainer.trainer_class),
            TrainerBattleProfile::for_class(previous_class),
            "{id}'s battle style"
        );
        assert_eq!(
            (trainer.ai_move_flags, trainer.ai_item_switch_flags),
            (move_flags, item_flags)
        );
    }
    let flint = &data.trainers.trainers["MN_LEAGUE_2"];
    let party = materialize_trainer_party(
        flint,
        &data.pokemon,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
    )
    .expect("materialize Flint's four authored teammates");
    let classic = ["SANDSTORM", "ROCK_SLIDE", "BARRIER", "PSYCHIC_M"];
    assert_eq!(flint.name, "FLINT");
    assert!(party.iter().map(|p| p.species.id.as_str()).eq([
        "BANDRAKE",
        "PELLETANK",
        "BURROOT",
        "FORTIGARD"
    ]));
    assert!(party[0].moves.iter().map(|m| m.name.as_str()).eq(classic));
    let ace = &party[3];
    assert_eq!(ace.level, 49);
    let mut latchquill = data
        .create_pokemon("LATCHQUILL", 49, Dv::from_non_hp(12, 12, 12, 12))
        .expect("level-matched Minnesota Latchquill");
    latchquill.moves = ["TACKLE", "PROTECT"]
        .iter()
        .map(|&id| {
            assert!(
                data.learnsets["LATCHQUILL"]
                    .iter()
                    .any(|entry| entry.0 <= 49 && entry.1 == id)
            );
            LearnedMove {
                name: id.into(),
                current_pp: data.moves[id].pp,
                pp_ups: 0,
            }
        })
        .collect();
    let state = BattleCombatState::new(latchquill.clone(), ace.clone())
        .with_parties(vec![latchquill], party)
        .with_party_indices(0, 3);
    if pack.report().manifests.last().is_some_and(|m| {
        matches!(
            m.as_str(),
            "northstar-northern-trails-v1" | "mothlight-battle-cafe-v1"
        )
    }) {
        assert!(
            state
                .enemy
                .moves
                .iter()
                .map(|m| m.name.as_str())
                .eq(classic)
        );
        let barrier = turn(data, state, 0, 2);
        assert_eq!(barrier.state.enemy.stat_boosts[&Stat::Defense], 2);
        let slide = turn(data, barrier.state, 0, 1);
        assert!(
            slide
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Damage {
            side: BattleSide::Enemy, move_name, damage, ..
        } if move_name == "ROCK_SLIDE" && *damage > 0))
        );
        return;
    }
    assert!(
        pack.report()
            .manifests
            .iter()
            .any(|m| m == "northstar-league-and-fair-v1")
    );
    assert!(state.enemy.moves.iter().map(|m| m.name.as_str()).eq([
        "DEFENSE_CURL",
        "ROCK_SLIDE",
        "ROLLOUT",
        "PSYCHIC_M"
    ]));
    let uncurled = turn(data, state.clone(), 0, 2);
    let base_damage = rollout_hit(&uncurled, 30, false);
    let setup = turn(data, state, 0, 0);
    let first = turn(data, setup.state, 0, 2);
    let first_damage = rollout_hit(&first, 60, true);
    assert!(first_damage > base_damage);
    let second = turn(data, first.state, 0, 1);
    assert!(rollout_hit(&second, 120, true) > first_damage);
    assert_eq!(second.state.enemy_rollout_turns, 3);
    let hp = second.state.player.hp;
    let block = turn(data, second.state, 1, 1);
    assert!(block.events.contains(&BattleEvent::MoveProtected {
        side: BattleSide::Enemy,
        target: BattleSide::Player,
        move_name: "ROLLOUT".into(),
    }));
    assert_eq!(block.state.player.hp, hp);
    assert_eq!(block.state.enemy_rollout_turns, 0);
    let recover = turn(data, block.state, 0, 1);
    assert!(recover.events.contains(&BattleEvent::MoveUsed {
        side: BattleSide::Enemy,
        move_name: "ROCK_SLIDE".into(),
    }));
}

#[test]
#[ignore = "requires a Minnesota northern-trails or league-and-fair external pack"]
fn flint_league_planner_uses_the_aces_new_moves_against_physical_opposition() {
    let pack = fixture();
    let data = pack.data();
    let flint = &data.trainers.trainers["MN_LEAGUE_2"];
    let mut party = materialize_trainer_party(
        flint,
        &data.pokemon,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
    )
    .expect("Flint's actual team");
    for partner in &mut party[..3] {
        partner.hp = 0;
    }
    let revised = pack
        .report()
        .manifests
        .iter()
        .any(|m| m == "northstar-league-and-fair-v1");
    for (species, physical_move) in [
        ("LATCHQUILL", "IRON_HEAD"),
        ("FURROWBANK", "SLAM"),
        ("TACONARCH", "ROCK_SLIDE"),
    ] {
        let mut player = data
            .create_pokemon(species, 49, Dv::from_non_hp(12, 12, 12, 12))
            .expect("level-matched regional opponent");
        assert!(
            data.learnsets[species]
                .iter()
                .any(|entry| entry.0 <= player.level && entry.1 == physical_move)
        );
        if !player.moves.iter().any(|m| m.name == physical_move) {
            player.moves[0] = LearnedMove {
                name: physical_move.into(),
                current_pp: data.moves[physical_move].pp,
                pp_ups: 0,
            };
        }
        let player_slot = player
            .moves
            .iter()
            .position(|m| m.name == physical_move)
            .expect("known attack");
        let mut state = BattleCombatState::new(player.clone(), party[3].clone())
            .with_parties(vec![player], party.clone())
            .with_party_indices(0, 3);
        // Flint has seen one attack used against his previous teammate.
        state.player_used_moves.push(physical_move.into());
        let mut rng = Random::new(41);
        let mut used = BTreeSet::new();
        let mut choices = Vec::new();
        for _ in 0..2 {
            if state.player.hp == 0 || state.enemy.hp == 0 || state.enemy_rollout_turns != 0 {
                break;
            }
            let selected = data
                .select_trainer_enemy_move_for_class(
                    &state,
                    &flint.trainer_class,
                    flint.ai_move_flags,
                    &mut rng,
                )
                .expect("Flint picks his move without seeing the player's command");
            let action = data
                .select_trainer_post_order_action_for_class(
                    &state,
                    &flint.trainer_id,
                    &flint.trainer_class,
                    "BATTLETYPE_NORMAL",
                    flint.ai_item_switch_flags,
                    &mut used,
                    selected.slot,
                    &mut rng,
                )
                .expect("Flint confirms his move");
            let BattleAction::Move { slot } = action else {
                break;
            };
            let name = state.enemy.moves[slot].name.clone();
            eprintln!("Flint vs. {species} using {physical_move}: {name}");
            let outcome = turn(data, state, player_slot, slot);
            if outcome.events.contains(&BattleEvent::MoveUsed {
                side: BattleSide::Enemy,
                move_name: name.clone(),
            }) {
                choices.push(name);
            }
            state = outcome.state;
        }
        assert!(!choices.is_empty(), "Flint never acted against {species}");
        if species == "FURROWBANK" {
            let expected = if revised {
                ["DEFENSE_CURL", "ROLLOUT"]
            } else {
                ["BARRIER", "ROCK_SLIDE"]
            };
            assert!(
                choices.iter().map(String::as_str).eq(expected),
                "Flint vs. {species}: {choices:?}"
            );
        }
    }
}
