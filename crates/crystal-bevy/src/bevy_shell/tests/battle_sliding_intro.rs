#[test]
fn battle_sliding_intro_uses_asm_bg_and_oam_steps() {
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    assert_eq!(
        battle_sliding_intro_offsets(0),
        (142.0 * scale, -144.0 * scale)
    );
    assert_eq!(
        battle_sliding_intro_offsets(36),
        (70.0 * scale, -72.0 * scale)
    );
    assert_eq!(battle_sliding_intro_offsets(71), (0.0, -2.0 * scale));
    assert_eq!(battle_sliding_intro_offsets(72), (0.0, 0.0));
}

#[test]
fn battle_sliding_intro_owns_73_frames_without_dismissing_narration() {
    let mut shell = route36_battle_shell_for_render_regression();
    let transition = shell.visible_battle_transition.as_mut().unwrap();
    transition.frame = visible_battle_transition_total_frames(transition) - 1;
    advance_visible_battle_transition(&mut shell);
    let messages = shell.battle_messages.clone();
    for frame in 0..73 {
        assert_eq!(shell.visible_battle_sliding_intro, Some(frame));
        assert!(visible_noninteractive_battle_animation_owns_input(&shell));
        // The slide only changes LCD overrides, not the VBlank handler.
        assert!(!visible_special_vblank_handler_active(&shell));
        press_visible_a_button(&mut shell).expect("A during intro is ignored");
        assert_eq!(shell.battle_messages, messages);
        advance_visible_battle_sliding_intro(&mut shell);
    }
    assert_eq!(shell.visible_battle_sliding_intro, None);
    // A wild/static encounter keeps ownership for its frontpic animation;
    // the encounter text has not been dismissed or begun revealing.
    assert!(visible_noninteractive_battle_animation_owns_input(&shell));
    assert!(visible_wild_entrance_animation_active(&shell));
    assert_eq!(shell.battle_messages, messages);
    assert!(shell.battle_text_reveal.is_none());
}

#[test]
fn battle_sliding_intro_renders_trainer_and_wild_without_hud_or_cry() {
    for kind in ["wild", "static", "trainer", "tutorial"] {
        let mut shell = route36_battle_shell_for_render_regression();
        shell.visible_battle_transition = None;
        shell.visible_battle_sliding_intro = Some(36);
        shell.pending_audio.clear();
        if kind == "wild" {
            shell
                .battle_message_scene
                .as_mut()
                .unwrap()
                .battle
                .as_mut()
                .unwrap()
                .kind = RuntimeBattleKind::Wild {
                map_name: "Route36".into(),
                battle_music: String::new(),
            };
        } else if kind == "tutorial" {
            shell
                .battle_message_scene
                .as_mut()
                .unwrap()
                .battle
                .as_mut()
                .unwrap()
                .battle_type = "BATTLETYPE_TUTORIAL".into();
        } else if kind == "trainer" {
            let scene = shell.battle_message_scene.as_mut().unwrap();
            scene.battle.as_mut().unwrap().kind = RuntimeBattleKind::Trainer {
                trainer_class: "BUG_CATCHER".into(),
                trainer_id: "WADE1".into(),
                trainer_name: "WADE".into(),
                event_flag: String::new(),
                seen_text: String::new(),
                win_text: String::new(),
                loss_text: String::new(),
                callback: String::new(),
                source_script: String::new(),
                reward: 0,
                encounter_music: String::new(),
                ai_move_flags: 0,
                ai_item_switch_flags: 0,
                ai_layers: vec![],
            };
            shell.battle_entry_messages_remaining = 3;
        }
        let mut app = battle_render_regression_app(shell);
        app.add_systems(Update, queue_battle_intro_cry);
        app.update();
        let world = app.world_mut();
        let shell = world.resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        assert!(shell.pending_audio.is_empty());
        assert!(shell.last_battle_cry_key.is_none());
        let mut hud = world.query_filtered::<Entity, With<BattleHudMarker>>();
        assert_eq!(hud.iter(world).count(), 0);
        let mut battlers =
            world.query_filtered::<(&Transform, &Sprite), With<BattleBattlerMarker>>();
        let pics: Vec<_> = battlers.iter(world).collect();
        assert_eq!(pics.len(), 2);
        for (transform, sprite) in pics {
            let width = sprite.custom_size.unwrap().x;
            assert!(transform.translation.x - width * 0.5 >= PLAYFIELD_LEFT);
            assert!(transform.translation.x + width * 0.5 <= PLAYFIELD_LEFT + PLAYFIELD_WIDTH);
        }
        save_live_battle_canvas_for_test(world, &format!("intro-{kind}-36.png"));
        let image_count = world.resource::<Assets<Image>>().len();
        {
            let mut shell = world.resource_mut::<BevyRuntimeShell>();
            shell.visible_battle_sliding_intro = Some(72);
            mark_runtime_snapshot_dirty(&mut shell);
        }
        app.update();
        save_live_battle_canvas_for_test(app.world_mut(), &format!("intro-{kind}-72.png"));
        assert_eq!(
            app.world().resource::<Assets<Image>>().len(),
            image_count,
            "intro frames reuse their palette-converted textures"
        );
    }
}

#[test]
fn battle_sliding_intro_art_uses_source_blackout_palette() {
    let shell = route36_battle_shell_for_render_regression();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let palette = load_named_predef_palette(&shell.asset_root, "PREDEFPAL_BLACKOUT").unwrap();
    for asset in [
        "battle-player:chris_back",
        "battle-player:kris_back",
        "battle-player:dude",
        "battle-trainer:bug_catcher",
        "pokemon:sudowoodo",
    ] {
        let frame =
            battle_sliding_intro_art(&mut art, &shell.asset_root, &mut images, asset).unwrap();
        let image = images.get(&frame.handle).unwrap();
        assert!(image.data.chunks_exact(4).any(|p| p[3] != 0));
        assert!(
            image
                .data
                .chunks_exact(4)
                .filter(|p| p[3] != 0)
                .all(|p| palette.iter().any(|colour| p[..3] == colour[..]))
        );
    }
}

#[test]
fn battle_sliding_intro_clips_first_frame_at_lcd_edges() {
    let mut app = App::new();
    app.add_systems(Update, |mut commands: Commands| {
        let enemy = SpriteFrame {
            handle: Handle::default(),
            size: Vec2::splat(56.0),
        };
        let player = SpriteFrame {
            handle: Handle::default(),
            size: Vec2::splat(48.0),
        };
        spawn_battle_sliding_intro_pic(&mut commands, &enemy, -48.0, 0.0);
        spawn_battle_sliding_intro_pic(&mut commands, &player, 158.0, 48.0);
    });
    app.update();
    let world = app.world_mut();
    let mut pics = world.query_filtered::<(&Sprite, &Transform), With<BattleBattlerMarker>>();
    let mut rects = pics
        .iter(world)
        .map(|(sprite, transform)| {
            let width = sprite.custom_size.unwrap().x;
            assert!(transform.translation.x - width * 0.5 >= PLAYFIELD_LEFT);
            assert!(transform.translation.x + width * 0.5 <= PLAYFIELD_LEFT + PLAYFIELD_WIDTH);
            sprite.rect.unwrap()
        })
        .collect::<Vec<_>>();
    rects.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
    assert_eq!(
        rects,
        vec![
            Rect::new(0.0, 0.0, 2.0, 48.0),
            Rect::new(48.0, 0.0, 56.0, 56.0)
        ]
    );
}
