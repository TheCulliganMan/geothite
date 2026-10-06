fn fishing_anchor_app() -> (App, TilePosition) {
    let (controller, water) = fishing_origin_controller_at_shore();
    let mut app = menu_render_test_app(controller.shell);
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
    (app, water)
}

fn fishing_anchor_set_trace(app: &mut App, bite: bool, native_vblank: bool) -> Vec<u8> {
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    let samples = if bite {
        let data = shell.shell.runtime().data();
        let map_name = shell.shell.session().snapshot().map_name;
        let group = data.map_fishing_group(&map_name).unwrap().unwrap();
        let (_, rod) = data.fishing_rod_item("GOOD_ROD").unwrap();
        let group = &data.fishing.groups[group];
        assert!(group.bite_threshold > 0);
        let mut lower = 0_u8;
        let roll = group.rod_tables[rod]
            .slots
            .iter()
            .find_map(|slot| {
                let roll = lower;
                lower = slot.threshold.saturating_add(1);
                slot.species
                    .as_ref()
                    .filter(|species| !matches!(species.as_str(), "MAGIKARP" | "UNOWN"))
                    .filter(|_| roll <= slot.threshold)
                    .map(|_| roll)
            })
            .unwrap();
        vec![0, 0, 0, 0_u8.wrapping_sub(roll), 0, roll, 0, 0, 0, 0]
    } else {
        vec![0, 1]
    };
    let session = shell.shell.session_mut();
    session.state_mut().random_state = crate::core::random::CrystalRandomState::default();
    // The graphical keyboard path executes one VBlank_Normal before dispatch.
    // Its two zero reads leave add/sub at zero, then the exact cast trace starts.
    // The direct controller path has no host VBlank and uses the cast alone.
    let prefix = if native_vblank {
        vec![0, 0]
    } else {
        Vec::new()
    };
    *session.divider_mut_for_tests() = crate::core::random::RuntimeDividerSource::replay(
        prefix.into_iter().chain(samples.iter().copied()),
    );
    samples
}

/// Navigate to the rod's production action menu without setting any cursor.
fn fishing_anchor_open_pack_rod(app: &mut App) {
    press_key_for_runtime_hotkey_app(app, KeyCode::Enter);
    for _ in 0..12 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let entries = visible_start_menu_entries(shell).unwrap();
        if entries.iter().any(|entry| entry == ">PACK") {
            break;
        }
        press_key_for_runtime_hotkey_app(app, KeyCode::ArrowDown);
    }
    assert!(
        visible_start_menu_entries(app.world().resource::<BevyRuntimeShell>())
            .unwrap()
            .iter()
            .any(|entry| entry == ">PACK")
    );
    press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
    for _ in 0..8 {
        if active_visible_field_pack_pocket(app.world().resource::<BevyRuntimeShell>())
            == FieldPackPocket::KeyItems
        {
            break;
        }
        press_key_for_runtime_hotkey_app(app, KeyCode::ArrowRight);
    }
    assert_eq!(
        active_visible_field_pack_pocket(app.world().resource::<BevyRuntimeShell>()),
        FieldPackPocket::KeyItems
    );
    for _ in 0..32 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let snapshot = shell.shell.snapshot().unwrap();
        if selected_field_pack_item_id_from_snapshot(&snapshot, shell, &FieldPackPocket::KeyItems)
            .as_deref()
            == Some("GOOD_ROD")
        {
            break;
        }
        press_key_for_runtime_hotkey_app(app, KeyCode::ArrowDown);
    }
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let snapshot = shell.shell.snapshot().unwrap();
        assert_eq!(
            selected_field_pack_item_id_from_snapshot(&snapshot, shell, &FieldPackPocket::KeyItems)
                .as_deref(),
            Some("GOOD_ROD")
        );
    }
    press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(
        visible_field_pack_action_entries(&snapshot, shell, &FieldPackPocket::KeyItems)
            .unwrap()
            .iter()
            .any(|entry| entry.trim() == ">USE")
    );
}

