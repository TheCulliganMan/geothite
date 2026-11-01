// The command update itself precedes the requested wait. These expectations
// are manual source-clock traces, not schedules produced by the compiler under
// test. Original graphics and command programs remain in the external pack.
#[test]
fn battle_anim_clock_wait_zero_and_final_wait_keep_the_return_update() {
    let shell = route36_battle_shell_for_render_regression();
    let mut snapshot = shell.shell.snapshot().unwrap();
    for wait in [0_u16, 1, 207] {
        Arc::make_mut(&mut snapshot.presentation)
            .battle_animations
            .insert(
                "BattleAnim_ClockWait".into(),
                vec![
                    "anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 0, 0, 0".into(),
                    format!("anim_wait {wait}"),
                    "anim_ret".into(),
                ],
            );
        let (_, completion, _, _, objects, _) =
            visible_battle_animation_definition(&snapshot, "BattleAnim_ClockWait".into(), 0)
                .unwrap();
        assert_eq!(objects[0].frame, 1);
        assert_eq!(completion - 1, wait + 2, "last source return update");
        assert_eq!(completion, wait + 3, "exclusive completion boundary");
    }
}

#[test]
fn battle_anim_clock_calls_and_loops_only_yield_on_source_waits() {
    let shell = route36_battle_shell_for_render_regression();
    let mut snapshot = shell.shell.snapshot().unwrap();
    Arc::make_mut(&mut snapshot.presentation)
        .battle_animations
        .insert(
            "BattleAnim_ClockChild".into(),
            vec![
                ".repeat".into(),
                "anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 0, 0, 0".into(),
                "anim_wait 1".into(),
                "anim_loop 2, .repeat".into(),
                "anim_ret".into(),
            ],
        );
    Arc::make_mut(&mut snapshot.presentation)
        .battle_animations
        .insert(
            "BattleAnim_ClockCaller".into(),
            vec![
                "anim_call BattleAnim_ClockChild".into(),
                "anim_call BattleAnim_ClockChild".into(),
                "anim_ret".into(),
            ],
        );
    let (_, completion, _, _, objects, _) =
        visible_battle_animation_definition(&snapshot, "BattleAnim_ClockCaller".into(), 0).unwrap();
    assert_eq!(
        objects
            .iter()
            .map(|object| object.frame)
            .collect::<Vec<_>>(),
        [1, 3, 5, 7]
    );
    assert_eq!(completion, 10);
}

#[test]
fn battle_anim_clock_gust_matches_original_spawn_sound_and_return_updates() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let (_, completion, sounds, cries, objects, _) =
        visible_move_animation_definition(&snapshot, "GUST", 0).unwrap();
    let wind_ticks = [1, 8, 15, 22, 29, 36, 43, 50, 57];
    assert_eq!(completion, 91);
    assert!(cries.is_empty());
    assert_eq!(objects.len(), 11);
    for (event, frame) in objects.iter().zip(wind_ticks) {
        assert_eq!(event.frame, frame);
        assert_eq!(
            event.command,
            VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_GUST".into(),
                x: 136,
                y: 72,
                param: 0,
            }
        );
    }
    for (event, (frame, x, y)) in objects[9..].iter().zip([(64, 144, 64), (73, 128, 32)]) {
        assert_eq!(event.frame, frame);
        assert_eq!(
            event.command,
            VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_HIT_YFIX".into(),
                x,
                y,
                param: 0x18,
            }
        );
    }
    assert_eq!(
        sounds.iter().map(|(frame, _)| *frame).collect::<Vec<_>>(),
        wind_ticks
    );
    for (_, sound) in sounds {
        assert_eq!(sound.id, "SFX_RAZOR_WIND");
        assert_eq!(sound.args, Some(BattleSoundArgs::new(0, 1)));
    }
}

#[test]
fn battle_anim_clock_known_bare_scripts_keep_exclusive_completion_boundaries() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    for (move_id, completion) in [
        ("TACKLE", 33),
        ("HEADBUTT", 66),
        ("FLAIL", 55),
        ("THIEF", 120),
        ("WATER_GUN", 127),
        ("SHADOW_BALL", 60),
        ("PSYCHIC_M", 176),
        ("HYPER_BEAM", 66),
        ("SURF", 191),
        ("GUST", 91),
        ("RECOVER", 84),
    ] {
        assert_eq!(
            visible_move_animation_definition(&snapshot, move_id, 0)
                .unwrap()
                .1,
            completion,
            "{move_id}"
        );
    }
}

