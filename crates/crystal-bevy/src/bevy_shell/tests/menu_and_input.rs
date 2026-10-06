#[test]
fn visible_start_menu_responds_to_normal_button_inputs() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before opening the start menu");

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Start])
        .expect("Start opens the start menu through normal input dispatch");
    assert_eq!(
        visible_start_menu_entries(&runtime_shell).expect("start menu entries"),
        vec![">PACK", " AB", " SAVE", " OPTION", " EXIT"]
    );

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Down])
        .expect("Down moves the start menu cursor through normal input dispatch");
    assert_eq!(
        visible_start_menu_entries(&runtime_shell).expect("moved start menu entries"),
        vec![" PACK", ">AB", " SAVE", " OPTION", " EXIT"]
    );

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A selects the highlighted start menu option through normal input dispatch");
    assert!(runtime_shell.start_menu_cursor.is_none());
    assert!(runtime_shell.trainer_card_open);
    assert_eq!(
        runtime_shell.last_action_status.as_deref(),
        Some("TRAINER CARD")
    );

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::B])
        .expect("B closes the opened start menu panel through normal input dispatch");
    assert!(!runtime_shell.trainer_card_open);
}

#[test]
fn visible_pokegear_phone_call_rings_twice_before_entering_the_compiled_asm_callback() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before calling a contact");
    runtime_shell
        .shell
        .initialize_permanent_phone_numbers()
        .expect("initialize permanent phone contacts");
    let snapshot = runtime_shell.shell.snapshot().expect("phone snapshot");
    let contacts = &snapshot.script_events.phone_numbers;
    assert!(
        !contacts.is_empty(),
        "new game must initialize permanent contacts"
    );
    runtime_shell.pokegear_menu_open = true;
    runtime_shell.pokegear_page = PokegearPage::Phone;
    runtime_shell.pokegear_phone_cursor = 0;
    let call_sounds_before = runtime_shell
        .last_audio_events
        .iter()
        .filter(|event| event.contains("SFX_CALL"))
        .count();

    start_visible_pokegear_phone_call(&mut runtime_shell).expect("start outgoing phone call");

    assert!(!runtime_shell.pokegear_menu_open);
    assert_eq!(
        runtime_shell
            .last_audio_events
            .iter()
            .filter(|event| event.contains("SFX_CALL"))
            .count(),
        call_sounds_before + 1,
        "PokegearPhone_MakePhoneCall must begin its first source ring"
    );
    assert!(
        !runtime_shell
            .shell
            .snapshot()
            .expect("phone state during first ring")
            .script_events
            .memory
            .contains_key("wPhoneScriptBank")
    );

    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    advance_visible_pokegear_phone_call(&mut runtime_shell, 1)
        .expect("advance to second outgoing ring");
    assert_eq!(
        runtime_shell
            .last_audio_events
            .iter()
            .filter(|event| event.contains("SFX_CALL"))
            .count(),
        call_sounds_before + 2,
        "PokegearPhone_MakePhoneCall must play exactly two source rings"
    );

    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    advance_visible_pokegear_phone_call(&mut runtime_shell, 1)
        .expect("enter compiled outgoing phone callback");
    assert!(
        runtime_shell
            .last_runtime_action
            .as_ref()
            .is_some_and(|record| record.action.contains("LoadPhoneScriptBank"))
    );
    assert!(
        runtime_shell
            .shell
            .snapshot()
            .expect("phone state after both rings")
            .script_events
            .memory
            .contains_key("wPhoneScriptBank")
    );
}

#[test]
fn visible_pokegear_phone_call_without_service_stays_in_the_contact_menu() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before calling a contact");
    runtime_shell
        .shell
        .initialize_permanent_phone_numbers()
        .expect("initialize permanent phone contacts");
    let cave_tile = TilePosition::new(4, 4);
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "WhirlIslandCave".to_string(),
        tile: cave_tile,
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("WhirlIslandCave", cave_tile, 0)
        .expect("start no-service cave session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    runtime_shell.pokegear_menu_open = true;
    runtime_shell.pokegear_page = PokegearPage::Phone;
    runtime_shell.pokegear_phone_cursor = 0;

    start_visible_pokegear_phone_call(&mut runtime_shell).expect("reject no-service phone call");

    assert!(runtime_shell.pokegear_menu_open);
    assert_eq!(
        runtime_shell
            .pokegear_phone_call
            .as_ref()
            .map(|call| call.phase),
        Some(VisiblePokegearPhoneCallPhase::NoServicePrompt)
    );
    assert_eq!(
        runtime_shell.pokegear_phone_status.as_deref(),
        Some("OUT OF SERVICE")
    );
    assert!(
        runtime_shell
            .last_audio_events
            .iter()
            .any(|event| event.contains("SFX_NO_SIGNAL"))
    );
    assert!(
        !runtime_shell
            .last_audio_events
            .iter()
            .any(|event| event.contains("SFX_CALL"))
    );
    assert!(
        !runtime_shell
            .shell
            .snapshot()
            .expect("no-service phone state")
            .script_events
            .memory
            .contains_key("wPhoneScriptBank")
    );
    press_visible_b_button(&mut runtime_shell).expect("dismiss no-service prompt");
    assert!(runtime_shell.pokegear_phone_call.is_none());
    assert!(runtime_shell.pokegear_menu_open);
    assert!(runtime_shell.pokegear_phone_status.is_none());
}

#[test]
fn visible_pokegear_phone_call_waits_ten_frames_then_hangs_up_on_a() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pokegear_phone_call = Some(VisiblePokegearPhoneCall {
        contact_id: "CONTACT_MOM".to_string(),
        phase: VisiblePokegearPhoneCallPhase::FinishDelay {
            frames_remaining: 10,
        },
    });
    runtime_shell.pokegear_menu_open = false;

    for expected_remaining in (1..10).rev() {
        assert!(
            advance_visible_pokegear_phone_call(&mut runtime_shell, 1)
                .expect("advance phone finish delay")
        );
        assert_eq!(
            runtime_shell
                .pokegear_phone_call
                .as_ref()
                .map(|call| call.phase),
            Some(VisiblePokegearPhoneCallPhase::FinishDelay {
                frames_remaining: expected_remaining,
            })
        );
    }
    assert!(
        advance_visible_pokegear_phone_call(&mut runtime_shell, 1)
            .expect("reach phone hangup input")
    );
    assert_eq!(
        runtime_shell
            .pokegear_phone_call
            .as_ref()
            .map(|call| call.phase),
        Some(VisiblePokegearPhoneCallPhase::AwaitHangup)
    );

    // PrintText leaves the final call page on the LCD until HangUp replaces
    // it with PokegearAskWhoCallText. It must not survive as a field dialog.
    let script = &mut runtime_shell.shell.session_mut().state_mut().script_runtime;
    script.text_window_open = true;
    script.active_text_label = Some("MomPhoneNoPokemonText".into());
    press_visible_a_button(&mut runtime_shell).expect("hang up outgoing call");

    assert!(!runtime_shell.shell.snapshot().unwrap().ui.text_window_open,
        "hangup must retire the call text before restoring the Phone prompt");
    let snapshot = runtime_shell.shell.snapshot().unwrap();
    // SetUpTextbox clears for four frames. Each 20-frame hold also calls
    // WaitBGMap (four frames), including the final contact-prompt upload.
    let mut hangup_frames = Vec::new();
    assert!(!runtime_shell.pending_audio.iter().any(|audio| audio.audio_id == "SFX_HANG_UP"),
        "HangUp_Beep must wait for the initial four-frame textbox upload");
    let stages = [("", 4), ("Click!", 24), ("", 4), ("……", 24),
        ("", 28), ("……", 24), ("", 28), ("……", 24), ("", 28)];
    for (stage, (expected, frames)) in stages.iter().enumerate() {
        assert!(runtime_shell.pokegear_phone_call.is_some(), "hangup stage {stage} must retain the card");
        assert!(!runtime_shell.pokegear_menu_open, "hangup must not accept menu input");
        let phase = runtime_shell.pokegear_phone_call.as_ref().unwrap().phase;
        press_visible_a_button(&mut runtime_shell).unwrap();
        press_visible_b_button(&mut runtime_shell).unwrap();
        assert_eq!(runtime_shell.pokegear_phone_call.as_ref().unwrap().phase, phase,
            "A/B cannot skip a hangup stage");
        assert_eq!(visible_pokegear_phone_prompt(&snapshot, &runtime_shell).unwrap(), *expected);
        for _ in 0..frames - 1 {
            hangup_frames.push(visible_pokegear_phone_prompt(&snapshot, &runtime_shell).unwrap());
            advance_visible_pokegear_phone_call(&mut runtime_shell, 1).unwrap();
        }
        assert_eq!(visible_pokegear_phone_prompt(&snapshot, &runtime_shell).unwrap(), *expected);
        hangup_frames.push(visible_pokegear_phone_prompt(&snapshot, &runtime_shell).unwrap());
        advance_visible_pokegear_phone_call(&mut runtime_shell, 1).unwrap();
    }
    hangup_frames.push(visible_pokegear_phone_prompt(&snapshot, &runtime_shell).unwrap());
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(PathBuf::from(directory).join("phone-hangup-runtime-frames.json"),
            serde_json::to_vec_pretty(&hangup_frames).unwrap()).unwrap();
    }
    assert!(runtime_shell.pokegear_phone_call.is_none());
    assert!(runtime_shell.pokegear_menu_open);
    assert_eq!(runtime_shell.pokegear_page, PokegearPage::Phone);
    assert!(
        runtime_shell
            .last_audio_events
            .iter()
            .any(|event| event.contains("SFX_HANG_UP"))
    );
}

#[test]
fn visible_incoming_phone_call_preserves_source_ring_and_hangup_timing() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().script_runtime.variables
        .insert("VAR_CALLERID".to_string(), "PHONE_ELM".to_string());
    runtime_shell.pending_audio.clear();
    runtime_shell.last_audio_events.clear();
    let map_name = runtime_shell
        .shell
        .snapshot()
        .expect("incoming phone map")
        .overworld
        .map_name;
    let ring_step = runtime_shell
        .shell
        .step_compiled_script_command(
            &map_name,
            "Script_ReceivePhoneCall",
            1,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .expect("execute RingTwice_StartCall source callasm");
    assert!(matches!(
        ring_step.boundary,
        Some(crate::RuntimeCompiledScriptBoundary::PhoneCallasm(
            crate::core::systems::script_runtime::ScriptPhoneCallasmPresentation::RingTwice
        ))
    ));
    integrate_visible_compiled_script_run(&mut runtime_shell, std::slice::from_ref(&ring_step))
        .expect("integrate incoming ring boundary");
    assert_eq!(
        runtime_shell
            .last_audio_events
            .iter()
            .filter(|event| event.contains("SFX_CALL"))
            .count(),
        1
    );
    advance_visible_incoming_phone_sequence(&mut runtime_shell, 59)
        .expect("advance before second ring");
    assert_eq!(
        runtime_shell
            .last_audio_events
            .iter()
            .filter(|event| event.contains("SFX_CALL"))
            .count(),
        1
    );
    advance_visible_incoming_phone_sequence(&mut runtime_shell, 1)
        .expect("start second ring after three source waits");
    assert_eq!(
        runtime_shell
            .last_audio_events
            .iter()
            .filter(|event| event.contains("SFX_CALL"))
            .count(),
        2
    );
    advance_visible_incoming_phone_sequence(&mut runtime_shell, 60)
        .expect("finish incoming double ring");
    assert!(runtime_shell.incoming_phone_sequence.is_none());

    begin_visible_incoming_phone_sequence(
        &mut runtime_shell,
        crate::core::systems::script_runtime::ScriptPhoneCallasmPresentation::HangUp,
    )
    .expect("begin incoming hangup");
    advance_visible_incoming_phone_sequence(&mut runtime_shell, 139)
        .expect("advance before hangup completion");
    assert!(runtime_shell.incoming_phone_sequence.is_some());
    advance_visible_incoming_phone_sequence(&mut runtime_shell, 1).expect("finish incoming hangup");
    assert!(runtime_shell.incoming_phone_sequence.is_none());
    assert_eq!(
        runtime_shell
            .last_audio_events
            .iter()
            .filter(|event| event.contains("SFX_HANG_UP"))
            .count(),
        1
    );
}

#[test]
fn select_without_a_registered_item_opens_the_exact_asm_textbox() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before pressing Select");
    let snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot before Select");
    assert!(snapshot.progression.registered_key_item.is_none());
    let expected = visible_asm_text(&snapshot, "_MayRegisterItemText")
        .expect("compiled SelectMenu no-registration text");

    let mut app = integrated_shell_test_app(runtime_shell);
    app.update();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ShiftRight);
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.field_notice.as_deref(), Some(expected.as_str()));
        assert!(shell.field_notice_scene.is_some());
        let notice_snapshot = shell
            .shell
            .snapshot()
            .expect("snapshot with SelectMenu notice");
        assert_eq!(
            visible_field_dialog_pages(&notice_snapshot, shell),
            Some(vec![
                "An item in your\nPACK may be".to_string(),
                "registered for use\non SELECT Button.".to_string(),
            ]),
            "ASM `para` must retain its MapTextbox page boundary"
        );
    }
    {
        let world = app.world_mut();
        let background = world
            .query_filtered::<&Sprite, With<SceneDialogTextBoxBackgroundMarker>>()
            .iter(world)
            .next()
            .expect("SelectMenu must render MapTextbox paper");
        assert_eq!(
            background.custom_size,
            Some(Vec2::new(
                TILE_SIZE * (FIELD_TEXT_BOX_WIDTH_TILES - 2.0),
                TILE_SIZE * (FIELD_TEXT_BOX_HEIGHT_TILES - 2.0),
            )),
            "SelectMenu must use Crystal's canonical 20x6 MapTextbox"
        );
    }

    app.update();
    {
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<Entity, With<DialogGlyphMarker>>()
                .iter(world)
                .next()
                .is_some(),
            "SelectMenu text must render when its typewriter reveals the first glyph"
        );
    }

    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        for _ in 0..256 {
            let snapshot = shell.shell.snapshot().expect("first notice page snapshot");
            if visible_field_dialogue_is_fully_revealed(&shell, &snapshot) {
                break;
            }
            tick_visible_field_text_reveal(&mut shell, true).expect("reveal first notice page");
        }
        press_visible_a_button(&mut shell).expect("advance across ASM para");
        assert_eq!(
            shell
                .field_text_reveal
                .as_ref()
                .map(|reveal| reveal.page_index),
            Some(1),
            "the first A must advance to the paragraph after `para`"
        );
        assert!(shell.field_notice.is_some());

        for _ in 0..256 {
            let snapshot = shell.shell.snapshot().expect("second notice page snapshot");
            if visible_field_dialogue_is_fully_revealed(&shell, &snapshot) {
                break;
            }
            tick_visible_field_text_reveal(&mut shell, true).expect("reveal second notice page");
        }
        press_visible_a_button(&mut shell).expect("close SelectMenu MapTextbox");
        assert!(shell.field_notice.is_none());
    }
}

#[test]
fn dst_confirmation_uses_the_live_dst_flag_not_special_execution_history() {
    for (dst, stale_history, expected) in [
        (true, "InitialClearDSTFlag", "12:34 DST,\nis that OK?"),
        (false, "InitialSetDSTFlag", "12:34,\nis that OK?"),
    ] {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let yes_no_index = runtime_shell
            .shell
            .runtime()
            .compiled_script_commands("MeetMomScript")
            .expect("compiled Mom script")
            .iter()
            .position(|command| {
                command.get("command").and_then(serde_json::Value::as_str) == Some("yesorno")
            })
            .expect("Mom DST yes/no command");
        *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
            .shell
            .runtime()
            .data()
            .overworld_session("PlayersHouse1F", TilePosition::new(7, 4), 0)
            .expect("PlayersHouse1F session");
        let state = runtime_shell.shell.session_mut().state_mut();
        state.overworld = crate::core::state::OverworldMemory::Active {
            map_name: "PlayersHouse1F".to_string(),
            tile: TilePosition::new(7, 4),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        state.time.dst = dst;
        state.time.game_time_hours = 12;
        state.time.game_time_minutes = 34;
        state.script_runtime.last_special_routine = Some(stale_history.to_string());
        state.script_runtime.text_window_open = true;
        state.script_runtime.active_text_label = Some("IsItDSTText".to_string());
        state.script_runtime.pending_yes_no = Some(crate::core::state::ScriptYesNoPrompt {
            source_script: "MeetMomScript".to_string(),
            command_index: yes_no_index,
        });
        sync_synthetic_current_map_image(&mut runtime_shell);

        let snapshot = runtime_shell.shell.snapshot().expect("DST prompt snapshot");
        assert_eq!(
            visible_field_dialog_pages(&snapshot, &runtime_shell),
            Some(vec![expected.to_string()])
        );
    }
}

#[test]
fn visible_pokedex_and_pokegear_overlays_do_not_render_debug_detail_rows() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before opening the start menu");
    runtime_shell
        .shell
        .set_script_flag_for_smoke(ENGINE_POKEDEX_FLAG)
        .expect("unlock Pokedex through engine flag storage");
    runtime_shell
        .shell
        .record_pokedex_seen("CHIKORITA")
        .expect("seen entry for the detail input test");
    runtime_shell
        .shell
        .set_script_flag_for_smoke(ENGINE_POKEGEAR_FLAG)
        .expect("unlock Pokegear through engine flag storage");

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Start])
        .expect("Start opens the start menu through normal input dispatch");
    assert_eq!(
        visible_start_menu_entries(&runtime_shell).expect("start menu entries"),
        vec![
            ">#DEX",
            " PACK",
            " <POKE>GEAR",
            " AB",
            " SAVE",
            " OPTION",
            " EXIT"
        ]
    );

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A opens Pokedex from the highlighted Start menu row");
    {
        let snapshot = runtime_shell.shell.snapshot().expect("Pokedex snapshot");
        let expected = visible_pokedex_menu_entries(&snapshot, &runtime_shell)
            .expect("valid Pokedex menu entries");
        let mut overlay_lines = Vec::new();
        append_visible_shell_surface_overlay(&snapshot, &runtime_shell, &mut overlay_lines);
        assert_eq!(
            overlay_lines, expected,
            "Pokedex overlay should be the same visible menu rows as the command panel"
        );
        assert_no_visible_pokedex_or_pokegear_debug_rows(&overlay_lines);
    }

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A opens Pokedex detail from the highlighted species");
    assert!(runtime_shell.pokedex_detail_open);
    {
        let snapshot = runtime_shell
            .shell
            .snapshot()
            .expect("Pokedex detail snapshot");
        let expected = visible_pokedex_menu_entries(&snapshot, &runtime_shell)
            .expect("valid Pokedex detail entries");
        let mut overlay_lines = Vec::new();
        append_visible_shell_surface_overlay(&snapshot, &runtime_shell, &mut overlay_lines);
        assert_eq!(
            overlay_lines, expected,
            "Pokedex detail overlay should be the same visible rows as the command panel"
        );
        assert_no_visible_pokedex_or_pokegear_debug_rows(&overlay_lines);
    }

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::B])
        .expect("B closes Pokedex detail");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::B])
        .expect("B closes Pokedex");

    assert!(
        runtime_shell.start_menu_cursor.is_some(),
        "CloseSubmenu returns to the Start menu"
    );
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Down])
        .expect("Down moves from Pokedex to Pack");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Down])
        .expect("Down moves from Pack to Pokegear");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A opens Pokegear from the highlighted Start menu row");
    {
        let snapshot = runtime_shell.shell.snapshot().expect("Pokegear snapshot");
        let expected = visible_pokegear_menu_entries(&snapshot, &runtime_shell)
            .expect("valid Pokegear menu entries");
        let mut overlay_lines = Vec::new();
        append_visible_shell_surface_overlay(&snapshot, &runtime_shell, &mut overlay_lines);
        assert_eq!(
            overlay_lines, expected,
            "Pokegear overlay should be the same visible menu rows as the command panel"
        );
        assert_no_visible_pokedex_or_pokegear_debug_rows(&overlay_lines);
    }

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A exits the Pokégear clock card");
    assert_eq!(
        runtime_shell.pokegear_exit,
        Some(VisiblePokegearExitPhase::Requested)
    );
    settle_visible_shell_smoke_until_idle(&mut runtime_shell).unwrap();
    assert!(!runtime_shell.pokegear_menu_open);
}

