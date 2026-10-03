// These fixtures reach PACK through VisibleShellController and retain the real
// Ball action's outcome. No capture flags, object cues, or result are injected.
fn capture_fixture_ball_count(snapshot: &RuntimeShellSnapshot, ball: &str) -> u16 {
    snapshot.bag.balls.iter().find(|item| item.item_id == ball)
        .map_or(0, |item| item.quantity)
}

fn capture_fixture_throw_from_pack(controller: &mut VisibleShellController, ball: &str) {
    assert!(controller.shell.quick_save_path.is_none());
    let before = controller.shell.shell.snapshot().unwrap();
    let count = capture_fixture_ball_count(&before, ball);
    let actions_before = controller.shell.deterministic_battle_actions.len();
    assert!(count > 0);
    assert!(controller.shell.visible_capture_animation.is_none());
    controller.press(GameButton::Down).unwrap(); // FIGHT -> PACK
    let menu = controller.shell.shell.snapshot().unwrap();
    assert_eq!(selected_visible_battle_action_readonly(&controller.shell, &menu, menu.battle.as_ref().unwrap()).unwrap(), VisibleBattleAction::Pack);
    controller.press(GameButton::A).unwrap(); // open PACK, Items pocket
    controller.press(GameButton::Right).unwrap(); // Balls pocket
    let ids = carried_ball_item_ids(&controller.shell.shell.snapshot().unwrap());
    assert_eq!(ids, ["POKE_BALL", "MASTER_BALL"]);
    let index = ids.iter().position(|id| id == ball).unwrap();
    for _ in 0..index {
        controller.press(GameButton::Down).unwrap();
    }
    controller.press(GameButton::A).unwrap(); // item action menu
    assert!(controller.shell.field_pack_action_cursor.is_some());
    assert_eq!(capture_fixture_ball_count(&controller.shell.shell.snapshot().unwrap(), ball), count);
    controller.press(GameButton::A).unwrap(); // USE: real Ball turn + enemy AI
    assert_eq!(controller.shell.deterministic_battle_actions.len(), actions_before + 1);
    assert_eq!(controller.shell.deterministic_battle_actions.back().unwrap().action(),
        &BattleAction::Ball { item_id: ball.into() });
    let after = controller.shell.shell.snapshot().unwrap();
    assert_eq!(capture_fixture_ball_count(&after, ball), count - 1);
    assert_eq!(after.party.slots.len(), before.party.slots.len(),
        "storage insertion remains deferred to the real post-catch flow");
    let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
    assert_eq!(capture.ball_id, ball);
    assert!(!capture.started, "the normal used-item page still owns input");
    assert!(controller.shell.last_error.is_none());
}

#[derive(Default)]
struct CaptureFixturePresentation {
    saw_ball_identity: bool,
    saw_sprite: bool,
    saw_hidden_enemy: bool,
    sparkle_params: std::collections::BTreeSet<u8>,
    wobble_sounds: usize,
}

fn capture_fixture_present_real_throw(
    controller: &mut VisibleShellController,
    ball: &str,
    blocked: bool,
) -> CaptureFixturePresentation {
    let snapshot = controller.snapshot().unwrap();
    // Controller::press settles an uninterruptible animation to the next input
    // boundary. Use its SAME per-frame input dispatcher for this final A, then
    // the native capture clock so actual intermediate render output is sampled.
    // The item action itself above goes through Controller::press throughout.
    apply_visible_shell_smoke_frame(&mut controller.shell, &[GameButton::A]).unwrap();
    assert!(controller.shell.visible_capture_animation.as_ref().unwrap().started);
    let sounds_before = controller.shell.last_audio_events.iter()
        .filter(|event| event.as_str() == "queued sound effect SFX_BALL_WOBBLE").count();
    let mut result = CaptureFixturePresentation::default();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for _ in 0..600 {
        if !controller.shell.visible_capture_animation.as_ref()
            .is_some_and(VisibleCaptureAnimation::throw_active) {
            break;
        }
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            capture_presented_battle(&mut commands, &snapshot, &controller.shell, true,
                &mut art, &mut images).unwrap();
            spawn_visible_move_animation_objects(&mut commands, &snapshot, &controller.shell,
                &mut art, &controller.shell.asset_root, &mut images).unwrap();
        }
        queue.apply(&mut world);
        let frame = world.resource::<VisualBattleFrame>();
        let expected_kind = if blocked { VisualBattleCueKind::CaptureDeflect }
            else { VisualBattleCueKind::Capture };
        result.saw_ball_identity |= frame.cues.iter().any(|cue|
            cue.kind == expected_kind && cue.move_id.as_ref() == ball);
        result.saw_hidden_enemy |= !frame.battlers[1].as_ref().unwrap().visible;
        let playback = art.battle_object_runtime.as_ref().unwrap();
        assert_eq!(playback.move_id, format!("THROW_{ball}"),
            "the actual retained object renderer must use the selected ball");
        for live in playback.slots.iter().flatten().filter(|live| !live.oam.entries.is_empty()) {
            if let VisibleMoveObjectCommand::Spawn { object_id, param, .. } =
                &playback.source[live.event_index].command {
                if object_id == "BATTLE_ANIM_OBJ_MASTER_BALL_SPARKLE" {
                    result.sparkle_params.insert(*param);
                }
            }
        }
        result.saw_sprite |= world.query::<&Handle<Image>>().iter(&world).next().is_some();
        advance_visible_capture_animation(&mut controller.shell).unwrap();
    }
    assert!(!controller.shell.visible_capture_animation.as_ref()
        .is_some_and(VisibleCaptureAnimation::throw_active), "capture clock did not finish");
    result.wobble_sounds = controller.shell.last_audio_events.iter()
        .filter(|event| event.as_str() == "queued sound effect SFX_BALL_WOBBLE").count() - sounds_before;
    assert!(result.saw_ball_identity && result.saw_sprite);
    assert!(controller.shell.last_error.is_none());
    result
}