#[cfg(feature = "voxel-view")]
#[test]
fn battle_anim_clock_gust_return_reaches_the_bridge_without_mutating_gameplay() {
    for player_move in [true, false] {
        let mut shell = immersive_battle_fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let checksum = shell.shell.state_checksum().unwrap();
        let (label, total, sounds, cries, objects, bg) =
            visible_move_animation_definition(&snapshot, "GUST", 0).unwrap();
        let mut animation = battle_anim_regression_timeline(objects, 0);
        animation.move_id = "GUST".into();
        animation.animation_label = label;
        animation.total_frames = total;
        animation.sound_events = sounds;
        animation.cry_events = cries;
        animation.bg_events = bg;
        animation.player_move = player_move;
        shell.visible_move_animations.push_back(animation);
        for _ in 0..90 {
            advance_visible_move_animation(&mut shell).unwrap();
        }
        assert_eq!(shell.visible_move_animations.front().unwrap().frame, 90);
        let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
        let source = frame.source.as_ref().unwrap();
        assert_eq!(source.frame, 90);
        assert!(
            !source.objects.is_empty(),
            "return update still owns source OAM"
        );
        advance_visible_move_animation(&mut shell).unwrap();
        assert!(shell.visible_move_animations.is_empty());
        assert_eq!(shell.shell.state_checksum().unwrap(), checksum);
    }
}

#[test]
fn battle_anim_clock_source_fix_does_not_extend_synthetic_fixed_delays() {
    for total in [1_u16, 14, 60, u16::MAX] {
        let mut shell = route36_battle_shell_for_render_regression();
        shell.visible_move_animations.clear();
        shell.visible_move_audio_wait = None;
        shell.battle_messages.clear();
        let checksum = shell.shell.state_checksum().unwrap();
        let mut animation = battle_anim_regression_timeline(Vec::new(), total - 1);
        animation.total_frames = total;
        shell.visible_move_animations.push_back(animation);
        advance_visible_move_animation(&mut shell).unwrap();
        assert!(
            shell.visible_move_animations.is_empty(),
            "fixed delay {total}"
        );
        assert_eq!(shell.shell.state_checksum().unwrap(), checksum);
    }
}

// Capture expectations are independently traced command ticks from
// pret/pokecrystal 5beda23ffa505f62e1dad7e3d7c214d1737b3358. No program or
// graphics data is embedded. Throw is tick zero; completion excludes anim_ret.
fn capture_clock_fixture(ball: &str, caught: bool, shakes: u8) -> VisibleCaptureAnimation {
    VisibleCaptureAnimation {
        trigger_message: String::new(),
        ball_id: ball.into(),
        animation_shakes: shakes,
        blocked: false,
        caught,
        started: true,
        complete: false,
        sprites_cleared: false,
        frame: 0,
    }
}