fn assert_no_visible_pokedex_or_pokegear_debug_rows(lines: &[String]) {
    assert!(
        !lines.iter().any(|entry| entry == "POKEDEX DETAIL"
            || entry == "POKEGEAR DETAIL"
            || entry.starts_with("pokedex selected=")
            || entry.starts_with("pokedex_entry ")
            || entry.starts_with("pokegear selected=")
            || entry.starts_with("pokegear phone_contacts=")),
        "Pokedex/Pokegear overlay must not expose Rust-only detail/debug rows: {lines:?}"
    );
}

#[test]
fn standalone_town_map_uses_asm_cursor_direction_and_cannot_change_pages() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pokegear_menu_open = true;
    runtime_shell.pokegear_standalone_map = true;
    runtime_shell.pokegear_page = PokegearPage::Map;
    let snapshot = runtime_shell.shell.snapshot().expect("Town Map snapshot");
    let indices = visible_pokegear_landmark_indices(&snapshot, true).expect("active region landmarks");
    assert!(indices.len() > 1);
    runtime_shell.pokegear_cursor = indices[0];

    move_visible_pokegear_cursor(&mut runtime_shell, -1).expect("Town Map Up");
    assert_eq!(runtime_shell.pokegear_cursor, indices[1]);

    move_visible_primary_cursor_left(&mut runtime_shell).expect("Town Map Left is ignored");
    assert_eq!(runtime_shell.pokegear_page, PokegearPage::Map);
    move_visible_primary_cursor_right(&mut runtime_shell).expect("Town Map Right is ignored");
    assert_eq!(runtime_shell.pokegear_page, PokegearPage::Map);
}

#[test]
fn pokegear_map_up_increments_and_down_decrements_landmarks() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Map;
    let snapshot = shell.shell.snapshot().unwrap();
    let indices = visible_pokegear_landmark_indices(&snapshot, false).unwrap();
    shell.pokegear_cursor = indices[1];
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pokegear_cursor, indices[2], "PokegearMap_ContinueMap.up increments");
    move_visible_pokegear_cursor(&mut shell, 1).unwrap();
    assert_eq!(shell.pokegear_cursor, indices[1]);
    shell.pokegear_cursor = *indices.last().unwrap();
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pokegear_cursor, indices[0]);
    move_visible_pokegear_cursor(&mut shell, 1).unwrap();
    assert_eq!(shell.pokegear_cursor, *indices.last().unwrap());
}

fn initialized_town_map_location_shell(map_name: &str) -> BevyRuntimeShell {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let asset_root = AssetRoot::new(root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let tile = crate::core::world::session::warp_tile_position_checked(
        &runtime.data().maps[map_name].events.warps[0]).unwrap();
    initialize_bevy_runtime_shell(asset_root, runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier, map_name: map_name.into(), tile_x: tile.x, tile_y: tile.y,
        }, BevyShellConfig { smoke_player_name: Some("AB".into()), ..Default::default() })
        .unwrap()
}

fn capture_town_map_location_frame(shell: &BevyRuntimeShell, label: &str) {
    let snapshot = shell.shell.snapshot().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_field_pokegear_screen(&mut commands, &snapshot, shell, &mut art,
        &shell.asset_root, &mut images).unwrap();
    queue.apply(&mut world);
    assert_eq!(art.font_error, None);
    let canvas = render_pc_audit_canvas(&mut world, &images, label);
    let reference = image::open(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/asm-oracle/fixtures")
        .join(format!("standalone-town-map-{}.png", label.strip_prefix("standalone-").unwrap())))
        .unwrap().to_rgba8();
    assert_eq!(reference.dimensions(), (160, 144));
    let scale = canvas.width() / 160;
    for y in 0..144 {
        for x in 0..160 {
            let actual = canvas.get_pixel(x * scale, y * scale).0.map(|channel| channel >> 3);
            let expected = reference.get_pixel(x, y).0.map(|channel| channel >> 3);
            assert_eq!(actual, expected, "{label}: source LCD pixel {x},{y}");
        }
    }
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        canvas.save(PathBuf::from(directory).join(format!("{label}.png"))).unwrap();
    }
}

#[test]
fn standalone_town_map_ship_entry_keeps_ship_cursor_and_source_wrap() {
    let mut shell = initialized_town_map_location_shell("FastShip1F");
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::OverworldTownMap { map_name: Some("FastShip1F".into()) }).unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[shell.pokegear_cursor].id, 95);
    assert_eq!(visible_pokegear_region(&snapshot, true).unwrap(), "KANTO");
    assert_eq!(visible_pokegear_region(&snapshot, false).unwrap(), "JOHTO");
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[
        visible_pokegear_initial_cursor_index(&snapshot, false).unwrap()].id, 1);
    capture_town_map_location_frame(&shell, "standalone-ship");
    move_visible_pokegear_cursor(&mut shell, 1).unwrap();
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[shell.pokegear_cursor].id, 94);
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[shell.pokegear_cursor].id, 88);
}

#[test]
fn standalone_town_map_special_entry_keeps_backup_landmark() {
    let mut shell = initialized_town_map_location_shell("Pokecenter2F");
    shell.shell.session_mut().state_mut().backup_warp_map_name = Some("CeladonDeptStore1F".into());
    mark_runtime_snapshot_dirty(&mut shell);
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::OverworldTownMap { map_name: Some("Pokecenter2F".into()) }).unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[shell.pokegear_cursor].constant,
        "LANDMARK_CELADON_CITY");
    capture_town_map_location_frame(&shell, "standalone-special-kanto");
    move_visible_pokegear_cursor(&mut shell, 1).unwrap();
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[shell.pokegear_cursor].id, 70);
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    assert_eq!(snapshot.presentation.pokegear_landmarks.landmarks[shell.pokegear_cursor].id, 71);
}

#[test]
fn pokegear_map_limits_match_source_regions_and_hall_of_fame_bit() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    let ids = |snapshot: &RuntimeShellSnapshot| visible_pokegear_landmark_indices(snapshot, false)
        .unwrap().iter().map(|index| snapshot.presentation.pokegear_landmarks.landmarks[*index].id)
        .collect::<Vec<_>>();
    assert_eq!(ids(&snapshot), (1..=46).collect::<Vec<_>>(),
        "Johto excludes SPECIAL and FAST_SHIP from cursor traversal");
    snapshot.overworld.map_name = "VictoryRoad".into();
    snapshot.progression.active_engine_flags.remove("ENGINE_CREDITS_SKIP");
    assert_eq!(ids(&snapshot), (88..=94).collect::<Vec<_>>());
    snapshot.progression.active_engine_flags.insert("ENGINE_CREDITS_SKIP".into());
    assert_eq!(ids(&snapshot), (47..=94).collect::<Vec<_>>());
}

#[test]
fn pokegear_special_room_uses_the_source_backup_map_region() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().backup_warp_map_name = Some("CeladonDeptStore1F".into());
    mark_runtime_snapshot_dirty(&mut shell);
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.overworld.map_name = "Pokecenter2F".into();
    assert_eq!(visible_pokegear_region(&snapshot, false).unwrap(), "KANTO");
}

#[test]
fn pokegear_map_cursor_survives_card_changes_until_gear_reopens() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().flags.engine_flags.insert("ENGINE_MAP_CARD".into(), true);
    mark_runtime_snapshot_dirty(&mut shell);
    open_visible_pokegear_menu(&mut shell).unwrap();
    cycle_visible_pokegear_page(&mut shell, 1).unwrap();
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    let selected = shell.pokegear_cursor;
    cycle_visible_pokegear_page(&mut shell, -1).unwrap();
    cycle_visible_pokegear_page(&mut shell, 1).unwrap();
    assert_eq!(shell.pokegear_cursor, selected, "Map_Init retains its cursor landmark");
    close_visible_pokegear_menu(&mut shell).unwrap();
    open_visible_pokegear_menu(&mut shell).unwrap();
    assert_ne!(shell.pokegear_cursor, selected, "Pokegear entry initializes the cursor again");
}

#[test]
fn pokegear_radio_ship_uses_the_live_time_of_day() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.overworld.map_name = "FastShip1F".into();
    snapshot.progression.radio_tuning_knob = 16;
    snapshot.progression.time.time_of_day = crate::core::world::encounters::TimeOfDay::Day;
    sync_visible_pokegear_radio(&mut shell, &snapshot).unwrap();
    assert_eq!(shell.pokegear_radio_station.as_deref(), Some("OAKS_POKEMON_TALK"));
}

#[test]
fn pokegear_radio_rejects_missing_landmark_metadata() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.overworld.map_name = "MissingRadioMap".into();
    snapshot.progression.radio_tuning_knob = 16;
    assert!(sync_visible_pokegear_radio(&mut shell, &snapshot).is_err(),
        "missing source location must not silently tune Johto stations");
}

#[test]
fn pokegear_radio_tunes_every_even_knob_position_without_station_wrapping() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pokegear_menu_open = true;
    runtime_shell.pokegear_page = PokegearPage::Radio;
    runtime_shell.pokegear_radio_station = None;
    let snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("initial radio snapshot");
    assert!(visible_pokegear_menu_entries(&snapshot, &runtime_shell).unwrap().is_empty(),
        "NoRadioName clears the name and textbox, without tuning instruction text");
    assert_eq!(visible_pokegear_radio_frequency(0), 0.5);

    for step in 0..7 {
        move_visible_pokegear_cursor(&mut runtime_shell, -1)
            .expect("advance through a no-signal tuning position");
        assert!(runtime_shell.pokegear_radio_station.is_none());
        if step == 0 {
            assert_eq!(runtime_shell.active_music.as_deref(), Some("MUSIC_NONE"));
            assert!(runtime_shell.pending_music_stop);
        }
    }
    move_visible_pokegear_cursor(&mut runtime_shell, -1)
        .expect("tune to the first exact station at knob position 16");
    assert_eq!(
        runtime_shell.pokegear_radio_station.as_deref(),
        Some("OAKS_POKEMON_TALK"),
        "the afternoon/evening branch of PKMNTalkAndPokedexShow is selected",
    );
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("tuned snapshot")
            .progression
            .radio_tuning_knob,
        16,
        "wRadioTuningKnob belongs to persistent gameplay state"
    );

    for _ in 0..40 {
        move_visible_pokegear_cursor(&mut runtime_shell, -1).expect("tune toward upper bound");
    }
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .unwrap()
            .progression
            .radio_tuning_knob,
        80,
        "Up clamps at source knob position 80"
    );
    move_visible_pokegear_cursor(&mut runtime_shell, -1).expect("Up at bound is ignored");
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .unwrap()
            .progression
            .radio_tuning_knob,
        80,
        "the source does not wrap from 20.5 back to 0.5"
    );
    assert_eq!(runtime_shell.active_music.as_deref(), Some("MUSIC_NONE"));
    let map_music = runtime_shell.shell.current_music_id().map(str::to_string);
    close_visible_pokegear_menu(&mut runtime_shell).expect("leave radio at no signal");
    assert_eq!(runtime_shell.active_music, map_music);
}

#[test]
fn pokemon_center_pc_preserves_boot_access_and_shutdown_choreography() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pending_audio.clear();

    activate_visible_special_routine_boundary(
        &mut runtime_shell,
        &SpecialRoutineEffect::PokemonCenterPc {
            party_count: 1,
            current_pc_box: 0,
        },
    )
    .expect("boot Pokemon Center PC");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PokecenterPCTurnOnText"),
    );
    assert_eq!(
        runtime_shell
            .pending_audio
            .last()
            .map(|audio| audio.audio_id.as_str()),
        Some("SFX_BOOT_PC"),
    );
    assert!(runtime_shell.pc_hub_cursor.is_none());

    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge PC boot text");
    assert!(runtime_shell.pc_hub_session_open);
    assert_eq!(
        runtime_shell.pc_hub_cursor.as_ref().unwrap().option_index,
        0
    );

    runtime_shell.pending_audio.clear();
    confirm_visible_pc_hub(&mut runtime_shell).expect("choose Bill's PC");
    assert_eq!(
        runtime_shell
            .pending_audio
            .last()
            .map(|audio| audio.audio_id.as_str()),
        Some("SFX_CHOOSE_PC_OPTION"),
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PokecenterBillsPCText"),
    );
    assert!(runtime_shell.bill_pc_action_cursor.is_some());
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge Bill's access text");
    assert!(runtime_shell.bill_pc_session_open);
    assert!(runtime_shell.bill_pc_action_cursor.is_some());

    close_visible_bill_pc_actions(&mut runtime_shell).expect("return to PC hub");
    let snapshot = runtime_shell.shell.snapshot().expect("PC hub snapshot");
    let turn_off = visible_pc_hub_actions(&snapshot).len() - 1;
    runtime_shell.pc_hub_cursor.as_mut().unwrap().option_index = turn_off;
    runtime_shell.pending_audio.clear();
    confirm_visible_pc_hub(&mut runtime_shell).expect("select Turn Off");
    assert!(runtime_shell.pending_audio.is_empty());
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PokecenterPCOaksClosedText"),
    );
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge link closed text");
    assert_eq!(
        runtime_shell
            .pending_audio
            .last()
            .map(|audio| audio.audio_id.as_str()),
        Some("SFX_SHUT_DOWN_PC"),
    );
    assert!(!runtime_shell.pc_hub_session_open);
}

#[test]
fn pokemon_center_pc_without_party_plays_only_choose_sound_before_refusal() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .storage
        .party
        .pokemon = std::array::from_fn(|_| None);
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    runtime_shell.pending_audio.clear();

    activate_visible_special_routine_boundary(
        &mut runtime_shell,
        &SpecialRoutineEffect::PokemonCenterPc {
            party_count: 0,
            current_pc_box: 0,
        },
    )
    .expect("refuse empty-party Pokemon Center PC");
    assert_eq!(
        runtime_shell
            .pending_audio
            .last()
            .map(|audio| audio.audio_id.as_str()),
        Some("SFX_CHOOSE_PC_OPTION"),
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PokecenterPCCantUseText"),
    );
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge no-party refusal");
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .all(|audio| audio.audio_id != "SFX_SHUT_DOWN_PC")
    );
}

#[test]
fn bills_pc_move_mode_confirms_then_saves_entry_and_every_move() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let save_path = std::env::temp_dir().join(format!(
        "crystal-bills-pc-move-{}-{}.crystalsave",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    runtime_shell.quick_save_path = Some(save_path.clone());
    let state = runtime_shell.shell.session_mut().state_mut();
    let party_mon = state.storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    party_mon.item = None;
    party_mon.mail = None;
    let box_mon = party_mon.clone();
    assert!(state.storage.pc_boxes[0].add_pokemon(box_mon));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_action_cursor = Some(MenuCursor {
        surface_id: "pc:bill-actions".to_string(),
        option_index: 3,
    });

    confirm_visible_bill_pc_action(&mut runtime_shell).expect("select MOVE without Mail");
    assert!(runtime_shell.save_flow.is_some());
    assert!(!runtime_shell.bill_pc_move_open);
    let snapshot = runtime_shell.shell.snapshot().expect("MOVE save snapshot");
    let mut prompt_entries = Vec::new();
    push_visible_save_dialog_entries(&mut prompt_entries, &snapshot, &runtime_shell)
        .expect("render MOVE save prompt");
    assert!(prompt_entries.join(" ").contains("Each time you move"));

    runtime_shell.save_flow.as_mut().unwrap().yes_no_index = 1;
    confirm_visible_save_menu(&mut runtime_shell).expect("decline initial MOVE save");
    assert!(!save_path.exists());
    assert_eq!(
        runtime_shell
            .bill_pc_action_cursor
            .as_ref()
            .unwrap()
            .option_index,
        3,
    );
    confirm_visible_bill_pc_action(&mut runtime_shell).expect("select MOVE again");

    confirm_visible_save_menu(&mut runtime_shell).expect("accept initial MOVE save");
    assert!(save_path.exists());
    assert!(matches!(
        runtime_shell.save_flow.as_ref().map(|flow| flow.stage),
        Some(VisibleSaveFlowStage::Saved)
    ));
    confirm_visible_save_menu(&mut runtime_shell).expect("acknowledge initial MOVE save");
    assert!(runtime_shell.bill_pc_move_open);
    runtime_shell.pending_audio.clear();

    confirm_visible_bill_pc_move(&mut runtime_shell).expect("open box source submenu");
    confirm_visible_bill_pc_pokemon_action(&mut runtime_shell).expect("choose MOVE");
    switch_visible_pc_move_container(&mut runtime_shell, -1).expect("switch to party");
    runtime_shell.storage_cursor.as_mut().unwrap().option_index = 1;
    confirm_visible_bill_pc_move(&mut runtime_shell).expect("move box Pokemon into party");

    let pending_snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot during MOVE save delay");
    assert_eq!(pending_snapshot.party.slots.len(), 1);
    assert_eq!(pending_snapshot.storage.boxes[0].slots.len(), 1);
    assert_eq!(
        runtime_shell.last_action_status.as_deref(),
        Some("SAVING... LEAVE ON!")
    );
    let mut saving_entries = Vec::new();
    push_visible_storage_dialog_entries(&mut saving_entries, &pending_snapshot, &runtime_shell)
        .expect("PC save display");
    assert!(
        saving_entries
            .iter()
            .any(|entry| entry == "Saving… Leave ON!")
    );
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .all(|audio| audio.audio_id != "SFX_SAVE")
    );
    let before_commit = runtime_shell
        .shell
        .runtime()
        .load_save(&save_path)
        .expect("load save during pre-move hold");
    assert_eq!(before_commit.storage.party.filled_slots(), 1);
    assert_eq!(before_commit.storage.pc_boxes[0].count, 1);

    advance_visible_bill_pc_move_save(&mut runtime_shell, 19)
        .expect("advance first 19 pre-move frames");
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("nineteenth-frame snapshot")
            .party
            .slots
            .len(),
        1
    );
    advance_visible_bill_pc_move_save(&mut runtime_shell, 1)
        .expect("commit move on twentieth frame");
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("post-commit snapshot")
            .party
            .slots
            .len(),
        2
    );
    assert!(matches!(
        runtime_shell
            .bill_pc_move_save
            .as_ref()
            .map(|save| (save.phase, save.frames_remaining)),
        Some((VisibleBillPcMoveSavePhase::AfterSave, 24))
    ));
    let retained = &runtime_shell.bill_pc_move_save.as_ref().unwrap().presentation;
    assert_eq!(retained.names.len(), 1, "saving retains the pre-insertion destination list");
    assert_eq!(retained.cursor.option_index, 1);
    assert_eq!(retained.pokemon.species_id, "CYNDAQUIL");
    press_visible_b_button(&mut runtime_shell).expect("B is ignored during post-save hold");
    assert!(runtime_shell.bill_pc_move_save.is_some());
    advance_visible_bill_pc_move_save(&mut runtime_shell, 23)
        .expect("advance first 23 post-save frames");
    assert!(runtime_shell.bill_pc_move_save.is_some());
    advance_visible_bill_pc_move_save(&mut runtime_shell, 1)
        .expect("finish twenty-fourth post-save frame");
    assert!(runtime_shell.bill_pc_move_save.is_none());
    assert_eq!(
        runtime_shell.last_action_status.as_deref(),
        Some("POKEMON MOVED")
    );

    assert_eq!(runtime_shell.storage_cursor.as_ref().unwrap().option_index, 1,
        "Move returns to the inserted destination row, not the top of the list");
    let saved = runtime_shell
        .shell
        .runtime()
        .load_save(&save_path)
        .expect("load per-move autosave");
    assert_eq!(saved.storage.party.filled_slots(), 2);
    assert_eq!(saved.storage.pc_boxes[0].count, 0);
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|audio| { audio.audio_id == "SFX_SAVE" })
    );

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_file(save_path.with_extension("crystalsave.bak"));
}

#[test]
fn bills_pc_move_mode_rejects_mail_before_save_confirmation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_action_cursor = Some(MenuCursor {
        surface_id: "pc:bill-actions".to_string(),
        option_index: 3,
    });

    confirm_visible_bill_pc_action(&mut runtime_shell).expect("check MOVE Mail gate");

    assert!(runtime_shell.save_flow.is_none());
    assert!(!runtime_shell.bill_pc_move_open);
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("There is a #MON holding MAIL.\n\nPlease remove the MAIL."),
    );
}

