use std::{collections::BTreeSet, path::PathBuf, sync::OnceLock};

use crystal_assets::{
    CompiledGamePack, build_trainer_rematches_modpack, read_verified_compiled_game_pack,
};
use crystal_core::{
    battle::start::TrainerBattleStartStatus,
    models::Dv,
    state::GameState,
    world::{
        map::{Direction, TilePosition},
        session::{OverworldInteraction, OverworldInteractionTarget},
    },
};

fn packs() -> &'static (CompiledGamePack, CompiledGamePack) {
    static PACKS: OnceLock<(CompiledGamePack, CompiledGamePack)> = OnceLock::new();
    PACKS.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let base = read_verified_compiled_game_pack(path.canonicalize().unwrap())
            .expect("verified external pack");
        let pack = build_trainer_rematches_modpack(&base).unwrap();
        (base, pack)
    })
}

fn state() -> GameState {
    let data = packs().1.data();
    let mut state = data
        .start_overworld_session_from_spawn(
            data.runtime_spawn_point(0).unwrap(),
            &data.audio_ids().into_iter().map(str::to_owned).collect(),
        )
        .unwrap()
        .0;
    for (slot, level) in [50, 40, 30, 5].into_iter().enumerate() {
        state.storage.party.pokemon[slot] =
            Some(data.create_pokemon("MEW", level, Dv::default()).unwrap());
    }
    state.sync_party_from_storage();
    state
}

fn interaction(map: &str, script: &str) -> OverworldInteraction {
    OverworldInteraction {
        map_name: map.into(),
        script: script.into(),
        player_tile: TilePosition::new(1, 1),
        facing: Direction::Up,
        target_tile: TilePosition::new(1, 0),
        target: OverworldInteractionTarget::Object {
            object_index: 0,
            object_identifier: None,
            object_type: "OBJECTTYPE_TRAINER".into(),
        },
    }
}

#[test]
fn trainer_rematches_preserve_original_scripts_content_and_audio_and_roundtrip() {
    let (base, pack) = packs();
    assert_ne!(base.identity().unwrap(), pack.identity().unwrap());
    assert_eq!(base.compiled_audio(), pack.compiled_audio());
    assert_eq!(
        base.audio_manifest().unwrap(),
        pack.audio_manifest().unwrap()
    );
    assert_eq!(base.runtime_files(), pack.runtime_files());
    let mut original = pack.data().clone();
    original.trainer_rematches = false;
    original.maps = base.data().maps.clone();
    assert_eq!(&original, base.data());
    for (name, module) in &base.data().maps {
        let changed = &pack.data().maps[name];
        for (label, script) in &module.scripts {
            assert_eq!(&changed.scripts[label], script);
        }
        assert_eq!(module.objects, changed.objects);
        assert_eq!(module.trainer_scripts, changed.trainer_scripts);
    }
    let path = std::env::temp_dir().join(format!(
        "trainer-rematches-{}.crystalpack",
        std::process::id()
    ));
    pack.write_preserving_storage(&path).unwrap();
    let loaded = read_verified_compiled_game_pack(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded.identity().unwrap(), pack.identity().unwrap());
    assert_eq!(loaded.data(), pack.data());
    assert!(build_trainer_rematches_modpack(pack).is_err());
}

