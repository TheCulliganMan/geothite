#[test]
fn native_gameplay_tick_journals_vblank_before_the_injected_rtc_sample() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.shell.set_runtime_journal_enabled(true);
    runtime_shell
        .shell
        .set_game_timer_counting(true)
        .expect("arm FinishContinue game timer");
    let state_before = runtime_shell.shell.session().state().clone();
    let retained_before = runtime_shell.shell.retained_runtime_commands().len();
    let sample = RuntimeRtcSample {
        date: GameDate::new(2000, 1, 2),
        hour: 13,
        minute: 14,
        second: 15,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(NativeRtcSource::fixed(sample))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(RuntimeTickTimer::new(0.0))
        .add_systems(Update, apply_keyboard_input);

    app.update();

    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(runtime_shell.last_error, None);
    assert_eq!(
        runtime_shell.shell.session().state().time.registers.hours,
        13
    );
    assert_eq!(
        runtime_shell.shell.session().state().time.registers.minutes,
        14
    );
    assert_eq!(
        runtime_shell.shell.session().state().time.registers.seconds,
        15
    );
    assert_eq!(
        runtime_shell.shell.session().state().time.game_time_frames,
        1
    );
    let timer_frame = &runtime_shell.shell.retained_runtime_commands()[retained_before];
    let timer_command =
        crystal_assets::decode_runtime_mutation_command_frame(timer_frame, &state_before)
            .expect("decode native VBlank command against the pre-tick state");
    let crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(timer_command) =
        timer_command
    else {
        panic!("native tick must journal a VBlank batch first");
    };
    assert_eq!(timer_command.vblanks, 1);
    assert_eq!(timer_command.normal_divider_trace.samples.len(), 2);

    let mut state_after_vblank = state_before;
    let mut divider = crystal_core::random::ReplayDivider::new(
        timer_command.normal_divider_trace.samples.iter().copied(),
    );
    let mut rng =
        crystal_core::random::CrystalRandom::new(state_after_vblank.random_state, &mut divider);
    rng.random(false).expect("replay native VBlank_Normal");
    state_after_vblank.random_state = rng.state();
    state_after_vblank.vblank_counter = state_after_vblank.vblank_counter.wrapping_add(1);
    state_after_vblank.advance_game_timer_vblank();
    let clock_frame = &runtime_shell.shell.retained_runtime_commands()[retained_before + 1];
    let command =
        crystal_assets::decode_runtime_mutation_command_frame(clock_frame, &state_after_vblank)
            .expect("decode native RTC command against the post-VBlank state");
    let crystal_assets::RuntimeMutationCommand::UpdateClockFromDatetime(command) = command else {
        panic!("native tick must journal its RTC sample after GameTimer and before input");
    };
    assert_eq!(
        (command.date, command.hour, command.minute, command.second),
        (sample.date, sample.hour, sample.minute, sample.second)
    );
}

#[test]
fn vblank_play_timer_counts_presentation_held_text_menu_and_battle_frames() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .set_game_timer_counting(true);
    let overworld_frame = runtime_shell.shell.session().state().frame_counter;
    runtime_shell.field_notice = Some("HELD TEXT".to_string());
    runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: "HELD TEXT".to_string(),
        page_index: 0,
        visible_chars: 0,
        frames_until_next_char: 1,
    });
    let mut app = integrated_shell_test_app(runtime_shell);

    app.update();
    {
        let runtime_shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            runtime_shell.shell.session().state().time.game_time_frames,
            1
        );
        assert_eq!(
            runtime_shell.shell.session().state().frame_counter,
            overworld_frame
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.field_notice = None;
        runtime_shell.field_text_reveal = None;
        runtime_shell.start_menu_cursor = Some(MenuCursor {
            surface_id: "start".to_string(),
            option_index: 0,
        });
    }
    app.update();
    {
        let runtime_shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            runtime_shell.shell.session().state().time.game_time_frames,
            2
        );
        assert_eq!(
            runtime_shell.shell.session().state().frame_counter,
            overworld_frame
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.start_menu_cursor = None;
        runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
            frame: 0,
            stronger_enemy: false,
            cave_environment: false,
            trainer_battle: true,
        });
    }
    app.update();
    {
        let runtime_shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            runtime_shell.shell.session().state().time.game_time_frames,
            3
        );
        assert_eq!(
            runtime_shell.shell.session().state().frame_counter,
            overworld_frame
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell
            .shell
            .session_mut()
            .state_mut()
            .set_game_logic_paused(true);
    }
    app.update();
    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(
        runtime_shell.shell.session().state().time.game_time_frames,
        3
    );
    assert_eq!(
        runtime_shell.shell.session().state().frame_counter,
        overworld_frame
    );
}

#[test]
fn native_title_main_menu_keeps_game_timer_counting_clear() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    let title = runtime_shell.title_menu.as_mut().expect("title menu");
    title
        .presentation_machine
        .memory
        .insert("wJumptableIndex".to_string(), 0x82);
    title
        .presentation_machine
        .memory
        .insert("wTitleScreenSelectedOption".to_string(), 0);
    title
        .presentation_machine
        .memory
        .insert("hSCX".to_string(), 0);
    let mut app = integrated_shell_test_app(runtime_shell);

    app.update();

    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert!(!runtime_shell.shell.session().state().game_timer_counting);
    assert_eq!(
        runtime_shell.shell.session().state().time.game_time_frames,
        0
    );
}

#[test]
fn native_catch_up_journals_one_exact_batched_game_timer_command() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.shell.set_runtime_journal_enabled(true);
    runtime_shell
        .shell
        .set_game_timer_counting(true)
        .expect("arm FinishContinue game timer");
    runtime_shell.field_notice = Some("HELD TEXT".to_string());
    runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: "HELD TEXT".to_string(),
        page_index: 0,
        visible_chars: 0,
        frames_until_next_char: 1,
    });
    let state_before = runtime_shell.shell.session().state().clone();
    let retained_before = runtime_shell.shell.retained_runtime_commands().len();
    let mut app = integrated_shell_test_app(runtime_shell);
    {
        let mut timer = app.world_mut().resource_mut::<RuntimeTickTimer>();
        timer.step_seconds = 999.0;
        timer.finished_vblanks = 120;
        timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    }

    app.update();

    let overworld_frame = {
        let runtime_shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            runtime_shell.shell.session().state().time.game_time_seconds,
            2
        );
        assert_eq!(
            runtime_shell.shell.session().state().time.game_time_frames,
            0
        );
        let retained = &runtime_shell.shell.retained_runtime_commands()[retained_before..];
        assert_eq!(retained.len(), 1);
        let command =
            crystal_assets::decode_runtime_mutation_command_frame(&retained[0], &state_before)
                .expect("decode batched catch-up VBlank command against pre-update state");
        let crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(command) = command
        else {
            panic!("catch-up tick must journal one VBlank batch");
        };
        assert_eq!(command.vblanks, 120);
        assert_eq!(command.normal_divider_trace.samples.len(), 240);
        runtime_shell.shell.session().state().frame_counter
    };
    {
        let timer = app.world().resource::<RuntimeTickTimer>();
        assert_eq!(timer.finished_ticks, 0);
        assert_eq!(timer.presentation_ticks, 0);
    }
    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.field_notice = None;
        runtime_shell.field_text_reveal = None;
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .state()
            .frame_counter,
        overworld_frame,
        "the capped input tick must be discarded while the modal owns the update"
    );
}

#[test]
fn battle_transition_consumes_every_bounded_catch_up_tick() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: 0,
        stronger_enemy: false,
        cave_environment: true,
        trainer_battle: false,
    });
    let mut timer = RuntimeTickTimer::new(999.0);
    timer.finished_vblanks = MAX_RUNTIME_CATCH_UP_TICKS;
    timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(native_rtc_source_for_test())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(timer)
        .add_systems(Update, apply_keyboard_input);

    app.update();

    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .visible_battle_transition
            .expect("battle transition remains active")
            .frame,
        MAX_RUNTIME_CATCH_UP_TICKS as u16,
        "battle entry must retain its 60 Hz duration when rendering falls below 60 FPS"
    );

    let terminal_frame = {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let transition = runtime_shell
            .visible_battle_transition
            .as_mut()
            .expect("battle transition remains active");
        let terminal_frame = visible_battle_transition_total_frames(transition) - 1;
        transition.frame = terminal_frame - 1;
        terminal_frame
    };
    {
        let mut timer = app.world_mut().resource_mut::<RuntimeTickTimer>();
        timer.finished_vblanks = MAX_RUNTIME_CATCH_UP_TICKS;
        timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .visible_battle_transition
            .expect("terminal black frame must be presented before battle setup")
            .frame,
        terminal_frame,
        "catch-up must not skip the transition's terminal black presentation"
    );
}

#[test]
fn retained_animations_consume_every_bounded_catch_up_tick() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.visible_fishing_animation = Some(VisibleFishingAnimation {
        phase: VisibleFishingPhase::Cast,
        frame: 10,
        facing_up: false,
        bite: false,
        starts_battle: false,
    });
    let mut timer = RuntimeTickTimer::new(999.0);
    timer.finished_vblanks = MAX_RUNTIME_CATCH_UP_TICKS;
    timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(native_rtc_source_for_test())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(timer)
        .add_systems(Update, apply_keyboard_input);

    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .visible_fishing_animation
            .expect("fishing cast remains active")
            .frame,
        15
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.visible_fishing_animation = None;
        runtime_shell.visible_heal_machine = Some(VisibleHealMachine {
            kind: 0,
            party_count: 1,
            frame: 1,
        });
    }
    {
        let mut timer = app.world_mut().resource_mut::<RuntimeTickTimer>();
        timer.finished_vblanks = MAX_RUNTIME_CATCH_UP_TICKS;
        timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .visible_heal_machine
            .as_ref()
            .expect("heal animation remains active")
            .frame,
        6
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.visible_heal_machine = None;
        runtime_shell.battle_hp_tween = Some(VisibleBattleHpTween {
            player_hp: 10,
            player_target_hp: 15,
            player_max_hp: 64,
            player_pixels: 10,
            player_target_pixels: 15,
            player_frames_until_step: 0,
            enemy_pixels: 20,
            enemy_target_pixels: 20,
            enemy_frames_until_step: 0,
        });
    }
    {
        let mut timer = app.world_mut().resource_mut::<RuntimeTickTimer>();
        timer.finished_vblanks = MAX_RUNTIME_CATCH_UP_TICKS;
        timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    }
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .battle_hp_tween
            .as_ref()
            .expect("HP tween remains retained")
            .player_pixels,
        13,
        "the two-frame HP pixel cadence must consume all five elapsed frames"
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.battle_hp_tween = None;
        runtime_shell.pending_field_notice_effect_frames = Some(32);
        runtime_shell.visible_map_name_sign = Some(VisibleMapNameSign {
            landmark: "TEST".to_string(),
            label: "TEST".to_string(),
            frames_remaining: 10,
        });
    }
    {
        let mut timer = app.world_mut().resource_mut::<RuntimeTickTimer>();
        timer.finished_vblanks = MAX_RUNTIME_CATCH_UP_TICKS;
        timer.finished_ticks = MAX_RUNTIME_CATCH_UP_TICKS;
    }
    app.update();
    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(runtime_shell.pending_field_notice_effect_frames, Some(27));
    assert_eq!(
        runtime_shell
            .visible_map_name_sign
            .as_ref()
            .expect("map sign remains visible")
            .frames_remaining,
        5
    );
}

#[test]
fn map_name_sign_render_key_tracks_visibility_without_redrawing_countdown_frames() {
    let mut shell = core_modular_title_shell_for_test();
    shell.visible_map_name_sign = Some(VisibleMapNameSign {
        landmark: "TEST".to_string(),
        label: "TEST".to_string(),
        frames_remaining: 60,
    });
    let hidden_key = shell_render_key(&shell);
    advance_visible_map_name_sign(&mut shell.visible_map_name_sign, 1);
    assert_eq!(shell_render_key(&shell), hidden_key);
    advance_visible_map_name_sign(&mut shell.visible_map_name_sign, 1);
    let visible_key = shell_render_key(&shell);
    assert_ne!(visible_key, hidden_key, "showing the route sign must bypass retained-world rendering");
    advance_visible_map_name_sign(&mut shell.visible_map_name_sign, 58);
    assert_eq!(shell_render_key(&shell), visible_key);
    advance_visible_map_name_sign(&mut shell.visible_map_name_sign, 1);
    assert_ne!(shell_render_key(&shell), visible_key);
}

#[test]
fn map_name_sign_show_boundary_invalidates_the_idle_renderer() {
    let mut sign = Some(VisibleMapNameSign {
        landmark: "TEST".to_string(),
        label: "TEST".to_string(),
        // PlaceMapNameSign's old value 59 initializes the text and exposes
        // the window after decrementing the WRAM timer to 58.
        frames_remaining: 59,
    });
    assert!(
        advance_visible_map_name_sign(&mut sign, 1),
        "the 59-to-58 window-show boundary must invalidate the idle renderer"
    );
    assert_eq!(
        sign
            .as_ref()
            .expect("map sign becomes visible")
            .frames_remaining,
        58
    );

    assert!(
        !advance_visible_map_name_sign(&mut sign, 1),
        "an already-visible countdown frame does not alter the retained surface"
    );

    sign.as_mut().expect("map sign remains retained").frames_remaining = 1;
    assert!(
        !advance_visible_map_name_sign(&mut sign, 1),
        "the old-1 pass leaves the window visible with timer zero"
    );
    assert_eq!(
        sign.as_ref()
            .expect("timer zero still owns the final visible frame")
            .frames_remaining,
        0
    );
    assert!(
        advance_visible_map_name_sign(&mut sign, 1),
        "the following old-zero pass hides the window"
    );
    assert!(sign.is_none());
}

#[test]
fn modal_early_return_does_not_replay_an_already_consumed_vblank_next_update() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .set_game_timer_counting(true);
    runtime_shell.field_notice = Some("HELD TEXT".to_string());
    runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: "HELD TEXT".to_string(),
        page_index: 0,
        visible_chars: 0,
        frames_until_next_char: 1,
    });
    let mut timer = RuntimeTickTimer::new(999.0);
    timer.finished_vblanks = 1;
    timer.finished_ticks = 1;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(native_rtc_source_for_test())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(timer)
        .add_systems(Update, apply_keyboard_input);

    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .state()
            .time
            .game_time_frames,
        1
    );

    // Both budgets were consumed atomically before the modal return. The
    // following update has neither an old VBlank nor queued input work.
    app.update();
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .state()
            .time
            .game_time_frames,
        1
    );

    let overworld_frame = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .state()
        .frame_counter;
    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.field_notice = None;
        runtime_shell.field_text_reveal = None;
    }
    app.update();
    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(
        runtime_shell.shell.session().state().frame_counter,
        overworld_frame,
        "closing a modal must not execute input frames queued while it was held"
    );
}

#[test]
fn game_timer_batch_preserves_large_count_and_caps_in_one_command() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.shell.set_runtime_journal_enabled(true);
    runtime_shell
        .shell
        .set_game_timer_counting(true)
        .expect("arm FinishContinue game timer");
    let state_before = runtime_shell.shell.session().state().clone();
    let retained_before = runtime_shell.shell.retained_runtime_commands().len();

    let outcome = runtime_shell
        .shell
        .advance_game_timer_vblanks(u32::MAX)
        .expect("advance one large exact VBlank batch");

    assert!(outcome.counted);
    assert_eq!(
        (
            outcome.hours,
            outcome.minutes,
            outcome.seconds,
            outcome.frames
        ),
        (999, 59, 59, 0)
    );
    let retained = &runtime_shell.shell.retained_runtime_commands()[retained_before..];
    assert_eq!(retained.len(), 1);
    assert_eq!(
        crystal_assets::decode_runtime_mutation_command_frame(&retained[0], &state_before)
            .expect("decode large batched VBlank command"),
        crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(
            crystal_assets::RuntimeGameTimerAdvanceCommand {
                vblanks: u32::MAX,
                normal_divider_trace: crystal_assets::RuntimeDividerTrace::new([]),
            },
        )
    );
    assert!(
        runtime_shell
            .shell
            .advance_game_timer_vblanks(0)
            .unwrap_err()
            .to_string()
            .contains("nonzero VBlank count")
    );
    assert_eq!(
        runtime_shell.shell.retained_runtime_commands().len(),
        retained_before + 1,
        "a rejected zero batch must not enter the journal"
    );
}

