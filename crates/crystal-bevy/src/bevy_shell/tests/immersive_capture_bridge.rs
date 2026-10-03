#[test]
fn immersive_capture_metadata_uses_source_four_tick_picture_phases() {
    for (ball, start) in [("POKE_BALL", 71), ("MASTER_BALL", 144)] {
        let mut capture = capture_clock_fixture(ball, true, 4);
        for (age, expected) in [
            (-1, VisualCapturePicture::Full), (0, VisualCapturePicture::Tiles(7)),
            (3, VisualCapturePicture::Tiles(7)), (4, VisualCapturePicture::Tiles(5)),
            (7, VisualCapturePicture::Tiles(5)), (8, VisualCapturePicture::Tiles(3)),
            (11, VisualCapturePicture::Tiles(3)), (12, VisualCapturePicture::Hidden),
        ] {
            capture.frame = (start + age) as u16;
            let sample = visible_capture_present_state(&capture);
            assert_eq!(sample.enemy_picture, expected, "{ball} tick {}", capture.frame);
            assert_eq!(sample.frame, capture.frame);
            assert_eq!(sample.ball_id.as_ref(), ball);
        }
    }
    let mut failure = capture_clock_fixture("POKE_BALL", false, 3);
    for (tick, expected) in [
        (436, VisualCapturePicture::Hidden), (437, VisualCapturePicture::Tiles(3)),
        (440, VisualCapturePicture::Tiles(3)), (441, VisualCapturePicture::Tiles(5)),
        (444, VisualCapturePicture::Tiles(5)), (445, VisualCapturePicture::Tiles(7)),
        (448, VisualCapturePicture::Tiles(7)), (449, VisualCapturePicture::Full),
    ] {
        failure.frame = tick;
        assert_eq!(visible_capture_present_state(&failure).enemy_picture, expected);
    }
    failure.blocked = true;
    for tick in 0..55 {
        failure.frame = tick;
        assert_eq!(visible_capture_present_state(&failure).enemy_picture, VisualCapturePicture::Full);
    }
}

#[test]
fn immersive_capture_source_lifetime_survives_expansion_cue_gap_and_retained_return() {
    for (ball, caught, blocked) in [("POKE_BALL", false, false), ("MASTER_BALL", true, false),
        ("GREAT_BALL", false, true)] {
        let mut shell = immersive_battle_fixture();
        let mut capture = capture_clock_fixture(ball, caught, if caught { 4 } else { 3 });
        capture.blocked = blocked;
        let snapshot = shell.shell.snapshot().unwrap();
        *shell.shell.session_mut().divider_mut_for_tests() =
            crate::core::random::RuntimeDividerSource::replay([0x1a, 0x2b]);
        let divider = shell.shell.session().divider_for_tests().clone();
        let checksum = shell.shell.state_checksum().unwrap();
        let commands_before = shell.shell.retained_runtime_commands().len();
        shell.pending_audio.clear();
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let ticks = if blocked { vec![0, 21, 54] }
            else if caught { vec![0, 37, 79, 144, 148, 152, 156, 507, 508, 530] }
            else { vec![0, 37, 71, 75, 79, 83, 434, 437, 441, 445, 449, 470] };
        for tick in ticks {
            capture.frame = tick;
            capture.complete = caught && tick >= capture.total_frames();
            capture.started = !capture.complete;
            shell.visible_capture_animation = Some(capture.clone());
            let mut world = World::new();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            capture_presented_battle(&mut Commands::new(&mut queue, &world),
                &snapshot, &shell, true, &mut art, &mut images).unwrap();
            queue.apply(&mut world);
            let frame = world.resource::<VisualBattleFrame>();
            assert_eq!(frame.capture.as_ref().unwrap(), &visible_capture_present_state(&capture));
            let source = frame.source.as_ref().expect("retained capture must always publish its source frame");
            assert_eq!(source.frame, capture.object_frame());
            assert_eq!(source.battler_offsets, [Vec2::ZERO; 2]);
            assert_eq!(source.battler_bgps, [0xe4; 2]);
            assert_eq!(frame.battlers[1].as_ref().unwrap().visible, !capture.enemy_hidden());
            if !caught && !blocked && tick >= 437 {
                assert!(!frame.cues.iter().any(|cue| cue.kind == VisualBattleCueKind::Capture));
                assert!(frame.source.is_some(), "cue retirement cannot end source publication");
            }
            // Extraction twice at one sample must neither restart nor advance
            // the shared object machine, including retained anim_ret palette.
            let first = source.clone();
            let next_tick = art.battle_object_runtime.as_ref().unwrap().next_tick;
            let descriptor = visible_capture_source_animation(&capture);
            let repeated = capture_immersive_source_frame_with_capture(&snapshot, &descriptor,
                &frame.battlers, &mut art, &shell.asset_root, &mut images, Some(&capture)).unwrap();
            assert_eq!(repeated, first);
            assert_eq!(art.battle_object_runtime.as_ref().unwrap().next_tick, next_tick);
            assert_eq!(shell.visible_capture_animation.as_ref().unwrap(), &capture);
        }
        assert_eq!(shell.shell.state_checksum().unwrap(), checksum);
        assert_eq!(shell.shell.session().divider_for_tests(), &divider);
        assert_eq!(shell.shell.retained_runtime_commands().len(), commands_before);
        assert!(shell.pending_audio.is_empty(), "render extraction cannot queue source sounds");
    }
}