#[test]
fn bills_pc_release_plays_cry_and_retains_the_source_farewell_sequence() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.item = None;
    pokemon.mail = None;
    pokemon.nickname = "EMBER".to_string();
    let boxed = pokemon.clone();
    assert!(state.storage.pc_boxes[0].add_pokemon(boxed.clone()));
    assert!(state.storage.pc_boxes[0].add_pokemon(boxed));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.pending_audio.clear();
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.storage_cursor = Some(MenuCursor {
        surface_id: storage_cursor_surface_id(0),
        option_index: 0,
    });

    request_visible_pc_pokemon_release(&mut runtime_shell)
        .expect("open release confirmation");
    confirm_visible_pc_release_prompt(&mut runtime_shell).expect("confirm release");

    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|audio| { audio.audio_id == "CRY_CYNDAQUIL" })
    );
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Released <PK><MN>.")
    );
    assert!(matches!(
        runtime_shell
            .pc_release_sequence
            .as_ref()
            .map(|sequence| (sequence.phase, sequence.frames_remaining)),
        Some((VisiblePcReleasePhase::Released, 80))
    ));
    press_visible_a_button(&mut runtime_shell).expect("A is ignored during release hold");
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Released <PK><MN>.")
    );
    advance_visible_pc_release_sequence(&mut runtime_shell, 79)
        .expect("advance first 79 release frames");
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Released <PK><MN>.")
    );
    advance_visible_pc_release_sequence(&mut runtime_shell, 1)
        .expect("enter farewell on eightieth release frame");
    assert_eq!(runtime_shell.pc_notice.as_deref(), Some("Bye, CYNDAQUIL!"),
        "ReleasePKMN_ByePKMN places the species name on the same line, not the nickname");
    assert!(matches!(
        runtime_shell
            .pc_release_sequence
            .as_ref()
            .map(|sequence| (sequence.phase, sequence.frames_remaining)),
        Some((VisiblePcReleasePhase::Bye, 50))
    ));
    press_visible_b_button(&mut runtime_shell).expect("B is ignored during farewell hold");
    advance_visible_pc_release_sequence(&mut runtime_shell, 49)
        .expect("advance first 49 farewell frames");
    assert!(runtime_shell.pc_release_sequence.is_some());
    advance_visible_pc_release_sequence(&mut runtime_shell, 1)
        .expect("finish fiftieth farewell frame");
    assert!(runtime_shell.pc_release_sequence.is_none());
    assert!(runtime_shell.pc_notice.is_none());
    assert_eq!(
        runtime_shell
            .storage_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(0)
    );
}

#[test]
fn bills_pc_deposit_plays_cry_and_holds_the_stored_message() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.item = None;
    pokemon.mail = None;
    pokemon.nickname = "EMBER".to_string();
    state.storage.party.pokemon[1] = Some(pokemon.clone());
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.pending_audio.clear();
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_deposit_open = true;
    runtime_shell.party_menu_open = false;
    runtime_shell.party_cursor = 0;
    runtime_shell.storage_cursor = Some(MenuCursor {
        surface_id: pc_party_surface_id().to_string(),
        option_index: 0,
    });

    deposit_visible_party_pokemon(&mut runtime_shell).expect("deposit selected Pokemon");

    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|audio| { audio.audio_id == "CRY_CYNDAQUIL" })
    );
    assert!(runtime_shell.pc_notice.is_none(), "PlayMonCry must finish before the success text is placed");
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 500).unwrap();
    assert!(runtime_shell.pc_notice.is_none(), "elapsed frames must not bypass WaitSFX");
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1).unwrap();
    assert_eq!(runtime_shell.pc_notice.as_deref(), Some("Stored EMBER!"));
    assert!(matches!(
        runtime_shell
            .pc_transfer_sequence
            .as_ref()
            .map(|sequence| (sequence.phase, sequence.frames_remaining)),
        Some((VisiblePcTransferPhase::SuccessHold, 50))
    ));
    press_visible_a_button(&mut runtime_shell).expect("A is ignored during stored-message hold");
    assert_eq!(runtime_shell.pc_notice.as_deref(), Some("Stored EMBER!"));
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 49)
        .expect("advance first 49 stored-message frames");
    assert!(runtime_shell.pc_transfer_sequence.is_some());
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1)
        .expect("finish fiftieth stored-message frame");
    assert!(runtime_shell.pc_transfer_sequence.is_none());
    assert!(runtime_shell.pc_notice.is_none());
    assert!(!runtime_shell.party_menu_open);
    assert!(runtime_shell.bill_pc_deposit_open);
    assert_eq!(runtime_shell.party_cursor, 0);
}

#[test]
fn bills_pc_withdraw_plays_cry_and_holds_the_got_message() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.item = None;
    pokemon.mail = None;
    pokemon.nickname = "EMBER".to_string();
    let boxed = pokemon.clone();
    assert!(state.storage.pc_boxes[0].add_pokemon(boxed.clone()));
    assert!(state.storage.pc_boxes[0].add_pokemon(boxed));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.pending_audio.clear();
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.storage_cursor = Some(MenuCursor {
        surface_id: storage_cursor_surface_id(0),
        option_index: 1,
    });

    withdraw_visible_pc_pokemon(&mut runtime_shell).expect("withdraw selected Pokemon");

    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|audio| { audio.audio_id == "CRY_CYNDAQUIL" })
    );
    assert!(runtime_shell.pc_notice.is_none(), "PlayMonCry must finish before the success text is placed");
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 500).unwrap();
    assert!(runtime_shell.pc_notice.is_none(), "elapsed frames must not bypass WaitSFX");
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1).unwrap();
    assert_eq!(runtime_shell.pc_notice.as_deref(), Some("Got EMBER!"));
    assert!(matches!(
        runtime_shell
            .pc_transfer_sequence
            .as_ref()
            .map(|sequence| (sequence.phase, sequence.frames_remaining)),
        Some((VisiblePcTransferPhase::SuccessHold, 50))
    ));
    press_visible_b_button(&mut runtime_shell).expect("B is ignored during got-message hold");
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 49)
        .expect("advance first 49 got-message frames");
    assert!(runtime_shell.pc_transfer_sequence.is_some());
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1)
        .expect("finish fiftieth got-message frame");
    assert!(runtime_shell.pc_transfer_sequence.is_none());
    assert!(runtime_shell.pc_notice.is_none());
    assert_eq!(
        runtime_shell
            .storage_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(0)
    );
}

#[test]
fn bills_pc_deposit_refusal_waits_for_wrong_sfx_then_fifty_frames() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0]
        .as_ref()
        .expect("party Pokemon")
        .clone();
    state.storage.party.pokemon[1] = Some(pokemon);
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.pending_audio.clear();
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_deposit_open = true;
    runtime_shell.party_menu_open = false;
    runtime_shell.party_cursor = 0;
    runtime_shell.storage_cursor = Some(MenuCursor {
        surface_id: pc_party_surface_id().to_string(),
        option_index: 0,
    });

    deposit_visible_party_pokemon(&mut runtime_shell).expect("refuse deposited Mail Pokemon");

    assert_eq!(runtime_shell.pc_notice.as_deref(), Some("Remove MAIL."));
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|audio| { audio.audio_id == "SFX_WRONG" })
    );
    assert!(matches!(
        runtime_shell
            .pc_transfer_sequence
            .as_ref()
            .map(|sequence| sequence.phase),
        Some(VisiblePcTransferPhase::RefusalWaitSfx)
    ));
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 50)
        .expect("refusal cannot advance while wrong SFX is queued");
    assert!(matches!(
        runtime_shell
            .pc_transfer_sequence
            .as_ref()
            .map(|sequence| sequence.phase),
        Some(VisiblePcTransferPhase::RefusalWaitSfx)
    ));
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1)
        .expect("enter post-SFX refusal hold");
    assert!(matches!(
        runtime_shell
            .pc_transfer_sequence
            .as_ref()
            .map(|sequence| (sequence.phase, sequence.frames_remaining)),
        Some((VisiblePcTransferPhase::RefusalHold, 50))
    ));
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 49)
        .expect("advance first 49 refusal frames");
    assert!(runtime_shell.pc_notice.is_some());
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1)
        .expect("finish fiftieth refusal frame");
    assert!(runtime_shell.pc_transfer_sequence.is_none());
    assert!(runtime_shell.pc_notice.is_none());
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("snapshot")
            .storage
            .party_count,
        2
    );
}

#[test]
fn bills_pc_change_box_requires_the_source_save_flow_before_switching() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let save_path = std::env::temp_dir().join(format!(
        "crystal-bevy-change-box-{}-{}.crystalsave",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    runtime_shell.quick_save_path = Some(save_path.clone());
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_box_cursor = Some(MenuCursor {
        surface_id: "pc:bill-boxes".to_string(),
        option_index: 1,
    });

    confirm_visible_bill_pc_box(&mut runtime_shell).expect("request box switch");

    assert!(runtime_shell.save_flow.is_none());
    assert!(runtime_shell.bill_pc_box_action_cursor.is_some());
    confirm_visible_bill_pc_box_action(&mut runtime_shell).expect("select SWITCH");
    assert!(runtime_shell.save_flow.is_some());
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("snapshot")
            .storage
            .current_pc_box,
        0
    );
    let snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("save prompt snapshot");
    let mut prompt_entries = Vec::new();
    push_visible_save_dialog_entries(&mut prompt_entries, &snapshot, &runtime_shell)
        .expect("render CHANGE BOX save prompt");
    assert!(prompt_entries.join(" ").contains("When you change a"));

    runtime_shell.save_flow.as_mut().unwrap().yes_no_index = 1;
    confirm_visible_save_menu(&mut runtime_shell).expect("decline box-change save");
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("declined snapshot")
            .storage
            .current_pc_box,
        0
    );
    assert!(!save_path.exists());

    confirm_visible_bill_pc_box(&mut runtime_shell).expect("open box actions again");
    confirm_visible_bill_pc_box_action(&mut runtime_shell).expect("select SWITCH again");
    confirm_visible_save_menu(&mut runtime_shell).expect("accept box-change save");
    assert!(save_path.exists());
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("switched snapshot")
            .storage
            .current_pc_box,
        1
    );
    assert!(matches!(
        runtime_shell.save_flow.as_ref().map(|flow| flow.stage),
        Some(VisibleSaveFlowStage::Saved)
    ));
    confirm_visible_save_menu(&mut runtime_shell).expect("acknowledge saved box change");
    assert!(runtime_shell.save_flow.is_none());
    assert_eq!(
        runtime_shell
            .bill_pc_box_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(1)
    );

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_file(save_path.with_extension("crystalsave.bak"));
}

#[test]
fn scripted_shop_and_bill_box_renderers_reject_invalid_retained_state() {
    let mut shop_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shop_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_shop = Some(crate::core::state::ScriptShopRequest {
        mart_type: "MARTTYPE_STANDARD".to_string(),
        mart_id: "MART_CHERRYGROVE".to_string(),
        inventory: vec![
            "POTION".to_string(),
            "ANTIDOTE".to_string(),
            "PARLYZ_HEAL".to_string(),
            "AWAKENING".to_string(),
        ],
        source_script: "CherrygroveMartClerkScript".to_string(),
        command_index: 3,
    });
    shop_shell.shop_welcome_seen = true;
    shop_shell.shop_top_cursor = Some(MenuCursor {
        surface_id: "wrong:shop-surface".to_string(),
        option_index: 2,
    });
    let snapshot = shop_shell
        .shell
        .presentation_snapshot()
        .expect("scripted shop snapshot");
    let shop_error = visible_scene_dialog_entries(&snapshot, &shop_shell)
        .expect_err("an invalid shop cursor must not silently select BUY")
        .to_string();
    assert!(shop_error.contains("shop:top"), "{shop_error}");

    let mut box_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    box_shell
        .shell
        .session_mut()
        .state_mut()
        .storage
        .pc_boxes
        .pop()
        .expect("remove BOX14 for the malformed retained-state fixture");
    box_shell.bill_pc_box_cursor = Some(MenuCursor {
        surface_id: "pc:bill-boxes".to_string(),
        option_index: crate::core::models::MAX_PC_BOXES - 1,
    });
    let snapshot = box_shell
        .shell
        .presentation_snapshot()
        .expect("Bill PC box snapshot");
    let box_error = visible_bill_pc_box_entries(&snapshot, &box_shell)
        .expect_err("a missing retained PC box must not render fabricated BOX data")
        .to_string();
    assert!(
        box_error.contains("Bill's PC box") && box_error.contains("missing from storage"),
        "{box_error}"
    );

    let mut mailbox_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    mailbox_shell.mailbox_cursor = Some(MenuCursor {
        surface_id: "pc:mailbox".to_string(),
        option_index: 0,
    });
    mailbox_shell.mailbox_action_cursor = Some(MenuCursor {
        surface_id: "pc:mailbox-actions".into(), option_index: 0,
    });
    let snapshot = mailbox_shell
        .shell
        .presentation_snapshot()
        .expect("empty mailbox snapshot");
    let mailbox_error = visible_scene_dialog_entries(&snapshot, &mailbox_shell)
        .expect_err("an empty mailbox must not fabricate a selectable first message")
        .to_string();
    assert!(mailbox_error.contains("mailbox"), "{mailbox_error}");
}

#[test]
fn invalid_scripted_shop_item_cursor_is_not_reinitialized_by_confirmation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .pending_shop = Some(crate::core::state::ScriptShopRequest {
        mart_type: "MARTTYPE_STANDARD".to_string(),
        mart_id: "MART_CHERRYGROVE".to_string(),
        inventory: vec![
            "POTION".to_string(),
            "ANTIDOTE".to_string(),
            "PARLYZ_HEAL".to_string(),
            "AWAKENING".to_string(),
        ],
        source_script: "CherrygroveMartClerkScript".to_string(),
        command_index: 3,
    });
    runtime_shell.menu_cursor = Some(MenuCursor {
        surface_id: "wrong:shop-items".to_string(),
        option_index: 0,
    });

    let error = buy_visible_shop_cursor_item(&mut runtime_shell)
        .expect_err("an invalid live shop-item cursor must fail before buying")
        .to_string();
    assert!(error.contains("shop:"), "{error}");
}

#[test]
fn invalid_script_yes_no_cursor_is_not_coerced_to_yes() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.held_item_swap_prompt = true;
    runtime_shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "wrong:yes-no-surface".to_string(),
        option_index: 1,
    });
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("yes/no presentation snapshot");

    let error = scene_dialog_yes_no_cursor_index(&snapshot, &runtime_shell)
        .expect_err("an invalid retained YES/NO cursor must not silently select YES")
        .to_string();
    assert!(error.contains("held-item swap"), "{error}");
}

#[test]
fn invalid_compiled_vertical_menu_surface_is_not_rebased_to_the_first_menu() {
    let menu = crate::RuntimeMenuSnapshot {
        menu_id: "TEST".to_string(),
        source: crate::RuntimeMenuSource::ScriptDefinition {
            map_name: "TestMap".to_string(),
        },
        definition: None,
        layout: crate::RuntimeMenuLayoutSnapshot {
            declared_coords: None,
            data_commands: Vec::new(),
            vertical_menus: vec![crate::RuntimeVerticalMenuSnapshot {
                source_script: "ScriptA".to_string(),
                loadmenu_command_index: 0,
                verticalmenu_command_index: 1,
                header_label: "Header".to_string(),
                data_label: None,
                options: vec!["ONE".to_string(), "TWO".to_string()],
                two_dimensional: false,
                rows: None,
                columns: None,
                spacing: None,
            }],
        },
        window_open: true,
        coords: None,
        menu_2d_requested: false,
    };
    let cursor = Some(MenuCursor {
        surface_id: "wrong:compiled-menu".to_string(),
        option_index: 0,
    });

    let target_error = active_menu_target_from_live_cursor(&menu, &cursor)
        .expect_err("an invalid live compiled-menu surface must not select the first menu")
        .to_string();
    assert!(
        target_error.contains("wrong:compiled-menu"),
        "{target_error}"
    );

    let selected_error = selected_vertical_menu(&menu, &cursor)
        .expect_err("compiled-menu confirmation must reject the same invalid surface")
        .to_string();
    assert!(
        selected_error.contains("wrong:compiled-menu"),
        "{selected_error}"
    );
}

#[test]
fn invalid_retained_elevator_surface_is_not_cleared_by_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.elevator_cursor = Some(MenuCursor {
        surface_id: "wrong:elevator-surface".to_string(),
        option_index: 0,
    });

    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("invalid elevator presentation snapshot");
    let render_error = visible_scene_dialog_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid retained elevator must fail its input-owning renderer")
        .to_string();
    assert!(
        render_error.contains("wrong:elevator-surface"),
        "{render_error}"
    );

    let move_error = move_visible_elevator_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid retained elevator must fail before navigation")
        .to_string();
    assert!(
        move_error.contains("wrong:elevator-surface"),
        "{move_error}"
    );
    assert!(runtime_shell.elevator_cursor.is_some());

    let confirm_error = select_visible_elevator_floor(&mut runtime_shell)
        .expect_err("an invalid retained elevator must fail before confirmation")
        .to_string();
    assert!(
        confirm_error.contains("wrong:elevator-surface"),
        "{confirm_error}"
    );
    assert!(runtime_shell.elevator_cursor.is_some());

    let routed_move_error = move_visible_primary_cursor(&mut runtime_shell, 1)
        .expect_err("direction dispatch must keep the invalid elevator as input owner")
        .to_string();
    assert!(
        routed_move_error.contains("wrong:elevator-surface"),
        "{routed_move_error}"
    );

    let routed_confirm_error = press_visible_a_button(&mut runtime_shell)
        .expect_err("A dispatch must keep the invalid elevator as input owner")
        .to_string();
    assert!(
        routed_confirm_error.contains("wrong:elevator-surface"),
        "{routed_confirm_error}"
    );
}

#[test]
fn invalid_field_pack_cursor_is_not_rendered_as_a_playable_diagnostic_row() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::Items);
    runtime_shell.bag_cursor = Some(MenuCursor {
        surface_id: "wrong:bag-surface".to_string(),
        option_index: 0,
    });
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("field Pack presentation snapshot");

    let error = visible_field_pack_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid input-owning Pack must fail rendering")
        .to_string();
    assert!(error.contains("bag:items"), "{error}");
}

#[test]
fn invalid_field_pack_cursor_is_not_reinitialized_by_navigation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::Items);
    runtime_shell.bag_cursor = Some(MenuCursor {
        surface_id: "wrong:bag-surface".to_string(),
        option_index: 0,
    });

    let error = move_visible_bag_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid live Pack cursor must fail before navigation")
        .to_string();
    assert!(error.contains("bag:items"), "{error}");
}

#[test]
fn invalid_field_pack_action_cursor_is_not_reinitialized_by_confirmation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .add_bag_item("POTION", 1)
        .expect("add a Pack item");
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::Items);
    runtime_shell.bag_cursor = Some(MenuCursor {
        surface_id: "bag:items".to_string(),
        option_index: 0,
    });
    open_visible_field_pack_action_menu(&mut runtime_shell).expect("open Pack actions");
    runtime_shell.field_pack_action_cursor = Some(MenuCursor {
        surface_id: "wrong:pack-actions".to_string(),
        option_index: 0,
    });

    let error = execute_visible_field_pack_action(&mut runtime_shell)
        .expect_err("an invalid live Pack action cursor must fail before confirmation")
        .to_string();
    assert!(error.contains("pack:actions"), "{error}");
}

#[test]
fn invalid_pc_storage_cursor_is_not_reinitialized_by_selection() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0]
        .as_ref()
        .expect("party Pokemon")
        .clone();
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.storage_cursor = Some(MenuCursor {
        surface_id: "wrong:pc-box".to_string(),
        option_index: 0,
    });

    let error = selected_current_box_slot_index(&mut runtime_shell)
        .expect_err("an invalid live PC cursor must fail before selecting a Pokemon")
        .to_string();
    assert!(error.contains("pc:box"), "{error}");
}

#[test]
fn invalid_party_cursor_is_not_clamped_to_a_playable_row() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.party_cursor = usize::MAX;
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("party presentation snapshot");

    let error = visible_party_menu_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid retained party cursor must fail closed")
        .to_string();
    assert!(error.contains("party cursor"), "{error}");
}

#[test]
fn party_move_reorder_does_not_invent_a_cancel_row() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let move_count = runtime_shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("party Pokemon")
        .moves
        .len();
    runtime_shell.party_cursor = 0;
    runtime_shell.party_move_reorder_open = true;
    runtime_shell.party_move_cursor = Some(MenuCursor {
        surface_id: party_move_reorder_surface_id(0),
        option_index: move_count,
    });
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("party move-reorder presentation snapshot");

    let error = visible_party_menu_entries(&snapshot, &runtime_shell)
        .expect_err("MOVE reorder has exactly the Pokemon's moves and no CANCEL row")
        .to_string();
    assert!(error.contains("move-reorder"), "{error}");
}