#[test]
fn normal_vblank_batch_records_div_and_replays_rng_with_the_timer() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.shell.set_runtime_journal_enabled(true);
    *runtime_shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay([0x12, 0x34, 0x56, 0x78]);
    let state_before = runtime_shell.shell.session().state().clone();
    let retained_before = runtime_shell.shell.retained_runtime_commands().len();

    runtime_shell
        .shell
        .advance_vblanks(2, 2)
        .expect("advance two VBlank_Normal frames");
    assert_eq!(
        runtime_shell.shell.session().state().vblank_counter,
        state_before.vblank_counter.wrapping_add(2),
    );

    let command_frame = &runtime_shell.shell.retained_runtime_commands()[retained_before];
    let command =
        crystal_assets::decode_runtime_mutation_command_frame(command_frame, &state_before)
            .expect("decode VBlank_Normal command against pre-update state");
    assert_eq!(
        command,
        crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(
            crystal_assets::RuntimeGameTimerAdvanceCommand {
                vblanks: 2,
                normal_divider_trace: crystal_assets::RuntimeDividerTrace::new([
                    0x12, 0x34, 0x56, 0x78,
                ]),
            },
        )
    );

    let mut expected_state = state_before.clone();
    let mut divider = crystal_core::random::ReplayDivider::new([0x12, 0x34, 0x56, 0x78]);
    let mut rng =
        crystal_core::random::CrystalRandom::new(expected_state.random_state, &mut divider);
    rng.random(false).expect("first VBlank_Normal Random");
    rng.random(false).expect("second VBlank_Normal Random");
    expected_state.random_state = rng.state();
    expected_state.advance_game_timer_vblanks(2);
    assert_eq!(
        runtime_shell.shell.session().state().random_state,
        expected_state.random_state,
    );

    let mut replay = core_modular_title_shell_for_test();
    replay.intro_screen = None;
    replay.title_menu = None;
    *replay.shell.session_mut().state_mut() = state_before;
    *replay.shell.session_mut().divider_mut_for_tests() = crystal_core::random::RuntimeDividerSource::replay([]);
    replay
        .shell
        .apply_runtime_command_frame(command_frame)
        .expect("replay recorded VBlank_Normal batch without host DIV");
    assert_eq!(
        replay.shell.session().state().random_state,
        expected_state.random_state,
    );
}

#[test]
fn battle_transition_vblank_uses_cutscene_handler_without_advancing_rng() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.shell.set_runtime_journal_enabled(true);
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: 0,
        stronger_enemy: false,
        cave_environment: false,
        trainer_battle: false,
    });
    *runtime_shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay([]);
    let state_before = runtime_shell.shell.session().state().clone();
    let retained_before = runtime_shell.shell.retained_runtime_commands().len();
    let sample = RuntimeRtcSample {
        date: GameDate::new(2000, 1, 1),
        hour: 12,
        minute: 0,
        second: 0,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(NativeRtcSource::fixed(sample))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(RuntimeTickTimer::new(0.0))
        .add_systems(Update, apply_keyboard_input);

    app.update();

    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(runtime_shell.last_error, None);
    assert_eq!(
        runtime_shell.shell.session().state().random_state,
        state_before.random_state,
    );
    assert_eq!(
        runtime_shell.shell.session().state().vblank_counter,
        state_before.vblank_counter,
        "VBlank_Cutscene does not increment hVBlankCounter"
    );
    let command = crystal_assets::decode_runtime_mutation_command_frame(
        &runtime_shell.shell.retained_runtime_commands()[retained_before],
        &state_before,
    )
    .expect("decode battle-transition VBlank command");
    let crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(command) = command else {
        panic!("battle transition must still advance the VBlank timer");
    };
    assert_eq!(command.vblanks, 1);
    assert!(command.normal_divider_trace.samples.is_empty());
}

#[test]
fn only_battle_anim_engine_frames_replace_vblank_normal() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;

    runtime_shell.visible_frontpic_animation = Some(VisibleFrontpicAnimation {
        species_id: "UNOWN".to_string(),
        speed: 1,
        pointer: 0,
        repeat: 0,
        wait: 0,
        frame: 0,
    });
    assert!(
        !visible_special_vblank_handler_active(&runtime_shell),
        "AnimateFrontpic runs under VBlank_Normal"
    );
    runtime_shell.visible_frontpic_animation = None;
    runtime_shell.visible_trainer_exit_animation = Some(VisibleTrainerExitAnimation {
        side: crate::core::battle::turn::BattleSide::Enemy,
        frame: 0,
        send_out_after: false,
    });
    assert!(
        !visible_special_vblank_handler_active(&runtime_shell),
        "SlideBattlePicOut runs under VBlank_Normal"
    );
    runtime_shell.visible_trainer_exit_animation = None;
    runtime_shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
        side: crate::core::battle::turn::BattleSide::Enemy,
        frame: 0,
        shiny: false,
    });
    assert!(
        visible_special_vblank_handler_active(&runtime_shell),
        "ANIM_SEND_OUT_MON installs VBlank_Cutscene"
    );
}

#[test]
fn trainer_card_phase_reads_hvblankcounter_not_gameplay_frame() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.shell.session_mut().state_mut().frame_counter = 63;
    runtime_shell.shell.session_mut().state_mut().vblank_counter = 16;

    open_visible_trainer_card(&mut runtime_shell).expect("open Trainer Card at VBlank 16");

    assert!(!runtime_shell.trainer_card_colon_visible);
    assert_eq!(runtime_shell.trainer_card_colon_ticks, 16);
}

#[test]
fn unown_puzzle_cursor_blinks_from_hvblankcounter_unless_holding_piece() {
    let mut puzzle = VisibleUnownPuzzle {
        puzzle_id: "hooh".to_string(),
        layout: [[0; 6]; 6],
        holding_piece: None,
        cursor_x: 0,
        cursor_y: 0,
        solved: false,
    };
    assert!(!visible_unown_puzzle_cursor_visible(&puzzle, 0x0f));
    assert!(visible_unown_puzzle_cursor_visible(&puzzle, 0x10));
    puzzle.holding_piece = Some(1);
    assert!(visible_unown_puzzle_cursor_visible(&puzzle, 0));
    puzzle.holding_piece = None;
    puzzle.solved = true;
    assert!(!visible_unown_puzzle_cursor_visible(&puzzle, 0x10),
        "the solved puzzle calls ClearSprites before waiting for A or B");
}

fn core_modular_title_shell_for_test() -> BevyRuntimeShell {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::Title {
            spawn_identifier,
            save_path: None,
        },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize title shell")
}

#[test]
fn dontrestartmapmusic_is_not_auto_consumed_before_map_reload() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .session_mut().state_mut()
        .script_runtime
        .map_music_restart_disabled = true;

    let snapshot = runtime_shell.shell.snapshot().expect("runtime snapshot");
    assert_eq!(visible_auto_runtime_flag(&snapshot), None);

    runtime_shell
        .shell
        .session_mut().state_mut()
        .script_runtime
        .map_music_requested = true;
    let snapshot = runtime_shell.shell.snapshot().expect("runtime snapshot");
    assert_eq!(
        visible_auto_runtime_flag(&snapshot),
        Some(RuntimeScriptRuntimeFlag::MapMusicRequested)
    );
    assert!(snapshot.script_events.map_music_restart_disabled);
}

#[test]
fn retained_fullscreen_lcd_survives_title_setup_and_hands_off_to_complete_overworld() {
    let mut app = integrated_shell_test_app(core_modular_title_shell_for_test());

    app.update();
    let retained = retained_fullscreen_surface(app.world_mut());
    let intro_image_count = app.world().resource::<Assets<Image>>().len();
    for _ in 0..4 {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            tick_visible_intro_screen(&mut runtime_shell).expect("advance intro LCD");
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
        assert_eq!(
            app.world().resource::<Assets<Image>>().len(),
            intro_image_count,
            "intro frames must update the retained image instead of accumulating textures"
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        finish_and_drain_visible_intro_for_test(&mut runtime_shell, "retained-lcd-regression")
            .expect("handoff intro to title");
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let title = runtime_shell.title_menu.as_mut().expect("title menu");
        title
            .presentation_machine
            .memory
            .insert("hSCX".to_string(), 0);
        title
            .presentation_machine
            .values
            .insert("title_suicune_frame".to_string(), 0);
        title
            .presentation_machine
            .memory
            .insert("wTitleScreenTimer".to_string(), 10_000);
        title
            .presentation_machine
            .memory
            .insert("wJumptableIndex".to_string(), 2);
        title
            .presentation_machine
            .memory
            .insert("wTitleScreenTimer".to_string(), 10_000);
    }
    for _ in 0..40 {
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    let (bounded_title_cache, bounded_title_images) = {
        let world = app.world();
        let rendered_art = world.resource::<RenderedTilesetArt>();
        let settled_keys = rendered_art
            .title_screen_cache
            .keys()
            .filter(|key| key.scx == 0 && key.show_version_window)
            .collect::<Vec<_>>();
        assert!(
            settled_keys.len() <= 4,
            "settled title animation must cache only its four Suicune frames"
        );
        assert!(
            settled_keys
                .iter()
                .all(|key| matches!(key.frame, 0 | 8 | 16 | 24))
        );
        (
            rendered_art.title_screen_cache.len(),
            world.resource::<Assets<Image>>().len(),
        )
    };
    for _ in 0..40 {
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    assert_eq!(
        app.world()
            .resource::<RenderedTilesetArt>()
            .title_screen_cache
            .len(),
        bounded_title_cache,
        "settled title cache must be modulo its finite animation cycle"
    );
    assert_eq!(
        app.world().resource::<Assets<Image>>().len(),
        bounded_title_images,
        "repeating settled title animation must not allocate more GPU images"
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        open_visible_title_main_menu(&mut runtime_shell).expect("open title main menu");
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    let main_menu_image_count = app.world().resource::<Assets<Image>>().len();
    for _ in 0..24 {
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
        assert_eq!(
            app.world().resource::<Assets<Image>>().len(),
            main_menu_image_count,
            "main-menu cursor/fade redraws must consume their transient frame"
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.title_menu = None;
        open_visible_gender_selection(&mut runtime_shell).expect("open gender screen");
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    for _ in 0..VISIBLE_GENDER_FADE_IN_FRAMES {
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    for delta in [1, -1] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            move_visible_gender_selection(&mut runtime_shell, delta).expect("move gender cursor");
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    let gender_image_count = app.world().resource::<Assets<Image>>().len();
    for delta in [1, -1] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            move_visible_gender_selection(&mut runtime_shell, delta)
                .expect("repeat gender cursor state");
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    assert_eq!(
        app.world().resource::<Assets<Image>>().len(),
        gender_image_count,
        "revisiting cached gender cursor states must not allocate images"
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.pending_gender_selection = None;
        open_visible_time_set_screen(&mut runtime_shell, VisibleTimeSetNext::OakIntro)
            .expect("open time-set screen");
        let time_set = runtime_shell.pending_time_set.as_mut().expect("time set");
        time_set.phase = VisibleTimeSetPhase::HourConfirm;
        time_set.visible_chars = visible_time_set_dialog_text(time_set).chars().count();
        time_set.text_timer = 0;
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    for _ in 0..2 {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            move_visible_time_set_direction(&mut runtime_shell, VisibleTimeSetDirection::Right)
                .expect("toggle time confirmation cursor");
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    let time_image_count = app.world().resource::<Assets<Image>>().len();
    for _ in 0..2 {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            move_visible_time_set_direction(&mut runtime_shell, VisibleTimeSetDirection::Left)
                .expect("repeat time confirmation cursor");
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    assert_eq!(
        app.world().resource::<Assets<Image>>().len(),
        time_image_count,
        "revisiting cached clock cursor states must not allocate images"
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.pending_time_set = None;
        open_visible_oak_intro_sequence(&mut runtime_shell).expect("open Oak intro");
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let oak_intro = runtime_shell.pending_oak_intro.as_mut().expect("Oak intro");
        oak_intro.scene_phase = VisibleOakIntroPhase::Text;
        oak_intro.fade_active = false;
        oak_intro.current_text = "HELLO!".to_string();
        oak_intro.visible_chars = oak_intro.current_text.chars().count();
        oak_intro.waiting_for_input = true;
        oak_intro.blink_timer = 30;
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell
            .pending_oak_intro
            .as_mut()
            .expect("Oak intro")
            .blink_timer = 0;
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    let oak_image_count = app.world().resource::<Assets<Image>>().len();
    for blink_timer in [30, 0] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime_shell
                .pending_oak_intro
                .as_mut()
                .expect("Oak intro")
                .blink_timer = blink_timer;
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    assert_eq!(
        app.world().resource::<Assets<Image>>().len(),
        oak_image_count,
        "revisiting cached Oak blink states must not allocate images"
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.pending_oak_intro = None;
        open_visible_player_name_input(&mut runtime_shell).expect("open naming screen");
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    for move_cursor in [
        move_visible_player_name_cursor_right as fn(&mut BevyRuntimeShell) -> Result<()>,
        move_visible_player_name_cursor_left,
    ] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            move_cursor(&mut runtime_shell).expect("move naming cursor");
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    let naming_image_count = app.world().resource::<Assets<Image>>().len();
    for move_cursor in [
        move_visible_player_name_cursor_right as fn(&mut BevyRuntimeShell) -> Result<()>,
        move_visible_player_name_cursor_left,
    ] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            move_cursor(&mut runtime_shell).expect("repeat naming cursor state");
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    assert_eq!(
        app.world().resource::<Assets<Image>>().len(),
        naming_image_count,
        "revisiting cached naming cursor states must not allocate images"
    );

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.pending_name_input = None;
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    app.update();
    {
        let world = app.world_mut();
        let mut presenters = world.query_filtered::<Entity, With<VisibleIntroSurface>>();
        assert_eq!(
            presenters.iter(world).count(),
            1,
            "the retained LCD must cover the deferred frame that stages the replacement map layers",
        );
        assert!(
            world
                .resource::<RenderedTilesetArt>()
                .presented_fullscreen_release_pending
        );
    }
    app.update();
    {
        let world = app.world_mut();
        let mut presenters = world.query_filtered::<Entity, With<VisibleIntroSurface>>();
        assert_eq!(
            presenters.iter(world).count(),
            0,
            "the retained LCD must release once both deferred map layers are query-visible",
        );
        let surfaces = retained_map_surface_pair(world);
        assert_base_map_surface_is_fully_opaque(world, &surfaces);
    }
    assert!(
        app.world()
            .resource::<RenderedTilesetArt>()
            .intro_presented_surface
            .is_some(),
        "the retained image allocation should remain ready for the next full-screen sequence"
    );
}

#[test]
fn cold_fullscreen_to_field_handoff_waits_until_both_map_layers_are_query_visible() {
    let mut app = integrated_shell_test_app(core_modular_title_shell_for_test());
    app.update();
    let retained = retained_fullscreen_surface(app.world_mut());
    {
        let world = app.world_mut();
        let mut map_layers = world.query_filtered::<Entity, With<PlayfieldTile>>();
        assert_eq!(
            map_layers.iter(world).count(),
            0,
            "intro fixture must begin with no staged field surface"
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.intro_screen = None;
        runtime_shell.title_menu = None;
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    app.update();
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    {
        let world = app.world_mut();
        let surfaces = retained_map_surface_pair(world);
        assert_base_map_surface_is_fully_opaque(world, &surfaces);
    }

    app.update();
    let world = app.world_mut();
    let mut presenters = world.query_filtered::<Entity, With<VisibleIntroSurface>>();
    assert_eq!(
        presenters.iter(world).count(),
        0,
        "the presenter may retire only on an update that begins with both map layers query-visible"
    );
    let surfaces = retained_map_surface_pair(world);
    assert_base_map_surface_is_fully_opaque(world, &surfaces);
}

#[test]
fn credits_redraws_reuse_one_presenter_one_image_and_one_decoded_source_bundle() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    open_visible_credits_screen(&mut runtime_shell, true).expect("open credits");
    let mut app = integrated_shell_test_app(runtime_shell);

    app.update();
    let retained = retained_fullscreen_surface(app.world_mut());
    let (source_address, source_shape, image_count) = {
        let world = app.world();
        let rendered_art = world.resource::<RenderedTilesetArt>();
        let sources = rendered_art
            .credits_sources
            .as_ref()
            .expect("credits decoded source bundle");
        assert_eq!(rendered_art.credits_source_error, None);
        (
            std::ptr::from_ref(sources).addr(),
            (
                sources.palette_sets.len(),
                sources.mon_frames.len(),
                sources.border_tiles.len(),
                sources.font.levels.len(),
                sources.copyright_tiles.len(),
                sources.the_end_levels.len(),
            ),
            world.resource::<Assets<Image>>().len(),
        )
    };

    for _ in 0..48 {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            tick_visible_credits_screen(&mut runtime_shell);
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
        assert_eq!(
            app.world().resource::<Assets<Image>>().len(),
            image_count,
            "animated credits frames must be consumed into the retained image"
        );
    }

    let rendered_art = app.world().resource::<RenderedTilesetArt>();
    let sources = rendered_art
        .credits_sources
        .as_ref()
        .expect("credits source bundle must remain cached");
    assert_eq!(
        std::ptr::from_ref(sources).addr(),
        source_address,
        "credits animation must reuse the original decoded source bundle"
    );
    assert_eq!(
        (
            sources.palette_sets.len(),
            sources.mon_frames.len(),
            sources.border_tiles.len(),
            sources.font.levels.len(),
            sources.copyright_tiles.len(),
            sources.the_end_levels.len(),
        ),
        source_shape
    );
    assert_eq!(rendered_art.credits_source_error, None);
}

#[test]
fn field_fullscreen_owner_reuses_presenter_and_releases_only_after_map_is_staged() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.visible_slot_machine = Some(VisibleSlotMachine {
        phase: VisibleSlotMachinePhase::Betting,
        animation: VisibleSlotMachineAnimation::None,
        yes_no_index: 0,
        bet: 1,
        coins: 1234,
        payout: 0,
        offsets: [14; 3],
        spin_ticks: [0; 3],
        spinning: [false; 3],
        next_reel: 1,
        actor: None,
        secondary_actor: None,
        background_y_offset: 0,
        windows: [
            ["CHERRY".into(), "SEVEN".into(), "SQUIRTLE".into()],
            ["PIKACHU".into(), "SEVEN".into(), "STARYU".into()],
            ["PIKACHU".into(), "SEVEN".into(), "PIKACHU".into()],
        ],
        message: "BET 1".to_string(),
    });
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut app = integrated_shell_test_app(runtime_shell);

    app.update();
    let retained = retained_fullscreen_surface(app.world_mut());
    assert!(
        app.world()
            .resource::<RenderedTilesetArt>()
            .slot_machine_sources
            .is_some(),
        "slot renderer must retain its decoded source art"
    );
    assert_eq!(
        app.world()
            .resource::<RenderedTilesetArt>()
            .slot_machine_source_error,
        None
    );

    for bet in [2, 1] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let machine = runtime_shell
                .visible_slot_machine
                .as_mut()
                .expect("slot machine open");
            machine.bet = bet;
            machine.message = format!("BET {bet}");
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    let bounded_image_count = app.world().resource::<Assets<Image>>().len();
    for bet in [2, 1, 2, 1, 2, 1] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let machine = runtime_shell
                .visible_slot_machine
                .as_mut()
                .expect("slot machine open");
            machine.bet = bet;
            machine.message = format!("BET {bet}");
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
        assert_eq!(
            app.world().resource::<Assets<Image>>().len(),
            bounded_image_count,
            "slot animation/control redraws must not accumulate full-screen image assets"
        );
    }

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.visible_slot_machine = None;
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    app.update();
    let world = app.world_mut();
    let mut presenters = world.query_filtered::<Entity, With<VisibleIntroSurface>>();
    assert_eq!(
        presenters.iter(world).count(),
        0,
        "closing the last full-LCD field owner must release its presenter"
    );
    let surfaces = retained_map_surface_pair(world);
    assert_base_map_surface_is_fully_opaque(world, &surfaces);
}

#[test]
fn retained_field_fullscreen_ownership_distinguishes_new_game_and_capture_name_choices() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell.pokedex_scripted_entry = true;
    assert!(
        !retained_field_fullscreen_active(&runtime_shell),
        "the sticky scripted-entry flag alone must not cover the battle with a stale Dex LCD"
    );

    runtime_shell.pending_name_choice = Some(VisibleNameChoice {
        nickname_pages: VecDeque::new(),
        options: vec!["YES".to_string(), "NO".to_string()],
        selected: 0,
        player_menu: None,
        player_phase: None,
        motion_step: 0,
        motion_frames_remaining: 0,
        pending_player_name: None,
    });
    assert!(
        retained_field_fullscreen_active(&runtime_shell),
        "the new-game preset-name menu must own a complete LCD background"
    );

    runtime_shell.pending_standard_capture = Some(PendingStandardCapture {
        outcome: crate::core::battle::capture::CaptureOutcome {
            caught: true,
            blocked: false,
            storage_full: false,
            wobble_count: 3,
            animation_shakes: 3,
            final_catch_rate: u8::MAX,
            ball_id: Some("POKE_BALL".to_string()),
        },
        scripted_static_wild: None,
        default_name: "SUDOWOODO".to_string(),
        prompt_for_nickname: true,
    });
    assert!(
        !retained_field_fullscreen_active(&runtime_shell),
        "the capture nickname YES/NO must reveal its retained battle background"
    );

    runtime_shell.pending_name_choice = None;
    runtime_shell.pending_standard_capture = None;
    runtime_shell.pokedex_menu_open = true;
    assert!(retained_field_fullscreen_active(&runtime_shell));
    runtime_shell.pokedex_detail_open = true;
    assert!(
        retained_field_fullscreen_active(&runtime_shell),
        "the nested Dex detail screen remains owned while the Dex menu is active"
    );

    runtime_shell.pokedex_menu_open = false;
    runtime_shell.pokedex_detail_open = false;
    assert!(!retained_field_fullscreen_active(&runtime_shell));
    runtime_shell.pending_name_input = Some(PendingNameInput {
        label: "SUDOWOODO'S\nNICKNAME?".to_string(),
        value: "".to_string(),
        max_length: 10,
        cursor_column: 0,
        cursor_row: 0,
        case: NameInputCase::Upper,
    });
    assert!(
        retained_field_fullscreen_active(&runtime_shell),
        "accepting the nickname prompt must hand ownership to the full-screen naming LCD"
    );
}

#[test]
fn new_game_name_choice_uses_source_menu_over_player_portrait_lcd() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    open_visible_name_choice(&mut runtime_shell).expect("open source player-name menu");
    for _ in 0..40 {
        tick_visible_player_name_choice(&mut runtime_shell).expect("finish MovePlayerPicRight");
    }
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut app = integrated_shell_test_app(runtime_shell);

    app.update();

    let world = app.world_mut();
    let menu_sizes = world
        .query_filtered::<&Sprite, With<SceneDialogMarker>>()
        .iter(world)
        .filter_map(|sprite| sprite.custom_size)
        .collect::<Vec<_>>();
    assert!(
        menu_sizes.contains(&Vec2::new(11.0 * TILE_SIZE, 12.0 * TILE_SIZE)),
        "preset-name menu must use the exact menu_coords 0,0,10,11 extent; sizes={menu_sizes:?}"
    );

    let header_backing = world
        .query_filtered::<(&Sprite, &Transform), With<SceneDialogMarker>>()
        .iter(world)
        .any(|(sprite, transform)| {
            sprite.color == Color::WHITE
                && sprite.custom_size.is_some_and(|size| size.y == TILE_SIZE && size.x >= 4.0 * TILE_SIZE)
                && transform.translation.z > 6.0
                && transform.translation.z < 6.1
        });
    assert!(header_backing, "NAME must replace the border tiles with an opaque white text backing");

    let retained = retained_fullscreen_surface(app.world_mut());
    let images = app.world().resource::<Assets<Image>>();
    let image = images
        .get(&retained.texture)
        .expect("retained name-choice backdrop image");
    assert!(image.data.chunks_exact(4).all(|pixel| pixel[3] == 255));
    assert!(
        image
            .data
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]),
        "NamePlayer must retain the shifted player portrait and OakText6 behind its menu"
    );
    let top_right = (19 * SOURCE_TILE_SIZE) * 4;
    assert_eq!(
        &image.data[top_right..top_right + 4],
        &[255, 255, 255, 255],
        "the untouched LCD background must remain source-white"
    );
}

#[test]
fn custom_player_name_return_retains_naming_then_clears_and_redraws_portrait() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    open_visible_name_choice(&mut runtime_shell).expect("open source player-name menu");
    for _ in 0..40 {
        tick_visible_player_name_choice(&mut runtime_shell).expect("finish MovePlayerPicRight");
    }
    confirm_visible_name_choice(&mut runtime_shell).expect("choose NEW NAME");
    runtime_shell.pending_name_input.as_mut().expect("NamingScreen").value = "GOLD".to_string();
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut app = integrated_shell_test_app(runtime_shell);
    app.update();
    let naming_texture = retained_fullscreen_surface(app.world_mut()).texture;

    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        confirm_visible_player_name_input(&mut shell).expect("finish NamingScreen");
    }
    app.update();
    assert_eq!(
        retained_fullscreen_surface(app.world_mut()).texture,
        naming_texture,
        "RotateThreePalettesRight must fade the retained NamingScreen"
    );

    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        for _ in 0..24 {
            tick_visible_player_name_choice(&mut shell).expect("finish fade out");
        }
    }
    app.update();
    let blank_texture = retained_fullscreen_surface(app.world_mut()).texture;
    let images = app.world().resource::<Assets<Image>>();
    assert!(
        images
            .get(&blank_texture)
            .expect("cleared custom-name LCD")
            .data
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 255, 255, 255]),
        "ClearTilemap and WaitBGMap must expose the source-white LCD beneath the white palette"
    );

    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        for _ in 0..4 {
            tick_visible_player_name_choice(&mut shell).expect("finish WaitBGMap");
        }
    }
    app.update();
    let portrait_texture = retained_fullscreen_surface(app.world_mut()).texture;
    let images = app.world().resource::<Assets<Image>>();
    let image = images.get(&portrait_texture).expect("redrawn player LCD");
    assert!(
        image
            .data
            .chunks_exact(4)
            .any(|pixel| pixel != [255, 255, 255, 255]),
        "DrawIntroPlayerPic must redraw the source player portrait before fade-in"
    );
    let textbox_start = OAK_INTRO_TEXTBOX_Y * SOURCE_TILE_SIZE * 160 * 4;
    assert!(
        image.data[textbox_start..]
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 255, 255, 255]),
        "ClearTilemap means OakText6 must not be invented after custom naming"
    );
}