#[test]
fn trainer_rematches_cover_every_trainer_table_without_phone_registration() {
    let data = packs().1.data();
    let initial = state();
    let mut tested = BTreeSet::new();
    for (map, module) in &data.maps {
        for request in module.trainer_scripts.values() {
            let mut state = initial.clone();
            let contact = interaction(map, &request.source_script);
            assert_eq!(
                data.resolve_overworld_interaction_dispatch(&state, &contact)
                    .unwrap()
                    .unwrap()
                    .script,
                request.source_script,
                "first battle must be unchanged"
            );
            state
                .flags
                .set_event_flag(&request.event_flag, true)
                .unwrap();
            let dispatch = data
                .resolve_overworld_interaction_dispatch(&state, &contact)
                .unwrap()
                .unwrap();
            assert!(
                data.is_trainer_rematch_script(&dispatch.script),
                "{map}: {}",
                request.source_script
            );
            let conversation = module
                .scripts
                .get(&dispatch.script)
                .unwrap_or_else(|| panic!("missing conversation {map}: {}", dispatch.script))
                .as_array()
                .unwrap();
            assert_eq!(conversation[0]["command"], "scall");
            assert_eq!(conversation[0]["args"][0], request.source_script);
            assert_eq!(conversation[1]["command"], "sjump");
            let battle_script = conversation[1]["args"][0].as_str().unwrap();
            let rematch = data
                .scripted_trainer_battle_request(map, battle_script, 7)
                .unwrap();
            assert_eq!(rematch.trainer_id, request.trainer_id);
            assert!(rematch.event_flag.is_empty() && rematch.callback.is_empty());
            let TrainerBattleStartStatus::Started(start) =
                data.trainer_battle_start(&state, rematch).unwrap()
            else {
                panic!("no rematch");
            };
            let trainer = data.trainers.get(&request.trainer_id).unwrap();
            let ace = trainer.party.iter().map(|p| p.level).max().unwrap().max(40);
            assert_eq!(start.enemy_party.iter().map(|p| p.level).max(), Some(ace));
            for (mon, original) in start.enemy_party.iter().zip(&trainer.party) {
                assert_eq!(mon.species.id, original.species);
                assert!(mon.level >= original.level && mon.level <= 100);
                assert!(!mon.moves.is_empty() && mon.moves.len() <= 4);
                assert_eq!(
                    mon.moves
                        .iter()
                        .map(|m| &m.name)
                        .collect::<BTreeSet<_>>()
                        .len(),
                    mon.moves.len()
                );
                for m in &mon.moves {
                    assert_eq!(m.current_pp, data.moves[&m.name].pp);
                }
            }
            data.validate_saved_trainer_enemy_party_for_script(
                trainer,
                &start.enemy_party,
                &start.enemy_pokemon,
                &start.source_script,
            )
            .unwrap();
            tested.insert(request.trainer_id.clone());
        }
    }
    assert!(
        tested.len() > 200,
        "expected the full ordinary trainer roster, got {}",
        tested.len()
    );
}

#[test]
fn trainer_rematches_append_to_every_eligible_object_conversation() {
    let (base, pack) = packs();
    let data = pack.data();
    let mut completed = state();
    for (map, module) in &base.data().maps {
        for flag in &module.script_flag_commands {
            if flag.command == "setevent" {
                completed.flags.set_event_flag(&flag.flag_id, true).unwrap();
            } else if flag.command == "setflag" {
                completed
                    .flags
                    .set_engine_flag(&flag.flag_id, true)
                    .unwrap();
            }
        }
        for request in module.trainer_scripts.values().chain(
            module
                .scripted_trainer_battles
                .iter()
                .map(|battle| &battle.request),
        ) {
            if !request.event_flag.is_empty() {
                completed
                    .flags
                    .set_event_flag(&request.event_flag, true)
                    .unwrap();
            }
            completed.script_runtime.memory.insert(
                format!("GEOTHITE_TRAINER_WON_{map}_{}", request.trainer_id),
                "1".into(),
            );
        }
    }
    let mut tested = 0;
    for (map, module) in &base.data().maps {
        for object in &module.objects {
            let Some(dispatch) = data
                .resolve_overworld_interaction_dispatch(
                    &completed,
                    &interaction(map, &object.script),
                )
                .unwrap()
            else {
                continue;
            };
            if !data.is_trainer_rematch_script(&dispatch.script) {
                continue;
            }
            let changed = &data.maps[map];
            let wrapper = changed.scripts[&dispatch.script].as_array().unwrap();
            assert_eq!(wrapper[0]["command"], "scall");
            assert_eq!(wrapper[0]["args"][0], object.script);
            assert_eq!(wrapper[1]["command"], "sjump");
            let battle = wrapper[1]["args"][0].as_str().unwrap();
            assert!(
                changed
                    .scripted_trainer_battles
                    .iter()
                    .any(|entry| entry.source_script == battle)
            );
            for index in [0, 1] {
                assert!(
                    changed
                        .script_control_commands
                        .iter()
                        .any(|entry| entry.source_script == dispatch.script
                            && entry.command_index == index)
                );
            }
            tested += 1;
        }
    }
    assert!(
        tested > 250,
        "expected the full set of trainer conversations, got {tested}"
    );
}

