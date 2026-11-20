use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::map::MapConnection;
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, attempt_ledge_jump,
    attempt_ledge_jump_with_occupied_tiles,
};
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

use super::{
    Direction, LineKind, TerrainMap, TilePosition, UNKNOWN, glyph_for_permission,
    resolve_permission, terrain_glyph,
};

#[test]
fn collision_tokens_have_distinct_accessible_glyphs() {
    let classify_collision_token = |token| {
        resolve_permission(token).map_or(UNKNOWN, |permission| {
            glyph_for_permission(permission, |_| Some(true))
        })
    };
    assert_eq!(classify_collision_token("WATER"), ('≈', LineKind::Water));
    assert_eq!(
        classify_collision_token("TALL_GRASS"),
        ('\"', LineKind::Grass)
    );
    assert_eq!(classify_collision_token("WALL"), ('#', LineKind::Normal));
    assert_eq!(classify_collision_token("DOOR"), ('D', LineKind::Hint));
    assert_eq!(classify_collision_token("HOP_DOWN"), ('↓', LineKind::Hint));
    assert_eq!(classify_collision_token("a5"), ('↙', LineKind::Hint));
    for token in ["07", "0f", "27", "62", "ff", "COLL_07"] {
        assert_eq!(
            classify_collision_token(token).0,
            '#',
            "hidden wall {token}"
        );
    }
    for token in ["CUT_TREE", "HEADBUTT_TREE_1D", "12"] {
        assert_eq!(
            classify_collision_token(token).0,
            'T',
            "hidden tree {token}"
        );
    }
    assert_eq!(classify_collision_token("RIGHT_WALL").0, '║');
    assert_eq!(classify_collision_token("CURRENT_UP").0, '≈');
    assert_eq!(
        classify_collision_token("unsupported"),
        ('?', LineKind::Danger)
    );
}

#[test]
fn ledge_hints_check_each_corner_direction_and_distinguish_missing_data() {
    use crystal_core::world::collision::permissions;

    for (permission, horizontal, vertical, both, h, v) in [
        (
            permissions::HOP_UP_LEFT,
            Direction::Left,
            Direction::Up,
            '↖',
            '←',
            '↑',
        ),
        (
            permissions::HOP_UP_RIGHT,
            Direction::Right,
            Direction::Up,
            '↗',
            '→',
            '↑',
        ),
        (
            permissions::HOP_DOWN_LEFT,
            Direction::Left,
            Direction::Down,
            '↙',
            '←',
            '↓',
        ),
        (
            permissions::HOP_DOWN_RIGHT,
            Direction::Right,
            Direction::Down,
            '↘',
            '→',
            '↓',
        ),
    ] {
        assert_eq!(glyph_for_permission(permission, |_| Some(true)).0, both);
        assert_eq!(
            glyph_for_permission(permission, |d| Some(d == horizontal)).0,
            h
        );
        assert_eq!(
            glyph_for_permission(permission, |d| Some(d == vertical)).0,
            v
        );
        assert_eq!(
            glyph_for_permission(permission, |_| Some(false)),
            ('.', LineKind::Normal)
        );
        assert_eq!(glyph_for_permission(permission, |_| None), UNKNOWN);
    }
    let collision = BTreeMap::from([
        ("00".to_string(), vec!["HOP_RIGHT".to_string(); 4]),
        ("01".to_string(), vec!["WALL".to_string(); 4]),
    ]);
    let map = TerrainMap {
        width: 3,
        height: 1,
        connections: &[],
        blocks: &[0, 1, 99],
        collision: &collision,
    };
    assert_eq!(map.glyph(TilePosition::new(0, 0)), ('.', LineKind::Normal));
    assert_eq!(map.glyph(TilePosition::new(4, 0)), UNKNOWN);
    let broken = TerrainMap {
        blocks: &[0, 99, 1],
        ..map
    };
    assert_eq!(broken.glyph(TilePosition::new(0, 0)), UNKNOWN);
}

#[test]
fn ledge_hints_at_map_edges_require_the_matching_directional_connection() {
    let collision = BTreeMap::from([("00".to_string(), vec!["HOP_UP_RIGHT".to_string(); 4])]);
    let map = TerrainMap {
        width: 1,
        height: 1,
        connections: &[],
        blocks: &[0],
        collision: &collision,
    };
    assert_eq!(map.glyph(TilePosition::new(1, 1)), ('.', LineKind::Normal));
    let connections = [MapConnection {
        direction: "east".into(),
        target_map: "NextMap".into(),
        offset: 0,
    }];
    let linked = TerrainMap {
        connections: &connections,
        ..map
    };
    assert_eq!(linked.glyph(TilePosition::new(1, 1)), ('→', LineKind::Hint));
    assert_eq!(
        linked.glyph(TilePosition::new(2, 0)),
        (' ', LineKind::Normal)
    );
}

