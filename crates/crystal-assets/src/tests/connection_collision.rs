fn connection_collision_fixture(direction: &str) -> GameDataSet {
    let mut source = test_map_module("Source", "SOURCE", None);
    source.attributes.width = 1;
    source.attributes.height = 1;
    source.blocks = vec![0];
    source.attributes.connections = vec![crystal_core::map::MapConnection {
        direction: direction.into(),
        target_map: "Neighbor".into(),
        offset: 0,
    }];
    let mut neighbor = test_map_module("Neighbor", "NEIGHBOR", None);
    neighbor.attributes.width = 1;
    neighbor.attributes.height = 1;
    neighbor.blocks = vec![1];
    GameDataSet {
        map_attributes: BTreeMap::from([
            ("Source".into(), source.attributes.clone()),
            ("Neighbor".into(), neighbor.attributes.clone()),
        ]),
        maps: map_payload(vec![source, neighbor]),
        tilesets: BTreeMap::from([(
            "johto".into(),
            TilesetDefinition {
                collision: BTreeMap::from([
                    ("0".into(), vec!["FLOOR".into(); 4]),
                    ("1".into(), vec!["WALL".into(); 4]),
                    ("2".into(), vec!["WATER".into(); 4]),
                    ("3".into(), vec!["LEFT_WALL".into(); 4]),
                ]),
                palette_map: vec![0; 4],
            },
        )]),
        ..GameDataSet::default()
    }
}

#[test]
fn connection_collision_checks_all_borders_and_saved_destination_edits() {
    for (name, direction, tile) in [
        ("east", Direction::Right, TilePosition::new(1, 0)),
        ("west", Direction::Left, TilePosition::new(0, 0)),
        ("north", Direction::Up, TilePosition::new(0, 0)),
        ("south", Direction::Down, TilePosition::new(0, 1)),
    ] {
        let data = connection_collision_fixture(name);
        let mut session = data.overworld_session("Source", tile, 0).expect("source");
        session.player.facing = direction;
        let mut state = GameState::default();
        assert!(
            data.blocked_connection_edge_target(&state, &session, direction, false)
                .expect("neighbor wall")
                .is_some(),
            "{name}"
        );
        state
            .map_block_overrides
            .insert("Neighbor".into(), BTreeMap::from([((0, 0), 0)]));
        assert!(
            data.blocked_connection_edge_target(&state, &session, direction, false)
                .expect("saved open floor")
                .is_none(),
            "{name}"
        );
        state
            .map_block_overrides
            .insert("Neighbor".into(), BTreeMap::from([((0, 0), 1)]));
        assert!(
            data.blocked_connection_edge_target(&state, &session, direction, false)
                .expect("saved closed wall")
                .is_some(),
            "{name}"
        );
        state
            .map_block_overrides
            .insert("Neighbor".into(), BTreeMap::from([((1, 0), 0)]));
        assert!(
            data.blocked_connection_edge_target(&state, &session, direction, false)
                .is_err(),
            "malformed edits must fail before indexing"
        );
    }
}

#[test]
fn connection_collision_preserves_surf_and_directional_walls() {
    let data = connection_collision_fixture("east");
    let mut session = data
        .overworld_session("Source", TilePosition::new(1, 0), 0)
        .expect("source");
    session.player.facing = Direction::Right;
    let mut state = GameState::default();
    state
        .map_block_overrides
        .insert("Neighbor".into(), BTreeMap::from([((0, 0), 2)]));
    assert!(
        data.blocked_connection_edge_target(&state, &session, Direction::Right, false)
            .expect("walking to water")
            .is_some()
    );
    session.player.mode = MovementMode::Surf;
    assert!(
        data.blocked_connection_edge_target(&state, &session, Direction::Right, false)
            .expect("surfing to water")
            .is_none()
    );
    state
        .map_block_overrides
        .insert("Neighbor".into(), BTreeMap::from([((0, 0), 3)]));
    assert!(
        data.blocked_connection_edge_target(&state, &session, Direction::Right, false)
            .expect("directional wall applies while surfing")
            .is_some()
    );
    session.player.mode = MovementMode::Normal;
    assert!(
        data.blocked_connection_edge_target(&state, &session, Direction::Right, false)
            .expect("directional wall applies while walking")
            .is_some()
    );
}

#[test]
fn connection_collision_checks_forced_turns_without_skipping_normal_turns() {
    let data = connection_collision_fixture("east");
    let mut session = data
        .overworld_session("Source", TilePosition::new(1, 0), 0)
        .expect("source");
    session.player.facing = Direction::Down;
    let state = GameState::default();
    assert!(
        data.blocked_connection_edge_target(&state, &session, Direction::Right, false)
            .expect("ordinary input only turns first")
            .is_none()
    );
    assert!(
        data.blocked_connection_edge_target(&state, &session, Direction::Right, true)
            .expect("continuous or forced input checks its actual landing")
            .is_some()
    );
}

#[test]
fn connection_collision_input_blocks_grid_continuous_ice_and_walk_tiles() {
    for (block, continuous, facing) in [
        (0, false, Direction::Right),
        (0, true, Direction::Down),
        (4, false, Direction::Down),
        (5, false, Direction::Down),
    ] {
        let mut data = connection_collision_fixture("east");
        let tileset = data.tilesets.get_mut("johto").expect("test tileset");
        tileset.collision.insert("4".into(), vec!["ICE".into(); 4]);
        tileset
            .collision
            .insert("5".into(), vec!["WALK_RIGHT".into(); 4]);
        tileset.palette_map.extend([0, 0]);
        data.maps.get_mut("Source").expect("source").blocks[0] = block;
        let mut session = data
            .overworld_session("Source", TilePosition::new(1, 0), 100)
            .expect("passable source edge");
        session.player.facing = facing;
        session.last_step_direction = Some(Direction::Right);
        let mut state = GameState::default();
        let frame = data
            .apply_overworld_input_with_movement(
                &mut state,
                &mut session,
                [GameButton::Right],
                &BTreeSet::new(),
                &mut ReplayDivider::new([]),
                continuous,
            )
            .expect("native input must reject the neighbor wall");
        assert!(
            matches!(frame.movement, Some(StepOutcome::Blocked { .. })),
            "block {block}, continuous {continuous}: {:?}",
            frame.movement
        );
        assert!(frame.connection.is_none());
        assert_eq!(session.map.name, "Source");
        assert_eq!(session.player.tile, TilePosition::new(1, 0));
        if block == 4 {
            assert_eq!(
                session.last_step_direction, None,
                "ice must stop at a border wall"
            );
        }
    }
}
