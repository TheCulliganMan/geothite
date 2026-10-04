fn scripted_trainer_anchor_fresh_shell() -> BevyRuntimeShell {
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
            map_name: "VioletGym".into(),
            tile_x: 4,
            tile_y: 1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            quick_save_path: None,
            ..Default::default()
        },
    )
    .unwrap();
    let mut shell = prepare_falkner_encounter_preview(shell).unwrap();
    shell.shell.set_runtime_journal_enabled(true);
    shell
}

fn scripted_trainer_anchor_dialogue_app() -> App {
    let shell = scripted_trainer_anchor_fresh_shell();
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
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    for _ in 0..240 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        assert!(shell.shell.snapshot().unwrap().battle.is_none());
        if shell
            .battle_origin
            .scripted_trainer_candidate
            .as_ref()
            .is_some_and(|candidate| candidate.dialogue_witnessed)
        {
            app.update();
            app.update();
            return app;
        }
    }
    panic!("ordinary A never reached Falkner's authored introduction");
}

fn scripted_trainer_anchor_start_from_dialogue(app: &mut App) {
    for _ in 0..600 {
        if app
            .world()
            .resource::<BevyRuntimeShell>()
            .battle_origin
            .bound_trainer
            .is_some()
        {
            return;
        }
        press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
    }
    panic!("Falkner dialogue never reached the authoritative trainer start");
}

