#[test]
fn walking_anchor_placement_is_deterministic_and_collision_bounded() {
    use crate::core::world::collision::permissions as p;
    assert!(walking_plain_support(p::FLOOR));
    assert!(walking_plain_support(p::TALL_GRASS));
    for blocked in [
        p::WATER,
        p::WALL,
        p::CUT_GRASS_28,
        p::WALK_RIGHT,
        p::HOP_DOWN,
        p::RIGHT_WALL,
        p::DOOR,
        p::ICE,
    ] {
        assert!(
            !walking_plain_support(blocked),
            "special collision {blocked:#x}"
        );
    }
    let source = IVec2::new(4, 4);
    let forward = [source + IVec2::X, source + IVec2::X * 2];
    let right = [source + IVec2::Y, source + IVec2::Y * 2];
    let both: Vec<_> = right.into_iter().chain(forward).collect();
    assert_eq!(
        walking_presentation_tile(source, IVec2::X, &both),
        Some(forward[1])
    );
    assert_eq!(
        walking_presentation_tile(source, IVec2::X, &right),
        Some(right[1])
    );
    assert_eq!(
        walking_presentation_tile(source, IVec2::X, &[forward[1]]),
        None,
        "an absent corridor cannot be jumped"
    );
    assert_eq!(walking_presentation_tile(source, IVec2::X, &[]), None);
}

#[test]
fn walking_anchor_interpolation_witness_never_replaces_landed_pose() {
    for direction in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        let from = Vec2::new(20.0, 30.0);
        let landed = from + direction * 16.0;
        for progress in [0.0, 0.125, 0.5, 0.875, 1.0] {
            assert!(walking_foot_is_on_original_step(
                from + (landed - from) * progress,
                from,
                landed
            ));
        }
        let side = Vec2::new(-direction.y, direction.x);
        assert!(!walking_foot_is_on_original_step(
            (from + landed) * 0.5 + side,
            from,
            landed
        ));
        assert!(!walking_foot_is_on_original_step(
            landed + direction,
            from,
            landed
        ));
        assert!(!walking_foot_is_on_original_step(
            Vec2::splat(f32::NAN),
            from,
            landed
        ));
    }
    assert!(!walking_snapshot_is_current(None, 20));
    assert!(!walking_snapshot_is_current(Some(19), 20));
    assert!(walking_snapshot_is_current(Some(20), 20));
    assert!(walking_snapshot_is_current(Some(0), u64::MAX));
}

#[test]
fn walking_anchor_eligibility_preserves_contact_and_rejects_unsupported_steps() {
    let mut origin = battle_origin_test_value();
    origin.source.mode = MovementMode::Normal;
    origin.source.facing = Direction::Right;
    let to = origin.source.tile;
    let from = TilePosition::new(to.x - 1, to.y);
    origin.contact = Some(BattleOriginContact::OverworldEncounter {
        map_id: origin.source.map_name.clone(),
        tile: to,
        surface: crate::core::world::encounters::EncounterSurface::Grass,
    });
    origin.authoritative_step = Some(StepOutcome::Moved {
        from,
        to,
        speed_multiplier: 1,
    });
    let original = origin.clone();
    assert_eq!(walking_origin_step(&origin), Some((from, to)));
    assert_eq!(
        origin, original,
        "eligibility cannot rewrite the checked result"
    );
    for mode in [
        MovementMode::Bike,
        MovementMode::Skate,
        MovementMode::Surf,
        MovementMode::SurfPika,
    ] {
        let mut other = origin.clone();
        other.source.mode = mode;
        assert_eq!(walking_origin_step(&other), None);
    }
    for step in [
        None,
        Some(StepOutcome::Turned {
            facing: Direction::Right,
        }),
        Some(StepOutcome::Blocked {
            at: to,
            facing: Direction::Right,
        }),
        Some(StepOutcome::Moved {
            from,
            to,
            speed_multiplier: 2,
        }),
    ] {
        let mut other = origin.clone();
        other.authoritative_step = step;
        assert_eq!(walking_origin_step(&other), None);
    }
    for surface in [
        crate::core::world::encounters::EncounterSurface::Water,
        crate::core::world::encounters::EncounterSurface::Rock,
    ] {
        let mut other = origin.clone();
        other.contact = Some(BattleOriginContact::OverworldEncounter {
            map_id: other.source.map_name.clone(),
            tile: to,
            surface,
        });
        assert_eq!(walking_origin_step(&other), None);
    }
    for mismatch in 0..5 {
        let mut other = origin.clone();
        match mismatch {
            0 => other.source.tile.x += 1,
            1 => other.source.map_name = "UnrelatedMap".into(),
            2 => other.source.facing = Direction::Left,
            3 => other.contact = None,
            4 => other.visual_step.ledge_jump = Some((from, to, 4)),
            _ => unreachable!(),
        }
        assert_eq!(walking_origin_step(&other), None);
    }
}

