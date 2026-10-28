use super::*;
use crate::map::MapAttributes;
use crate::world::collision::{MetatileCollision, TilesetCollision};
use crate::world::map::{OverworldMapData, TilePosition};
use crate::world::movement::{
    MovementMode, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_step_with_occupied_tiles,
};

fn crossing() -> (OverworldMapData, TilesetCollision) {
    let attributes = MapAttributes {
        tileset_name: "bridge_fixture".into(),
        border_block: 0,
        width: 3,
        height: 5,
        connections: Vec::new(),
        time_of_day: None,
        phone_service: 0,
        phone_flag: false,
        environment: None,
        location: None,
        music: None,
        palette: None,
        fishing_group: None,
        map_constant: None,
        map_group_constant: None,
        blocks_label: None,
        map_scripts_label: None,
        map_events_label: None,
        connection_flags: None,
    };
    // A road runs west/east across the middle row, above north/south water.
    let map = OverworldMapData::from_attributes(
        "BridgeFixture",
        &attributes,
        vec![0, 1, 0, 0, 3, 0, 0, 2, 0, 0, 3, 0, 0, 1, 0],
    );
    let tileset = TilesetCollision {
        metatiles: [
            permissions::FLOOR,
            permissions::WATER,
            NORTH_SOUTH_DECK,
            NORTH_SOUTH_RAIL,
        ]
        .map(|permission| MetatileCollision {
            collision: [permission; 4],
        })
        .to_vec(),
    };
    (map, tileset)
}

fn step(
    player: &mut PlayerMovementState,
    direction: Direction,
    map: &OverworldMapData,
    tileset: &TilesetCollision,
    occupied: &[OccupiedTile],
) -> StepOutcome {
    attempt_step_with_occupied_tiles(
        player,
        direction,
        map,
        tileset,
        StepOptions {
            force_step_after_turn: true,
            ..StepOptions::default()
        },
        occupied,
    )
}

#[test]
fn surfers_cross_both_rails_and_deck_without_dismount_or_sideways_climb() {
    let (map, tileset) = crossing();
    let mut player =
        PlayerMovementState::new(TilePosition::new(2, 0)).with_mode(MovementMode::Surf);
    for y in 1..10 {
        assert!(matches!(
            step(&mut player, Direction::Down, &map, &tileset, &[]),
            StepOutcome::Moved { .. }
        ));
        assert_eq!(player.tile, TilePosition::new(2, y));
        assert_eq!(player.mode, MovementMode::Surf);
        if (2..=7).contains(&y) {
            assert!(matches!(
                step(&mut player, Direction::Left, &map, &tileset, &[]),
                StepOutcome::Blocked { .. }
            ));
            assert_eq!(player.tile, TilePosition::new(2, y));
        }
    }
    for y in (0..9).rev() {
        assert!(matches!(
            step(&mut player, Direction::Up, &map, &tileset, &[]),
            StepOutcome::Moved { .. }
        ));
        assert_eq!(player.tile, TilePosition::new(2, y));
        assert_eq!(player.mode, MovementMode::Surf);
    }
    // Genuine riverbank access retains the original Surf-to-land behavior.
    assert!(matches!(
        step(&mut player, Direction::Left, &map, &tileset, &[]),
        StepOutcome::Moved { .. }
    ));
    assert_eq!(player.mode, MovementMode::Normal);
}

#[test]
fn walkers_keep_the_road_and_rail_collision_and_upper_resident() {
    let (map, tileset) = crossing();
    let mut player = PlayerMovementState::new(TilePosition::new(0, 4));
    for x in 1..6 {
        assert!(matches!(
            step(&mut player, Direction::Right, &map, &tileset, &[]),
            StepOutcome::Moved { .. }
        ));
        assert_eq!(player.tile, TilePosition::new(x, 4));
        assert_eq!(player.mode, MovementMode::Normal);
        if (2..=3).contains(&x) {
            assert!(matches!(
                step(&mut player, Direction::Up, &map, &tileset, &[]),
                StepOutcome::Blocked { .. }
            ));
        }
    }
    let mut player = PlayerMovementState::new(TilePosition::new(1, 4));
    let resident = [OccupiedTile {
        tile: TilePosition::new(2, 4),
        object_identifier: Some("Upper".into()),
    }];
    assert!(matches!(
        step(&mut player, Direction::Right, &map, &tileset, &resident),
        StepOutcome::BlockedByObject { .. }
    ));
    assert_eq!(player.tile, TilePosition::new(1, 4));
}

