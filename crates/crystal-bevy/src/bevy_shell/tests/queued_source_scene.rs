// Future queued source-only animations must not replace the currently visible
// HP/result presentation. These tests use the production message/recall staging
// helpers and the existing integrated controller app for the terminal KO path.
fn queued_source_scene_faint_fixture() -> (BevyRuntimeShell, RuntimeShellSnapshot) {
    let mut shell = immersive_battle_fixture();
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.battle.as_mut().unwrap().enemy_pokemon.hp = 0;
    stage_visible_battle_messages(
        &mut shell,
        &snapshot,
        &[crate::core::battle::turn::BattleEvent::Fainted {
            side: crate::core::battle::turn::BattleSide::Enemy,
        }],
    );
    assert_eq!(shell.visible_move_animations.len(), 1);
    assert!(!shell.visible_move_animations.front().unwrap().started);
    (shell, snapshot)
}

fn queued_source_scene_guard_matrix(shell: &BevyRuntimeShell, expected: bool) {
    let animations = shell.visible_move_animations.clone();
    let capture = shell.visible_capture_animation.clone();
    let send_out = shell.visible_send_out_animation.clone();
    let trainer_exit = shell.visible_trainer_exit_animation.clone();
    for allow_rows in [false, true] {
        for allow_capture in [false, true] {
            assert_eq!(
                immersive_battle_requires_source_scene_supported(shell, allow_rows, allow_capture),
                expected,
                "allow_rows={allow_rows}, allow_capture={allow_capture}, animations={animations:?}"
            );
        }
    }
    assert_eq!(shell.visible_move_animations, animations);
    assert_eq!(shell.visible_capture_animation, capture);
    assert_eq!(shell.visible_send_out_animation, send_out);
    assert_eq!(shell.visible_trainer_exit_animation, trainer_exit);
}

fn queued_source_scene_extract_unchanged(
    shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
) -> VisualBattleFrame {
    let authority = shell.shell.snapshot().unwrap();
    let checksum = shell.shell.state_checksum().unwrap();
    let divider = shell.shell.session().divider_for_tests().clone();
    let commands_before = shell.shell.retained_runtime_commands().len();
    let animations = shell.visible_move_animations.clone();
    let hp = shell.battle_hp_tween;
    let messages = shell.battle_messages.clone();
    let text = shell.battle_text_reveal.clone();
    let retained = shell.battle_message_scene.clone();
    let audio = shell.pending_audio.clone();
    let mut world = World::new();
    for _ in 0..2 {
        let prior = world.get_resource::<VisualBattleFrame>().cloned();
        let next_tick = art
            .battle_object_runtime
            .as_ref()
            .map(|playback| playback.next_tick);
        let mut queue = bevy::ecs::world::CommandQueue::default();
        capture_presented_battle(
            &mut Commands::new(&mut queue, &world),
            snapshot,
            shell,
            true,
            art,
            images,
        )
        .unwrap();
        queue.apply(&mut world);
        if let Some(prior) = prior {
            assert_eq!(world.resource::<VisualBattleFrame>(), &prior);
            assert_eq!(
                art.battle_object_runtime
                    .as_ref()
                    .map(|playback| playback.next_tick),
                next_tick,
                "extracting a held sample twice must not advance the source interpreter"
            );
        }
    }
    assert_eq!(shell.shell.snapshot().unwrap(), authority);
    assert_eq!(shell.shell.state_checksum().unwrap(), checksum);
    assert_eq!(shell.shell.session().divider_for_tests(), &divider);
    assert_eq!(
        shell.shell.retained_runtime_commands().len(),
        commands_before
    );
    assert_eq!(shell.visible_move_animations, animations);
    assert_eq!(shell.battle_hp_tween, hp);
    assert_eq!(shell.battle_messages, messages);
    assert_eq!(shell.battle_text_reveal, text);
    assert_eq!(shell.battle_message_scene, retained);
    assert_eq!(shell.pending_audio, audio);
    world.remove_resource::<VisualBattleFrame>().unwrap()
}

