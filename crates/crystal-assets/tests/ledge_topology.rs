use crystal_assets::modpack::{GameDataSet, MapModule, ModpackCompiler, PlayabilityRules};
use crystal_assets::{AssetRoot, TilesetDefinition};
use crystal_core::map::{MapAttributes, MapConnection, MapEvents, MapSceneTable};
use std::collections::BTreeMap;

fn test_map_module(id: &str, map_constant: &str, connection_target: Option<&str>) -> MapModule {
    MapModule {
        id: id.to_string(),
        attributes: MapAttributes {
            tileset_name: "johto".to_string(),
            border_block: 0,
            width: 1,
            height: 1,
            connections: connection_target
                .map(|target| {
                    vec![MapConnection {
                        direction: "east".to_string(),
                        target_map: target.to_string(),
                        offset: 0,
                    }]
                })
                .unwrap_or_default(),
            time_of_day: None,
            phone_service: 0,
            phone_flag: false,
            environment: Some("route".to_string()),
            location: Some("johto".to_string()),
            music: None,
            palette: None,
            fishing_group: None,
            map_constant: Some(map_constant.to_string()),
            map_group_constant: None,
            blocks_label: None,
            map_scripts_label: None,
            map_events_label: None,
            connection_flags: None,
        },
        scripts: BTreeMap::new(),
        trainer_scripts: BTreeMap::new(),
        scripted_trainer_battles: Vec::new(),
        scripted_wild_battles: Vec::new(),
        script_item_grants: Vec::new(),
        script_item_checks: Vec::new(),
        script_item_takes: Vec::new(),
        script_economy_commands: Vec::new(),
        gift_pokemon_scripts: Vec::new(),
        script_flag_commands: Vec::new(),
        script_scene_commands: Vec::new(),
        script_audio_commands: Vec::new(),
        script_block_changes: Vec::new(),
        script_object_commands: Vec::new(),
        script_movements: Vec::new(),
        script_map_commands: Vec::new(),
        script_text_commands: Vec::new(),
        script_text_bodies: BTreeMap::new(),
        script_menu_definitions: BTreeMap::new(),
        script_vertical_menus: BTreeMap::new(),
        script_elevators: BTreeMap::new(),
        script_variable_commands: Vec::new(),
        script_control_commands: Vec::new(),
        script_field_pickups: Vec::new(),
        script_shop_commands: Vec::new(),
        script_phone_commands: Vec::new(),
        script_runtime_commands: Vec::new(),
        script_swarm_commands: Vec::new(),
        map_script_section_commands: Vec::new(),
        map_event_section_commands: Vec::new(),
        scenes: MapSceneTable::default(),
        events: MapEvents::default(),
        objects: Vec::new(),
        blocks: vec![1],
    }
}

fn test_tileset_definition() -> TilesetDefinition {
    TilesetDefinition {
        collision: (0..=10)
            .map(|metatile_id| {
                (
                    format!("{metatile_id:x}"),
                    vec![
                        "FLOOR".to_string(),
                        "FLOOR".to_string(),
                        "FLOOR".to_string(),
                        "FLOOR".to_string(),
                    ],
                )
            })
            .collect(),
        palette_map: vec![0],
    }
}

#[test]
fn topology_uses_one_way_ledge_jumps_and_rejects_wall_landings() {
    let mut ledge = test_map_module("Ledge", "LEDGE_MAP", Some("Goal"));
    ledge.attributes.height = 2;
    ledge.blocks = vec![2, 1];
    let mut goal = test_map_module("Goal", "GOAL_MAP", None);
    goal.attributes.height = 2;
    goal.blocks = vec![0, 1];
    let mut tileset = test_tileset_definition();
    tileset
        .collision
        .insert("0".into(), ["WALL"; 4].map(str::to_string).to_vec());
    tileset.collision.insert(
        "2".into(),
        ["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]
            .map(str::to_string)
            .to_vec(),
    );
    let mut data = GameDataSet {
        maps: [("Ledge".into(), ledge), ("Goal".into(), goal)]
            .into_iter()
            .collect(),
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        playability: PlayabilityRules {
            start_tiles: vec![crystal_assets::modpack::PlayabilityStart {
                map: "Ledge".into(),
                tile: crystal_core::world::map::TilePosition::new(0, 0),
            }],
            goal_maps: vec!["Goal".into()],
            require_all_maps_reachable: true,
            ..PlayabilityRules::default()
        },
        ..GameDataSet::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let forward = ModpackCompiler::new(&assets).audit(&data);
    assert!(
        forward.reachable_maps.iter().any(|map| map == "Goal"),
        "authored downward jump reaches the lower exit"
    );
    assert!(
        forward
            .graph_edges
            .iter()
            .any(|edge| edge.from == "Ledge" && edge.to == "Ledge" && edge.kind == "ledge")
    );

    data.playability.start_tiles[0].tile = crystal_core::world::map::TilePosition::new(0, 2);
    data.maps.get_mut("Goal").unwrap().blocks = vec![1, 0];
    let reverse = ModpackCompiler::new(&assets).audit(&data);
    assert!(
        !reverse.reachable_maps.iter().any(|map| map == "Goal"),
        "a downward ledge cannot be climbed to the upper exit"
    );

    data.playability.start_tiles[0].tile = crystal_core::world::map::TilePosition::new(0, 0);
    data.maps.get_mut("Goal").unwrap().blocks = vec![0, 1];
    data.maps.get_mut("Ledge").unwrap().blocks[1] = 0;
    let wall = ModpackCompiler::new(&assets).audit(&data);
    assert!(
        !wall.reachable_maps.iter().any(|map| map == "Goal"),
        "wall landing cannot be accepted as a jump"
    );
    assert!(!wall.graph_edges.iter().any(|edge| edge.kind == "ledge"));
}