#[test]
fn invalid_party_controller_cursors_are_not_clamped_before_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.party_cursor = usize::MAX;

    let regular_error = move_visible_regular_party_menu_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid Pokemon/CANCEL cursor must fail before navigation")
        .to_string();
    assert!(regular_error.contains("party cursor"), "{regular_error}");

    let slot_error = move_visible_party_slot_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid Pokemon-only cursor must fail before navigation")
        .to_string();
    assert!(slot_error.contains("party cursor"), "{slot_error}");

    let selection_error = selected_party_index(&mut runtime_shell)
        .expect_err("an invalid selected party index must not become the final Pokemon")
        .to_string();
    assert!(
        selection_error.contains("party cursor"),
        "{selection_error}"
    );

    let action_error = open_visible_party_action_menu(&mut runtime_shell)
        .expect_err("only the exact trailing CANCEL row may close the party menu")
        .to_string();
    assert!(action_error.contains("party cursor"), "{action_error}");
}

#[test]
fn invalid_battle_pack_target_cursor_is_not_clamped_before_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.battle_pack_target_mode = Some(BattlePackTargetMode::PartyPokemon);
    runtime_shell.party_cursor = usize::MAX;

    let primary_error = move_visible_battle_pack_target_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid battle-item target cursor must fail before navigation")
        .to_string();
    assert!(primary_error.contains("party cursor"), "{primary_error}");

    let secondary_error = move_visible_battle_pack_target_secondary_cursor(&mut runtime_shell, 1)
        .expect_err("secondary target navigation must reject the same invalid cursor")
        .to_string();
    assert!(
        secondary_error.contains("party cursor"),
        "{secondary_error}"
    );
}

#[test]
fn invalid_start_and_party_action_cursors_are_not_reinitialized_by_confirmation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.start_menu_cursor = Some(MenuCursor {
        surface_id: "wrong:start-menu".to_string(),
        option_index: 0,
    });
    let start_error = selected_visible_start_menu_option(&mut runtime_shell)
        .expect_err("an invalid live start-menu cursor must fail before selection")
        .to_string();
    assert!(start_error.contains(START_MENU_SURFACE_ID), "{start_error}");

    runtime_shell.party_cursor = 0;
    open_visible_party_action_menu(&mut runtime_shell).expect("open party actions");
    runtime_shell.party_action_cursor = Some(MenuCursor {
        surface_id: "wrong:party-actions".to_string(),
        option_index: 0,
    });
    let party_error = execute_visible_party_action(&mut runtime_shell)
        .expect_err("an invalid live party-action cursor must fail before selection")
        .to_string();
    assert!(party_error.contains("party:actions"), "{party_error}");
}

#[test]
fn party_menu_construction_applies_the_source_invalid_cursor_initialization() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .add_party_pokemon(
            "ABRA",
            10,
            None,
            None,
            "AB",
            1,
            crate::core::models::Dv::from_non_hp(10, 10, 10, 10),
        )
        .expect("add a second party Pokemon");
    runtime_shell.party_cursor = usize::MAX;

    open_visible_party_menu(&mut runtime_shell).expect("construct source party menu");

    assert_eq!(runtime_shell.party_cursor, 0);
}

#[test]
fn invalid_party_summary_state_is_not_wrapped_or_clamped_by_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.party_summary_open = true;
    runtime_shell.party_summary_page = 0;

    let page_error = cycle_visible_party_summary_page(&mut runtime_shell, 1)
        .expect_err("an invalid summary page must fail before cycling")
        .to_string();
    assert!(page_error.contains("summary page"), "{page_error}");

    runtime_shell.party_summary_page = 1;
    runtime_shell.party_cursor = usize::MAX;
    let party_error = move_visible_party_summary_pokemon(&mut runtime_shell, 1)
        .expect_err("an invalid summary Pokemon must fail before navigation")
        .to_string();
    assert!(party_error.contains("party cursor"), "{party_error}");
}

#[test]
fn invalid_pokedex_cursor_is_not_clamped_by_navigation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pokedex_menu_open = true;
    runtime_shell.pokedex_cursor = usize::MAX;

    let error = move_visible_pokedex_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid Pokedex cursor must fail before navigation")
        .to_string();
    assert!(error.contains("Pokedex cursor"), "{error}");
}

#[test]
fn invalid_pokegear_phone_cursor_is_not_clamped_by_navigation() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .initialize_permanent_phone_numbers()
        .expect("initialize permanent phone contacts");
    runtime_shell.pokegear_menu_open = true;
    runtime_shell.pokegear_page = PokegearPage::Phone;
    runtime_shell.pokegear_phone_cursor = usize::MAX;

    let error = move_visible_pokegear_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid Pokegear phone cursor must fail before navigation")
        .to_string();
    assert!(error.contains("phone cursor"), "{error}");
}

#[test]
fn invalid_pokegear_map_cursor_is_not_rebased_to_the_first_landmark() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pokegear_menu_open = true;
    runtime_shell.pokegear_page = PokegearPage::Map;
    runtime_shell.pokegear_cursor = usize::MAX;

    let error = move_visible_pokegear_cursor(&mut runtime_shell, 1)
        .expect_err("an invalid Pokegear map cursor must fail before navigation")
        .to_string();
    assert!(error.contains("landmark cursor"), "{error}");
}

#[test]
fn options_directions_refresh_the_rendered_settings() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_options_menu(&mut runtime_shell).expect("open Options");
    for row in 0..7 {
        runtime_shell.options_cursor = row;
        let before = cached_runtime_snapshot(&mut runtime_shell).expect("render before input");
        dispatch_visible_options_direction(&mut runtime_shell, GameButton::Right);
        assert_eq!(runtime_shell.last_error, None);
        let after = cached_runtime_snapshot(&mut runtime_shell).expect("render after input");
        assert_ne!(
            option_value_for_item(&before.trainer.options, OPTIONS_MENU_ITEMS[row]),
            option_value_for_item(&after.trainer.options, OPTIONS_MENU_ITEMS[row]),
            "the rendered value must refresh for {:?}",
            OPTIONS_MENU_ITEMS[row]
        );
        dispatch_visible_options_direction(&mut runtime_shell, GameButton::Left);
        let restored = cached_runtime_snapshot(&mut runtime_shell).expect("render reverse input");
        assert_eq!(
            option_value_for_item(&before.trainer.options, OPTIONS_MENU_ITEMS[row]),
            option_value_for_item(&restored.trainer.options, OPTIONS_MENU_ITEMS[row])
        );
        assert!(runtime_shell.options_menu_open);
    }
}

#[test]
fn options_directions_redraw_the_cursor_and_wrap_like_asm() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_options_menu(&mut runtime_shell).expect("open Options");
    for direction in [
        GameButton::Down,
        GameButton::Up,
        GameButton::Up,
        GameButton::Down,
    ] {
        let revision = runtime_shell.snapshot_revision;
        dispatch_visible_options_direction(&mut runtime_shell, direction);
        assert_ne!(
            runtime_shell.snapshot_revision, revision,
            "cursor movement must redraw"
        );
    }
    assert_eq!(runtime_shell.options_cursor, 0);
    dispatch_visible_options_direction(&mut runtime_shell, GameButton::Up);
    assert_eq!(runtime_shell.options_cursor, 7);
    confirm_visible_options_selection(&mut runtime_shell).expect("A on CANCEL");
    assert!(!runtime_shell.options_menu_open);
}

#[test]
fn options_live_keys_refresh_settings_and_frame_preview() {
    let mut app = integrated_shell_test_app(core_modular_title_shell_for_test());
    open_title_main_menu_for_test(&mut app);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowDown);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    for item in OPTIONS_MENU_ITEMS.iter().copied().take(7) {
        let before = {
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert!(shell.options_menu_open);
            let snapshot = shell.shell.snapshot().expect("settings before input");
            option_value_for_item(&snapshot.trainer.options, item)
        };
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
        {
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            let snapshot = shell.shell.snapshot().expect("settings after input");
            assert_ne!(option_value_for_item(&snapshot.trainer.options, item), before);
            assert_eq!(
                app.world().resource::<RenderedTilesetArt>().selected_window_frame_id,
                textbox_frame_id(snapshot.trainer.options.frame)
            );
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowDown);
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    assert!(!app.world().resource::<BevyRuntimeShell>().options_menu_open);
}

#[test]
fn options_menu_open_initializes_the_source_text_speed_row() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.options_cursor = usize::MAX;

    open_visible_options_menu(&mut runtime_shell).expect("open Options from source row zero");

    assert_eq!(runtime_shell.options_cursor, 0);
}

#[test]
fn auxiliary_overworld_menu_renderers_reject_invalid_retained_state() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");

    runtime_shell.pokedex_cursor = usize::MAX;
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Pokedex presentation snapshot");
    let pokedex_error = visible_pokedex_menu_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid Pokedex cursor must fail rendering")
        .to_string();
    assert!(pokedex_error.contains("Pokedex cursor"), "{pokedex_error}");

    runtime_shell.pokedex_cursor = 0;
    runtime_shell.pokedex_detail_open = true;
    runtime_shell.pokedex_detail_page = usize::MAX;
    let pokedex_page_error = visible_pokedex_menu_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid Pokedex detail page must fail rendering")
        .to_string();
    assert!(
        pokedex_page_error.contains("Pokedex detail page"),
        "{pokedex_page_error}"
    );

    runtime_shell.pokedex_detail_open = false;
    runtime_shell
        .shell
        .initialize_permanent_phone_numbers()
        .expect("initialize permanent phone contacts");
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Pokegear presentation snapshot");
    runtime_shell.pokegear_page = PokegearPage::Phone;
    runtime_shell.pokegear_phone_cursor = usize::MAX;
    let phone_error = visible_pokegear_menu_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid Pokegear phone cursor must fail rendering")
        .to_string();
    assert!(phone_error.contains("phone cursor"), "{phone_error}");

    runtime_shell.pokegear_page = PokegearPage::Radio;
    runtime_shell.pokegear_radio_station = Some("MAPRADIO_POKEMON_CHANNEL".to_string());
    runtime_shell.pokegear_radio_broadcast = None;
    let radio_error = visible_pokegear_menu_entries(&snapshot, &runtime_shell)
        .expect_err("an missing Pokegear broadcast must fail rendering")
        .to_string();
    assert!(radio_error.contains("live broadcast"), "{radio_error}");

    runtime_shell.options_cursor = usize::MAX;
    let options_error = visible_options_menu_entries(&snapshot, &runtime_shell)
        .expect_err("an invalid Options cursor must fail rendering")
        .to_string();
    assert!(options_error.contains("Options cursor"), "{options_error}");

    let mut missing_landmark_snapshot = snapshot.clone();
    let active_map = missing_landmark_snapshot.overworld.map_name.clone();
    Arc::make_mut(&mut missing_landmark_snapshot.presentation)
        .pokegear_landmarks
        .map_to_landmark
        .remove(&active_map);
    let region_error = visible_pokegear_region(&missing_landmark_snapshot, false)
        .expect_err("a missing Town Map location must not become JOHTO")
        .to_string();
    assert!(region_error.contains("landmark mapping"), "{region_error}");
}

#[test]
fn invalid_pokedex_detail_page_is_not_advanced_or_wrapped_by_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pokedex_menu_open = true;
    runtime_shell.pokedex_detail_open = true;
    runtime_shell.pokedex_cursor = 0;
    runtime_shell.pokedex_detail_page = usize::MAX;

    let error = press_visible_pokedex_a_button(&mut runtime_shell)
        .expect_err("an invalid retained Pokedex page must fail before A advances it")
        .to_string();
    assert!(error.contains("Pokedex detail page"), "{error}");
}

#[test]
fn bills_pc_change_box_name_uses_the_source_eight_character_naming_screen() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_box_cursor = Some(MenuCursor {
        surface_id: "pc:bill-boxes".to_string(),
        option_index: 0,
    });

    confirm_visible_bill_pc_box(&mut runtime_shell).expect("open box actions");
    runtime_shell
        .bill_pc_box_action_cursor
        .as_mut()
        .expect("box action cursor")
        .option_index = 1;
    confirm_visible_bill_pc_box_action(&mut runtime_shell).expect("select NAME");

    let input = runtime_shell
        .pending_name_input
        .as_mut()
        .expect("box naming screen");
    assert_eq!(input.label, "BOX NAME?");
    assert_eq!(input.max_length, 8);
    input.value = "FIRE".to_string();
    confirm_visible_player_name_input(&mut runtime_shell).expect("confirm box name");

    let snapshot = runtime_shell.shell.snapshot().expect("named snapshot");
    assert_eq!(snapshot.storage.boxes[0].name, "FIRE");
    assert!(runtime_shell.pending_name_input.is_none());
    assert!(runtime_shell.bill_pc_box_cursor.is_some());
    assert!(runtime_shell.bill_pc_box_action_cursor.is_none());
}

#[test]
fn bills_pc_change_box_print_empty_uses_source_refusal_sfx_and_delay() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.pending_audio.clear();
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_box_cursor = Some(MenuCursor {
        surface_id: "pc:bill-boxes".to_string(),
        option_index: 0,
    });

    confirm_visible_bill_pc_box(&mut runtime_shell).expect("open box actions");
    runtime_shell
        .bill_pc_box_action_cursor
        .as_mut()
        .expect("box action cursor")
        .option_index = 2;
    confirm_visible_bill_pc_box_action(&mut runtime_shell).expect("select PRINT");

    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("There's no <PK><MN>.")
    );
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|audio| { audio.audio_id == "SFX_WRONG" })
    );
    assert!(matches!(
        runtime_shell
            .pc_transfer_sequence
            .as_ref()
            .map(|sequence| sequence.phase),
        Some(VisiblePcTransferPhase::RefusalWaitSfx)
    ));
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 1).expect("finish refusal SFX wait");
    advance_visible_pc_transfer_sequence(&mut runtime_shell, 50)
        .expect("finish source refusal delay");
    assert!(runtime_shell.pc_notice.is_none());
    assert!(runtime_shell.bill_pc_box_cursor.is_some());
}

#[test]
fn bills_pc_change_box_print_without_link_uses_source_printer_error() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = runtime_shell.shell.session_mut().state_mut();
    let boxed = state.storage.party.pokemon[0]
        .as_ref()
        .expect("party Pokemon")
        .clone();
    assert!(state.storage.pc_boxes[0].add_pokemon(boxed));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    runtime_shell.bill_pc_session_open = true;
    runtime_shell.bill_pc_box_cursor = Some(MenuCursor {
        surface_id: "pc:bill-boxes".to_string(),
        option_index: 0,
    });

    confirm_visible_bill_pc_box(&mut runtime_shell).expect("open box actions");
    runtime_shell
        .bill_pc_box_action_cursor
        .as_mut()
        .expect("box action cursor")
        .option_index = 2;
    confirm_visible_bill_pc_box_action(&mut runtime_shell).expect("select PRINT");

    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Printer Error 2\n\nCheck the Game Boy\nPrinter Manual.")
    );
    for _ in 0..256 {
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("snapshot");
        if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) {
            break;
        }
        tick_visible_field_text_reveal(&mut runtime_shell, true).expect("reveal printer status");
    }
    press_visible_a_button(&mut runtime_shell).expect("A is ignored by printer status");
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Printer Error 2\n\nCheck the Game Boy\nPrinter Manual.")
    );
    press_visible_b_button(&mut runtime_shell).expect("B cancels box printing");
    assert!(runtime_shell.pc_notice.is_none());
    assert!(runtime_shell.bill_pc_box_cursor.is_some());
    assert!(runtime_shell.bill_pc_box_action_cursor.is_none());
}

#[test]
fn unown_printer_special_opens_authored_menu_instead_of_debug_boundary() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");

    assert!(
        activate_visible_special_routine_boundary(
            &mut runtime_shell,
            &crate::core::systems::special_routines::SpecialRoutineEffect::UnownPrinter {
                letters: vec![1, 2, 26],
            },
        )
        .expect("activate Unown Printer")
    );

    assert!(runtime_shell.special_boundary.is_none());
    assert_eq!(
        runtime_shell.visible_unown_printer,
        Some(VisibleUnownPrinter {
            selected: 0,
            letters: vec![1, 2, 26],
        })
    );
    assert!(retained_field_fullscreen_active(&runtime_shell));
    assert_eq!(
        visible_field_command_entries(
            &runtime_shell
                .shell
                .presentation_snapshot()
                .expect("snapshot"),
            &runtime_shell,
        )
        .expect("Unown Printer entries"),
        vec![
            " ALPH RUINS STAMP",
            "A",
            "A PRINT",
            "B CANCEL",
            "← PREVIOUS",
            "→ NEXT",
            "Do what?",
        ]
    );

    move_visible_unown_printer(&mut runtime_shell, -1).expect("wrap left to VACANT");
    assert_eq!(
        runtime_shell
            .visible_unown_printer
            .as_ref()
            .expect("printer remains open")
            .selected,
        26
    );
    move_visible_unown_printer(&mut runtime_shell, 1).expect("wrap right to A");
    assert_eq!(
        runtime_shell
            .visible_unown_printer
            .as_ref()
            .expect("printer remains open")
            .selected,
        0
    );

    print_visible_unown_stamp(&mut runtime_shell).expect("attempt stamp print");
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Printer Error 2\n\nCheck the Game Boy\nPrinter Manual.")
    );
    for _ in 0..256 {
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("snapshot");
        if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) {
            break;
        }
        tick_visible_field_text_reveal(&mut runtime_shell, true).expect("reveal printer status");
    }
    press_visible_a_button(&mut runtime_shell).expect("A is ignored by printer status");
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        Some("Printer Error 2\n\nCheck the Game Boy\nPrinter Manual.")
    );
    press_visible_b_button(&mut runtime_shell).expect("B cancels stamp printing");
    assert!(runtime_shell.pc_notice.is_none());
    assert!(runtime_shell.visible_unown_printer.is_some());

    let mut unavailable_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    assert!(
        !activate_visible_special_routine_boundary(
            &mut unavailable_shell,
            &crate::core::systems::special_routines::SpecialRoutineEffect::UnownPrinter {
                letters: vec![],
            },
        )
        .expect("ignore unavailable Unown Printer")
    );
    assert!(unavailable_shell.special_boundary.is_none());
    assert!(unavailable_shell.visible_unown_printer.is_none());
}

#[test]
fn buenas_password_requires_the_source_three_choice_input_and_allows_a_wrong_answer() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "RadioTower2F".to_string(),
        tile: TilePosition::new(8, 7),
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("RadioTower2F", TilePosition::new(8, 7), 0)
        .expect("start Radio Tower 2F session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("Buena")
        .expect("compiled Buena script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("BuenasPassword")
        })
        .expect("Buena script password special");
    assert!(
        open_visible_buena_password_for_script_command(&mut runtime_shell, "Buena", command_index,)
            .expect("intercept Buena password special")
    );
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "RadioTower2F",
        "Buena",
        command_index + 1,
    );
    let options = runtime_shell
        .visible_buena_password
        .as_ref()
        .expect("Buena menu")
        .options
        .clone();
    let correct = options[runtime_shell
        .shell
        .session()
        .state()
        .buenas_password
        .option_index]
        .clone();
    for host_mirror in ["_buena_category", "_buena_category_type", "_buena_password"] {
        assert!(
            !runtime_shell
                .shell
                .session()
                .state()
                .script_runtime
                .variables
                .contains_key(host_mirror),
            "Buena's source state must not gain the immediate {host_mirror} mirror"
        );
    }
    assert_eq!(options.len(), 3);
    assert!(runtime_shell.special_boundary.is_none());
    assert!(!scene_dialog_surface_active(
        &runtime_shell
            .shell
            .presentation_snapshot()
            .expect("snapshot"),
        &runtime_shell,
    ));
    assert_eq!(
        runtime_shell
            .visible_buena_password
            .as_ref()
            .expect("Buena menu")
            .options,
        options
    );

    press_visible_b_button(&mut runtime_shell).expect("B is disabled");
    assert!(runtime_shell.visible_buena_password.is_some());

    let wrong_index = options
        .iter()
        .position(|option| option != &correct)
        .expect("three choices include a wrong answer");
    runtime_shell
        .visible_buena_password
        .as_mut()
        .expect("Buena menu")
        .cursor
        .option_index = wrong_index;
    resolve_visible_buena_password_selection(&mut runtime_shell).expect("submit wrong answer");

    assert!(runtime_shell.visible_buena_password.is_none());
    assert!(
        !runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .variables
            .contains_key("BUENA_PASSWORD")
    );
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref(),
        Some("0")
    );
}

