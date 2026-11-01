fn battle_origin_test_value() -> BattlePresentationOrigin {
    BattlePresentationOrigin {
        generation: 0,
        source: crate::core::world::session::OverworldSnapshot {
            frame: 42,
            map_name: "Route32".to_string(),
            tile: TilePosition::new(10, 14),
            facing: Direction::Right,
            mode: MovementMode::Normal,
        },
        kind: BattleOriginKind::Wild,
        battle_type: "BATTLETYPE_NORMAL".to_string(),
        contact: Some(BattleOriginContact::FishingWaterTarget {
            map_id: "Route32".to_string(),
            tile: TilePosition::new(11, 14),
        }),
        authoritative_step: None,
        visual_step: BattleOriginVisualStep {
            walk_from: None,
            walk_ticks_remaining: 0,
            walk_total_ticks: 8,
            ledge_jump: None,
        },
    }
}

#[test]
fn battle_origin_pending_value_is_frozen_and_generation_survives_reset() {
    let mut state = VisibleBattleOriginState::default();
    let first = battle_origin_test_value();
    state.stage(first.clone());
    let frozen = state.published().unwrap().clone();
    assert_eq!(frozen.generation, 1);
    assert_eq!(frozen.source, first.source);
    assert_eq!(frozen.contact, first.contact);
    let mut later_pose = first.clone();
    later_pose.source.map_name = "Elsewhere".to_string();
    later_pose.source.tile = TilePosition::new(99, 99);
    later_pose.source.facing = Direction::Down;
    later_pose.source.mode = MovementMode::Surf;
    state.stage(later_pose);
    assert!(Arc::ptr_eq(state.published().unwrap(), &frozen));
    state.clear();
    assert!(state.published().is_none());
    state.stage(first);
    let repeated = state.published().unwrap();
    assert_eq!(repeated.generation, frozen.generation + 1);
    assert_eq!(repeated.source, frozen.source);
    assert_eq!(
        frozen.generation, 1,
        "readers retain the original immutable value"
    );
}

#[test]
fn battle_origin_prepare_freezes_real_source_without_changing_gameplay_or_rng() {
    let mut controller = VisibleShellController {
        shell: route36_overworld_shell_for_battle_render_regression(),
    };
    let source = controller.shell.shell.session().snapshot();
    controller
        .shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .unwrap();
    let session_before = controller.shell.shell.session().clone();
    let checksum_before = controller.shell.shell.state_checksum().unwrap();
    let commands_before = controller.shell.shell.retained_runtime_commands().to_vec();
    let results_before = controller.shell.shell.retained_runtime_results().to_vec();
    prepare_visible_battle_entry(&mut controller.shell).unwrap();
    let origin = controller.battle_presentation_origin().unwrap().clone();
    assert_eq!(origin.source, source);
    assert_eq!(origin.kind, BattleOriginKind::StaticWild);
    assert_eq!(
        origin.contact, None,
        "the entry supplies no checked tree/water contact"
    );
    assert_eq!(
        controller.shell.shell.session(),
        &session_before,
        "includes divider and RNG"
    );
    assert_eq!(
        controller.shell.shell.state_checksum().unwrap(),
        checksum_before
    );
    assert_eq!(
        controller.shell.shell.retained_runtime_commands(),
        commands_before
    );
    assert_eq!(
        controller.shell.shell.retained_runtime_results(),
        results_before
    );
    controller.shell.player_walk_from = Some(TilePosition::new(80, 80));
    controller.shell.player_walk_frame_ticks = 3;
    prepare_visible_battle_entry_after_visible_step(&mut controller.shell).unwrap();
    assert_eq!(controller.battle_presentation_origin(), Some(&origin));
    assert_eq!(controller.shell.shell.session(), &session_before);
}

