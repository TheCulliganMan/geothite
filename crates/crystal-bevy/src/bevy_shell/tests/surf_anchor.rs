fn surf_anchor_preview_shell() -> BevyRuntimeShell {
    let asset_root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap(),
    );
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "Route44".into(),
            tile_x: 38,
            tile_y: 3,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            quick_save_path: None,
            ..Default::default()
        },
    )
    .unwrap();
    prepare_surf_encounter_preview(shell).expect("checked fresh Route44 Surf setup")
}

fn surf_anchor_app_at_shore() -> App {
    let mut shell = surf_anchor_preview_shell();
    shell.shell.set_runtime_journal_enabled(true);
    let mut app = menu_render_test_app(shell);
    app.init_resource::<crystal_render_api::VisualWorldFrame>()
        .init_resource::<crystal_render_api::BattleLocationFrame>()
        .add_systems(
            Update,
            publish_visible_battle_location
                .after(play_pending_audio)
                .after(tick_visible_screen_fade)
                .before(render_playfield),
        )
        .add_systems(
            Update,
            publish_visual_world_frame
                .after(render_playfield)
                .after(sync_visible_player_sprite)
                .after(sync_visible_object_sprites),
        );
    app.update();
    app.update();
    app
}

/// Only this unit helper installs deterministic samples. The native disposable
/// fixture above retains the actual divider, rate, slot and level variation.
fn surf_anchor_set_test_trace(app: &mut App) {
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    shell.shell.session_mut().state_mut().random_state =
        crate::core::random::CrystalRandomState::default();
    *shell.shell.session_mut().divider_mut_for_tests() =
        crate::core::random::RuntimeDividerSource::replay(vec![0; 32768]);
}

fn surf_anchor_mount_through_keyboard(app: &mut App) {
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .snapshot()
            .mode,
        MovementMode::Normal
    );
    press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            shell.pending_contextual_field_move,
            Some(PartyFieldMove::Surf),
            "first A state: error={:?}, notice={:?}, mode={:?}, mount={:?}, cursor={:?}",
            shell.last_error,
            shell.field_notice,
            shell.shell.session().snapshot().mode,
            shell.pending_surf_start_from,
            shell.yes_no_cursor
        );
        assert_eq!(
            shell.yes_no_cursor.as_ref().unwrap().surface_id,
            "field:move-confirm"
        );
        assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index, 0);
        assert_eq!(
            shell.shell.session().snapshot().tile,
            TilePosition::new(38, 3)
        );
        assert_eq!(shell.shell.session().snapshot().mode, MovementMode::Normal);
    }
    let mut witnessed_mount = false;
    for _ in 0..160 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert!(shell.battle_origin.published().is_none());
        assert!(shell.battle_origin.bound_surf.is_none());
        assert!(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .is_none()
        );
        if shell.shell.session().snapshot().mode == MovementMode::Surf {
            assert_eq!(
                shell.shell.session().snapshot().tile,
                TilePosition::new(38, 4)
            );
            if shell.player_walk_frame_ticks > 0 {
                witnessed_mount = true;
                assert_eq!(shell.player_walk_from, Some(TilePosition::new(38, 3)));
                assert_eq!(shell.player_walk_total_ticks, 16);
            }
            if shell.player_walk_frame_ticks == 0
                && shell.pending_surf_start_from.is_none()
                && shell.field_notice.is_none()
            {
                assert!(
                    witnessed_mount,
                    "the actual sixteen-frame source mount was rendered"
                );
                return;
            }
            app.update();
        } else {
            press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
        }
    }
    panic!("ordinary A/Yes must finish the authored Surf mount");
}

fn surf_anchor_roll_until_battle(app: &mut App) {
    for step in 0..24 {
        let (key, target) = match step {
            0 => (KeyCode::ArrowDown, TilePosition::new(38, 5)),
            1 => (KeyCode::ArrowDown, TilePosition::new(38, 6)),
            _ if step % 2 == 0 => (KeyCode::ArrowLeft, TilePosition::new(37, 6)),
            _ => (KeyCode::ArrowRight, TilePosition::new(38, 6)),
        };
        for _ in 0..32 {
            press_key_for_runtime_hotkey_app(app, key);
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            assert_eq!(shell.shell.session().snapshot().mode, MovementMode::Surf);
            if shell.battle_origin.published().is_some() {
                return;
            }
            assert!(
                app.world()
                    .resource::<crystal_render_api::BattleLocationFrame>()
                    .location
                    .is_none()
            );
            if shell.shell.session().snapshot().tile == target {
                for _ in 0..10 {
                    app.update();
                }
                break;
            }
        }
    }
    panic!("ordinary Surf input must consume cooldown and roll the source water encounter");
}

fn surf_anchor_committed_app() -> App {
    let mut app = surf_anchor_app_at_shore();
    surf_anchor_set_test_trace(&mut app);
    surf_anchor_mount_through_keyboard(&mut app);
    surf_anchor_roll_until_battle(&mut app);
    app
}