#[test]
fn transparent_tileset_color_zero_is_opaque_on_base_and_clear_on_priority() {
    let source = image::RgbaImage::from_pixel(
        SOURCE_TILE_SIZE as u32,
        SOURCE_TILE_SIZE as u32,
        image::Rgba([255, 255, 255, 0]),
    );
    let palette: Palette = [
        [17, 34, 51],
        [68, 85, 102],
        [119, 136, 153],
        [170, 187, 204],
    ];
    let mut base = vec![0_u8; SOURCE_TILE_SIZE * SOURCE_TILE_SIZE * 4];
    copy_source_tile_rgba(&source, SOURCE_TILE_SIZE, 0, Some(&palette), &mut base);
    assert!(
        base.chunks_exact(4).all(|pixel| pixel == [17, 34, 51, 255]),
        "exported alpha-zero BG pixels are hardware color zero, not holes to ClearColor"
    );

    let mut unpaletted_base = vec![0_u8; SOURCE_TILE_SIZE * SOURCE_TILE_SIZE * 4];
    copy_source_tile_rgba(&source, SOURCE_TILE_SIZE, 0, None, &mut unpaletted_base);
    assert!(
        unpaletted_base
            .chunks_exact(4)
            .all(|pixel| pixel == [255, 255, 255, 255]),
        "already-coloured tilesets must also keep Game Boy color zero opaque"
    );

    let mut priority = base.clone();
    clear_source_tile_palette_zero_alpha(&source, SOURCE_TILE_SIZE, 0, &mut priority);
    assert!(
        priority.chunks_exact(4).all(|pixel| pixel[3] == 0),
        "the separately composed priority layer must still clear color-zero pixels"
    );
}

#[cfg(feature = "fullscreen-scaling")]
#[test]
fn fullscreen_name_choices_separate_portrait_text_and_controls() {
    let mut shell = core_modular_title_shell_for_test();
    shell.intro_screen = None;
    shell.title_menu = None;
    open_visible_name_choice(&mut shell).unwrap();
    for _ in 0..40 { tick_visible_player_name_choice(&mut shell).unwrap(); }
    mark_runtime_snapshot_dirty(&mut shell);
    let mut app = integrated_shell_test_app(shell);
    app.world_mut().spawn((Window { resolution: WindowResolution::new(1920.0, 1080.0).with_scale_factor_override(1.0), ..default() }, bevy::window::PrimaryWindow));
    app.add_systems(Startup, setup_fullscreen_scene).add_systems(PostUpdate,
        (sync_fullscreen_scaling, sync_fullscreen_scene_layout, sync_fullscreen_world_layout).chain());
    app.update();
    let world = app.world_mut();
    let (sprite, transform) = world.query_filtered::<(&Sprite, &Transform), With<VisibleIntroSurface>>().single(world);
    assert_eq!(sprite.rect.unwrap().size(), Vec2::splat(56.0));
    assert!(sprite.custom_size.unwrap().y > 800.0);
    assert!(transform.translation.x < 0.0);
    let (sprite, transform, visibility) = world.query_filtered::<(&Sprite, &Transform, &Visibility), With<FullscreenBootDialogue>>().single(world);
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(sprite.custom_size, Some(Vec2::new(640.0, 192.0)));
    assert!(transform.translation.y < -500.0);
    let controls = world.query_filtered::<&Transform, With<FullscreenDialogRoot>>().single(world);
    assert_eq!(controls.scale, Vec3::ONE);
    assert!(controls.translation.x > 500.0);
}

#[test]
fn bookshelf_jumptext_remains_open_after_printing_until_player_confirms() {
    // JumpTextScript executes repeattext, waitbutton, closetext, end.
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .close_active_menu()
        .expect("close the completed map callback surface");
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .options
        .no_text_scroll = true;
    runtime_shell
        .shell
        .apply_script_text_command("PlayersHouse2F", "PictureBookshelfScript", 0)
        .expect("execute the source farjumptext");
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(native_rtc_source_for_test())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(RuntimeTickTimer::new(0.0))
        .add_systems(Update, apply_keyboard_input);
    for _ in 0..3 {
        app.update();
    }
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    assert!(
        shell
            .shell
            .session()
            .state()
            .script_runtime
            .text_window_open,
        "finishing PrintText must not consume JumpTextScript's waitbutton"
    );
    assert!(
        shell
            .shell
            .session()
            .state()
            .script_runtime
            .pending_text_wait
            .is_some()
    );
    press_visible_a_button(&mut shell).expect("acknowledge the bookshelf text");
    assert!(
        !shell
            .shell
            .session()
            .state()
            .script_runtime
            .text_window_open
    );
}

#[test]
fn empty_pc_withdraw_list_has_a_working_cancel_entry() {
    let mut shell = core_modular_title_shell_for_test();
    shell.intro_screen = None;
    shell.title_menu = None;
    shell.bill_pc_session_open = true;
    shell.bill_pc_action_cursor = Some(MenuCursor {
        surface_id: "pc:bill-actions".to_string(),
        option_index: 0,
    });
    confirm_visible_bill_pc_action(&mut shell).expect("open empty Withdraw list");
    assert!(shell.storage_cursor.is_some());
    press_visible_a_button(&mut shell).expect("empty list selects CANCEL");
    assert!(shell.storage_cursor.is_none());
    assert!(shell.bill_pc_action_cursor.is_some());
}

#[test]
fn autonomous_waitsfx_does_not_acknowledge_an_unread_text_page() {
    let mut shell = core_modular_title_shell_for_test();
    shell.shell.close_active_menu().expect("finish map callback surface");
    shell.shell.session_mut().state_mut().script_runtime.text_window_open = true;
    mark_runtime_snapshot_dirty(&mut shell);
    shell.field_notice = Some("First page.\n\nSecond page.".to_string());
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: "First page.\u{1e}Second page.".to_string(),
        page_index: 0,
        visible_chars: "First page.".len(),
        frames_until_next_char: 0,
    });
    shell.visible_wait_sfx_boundary = true;
    let snapshot = shell.shell.presentation_snapshot().expect("text snapshot");
    advance_visible_wait_sfx_boundary(&mut shell, &snapshot, true).expect("poll sound fence");
    assert_eq!(shell.field_text_reveal.as_ref().unwrap().page_index, 0,
        "an autonomous sound fence must not supply the player's page acknowledgement");
}

#[test]
fn pokegear_clock_uses_the_full_weekday_and_spaced_meridiem() {
    let runtime_shell = core_modular_title_shell_for_test();
    let mut snapshot = runtime_shell.shell.snapshot().expect("clock snapshot");
    snapshot.progression.time.day_of_week = 3;
    snapshot.progression.time.registers.hours = 0;
    snapshot.progression.time.registers.minutes = 7;
    let entries = visible_pokegear_menu_entries(&snapshot, &runtime_shell).expect("clock entries");
    assert_eq!(entries, ["WEDNESDAY", "12:07 AM"]);
}