#[test]
fn fishing_anchor_pack_and_registered_inputs_freeze_checked_water_through_entry_and_exit() {
    use bevy::ecs::system::RunSystemOnce;
    for pack in [false, true] {
        let (mut app, water) = fishing_anchor_app();
        if pack {
            fishing_anchor_open_pack_rod(&mut app);
        }
        let retained_pack_scene = app
            .world()
            .resource::<BevyRuntimeShell>()
            .battle_origin
            .fishing_pack_scene
            .as_ref()
            .and_then(|pack| pack.scene.clone());
        assert_eq!(retained_pack_scene.is_some(), pack);
        assert!(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .is_none(),
            "retained Pack scenery is not an encounter"
        );
        let samples = fishing_anchor_set_trace(&mut app, true, true);
        let (source, mut replay, commands_before) = {
            let shell = app.world().resource::<BevyRuntimeShell>();
            (
                shell.shell.session().snapshot(),
                shell.shell.session().clone(),
                shell.shell.retained_runtime_commands().len(),
            )
        };
        press_key_for_runtime_hotkey_app(
            &mut app,
            if pack {
                KeyCode::KeyZ
            } else {
                KeyCode::ShiftRight
            },
        );
        assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
        let location = app
            .world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .clone()
            .expect("committed fishing battle must publish its checked contact");
        let shell = app.world().resource::<BevyRuntimeShell>();
        let origin = shell.battle_origin.pending.as_ref().unwrap().clone();
        let commands = &shell.shell.retained_runtime_commands()[commands_before..];
        assert_eq!(
            commands.len(),
            2,
            "one native VBlank then exactly one bag cast"
        );
        let crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(timer) =
            crystal_assets::decode_runtime_mutation_command_frame(&commands[0], replay.state())
                .unwrap()
        else {
            panic!("native input must first execute its source VBlank");
        };
        assert_eq!(timer.vblanks, 1);
        assert_eq!(timer.normal_divider_trace.samples, [0, 0]);
        let runtime = shell.shell.runtime().clone();
        replay
            .apply_runtime_command_frame(&runtime, &commands[0])
            .unwrap();
        let crystal_assets::RuntimeMutationCommand::UseBagFishingRodInField(command) =
            crystal_assets::decode_runtime_mutation_command_frame(&commands[1], replay.state())
                .unwrap()
        else {
            panic!("production rod input must use authoritative bag cast");
        };
        assert_eq!(command.item_id, "GOOD_ROD");
        assert_eq!(command.divider_trace.samples, samples);
        replay
            .apply_runtime_command_frame(&runtime, &commands[1])
            .unwrap();
        assert_eq!(replay.state(), shell.shell.session().state());
        assert_eq!(origin.source, source);
        assert_eq!(location.generation, origin.generation);
        assert_eq!(location.source.map_id.as_ref(), source.map_name);
        assert_eq!(
            location.source.core_tile,
            IVec2::new(source.tile.x.into(), source.tile.y.into())
        );
        assert_eq!(
            location.target,
            crystal_render_api::VisualBattleTarget::FishingWater {
                core_tile: IVec2::new(water.x.into(), water.y.into()),
            }
        );
        let anchors = location.anchors.as_ref().expect("matching pre-cast scene");
        if let Some(scene) = retained_pack_scene {
            assert_eq!(
                anchors.terrain, scene.terrain,
                "Pack USE must bind its exact opening scene"
            );
            assert_eq!(anchors.source_foot, scene.source_foot);
        }
        assert!(
            shell.battle_origin.fishing_pack_scene.is_none(),
            "cast consumes private Pack evidence"
        );
        assert_eq!(
            anchors.source_actor,
            crystal_render_api::VisualActorId::Player
        );
        assert_eq!(anchors.target_actor, None);
        assert_eq!(
            anchors.target_foot - anchors.source_foot,
            visual_facing(source.facing) * TILE_SIZE * 2.0
        );
        assert_eq!(
            location.source.facing,
            IVec2::new(
                i32::from(water.x - source.tile.x),
                i32::from(water.y - source.tile.y)
            )
        );
        let session = shell.shell.session().clone();
        let commands = shell.shell.retained_runtime_commands().to_vec();
        let results = shell.shell.retained_runtime_results().to_vec();
        // Isolate publication from the normal frame tick to prove it is read-only.
        app.world_mut()
            .run_system_once(publish_visible_battle_location);
        app.world_mut()
            .run_system_once(publish_visible_battle_location);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session(), &session);
        assert_eq!(shell.shell.retained_runtime_commands(), commands);
        assert_eq!(shell.shell.retained_runtime_results(), results);
        assert!(Arc::ptr_eq(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .as_ref()
                .unwrap(),
            &location
        ));

        let mut controller = VisibleShellController {
            shell: app
                .world_mut()
                .remove_resource::<BevyRuntimeShell>()
                .unwrap(),
        };
        fishing_origin_assert_consumed_divider(&controller, samples.len() + 2);
        fishing_origin_advance_to_notice(&mut controller);
        assert_eq!(controller.shell.shell.session(), &session);
        controller.press(GameButton::A).unwrap();
        assert!(Arc::ptr_eq(
            controller.shell.battle_origin.active.as_ref().unwrap(),
            &origin
        ));
        prepare_visible_battle_entry_after_visible_step(&mut controller.shell).unwrap();
        assert!(Arc::ptr_eq(
            controller
                .shell
                .battle_origin
                .bound_fishing
                .as_ref()
                .unwrap()
                .publication
                .as_ref()
                .unwrap(),
            &location
        ));
        reset_visible_battle_exit_state(&mut controller.shell);
        assert!(
            controller.shell.battle_origin.bound_fishing.is_some(),
            "terminal text retains location"
        );
        controller.shell.battle_messages.clear();
        reset_visible_battle_exit_state(&mut controller.shell);
        assert!(controller.shell.battle_origin.bound_fishing.is_none());
        app.insert_resource(controller.shell);
        app.world_mut()
            .run_system_once(publish_visible_battle_location);
        assert!(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .is_none()
        );
    }
}