#[test]
fn battle_origin_pending_contact_and_in_flight_step_survive_entry_reset() {
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    let source = shell.shell.session().snapshot();
    shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .unwrap();
    // This typed fixture exercises the pending-origin seam separately from RNG
    // selection; it is not a playthrough of a water encounter on Route36.
    shell.player_walk_from = Some(TilePosition::new(source.tile.x - 1, source.tile.y));
    shell.player_walk_frame_ticks = 5;
    shell.player_walk_total_ticks = 8;
    let step = StepOutcome::Moved {
        from: shell.player_walk_from.unwrap(),
        to: source.tile,
        speed_multiplier: 1,
    };
    let contact = BattleOriginContact::OverworldEncounter {
        map_id: source.map_name.clone(),
        tile: source.tile,
        surface: crate::core::world::encounters::EncounterSurface::Water,
    };
    stage_visible_battle_origin(
        &mut shell,
        &source,
        BattleOriginKind::Wild,
        "BATTLETYPE_NORMAL",
        Some(contact.clone()),
        Some(step.clone()),
    );
    let pending = shell.battle_origin.pending.as_ref().unwrap().clone();
    shell.player_walk_frame_ticks = 0;
    shell.player_walk_from = None;
    prepare_visible_battle_entry_after_visible_step(&mut shell).unwrap();
    let active = shell.battle_origin.active.as_ref().unwrap();
    assert!(Arc::ptr_eq(active, &pending));
    assert_eq!(active.source, source);
    assert_eq!(active.contact, Some(contact));
    assert_eq!(active.authoritative_step, Some(step));
    assert_eq!(active.visual_step.walk_ticks_remaining, 5);
    assert_eq!(active.visual_step.walk_total_ticks, 8);
    assert_ne!(active.visual_step.walk_from, Some(active.source.tile));
    assert!(shell.battle_origin.pending.is_none());
}

#[test]
fn battle_origin_wild_frame_preserves_checked_contact_and_source_pose() {
    let mut shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let battle = snapshot.battle.as_ref().unwrap();
    let encounter = crate::core::world::session::WildEncounterRoll {
        map_name: "CheckedEncounterMap".to_string(),
        tile: TilePosition::new(11, 14),
        surface: crate::core::world::encounters::EncounterSurface::Water,
        time: crate::core::world::encounters::TimeOfDay::Day,
        threshold: 1,
        encounter_roll: 0,
        slot_percent_roll: Some(0),
        level_roll: None,
        roaming_slot: None,
        resolved: None,
        repelled_by: None,
    };
    // Explicitly different positions catch accidental substitution of the
    // player tile for encounter contact. This tests result projection only.
    let mut source = snapshot.overworld.clone();
    source.tile = TilePosition::new(10, 14);
    source.facing = Direction::Right;
    source.mode = MovementMode::Surf;
    let frame = crate::RuntimeOverworldFrame {
        snapshot: source.clone(),
        input_mask: 0,
        pressed_mask: 0,
        autonomous_objects_changed: false,
        movement: None,
        ledge_jump: None,
        grass_rustle: None,
        phone_call: None,
        step_events: None,
        coord_event: None,
        trainer_sight: None,
        interaction: None,
        warp: None,
        connection: None,
        wild_encounter: Some(encounter.clone()),
        wild_battle: Some(crate::core::battle::start::WildBattleStart {
            battle_type: battle.battle_type.clone(),
            battle_music: battle.battle_music.clone(),
            encounter: encounter.clone(),
            enemy_pokemon: battle.enemy_pokemon.clone(),
            enemy_party: battle.enemy_party.clone(),
        }),
        state_checksum: snapshot.state_checksum.clone(),
    };
    shell.battle_origin.clear();
    let session_before = shell.shell.session().clone();
    capture_visible_wild_battle_origin(&mut shell, &frame);
    let origin = shell.battle_origin.published().unwrap();
    assert_eq!(origin.source, source);
    assert_eq!(
        origin.contact,
        Some(BattleOriginContact::OverworldEncounter {
            map_id: encounter.map_name,
            tile: encounter.tile,
            surface: encounter.surface,
        })
    );
    assert_eq!(shell.shell.session(), &session_before);
}