#[test]
fn scripted_trainer_anchor_real_falkner_dialogue_party_and_frozen_lifetime() {
    use bevy::ecs::system::RunSystemOnce;
    let mut app = scripted_trainer_anchor_dialogue_app();
    let shell = app.world().resource::<BevyRuntimeShell>();
    let candidate = shell
        .battle_origin
        .scripted_trainer_candidate
        .as_ref()
        .unwrap()
        .clone();
    let evidence = &candidate.evidence;
    assert!(
        candidate.faceplayer_witnessed
            && candidate.dialogue_witnessed
            && !candidate.loadtrainer_witnessed
    );
    assert_eq!(evidence.source.map_name, "VioletGym");
    assert_eq!(evidence.source.tile, TilePosition::new(4, 1));
    assert_eq!(evidence.source.facing, Direction::Right);
    assert_eq!(evidence.target_tile, TilePosition::new(5, 1));
    assert_eq!(evidence.target_facing, Direction::Left);
    assert_eq!(evidence.target_identifier, "VIOLETGYM_FALKNER");
    assert_eq!(evidence.object_script, "VioletGymFalknerScript");
    assert_eq!(evidence.request.trainer_class, "FALKNER");
    assert_eq!(evidence.request.trainer_id, "FALKNER1");
    assert_eq!(evidence.request.event_flag, "");
    assert_eq!(
        evidence.provenance,
        crystal_render_api::VisualBattleTrainerProvenance::Scripted {
            loadtrainer_command_index: 8,
            startbattle_command_index: 9
        }
    );
    assert_eq!(evidence.placement.presentation_core_tile, IVec2::new(4, 3));
    assert_eq!(evidence.placement.walkable_core_tiles.len(), 23);
    assert!(
        !evidence
            .placement
            .walkable_core_tiles
            .contains(&IVec2::new(5, 1))
    );
    assert!(
        evidence.anchors.is_some(),
        "completed post-faceplayer dialogue must supply exact scene evidence"
    );
    assert!(shell.battle_origin.published().is_none());
    assert!(
        shell.pending_trainer_intro.is_none(),
        "scripted object never invents a trainer-table intro"
    );
    let before = shell.shell.session().clone();
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session(), &before);
    assert_eq!(shell.shell.retained_runtime_commands(), commands);
    assert_eq!(shell.shell.retained_runtime_results(), results);
    scripted_trainer_anchor_start_from_dialogue(&mut app);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let origin = shell.battle_origin.published().unwrap().clone();
    let location = app
        .world()
        .resource::<crystal_render_api::BattleLocationFrame>()
        .location
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(origin.source, evidence.source);
    assert!(
        matches!(&origin.contact, Some(BattleOriginContact::ScriptedTrainerObjectTarget {
        tile, object_identifier, source_script, loadtrainer_command_index: 8, startbattle_command_index: 9, ..
    }) if *tile == TilePosition::new(5, 1) && object_identifier == "VIOLETGYM_FALKNER" && source_script == "VioletGymFalknerScript")
    );
    let crystal_render_api::VisualBattleTarget::Trainer {
        contact,
        presentation,
    } = &location.target
    else {
        panic!("scripted trainer target");
    };
    assert_eq!(contact.provenance, evidence.provenance);
    assert_eq!(contact.event_flag.as_ref(), "");
    assert_eq!(contact.core_tile, IVec2::new(5, 1));
    assert_eq!(contact.facing, IVec2::NEG_X);
    assert_eq!(presentation.presentation_core_tile, IVec2::new(4, 3));
    assert_eq!(contact.witnessed_foot, evidence.witnessed_foot);
    let anchors = location.anchors.as_ref().unwrap();
    assert!(Arc::ptr_eq(anchors, evidence.anchors.as_ref().unwrap()));
    assert_eq!(anchors.target_actor, None);
    assert_ne!(Some(anchors.target_foot), contact.witnessed_foot);
    let snapshot = shell.shell.snapshot().unwrap();
    let battle = snapshot.battle.as_ref().unwrap();
    assert_eq!(
        battle
            .enemy_party
            .iter()
            .map(|pokemon| (pokemon.species.id.as_str(), pokemon.level))
            .collect::<Vec<_>>(),
        vec![("PIDGEY", 7), ("PIDGEOTTO", 9)]
    );
    assert_eq!(battle.enemy_pokemon.species.id, "PIDGEY");
    assert!(
        !shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_BEAT_FALKNER")
            .unwrap()
    );
    assert_eq!(snapshot.overworld.tile, TilePosition::new(4, 1));
    assert_eq!(
        snapshot
            .visible_object_runtime_tiles
            .get("VIOLETGYM_FALKNER")
            .copied()
            .or_else(|| snapshot
                .visible_objects
                .iter()
                .find(|object| object.object_identifier.as_deref() == Some("VIOLETGYM_FALKNER"))
                .and_then(object_tile_position_checked)),
        Some(TilePosition::new(5, 1))
    );
    let before = shell.shell.session().clone();
    let commands = shell.shell.retained_runtime_commands().to_vec();
    let results = shell.shell.retained_runtime_results().to_vec();
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(
        shell.shell.session(),
        &before,
        "publication preserves source clocks, RNG, save state and actor positions"
    );
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
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.battle_origin.scripted_trainer_candidate.is_none());
    reset_visible_battle_exit_state(&mut shell);
    assert!(
        shell.battle_origin.bound_trainer.is_some(),
        "terminal narration keeps frozen provenance"
    );
    shell.battle_messages.clear();
    reset_visible_battle_exit_state(&mut shell);
    assert!(shell.battle_origin.bound_trainer.is_none());
    drop(shell);
    app.world_mut()
        .run_system_once(publish_visible_battle_location);
    assert!(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_none()
    );
}

