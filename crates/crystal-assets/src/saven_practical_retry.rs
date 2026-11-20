//! Separates a general-purpose rematch with local medicine from a learned Rock combination.
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleSide, battle_action_locked_before_menu,
};
use crystal_core::state::GameState;
use crystal_core::systems::shop::close_active_shop;

use super::super::super::{
    TrialDivider, attack, badge_battle_parties, challenge_state, competitive_move_slots,
    ely_challenger_tactics, trial_state,
};
use super::{catch_bristub, rollout_lesson};
use crate::{GameDataSet, read_verified_compiled_game_pack};

const POTION: &str = "SUPER_POTION";

fn buy_two(data: &GameDataSet, state: &mut GameState) {
    let mart = "MnStCloudMart";
    let clerk = data.maps[mart]
        .script_shop_commands
        .iter()
        .find(|command| command.mart_id == "MART_MN_ST_CLOUD")
        .expect("town's ordinary medicine counter");
    state.money = 1_400;
    data.open_script_shop(state, mart, mart, &clerk.source_script, clerk.command_index)
        .expect("open St. Cloud stock");
    assert!(
        data.buy_shop_item(state, POTION, 2)
            .expect("two Super Potions")
            .success
    );
    assert_eq!(state.money, 0);
    assert_eq!(state.bag.quantity(&data.items[POTION]), 2);
    close_active_shop(state).expect("leave shop");
}

fn can_switch(combat: &BattleCombatState) -> bool {
    !battle_action_locked_before_menu(combat, BattleSide::Player)
        && combat.player_escape_trap.is_none()
        && combat.player_trap.is_none()
}

fn flying_reserve_for_dig(combat: &BattleCombatState) -> Option<usize> {
    let flying = |p: &crystal_core::models::Pokemon| {
        p.species.type1 == "FLYING" || p.species.type2 == "FLYING"
    };
    if combat.enemy_airborne_move.as_deref() != Some("DIG")
        || flying(&combat.player)
        || !can_switch(combat)
    {
        return None;
    }
    combat
        .player_party
        .iter()
        .position(|p| p.hp > 0 && flying(p))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    Four,
    FourWithMedicine,
    FiveWithBristub,
    FiveWithLatchquill,
    CurlSwitch,
    CurlLead,
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves 54 prepared St. Cloud battles"]
fn st_cloud_practical_retries_with_two_super_potions_or_defense_curl() {
    let path = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Minnesota pack");
    let pack = read_verified_compiled_game_pack(path).expect("verified pack");
    let data = pack.data();
    for (slot, starter) in ["POUCHID", "KINDLIT", "BIRCHIT"].into_iter().enumerate() {
        let ordinary = badge_battle_parties::party(data, slot, 5);
        let catch = catch_bristub(data, &ordinary);
        let mut raised = catch.clone();
        badge_battle_parties::train(data, &mut raised, 28);
        let curl = rollout_lesson(data, &raised, "BIND");
        assert!(curl.moves.iter().any(|m| m.name == "DEFENSE_CURL"));
        for seed in [1701, 4242, 9001] {
            for plan in [
                Plan::Four,
                Plan::FourWithMedicine,
                Plan::FiveWithBristub,
                Plan::FiveWithLatchquill,
                Plan::CurlSwitch,
                Plan::CurlLead,
            ] {
                let combo = matches!(plan, Plan::CurlSwitch | Plan::CurlLead);
                let medicine = !combo && plan != Plan::Four;
                let mut team = ordinary.clone();
                match plan {
                    Plan::Four | Plan::FourWithMedicine => {}
                    Plan::FiveWithBristub => team.push(catch.clone()),
                    Plan::FiveWithLatchquill => team.push(raised.clone()),
                    Plan::CurlSwitch => team.push(curl.clone()),
                    Plan::CurlLead => team.insert(0, curl.clone()),
                }
                let mut state = trial_state(data, 5, team);
                if medicine {
                    buy_two(data, &mut state);
                }
                let mut tactics = ely_challenger_tactics::Tactics::default();
                let mut paid_switches = 0;
                let mut dig_switches = 0;
                let mut pending_dig: Option<(usize, u16)> = None;
                let mut unharmed = 0;
                let mut curls = 0;
                let mut rollouts = 0;
                let mut used = 0;
                let result = challenge_state(
                    data,
                    &mut state,
                    "MN_GYM_5",
                    &mut TrialDivider(seed),
                    |data, state, combat| {
                        if let Some((slot, hp)) = pending_dig.take()
                            && combat.enemy.species.id == "BANKWING"
                            && combat.enemy_airborne_move.is_none()
                            && combat.enemy_last_move.as_deref() == Some("DIG")
                            && combat.player_party_index == slot
                            && combat.player.hp == hp
                        {
                            unharmed += 1;
                        }
                        let choice = if combo {
                            if plan == Plan::CurlSwitch
                                && paid_switches == 0
                                && combat.enemy.species.id == "SILOHAWK"
                                && combat.player_party_index != 4
                                && combat.player_party[4].hp > 0
                                && can_switch(combat)
                            {
                                BattleAction::Switch { party_index: 4 }
                            } else if combat.player.species.id == "LATCHQUILL"
                                && matches!(
                                    combat.enemy.species.id.as_str(),
                                    "DEEPULSE" | "SILOHAWK"
                                )
                                && !battle_action_locked_before_menu(combat, BattleSide::Player)
                            {
                                let name = if combat.player_defense_curled {
                                    "ROLLOUT"
                                } else {
                                    "DEFENSE_CURL"
                                };
                                let eligible = competitive_move_slots(combat, BattleSide::Player);
                                if let Some(slot) = eligible.into_iter().find(|i| {
                                    combat.player.moves.get(*i).is_some_and(|m| m.name == name)
                                }) {
                                    if name == "ROLLOUT" {
                                        rollouts += 1;
                                    } else {
                                        curls += 1;
                                    }
                                    BattleAction::Move { slot }
                                } else {
                                    attack(data, combat)
                                }
                            } else {
                                attack(data, combat)
                            }
                        } else if let Some(party_index) = flying_reserve_for_dig(combat) {
                            dig_switches += 1;
                            pending_dig = Some((party_index, combat.player_party[party_index].hp));
                            BattleAction::Switch { party_index }
                        } else {
                            tactics.choose(data, state, combat, medicine, false)
                        };
                        paid_switches +=
                            usize::from(matches!(&choice, BattleAction::Switch { .. }));
                        used += usize::from(matches!(&choice, BattleAction::PartyItem { .. }));
                        choice
                    },
                    if combo {
                        None
                    } else {
                        Some(ely_challenger_tactics::best)
                    },
                    !combo,
                );
                let remaining = state.bag.quantity(&data.items[POTION]);
                assert_eq!(usize::from(remaining) + used, if medicine { 2 } else { 0 });
                assert_eq!(
                    if result.won {
                        result.enemy_left
                    } else {
                        result.player_left
                    },
                    0
                );
                println!(
                    "saven-practical,{starter},{seed},{plan:?},{},{},{},{},{used},{paid_switches},{dig_switches},{curls},{rollouts},{unharmed}",
                    result.won, result.turns, result.player_left, result.enemy_left
                );
            }
        }
    }
}