#[test]
fn pokegear_and_pc_render_audit_surfaces_fit_the_gameboy_screen() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.initialize_permanent_phone_numbers().expect("source permanent contacts");
    mark_runtime_snapshot_dirty(&mut shell);
    let mut snapshot = shell.shell.snapshot().expect("render audit snapshot");
    for flag in ["ENGINE_MAP_CARD", "ENGINE_PHONE_CARD", "ENGINE_RADIO_CARD"] {
        snapshot.progression.active_engine_flags.insert(flag.into());
    }
    let data = shell.shell.runtime().data();
    let mut pokemon = crate::core::models::pokemon::create_pokemon_from_known_dvs(
        &snapshot.party.slots[0].pokemon.species, 10, crate::core::models::Dv::default(),
        &data.learnsets, &data.moves, &data.growth_rates).unwrap();
    pokemon.item = None;
    pokemon.mail = None;
    pokemon.original_trainer_name = "CHRIS".into();
    pokemon.original_trainer_id = 0;
    snapshot.party.slots[0].pokemon = pokemon.clone();
    let pc_box = snapshot.storage.boxes.iter_mut()
        .find(|pc_box| pc_box.index == snapshot.storage.current_pc_box).unwrap();
    pc_box.slots = vec![crate::RuntimePcBoxSlotSnapshot { index: 0, pokemon }];
    pc_box.count = 1;
    // Exercise the shipped, materialized pack. Loose repository graphics hid
    // missing Clock/Phone/Radio RLE files from this audit.
    let asset_root = shell.asset_root.clone();
    for label in ["clock", "phone", "phone-actions", "phone-ringing", "phone-conversation", "phone-hangup-click", "phone-hangup-dots", "phone-hangup-blank", "phone-after-hangup", "radio", "phone-deletable-actions", "phone-delete-confirm", "phone-deleted", "pc-withdraw", "pc-withdraw-actions", "pc-withdraw-release", "pc-stats-egg", "pc-stats-pink", "pc-stats-green", "pc-stats-blue", "pc-stats-return", "pc-move", "pc-move-actions", "pc-move-cancel", "pc-move-inserting", "pc-move-other-box", "pc-move-restored", "pc-move-saving", "pc-move-saved", "pc-withdraw-empty", "pc-deposit", "pc-deposit-actions", "pc-deposit-refusal", "pc-released", "pc-release-bye", "pc-withdraw-cry", "pc-withdraw-success"] {
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        let mut commands = Commands::new(&mut queue, &world);
        if label.starts_with("pc-") {
            if let Some(slot) = snapshot.storage.boxes[0].slots.first_mut() {
                slot.pokemon.is_egg = label == "pc-stats-egg";
                slot.pokemon.happiness = if slot.pokemon.is_egg { 20 } else { 70 };
            }
            shell.bill_pc_pokemon_summary = match label {
                "pc-stats-egg" | "pc-stats-pink" => Some(1), "pc-stats-green" => Some(2), "pc-stats-blue" => Some(3), _ => None,
            }.map(|page| VisiblePcPokemonSummary {
                location: VisiblePcPokemonLocation::Box { box_index: 0, box_slot: 0 }, page,
            });
            shell.bill_pc_move_open = label.starts_with("pc-move");
            shell.bill_pc_move_party_open = false;
            shell.bill_pc_move_loaded_box = if matches!(label, "pc-move-other-box" | "pc-move-saving" | "pc-move-saved") { 1 } else { 0 };
            shell.bill_pc_move_source = matches!(label, "pc-move-inserting" | "pc-move-other-box").then_some(VisiblePcMoveSource {
                location: crystal_assets::RuntimePokemonStorageLocation::Box { box_index: 0, slot: 0 },
                scroll: 0,
            });
            shell.bill_pc_move_save = (label == "pc-move-saving").then(|| VisibleBillPcMoveSave {
                presentation: VisiblePcMovePresentation {
                    names: Vec::new(),
                    pokemon: visible_pc_pokemon_info(&snapshot.storage.boxes[0].slots[0].pokemon),
                    cursor: MenuCursor { surface_id: storage_cursor_surface_id(1), option_index: 0 },
                    scroll: 0,
                },
                source: crystal_assets::RuntimePokemonStorageLocation::Box { box_index: 0, slot: 0 },
                target: crystal_assets::RuntimePokemonStorageLocation::Box { box_index: 1, slot: 0 },
                phase: VisibleBillPcMoveSavePhase::BeforeMove,
                frames_remaining: 11,
            });
            shell.pc_notice = None;
            shell.pc_transfer_sequence = None;
            shell.pc_release_sequence = None;
            shell.bill_pc_deposit_open = label.starts_with("pc-deposit");
            shell.bill_pc_pokemon_action_cursor = matches!(label, "pc-stats-return" | "pc-move-actions" | "pc-withdraw-actions" | "pc-withdraw-release" | "pc-deposit-actions" | "pc-deposit-refusal" | "pc-released" | "pc-release-bye" | "pc-withdraw-cry" | "pc-withdraw-success").then(|| MenuCursor {
                surface_id: "pc:pokemon-actions".to_string(), option_index: if label == "pc-stats-return" { 1 } else if matches!(label, "pc-withdraw-release" | "pc-released" | "pc-release-bye") { 2 } else { 0 },
            });
            shell.pending_pc_release = (label == "pc-withdraw-release").then_some(VisiblePcReleasePrompt {
                location: VisiblePcPokemonLocation::Box { box_index: snapshot.storage.current_pc_box, box_slot: 0 },
            });
            shell.yes_no_cursor = shell.pending_pc_release.as_ref().map(|_| MenuCursor {
                surface_id: "pc:release-confirm".to_string(), option_index: 0,
            });
            if label == "pc-move-saved" {
                let slot = snapshot.storage.boxes[0].slots[0].clone();
                snapshot.storage.boxes[1].slots = vec![slot];
                snapshot.storage.boxes[1].count = 1;
                snapshot.storage.boxes[0].slots.clear();
                snapshot.storage.boxes[0].count = 0;
            }
            if label == "pc-withdraw-empty" {
                let pc_box = snapshot.storage.boxes.iter_mut()
                    .find(|pc_box| pc_box.index == snapshot.storage.current_pc_box).unwrap();
                pc_box.slots.clear();
                pc_box.count = 0;
            }
            shell.storage_cursor = Some(MenuCursor {
                surface_id: if shell.bill_pc_deposit_open { pc_party_surface_id().to_string() } else { storage_cursor_surface_id(visible_pc_box_index(&snapshot, &shell)) },
                option_index: if label == "pc-move-cancel" { 1 } else { 0 },
            });
            if label == "pc-deposit-refusal" {
                shell.pc_notice = Some("It's your last <PK><MN>!".into());
                shell.pc_transfer_sequence = Some(VisiblePcTransferSequence {
                    kind: VisiblePcTransferKind::Deposit,
                    box_index: snapshot.storage.current_pc_box,
                    phase: VisiblePcTransferPhase::RefusalHold,
                    frames_remaining: 40,
                    close_submenu_after_hold: true,
                    success: None,
                });
            }
            if matches!(label, "pc-released" | "pc-release-bye") {
                shell.pc_notice = Some(if label == "pc-released" { "Released <PK><MN>." } else { "Bye, CYNDAQUIL!" }.into());
                shell.pc_release_sequence = Some(VisiblePcReleaseSequence {
                    box_index: snapshot.storage.current_pc_box,
                    species_name: "CYNDAQUIL".into(),
                    retained_names: vec!["CYNDAQUIL".into()],
                    phase: if label == "pc-released" { VisiblePcReleasePhase::Released } else { VisiblePcReleasePhase::Bye },
                    frames_remaining: 30,
                });
            }
            if matches!(label, "pc-withdraw-cry" | "pc-withdraw-success") {
                let holding_message = label == "pc-withdraw-success";
                shell.pc_notice = holding_message.then(|| "Got CYNDAQUIL!".into());
                shell.pc_transfer_sequence = Some(VisiblePcTransferSequence {
                    kind: VisiblePcTransferKind::Withdraw,
                    box_index: snapshot.storage.current_pc_box,
                    phase: if holding_message { VisiblePcTransferPhase::SuccessHold } else { VisiblePcTransferPhase::SuccessWaitCry },
                    frames_remaining: if holding_message { 40 } else { 0 },
                    close_submenu_after_hold: false,
                    success: Some(VisiblePcTransferSuccess {
                        names: vec!["CYNDAQUIL".into()],
                        pokemon: visible_pc_pokemon_info(&snapshot.party.slots[0].pokemon),
                        message: "Got CYNDAQUIL!".into(),
                    }),
                });
            }
            spawn_field_storage_screen(&mut commands, &snapshot, &shell, &mut art,
                &asset_root, &mut images).expect("render PC withdraw");
        } else {
            shell.pokegear_page = match label {
                "clock" => PokegearPage::Clock,
                label if label.starts_with("phone") => PokegearPage::Phone,
                _ => PokegearPage::Radio,
            };
            shell.pokegear_phone_call = (matches!(label, "phone-ringing" | "phone-conversation") || label.starts_with("phone-hangup-")).then(|| VisiblePokegearPhoneCall {
                contact_id: "PHONE_MOM".into(),
                phase: if label == "phone-ringing" { VisiblePokegearPhoneCallPhase::Ringing { rings_started: 1 } }
                    else if label == "phone-conversation" { VisiblePokegearPhoneCallPhase::Calling }
                    else { VisiblePokegearPhoneCallPhase::HangingUp { frames_remaining: match label {
                        "phone-hangup-click" => VISIBLE_POKEGEAR_HANGUP_FRAMES - 10,
                        "phone-hangup-dots" => VISIBLE_POKEGEAR_HANGUP_FRAMES - 40,
                        "phone-hangup-blank" => VISIBLE_POKEGEAR_HANGUP_FRAMES - 60,
                        _ => unreachable!(),
                    } } },
            });
            if label == "phone-ringing" {
                shell.pokegear_menu_open = false;
                assert!(retained_field_fullscreen_active(&shell),
                    "hInMenu=0 during a call must not retire the Phone card");
            }
            shell.pokegear_phone_menu = (label == "phone-actions").then(|| VisiblePokegearPhoneMenu {
                contact_id: "PHONE_MOM".into(), can_delete: false, cursor: 0, delete_confirmation: None,
            });
            if matches!(label, "phone-deletable-actions" | "phone-delete-confirm" | "phone-deleted") {
                snapshot.script_events.phone_number_order = vec![Some("PHONE_MOM".into()), Some("PHONE_ELM".into())];
                if label != "phone-deleted" {
                    snapshot.script_events.phone_number_order.push(Some("PHONE_BILL".into()));
                    shell.pokegear_phone_menu = Some(VisiblePokegearPhoneMenu {
                        contact_id: "PHONE_BILL".into(), can_delete: true,
                        cursor: if label == "phone-delete-confirm" { 1 } else { 0 },
                        delete_confirmation: (label == "phone-delete-confirm").then_some(0),
                    });
                }
                snapshot.script_events.phone_numbers = snapshot.script_events.phone_number_order.iter().flatten().cloned().collect();
                shell.pokegear_phone_cursor = 2;
            }
            if label == "phone-conversation" {
                snapshot.ui.text_window_open = true;
                snapshot.ui.text = Some(shell.shell.text_snapshot("MomPhoneNoPokemonText").expect("source Mom call text"));
                let pages = visible_field_dialog_pages(&snapshot, &shell).expect("source phone pages");
                shell.field_text_reveal = Some(VisibleFieldTextReveal {
                    text: pages.join("\u{1e}"), page_index: 0,
                    visible_chars: pages[0].chars().count(), frames_until_next_char: 0,
                });
                shell.shell.session_mut().state_mut().vblank_counter = 0;
            }
            spawn_field_pokegear_screen(&mut commands, &snapshot, &shell, &mut art,
                &asset_root, &mut images).expect("render Pokégear");
            if label == "phone-conversation" {
                spawn_scene_dialog(&mut commands, &snapshot, &shell, &mut art,
                    &asset_root, &mut images).expect("render phone conversation in the card");
                snapshot.ui.text_window_open = false;
                snapshot.ui.text = None;
                shell.field_text_reveal = None;
            }
        }
        queue.apply(&mut world);
        assert_eq!(art.font_error, None, "{label} font rendering");
        let canvas = render_pc_audit_canvas(&mut world, &images, label);
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            std::fs::create_dir_all(&directory).expect("create render audit directory");
            canvas.save(PathBuf::from(directory).join(format!("{label}.png")))
                .expect("save rendered UI audit");
        }
    }
}

#[test]
fn pokegear_cards_match_original_rom_lcd() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.initialize_permanent_phone_numbers().unwrap();
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.trainer.player_gender = PLAYER_GENDER_FEMALE;
    snapshot.overworld.map_name = "OlivinePokecenter1F".into();
    snapshot.progression.time.day_of_week = 0;
    snapshot.progression.time.registers.hours = 0;
    snapshot.progression.time.registers.minutes = 0;
    for flag in ["ENGINE_MAP_CARD", "ENGINE_PHONE_CARD", "ENGINE_RADIO_CARD"] {
        snapshot.progression.active_engine_flags.insert(flag.into());
    }
    shell.pokegear_cursor = visible_pokegear_initial_cursor_index(&snapshot, false).unwrap();
    // trace.json records BlueWalk OAM tiles $14..$17 without X flip.
    shell.pokegear_map_animation_frame = 9;
    let reference_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tools/asm-oracle/fixtures");
    let mut cases = [("clock", PokegearPage::Clock), ("map", PokegearPage::Map),
        ("phone", PokegearPage::Phone), ("phone-actions", PokegearPage::Phone),
        ("radio", PokegearPage::Radio)].into_iter()
        .map(|(label, page)| (label, page, 1_u8, 7_u8, reference_root.join("pokegear-female")))
        .collect::<Vec<_>>();
    cases.push(("map", PokegearPage::Map, 0, 7, reference_root.join("pokegear-male")));
    for gender in 0..2 {
        for cards in 0..8 {
            cases.push(("clock", PokegearPage::Clock, gender, cards,
                reference_root.join(format!("pokegear-clock-cards/{gender}/{cards}"))));
        }
    }
    let mut differences = Vec::new();
    for (label, page, gender, cards, reference) in cases {
        snapshot.trainer.player_gender = gender;
        // wPokegearFlags source bit order differs from the visible tab order.
        for (bit, flag) in ["ENGINE_MAP_CARD", "ENGINE_RADIO_CARD", "ENGINE_PHONE_CARD"].into_iter().enumerate() {
            if cards & (1 << bit) != 0 { snapshot.progression.active_engine_flags.insert(flag.into()); }
            else { snapshot.progression.active_engine_flags.remove(flag); }
        }
        let identity = format!("{label}-gender{gender}-cards{cards}");
        shell.pokegear_page = page;
        shell.pokegear_phone_menu = (label == "phone-actions").then(|| VisiblePokegearPhoneMenu {
            contact_id: "PHONE_MOM".into(), can_delete: false, cursor: 0, delete_confirmation: None,
        });
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        spawn_field_pokegear_screen(&mut Commands::new(&mut queue, &world), &snapshot,
            &shell, &mut art, &shell.asset_root, &mut images).unwrap();
        queue.apply(&mut world);
        assert_eq!(art.font_error, None, "{label}");
        let canvas = render_pc_audit_canvas(&mut world, &images, label);
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            canvas.save(PathBuf::from(directory).join(format!("{identity}.png"))).unwrap();
        }
        let rom = image::open(reference.join(format!("rom-{label}.png"))).unwrap().to_rgba8();
        assert_eq!(rom.dimensions(), (160, 144));
        let scale = canvas.width() / 160;
        assert_eq!(canvas.dimensions(), (160 * scale, 144 * scale));
        let count = canvas.enumerate_pixels().filter(|(x, y, pixel)| {
            let expected = rom.get_pixel(x / scale, y / scale);
            pixel[3] != 255 || (0..3).any(|channel| pixel[channel] >> 3 != expected[channel] >> 3)
        }).count();
        differences.push((identity, count));
    }
    assert!(differences.iter().all(|(_, count)| *count == 0), "RGB5 pixel differences: {differences:?}");
}

#[test]
fn pc_stats_returns_to_the_same_pokemon_submenu() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    mark_runtime_snapshot_dirty(&mut shell);
    shell.storage_cursor = Some(MenuCursor {
        surface_id: storage_cursor_surface_id(0), option_index: 0,
    });
    open_visible_bill_pc_pokemon_actions(&mut shell).expect("open Pokémon submenu");
    shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = 1;
    confirm_visible_bill_pc_pokemon_action(&mut shell).expect("open STATS");
    assert!(shell.bill_pc_pokemon_summary.is_some());
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().map(|cursor| cursor.option_index),
        Some(1), "BillsPC stats returns to the same submenu and cursor");
    press_visible_b_button(&mut shell).expect("close STATS");
    assert!(shell.bill_pc_pokemon_summary.is_none());
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().map(|cursor| cursor.option_index), Some(1));
}

#[test]
fn pc_egg_stats_ignores_page_arrows_and_a_closes_on_any_retained_page() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None; pokemon.mail = None;
    pokemon.is_egg = true;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
    pokemon.is_egg = false;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    mark_runtime_snapshot_dirty(&mut shell);
    shell.bill_pc_session_open = true;
    shell.bill_pc_action_cursor = Some(MenuCursor { surface_id: "pc:bill-actions".into(), option_index: 0 });
    confirm_visible_bill_pc_action(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 1, "EggStatsJoypad masks Left/Right");
    move_visible_primary_cursor_left(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 1);
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_summary.is_none(), "Egg A exits instead of advancing pages");
    press_visible_a_button(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 2);
    move_visible_primary_cursor_up(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 2, "browsing through an Egg preserves hidden page bits");
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_summary.is_none());
}

#[test]
fn party_stats_browsing_keeps_the_current_page() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    shell.pending_audio.clear();
    shell.party_summary_open = true;
    shell.party_summary_page = 2;
    move_visible_party_summary_pokemon(&mut shell, 1).unwrap();
    assert_eq!(shell.party_cursor, 1);
    assert_eq!(shell.party_summary_page, 2, "MonStatsInit does not reset page bits when changing Pokemon");
}