// A typed projection fixture over an actual rendered source frame. It is not
// a claim that this shore produced a grass encounter; native/production tests
// must separately exercise ordinary movement and the source encounter roll.
fn walking_anchor_projection_fixture() -> (
    BevyRuntimeShell,
    RenderedViewport,
    crystal_render_api::VisualWorldFrame,
    VisibleBoundWalkingEncounter,
) {
    let (mut app, _) = fishing_anchor_app();
    let mut frame = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    let mut rendered = app
        .world_mut()
        .remove_resource::<RenderedViewport>()
        .unwrap();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let mut source = shell.shell.session().snapshot();
    let scene = capture_visible_fishing_source_frame(
        &source,
        frame.source_map_size_core_tiles,
        &rendered,
        Some(&frame),
    )
    .unwrap();
    let facing = [
        source.facing,
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Up,
    ]
    .into_iter()
    .find(|direction| {
        let (dx, dy) = direction.delta();
        let before = TilePosition::new(source.tile.x - dx * 2, source.tile.y - dy * 2);
        let after = TilePosition::new(source.tile.x + dx * 2, source.tile.y + dy * 2);
        fishing_source_support(&scene.terrain, before).is_some()
            && fishing_source_support(&scene.terrain, after).is_some()
    })
    .expect("projection fixture has complete original-map cells on both sides");
    source.facing = facing;
    {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        overworld.set_player_facing(facing);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    rendered.player_sprite_facing = Some(facing);
    let (dx, dy) = source.facing.delta();
    let from = TilePosition::new(source.tile.x - dx, source.tile.y - dy);
    let to = source.tile;
    let mut origin = battle_origin_test_value();
    origin.source = source;
    origin.contact = Some(BattleOriginContact::OverworldEncounter {
        map_id: origin.source.map_name.clone(),
        tile: to,
        surface: crate::core::world::encounters::EncounterSurface::Grass,
    });
    origin.authoritative_step = Some(StepOutcome::Moved {
        from,
        to,
        speed_multiplier: 1,
    });
    origin.visual_step = BattleOriginVisualStep {
        walk_from: Some(from),
        walk_ticks_remaining: 4,
        walk_total_ticks: 8,
        ledge_jump: None,
    };
    origin.generation = 42;
    let step_delta = visual_facing(origin.source.facing) * frame.tile_size * 2.0;
    let player = frame
        .actors
        .iter_mut()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .unwrap();
    player.facing = Some(visual_facing(origin.source.facing));
    player.center -= step_delta * 0.5;
    let core = IVec2::new(i32::from(to.x), i32::from(to.y));
    let facing = IVec2::new(i32::from(dx), i32::from(dy));
    let bound = VisibleBoundWalkingEncounter {
        origin: Arc::new(origin),
        from,
        map_size: frame.source_map_size_core_tiles.unwrap(),
        placement: crystal_render_api::VisualBattleDerivedGrassPlacement {
            step_from_core_tile: core - facing,
            witnessed_player_foot: None,
            presentation_core_tile: core + facing * 2,
            walkable_core_tiles: [core, core + facing, core + facing * 2].into(),
        },
        minimum_snapshot_revision: rendered.snapshot_revision.unwrap(),
        warm_checked: false,
        capture_attempted: false,
        anchors: None,
        publication: None,
    };
    (shell, rendered, frame, bound)
}

#[test]
fn walking_anchor_freezes_original_grid_and_derives_both_supports_once() {
    let (mut shell, rendered, frame, mut bound) = walking_anchor_projection_fixture();
    let session = shell.shell.session().clone();
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    let origin = bound.origin.clone();
    let walk = (
        shell.player_walk_from,
        shell.player_walk_frame_ticks,
        shell.player_walk_total_ticks,
    );
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
    let anchors = bound
        .anchors
        .clone()
        .expect("interpolated source frame contains actual destination support");
    let witness = bound.placement.witnessed_player_foot.unwrap();
    let step_delta = visual_facing(origin.source.facing) * frame.tile_size * 2.0;
    assert_eq!(anchors.target_actor, None);
    assert_eq!(anchors.source_foot - witness, step_delta * 0.5);
    assert_eq!(anchors.target_foot - anchors.source_foot, step_delta * 2.0);
    assert!(anchors.terrain.matches_built_frame(&frame));
    let mut later = frame.clone();
    later.center += Vec2::splat(100.0);
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&later), true);
    assert!(Arc::ptr_eq(bound.anchors.as_ref().unwrap(), &anchors));
    shell.battle_origin.active = Some(origin.clone());
    shell.battle_origin.bound_walking = Some(bound);
    let first = publish_visible_walking_location(&mut shell, &rendered, Some(&frame)).unwrap();
    let next = publish_visible_walking_location(&mut shell, &rendered, None).unwrap();
    assert!(Arc::ptr_eq(&first, &next));
    assert_eq!(
        first.target.core_tile(),
        first.source.core_tile,
        "contact remains the authoritative encounter tile"
    );
    assert_eq!(shell.shell.session(), &session, "includes RNG and clock");
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
    assert_eq!(
        (
            shell.player_walk_from,
            shell.player_walk_frame_ticks,
            shell.player_walk_total_ticks
        ),
        walk
    );
    assert_eq!(
        shell.battle_origin.published().unwrap().as_ref(),
        origin.as_ref()
    );
    shell.battle_origin.clear();
    assert!(publish_visible_walking_location(&mut shell, &rendered, Some(&frame)).is_none());
    assert!(
        first.anchors.is_some(),
        "retained readers keep their immutable evidence"
    );
}