#[test]
fn queued_source_scene_faint_waits_through_zero_hp_and_effectiveness_text() {
    let (mut shell, snapshot) = queued_source_scene_faint_fixture();
    shell
        .battle_messages
        .push_front("It's super effective!".into());
    shell.battle_message_scene = Some(Arc::new(snapshot.clone()));
    let player = &snapshot.party.slots[0].pokemon;
    shell.battle_hp_tween = Some(VisibleBattleHpTween {
        player_hp: player.hp,
        player_target_hp: player.hp,
        player_max_hp: player.max_hp,
        player_pixels: 48,
        player_target_pixels: 48,
        player_frames_until_step: 0,
        enemy_pixels: 48,
        enemy_target_pixels: 0,
        enemy_frames_until_step: 1,
    });
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for pixels in [48, 1, 0] {
        shell.battle_hp_tween.as_mut().unwrap().enemy_pixels = pixels;
        queued_source_scene_guard_matrix(&shell, false);
        let frame = queued_source_scene_extract_unchanged(&shell, &snapshot, &mut art, &mut images);
        assert!(frame.active && !frame.use_source_scene);
        assert!(
            frame.battlers[1].as_ref().unwrap().visible,
            "zero HP must retain the enemy until its real faint begins"
        );
        assert!(frame.source.is_none());
        assert!(
            !frame
                .cues
                .iter()
                .any(|cue| cue.kind == VisualBattleCueKind::Faint)
        );
        assert_eq!(
            frame
                .cues
                .iter()
                .any(|cue| cue.kind == VisualBattleCueKind::Impact
                    && cue.side == VisualBattleSide::Enemy),
            pixels > 0
        );
        assert!(
            art.battle_object_runtime.is_none(),
            "a queued faint must not start its source clock"
        );
    }
    shell.visible_move_animations.front_mut().unwrap().started = true;
    for tick in [0, 6, 12] {
        shell.visible_move_animations.front_mut().unwrap().frame = tick;
        queued_source_scene_guard_matrix(&shell, true);
        let frame = queued_source_scene_extract_unchanged(&shell, &snapshot, &mut art, &mut images);
        assert!(frame.active && frame.use_source_scene);
        assert!(
            frame
                .cues
                .iter()
                .any(|cue| cue.kind == VisualBattleCueKind::Faint)
        );
        assert!(
            !frame
                .cues
                .iter()
                .any(|cue| cue.kind == VisualBattleCueKind::Impact)
        );
        assert_eq!(frame.source.as_ref().unwrap().frame, tick);
        let rows = visible_move_battler_row_extractions(shell.visible_move_animations.front());
        assert_eq!(rows.1.unwrap().rows, (tick / 2 + 1) as u8);
    }
}

#[test]
fn queued_source_scene_recall_and_label_alias_wait_until_started() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    queue_visible_player_recall_animation(&mut shell, &snapshot, "CYNDAQUIL, come back!");
    let recall = shell.visible_move_animations.front().unwrap().clone();
    assert_eq!(recall.move_id, "RETURN_MON");
    for (move_id, label) in [
        ("RETURN_MON", "BattleAnim_ReturnMon"),
        ("RETURN_MON", "RECALL_ID_ONLY"),
        ("RECALL_LABEL_ALIAS", "BattleAnim_ReturnMon"),
    ] {
        for started in [false, true] {
            let mut animation = recall.clone();
            animation.move_id = move_id.into();
            animation.animation_label = label.into();
            animation.started = started;
            // Recall's source tile operations are delayed by 50 ticks. Even
            // frame zero must retain the intentional fallback once it starts.
            shell.visible_move_animations = VecDeque::from([animation]);
            queued_source_scene_guard_matrix(&shell, started);
        }
    }
}

#[test]
fn queued_source_scene_preserves_capture_send_out_and_trainer_exit_gates() {
    let (mut shell, _) = queued_source_scene_faint_fixture();
    shell.visible_capture_animation = Some(capture_clock_fixture("POKE_BALL", true, 4));
    let capture = shell.visible_capture_animation.clone();
    for allow_rows in [false, true] {
        for allow_capture in [false, true] {
            assert_eq!(
                immersive_battle_requires_source_scene_supported(&shell, allow_rows, allow_capture),
                !allow_capture,
                "only the explicit capture allowance may bypass capture's independent gate"
            );
        }
    }
    assert_eq!(shell.visible_capture_animation, capture);
    shell.visible_capture_animation = None;
    shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
        side: crate::core::battle::turn::BattleSide::Player,
        frame: 0,
        shiny: false,
    });
    queued_source_scene_guard_matrix(&shell, true);
    shell.visible_send_out_animation = None;
    shell.visible_trainer_exit_animation = Some(VisibleTrainerExitAnimation {
        side: crate::core::battle::turn::BattleSide::Enemy,
        frame: 0,
        send_out_after: false,
    });
    queued_source_scene_guard_matrix(&shell, true);
}