#[test]
fn scripted_trainer_anchor_rejects_cold_stale_wrong_or_missing_actor_evidence() {
    let mut app = scripted_trainer_anchor_dialogue_app();
    let mut evidence = app
        .world()
        .resource::<BevyRuntimeShell>()
        .battle_origin
        .scripted_trainer_candidate
        .as_ref()
        .unwrap()
        .evidence
        .clone();
    evidence.anchors = None;
    evidence.witnessed_actor = None;
    evidence.witnessed_foot = None;
    evidence.capture_attempted = false;
    evidence.warm_checked = false;
    let frame = app
        .world()
        .resource::<crystal_render_api::VisualWorldFrame>()
        .clone();
    let mut rendered = app
        .world_mut()
        .remove_resource::<RenderedViewport>()
        .unwrap();
    let roster = app
        .world_mut()
        .query::<&VisibleObjectSprite>()
        .iter(app.world())
        .map(|object| {
            (
                object.object_index,
                object.object_identifier.clone(),
                object.source_id.to_string(),
            )
        })
        .collect::<Vec<_>>();
    let objects = roster
        .iter()
        .map(|(slot, id, source)| (*slot, id.as_deref(), source.as_str()))
        .collect::<Vec<_>>();
    let actor = crystal_render_api::VisualActorId::Object(evidence.target_index as u32);
    for defect in 0..8 {
        let mut candidate = evidence.clone();
        let mut bad = frame.clone();
        match defect {
            0 => bad.active = false,
            1 => bad.terrain_revision += 1,
            2 => bad.actors.retain(|entry| entry.id != actor),
            3 => {
                bad.actors
                    .iter_mut()
                    .find(|entry| entry.id == actor)
                    .unwrap()
                    .facing = Some(Vec2::Y)
            }
            4 => {
                bad.actors
                    .iter_mut()
                    .find(|entry| entry.id == actor)
                    .unwrap()
                    .center
                    .x += 16.0
            }
            5 => {
                bad.actors
                    .iter_mut()
                    .find(|entry| entry.id == actor)
                    .unwrap()
                    .source_id = Arc::from("other_object")
            }
            6 => bad
                .actors
                .retain(|entry| entry.id != crystal_render_api::VisualActorId::Player),
            7 => {}
            _ => unreachable!(),
        }
        freeze_visible_trainer_anchors(
            &mut candidate,
            &rendered,
            (defect != 7).then_some(&bad),
            &objects,
        );
        assert!(
            candidate.capture_attempted && candidate.anchors.is_none(),
            "defect {defect}"
        );
        freeze_visible_trainer_anchors(&mut candidate, &rendered, Some(&frame), &objects);
        assert!(
            candidate.anchors.is_none(),
            "later scene cannot repair a rejected witness"
        );
    }
    let revision = rendered.snapshot_revision;
    rendered.snapshot_revision = Some(evidence.minimum_snapshot_revision.wrapping_sub(1));
    let mut candidate = evidence.clone();
    freeze_visible_trainer_anchors(&mut candidate, &rendered, Some(&frame), &objects);
    assert!(!candidate.capture_attempted && candidate.anchors.is_none());
    rendered.snapshot_revision = revision;
    freeze_visible_trainer_anchors(&mut candidate, &rendered, Some(&frame), &objects);
    assert!(candidate.anchors.is_some());
    let mut candidate = evidence;
    freeze_visible_trainer_anchors(&mut candidate, &rendered, Some(&frame), &[]);
    assert!(candidate.anchors.is_none());
}