#[test]
fn pc_stats_rebuilds_boxed_party_fields_without_mutating_storage() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    pokemon.hp = 1;
    pokemon.max_hp = 1;
    pokemon.attack = 1;
    pokemon.defense = 1;
    pokemon.speed = 1;
    pokemon.special_attack = 1;
    pokemon.special_defense = 1;
    pokemon.status = Some("POISON".into());
    pokemon.experience = 500_000; // Stats uses stored level, not CalcLevel.
    state.storage.party.pokemon[0] = Some(pokemon.clone());
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
    pokemon.is_egg = true;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    let snapshot = shell.shell.snapshot().unwrap();
    let boxed = visible_pc_summary_pokemon(&snapshot,
        VisiblePcPokemonLocation::Box { box_index: 0, box_slot: 0 }).unwrap();
    assert_eq!(boxed.hp, 27, "CalcBufferMonStats reconstructs boxed HP from level/DVs/stat experience");
    assert_eq!((boxed.attack, boxed.defense, boxed.speed, boxed.special_attack, boxed.special_defense),
        (15, 13, 18, 17, 15));
    assert_eq!(boxed.level, 10);
    assert!(boxed.status.is_none());
    let egg = visible_pc_summary_pokemon(&snapshot,
        VisiblePcPokemonLocation::Box { box_index: 0, box_slot: 1 }).unwrap();
    assert_eq!(egg.hp, 0);
    assert_eq!(egg.max_hp, 27);
    assert_eq!(snapshot.storage.boxes[0].slots[0].pokemon.hp, 1, "the source temp-mon copy does not mutate saved storage");
    let party = visible_pc_summary_pokemon(&snapshot, VisiblePcPokemonLocation::Party(0)).unwrap();
    assert_eq!(party.hp, 1);
    assert_eq!(party.status.as_deref(), Some("POISON"));
}

#[test]
fn pc_stats_vertical_input_browses_real_pokemon_and_retains_the_page() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    for index in 0..8 {
        pokemon.nickname = format!("MON{index}");
        assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
        if index == 0 { state.storage.party.pokemon[0] = Some(pokemon.clone()); }
        else if index < 6 { assert!(state.storage.party.add_pokemon(pokemon.clone())); }
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    for party in [false, true] {
        shell.bill_pc_action_cursor = None; // Directly enter the already-saved Move fixture.
        open_visible_bill_pc_move_mode(&mut shell).unwrap();
        if party { switch_visible_pc_move_container(&mut shell, -1).unwrap(); }
        press_visible_a_button(&mut shell).unwrap();
        move_visible_primary_cursor_down(&mut shell).unwrap();
        press_visible_a_button(&mut shell).unwrap();
        move_visible_primary_cursor_right(&mut shell).unwrap();
        assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 2);
        move_visible_primary_cursor_down(&mut shell).unwrap();
        let location = shell.bill_pc_pokemon_summary.as_ref().unwrap().location;
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(visible_pc_pokemon_at(&snapshot, location).unwrap().nickname, "MON1",
            "StatsScreenDPad loads the next Pokemon instead of ignoring Down");
        for _ in 0..8 { move_visible_primary_cursor_down(&mut shell).unwrap(); }
        let last = if party { 5 } else { 7 };
        assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, last,
            "Stats browsing excludes the CANCEL entry and stops at the last Pokemon");
        assert_eq!(shell.pc_list_scroll, last - 4);
        assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 2);
        press_visible_b_button(&mut shell).unwrap();
        assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 1);
        assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, last);
        press_visible_a_button(&mut shell).unwrap();
        for _ in 0..10 { move_visible_primary_cursor_up(&mut shell).unwrap(); }
        assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0);
        assert_eq!(shell.pc_list_scroll, 0);
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(visible_pc_pokemon_at(&snapshot, shell.bill_pc_pokemon_summary.as_ref().unwrap().location)
            .unwrap().nickname, "MON0");
        press_visible_b_button(&mut shell).unwrap();
        press_visible_b_button(&mut shell).unwrap();
        press_visible_b_button(&mut shell).unwrap();
    }
}

#[test]
fn pc_deposit_and_withdraw_ignore_horizontal_box_browsing() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.bill_pc_session_open = true;
    for action_index in [0, 1] {
        shell.bill_pc_action_cursor = Some(MenuCursor {
            surface_id: "pc:bill-actions".to_string(), option_index: action_index,
        });
        confirm_visible_bill_pc_action(&mut shell).unwrap();
        let before = shell.storage_cursor.clone();
        move_visible_primary_cursor_left(&mut shell).unwrap();
        assert_eq!(shell.shell.snapshot().unwrap().storage.current_pc_box, 0,
            "Withdraw_UpDown does not read PAD_LEFT/PAD_RIGHT");
        assert_eq!(shell.storage_cursor, before);
        move_visible_primary_cursor_right(&mut shell).unwrap();
        assert_eq!(shell.shell.snapshot().unwrap().storage.current_pc_box, 0);
        assert_eq!(shell.storage_cursor, before);
        press_visible_b_button(&mut shell).unwrap();
    }
}

#[test]
fn pc_move_same_position_still_runs_the_source_save() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    for _ in 0..crate::core::models::MAX_BOX_MONS {
        assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
    }
    mark_runtime_snapshot_dirty(&mut shell);
    open_visible_bill_pc_move_mode(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_move_save.is_some(),
        "CheckTrivialMove adjusts indices; it does not cancel the source save");
    assert_eq!(shell.bill_pc_move_save.as_ref().unwrap().frames_remaining, 20);
}

#[test]
fn pc_move_full_destination_refuses_before_saving_and_retains_insertion() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
    for _ in 0..crate::core::models::MAX_BOX_MONS {
        assert!(state.storage.pc_boxes[1].add_pokemon(pokemon.clone()));
    }
    for _ in 1..6 { assert!(state.storage.party.add_pokemon(pokemon.clone())); }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    for party in [false, true] {
        open_visible_bill_pc_move_mode(&mut shell).unwrap();
        press_visible_a_button(&mut shell).unwrap();
        press_visible_a_button(&mut shell).unwrap();
        switch_visible_pc_move_container(&mut shell, if party { -1 } else { 1 }).unwrap();
        press_visible_a_button(&mut shell).unwrap();
        assert!(shell.bill_pc_move_save.is_none(), "capacity is checked before the save hold");
        assert!(shell.bill_pc_move_source.is_some());
        assert_eq!(shell.pc_notice.as_deref(), Some("There's no room!"));
        assert!(matches!(shell.pc_transfer_sequence.as_ref().map(|sequence| sequence.phase),
            Some(VisiblePcTransferPhase::RefusalWaitSfx)));
        shell.pending_audio.clear();
        advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
        assert_eq!(shell.pc_transfer_sequence.as_ref().unwrap().frames_remaining, 50);
        press_visible_b_button(&mut shell).unwrap();
        assert!(shell.bill_pc_move_source.is_some(), "B cannot skip refusal");
        advance_visible_pc_transfer_sequence(&mut shell, 49).unwrap();
        assert!(shell.pc_notice.is_some());
        advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
        assert!(shell.pc_notice.is_none());
        assert!(shell.bill_pc_move_source.is_some(), "no-room returns to insertion");
        assert_eq!(shell.shell.snapshot().unwrap().storage.current_pc_box, 0);
        press_visible_b_button(&mut shell).unwrap();
    }
}

#[test]
fn pc_move_cancel_restores_source_window_without_changing_saved_box() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    for index in 0..8 {
        pokemon.nickname = format!("SOURCE{index}");
        assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
        if index == 0 { state.storage.party.pokemon[0] = Some(pokemon.clone()); }
        else if index < 6 { assert!(state.storage.party.add_pokemon(pokemon.clone())); }
    }
    for index in 0..3 {
        pokemon.nickname = format!("TARGET{index}");
        assert!(state.storage.pc_boxes[1].add_pokemon(pokemon.clone()));
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    for party in [false, true] {
        open_visible_bill_pc_move_mode(&mut shell).unwrap();
        if party { switch_visible_pc_move_container(&mut shell, -1).unwrap(); }
        for _ in 0..5 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
        assert_eq!(shell.pc_list_scroll, 1);
        press_visible_a_button(&mut shell).unwrap();
        press_visible_a_button(&mut shell).unwrap();
        assert!(shell.bill_pc_move_source.is_some());
        switch_visible_pc_move_container(&mut shell, if party { 2 } else { 1 }).unwrap();
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(snapshot.storage.current_pc_box, 0,
            "Move browsing changes wBillsPC_LoadedBox, never saved wCurBox");
        let mut entries = Vec::new();
        push_visible_storage_dialog_entries(&mut entries, &snapshot, &shell).unwrap();
        assert_eq!(entries[0], snapshot.storage.boxes[1].name);
        assert!(entries.iter().any(|entry| entry.contains("TARGET0")));
        move_visible_storage_cursor_down(&mut shell).unwrap();
        press_visible_b_button(&mut shell).unwrap();
        assert!(shell.bill_pc_move_source.is_none());
        assert_eq!(shell.bill_pc_move_party_open, party);
        assert_eq!(shell.storage_cursor.as_ref().unwrap().surface_id,
            if party { pc_party_surface_id().to_string() } else { storage_cursor_surface_id(0) });
        assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 5);
        assert_eq!(shell.pc_list_scroll, 1, "B restores BackupScrollPosition");
        assert_eq!(shell.shell.snapshot().unwrap().storage.current_pc_box, 0);
    }
    open_visible_bill_pc_move_mode(&mut shell).unwrap();
    for box_index in 1..crate::core::models::MAX_PC_BOXES {
        switch_visible_pc_move_container(&mut shell, 1).unwrap();
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(snapshot.storage.current_pc_box, 0);
        assert_eq!(visible_storage_box(&snapshot, &shell).unwrap().index, box_index);
        let mut entries = Vec::new();
        push_visible_storage_dialog_entries(&mut entries, &snapshot, &shell).unwrap();
        assert_eq!(entries[0], snapshot.storage.boxes[box_index].name);
    }
    switch_visible_pc_move_container(&mut shell, 1).unwrap();
    assert!(shell.bill_pc_move_party_open);
    switch_visible_pc_move_container(&mut shell, 1).unwrap();
    assert!(!shell.bill_pc_move_party_open);
    assert_eq!(shell.bill_pc_move_loaded_box, 0);
}

#[test]
fn pc_move_opens_source_submenu_before_picking_up_a_pokemon() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    mark_runtime_snapshot_dirty(&mut shell);
    open_visible_bill_pc_move_mode(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_move_source.is_none(), "A opens the submenu, not insertion mode");
    assert!(shell.bill_pc_pokemon_action_cursor.is_some());
    move_visible_primary_cursor_up(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 0,
        "STATICMENU_CURSOR without STATICMENU_WRAP stops at the top");
    move_visible_primary_cursor_left(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.shell.snapshot().unwrap().storage.current_pc_box, 0,
        "submenu horizontal input must not browse boxes behind the popup");
    assert_eq!(visible_pc_pokemon_action_labels(&shell).as_ref(), ["MOVE", "STATS", "CANCEL"]);
    move_visible_primary_cursor_down(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 2,
        "the submenu stops at CANCEL");
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_action_cursor.is_none());
    assert!(shell.pending_pc_release.is_none(), "Move option three is CANCEL, never RELEASE");
    assert!(shell.bill_pc_move_open);
    press_visible_a_button(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().location,
        VisiblePcPokemonLocation::Box { box_index: 0, box_slot: 0 });
    press_visible_b_button(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 1);
    press_visible_b_button(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_move_source.is_some());
    assert!(shell.bill_pc_pokemon_action_cursor.is_none());
    press_visible_b_button(&mut shell).unwrap();
    switch_visible_pc_move_container(&mut shell, -1).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().location, VisiblePcPokemonLocation::Party(0));
    move_visible_primary_cursor_left(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 3);
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().page, 1);
    move_visible_primary_cursor_left(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_summary.is_none(), "A exits the blue Stats page");
    move_visible_primary_cursor_up(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_move_source.is_none(), "BillsPC_CheckMail_PreventBlackout runs before insertion");
    assert_eq!(shell.pc_notice.as_deref(), Some("It's your last <PK><MN>!"));
}

#[test]
fn pc_move_lists_keep_five_rows_and_include_cancel() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    for index in 0..8 {
        pokemon.nickname = format!("MON{index}");
        assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
        if index == 0 { state.storage.party.pokemon[0] = Some(pokemon.clone()); }
        else if index < 6 { assert!(state.storage.party.add_pokemon(pokemon.clone())); }
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    for party in [false, true] {
        for inserting in [false, true] {
            shell.bill_pc_move_open = true;
            shell.bill_pc_move_party_open = party;
            shell.bill_pc_move_source = inserting.then_some(VisiblePcMoveSource {
                location: crystal_assets::RuntimePokemonStorageLocation::Box { box_index: 0, slot: 0 }, scroll: 0,
            });
            shell.pc_list_scroll = 0;
            shell.storage_cursor = Some(MenuCursor {
                surface_id: if party { pc_party_surface_id().to_string() } else { storage_cursor_surface_id(0) },
                option_index: 0,
            });
            move_visible_storage_cursor_up(&mut shell).unwrap();
            assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0,
                "Move uses BillsPC_PressUp and must not wrap");
            for _ in 0..3 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
            let rows = |shell: &BevyRuntimeShell| {
                let mut entries = Vec::new();
                push_visible_storage_dialog_entries(&mut entries, &shell.shell.snapshot().unwrap(), shell).unwrap();
                entries.into_iter().skip(1).collect::<Vec<_>>()
            };
            assert_eq!(rows(&shell), [" MON0", " MON1", " MON2", ">MON3", " MON4"]);
            for _ in 0..2 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
            assert_eq!(rows(&shell), [" MON1", " MON2", " MON3", " MON4", ">MON5"]);
            move_visible_storage_cursor_up(&mut shell).unwrap();
            assert_eq!(rows(&shell), [" MON1", " MON2", " MON3", ">MON4", " MON5"]);
            for _ in 0..12 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
            assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, if party { 6 } else { 8 });
            assert_eq!(rows(&shell).last().map(String::as_str), Some(">CANCEL"),
                "CopyBoxmonSpecies and RefreshTextboxes append CANCEL in both Move phases");
            if !inserting {
                confirm_visible_bill_pc_move(&mut shell).unwrap();
                assert!(!shell.bill_pc_move_open, "selecting CANCEL must leave Move");
                assert_eq!(shell.bill_pc_action_cursor.as_ref().unwrap().option_index, 3);
                shell.bill_pc_action_cursor = None;
            }
        }
    }
    open_visible_bill_pc_move_mode(&mut shell).unwrap();
    press_visible_b_button(&mut shell).unwrap();
    assert!(!shell.bill_pc_move_open, "B must leave source selection as CANCEL does");
    assert_eq!(shell.bill_pc_action_cursor.as_ref().unwrap().option_index, 3);

}

#[test]
fn pc_withdraw_list_keeps_five_rows_and_stops_at_both_ends() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    for index in 0..8 {
        pokemon.nickname = format!("MON{index}");
        assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
    }
    mark_runtime_snapshot_dirty(&mut shell);
    shell.storage_cursor = Some(MenuCursor { surface_id: storage_cursor_surface_id(0), option_index: 0 });
    move_visible_storage_cursor_up(&mut shell).unwrap();
    assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0, "BillsPC_PressUp stops at the first row");
    for _ in 0..3 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
    let rows = |shell: &BevyRuntimeShell| {
        let mut entries = Vec::new();
        push_visible_storage_dialog_entries(&mut entries, &shell.shell.snapshot().unwrap(), shell).unwrap();
        entries.into_iter().skip(1).collect::<Vec<_>>()
    };
    assert_eq!(rows(&shell), [" MON0", " MON1", " MON2", ">MON3", " MON4"]);
    for _ in 0..2 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
    assert_eq!(rows(&shell), [" MON1", " MON2", " MON3", " MON4", ">MON5"]);
    move_visible_storage_cursor_up(&mut shell).unwrap();
    assert_eq!(rows(&shell), [" MON1", " MON2", " MON3", ">MON4", " MON5"]);
    for _ in 0..10 { move_visible_storage_cursor_down(&mut shell).unwrap(); }
    assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 8);
    assert_eq!(rows(&shell), [" MON4", " MON5", " MON6", " MON7", ">CANCEL"]);
}

#[test]
fn pc_boxed_egg_release_waits_for_sound_and_fifty_frames() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    pokemon.is_egg = true;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    mark_runtime_snapshot_dirty(&mut shell);
    shell.storage_cursor = Some(MenuCursor { surface_id: storage_cursor_surface_id(0), option_index: 0 });
    open_visible_bill_pc_pokemon_actions(&mut shell).unwrap();
    shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = 2;
    confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
    assert!(shell.pending_pc_release.is_none());
    assert!(shell.pc_transfer_sequence.is_some(), "BillsPC_IsMonAnEgg must wait for SFX_WRONG and 50 frames");
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.pc_notice.is_some());
    shell.pending_audio.clear();
    shell.transient_audio_playing = false;
    advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
    advance_visible_pc_transfer_sequence(&mut shell, 49).unwrap();
    assert!(shell.pc_notice.is_some());
    advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
    assert!(shell.pc_notice.is_none());
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 2);
    assert_eq!(shell.shell.session().state().storage.pc_boxes[0].count, 1);
}