#[test]
fn east_west_underpasses_rotate_the_water_axis_and_other_permissions_are_exact() {
    for permission in [EAST_WEST_DECK, EAST_WEST_RAIL] {
        assert_eq!(
            can_enter(permission, Direction::Left, PlayerTraversalState::Surf),
            Some(true)
        );
        assert_eq!(
            can_enter(permission, Direction::Right, PlayerTraversalState::Surf),
            Some(true)
        );
        assert_eq!(
            can_enter(permission, Direction::Up, PlayerTraversalState::Surf),
            Some(false)
        );
        assert!(blocks_exit(
            permission,
            Direction::Down,
            PlayerTraversalState::Surf
        ));
    }
    for permission in 0..=u8::MAX {
        if is_bridge(permission) {
            continue;
        }
        for traversal in [PlayerTraversalState::Walk, PlayerTraversalState::Surf] {
            assert_eq!(layer_permission(permission, traversal), permission);
            assert_eq!(can_enter(permission, Direction::Down, traversal), None);
            assert!(!blocks_exit(permission, Direction::Down, traversal));
        }
    }
}

fn resident(swimming: bool) -> crate::map::ObjectEvent {
    crate::map::ObjectEvent {
        sprite: "SPRITE_TEACHER".into(),
        sprite_has_facings: true,
        x: 2,
        y: 4,
        spritemovedata: if swimming {
            "SPRITEMOVEDATA_SWIM_WANDER"
        } else {
            "SPRITEMOVEDATA_STILL"
        }
        .into(),
        move_range_x: 0,
        move_range_y: 0,
        hram_x: -1,
        hram_y: -1,
        pal: 0,
        object_type: "OBJECTTYPE_TRAINER".into(),
        radius: 4,
        script: "BridgeResidentScript".into(),
        label: None,
        event_flag: "-1".into(),
        object_identifier: Some("BridgeResident".into()),
        sightline_direction_override: Some("UP".into()),
    }
}

#[test]
fn session_keeps_resident_collision_and_interactions_on_the_correct_bridge_layer() {
    use crate::map::MapEvents;
    use crate::world::session::OverworldSession;

    for swimming in [false, true] {
        let (map, tileset) = crossing();
        let mut session = OverworldSession::with_events_and_objects(
            map,
            MapEvents::default(),
            vec![resident(swimming)],
            tileset,
            TilePosition::new(2, 3),
        );
        session.player.mode = MovementMode::Surf;
        session.player.facing = Direction::Down;
        session
            .finish_map_object_setup()
            .expect("bridge residents load");
        assert_eq!(
            session.occupied_tiles().len(),
            1,
            "public scene occupancy retains every resident"
        );
        // A stationary resident above the player cannot talk or spot them.
        // Swimming objects remain collision owners in the lower plane.
        if !swimming {
            assert_eq!(
                session.check_interaction_checked(1).expect("bridge A"),
                None
            );
            assert_eq!(
                session.check_trainer_sight_checked().expect("bridge sight"),
                None
            );
        }
        let result = session.step(
            Direction::Down,
            StepOptions {
                force_step_after_turn: true,
                ..StepOptions::default()
            },
        );
        assert_eq!(
            matches!(result, StepOutcome::BlockedByObject { .. }),
            swimming
        );
        assert_eq!(session.player.mode, MovementMode::Surf);
    }
    let (map, tileset) = crossing();
    let mut session = OverworldSession::with_events_and_objects(
        map,
        MapEvents::default(),
        vec![resident(false)],
        tileset,
        TilePosition::new(1, 4),
    );
    session.player.facing = Direction::Right;
    session
        .finish_map_object_setup()
        .expect("upper resident loads");
    assert!(
        session
            .check_interaction_checked(1)
            .expect("upper deck A")
            .is_some()
    );
    assert!(matches!(
        session.step(
            Direction::Right,
            StepOptions {
                force_step_after_turn: true,
                ..StepOptions::default()
            }
        ),
        StepOutcome::BlockedByObject { .. }
    ));
}