#[test]
fn surf_anchor_preview_awaits_real_controller_a_yes_before_water_movement() {
    let mut controller = VisibleShellController {
        shell: surf_anchor_preview_shell(),
    };
    let before = controller.shell.shell.snapshot().unwrap();
    assert_eq!(before.overworld.tile, TilePosition::new(38, 3));
    assert_eq!(before.overworld.mode, MovementMode::Normal);
    assert!(before.battle.is_none() && controller.shell.battle_origin.published().is_none());
    assert_eq!(before.party.slots.len(), 1);
    assert_eq!(before.party.slots[0].pokemon.species.id, "TOTODILE");
    assert_eq!(before.party.slots[0].pokemon.level, 20);
    assert_eq!(before.party.slots[0].pokemon.moves[0].name, "SURF");
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .wild_encounter_cooldown,
        5
    );
    controller.press(GameButton::A).unwrap();
    assert_eq!(
        controller.shell.pending_contextual_field_move,
        Some(PartyFieldMove::Surf),
        "first controller A state: error={:?}, notice={:?}, mode={:?}, mount={:?}",
        controller.shell.last_error,
        controller.shell.field_notice,
        controller.shell.shell.session().snapshot().mode,
        controller.shell.pending_surf_start_from
    );
    assert_eq!(
        controller.shell.shell.session().snapshot().mode,
        MovementMode::Normal
    );
    for _ in 0..12 {
        controller.press(GameButton::A).unwrap();
        if controller.shell.shell.session().snapshot().mode == MovementMode::Surf {
            break;
        }
    }
    assert_eq!(
        controller.shell.shell.session().snapshot().mode,
        MovementMode::Surf
    );
    assert_eq!(
        controller.shell.shell.session().snapshot().tile,
        TilePosition::new(38, 4)
    );
    assert!(controller.shell.battle_origin.bound_surf.is_none());
    assert!(controller.shell.shell.snapshot().unwrap().battle.is_none());
    assert_eq!(
        controller
            .shell
            .shell
            .current_encounter_surface_checked()
            .unwrap(),
        Some(crate::core::world::encounters::EncounterSurface::Water)
    );
}

#[test]
fn surf_anchor_classifier_is_distinct_and_accepts_only_plain_water_and_ordinary_surf() {
    use crate::core::world::encounters::EncounterSurface;
    for permission in u8::MIN..=u8::MAX {
        assert_eq!(
            surf_plain_support(permission),
            permission == crate::core::world::collision::permissions::WATER
        );
    }
    let mut origin = battle_origin_test_value();
    origin.source.mode = MovementMode::Surf;
    let to = origin.source.tile;
    let from = TilePosition::new(to.x - 1, to.y);
    origin.contact = Some(BattleOriginContact::OverworldEncounter {
        map_id: origin.source.map_name.clone(),
        tile: to,
        surface: EncounterSurface::Water,
    });
    origin.authoritative_step = Some(StepOutcome::Moved {
        from,
        to,
        speed_multiplier: 1,
    });
    assert_eq!(surf_origin_step(&origin), Some((from, to)));
    assert_eq!(walking_origin_step(&origin), None);
    for mode in [
        MovementMode::Normal,
        MovementMode::Bike,
        MovementMode::Skate,
        MovementMode::SurfPika,
    ] {
        let mut other = origin.clone();
        other.source.mode = mode;
        assert_eq!(surf_origin_step(&other), None);
    }
    for defect in 0..12 {
        let mut other = origin.clone();
        match defect {
            0 => other.contact = None,
            1 => other.source.map_name = "OtherWater".into(),
            2 => other.source.tile.x += 1,
            3 => other.source.facing = Direction::Left,
            4 => other.kind = BattleOriginKind::StaticWild,
            5 => other.battle_type = "BATTLETYPE_FISH".into(),
            6 => other.authoritative_step = None,
            7 => {
                other.authoritative_step = Some(StepOutcome::Turned {
                    facing: Direction::Right,
                })
            }
            8 => {
                other.authoritative_step = Some(StepOutcome::Moved {
                    from,
                    to,
                    speed_multiplier: 2,
                })
            }
            9 => {
                other.authoritative_step = Some(StepOutcome::Moved {
                    from: TilePosition::new(from.x - 1, from.y),
                    to,
                    speed_multiplier: 1,
                })
            }
            10 => other.visual_step.ledge_jump = Some((from, to, 0)),
            11 => {
                other.contact = Some(BattleOriginContact::OverworldEncounter {
                    map_id: other.source.map_name.clone(),
                    tile: to,
                    surface: EncounterSurface::Grass,
                })
            }
            _ => unreachable!(),
        }
        assert_eq!(
            surf_origin_step(&other),
            None,
            "unsupported Surf origin {defect}"
        );
    }
}