#[test]
fn pc_egg_withdrawal_does_not_play_the_unhatched_species_cry() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut egg = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    egg.is_egg = true;
    egg.item = None;
    egg.mail = None;
    assert!(state.storage.pc_boxes[0].add_pokemon(egg));
    mark_runtime_snapshot_dirty(&mut shell);
    shell.pending_audio.clear();
    shell.storage_cursor = Some(MenuCursor { surface_id: storage_cursor_surface_id(0), option_index: 0 });
    withdraw_visible_pc_pokemon(&mut shell).unwrap();
    assert!(!shell.pending_audio.iter().any(|audio| audio.audio_id.starts_with("CRY_")),
        "GetCryIndex rejects EGG; the unhatched species must stay silent");
}

#[test]
fn pc_success_notices_retain_the_selected_submenu_until_the_last_frame() {
    for action in ["deposit", "withdraw", "release"] {
        let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let state = shell.shell.session_mut().state_mut();
        let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
        pokemon.item = None;
        pokemon.mail = None;
        for index in 0..2 {
            pokemon.nickname = format!("MON{index}");
            state.storage.party.pokemon[index] = Some(pokemon.clone());
            assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
        }
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut shell);
        shell.bill_pc_deposit_open = action == "deposit";
        shell.storage_cursor = Some(MenuCursor {
            surface_id: if action == "deposit" { pc_party_surface_id().into() } else { storage_cursor_surface_id(0) },
            option_index: 1,
        });
        open_visible_bill_pc_pokemon_actions(&mut shell).unwrap();
        shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = if action == "release" { 2 } else { 0 };
        confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
        if action == "release" { confirm_visible_pc_release_prompt(&mut shell).unwrap(); }
        assert!(shell.bill_pc_pokemon_action_cursor.is_some(), "{action}: the source retains its submenu during the notice");
        let retained_names = if action == "release" {
            &shell.pc_release_sequence.as_ref().unwrap().retained_names
        } else {
            &shell.pc_transfer_sequence.as_ref().unwrap().success.as_ref().unwrap().names
        };
        assert_eq!(retained_names, &["MON0", "MON1"], "{action}: removal must not rewrite the retained list");
        if action != "release" {
            assert!(shell.pc_notice.is_none());
            assert_eq!(shell.pc_transfer_sequence.as_ref().unwrap().success.as_ref().unwrap().pokemon.level, 10,
                "the cry retains the level displayed before the core withdrawal conversion");
            assert_eq!(shell.pc_transfer_sequence.as_ref().unwrap().success.as_ref().unwrap().pokemon.species_id,
                shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().species.id);
            shell.pending_audio.clear();
            shell.transient_audio_playing = false;
            advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
        }
        if action == "release" {
            advance_visible_pc_release_sequence(&mut shell, 129).unwrap();
        } else {
            advance_visible_pc_transfer_sequence(&mut shell, 49).unwrap();
        }
        assert!(shell.bill_pc_pokemon_action_cursor.is_some());
        if action == "release" {
            advance_visible_pc_release_sequence(&mut shell, 1).unwrap();
        } else {
            advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
        }
        assert!(shell.bill_pc_pokemon_action_cursor.is_none());
        assert!(shell.pc_notice.is_none());
        assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0);
    }
}

#[test]
fn pc_capacity_refusals_preserve_the_selected_mon_and_action_menu() {
    for (deposit, capacity) in [(false, true), (true, true), (true, false)] {
        let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let state = shell.shell.session_mut().state_mut();
        let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
        pokemon.item = None;
        pokemon.mail = None;
        for (index, slot) in state.storage.party.pokemon.iter_mut().enumerate() {
            let mut member = pokemon.clone();
            if !capacity && index != 1 { member.hp = 0; }
            *slot = Some(member);
        }
        for _ in 0..if deposit && capacity { 20 } else { 2 } {
            assert!(state.storage.pc_boxes[0].add_pokemon(pokemon.clone()));
        }
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut shell);
        shell.bill_pc_deposit_open = deposit;
        shell.storage_cursor = Some(MenuCursor {
            surface_id: if deposit { pc_party_surface_id().into() } else { storage_cursor_surface_id(0) }, option_index: 1,
        });
        open_visible_bill_pc_pokemon_actions(&mut shell).unwrap();
        confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
        assert!(shell.pc_transfer_sequence.is_some());
        assert!(shell.bill_pc_pokemon_action_cursor.is_some(),
            "the source retains the submenu until the refusal sound and hold finish");
        shell.pending_audio.clear();
        shell.transient_audio_playing = false;
        advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
        advance_visible_pc_transfer_sequence(&mut shell, 50).unwrap();
        assert_eq!(shell.storage_cursor.as_ref().map(|cursor| cursor.option_index), Some(1), "deposit={deposit}: failed transfer keeps the list position");
        assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().map(|cursor| cursor.option_index), if capacity { Some(0) } else { None }, "deposit={deposit} capacity={capacity}: source refusal destination");
    }
}

#[test]
fn pc_removing_last_boxed_pokemon_keeps_the_empty_cancel_list() {
    for release in [true, false] {
        let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
        let state = shell.shell.session_mut().state_mut();
        let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
        pokemon.item = None;
        pokemon.mail = None;
        assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
        mark_runtime_snapshot_dirty(&mut shell);
        shell.storage_cursor = Some(MenuCursor { surface_id: storage_cursor_surface_id(0), option_index: 0 });
        if release {
            release_visible_pc_pokemon(&mut shell).unwrap();
            advance_visible_pc_release_sequence(&mut shell, 130).unwrap();
        } else {
            withdraw_visible_pc_pokemon(&mut shell).unwrap();
            shell.pending_audio.clear();
            shell.transient_audio_playing = false;
            advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
            advance_visible_pc_transfer_sequence(&mut shell, 50).unwrap();
        }
        assert_eq!(shell.shell.session().state().storage.pc_boxes[0].count, 0);
        assert_eq!(shell.storage_cursor.as_ref().map(|cursor| cursor.option_index), Some(0),
            "release={release}: the rebuilt source list still owns CANCEL");
    }
}

#[test]
fn pc_deposit_opens_source_submenu_before_moving_a_party_pokemon() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let pokemon = state.storage.party.pokemon[0].as_mut().unwrap();
    pokemon.item = None;
    pokemon.mail = None;
    state.storage.party.pokemon[1] = Some(pokemon.clone());
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    shell.bill_pc_session_open = true;
    shell.bill_pc_action_cursor = Some(MenuCursor { surface_id: "pc:bill-actions".into(), option_index: 1 });
    confirm_visible_bill_pc_action(&mut shell).unwrap();
    assert!(shell.bill_pc_deposit_open);
    assert!(!shell.party_menu_open);
    open_visible_bill_pc_pokemon_actions(&mut shell).unwrap();
    assert_eq!(shell.shell.session().state().storage.party.filled_slots(), 2);
    let snapshot = shell.shell.snapshot().unwrap();
    let mut entries = Vec::new();
    push_visible_storage_dialog_entries(&mut entries, &snapshot, &shell).unwrap();
    assert_eq!(&entries[1..], &[">DEPOSIT", " STATS", " RELEASE", " CANCEL"]);
    shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = 1;
    confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_summary.as_ref().unwrap().location, VisiblePcPokemonLocation::Party(0));
    press_visible_b_button(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 1);
    shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = 2;
    confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
    assert_eq!(shell.pending_pc_release.as_ref().unwrap().location, VisiblePcPokemonLocation::Party(0));
    press_visible_b_button(&mut shell).unwrap();
    assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().unwrap().option_index, 2);
    shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = 0;
    confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
    assert_eq!(shell.shell.session().state().storage.party.filled_slots(), 1);
    assert_eq!(shell.shell.session().state().storage.pc_boxes[0].count, 1);
    shell.pending_audio.clear();
    shell.transient_audio_playing = false;
    advance_visible_pc_transfer_sequence(&mut shell, 1).unwrap();
    advance_visible_pc_transfer_sequence(&mut shell, 50).unwrap();
    assert!(shell.bill_pc_deposit_open);
    assert!(!shell.party_menu_open);
    assert_eq!(shell.storage_cursor.as_ref().unwrap().surface_id, pc_party_surface_id());
    press_visible_b_button(&mut shell).unwrap();
    assert!(!shell.bill_pc_deposit_open);
    assert!(shell.bill_pc_action_cursor.is_some());
}

#[test]
fn pc_party_release_checks_source_refusals_then_compacts_the_party() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let initial = shell.shell.session().state().clone();
    for (case, expected) in [("last", "last party"), ("fainted", "last usable"), ("mail", "holding mail"), ("egg", "Egg"), ("valid", "")] {
        let state = shell.shell.session_mut().state_mut();
        *state = initial.clone();
        let first = state.storage.party.pokemon[0].as_mut().unwrap();
        if case != "mail" { first.item = None; first.mail = None; }
        let mut second = first.clone();
        second.nickname = "SECOND".into();
        second.item = None;
        second.mail = None;
        if case == "fainted" { second.hp = 0; }
        if case == "egg" { first.is_egg = true; }
        if case != "last" { state.storage.party.pokemon[1] = Some(second); }
        state.sync_party_from_storage();
        let before = state.clone();
        let result = shell.shell.apply_runtime_mutation_command(
            crate::RuntimeMutationCommand::ReleasePartyPokemon(crate::assets::RuntimePartySlotCommand { party_index: 0 }),
        );
        if case == "valid" {
            assert!(matches!(result.unwrap().result, crate::RuntimeMutationResult::PartyPokemonReleased(_)));
            let state = shell.shell.session().state();
            assert_eq!(state.storage.party.filled_slots(), 1);
            assert_eq!(state.storage.party.pokemon[0].as_ref().unwrap().nickname, "SECOND");
        } else {
            let error = format!("{:#}", result.unwrap_err());
            assert!(error.contains(expected), "{case}: {error}");
            assert_eq!(shell.shell.session().state(), &before, "refusal must not remove or reorder Pokemon");
        }
    }
}

#[test]
fn pc_declining_release_returns_to_the_release_submenu_row() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None;
    pokemon.mail = None;
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    mark_runtime_snapshot_dirty(&mut shell);
    shell.storage_cursor = Some(MenuCursor {
        surface_id: storage_cursor_surface_id(0), option_index: 0,
    });
    for cancel_with_b in [false, true] {
        open_visible_bill_pc_pokemon_actions(&mut shell).unwrap();
        move_visible_primary_cursor_down(&mut shell).unwrap();
        move_visible_primary_cursor_down(&mut shell).unwrap();
        assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0);
        press_visible_a_button(&mut shell).unwrap();
        assert!(shell.pending_pc_release.is_some());
        if cancel_with_b {
            press_visible_b_button(&mut shell).unwrap();
        } else {
            move_visible_primary_cursor_down(&mut shell).unwrap();
            move_visible_primary_cursor_down(&mut shell).unwrap();
            move_visible_primary_cursor_left(&mut shell).unwrap();
            move_visible_primary_cursor_right(&mut shell).unwrap();
            assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index, 1);
            assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0);
            assert_eq!(shell.shell.snapshot().unwrap().storage.current_pc_box, 0);
            press_visible_a_button(&mut shell).unwrap();
        }
        assert!(shell.pending_pc_release.is_none());
        assert_eq!(shell.bill_pc_pokemon_action_cursor.as_ref().map(|cursor| cursor.option_index),
            Some(2), "BillsPC_Withdraw.FailedRelease restores wMenuCursorY");
        assert_eq!(shell.shell.session().state().storage.pc_boxes[0].count, 1);
    }
}

#[test]
fn pokegear_contact_submenu_gates_calls_and_confirms_deletion() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    state.script_runtime.phone_number_order = vec![Some("PHONE_MOM".into()), Some("PHONE_ELM".into()), Some("PHONE_BILL".into())];
    state.script_runtime.phone_numbers = state.script_runtime.phone_number_order.iter().flatten().cloned().collect();
    mark_runtime_snapshot_dirty(&mut shell);
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Phone;
    inspect_visible_pokegear_selection(&mut shell).unwrap();
    assert!(shell.pokegear_phone_call.is_none(), "A must open the contact submenu before calling");
    assert!(!shell.pokegear_phone_menu.as_ref().unwrap().can_delete);
    move_visible_pokegear_cursor(&mut shell, 10).unwrap();
    assert_eq!(shell.pokegear_phone_menu.as_ref().unwrap().cursor, 1);
    cycle_visible_pokegear_page(&mut shell, -1).unwrap();
    assert_eq!(shell.pokegear_page, PokegearPage::Phone);
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokegear_menu_open);
    assert!(shell.pokegear_phone_menu.is_none());
    shell.pokegear_phone_cursor = 2;
    for accept in [false, true] {
        inspect_visible_pokegear_selection(&mut shell).unwrap();
        assert!(shell.pokegear_phone_menu.as_ref().unwrap().can_delete);
        move_visible_pokegear_cursor(&mut shell, 1).unwrap();
        inspect_visible_pokegear_selection(&mut shell).unwrap();
        assert_eq!(shell.pokegear_phone_menu.as_ref().unwrap().delete_confirmation, Some(0));
        if accept { inspect_visible_pokegear_selection(&mut shell).unwrap(); }
        else { press_visible_b_button(&mut shell).unwrap(); }
        assert!(shell.pokegear_phone_menu.is_none());
        assert_eq!(shell.shell.session().state().script_runtime.phone_numbers.contains("PHONE_BILL"), !accept);
    }
    assert_eq!(shell.pokegear_phone_cursor, 2);
}

#[test]
fn pokegear_returns_to_its_start_menu_cursor_but_scripted_map_does_not() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.start_menu_cursor = Some(MenuCursor {
        surface_id: START_MENU_SURFACE_ID.to_string(), option_index: 2,
    });
    open_visible_pokegear_menu(&mut shell).unwrap();
    shell.start_menu_cursor = None; // select_visible_start_menu_option suspends it.
    close_visible_pokegear_menu(&mut shell).unwrap();
    assert_eq!(shell.start_menu_cursor.as_ref().map(|cursor| cursor.option_index), Some(2));
    shell.start_menu_cursor = None;
    open_visible_pokegear_menu(&mut shell).unwrap();
    shell.pokegear_standalone_map = true;
    close_visible_pokegear_menu(&mut shell).unwrap();
    assert!(shell.start_menu_cursor.is_none());
}

#[test]
fn pokegear_clock_buttons_exit_and_card_navigation_stops_at_the_edges() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for button in [GameButton::A, GameButton::B, GameButton::Start, GameButton::Select] {
        shell.pokegear_menu_open = true;
        shell.pokegear_page = PokegearPage::Clock;
        match button {
            GameButton::A => press_visible_a_button(&mut shell),
            GameButton::B => press_visible_b_button(&mut shell),
            GameButton::Start => press_visible_start_button(&mut shell),
            GameButton::Select => press_visible_select_button(&mut shell),
            _ => unreachable!(),
        }.expect("clock button input");
        assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::Requested), "{button:?} must request the clock exit");
        close_visible_pokegear_menu(&mut shell).unwrap();
    }
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Clock;
    cycle_visible_pokegear_page(&mut shell, -1).expect("left edge");
    assert_eq!(shell.pokegear_page, PokegearPage::Clock);
}

#[test]
fn pokegear_contacts_keep_the_saved_phone_list_order() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().expect("phone snapshot");
    snapshot.script_events.phone_numbers = ["PHONE_ELM".into(), "PHONE_MOM".into()].into();
    snapshot.script_events.phone_number_order = vec![Some("PHONE_MOM".into()), Some("PHONE_ELM".into())];
    assert_eq!(visible_pokegear_phone_slots(&snapshot).unwrap().into_iter().flatten().collect::<Vec<_>>(), ["PHONE_MOM", "PHONE_ELM"],
        "wPhoneList order must not be replaced by alphabetical set order");
}

#[test]
fn missing_server_clock_pauses_gameplay_without_using_device_time() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell.intro_screen = None;
    runtime_shell.title_menu = None;
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Enter);
    let before = runtime_shell.shell.session().state().clone();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(NativeRtcSource::Server)
        .insert_resource(keys)
        .insert_resource(RuntimeTickTimer::new(0.0))
        .add_systems(Update, (apply_keyboard_input, apply_runtime_hotkeys).chain());
    app.update();
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session().state(), &before);
    assert_eq!(shell.latest_rtc_sample, None);
    assert!(shell.start_menu_cursor.is_none());
    assert_eq!(shell.last_error.as_deref(), Some(SERVER_CLOCK_UNAVAILABLE));
}