#[test]
fn buena_remember_password_special_opens_its_source_yes_no_menu_without_host_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "RadioTower2F".to_string(),
        tile: TilePosition::new(8, 7),
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("RadioTower2F", TilePosition::new(8, 7), 0)
        .expect("start Radio Tower 2F session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("Buena")
        .expect("compiled Buena script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("AskRememberPassword")
        })
        .expect("Buena remember-password special");
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "RadioTower2F",
        "Buena",
        command_index,
    );

    execute_visible_active_script_step(&mut runtime_shell)
        .expect("AskRememberPassword must open its own source input surface");
    assert_eq!(
        runtime_shell
            .yes_no_cursor
            .as_ref()
            .map(|cursor| (cursor.surface_id.as_str(), cursor.option_index)),
        Some(("script:remember-password", 0))
    );
    assert!(runtime_shell.pending_remember_password.is_some());
    assert!(runtime_shell.special_boundary.is_none());
    let prompt_snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("remember-password prompt snapshot");
    assert!(scene_dialog_surface_active(
        &prompt_snapshot,
        &runtime_shell
    ));
    assert!(scene_dialog_yes_no_active(&prompt_snapshot, &runtime_shell));
    assert_eq!(
        scene_dialog_yes_no_cursor_index(&prompt_snapshot, &runtime_shell)
            .expect("valid remember-password cursor"),
        0
    );

    press_visible_b_button(&mut runtime_shell).expect("B selects the source NO row");
    for _ in 0..14 {
        assert!(
            advance_visible_remember_password_prompt(&mut runtime_shell)
                .expect("advance retained Buena menu delay")
        );
        assert!(runtime_shell.pending_remember_password.is_some());
        assert_eq!(
            runtime_shell
                .shell
                .session()
                .state()
                .script_runtime
                .variables
                .get("_remember_password"),
            None,
            "AskRememberPassword does not return until its 15-frame menu delay ends"
        );
    }
    assert!(
        advance_visible_remember_password_prompt(&mut runtime_shell)
            .expect("finish Buena menu delay and resume the script")
    );
    assert!(runtime_shell.pending_remember_password.is_none());
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref(),
        Some("0")
    );
    assert!(
        !runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .variables
            .contains_key("_remember_password")
    );
    assert!(
        !runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .variables
            .contains_key("_yes_no_result")
    );
    assert_eq!(
        runtime_shell
            .shell
            .presentation_snapshot()
            .expect("forgot-password branch snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("RadioTower2FBuenaComeBackAfterListeningText")
    );
}

#[test]
fn battle_tower_action_is_silent_and_reaches_the_authored_receptionist_text() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "BattleTower1F".to_string(),
        tile: TilePosition::new(10, 9),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("BattleTower1F", TilePosition::new(10, 9), 0)
        .expect("start Battle Tower 1F session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "BattleTower1F",
        "BattleTower1FReceptionistScript",
        0,
    );
    continue_visible_script_after_prompt(&mut runtime_shell)
        .expect("run receptionist to authored dialogue");

    assert!(
        runtime_shell.special_boundary.is_none(),
        "BattleTowerAction is an SRAM/wScriptVar command and owns no ASM presentation"
    );
    assert_eq!(
        runtime_shell
            .shell
            .presentation_snapshot()
            .expect("Battle Tower snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_BattleTowerWelcomesYou")
    );
}

#[test]
fn battle_tower_challenge_menu_uses_source_choices_and_cancel_result() {
    fn shell_at_challenge_menu() -> (BevyRuntimeShell, usize) {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
            map_name: "BattleTower1F".to_string(),
            tile: TilePosition::new(10, 9),
            facing: Direction::Up,
            mode: MovementMode::Normal,
        };
        *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
            .shell
            .runtime()
            .data()
            .overworld_session("BattleTower1F", TilePosition::new(10, 9), 0)
            .expect("start Battle Tower 1F session");
        sync_synthetic_current_map_image(&mut runtime_shell);
        let command_index = runtime_shell
            .shell
            .runtime()
            .compiled_script_commands("Script_Menu_ChallengeExplanationCancel")
            .expect("compiled Battle Tower menu script")
            .iter()
            .position(|command| {
                command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                    && command
                        .get("args")
                        .and_then(serde_json::Value::as_array)
                        .and_then(|args| args.first())
                        .and_then(serde_json::Value::as_str)
                        == Some("Menu_ChallengeExplanationCancel")
            })
            .expect("Battle Tower challenge menu special");
        runtime_shell
            .shell
            .session_mut().state_mut()
            .script_runtime
            .script_value = Some("1".to_string());
        (runtime_shell, command_index)
    }

    let (mut explanation_shell, command_index) = shell_at_challenge_menu();
    assert!(
        open_visible_battle_tower_challenge_menu_for_script_command(
            &mut explanation_shell,
            "Script_Menu_ChallengeExplanationCancel",
            command_index,
        )
        .expect("open source Battle Tower challenge menu")
    );
    arm_visible_active_script_cursor_with_origin(
        &mut explanation_shell,
        "BattleTower1F",
        "Script_Menu_ChallengeExplanationCancel",
        command_index + 1,
    );
    assert!(explanation_shell.special_boundary.is_none());
    assert_eq!(
        explanation_shell
            .visible_battle_tower_challenge_menu
            .as_ref()
            .expect("challenge menu")
            .cursor
            .option_index,
        0
    );
    move_visible_primary_cursor_down(&mut explanation_shell).expect("select Explanation");
    resolve_visible_battle_tower_challenge_menu(&mut explanation_shell, false)
        .expect("submit Explanation");
    assert_eq!(
        explanation_shell
            .shell
            .presentation_snapshot()
            .expect("explanation snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_BattleTowerIntroduction_2")
    );

    let (mut cancel_shell, command_index) = shell_at_challenge_menu();
    open_visible_battle_tower_challenge_menu_for_script_command(
        &mut cancel_shell,
        "Script_Menu_ChallengeExplanationCancel",
        command_index,
    )
    .expect("open source Battle Tower challenge menu");
    arm_visible_active_script_cursor_with_origin(
        &mut cancel_shell,
        "BattleTower1F",
        "Script_Menu_ChallengeExplanationCancel",
        command_index + 1,
    );
    resolve_visible_battle_tower_challenge_menu(&mut cancel_shell, true)
        .expect("cancel challenge menu");
    assert_eq!(
        cancel_shell
            .shell
            .presentation_snapshot()
            .expect("cancel snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_WeHopeToServeYouAgain")
    );
}

#[test]
fn battle_tower_rule_check_is_silent_on_success_and_queues_authored_failure_text() {
    let mut invalid_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let invalid = invalid_shell
        .shell
        .apply_declared_special_routine("CheckForBattleTowerRules")
        .expect("check invalid Battle Tower party");
    assert!(
        activate_visible_special_routine_boundary(&mut invalid_shell, &invalid.outcome.effect,)
            .expect("present Battle Tower rejection")
    );
    assert_eq!(
        invalid_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("ExcuseMeYoureNotReadyText")
    );
    assert!(
        invalid_shell
            .special_boundary
            .as_ref()
            .expect("opening rejection page")
            .details
            .join("\n")
            .contains("You're not ready")
    );
    assert_eq!(
        invalid_shell
            .special_boundary_queue
            .back()
            .map(|boundary| boundary.label.as_str()),
        Some("BattleTowerReturnWhenReadyText")
    );

    let mut valid_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let species = ["CHIKORITA", "CYNDAQUIL", "TOTODILE"]
        .map(|species_id| valid_shell.shell.runtime().data().pokemon[species_id].clone());
    {
        let state = valid_shell.shell.session_mut().state_mut();
        state.storage.party.pokemon = std::array::from_fn(|_| None);
        for (index, species) in species.into_iter().enumerate() {
            state.storage.party.pokemon[index] = Some(crate::core::models::Pokemon::new_for_tests(
                species,
                10,
                crate::core::models::Dv::default(),
            ));
        }
        state.sync_party_from_storage();
    }
    let valid = valid_shell
        .shell
        .apply_declared_special_routine("CheckForBattleTowerRules")
        .expect("check valid Battle Tower party");
    assert!(
        !activate_visible_special_routine_boundary(&mut valid_shell, &valid.outcome.effect,)
            .expect("valid rule check is source-silent")
    );
    assert!(valid_shell.special_boundary.is_none());
}

#[test]
fn battle_tower_room_menu_selects_a_level_or_returns_source_cancel_code() {
    fn shell_at_room_menu() -> (BevyRuntimeShell, usize) {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
            map_name: "BattleTower1F".to_string(),
            tile: TilePosition::new(10, 9),
            facing: Direction::Up,
            mode: MovementMode::Normal,
        };
        *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
            .shell
            .runtime()
            .data()
            .overworld_session("BattleTower1F", TilePosition::new(10, 9), 0)
            .expect("start Battle Tower 1F session");
        sync_synthetic_current_map_image(&mut runtime_shell);
        let command_index = runtime_shell
            .shell
            .runtime()
            .compiled_script_commands("Script_ChooseChallenge")
            .expect("compiled Battle Tower challenge script")
            .iter()
            .position(|command| {
                command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                    && command
                        .get("args")
                        .and_then(serde_json::Value::as_array)
                        .and_then(|args| args.first())
                        .and_then(serde_json::Value::as_str)
                        == Some("BattleTowerRoomMenu")
            })
            .expect("Battle Tower room menu special");
        (runtime_shell, command_index)
    }

    let (mut selection_shell, command_index) = shell_at_room_menu();
    assert!(
        open_visible_battle_tower_room_menu_for_script_command(
            &mut selection_shell,
            "Script_ChooseChallenge",
            command_index,
        )
        .expect("open source Battle Tower room menu")
    );
    arm_visible_active_script_cursor_with_origin(
        &mut selection_shell,
        "BattleTower1F",
        "Script_ChooseChallenge",
        command_index + 1,
    );
    assert_eq!(
        selection_shell
            .visible_battle_tower_room_menu
            .as_ref()
            .expect("room menu")
            .level_groups,
        vec![1, 2, 3, 4]
    );
    assert!(selection_shell.special_boundary.is_none());
    move_visible_primary_cursor_down(&mut selection_shell)
        .expect("source picker wraps Down from L10 to Cancel");
    assert_eq!(
        selection_shell
            .visible_battle_tower_room_menu
            .as_ref()
            .expect("room menu")
            .cursor
            .option_index,
        4
    );
    move_visible_primary_cursor_up(&mut selection_shell)
        .expect("source picker wraps Up from Cancel to L10");
    press_visible_a_button(&mut selection_shell).expect("select level 10");
    assert!(selection_shell.visible_battle_tower_room_menu.is_none());
    assert_eq!(
        selection_shell
            .shell
            .session()
            .state()
            .battle_tower
            .level_group,
        1
    );
    assert_eq!(
        selection_shell
            .shell
            .presentation_snapshot()
            .expect("room selection snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_RightThisWayToYourBattleRoom")
    );

    let (mut cancel_shell, command_index) = shell_at_room_menu();
    open_visible_battle_tower_room_menu_for_script_command(
        &mut cancel_shell,
        "Script_ChooseChallenge",
        command_index,
    )
    .expect("open source Battle Tower room menu");
    arm_visible_active_script_cursor_with_origin(
        &mut cancel_shell,
        "BattleTower1F",
        "Script_ChooseChallenge",
        command_index + 1,
    );
    press_visible_b_button(&mut cancel_shell).expect("request room-menu cancellation");
    assert!(matches!(
        cancel_shell
            .visible_battle_tower_room_menu
            .as_ref()
            .expect("cancel confirmation")
            .phase,
        VisibleBattleTowerRoomMenuPhase::ConfirmCancel { yes_no_index: 0 }
    ));
    press_visible_a_button(&mut cancel_shell).expect("confirm room-menu cancellation");
    assert!(cancel_shell.visible_battle_tower_room_menu.is_none());
    assert_eq!(
        cancel_shell
            .shell
            .presentation_snapshot()
            .expect("room cancel snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_WantToGoIntoABattleRoom")
    );

    let (mut rejected_shell, command_index) = shell_at_room_menu();
    let starter_species = rejected_shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("starter")
        .species
        .clone();
    rejected_shell
        .shell
        .session_mut()
        .state_mut()
        .storage
        .party
        .pokemon[0] = Some(crate::core::models::Pokemon::new_for_tests(
        starter_species,
        20,
        crate::core::models::Dv::default(),
    ));
    rejected_shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    open_visible_battle_tower_room_menu_for_script_command(
        &mut rejected_shell,
        "Script_ChooseChallenge",
        command_index,
    )
    .expect("open room menu for over-level party");
    arm_visible_active_script_cursor_with_origin(
        &mut rejected_shell,
        "BattleTower1F",
        "Script_ChooseChallenge",
        command_index + 1,
    );
    press_visible_a_button(&mut rejected_shell).expect("reject level 10 room");
    assert!(matches!(
        rejected_shell
            .visible_battle_tower_room_menu
            .as_ref()
            .expect("retained rejected room menu")
            .phase,
        VisibleBattleTowerRoomMenuPhase::Rejection { ref message }
            if message == "A party POKéMON\ntops this level."
    ));
    assert_eq!(
        rejected_shell
            .shell
            .session()
            .state()
            .battle_tower
            .level_group,
        0
    );
    press_visible_a_button(&mut rejected_shell).expect("return to level picker");
    assert!(matches!(
        rejected_shell
            .visible_battle_tower_room_menu
            .as_ref()
            .expect("room picker reopened")
            .phase,
        VisibleBattleTowerRoomMenuPhase::PickLevel
    ));
}

#[test]
fn battle_tower_receptionist_escort_launches_canonical_opponent_battle() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "BattleTower1F".to_string(),
        tile: TilePosition::new(7, 7),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("BattleTower1F", TilePosition::new(7, 7), 0)
        .expect("start Battle Tower 1F session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    runtime_shell.shell.session_mut().state_mut().battle_tower.level_group = 1;
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "BattleTower1F",
        "Script_WalkToBattleTowerElevator",
        0,
    );

    continue_visible_script_after_prompt(&mut runtime_shell).expect("start receptionist escort");
    for _ in 0..128 {
        settle_visible_shell_smoke_until_idle(&mut runtime_shell)
            .expect("settle receptionist escort boundary");
        if runtime_shell
            .shell
            .session()
            .state()
            .battle_tower
            .loaded_trainer_id
            .is_some()
        {
            break;
        }
        if runtime_shell.visible_wait_sfx_boundary {
            let retained_transient_audio = runtime_shell.transient_audio_playing;
            runtime_shell.transient_audio_playing = false;
            let presentation = runtime_shell
                .shell
                .presentation_snapshot()
                .expect("escort waitsfx snapshot");
            advance_visible_wait_sfx_boundary(&mut runtime_shell, &presentation, false)
                .expect("complete elevator sound fence");
            runtime_shell.transient_audio_playing = retained_transient_audio;
        }
        if runtime_shell.visible_script_movement.is_some() {
            advance_visible_script_movement(&mut runtime_shell)
                .expect("advance receptionist escort movement");
        }
    }

    assert!(
        runtime_shell
            .shell
            .session()
            .state()
            .battle_tower
            .loaded_trainer_id
            .is_some(),
        "battle-room entry must load its source-selected opponent: cursor={:?} scene={:?} movement={:?} boundary={:?} events={:?}",
        runtime_shell.active_script_cursor,
        runtime_shell.pending_scene_script,
        runtime_shell.visible_script_movement,
        runtime_shell.special_boundary,
        runtime_shell.last_audio_events
    );
    assert!(
        runtime_shell.special_boundary.is_none(),
        "opponent loading is source-silent, not a diagnostic boundary: {:?}",
        runtime_shell.special_boundary
    );
    let state = runtime_shell.shell.session().state();
    let intro_text = state
        .script_runtime
        .variables
        .get("battle_tower_intro_text")
        .expect("canonical Battle Tower intro text");
    assert!(
        runtime_shell
            .shell
            .runtime()
            .data()
            .asm_text
            .contains_key(intro_text),
        "selected Battle Tower intro must exist in the ASM text catalog"
    );
    assert_eq!(
        runtime_shell.shell.current_map_name(),
        "BattleTowerBattleRoom"
    );
}

#[test]
fn battle_tower_win_resumes_the_room_loop_instead_of_the_failure_warp() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "BattleTowerBattleRoom".to_string(),
        tile: TilePosition::new(4, 6),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("BattleTowerBattleRoom", TilePosition::new(4, 6), 0)
        .expect("start Battle Tower battle-room session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    runtime_shell.shell.session_mut().state_mut().battle_tower.level_group = 1;
    runtime_shell
        .shell
        .load_battle_tower_opponent_special("BATTLETOWERBATTLEROOM_YOUNGSTER".to_string())
        .expect("load source-selected Battle Tower opponent");

    let battle_command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("Script_BattleRoomLoop")
        .expect("compiled Battle Tower room loop")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("BattleTowerBattle")
        })
        .expect("BattleTowerBattle command");
    runtime_shell
        .shell
        .apply_compiled_script_command(
            "BattleTowerBattleRoom",
            "Script_BattleRoomLoop",
            battle_command_index,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .expect("start the source BattleTowerBattle special");
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "BattleTowerBattleRoom",
        "Script_BattleRoomLoop",
        battle_command_index + 1,
    );
    runtime_shell.shell.session_mut().state_mut().battle_result = 0;

    complete_visible_battle_tower_battle(&mut runtime_shell)
        .expect("resume Battle Tower room script after a win");
    assert_eq!(
        runtime_shell.visible_walk_warp_phase,
        Some(VisibleWalkWarpPhase::MapReloadFadeIn),
        "reloadmap must retain the room-script cursor until its white fade completes"
    );
    assert!(
        advance_visible_smoke_walk_warp_phase(&mut runtime_shell)
            .expect("complete Battle Tower reloadmap fade-in")
    );

    assert_eq!(
        runtime_shell.shell.current_map_name(),
        "BattleTowerBattleRoom"
    );
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .battle_tower
            .beaten_trainers,
        1
    );
    assert!(
        runtime_shell.visible_script_movement.is_some(),
        "the win branch must reach the opponent walk-out movement before offering the next battle: cursor={:?} boundary={:?} events={:?}",
        runtime_shell.active_script_cursor,
        runtime_shell.special_boundary,
        runtime_shell.last_audio_events
    );
}

