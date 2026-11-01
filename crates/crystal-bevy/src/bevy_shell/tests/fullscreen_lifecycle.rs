// Presentation-only phase fixtures isolate deferred ECS lifecycle. Actual
// service entry and continuation are covered by the controller service tests.
#[test]
fn fullscreen_field_returns_preserve_layers_and_reuse_the_lcd_image() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    {
        let state = shell.shell.session_mut().state_mut();
        let mon = state.storage.party.pokemon[0].as_mut().unwrap();
        mon.original_trainer_name = state.player_name.clone();
        mon.original_trainer_id = state.player_id;
        state.sync_party_from_storage();
    }
    let mut app = integrated_shell_test_app(shell);
    app.insert_resource(bevy::ecs::error::FallbackErrorHandler(bevy::ecs::error::panic));
    app.update();
    let field = retained_map_surface_pair(app.world_mut());
    let mut texture = None;
    for owner in ["party", "naming", "rater", "naming"] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            if owner != "naming" {
                if owner == "rater" { shell.pending_script_party_selection = Some(PendingScriptPartySelection::NameRater); }
                open_visible_party_menu(&mut shell).unwrap();
            } else {
                shell.pending_name_input = Some(PendingNameInput {
                    label:"POKéMON'S NAME?".into(), value:String::new(), max_length:10,
                    cursor_column:0, cursor_row:0, case:NameInputCase::Upper,
                });
            }
            mark_runtime_snapshot_dirty(&mut shell);
        }
        app.update();
        let surface = retained_fullscreen_surface(app.world_mut());
        assert_retained_fullscreen_surface_nonblack(app.world(), &surface, owner);
        if let Some(texture) = &texture { assert_eq!(texture, &surface.texture, "reuse the owned LCD image allocation"); }
        texture = Some(surface.texture.clone());
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &surface);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            if owner == "rater" {
                resolve_visible_script_party_selection(&mut shell, Some(0)).unwrap();
                assert!(shell.special_boundary.is_some() && shell.pc_confirmation.is_some());
            } else {
                shell.pending_name_input = None;
                shell.party_menu_open = false;
            }
            mark_runtime_snapshot_dirty(&mut shell);
        }
        // Both map layers already exist. Retire the presenter and perform
        // the full field refresh in one update; stale query results may not
        // queue later removals or inserts against the retired entity.
        app.update();
        assert!(app.world().get_entity(surface.entity).is_err(), "retired presenter must actually be removed");
        assert!(app.world().resource::<RenderedTilesetArt>().presented_fullscreen_entity.is_none());
        assert_eq!(retained_map_surface_pair(app.world_mut()), field, "return to the existing field layers");
        assert_main_camera_presents_one_logical_lcd(app.world_mut());
        assert!(app.world().resource::<BevyRuntimeShell>().last_error.is_none());
        if owner == "rater" {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            shell.pending_script_party_selection = None;
            shell.pc_confirmation = None;
            shell.pc_notice = None;
            shell.yes_no_cursor = None;
            shell.special_boundary = None;
            shell.special_boundary_queue.clear();
            mark_runtime_snapshot_dirty(&mut shell);
        }
    }
}

#[test]
fn fullscreen_cleanup_ignores_retired_presenters_and_clears_live_ownership() {
    for clear_visible in [false, true] {
        let mut world = World::new();
        world.insert_resource(bevy::ecs::error::FallbackErrorHandler(bevy::ecs::error::panic));
        let retired = world.spawn((TitleScreenMarker, VisibleIntroSurface)).id();
        let live = world.spawn((TitleScreenMarker, VisibleIntroSurface)).id();
        let snapshot = vec![retired, live];
        let mut art = RenderedTilesetArt::default();
        art.presented_fullscreen_entity = Some(retired);
        let queued = std::collections::HashSet::from([retired]);
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        remove_presented_fullscreen_entity(&mut commands, &mut art);
        for entity in snapshot {
            clear_intro_surface_ownership(&mut commands, entity, &queued, clear_visible);
        }
        queue.apply(&mut world);
        assert!(art.presented_fullscreen_entity.is_none());
        assert!(world.get_entity(retired).is_err());
        assert!(world.get::<TitleScreenMarker>(live).is_none());
        assert_eq!(world.get::<VisibleIntroSurface>(live).is_none(), clear_visible);
    }
}
