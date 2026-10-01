// External-pack regressions for the render-only bridge, alongside the existing
// production controller tests. These use the exact same Route36 battle fixture.
fn immersive_battle_fixture() -> BevyRuntimeShell {
    let mut shell = route36_battle_shell_for_render_regression();
    shell.visible_battle_transition = None;
    shell.visible_battle_sliding_intro = None;
    shell.visible_send_out_animation = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_player_send_out_pending = false;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    shell.visible_move_animations.clear();
    shell
}

fn extract_immersive_battle_fixture(
    shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    active: bool,
) -> VisualBattleFrame {
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    capture_presented_battle(
        &mut commands,
        snapshot,
        shell,
        active,
        &mut RenderedTilesetArt::default(),
        &mut Assets::<Image>::default(),
    )
    .unwrap();
    queue.apply(&mut world);
    world.remove_resource::<VisualBattleFrame>().unwrap()
}

#[cfg(feature = "fullscreen-scaling")]
#[test]
fn immersive_battle_fullscreen_backdrop_preserves_arena_and_classic_modals() {
    let mut shell = immersive_battle_fixture();
    shell.battle_lcd_animation_active = true;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(shell)
        .init_resource::<RenderedViewport>()
        .init_resource::<Assets<Image>>()
        .init_resource::<crystal_voxel_view::BattleViewStatus>()
        .add_systems(Update, sync_fullscreen_scene_layout);
    app.world_mut()
        .spawn((Window::default(), bevy::window::PrimaryWindow));
    let backdrop = app
        .world_mut()
        .spawn((SpriteBundle::default(), FullscreenSceneBackdrop))
        .id();
    // This is the actual fullscreen compositor, including its battle-modal
    // decision. F3 must hide the opaque LCD backing and restore it in classic.
    for active in [false, true, false, true] {
        app.world_mut()
            .resource_mut::<crystal_voxel_view::BattleViewStatus>()
            .active = active;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(backdrop).unwrap(),
            if active {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            }
        );
        assert_eq!(
            app.world().get::<Sprite>(backdrop).unwrap().color,
            Color::BLACK
        );
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.battle_lcd_animation_active = false;
        shell.battle_message_scene = None;
        shell.party_menu_open = true;
    }
    app.world_mut()
        .resource_mut::<crystal_voxel_view::BattleViewStatus>()
        .active = false;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(backdrop).unwrap(),
        Visibility::Inherited,
        "party screens must keep their ordinary opaque fullscreen background"
    );
}

#[test]
fn immersive_battle_extracts_presented_party_slot_without_mutating_authority() {
    let shell = immersive_battle_fixture();
    let before = shell.shell.snapshot().unwrap();
    let mut presented = before.clone();
    // A renderer must use the currently presented slot identity, not assume
    // lead index zero or request a newer runtime snapshot behind the dialogue.
    presented.party.slots[0].index = 3;
    presented.battle.as_mut().unwrap().active_player_party_index = Some(3);
    let frame = extract_immersive_battle_fixture(&shell, &presented, true);
    assert!(frame.active);
    assert_eq!(frame.validate(), Ok(()));
    assert_eq!(frame.battlers[0].as_ref().unwrap().party_index, Some(3));
    assert_eq!(
        frame.battlers[0].as_ref().unwrap().species_id.as_ref(),
        "CYNDAQUIL"
    );
    assert!((frame.battlers[0].as_ref().unwrap().pokedex_size_m.unwrap() - 0.508).abs() < 0.00001);
    assert_eq!(
        shell.shell.snapshot().unwrap(),
        before,
        "render extraction must leave battle, PP, HP and all controller state unchanged"
    );
}

#[test]
fn immersive_battle_uses_retained_transform_without_sampling_future_authority() {
    let shell = immersive_battle_fixture();
    let mut presented = shell.shell.snapshot().unwrap();
    presented
        .battle
        .as_mut()
        .unwrap()
        .player_transformed_species = Some("TOTODILE".into());
    let frame = extract_immersive_battle_fixture(&shell, &presented, true);
    assert_eq!(
        frame.battlers[0].as_ref().unwrap().species_id.as_ref(),
        "TOTODILE"
    );
    assert!((frame.battlers[0].as_ref().unwrap().pokedex_size_m.unwrap() - 0.6096).abs() < 0.00001);
    assert!(
        shell
            .shell
            .snapshot()
            .unwrap()
            .battle
            .unwrap()
            .player_transformed_species
            .is_none()
    );
}

#[test]
fn immersive_battle_closes_for_world_and_waits_for_existing_intro() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(!extract_immersive_battle_fixture(&shell, &snapshot, false).active);
    shell.battle_entry_messages_remaining = 1;
    assert!(!extract_immersive_battle_fixture(&shell, &snapshot, true).active);
    shell.battle_entry_messages_remaining = 0;
    let mut world = snapshot;
    world.battle = None;
    assert!(!extract_immersive_battle_fixture(&shell, &world, true).active);
}

#[test]
fn immersive_battle_substitute_keeps_honest_source_art() {
    let shell = immersive_battle_fixture();
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.battle.as_mut().unwrap().player_substitute_hp = 12;
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(!frame.battlers[0].as_ref().unwrap().allow_species_model);
    assert_ne!(
        frame.battlers[0].as_ref().unwrap().texture,
        Handle::default()
    );
}

#[test]
fn immersive_battle_move_without_visible_hp_loss_never_implies_impact() {
    let shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(
        !frame
            .cues
            .iter()
            .any(|cue| cue.kind == VisualBattleCueKind::Impact)
    );
}

#[test]
fn immersive_battle_source_hud_eraser_does_not_restore_fainted_hp() {
    let (player_x, player_y) = field_window_center(9.0, 7.0, 11.0, 5.0);
    let (enemy_x, enemy_y) = field_window_center(1.0, 0.0, 10.0, 4.0);
    assert!(immersive_battle_hud_is_erased(
        Vec2::new(player_x, player_y),
        [true, false]
    ));
    assert!(!immersive_battle_hud_is_erased(
        Vec2::new(enemy_x, enemy_y),
        [true, false]
    ));
    assert!(immersive_battle_hud_is_erased(
        Vec2::new(enemy_x, enemy_y),
        [false, true]
    ));
    assert!(!immersive_battle_hud_is_erased(Vec2::ZERO, [false, false]));
}

#[test]
fn immersive_battle_pending_hp_target_waits_for_visible_animation_to_release_input() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let player = &snapshot.party.slots[0].pokemon;
    shell.battle_hp_tween = Some(VisibleBattleHpTween {
        player_hp: player.hp,
        player_target_hp: player.hp.saturating_sub(4),
        player_max_hp: player.max_hp,
        player_pixels: 48,
        player_target_pixels: 40,
        player_frames_until_step: 0,
        enemy_pixels: 48,
        enemy_target_pixels: 48,
        enemy_frames_until_step: 0,
    });
    shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
        side: crate::core::battle::turn::BattleSide::Player,
        frame: 12,
        shiny: false,
    });
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(
        !frame
            .cues
            .iter()
            .any(|cue| cue.kind == VisualBattleCueKind::Impact),
        "a staged future HP target must not show impact while another animation owns the frame"
    );
    shell.visible_send_out_animation = None;
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    assert!(
        frame
            .cues
            .iter()
            .any(|cue| cue.kind == VisualBattleCueKind::Impact)
    );
}