#[test]
fn fishing_anchor_no_bite_and_pack_cancel_never_publish_a_location() {
    use bevy::ecs::system::RunSystemOnce;
    for pack in [false, true] {
        let (mut app, _) = fishing_anchor_app();
        if pack {
            fishing_anchor_open_pack_rod(&mut app);
        }
        fishing_anchor_set_trace(&mut app, false, true);
        press_key_for_runtime_hotkey_app(
            &mut app,
            if pack {
                KeyCode::KeyZ
            } else {
                KeyCode::ShiftRight
            },
        );
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        assert!(shell.visible_fishing_animation.is_some());
        assert!(shell.shell.snapshot().unwrap().battle.is_none());
        assert!(shell.battle_origin.bound_fishing.is_none());
        assert!(shell.battle_origin.published().is_none());
        assert!(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .is_none()
        );
        let mut controller = VisibleShellController {
            shell: app
                .world_mut()
                .remove_resource::<BevyRuntimeShell>()
                .unwrap(),
        };
        fishing_origin_assert_consumed_divider(&controller, 4);
        fishing_origin_advance_to_notice(&mut controller);
        controller.press(GameButton::B).unwrap();
        assert!(controller.shell.visible_fishing_animation.is_none());
        app.insert_resource(controller.shell);
        app.world_mut()
            .run_system_once(publish_visible_battle_location);
        assert!(
            app.world()
                .resource::<crystal_render_api::BattleLocationFrame>()
                .location
                .is_none()
        );
    }
    let (mut app, _) = fishing_anchor_app();
    fishing_anchor_open_pack_rod(&mut app);
    let (state_before, commands_before) = {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        *shell.shell.session_mut().divider_mut_for_tests() =
            crate::core::random::RuntimeDividerSource::replay([0, 0]);
        (
            shell.shell.session().state().clone(),
            shell.shell.retained_runtime_commands().len(),
        )
    };
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    assert!(shell.field_pack_action_cursor.is_none());
    let commands = &shell.shell.retained_runtime_commands()[commands_before..];
    assert_eq!(
        commands.len(),
        1,
        "cancel only advances the native clock; it never casts"
    );
    let crystal_assets::RuntimeMutationCommand::AdvanceGameTimerVBlanks(timer) =
        crystal_assets::decode_runtime_mutation_command_frame(&commands[0], &state_before).unwrap()
    else {
        panic!("Pack cancel must not issue a bag cast");
    };
    assert_eq!(timer.vblanks, 1);
    assert_eq!(timer.normal_divider_trace.samples, [0, 0]);
    assert!(shell.battle_origin.bound_fishing.is_none());
    assert!(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_none()
    );
}

