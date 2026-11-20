//! Reconstructs five individually earned partner histories in a separate, current-pack trial.
use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;

use crystal_core::battle::start::switch_active_battle_party_index;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleSide, active_battle_combat_state, preview_battle_move,
};
use crystal_core::models::{LearnedMove, Pokemon};
use crystal_core::random::BattleRandomSource;
use crystal_core::save::{SaveGame, SaveModpackIdentity, read_save_game_bytes_for_modpack};
use crystal_core::state::GameState;
use crystal_core::systems::experience::calculate_experience;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::shop::close_active_shop;
use crystal_core::world::encounters::TimeOfDay;
use crystal_core::world::map::TilePosition;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{TrialDivider, ely_road_training, trial_state};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePackIdentity, GameDataSet};

const PACK_SHA: &str = "b1bb11e4de044800bcb71d76ded8b286ed9cbc502dcf156fb223001386b7598b";
const SAVE_SHA: &str = "5a1f42f637b53adaf1a71c6be5bb1456c49452bcc002a2c3025d9523d6c9eca5";
const CONTENT_HASH: &str = "c6feb99daa624a3772c5e76cfc8878b0760d4ce9b498b47eacf145b5156f1d22";

fn read_archive() -> SaveGame {
    #[derive(Deserialize)]
    struct Receipt {
        identity: CompiledGamePackIdentity,
    }
    let source = fs::read(std::env::var_os("MN_ELY_EARNED_PACK").expect("retained V16 archive"))
        .expect("original earned campaign archive");
    assert_eq!(format!("{:x}", Sha256::digest(&source)), PACK_SHA);
    // Pin the entire historical archive before extracting its own CBOR identity.
    // Current gameplay data are always loaded from the separate current archive.
    assert_eq!(&source[..12], b"CRYSTALPACK\0");
    let receipt: Receipt = ciborium::from_reader(&source[22..]).expect("original archive identity");
    assert_eq!(receipt.identity.content_hash, CONTENT_HASH);
    let identity =
        SaveModpackIdentity::from_compiled_pack_bytes(receipt.identity.runtime_modpack_id, &source)
            .expect("original native pack identity");
    let path = std::env::var_os("MN_ELY_EARNED_SAVE").expect("retained first registration save");
    let bytes = fs::read(path).expect("read original registration");
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), SAVE_SHA);
    read_save_game_bytes_for_modpack(
        &bytes,
        "V16 first Ely registration",
        &identity,
        CONTENT_HASH,
    )
    .expect("decode solely against the original pack")
}

fn rebuild(data: &GameDataSet, old: &Pokemon) -> Pokemon {
    let id = &old.species.id;
    let mut partner = data
        .create_pokemon(id, old.level, old.dvs)
        .expect("current species");
    let experience_at = |level| {
        calculate_experience(&data.growth_rates, &partner.species.growth_rate, level)
            .expect("current experience curve")
    };
    assert!((experience_at(old.level)..experience_at(old.level + 1)).contains(&old.experience));
    partner.original_trainer_id = old.original_trainer_id;
    partner
        .original_trainer_name
        .clone_from(&old.original_trainer_name);
    partner.happiness = old.happiness;
    partner.experience = old.experience;
    partner.hp_exp = old.hp_exp;
    partner.attack_exp = old.attack_exp;
    partner.defense_exp = old.defense_exp;
    partner.speed_exp = old.speed_exp;
    partner.special_exp = old.special_exp;
    // A partner can earn stat experience between level-ups. Its stored
    // stats may trail the freshly calculated ceiling until a later refresh.
    assert_eq!(
        partner.species.base_stats, old.species.base_stats,
        "{id} changed base stats"
    );
    partner.max_hp = old.max_hp;
    partner.hp = old.hp;
    partner.attack = old.attack;
    partner.defense = old.defense;
    partner.speed = old.speed;
    partner.special_attack = old.special_attack;
    partner.special_defense = old.special_defense;
    partner.status.clone_from(&old.status);
    partner.moves = old
        .moves
        .iter()
        .map(|mv| {
            assert!(
                data.learnsets[id]
                    .iter()
                    .any(|entry| entry.0 <= old.level && entry.1 == mv.name)
                    || partner.species.tmhm_learnset.contains(&mv.name),
                "ineligible historical move {id}/{}",
                mv.name
            );
            assert_eq!(mv.pp_ups, 0);
            LearnedMove {
                name: mv.name.clone(),
                current_pp: mv.current_pp,
                pp_ups: 0,
            }
        })
        .collect();
    assert!(old.item.is_none());
    partner
        .validate_saved_state()
        .expect("legal current partner, including stored battle stats");
    partner
}