#[test]
fn queued_source_scene_real_ko_changes_fallback_only_when_faint_starts() {
    let mut shell = immersive_battle_fixture();
    shell.battle_hp_tween = None;
    {
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0].as_mut().unwrap().moves =
            vec![crate::core::models::LearnedMove {
                name: "SWIFT".into(),
                current_pp: 20,
                pp_ups: 0,
            }];
        state.sync_party_from_storage();
        let crate::core::state::BattleMemory::StaticWild {
            enemy_pokemon,
            enemy_party,
            ..
        } = &mut state.battle
        else {
            panic!("static wild fixture");
        };
        enemy_pokemon.hp = 1;
        enemy_pokemon.moves = vec![crate::core::models::LearnedMove {
            name: "SPLASH".into(),
            current_pp: 40,
            pp_ups: 0,
        }];
        enemy_party[0] = enemy_pokemon.clone();
        state.script_runtime.active_battle_combat = None;
    }
    mark_runtime_snapshot_dirty(&mut shell);
    shell.battle_message_scene = Some(Arc::new(shell.shell.snapshot().unwrap()));
    resolve_visible_battle_move(&mut shell, 0).unwrap();
    assert!(
        shell
            .visible_move_animations
            .iter()
            .any(|animation| animation.move_id == "FAINT_MON")
    );
    let mut app = menu_render_test_app(shell);
    app.update();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut saw_pending = false;
    let mut saw_result = false;
    let mut saw_started = false;
    let mut saw_completed = false;
    for _ in 0..2000 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        let faint = shell
            .visible_move_animations
            .front()
            .filter(|animation| animation.move_id == "FAINT_MON");
        if let Some(faint) = faint {
            let snapshot = shell
                .battle_message_scene
                .as_ref()
                .expect("retained KO scene");
            if snapshot.battle.as_ref().unwrap().enemy_pokemon.hp == 0 {
                queued_source_scene_guard_matrix(shell, faint.started);
                if !faint.started {
                    let result = shell
                        .battle_messages
                        .front()
                        .is_some_and(|message| message.contains("effective"));
                    if !saw_pending || (result && !saw_result) {
                        let frame = queued_source_scene_extract_unchanged(
                            shell,
                            snapshot,
                            &mut art,
                            &mut images,
                        );
                        assert!(frame.active && !frame.use_source_scene);
                        assert!(frame.battlers[1].as_ref().unwrap().visible);
                        assert!(frame.source.is_none());
                    }
                    saw_pending = true;
                    saw_result |= result;
                } else if !saw_started {
                    assert!(
                        !shell
                            .battle_hp_tween
                            .as_ref()
                            .is_some_and(visible_battle_hp_tween_active)
                    );
                    let frame = queued_source_scene_extract_unchanged(
                        shell,
                        snapshot,
                        &mut art,
                        &mut images,
                    );
                    assert!(frame.active && frame.use_source_scene);
                    assert!(
                        frame
                            .cues
                            .iter()
                            .any(|cue| cue.kind == VisualBattleCueKind::Faint)
                    );
                    saw_started = true;
                }
            }
        } else if saw_started
            && !shell
                .visible_move_animations
                .iter()
                .any(|animation| animation.move_id == "FAINT_MON")
        {
            let snapshot = shell
                .battle_message_scene
                .as_ref()
                .expect("retained reward scene");
            let frame =
                queued_source_scene_extract_unchanged(shell, snapshot, &mut art, &mut images);
            assert!(frame.active && !frame.use_source_scene);
            assert!(!frame.battlers[1].as_ref().unwrap().visible);
            assert!(shell.battle_fainted_hud[1]);
            saw_completed = true;
            break;
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    assert!(
        saw_pending && saw_result && saw_started && saw_completed,
        "real KO boundaries: pending={saw_pending}, result={saw_result}, started={saw_started}, completed={saw_completed}"
    );
}