#[test]
fn fishing_anchor_stale_or_missing_scene_fails_once_without_changing_runtime() {
    let (mut app, _) = fishing_anchor_app();
    let frame = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    let rendered = app
        .world_mut()
        .remove_resource::<RenderedViewport>()
        .unwrap();
    let mut controller = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .unwrap(),
    };
    // Use the real controller route, but delay publication to inspect its evidence.
    app.insert_resource(controller.shell);
    fishing_anchor_set_trace(&mut app, true, false);
    controller.shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    controller.press(GameButton::Select).unwrap();
    let session = controller.shell.shell.session().clone();
    let commands = controller.shell.shell.retained_runtime_commands().to_vec();
    let results = controller.shell.shell.retained_runtime_results().to_vec();
    let prototype = controller.shell.battle_origin.bound_fishing.take().unwrap();
    for defect in 0..10 {
        let mut bound = VisibleBoundFishingEncounter {
            origin: prototype.origin.clone(),
            water_tile: prototype.water_tile,
            map_size: prototype.map_size,
            capture_attempted: false,
            anchors: None,
            publication: None,
        };
        let mut bad = frame.clone();
        match defect {
            0 => bad.map_id = Arc::from("UnrelatedMap"),
            1 => bad.grid_origin.x += 1,
            2 => bad
                .actors
                .retain(|actor| actor.id != crystal_render_api::VisualActorId::Player),
            3 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .center
                    .y += 1.0
            }
            4 => bad.terrain_revision = bad.terrain_revision.wrapping_add(1),
            5 => bad.source_map_size_core_tiles = None,
            6 => bad.active = false,
            7 => {} // absent world publication
            8 => {
                bad.actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .unwrap()
                    .facing = None
            }
            9 => bound.water_tile = TilePosition::new(-1, -1),
            _ => unreachable!(),
        }
        freeze_visible_fishing_encounter_anchors(
            &mut bound,
            &rendered,
            (defect != 7).then_some(&bad),
        );
        assert!(bound.capture_attempted);
        assert!(bound.anchors.is_none(), "defect {defect} must fail closed");
        freeze_visible_fishing_encounter_anchors(&mut bound, &rendered, Some(&frame));
        assert!(
            bound.anchors.is_none(),
            "late warm frame must not repair source evidence"
        );
        assert!(fishing_encounter_location(&mut bound).anchors.is_none());
    }
    assert_eq!(controller.shell.shell.session(), &session);
    assert_eq!(controller.shell.shell.retained_runtime_commands(), commands);
    assert_eq!(controller.shell.shell.retained_runtime_results(), results);
}

