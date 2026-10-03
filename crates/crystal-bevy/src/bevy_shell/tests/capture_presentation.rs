// Draw the production textbox and inspect its actual glyph sprites. Queue or
// controller text alone cannot detect the retained-text branch hiding a page.
fn capture_presentation_assert_drawn_text(shell: &BevyRuntimeShell, expected: &str) {
    let scene = shell.battle_message_scene.as_ref().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    spawn_battle_command_menu(
        &mut Commands::new(&mut queue, &world), scene, shell,
        scene.battle.as_ref().unwrap(), &mut art, &shell.asset_root, &mut images,
    ).unwrap();
    queue.apply(&mut world);
    let actual = world.query::<(&Handle<Image>, &Transform)>().iter(&world)
        .filter(|(_, transform)| (transform.translation.z - 3.8).abs() < 0.001)
        .map(|(image, transform)| (image.clone(), transform.translation.truncate()))
        .collect::<Vec<_>>();
    let mut expected_glyphs = Vec::new();
    for (row, line) in expected.split('\n').enumerate() {
        let (x, y) = battle_hud_tile_origin(1.0, 14.0 + row as f32 * 2.0);
        for (column, glyph) in bitmap_text_frames(&mut art, &shell.asset_root, &mut images, line)
            .into_iter().enumerate() {
            expected_glyphs.push((glyph.handle, Vec2::new(x + column as f32 * BITMAP_FONT_ADVANCE, y)));
        }
    }
    assert!(!expected_glyphs.is_empty());
    assert_eq!(actual.len(), expected_glyphs.len(), "wrong rendered narration: {expected:?}");
    for glyph in expected_glyphs {
        assert!(actual.contains(&glyph), "missing rendered glyph in {expected:?}: {glyph:?}");
    }
}

fn capture_presentation_player_hp(scene: &RuntimeShellSnapshot) -> u16 {
    let index = scene.battle.as_ref().unwrap().active_player_party_index.unwrap();
    scene.party.slots.iter().find(|slot| slot.index == index).unwrap().pokemon.hp
}

fn capture_presentation_assert_held_hp(shell: &BevyRuntimeShell, expected: u16) {
    assert_eq!(capture_presentation_player_hp(shell.battle_message_scene.as_ref().unwrap()), expected);
    if let Some(tween) = &shell.battle_hp_tween {
        assert_eq!(tween.player_hp, expected, "rendered numeric HP advanced early");
        assert_eq!(tween.player_target_hp, expected, "future damage was targeted early");
        assert!(!visible_battle_hp_tween_active(tween), "HP must stay still at this narration boundary");
    }
}

fn capture_presentation_finish_text_without_input(shell: &mut BevyRuntimeShell) -> String {
    let text = shell.battle_messages.front().cloned().unwrap();
    let scene = shell.battle_message_scene.as_ref().unwrap().clone();
    for _ in 0..512 {
        if visible_battle_message_is_complete(shell, &text) {
            return battle_message_page(&text, shell.battle_text_reveal.as_ref().unwrap().page_index);
        }
        if advance_visible_battle_text_frames(shell, &scene, false, 1) {
            mark_runtime_presentation_dirty(shell);
        }
    }
    panic!("narration did not reveal without input: {text:?}");
}

fn capture_presentation_drawn_objects(shell: &BevyRuntimeShell) -> usize {
    let scene = shell.battle_message_scene.as_ref().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    spawn_visible_move_animation_objects(
        &mut Commands::new(&mut queue, &world), scene, shell,
        &mut RenderedTilesetArt::default(), &shell.asset_root, &mut Assets::<Image>::default(),
    ).unwrap();
    queue.apply(&mut world);
    world.query_filtered::<Entity, With<BattleSourceObjectMarker>>().iter(&world).count()
}

fn capture_presentation_play_throw(controller: &mut VisibleShellController, hp_before: u16) -> String {
    let used = controller.shell.battle_messages.front().cloned().unwrap();
    assert!(used.contains(" used the "));
    capture_presentation_assert_held_hp(&controller.shell, hp_before);
    capture_presentation_assert_drawn_text(&controller.shell, &battle_message_page(&used, 0));
    apply_visible_shell_smoke_frame(&mut controller.shell, &[GameButton::A]).unwrap();
    assert!(controller.shell.visible_capture_animation.as_ref().unwrap().throw_active());
    for _ in 0..600 {
        let Some(capture) = controller.shell.visible_capture_animation.as_ref()
            .filter(|capture| capture.throw_active()) else { break; };
        capture_presentation_assert_held_hp(&controller.shell, hp_before);
        if capture.frame == 0 || capture.frame == capture.total_frames() - 1 {
            capture_presentation_assert_drawn_text(&controller.shell, &battle_message_page(&used, 0));
        }
        advance_visible_battle_animation_frame(&mut controller.shell).unwrap();
    }
    assert!(!controller.shell.visible_capture_animation.as_ref()
        .is_some_and(VisibleCaptureAnimation::throw_active));
    capture_presentation_assert_held_hp(&controller.shell, hp_before);
    // No A/Z: the native text clock alone must replace retained used-ball text.
    let page = capture_presentation_finish_text_without_input(&mut controller.shell);
    capture_presentation_assert_drawn_text(&controller.shell, &page);
    controller.shell.battle_messages.front().cloned().unwrap()
}