fn immersive_source_animation(
    snapshot: &RuntimeShellSnapshot,
    move_id: &str,
) -> VisibleMoveAnimation {
    let (animation_label, total_frames, sound_events, cry_events, object_events, bg_events) =
        visible_move_animation_definition(snapshot, move_id, 0).expect("actual pack move script");
    VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: move_id.into(),
        animation_label,
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames,
        sound_events,
        next_sound_event: 0,
        cry_events,
        next_cry_event: 0,
        object_events,
        bg_events,
        actor_species_override: None,
        actor_shiny_override: None,
    }
}

#[test]
fn immersive_battle_native_oam_stays_unscrolled_through_zero_crossings_and_palette_modes() {
    use bevy::render::view::RenderLayers;
    use crystal_render_api::BattleFlashMode;

    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let mut animation = immersive_source_animation(&snapshot, "HYPER_BEAM");
    animation.frame = 13;
    shell.visible_move_animations.push_back(animation);
    let before = shell.visible_move_animations.clone();
    let mut app = App::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        capture_presented_battle(
            &mut commands,
            &snapshot,
            &shell,
            true,
            &mut art,
            &mut images,
        )
        .unwrap();
        spawn_visible_move_animation_objects(
            &mut commands,
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
    }
    queue.apply(app.world_mut());
    let source = app
        .world()
        .resource::<VisualBattleFrame>()
        .source
        .clone()
        .unwrap();
    assert!(source.line_x_offsets.is_none() && source.line_y_offsets.is_none());
    assert!(
        !source.objects.is_empty(),
        "the fixture must contain actual beam OAM"
    );
    let native = {
        let world = app.world_mut();
        world
            .query::<(Entity, &ImmersiveBattleSourceObject, &Transform, &Sprite)>()
            .iter(world)
            .map(|(entity, slot, transform, sprite)| {
                (entity, slot.0, *transform, sprite.rect, sprite.custom_size)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(native.len(), source.objects.len());
    for (entity, ..) in &native {
        app.world_mut()
            .entity_mut(*entity)
            .insert(RenderLayers::layer(
                crystal_voxel_view::HIDDEN_CLASSIC_WORLD_RENDER_LAYER,
            ));
    }
    let replaced_bg = app
        .world_mut()
        .spawn((SpriteBundle::default(), FixedBattleCanvasMarker))
        .id();
    app.add_plugins(MinimalPlugins)
        .insert_resource(shell)
        .init_resource::<RenderedViewport>()
        .init_resource::<crystal_voxel_view::VoxelViewStatus>()
        .insert_resource(crystal_voxel_view::VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .insert_resource(crystal_voxel_view::BattleViewStatus {
            active: true,
            ..default()
        })
        .insert_resource(BattleFlashMode::Full)
        .add_systems(
            Update,
            (
                apply_visible_battle_screen_offset,
                sync_immersive_battle_layers,
            )
                .chain(),
        );
    // Include both global signs, exact zero and a line-buffer phase. Ownership
    // must not change when the source scroll register crosses zero or resets.
    for (offset, line_scroll) in [
        (Vec2::new(3.0, -2.0), false),
        (Vec2::ZERO, false),
        (Vec2::new(-3.0, 2.0), false),
        (Vec2::ZERO, true),
        (Vec2::ZERO, false),
    ] {
        for mode in [BattleFlashMode::Full, BattleFlashMode::Reduced] {
            app.world_mut().insert_resource(mode);
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            let current = frame.source.as_mut().unwrap();
            current.screen_offset = offset;
            current.line_y_offsets = line_scroll.then_some([2; 95]);
            let expected_frame = frame.clone();
            drop(frame);
            app.update();
            for (entity, slot, transform, rect, size) in &native {
                let object = source
                    .objects
                    .iter()
                    .find(|object| object.slot == *slot)
                    .unwrap();
                assert!(
                    app.world().get::<RenderLayers>(*entity).is_none(),
                    "source OAM must remain in the native pass at {offset:?}"
                );
                assert_eq!(app.world().get::<Transform>(*entity), Some(transform));
                let sprite = app.world().get::<Sprite>(*entity).unwrap();
                assert_eq!((sprite.rect, sprite.custom_size), (*rect, *size));
                assert_eq!(
                    app.world().get::<Handle<Image>>(*entity),
                    Some(if mode == BattleFlashMode::Reduced {
                        &object.neutral_texture
                    } else {
                        &object.texture
                    })
                );
            }
            assert_eq!(
                app.world().get::<RenderLayers>(replaced_bg),
                Some(&RenderLayers::layer(
                    crystal_voxel_view::HIDDEN_CLASSIC_WORLD_RENDER_LAYER,
                ))
            );
            assert_eq!(*app.world().resource::<VisualBattleFrame>(), expected_frame);
            assert_eq!(
                app.world()
                    .resource::<BevyRuntimeShell>()
                    .visible_move_animations,
                before
            );
            assert_eq!(
                app.world()
                    .resource::<BevyRuntimeShell>()
                    .shell
                    .snapshot()
                    .unwrap(),
                snapshot
            );
        }
    }
    app.world_mut()
        .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
        .enabled = false;
    app.update();
    assert!(app.world().get::<RenderLayers>(replaced_bg).is_none());
    for (entity, ..) in &native {
        assert!(app.world().get::<RenderLayers>(*entity).is_none());
    }
}

#[test]
fn immersive_battle_shadow_ball_preserves_actual_source_palette_and_object_frames() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let animation = immersive_source_animation(&snapshot, "SHADOW_BALL");
    assert_eq!(animation.animation_label, "BattleAnim_ShadowBall");
    assert_eq!(animation.total_frames, 57);
    assert_eq!(
        animation.bg_events.len(),
        1,
        "Shadow Ball is an inversion, not an invented repeated flash"
    );
    assert_eq!(
        (animation.bg_events[0].frame, animation.bg_events[0].param),
        (1, 0x1b)
    );
    assert_eq!(animation.bg_events[0].effect_id, "BATTLE_PALETTE_BGP");
    assert_eq!(
        animation
            .object_events
            .iter()
            .map(|event| event.frame)
            .collect::<Vec<_>>(),
        [1, 33]
    );
    shell.visible_move_animations.push_back(animation);
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut first_ball = None;
    let mut last_ball = None;
    for tick in 0..57 {
        shell.visible_move_animations.front_mut().unwrap().frame = tick;
        let before_animation = shell.visible_move_animations.clone();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            capture_presented_battle(
                &mut commands,
                &snapshot,
                &shell,
                true,
                &mut art,
                &mut images,
            )
            .unwrap();
        }
        queue.apply(&mut world);
        let presented = world.remove_resource::<VisualBattleFrame>().unwrap();
        let source = presented.source.as_ref().unwrap();
        assert_eq!(source.frame, tick);
        assert_eq!(source.bgp, if tick == 0 { 0xe4 } else { 0x1b });
        assert!(
            !presented
                .cues
                .iter()
                .any(|cue| cue.kind == VisualBattleCueKind::Impact)
        );
        if tick < 33 {
            assert!(
                source
                    .objects
                    .iter()
                    .all(|object| object.object_id.as_ref() != "BATTLE_ANIM_OBJ_BALL_POOF")
            );
        }
        if tick == 33 {
            assert!(
                source
                    .objects
                    .iter()
                    .any(|object| object.object_id.as_ref() == "BATTLE_ANIM_OBJ_BALL_POOF")
            );
        }
        for object in &source.objects {
            if object.object_id.as_ref() == "BATTLE_ANIM_OBJ_SHADOW_BALL" {
                first_ball.get_or_insert(object.center);
                last_ball = Some(object.center);
            }
        }
        // Invoke the actual classic draw after bridge extraction. It must reuse
        // the same object tick, art, palettes and positions, including OAM cuts.
        {
            let mut commands = Commands::new(&mut queue, &world);
            spawn_visible_move_animation_objects(
                &mut commands,
                &snapshot,
                &shell,
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .unwrap();
        }
        queue.apply(&mut world);
        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let mut sprites = world.query::<(&Handle<Image>, &Transform)>();
        for object in &source.objects {
            assert!(
                sprites.iter(&world).any(|(image, pose)| {
                    *image == object.texture
                        && (pose.translation.x - (PLAYFIELD_LEFT + object.center.x * scale)).abs()
                            < 0.001
                        && (pose.translation.y - (PLAYFIELD_TOP - object.center.y * scale)).abs()
                            < 0.001
                }),
                "classic and 3D disagreed at source frame {tick}"
            );
        }
        assert_eq!(
            shell.visible_move_animations, before_animation,
            "rendering cannot advance source timing"
        );
    }
    assert_ne!(
        first_ball, last_ball,
        "the actual source callback must move the ball"
    );
    assert_eq!(
        shell.shell.snapshot().unwrap(),
        snapshot,
        "render extraction cannot change HP, PP or input state"
    );
    shell.visible_move_animations.clear();
    assert!(
        extract_immersive_battle_fixture(&shell, &snapshot, true)
            .source
            .is_none()
    );
    assert!(
        extract_immersive_battle_fixture(&shell, &snapshot, false)
            .source
            .is_none()
    );
}

#[test]
fn immersive_battle_current_frame_never_exposes_future_flash_events() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let mut animation = immersive_source_animation(&snapshot, "SHADOW_BALL");
    animation.object_events.clear();
    animation.bg_events = vec![VisibleMoveBgEvent {
        frame: 8,
        effect_id: "BATTLE_BG_EFFECT_FLASH_WHITE".into(),
        duration: 0,
        target: "2".into(),
        param: 4,
        incremented: false,
    }];
    shell.visible_move_animations.push_back(animation);
    for (tick, expected) in [
        (0, 0xe4),
        (7, 0xe4),
        (8, 0),
        (10, 0),
        (11, 0xe4),
        (14, 0),
        (17, 0xe4),
    ] {
        shell.visible_move_animations.front_mut().unwrap().frame = tick;
        let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
        assert_eq!(
            frame.source.unwrap().bgp,
            expected,
            "source flash at frame {tick}"
        );
    }
}

#[test]
fn immersive_battle_psychic_and_hyper_beam_use_actual_source_frames() {
    for (move_id, label, duration, spawn_frames) in [
        (
            "PSYCHIC_M",
            "BattleAnim_PsychicM",
            165,
            vec![1, 9, 17, 25, 33, 41, 49, 57],
        ),
        (
            "HYPER_BEAM",
            "BattleAnim_HyperBeam",
            61,
            vec![1, 5, 9, 13, 13],
        ),
    ] {
        let mut shell = immersive_battle_fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let animation = immersive_source_animation(&snapshot, move_id);
        assert_eq!(animation.animation_label, label);
        assert_eq!(animation.total_frames, duration);
        assert_eq!(
            animation
                .object_events
                .iter()
                .filter_map(|event| {
                    matches!(event.command, VisibleMoveObjectCommand::Spawn { .. })
                        .then_some(event.frame)
                })
                .collect::<Vec<_>>(),
            spawn_frames
        );
        shell.visible_move_animations.push_back(animation);
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let mut saw_wave = false;
        let mut saw_tip = false;
        for tick in 0..duration {
            shell.visible_move_animations.front_mut().unwrap().frame = tick;
            let before_animation = shell.visible_move_animations.clone();
            let mut world = World::new();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            capture_presented_battle(
                &mut Commands::new(&mut queue, &world),
                &snapshot,
                &shell,
                true,
                &mut art,
                &mut images,
            )
            .unwrap();
            queue.apply(&mut world);
            let presented = world.remove_resource::<VisualBattleFrame>().unwrap();
            let source = presented.source.as_ref().unwrap();
            let current = shell.visible_move_animations.front().unwrap();
            assert_eq!(source.frame, tick);
            assert_eq!(
                source.bgp,
                visible_battle_dmg_palette_registers(Some(current)).bgp
            );
            assert_eq!(
                source.line_x_offsets,
                visible_battle_line_x_offsets(Some(current))
            );
            assert!(
                presented
                    .cues
                    .iter()
                    .all(|cue| cue.kind != VisualBattleCueKind::Impact)
            );
            if move_id == "PSYCHIC_M" {
                assert_eq!(source.line_x_offsets.is_some(), (1..161).contains(&tick));
                saw_wave |= source
                    .objects
                    .iter()
                    .any(|object| object.object_id.as_ref() == "BATTLE_ANIM_OBJ_WAVE");
            } else {
                assert!(source.line_x_offsets.is_none());
                let tip = source
                    .objects
                    .iter()
                    .any(|object| object.object_id.as_ref() == "BATTLE_ANIM_OBJ_BEAM_TIP");
                assert!(!tip || tick >= 13, "must not expose a future beam tip");
                saw_tip |= tip;
            }
            // Compare against the actual classic renderer on every frame,
            // including delayed spawns, frameset deletion and source clipping.
            spawn_visible_move_animation_objects(
                &mut Commands::new(&mut queue, &world),
                &snapshot,
                &shell,
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .unwrap();
            queue.apply(&mut world);
            let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
            let mut sprites = world.query::<(&Handle<Image>, &Transform)>();
            for object in &source.objects {
                assert!(
                    sprites
                        .iter(&world)
                        .any(|(image, pose)| *image == object.texture
                            && (pose.translation.x - (PLAYFIELD_LEFT + object.center.x * scale))
                                .abs()
                                < 0.001
                            && (pose.translation.y - (PLAYFIELD_TOP - object.center.y * scale))
                                .abs()
                                < 0.001),
                    "{move_id} classic/3D object mismatch at frame {tick}"
                );
            }
            assert_eq!(shell.visible_move_animations, before_animation);
        }
        assert!(if move_id == "PSYCHIC_M" {
            saw_wave
        } else {
            saw_tip
        });
        assert_eq!(shell.shell.snapshot().unwrap(), snapshot);
    }
}

#[cfg(feature = "location-tester")]
fn immersive_battle_move_preview_controller(
    shadow_ball: bool,
    psychic: bool,
    hyper_beam: bool,
    surf: bool,
) -> VisibleShellController {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let asset_root = AssetRoot::new(root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "Route36".into(),
            tile_x: 20,
            tile_y: 8,
        },
        BevyShellConfig::default(),
    )
    .unwrap();
    let shell =
        prepare_immersive_battle_preview(shell, shadow_ball, psychic, hyper_beam, surf, false).unwrap();
    VisibleShellController { shell }
}

#[cfg(feature = "location-tester")]
#[test]
fn immersive_battle_shadow_ball_preview_uses_legal_tm_and_normal_controller() {
    let mut controller = immersive_battle_move_preview_controller(true, false, false, false);
    let before = controller.snapshot().unwrap();
    assert_eq!(before.party.slots[0].pokemon.species.id, "GENGAR");
    assert_eq!(before.party.slots[0].pokemon.moves[0].name, "SHADOW_BALL");
    let pp = before.party.slots[0].pokemon.moves[0].current_pp;
    controller.press(GameButton::A).unwrap();
    controller.press(GameButton::A).unwrap();
    let after = controller.snapshot().unwrap();
    let pp_after = after
        .battle
        .as_ref()
        .map(|battle| battle.player_moves[0].current_pp)
        .unwrap_or(after.party.slots[0].pokemon.moves[0].current_pp);
    assert_eq!(pp_after, pp - 1);
    assert!(
        controller
            .shell
            .visible_move_animations
            .iter()
            .any(|animation| animation.move_id == "SHADOW_BALL")
    );
}

#[cfg(feature = "location-tester")]
#[test]
fn immersive_battle_psychic_preview_uses_legal_tm_and_normal_controller() {
    let mut controller = immersive_battle_move_preview_controller(false, true, false, false);
    let before = controller.shell.shell.snapshot().unwrap();
    assert_eq!(before.party.slots[0].pokemon.species.id, "KADABRA");
    assert_eq!(before.party.slots[0].pokemon.level, 20);
    assert_eq!(before.party.slots[0].pokemon.moves[0].name, "PSYCHIC_M");
    assert_eq!(battle_move_display_name(&before, "PSYCHIC_M"), "PSYCHIC");
    let pp = before.party.slots[0].pokemon.moves[0].current_pp;
    assert_eq!(pp, 10, "TM29 must supply the source move's legal PP");
    controller.press(GameButton::A).unwrap();
    controller.press(GameButton::A).unwrap();
    let after = controller.shell.shell.snapshot().unwrap();
    let pp_after = after
        .battle
        .as_ref()
        .map(|battle| battle.player_moves[0].current_pp)
        .unwrap_or(after.party.slots[0].pokemon.moves[0].current_pp);
    assert_eq!(pp_after, pp - 1);
    let animation = controller
        .shell
        .visible_move_animations
        .iter()
        .find(|animation| animation.player_move && animation.move_id == "PSYCHIC_M")
        .expect("the actual controller turn must queue Psychic's source animation");
    assert_eq!(animation.animation_label, "BattleAnim_PsychicM");
    assert!(animation.total_frames > 0);
    assert!(!animation.bg_events.is_empty());
}

#[cfg(feature = "location-tester")]
#[test]
fn immersive_battle_hyper_beam_preview_recharges_through_normal_controller() {
    let mut controller = immersive_battle_move_preview_controller(false, false, true, false);
    let before = controller.shell.shell.snapshot().unwrap();
    assert_eq!(before.party.slots[0].pokemon.species.id, "RATICATE");
    assert_eq!(before.party.slots[0].pokemon.level, 20);
    assert_eq!(before.party.slots[0].pokemon.moves[0].name, "HYPER_BEAM");
    let pp = before.party.slots[0].pokemon.moves[0].current_pp;
    assert_eq!(pp, 5, "TM15 must supply the source move's legal PP");
    let enemy_hp_before = before.battle.as_ref().unwrap().enemy_pokemon.hp;
    controller.press(GameButton::A).unwrap();
    controller.press(GameButton::A).unwrap();
    let after_attack = controller.shell.shell.snapshot().unwrap();
    let battle = after_attack.battle.as_ref().unwrap();
    assert_eq!(battle.player_moves[0].current_pp, pp - 1);
    assert!(
        battle.enemy_pokemon.hp > 0,
        "the recharge target must survive"
    );
    assert!(battle.enemy_pokemon.hp < enemy_hp_before);
    assert!(battle.commands.player_turn_automatic);
    let enemy_hp_after_attack = battle.enemy_pokemon.hp;
    let player_turns_after_attack = battle.player_turns_taken;
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .script_runtime
            .active_battle_combat
            .as_ref()
            .unwrap()
            .player_recharge_move
            .as_deref(),
        Some("HYPER_BEAM")
    );
    let animation = controller
        .shell
        .visible_move_animations
        .iter()
        .find(|animation| animation.player_move && animation.move_id == "HYPER_BEAM")
        .expect("the actual controller turn must queue Hyper Beam's source animation");
    assert_eq!(animation.animation_label, "BattleAnim_HyperBeam");
    assert!(animation.total_frames > 0);
    assert!(!animation.object_events.is_empty());

    // Acknowledge the real first-turn messages and let the production
    // controller resume its retained recharge automatically. Never resolve a
    // core turn directly or fabricate recharge state for this integration test.
    let mut saw_recharge = false;
    for _ in 0..64 {
        if controller
            .shell
            .battle_messages
            .iter()
            .any(|message| message.contains("must recharge!"))
        {
            saw_recharge = true;
            break;
        }
        assert!(
            !controller.shell.battle_messages.is_empty(),
            "recharge must resume before a new command menu is exposed"
        );
        controller.press(GameButton::A).unwrap();
    }
    assert!(saw_recharge, "the real second turn must narrate recharge");
    let after_recharge = controller.shell.shell.snapshot().unwrap();
    let battle = after_recharge.battle.as_ref().unwrap();
    assert_eq!(battle.player_moves[0].current_pp, pp - 1);
    assert_eq!(battle.enemy_pokemon.hp, enemy_hp_after_attack);
    assert_eq!(battle.player_turns_taken, player_turns_after_attack);
    assert!(!battle.commands.player_turn_automatic);
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .script_runtime
            .active_battle_combat
            .as_ref()
            .unwrap()
            .player_recharge_move
            .is_none()
    );
    assert!(
        !controller
            .shell
            .visible_move_animations
            .iter()
            .any(|animation| animation.player_move && animation.move_id == "HYPER_BEAM"),
        "recharge must not queue another Hyper Beam attack"
    );
}