#[test]
fn surf_anchor_real_keyboard_mount_and_encounter_preserve_contact_journal_and_lifetime() {
    use bevy::ecs::system::RunSystemOnce;
    let mut app = surf_anchor_app_at_shore();
    surf_anchor_set_test_trace(&mut app);
    let (mut replay, commands_before) = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        (
            shell.shell.session().clone(),
            shell.shell.retained_runtime_commands().len(),
        )
    };
    surf_anchor_mount_through_keyboard(&mut app);
    surf_anchor_roll_until_battle(&mut app);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let origin = shell.battle_origin.published().unwrap().clone();
    let (from, to) = surf_origin_step(&origin).expect("actual ordinary Surf step");
    assert!(matches!(to, TilePosition { x: 37 | 38, y: 6 }));
    assert_eq!(origin.source.mode, MovementMode::Surf);
    assert_eq!(origin.visual_step.walk_from, Some(from));
    assert!(origin.visual_step.walk_ticks_remaining > 0);
    assert_eq!(origin.visual_step.walk_total_ticks, 8);
    assert_eq!(
        origin.contact,
        Some(BattleOriginContact::OverworldEncounter {
            map_id: "Route44".into(),
            tile: to,
            surface: crate::core::world::encounters::EncounterSurface::Water
        })
    );
    assert!(shell.battle_origin.bound_walking.is_none());
    let bound = shell.battle_origin.bound_surf.as_ref().unwrap();
    assert_eq!(
        bound.placement.presentation_core_tile,
        if origin.source.facing == Direction::Left {
            IVec2::new(35, 6)
        } else {
            IVec2::new(40, 6)
        }
    );
    let overworld = shell.shell.session().overworld();
    let occupied = overworld.occupied_tiles_checked().unwrap();
    for tile in bound.placement.water_core_tiles.iter() {
        let tile = TilePosition::new(tile.x as i16, tile.y as i16);
        assert!(!occupied.iter().any(|entry| entry.tile == tile));
        assert!(
            crate::core::world::collision::sample_collision(
                &overworld.map,
                &overworld.tileset,
                tile
            )
            .is_some_and(|sample| surf_plain_support(sample.permission))
        );
    }
    let runtime = shell.shell.runtime().clone();
    for command in &shell.shell.retained_runtime_commands()[commands_before..] {
        replay
            .apply_runtime_command_frame(&runtime, command)
            .unwrap();
    }
    assert_eq!(
        replay.state(),
        shell.shell.session().state(),
        "journal reproduces source A/Yes, mounting, clock, RNG, moves and encounter"
    );
    for _ in 0..16 {
        app.update();
        if app
            .world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_some()
        {
            break;
        }
    }
    let location = app
        .world()
        .resource::<crystal_render_api::BattleLocationFrame>()
        .location
        .clone()
        .unwrap();
    let anchors = location
        .anchors
        .as_ref()
        .expect("actual Surf avatar and retained water frame");
    let crystal_render_api::VisualBattleTarget::SurfWater {
        core_tile,
        presentation,
    } = &location.target
    else {
        panic!("distinct SurfWater contract");
    };
    assert_eq!(*core_tile, IVec2::new(i32::from(to.x), i32::from(to.y)));
    assert_eq!(location.source.core_tile, *core_tile);
    assert_eq!(
        presentation.step_from_core_tile,
        IVec2::new(i32::from(from.x), i32::from(from.y))
    );
    assert_eq!(anchors.target_actor, None);
    assert_eq!(
        anchors.source_foot,
        fishing_source_support(&anchors.terrain, to).unwrap()
    );
    assert!(walking_foot_is_on_original_step(
        presentation.witnessed_player_foot.unwrap(),
        fishing_source_support(&anchors.terrain, from).unwrap(),
        anchors.source_foot
    ));
    assert_eq!(
        anchors.target_foot - anchors.source_foot,
        visual_facing(origin.source.facing) * TILE_SIZE * 4.0
    );
    let (session, commands, results) = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        (
            shell.shell.session().clone(),
            shell.shell.retained_runtime_commands().to_vec(),
            shell.shell.retained_runtime_results().to_vec(),
        )
    };
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session(), &session);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
    let mut controller = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .unwrap(),
    };
    prepare_visible_battle_entry_after_visible_step(&mut controller.shell).unwrap();
    prepare_visible_battle_entry_after_visible_step(&mut controller.shell).unwrap();
    assert!(Arc::ptr_eq(
        controller.shell.battle_origin.published().unwrap(),
        &origin
    ));
    assert!(Arc::ptr_eq(
        controller
            .shell
            .battle_origin
            .bound_surf
            .as_ref()
            .unwrap()
            .publication
            .as_ref()
            .unwrap(),
        &location
    ));
    assert!(!controller.shell.battle_messages.is_empty());
    reset_visible_battle_exit_state(&mut controller.shell);
    assert!(
        controller.shell.battle_origin.bound_surf.is_some(),
        "terminal text retains Surf evidence"
    );
    controller.shell.battle_messages.clear();
    reset_visible_battle_exit_state(&mut controller.shell);
    assert!(controller.shell.battle_origin.bound_surf.is_none());
    app.insert_resource(controller.shell);
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    assert!(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_none()
    );
    assert!(
        location.anchors.is_some(),
        "retained readers remain immutable after exit"
    );
}

#[test]
fn surf_anchor_actual_avatar_witness_rejects_cold_stale_wrong_mode_and_wrong_source_once() {
    let mut app = surf_anchor_committed_app();
    let frame = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    let mut rendered = app
        .world_mut()
        .remove_resource::<RenderedViewport>()
        .unwrap();
    let shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let mut prototype = shell.battle_origin.bound_surf.as_ref().unwrap().clone();
    prototype.warm_checked = false;
    prototype.capture_attempted = false;
    prototype.anchors = None;
    prototype.placement.witnessed_player_foot = None;
    prototype.publication = None;
    let actor = frame
        .actors
        .iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .unwrap();
    assert_eq!(
        actor.source_id.as_ref(),
        "surf",
        "actual mounted avatar identity"
    );
    assert_eq!(rendered.player_sprite_mode, Some(MovementMode::Surf));
    let mut good = prototype.clone();
    freeze_visible_surf_anchors(&mut good, &rendered, Some(&frame), true);
    assert!(
        good.anchors.is_some(),
        "warm actual source avatar and water correspond"
    );
    for defect in 0..13 {
        let mut bad = frame.clone();
        let mut bound = prototype.clone();
        let old_mode = rendered.player_sprite_mode;
        let old_revision = rendered.snapshot_revision;
        let old_tile = rendered.tile;
        match defect {
            0 => bad.active = false,
            1 => bad.terrain_revision += 1,
            2 => bad.grid_origin.x += 1,
            3 => bad.source_map_size_core_tiles = None,
            4 => bad
                .actors
                .retain(|actor| actor.id != crystal_render_api::VisualActorId::Player),
            5 => {}
            6 => bad.map_id = Arc::from("UnrelatedWater"),
            7 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .source_id = Arc::from("chris")
            }
            8 => rendered.player_sprite_mode = Some(MovementMode::Normal),
            9 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .center += Vec2::new(2.0, 2.0)
            }
            10 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .facing = None
            }
            11 => {
                rendered.snapshot_revision = Some(bound.minimum_snapshot_revision.wrapping_sub(1))
            }
            12 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .source_id = Arc::from("surfing_pikachu")
            }
            _ => unreachable!(),
        }
        // A stale destination revision is permitted to wait only within the
        // finite entry window. Closing that window must freeze fallback forever.
        if defect == 11 {
            rendered.tile = Some(bound.origin.source.tile);
        }
        freeze_visible_surf_anchors(&mut bound, &rendered, (defect != 5).then_some(&bad), true);
        if !bound.capture_attempted {
            freeze_visible_surf_anchors(
                &mut bound,
                &rendered,
                (defect != 5).then_some(&bad),
                false,
            );
        }
        assert!(bound.capture_attempted, "Surf witness defect {defect}");
        assert!(bound.anchors.is_none(), "Surf witness defect {defect}");
        rendered.player_sprite_mode = old_mode;
        rendered.snapshot_revision = old_revision;
        rendered.tile = old_tile;
        freeze_visible_surf_anchors(&mut bound, &rendered, Some(&frame), true);
        assert!(
            bound.anchors.is_none(),
            "later water cannot repair defect {defect}"
        );
    }
}