#[test]
fn walking_anchor_rejects_cold_stale_and_off_step_frames_without_late_repair() {
    let (_shell, rendered, good_frame, prototype) = walking_anchor_projection_fixture();
    for defect in 0..8 {
        let mut frame = good_frame.clone();
        let mut bound = prototype.clone();
        match defect {
            0 => frame.active = false,
            1 => frame.map_id = Arc::from("OtherMap"),
            2 => frame.terrain_revision += 1,
            3 => frame.grid_origin.x += 1,
            4 => frame.source_map_size_core_tiles = None,
            5 => {
                frame
                    .actors
                    .iter_mut()
                    .find(|a| a.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .center += Vec2::new(1.0, 1.0)
            }
            6 => frame
                .actors
                .retain(|a| a.id != crystal_render_api::VisualActorId::Player),
            7 => {}
            _ => unreachable!(),
        }
        freeze_visible_walking_anchors(
            &mut bound,
            &rendered,
            (defect != 7).then_some(&frame),
            true,
        );
        assert!(bound.capture_attempted, "defect {defect}");
        assert!(bound.anchors.is_none(), "defect {defect}");
        freeze_visible_walking_anchors(&mut bound, &rendered, Some(&good_frame), true);
        assert!(
            bound.anchors.is_none(),
            "late replacement cannot repair defect {defect}"
        );
    }
}

#[test]
fn walking_anchor_waits_only_for_the_original_step_publication() {
    let (_, mut rendered, frame, mut bound) = walking_anchor_projection_fixture();
    let destination = rendered.tile;
    let revision = rendered.snapshot_revision.unwrap();
    let step_delta = visual_facing(bound.origin.source.facing) * frame.tile_size * 2.0;
    let mut previous_frame = frame.clone();
    previous_frame
        .actors
        .iter_mut()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .unwrap()
        .center -= step_delta;
    rendered.tile = Some(bound.from);
    rendered.snapshot_revision = Some(revision.wrapping_sub(1));
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&previous_frame), true);
    assert!(
        !bound.capture_attempted,
        "a coherent previous walk still approaching from may await the committed step"
    );
    rendered.tile = destination;
    rendered.snapshot_revision = Some(revision);
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
    assert!(bound.anchors.is_some());
    let (_, mut rendered, frame, mut expired) = walking_anchor_projection_fixture();
    rendered.snapshot_revision = Some(expired.minimum_snapshot_revision.wrapping_sub(1));
    freeze_visible_walking_anchors(&mut expired, &rendered, Some(&frame), true);
    assert!(!expired.capture_attempted);
    freeze_visible_walking_anchors(&mut expired, &rendered, Some(&frame), false);
    assert!(expired.capture_attempted);
    rendered.snapshot_revision = Some(expired.minimum_snapshot_revision);
    freeze_visible_walking_anchors(&mut expired, &rendered, Some(&frame), true);
    assert!(
        expired.anchors.is_none(),
        "missed scene window must remain fallback"
    );
}