#[cfg(feature = "location-tester")]
#[test]
fn immersive_battle_surf_preview_teaches_nonconsumable_hm_and_uses_normal_turn() {
    let mut controller = immersive_battle_move_preview_controller(false, false, false, true);
    let before = controller.shell.shell.snapshot().unwrap();
    assert_eq!(before.party.slots[0].pokemon.species.id, "TOTODILE");
    assert_eq!(before.party.slots[0].pokemon.level, 20);
    assert_eq!(before.party.slots[0].pokemon.moves[0].name, "SURF");
    assert_eq!(before.party.slots[0].pokemon.moves[0].current_pp, 15);
    let surf = before
        .moves
        .iter()
        .find(|entry| entry.move_id == "SURF")
        .unwrap();
    assert_eq!(
        (surf.source_index, surf.power, surf.accuracy, surf.pp),
        (57, 95, 100, 15)
    );
    assert_eq!(surf.move_type, "WATER");
    assert_eq!(surf.effect, "NORMAL_HIT");
    assert_eq!(
        before
            .bag
            .tm_hm
            .iter()
            .find(|item| item.item_id == "HM_SURF")
            .unwrap()
            .quantity,
        1
    );
    controller.press(GameButton::A).unwrap();
    controller.press(GameButton::A).unwrap();
    let after = controller.shell.shell.snapshot().unwrap();
    let pp = after
        .battle
        .as_ref()
        .map(|battle| battle.player_moves[0].current_pp)
        .unwrap_or(after.party.slots[0].pokemon.moves[0].current_pp);
    assert_eq!(pp, 14, "the authoritative turn spends exactly one Surf PP");
    assert_eq!(
        after
            .bag
            .tm_hm
            .iter()
            .find(|item| item.item_id == "HM_SURF")
            .unwrap()
            .quantity,
        1
    );
    // Do not force accuracy/damage or require a hit to validate legal teaching.
    assert!(
        controller
            .shell
            .visible_move_animations
            .iter()
            .any(|animation| { animation.player_move && animation.move_id == "SURF" })
    );
}

