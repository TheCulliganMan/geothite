//! A bounded, native Route 13 training outing before the Ely League.
use std::cell::Cell;
use std::collections::BTreeSet;

use crystal_core::battle::turn::{BattleAction, active_battle_combat_state};
use crystal_core::random::CrystalRandom;
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::world::encounters::{EncounterSurface, TimeOfDay};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;

use super::{SHOPPING, TrialDivider, attack};
use crate::{
    GameDataSet, RuntimeHeldItemCommand, RuntimeMutationCommand, RuntimePartySlotCommand,
    RuntimePcDepositCommand, RuntimePcWithdrawCommand,
};

const ROUTE: &str = "MnR13C00R02";
const CLINIC: &str = "MnElyPokecenter1F";

pub(super) fn field_battle(
    data: &GameDataSet,
    state: &mut GameState,
    grass: &OverworldSession,
    divider: &mut TrialDivider,
) -> (String, u8) {
    let mut rng = CrystalRandom::new(state.random_state, divider);
    for _ in 0..512 {
        let roll = data
            .check_wild_encounter_after_step(state, grass, &mut rng)
            .expect("Route 13 step encounter");
        if let Some(battle) = data
            .start_resolved_wild_encounter_after_step(state, grass, &roll, &mut rng)
            .expect("native encounter start")
        {
            assert_eq!(battle.encounter.surface, EncounterSurface::Grass);
            assert_eq!(battle.encounter.time, TimeOfDay::Day);
            state.random_state = rng.state();
            return (battle.enemy_pokemon.species.id, battle.enemy_pokemon.level);
        }
    }
    panic!("no Route 13 encounter in 512 grass checks");
}

fn learn(data: &GameDataSet, state: &mut GameState) {
    while let Some(offer) = state.pending_move_learn.as_ref() {
        let old: &[&str] = match offer.learned_move.name.as_str() {
            "PSYCHIC_M" => &["PSYBEAM", "PETAL_DANCE"],
            "CRUNCH" => &["FAINT_ATTACK"],
            "EARTHQUAKE" => &["MAGNITUDE"],
            "SLUDGE_BOMB" => &["SLUDGE", "RAZOR_WIND"],
            _ => &[],
        };
        let moves = &state.storage.party.pokemon[offer.party_index]
            .as_ref()
            .expect("student")
            .moves;
        if let Some(slot) = old
            .iter()
            .find_map(|name| moves.iter().position(|m| m.name == *name))
        {
            let change = data
                .replace_pending_move_learn(state, slot)
                .expect("accept level-up offer");
            println!(
                "road-move,{},{}->{}",
                state.storage.party.pokemon[0]
                    .as_ref()
                    .expect("student")
                    .species
                    .id,
                change
                    .resolution
                    .replaced_move
                    .as_deref()
                    .expect("old move"),
                change.resolution.learned_move
            );
        } else {
            data.decline_pending_move_learn(state)
                .expect("retain prepared moves");
        }
    }
}

// Positive XP means the solo partner actually survived and defeated this opponent.
// A wild escape or double knockout earns nothing.
fn fight(data: &GameDataSet, state: &mut GameState, divider: &mut TrialDivider) -> i32 {
    for _ in 0..160 {
        let combat = active_battle_combat_state(state).expect("live wild battle");
        let selected = Cell::new(0);
        let turn = data
            .resolve_active_battle_turn_with_enemy_ai_actions_with_divider(
                state,
                attack(data, &combat),
                divider,
                &mut |c, rng| {
                    let pick = data
                        .select_wild_enemy_move_for_battle(c, "BATTLETYPE_NORMAL", rng)
                        .expect("installed wild AI");
                    selected.set(pick.slot);
                    Ok(pick)
                },
                &mut |_, _| {
                    Ok(BattleAction::Move {
                        slot: selected.get(),
                    })
                },
            )
            .expect("actual wild battle turn");
        if turn.state.player.hp == 0 || matches!(state.battle, BattleMemory::Inactive) {
            return 0;
        }
        if turn.state.enemy.hp == 0 {
            let earned = data
                .claim_active_wild_battle_rewards_now(state, divider)
                .expect("award a real knockout");
            learn(data, state);
            return earned.experience_awarded;
        }
    }
    panic!("wild battle exceeded 160 turns");
}