#[test]
fn immersive_capture_fixture_poke_ball_fails_from_real_pack_action() {
    let mut controller = immersive_battle_move_and_ball_preview_controller(
        false, false, false, false, false, false, true);
    let before = controller.shell.shell.snapshot().unwrap();
    let enemy = &before.battle.as_ref().unwrap().enemy_pokemon;
    assert_eq!(enemy.species.id, "SUDOWOODO");
    assert_eq!(enemy.hp, enemy.max_hp);
    let reads_before = capture_fixture_divider_counts(&controller).0;
    let enemy_pp_before = before.battle.as_ref().unwrap().enemy_moves[1].current_pp;
    assert!(enemy_pp_before > 0, "the native selected enemy slot must be legal");
    capture_fixture_throw_from_pack(&mut controller, "POKE_BALL");
    capture_fixture_assert_finite_enemy_response(&controller, reads_before, enemy_pp_before);
    let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
    assert!(!capture.caught && !capture.blocked);
    assert_eq!(capture.animation_shakes, 0);
    assert!(controller.shell.pending_standard_capture.is_none());
    assert!(controller.shell.battle_messages.iter().any(|text| text.contains("broke free!")));
    let presentation = capture_fixture_present_real_throw(&mut controller, "POKE_BALL", false);
    assert!(presentation.saw_hidden_enemy);
    assert_eq!(presentation.wobble_sounds, 0);
    assert!(presentation.sparkle_params.is_empty());
    assert!(controller.shell.visible_capture_animation.is_none());
    let snapshot = controller.snapshot().unwrap();
    let frame = extract_immersive_battle_fixture(&controller.shell, &snapshot, true);
    assert!(frame.battlers[1].as_ref().unwrap().visible, "breakout must restore the real opponent");
    capture_fixture_return_to_commands(&mut controller);
}

#[test]
fn immersive_capture_fixture_master_ball_succeeds_with_real_wobbles_and_sparkles() {
    let mut controller = immersive_battle_move_preview_controller(false, false, false, false, true, false);
    assert_eq!(controller.shell.shell.snapshot().unwrap().party.slots[0].pokemon.species.id, "PIDGEOTTO");
    capture_fixture_throw_from_pack(&mut controller, "MASTER_BALL");
    let outcome = &controller.shell.pending_standard_capture.as_ref().unwrap().outcome;
    assert!(outcome.caught && !outcome.blocked && !outcome.storage_full);
    assert_eq!(outcome.ball_id.as_deref(), Some("MASTER_BALL"));
    assert_eq!((outcome.wobble_count, outcome.animation_shakes), (3, 4));
    let presentation = capture_fixture_present_real_throw(&mut controller, "MASTER_BALL", false);
    assert!(presentation.saw_hidden_enemy);
    assert_eq!(presentation.wobble_sounds, 3);
    assert_eq!(presentation.sparkle_params, (0x30..0x38).collect());
    let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
    assert!(capture.complete && capture.caught && capture.ball_visible());
    assert!(controller.shell.battle_messages.front().unwrap().starts_with("Gotcha!"));
}

#[test]
fn immersive_capture_fixture_vance_blocks_real_poke_ball_and_consumes_it() {
    let mut controller = immersive_battle_move_preview_controller(false, false, false, false, false, true);
    let before = controller.shell.shell.snapshot().unwrap();
    assert!(matches!(before.battle.as_ref().unwrap().kind, RuntimeBattleKind::Trainer { .. }));
    assert_eq!(before.overworld.map_name, "Route44");
    assert_eq!(before.battle.as_ref().unwrap().enemy_pokemon.species.id, "PIDGEOTTO");
    capture_fixture_throw_from_pack(&mut controller, "POKE_BALL");
    let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
    assert!(capture.blocked && !capture.caught);
    assert_eq!(capture.animation_shakes, 0);
    assert!(controller.shell.pending_standard_capture.is_none());
    assert!(controller.shell.battle_messages.iter().any(|text| text.contains("blocked the BALL!")));
    let presentation = capture_fixture_present_real_throw(&mut controller, "POKE_BALL", true);
    assert!(!presentation.saw_hidden_enemy);
    assert_eq!(presentation.wobble_sounds, 0);
    assert!(presentation.sparkle_params.is_empty());
    assert!(controller.shell.visible_capture_animation.is_none());
}