#[test]
fn walking_anchor_rejects_invalid_pre_step_witnesses_without_late_repair() {
    let (_, mut rendered, good_frame, prototype) = walking_anchor_projection_fixture();
    let destination = rendered.tile;
    let revision = rendered.snapshot_revision;
    let facing = visual_facing(prototype.origin.source.facing);
    let step_delta = facing * good_frame.tile_size * 2.0;
    let mut previous_frame = good_frame.clone();
    previous_frame
        .actors
        .iter_mut()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .unwrap()
        .center -= step_delta;
    for defect in 0..6 {
        let mut frame = previous_frame.clone();
        let mut bound = prototype.clone();
        match defect {
            0 => frame
                .actors
                .retain(|actor| actor.id != crystal_render_api::VisualActorId::Player),
            1 => frame.terrain_revision += 1,
            2 => frame.grid_origin.x += 1,
            3 => frame.map_texture = Handle::<Image>::weak_from_u128(u128::MAX),
            4 => {
                frame
                    .actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .center += Vec2::new(-facing.y, facing.x)
            }
            5 => {
                frame
                    .actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .center += step_delta
            }
            _ => unreachable!(),
        }
        rendered.tile = Some(bound.from);
        rendered.snapshot_revision = revision.map(|revision| revision.wrapping_sub(1));
        freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
        assert!(bound.capture_attempted, "invalid pre-step witness {defect}");
        assert!(bound.anchors.is_none(), "invalid pre-step witness {defect}");
        rendered.tile = destination;
        rendered.snapshot_revision = revision;
        freeze_visible_walking_anchors(&mut bound, &rendered, Some(&good_frame), true);
        assert!(
            bound.anchors.is_none(),
            "later destination evidence cannot repair pre-step witness {defect}"
        );
    }
}

#[test]
fn walking_anchor_settled_pre_step_frame_derives_landing_inside_original_grid() {
    let (_, mut rendered, mut frame, mut bound) = walking_anchor_projection_fixture();
    rendered.tile = Some(bound.from);
    rendered.snapshot_revision = Some(bound.minimum_snapshot_revision.wrapping_sub(1));
    let step_delta = visual_facing(bound.origin.source.facing) * frame.tile_size * 2.0;
    frame
        .actors
        .iter_mut()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .unwrap()
        .center -= step_delta * 0.5;
    let origin = bound.origin.clone();
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
    let anchors = bound
        .anchors
        .as_ref()
        .expect("exact previous settled feet and original destination acreage");
    assert!(anchors.terrain.matches_built_frame(&frame));
    assert_eq!(
        anchors.source_foot - bound.placement.witnessed_player_foot.unwrap(),
        step_delta
    );
    assert!(Arc::ptr_eq(&bound.origin, &origin));
    assert_eq!(bound.origin.source.tile, origin.source.tile);
    assert_ne!(bound.origin.source.tile, bound.from);
}

fn walking_anchor_route29_app() -> (App, TilePosition, TilePosition, Direction) {
    walking_anchor_route29_app_with_run(1)
}