#[test]
fn surf_anchor_binding_excludes_mount_special_water_and_competing_source_paths() {
    use crate::core::world::collision::{permissions as p, sample_collision};
    use crate::core::world::session::{
        ConnectionDestination, ConnectionTransition, ConnectionTrigger, CoordEventTrigger,
        OverworldInteraction, OverworldInteractionTarget, WarpDestination, WarpTransition,
        WarpTrigger,
    };
    let mut app = surf_anchor_committed_app();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let origin = shell.battle_origin.published().unwrap().clone();
    let frame = shell.shell.last_frame().unwrap().clone();
    assert_eq!(origin.source, frame.snapshot);
    shell.battle_origin.active = None;
    shell.battle_origin.pending = Some(origin.clone());
    let session = shell.shell.session().clone();
    let (from, to) = surf_origin_step(&origin).unwrap();
    for tile in [from, to] {
        let sample =
            sample_collision(&session.overworld().map, &session.overworld().tileset, tile).unwrap();
        for permission in [
            p::FLOOR,
            p::ICE,
            p::WALK_RIGHT,
            p::HOP_DOWN,
            p::WARP_PANEL,
            p::DOOR,
            p::CURRENT_DOWN,
            p::CURRENT_RIGHT,
            p::CURRENT_LEFT,
            p::CURRENT_UP,
            p::WATERFALL_RIGHT,
            p::WATERFALL_LEFT,
            p::WATERFALL_UP,
            p::WATERFALL,
            p::WHIRLPOOL,
            p::WHIRLPOOL_2C,
        ] {
            *shell.shell.session_mut() = session.clone();
            shell.battle_origin.bound_surf = None;
            shell.shell.session_mut().overworld_mut().tileset.metatiles
                [usize::from(sample.metatile_id)]
            .collision[sample.quadrant] = permission;
            let before = shell.shell.session().clone();
            let commands = shell.shell.retained_runtime_commands().to_vec();
            let results = shell.shell.retained_runtime_results().to_vec();
            bind_visible_surf_encounter(&mut shell, &frame);
            assert!(
                shell.battle_origin.bound_surf.is_none(),
                "special water endpoint {permission:#x}"
            );
            assert_eq!(shell.shell.session(), &before);
            assert_eq!(shell.shell.retained_runtime_commands(), commands);
            assert_eq!(shell.shell.retained_runtime_results(), results);
        }
    }
    *shell.shell.session_mut() = session.clone();
    let interaction = OverworldInteraction {
        map_name: "Route44".into(),
        player_tile: to,
        facing: origin.source.facing,
        target_tile: to,
        script: "TestScript".into(),
        target: OverworldInteractionTarget::Collision {
            permission: p::WATER,
        },
    };
    let warp = shell.shell.session().overworld().map_events.warps[0].clone();
    for defect in 0..9 {
        let mut candidate = frame.clone();
        shell.battle_origin.bound_surf = None;
        match defect {
            0 => {
                candidate.coord_event = Some(CoordEventTrigger {
                    map_name: "Route44".into(),
                    tile: to,
                    scene_id: "SCENE_TEST".into(),
                    script_name: "TestScript".into(),
                })
            }
            1 => candidate.trainer_sight = Some(interaction.clone()),
            2 => candidate.interaction = Some(interaction.clone()),
            3 => {
                candidate.phone_call = Some(crystal_assets::IncomingPhoneCall {
                    kind: crystal_assets::IncomingPhoneCallKind::Ordinary,
                    contact_id: "TEST".into(),
                    caller_script: "TestCaller".into(),
                    receive_script: "TestReceive".into(),
                    delay_frames: 0,
                })
            }
            4 => {
                candidate.ledge_jump =
                    Some(crate::core::world::movement::LedgeJumpOutcome::Jumped {
                        from,
                        over: to,
                        to,
                        speed_multiplier: 1,
                    })
            }
            5 => {
                candidate.connection = Some(ConnectionTransition {
                    trigger: ConnectionTrigger {
                        map_name: "Route44".into(),
                        tile: to,
                        connection: crate::core::map::MapConnection {
                            direction: "east".into(),
                            target_map: "OtherWater".into(),
                            offset: 0,
                        },
                    },
                    destination: ConnectionDestination {
                        map_name: "OtherWater".into(),
                        tile: to,
                    },
                })
            }
            6 => {
                candidate.warp = Some(WarpTransition {
                    trigger: WarpTrigger {
                        map_name: "Route44".into(),
                        tile: to,
                        permission: p::WATER,
                        warp: warp.clone(),
                    },
                    destination: WarpDestination {
                        map_name: "OtherWater".into(),
                        tile: to,
                        warp: warp.clone(),
                    },
                })
            }
            7 => {
                candidate.movement = Some(StepOutcome::Turned {
                    facing: origin.source.facing,
                })
            }
            8 => shell.pending_surf_start_from = Some(from),
            _ => unreachable!(),
        }
        bind_visible_surf_encounter(&mut shell, &candidate);
        assert!(
            shell.battle_origin.bound_surf.is_none(),
            "competing source path {defect}"
        );
        shell.pending_surf_start_from = None;
    }
    bind_visible_surf_encounter(&mut shell, &frame);
    assert!(shell.battle_origin.bound_surf.is_some());
    // Even a water-typed committed record cannot turn the scripted shore mount
    // into an ordinary Surf encounter: the original from tile is still land.
    let mut mount = (*origin).clone();
    mount.source.tile = TilePosition::new(38, 4);
    mount.source.facing = Direction::Down;
    mount.contact = Some(BattleOriginContact::OverworldEncounter {
        map_id: "Route44".into(),
        tile: mount.source.tile,
        surface: crate::core::world::encounters::EncounterSurface::Water,
    });
    mount.authoritative_step = Some(StepOutcome::Moved {
        from: TilePosition::new(38, 3),
        to: mount.source.tile,
        speed_multiplier: 1,
    });
    {
        let overworld = shell.shell.session_mut().overworld_mut();
        overworld.player.tile = mount.source.tile;
        overworld.player.facing = mount.source.facing;
    }
    let mut mount_frame = frame.clone();
    mount_frame.snapshot = mount.source.clone();
    mount_frame.movement = mount.authoritative_step.clone();
    shell.battle_origin.pending = Some(Arc::new(mount));
    shell.battle_origin.bound_surf = None;
    bind_visible_surf_encounter(&mut shell, &mount_frame);
    assert!(
        shell.battle_origin.bound_surf.is_none(),
        "land-to-water mount is excluded"
    );
    *shell.shell.session_mut() = session;
    shell.battle_origin.pending = Some(origin);
    bind_visible_surf_encounter(&mut shell, &frame);
    assert!(shell.battle_origin.bound_surf.is_some());
    reset_visible_navigation_state(&mut shell);
    assert!(
        shell.battle_origin.bound_surf.is_some(),
        "ordinary navigation reset preserves committed provenance across retained narration"
    );
    reset_visible_battle_presentation(&mut shell);
    assert!(shell.battle_origin.bound_surf.is_none());
}

