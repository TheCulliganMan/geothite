// Real production input and checked-use fixtures. Coordinates below are only
// fixture setup; production obtains both positions from RuntimeSquirtBottleUse.
fn encounter_anchor_shell_at_tree() -> BevyRuntimeShell {
    let asset_root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap(),
    );
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier,
            map_name: "Route36".into(),
            tile_x: 35,
            tile_y: 10,
        },
        BevyShellConfig {
            smoke_player_name: Some("TEST".into()),
            ..Default::default()
        },
    )
    .unwrap();
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
    shell
        .shell
        .add_party_pokemon(
            "CYNDAQUIL",
            20,
            None,
            None,
            "TEST",
            1,
            Dv::from_non_hp(10, 10, 10, 10),
        )
        .unwrap();
    shell.shell.add_bag_item("SQUIRTBOTTLE", 1).unwrap();
    shell.shell.register_key_item("SQUIRTBOTTLE").unwrap();
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
    {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        overworld.set_player_facing(Direction::Up);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    mark_runtime_snapshot_dirty(&mut shell);
    shell.shell.set_runtime_journal_enabled(true);
    assert_eq!(
        shell
            .shell
            .current_overworld_interaction_checked()
            .unwrap()
            .map(|interaction| interaction.script),
        Some("SudowoodoScript".into())
    );
    shell
}

fn encounter_anchor_app() -> App {
    let mut app = menu_render_test_app(encounter_anchor_shell_at_tree());
    app.init_resource::<crystal_render_api::VisualWorldFrame>()
        .init_resource::<crystal_render_api::BattleLocationFrame>()
        .init_resource::<BattlePresentationOriginFrame>()
        .add_systems(
            Update,
            publish_visible_battle_location
                .after(play_pending_audio)
                .after(tick_visible_screen_fade)
                .before(render_playfield),
        )
        .add_systems(
            Update,
            publish_visible_battle_origin
                .after(publish_visible_battle_location)
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
    app
}

fn encounter_anchor_advance_to_yes_no(app: &mut App) {
    for _ in 0..600 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        if shell.shell.snapshot().unwrap().ui.pending_yes_no.is_some() {
            return;
        }
        assert!(shell.shell.snapshot().unwrap().battle.is_none());
        press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
    }
    panic!("the compiled object interaction did not reach its yes/no boundary");
}