#[test]
fn immersive_capture_presentation_draws_current_pages_and_retains_caught_ball_until_gotcha_dismissal() {
    for (ball, master, blocked, expected_result) in [
        ("POKE_BALL", false, false, "broke free!"),
        ("MASTER_BALL", true, false, "Gotcha!"),
        ("POKE_BALL", false, true, "blocked the BALL!"),
    ] {
        let mut controller = immersive_battle_move_and_ball_preview_controller(
            false, false, false, false, master, blocked, !master && !blocked,
        );
        let before = controller.shell.shell.snapshot().unwrap();
        let hp_before = capture_presentation_player_hp(&before);
        capture_fixture_throw_from_pack(&mut controller, ball);
        assert_eq!(controller.shell.battle_messages.len(), controller.shell.battle_message_scenes.len());
        let result = capture_presentation_play_throw(&mut controller, hp_before);
        assert!(result.contains(expected_result), "{ball}: {result:?}");
        if master {
            let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
            assert!(capture.complete && !capture.sprites_cleared && capture.ball_visible());
            assert!(capture_presentation_drawn_objects(&controller.shell) > 0);
            controller.press(GameButton::A).unwrap(); // actual Gotcha dismissal
            let capture = controller.shell.visible_capture_animation.as_ref().unwrap();
            assert!(capture.sprites_cleared && !capture.ball_visible());
            assert_eq!(capture_presentation_drawn_objects(&controller.shell), 0);
        }
    }
}

fn capture_presentation_finish_enemy_gust(controller: &mut VisibleShellController, hp_before: u16, hp_after: u16) {
    let trigger = controller.shell.visible_move_animations.iter()
        .find(|animation| !animation.player_move && animation.move_id == "GUST")
        .expect("real enemy Gust response").trigger_message.clone();
    for _ in 0..16 {
        capture_presentation_assert_held_hp(&controller.shell, hp_before);
        if controller.shell.battle_messages.front() == Some(&trigger) { break; }
        controller.press(GameButton::A).unwrap();
    }
    assert_eq!(controller.shell.battle_messages.front(), Some(&trigger));
    let page = capture_presentation_finish_text_without_input(&mut controller.shell);
    capture_presentation_assert_drawn_text(&controller.shell, &page);
    capture_presentation_assert_held_hp(&controller.shell, hp_before);
    apply_visible_shell_smoke_frame(&mut controller.shell, &[GameButton::A]).unwrap();
    assert!(controller.shell.visible_move_animations.front().unwrap().started);
    for _ in 0..2048 {
        if !visible_battle_animation_owns_frame(&controller.shell) { break; }
        capture_presentation_assert_held_hp(&controller.shell, hp_before);
        advance_visible_battle_animation_frame(&mut controller.shell).unwrap();
    }
    assert!(!visible_battle_animation_owns_frame(&controller.shell));
    let tween = controller.shell.battle_hp_tween.as_ref().unwrap();
    assert_eq!(tween.player_hp, hp_before, "damage begins after the move's visible animation");
    assert_eq!(tween.player_target_hp, hp_after);
    controller.wait_frames(1).unwrap(); // normal controller finishes the real HP tween
    assert_eq!(controller.shell.battle_hp_tween.as_ref().unwrap().player_hp, hp_after);
    assert!(controller.shell.last_error.is_none());
}

#[test]
fn immersive_capture_presentation_blocks_future_enemy_hp_until_response_animation_finishes() {
    let mut controller = immersive_battle_move_preview_controller(false, false, false, false, false, true);
    let hp_before = capture_presentation_player_hp(&controller.shell.shell.snapshot().unwrap());
    capture_fixture_throw_from_pack(&mut controller, "POKE_BALL");
    let hp_after = capture_presentation_player_hp(&controller.shell.shell.snapshot().unwrap());
    assert!(hp_after < hp_before, "real enemy damage must already exist in authority");
    capture_presentation_play_throw(&mut controller, hp_before);
    capture_presentation_finish_enemy_gust(&mut controller, hp_before, hp_after);
}

#[test]
fn immersive_capture_presentation_keeps_non_ball_turn_scene_staging_unchanged() {
    let mut controller = immersive_battle_move_preview_controller(false, false, false, false, false, true);
    let hp_before = capture_presentation_player_hp(&controller.shell.shell.snapshot().unwrap());
    controller.press(GameButton::A).unwrap(); // FIGHT
    controller.press(GameButton::A).unwrap(); // actual first move
    assert!(controller.shell.visible_capture_animation.is_none());
    assert!(matches!(controller.shell.deterministic_battle_actions.back().unwrap().action(), BattleAction::Move { .. }));
    assert_eq!(controller.shell.battle_messages.len(), controller.shell.battle_message_scenes.len());
    let hp_after = capture_presentation_player_hp(&controller.shell.shell.snapshot().unwrap());
    assert!(hp_after < hp_before);
    capture_presentation_finish_enemy_gust(&mut controller, hp_before, hp_after);
}
