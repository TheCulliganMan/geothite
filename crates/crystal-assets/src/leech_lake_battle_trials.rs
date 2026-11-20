//! Compares optional Leech Lake fights with ordinary daytime catch histories.
use crystal_core::battle::start::TrainerBattleStartStatus;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use super::{TrialDivider, attack, badge_battle_parties, fight_active_trainer, trial_state};
use crate::read_verified_compiled_game_pack;

const ROUTE: &str = "MnR07C00R00";
// Southeast spur to the north bank and west branch. Each tile is a sight contact.
const TRAINERS: [(&str, TilePosition, Direction); 3] = [
    ("MN_R07_T05", TilePosition::new(49, 31), Direction::Right),
    ("MN_R07_T07", TilePosition::new(40, 15), Direction::Right),
    ("MN_R07_T08", TilePosition::new(15, 13), Direction::Left),
];

#[derive(Debug, PartialEq, Eq)]
struct Resources {
    hp: u32,
    max_hp: u32,
    pp: u32,
    status_count: usize,
}

fn resources(state: &GameState) -> Resources {
    let party = || state.storage.party.pokemon.iter().flatten();
    Resources {
        hp: party().map(|p| u32::from(p.hp)).sum(),
        max_hp: party().map(|p| u32::from(p.max_hp)).sum(),
        pp: party()
            .flat_map(|p| &p.moves)
            .map(|m| u32::from(m.current_pp))
            .sum(),
        status_count: party().filter(|p| p.status.is_some()).count(),
    }
}

#[test]
#[ignore = "requires MN_ROUTE7_BATTLE_PACK and runs actual first-time map trainer battles"]
fn leech_lake_route_battle_costs() {
    let path = std::env::var_os("MN_ROUTE7_BATTLE_PACK").expect("parent or candidate archive");
    let pack = read_verified_compiled_game_pack(path).expect("verified Minnesota pack");
    let data = pack.data();
    assert!(data.competitive_battles);
    let seed = std::env::var("MN_BATTLE_REVIEW_SEED")
        .ok()
        .map(|v| v.parse::<u32>().expect("decimal seed"))
        .unwrap_or(1701);
    for (starter_index, starter) in ["POUCHID", "KINDLIT", "BIRCHIT"].iter().enumerate() {
        let party = badge_battle_parties::party(data, starter_index, 6);
        // Keep Lake arrival levels and the sixth earned badge, without Bemidji training.
        let arrival = trial_state(data, 7, party);
        for mode in ["isolated", "consecutive"] {
            let mut state = arrival.clone();
            let mut divider = TrialDivider(seed);
            for (id, tile, facing) in TRAINERS {
                if mode == "isolated" {
                    state = arrival.clone();
                    divider = TrialDivider(seed);
                }
                state.overworld = OverworldMemory::Active {
                    map_name: ROUTE.into(),
                    tile,
                    facing,
                    mode: MovementMode::Normal,
                };
                let before = resources(&state);
                let source = data
                    .scripted_trainer_battle_request(ROUTE, id, 0)
                    .expect("authored map trainer");
                let start = data
                    .start_scripted_trainer_battle(&mut state, ROUTE, ROUTE, id, 0)
                    .expect("first-time route battle");
                let TrainerBattleStartStatus::Started(start) = start else {
                    panic!("{id} already defeated");
                };
                assert_eq!(start.trainer_id, id);
                assert_eq!(start.enemy_party.len(), 2);
                assert_eq!(
                    before,
                    resources(&state),
                    "encounter start restored resources"
                );
                let result = fight_active_trainer(
                    data,
                    &mut state,
                    &source,
                    &mut divider,
                    |data, _, combat| attack(data, combat),
                    Some(|_, combat| {
                        combat
                            .player_party
                            .iter()
                            .position(|p| p.hp > 0)
                            .expect("healthy reserve")
                    }),
                    false,
                );
                let after = resources(&state);
                let end = data
                    .complete_scripted_trainer_battle(
                        &mut state,
                        ROUTE,
                        ROUTE,
                        id,
                        0,
                        result.won,
                        false,
                        &mut divider,
                    )
                    .expect("normal route win/loss");
                assert_eq!(end.continued_after_battle, result.won);
                assert_eq!(
                    state
                        .flags
                        .is_event_flag_set(&source.event_flag)
                        .expect("trainer flag"),
                    result.won
                );
                if result.won {
                    assert_eq!(result.enemy_left, 0);
                    assert!(before.max_hp <= after.max_hp, "earned level-up stats");
                    assert_eq!(after, resources(&state), "victory restored resources");
                }
                println!(
                    "leech-cost,{mode},{starter},{seed},{id},{},{},{},{},{},{},{},{},{},{},{},{}",
                    result.won,
                    result.turns,
                    result.player_left,
                    result.enemy_left,
                    before.hp,
                    before.max_hp,
                    after.hp,
                    after.max_hp,
                    before.pp,
                    after.pp,
                    after.status_count,
                    end.prize_money,
                );
                if !result.won && mode == "consecutive" {
                    break;
                }
            }
        }
    }
}
