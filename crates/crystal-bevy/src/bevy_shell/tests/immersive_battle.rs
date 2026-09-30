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
    let shell = prepare_immersive_battle_preview(shell, shadow_ball, psychic, hyper_beam).unwrap();
    VisibleShellController { shell }
}

#[cfg(feature = "location-tester")]
#[test]
fn immersive_battle_shadow_ball_preview_uses_legal_tm_and_normal_controller() {
    let mut controller = immersive_battle_move_preview_controller(true, false, false);
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
    let mut controller = immersive_battle_move_preview_controller(false, true, false);
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
    let mut controller = immersive_battle_move_preview_controller(false, false, true);
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