#[test]
fn surf_anchor_support_excludes_runtime_occupancy_and_requires_verified_metadata() {
    let mut app = surf_anchor_committed_app();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let origin = shell.battle_origin.published().unwrap().clone();
    let frame = shell.shell.last_frame().unwrap().clone();
    shell.battle_origin.active = None;
    shell.battle_origin.pending = Some(origin.clone());
    let initial = shell.battle_origin.bound_surf.take().unwrap();
    let blocked_core = initial.placement.presentation_core_tile;
    let blocked = TilePosition::new(blocked_core.x as i16, blocked_core.y as i16);
    let session = shell.shell.session().clone();
    let object_id = {
        let overworld = shell.shell.session().overworld();
        overworld
            .objects
            .iter()
            .enumerate()
            .find_map(|(index, object)| {
                overworld
                    .object_has_loaded_struct(index)
                    .then(|| object.object_identifier.clone())
                    .flatten()
            })
            .expect("Route44 has a loaded ordinary actor for occupancy regression")
    };
    shell
        .shell
        .session_mut()
        .overworld_mut()
        .set_object_runtime_tile(&object_id, blocked)
        .unwrap();
    assert!(
        shell
            .shell
            .session()
            .overworld()
            .occupied_tiles_checked()
            .unwrap()
            .iter()
            .any(|entry| entry.tile == blocked)
    );
    let before = shell.shell.session().clone();
    bind_visible_surf_encounter(&mut shell, &frame);
    if let Some(bound) = &shell.battle_origin.bound_surf {
        assert!(!bound.placement.water_core_tiles.contains(&blocked_core));
        assert_ne!(bound.placement.presentation_core_tile, blocked_core);
    }
    assert_eq!(
        shell.shell.session(),
        &before,
        "occupancy observation cannot move the actor or trainer"
    );
    *shell.shell.session_mut() = session.clone();
    // A second genuine runtime metadata ID proves this source classifier is
    // not a Route44 preview-name allowlist. Contact and terrain still match.
    for (map_name, accepted) in [("Route32", true), ("UnknownSurfMap", false)] {
        *shell.shell.session_mut() = session.clone();
        shell.shell.session_mut().overworld_mut().map.name = map_name.into();
        let mut candidate_origin = (*origin).clone();
        candidate_origin.source.map_name = map_name.into();
        candidate_origin.contact = Some(BattleOriginContact::OverworldEncounter {
            map_id: map_name.into(),
            tile: candidate_origin.source.tile,
            surface: crate::core::world::encounters::EncounterSurface::Water,
        });
        let mut candidate_frame = frame.clone();
        candidate_frame.snapshot = candidate_origin.source.clone();
        shell.battle_origin.pending = Some(Arc::new(candidate_origin));
        shell.battle_origin.bound_surf = None;
        bind_visible_surf_encounter(&mut shell, &candidate_frame);
        assert_eq!(
            shell.battle_origin.bound_surf.is_some(),
            accepted,
            "checked runtime metadata gate for {map_name}"
        );
    }
}