/// Trains the six familiar partners one at a time. The other five can stay at
/// Ely's PC. Every challenger uses the same bounds and fixed route RNG seeds.
pub(super) fn train(data: &GameDataSet, state: &mut GameState) {
    let before = state.storage.party.clone();
    assert_eq!(before.filled_slots(), 6);
    assert_eq!(state.money, 0);
    let supplies = state.bag.clone();
    assert_eq!(state.storage.pc_boxes[state.current_pc_box].count, 0);
    state.time.time_of_day = TimeOfDay::Day;
    state.last_spawn_map_constant = Some("MN_ELY_POKECENTER_1F".into());
    let grass = data
        .overworld_session(ROUTE, TilePosition::new(34, 23), state.frame_counter)
        .expect("grass north of Ely");
    let mut pc = data
        .overworld_session(CLINIC, TilePosition::new(9, 2), state.frame_counter)
        .expect("Ely PC");
    let clinic = data
        .overworld_session(CLINIC, TilePosition::new(3, 3), state.frame_counter)
        .expect("Ely nurse counter");
    let audio = data
        .audio_ids()
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let apply = |state: &mut GameState, session: &mut OverworldSession, cmd| {
        data.apply_runtime_mutation_command(state, session, cmd, &audio, &audio, &audio)
            .expect("native field action");
    };
    let nurse = |state: &mut GameState| {
        data.commit_overworld_snapshot(state, &clinic, SpawnMemoryUpdate::Preserve);
        data.apply_special_routine(state, "HealParty", &audio)
            .expect("Ely nurse heals HP, status and PP");
    };

    let deposit = |index| {
        RuntimeMutationCommand::DepositPartyPokemonToCurrentBox(RuntimePcDepositCommand {
            party_index: index,
        })
    };
    let withdraw = || {
        RuntimeMutationCommand::WithdrawCurrentBoxPokemonToParty(RuntimePcWithdrawCommand {
            box_slot: 0,
        })
    };
    data.commit_overworld_snapshot(state, &pc, SpawnMemoryUpdate::Preserve);
    // Repeatedly depositing slot one leaves the first traveler in the party
    // and keeps the remaining five in their original order in the box.
    for _ in 1..6 {
        apply(state, &mut pc, deposit(1));
    }
    for (slot, member) in before.pokemon.iter().flatten().enumerate() {
        assert_eq!(state.storage.party.filled_slots(), 1);
        let current = state.storage.party.pokemon[0]
            .as_ref()
            .expect("traveling partner");
        assert_eq!(current.species.id, member.species.id);
        assert_eq!(
            (current.level, current.experience),
            (member.level, member.experience)
        );
        apply(
            state,
            &mut pc,
            RuntimeMutationCommand::TakeHeldItemFromPartyPokemon(RuntimePartySlotCommand {
                party_index: 0,
            }),
        );
        let mut divider = TrialDivider(13020 + slot as u32);
        let (mut wins, mut blackouts, mut visits, mut xp) = (0, 0, 0, 0);
        let mut opponents = Vec::new();
        for _ in 0..16 {
            let mut field = grass.clone();
            data.commit_overworld_snapshot(state, &field, SpawnMemoryUpdate::Preserve);
            let (species, level) = field_battle(data, state, &field, &mut divider);
            let earned = fight(data, state, &mut divider);
            let partner = state.storage.party.pokemon[0]
                .as_ref()
                .expect("sole traveling partner");
            if partner.hp == 0 {
                apply(
                    state,
                    &mut field,
                    RuntimeMutationCommand::CompleteBattleLoss,
                );
                apply(
                    state,
                    &mut field,
                    RuntimeMutationCommand::ResolveBlackoutToLastSpawn,
                );
                assert_eq!(field.map.name, CLINIC);
                assert_eq!(state.money, 0);
                blackouts += 1;
            } else if partner.hp * 4 < partner.max_hp * 3
                || partner.status.is_some()
                || partner.moves.iter().any(|m| m.current_pp == 0)
            {
                nurse(state);
                visits += 1;
            }
            opponents.push(format!("{species}@{level}:{earned}"));
            wins += usize::from(earned > 0);
            xp += earned;
            if wins == 12 {
                break;
            }
        }
        nurse(state);
        data.commit_overworld_snapshot(state, &pc, SpawnMemoryUpdate::Preserve);
        apply(
            state,
            &mut pc,
            RuntimeMutationCommand::GiveBagItemToPartyPokemon(RuntimeHeldItemCommand {
                item_id: "GOLD_BERRY".into(),
                party_index: 0,
            }),
        );
        let trained = state.storage.party.pokemon[0]
            .as_ref()
            .expect("return trained partner");
        assert_eq!(trained.species.id, member.species.id);
        assert_eq!(trained.item.as_deref(), Some("GOLD_BERRY"));
        assert_eq!(trained.experience - member.experience, xp);
        assert!(wins > 0);
        assert!(
            trained.hp_exp > member.hp_exp
                && trained.attack_exp > member.attack_exp
                && trained.defense_exp > member.defense_exp
                && trained.speed_exp > member.speed_exp
                && trained.special_exp > member.special_exp
        );
        assert_eq!(trained.hp, trained.max_hp);
        println!(
            "road-training,{},{},{}->{},wins={wins}/{},blackouts={blackouts},nurse_visits={},xp={xp},stat_exp={}/{}/{}/{}/{},opponents={}",
            slot + 1,
            trained.species.id,
            member.level,
            trained.level,
            opponents.len(),
            visits + 1,
            trained.hp_exp,
            trained.attack_exp,
            trained.defense_exp,
            trained.speed_exp,
            trained.special_exp,
            opponents.join("|")
        );
        // Withdraw first so the PC always leaves an able traveler. After
        // the sixth exchange the first partner is back at the head of the line.
        apply(state, &mut pc, withdraw());
        apply(state, &mut pc, deposit(0));
    }
    for _ in 1..6 {
        apply(state, &mut pc, withdraw());
    }
    assert_eq!(state.storage.pc_boxes[state.current_pc_box].count, 0);
    assert_eq!(state.storage.party.filled_slots(), 6);
    assert!(
        state.bag == supplies,
        "all original Bag contents must survive the outing"
    );
    assert_eq!(state.money, 0);
    for (item, count) in SHOPPING {
        assert_eq!(
            state.bag.quantity(&data.items[item]),
            count,
            "intact Ely receipt"
        );
    }
}