fn battle_tower_room_decision_shell() -> (BevyRuntimeShell, PathBuf) {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let save_path = std::env::temp_dir().join(format!(
        "crystal-battle-tower-room-decision-{}-{}.crystalsave",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    runtime_shell.quick_save_path = Some(save_path.clone());

    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "BattleTower1F".to_string(),
        tile: TilePosition::new(7, 7),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("BattleTower1F", TilePosition::new(7, 7), 0)
        .expect("start Battle Tower 1F save session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let scene_table = runtime_shell
        .shell
        .runtime()
        .data()
        .map_scene_table("BattleTower1F")
        .expect("Battle Tower 1F scenes")
        .clone();
    runtime_shell
        .shell
        .session_mut().state_mut()
        .scenes
        .enter_map("BattleTower1F", &scene_table)
        .expect("arm Battle Tower resume scene");
    runtime_shell
        .shell
        .save(&save_path)
        .expect("write pre-entry Battle Tower quick-save");

    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "BattleTowerBattleRoom".to_string(),
        tile: TilePosition::new(4, 6),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("BattleTowerBattleRoom", TilePosition::new(4, 6), 0)
        .expect("start Battle Tower battle-room session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let tower = &mut runtime_shell.shell.session_mut().state_mut().battle_tower;
    tower.level_group = 1;
    tower.beaten_trainers = 3;
    tower.reward_item = "HP_UP".to_string();
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "BattleTowerBattleRoom",
        "Script_DontBattleNextOpponent",
        0,
    );
    continue_visible_script_after_prompt(&mut runtime_shell)
        .expect("run room decision script to its first prompt");
    (runtime_shell, save_path)
}

fn advance_battle_tower_room_decision_to_yes_no(runtime_shell: &mut BevyRuntimeShell) {
    for _ in 0..4096 {
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("room decision presentation snapshot");
        if snapshot.ui.pending_yes_no.is_some() {
            return;
        }
        if snapshot.script_events.pending_text_label.is_some() {
            advance_visible_text_label(runtime_shell).expect("open room decision text");
            continue;
        }
        if !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot) {
            tick_visible_field_text_reveal(runtime_shell, true).expect("reveal room decision text");
            continue;
        }
        press_visible_a_button(runtime_shell).expect("advance room decision text");
    }
    panic!("Battle Tower room decision did not reach a yes/no prompt");
}

#[test]
fn battle_tower_save_and_quit_updates_sram_save_then_reboots_to_the_intro() {
    let (mut runtime_shell, save_path) = battle_tower_room_decision_shell();
    advance_battle_tower_room_decision_to_yes_no(&mut runtime_shell);
    accept_visible_pending_yes_no(&mut runtime_shell).expect("save and end the session");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("finish Battle Tower save-and-reset script");

    assert!(
        runtime_shell.intro_screen.is_some(),
        "Reset must reboot through the intro"
    );
    assert!(
        runtime_shell.title_menu.is_some(),
        "Reset must rebuild the title flow"
    );
    assert!(
        runtime_shell.special_boundary.is_none(),
        "Reset is not a diagnostic boundary"
    );
    assert!(runtime_shell.active_script_cursor.is_none());

    let saved = runtime_shell
        .shell
        .runtime()
        .load_save(&save_path)
        .expect("load Battle Tower SRAM-updated save");
    assert!(matches!(
        saved.overworld,
        crate::core::state::OverworldMemory::Active { ref map_name, .. }
            if map_name == "BattleTower1F"
    ));
    assert_eq!(
        saved
            .scenes
            .map_scenes
            .get("BattleTower1F")
            .map(String::as_str),
        Some("SCENE_BATTLETOWER1F_CHECKSTATE")
    );
    assert_eq!(saved.battle_tower.challenge_state, 1);
    assert!(saved.battle_tower.quick_saved);
    assert_eq!(saved.battle_tower.level_group, 1);
    assert_eq!(saved.battle_tower.beaten_trainers, 3);
    assert_eq!(saved.battle_tower.reward_item, "HP_UP");

    skip_visible_intro_screen(&mut runtime_shell, GameButton::Start).expect("skip reboot intro");
    advance_visible_title_to_main_menu(&mut runtime_shell).expect("open reboot title menu");
    select_visible_title_menu_option(&mut runtime_shell).expect("open Continue summary");
    assert!(runtime_shell.visible_continue_screen.is_some());
    select_visible_title_menu_option(&mut runtime_shell).expect("continue Battle Tower save");
    assert_eq!(runtime_shell.shell.current_map_name(), "BattleTower1F");
    assert_eq!(
        runtime_shell
            .shell
            .presentation_snapshot()
            .expect("resumed Battle Tower presentation")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_WeveBeenWaitingForYou")
    );
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("resume the saved Battle Tower challenge");
    let resumed = runtime_shell
        .shell
        .snapshot()
        .expect("resumed Battle Tower battle snapshot");
    assert_eq!(resumed.overworld.map_name, "BattleTowerBattleRoom");
    assert_eq!(
        resumed.battle_tower.beaten_trainers, 4,
        "loading the resumed opponent increments the source SRAM counter before battle"
    );
    assert_eq!(
        resumed
            .battle
            .as_ref()
            .map(|battle| battle.battle_type.as_str()),
        Some("BATTLETYPE_BATTLE_TOWER")
    );
    assert!(runtime_shell.special_boundary.is_none());

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_file(save_path.with_extension("crystalsave.bak"));
}

#[test]
fn battle_tower_cancel_clears_the_challenge_and_finishes_the_1f_farewell() {
    let (mut runtime_shell, save_path) = battle_tower_room_decision_shell();
    advance_battle_tower_room_decision_to_yes_no(&mut runtime_shell);
    decline_visible_pending_yes_no(&mut runtime_shell).expect("decline session save");
    advance_battle_tower_room_decision_to_yes_no(&mut runtime_shell);
    accept_visible_pending_yes_no(&mut runtime_shell).expect("cancel challenge");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("finish Battle Tower cancellation warp and farewell");

    let state = runtime_shell.shell.session().state();
    assert_eq!(runtime_shell.shell.current_map_name(), "BattleTower1F");
    assert!(matches!(
        state.overworld,
        crate::core::state::OverworldMemory::Active { tile, .. }
            if tile == TilePosition::new(7, 7)
    ));
    assert_eq!(state.battle_tower.challenge_state, 0);
    assert!(!state.battle_tower.quick_saved);
    assert_eq!(state.battle_tower.beaten_trainers, 0);
    assert_eq!(state.battle_tower.record_state, 0);
    assert_eq!(state.battle_tower.record_last_day, None);
    assert_eq!(state.battle_tower.record_reset_counter, 0);
    assert!(runtime_shell.special_boundary.is_none());
    assert!(runtime_shell.active_script_cursor.is_none());

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_file(save_path.with_extension("crystalsave.bak"));
}

#[test]
fn battle_tower_declining_save_and_cancel_returns_to_the_next_opponent_loop() {
    let (mut runtime_shell, save_path) = battle_tower_room_decision_shell();
    advance_battle_tower_room_decision_to_yes_no(&mut runtime_shell);
    decline_visible_pending_yes_no(&mut runtime_shell).expect("decline session save");
    advance_battle_tower_room_decision_to_yes_no(&mut runtime_shell);
    decline_visible_pending_yes_no(&mut runtime_shell).expect("continue the challenge");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("return to the next Battle Tower opponent");

    let snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("continued Battle Tower snapshot");
    assert_eq!(snapshot.overworld.map_name, "BattleTowerBattleRoom");
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .battle_tower
            .beaten_trainers,
        4
    );
    assert!(
        snapshot.battle.is_some(),
        "the next source-selected opponent must start"
    );
    assert!(runtime_shell.special_boundary.is_none());

    let persisted = runtime_shell
        .shell
        .runtime()
        .load_save(&save_path)
        .expect("load Battle Tower SRAM-updated pre-entry quick-save");
    assert!(matches!(
        persisted.overworld,
        crate::core::state::OverworldMemory::Active { ref map_name, .. }
            if map_name == "BattleTower1F"
    ));
    assert_eq!(persisted.battle_tower.challenge_state, 2);
    assert_eq!(persisted.battle_tower.beaten_trainers, 4);

    load_visible_runtime_save(&mut runtime_shell, &save_path, "title_continue")
        .expect("continue after leaving the in-progress opponent battle");
    take_visible_deferred_script(&mut runtime_shell)
        .expect("enter the deferred left-without-saving script");
    assert_eq!(
        runtime_shell
            .shell
            .presentation_snapshot()
            .expect("left-without-saving scene snapshot")
            .script_events
            .pending_text_label
            .as_deref(),
        Some("Text_BattleTower_LeftWithoutSaving"),
        "resume cursor={:?}, boundary={:?}, action={:?}, text={:?}",
        runtime_shell.active_script_cursor,
        runtime_shell.special_boundary,
        runtime_shell.last_runtime_action,
        runtime_shell.shell.snapshot().unwrap().ui.text,
    );
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("finish Battle Tower left-without-saving cancellation");
    assert_eq!(runtime_shell.shell.current_map_name(), "BattleTower1F");
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .battle_tower
            .challenge_state,
        0
    );
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .battle_tower
            .beaten_trainers,
        0
    );

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_file(save_path.with_extension("crystalsave.bak"));
}

#[test]
fn photo_studio_without_a_printer_reports_the_source_failure_instead_of_success() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pending_script_party_selection = Some(PendingScriptPartySelection::PhotoStudio);
    runtime_shell.party_menu_open = true;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
        .expect("select a Pokemon for the Photo Studio");

    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("HoldStillText")
    );
    assert_eq!(
        runtime_shell
            .special_boundary_queue
            .iter()
            .map(|boundary| boundary.label.as_str())
            .collect::<Vec<_>>(),
        Vec::<&str>::new()
    );
    assert_eq!(runtime_shell.pending_photo_studio_commit, Some(0));
    assert!(
        runtime_shell
            .shell
            .snapshot()
            .expect("pre-print snapshot")
            .ui
            .active_pokemon_picture
            .is_none()
    );
    assert_eq!(runtime_shell.pending_special_sound, None);

    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge Hold Still");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PrinterError2")
    );
    assert!(runtime_shell.pending_photo_studio_commit.is_none());
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("printing snapshot")
            .ui
            .active_pokemon_picture
            .as_deref(),
        Some("CYNDAQUIL")
    );
    press_visible_a_button(&mut runtime_shell).expect("A is ignored by the printer error");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PrinterError2")
    );
    press_visible_b_button(&mut runtime_shell).expect("B cancels the disconnected print");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("NoPhotoText")
    );
    assert!(
        runtime_shell
            .shell
            .snapshot()
            .expect("post-printer snapshot")
            .ui
            .active_pokemon_picture
            .is_none()
    );
}

#[test]
fn photo_studio_rejects_an_egg_without_opening_the_picture_surface() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.is_egg = true;
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.pending_script_party_selection = Some(PendingScriptPartySelection::PhotoStudio);
    runtime_shell.party_menu_open = true;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
        .expect("select an Egg for the Photo Studio");

    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("EggPhotoText")
    );
    let source = runtime_shell
        .shell
        .text_snapshot("_EggPhotoText")
        .expect("exported Photo Studio Egg text");
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Photo Studio Egg snapshot");
    let expected_pages = render_visible_asm_text_pages(
        source
            .asm_text
            .as_deref()
            .expect("Photo Studio Egg ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .iter()
            .flat_map(|boundary| boundary.details.iter().cloned())
            .collect::<Vec<_>>(),
        expected_pages
    );
    assert!(runtime_shell.pending_photo_studio_commit.is_none());
    assert!(
        runtime_shell
            .shell
            .snapshot()
            .expect("Egg refusal snapshot")
            .ui
            .active_pokemon_picture
            .is_none()
    );
}

#[test]
fn photo_studio_special_prints_its_intro_before_opening_party_selection() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "CianwoodPhotoStudio".to_string(),
        tile: TilePosition::new(2, 4),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("CianwoodPhotoStudio", TilePosition::new(2, 4), 0)
        .expect("start Photo Studio session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("CianwoodPhotoStudioFishingGuruScript")
        .expect("compiled Photo Studio script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("PhotoStudio")
        })
        .expect("PhotoStudio special command");

    assert!(
        open_visible_script_party_selection_for_command(
            &mut runtime_shell,
            "CianwoodPhotoStudioFishingGuruScript",
            command_index,
        )
        .expect("intercept PhotoStudio special")
    );

    assert!(!runtime_shell.party_menu_open);
    assert!(matches!(
        runtime_shell.pending_script_party_selection,
        Some(PendingScriptPartySelection::PhotoStudio)
    ));
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("WhichMonPhotoText")
    );
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge Photo Studio intro");
    assert!(runtime_shell.party_menu_open);
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn poke_seer_special_prints_its_intro_before_opening_party_selection() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "PokeSeersHouse".to_string(),
        tile: TilePosition::new(2, 4),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("PokeSeersHouse", TilePosition::new(2, 4), 0)
        .expect("start Poke Seer's House session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("SeerScript")
        .expect("compiled Poke Seer script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("PokeSeer")
        })
        .expect("PokeSeer special command");

    assert!(
        open_visible_script_party_selection_for_command(
            &mut runtime_shell,
            "SeerScript",
            command_index,
        )
        .expect("intercept PokeSeer special")
    );

    assert!(!runtime_shell.party_menu_open);
    assert!(matches!(
        runtime_shell.pending_script_party_selection,
        Some(PendingScriptPartySelection::PokeSeer)
    ));
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("SeerSeeAllText")
    );
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Poke Seer presentation snapshot");
    let source = runtime_shell
        .shell
        .text_snapshot("_SeerSeeAllText")
        .expect("exported Poke Seer intro");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Poke Seer ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    let actual_pages = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(actual_pages, expected_pages);
    for _ in 0..actual_pages.len() {
        close_visible_special_boundary(&mut runtime_shell)
            .expect("acknowledge one Poke Seer intro page");
    }
    assert!(runtime_shell.party_menu_open);
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn name_rater_special_preserves_the_exported_intro_pages_before_its_prompt() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "GoldenrodNameRater".to_string(),
        tile: TilePosition::new(2, 4),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("GoldenrodNameRater", TilePosition::new(2, 4), 0)
        .expect("start Goldenrod Name Rater session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("GoldenrodNameRater")
        .expect("compiled Name Rater script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("NameRater")
        })
        .expect("NameRater special command");

    assert!(
        open_visible_script_party_selection_for_command(
            &mut runtime_shell,
            "GoldenrodNameRater",
            command_index,
        )
        .expect("intercept Name Rater special")
    );

    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Name Rater presentation snapshot");
    let source = runtime_shell
        .shell
        .text_snapshot("_NameRaterHelloText")
        .expect("exported Name Rater intro");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Name Rater ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    let mut actual_pages = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    actual_pages.push(
        runtime_shell
            .pc_notice
            .clone()
            .expect("Name Rater final yes/no page"),
    );
    assert_eq!(actual_pages, expected_pages);
    assert!(
        actual_pages.len() > 1,
        "the source intro has multiple input-gated pages"
    );
    assert!(runtime_shell.pc_confirmation.is_some());
    assert!(!runtime_shell.party_menu_open);

    for _ in 0..actual_pages.len() - 1 {
        close_visible_special_boundary(&mut runtime_shell)
            .expect("advance one Name Rater intro page");
    }
    assert!(runtime_shell.special_boundary.is_none());
    assert!(runtime_shell.pc_confirmation.is_some());
}

#[test]
fn name_rater_and_move_deleter_print_which_mon_before_party_selection() {
    for (pending, expected_label) in [
        (
            PendingScriptPartySelection::NameRater,
            "NameRaterWhichMonText",
        ),
        (
            PendingScriptPartySelection::MoveDeletion { party_index: None },
            "DeleterAskWhichMonText",
        ),
    ] {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::ScriptPartyIntro(pending));
        runtime_shell.yes_no_cursor = Some(MenuCursor {
            surface_id: "pc:confirmation".to_string(),
            option_index: 0,
        });

        resolve_visible_pc_confirmation(&mut runtime_shell, true)
            .expect("accept service introduction");

        assert!(!runtime_shell.party_menu_open);
        assert_eq!(
            runtime_shell
                .special_boundary
                .as_ref()
                .map(|boundary| boundary.label.as_str()),
            Some(expected_label)
        );
        close_visible_special_boundary(&mut runtime_shell)
            .expect("acknowledge which-Pokemon prompt");
        assert!(runtime_shell.party_menu_open);
        assert!(runtime_shell.special_boundary.is_none());
    }
}

#[test]
fn day_care_lady_intro_and_which_mon_prompt_preserve_exported_pages() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "DayCare".to_string(),
        tile: TilePosition::new(2, 7),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("DayCare", TilePosition::new(2, 7), 0)
        .expect("start Day-Care session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("DayCareLadyScript")
        .expect("compiled Day-Care lady script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("DayCareLady")
        })
        .expect("DayCareLady special command");

    assert!(
        open_visible_day_care_for_script_command(
            &mut runtime_shell,
            "DayCareLadyScript",
            command_index,
        )
        .expect("intercept Day-Care lady special")
    );

    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Day-Care presentation snapshot");
    let source = runtime_shell
        .shell
        .text_snapshot("_DayCareLadyIntroText")
        .expect("exported Day-Care lady intro");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Day-Care ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    let mut actual_pages = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    actual_pages.push(runtime_shell.pc_notice.clone().expect("final yes/no page"));
    assert_eq!(actual_pages, expected_pages);
    assert!(runtime_shell.shell.session().state().day_care.lady.active);

    for _ in 0..expected_pages.len() - 1 {
        close_visible_special_boundary(&mut runtime_shell).expect("advance Day-Care introduction");
    }
    resolve_visible_pc_confirmation(&mut runtime_shell, true).expect("accept Day-Care service");
    assert!(!runtime_shell.party_menu_open);
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("WhatShouldIRaiseText")
    );
}

#[test]
fn day_care_growth_and_fee_pages_resolve_exact_decimal_operands() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "DayCare".to_string(),
        tile: TilePosition::new(2, 7),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("DayCare", TilePosition::new(2, 7), 0)
        .expect("start Day-Care session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    let mut resident = runtime_shell.shell.session().state().storage.party.pokemon[0]
        .clone()
        .expect("party Pokemon");
    resident.nickname = "EMBER".to_string();
    let current_level = resident.level.saturating_add(2);
    resident.experience = crate::core::systems::experience::calculate_experience(
        runtime_shell.shell.runtime().growth_rates(),
        &resident.species.growth_rate,
        current_level,
    )
    .expect("derive resident current level from stored EXP");
    runtime_shell.shell.session_mut().state_mut().day_care.lady.pokemon = Some(resident);
    runtime_shell.shell.session_mut().state_mut().day_care.lady.active = true;
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("DayCareLadyScript")
        .expect("compiled Day-Care lady script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("DayCareLady")
        })
        .expect("DayCareLady special command");

    assert!(
        open_visible_day_care_for_script_command(
            &mut runtime_shell,
            "DayCareLadyScript",
            command_index,
        )
        .expect("open Day-Care withdrawal")
    );
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Day-Care presentation snapshot");
    let intro = runtime_shell
        .shell
        .text_snapshot("_AreWeGeniusesText")
        .expect("exported Day-Care growth intro");
    let intro_pages = render_visible_asm_text_pages(
        intro.asm_text.as_deref().expect("Day-Care ASM text"),
        &BTreeMap::from([("STRING_BUFFER_1".to_string(), "EMBER".to_string())]),
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    assert_eq!(
        runtime_shell.pc_notice.as_deref(),
        intro_pages.last().map(String::as_str)
    );

    resolve_visible_pc_confirmation(&mut runtime_shell, true)
        .expect("request Day-Care growth result");
    let mut named_buffers = snapshot.script_events.named_buffers.clone();
    named_buffers.insert("STRING_BUFFER_1".to_string(), "EMBER".to_string());
    named_buffers.insert("wStringBuffer2 + 1".to_string(), "2".to_string());
    named_buffers.insert("wStringBuffer2 + 2".to_string(), "300".to_string());
    let source = runtime_shell
        .shell
        .text_snapshot("_YourMonHasGrownText")
        .expect("exported Day-Care growth text");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Day-Care ASM text"),
        &named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    let mut actual_pages = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    actual_pages.push(runtime_shell.pc_notice.clone().expect("final price page"));
    assert_eq!(actual_pages, expected_pages);
    assert!(actual_pages.iter().all(|page| !page.contains("<DECIMAL:")));
}

#[test]
fn day_care_received_egg_holds_the_source_text_for_120_frames() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect = SpecialRoutineEffect::DayCareInteraction {
        caretaker: "man".to_string(),
        action: "collect_egg".to_string(),
        success: true,
        pokemon: Some("EGG".to_string()),
        level: Some(5),
        reason: None,
    };

    assert!(
        activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("activate received Egg result")
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("DayCareEgg")
    );
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge received Egg text");
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(120));
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("DayCareEggDelay")
    );

    for _ in 0..119 {
        assert!(
            advance_visible_special_text_pause(&mut runtime_shell)
                .expect("advance received Egg delay")
        );
    }
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(1));
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("DayCareEggDelay")
    );
    assert!(
        advance_visible_special_text_pause(&mut runtime_shell).expect("finish received Egg delay")
    );
    assert_eq!(runtime_shell.visible_special_text_pause_frames, None);
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("DayCareEggCareText")
    );
}

#[test]
fn linked_friend_success_uses_the_silent_source_50_frame_tail() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect = SpecialRoutineEffect::LinkedFriendResult {
        success: true,
        link_mode: 0,
        delay_frames: 50,
    };

    assert!(
        activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("activate linked-friend success")
    );
    assert!(runtime_shell.special_boundary.is_none());
    assert_eq!(
        runtime_shell.visible_internal_special_delay_frames,
        Some(50)
    );
}

#[test]
fn ordinary_link_results_do_not_open_a_host_diagnostic_boundary() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect = SpecialRoutineEffect::LinkResult {
        success: true,
        link_mode: 2,
        delay_frames: 0,
    };

    assert!(
        !activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("apply silent link result")
    );
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn link_room_request_waits_for_online_peer_without_a_diagnostic_modal() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect = SpecialRoutineEffect::LinkAction { action: 2, room: 2 };

    assert!(
        !activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("start online Colosseum matchmaking")
    );
    assert_eq!(runtime_shell.pending_link_room_selection, Some(2));
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn hosted_peer_readiness_completes_the_internal_wait_and_starts_source_tail() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pending_linked_friend_wait = true;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .link_session
        .serial_connection_status =
        crate::core::state::LinkSerialConnectionStatus::UsingInternalClock;

    complete_visible_linked_friend_wait(&mut runtime_shell)
        .expect("complete hosted linked-friend wait");

    assert!(!runtime_shell.pending_linked_friend_wait);
    assert_eq!(
        runtime_shell.visible_internal_special_delay_frames,
        Some(50)
    );
    assert!(runtime_shell.special_boundary.is_none());
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref(),
        Some("1")
    );
}