#[test]
fn overlapping_bridge_residents_choose_the_first_same_plane_interaction() {
    use crate::map::MapEvents;
    use crate::world::session::{OverworldInteractionTarget, OverworldSession};

    for mode in [
        MovementMode::Normal,
        MovementMode::Surf,
        MovementMode::SurfPika,
    ] {
        let (map, bank) = crossing();
        let mut upper = resident(false);
        upper.x = 3;
        upper.script = "UpperScript".into();
        upper.object_identifier = Some("Upper".into());
        let mut lower = resident(true);
        lower.x = 3;
        lower.script = "LowerScript".into();
        lower.object_identifier = Some("Lower".into());
        let mut session = OverworldSession::with_events_and_objects(
            map,
            MapEvents::default(),
            vec![upper, lower],
            bank,
            TilePosition::new(2, 4),
        );
        session.player.mode = mode;
        session.player.facing = Direction::Right;
        let selected = session
            .check_interaction_checked(1)
            .expect("checked same-plane interaction")
            .expect("same-plane resident is not hidden by other plane's earlier slot");
        let expected = if mode == MovementMode::Normal {
            "Upper"
        } else {
            "Lower"
        };
        assert!(matches!(selected.target,
            OverworldInteractionTarget::Object { object_identifier: Some(id), .. } if id == expected));
    }
}

#[test]
fn lower_bridge_swimmer_still_blocks_surf_when_an_upper_resident_shares_xy() {
    use crate::map::MapEvents;
    use crate::world::session::OverworldSession;

    let (map, bank) = crossing();
    let mut upper = resident(false);
    upper.object_identifier = Some("Upper".into());
    let mut lower = resident(true);
    lower.object_identifier = Some("Lower".into());
    let mut session = OverworldSession::with_events_and_objects(
        map,
        MapEvents::default(),
        vec![upper, lower],
        bank,
        TilePosition::new(2, 3),
    );
    session.player.mode = MovementMode::Surf;
    session.player.facing = Direction::Down;
    let outcome = session
        .step_checked(Direction::Down, StepOptions::default())
        .expect("native bridge step");
    assert!(matches!(outcome, StepOutcome::BlockedByObject {
        object_identifier: Some(id), ..
    } if id == "Lower"));
    assert_eq!(session.player.tile, TilePosition::new(2, 3));
    assert_eq!(session.player.mode, MovementMode::Surf);
}

#[test]
fn occupied_shore_blocks_bridge_dismount_for_residents_and_strength_boulders() {
    use crate::map::MapEvents;
    use crate::world::session::OverworldSession;

    for mode in [MovementMode::Surf, MovementMode::SurfPika] {
        for boulder in [false, true] {
            let (mut map, tileset) = crossing();
            // Replace the southern Water mouth with ordinary Floor. The lower
            // bridge lane may dismount here, but cannot overlap its occupant.
            map.metatile_ids[13] = 0;
            let mut obstacle = resident(false);
            obstacle.y = 8;
            obstacle.object_identifier = Some("ShoreObstacle".into());
            if boulder {
                obstacle.sprite = "SPRITE_BOULDER".into();
                obstacle.spritemovedata = "SPRITEMOVEDATA_STRENGTH_BOULDER".into();
            }
            let mut session = OverworldSession::with_events_and_objects(
                map,
                MapEvents::default(),
                vec![obstacle],
                tileset,
                TilePosition::new(2, 7),
            );
            session.player.mode = mode;
            session.player.facing = Direction::Down;
            session
                .finish_map_object_setup()
                .expect("shore obstacle loads");
            assert_eq!(
                session
                    .step_checked(Direction::Down, StepOptions::default())
                    .expect("checked bridge-to-shore step"),
                StepOutcome::BlockedByObject {
                    at: TilePosition::new(2, 8),
                    facing: Direction::Down,
                    object_identifier: Some("ShoreObstacle".into()),
                },
                "{mode:?}, Strength boulder={boulder}"
            );
            assert_eq!(session.player.tile, TilePosition::new(2, 7));
            assert_eq!(session.player.mode, mode);
        }
    }
}