#[test]
fn immersive_battle_surf_preserves_source_program_axis_and_object_lifetime() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let animation = immersive_source_animation(&snapshot, "SURF");
    assert_eq!(animation.animation_label, "BattleAnim_Surf");
    assert_eq!(animation.total_frames, 185);
    assert_eq!(
        animation.sound_events,
        [1, 33, 65, 97].map(|frame| (
            frame,
            VisibleMoveSound {
                id: "SFX_SURF".to_string(),
                args: Some(BattleSoundArgs::new(0, 1))
            }
        ))
    );
    assert!(animation.cry_events.is_empty());
    assert_eq!(animation.object_events.len(), 2);
    assert_eq!(animation.object_events[0].frame, 1);
    assert!(matches!(&animation.object_events[0].command,
        VisibleMoveObjectCommand::Spawn { object_id, x: 88, y: 104, param: 8 }
        if object_id == "BATTLE_ANIM_OBJ_SURF"));
    assert_eq!(animation.object_events[1].frame, 129);
    assert!(matches!(
        animation.object_events[1].command,
        VisibleMoveObjectCommand::Increment { index: 1 }
    ));
    assert_eq!(animation.bg_events.len(), 1);
    assert_eq!(animation.bg_events[0].frame, 1);
    assert_eq!(animation.bg_events[0].effect_id, "BATTLE_BG_EFFECT_SURF");
    shell.visible_move_animations.push_back(animation);
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut saw_crest = false;
    for tick in 0..185 {
        shell.visible_move_animations.front_mut().unwrap().frame = tick;
        let before_animation = shell.visible_move_animations.clone();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        capture_presented_battle(
            &mut Commands::new(&mut queue, &world),
            &snapshot,
            &shell,
            true,
            &mut art,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        let presented = world.remove_resource::<VisualBattleFrame>().unwrap();
        let source = presented
            .source
            .as_ref()
            .expect("retain source trace even during fallback");
        assert_eq!(source.frame, tick);
        assert_eq!(source.bgp, 0xe4, "Surf does not flash BGP");
        assert!(
            source.line_x_offsets.is_none(),
            "Surf must never bend horizontally"
        );
        assert_eq!(
            source.line_y_offsets,
            visible_battle_line_y_offsets(shell.visible_move_animations.front())
        );
        assert_eq!(source.line_y_offsets.is_some(), (1..183).contains(&tick));
        assert!(
            source
                .objects
                .iter()
                .all(|object| object.object_id.as_ref() == "BATTLE_ANIM_OBJ_SURF")
        );
        saw_crest |= !source.objects.is_empty();
        if tick >= 183 {
            assert!(
                source.objects.is_empty(),
                "source VM finished the exit, including any final-tick OAM"
            );
        }
        spawn_visible_move_animation_objects(
            &mut Commands::new(&mut queue, &world),
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let mut sprites = world.query::<(
            &Handle<Image>,
            &Transform,
            Option<&BattleSourceObjectMarker>,
        )>();
        let mut crops = world.query::<(&Handle<Image>, &Sprite, &Transform)>();
        for object in &source.objects {
            let image = images.get(&object.texture).unwrap();
            let image_size = Vec2::new(
                image.texture_descriptor.size.width as f32,
                image.texture_descriptor.size.height as f32,
            );
            assert!(
                crops.iter(&world).any(|(texture, sprite, pose)| {
                    *texture == object.texture
                        && sprite.rect.is_some_and(|rect| {
                            rect.min / image_size == object.uv_rect.min
                                && rect.max / image_size == object.uv_rect.max
                        })
                        && sprite.custom_size == Some(object.size * scale)
                        && (pose.translation.x - (PLAYFIELD_LEFT + object.center.x * scale)).abs()
                            < 0.001
                        && (pose.translation.y - (PLAYFIELD_TOP - object.center.y * scale)).abs()
                            < 0.001
                }),
                "native and3D crop must select the same cached pixels at frame {tick}"
            );
            let bounds = Rect::from_center_size(object.center, object.size);
            assert!(
                bounds.min.cmpge(Vec2::ZERO).all()
                    && bounds.max.cmple(Vec2::new(160.0, 144.0)).all(),
                "Surf source crop escaped the LCD at frame {tick}: {bounds:?}"
            );
            assert!(
                sprites.iter(&world).any(|(image, pose, source_oam)| {
                    source_oam.is_some()
                        && *image == object.texture
                        && (pose.translation.x - (PLAYFIELD_LEFT + object.center.x * scale)).abs()
                            < 0.001
                        && (pose.translation.y - (PLAYFIELD_TOP - object.center.y * scale)).abs()
                            < 0.001
                }),
                "Surf source OAM mismatch at frame {tick}"
            );
        }
        assert_eq!(shell.visible_move_animations, before_animation);
    }
    assert!(saw_crest);
    assert_eq!(
        shell.shell.snapshot().unwrap(),
        snapshot,
        "presentation must not change PP/HP/state"
    );
}

#[test]
fn immersive_battle_projects_source_oam_and_restores_classic_through_resize_and_f3() {
    let mut shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let mut animation = immersive_source_animation(&snapshot, "SURF");
    animation.frame = 65;
    shell.visible_move_animations.push_back(animation);
    let frame = extract_immersive_battle_fixture(&shell, &snapshot, true);
    let source = frame.source.as_ref().unwrap();
    assert!(!source.objects.is_empty());
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(frame.clone())
        .insert_resource(crystal_voxel_view::BattleViewStatus {
            active: true,
            ..default()
        })
        .init_resource::<crystal_render_api::VisualBattleCanvas>()
        .init_resource::<crystal_voxel_view::BattleSceneLayout>()
        .add_systems(Update, sync_immersive_battle_source_object_layout);
    let mut originals = Vec::new();
    for object in &source.objects {
        let transform = Transform::from_xyz(
            PLAYFIELD_LEFT + object.center.x * 4.0,
            PLAYFIELD_TOP - object.center.y * 4.0,
            3.8,
        );
        let size = object.size * 4.0;
        let crop = Some(Rect::new(1.0, 2.0, 9.0, 10.0));
        let entity = app
            .world_mut()
            .spawn((
                SpriteBundle {
                    transform,
                    sprite: Sprite {
                        custom_size: Some(size),
                        rect: crop,
                        ..default()
                    },
                    ..default()
                },
                ImmersiveBattleSourceObject(object.slot),
            ))
            .id();
        originals.push((entity, transform, size, crop));
    }
    for viewport in [Vec2::new(1180.0, 812.0), Vec2::new(720.0, 960.0)] {
        app.world_mut()
            .resource_mut::<crystal_render_api::VisualBattleCanvas>()
            .size = viewport;
        for active in [true, false, true, false] {
            app.world_mut()
                .resource_mut::<crystal_voxel_view::BattleViewStatus>()
                .active = active;
            app.update();
            for ((entity, original, size, crop), object) in originals.iter().zip(&source.objects) {
                let pose = app.world().get::<Transform>(*entity).unwrap();
                let sprite = app.world().get::<Sprite>(*entity).unwrap();
                assert_eq!(
                    sprite.rect, *crop,
                    "projection cannot alter source UV clipping"
                );
                if active {
                    let projected = crystal_voxel_view::battle_source_overlay_rect(
                        app.world().resource::<crystal_voxel_view::BattleSceneLayout>(),
                        object.center,
                        object.size,
                        viewport,
                    )
                    .unwrap();
                    assert_eq!(
                        pose.translation.truncate(),
                        Vec2::new(
                            projected.center().x - viewport.x * 0.5,
                            viewport.y * 0.5 - projected.center().y
                        )
                    );
                    assert_eq!(sprite.custom_size, Some(projected.size()));
                    assert_eq!(pose.translation.z, original.translation.z);
                } else {
                    assert_eq!(pose, original);
                    assert_eq!(sprite.custom_size, Some(*size));
                    assert!(
                        app.world()
                            .get::<ImmersiveBattleSourceObjectLayout>(*entity)
                            .is_none()
                    );
                }
            }
            assert_eq!(*app.world().resource::<VisualBattleFrame>(), frame);
        }
    }
    assert_eq!(shell.shell.snapshot().unwrap(), snapshot);
}

#[test]
fn immersive_surf_sound_events_keep_source_args_and_queue_once_at_source_frames() {
    for player_move in [true, false] {
        let mut shell = immersive_battle_fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        let mut animation = immersive_source_animation(&snapshot, "SURF");
        animation.player_move = player_move;
        shell.pending_audio.clear();
        shell.visible_move_animations.push_back(animation);
        let mut dispatched = Vec::new();
        for frame in 1..185 {
            advance_visible_move_animation(&mut shell).unwrap();
            for command in std::mem::take(&mut shell.pending_audio) {
                if command.audio_id == "SFX_SURF" {
                    let playback = command
                        .battle_sound
                        .expect("source operands must reach playback");
                    assert_eq!(playback.args.packed, 1);
                    assert_eq!(playback.player_move, player_move);
                    dispatched.push(frame);
                }
            }
            let before = shell.pending_audio.clone();
            let _ = extract_immersive_battle_fixture(&shell, &snapshot, true);
            assert_eq!(
                shell.pending_audio, before,
                "render extraction cannot replay sounds"
            );
        }
        assert_eq!(dispatched, [1, 33, 65, 97]);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn immersive_surf_stereo_mask_preserves_pack_pcm_hash_format_and_frame_count() {
    let shell = immersive_battle_fixture();
    let command = BevyAudioCommand {
        battle_sound: Some(BattleSoundPlayback {
            args: BattleSoundArgs::new(0, 1),
            player_move: true,
        }),
        cry_parameters: None,
        audio_id: "SFX_SURF".into(),
        kind: ModpackAudioKind::SoundEffect,
        mode: ModpackAudioPlaybackMode::RawPcm,
        looped: false,
    };
    let source = shell
        .shell
        .runtime()
        .audio()
        .require_sound_effect("SFX_SURF")
        .unwrap()
        .source
        .clone();
    let decoded = decoded_audio_program_source(&command, source).unwrap();
    assert_eq!(decoded.format.sample_rate_hz, 22_050);
    assert_eq!(decoded.format.channels, 2);
    assert_eq!(decoded.format.bits_per_sample, 16);
    assert_eq!(decoded.bytes.len(), 59_069 * 4);
    let hash = decoded.bytes.iter().fold(0x811c9dc5_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x01000193)
    });
    assert_eq!(hash, 0x4b58b775);
    assert!(decoded.loop_range.is_none());
    let before = Arc::clone(&decoded.samples);
    assert!(
        before.chunks_exact(2).all(|frame| frame[0] == frame[1]),
        "the source program must remain bilateral for this static mask shortcut"
    );
    let right = pcm_samples_for_audio_command(&before, Sound::Stereo, &command);
    assert_eq!(right.len(), before.len());
    assert!(
        right
            .chunks_exact(2)
            .zip(before.chunks_exact(2))
            .all(|(actual, expected)| { actual[0] == 0 && actual[1] == expected[1] })
    );
    assert!(Arc::ptr_eq(&before, &decoded.samples));
}

#[test]
fn immersive_surf_source_busy_clock_uses_unpadded_channels_and_equal_priority_restarts() {
    let shell = immersive_battle_fixture();
    let snapshot = shell.shell.snapshot().unwrap();
    let animation = immersive_source_animation(&snapshot, "SURF");
    let source = &shell
        .shell
        .runtime()
        .audio()
        .require_sound_effect("SFX_SURF")
        .unwrap()
        .source;
    let AudioProgramSource::Midi { midi_base64, .. } = source else {
        panic!("source MIDI program");
    };
    let program = crystal_audio::synth::decode_midi(midi_base64).unwrap();
    let timing = crystal_audio::synth::source_channel_timing(&program).unwrap();
    assert_eq!(
        timing.channel_frames,
        BTreeMap::from([(5, 144), (6, 160), (8, 156)])
    );
    assert!(timing.looping_channels.is_empty());
    let commands = animation
        .sound_events
        .iter()
        .map(|(_, sound)| BevyAudioCommand {
            battle_sound: sound.args.map(|args| BattleSoundPlayback {
                args,
                player_move: true,
            }),
            cry_parameters: None,
            audio_id: sound.id.clone(),
            kind: ModpackAudioKind::SoundEffect,
            mode: ModpackAudioPlaybackMode::RawPcm,
            looped: false,
        })
        .collect::<Vec<_>>();
    let priorities = shell.shell.runtime().audio().sound_effect_priorities();
    let priority = priorities["SFX_SURF"];
    assert_eq!(priority, 83);
    // Same-ID/equal-priority source requests are admitted while Surf is busy.
    assert_eq!(
        source_ordered_pending_audio(
            commands.clone(),
            Some(ModpackAudioKind::SoundEffect),
            priority,
            priorities
        )
        .unwrap(),
        commands
    );
    assert_eq!(
        animation
            .sound_events
            .iter()
            .map(|(frame, _)| frame + 160)
            .collect::<Vec<_>>(),
        [161, 193, 225, 257]
    );
    assert_eq!(
        visible_surf_source_audio_wait(&shell, &animation).unwrap(),
        72
    );
    let mut unknown = animation.clone();
    unknown.sound_events[0].1.args = Some(BattleSoundArgs::new(6, 2));
    assert!(visible_surf_source_audio_wait(&shell, &unknown).is_err());
}

#[test]
fn immersive_surf_waits_for_virtual_audio_after_script_return_without_extending_source_frames() {
    for device_busy in [false, true] {
        let mut shell = immersive_battle_fixture();
        let snapshot = shell.shell.snapshot().unwrap();
        shell.pending_audio.clear();
        shell.transient_audio_playing = device_busy;
        shell
            .visible_move_animations
            .push_back(immersive_source_animation(&snapshot, "SURF"));
        for _ in 1..=184 {
            advance_visible_move_animation(&mut shell).unwrap();
        }
        assert_eq!(shell.visible_move_animations.front().unwrap().frame, 184);
        assert!(shell.visible_move_audio_wait.is_none());
        advance_visible_move_animation(&mut shell).unwrap();
        assert_eq!(shell.visible_move_audio_wait, Some(72));
        let retained = shell.battle_message_scene.clone();
        for elapsed in 0..72 {
            let animation = shell
                .visible_move_animations
                .front()
                .expect("wrapper still owns presentation");
            assert_eq!((animation.frame, animation.total_frames), (185, 185));
            assert_eq!(shell.visible_move_audio_wait, Some(72 - elapsed));
            assert!(visible_battle_animation_owns_frame(&shell));
            assert!(
                extract_immersive_battle_fixture(&shell, &snapshot, true)
                    .source
                    .is_none(),
                "OAM/script frame extraction ends at anim_ret, even while sound is busy"
            );
            assert_eq!(
                shell.battle_message_scene, retained,
                "HP scene cannot advance before WaitSFX"
            );
            advance_visible_move_animation(&mut shell).unwrap();
        }
        assert!(shell.visible_move_audio_wait.is_none());
        assert!(
            shell.visible_move_animations.is_empty(),
            "source97+160 ends at wrapper tick257"
        );
        assert_eq!(
            shell
                .pending_audio
                .iter()
                .filter(|sound| sound.audio_id == "SFX_SURF")
                .count(),
            4
        );
        assert_eq!(
            shell.shell.snapshot().unwrap(),
            snapshot,
            "presentation wait cannot mutate PP/HP/core state"
        );
    }
}

#[test]
fn immersive_psychic_shadow_ball_source_sounds_keep_frames_actor_and_operands() {
    for (move_id, sound_id, expected_frames) in [
        (
            "PSYCHIC_M",
            "SFX_PSYCHIC",
            vec![1, 9, 17, 25, 33, 41, 49, 57],
        ),
        ("SHADOW_BALL", "SFX_SLUDGE_BOMB", vec![1]),
    ] {
        for player_move in [true, false] {
            let mut shell = immersive_battle_fixture();
            let snapshot = shell.shell.snapshot().unwrap();
            let mut animation = immersive_source_animation(&snapshot, move_id);
            animation.player_move = player_move;
            let total_frames = animation.total_frames;
            assert_eq!(
                animation
                    .sound_events
                    .iter()
                    .map(|(f, _)| *f)
                    .collect::<Vec<_>>(),
                expected_frames
            );
            assert!(animation.sound_events.iter().all(|(_, sound)| {
                sound.id == sound_id && sound.args == Some(BattleSoundArgs::new(6, 2))
            }));
            shell.pending_audio.clear();
            shell.visible_move_animations.push_back(animation);
            let mut dispatched = Vec::new();
            for frame in 1..=total_frames {
                advance_visible_move_animation(&mut shell).unwrap();
                for command in std::mem::take(&mut shell.pending_audio) {
                    if command.audio_id == sound_id {
                        let playback = command.battle_sound.expect("source sound parameters");
                        assert_eq!(playback.args.packed, 26);
                        assert_eq!(playback.player_move, player_move);
                        dispatched.push(frame);
                    }
                }
            }
            assert_eq!(dispatched, expected_frames);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn immersive_audited_battle_sound_masks_preserve_source_programs_and_canonical_pcm() {
    let shell = immersive_battle_fixture();
    assert_eq!(
        AUDITED_BATTLE_SOUND_IDS.len(),
        24,
        "23 audited mode-2/3 IDs plus Surf"
    );
    for &audio_id in AUDITED_BATTLE_SOUND_IDS {
        let source = shell
            .shell
            .runtime()
            .audio()
            .require_sound_effect(audio_id)
            .unwrap()
            .source
            .clone();
        let AudioProgramSource::Midi {
            midi_base64,
            payload_hash,
            byte_len,
            loop_start_sample,
            loop_end_sample,
            ..
        } = &source
        else {
            panic!("bounded mask audit requires the bundled source program");
        };
        let expected_hash = payload_hash.clone();
        let expected_byte_len = *byte_len;
        assert!(loop_start_sample.is_none() && loop_end_sample.is_none());
        let program = crystal_audio::synth::decode_midi(midi_base64).unwrap();
        assert!(!program.music_data.channels.is_empty());
        assert!(
            program
                .music_data
                .channels
                .values()
                .all(|source| { matches!(source.number, Some(5..=8)) })
        );
        for command in program
            .music_data
            .channels
            .values()
            .chain(program.music_data.subroutines.values())
            .chain(program.music_data.shared_sources.values())
            .flat_map(|source| source.commands.iter())
        {
            assert!(
                !matches!(
                    command.command.as_str(),
                    "stereo_panning" | "force_stereo_panning" | "restart_channel" | "new_song"
                ),
                "a changed program requires a new panning audit: {audio_id}"
            );
        }
        let mut command = BevyAudioCommand {
            battle_sound: Some(BattleSoundPlayback {
                args: BattleSoundArgs::new(6, 2),
                player_move: true,
            }),
            cry_parameters: None,
            audio_id: audio_id.into(),
            kind: ModpackAudioKind::SoundEffect,
            mode: ModpackAudioPlaybackMode::RawPcm,
            looped: false,
        };
        // Production decoding validates synthesized hash/length against the
        // pack before the canonical waveform can enter the cache.
        let decoded = decoded_audio_program_source(&command, source).unwrap();
        assert_eq!(decoded.format.sample_rate_hz, 22_050);
        assert_eq!(decoded.format.channels, 2);
        assert_eq!(decoded.format.bits_per_sample, 16);
        assert_eq!(decoded.bytes.len(), expected_byte_len);
        assert_eq!(
            format!(
                "{:08x}",
                decoded.bytes.iter().fold(0x811c9dc5_u32, |hash, byte| {
                    (hash ^ u32::from(*byte)).wrapping_mul(0x01000193)
                })
            ),
            expected_hash
        );
        assert!(decoded.loop_range.is_none());
        let canonical = Arc::clone(&decoded.samples);
        let immutable = canonical.to_vec();
        assert_eq!(canonical.len(), expected_byte_len / 2);
        assert!(
            canonical.chunks_exact(2).all(|f| f[0] == f[1]),
            "{audio_id}"
        );
        let original_key = BevyAudioCacheKey::from_command(&command);
        for player_move in [true, false] {
            for tracks in 0..4 {
                command.battle_sound = Some(BattleSoundPlayback {
                    args: BattleSoundArgs::new(6, tracks),
                    player_move,
                });
                assert_eq!(BevyAudioCacheKey::from_command(&command), original_key);
                let routed = pcm_samples_for_audio_command(&canonical, Sound::Stereo, &command);
                assert_eq!(routed.len(), canonical.len());
                let right = (tracks == 1 || tracks == 3) == player_move;
                assert!(
                    routed
                        .chunks_exact(2)
                        .zip(canonical.chunks_exact(2))
                        .all(|(r, c)| {
                            if right {
                                r[0] == 0 && r[1] == c[1]
                            } else {
                                r[0] == c[0] && r[1] == 0
                            }
                        }),
                    "{audio_id}: player={player_move}, tracks={tracks}"
                );
                assert_eq!(
                    pcm_samples_for_audio_command(&canonical, Sound::Mono, &command),
                    pcm_samples_for_sound_option(&canonical, Sound::Mono)
                );
                assert_eq!(canonical.as_ref(), immutable.as_slice());
                assert!(Arc::ptr_eq(&canonical, &decoded.samples));
            }
        }
    }
}

#[test]
fn immersive_row_prototype_consumes_source_oam_through_terminal_tick() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let bundle = battle_anim_render_bundle(&mut RenderedTilesetArt::default(), &snapshot).unwrap();
    for (move_id, show, retire) in [("TACKLE", 20, 25), ("WATER_GUN", 94, 98)] {
        for player_move in [true, false] {
            let mut animation = battler_row_regression_animation(&snapshot, move_id, player_move);
            let player_strip = if move_id == "TACKLE" {
                !player_move
            } else {
                player_move
            };
            let index = usize::from(!player_strip);
            let mut playback = new_visible_battle_objects(&bundle, &animation).unwrap();
            for frame in 0..=animation.total_frames {
                animation.frame = frame;
                advance_visible_battle_objects(&mut playback, &bundle, &animation).unwrap();
                assert!(immersive_row_prototype_oam_supported(&playback.battler_rows));
                if frame == 2 {
                    let mut partial = playback.battler_rows.clone();
                    partial.iter_mut().flatten().next().unwrap().oam.rows[0][0] = false;
                    assert!(!immersive_row_prototype_oam_supported(&partial), "partial OAM must retain classic fallback");
                }
                let rows = immersive_row_prototype_rows(&animation, &playback.battler_rows);
                assert!(rows[1 - index].is_none());
                if (1..=retire).contains(&frame) {
                    let row = rows[index].expect("source OAM owns lifetime, including deinit tick");
                    assert_eq!(
                        row.source_y,
                        if player_strip {
                            Vec2::new(48.0, 64.0)
                        } else {
                            Vec2::new(40.0, 56.0)
                        }
                    );
                    assert_eq!(row.bg_cleared, frame > 1 && frame < show);
                } else {
                    assert!(rows[index].is_none());
                }
                advance_visible_battle_objects(&mut playback, &bundle, &animation).unwrap();
                assert_eq!(
                    immersive_row_prototype_rows(&animation, &playback.battler_rows),
                    rows,
                    "extracting F3 view at the same source frame is idempotent"
                );
            }
        }
    }
}

#[test]
fn immersive_row_prototype_tackle_uses_exact_source_scroll_band() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    for player_move in [true, false] {
        let mut animation = battler_row_regression_animation(&snapshot, "TACKLE", player_move);
        // Setup stores distance 0. Forward fill writes 0,2,4,6,8, then
        // return starts at10; fill precedes the +/-2 register update.
        for (age, distance) in [0, 0, 2, 4, 6, 8, 10, 8, 6, 4, 2, 0]
            .into_iter()
            .enumerate()
        {
            animation.frame = 7 + age as u16;
            let rows = immersive_row_prototype_tackle_scx(&animation).unwrap();
            for (row, value) in rows.into_iter().enumerate() {
                let in_band = if player_move {
                    (47..95).contains(&row)
                } else {
                    row < 55
                };
                assert_eq!(
                    value,
                    if in_band {
                        if player_move { -distance } else { distance }
                    } else {
                        0
                    }
                );
            }
        }
        animation.frame = 19;
        assert!(immersive_row_prototype_tackle_scx(&animation).is_none());
    }
}

#[test]
fn immersive_row_prototype_rejects_other_moves_and_unsupported_appearances() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let battler = |side| {
        Some(VisualBattleBattler {
            side,
            species_id: Arc::from("GENGAR"),
            pokedex_size_m: Some(1.4986),
            party_index: None,
            texture: Handle::weak_from_u128(15),
            texture_size: Vec2::splat(56.0),
            source_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            source_opaque_rect: Rect::new(96.0, 0.0, 152.0, 56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        })
    };
    let mut battlers = [
        battler(VisualBattleSide::Player),
        battler(VisualBattleSide::Enemy),
    ];
    for move_id in ["TACKLE", "WATER_GUN"] {
        let mut animation = battler_row_regression_animation(&snapshot, move_id, true);
        assert!(immersive_row_prototype_supported(&animation, &battlers));
        animation.animation_label = format!(
            "BattleAnim_SubstituteLower -> {}",
            animation.animation_label
        );
        assert!(!immersive_row_prototype_supported(&animation, &battlers));
    }
    let animation = battler_row_regression_animation(&snapshot, "TACKLE", true);
    battlers[0].as_mut().unwrap().allow_species_model = false;
    assert!(!immersive_row_prototype_supported(&animation, &battlers));
    battlers[0].as_mut().unwrap().allow_species_model = true;
    battlers[0].as_mut().unwrap().shiny = true;
    assert!(!immersive_row_prototype_supported(&animation, &battlers));
    battlers[0].as_mut().unwrap().shiny = false;
    battlers[1].as_mut().unwrap().visible = false;
    assert!(!immersive_row_prototype_supported(&animation, &battlers));
    battlers[1].as_mut().unwrap().visible = true;
    for move_id in ["GROWL", "BODY_SLAM", "SURF"] {
        assert!(!immersive_row_prototype_supported(
            &battler_row_regression_animation(&snapshot, move_id, true),
            &battlers
        ));
    }
}