#[test]
fn pokegear_phone_preserves_empty_slots_and_scrolls_only_at_window_edges() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let runtime = &mut shell.shell.session_mut().state_mut().script_runtime;
    runtime.phone_numbers = ["PHONE_MOM".into(), "PHONE_ELM".into()].into();
    runtime.phone_number_order = vec![Some("PHONE_MOM".into()), None, Some("PHONE_ELM".into())];
    mark_runtime_snapshot_dirty(&mut shell);
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Phone;
    move_visible_pokegear_cursor(&mut shell, -1).expect("Up at first slot");
    assert_eq!(shell.pokegear_phone_cursor, 0);
    move_visible_pokegear_cursor(&mut shell, 1).expect("select empty slot");
    start_visible_pokegear_phone_call(&mut shell).expect("A on contact zero is ignored");
    assert!(shell.pokegear_phone_call.is_none());
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(visible_pokegear_phone_slots(&snapshot).unwrap()[2], Some("PHONE_ELM"));
    assert_eq!(visible_pokegear_phone_entries(&snapshot, &shell).unwrap()[2], ">----------");
    for _ in 0..3 { move_visible_pokegear_cursor(&mut shell, 1).unwrap(); }
    assert_eq!((shell.pokegear_phone_cursor, shell.pokegear_phone_scroll), (4, 1));
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    assert_eq!((shell.pokegear_phone_cursor, shell.pokegear_phone_scroll), (3, 1));
    for _ in 0..10 { move_visible_pokegear_cursor(&mut shell, 1).unwrap(); }
    assert_eq!((shell.pokegear_phone_cursor, shell.pokegear_phone_scroll), (9, 6));
}

#[test]
fn stats_source_print_level_uses_three_tiles_at_every_level() {
    assert_eq!(visible_print_level_text(5), "\u{e10a}5 ");
    assert_eq!(visible_print_level_text(10), "\u{e10a}10");
    assert_eq!(visible_print_level_text(100), "100");
    let mut map = SourceStatsLayout::default();
    stats_write_text(&mut map, 17, 14, &visible_print_level_text(100)).unwrap();
}

#[test]
fn stats_shiny_palette_loads_the_two_source_colors() {
    let root = progression_shell_on_map_for_test("LakeOfRage").asset_root;
    let palette = load_pokemon_palette(&root, "cyndaquil", PokemonSpriteSide::Front, true).unwrap();
    assert_eq!(palette, [[255, 255, 255],
        [normalize_palette_component(29), normalize_palette_component(23), normalize_palette_component(9)],
        [normalize_palette_component(22), 0, normalize_palette_component(19)], [0, 0, 0]]);
}

fn render_pc_audit_canvas(world: &mut World, images: &Assets<Image>, label: &str) -> image::RgbaImage {
        let mut query = world.query::<(&Sprite, &Transform, &Handle<Image>)>();
        let mut sprites = query.iter(&world).collect::<Vec<_>>();
        sprites.sort_by(|a, b| a.1.translation.z.total_cmp(&b.1.translation.z));
        let mut canvas = image::RgbaImage::new(PLAYFIELD_WIDTH as u32, PLAYFIELD_HEIGHT as u32);
        for (sprite, transform, handle) in sprites {
            let size = sprite.custom_size.expect("explicit sprite size");
            let left = transform.translation.x - size.x / 2.0 - PLAYFIELD_LEFT;
            let top = PLAYFIELD_TOP - transform.translation.y - size.y / 2.0;
            assert!(left >= -0.01 && left + size.x <= PLAYFIELD_WIDTH + 0.01,
                "{label}: horizontal clipping at {left} with width {}", size.x);
            assert!(top >= -0.01 && top + size.y <= PLAYFIELD_HEIGHT + 0.01,
                "{label}: vertical clipping at {top} with height {}", size.y);
            let mut raster = if let Some(source) = images.get(handle) {
                image::RgbaImage::from_raw(source.width(), source.height(), source.data.clone())
                    .expect("RGBA sprite")
            } else {
                // Bevy's default image handle is its solid white sprite texture.
                assert_eq!(*handle, Handle::<Image>::default(), "missing authored sprite");
                image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]))
            };
            if let Some(rect) = sprite.rect {
                raster = image::imageops::crop_imm(&raster,
                    rect.min.x as u32, rect.min.y as u32,
                    rect.width() as u32, rect.height() as u32).to_image();
            }
            let color = sprite.color.to_srgba();
            for pixel in raster.pixels_mut() {
                for (channel, tint) in pixel.0.iter_mut().zip([color.red, color.green, color.blue, color.alpha]) {
                    *channel = (f32::from(*channel) * tint).round() as u8;
                }
            }
            if sprite.flip_x { image::imageops::flip_horizontal_in_place(&mut raster); }
            if sprite.flip_y { image::imageops::flip_vertical_in_place(&mut raster); }
            let scaled = image::imageops::resize(&raster, size.x as u32, size.y as u32,
                image::imageops::FilterType::Nearest);
            image::imageops::overlay(&mut canvas, &scaled, left.round() as i64, top.round() as i64);
        }
    canvas
}

#[test]
fn stats_level_and_shiny_variants_render_without_clipping() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let snapshot = shell.shell.snapshot().unwrap();
    let data = shell.shell.runtime().data();
    for (variant, level, dvs) in [
        ("level5", 5, crate::core::models::Dv::default()),
        ("level100", 100, crate::core::models::Dv::default()),
        ("unown-z", 10, crate::core::models::Dv { attack: 6, defense: 6, speed: 6, special: 6, hp: 0 }),
        ("shiny", 10, crate::core::models::Dv { attack: 2, defense: 10, speed: 10, special: 10, hp: 0 }),
    ] {
        let species = if variant == "unown-z" {
            data.saved_species("UNOWN").unwrap()
        } else { snapshot.party.slots[0].pokemon.species.clone() };
        let mut pokemon = crate::core::models::pokemon::create_pokemon_from_known_dvs(
            &species, level, dvs,
            &data.learnsets, &data.moves, &data.growth_rates).unwrap();
        pokemon.item = None; pokemon.mail = None;
        pokemon.original_trainer_name = "CHRIS".into();
        pokemon.original_trainer_id = 0;
        for (page, name) in [(1, "pink"), (2, "green"), (3, "blue")] {
            let mut world = World::new();
            let mut images = Assets::<Image>::default();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let mut commands = Commands::new(&mut queue, &world);
            spawn_source_stats_screen(&mut commands, &snapshot, &shell, &pokemon,
                page, &mut RenderedTilesetArt::default(), &mut images, 4.1).unwrap();
            queue.apply(&mut world);
            let canvas = render_pc_audit_canvas(&mut world, &images, variant);
            if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
                let directory = PathBuf::from(directory).join(variant);
                std::fs::create_dir_all(&directory).unwrap();
                canvas.save(directory.join(format!("pc-stats-{name}.png"))).unwrap();
            }
        }
    }
}

#[test]
fn stats_observation_reports_the_active_pc_and_party_page() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut pokemon = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    pokemon.item = None; pokemon.mail = None;
    state.storage.party.pokemon[0] = Some(pokemon.clone());
    assert!(state.storage.pc_boxes[0].add_pokemon(pokemon));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut shell);
    shell.storage_cursor = Some(MenuCursor { surface_id: storage_cursor_surface_id(0), option_index: 0 });
    let snapshot = shell.shell.snapshot().unwrap();
    for page in 1..=3 {
        shell.bill_pc_pokemon_summary = Some(VisiblePcPokemonSummary {
            location: VisiblePcPokemonLocation::Box { box_index: 0, box_slot: 0 }, page,
        });
        shell.party_summary_page = page;
        let mut pc = Vec::new();
        push_visible_storage_dialog_entries(&mut pc, &snapshot, &shell).unwrap();
        assert_eq!(visible_scene_dialog_entries(&snapshot, &shell).unwrap(), pc,
            "the public scene observation must not truncate the Stats page to six rows");
        let party = visible_party_summary_entries(&snapshot, &shell).unwrap();
        for entries in [&pc, &party] {
            let text = entries.join("\n");
            assert!(!text.contains("CANCEL"), "Stats is not the underlying storage list: {text}");
            assert_eq!(text.contains("EXP POINTS"), page == 1, "{text}");
            assert_eq!(text.contains("ITEM"), page == 2, "{text}");
            assert_eq!(text.contains("SPCL.ATK"), page == 3, "{text}");
            assert!(!text.contains("HAP "), "hidden happiness is not visible Stats text");
        }
    }

    let mut egg_snapshot = snapshot.clone();
    egg_snapshot.storage.boxes[0].slots[0].pokemon.is_egg = true;
    egg_snapshot.storage.boxes[0].slots[0].pokemon.happiness = 20;
    let mut entries = Vec::new();
    push_visible_storage_dialog_entries(&mut entries, &egg_snapshot, &shell).unwrap();
    let text = entries.join("\n");
    assert!(text.contains("Wonder what's"), "{text}");
    assert!(text.contains("more time, though."), "{text}");
    assert!(!text.contains("SPCL.ATK"), "Egg hides the retained blue page");
    assert!(!text.contains("CYNDAQUIL"), "Egg must not expose its hidden species");
}

#[test]
fn pc_unown_portraits_load_all_dv_selected_forms() {
    use crate::core::models::{BaseStats, Dv, Pokemon, PokemonSpecies};
    let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let species = PokemonSpecies::new_for_tests("UNOWN", BaseStats {
        hp: 48, attack: 72, defense: 48, speed: 48, special_attack: 72, special_defense: 48,
    });
    let mut portraits = std::collections::HashSet::new();
    for letter in 0..26_u8 {
        let packed = letter * 10;
        let dvs = Dv { attack: ((packed >> 6) & 3) << 1,
            defense: ((packed >> 4) & 3) << 1, speed: ((packed >> 2) & 3) << 1,
            special: (packed & 3) << 1, hp: 0 };
        let pokemon = Pokemon::new_for_tests(species.clone(), 10, dvs);
        let mut images = Assets::<Image>::default();
        let picture = load_pc_pokemon_picture(&root, Some(&visible_pc_pokemon_info(&pokemon)),
            true, &mut images).expect("PC Stats must load the DV-selected Unown frontpic");
        assert!(portraits.insert(images.get(&picture.handle).unwrap().data.clone()),
            "Unown form {} repeated another form", letter + 1);
    }
}


#[test]
fn party_stats_suppresses_cries_for_eggs_fainted_frozen_and_asleep_pokemon() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let base = shell.shell.session_mut().state_mut().storage.party.pokemon[0].as_ref().unwrap().clone();
    for (egg, hp, status, expected_cry) in [
        (true, 10, None, false), (false, 0, None, false),
        (false, 10, Some("FREEZE"), false), (false, 10, Some("SLEEP"), false),
        (false, 10, None, true), (false, 10, Some("POISON"), true),
        (false, 10, Some("BURN"), true), (false, 10, Some("PARALYSIS"), true),
    ] {
        let mut pokemon = base.clone();
        pokemon.is_egg = egg; pokemon.hp = hp; pokemon.status = status.map(str::to_owned);
        pokemon.item = None; pokemon.mail = None;
        pokemon.sleep_turns = if status == Some("SLEEP") { 3 } else { 0 };
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(pokemon.clone());
        state.storage.party.pokemon[1] = Some(pokemon);
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut shell);
        shell.party_cursor = 0;
        shell.pending_audio.clear(); shell.transient_audio_playing = false;
        open_visible_party_summary(&mut shell).unwrap();
        assert_eq!(shell.pending_audio.iter().any(|audio| matches!(audio.kind, ModpackAudioKind::Cry)),
            expected_cry, "Stats opening: egg={egg} hp={hp} status={status:?}");
        shell.pending_audio.clear();
        move_visible_party_summary_pokemon(&mut shell, 1).unwrap();
        assert_eq!(shell.pending_audio.iter().any(|audio| matches!(audio.kind, ModpackAudioKind::Cry)),
            expected_cry, "Stats browsing: egg={egg} hp={hp} status={status:?}");
        close_visible_party_summary(&mut shell);
    }
}

#[test]
fn party_stats_exit_buttons_wait_for_pending_and_playing_cries() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.party_menu_open = true;
    for confirm in [false, true] {
        open_visible_party_summary(&mut shell).unwrap();
        shell.party_summary_page = 3;
        assert!(!shell.pending_audio.is_empty());
        let press = if confirm { press_visible_a_button } else { press_visible_b_button };
        press(&mut shell).unwrap();
        assert!(shell.party_summary_open, "exit during queued cry: A={confirm}");
        shell.pending_audio.clear();
        shell.transient_audio_playing = true;
        press(&mut shell).unwrap();
        assert!(shell.party_summary_open, "exit during playing cry: A={confirm}");
        shell.transient_audio_playing = false;
        press(&mut shell).unwrap();
        assert!(!shell.party_summary_open, "exit after cry: A={confirm}");
    }
}

#[test]
fn party_egg_stats_plays_boops_only_below_six_hatch_cycles() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut egg = shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_ref().unwrap().clone();
    egg.is_egg = true;
    egg.item = None;
    egg.mail = None;
    for happiness in [0, 5, 6, 10, 11, 20] {
        egg.happiness = happiness;
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(egg.clone());
        state.storage.party.pokemon[1] = Some(egg.clone());
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut shell);
        shell.party_cursor = 0;
        shell.pending_audio.clear();
        shell.transient_audio_playing = false;
        open_visible_party_summary(&mut shell).unwrap();
        for browsing in [false, true] {
            if browsing {
                shell.pending_audio.clear();
                move_visible_party_summary_pokemon(&mut shell, 1).unwrap();
            }
            assert_eq!(shell.pending_audio.iter()
                .filter(|audio| audio.audio_id == "SFX_2_BOOPS").count(),
                usize::from(happiness < 6), "happiness={happiness}, browsing={browsing}");
            assert!(!shell.pending_audio.iter()
                .any(|audio| matches!(audio.kind, ModpackAudioKind::Cry)));
        }
        close_visible_party_summary(&mut shell);
    }
}

#[test]
fn pc_egg_stats_plays_boops_and_waits_before_browsing_or_exiting() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let state = shell.shell.session_mut().state_mut();
    let mut egg = state.storage.party.pokemon[0].as_ref().unwrap().clone();
    egg.item = None;
    egg.mail = None;
    egg.is_egg = true;
    egg.happiness = 5;
    assert!(state.storage.pc_boxes[0].add_pokemon(egg.clone()));
    assert!(state.storage.pc_boxes[0].add_pokemon(egg));
    mark_runtime_snapshot_dirty(&mut shell);
    shell.storage_cursor = Some(MenuCursor {
        surface_id: storage_cursor_surface_id(0), option_index: 0,
    });
    shell.pending_audio.clear();
    open_visible_bill_pc_pokemon_actions(&mut shell).unwrap();
    shell.bill_pc_pokemon_action_cursor.as_mut().unwrap().option_index = 1;
    confirm_visible_bill_pc_pokemon_action(&mut shell).unwrap();
    assert_eq!(shell.pending_audio.iter().filter(|audio| audio.audio_id == "SFX_2_BOOPS").count(), 1);
    move_visible_pc_summary_pokemon(&mut shell, 1).unwrap();
    assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 0);
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_summary.is_some());
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_summary.is_some());
    shell.pending_audio.clear();
    shell.transient_audio_playing = false;
    move_visible_pc_summary_pokemon(&mut shell, 1).unwrap();
    assert_eq!(shell.storage_cursor.as_ref().unwrap().option_index, 1);
    assert_eq!(shell.pending_audio.iter().filter(|audio| audio.audio_id == "SFX_2_BOOPS").count(), 1);
    shell.pending_audio.clear();
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.bill_pc_pokemon_summary.is_none());
}

