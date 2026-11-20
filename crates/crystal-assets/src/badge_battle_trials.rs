//! Isolated first-visit fights with three ordinary, geographically available parties.
#[path = "badge_battle_parties.rs"]
mod badge_battle_parties;
#[path = "badge_battle_retry.rs"]
mod badge_battle_retry;
#[path = "badge_battle_saven.rs"]
mod badge_battle_saven;
#[path = "ely_challenge_run.rs"]
mod ely_challenge_run;
#[path = "ely_challenger_parties.rs"]
mod ely_challenger_parties;

use std::cell::Cell;
use std::collections::BTreeSet;
use std::convert::Infallible;

use crystal_core::battle::start::{TrainerBattleStartStatus, switch_active_battle_party_index};
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleEvent, BattleSide, active_battle_combat_state,
};
use crystal_core::models::Pokemon;
use crystal_core::random::DividerSource;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::item_use::ItemUseContext;
use crystal_core::systems::script_flags::{BADGE_ENGINE_FLAGS, apply_script_flag_mutation};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use super::competitive_move_slots;
use crate::{GameDataSet, read_verified_compiled_game_pack};

struct TrialDivider(u32);

impl DividerSource for TrialDivider {
    type Error = Infallible;
    fn next_divider(&mut self) -> Result<u8, Self::Error> {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        Ok((self.0 >> 24) as u8)
    }
}

// This policy sees current typing, stats and conditions. It neither reads
// a future random roll nor chooses a different team for a particular leader.
fn attack(data: &GameDataSet, combat: &BattleCombatState) -> BattleAction {
    let mut search = data.competitive_search();
    let slots = competitive_move_slots(combat, BattleSide::Player);
    let slot = slots
        .into_iter()
        .max_by_key(|&slot| {
            let score = combat.player.moves.get(slot).and_then(|m| {
                data.competitive_move_expected_damage(combat, BattleSide::Player, m, &mut search)
            });
            (score.unwrap_or(0), std::cmp::Reverse(slot))
        })
        .expect("at least the Struggle carrier");
    BattleAction::Move { slot }
}

struct Finish {
    won: bool,
    turns: u16,
    enemy_left: usize,
    player_left: usize,
}

fn challenge(
    data: &GameDataSet,
    stage: usize,
    trainer_id: &str,
    starter: usize,
    seed: u32,
) -> Finish {
    challenge_with_party(
        data,
        stage,
        trainer_id,
        badge_battle_parties::party(data, starter, stage),
        seed,
        attack,
    )
}

fn challenge_with_party(
    data: &GameDataSet,
    stage: usize,
    trainer_id: &str,
    party: Vec<Pokemon>,
    seed: u32,
    mut choose: impl FnMut(&GameDataSet, &BattleCombatState) -> BattleAction,
) -> Finish {
    let mut state = trial_state(data, stage, party);
    challenge_state(
        data,
        &mut state,
        trainer_id,
        &mut TrialDivider(seed),
        |data, _, combat| choose(data, combat),
        None,
        false,
    )
}

fn trial_state(data: &GameDataSet, stage: usize, party: Vec<Pokemon>) -> GameState {
    let mut state = GameState::reset_wram_for_new_game();
    let gyms = [
        "MnRochesterGym",
        "MnMankatoGym",
        "MnMinneapolisGym",
        "MnStPaulGym",
        "MnStCloudGym",
        "MnBrainerdGym",
        "MnBemidjiGym",
        "MnDuluthGym",
    ];
    for (index, gym) in gyms.iter().enumerate() {
        let badge = BADGE_ENGINE_FLAGS[index];
        let earned = index + 1 < stage;
        if earned {
            let grant = data.maps[*gym]
                .script_flag_commands
                .iter()
                .find(|command| command.command == "setflag" && command.flag_id == badge)
                .expect("badge awarded by the installed gym");
            apply_script_flag_mutation(&mut state, grant.clone())
                .expect("native gym badge grant updates both representations");
        }
        assert_eq!(
            state.flags.is_engine_flag_set(badge).expect("badge flag"),
            earned
        );
        assert_eq!(state.badges.johto[index], earned);
    }
    for pokemon in party {
        assert!(state.storage.party.add_pokemon(pokemon));
    }
    state.sync_party_from_storage();
    state
}

