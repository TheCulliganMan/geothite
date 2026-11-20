//! Repeat Sivren's fight after one ordinary daytime catch on the east road.
use crystal_core::battle::turn::{BattleAction, BattleSide, battle_action_locked_before_menu};
use crystal_core::models::{Dv, Pokemon};
use crystal_core::state::GameState;

use super::{attack, badge_battle_parties, challenge_with_party};
use crate::{GameDataSet, read_verified_compiled_game_pack};

fn east_road_reedraft(data: &GameDataSet, party: &[Pokemon]) -> Pokemon {
    let route = "MnR18C00R00";
    let encounters = data.wild_encounters[route]
        .grass
        .as_ref()
        .expect("grass on the east road out of Brainerd");
    let slot = &encounters.day[0];
    assert_eq!((slot.species.as_str(), slot.level), ("REEDRAFT", 29));
    let mut state = GameState::reset_wram_for_new_game();
    for member in party {
        assert!(state.storage.party.add_pokemon(member.clone()));
    }
    // A normal step with this rate roll retains the authored encounter level.
    let level = data.ordinary_challenge_level(&state, route, slot.level, 2, 0);
    assert_eq!(level, 29);
    let mut reedraft = data
        .create_pokemon(&slot.species, level, Dv::from_non_hp(8, 8, 8, 8))
        .expect("native encounter partner");
    data.equip_wild_challenge_moves(&mut reedraft, route)
        .expect("same moves equipped on a live wild encounter");
    assert!(
        reedraft
            .moves
            .iter()
            .any(|m| matches!(m.name.as_str(), "MEGA_DRAIN" | "GIGA_DRAIN")),
        "the catch actually carries a draining Grass move: {:?}",
        reedraft.moves,
    );
    reedraft
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and runs 27 full production battles"]
fn brainerd_east_road_catch_and_paid_switch_against_sivren() {
    let file = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Minnesota pack");
    let pack = read_verified_compiled_game_pack(file).expect("verified archive");
    let data = pack.data();
    let mut total_switches = 0;
    for starter in 0..3 {
        let party = badge_battle_parties::party(data, starter, 6);
        assert_eq!(party.len(), 5, "one free slot after the Route 6 catch");
        let reedraft = east_road_reedraft(data, &party);
        if starter == 0 {
            let moves: Vec<_> = reedraft.moves.iter().map(|m| m.name.as_str()).collect();
            println!("east-road catch,REEDRAFT,29,{}", moves.join(","));
        }
        for seed in [1701, 4242, 9001] {
            for policy in ["ordinary", "sixth", "save-for-bankmaw"] {
                let mut team = party.clone();
                if policy != "ordinary" {
                    team.push(reedraft.clone());
                }
                let mut switched = false;
                let mut bankmaw_seen = false;
                let mut faceoffs = 0;
                let finish =
                    challenge_with_party(data, 6, "MN_GYM_6", team, seed, |data, combat| {
                        if combat.enemy.species.id == "BANKMAW" {
                            bankmaw_seen = true;
                            if combat.player.species.id == "REEDRAFT" {
                                faceoffs += 1;
                            } else if policy == "save-for-bankmaw"
                                && !switched
                                && !battle_action_locked_before_menu(combat, BattleSide::Player)
                                && combat.player_party[5].hp > 0
                            {
                                switched = true;
                                return BattleAction::Switch { party_index: 5 };
                            }
                        }
                        attack(data, combat)
                    });
                assert_eq!(
                    if finish.won {
                        finish.enemy_left
                    } else {
                        finish.player_left
                    },
                    0
                );
                total_switches += usize::from(switched);
                println!(
                    "lake-retry,{starter},{seed},{policy},{},{},{},{},{bankmaw_seen},{switched},{faceoffs}",
                    finish.won, finish.turns, finish.player_left, finish.enemy_left
                );
            }
        }
    }
    assert!(total_switches > 0, "no paid switch occurred in any retry");
}