fn capture_fixture_divider_counts(controller: &VisibleShellController) -> (usize, usize) {
    let crystal_core::random::RuntimeDividerSource::Replay(divider) =
        controller.shell.shell.session().divider_for_tests()
    else {
        panic!("ordinary failure fixture must retain its bounded DIV replay");
    };
    (divider.consumed(), divider.remaining())
}

fn capture_fixture_assert_finite_enemy_response(
    controller: &VisibleShellController,
    reads_before: usize,
    enemy_pp_before: u8,
) {
    let (consumed, remaining) = capture_fixture_divider_counts(controller);
    let turn_reads = consumed - reads_before;
    assert!((1..=64).contains(&turn_reads),
        "ordinary Ball turn used {turn_reads} DIV samples; damage rejection must terminate promptly");
    assert_eq!(consumed + remaining, 65_536);
    assert_eq!(controller.shell.shell.session().state().random_state.sub, 253);
    let snapshot = controller.shell.shell.snapshot().unwrap();
    assert_eq!(snapshot.battle.as_ref().unwrap().enemy_moves[1].current_pp,
        enemy_pp_before - 1, "the normally selected enemy action must actually execute");
}

fn capture_fixture_return_to_commands(controller: &mut VisibleShellController) {
    for _ in 0..64 {
        if controller.shell.battle_messages.is_empty()
            && controller.shell.visible_capture_animation.is_none()
            && controller.shell.visible_move_animations.is_empty()
            && controller.shell.battle_action_cursor.is_some()
        {
            return;
        }
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.last_error.is_none());
    }
    panic!("real failed capture/enemy response did not return command-menu input");
}

#[test]
fn immersive_capture_fixture_failure_survives_native_vblank_warmup_and_recording_budget() {
    let mut controller = immersive_battle_move_and_ball_preview_controller(
        false, false, false, false, false, false, true,
    );
    assert_eq!(capture_fixture_divider_counts(&controller), (0, 65_536));
    assert!(!visible_special_vblank_handler_active(&controller.shell));
    let actions_before = controller.shell.deterministic_battle_actions.len();
    let vblank_before = controller.shell.shell.session().state().vblank_counter;

    // Native advance_visible_runtime calls this exact authoritative hook before
    // modal/menu early returns. Controller::wait_frames alone does not sample
    // VBlank_Normal DIV, so it would be a false warmup test here. One batch is
    // equivalent to 180 seconds of no-action native frames at a conservative
    // 60 Hz; count every actual read so a no-op hook cannot falsely pass.
    const ARM_VBLANKS: u32 = 180 * 60;
    controller.shell.shell.advance_vblanks(ARM_VBLANKS, ARM_VBLANKS).unwrap();
    assert_eq!(capture_fixture_divider_counts(&controller), (21_600, 43_936));
    assert_eq!(controller.shell.shell.session().state().vblank_counter,
        vblank_before.wrapping_add(ARM_VBLANKS as u8));
    assert_eq!(controller.shell.shell.session().state().random_state.sub, 253);
    assert_eq!(controller.shell.deterministic_battle_actions.len(), actions_before);
    assert!(controller.shell.visible_capture_animation.is_none());

    // Do not reset the RNG or replace the divider after warmup. The actual
    // PACK/USE turn must still consume a ball and resolve an ordinary failure.
    let enemy_pp_before = controller.shell.shell.snapshot().unwrap()
        .battle.as_ref().unwrap().enemy_moves[1].current_pp;
    assert!(enemy_pp_before > 0);
    capture_fixture_throw_from_pack(&mut controller, "POKE_BALL");
    capture_fixture_assert_finite_enemy_response(&controller, 21_600, enemy_pp_before);
    let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
    assert!(!capture.caught && !capture.blocked);
    assert_eq!(capture.animation_shakes, 0);
    assert!(controller.shell.pending_standard_capture.is_none());
    let (after_turn, remaining) = capture_fixture_divider_counts(&controller);
    assert!(after_turn > 21_600, "the real Ball turn must consume DIV samples");
    assert_eq!(after_turn + remaining, 65_536);

    // Also exercise the entire permitted 300-second recorder budget through
    // the same native hook. This is budget verification, not simulated video.
    const RECORD_VBLANKS: u32 = 300 * 60;
    controller.shell.shell.advance_vblanks(RECORD_VBLANKS, RECORD_VBLANKS).unwrap();
    let (after_recording, remaining) = capture_fixture_divider_counts(&controller);
    assert_eq!(after_recording, after_turn + 36_000);
    assert_eq!(after_recording + remaining, 65_536);
    assert!(remaining > 0, "the bounded replay must retain turn overhead");
    assert_eq!(controller.shell.shell.session().state().random_state.sub, 253);
    assert!(controller.shell.last_error.is_none());
}

include!("capture_presentation.rs");