#[test]
fn battle_origin_fishing_keeps_shore_pose_distinct_from_checked_water_contact() {
    let origin = battle_origin_test_value();
    let Some(BattleOriginContact::FishingWaterTarget { map_id, tile }) = &origin.contact else {
        panic!("expected checked water contact");
    };
    assert_eq!(map_id, &origin.source.map_name);
    assert_ne!(*tile, origin.source.tile);
    assert_eq!(origin.source.mode, MovementMode::Normal);
    assert_eq!(origin.source.facing, Direction::Right);
    let mut absent = origin.clone();
    absent.contact = None;
    absent.source.map_name = "WaterRouteOrCaveIsNotEvidence".to_string();
    let mut state = VisibleBattleOriginState::default();
    state.stage(absent);
    assert_eq!(state.published().unwrap().contact, None);
}

#[test]
fn battle_origin_retained_narration_clears_only_on_visible_exit() {
    let mut shell = route36_battle_shell_for_render_regression();
    let origin = shell.battle_origin.active.as_ref().unwrap().clone();
    assert!(!shell.battle_messages.is_empty());
    reset_visible_battle_exit_state(&mut shell);
    assert!(Arc::ptr_eq(
        shell.battle_origin.active.as_ref().unwrap(),
        &origin
    ));
    shell.battle_messages.clear();
    reset_visible_battle_exit_state(&mut shell);
    assert!(shell.battle_origin.published().is_none());
    assert_eq!(origin.source.map_name, "Route36");
}

#[test]
fn battle_origin_save_reload_clears_active_and_pending_provenance() {
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    let path = std::env::temp_dir().join(format!(
        "crystal-battle-origin-{}.crystalsave",
        uuid::Uuid::new_v4(),
    ));
    shell.shell.save(&path).unwrap();
    shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .unwrap();
    prepare_visible_battle_entry(&mut shell).unwrap();
    let previous_generation = shell.battle_origin.active.as_ref().unwrap().generation;
    shell.battle_origin.stage(battle_origin_test_value());
    load_visible_runtime_save(&mut shell, &path, "origin_regression").unwrap();
    assert!(shell.battle_origin.active.is_none());
    assert!(shell.battle_origin.pending.is_none());
    assert!(shell.shell.snapshot().unwrap().battle.is_none());
    assert!(shell.battle_origin.generation >= previous_generation);
    let _ = std::fs::remove_file(path);
}

#[test]
fn battle_origin_publication_is_read_only_and_clears_the_previous_frame() {
    let shell = route36_battle_shell_for_render_regression();
    let session_before = shell.shell.session().clone();
    let expected = shell.battle_origin.published().unwrap().clone();
    #[derive(Resource, Default)]
    struct OriginChanges(usize);
    fn count_changes(
        frame: Res<BattlePresentationOriginFrame>,
        mut changes: ResMut<OriginChanges>,
    ) {
        if frame.is_changed() {
            changes.0 += 1;
        }
    }
    let mut app = App::new();
    app.insert_resource(shell)
        .init_resource::<BattlePresentationOriginFrame>()
        .init_resource::<OriginChanges>()
        .add_systems(
            Update,
            (publish_visible_battle_origin, count_changes).chain(),
        );
    app.update();
    let frame = app.world().resource::<BattlePresentationOriginFrame>();
    assert_eq!(frame.origin(), Some(expected.as_ref()));
    assert!(Arc::ptr_eq(frame.origin.as_ref().unwrap(), &expected));
    assert_eq!(
        app.world().resource::<BevyRuntimeShell>().shell.session(),
        &session_before
    );
    assert_eq!(app.world().resource::<OriginChanges>().0, 1);
    app.update();
    assert_eq!(
        app.world().resource::<OriginChanges>().0,
        1,
        "unchanged origin stays quiet"
    );
    app.world_mut()
        .resource_mut::<BevyRuntimeShell>()
        .battle_origin
        .clear();
    app.update();
    assert!(app
        .world()
        .resource::<BattlePresentationOriginFrame>()
        .origin()
        .is_none());
    assert_eq!(app.world().resource::<OriginChanges>().0, 2);
    assert_eq!(
        app.world().resource::<BevyRuntimeShell>().shell.session(),
        &session_before
    );
}