#[test]
fn mailbox_windows_match_original_rom_lcd() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mail = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().mail.clone().unwrap();
    shell.shell.session_mut().state_mut().mailbox = (0..6).map(|index| {
        let mut mail = mail.clone();
        mail.author = format!("MAIL{index}");
        crate::core::state::MailboxMail { item_id: "FLOWER_MAIL".into(), mail }
    }).collect();
    mark_runtime_snapshot_dirty(&mut shell);
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/asm-oracle/fixtures/mailbox");
    let mut differences = Vec::new();
    for index in 0usize..9 {
        let label = if index < 7 { format!("mailbox-{index}") }
            else if index == 7 { "mailbox-actions".into() } else { "mailbox-confirm".into() };
        shell.mailbox_scroll = if index < 7 { index.saturating_sub(3) } else { 3 };
        shell.mailbox_cursor = Some(MenuCursor {
            surface_id: "pc:mailbox".into(), option_index: if index < 7 { index } else { 5 },
        });
        shell.mailbox_action_cursor = (index >= 7).then(|| MenuCursor {
            surface_id: "pc:mailbox-actions".into(), option_index: if index == 7 { 0 } else { 1 },
        });
        if index == 8 {
            confirm_visible_mailbox_action(&mut shell).unwrap();
            let question = shell.pc_notice.clone().unwrap();
            shell.field_text_reveal = Some(VisibleFieldTextReveal {
                text: question.clone(), page_index: 0, visible_chars: question.chars().count(), frames_until_next_char: 0,
            });
        }
        let mut snapshot = shell.shell.snapshot().unwrap();
        snapshot.overworld.map_name = "OlivinePokecenter1F".into();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        let mut commands = Commands::new(&mut queue, &world);
        let palette = source_map_text_palette(&snapshot, &shell.asset_root).unwrap();
        // The oracle deliberately clears the parent backdrop to isolate menus.
        commit_presented_fullscreen_solid(&mut commands, &mut art,
            [palette[0][0], palette[0][1], palette[0][2], 255], 3.0, &mut images).unwrap();
        spawn_scene_dialog(&mut commands, &snapshot, &shell, &mut art, &shell.asset_root, &mut images).unwrap();
        queue.apply(&mut world);
        assert_eq!(art.font_error, None, "{label}");
        let canvas = render_pc_audit_canvas(&mut world, &images, &label);
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            canvas.save(PathBuf::from(directory).join(format!("{label}.png"))).unwrap();
        }
        let rom = image::open(reference.join(format!("rom-{label}.png"))).unwrap().to_rgba8();
        assert_eq!(rom.dimensions(), (160, 144));
        let scale = canvas.width() / 160;
        assert_eq!(canvas.dimensions(), (160 * scale, 144 * scale));
        let count = canvas.enumerate_pixels().filter(|(x, y, pixel)| {
            let expected = rom.get_pixel(x / scale, y / scale);
            pixel[3] != 255 || (0..3).any(|channel| pixel[channel] >> 3 != expected[channel] >> 3)
        }).count();
        differences.push((label, count));
    }
    assert!(differences.iter().all(|(_, count)| *count == 0), "mailbox RGB5 differences: {differences:?}");
}

#[test]
fn unown_solved_cancel_box_uses_the_source_void_tile() {
    let mut strip = image::RgbaImage::from_pixel(152, 8, image::Rgba([32, 32, 32, 255]));
    for y in 0..8 { for x in 16..24 {
        strip.put_pixel(x, y, image::Rgba([248, 248, 248, 255]));
    }}
    let sources = UnownPuzzleRenderSources {
        pieces: HashMap::from([("kabuto".into(), vec![image::RgbaImage::new(24, 24); 16])]),
        cursor: image::RgbaImage::new(16, 16), start_cancel: strip,
    };
    let puzzle = VisibleUnownPuzzle {
        puzzle_id: "KABUTO".into(), layout: [[0; 6]; 6], holding_piece: None,
        cursor_x: 0, cursor_y: 0, solved: true,
    };
    let mut images = Assets::<Image>::default();
    let frame = render_visible_unown_puzzle_frame(&sources, &puzzle, false, &mut images).unwrap();
    let pixels = &images.get(&frame.handle).unwrap().data;
    for y in 128..136 { for x in 40..120 {
        let offset = (y * 160 + x) * 4;
        assert_eq!(&pixels[offset..offset + 4], &[248, 248, 248, 255],
            "PlaceStartCancelBoxBorder fills the cleared lettering with PUZZLE_VOID");
    }}
}

fn menu_render_test_app(shell: BevyRuntimeShell) -> App {
    let mut app = integrated_shell_test_app(shell);
    // Browser packs contain MIDI music. These native LCD tests exercise
    // rendering and input, without invoking the browser synthesizer.
    app.add_systems(
        Update,
        (|mut shell: ResMut<BevyRuntimeShell>| {
            shell.pending_audio.clear();
            shell.transient_audio_playing = false;
        })
        .after(queue_battle_intro_cry)
        .before(play_pending_audio),
    );
    app
}

fn save_live_menu_lcd_for_test(world: &mut World, name: &str) {
    let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") else {
        return;
    };
    let sprites = world.query_filtered::<(&Sprite, &Transform, &Handle<Image>),
        Or<(With<FieldCommandMarker>, With<VisibleIntroSurface>)>>()
        .iter(world).map(|(sprite, transform, image)| (sprite.clone(), *transform, image.clone()))
        .collect::<Vec<_>>();
    let mut lcd = World::new();
    for sprite in sprites {
        lcd.spawn(sprite);
    }
    let canvas = render_pc_audit_canvas(&mut lcd, world.resource::<Assets<Image>>(), name);
    std::fs::create_dir_all(&directory).unwrap();
    canvas.save(PathBuf::from(directory).join(name)).unwrap();
}

#[test]
fn party_and_stats_retain_the_lcd_through_live_input_and_idle_frames() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .unwrap()
        .item = None;
    shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .unwrap()
        .mail = None;
    shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    let mut app = menu_render_test_app(shell);
    app.update();
    app.update();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    assert!(app.world().resource::<BevyRuntimeShell>().party_menu_open);
    let retained = retained_fullscreen_surface(app.world_mut());
    {
        let art = app.world().resource::<RenderedTilesetArt>();
        let frame = art.intro_presented_surface.as_ref().unwrap();
        let pixels = &app
            .world()
            .resource::<Assets<Image>>()
            .get(&frame.handle)
            .unwrap()
            .data;
        assert!(
            pixels.chunks_exact(4).all(|pixel| pixel[3] == 255),
            "party background must be fully opaque"
        );
        assert!(
            pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[..3] == [255, 255, 255])
                .count()
                > 160 * 144 / 2,
            "the white LCD background must remain behind the Pokemon icon"
        );
        assert!(
            pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[..3] == [0, 0, 0])
                .count()
                > 50,
            "the retained LCD must contain menu text and borders, not only an OAM sprite"
        );
    }
    save_live_menu_lcd_for_test(app.world_mut(), "pokemon-menu.png");
    for _ in 0..3 {
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .party_action_cursor
            .is_some()
    );
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .party_summary_open
    );
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    // Finish the cry before the source StatsScreenWaitCry joypad boundary.
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.pending_audio.clear();
        shell.transient_audio_playing = false;
    }
    for page in [2, 3, 1] {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .party_summary_page,
            page
        );
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
        save_live_menu_lcd_for_test(app.world_mut(), &format!("pokemon-stats-{page}.png"));
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    assert!(
        !app.world()
            .resource::<BevyRuntimeShell>()
            .party_summary_open
    );
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    app.update();
    assert!(!app.world().resource::<BevyRuntimeShell>().party_menu_open);
    assert_eq!(
        selected_visible_start_menu_option(&mut app.world_mut().resource_mut::<BevyRuntimeShell>())
            .unwrap(),
        StartMenuOption::Pokemon,
        "CloseSubmenu returns to the selected START menu row"
    );
    assert!(
        app.world()
            .resource::<RenderedTilesetArt>()
            .presented_fullscreen_entity
            .is_none()
    );
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
}

#[test]
fn party_raster_matches_asm_egg_level_hp_and_submenu_coordinates() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    let font =
        crate::open_runtime_image(shell.asset_root.runtime_assets().join("gfx/font/font.png"))
            .unwrap()
            .to_rgba8();
    let tile = |image: &[u8], x: usize, y: usize| -> Vec<u8> {
        (y * 8..y * 8 + 8)
            .flat_map(|row| {
                image[(row * 160 + x * 8) * 4..(row * 160 + x * 8 + 8) * 4]
                    .iter()
                    .copied()
            })
            .collect()
    };
    let mut expected = vec![255; 160 * 144 * 4];
    draw_time_set_text(&font, "EGG", 24, 8, &mut expected).unwrap();
    snapshot.party.slots[0].pokemon.is_egg = true;
    snapshot.party.slots[0].pokemon.nickname = "EGG".into();
    let save_frame = |images: &Assets<Image>, frame: &SpriteFrame, name: &str| {
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            let directory = PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            let image = images.get(&frame.handle).unwrap();
            image::RgbaImage::from_raw(image.width(), image.height(), image.data.clone())
                .unwrap()
                .save(directory.join(name))
                .unwrap();
        }
    };
    let mut images = Assets::<Image>::default();
    let frame = load_visible_field_party_frame(&snapshot, &shell, 0, &mut images).unwrap();
    save_frame(&images, &frame, "party-egg.png");
    let pixels = &images.get(&frame.handle).unwrap().data;
    for x in 3..6 {
        assert_eq!(
            tile(pixels, x, 1),
            tile(&expected, x, 1),
            "PlacePartyNicknames includes Eggs"
        );
    }
    for x in 5..20 {
        assert_eq!(
            tile(pixels, x, 2),
            vec![255; 256],
            "Egg rows omit status, level, and HP"
        );
    }

    let frame = load_visible_field_party_frame(&snapshot, &shell, 1, &mut images).unwrap();
    let pixels = &images.get(&frame.handle).unwrap().data;
    expected.fill(255);
    draw_time_set_text(&font, "▶CANCEL", 0, 3 * 8, &mut expected).unwrap();
    for x in 0..7 {
        assert_eq!(
            tile(pixels, x, 3),
            tile(&expected, x, 3),
            "PartyMenu2DMenuData keeps the CANCEL cursor at column zero"
        );
    }

    snapshot.party.slots[0].pokemon.is_egg = false;
    snapshot.party.slots[0].pokemon.level = 100;
    snapshot.party.slots[0].pokemon.hp = snapshot.party.slots[0].pokemon.max_hp;
    let frame = load_visible_field_party_frame(&snapshot, &shell, 0, &mut images).unwrap();
    save_frame(&images, &frame, "party-level100.png");
    let pixels = &images.get(&frame.handle).unwrap().data;
    expected.fill(255);
    draw_time_set_text(&font, "100", 64, 16, &mut expected).unwrap();
    for x in 8..11 {
        assert_eq!(
            tile(pixels, x, 2),
            tile(&expected, x, 2),
            "PlacePartyMonLevel replaces LV at 100"
        );
    }
    for x in 5..8 {
        assert_eq!(
            tile(pixels, x, 2),
            vec![255; 256],
            "PlaceNonFaintStatus leaves healthy status blank"
        );
    }
    // DrawBattleHPBar puts HP: at columns 11/12 and the end cap at 19.
    let extra = crate::read_runtime_asset(
        shell
            .asset_root
            .runtime_assets()
            .join("gfx/font/font_battle_extra.2bpp"),
    )
    .unwrap();
    let colors = stats_palette_colors(&shell.asset_root, "gfx/battle/hp_bar.pal").unwrap();
    let palette = [[255, 255, 255], colors[0], colors[1], [0, 0, 0]];
    for (x, id) in [(11, 0x60), (12, 0x61), (13, 0x6a), (19, 0x6b)] {
        draw_paletted_2bpp_tile(&extra, id - 0x60, &palette, x, 2, &mut expected).unwrap();
        assert_eq!(
            tile(pixels, x, 2),
            tile(&expected, x, 2),
            "source HP tile at {x}"
        );
    }
    shell.party_action_cursor = Some(MenuCursor {
        surface_id: "party:actions".into(),
        option_index: 0,
    });
    let actions = visible_party_actions(&snapshot, &shell).unwrap();
    let top = 18 - 2 * (actions.len() + 1);
    let frame = load_visible_field_party_frame(&snapshot, &shell, 0, &mut images).unwrap();
    save_frame(&images, &frame, "party-actions.png");
    let pixels = &images.get(&frame.handle).unwrap().data;
    expected.fill(255);
    draw_time_set_text(&font, "STATS", 64, (top + 2) * 8, &mut expected).unwrap();
    for x in 8..13 {
        assert_eq!(
            tile(pixels, x, top + 2),
            tile(&expected, x, top + 2),
            "MonSubmenu.GetTopCoord bottom-aligns actions"
        );
    }
}

#[test]
fn party_stats_joypad_is_edge_only_and_uses_asm_button_priority() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_party_menu(&mut shell).unwrap();
    open_visible_party_summary(&mut shell).unwrap();
    shell.pending_audio.clear();
    shell.transient_audio_playing = false;
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ArrowRight);
    keys.press(KeyCode::KeyZ);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(
        shell.party_summary_page, 2,
        "StatsScreen_JoypadAction gives Right priority over A"
    );
    keys.clear();
    for _ in 0..40 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
    }
    assert_eq!(
        shell.party_summary_page, 2,
        "StatsScreen_GetJoypad reads hJoyPressed without menu repeat"
    );
    assert!(shell.party_summary_open);
    keys.reset_all();
    keys.press(KeyCode::KeyX);
    keys.press(KeyCode::KeyZ);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(!shell.party_summary_open);
    assert!(
        shell.party_menu_open,
        "one input sample must not also close the parent party menu"
    );
}

#[test]
fn trainer_status_retains_its_lcd_during_live_page_changes() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut app = menu_render_test_app(shell);
    app.update();
    app.update();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let snapshot = shell.shell.snapshot().unwrap();
        let index = visible_start_menu_options(&shell, &snapshot)
            .iter()
            .position(|option| *option == StartMenuOption::TrainerCard)
            .unwrap();
        shell.start_menu_cursor.as_mut().unwrap().option_index = index;
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    assert!(app.world().resource::<BevyRuntimeShell>().trainer_card_open);
    save_live_menu_lcd_for_test(app.world_mut(), "trainer-status.png");
    let retained = retained_fullscreen_surface(app.world_mut());
    for _ in 0..3 {
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
    assert_eq!(
        app.world().resource::<BevyRuntimeShell>().trainer_card_page,
        VisibleTrainerCardPage::JohtoBadges
    );
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowLeft);
    assert_eq!(
        app.world().resource::<BevyRuntimeShell>().trainer_card_page,
        VisibleTrainerCardPage::Info
    );
    assert_retained_fullscreen_surface(app.world_mut(), &retained);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    app.update();
    assert!(!app.world().resource::<BevyRuntimeShell>().trainer_card_open);
    assert_eq!(
        selected_visible_start_menu_option(&mut app.world_mut().resource_mut::<BevyRuntimeShell>())
            .unwrap(),
        StartMenuOption::TrainerCard
    );
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
}

#[test]
fn party_give_item_keeps_start_closed_until_the_party_menu_exits() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    toggle_visible_start_menu(&mut shell).unwrap();
    select_visible_start_menu_option(&mut shell).unwrap();
    assert!(shell.party_menu_open);
    assert!(shell.party_return_start_menu_cursor.is_some());
    shell.party_give_take_cursor = Some(MenuCursor {
        surface_id: "party:give-take".into(),
        option_index: 0,
    });
    confirm_visible_party_give_take(&mut shell).unwrap();
    assert!(visible_field_pack_is_open(&shell));
    assert!(
        shell.start_menu_cursor.is_none(),
        "GIVE must not render START over the Pack"
    );
    assert!(shell.party_return_start_menu_cursor.is_some());
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.party_menu_open);
    assert!(shell.start_menu_cursor.is_none());
    press_visible_b_button(&mut shell).unwrap();
    assert!(!shell.party_menu_open);
    assert_eq!(
        selected_visible_start_menu_option(&mut shell).unwrap(),
        StartMenuOption::Pokemon
    );
}

#[test]
fn party_give_item_success_returns_to_party_with_the_start_cursor_retained() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .unwrap();
    pokemon.item = None;
    pokemon.mail = None;
    shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    shell.shell.add_bag_item("BERRY", 1).unwrap();
    toggle_visible_start_menu(&mut shell).unwrap();
    select_visible_start_menu_option(&mut shell).unwrap();
    shell.party_give_take_cursor = Some(MenuCursor {
        surface_id: "party:give-take".into(),
        option_index: 0,
    });
    confirm_visible_party_give_take(&mut shell).unwrap();
    give_selected_held_item(&mut shell).unwrap();
    assert!(shell.party_menu_open);
    assert!(shell.start_menu_cursor.is_none());
    assert!(shell.party_return_start_menu_cursor.is_some());
    assert!(!visible_field_pack_is_open(&shell));
    assert_eq!(
        shell.shell.session().state().storage.party.pokemon[0]
            .as_ref()
            .unwrap()
            .item
            .as_deref(),
        Some("BERRY")
    );
}

#[test]
fn field_pack_keeps_its_visible_lcd_across_pockets_actions_and_idle_frames() {
    let mut shell = progression_shell_on_map_for_test("Route36");
    shell.shell.add_bag_item("SQUIRTBOTTLE", 1).unwrap();
    let mut app = menu_render_test_app(shell);
    app.update();
    app.update();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowDown);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    assert!(visible_field_pack_is_open(app.world().resource::<BevyRuntimeShell>()));
    let retained = retained_fullscreen_surface(app.world_mut());
    for key in [None, Some(KeyCode::ArrowRight), None, Some(KeyCode::ArrowRight),
        Some(KeyCode::KeyZ), None, Some(KeyCode::KeyX), None] {
        if let Some(key) = key { press_key_for_runtime_hotkey_app(&mut app, key); }
        for _ in 0..3 {
            app.update();
            assert_retained_fullscreen_surface(app.world_mut(), &retained);
        }
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(visible_field_pack_is_open(shell));
    assert_eq!(active_visible_field_pack_pocket(shell), FieldPackPocket::KeyItems);
    save_live_menu_lcd_for_test(app.world_mut(), "field-pack-key-items.png");
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    assert!(!visible_field_pack_is_open(app.world().resource::<BevyRuntimeShell>()));
}