#[test]
fn surf_anchor_run_returns_to_committed_water_and_ordinary_shore_step_dismounts() {
    let mut app = surf_anchor_committed_app();
    let mut controller = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .unwrap(),
    };
    let origin = controller.shell.battle_origin.published().unwrap().clone();
    for _ in 0..64 {
        if controller.shell.battle_action_cursor.is_some()
            && controller.shell.battle_messages.is_empty()
        {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    assert!(controller.shell.battle_action_cursor.is_some());
    assert!(controller.shell.battle_messages.is_empty());
    assert!(Arc::ptr_eq(
        controller.shell.battle_origin.published().unwrap(),
        &origin
    ));
    controller.press(GameButton::Down).unwrap();
    controller.press(GameButton::Right).unwrap();
    let ready = controller.snapshot().unwrap();
    assert!(
        ready.ui.menu.as_ref().unwrap().layout.vertical_menus[0]
            .options
            .iter()
            .any(|entry| entry.trim() == ">RUN")
    );
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.shell.snapshot().unwrap().battle.is_none());
    let terminal_text = controller
        .shell
        .battle_messages
        .front()
        .cloned()
        .expect("successful RUN retains its source narration");
    assert_eq!(terminal_text, "Got away safely!");
    assert!(
        visible_battle_message_is_complete(&controller.shell, &terminal_text),
        "renderer-neutral settling reveals terminal text after core battle ends"
    );
    assert!(Arc::ptr_eq(
        controller.shell.battle_origin.published().unwrap(),
        &origin
    ));
    for _ in 0..64 {
        if controller.shell.shell.snapshot().unwrap().battle.is_none()
            && controller.shell.battle_origin.published().is_none()
        {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    let returned = controller.shell.shell.snapshot().unwrap();
    assert!(
        returned.battle.is_none(),
        "ordinary RUN succeeds with the unit's source random trace"
    );
    assert!(controller.shell.battle_origin.published().is_none());
    assert!(controller.shell.battle_origin.bound_surf.is_none());
    assert_eq!(returned.overworld.tile, origin.source.tile);
    assert_eq!(returned.overworld.facing, origin.source.facing);
    assert_eq!(returned.overworld.mode, MovementMode::Surf);
    let mut targets = Vec::new();
    if returned.overworld.tile.x == 37 {
        targets.push((GameButton::Right, TilePosition::new(38, 6)));
    }
    targets.extend([
        (GameButton::Up, TilePosition::new(38, 5)),
        (GameButton::Up, TilePosition::new(38, 4)),
        (GameButton::Up, TilePosition::new(38, 3)),
    ]);
    for (button, target) in targets {
        for _ in 0..4 {
            controller.press(button).unwrap();
            if controller.shell.shell.session().snapshot().tile == target {
                break;
            }
        }
        let snapshot = controller.shell.shell.snapshot().unwrap();
        assert!(snapshot.battle.is_none());
        assert_eq!(snapshot.overworld.tile, target);
        assert_eq!(
            snapshot.overworld.mode,
            if target.y == 3 {
                MovementMode::Normal
            } else {
                MovementMode::Surf
            }
        );
    }
    assert!(controller.shell.battle_origin.published().is_none());
}

#[test]
fn surf_anchor_prompt_follows_the_actual_packed_tx_far_body_without_runtime_mutation() {
    let mut shell = surf_anchor_preview_shell();
    let snapshot = shell.shell.snapshot().unwrap();
    let definitions = &shell
        .shell
        .runtime()
        .data()
        .global_scripts
        .as_ref()
        .unwrap()
        .definitions;
    let wrapper = definitions.get("AskSurfText").unwrap();
    assert_eq!(
        wrapper,
        &serde_json::json!([
            {"command":"text_far","args":["_AskSurfText"]},
            {"command":"text_end","args":[]}
        ])
    );
    assert!(!snapshot.presentation.asm_text.contains_key("AskSurfText"));
    let target = visible_contextual_field_text_label(
        "AskSurfText",
        &snapshot.presentation.asm_text,
        Some(definitions),
    )
    .unwrap();
    assert_eq!(target, "_AskSurfText");
    let body = shell
        .shell
        .text_snapshot(&target)
        .unwrap()
        .asm_text
        .unwrap();
    assert_eq!(body, snapshot.presentation.asm_text[&target]);
    assert_eq!(body, "The water is calm.\nWant to SURF?");
    assert_eq!(
        visible_contextual_field_text_label(&target, &snapshot.presentation.asm_text, None)
            .unwrap(),
        target
    );
    let session = shell.shell.session().clone();
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    assert_eq!(visible_asm_text(&snapshot, &target).unwrap(), body);
    assert_eq!(shell.shell.session(), &session);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
    assert!(
        open_visible_contextual_field_move_prompt(
            &mut shell,
            PartyFieldMove::Surf,
            "MissingPrompt"
        )
        .is_err()
    );
    assert!(shell.pending_contextual_field_move.is_none());
    assert!(shell.yes_no_cursor.is_none());
    assert!(shell.field_notice.is_none());
    assert_eq!(shell.shell.session(), &session);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
    open_visible_contextual_field_move_prompt(&mut shell, PartyFieldMove::Surf, "AskSurfText")
        .unwrap();
    assert_eq!(shell.field_notice.as_deref(), Some(body.as_str()));
}

#[test]
fn surf_anchor_prompt_rejects_malformed_cycles_and_oversized_source_pointer_chains() {
    let catalog = BTreeMap::from([("Body".to_string(), "Source prompt".to_string())]);
    let wrapper = |target: &str| {
        serde_json::json!([
            {"command":"text_far","args":[target]}, {"command":"text_end","args":[]}
        ])
    };
    let definitions = BTreeMap::from([("Ask".into(), wrapper("Body"))]);
    assert_eq!(
        visible_contextual_field_text_label("Ask", &catalog, Some(&definitions)).unwrap(),
        "Body"
    );
    for bad in [
        serde_json::json!([]),
        serde_json::json!([{"command":"text_far","args":["Body"]}]),
        serde_json::json!([{"command":"text_far","args":["Body"]},{"command":"done","args":[]}]),
        serde_json::json!([{"command":"text_far","args":["Body","Other"]},{"command":"text_end","args":[]}]),
        serde_json::json!([{"command":"text_far","args":["Body"]},{"command":"text_end","args":["Extra"]}]),
        serde_json::json!([{"command":"text_far","args":["Body"]},{"command":"text_end","args":[]},{"command":"text","args":["Extra"]}]),
        wrapper(""),
        wrapper(" Body"),
        wrapper("Missing"),
        wrapper("Ask"),
    ] {
        let definitions = BTreeMap::from([("Ask".to_string(), bad)]);
        assert!(visible_contextual_field_text_label("Ask", &catalog, Some(&definitions)).is_err());
    }
    let mut definitions = BTreeMap::from([
        ("Ask".to_string(), wrapper("Other")),
        ("Other".to_string(), wrapper("Ask")),
    ]);
    assert!(
        visible_contextual_field_text_label("Ask", &catalog, Some(&definitions))
            .unwrap_err()
            .to_string()
            .contains("cycle")
    );
    definitions.clear();
    for index in 0..8 {
        definitions.insert(
            format!("Pointer{index}"),
            wrapper(&format!("Pointer{}", index + 1)),
        );
    }
    definitions.insert("Pointer8".into(), wrapper("Body"));
    assert!(
        visible_contextual_field_text_label("Pointer0", &catalog, Some(&definitions))
            .unwrap_err()
            .to_string()
            .contains("eight")
    );
    // A direct catalog body wins exactly as before even with an unrelated or
    // malformed definition under the same name.
    definitions.insert("Body".into(), serde_json::json!({"bad":"definition"}));
    assert_eq!(
        visible_contextual_field_text_label("Body", &catalog, Some(&definitions)).unwrap(),
        "Body"
    );
}

fn surf_anchor_field_use_count(
    shell: &BevyRuntimeShell,
    initial: &crystal_runtime::RuntimeOverworldSession,
    commands_before: usize,
) -> usize {
    let mut replay = initial.clone();
    let mut count = 0;
    for command in &shell.shell.retained_runtime_commands()[commands_before..] {
        if matches!(
            crystal_assets::decode_runtime_mutation_command_frame(command, replay.state()).unwrap(),
            crystal_assets::RuntimeMutationCommand::UseSurfFieldMove(_)
        ) {
            count += 1;
        }
        replay
            .apply_runtime_command_frame(shell.shell.runtime(), command)
            .unwrap();
    }
    assert_eq!(
        replay.state(),
        shell.shell.session().state(),
        "modal input journal replays exactly"
    );
    count
}

#[test]
fn surf_anchor_keyboard_prompt_owns_only_fresh_input_and_no_cancels_without_field_use() {
    for (simultaneous_b, simultaneous_down) in [(false, false), (true, false), (false, true)] {
        let mut app = surf_anchor_app_at_shore();
        let (initial, commands_before) = {
            let shell = app.world().resource::<BevyRuntimeShell>();
            (
                shell.shell.session().clone(),
                shell.shell.retained_runtime_commands().len(),
            )
        };
        for _ in 0..3 {
            app.update();
        }
        assert!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .pending_contextual_field_move
                .is_none()
        );
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::KeyZ);
            if simultaneous_b {
                keys.press(KeyCode::KeyX);
            }
            if simultaneous_down {
                keys.press(KeyCode::ArrowDown);
            }
        }
        app.update();
        {
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            assert_eq!(
                shell.pending_contextual_field_move,
                Some(PartyFieldMove::Surf)
            );
            assert_eq!(shell.shell.session().snapshot().mode, MovementMode::Normal);
            assert_eq!(
                shell.shell.session().snapshot().tile,
                TilePosition::new(38, 3)
            );
            assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index, 0);
            assert_eq!(
                surf_anchor_field_use_count(shell, &initial, commands_before),
                0
            );
        }
        // Hold the original buttons through further source frames. Only new
        // edges can own the newly opened confirmation.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset(KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        for _ in 0..80 {
            app.update();
        }
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .pending_contextual_field_move,
            Some(PartyFieldMove::Surf)
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowDown);
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .yes_no_cursor
                .as_ref()
                .unwrap()
                .option_index,
            1
        );
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
        {
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            assert!(shell.pending_contextual_field_move.is_none());
            assert!(shell.pending_surf_start_from.is_none());
            assert_eq!(shell.shell.session().snapshot().mode, MovementMode::Normal);
            assert_eq!(
                shell.shell.session().snapshot().tile,
                TilePosition::new(38, 3)
            );
            assert_eq!(
                surf_anchor_field_use_count(shell, &initial, commands_before),
                0
            );
        }
        app.update();
        surf_anchor_mount_through_keyboard(&mut app);
        assert_eq!(
            surf_anchor_field_use_count(
                app.world().resource::<BevyRuntimeShell>(),
                &initial,
                commands_before
            ),
            1
        );
    }
}