#[test]
fn trainer_rematches_scale_moves_for_every_scripted_team_and_reject_forged_saves() {
    let data = packs().1.data();
    let mut state = state();
    for p in state.storage.party.pokemon.iter_mut().flatten() {
        p.level = 100;
    }
    let mut teams = BTreeSet::new();
    let mut upgraded = 0;
    for (map, module) in &data.maps {
        for battle in &module.scripted_trainer_battles {
            if !data.is_trainer_rematch_script(&battle.source_script) {
                continue;
            }
            for (index, command) in [(5, "winlosstext"), (6, "loadtrainer")] {
                assert!(
                    module
                        .script_runtime_commands
                        .iter()
                        .any(|entry| entry.source_script == battle.source_script
                            && entry.command_index == index
                            && entry.command == command)
                );
            }
            let TrainerBattleStartStatus::Started(start) = data
                .trainer_battle_start(&state, battle.request.clone())
                .unwrap()
            else {
                panic!("no battle");
            };
            let trainer = data.trainers.get(&start.trainer_id).unwrap();
            assert_eq!(start.enemy_party.iter().map(|p| p.level).max(), Some(100));
            let base = crystal_core::battle::start::materialize_trainer_party(
                trainer,
                &data.pokemon,
                &data.learnsets,
                &data.moves,
                &data.growth_rates,
            )
            .unwrap();
            for (mon, original) in start.enemy_party.iter().zip(base) {
                if mon.moves != original.moves {
                    upgraded += 1;
                }
                for m in &mon.moves {
                    assert!(
                        original.moves.iter().any(|old| old.name == m.name)
                            || data.learnsets[&mon.species.id]
                                .iter()
                                .any(|entry| entry.0 <= mon.level && entry.1 == m.name)
                    );
                }
            }
            data.validate_saved_trainer_enemy_party_for_script(
                trainer,
                &start.enemy_party,
                &start.enemy_pokemon,
                &start.source_script,
            )
            .unwrap();
            let mut forged = start.enemy_party.clone();
            forged[0].moves[0].name = "STRUGGLE".into();
            assert!(
                data.validate_saved_trainer_enemy_party_for_script(
                    trainer,
                    &forged,
                    &start.enemy_pokemon,
                    &start.source_script
                )
                .is_err()
            );
            teams.insert((map, start.trainer_id.clone()));
        }
    }
    assert!(teams.len() > 300 && upgraded > 300);
}

#[test]
fn trainer_rematches_record_only_victories_and_keep_them_in_saves() {
    let data = packs().1.data();
    let module = &data.maps["VioletGym"];
    let request = &module.trainer_scripts["TrainerBirdKeeperAbe"];
    for won in [false, true] {
        let mut state = state();
        data.start_scripted_trainer_battle(
            &mut state,
            "VioletGym",
            "VioletGym",
            &request.source_script,
            0,
        )
        .unwrap();
        if won {
            if let crystal_core::state::BattleMemory::Trainer {
                enemy_party,
                enemy_pokemon,
                ..
            } = &mut state.battle
            {
                enemy_pokemon.hp = 0;
                for (index, mon) in enemy_party.iter_mut().enumerate() {
                    mon.hp = 0;
                    state.battle_rewarded_enemy_party_indices.insert(index);
                }
            }
        }
        let mut divider = crystal_core::random::ReplayDivider::new(vec![0; 512]);
        data.complete_scripted_trainer_battle(
            &mut state,
            "VioletGym",
            "VioletGym",
            &request.source_script,
            0,
            won,
            false,
            &mut divider,
        )
        .unwrap();
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&state, &mut bytes).unwrap();
        let restored: GameState = ciborium::de::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(
            restored.script_runtime.memory.contains_key(&format!(
                "GEOTHITE_TRAINER_WON_VioletGym_{}",
                request.trainer_id
            )),
            won
        );
        let routed = data
            .resolve_overworld_interaction_dispatch(
                &restored,
                &interaction("VioletGym", &request.source_script),
            )
            .unwrap()
            .unwrap();
        assert_eq!(data.is_trainer_rematch_script(&routed.script), won);
        let mut reset = restored.clone();
        reset
            .flags
            .set_event_flag(&request.event_flag, false)
            .unwrap();
        let routed = data
            .resolve_overworld_interaction_dispatch(
                &reset,
                &interaction("VioletGym", &request.source_script),
            )
            .unwrap()
            .unwrap();
        assert!(
            !data.is_trainer_rematch_script(&routed.script),
            "a cleared defeat flag must require the original fight"
        );
    }
}