#[test]
fn online_link_room_session_blocks_only_on_the_hosted_trade_or_battle() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let session = SpecialRoutineEffect::LinkRoom {
        room: "TradeCenter".to_string(),
        link_mode: 2,
        session: true,
    };

    assert!(
        activate_visible_special_routine_boundary(&mut runtime_shell, &session)
            .expect("enter hosted Trade Center session")
    );
    assert!(runtime_shell.pending_link_room_session);
    assert!(runtime_shell.special_boundary.is_none());

    let entry = SpecialRoutineEffect::LinkRoom {
        room: "TimeCapsule".to_string(),
        link_mode: 1,
        session: false,
    };
    assert!(
        !activate_visible_special_routine_boundary(&mut runtime_shell, &entry)
            .expect("enter Time Capsule map")
    );
    assert!(!runtime_shell.pending_link_room_session);
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn cancelled_online_link_flow_releases_the_exact_authored_script_boundary() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.pending_linked_friend_wait = true;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .link_session
        .serial_connection_status = crate::core::state::LinkSerialConnectionStatus::NotEstablished;

    cancel_visible_online_link_flow(&mut runtime_shell)
        .expect("cancel hosted linked-friend wait through the source result");

    assert!(!runtime_shell.pending_linked_friend_wait);
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref(),
        Some("0")
    );
    assert!(
        runtime_shell
            .visible_internal_special_delay_frames
            .is_none()
    );

    runtime_shell.pending_link_room_session = true;
    cancel_visible_online_link_flow(&mut runtime_shell)
        .expect("cancel hosted room session through link-return");
    assert!(!runtime_shell.pending_link_room_session);
}

#[test]
fn cable_club_clock_owner_branch_is_silent() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect = SpecialRoutineEffect::CableClubCheckWhichChris {
        external_clock_player: true,
    };

    assert!(
        !activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("apply cable-club clock-owner branch")
    );
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn time_capsule_compatibility_result_is_silent() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect = SpecialRoutineEffect::TimeCapsuleCompatibility {
        result_code: 2,
        mon_name: Some("PIKACHU".to_string()),
        move_name: Some("SKETCH".to_string()),
    };

    assert!(
        !activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("apply Time Capsule compatibility result")
    );
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn timed_link_results_use_their_silent_source_frame_tail() {
    for delay_frames in [1, 6, 12, 40] {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let effect = SpecialRoutineEffect::LinkResult {
            success: true,
            link_mode: 0,
            delay_frames,
        };

        assert!(
            activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
                .expect("apply timed link result")
        );
        assert!(runtime_shell.special_boundary.is_none());
        assert_eq!(
            runtime_shell.visible_internal_special_delay_frames,
            Some(delay_frames)
        );
    }
}

#[test]
fn link_quick_save_uses_the_silent_source_30_frame_tail() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let save_path = std::env::temp_dir().join(format!(
        "crystal-link-quick-save-{}-{}.crystalsave",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    runtime_shell.quick_save_path = Some(save_path.clone());
    let effect = SpecialRoutineEffect::QuickSave {
        requested: true,
        delay_frames: 30,
    };

    assert!(
        activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("activate link quick-save")
    );
    assert!(save_path.is_file());
    assert!(runtime_shell.special_boundary.is_none());
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref(),
        Some("1")
    );
    assert_eq!(
        runtime_shell.visible_internal_special_delay_frames,
        Some(30)
    );
    let _ = std::fs::remove_file(save_path);
}

#[test]
fn link_quick_save_failure_returns_false_and_still_waits_30_frames() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.quick_save_path = None;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .script_value = Some("1".to_string());
    let effect = SpecialRoutineEffect::QuickSave {
        requested: true,
        delay_frames: 30,
    };

    assert!(
        activate_visible_special_routine_boundary(&mut runtime_shell, &effect)
            .expect("failed link quick-save returns through wScriptVar")
    );
    assert_eq!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref(),
        Some("0")
    );
    assert_eq!(
        runtime_shell.visible_internal_special_delay_frames,
        Some(30)
    );
}

#[test]
fn name_rater_preserves_better_name_choice_and_what_name_boundaries() {
    let prepare = || {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let player_id = runtime_shell.shell.session().state().player_id;
        let player_name = runtime_shell.shell.session().state().player_name.clone();
        let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
            .as_mut()
            .expect("party Pokemon");
        pokemon.nickname = "EMBER".to_string();
        pokemon.original_trainer_id = player_id;
        pokemon.original_trainer_name = player_name;
        runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
        runtime_shell.pending_script_party_selection = Some(PendingScriptPartySelection::NameRater);
        runtime_shell.party_menu_open = true;
        runtime_shell.party_cursor = 0;
        runtime_shell
    };

    let mut declined = prepare();
    resolve_visible_script_party_selection(&mut declined, Some(0)).expect("choose owned Pokemon");
    assert!(declined.pending_name_input.is_none());
    let snapshot = declined
        .shell
        .presentation_snapshot()
        .expect("Name Rater better-name snapshot");
    let mut buffers = snapshot.script_events.named_buffers.clone();
    buffers.insert("STRING_BUFFER_1".to_string(), "EMBER".to_string());
    let source = declined
        .shell
        .text_snapshot("_NameRaterBetterNameText")
        .expect("exported better-name text");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("better-name ASM text"),
        &buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    let mut actual_pages = declined
        .special_boundary
        .iter()
        .chain(declined.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    actual_pages.push(
        declined
            .pc_notice
            .clone()
            .expect("better-name final yes/no page"),
    );
    assert_eq!(actual_pages, expected_pages);
    assert!(declined.pc_confirmation.is_some());
    for _ in 0..actual_pages.len() - 1 {
        close_visible_special_boundary(&mut declined).expect("advance better-name source page");
    }
    resolve_visible_pc_confirmation(&mut declined, false).expect("decline renaming");
    assert_eq!(
        declined
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("NameRaterComeAgainText")
    );

    let mut accepted = prepare();
    resolve_visible_script_party_selection(&mut accepted, Some(0)).expect("choose owned Pokemon");
    while accepted.special_boundary.is_some() {
        close_visible_special_boundary(&mut accepted)
            .expect("advance accepted better-name source page");
    }
    resolve_visible_pc_confirmation(&mut accepted, true).expect("accept renaming");
    assert!(accepted.pending_name_input.is_none());
    assert_eq!(
        accepted
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("NameRaterWhatNameText")
    );
    close_visible_special_boundary(&mut accepted).expect("acknowledge What name");
    assert_eq!(
        accepted
            .pending_name_input
            .as_ref()
            .map(|input| input.label.as_str()),
        Some("POKéMON'S NAME?")
    );
}

#[test]
fn name_rater_completion_preserves_every_exported_printtext_page() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let player_id = runtime_shell.shell.session().state().player_id;
    let player_name = runtime_shell.shell.session().state().player_name.clone();
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.nickname = "EMBER".to_string();
    pokemon.original_trainer_id = player_id;
    pokemon.original_trainer_name = player_name;
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.party_menu_open = true;
    runtime_shell.party_cursor = 0;
    runtime_shell.pending_name_input = Some(PendingNameInput {
        label: "POKéMON'S NAME?".to_string(),
        value: "BLAZE".to_string(),
        max_length: 10,
        cursor_column: 0,
        cursor_row: 0,
        case: NameInputCase::Upper,
    });

    confirm_visible_player_name_input(&mut runtime_shell)
        .expect("confirm a different Name Rater nickname");

    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("renamed Pokemon snapshot");
    let mut expected_pages = Vec::new();
    for text_target in ["_NameRaterNamedText", "_NameRaterFinishedText"] {
        let source = runtime_shell
            .shell
            .text_snapshot(text_target)
            .expect("exported Name Rater completion text");
        expected_pages.extend(render_visible_asm_text_pages(
            source.asm_text.as_deref().expect("Name Rater ASM text"),
            &snapshot.script_events.named_buffers,
            &snapshot.trainer.player_name,
            visible_rival_name(&snapshot),
            snapshot.progression.time.day_of_week,
        ));
    }
    let actual_pages = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(actual_pages, expected_pages);
}

#[test]
fn move_deleter_prints_which_move_before_opening_the_move_list() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = vec![
        crate::core::models::LearnedMove {
            name: "TACKLE".to_string(),
            current_pp: 35,
            pp_ups: 0,
        },
        crate::core::models::LearnedMove {
            name: "LEER".to_string(),
            current_pp: 30,
            pp_ups: 0,
        },
    ];
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.pending_script_party_selection =
        Some(PendingScriptPartySelection::MoveDeletion { party_index: None });
    runtime_shell.party_menu_open = true;
    runtime_shell.party_cursor = 0;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
        .expect("choose Move Deleter Pokemon");

    assert!(runtime_shell.party_move_cursor.is_none());
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("DeleterAskWhichMoveText")
    );
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge which-move prompt");
    assert_eq!(
        runtime_shell
            .party_move_cursor
            .as_ref()
            .map(|cursor| cursor.surface_id.as_str()),
        Some("party:0:moves")
    );
}

#[test]
fn move_deleter_waits_before_and_after_the_deleted_move_sound() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = vec![
        crate::core::models::LearnedMove {
            name: "TACKLE".to_string(),
            current_pp: 35,
            pp_ups: 0,
        },
        crate::core::models::LearnedMove {
            name: "LEER".to_string(),
            current_pp: 30,
            pp_ups: 0,
        },
    ];
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::MoveDeletion {
        party_index: 0,
        move_index: 0,
    });
    runtime_shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "pc:confirmation".to_string(),
        option_index: 0,
    });
    runtime_shell.transient_audio_playing = true;

    resolve_visible_pc_confirmation(&mut runtime_shell, true).expect("confirm move deletion");

    assert!(runtime_shell.visible_wait_sfx_boundary);
    assert!(runtime_shell.special_boundary.is_none());
    assert_eq!(
        runtime_shell
            .pending_wait_play_sfx
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["SFX_MOVE_DELETED"]
    );
    assert!(
        !runtime_shell
            .pending_audio
            .iter()
            .any(|command| command.audio_id == "SFX_MOVE_DELETED")
    );

    runtime_shell.transient_audio_playing = false;
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("snapshot");
    advance_visible_wait_sfx_boundary(&mut runtime_shell, &snapshot, false)
        .expect("play deletion sound after initial wait");
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|command| command.audio_id == "SFX_MOVE_DELETED")
    );
    assert!(runtime_shell.special_boundary.is_none());

    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("snapshot");
    advance_visible_wait_sfx_boundary(&mut runtime_shell, &snapshot, false)
        .expect("finish deletion sound wait");
    assert!(!runtime_shell.visible_wait_sfx_boundary);
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("DeleterForgotMoveText")
    );
    let source = runtime_shell
        .shell
        .text_snapshot("_DeleterForgotMoveText")
        .expect("exported Move Deleter completion text");
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Move Deleter completion snapshot");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Move Deleter ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .iter()
            .flat_map(|boundary| boundary.details.iter().cloned())
            .collect::<Vec<_>>(),
        expected_pages
    );
}

#[test]
fn tmhm_replacement_rejects_hm_without_mutating_the_pokemon() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .add_bag_item("TM_HEADBUTT", 1)
        .expect("add a TM");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = ["CUT", "TACKLE", "LEER", "SMOKESCREEN"]
        .into_iter()
        .map(|name| crate::core::models::LearnedMove {
            name: name.to_string(),
            current_pp: 20,
            pp_ups: 0,
        })
        .collect();
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    let before = runtime_shell.shell.session().state().storage.party.pokemon[0]
        .clone()
        .expect("party Pokemon before refusal");
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::TmHm);
    runtime_shell.field_pack_target_mode = Some(FieldPackTargetMode::TmHmPokemon);
    runtime_shell.tmhm_cursor = Some(MenuCursor {
        surface_id: "bag:tmhm".to_string(),
        option_index: 0,
    });
    runtime_shell.party_cursor = 0;
    runtime_shell.tmhm_forget_menu_open = true;
    runtime_shell.party_move_cursor = Some(MenuCursor {
        surface_id: party_move_cursor_surface_id(0),
        option_index: 0,
    });

    let preview_error = runtime_shell
        .shell
        .preview_tmhm_on_party_pokemon("TM_HEADBUTT", 0, Some(0))
        .expect_err("the authoritative mutation rejects replacing CUT");
    assert!(matches!(
        preview_error.downcast_ref::<TmHmLearnError>(),
        Some(TmHmLearnError::CannotForgetHmMove { move_id }) if move_id == "CUT"
    ));

    confirm_visible_tmhm_target(&mut runtime_shell).expect("reject forgetting CUT");

    assert_eq!(
        runtime_shell.shell.session().state().storage.party.pokemon[0].as_ref(),
        Some(&before)
    );
    let source = runtime_shell
        .shell
        .text_snapshot("_MoveCantForgetHMText")
        .expect("exported HM refusal");
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("TM/HM presentation snapshot");
    let expected = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("HM refusal ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    assert_eq!(runtime_shell.field_notice.as_ref(), expected.first());
    assert!(runtime_shell.tmhm_forget_menu_open);
}

#[test]
fn tmhm_replacement_uses_source_text_pause_and_sound_boundaries() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .add_bag_item("TM_HEADBUTT", 1)
        .expect("add a TM");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = ["TACKLE", "LEER", "SMOKESCREEN", "EMBER"]
        .into_iter()
        .map(|name| crate::core::models::LearnedMove {
            name: name.to_string(),
            current_pp: 20,
            pp_ups: 0,
        })
        .collect();
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::TmHm);
    runtime_shell.field_pack_target_mode = Some(FieldPackTargetMode::TmHmPokemon);
    runtime_shell.tmhm_cursor = Some(MenuCursor {
        surface_id: "bag:tmhm".to_string(),
        option_index: 0,
    });
    runtime_shell.party_cursor = 0;
    runtime_shell.tmhm_forget_menu_open = true;
    runtime_shell.party_move_cursor = Some(MenuCursor {
        surface_id: party_move_cursor_surface_id(0),
        option_index: 0,
    });

    teach_selected_tmhm_on(&mut runtime_shell, 0).expect("replace TACKLE with HEADBUTT");

    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("Text_1_2_and_Poof")
    );
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(30));
    assert_eq!(
        runtime_shell
            .special_boundary_queue
            .front()
            .map(|boundary| boundary.label.as_str()),
        Some("MoveForgotPoofText")
    );
}

#[test]
fn tmhm_forget_menu_renders_the_source_cancel_row() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .add_bag_item("TM_HEADBUTT", 1)
        .expect("add a TM");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = ["TACKLE", "LEER", "SMOKESCREEN", "EMBER"]
        .into_iter()
        .map(|name| crate::core::models::LearnedMove {
            name: name.to_string(),
            current_pp: 20,
            pp_ups: 0,
        })
        .collect();
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::TmHm);
    runtime_shell.field_pack_target_mode = Some(FieldPackTargetMode::TmHmPokemon);
    runtime_shell.tmhm_cursor = Some(MenuCursor {
        surface_id: "bag:tmhm".to_string(),
        option_index: 0,
    });
    runtime_shell.party_cursor = 0;
    runtime_shell.tmhm_forget_menu_open = true;
    runtime_shell.party_move_cursor = Some(MenuCursor {
        surface_id: party_move_cursor_surface_id(0),
        option_index: 4,
    });

    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("TM/HM presentation snapshot");
    let entries = visible_field_pack_entries(&snapshot, &runtime_shell)
        .expect("source-valid TM/HM CANCEL row");

    assert!(
        entries.iter().any(|entry| entry == ">CANCEL"),
        "{entries:?}"
    );
}

#[test]
fn tmhm_refusal_and_full_moves_decision_use_exported_source_text() {
    fn configured_tm_shell(item_id: &str) -> BevyRuntimeShell {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        runtime_shell
            .shell
            .add_bag_item(item_id, 1)
            .expect("add a TM");
        runtime_shell.field_pack_pocket = Some(FieldPackPocket::TmHm);
        runtime_shell.field_pack_target_mode = Some(FieldPackTargetMode::TmHmPokemon);
        runtime_shell.tmhm_cursor = Some(MenuCursor {
            surface_id: "bag:tmhm".to_string(),
            option_index: 0,
        });
        runtime_shell.party_cursor = 0;
        runtime_shell
    }

    fn expected_pages(
        runtime_shell: &BevyRuntimeShell,
        text_target: &str,
        nickname: &str,
        move_id: &str,
    ) -> Vec<String> {
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("TM/HM presentation snapshot");
        let mut named_buffers = snapshot.script_events.named_buffers.clone();
        named_buffers.insert("wMonOrItemNameBuffer".to_string(), nickname.to_string());
        named_buffers.insert("STRING_BUFFER_1".to_string(), nickname.to_string());
        named_buffers.insert("STRING_BUFFER_2".to_string(), move_id.to_string());
        let source = runtime_shell
            .shell
            .text_snapshot(text_target)
            .expect("exported TM/HM text");
        render_visible_asm_text_pages(
            source.asm_text.as_deref().expect("TM/HM ASM text"),
            &named_buffers,
            &snapshot.trainer.player_name,
            visible_rival_name(&snapshot),
            snapshot.progression.time.day_of_week,
        )
    }

    for (kind, item_id, move_id, text_target) in [
        (
            "incompatible",
            "TM_DYNAMICPUNCH",
            "DYNAMICPUNCH",
            "_TMHMNotCompatibleText",
        ),
        ("known", "TM_HEADBUTT", "HEADBUTT", "_KnowsMoveText"),
        ("full", "TM_HEADBUTT", "HEADBUTT", "_AskForgetMoveText"),
    ] {
        let mut runtime_shell = configured_tm_shell(item_id);
        let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
            .as_mut()
            .expect("party Pokemon");
        match kind {
            "incompatible" => {}
            "known" => pokemon.moves.push(crate::core::models::LearnedMove {
                name: "HEADBUTT".to_string(),
                current_pp: 5,
                pp_ups: 0,
            }),
            "full" => {
                pokemon.moves = ["TACKLE", "LEER", "SMOKESCREEN", "EMBER"]
                    .into_iter()
                    .map(|name| crate::core::models::LearnedMove {
                        name: name.to_string(),
                        current_pp: 20,
                        pp_ups: 0,
                    })
                    .collect();
            }
            _ => unreachable!(),
        }
        let nickname = pokemon.nickname.clone();
        runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();

        confirm_visible_tmhm_target(&mut runtime_shell).expect("resolve TM/HM target");

        let actual = runtime_shell
            .field_notice
            .iter()
            .chain(runtime_shell.field_notice_queue.iter())
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            expected_pages(&runtime_shell, text_target, &nickname, move_id,),
            "{kind} branch",
        );
        if kind == "full" {
            assert!(runtime_shell.tmhm_decision_prompt_cursor.is_none());
            assert_eq!(
                runtime_shell.pending_tmhm_text_stage,
                Some(VisibleTmHmTextStage::Decision(
                    VisibleTmHmDecision::ForgetMove,
                ))
            );
        }
    }
}

#[test]
fn tmhm_boot_and_teach_prompt_preserve_exported_source_pages() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell
        .shell
        .add_bag_item("TM_DYNAMICPUNCH", 1)
        .expect("add a TM");
    runtime_shell.field_pack_pocket = Some(FieldPackPocket::TmHm);
    runtime_shell.tmhm_cursor = Some(MenuCursor {
        surface_id: "bag:tmhm".to_string(),
        option_index: 0,
    });

    open_visible_tmhm_teach_prompt(&mut runtime_shell).expect("boot the selected TM");

    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("TM/HM presentation snapshot");
    let move_id = snapshot.bag.tm_hm[0]
        .move_id
        .as_deref()
        .expect("TM move id");
    let mut named_buffers = snapshot.script_events.named_buffers.clone();
    named_buffers.insert("STRING_BUFFER_2".to_string(), move_id.replace('_', " "));
    let render = |text_target: &str| {
        let source = runtime_shell
            .shell
            .text_snapshot(text_target)
            .expect("exported TM/HM text");
        render_visible_asm_text_pages(
            source.asm_text.as_deref().expect("TM/HM ASM text"),
            &named_buffers,
            &snapshot.trainer.player_name,
            visible_rival_name(&snapshot),
            snapshot.progression.time.day_of_week,
        )
    };
    let expected_boot = render("_BootedTMText");
    let expected_contained = render("_ContainedMoveText");
    assert_eq!(runtime_shell.field_notice.as_ref(), expected_boot.first());

    let boot_page = runtime_shell.field_notice.clone().expect("boot notice");
    runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: boot_page.clone(),
        page_index: 0,
        visible_chars: boot_page.chars().count(),
        frames_until_next_char: 0,
    });
    press_visible_a_button(&mut runtime_shell).expect("advance the boot text");

    let actual_contained = runtime_shell
        .field_notice
        .iter()
        .chain(runtime_shell.field_notice_queue.iter())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(actual_contained, expected_contained);
    assert!(runtime_shell.tmhm_teach_prompt_cursor.is_none());

    for _ in 0..expected_contained.len() {
        let page = runtime_shell
            .field_notice
            .clone()
            .expect("current contained-move page");
        runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
            text: page.clone(),
            page_index: 0,
            visible_chars: page.chars().count(),
            frames_until_next_char: 0,
        });
        press_visible_a_button(&mut runtime_shell).expect("advance contained-move text");
    }
    assert_eq!(
        runtime_shell.field_notice.as_ref(),
        expected_contained.last()
    );
    assert!(runtime_shell.tmhm_teach_prompt_cursor.is_some());

    runtime_shell.tmhm_teach_prompt_cursor = Some(MenuCursor {
        surface_id: "pack:tmhm:teach-prompt".to_string(),
        option_index: 1,
    });
    let final_page = runtime_shell
        .field_notice
        .clone()
        .expect("teach question beneath yes/no");
    runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: final_page.clone(),
        page_index: 0,
        visible_chars: final_page.chars().count(),
        frames_until_next_char: 0,
    });
    press_visible_a_button(&mut runtime_shell).expect("decline teaching the TM");
    assert!(runtime_shell.field_notice.is_none());
    assert!(runtime_shell.tmhm_teach_prompt_cursor.is_none());
    assert_eq!(runtime_shell.field_pack_pocket, Some(FieldPackPocket::TmHm));
}