#[test]
fn surf_anchor_controller_no_input_and_a_b_open_only_once_then_no_and_fresh_yes_work() {
    for (simultaneous_b, simultaneous_down) in [(false, false), (true, false), (false, true)] {
        let mut controller = VisibleShellController {
            shell: surf_anchor_preview_shell(),
        };
        controller.set_runtime_journal_enabled(true);
        let initial = controller.shell.shell.session().clone();
        let commands_before = controller.shell.shell.retained_runtime_commands().len();
        controller.wait_frames(3).unwrap();
        assert!(controller.shell.pending_contextual_field_move.is_none());
        if simultaneous_b || simultaneous_down {
            let second = if simultaneous_b {
                GameButton::B
            } else {
                GameButton::Down
            };
            let outcome =
                apply_visible_shell_smoke_frame(&mut controller.shell, &[GameButton::A, second])
                    .unwrap();
            controller.complete_overworld_outcome(outcome).unwrap();
            settle_visible_shell_until_input(&mut controller.shell).unwrap();
        } else {
            controller.press(GameButton::A).unwrap();
        }
        assert_eq!(
            controller.shell.pending_contextual_field_move,
            Some(PartyFieldMove::Surf)
        );
        assert_eq!(
            controller.shell.shell.session().snapshot().mode,
            MovementMode::Normal
        );
        assert_eq!(
            surf_anchor_field_use_count(&controller.shell, &initial, commands_before),
            0
        );
        assert_eq!(
            controller
                .shell
                .yes_no_cursor
                .as_ref()
                .unwrap()
                .option_index,
            0
        );
        controller.wait_frames(3).unwrap();
        assert_eq!(
            controller.shell.pending_contextual_field_move,
            Some(PartyFieldMove::Surf)
        );
        controller.press(GameButton::Down).unwrap();
        assert_eq!(
            controller
                .shell
                .yes_no_cursor
                .as_ref()
                .unwrap()
                .option_index,
            1
        );
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.pending_contextual_field_move.is_none());
        assert!(controller.shell.pending_surf_start_from.is_none());
        assert_eq!(
            controller.shell.shell.session().snapshot().tile,
            TilePosition::new(38, 3)
        );
        assert_eq!(
            controller.shell.shell.session().snapshot().mode,
            MovementMode::Normal
        );
        assert_eq!(
            surf_anchor_field_use_count(&controller.shell, &initial, commands_before),
            0
        );
        controller.press(GameButton::A).unwrap();
        assert_eq!(
            controller.shell.pending_contextual_field_move,
            Some(PartyFieldMove::Surf)
        );
        assert_eq!(
            controller
                .shell
                .yes_no_cursor
                .as_ref()
                .unwrap()
                .option_index,
            0
        );
        for _ in 0..12 {
            controller.press(GameButton::A).unwrap();
            if controller.shell.shell.session().snapshot().mode == MovementMode::Surf {
                break;
            }
        }
        assert_eq!(
            controller.shell.shell.session().snapshot().mode,
            MovementMode::Surf
        );
        assert_eq!(
            controller.shell.shell.session().snapshot().tile,
            TilePosition::new(38, 4)
        );
        assert_eq!(
            surf_anchor_field_use_count(&controller.shell, &initial, commands_before),
            1
        );
    }
}