#[test]
fn empty_shore_dismounts_but_upper_bridge_actor_remains_on_its_own_plane() {
    use crate::map::MapEvents;
    use crate::world::session::OverworldSession;

    for mode in [MovementMode::Surf, MovementMode::SurfPika] {
        let (mut map, tileset) = crossing();
        map.metatile_ids[13] = 0;
        let mut session = OverworldSession::with_events_and_objects(
            map,
            MapEvents::default(),
            vec![resident(false)],
            tileset,
            TilePosition::new(2, 3),
        );
        session.player.mode = mode;
        session.player.facing = Direction::Down;
        session
            .finish_map_object_setup()
            .expect("upper bridge resident loads");
        // The resident is above(2,4). Underpass movement retains Surf while
        // crossing that occupied upper deck and both bridge rails.
        for y in 4..=7 {
            assert!(matches!(
                session
                    .step_checked(Direction::Down, StepOptions::default())
                    .expect("checked lower bridge step"),
                StepOutcome::Moved { .. }
            ));
            assert_eq!(session.player.tile, TilePosition::new(2, y));
            assert_eq!(session.player.mode, mode);
        }
        assert!(matches!(
            session
                .step_checked(Direction::Down, StepOptions::default())
                .expect("checked empty-shore dismount"),
            StepOutcome::Moved { .. }
        ));
        assert_eq!(session.player.tile, TilePosition::new(2, 8));
        assert_eq!(session.player.mode, MovementMode::Normal);
    }
}

#[test]
fn strength_shore_lookup_preserves_water_control_and_rejects_upper_bridge_boulder() {
    use crate::map::MapEvents;
    use crate::world::session::OverworldSession;

    for mode in [MovementMode::Surf, MovementMode::SurfPika] {
        for bridge_origin in [false, true] {
            let (mut map, tileset) = crossing();
            map.metatile_ids[13] = 0;
            if !bridge_origin {
                // Control: ordinary Water immediately before the same shore.
                map.metatile_ids[10] = 1;
            }
            let mut boulder = resident(false);
            boulder.y = 8;
            boulder.sprite = "SPRITE_BOULDER".into();
            boulder.spritemovedata = "SPRITEMOVEDATA_STRENGTH_BOULDER".into();
            boulder.object_identifier = Some("ShoreBoulder".into());
            let mut session = OverworldSession::with_events_and_objects(
                map,
                MapEvents::default(),
                vec![boulder],
                tileset,
                TilePosition::new(2, 7),
            );
            session.player.mode = mode;
            session.player.facing = Direction::Down;
            session
                .finish_map_object_setup()
                .expect("shore boulder loads");
            assert_eq!(
                session
                    .request_strength_boulder_push_checked(Direction::Down, StepOptions::default(),)
                    .expect("checked shore Strength request"),
                Some("ShoreBoulder".into()),
                "{mode:?}, bridge origin={bridge_origin}"
            );
            // The request queues native movement; it does not mutate positions.
            assert_eq!(session.player.tile, TilePosition::new(2, 7));
            assert_eq!(session.player.mode, mode);
        }
        let (map, tileset) = crossing();
        let mut boulder = resident(false);
        boulder.sprite = "SPRITE_BOULDER".into();
        boulder.spritemovedata = "SPRITEMOVEDATA_STRENGTH_BOULDER".into();
        boulder.object_identifier = Some("UpperBoulder".into());
        let mut session = OverworldSession::with_events_and_objects(
            map,
            MapEvents::default(),
            vec![boulder],
            tileset,
            TilePosition::new(2, 3),
        );
        session.player.mode = mode;
        session.player.facing = Direction::Down;
        session
            .finish_map_object_setup()
            .expect("upper boulder loads");
        assert_eq!(
            session
                .request_strength_boulder_push_checked(Direction::Down, StepOptions::default())
                .expect("checked upper-plane Strength rejection"),
            None,
            "{mode:?} cannot push the boulder above the underpass"
        );
    }
}