#[test]
fn fishing_anchor_save_load_retires_binding_and_does_not_alias_the_next_cast() {
    let (mut app, _) = fishing_anchor_app();
    let save = std::env::temp_dir().join(format!(
        "fishing-anchor-{}.crystalsave",
        uuid::Uuid::new_v4()
    ));
    app.world_mut()
        .resource_mut::<BevyRuntimeShell>()
        .shell
        .save(&save)
        .unwrap();
    fishing_anchor_set_trace(&mut app, true, true);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ShiftRight);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
    let first = app
        .world()
        .resource::<crystal_render_api::BattleLocationFrame>()
        .location
        .clone()
        .unwrap();
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        // The finite cast trace has ended. Continue preserves the hardware
        // divider by design, so restore live fixture hardware for warmup frames.
        *shell.shell.session_mut().divider_mut_for_tests() =
            crate::core::random::RuntimeDividerSource::live();
        // Native load actions invalidate cached input/render snapshots through
        // this wrapper; calling the low-level loader alone skips that boundary.
        run_bevy_action(&mut shell, |shell| {
            load_visible_runtime_save(shell, &save, "fishing_anchor_test")
        });
        assert_eq!(shell.last_error, None);
        assert!(shell.battle_origin.bound_fishing.is_none());
    }
    app.update();
    app.update();
    assert!(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_none()
    );
    fishing_anchor_set_trace(&mut app, true, true);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ShiftRight);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
    let next = app
        .world()
        .resource::<crystal_render_api::BattleLocationFrame>()
        .location
        .as_ref()
        .unwrap();
    assert_ne!(next.generation, first.generation);
    assert!(!Arc::ptr_eq(next, &first));
    std::fs::remove_file(save).unwrap();
}

#[test]
fn fishing_anchor_pack_evidence_is_one_shot_and_clears_on_close_navigation_or_stale_pose() {
    let (mut app, _) = fishing_anchor_app();
    let frame = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    let rendered = app
        .world_mut()
        .remove_resource::<RenderedViewport>()
        .unwrap();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    open_visible_field_pack(&mut shell).unwrap();
    let session = shell.shell.session().clone();
    capture_visible_fishing_pack_scene(&mut shell, &rendered, None);
    assert!(
        shell
            .battle_origin
            .fishing_pack_scene
            .as_ref()
            .unwrap()
            .capture_attempted
    );
    capture_visible_fishing_pack_scene(&mut shell, &rendered, Some(&frame));
    assert!(
        shell
            .battle_origin
            .fishing_pack_scene
            .as_ref()
            .unwrap()
            .scene
            .is_none(),
        "a cold Pack opening cannot adopt a later warm frame"
    );
    assert_eq!(shell.shell.session(), &session);
    close_visible_field_pack_without_log(&mut shell);
    capture_visible_fishing_pack_scene(&mut shell, &rendered, Some(&frame));
    assert!(shell.battle_origin.fishing_pack_scene.is_none());

    open_visible_field_pack(&mut shell).unwrap();
    capture_visible_fishing_pack_scene(&mut shell, &rendered, Some(&frame));
    assert!(
        shell
            .battle_origin
            .fishing_pack_scene
            .as_ref()
            .unwrap()
            .scene
            .is_some()
    );
    // Tamper only with the retained presentation claim, not runtime authority.
    shell
        .battle_origin
        .fishing_pack_scene
        .as_mut()
        .unwrap()
        .source
        .tile
        .x += 1;
    capture_visible_fishing_pack_scene(&mut shell, &rendered, Some(&frame));
    assert!(
        shell
            .battle_origin
            .fishing_pack_scene
            .as_ref()
            .unwrap()
            .scene
            .is_none()
    );
    assert_eq!(shell.shell.session(), &session);
    reset_visible_navigation_state(&mut shell);
    assert!(shell.battle_origin.fishing_pack_scene.is_none());

    open_visible_field_pack(&mut shell).unwrap();
    capture_visible_fishing_pack_scene(&mut shell, &rendered, Some(&frame));
    assert!(
        shell
            .battle_origin
            .fishing_pack_scene
            .as_ref()
            .unwrap()
            .scene
            .is_some()
    );
    shell.battle_origin.clear();
    assert!(
        shell.battle_origin.fishing_pack_scene.is_none(),
        "load/battle reset drops pre-menu evidence"
    );
}