#[test]
fn scripted_trainer_anchor_controller_cold_start_and_defeated_branch_are_truthful() {
    let mut controller = VisibleShellController {
        shell: scripted_trainer_anchor_fresh_shell(),
    };
    controller.press(GameButton::A).unwrap();
    assert!(
        controller
            .shell
            .battle_origin
            .scripted_trainer_candidate
            .as_ref()
            .is_some_and(|candidate| candidate.dialogue_witnessed)
    );
    for _ in 0..100 {
        if controller.shell.shell.snapshot().unwrap().battle.is_some() {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    assert!(controller.shell.shell.snapshot().unwrap().battle.is_some());
    let bound = controller
        .shell
        .battle_origin
        .bound_trainer
        .as_ref()
        .unwrap();
    assert!(
        bound.candidate.anchors.is_none(),
        "a controller without a rendered scene cannot invent anchors"
    );
    assert!(bound.candidate.witnessed_actor.is_none());
    assert!(matches!(
        bound.origin.contact,
        Some(BattleOriginContact::ScriptedTrainerObjectTarget { .. })
    ));
    let mut defeated = VisibleShellController {
        shell: scripted_trainer_anchor_fresh_shell(),
    };
    defeated
        .shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_BEAT_FALKNER", true)
        .unwrap();
    defeated.press(GameButton::A).unwrap();
    for _ in 0..12 {
        defeated.press(GameButton::A).unwrap();
    }
    assert!(defeated.shell.shell.snapshot().unwrap().battle.is_none());
    assert!(
        defeated
            .shell
            .battle_origin
            .scripted_trainer_candidate
            .is_none()
    );
    assert!(defeated.shell.battle_origin.bound_trainer.is_none());
    assert!(defeated.shell.battle_origin.published().is_none());
}

#[test]
fn scripted_trainer_anchor_dispatch_lineage_reset_and_unwitnessed_start_are_rejected() {
    let mut app = scripted_trainer_anchor_dialogue_app();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let candidate = shell
        .battle_origin
        .scripted_trainer_candidate
        .as_ref()
        .unwrap()
        .clone();
    let snapshot = shell.shell.snapshot().unwrap();
    for defect in 0..5 {
        let mut changed = snapshot.clone();
        match defect {
            0 => changed.overworld.tile.x += 1,
            1 => changed.script_events.last_talked_object = Some("VIOLETGYM_YOUNGSTER1".into()),
            2 => {
                changed.visible_object_facings.insert(
                    candidate.evidence.target_identifier.clone(),
                    Direction::Down,
                );
            }
            3 => {
                changed.visible_objects[candidate.evidence.target_index].script =
                    "TrainerBirdKeeperAbe".into()
            }
            4 => {
                changed.visible_objects[candidate.evidence.target_index].object_type =
                    "OBJECTTYPE_TRAINER".into()
            }
            _ => unreachable!(),
        }
        assert!(
            !trainer_candidate_matches_snapshot(&candidate.evidence, &changed),
            "defect {defect}"
        );
    }
    shell
        .shell
        .step_compiled_script_command(
            "VioletGym",
            "VioletGymFalknerScript",
            7,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .unwrap();
    // A genuine loadtrainer result at index 8 still cannot skip the expected
    // dialogue continuation at index 5 and claim the earlier object contact.
    let load = shell
        .shell
        .step_compiled_script_command(
            "VioletGym",
            "VioletGymFalknerScript",
            8,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .unwrap();
    observe_visible_scripted_trainer_step(&mut shell, &load);
    assert!(shell.battle_origin.scripted_trainer_candidate.is_none());
    // Even matching forward lineage must fail closed for world changes that
    // could invalidate the captured terrain or the bounded occupancy cells.
    for command in [
        "changeblock",
        "applymovement",
        "disappear",
        "warp",
        "earthquake",
    ] {
        let mut pending = candidate.clone();
        pending.expected_step.command_index = load.command_index;
        shell.battle_origin.scripted_trainer_candidate = Some(pending);
        let mut changed = load.clone();
        changed.command = command.into();
        observe_visible_scripted_trainer_step(&mut shell, &changed);
        assert!(
            shell.battle_origin.scripted_trainer_candidate.is_none(),
            "{command}"
        );
        assert!(shell.battle_origin.bound_trainer.is_none());
    }
    let start = shell
        .shell
        .step_compiled_script_command(
            "VioletGym",
            "VioletGymFalknerScript",
            9,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .unwrap();
    observe_visible_scripted_trainer_step(&mut shell, &start);
    assert!(shell.battle_origin.bound_trainer.is_none());
    prepare_visible_battle_entry(&mut shell).unwrap();
    let generation = shell.battle_origin.published().unwrap().generation;
    assert!(
        shell.battle_origin.published().unwrap().contact.is_none(),
        "unwitnessed direct start has no object provenance"
    );
    shell.battle_origin.scripted_trainer_candidate = Some(candidate.clone());
    reset_visible_navigation_state(&mut shell);
    assert!(shell.battle_origin.scripted_trainer_candidate.is_none());
    shell.battle_origin.scripted_trainer_candidate = Some(candidate);
    reset_visible_battle_presentation(&mut shell);
    assert!(shell.battle_origin.scripted_trainer_candidate.is_none());
    assert!(shell.battle_origin.published().is_none());
    assert_eq!(shell.battle_origin.generation, generation);
}