#[test]
fn encounter_anchor_registered_input_binds_only_real_script_battle_and_keeps_frozen_frame() {
    let mut app = encounter_anchor_app();
    let before_pose = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .snapshot();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ShiftRight);
    let (source, frozen) = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let candidate = shell
            .battle_origin
            .static_candidate
            .as_ref()
            .expect("checked use candidate");
        assert!(same_static_encounter_pose(&before_pose, &candidate.source));
        assert_eq!(candidate.target_tile, TilePosition::new(35, 9));
        assert_eq!(candidate.object_script, "SudowoodoScript");
        assert_eq!(candidate.trigger_script, "WateredWeirdTreeScript");
        assert!(candidate.witnessed_start.is_none());
        assert!(
            shell.battle_origin.published().is_none(),
            "a checked use is not a committed encounter"
        );
        assert!(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .is_none()
        );
        (
            candidate.source.clone(),
            candidate
                .anchors
                .as_ref()
                .expect("matching warmed preceding frame")
                .clone(),
        )
    };
    for _ in 0..1600 {
        if app
            .world()
            .resource::<BevyRuntimeShell>()
            .battle_origin
            .bound_static
            .is_some()
        {
            break;
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    let location = app
        .world()
        .resource::<crystal_render_api::BattleLocationFrame>()
        .location
        .as_ref()
        .expect("real compiled start must bind")
        .clone();
    let shell = app.world().resource::<BevyRuntimeShell>();
    let origin = shell.battle_origin.active.as_ref().unwrap();
    assert_eq!(origin.source, source);
    assert_eq!(location.generation, origin.generation);
    assert_eq!(location.target.object_script.as_ref(), "SudowoodoScript");
    assert_eq!(
        location.target.trigger_script.as_ref(),
        "WateredWeirdTreeScript"
    );
    assert_eq!(
        location.target.battle_source_script.as_ref(),
        "WateredWeirdTreeScript"
    );
    assert_eq!(location.target.startbattle_command_index, 12);
    assert_eq!(location.source.core_tile, IVec2::new(35, 10));
    assert_eq!(location.target.core_tile, IVec2::new(35, 9));
    let (width, height) = shell.shell.session().overworld().map.tile_bounds();
    assert_eq!(
        location.source_map_size_core_tiles,
        Some(UVec2::new(width.into(), height.into()))
    );
    assert_eq!(
        frozen.terrain.source_map_size_core_tiles,
        location.source_map_size_core_tiles
    );
    assert!(Arc::ptr_eq(location.anchors.as_ref().unwrap(), &frozen));
    let state_after_start = shell.shell.session().clone();
    // Publication alone cannot consume a DIV sample, modify a command/result,
    // or move the player; it also keeps the same immutable location Arc.
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    let mut publication = App::new();
    let mut owned_shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    prepare_visible_battle_entry_after_visible_step(&mut owned_shell).unwrap();
    publication
        .insert_resource(owned_shell)
        .insert_resource(RenderedViewport::default())
        .insert_resource(crystal_render_api::BattleLocationFrame {
            location: Some(location.clone()),
        })
        .add_systems(Update, publish_visible_battle_location);
    publication.update();
    publication.update();
    let shell = publication.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session(), &state_after_start);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
    assert!(Arc::ptr_eq(
        publication
            .world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .as_ref()
            .unwrap(),
        &location
    ));
    let mut shell = publication.world_mut().resource_mut::<BevyRuntimeShell>();
    reset_visible_battle_exit_state(&mut shell);
    assert!(
        shell.battle_origin.bound_static.is_some(),
        "retain through terminal narration"
    );
    shell.battle_messages.clear();
    reset_visible_battle_exit_state(&mut shell);
    drop(shell);
    publication.update();
    assert!(
        publication
            .world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_none()
    );
}

#[test]
fn encounter_anchor_real_cancel_discards_retained_candidate_before_later_direct_battle() {
    // Bag/registered use intentionally skips Yes/No. Exercise cancellation at
    // the real object interaction prompt, with a retained checked-use candidate
    // injected only to verify that cancellation cannot leave old provenance.
    let mut donor = encounter_anchor_shell_at_tree();
    let before = donor.shell.snapshot().unwrap();
    let checked = donor
        .shell
        .use_bag_squirtbottle_in_field("SQUIRTBOTTLE")
        .unwrap();
    stage_visible_squirtbottle_candidate(&mut donor, &before, &checked);
    let candidate = donor.battle_origin.static_candidate.take().unwrap();
    let mut app = encounter_anchor_app();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    encounter_anchor_advance_to_yes_no(&mut app);
    app.world_mut()
        .resource_mut::<BevyRuntimeShell>()
        .battle_origin
        .static_candidate = Some(candidate);
    for _ in 0..120 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
        if app
            .world()
            .resource::<BevyRuntimeShell>()
            .battle_origin
            .static_candidate
            .is_none()
        {
            break;
        }
    }
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.battle_origin.static_candidate.is_none());
    assert!(shell.battle_origin.published().is_none());
    assert!(shell.shell.snapshot().unwrap().battle.is_none());
    shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .unwrap();
    prepare_visible_battle_entry(&mut shell).unwrap();
    assert!(shell.battle_origin.bound_static.is_none());
    assert!(
        shell
            .battle_origin
            .active
            .as_ref()
            .unwrap()
            .contact
            .is_none()
    );
}

