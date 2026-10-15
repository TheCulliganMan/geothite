#[test]
fn day_cycle_publishes_live_clock_only_for_automatic_outdoor_palettes() {
    use bevy::ecs::system::RunSystemOnce;
    use crystal_render_api::{VisualWorldClock, VisualWorldFrame};
    for (map, x, y, expected) in [
        ("NewBarkTown", 8, 8, Some(1080)),
        ("PlayersHouse2F", 3, 3, None),
        ("UnionCave1F", 9, 26, None),
    ] {
        let root = AssetRoot::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        );
        let runtime = workspace_desktop_runtime(&root);
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let mut shell = initialize_bevy_runtime_shell(
            root,
            runtime,
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier,
                map_name: map.into(),
                tile_x: x,
                tile_y: y,
            },
            BevyShellConfig {
                render_test_hour: Some(18),
                smoke_player_name: Some("TEST".into()),
                ..Default::default()
            },
        )
        .unwrap();
        complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
        settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
        let mut app = integrated_shell_test_app(shell);
        app.insert_resource(NativeRtcSource::fixed(RuntimeRtcSample {
            date: GameDate::new(2000, 1, 1),
            hour: 18,
            minute: 0,
            second: 0,
        }));
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldClock>()
            .add_systems(Update, publish_visual_world_frame.after(render_playfield));
        app.update();
        app.update();
        assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
        assert_eq!(
            app.world().resource::<VisualWorldClock>().minute_of_day,
            expected,
            "{map}"
        );
        if expected.is_some() {
            // A minute change must publish while retaining exactly the same
            // classic map texture and terrain: no movement or palette rebuild.
            let texture = app
                .world()
                .resource::<RenderedViewport>()
                .map_texture
                .clone();
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                shell
                    .shell
                    .update_clock_from_datetime(GameDate::new(2000, 1, 1), 18, 17, 0)
                    .unwrap();
            }
            app.world_mut()
                .run_system_once(publish_visual_world_frame)
                .unwrap();
            assert_eq!(
                app.world().resource::<VisualWorldClock>().minute_of_day,
                Some(1097)
            );
            assert_eq!(
                app.world().resource::<RenderedViewport>().map_texture,
                texture
            );
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                shell
                    .shell
                    .update_clock_from_datetime(GameDate::new(2000, 1, 1), 18, 17, 30)
                    .unwrap();
            }
            app.world_mut()
                .run_system_once(publish_visual_world_frame)
                .unwrap();
            assert_eq!(
                app.world().resource::<VisualWorldClock>().seconds_of_day(),
                Some(1097 * 60 + 30)
            );
            assert_eq!(
                app.world().resource::<RenderedViewport>().map_texture,
                texture
            );
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                let (_, snapshot) = shell.cached_snapshot.as_mut().unwrap();
                Arc::make_mut(snapshot).overworld.map_name = "PlayersHouse2F".into();
            }
            app.world_mut()
                .run_system_once(publish_visual_world_frame)
                .unwrap();
            assert_eq!(
                app.world().resource::<VisualWorldClock>().minute_of_day,
                Some(1097),
                "a pending warp must keep the retained field light"
            );
            app.world_mut()
                .resource_mut::<RenderedViewport>()
                .visual_clock_enabled = false;
            app.world_mut()
                .run_system_once(publish_visual_world_frame)
                .unwrap();
            assert_eq!(
                app.world().resource::<VisualWorldClock>().minute_of_day,
                None
            );
        }
    }
}
