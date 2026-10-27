//! Full-moveset rewards must finish before the battle's script cleanup.
use crystal_core::models::{Dv, LearnedMove, Pokemon};
use crystal_core::state::{BattleMemory, GameState, PendingMoveLearn};
use crystal_core::systems::evolution::EvolutionEntry;
use crystal_core::world::encounters::TimeOfDay;

use super::{species, test_move};
use crate::GameDataSet;

fn fixture() -> (GameDataSet, GameState) {
    let mut data = GameDataSet::default();
    let mon = species();
    data.pokemon.insert(mon.id.clone(), mon.clone());
    data.learnsets.insert(mon.id.clone(), Vec::new());
    data.evolutions.0.insert(mon.id.clone(), Vec::new());
    for id in ["TACKLE", "GROWL", "LEER", "SPLASH", "BITE"] {
        data.moves.insert(id.into(), test_move(id));
    }
    let mut p = Pokemon::new_for_tests(mon, 10, Dv::default());
    p.moves = ["TACKLE", "GROWL", "LEER", "SPLASH"]
        .into_iter()
        .map(|id| LearnedMove {
            name: id.into(),
            current_pp: 35,
            pp_ups: 0,
        })
        .collect();
    let mut enemy = p.clone();
    enemy.hp = 0;
    let mut state = GameState::default();
    state.storage.party.pokemon[0] = Some(p.clone());
    state.sync_party_from_storage();
    state.battle = BattleMemory::Wild {
        battle_type: "BATTLETYPE_NORMAL".into(),
        battle_music: String::new(),
        map_name: "Route29".into(),
        roaming_slot: None,
        enemy_pokemon: enemy.clone(),
        enemy_party: vec![enemy],
    };
    state.battle_active_party_index = Some(0);
    state.battle_active_enemy_party_index = Some(0);
    state.battle_evolvable_party_indices.insert(0);
    state.pending_move_learn = Some(PendingMoveLearn {
        party_index: 0,
        species_id: p.species.id,
        level: 10,
        learned_move: LearnedMove {
            name: "BITE".into(),
            current_pp: 35,
            pp_ups: 0,
        },
        defer_level_evolution: true,
    });
    (data, state)
}

#[test]
fn deferred_reward_move_replacement_or_decline_allows_no_evolution() {
    for replace in [true, false] {
        let (data, mut state) = fixture();
        let before = state.clone();
        let result = if replace {
            data.replace_pending_move_learn(&mut state, 0)
        } else {
            data.decline_pending_move_learn(&mut state)
        }
        .expect("completed battle reward check");
        assert!(
            result
                .deferred_evolution
                .expect("deferred check ran")
                .target_species
                .is_none()
        );
        assert!(state.pending_move_learn.is_none());
        assert!(state.battle_evolvable_party_indices.is_empty());
        assert_eq!(
            state.battle, before.battle,
            "reward must not end or restart the battle"
        );
        assert_eq!(state.flags, before.flags);
        assert_eq!(state.bag, before.bag);
        assert_eq!(
            state.storage.party.pokemon[0]
                .as_ref()
                .expect("partner")
                .moves[0]
                .name,
            if replace { "BITE" } else { "TACKLE" }
        );
    }
}

#[test]
fn deferred_reward_move_decision_can_evolve_only_after_last_enemy_faints() {
    for alive in [true, false] {
        let (mut data, mut state) = fixture();
        let original = state.storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .species
            .id
            .clone();
        let mut evolved = species();
        evolved.id = "NEW_MON_EVOLVED".into();
        data.pokemon.insert(evolved.id.clone(), evolved.clone());
        data.learnsets.insert(evolved.id.clone(), Vec::new());
        data.evolutions.0.insert(evolved.id.clone(), Vec::new());
        data.evolutions.0.insert(
            original.clone(),
            vec![EvolutionEntry::level(evolved.id, 10)],
        );
        let BattleMemory::Wild {
            enemy_pokemon,
            mut enemy_party,
            ..
        } = state.battle
        else {
            panic!("wild fixture");
        };
        if alive {
            let mut next = enemy_pokemon.clone();
            next.hp = 1;
            enemy_party.push(next);
        }
        state.battle = BattleMemory::Trainer {
            battle_type: "BATTLETYPE_NORMAL".into(),
            trainer_class: "YOUNGSTER".into(),
            trainer_id: "FIXTURE".into(),
            trainer_name: "FIXTURE".into(),
            event_flag: String::new(),
            seen_text: String::new(),
            win_text: String::new(),
            loss_text: String::new(),
            callback: String::new(),
            source_script: String::new(),
            enemy_pokemon,
            enemy_party,
            reward: 0,
            encounter_music: String::new(),
            ai_move_flags: 0,
            ai_item_switch_flags: 0,
            ai_layers: Vec::new(),
        };
        let result = data
            .decline_pending_move_learn(&mut state)
            .expect("move decision");
        let p = state.storage.party.pokemon[0].as_ref().expect("partner");
        if alive {
            assert!(result.deferred_evolution.is_none());
            assert_eq!(p.species.id, original);
            assert!(state.battle_evolvable_party_indices.contains(&0));
        } else {
            assert_eq!(p.species.id, "NEW_MON_EVOLVED");
            assert!(
                result
                    .deferred_evolution
                    .expect("eligible check")
                    .target_species
                    .is_some()
            );
            assert!(state.battle_evolvable_party_indices.is_empty());
        }
    }
}

#[test]
fn deferred_reward_guard_keeps_field_evolution_out_of_live_battles() {
    let (data, mut state) = fixture();
    state.pending_move_learn = None;
    if let BattleMemory::Wild {
        enemy_pokemon,
        enemy_party,
        ..
    } = &mut state.battle
    {
        enemy_pokemon.hp = 1;
        enemy_party[0].hp = 1;
    }
    let before = state.clone();
    assert!(
        data.resolve_deferred_level_evolution(&mut state, 0, TimeOfDay::Day)
            .is_err()
    );
    assert_eq!(state, before);
}
