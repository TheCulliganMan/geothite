use super::{map_with_blocks, object, tileset};
use crate::map::MapEvents;
use crate::random::{CrystalRandom, CrystalRandomState, ReplayDivider};
use crate::state::GameState;
use crate::systems::map_context::{apply_state_object_overrides, sync_state_object_overrides};
use crate::systems::script_objects::{ScriptObjectCommand, apply_writeobjectxy_command};
use crate::world::collision::{MetatileCollision, permissions};
use crate::world::map::TilePosition;
use crate::world::movement::MovementMode;
use crate::world::session::OverworldSession;
use std::collections::BTreeSet;

#[test]
fn minnesota_fair_fixed_trainer_waits_for_the_last_approach_tile_to_clear() {
    let mut eli = object("ELI", 36, 26, "-1");
    eli.object_type = "OBJECTTYPE_TRAINER".into();
    eli.sightline_direction_override = Some("DOWN".into());
    eli.radius = 2;
    let ruth = object("RUTH", 34, 27, "-1");
    let mut map = map_with_blocks(20, 16, vec![0; 320]);
    map.name = "MnStateFairgrounds".into();
    let mut session = OverworldSession::with_events_and_objects(
        map,
        MapEvents::default(),
        vec![eli, ruth],
        tileset(),
        TilePosition::new(36, 28),
    );
    let spotted = |session: &OverworldSession| {
        session
            .check_trainer_sight_checked()
            .expect("fair sight")
            .is_some()
    };
    assert!(spotted(&session));
    session
        .set_object_runtime_tile("RUTH", TilePosition::new(36, 27))
        .expect("earlier trainer approached");
    assert!(!spotted(&session), "Eli cannot approach through Ruth");
    session
        .set_object_runtime_tile("RUTH", TilePosition::new(37, 27))
        .expect("Ruth steps aside");
    session
        .object_last_runtime_tiles
        .insert("RUTH".into(), TilePosition::new(36, 27));
    session
        .object_last_tiles_occupied_until_frame
        .insert("RUTH".into(), 8);
    session.frame = 7;
    assert!(
        !spotted(&session),
        "the guest still occupies the departure tile"
    );
    session.frame = 8;
    assert!(spotted(&session));
    session.player.tile.y = 29;
    assert!(!spotted(&session), "a fixed lane keeps its authored range");
}

#[test]
fn minnesota_fixed_trainer_respects_terrain_without_stepping_on_the_players_warp() {
    let mut trainer = object("TRAINER", 2, 0, "-1");
    trainer.object_type = "OBJECTTYPE_TRAINER".into();
    trainer.sightline_direction_override = Some("DOWN".into());
    trainer.radius = 2;
    let mut map = map_with_blocks(3, 2, vec![0, 1, 0, 0, 2, 0]);
    map.name = "MnStPaulGym".into();
    let mut collision = tileset();
    collision.metatiles.push(MetatileCollision {
        collision: [permissions::FLOOR; 4],
    });
    collision.metatiles.push(MetatileCollision {
        collision: [permissions::WARP_PANEL; 4],
    });
    let mut session = OverworldSession::with_events_and_objects(
        map,
        MapEvents::default(),
        vec![trainer],
        collision,
        TilePosition::new(2, 2),
    );
    let spotted = |session: &OverworldSession| {
        session
            .check_trainer_sight_checked()
            .expect("gym sight")
            .is_some()
    };
    assert!(
        spotted(&session),
        "the player can stand on the St. Paul panel"
    );
    for obstruction in [
        permissions::WALL,
        permissions::COUNTER,
        permissions::DOWN_WALL,
        permissions::HOP_DOWN,
        permissions::HOP_UP,
        permissions::WATER,
        permissions::WARP_PANEL,
        crate::world::bridge::NORTH_SOUTH_RAIL,
    ] {
        session.tileset.metatiles[1].collision[2] = obstruction;
        assert!(!spotted(&session), "intermediate {obstruction:#x}");
    }
    session.tileset.metatiles[1].collision[2] = crate::world::bridge::NORTH_SOUTH_DECK;
    assert!(spotted(&session), "a walker can approach on a deck");
    session.player.mode = MovementMode::Surf;
    session.tileset.metatiles[2].collision[0] = permissions::WATER;
    assert!(
        !spotted(&session),
        "a walker approaching on the deck cannot challenge a surfer on the river below"
    );
    session.tileset.metatiles[1].collision[2] = permissions::FLOOR;
    assert!(
        spotted(&session),
        "the same surfer is visible from an ordinary shoreline"
    );
    session.objects[0].spritemovedata = "SPRITEMOVEDATA_SWIM_WANDER".into();
    session.tileset.metatiles[1].collision[0] = permissions::WATER;
    session.tileset.metatiles[1].collision[2] = crate::world::bridge::NORTH_SOUTH_DECK;
    assert!(
        spotted(&session),
        "a swimmer can approach along the underpass"
    );
    session.tileset.metatiles[1].collision[2] = crate::world::bridge::EAST_WEST_DECK;
    assert!(
        !spotted(&session),
        "a swimmer cannot cut across a bridge abutment"
    );
}

