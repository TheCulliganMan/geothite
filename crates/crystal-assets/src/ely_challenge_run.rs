//! Consecutive first-challenge League battles, using one party and a finite Ely shop receipt.
#[path = "ely_challenger_tactics.rs"]
mod ely_challenger_tactics;
#[path = "ely_earned_party.rs"]
mod ely_earned_party;
#[path = "ely_road_training.rs"]
mod ely_road_training;

use std::collections::BTreeSet;

use crystal_core::battle::damage::{DamageContext, calculate_damage, type_effectiveness};
use crystal_core::battle::start::materialize_trainer_party;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleSide, battle_action_locked_before_menu,
};
use crystal_core::models::Pokemon;
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::systems::script_flags::apply_script_flag_mutation;
use crystal_core::systems::shop::close_active_shop;
use crystal_core::world::map::TilePosition;

use super::{TrialDivider, attack, challenge_state, ely_challenger_parties, trial_state};
use crate::{
    GameDataSet, RuntimeMutationCommand, RuntimePartySwapCommand, read_verified_compiled_game_pack,
};

const ROOMS: [(&str, &str); 5] = [
    ("MnLeagueRoom1", "MN_LEAGUE_1"),
    ("MnLeagueRoom2", "MN_LEAGUE_2"),
    ("MnLeagueRoom3", "MN_LEAGUE_3"),
    ("MnLeagueRoom4", "MN_LEAGUE_4"),
    ("MnLeagueChampion", "MN_CHAMPION"),
];
const SHOPPING: [(&str, u16); 3] = [("HYPER_POTION", 6), ("REVIVE", 6), ("FULL_HEAL", 3)];
const RUN: &str = "wMnLeagueRun";
const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";

fn buy_ely_supplies(data: &GameDataSet, state: &mut GameState) {
    let mart = "MnElyMart";
    let seller = data.maps[mart]
        .script_shop_commands
        .iter()
        .find(|command| command.mart_id == "MART_MN_ELY")
        .expect("Ely shop counter");
    state.money = 18_000;
    data.open_script_shop(
        state,
        mart,
        mart,
        &seller.source_script,
        seller.command_index,
    )
    .expect("open the actual Ely stock");
    for (item, count) in SHOPPING {
        assert!(
            data.buy_shop_item(state, item, count)
                .expect("actual local transaction")
                .success
        );
        assert_eq!(state.bag.quantity(&data.items[item]), count);
    }
    assert_eq!(state.money, 0, "one 18,000 budget before entering");
    close_active_shop(state).expect("leave the Ely counter");
}

fn apply_run_counter(
    data: &GameDataSet,
    state: &mut GameState,
    map: &str,
    script: &str,
    value: usize,
) {
    let command = data.maps[map]
        .script_variable_commands
        .iter()
        .find(|c| {
            c.source_script == script && c.command == "loadmem" && c.target.as_deref() == Some(RUN)
        })
        .expect("authored League counter write");
    data.apply_script_variable_command_now(state, map, map, script, command.command_index)
        .expect("apply the League progression opcode");
    assert_eq!(state.script_runtime.memory[RUN], value.to_string());
}

fn apply_run_flag(
    data: &GameDataSet,
    state: &mut GameState,
    map: &str,
    script: &str,
    flag: &str,
    command: &str,
) {
    let grant = data.maps[map]
        .script_flag_commands
        .iter()
        .find(|c| c.source_script == script && c.command == command && c.flag_id == flag)
        .expect("authored League flag");
    apply_script_flag_mutation(state, grant.clone()).expect("native League flag effect");
}

fn register(data: &GameDataSet, state: &mut GameState) {
    let lobby = "MnLeagueLobby";
    let clerk = "MnElyRegister";
    // The fixture begins after the Boundary Intake resolution. Its installed
    // story grant and the eight earned badges must satisfy the registrar.
    apply_run_flag(
        data,
        state,
        "MnBoundaryIntake",
        "MnVerdigrimBypass",
        "EVENT_MN_SURVEY_RESTORED",
        "setevent",
    );
    for guard in data.maps[lobby]
        .script_flag_commands
        .iter()
        .filter(|c| c.source_script == clerk)
    {
        match guard.command.as_str() {
            "checkflag" => assert!(
                state
                    .flags
                    .is_engine_flag_set(&guard.flag_id)
                    .expect("registration badge")
            ),
            "checkevent" => assert!(
                state
                    .flags
                    .is_event_flag_set(&guard.flag_id)
                    .expect("registration event")
            ),
            _ => {}
        }
    }
    apply_run_counter(data, state, lobby, clerk, 0);
    apply_run_flag(data, state, lobby, clerk, ACTIVE, "setevent");
}