#[test]
fn capture_clock_success_matches_every_source_milestone_and_sound() {
    for ball in ["POKE_BALL", "GREAT_BALL", "ULTRA_BALL", "MASTER_BALL"] {
        let capture = capture_clock_fixture(ball, true, 4);
        let master = ball == "MASTER_BALL";
        let expected = if master {
            [37, 54, 144, 153, 170, 203, 311, 360, 409, 458, 507, 508]
        } else {
            [37, 54, 71, 80, 97, 130, 238, 287, 336, 385, 434, 435]
        };
        assert_eq!(
            [
                capture.opening_lid_frame(),
                capture.initial_poof_frame(),
                capture.shake_entry_frame(),
                capture.lid_increment_frame(),
                capture.change_dex_sound_frame(),
                capture.bounce_sound_frame(),
                capture.shake_setup_frame(),
                capture.shake_check_frame(0),
                capture.shake_check_frame(1),
                capture.shake_check_frame(2),
                capture.return_frame(),
                capture.total_frames(),
            ],
            expected,
            "{ball}: matching only Master completion misses the cancelled errors"
        );
        assert_eq!(capture.master_ball_special_frame(), master.then_some(79));
        let events = capture.object_events();
        let increments = events
            .iter()
            .filter_map(|event| match event.command {
                VisibleMoveObjectCommand::Increment { index } => Some((event.frame, index)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            increments,
            [
                (expected[3], 2),
                (expected[4], 1),
                (expected[7], 1),
                (expected[8], 1),
                (expected[9], 1),
            ],
            "{ball}: the fourth check must not wobble"
        );
        let poofs = events
            .iter()
            .filter_map(|event| match &event.command {
                VisibleMoveObjectCommand::Spawn { object_id, .. }
                    if object_id == "BATTLE_ANIM_OBJ_BALL_POOF" =>
                {
                    Some(event.frame)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(poofs, [54]);
        let lid = events
            .iter()
            .filter(|event| event.frame == 37)
            .collect::<Vec<_>>();
        assert_eq!(lid.len(), 2);
        assert!(
            matches!(&lid[0].command, VisibleMoveObjectCommand::Spawn { object_id, .. }
            if object_id == "BATTLE_ANIM_OBJ_POKE_BALL")
        );
        assert_eq!(
            lid[1].command,
            VisibleMoveObjectCommand::Set { index: 2, value: 7 }
        );
        let sparkles = events
            .iter()
            .filter_map(|event| match &event.command {
                VisibleMoveObjectCommand::Spawn {
                    object_id, param, ..
                } if object_id == "BATTLE_ANIM_OBJ_MASTER_BALL_SPARKLE" => {
                    Some((event.frame, *param))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let expected_sparkles = if master {
            (0x30..0x38).map(|param| (79, param)).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        assert_eq!(sparkles, expected_sparkles);
        let sounds = (0..=capture.total_frames())
            .filter_map(|frame| capture.sound_at_frame(frame).map(|sound| (frame, sound)))
            .collect::<Vec<_>>();
        let mut expected_sounds = vec![(54, "SFX_BALL_POOF")];
        if master {
            expected_sounds.push((79, "SFX_MASTER_BALL"));
        }
        expected_sounds.extend([
            (expected[4], "SFX_CHANGE_DEX_MODE"),
            (expected[5], "SFX_BALL_BOUNCE"),
            (expected[7], "SFX_BALL_WOBBLE"),
            (expected[8], "SFX_BALL_WOBBLE"),
            (expected[9], "SFX_BALL_WOBBLE"),
        ]);
        assert_eq!(sounds, expected_sounds, "{ball}");
    }
}

#[test]
fn capture_clock_failures_and_blocked_throw_keep_return_then_completion() {
    for (shakes, failure, expand, returned, completion) in [
        (0, 287, 290, 323, 324),
        (1, 336, 339, 372, 373),
        (2, 385, 388, 421, 422),
        (3, 434, 437, 470, 471),
    ] {
        let capture = capture_clock_fixture("POKE_BALL", false, shakes);
        assert_eq!(capture.break_free_frame(), failure);
        assert_eq!(capture.expansion_start_frame(), expand);
        assert_eq!(capture.return_frame(), returned);
        assert_eq!(capture.total_frames(), completion);
        let events = capture.object_events();
        let escape = events
            .iter()
            .filter(|event| event.frame == failure)
            .collect::<Vec<_>>();
        assert_eq!(escape.len(), 2);
        assert_eq!(
            escape[0].command,
            VisibleMoveObjectCommand::Set {
                index: 1,
                value: 11
            }
        );
        assert!(
            matches!(&escape[1].command, VisibleMoveObjectCommand::Spawn { object_id, .. }
            if object_id == "BATTLE_ANIM_OBJ_BALL_POOF")
        );
        assert_eq!(capture.sound_at_frame(failure), Some("SFX_BALL_POOF"));
        let wobble_ticks = (0..=completion)
            .filter(|frame| capture.sound_at_frame(*frame) == Some("SFX_BALL_WOBBLE"))
            .collect::<Vec<_>>();
        assert_eq!(wobble_ticks, [287, 336, 385][..usize::from(shakes)]);
    }
    let mut blocked = capture_clock_fixture("POKE_BALL", false, 0);
    blocked.blocked = true;
    assert_eq!(blocked.blocked_hit_frame(), 21);
    assert_eq!(blocked.return_frame(), 54);
    assert_eq!(blocked.total_frames(), 55);
    let events = blocked.object_events();
    assert_eq!(
        events.iter().map(|event| event.frame).collect::<Vec<_>>(),
        [0, 21]
    );
    assert!(
        matches!(&events[1].command, VisibleMoveObjectCommand::Spawn { object_id, .. }
        if object_id == "BATTLE_ANIM_OBJ_HIT_YFIX")
    );
    for frame in 0..=55 {
        blocked.frame = frame;
        assert!(!blocked.enemy_hidden());
        assert_eq!(blocked.enemy_clip_tiles(), None);
        assert_eq!(blocked.sound_at_frame(frame), None);
    }
}

#[test]
fn capture_clock_resize_phases_each_hold_four_command_ticks() {
    for (ball, shrink) in [("POKE_BALL", 71), ("MASTER_BALL", 144)] {
        let mut capture = capture_clock_fixture(ball, true, 4);
        capture.frame = shrink - 1;
        assert!(!capture.enemy_hidden());
        assert_eq!(capture.enemy_clip_tiles(), None);
        for (age, tiles) in [7, 7, 7, 7, 5, 5, 5, 5, 3, 3, 3, 3].into_iter().enumerate() {
            capture.frame = shrink + age as u16;
            assert!(!capture.enemy_hidden(), "{ball} age {age}");
            assert_eq!(capture.enemy_clip_tiles(), Some(tiles), "{ball} age {age}");
        }
        for age in 12..=17 {
            capture.frame = shrink + age;
            assert!(capture.enemy_hidden(), "{ball} age {age}");
            assert_eq!(capture.enemy_clip_tiles(), None);
        }
    }
    for (shakes, expand) in [(0, 290), (1, 339), (2, 388), (3, 437)] {
        let mut capture = capture_clock_fixture("POKE_BALL", false, shakes);
        capture.frame = expand - 1;
        assert!(capture.enemy_hidden());
        assert_eq!(capture.enemy_clip_tiles(), None);
        for (age, tiles) in [3, 3, 3, 3, 5, 5, 5, 5, 7, 7, 7, 7].into_iter().enumerate() {
            capture.frame = expand + age as u16;
            assert!(!capture.enemy_hidden(), "{shakes} wobbles, age {age}");
            assert_eq!(capture.enemy_clip_tiles(), Some(tiles));
        }
        capture.frame = expand + 12;
        assert!(!capture.enemy_hidden());
        assert_eq!(capture.enemy_clip_tiles(), None, "EnterMon has ended");
    }
}

#[test]
fn capture_clock_advance_queues_exact_sounds_without_gameplay_or_rng_mutation() {
    for (ball, caught, shakes, blocked, completion) in [
        ("POKE_BALL", true, 4, false, 435),
        ("GREAT_BALL", true, 4, false, 435),
        ("ULTRA_BALL", true, 4, false, 435),
        ("MASTER_BALL", true, 4, false, 508),
        ("POKE_BALL", false, 0, false, 324),
        ("POKE_BALL", false, 1, false, 373),
        ("POKE_BALL", false, 2, false, 422),
        ("POKE_BALL", false, 3, false, 471),
        ("POKE_BALL", false, 0, true, 55),
    ] {
        let mut shell = route36_battle_shell_for_render_regression();
        let mut capture = capture_clock_fixture(ball, caught, shakes);
        capture.blocked = blocked;
        shell.visible_capture_animation = Some(capture.clone());
        shell.pending_audio.clear();
        // Replay must not even consume a divider sample, regardless of whether
        // equal RNG registers or a broad state checksum could hide a read.
        *shell.shell.session_mut().divider_mut_for_tests() =
            crate::core::random::RuntimeDividerSource::replay([0x1a, 0x2b]);
        let divider = shell.shell.session().divider_for_tests().clone();
        let checksum = shell.shell.state_checksum().unwrap();
        let messages = shell.battle_messages.clone();
        let command_count = shell.shell.retained_runtime_commands().len();
        let mut sounds = Vec::new();
        for frame in 1..=completion {
            advance_visible_capture_animation(&mut shell).unwrap();
            sounds.extend(
                shell
                    .pending_audio
                    .drain(..)
                    .map(|command| (frame, command.audio_id)),
            );
            if frame < completion {
                let live = shell.visible_capture_animation.as_ref().unwrap();
                assert_eq!(live.frame, frame);
                assert!(live.started && !live.complete);
            }
        }
        if caught {
            let live = shell.visible_capture_animation.as_ref().unwrap();
            assert_eq!(live.frame, completion);
            assert_eq!(live.object_frame(), completion - 1);
            assert!(!live.started && live.complete && !live.sprites_cleared);
            assert_eq!(live.animation_shakes, 4);
        } else {
            assert!(shell.visible_capture_animation.is_none());
        }
        let mut expected = if blocked {
            Vec::new()
        } else {
            let master = ball == "MASTER_BALL";
            let mut result = vec![(54, "SFX_BALL_POOF".to_string())];
            if master {
                result.push((79, "SFX_MASTER_BALL".to_string()));
            }
            result.extend([
                (
                    if master { 170 } else { 97 },
                    "SFX_CHANGE_DEX_MODE".to_string(),
                ),
                (
                    if master { 203 } else { 130 },
                    "SFX_BALL_BOUNCE".to_string(),
                ),
            ]);
            let checks = if master {
                [360, 409, 458, 507]
            } else {
                [287, 336, 385, 434]
            };
            let wobbles = if caught { 3 } else { usize::from(shakes) };
            result.extend(
                checks[..wobbles]
                    .iter()
                    .map(|frame| (*frame, "SFX_BALL_WOBBLE".to_string())),
            );
            if !caught {
                result.push((checks[usize::from(shakes)], "SFX_BALL_POOF".to_string()));
            }
            result
        };
        if caught {
            expected.push((completion, "SFX_CAUGHT_MON".to_string()));
        }
        assert_eq!(
            sounds, expected,
            "{ball} caught={caught} shakes={shakes} blocked={blocked}"
        );
        advance_visible_capture_animation(&mut shell).unwrap();
        assert!(
            shell.pending_audio.is_empty(),
            "completed playback must be inert"
        );
        assert_eq!(shell.shell.state_checksum().unwrap(), checksum);
        assert_eq!(shell.shell.session().divider_for_tests(), &divider);
        assert_eq!(shell.shell.retained_runtime_commands().len(), command_count);
        assert_eq!(shell.battle_messages, messages);
    }
}

#[test]
fn capture_clock_object_extraction_repeats_and_rewinds_without_post_return_callbacks() {
    for ball in ["POKE_BALL", "MASTER_BALL"] {
        let mut full_result = None;
        for mode in [
            crystal_render_api::BattleFlashMode::Full,
            crystal_render_api::BattleFlashMode::Reduced,
        ] {
            let mut shell = route36_battle_shell_for_render_regression();
            shell.visible_battle_transition = None;
            shell.battle_entry_messages_remaining = 0;
            shell.battle_enemy_send_out_pending = false;
            shell.battle_player_send_out_pending = false;
            shell.battle_messages.clear();
            shell.battle_message_scenes.clear();
            let mut capture = capture_clock_fixture(ball, true, 4);
            let returned = if ball == "MASTER_BALL" { 507 } else { 434 };
            capture.frame = returned;
            shell.visible_capture_animation = Some(capture);
            let checksum = shell.shell.state_checksum().unwrap();
            let mut app = battle_render_regression_app(shell);
            app.insert_resource(mode);
            app.update();
            let object_bytes = |app: &App| {
                let playback = app
                    .world()
                    .resource::<RenderedTilesetArt>()
                    .battle_object_runtime
                    .as_ref()
                    .unwrap();
                (
                    playback.next_tick,
                    (0..10)
                        .map(|slot| playback.machine.object(slot).to_vec())
                        .collect::<Vec<_>>(),
                )
            };
            let at_return = object_bytes(&app);
            assert_eq!(at_return.0, u32::from(returned) + 1);
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                advance_visible_capture_animation(&mut shell).unwrap();
            }
            app.update();
            assert_eq!(
                object_bytes(&app),
                at_return,
                "completion cannot run one more callback"
            );
            for frame in [returned + 1, 37, returned, returned + 1] {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                shell.visible_capture_animation.as_mut().unwrap().frame = frame;
                mark_runtime_presentation_dirty(&mut shell);
                drop(shell);
                app.update();
                assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
                if frame >= returned {
                    assert_eq!(object_bytes(&app), at_return);
                }
            }
            assert_eq!(
                app.world()
                    .resource::<BevyRuntimeShell>()
                    .shell
                    .state_checksum()
                    .unwrap(),
                checksum
            );
            if let Some(full_result) = &full_result {
                assert_eq!(
                    &at_return, full_result,
                    "Full/Reduced must share source object timing"
                );
            } else {
                full_result = Some(at_return);
            }
        }
    }
}