fn walking_anchor_route29_app_with_run(
    run_steps: i16,
) -> (App, TilePosition, TilePosition, Direction) {
    use crate::core::world::collision::{is_grass_encounter_permission, sample_collision};
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    let runtime = shell.shell.runtime().clone();
    let data = runtime.data();
    let map = data.overworld_map("Route29").unwrap();
    let tileset = data
        .tileset_collision(data.map_tileset_name("Route29").unwrap())
        .unwrap();
    let (width, height) = map.checked_tile_bounds().unwrap();
    let is_open = |tile: TilePosition| {
        sample_collision(&map, &tileset, tile)
            .is_some_and(|sample| walking_plain_support(sample.permission))
            && !data.maps["Route29"].objects.iter().any(|object| {
                crate::core::world::session::object_tile_position_checked(object).is_some_and(
                    |other| {
                        (i32::from(other.x) - i32::from(tile.x)).abs() <= 1
                            && (i32::from(other.y) - i32::from(tile.y)).abs() <= 1
                    },
                )
            })
    };
    let mut selected = None;
    'tiles: for y in 2..height.saturating_sub(2) {
        for x in 2..width.saturating_sub(2) {
            let target = TilePosition::new(x as i16, y as i16);
            if !is_open(target)
                || !sample_collision(&map, &tileset, target)
                    .is_some_and(|sample| is_grass_encounter_permission(sample.permission))
            {
                continue;
            }
            for facing in [
                Direction::Right,
                Direction::Down,
                Direction::Left,
                Direction::Up,
            ] {
                let (dx, dy) = facing.delta();
                let from = TilePosition::new(target.x - dx, target.y - dy);
                if !is_open(from) {
                    continue;
                }
                let encounter = TilePosition::new(
                    target.x + dx * (run_steps - 1),
                    target.y + dy * (run_steps - 1),
                );
                if !(0..run_steps).all(|step| {
                    is_open(TilePosition::new(
                        target.x + dx * step,
                        target.y + dy * step,
                    ))
                }) || !sample_collision(&map, &tileset, encounter)
                    .is_some_and(|sample| is_grass_encounter_permission(sample.permission))
                {
                    continue;
                }
                let core = IVec2::new(i32::from(encounter.x), i32::from(encounter.y));
                let open: Vec<_> = (-3..=3)
                    .flat_map(|oy| (-3..=3).map(move |ox| (ox, oy)))
                    .filter_map(|(ox, oy)| {
                        let p = core + IVec2::new(ox, oy);
                        let tile = TilePosition::new(p.x as i16, p.y as i16);
                        is_open(tile).then_some(p)
                    })
                    .collect();
                if walking_presentation_tile(core, IVec2::new(i32::from(dx), i32::from(dy)), &open)
                    .is_some()
                {
                    selected = Some((from, target, facing));
                    break 'tiles;
                }
            }
        }
    }
    let (from, target, facing) =
        selected.expect("compiled Route29 has an unoccupied grass step and presentation corridor");
    {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        data.transition_overworld_session(
            state,
            overworld,
            "Route29",
            from,
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
            &runtime.music_ids(),
        )
        .unwrap();
        overworld.set_player_facing(facing);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
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
    assert!(
        app.world()
            .resource::<crystal_render_api::VisualWorldFrame>()
            .active
    );
    (app, from, target, facing)
}

