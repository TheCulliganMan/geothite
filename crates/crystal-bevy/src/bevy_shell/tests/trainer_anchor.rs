fn trainer_anchor_fresh_shell() -> BevyRuntimeShell {
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
            tile_x: 5,
            tile_y: 11,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            quick_save_path: None,
            ..Default::default()
        },
    )
    .unwrap();
    let mut shell = prepare_gym_encounter_preview(shell).unwrap();
    shell.shell.set_runtime_journal_enabled(true);
    shell
}

fn trainer_anchor_seen_text_app() -> App {
    let shell = trainer_anchor_fresh_shell();
    let initial = shell.shell.snapshot().unwrap();
    assert_eq!(initial.overworld.tile, TilePosition::new(5, 11));
    assert!(initial.battle.is_none());
    assert_eq!(initial.party.slots.len(), 1);
    assert_eq!(initial.party.slots[0].pokemon.species.id, "CYNDAQUIL");
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
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowUp);
    let sight = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .last_frame()
        .and_then(|frame| frame.trainer_sight.as_ref())
        .expect("ordinary Up must trigger Abe sight");
    assert_eq!(sight.target_tile, TilePosition::new(2, 10));
    assert_eq!(sight.player_tile, TilePosition::new(5, 10));
    assert_eq!(sight.script, "TrainerBirdKeeperAbe");
    for _ in 0..240 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        assert!(
            shell.battle_origin.published().is_none(),
            "seen text is still the field"
        );
        assert!(shell.shell.snapshot().unwrap().battle.is_none());
        if shell.pending_trainer_intro.is_some() {
            // Publication observes the preceding completed renderer frame.
            app.update();
            app.update();
            return app;
        }
    }
    panic!("ordinary trainer approach never reached seen text");
}