#[test]
#[ignore = "requires an installed Minnesota pack via MN_BADGE_TRAILS_PACK"]
fn route15_ledge_hints_follow_native_walking_landings() {
    let path = PathBuf::from(std::env::var_os("MN_BADGE_TRAILS_PACK").expect("Minnesota pack"));
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified pack"),
    )
    .expect("installed maps");
    let route = "MnR15C00R00";
    let mut game = RuntimeGameShell::new_game_at_runtime_tile(
        root,
        runtime.clone(),
        runtime.title_new_game_spawn_identifier().expect("HOME"),
        route,
        23,
        16,
    )
    .expect("walkable west bank");
    if let Some(directory) = std::env::var_os("MN_LEDGE_REVIEW_DIR").map(PathBuf::from) {
        std::fs::create_dir_all(&directory).expect("review directory");
        game.save(directory.join("route15-west-bank.crystalsave"))
            .expect("render fixture");
    }
    let mut snapshot = game
        .presentation_snapshot()
        .expect("active blocks and collisions");
    let map = snapshot
        .maps
        .iter()
        .find(|m| m.map_name == route)
        .expect("west bank map");
    let terrain = runtime
        .data()
        .overworld_map(route)
        .expect("native walking geometry");
    let tiles = runtime
        .data()
        .tileset_collision(&map.attributes.tileset_name)
        .expect("native walking permissions");
    for y in 10..=18 {
        let origin = TilePosition::new(24, y);
        let mut walker = PlayerMovementState::new(origin);
        assert!(
            matches!(attempt_ledge_jump(&mut walker, Direction::Right, &terrain, &tiles, StepOptions::default()),
            LedgeJumpOutcome::BlockedLanding { at, .. } if at == TilePosition::new(26, y))
        );
        assert_eq!(walker.tile, origin);
        assert_eq!(
            terrain_glyph(&snapshot, map, origin).0,
            if y == 18 { '↓' } else { '.' }
        );
    }
    for x in 22..=24 {
        let origin = TilePosition::new(x, 18);
        let landing = TilePosition::new(x, 20);
        let mut walker = PlayerMovementState::new(origin);
        assert!(
            matches!(attempt_ledge_jump(&mut walker, Direction::Down, &terrain, &tiles, StepOptions::default()),
            LedgeJumpOutcome::Jumped { over, to, .. } if over == TilePosition::new(x, 19) && to == landing)
        );
        assert_eq!(walker.tile, landing);
        assert_eq!(terrain_glyph(&snapshot, map, origin), ('↓', LineKind::Hint));
    }
    // A trainer may briefly stand below the meadow drop. Its terrain hint stays stable.
    let origin = TilePosition::new(24, 18);
    let landing = TilePosition::new(24, 20);
    snapshot
        .visible_object_runtime_tiles
        .insert("passerby".into(), landing);
    let actor = OccupiedTile {
        tile: landing,
        object_identifier: Some("passerby".into()),
    };
    assert!(
        matches!(attempt_ledge_jump_with_occupied_tiles(&mut PlayerMovementState::new(origin), Direction::Down, &terrain, &tiles, StepOptions::default(), &[actor]),
        LedgeJumpOutcome::BlockedByObject { at, .. } if at == landing)
    );
    assert_eq!(terrain_glyph(&snapshot, map, origin), ('↓', LineKind::Hint));

    // Crystal uses the same east-facing face in Fuchsia, above dry ground.
    let city = snapshot
        .maps
        .iter()
        .find(|m| m.map_name == "FuchsiaCity")
        .expect("original Kanto example");
    let city_terrain = runtime
        .data()
        .overworld_map(&city.map_name)
        .expect("Kanto walking geometry");
    let city_tiles = runtime
        .data()
        .tileset_collision(&city.attributes.tileset_name)
        .expect("Kanto walking permissions");
    for y in [28, 29] {
        let origin = TilePosition::new(22, y);
        let mut walker = PlayerMovementState::new(origin);
        assert!(
            matches!(attempt_ledge_jump(&mut walker, Direction::Right, &city_terrain, &city_tiles, StepOptions::default()),
            LedgeJumpOutcome::Jumped { over, to, .. } if over == TilePosition::new(23, y) && to == TilePosition::new(24, y))
        );
        assert_eq!(
            terrain_glyph(&snapshot, city, origin),
            ('→', LineKind::Hint)
        );
    }
}
