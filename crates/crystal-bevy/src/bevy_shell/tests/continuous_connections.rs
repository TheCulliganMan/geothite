// Exercise the production keyboard dispatcher, including destination callbacks.
#[test]
fn natural_movement_connected_maps_preserve_position_and_held_motion() {
    for (map, tile, key_code, direction) in [
        (
            "Route29",
            TilePosition::new(1, 6),
            KeyCode::ArrowLeft,
            Direction::Left,
        ),
        (
            "CherrygroveCity",
            TilePosition::new(38, 6),
            KeyCode::ArrowRight,
            Direction::Right,
        ),
        (
            "Route28",
            TilePosition::new(1, 16),
            KeyCode::ArrowLeft,
            Direction::Left,
        ),
        (
            "SilverCaveOutside",
            TilePosition::new(38, 34),
            KeyCode::ArrowRight,
            Direction::Right,
        ),
        (
            "Route30",
            TilePosition::new(6, 52),
            KeyCode::ArrowDown,
            Direction::Down,
        ),
        (
            "CherrygroveCity",
            TilePosition::new(16, 1),
            KeyCode::ArrowUp,
            Direction::Up,
        ),
    ] {
        let mut app = app();
        let old = app.world().resource::<BevyRuntimeShell>();
        let shell = initialize_bevy_runtime_shell(
            old.asset_root.clone(),
            old.shell.runtime().clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: old
                    .shell
                    .runtime()
                    .title_new_game_spawn_identifier()
                    .expect("spawn"),
                map_name: map.into(),
                tile_x: tile.x,
                tile_y: tile.y,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                render_test_hour: Some(12),
                ..Default::default()
            },
        )
        .expect("connection fixture");
        app.insert_resource(shell);
        let (dx, dy) = direction.delta();
        let vector = Vec2::new(f32::from(dx), f32::from(dy));
        let lateral = Vec2::new(-vector.y, vector.x) * 0.15;
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            shell.natural_movement.enabled = true;
            shell.natural_movement.anchor = Some((map.into(), tile));
            shell.natural_movement.position.offset = lateral.to_array();
        }
        let mut origin = Vec2::ZERO;
        let mut previous_map = map.to_owned();
        let mut previous_tile = tile;
        let mut previous_position = Vec2::new(f32::from(tile.x), f32::from(tile.y)) + lateral;
        let mut crossings = 0;
        let mut after_crossing = 0;
        key(&mut app, key_code, true);
        for tick in 0..48 {
            app.update();
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None, "{map}, tick {tick}");
            let world = shell.shell.session().overworld();
            let current = world.player.tile;
            if world.map.name != previous_map {
                crossings += 1;
                origin += Vec2::new(
                    f32::from(previous_tile.x + dx - current.x),
                    f32::from(previous_tile.y + dy - current.y),
                );
                assert_eq!(
                    shell.player_walk_frame_ticks, 0,
                    "{map}: a connected free step must not start a centered tile tween"
                );
                assert!(
                    shell.pending_overworld_step_boundary.is_none(),
                    "{map}: arrival callbacks must settle without a synthetic landing wait"
                );
                assert!(
                    shell.screen_fade.is_none(),
                    "{map}: connections must not fade"
                );
            }
            let position = origin
                + Vec2::new(f32::from(current.x), f32::from(current.y))
                + Vec2::from_array(shell.natural_movement.position.offset);
            let advance = position - previous_position;
            assert!(
                (advance - vector * 0.125).length() < 0.0001,
                "{map}, tick {tick}: held movement must advance 0.125 cells, got {advance:?}; position {position:?}"
            );
            previous_position = position;
            previous_map.clone_from(&world.map.name);
            previous_tile = current;
            if crossings > 0 {
                after_crossing += 1;
                if after_crossing == 6 {
                    break;
                }
            }
        }
        assert_eq!(
            crossings, 1,
            "{map}: keyboard must execute the real connection"
        );
        assert_eq!(
            after_crossing, 6,
            "{map}: held input must continue after arrival"
        );
    }
}
