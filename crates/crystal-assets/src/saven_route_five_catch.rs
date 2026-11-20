//! Return to Saven with a Route 5 catch. Each choice runs against the real trainer.
#[path = "saven_practical_retry.rs"]
mod saven_practical_retry;

use std::collections::BTreeSet;

use crystal_core::battle::turn::{BattleAction, BattleSide, battle_action_locked_before_menu};
use crystal_core::models::{Dv, Pokemon};

use super::super::{attack, badge_battle_parties, challenge_with_party, trial_state};
use crate::{GameDataSet, RuntimeMoveReminderCommand, read_verified_compiled_game_pack};

fn catch_bristub(data: &GameDataSet, party: &[Pokemon]) -> Pokemon {
    let route = "MnR05C00R01";
    let grass = data.wild_encounters[route]
        .grass
        .as_ref()
        .expect("Route 5 grass");
    let slot = &grass.day[3];
    assert_eq!((slot.species.as_str(), slot.level), ("BRISTUB", 24));
    let state = trial_state(data, 5, party.to_vec());
    let level = data.ordinary_challenge_level(&state, route, slot.level, 2, 0);
    assert_eq!(level, slot.level, "native-level ordinary encounter");
    let mut bristub = data
        .create_pokemon(&slot.species, level, Dv::from_non_hp(8, 8, 8, 8))
        .expect("create the native rolled species and level");
    data.equip_wild_challenge_moves(&mut bristub, route)
        .expect("native capture moves");
    bristub
}

fn rollout_lesson(data: &GameDataSet, trained: &Pokemon, forgotten: &str) -> Pokemon {
    if trained.moves.iter().any(|m| m.name == "ROLLOUT") {
        return trained.clone();
    }
    assert!(
        data.maps["MnMinneapolisToolWorkshop"]
            .objects
            .iter()
            .any(|actor| actor.script == "MN_MP_INTERIOR_TOOL_APPRENTICE")
    );
    let mut state = trial_state(data, 5, vec![trained.clone()]);
    state.money = 500;
    let forget = trained
        .moves
        .iter()
        .position(|m| m.name == forgotten)
        .expect("partner knows the move selected for replacement");
    data.relearn_party_move(
        &mut state,
        RuntimeMoveReminderCommand {
            party_index: 0,
            move_id: "ROLLOUT".into(),
            forget_index: Some(forget),
        },
        &BTreeSet::new(),
    )
    .expect("available paid Rollout lesson");
    assert_eq!(state.money, 0);
    state.storage.party.pokemon[0]
        .take()
        .expect("workshop student")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    Ordinary,
    Bristub,
    Latchquill,
    Switch,
    RolloutSwitch,
    RolloutLead,
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and compares 54 actual gym battles"]
fn st_cloud_route_five_catch_evolution_and_paid_switch() {
    let file = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Minnesota pack");
    let pack = read_verified_compiled_game_pack(file).expect("verified archive");
    let data = pack.data();
    for (index, starter) in ["POUCHID", "KINDLIT", "BIRCHIT"].into_iter().enumerate() {
        let ordinary = badge_battle_parties::party(data, index, 5);
        assert_eq!(ordinary.len(), 4);
        let caught = catch_bristub(data, &ordinary);
        let mut evolved = caught.clone();
        badge_battle_parties::train(data, &mut evolved, 28);
        assert_eq!(evolved.species.id, "LATCHQUILL");
        let taught = rollout_lesson(data, &evolved, "DEFENSE_CURL");
        if index == 0 {
            for mon in [&caught, &evolved, &taught] {
                let moves = mon
                    .moves
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join("/");
                println!("saven-catch,{},{},{}", mon.species.id, mon.level, moves);
            }
        }
        for seed in [1701, 4242, 9001] {
            for plan in [
                Plan::Ordinary,
                Plan::Bristub,
                Plan::Latchquill,
                Plan::Switch,
                Plan::RolloutSwitch,
                Plan::RolloutLead,
            ] {
                let mut team = ordinary.clone();
                match plan {
                    Plan::Ordinary => {}
                    Plan::Bristub => team.push(caught.clone()),
                    Plan::Latchquill | Plan::Switch => team.push(evolved.clone()),
                    Plan::RolloutSwitch => team.push(taught.clone()),
                    Plan::RolloutLead => team.insert(0, taught.clone()),
                }
                let mut switched = false;
                let mut rollout_starts = 0;
                let mut catch_turns = 0;
                let mut trace = Vec::new();
                let finish =
                    challenge_with_party(data, 5, "MN_GYM_5", team, seed, |data, combat| {
                        let hawk = combat.enemy.species.id == "SILOHAWK";
                        let quill = combat.player.species.id == "LATCHQUILL";
                        catch_turns += u16::from(quill || combat.player.species.id == "BRISTUB");
                        let locked = battle_action_locked_before_menu(combat, BattleSide::Player);
                        let choice = if matches!(plan, Plan::Switch | Plan::RolloutSwitch)
                            && hawk
                            && !quill
                            && !switched
                            && !locked
                            && combat.player_party[4].hp > 0
                            && combat.player_escape_trap.is_none()
                            && combat.player_trap.is_none()
                        {
                            switched = true;
                            BattleAction::Switch { party_index: 4 }
                        } else if quill
                            && (hawk || combat.enemy.species.id == "DEEPULSE")
                            && matches!(plan, Plan::RolloutLead | Plan::RolloutSwitch)
                            && !locked
                            && let Some(slot) = combat
                                .player
                                .moves
                                .iter()
                                .position(|m| m.name == "ROLLOUT" && m.current_pp > 0)
                            && super::super::competitive_move_slots(combat, BattleSide::Player)
                                .contains(&slot)
                        {
                            rollout_starts += 1;
                            BattleAction::Move { slot }
                        } else {
                            attack(data, combat)
                        };
                        if seed == 1701 && matches!(plan, Plan::Switch | Plan::RolloutSwitch) {
                            let action = match &choice {
                                BattleAction::Switch { .. } => "SWITCH",
                                BattleAction::Move { slot } if !locked => combat
                                    .player
                                    .moves
                                    .get(*slot)
                                    .map_or("STRUGGLE", |m| m.name.as_str()),
                                BattleAction::Move { .. } => "LOCKED",
                                _ => unreachable!("only switching or attacking"),
                            };
                            trace.push(format!(
                                "{}({})/{}({})/{action}/last={}",
                                combat.player.species.id,
                                combat.player.hp,
                                combat.enemy.species.id,
                                combat.enemy.hp,
                                combat.enemy_last_move.as_deref().unwrap_or("-")
                            ));
                        }
                        choice
                    });
                assert_eq!(
                    if finish.won {
                        finish.enemy_left
                    } else {
                        finish.player_left
                    },
                    0
                );
                println!(
                    "saven-return,{starter},{seed},{plan:?},{},{},{},{},{switched},{catch_turns},{rollout_starts}",
                    finish.won, finish.turns, finish.player_left, finish.enemy_left
                );
                if !trace.is_empty() {
                    println!("saven-switch-trace,{starter},{plan:?},{}", trace.join("|"));
                }
            }
        }
    }
}