fn pack_string(data: &GameDataSet, state: &GameState) -> String {
    SHOPPING
        .iter()
        .map(|(item, _)| state.bag.quantity(&data.items[*item]).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn party_string(state: &GameState) -> String {
    state
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .map(|p| {
            let pp = p
                .moves
                .iter()
                .map(|m| m.current_pp.to_string())
                .collect::<Vec<_>>()
                .join(":");
            format!(
                "{}@{}[{}/{}hp;xp{};{};{};pp{}]",
                p.species.id,
                p.level,
                p.hp,
                p.max_hp,
                p.experience,
                p.status.as_deref().unwrap_or("-"),
                p.item.as_deref().unwrap_or("-"),
                pp
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn treat(data: &GameDataSet, state: &mut GameState, slot: usize, item: &str) {
    let before = state.bag.quantity(&data.items[item]);
    if before == 0 {
        return;
    }
    let (used, _) = data
        .use_bag_item_on_party_pokemon_now(state, item, slot)
        .expect("field treatment");
    assert!(used.consumed);
    assert_eq!(state.bag.quantity(&data.items[item]), before - 1);
}

fn recover_between_rooms(data: &GameDataSet, state: &mut GameState) {
    for slot in 0..6 {
        if state.storage.party.pokemon[slot]
            .as_ref()
            .expect("six challengers")
            .hp
            == 0
        {
            treat(data, state, slot, "REVIVE");
        }
        let mon = state.storage.party.pokemon[slot]
            .as_ref()
            .expect("same partner");
        if mon.hp == 0 {
            continue;
        }
        if mon.status.is_some() {
            treat(data, state, slot, "FULL_HEAL");
        }
        let mon = state.storage.party.pokemon[slot]
            .as_ref()
            .expect("same partner");
        if u32::from(mon.hp) * 4 <= u32::from(mon.max_hp) * 3 {
            treat(data, state, slot, "HYPER_POTION");
        }
    }
}

// Estimate a partner's best direct hit from its own known moves, and how well it handles
// the opponent's two public types. Never inspect the opponent's selected move or random rolls.
fn matchup(data: &GameDataSet, partner: &Pokemon, foe: &Pokemon) -> (i32, u16) {
    let mut damage = 0;
    for learned in &partner.moves {
        let m = &data.moves[&learned.name];
        if learned.current_pp == 0
            || m.power == 0
            || matches!(m.effect.as_str(), "SELFDESTRUCT" | "DREAM_EATER" | "SNORE")
        {
            continue;
        }
        let hit = calculate_damage(
            partner,
            foe,
            m,
            &data.battle_stat_multipliers,
            &data.type_categories,
            &data.type_effectiveness,
            &data.weather_modifiers,
            DamageContext {
                random_roll: 236,
                ..Default::default()
            },
        )
        .expect("ordinary public matchup")
        .damage;
        damage = damage.max((u32::from(hit) * u32::from(m.accuracy) / 100) as u16);
    }
    let mut vulnerability = 0;
    for t in [&foe.species.type1, &foe.species.type2] {
        let mut risk = 10;
        for (i, own) in [&partner.species.type1, &partner.species.type2]
            .iter()
            .enumerate()
        {
            if i == 1 && partner.species.type1 == partner.species.type2 {
                continue;
            }
            let eff = type_effectiveness(&data.type_effectiveness, t, *own).expect("pack types");
            risk = risk * i32::from(eff.numerator) / i32::from(eff.denominator);
        }
        vulnerability = vulnerability.max(risk);
    }
    let offense = i32::from(damage.min(foe.max_hp)) * 100 / i32::from(foe.max_hp);
    let health = i32::from(partner.hp) * 100 / i32::from(partner.max_hp);
    (
        offense * 20 / (8 + vulnerability)
            + health / 3
            + if partner.speed > foe.speed { 12 } else { 0 },
        damage,
    )
}

fn best_partner(data: &GameDataSet, party: &[Pokemon], foe: &Pokemon) -> usize {
    party
        .iter()
        .enumerate()
        .filter(|(_, p)| p.hp > 0)
        .max_by_key(|(slot, p)| (matchup(data, p, foe).0, std::cmp::Reverse(*slot)))
        .expect("at least one able replacement")
        .0
}

fn choose_replacement(data: &GameDataSet, combat: &BattleCombatState) -> usize {
    best_partner(data, &combat.player_party, &combat.enemy)
}

// A returning challenger may spend one turn on status or recovery against each
// known species when a direct hit cannot finish it. A missed status still costs the turn.
fn scouted_support(
    data: &GameDataSet,
    combat: &BattleCombatState,
    tried: &mut BTreeSet<String>,
) -> Option<BattleAction> {
    let foe = &combat.enemy;
    let mon = &combat.player;
    if tried.contains(&foe.species.id)
        || battle_action_locked_before_menu(combat, BattleSide::Player)
        || matchup(data, mon, foe).1 >= foe.hp
    {
        return None;
    }
    let grass = foe.species.type1 == "GRASS" || foe.species.type2 == "GRASS";
    for slot in super::competitive_move_slots(combat, BattleSide::Player) {
        let Some(m) = mon.moves.get(slot) else {
            continue;
        };
        let useful = match m.name.as_str() {
            "STUN_SPORE" | "SLEEP_POWDER" | "SPORE" => {
                !grass && foe.status.is_none() && combat.enemy_safeguard_turns == 0
            }
            "LEECH_SEED" => {
                !grass && combat.enemy_leech_seed_source.is_none() && foe.hp > foe.max_hp / 2
            }
            "SYNTHESIS" | "RECOVER" => mon.hp <= mon.max_hp / 2 && mon.speed > foe.speed,
            _ => false,
        };
        if useful {
            tried.insert(foe.species.id.clone());
            return Some(BattleAction::Move { slot });
        }
    }
    None
}

fn prepared_action(
    data: &GameDataSet,
    combat: &BattleCombatState,
    switched_on: &mut BTreeSet<String>,
) -> BattleAction {
    if switched_on.len() < 3
        && !switched_on.contains(&combat.enemy.species.id)
        && !battle_action_locked_before_menu(combat, BattleSide::Player)
        && combat.player_escape_trap.is_none()
        && combat.player_trap.is_none()
    {
        let slot = choose_replacement(data, combat);
        let (current, hit) = matchup(data, &combat.player, &combat.enemy);
        if slot != combat.player_party_index
            && hit < combat.enemy.hp
            && matchup(data, &combat.player_party[slot], &combat.enemy).0 > current + 45
        {
            switched_on.insert(combat.enemy.species.id.clone());
            return BattleAction::Switch { party_index: slot };
        }
    }
    attack(data, combat)
}

fn scouted_lead(data: &GameDataSet, state: &mut GameState, room: &str, id: &str, tactical: bool) {
    let team = materialize_trainer_party(
        &data.trainers.trainers[id],
        &data.pokemon,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
    )
    .expect("previously seen opening partner");
    let party = state
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let slot = if tactical {
        ely_challenger_tactics::lead(data, &party, &team[0])
    } else {
        best_partner(data, &party, &team[0])
    };
    if slot > 0 {
        let mut session = data
            .overworld_session(room, TilePosition::new(4, 4), 0)
            .expect("League room");
        let ids = BTreeSet::new();
        data.apply_runtime_mutation_command(
            state,
            &mut session,
            RuntimeMutationCommand::SwapPartyPokemon(RuntimePartySwapCommand {
                first_party_index: 0,
                second_party_index: slot,
            }),
            &ids,
            &ids,
            &ids,
        )
        .expect("field party reorder");
    }
}

fn trace_casey(
    starter: usize,
    seed: u32,
    turn: usize,
    combat: &BattleCombatState,
    action: &BattleAction,
) {
    let decision = match action {
        BattleAction::Move { slot } => combat
            .player
            .moves
            .get(*slot)
            .map_or("STRUGGLE", |m| m.name.as_str())
            .to_string(),
        BattleAction::Switch { party_index } => {
            format!("SWITCH {}", combat.player_party[*party_index].species.id)
        }
        other => format!("{other:?}"),
    };
    println!(
        "ely-casey-turn,{starter},{seed},{turn},{},{}/{},{},{}/{},{},{}",
        combat.player.species.id,
        combat.player.hp,
        combat.player.max_hp,
        combat.enemy.species.id,
        combat.enemy.hp,
        combat.enemy.max_hp,
        decision,
        combat.enemy_last_move.as_deref().unwrap_or("-")
    );
}

fn fight(
    data: &GameDataSet,
    state: &mut GameState,
    divider: &mut TrialDivider,
    starter: usize,
    seed: u32,
    mode: &str,
    index: usize,
) -> bool {
    let (map, trainer) = ROOMS[index];
    let tactical = mode.starts_with("tactical");
    let scouted = tactical || matches!(mode, "scouted" | "trained");
    if scouted {
        scouted_lead(data, state, map, trainer, tactical);
    }
    println!(
        "ely-enter,{mode},{starter},{seed},{trainer},{},{}",
        pack_string(data, state),
        party_string(state)
    );
    let mut switched_on = BTreeSet::new();
    let mut support = BTreeSet::new();
    let trace = (mode == "isolated" || tactical)
        && trainer == "MN_CHAMPION"
        && std::env::var("MN_ELY_CASEY_TRACE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            == Some(seed);
    let mut turn = 0;
    let mut tactics = ely_challenger_tactics::Tactics::default();
    let finish = challenge_state(
        data,
        state,
        trainer,
        divider,
        |data, current, combat| {
            let choice = if tactical {
                tactics.choose(
                    data,
                    current,
                    combat,
                    mode != "tactical-moves",
                    mode.contains("triage"),
                )
            } else {
                let choice = prepared_action(data, combat, &mut switched_on);
                if scouted && matches!(choice, BattleAction::Move { .. }) {
                    scouted_support(data, combat, &mut support).unwrap_or(choice)
                } else {
                    choice
                }
            };
            turn += 1;
            if trace && turn <= 20 {
                trace_casey(starter, seed, turn, combat, &choice);
            }
            choice
        },
        Some(if tactical {
            ely_challenger_tactics::best
        } else {
            choose_replacement
        }),
        tactical,
    );
    let end_party = party_string(state);
    if !support.is_empty() {
        println!("ely-support,{mode},{starter},{seed},{trainer},{support:?}");
    }
    println!(
        "ely-fight,{mode},{starter},{seed},{trainer},{},{},{},{},{},{},{}",
        finish.won,
        finish.turns,
        finish.player_left,
        finish.enemy_left,
        switched_on.len(),
        pack_string(data, state),
        end_party
    );
    if !finish.won {
        return false;
    }
    let entry = data.maps[map]
        .scripted_trainer_battles
        .iter()
        .find(|b| b.request.trainer_id == trainer)
        .expect("installed League fight");
    let money = state.money;
    let result = data
        .complete_scripted_trainer_battle(
            state,
            map,
            map,
            &entry.source_script,
            entry.startbattle_command_index,
            true,
            false,
            divider,
        )
        .expect("finish victory and award prize");
    assert!(result.continued_after_battle && result.prize_money > 0);
    assert_eq!(state.money, money + result.prize_money);
    assert!(
        matches!(state.battle, BattleMemory::Inactive)
            && state.script_runtime.active_battle_combat.is_none()
    );
    assert_eq!(
        end_party.replace(";BAD_POISON;", ";POISON;"),
        party_string(state),
        "terminal cleanup must preserve earned party resources"
    );
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .all(|p| p.stat_boosts == crystal_core::models::pokemon::default_stat_boosts()),
        "between rooms all temporary stat changes clear"
    );
    if !matches!(
        mode,
        "isolated" | "tactical-fresh" | "tactical-moves" | "tactical-triage-fresh"
    ) {
        apply_run_counter(data, state, map, &entry.source_script, index + 1);
    }
    println!(
        "ely-prize,{mode},{starter},{seed},{trainer},{},{}",
        result.prize_money, state.money
    );
    true
}

fn run_prepared(mode: &str) {
    run_cases(mode, &[0, 1, 2], &[1701, 4242, 9001]);
}

fn run_cases(mode: &str, starters: &[usize], seeds: &[u32]) {
    let file = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Minnesota pack");
    let pack = read_verified_compiled_game_pack(file).expect("verified pack");
    let data = pack.data();
    let earned = mode == "tactical-triage-earned";
    for &starter in starters {
        if earned && starter != 1 {
            continue; // The archived starter is Kindlit.
        }
        let mut departure = if earned {
            ely_earned_party::departure(data)
        } else {
            trial_state(data, 9, ely_challenger_parties::party(data, starter))
        };
        if mode != "isolated" {
            if !earned {
                buy_ely_supplies(data, &mut departure);
            }
            if mode == "trained" {
                // Pay before going out so a solo training loss never risks the
                // budget reserved for these items. No bag supplies go on the outing.
                ely_road_training::train(data, &mut departure);
                assert_eq!(pack_string(data, &departure), "6/6/3");
                assert_eq!(departure.money, 0);
            }
            register(data, &mut departure);
        }
        for &seed in seeds {
            if matches!(
                mode,
                "isolated" | "tactical-fresh" | "tactical-moves" | "tactical-triage-fresh"
            ) {
                let first = if mode == "isolated" { 3 } else { 4 };
                for index in first..5 {
                    fight(
                        data,
                        &mut departure.clone(),
                        &mut TrialDivider(seed),
                        starter,
                        seed,
                        mode,
                        index,
                    );
                }
                continue;
            }
            let mut state = departure.clone();
            let mut divider = TrialDivider(seed);
            let mut wins = 0;
            for index in 0..5 {
                assert!(
                    state
                        .flags
                        .is_event_flag_set(ACTIVE)
                        .expect("registered League run")
                );
                assert_eq!(state.script_runtime.memory[RUN], index.to_string());
                if !fight(data, &mut state, &mut divider, starter, seed, mode, index) {
                    break;
                }
                wins += 1;
                if wins < 5 {
                    if mode.starts_with("tactical") {
                        ely_challenger_tactics::regroup(data, &mut state, wins == 4);
                    } else {
                        recover_between_rooms(data, &mut state);
                    }
                }
            }
            println!(
                "ely-finish,{mode},{starter},{seed},{wins},{},{}",
                pack_string(data, &state),
                state.money
            );
            if wins == 5 {
                let hall = "MnLeagueHallOfFame";
                let record = "MnElyRecordVictory";
                apply_run_flag(
                    data,
                    &mut state,
                    hall,
                    record,
                    "EVENT_MN_LEAGUE_CHAMPION",
                    "setevent",
                );
                apply_run_flag(data, &mut state, hall, record, ACTIVE, "clearevent");
                apply_run_counter(data, &mut state, hall, record, 0);
                assert!(
                    state
                        .flags
                        .is_event_flag_set("EVENT_MN_LEAGUE_CHAMPION")
                        .expect("earned Champion")
                );
            }
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine League sequences"]
fn ely_prepared_challengers_carry_party_and_bag_through_league() {
    run_prepared("prepared");
}
#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine scouted return sequences"]
fn ely_scouted_challengers_choose_their_lead_at_each_door() {
    run_prepared("scouted");
}
#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves 18 fresh late League battles"]
fn ely_prepared_challengers_face_halden_and_casey_at_full_strength() {
    run_prepared("isolated");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and runs Route 13 preparation and nine League sequences"]
fn ely_scouted_parties_return_after_route_thirteen_training() {
    run_prepared("trained");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine fresh Champion battles"]
fn ely_tactical_challengers_face_casey_with_the_bag() {
    run_prepared("tactical-fresh");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine fresh Champion battles"]
fn ely_tactical_challengers_face_casey_without_opening_the_bag() {
    run_prepared("tactical-moves");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine five-room attempts"]
fn ely_tactical_challengers_spend_one_receipt_across_the_league() {
    run_prepared("tactical");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine fresh Champion battles"]
fn ely_triage_challengers_restore_a_reserve_before_caseys_next_attack() {
    run_prepared("tactical-triage-fresh");
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine five-room attempts"]
fn ely_triage_challengers_spend_one_receipt_across_the_league() {
    run_prepared("tactical-triage");
}

#[test]
#[ignore = "requires a current Minnesota League archive and the original earned V16 archive and save"]
fn ely_history_five_earned_partners_and_a_new_catch_face_the_circuit() {
    run_prepared("tactical-triage-earned");
}

#[test]
#[ignore = "requires a current Minnesota League archive"]
fn ely_casey_final_partner_receives_only_an_actual_shift_offer() {
    run_cases("tactical-triage-fresh", &[2], &[1701]);
}