#[test]
fn walking_anchor_real_route29_keyboard_encounter_retains_landing_and_replays_journal() {
    use bevy::ecs::system::RunSystemOnce;
    let (mut app, from, to, facing) = walking_anchor_route29_app();
    let (mut replay, commands_before) = {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        // A finite fixture trace feeds the ordinary source Random calls. The
        // map's encounter cooldown, rate, slot choice, species and DVs still
        // execute normally, with no direct battle start or outcome override.
        shell.shell.session_mut().state_mut().random_state =
            crate::core::random::CrystalRandomState::default();
        *shell.shell.session_mut().divider_mut_for_tests() =
            crate::core::random::RuntimeDividerSource::replay(vec![0; 8192]);
        (
            shell.shell.session().clone(),
            shell.shell.retained_runtime_commands().len(),
        )
    };
    let key = |direction| match direction {
        Direction::Up => KeyCode::ArrowUp,
        Direction::Down => KeyCode::ArrowDown,
        Direction::Left => KeyCode::ArrowLeft,
        Direction::Right => KeyCode::ArrowRight,
    };
    let reverse = match facing {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    };
    // Walk through the source's initial five-step encounter cooldown using
    // ordinary input. Turning and all eight-frame landings remain intact.
    'steps: for step in 0..16 {
        let direction = if step % 2 == 0 { facing } else { reverse };
        let destination = if step % 2 == 0 { to } else { from };
        for _ in 0..24 {
            press_key_for_runtime_hotkey_app(&mut app, key(direction));
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            if shell.battle_origin.published().is_some() {
                break 'steps;
            }
            assert!(
                app.world()
                    .resource::<crystal_render_api::BattleLocationFrame>()
                    .location
                    .is_none(),
                "turns and non-encounter steps must not publish a location"
            );
            if shell.shell.session().snapshot().tile == destination {
                // Let the existing walk finish before reversing the next step.
                for _ in 0..10 {
                    app.update();
                }
                break;
            }
        }
    }
    let origin = app
        .world()
        .resource::<BevyRuntimeShell>()
        .battle_origin
        .published()
        .expect("ordinary Route29 grass step produced a source encounter")
        .clone();
    assert_eq!(origin.kind, BattleOriginKind::Wild);
    assert_eq!(origin.source.mode, MovementMode::Normal);
    assert_eq!(origin.battle_type, "BATTLETYPE_NORMAL");
    let Some(StepOutcome::Moved {
        from: actual_from,
        to: actual_to,
        speed_multiplier: 1,
    }) = origin.authoritative_step.clone()
    else {
        panic!("ordinary one-tile source step required");
    };
    assert_eq!(origin.source.tile, actual_to);
    assert_eq!(
        origin.contact,
        Some(BattleOriginContact::OverworldEncounter {
            map_id: "Route29".into(),
            tile: actual_to,
            surface: crate::core::world::encounters::EncounterSurface::Grass,
        })
    );
    assert_eq!(origin.visual_step.walk_from, Some(actual_from));
    assert!(origin.visual_step.walk_ticks_remaining > 0);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let source_frame = shell.shell.last_frame().unwrap();
    assert!(source_frame.wild_battle.is_some());
    assert_eq!(source_frame.movement, origin.authoritative_step);
    let runtime = shell.shell.runtime().clone();
    let mut inputs = 0;
    for command in &shell.shell.retained_runtime_commands()[commands_before..] {
        if matches!(
            crystal_assets::decode_runtime_mutation_command_frame(command, replay.state()).unwrap(),
            crystal_assets::RuntimeMutationCommand::ApplyOverworldInput(_)
        ) {
            inputs += 1;
        }
        replay
            .apply_runtime_command_frame(&runtime, command)
            .unwrap();
    }
    assert!(
        inputs >= 5,
        "source cooldown was consumed through actual input"
    );
    assert_eq!(
        replay.state(),
        shell.shell.session().state(),
        "journal reproduces source clock, RNG, movement and encounter"
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
        .expect("the original grass walk published an actual matching source frame");
    assert_eq!(location.source.core_tile, location.target.core_tile());
    assert_eq!(anchors.target_actor, None);
    let crystal_render_api::VisualBattleTarget::WalkingGrass { presentation, .. } =
        &location.target
    else {
        panic!("explicit walking presentation required");
    };
    assert_ne!(
        presentation.presentation_core_tile,
        location.target.core_tile()
    );
    assert!(presentation.witnessed_player_foot.is_some());
    let before = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .clone();
    let command_count = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .retained_runtime_commands()
        .len();
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    assert_eq!(
        app.world().resource::<BevyRuntimeShell>().shell.session(),
        &before
    );
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .retained_runtime_commands()
            .len(),
        command_count
    );
    for _ in 0..16 {
        app.update();
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.player_walk_frame_ticks, 0);
    assert_eq!(shell.shell.session().snapshot().tile, actual_to);
    assert!(Arc::ptr_eq(
        shell.battle_origin.published().unwrap(),
        &origin
    ));
    assert!(Arc::ptr_eq(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .as_ref()
            .unwrap(),
        &location
    ));
    assert_eq!(shell.last_error, None);
}