#[test]
fn immersive_capture_bridge_keeps_all_other_source_fallback_reasons() {
    let mut shell = immersive_battle_fixture();
    shell.visible_capture_animation = Some(capture_clock_fixture("POKE_BALL", true, 4));
    assert!(immersive_battle_requires_source_scene(&shell));
    assert!(!immersive_battle_requires_source_scene_supported(&shell, false, true));
    shell.visible_trainer_exit_animation = Some(VisibleTrainerExitAnimation {
        side: crate::core::battle::turn::BattleSide::Enemy, frame: 0, send_out_after: false,
    });
    assert!(immersive_battle_requires_source_scene_supported(&shell, false, true));
}

#[cfg(feature = "location-tester")]
#[test]
fn immersive_capture_bridge_keeps_real_queued_enemy_response_eligible_until_it_starts() {
    for blocked in [false, true] {
        let mut controller = immersive_battle_move_and_ball_preview_controller(
            false, false, false, false, false, blocked, !blocked,
        );
        let before = controller.shell.shell.snapshot().unwrap();
        let hp_before = capture_presentation_player_hp(&before);
        let reads_before = capture_fixture_divider_counts(&controller).0;
        let enemy_pp_before = before.battle.as_ref().unwrap().enemy_moves[1].current_pp;
        capture_fixture_throw_from_pack(&mut controller, "POKE_BALL");
        let after = controller.shell.shell.snapshot().unwrap();
        let hp_after = capture_presentation_player_hp(&after);
        capture_fixture_assert_finite_enemy_response(&controller, reads_before, enemy_pp_before);
        assert_eq!(after.battle.as_ref().unwrap().enemy_turns_taken, 1);
        let queued = controller.shell.visible_move_animations.clone();
        assert!(!queued.is_empty(), "the real failed/blocked Ball must queue its response");
        assert!(queued.iter().all(|animation| !animation.started));
        let response = queued.front().unwrap();
        assert!(!response.player_move);
        if blocked {
            assert_eq!(response.move_id, "GUST");
            assert!(hp_after < hp_before, "Vance's actual Gust must deal damage");
        } else {
            // Sudowoodo's selected slot is Mimic. This first Ball turn gives
            // it no player move to copy, so the real response consumes PP,
            // preserves HP and queues the source's failed-move delay.
            assert_eq!(before.battle.as_ref().unwrap().enemy_moves[1].name, "MIMIC");
            assert_eq!(response.move_id, "MIMIC");
            assert_eq!(response.animation_label, "BattleCommand_MoveDelay");
            assert_eq!(response.total_frames, 40);
            assert_eq!(hp_after, hp_before);
        }
        let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
        assert!(!capture.caught);
        assert_eq!(capture.blocked, blocked);
        let total = capture.total_frames();
        let expansion = capture.expansion_start_frame();
        let samples = [0, 21, 37, 71, 75, 79, 83,
            expansion, expansion + 4, expansion + 8, expansion + 12, total - 1];
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        // Sample the pending used-item page, then advance the same native
        // dispatcher through actual throw/return ticks. Never clear the queue
        // or inject capture outcomes/frames to make renderer admission pass.
        for step in 0..=total {
            if step == 1 {
                apply_visible_shell_smoke_frame(&mut controller.shell, &[GameButton::A]).unwrap();
            }
            let capture = controller.shell.visible_capture_animation.as_ref().unwrap().clone();
            assert_eq!(controller.shell.visible_move_animations, queued);
            capture_presentation_assert_held_hp(&controller.shell, hp_before);
            if step == 0 || samples.contains(&capture.frame) {
                // render_playfield selects this retained pre-response scene,
                // not the authoritative post-turn snapshot inspected above.
                let scene = controller.shell.battle_message_scene.as_ref().unwrap().clone();
                assert_eq!(scene.battle.as_ref().unwrap().enemy_turns_taken, 0);
                let checksum = controller.shell.shell.state_checksum().unwrap();
                let divider = controller.shell.shell.session().divider_for_tests().clone();
                let audio = controller.shell.last_audio_events.clone();
                let pending_audio = controller.shell.pending_audio.len();
                let commands = controller.shell.shell.retained_runtime_commands().len();
                let mut world = World::new();
                let mut queue = bevy::ecs::world::CommandQueue::default();
                capture_presented_battle(&mut Commands::new(&mut queue, &world),
                    &scene, &controller.shell, true, &mut art, &mut images).unwrap();
                queue.apply(&mut world);
                let frame = world.resource::<VisualBattleFrame>();
                // Test the pure source eligibility independently of the
                // process-wide opt-in; no unsafe environment mutation is needed.
                assert!(immersive_capture_source_supported(&controller.shell, &scene, &frame.battlers));
                assert!(!immersive_battle_requires_source_scene_supported(&controller.shell, false, true));
                assert_eq!(frame.use_source_scene, !immersive_capture_prototype_enabled());
                assert_eq!(frame.capture.as_ref().unwrap(), &visible_capture_present_state(&capture));
                assert!(!frame.cues.iter().any(|cue| cue.kind == VisualBattleCueKind::Move));
                if capture.started {
                    assert_eq!(frame.source.as_ref().unwrap().frame, capture.object_frame());
                    assert_eq!(art.battle_object_runtime.as_ref().unwrap().move_id, "THROW_POKE_BALL");
                } else {
                    assert!(frame.source.is_none());
                }
                assert_eq!(controller.shell.visible_capture_animation.as_ref().unwrap(), &capture);
                assert_eq!(controller.shell.visible_move_animations, queued);
                assert_eq!(controller.shell.shell.state_checksum().unwrap(), checksum);
                assert_eq!(controller.shell.shell.session().divider_for_tests(), &divider);
                assert_eq!(controller.shell.last_audio_events, audio);
                assert_eq!(controller.shell.pending_audio.len(), pending_audio);
                assert_eq!(controller.shell.shell.retained_runtime_commands().len(), commands);
            }
            if step > 0 {
                advance_visible_battle_animation_frame(&mut controller.shell).unwrap();
            }
        }
        assert!(controller.shell.visible_capture_animation.is_none());
        assert_eq!(controller.shell.visible_move_animations, queued);
        capture_presentation_assert_held_hp(&controller.shell, hp_before);

        // Let ordinary narration/input start the original queued response.
        // A started move still fails the capture-only support predicate.
        let trigger = &queued.front().unwrap().trigger_message;
        for _ in 0..16 {
            if controller.shell.battle_messages.front() == Some(trigger) { break; }
            controller.press(GameButton::A).unwrap();
        }
        assert_eq!(controller.shell.battle_messages.front(), Some(trigger));
        capture_presentation_finish_text_without_input(&mut controller.shell);
        apply_visible_shell_smoke_frame(&mut controller.shell, &[GameButton::A]).unwrap();
        assert!(controller.shell.visible_move_animations.front().unwrap().started);
        let scene = controller.shell.battle_message_scene.as_ref().unwrap().clone();
        let frame = extract_immersive_battle_fixture(&controller.shell, &scene, true);
        assert!(!immersive_capture_source_supported(&controller.shell, &scene, &frame.battlers));
        assert!(frame.capture.is_none());
        capture_fixture_return_to_commands(&mut controller);
        assert_eq!(capture_presentation_player_hp(&controller.shell.shell.snapshot().unwrap()), hp_after);
        assert!(controller.shell.last_error.is_none());
    }
}