#[test]
fn encounter_anchor_checked_capture_rejects_stale_identity_pose_and_missing_frame_without_mutation()
{
    let mut app = encounter_anchor_app();
    let world = app.world_mut();
    let objects: Vec<_> = world
        .query::<&VisibleObjectSprite>()
        .iter(world)
        .map(|object| {
            (
                object.object_index,
                object.object_identifier.clone(),
                object.source_id.to_string(),
            )
        })
        .collect();
    let frame = world
        .remove_resource::<crystal_render_api::VisualWorldFrame>()
        .unwrap();
    let rendered = world.remove_resource::<RenderedViewport>().unwrap();
    let mut shell = world.remove_resource::<BevyRuntimeShell>().unwrap();
    let before = shell.shell.snapshot().unwrap();
    let checked = shell
        .shell
        .use_bag_squirtbottle_in_field("SQUIRTBOTTLE")
        .unwrap();
    let session = shell.shell.session().clone();
    let checksum = shell.shell.state_checksum().unwrap();
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    let object_refs: Vec<_> = objects
        .iter()
        .map(|(slot, id, source)| (*slot, id.as_deref(), source.as_str()))
        .collect();
    // Each case starts from the same actual checked runtime result and warmed
    // scene. Only the untrusted presentation evidence changes.
    for defect in 0..10 {
        stage_visible_squirtbottle_candidate(&mut shell, &before, &checked);
        let candidate = shell.battle_origin.static_candidate.as_mut().unwrap();
        let mut bad = frame.clone();
        let mut roster = object_refs.clone();
        match defect {
            0 => bad.map_id = Arc::from("UnrelatedMap"),
            1 => bad.grid_origin.x += 1,
            2 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .center
                    .x += 1.0
            }
            3 => roster.retain(|(_, id, _)| *id != Some(candidate.target_identifier.as_str())),
            4 => {
                let target = roster
                    .iter()
                    .find(|(_, id, _)| *id == Some(candidate.target_identifier.as_str()))
                    .unwrap();
                roster.push(*target);
            }
            5 => bad.terrain_revision = bad.terrain_revision.wrapping_add(1),
            6 => bad.active = false,
            7 => {} // absent resource
            8 => {
                bad.source_map_size_core_tiles =
                    bad.source_map_size_core_tiles.map(|size| size + UVec2::X)
            }
            9 => bad.source_map_size_core_tiles = None,
            _ => unreachable!(),
        }
        freeze_visible_static_encounter_anchors(
            candidate,
            &rendered,
            (defect != 7).then_some(&bad),
            &roster,
        );
        assert!(
            candidate.anchors.is_none(),
            "defect {defect} must fail closed"
        );
        // A later warm frame must not retroactively turn a failed source
        // observation into authoritative entry evidence.
        freeze_visible_static_encounter_anchors(candidate, &rendered, Some(&frame), &object_refs);
        assert!(candidate.anchors.is_none());
    }
    stage_visible_squirtbottle_candidate(&mut shell, &before, &checked);
    let candidate = shell.battle_origin.static_candidate.as_mut().unwrap();
    assert_eq!(candidate.source, before.overworld);
    assert_eq!(candidate.source.tile, checked.player_tile);
    assert_eq!(candidate.target_tile, checked.target_tile);
    assert_eq!(
        Some(candidate.target_identifier.as_str()),
        checked.target_object_identifier.as_deref()
    );
    assert_eq!(
        Some(candidate.trigger_script.as_str()),
        checked.target_script.as_deref()
    );
    freeze_visible_static_encounter_anchors(candidate, &rendered, Some(&frame), &object_refs);
    let anchors = candidate
        .anchors
        .as_ref()
        .expect("unmodified source-backed evidence");
    assert!(anchors.terrain.matches_built_frame(&frame));
    let mut built_other_grid = frame.clone();
    built_other_grid.grid_origin.x += 1;
    assert!(!anchors.terrain.matches_built_frame(&built_other_grid));
    assert_ne!(anchors.source_foot, anchors.target_foot);
    let target_slot = objects
        .iter()
        .find(|(_, id, _)| id.as_deref() == checked.target_object_identifier.as_deref())
        .unwrap()
        .0;
    assert_eq!(
        anchors.target_actor,
        crystal_render_api::VisualActorId::Object(target_slot as u32)
    );
    assert_eq!(
        shell.shell.session(),
        &session,
        "includes RNG and DIV trace"
    );
    assert_eq!(shell.shell.state_checksum().unwrap(), checksum);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
}