#[test]
fn walking_anchor_continuous_steps_capture_the_retained_field_before_battle_replaces_it() {
    let (mut app, from, to, facing) = walking_anchor_route29_app_with_run(3);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.shell.session_mut().state_mut().random_state =
            crate::core::random::CrystalRandomState::default();
        *shell.shell.session_mut().divider_mut_for_tests() =
            crate::core::random::RuntimeDividerSource::replay(vec![0; 8192]);
        assert_eq!(shell.shell.session().state().wild_encounter_cooldown, 5);
    }
    let key = |direction| match direction {
        Direction::Up => KeyCode::ArrowUp,
        Direction::Down => KeyCode::ArrowDown,
        Direction::Left => KeyCode::ArrowLeft,
        Direction::Right => KeyCode::ArrowRight,
    };
    // Consume two source cooldown steps through ordinary input, then leave
    // three adjacent checked tiles for the uninterrupted approach below.
    for (direction, destination) in [(facing, to), (visible_opposite_direction(facing), from)] {
        for _ in 0..24 {
            press_key_for_runtime_hotkey_app(&mut app, key(direction));
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            assert!(shell.battle_origin.published().is_none());
            if shell.shell.session().snapshot().tile == destination {
                break;
            }
        }
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .shell
                .session()
                .snapshot()
                .tile,
            destination
        );
        for _ in 0..10 {
            app.update();
        }
    }
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .state()
            .wild_encounter_cooldown,
        3
    );
    let mut committed_from_motion = false;
    for _ in 0..80 {
        let preceding_walk = {
            let shell = app.world().resource::<BevyRuntimeShell>();
            let rendered = app.world().resource::<RenderedViewport>();
            let frame = app
                .world()
                .resource::<crystal_render_api::VisualWorldFrame>();
            shell.player_walk_from.and_then(|walk_from| {
                let source = shell.shell.session().snapshot();
                capture_visible_walking_source_frame(
                    &source,
                    walk_from,
                    frame.source_map_size_core_tiles?,
                    rendered,
                    Some(frame),
                )
                .map(|scene| (source.tile, scene))
            })
        };
        press_key_for_runtime_hotkey_app(&mut app, key(facing));
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        if let Some(origin) = shell.battle_origin.published() {
            let (prior_tile, prior_scene) = preceding_walk
                .expect("the source encounter followed a coherent unfinished rendered walk");
            let (step_from, _) = walking_origin_step(origin).unwrap();
            assert_eq!(prior_tile, step_from);
            let prior_landing = fishing_source_support(&prior_scene.terrain, prior_tile).unwrap();
            assert!(
                (prior_scene.source_foot - prior_landing).length_squared() > 0.0001,
                "the preceding player had not settled before commitment"
            );
            let bound = shell.battle_origin.bound_walking.as_ref().unwrap();
            assert!(bound.warm_checked);
            assert!(
                !bound.capture_attempted,
                "coherent preceding motion must wait"
            );
            assert!(matches!(
                shell.pending_overworld_step_boundary,
                Some(PendingOverworldStepBoundary::WildBattle)
            ));
            assert!(
                shell.battle_lcd_animation_active,
                "core battle state is already committed"
            );
            assert!(
                app.world()
                    .resource::<crystal_render_api::VisualWorldFrame>()
                    .active,
                "the real render/extract pipeline must keep the pending walk's field active"
            );
            committed_from_motion = true;
            break;
        }
    }
    assert!(
        committed_from_motion,
        "ordinary fifth step must produce the grass encounter"
    );
    app.update();
    let location = app
        .world()
        .resource::<crystal_render_api::BattleLocationFrame>()
        .location
        .clone()
        .expect("next publication captures the committed walk");
    assert!(location.anchors.is_some());
    let mut battle_replaced_field = false;
    for _ in 0..160 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        let retained_field = matches!(
            shell.pending_overworld_step_boundary,
            Some(PendingOverworldStepBoundary::WildBattle)
        ) || shell
            .visible_battle_transition
            .is_some_and(|transition| transition.frame < 3);
        assert_eq!(
            app.world()
                .resource::<crystal_render_api::VisualWorldFrame>()
                .active,
            retained_field,
            "world extraction must end at the existing map-replacement boundary"
        );
        assert!(Arc::ptr_eq(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .as_ref()
                .unwrap(),
            &location
        ));
        if shell.visible_battle_transition.is_none() {
            assert!(shell.battle_lcd_animation_active);
            battle_replaced_field = true;
            break;
        }
    }
    assert!(battle_replaced_field);
    assert!(
        app.world_mut()
            .query_filtered::<(), Or<(
                With<BattleBattlerMarker>,
                With<BattleHudMarker>,
                With<BattleCommandMarker>,
                With<FixedBattleCanvasMarker>,
                With<BattleWindowFrameMarker>,
            )>>()
            .iter(app.world())
            .next()
            .is_some()
    );
}
