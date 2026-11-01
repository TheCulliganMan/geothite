fn rematch_controller_for_test(map: &str, object: &str) -> VisibleShellController {
    let path = std::env::var_os("CRYSTAL_REMATCH_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/trainer-rematches-tui.crystalpack")
        });
    let path = path.canonicalize().unwrap();
    let asset_root = AssetRoot::new(path.parent().unwrap());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded).unwrap();
    assert!(runtime.data().trainer_rematches);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "VioletGym".into(),
            tile_x: 5,
            tile_y: 11,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            quick_save_path: None,
            ..Default::default()
        },
    )
    .unwrap();
    if map == "Route30" {
        shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag("EVENT_ROUTE_30_YOUNGSTER_JOEY", false)
            .unwrap();
    }
    VisibleShellController {
        shell: prepare_map_trainer_encounter_preview_for_object(shell, map, Some(object)).unwrap(),
    }
}

#[test]
fn renderer_neutral_controller_trainer_rematches_talk_text_scaled_battle_and_save() {
    for (map, object, flag, standard_first, standard_last) in [
        (
            "VioletGym",
            "VIOLETGYM_YOUNGSTER2",
            "EVENT_BEAT_BIRD_KEEPER_ABE",
            "This is pathetic,",
            "rookie trainer",
        ),
        (
            "VioletGym",
            "VIOLETGYM_FALKNER",
            "EVENT_BEAT_FALKNER",
            "There are",
            "master!",
        ),
        (
            "Route30",
            "ROUTE30_YOUNGSTER2",
            "EVENT_BEAT_YOUNGSTER_JOEY",
            "I've been waiting!",
            "Let's battle now!",
        ),
    ] {
        let leader = object == "VIOLETGYM_FALKNER";
        let mut controller = rematch_controller_for_test(map, object);
        let trainer = controller.snapshot().unwrap().trainer;
        // Complete the first win's progression as an existing save would.
        {
            let state = controller.shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0] = None;
            state.sync_party_from_storage();
            state.flags.set_event_flag(flag, true).unwrap();
            if leader {
                state
                    .flags
                    .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
                    .unwrap();
                state.badges.johto[0] = true;
                state
                    .flags
                    .set_event_flag("EVENT_GOT_TM31_MUD_SLAP", true)
                    .unwrap();
            }
            if map == "Route30" {
                state
                    .flags
                    .set_engine_flag("ENGINE_JOEY_READY_FOR_REMATCH", true)
                    .unwrap();
            }
        }
        controller
            .shell
            .shell
            .add_party_pokemon(
                "MEW",
                20,
                None,
                None,
                &trainer.player_name,
                trainer.player_id,
                Dv::from_non_hp(9, 9, 9, 9),
            )
            .unwrap();
        let before = controller.snapshot().unwrap();
        let flags = controller.shell.shell.session().state().flags.clone();
        controller.press(GameButton::A).unwrap();
        let talking = controller.snapshot().unwrap();
        assert!(
            talking.battle.is_none(),
            "rematch must show text before starting"
        );
        assert!(
            visible_field_dialog_text(&talking, &controller.shell)
                .is_some_and(|text| text.contains(standard_first)),
            "the trainer's standard post-battle dialogue must come first"
        );
        assert!(
            !visible_field_dialog_text(&talking, &controller.shell)
                .unwrap()
                .contains("I've trained hard!")
        );
        let path = std::env::temp_dir().join(format!(
            "rematch-controller-{}-{object}.crystalsave",
            std::process::id()
        ));
        controller.save(&path).unwrap();
        let saved_dialogue = controller.shell.shell.runtime().load_save(&path).unwrap();
        assert!(
            controller
                .shell
                .shell
                .runtime()
                .data()
                .trainer_rematch_dialogue_return(&saved_dialogue)
                .is_some(),
            "the appended rematch must persist while standard dialogue is active"
        );
        controller.press(GameButton::Down).unwrap();
        assert_eq!(
            controller.snapshot().unwrap().overworld.tile,
            before.overworld.tile,
            "rematch dialogue must own movement input"
        );
        let mut saw_rematch_line = false;
        let mut saw_last_standard_page = false;
        for _ in 0..160 {
            let snapshot = controller.snapshot().unwrap();
            if let Some(text) = visible_field_dialog_text(&snapshot, &controller.shell) {
                if text.contains(standard_last) {
                    saw_last_standard_page = true;
                }
                if text.contains("I've trained hard!") {
                    assert!(
                        saw_last_standard_page,
                        "all standard pages must finish first"
                    );
                    saw_rematch_line = true;
                }
            }
            if snapshot.battle.is_some() {
                break;
            }
            controller.press(GameButton::A).unwrap();
        }
        assert!(
            saw_rematch_line,
            "the appended line must appear before battle"
        );
        let battle = controller.snapshot().unwrap();
        let battle = battle
            .battle
            .as_ref()
            .expect("talking again starts an authoritative trainer battle");
        let level = before
            .party
            .slots
            .iter()
            .map(|slot| slot.pokemon.level)
            .max()
            .unwrap();
        assert_eq!(
            battle.enemy_pokemon.level,
            if leader {
                level.saturating_sub(2)
            } else {
                level
            }
        );
        assert!(battle.enemy_pokemon.moves.iter().all(|m| m.current_pp > 0));
        assert_eq!(controller.shell.shell.session().state().flags, flags);
        controller.save(&path).unwrap();
        let saved = controller.shell.shell.runtime().load_save(&path).unwrap();
        assert_eq!(
            saved.battle,
            controller.shell.shell.session().state().battle,
            "scaled team and moves must survive a production save/load"
        );
        for _ in 0..160 {
            if controller
                .snapshot()
                .unwrap()
                .ui
                .menu
                .as_ref()
                .is_some_and(|menu| {
                    menu.layout.vertical_menus[0]
                        .options
                        .iter()
                        .any(|entry| entry == ">FIGHT")
                })
            {
                break;
            }
            controller.press(GameButton::A).unwrap();
        }
        controller.press(GameButton::A).unwrap();
        let moves = controller.snapshot().unwrap();
        let pp = moves.battle.as_ref().unwrap().player_moves[0].current_pp;
        controller.press(GameButton::A).unwrap();
        let after_turn = controller.snapshot().unwrap();
        assert!(
            after_turn.party.slots[0].pokemon.moves[0].current_pp < pp,
            "the rematch must execute moves through the real battle engine"
        );
        assert!(
            !controller.shell.battle_messages.is_empty(),
            "rematch turn must expose battle dialogue"
        );
        std::fs::remove_file(&path).unwrap();
        let _ = std::fs::remove_file(crate::core::save::save_backup_path(&path));
    }
}