#[test]
fn encounter_anchor_refusal_navigation_load_and_unwitnessed_start_clear_candidates() {
    let mut controller = VisibleShellController {
        shell: encounter_anchor_shell_at_tree(),
    };
    let save = std::env::temp_dir().join(format!(
        "crystal-encounter-anchor-{}.crystalsave",
        uuid::Uuid::new_v4()
    ));
    controller.shell.shell.save(&save).unwrap();
    controller.press(GameButton::Select).unwrap();
    assert!(controller.shell.battle_origin.static_candidate.is_some());
    load_visible_runtime_save(&mut controller.shell, &save, "encounter_anchor_test").unwrap();
    assert!(controller.shell.battle_origin.static_candidate.is_none());
    assert!(controller.shell.battle_origin.bound_static.is_none());
    assert_eq!(
        controller.shell.shell.session().snapshot().facing,
        Direction::Up
    );
    assert_eq!(
        controller
            .shell
            .shell
            .current_overworld_interaction_checked()
            .unwrap()
            .map(|interaction| interaction.script),
        Some("SudowoodoScript".into()),
    );
    // Stage the real checked result directly so navigation and refusal can be
    // inspected without executing unrelated script steps.
    let before = controller.shell.shell.snapshot().unwrap();
    let checked = controller
        .shell
        .shell
        .use_bag_squirtbottle_in_field("SQUIRTBOTTLE")
        .unwrap();
    assert_eq!(
        checked.target_script.as_deref(),
        Some("WateredWeirdTreeScript")
    );
    assert_eq!(
        checked.target_object_identifier.as_deref(),
        Some("ROUTE36_WEIRD_TREE")
    );
    stage_visible_squirtbottle_candidate(&mut controller.shell, &before, &checked);
    assert!(controller.shell.battle_origin.static_candidate.is_some());
    reset_visible_navigation_state(&mut controller.shell);
    assert!(controller.shell.battle_origin.static_candidate.is_none());
    stage_visible_squirtbottle_candidate(&mut controller.shell, &before, &checked);
    assert!(controller.shell.battle_origin.static_candidate.is_some());
    handle_visible_field_action_refusal(
        &mut controller.shell,
        "SQUIRTBOTTLE",
        "SQUIRTBOTTLE CAN'T BE USED HERE",
        anyhow::anyhow!("test refusal"),
    )
    .unwrap();
    assert!(controller.shell.battle_origin.static_candidate.is_none());
    stage_visible_squirtbottle_candidate(&mut controller.shell, &before, &checked);
    assert!(controller.shell.battle_origin.static_candidate.is_some());
    controller
        .shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .unwrap();
    prepare_visible_battle_entry(&mut controller.shell).unwrap();
    assert!(controller.shell.battle_origin.static_candidate.is_none());
    assert!(
        controller.shell.battle_origin.bound_static.is_none(),
        "unwitnessed same-map start cannot claim checked contact"
    );
    assert!(
        controller
            .battle_presentation_origin()
            .unwrap()
            .contact
            .is_none()
    );
    std::fs::remove_file(save).unwrap();
}