fn surf_anchor_assert_live_player_art(
    app: &mut App,
    expected_id: &str,
    expected_mode: MovementMode,
) -> (Entity, Handle<Image>) {
    let (entity, source_id, standing, walking) = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(Entity, &PlayerSpriteFrames), With<PlayerMarker>>();
        let (entity, frames) = query.get_single(world).unwrap();
        (
            entity,
            frames.source_id.clone(),
            frames.standing.clone(),
            frames.walking.clone(),
        )
    };
    let shell = app.world().resource::<BevyRuntimeShell>();
    let rendered = app.world().resource::<RenderedViewport>();
    let frame = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>();
    let source = shell.shell.session().snapshot();
    assert_eq!(source.mode, expected_mode);
    assert_eq!(rendered.player_sprite_mode, Some(expected_mode));
    assert_eq!(source_id.as_ref(), expected_id);
    let player = frame
        .actors
        .iter()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .unwrap();
    assert_eq!(player.source_id.as_ref(), expected_id);
    assert!(
        player.texture == standing || walking.as_ref() == Some(&player.texture),
        "published actor uses the corresponding retained source sprite art"
    );
    let scene = capture_visible_fishing_source_frame(
        &source,
        frame.source_map_size_core_tiles,
        rendered,
        Some(frame),
    )
    .expect("settled actual avatar must have matching source footing");
    assert_eq!(
        scene.source_foot,
        player.center - Vec2::Y * player.size.y * 0.5
    );
    assert_eq!(
        scene.source_foot,
        fishing_source_support(&scene.terrain, source.tile).unwrap()
    );
    (entity, standing)
}

#[test]
fn surf_anchor_retained_player_art_tracks_normal_surf_normal_with_actual_footing() {
    let mut app = surf_anchor_app_at_shore();
    let (normal_entity, normal_texture) =
        surf_anchor_assert_live_player_art(&mut app, "chris", MovementMode::Normal);
    surf_anchor_mount_through_keyboard(&mut app);
    for _ in 0..2 {
        app.update();
    }
    let (surf_entity, surf_texture) =
        surf_anchor_assert_live_player_art(&mut app, "surf", MovementMode::Surf);
    assert_eq!(
        normal_entity, surf_entity,
        "mount updates the retained actual player"
    );
    assert_ne!(normal_texture, surf_texture);
    for _ in 0..32 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowUp);
        if app
            .world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .snapshot()
            .tile
            == TilePosition::new(38, 3)
        {
            break;
        }
    }
    for _ in 0..12 {
        app.update();
    }
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .snapshot()
            .tile,
        TilePosition::new(38, 3)
    );
    let (returned_entity, returned_texture) =
        surf_anchor_assert_live_player_art(&mut app, "chris", MovementMode::Normal);
    assert_eq!(
        surf_entity, returned_entity,
        "shore exit updates the same retained player"
    );
    assert_ne!(returned_texture, surf_texture);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .battle_origin
            .published()
            .is_none()
    );
}