#[test]
fn minnesota_patrol_checks_for_a_stationary_player_on_landing() {
    let mut trainer = object("RUNNER", 1, 1, "-1");
    trainer.object_type = "OBJECTTYPE_TRAINER".into();
    trainer.spritemovedata = "SPRITEMOVEDATA_PATROL_CLOCKWISE".into();
    trainer.move_range_x = 2;
    trainer.move_range_y = 1;
    trainer.radius = 2;
    let mut map = map_with_blocks(4, 3, vec![0; 12]);
    map.name = "MnStateFairgrounds".into();
    let mut session = OverworldSession::with_events_and_objects(
        map,
        MapEvents::default(),
        vec![trainer],
        tileset(),
        TilePosition::new(5, 1),
    );
    let mut divider = ReplayDivider::new([0; 64]);
    let mut rng = CrystalRandom::new(CrystalRandomState::default(), &mut divider);
    for frame in 1..=9 {
        session.frame = frame;
        session
            .advance_autonomous_objects_exact(&mut rng)
            .expect("patrol advances");
        assert_eq!(session.trainer_patrol_landed_since(frame - 1), frame == 9);
        assert_eq!(
            session
                .check_trainer_sight_checked()
                .expect("patrol sight")
                .is_some(),
            frame == 9
        );
    }
    assert_eq!(
        session
            .object_runtime_tile_by_id("RUNNER")
            .expect("settled runner"),
        TilePosition::new(2, 1)
    );
}

#[test]
fn minnesota_patrol_returns_to_its_original_circuit_around_a_guest_after_a_battle() {
    let mut trainer = object("RUNNER", 1, 1, "-1");
    trainer.object_type = "OBJECTTYPE_TRAINER".into();
    trainer.spritemovedata = "SPRITEMOVEDATA_PATROL_CLOCKWISE".into();
    trainer.move_range_x = 2;
    trainer.move_range_y = 1;
    trainer.radius = 2;
    let guest = object("GUEST", 4, 1, "-1");
    let mut map = map_with_blocks(12, 3, vec![0; 36]);
    map.name = "MnStateFairgrounds".into();
    let mut session = OverworldSession::with_events_and_objects(
        map,
        MapEvents::default(),
        vec![trainer, guest],
        tileset(),
        TilePosition::new(5, 4),
    );
    let fresh = session.clone();
    session
        .set_object_runtime_tile("RUNNER", TilePosition::new(5, 1))
        .expect("trainer approached");
    session.last_talked_object_identifier = Some("RUNNER".into());
    let command = ScriptObjectCommand {
        source_script: "SeenByTrainerScript".into(),
        command_index: 5,
        command: "writeobjectxy".into(),
        object_id: Some("LAST_TALKED".into()),
        target_object_id: None,
        x: None,
        y: None,
        direction: None,
        movement: None,
        emote: None,
        duration: None,
    };
    apply_writeobjectxy_command(&mut session, &command).expect("commit the approach");
    assert_eq!(session.objects[0].x, 5);
    let mut state = GameState::default();
    sync_state_object_overrides(&mut state, &session).expect("save reached coordinates");
    let mut session = fresh;
    apply_state_object_overrides(&mut session, &state).expect("restore approached trainer");
    let mut divider = ReplayDivider::new([0; 128]);
    let mut rng = CrystalRandom::new(CrystalRandomState::default(), &mut divider);
    let mut previous = TilePosition::new(5, 1);
    let mut visited = BTreeSet::new();
    for frame in 1..=100 {
        session.frame = frame;
        session
            .advance_autonomous_objects_exact(&mut rng)
            .expect("patrol returns");
        let tile = session
            .object_runtime_tile_by_id("RUNNER")
            .expect("live runner");
        assert!(tile.x.abs_diff(previous.x) + tile.y.abs_diff(previous.y) <= 1);
        assert_ne!(
            tile,
            TilePosition::new(4, 1),
            "return cannot walk through a guest"
        );
        if frame > 40 {
            visited.insert((tile.x, tile.y));
        }
        previous = tile;
    }
    assert_eq!(
        visited,
        BTreeSet::from([(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)])
    );
    let away: Vec<_> = (6..=22).map(|x| TilePosition::new(x, 4)).collect();
    session
        .advance_object_struct_roster_along_player_path(&away)
        .expect("leave the midway");
    assert!(!session.object_has_loaded_struct(0));
    let back: Vec<_> = (5..=21).rev().map(|x| TilePosition::new(x, 4)).collect();
    session
        .advance_object_struct_roster_along_player_path(&back)
        .expect("return to the midway");
    assert!(session.object_has_loaded_struct(0));
    let tile = session
        .object_runtime_tile_by_id("RUNNER")
        .expect("reloaded runner");
    assert!(
        visited.contains(&(tile.x, tile.y)),
        "the runner resumes on the circuit after a revisit"
    );
}