// Isolated tests retain their original forced-slot order. A prepared challenger
// can choose the next partner after a faint and keep the party across rooms.
fn challenge_state(
    data: &GameDataSet,
    state: &mut GameState,
    trainer_id: &str,
    divider: &mut TrialDivider,
    mut choose: impl FnMut(&GameDataSet, &GameState, &BattleCombatState) -> BattleAction,
    replacement: Option<fn(&GameDataSet, &BattleCombatState) -> usize>,
    use_shift: bool,
) -> Finish {
    let (room, entry) = data
        .maps
        .values()
        .find_map(|map| {
            let battle = map
                .scripted_trainer_battles
                .iter()
                .find(|b| b.request.trainer_id == trainer_id)?;
            Some((map, battle))
        })
        .expect("trainer is actually invoked by a map");
    state.overworld = OverworldMemory::Active {
        map_name: room.id.clone(),
        tile: TilePosition::new(0, 0),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    let started = data
        .start_scripted_trainer_battle(
            state,
            &room.id,
            &room.id,
            &entry.source_script,
            entry.startbattle_command_index,
        )
        .expect("native boss script");
    assert!(matches!(started, TrainerBattleStartStatus::Started(_)));
    let trainer = &data.trainers.trainers[trainer_id];
    let mut used = BTreeSet::new();
    for turns in 1..=160 {
        let combat = active_battle_combat_state(state).expect("live fight");
        let choice = choose(data, state, &combat);
        // The live ResolveActiveBattleTurn mutation spends the Bag item before
        // the ordinary battle engine applies its effect and the opposing action.
        let item_used = if let BattleAction::PartyItem {
            item_id,
            party_index,
            ..
        } = &choice
        {
            let before = state.bag.quantity(&data.items[item_id]);
            let receipt = data
                .use_bag_item(state, item_id, ItemUseContext::Battle)
                .expect("spend a purchased item");
            assert!(receipt.consumed && before > 0);
            assert_eq!(state.bag.quantity(&data.items[item_id]), before - 1);
            Some((item_id.clone(), *party_index))
        } else {
            None
        };
        let selected = Cell::new(None);
        let turn = data
            .resolve_active_battle_turn_with_enemy_ai_actions_with_divider(
                state,
                choice,
                divider,
                &mut |c, rng| {
                    let m = data
                        .select_trainer_enemy_move_for_class(
                            c,
                            &trainer.trainer_class,
                            trainer.ai_move_flags,
                            rng,
                        )
                        .expect("field trainer move");
                    selected.set(Some(m.slot));
                    Ok(m)
                },
                &mut |c, rng| {
                    Ok(data
                        .select_trainer_post_order_action_for_class(
                            c,
                            trainer_id,
                            &trainer.trainer_class,
                            &entry.request.battle_type,
                            trainer.ai_item_switch_flags,
                            &mut used,
                            selected.get().expect("preselected move"),
                            rng,
                        )
                        .expect("field trainer action"))
                },
            )
            .unwrap_or_else(|e| panic!("{trainer_id}, turn {turns}: {e:#}"));
        if let Some((item, slot)) = item_used {
            assert!(turn.events.iter().any(|event| matches!(event, BattleEvent::BattlePartyItemEffect { side: BattleSide::Player, party_index, .. } if *party_index == slot)));
            println!("ely-used,{trainer_id},{turns},{item},{slot}");
        }
        let enemy_left = turn.state.enemy_party.iter().filter(|p| p.hp > 0).count();
        let player_left = turn.state.player_party.iter().filter(|p| p.hp > 0).count();
        if player_left == 0 {
            return Finish {
                won: false,
                turns,
                enemy_left,
                player_left,
            };
        }
        if turn.state.player.hp == 0 {
            // Both partners can faint on one turn. Complete the player's
            // normal replacement before awarding the opposing knockout.
            let i = replacement.map_or_else(
                || {
                    state
                        .storage
                        .party
                        .pokemon
                        .iter()
                        .position(|p| p.as_ref().is_some_and(|p| p.hp > 0))
                        .expect("able partner")
                },
                |replacement| replacement(data, &turn.state),
            );
            switch_active_battle_party_index(state, i, &data.items)
                .expect("normal forced player switch");
        }
        if turn.state.enemy.hp == 0 {
            data.claim_active_trainer_battle_rewards_now(state)
                .expect("normal faint reward");
            if replacement.is_some() {
                // The prepared team keeps its selected four moves on a level-up.
                while state.pending_move_learn.is_some() {
                    data.decline_pending_move_learn(state)
                        .expect("keep prepared moves");
                }
            }
            let shift = if use_shift
                && turn.state.player.hp > 0
                && state.options.battle_style == crystal_core::state::BattleStyle::Shift
            {
                let current = active_battle_combat_state(state).expect("trainer's announcement");
                // This battle began on the current build. Only a published
                // trainer announcement actually gives its player a Shift turn.
                state.pending_trainer_replacement().map(|index| {
                    let preview = crystal_core::battle::turn::preview_battle_replacement(
                        &current,
                        BattleSide::Enemy,
                        index,
                    )
                    .expect("announced partner");
                    (index, replacement.expect("Shift decision")(data, &preview))
                })
            } else {
                None
            };
            if data
                .advance_active_trainer_battle(state)
                .expect("trainer replacement")
                .trainer_defeated
            {
                return Finish {
                    won: true,
                    turns,
                    enemy_left,
                    player_left,
                };
            }
            // Shift is an optional action after an enemy knockout. A double
            // knockout has already taken its mandatory player replacement.
            if let Some((announced, slot)) = shift {
                let next = active_battle_combat_state(state)
                    .expect("trainer sent out the announced partner");
                assert_eq!(next.enemy_party_index, announced);
                if slot != next.player_party_index {
                    switch_active_battle_party_index(state, slot, &data.items)
                        .expect("normal Shift switch");
                    println!(
                        "ely-shift,{trainer_id},{turns},{},{}",
                        next.player.species.id, next.player_party[slot].species.id
                    );
                }
            }
        }
    }
    panic!("{trainer_id}: exceeded 160 turns");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and runs full production battle turns"]
fn representative_badge_and_league_battles() {
    let p =
        std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("selected Minnesota pack");
    let pack = read_verified_compiled_game_pack(p).expect("verified archive");
    let data = pack.data();
    assert!(data.competitive_battles);
    let only = std::env::var("MN_BATTLE_REVIEW_TRAINER").ok();
    let seed = std::env::var("MN_BATTLE_REVIEW_SEED")
        .ok()
        .map(|v| v.parse::<u32>().expect("decimal seed"))
        .unwrap_or(1701);
    let mut trials = 0;
    for (i, id) in [
        "MN_GYM_1",
        "MN_GYM_2",
        "MN_GYM_3",
        "MN_GYM_4",
        "MN_GYM_5",
        "MN_GYM_6",
        "MN_GYM_7",
        "MN_GYM_8",
        "MN_LEAGUE_1",
        "MN_LEAGUE_2",
        "MN_LEAGUE_3",
        "MN_LEAGUE_4",
        "MN_CHAMPION",
    ]
    .iter()
    .enumerate()
    {
        if only.as_ref().is_some_and(|filter| filter != id) {
            continue;
        }
        for starter in 0..3 {
            let f = challenge(data, (i + 1).min(9), id, starter, seed);
            assert_eq!(if f.won { f.enemy_left } else { f.player_left }, 0);
            println!(
                "trial,{id},{starter},{seed},{},{},{},{}",
                f.won, f.turns, f.player_left, f.enemy_left
            );
            trials += 1;
        }
    }
    assert_eq!(trials, if only.is_some() { 3 } else { 39 });
}