#[test]
fn trainer_anchor_real_abe_sight_preserves_settled_contact_and_read_only_publication() {
    use bevy::ecs::system::RunSystemOnce;
    let mut app = trainer_anchor_seen_text_app();
    let shell = app.world().resource::<BevyRuntimeShell>();
    let candidate = shell
        .battle_origin
        .trainer_candidate
        .as_ref()
        .expect("completed trainer-table intro stages contact")
        .clone();
    assert_eq!(candidate.source.map_name, "VioletGym");
    assert_eq!(candidate.source.tile, TilePosition::new(5, 10));
    assert_eq!(candidate.source.facing, Direction::Left);
    assert_eq!(candidate.target_tile, TilePosition::new(4, 10));
    assert_eq!(candidate.target_facing, Direction::Right);
    assert_eq!(candidate.target_identifier, "VIOLETGYM_YOUNGSTER2");
    assert_eq!(candidate.intro.source_script, "TrainerBirdKeeperAbe");
    assert_eq!(candidate.intro.command_index, 0);
    assert_eq!(
        candidate.placement.presentation_core_tile,
        IVec2::new(5, 12)
    );
    assert!(
        !candidate
            .placement
            .walkable_core_tiles
            .contains(&IVec2::new(4, 10))
    );
    assert_eq!(candidate.map_size, UVec2::new(10, 16));
    assert!(
        candidate.anchors.is_some(),
        "completed scene must witness player, trainer and terrain"
    );
    assert!(candidate.witnessed_actor.is_some());
    assert!(candidate.witnessed_foot.is_some());
    assert!(
        app.world()
            .resource::<crystal_render_api::VisualWorldFrame>()
            .active
    );
    assert!(
        app.world()
            .resource::<crystal_render_api::BattleLocationFrame>()
            .location
            .is_none()
    );
    for _ in 0..600 {
        if app
            .world()
            .resource::<BevyRuntimeShell>()
            .battle_origin
            .bound_trainer
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
        .clone()
        .expect("actual successful trainer start binds the frozen scene");
    let shell = app.world().resource::<BevyRuntimeShell>();
    let origin = shell.battle_origin.published().unwrap().clone();
    assert_eq!(origin.kind, BattleOriginKind::Trainer);
    assert_eq!(origin.source, candidate.source);
    assert_eq!(location.generation, origin.generation);
    assert_eq!(location.source.core_tile, IVec2::new(5, 10));
    assert_eq!(location.target.core_tile(), IVec2::new(4, 10));
    let crystal_render_api::VisualBattleTarget::Trainer {
        contact,
        presentation,
    } = &location.target
    else {
        panic!("checked trainer-table contact required");
    };
    assert_eq!(
        contact.battle_source_script.as_ref(),
        "TrainerBirdKeeperAbe"
    );
    assert_eq!(contact.trainer_command_index, 0);
    assert_eq!(contact.trainer_class.as_ref(), "BIRD_KEEPER");
    assert_eq!(contact.trainer_id.as_ref(), candidate.request.trainer_id);
    assert_eq!(contact.facing, IVec2::X);
    assert_eq!(contact.witnessed_foot, candidate.witnessed_foot);
    assert_eq!(presentation.presentation_core_tile, IVec2::new(5, 12));
    let anchors = location.anchors.as_ref().unwrap();
    assert!(Arc::ptr_eq(anchors, candidate.anchors.as_ref().unwrap()));
    assert_eq!(
        anchors.target_actor, None,
        "the trainer's Spearow has no field actor"
    );
    assert_ne!(Some(anchors.target_foot), contact.witnessed_foot);
    assert!(
        matches!(&origin.contact, Some(BattleOriginContact::TrainerObjectTarget {
        tile, object_identifier, trainer_command_index: 0, ..
    }) if *tile == TilePosition::new(4,10) && object_identifier == "VIOLETGYM_YOUNGSTER2")
    );
    let battle = shell.shell.snapshot().unwrap().battle.unwrap();
    assert_eq!(battle.enemy_pokemon.species.id, "SPEAROW");
    assert_eq!(battle.enemy_pokemon.level, 9);
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
        "publication preserves game state, divider and RNG"
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
    assert!(!shell.battle_messages.is_empty());
    reset_visible_battle_exit_state(&mut shell);
    assert!(
        shell.battle_origin.bound_trainer.is_some(),
        "terminal narration retains its source"
    );
    shell.battle_messages.clear();
    reset_visible_battle_exit_state(&mut shell);
    assert!(shell.battle_origin.bound_trainer.is_none());
    assert!(shell.battle_origin.trainer_candidate.is_none());
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
fn trainer_anchor_rejects_cold_stale_missing_or_wrong_actor_evidence() {
    let mut app = trainer_anchor_seen_text_app();
    let mut prototype = app
        .world()
        .resource::<BevyRuntimeShell>()
        .battle_origin
        .trainer_candidate
        .as_ref()
        .unwrap()
        .clone();
    prototype.capture_attempted = false;
    prototype.warm_checked = false;
    prototype.anchors = None;
    prototype.witnessed_actor = None;
    prototype.witnessed_foot = None;
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
        .map(|(index, id, source)| (*index, id.as_deref(), source.as_str()))
        .collect::<Vec<_>>();
    let actor = crystal_render_api::VisualActorId::Object(prototype.target_index as u32);
    for defect in 0..10 {
        let mut candidate = prototype.clone();
        let mut bad = frame.clone();
        match defect {
            0 => bad.active = false,
            1 => bad.map_id = Arc::from("OtherMap"),
            2 => bad.terrain_revision += 1,
            3 => bad.source_map_size_core_tiles = None,
            4 => bad.actors.retain(|entry| entry.id != actor),
            5 => {
                bad.actors
                    .iter_mut()
                    .find(|entry| entry.id == actor)
                    .unwrap()
                    .center
                    .x -= 16.0
            }
            6 => {
                bad.actors
                    .iter_mut()
                    .find(|entry| entry.id == actor)
                    .unwrap()
                    .facing = Some(Vec2::NEG_X)
            }
            7 => {
                bad.actors
                    .iter_mut()
                    .find(|entry| entry.id == actor)
                    .unwrap()
                    .source_id = Arc::from("wrong_trainer")
            }
            8 => bad
                .actors
                .retain(|entry| entry.id != crystal_render_api::VisualActorId::Player),
            9 => {}
            _ => unreachable!(),
        }
        freeze_visible_trainer_anchors(
            &mut candidate,
            &rendered,
            (defect != 9).then_some(&bad),
            &objects,
        );
        assert!(candidate.capture_attempted, "defect {defect}");
        assert!(candidate.anchors.is_none(), "defect {defect}");
        assert!(candidate.witnessed_actor.is_none() && candidate.witnessed_foot.is_none());
        freeze_visible_trainer_anchors(&mut candidate, &rendered, Some(&frame), &objects);
        assert!(
            candidate.anchors.is_none(),
            "later scene cannot repair defect {defect}"
        );
    }
    let revision = rendered.snapshot_revision;
    rendered.snapshot_revision = Some(prototype.minimum_snapshot_revision.wrapping_sub(1));
    let mut pending = prototype.clone();
    freeze_visible_trainer_anchors(&mut pending, &rendered, Some(&frame), &objects);
    assert!(
        !pending.capture_attempted,
        "coherent approach can await completed intro publication"
    );
    assert!(pending.anchors.is_none());
    rendered.snapshot_revision = revision;
    freeze_visible_trainer_anchors(&mut pending, &rendered, Some(&frame), &objects);
    assert!(pending.anchors.is_some());
    let mut missing_roster = prototype;
    freeze_visible_trainer_anchors(&mut missing_roster, &rendered, Some(&frame), &[]);
    assert!(missing_roster.anchors.is_none());
}

#[test]
fn trainer_anchor_matches_successful_identity_and_rejects_changed_contacts() {
    let mut app = trainer_anchor_seen_text_app();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let candidate = shell
        .battle_origin
        .trainer_candidate
        .as_ref()
        .unwrap()
        .clone();
    let snapshot = shell.shell.snapshot().unwrap();
    for defect in 0..7 {
        let mut bad = snapshot.clone();
        match defect {
            0 => bad.overworld.tile.x += 1,
            1 => bad.overworld.map_name = "OtherMap".into(),
            2 => bad.overworld.facing = Direction::Right,
            3 => bad
                .visible_object_runtime_tiles
                .insert(
                    candidate.target_identifier.clone(),
                    TilePosition::new(2, 10),
                )
                .map(|_| ())
                .unwrap_or(()),
            4 => bad
                .visible_object_facings
                .insert(candidate.target_identifier.clone(), Direction::Left)
                .map(|_| ())
                .unwrap_or(()),
            5 => bad.visible_objects[candidate.target_index].script = "TrainerBirdKeeperRod".into(),
            6 => bad.script_events.last_talked_object = Some("VIOLETGYM_YOUNGSTER1".into()),
            _ => unreachable!(),
        }
        assert!(
            !trainer_candidate_matches_snapshot(&candidate, &bad),
            "defect {defect}"
        );
    }
    let started = shell
        .shell
        .start_scripted_trainer_battle("VioletGym", "TrainerBirdKeeperAbe", 0)
        .unwrap();
    let crate::TrainerBattleStartStatus::Started(start) = &started else {
        panic!("unbeaten Abe");
    };
    let battle = shell.shell.snapshot().unwrap().battle.unwrap();
    assert!(trainer_candidate_matches_start(
        &candidate,
        &candidate.intro,
        start,
        &battle
    ));
    for defect in 0..5 {
        let mut altered = start.clone();
        match defect {
            0 => altered.trainer_id = "ROD".into(),
            1 => altered.source_script = "TrainerBirdKeeperRod".into(),
            2 => altered.event_flag = "EVENT_BEAT_BIRD_KEEPER_ROD".into(),
            3 => altered.trainer_class = "FALKNER".into(),
            4 => altered.battle_type = "BATTLETYPE_NORMAL".into(),
            _ => unreachable!(),
        }
        assert!(!trainer_candidate_matches_start(
            &candidate,
            &candidate.intro,
            &altered,
            &battle
        ));
    }
    let mut wrong_intro = candidate.intro.clone();
    wrong_intro.command_index = 9;
    assert!(!trainer_candidate_matches_start(
        &candidate,
        &wrong_intro,
        start,
        &battle
    ));
    let rejected = crate::TrainerBattleStartStatus::AlreadyDefeated {
        event_flag: candidate.request.event_flag.clone(),
        callback: candidate.request.callback.clone(),
    };
    bind_visible_trainer_encounter(&mut shell, &candidate.intro, &rejected);
    assert!(shell.battle_origin.trainer_candidate.is_none());
    assert!(shell.battle_origin.bound_trainer.is_none());
    assert!(shell.battle_origin.published().is_none());
    shell.battle_origin.trainer_candidate = Some(candidate.clone());
    shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(&candidate.request.event_flag, true)
        .unwrap();
    assert!(!trainer_candidate_unbeaten(&shell, &candidate));
    bind_visible_trainer_encounter(&mut shell, &candidate.intro, &started);
    assert!(shell.battle_origin.published().is_none());
}

#[test]
fn trainer_anchor_direct_start_and_reset_cannot_reuse_seen_text_evidence() {
    let mut app = trainer_anchor_seen_text_app();
    let mut shell = app
        .world_mut()
        .remove_resource::<BevyRuntimeShell>()
        .unwrap();
    let candidate = shell
        .battle_origin
        .trainer_candidate
        .as_ref()
        .unwrap()
        .clone();
    reset_visible_navigation_state(&mut shell);
    assert!(shell.battle_origin.trainer_candidate.is_none());
    shell
        .shell
        .start_scripted_trainer_battle("VioletGym", "TrainerBirdKeeperAbe", 0)
        .unwrap();
    prepare_visible_battle_entry(&mut shell).unwrap();
    let origin = shell.battle_origin.published().unwrap().clone();
    assert_eq!(origin.kind, BattleOriginKind::Trainer);
    assert!(
        origin.contact.is_none(),
        "a direct trainer start has no sight witness"
    );
    shell.battle_origin.trainer_candidate = Some(candidate);
    reset_visible_battle_presentation(&mut shell);
    assert!(shell.battle_origin.trainer_candidate.is_none());
    assert!(shell.battle_origin.bound_trainer.is_none());
    assert!(shell.battle_origin.published().is_none());
    assert_eq!(
        shell.battle_origin.generation, origin.generation,
        "reload preserves serial monotonicity"
    );
}
