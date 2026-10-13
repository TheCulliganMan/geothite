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

fn warp(index: u16, x: u16, target: &str, destination: i16) -> crystal_core::map::WarpEvent {
    crystal_core::map::WarpEvent {
        index,
        x,
        y: 0,
        target_map_constant: format!("{target}_MAP").to_uppercase(),
        target_map: format!("{target}_MAP").to_uppercase(),
        target_warp_id: destination,
    }
}

#[test]
fn blocked_border_certificate_requires_a_real_component_connected_warp_detour() {
    let mut start = test_map_module("Start", "START_MAP", Some("Goal"));
    start.attributes.width = 2;
    start.blocks = vec![1, 0];
    start.events.warps = vec![warp(1, 0, "Gate", 1)];
    let mut goal = test_map_module("Goal", "GOAL_MAP", None);
    goal.attributes.width = 2;
    goal.blocks = vec![0, 1];
    goal.events.warps = vec![warp(1, 3, "Gate", 2)];
    let mut gate = test_map_module("Gate", "GATE_MAP", None);
    gate.events.warps = vec![warp(1, 0, "Start", 1), warp(2, 1, "Goal", 1)];
    let mut tileset = test_tileset_definition();
    tileset
        .collision
        .insert("0".into(), ["WALL"; 4].map(str::to_string).to_vec());
    let mut data = GameDataSet {
        maps: [
            ("Start".into(), start),
            ("Gate".into(), gate),
            ("Goal".into(), goal),
        ]
        .into_iter()
        .collect(),
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        playability: PlayabilityRules {
            start_tiles: vec![crystal_assets::modpack::PlayabilityStart {
                map: "Start".into(),
                tile: crystal_core::world::map::TilePosition::new(0, 0),
            }],
            require_all_maps_reachable: true,
            connection_warp_routes: [(
                "Start".into(),
                [(
                    "Goal".into(),
                    vec!["Start".into(), "Gate".into(), "Goal".into()],
                )]
                .into_iter()
                .collect(),
            )]
            .into_iter()
            .collect(),
            ..PlayabilityRules::default()
        },
        ..GameDataSet::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let compiler = ModpackCompiler::new(&assets);
    let valid = compiler.audit(&data);
    assert!(valid.reachable_maps.iter().any(|map| map == "Goal"), "{:?}", valid.diagnostics);
    assert!(!valid.diagnostics.iter().any(|d| matches!(
        d.code.as_str(),
        "invalid_connection_warp_route"
            | "unreachable_connection"
            | "unreachable_connection_destination"
    )));
    assert!(
        !valid
            .graph_edges
            .iter()
            .any(|edge| edge.kind == "connection"),
        "a gate detour is never invented as a direct connection"
    );

    let original = data.clone();
    data.maps.get_mut("Gate").unwrap().attributes.width = 3;
    data.maps.get_mut("Gate").unwrap().blocks = vec![1, 0, 1];
    data.maps.get_mut("Gate").unwrap().events.warps[1].x = 5;
    let separated = compiler.audit(&data);
    assert!(
        separated
            .diagnostics
            .iter()
            .any(|d| d.code == "invalid_connection_warp_route"),
        "warp endpoints separated by a wall cannot certify the route"
    );
    assert!(!separated.reachable_maps.iter().any(|map| map == "Goal"));

    data = original.clone();
    data.maps.get_mut("Start").unwrap().blocks[1] = 1;
    data.maps.get_mut("Goal").unwrap().blocks[0] = 1;
    assert!(
        compiler
            .audit(&data)
            .diagnostics
            .iter()
            .any(|d| d.code == "invalid_connection_warp_route"),
        "opening the direct border invalidates its blocked-border certificate"
    );

    data = original;
    data.maps.get_mut("Start").unwrap().attributes.connections[0].offset = i32::MAX;
    assert!(
        compiler
            .audit(&data)
            .diagnostics
            .iter()
            .any(|d| d.code == "invalid_connection_warp_route"),
        "a real detour cannot hide invalid connection offset bounds"
    );
}