pub(super) fn departure(data: &GameDataSet) -> GameState {
    let save = read_archive();
    let original = save.state();
    assert_eq!(original.badges.johto, [true; 8]);
    assert_eq!(original.hall_of_fame.count, 0);
    assert_eq!(original.script_runtime.memory["wMnLeagueRun"], "0");
    assert!(original.storage.pc_boxes.iter().all(|b| b.count == 0));
    let eligible = RegionalRoster::current()
        .expect("current Minnesota roster")
        .allowed;
    assert!(!eligible.contains("MUDMENDER"));
    let members = original
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .filter(|p| eligible.contains(&p.species.id))
        .map(|p| rebuild(data, p))
        .collect::<Vec<_>>();
    assert_eq!(members.len(), 5);
    let mut state = trial_state(data, 9, members);
    state.money = original.money;
    assert_eq!(state.money, 17_337);
    for (item, expected) in [
        ("HYPER_POTION", 6),
        ("REVIVE", 1),
        ("FULL_HEAL", 2),
        ("AWAKENING", 1),
        ("ULTRA_BALL", 1),
    ] {
        let item = &data.items[item];
        let count = original.bag.quantity(item);
        assert_eq!(count, expected);
        assert!(
            state
                .bag
                .add_item(item, count)
                .expect("historically owned supply")
        );
    }
    let map = "MnElyMart";
    let counter = data.maps[map]
        .script_shop_commands
        .iter()
        .find(|c| c.mart_id == "MART_MN_ELY")
        .expect("current Ely mart");
    data.open_script_shop(
        &mut state,
        map,
        map,
        &counter.source_script,
        counter.command_index,
    )
    .expect("Ely counter");
    for (item, n) in [
        ("ULTRA_BALL", 5),
        ("REVIVE", 5),
        ("HYPER_POTION", 2),
        ("FULL_HEAL", 1),
    ] {
        assert!(
            data.buy_shop_item(&mut state, item, n)
                .expect("ordinary purchase")
                .success
        );
    }
    close_active_shop(&mut state).expect("leave shop");
    assert_eq!(state.money, 837);
    catch_road_partner(data, &mut state);
    let clinic = data
        .overworld_session(
            "MnElyPokecenter1F",
            TilePosition::new(3, 3),
            state.frame_counter,
        )
        .expect("Ely clinic");
    data.commit_overworld_snapshot(&mut state, &clinic, SpawnMemoryUpdate::Preserve);
    data.apply_special_routine(
        &mut state,
        "HealParty",
        &data
            .audio_ids()
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>(),
    )
    .expect("clinic heals before registration");
    state
}

fn catch_road_partner(data: &GameDataSet, state: &mut GameState) {
    let mut divider = TrialDivider(13020); // Same first Route 13 stream as the training comparison.
    state.time.time_of_day = TimeOfDay::Day;
    let grass = data
        .overworld_session(
            "MnR13C00R02",
            TilePosition::new(34, 23),
            state.frame_counter,
        )
        .expect("Route 13 grass");
    data.commit_overworld_snapshot(state, &grass, SpawnMemoryUpdate::Preserve);
    let (species, level) = ely_road_training::field_battle(data, state, &grass, &mut divider);
    assert!(
        RegionalRoster::current()
            .expect("current roster")
            .allowed
            .contains(&species)
    );
    for turn in 1..=16 {
        let combat = active_battle_combat_state(state).expect("fresh Route 13 opponent");
        let selected = Cell::new(0);
        let mut select = |combat: &BattleCombatState, rng: &mut dyn BattleRandomSource| {
            let choice = data
                .select_wild_enemy_move_for_battle(combat, "BATTLETYPE_NORMAL", rng)
                .expect("normal Route 13 wild AI");
            selected.set(choice.slot);
            Ok(choice)
        };
        let mut action = |_: &BattleCombatState, _: &mut dyn BattleRandomSource| {
            Ok(BattleAction::Move {
                slot: selected.get(),
            })
        };
        // Try to reach red HP. Leave room for a high roll and use single-hit
        // attacks with no poison, recoil or other lingering damage.
        let soften = combat
            .player
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.current_pp > 0
                    && matches!(
                        m.name.as_str(),
                        "GUST"
                            | "WING_ATTACK"
                            | "MUD_SLAP"
                            | "QUICK_ATTACK"
                            | "BITE"
                            | "SHADOW_BALL"
                            | "HEADBUTT"
                    )
            })
            .filter_map(|(slot, m)| {
                let preview = preview_battle_move(
                    &combat,
                    BattleSide::Player,
                    &data.moves[&m.name],
                    &data.items,
                    &data.move_priorities,
                    &data.battle_stat_multipliers,
                    &data.type_categories,
                    &data.type_effectiveness,
                    &data.weather_modifiers,
                )
                .expect("estimate chip damage");
                let [_, high] = preview.damage_on_hit?;
                (high > 0 && high * 2 < combat.enemy.hp).then_some((high, slot))
            })
            .max();
        let result = if combat.enemy.hp * 4 > combat.enemy.max_hp
            && let Some((_, slot)) = soften
        {
            data.resolve_active_battle_turn_with_enemy_ai_actions_with_divider(
                state,
                BattleAction::Move { slot },
                &mut divider,
                &mut select,
                &mut action,
            )
            .expect("native weakening turn")
        } else {
            let (capture, battle) = data
                .resolve_active_battle_ball_turn_with_enemy_ai_actions_with_divider(
                    state,
                    "ULTRA_BALL",
                    BattleAction::Move { slot: 0 },
                    &mut divider,
                    Some((&mut select, &mut action)),
                )
                .expect("actual paid Ultra Ball and wild response");
            if capture.caught {
                let outcome = data
                    .complete_active_wild_capture(state, &capture, None, &mut divider)
                    .expect("store wild partner and award native experience");
                assert!(outcome.stored.is_some());
                while state.pending_move_learn.is_some() {
                    data.decline_pending_move_learn(state)
                        .expect("retain earned moves after catching");
                }
                assert_eq!(state.storage.party.filled_slots(), 6);
                println!(
                    "ely-history-catch,{species},{level},{turn},{} balls left",
                    state.bag.quantity(&data.items["ULTRA_BALL"])
                );
                return;
            }
            assert!(
                state.bag.quantity(&data.items["ULTRA_BALL"]) > 0,
                "all six paid balls broke"
            );
            battle
        };
        assert!(result.state.enemy.hp > 0, "wild partner fainted");
        if result.state.player.hp == 0 {
            let slot = state
                .storage
                .party
                .pokemon
                .iter()
                .position(|p| p.as_ref().is_some_and(|p| p.hp > 0))
                .expect("healthy reserve");
            switch_active_battle_party_index(state, slot, &data.items)
                .expect("replace fainted partner");
        }
    }
    panic!("no Route 13 catch within available turns");
}