#[test]
fn encounter_anchor_observer_requires_exact_actual_compiled_continuation() {
    let mut shell = encounter_anchor_shell_at_tree();
    let before = shell.shell.snapshot().unwrap();
    let checked = shell
        .shell
        .use_bag_squirtbottle_in_field("SQUIRTBOTTLE")
        .unwrap();
    stage_visible_squirtbottle_candidate(&mut shell, &before, &checked);
    let candidate = shell.battle_origin.static_candidate.take().unwrap();
    let next = shell
        .shell
        .run_pending_next_script_until_boundary(
            256,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .unwrap();
    let step = next
        .run
        .steps
        .first()
        .expect("actual compiled trigger step");
    assert_eq!(
        step.source_script,
        checked.target_script.as_deref().unwrap()
    );
    assert_eq!(step.command_index, 0);
    let session = shell.shell.session().clone();
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    for defect in 0..4 {
        shell.battle_origin.static_candidate = Some(candidate.clone());
        let mut other = step.clone();
        match defect {
            0 => other.origin_map_name = "UnrelatedMap".into(),
            1 => other.source_script = "UnrelatedScript".into(),
            2 => other.command_index += 1,
            3 => {
                shell
                    .battle_origin
                    .static_candidate
                    .as_mut()
                    .unwrap()
                    .source
                    .tile
                    .x += 1
            }
            _ => unreachable!(),
        }
        observe_visible_static_encounter_step(&mut shell, &other);
        assert!(shell.battle_origin.static_candidate.is_none());
    }
    shell.battle_origin.static_candidate = Some(candidate);
    observe_visible_static_encounter_step(&mut shell, step);
    assert_eq!(
        shell
            .battle_origin
            .static_candidate
            .as_ref()
            .unwrap()
            .expected_step,
        step.next_cursor
    );
    assert!(shell.battle_origin.published().is_none());
    assert_eq!(shell.shell.session(), &session);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
}

#[test]
fn visual_world_source_extent_belongs_to_the_built_scene_and_republishes_metadata_changes() {
    use bevy::ecs::system::RunSystemOnce;

    let mut app = encounter_anchor_app();
    let source = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    let expected_size = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let (width, height) = shell
            .shell
            .runtime()
            .data()
            .saved_map_tile_bounds(source.map_id.as_ref())
            .unwrap();
        UVec2::new(u32::from(width), u32::from(height))
    };
    assert_eq!(source.source_map_size_core_tiles, Some(expected_size));
    assert_eq!(
        app.world()
            .resource::<RenderedViewport>()
            .source_map_size_core_tiles,
        Some(expected_size)
    );
    // Advance the authority to another real map without rendering its scene.
    // Extraction must describe the already built Route36 terrain, not the live
    // map's dimensions or a queued terrain request.
    let (session, commands, results) = {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let runtime = shell.shell.runtime().clone();
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        runtime
            .data()
            .transition_overworld_session(
                state,
                overworld,
                "NewBarkTown",
                TilePosition::new(13, 6),
                crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
                &runtime.music_ids(),
            )
            .unwrap();
        let other_bounds = runtime.data().saved_map_tile_bounds("NewBarkTown").unwrap();
        assert_ne!(
            expected_size,
            UVec2::new(other_bounds.0.into(), other_bounds.1.into())
        );
        (
            shell.shell.session().clone(),
            shell.shell.retained_runtime_commands().to_vec(),
            shell.shell.retained_runtime_results().to_vec(),
        )
    };
    app.world_mut().run_system_once(publish_visual_world_frame);
    let retained = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>();
    assert_eq!(retained.map_id, source.map_id);
    assert_eq!(retained.source_map_size_core_tiles, Some(expected_size));
    assert_eq!(retained.grid_origin, source.grid_origin);
    assert_eq!(retained.terrain_revision, source.terrain_revision);

    // A metadata-only change must not disappear behind the unchanged-terrain
    // fast path. Unknown bounds preserve ordinary legacy frame publication.
    app.world_mut()
        .resource_mut::<RenderedViewport>()
        .source_map_size_core_tiles = None;
    app.world_mut().run_system_once(publish_visual_world_frame);
    let unknown = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>();
    assert!(unknown.active);
    assert_eq!(unknown.source_map_size_core_tiles, None);
    assert_eq!(unknown.tiles, source.tiles);
    app.world_mut()
        .resource_mut::<RenderedViewport>()
        .source_map_size_core_tiles = Some(expected_size);
    app.world_mut().run_system_once(publish_visual_world_frame);
    assert_eq!(
        app.world()
            .resource::<crystal_render_api::VisualWorldFrame>()
            .source_map_size_core_tiles,
        Some(expected_size)
    );
    app.world_mut()
        .resource_mut::<RenderedViewport>()
        .source_map_size_core_tiles = Some(UVec2::ZERO);
    app.world_mut().run_system_once(publish_visual_world_frame);
    assert!(
        !app.world()
            .resource::<crystal_render_api::VisualWorldFrame>()
            .active
    );
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session(), &session);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
}