#[test]
fn trainer_rematches_preserve_elite_four_progression_on_a_new_league_run() {
    let data = packs().1.data();
    let module = &data.maps["WillsRoom"];
    let battle = module
        .scripted_trainer_battles
        .iter()
        .find(|battle| {
            battle.request.trainer_class == "WILL"
                && !data.is_trainer_rematch_script(&battle.source_script)
        })
        .unwrap();
    let flag = module
        .script_flag_commands
        .iter()
        .find(|flag| flag.command == "setevent" && flag.flag_id.starts_with("EVENT_BEAT_ELITE_"))
        .unwrap();
    let object = module
        .objects
        .iter()
        .find(|object| object.script == battle.source_script)
        .unwrap();
    let contact = interaction("WillsRoom", &object.script);
    let mut state = state();
    state.script_runtime.memory.insert(
        format!(
            "GEOTHITE_TRAINER_WON_WillsRoom_{}",
            battle.request.trainer_id
        ),
        "1".into(),
    );
    state.flags.set_event_flag(&flag.flag_id, true).unwrap();
    let route = |state: &GameState| {
        data.resolve_overworld_interaction_dispatch(state, &contact)
            .unwrap()
            .unwrap()
            .script
    };
    assert!(data.is_trainer_rematch_script(&route(&state)));
    state.flags.set_event_flag(&flag.flag_id, false).unwrap();
    assert_eq!(
        route(&state),
        contact.script,
        "a new League challenge must use its original door-opening battle"
    );
}

#[test]
fn trainer_rematches_ignore_eggs_lead_order_and_fainting_and_never_scale_down() {
    let data = packs().1.data();
    let source = format!(
        "GeothiteRematch_VioletGym_{}",
        data.maps["VioletGym"].trainer_scripts["TrainerBirdKeeperAbe"].trainer_id
    );
    let request = data
        .scripted_trainer_battle_request("VioletGym", &source, 7)
        .unwrap();
    let mut state = state();
    let start = |state: &GameState| {
        let TrainerBattleStartStatus::Started(start) =
            data.trainer_battle_start(state, request.clone()).unwrap()
        else {
            panic!("no battle");
        };
        start.enemy_pokemon.level
    };
    assert_eq!(start(&state), 40);
    state.storage.party.pokemon[0].as_mut().unwrap().hp = 0;
    state.storage.party.pokemon.swap(0, 3);
    let mut egg = data.create_pokemon("TOGEPI", 100, Dv::default()).unwrap();
    egg.is_egg = true;
    state.storage.party.pokemon[4] = Some(egg);
    assert_eq!(start(&state), 40);
    for mon in state.storage.party.pokemon.iter_mut().flatten() {
        mon.level = 1;
    }
    assert_eq!(start(&state), 9);
    for mon in state.storage.party.pokemon.iter_mut().flatten() {
        mon.level = 100;
    }
    assert_eq!(start(&state), 100);
}

#[test]
fn trainer_rematches_leave_pending_badges_and_tm_rewards_to_original_scripts() {
    let data = packs().1.data();
    let mut state = state();
    let contact = interaction("VioletGym", "VioletGymFalknerScript");
    state
        .flags
        .set_event_flag("EVENT_BEAT_FALKNER", true)
        .unwrap();
    let route = |state: &GameState| {
        data.resolve_overworld_interaction_dispatch(state, &contact)
            .unwrap()
            .unwrap()
            .script
    };
    assert_eq!(route(&state), contact.script);
    state
        .flags
        .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
        .unwrap();
    state.badges.johto[0] = true;
    assert_eq!(
        route(&state),
        contact.script,
        "bag-full TM retry must remain available"
    );
    for flag in &data.maps["VioletGym"].script_flag_commands {
        if flag.command == "setevent" && flag.flag_id.starts_with("EVENT_GOT_TM") {
            state.flags.set_event_flag(&flag.flag_id, true).unwrap();
        }
    }
    assert!(data.is_trainer_rematch_script(&route(&state)));
}