#[test]
fn move_tutor_known_incompatible_and_stop_branches_use_exported_text() {
    fn source_pages(
        runtime_shell: &BevyRuntimeShell,
        text_target: &str,
        nickname: &str,
        move_id: &str,
    ) -> Vec<String> {
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("Move Tutor presentation snapshot");
        let mut named_buffers = snapshot.script_events.named_buffers.clone();
        named_buffers.insert("wMonOrItemNameBuffer".to_string(), nickname.to_string());
        named_buffers.insert("STRING_BUFFER_1".to_string(), nickname.to_string());
        named_buffers.insert("STRING_BUFFER_2".to_string(), move_id.replace('_', " "));
        let source = runtime_shell
            .shell
            .text_snapshot(text_target)
            .expect("exported Move Tutor text");
        render_visible_asm_text_pages(
            source.asm_text.as_deref().expect("Move Tutor ASM text"),
            &named_buffers,
            &snapshot.trainer.player_name,
            visible_rival_name(&snapshot),
            snapshot.progression.time.day_of_week,
        )
    }

    for (move_id, known, text_target, expected_label) in [
        ("FLAMETHROWER", true, "_KnowsMoveText", "KnowsMoveText"),
        (
            "ICE_BEAM",
            false,
            "_TMHMNotCompatibleText",
            "TMHMNotCompatibleText",
        ),
    ] {
        let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
            .as_mut()
            .expect("party Pokemon");
        if known {
            pokemon.moves.push(crate::core::models::LearnedMove {
                name: move_id.to_string(),
                current_pp: 15,
                pp_ups: 0,
            });
        }
        let nickname = pokemon.nickname.clone();
        runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
        runtime_shell.pending_script_party_selection =
            Some(PendingScriptPartySelection::MoveTutor {
                move_id: move_id.to_string(),
                party_index: None,
            });
        runtime_shell.party_menu_open = true;

        resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
            .expect("resolve Move Tutor refusal");

        let actual = runtime_shell
            .special_boundary
            .iter()
            .chain(runtime_shell.special_boundary_queue.iter())
            .flat_map(|boundary| boundary.details.iter().cloned())
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            source_pages(&runtime_shell, text_target, &nickname, move_id)
        );
        assert!(
            runtime_shell
                .special_boundary
                .as_ref()
                .is_some_and(|boundary| boundary.label == expected_label)
        );
    }

    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let nickname = runtime_shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("party Pokemon")
        .nickname
        .clone();
    runtime_shell.pc_confirmation = Some(VisiblePcConfirmation::MoveTutorStop {
        move_id: "FLAMETHROWER".to_string(),
        party_index: 0,
    });
    runtime_shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "pc:confirmation".to_string(),
        option_index: 0,
    });

    resolve_visible_pc_confirmation(&mut runtime_shell, true)
        .expect("stop learning the Move Tutor move");

    let actual = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        source_pages(
            &runtime_shell,
            "_DidNotLearnMoveText",
            &nickname,
            "FLAMETHROWER",
        )
    );
}

#[test]
fn move_tutor_four_move_prompt_preserves_every_exported_source_page() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = ["TACKLE", "LEER", "SMOKESCREEN", "EMBER"]
        .into_iter()
        .map(|name| crate::core::models::LearnedMove {
            name: name.to_string(),
            current_pp: 20,
            pp_ups: 0,
        })
        .collect();
    let nickname = pokemon.nickname.clone();
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.pending_script_party_selection = Some(PendingScriptPartySelection::MoveTutor {
        move_id: "FLAMETHROWER".to_string(),
        party_index: None,
    });
    runtime_shell.party_menu_open = true;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
        .expect("select the four-move Pokemon");

    let mut named_buffers = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Move Tutor snapshot")
        .script_events
        .named_buffers;
    named_buffers.insert("wMonOrItemNameBuffer".to_string(), nickname);
    named_buffers.insert("STRING_BUFFER_2".to_string(), "FLAMETHROWER".to_string());
    let source = runtime_shell
        .shell
        .text_snapshot("_AskForgetMoveText")
        .expect("exported Move Tutor prompt");
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("Move Tutor presentation snapshot");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Move Tutor ASM text"),
        &named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    let mut actual_pages = runtime_shell
        .special_boundary
        .iter()
        .chain(runtime_shell.special_boundary_queue.iter())
        .flat_map(|boundary| boundary.details.iter().cloned())
        .collect::<Vec<_>>();
    actual_pages.push(
        runtime_shell
            .pc_notice
            .clone()
            .expect("final source page under yes/no prompt"),
    );
    assert_eq!(actual_pages, expected_pages);
    assert_eq!(
        runtime_shell.pc_confirmation,
        Some(VisiblePcConfirmation::MoveTutorForget {
            move_id: "FLAMETHROWER".to_string(),
            party_index: 0,
        })
    );

    for _ in 0..expected_pages.len() - 1 {
        close_visible_special_boundary(&mut runtime_shell)
            .expect("advance one Move Tutor prompt page");
    }
    resolve_visible_pc_confirmation(&mut runtime_shell, true).expect("agree to forget a move");
    let move_list_prompt = runtime_shell
        .shell
        .text_snapshot("_MoveAskForgetText")
        .expect("exported move-list prompt");
    let move_list_pages = render_visible_asm_text_pages(
        move_list_prompt
            .asm_text
            .as_deref()
            .expect("Move Tutor move-list ASM text"),
        &named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    assert_eq!(runtime_shell.pc_notice.as_ref(), move_list_pages.first());
    assert!(runtime_shell.party_move_cursor.is_some());
}

#[test]
fn move_tutor_replacement_plays_each_sound_at_its_source_text_boundary() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.moves = ["TACKLE", "LEER", "SMOKESCREEN", "EMBER"]
        .into_iter()
        .map(|name| crate::core::models::LearnedMove {
            name: name.to_string(),
            current_pp: 20,
            pp_ups: 0,
        })
        .collect();
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.pending_script_party_selection = Some(PendingScriptPartySelection::MoveTutor {
        move_id: "FLAMETHROWER".to_string(),
        party_index: Some(0),
    });
    runtime_shell.party_menu_open = true;
    runtime_shell.party_move_cursor = Some(MenuCursor {
        surface_id: party_move_cursor_surface_id(0),
        option_index: 0,
    });
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
        .expect("replace the selected move");

    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("Text_1_2_and_Poof")
    );
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(30));
    assert!(runtime_shell.pending_audio.is_empty());

    for _ in 0..29 {
        advance_visible_special_text_pause(&mut runtime_shell).expect("advance the count pause");
    }
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(1));
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("Text_1_2_and_Poof")
    );
    advance_visible_special_text_pause(&mut runtime_shell).expect("finish the count pause");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("MoveForgotPoofText")
    );
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(
        runtime_shell.pending_audio[0].audio_id,
        "SFX_SWITCH_POKEMON"
    );

    runtime_shell.pending_audio.clear();
    for _ in 0..29 {
        advance_visible_special_text_pause(&mut runtime_shell).expect("advance the poof pause");
    }
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(1));
    advance_visible_special_text_pause(&mut runtime_shell).expect("finish the poof pause");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("MoveForgotText")
    );
    assert!(runtime_shell.pending_audio.is_empty());

    close_visible_special_boundary(&mut runtime_shell)
        .expect("acknowledge the Pokemon-forgot source page");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("MoveForgotText")
    );
    assert!(runtime_shell.pending_audio.is_empty());

    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge the source And page");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("LearnedMoveText")
    );
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(
        runtime_shell.pending_audio[0].audio_id,
        "SFX_DEX_FANFARE_50_79"
    );
}

#[test]
fn magikarp_length_special_prints_the_measurement_before_the_map_branch() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let species = runtime_shell.shell.runtime().data().pokemon["MAGIKARP"].clone();
    let mut magikarp = crate::core::models::Pokemon::new_for_tests(
        species,
        10,
        crate::core::models::Dv::default(),
    );
    magikarp.original_trainer_id = runtime_shell.shell.session().state().player_id;
    magikarp.original_trainer_name = runtime_shell.shell.session().state().player_name.clone();
    runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0] = Some(magikarp);
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    runtime_shell.pending_script_party_selection =
        Some(PendingScriptPartySelection::CheckMagikarpLength);
    runtime_shell.party_menu_open = true;
    runtime_shell.party_cursor = 0;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(0))
        .expect("measure selected Magikarp");

    let formatted = runtime_shell
        .shell
        .session().state()
        .script_runtime
        .named_buffers["STRING_BUFFER_1"]
        .clone();
    let boundary = runtime_shell
        .special_boundary
        .as_ref()
        .expect("measurement text boundary");
    assert_eq!(boundary.label, "MagikarpGuruMeasureText");
    assert_eq!(
        boundary.details,
        vec![
            "Let me measure\nthat MAGIKARP.".to_string(),
            format!("…Hm, it measures\n{formatted}."),
        ]
    );
}

#[test]
fn print_diploma_shows_the_diploma_then_requires_b_to_cancel_the_printer_error() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let effect =
        crate::core::systems::special_routines::SpecialRoutineEffect::RuntimeVisualCommand {
            kind: crate::core::state::ScriptGraphicsRuntimeKind::PrintDiploma,
        };

    assert!(
        activate_visible_special_routine_boundary(&mut runtime_shell, &effect,)
            .expect("activate diploma printing")
    );
    assert!(runtime_shell.visible_diploma.is_some());
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("PrinterError2")
    );

    press_visible_a_button(&mut runtime_shell).expect("A is ignored by printer status");
    assert!(runtime_shell.visible_diploma.is_some());
    assert!(runtime_shell.special_boundary.is_some());
    press_visible_b_button(&mut runtime_shell).expect("B cancels diploma printing");
    assert!(runtime_shell.visible_diploma.is_none());
    assert!(runtime_shell.special_boundary.is_none());
}

#[test]
fn poke_seer_preserves_the_asm_high_byte_ot_check_and_complete_advice() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().player_id = 0x1201;
    let pokemon = runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("party Pokemon");
    pokemon.nickname = "EMBER".to_string();
    pokemon.original_trainer_id = 0x12ff;
    pokemon.original_trainer_name = "OTHER".to_string();
    pokemon.level = 80;
    pokemon.caught_data = Some(crate::core::models::pokemon::CaughtData {
        level: 10,
        time_of_day: Some(crate::core::world::encounters::TimeOfDay::Day),
        original_trainer_gender: 0,
        location: 1,
    });
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("snapshot");
    let pokemon = snapshot.party.slots[0].pokemon.clone();

    let messages = visible_poke_seer_messages(&runtime_shell, &snapshot, &pokemon, "EMBER")
        .expect("render met Poke Seer messages");
    assert_eq!(messages[0].label, "SeerNameLocationText");
    assert_eq!(messages.last().expect("advice").label, "SeerMightyText");
    let caught_location = snapshot
        .presentation
        .pokegear_landmarks
        .landmarks
        .iter()
        .find(|landmark| landmark.id == 1)
        .expect("caught landmark")
        .name
        .clone();
    let source_buffers = BTreeMap::from([
        ("wSeerNickname".to_string(), "EMBER".to_string()),
        ("wSeerOT".to_string(), "OTHER".to_string()),
        ("wSeerCaughtLocation".to_string(), caught_location),
        ("wSeerCaughtLevelString".to_string(), "10".to_string()),
        ("wSeerTimeOfDay".to_string(), "Day".to_string()),
    ]);
    let mut expected_pages = Vec::new();
    for text_target in [
        "_SeerNameLocationText",
        "_SeerTimeLevelText",
        "_SeerMightyText",
    ] {
        let source = runtime_shell
            .shell
            .text_snapshot(text_target)
            .expect("exported Poke Seer branch text");
        expected_pages.extend(render_visible_asm_text_pages(
            source.asm_text.as_deref().expect("Poke Seer ASM text"),
            &source_buffers,
            &snapshot.trainer.player_name,
            visible_rival_name(&snapshot),
            snapshot.progression.time.day_of_week,
        ));
    }
    let actual_pages = messages
        .iter()
        .flat_map(|message| message.details.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(actual_pages, expected_pages);
    assert!(
        messages
            .last()
            .expect("advice")
            .details
            .join("\n")
            .contains("It looks brimming\nwith confidence.")
    );

    let mut traded = pokemon.clone();
    traded.original_trainer_id = 0x3401;
    let messages = visible_poke_seer_messages(&runtime_shell, &snapshot, &traded, "EMBER")
        .expect("render traded Poke Seer messages");
    assert_eq!(messages[0].label, "SeerTradeText");
    let traded_text = messages
        .iter()
        .flat_map(|message| message.details.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(traded_text.contains("came from OTHER"));
    assert!(!traded_text.contains("<RAM:"));

    let mut level_only = pokemon.clone();
    level_only
        .caught_data
        .as_mut()
        .expect("caught data")
        .location = 0x7f;
    let messages = visible_poke_seer_messages(&runtime_shell, &snapshot, &level_only, "EMBER")
        .expect("render event-location Poke Seer messages");
    assert_eq!(messages[0].label, "SeerNoLocationText");
    assert!(
        messages
            .iter()
            .all(|message| message.label != "SeerTimeLevelText")
    );
    assert_eq!(messages.last().expect("advice").label, "SeerMightyText");

    let mut egg = pokemon.clone();
    egg.is_egg = true;
    let messages = visible_poke_seer_messages(&runtime_shell, &snapshot, &egg, "EGG")
        .expect("render Egg Poke Seer messages");
    assert!(
        messages
            .iter()
            .all(|message| message.label == "SeerEggText")
    );
    let source = runtime_shell
        .shell
        .text_snapshot("_SeerEggText")
        .expect("exported Poke Seer Egg text");
    assert_eq!(
        messages
            .iter()
            .flat_map(|message| message.details.iter().cloned())
            .collect::<Vec<_>>(),
        render_visible_asm_text_pages(
            source.asm_text.as_deref().expect("Poke Seer ASM text"),
            &snapshot.script_events.named_buffers,
            &snapshot.trainer.player_name,
            visible_rival_name(&snapshot),
            snapshot.progression.time.day_of_week,
        )
    );

    let mut mighty = pokemon;
    mighty.level = 100;
    mighty.caught_data.as_mut().expect("caught data").level = 1;
    let messages = visible_poke_seer_messages(&runtime_shell, &snapshot, &mighty, "EMBER")
        .expect("render mighty Poke Seer messages");
    let advice = messages
        .iter()
        .filter(|message| message.label == "SeerImpressedText")
        .flat_map(|message| message.details.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(messages.last().expect("advice").label, "SeerImpressedText");
    assert!(advice.contains("seeing EMBER"), "{advice:?}");
    assert!(advice.contains("in battle would\nexcite anyone."));

    let mut zero_caught_data = mighty;
    zero_caught_data.caught_data = Some(crate::core::models::pokemon::CaughtData {
        level: 0,
        time_of_day: None,
        original_trainer_gender: 0,
        location: 0,
    });
    let messages =
        visible_poke_seer_messages(&runtime_shell, &snapshot, &zero_caught_data, "EMBER")
            .expect("render unknown Poke Seer messages");
    assert!(
        messages
            .iter()
            .all(|message| message.label == "SeerCantTellAThingText")
    );
    let source = runtime_shell
        .shell
        .text_snapshot("_SeerCantTellAThingText")
        .expect("exported Poke Seer unknown text");
    let expected_pages = render_visible_asm_text_pages(
        source.asm_text.as_deref().expect("Poke Seer ASM text"),
        &snapshot.script_events.named_buffers,
        &snapshot.trainer.player_name,
        visible_rival_name(&snapshot),
        snapshot.progression.time.day_of_week,
    );
    assert_eq!(
        messages
            .iter()
            .flat_map(|message| message.details.iter().cloned())
            .collect::<Vec<_>>(),
        expected_pages
    );
}

#[test]
fn npc_trade_dialogs_preserve_the_asm_species_buffers_and_gender_suffix() {
    let girl = crystal_assets::NpcTradeRule {
        dialog_set: "TRADE_DIALOGSET_GIRL".to_string(),
        requested_species: "DRAGONAIR".to_string(),
        offered_species: "DODRIO".to_string(),
        gender_requirement: "TRADE_GENDER_FEMALE".to_string(),
        ..Default::default()
    };
    assert_eq!(
        visible_npc_trade_intro_text(&girl),
        "DRAGONAIR♀'s cute, but I don't have it.\nDo you have DRAGONAIR♀?\nWant to trade it for my DODRIO?"
    );
    assert_eq!(
        visible_npc_trade_result_text(&girl, "2"),
        "Wow! Thank you!\nI always wanted DRAGONAIR♀!"
    );
    assert_eq!(
        visible_completed_npc_trade_text(&girl),
        "How is that DODRIO I traded you doing?\n\nYour DRAGONAIR♀'s so cute!"
    );

    let happy = crystal_assets::NpcTradeRule {
        dialog_set: "TRADE_DIALOGSET_HAPPY".to_string(),
        requested_species: "KRABBY".to_string(),
        offered_species: "VOLTORB".to_string(),
        gender_requirement: "TRADE_GENDER_EITHER".to_string(),
        ..Default::default()
    };
    assert_eq!(
        visible_completed_npc_trade_text(&happy),
        "Hi! The KRABBY you traded me is doing great!"
    );
}

#[test]
fn successful_npc_trade_waits_for_the_cable_prompt_before_exchanging_pokemon() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "GoldenrodDeptStore5F".to_string(),
        tile: TilePosition::new(10, 3),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("GoldenrodDeptStore5F", TilePosition::new(10, 3), 0)
        .expect("start Mike's trade map session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    runtime_shell
        .shell
        .add_party_pokemon(
            "ABRA",
            10,
            None,
            None,
            "AB",
            1,
            crate::core::models::Dv::from_non_hp(10, 10, 10, 10),
        )
        .expect("add requested Abra");
    let command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("Mike")
        .expect("compiled Mike trade script")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("trade")
        })
        .expect("Mike trade command");
    runtime_shell.pending_script_party_selection = Some(PendingScriptPartySelection::NpcTrade {
        origin_map_name: "GoldenrodDeptStore5F".to_string(),
        source_script: "Mike".to_string(),
        command_index,
        trade_id: "NPC_TRADE_MIKE".to_string(),
    });
    runtime_shell.party_menu_open = true;
    runtime_shell.party_cursor = 1;

    resolve_visible_script_party_selection(&mut runtime_shell, Some(1))
        .expect("select requested Abra");

    let snapshot = runtime_shell.shell.snapshot().expect("pre-cable snapshot");
    assert!(snapshot.script_events.completed_trades.is_empty());
    assert!(
        snapshot
            .party
            .slots
            .iter()
            .any(|slot| slot.pokemon.species.id == "ABRA")
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("NPCTradeCableText")
    );

    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge cable prompt");
    let snapshot = runtime_shell.shell.snapshot().expect("post-trade snapshot");
    assert_eq!(
        snapshot.script_events.completed_trades,
        vec!["NPC_TRADE_MIKE"]
    );
    assert!(
        snapshot
            .party
            .slots
            .iter()
            .any(|slot| slot.pokemon.species.id == "MACHOP")
    );
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("Text_NPCTraded")
    );
    close_visible_special_boundary(&mut runtime_shell).expect("acknowledge traded-for text");
    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("NPCTradeCompleteText1")
    );
}

include!("menu_and_input_battle_tower.rs");
