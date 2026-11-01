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
fn verifier_checks_all_connection_border_pairs_before_rejecting_a_destination() {
    let start = test_map_module("Start", "START_MAP", Some("Goal"));
    let mut goal = test_map_module("Goal", "GOAL_MAP", None);
    goal.blocks = vec![2];
    let mut tileset = test_tileset_definition();
    // The first east-boundary source tile faces a wall; the lower pair is open.
    tileset.collision.insert(
        "2".into(),
        ["WALL", "FLOOR", "FLOOR", "FLOOR"]
            .map(str::to_string)
            .to_vec(),
    );
    let mut data = GameDataSet {
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        maps: [("Start".into(), start), ("Goal".into(), goal)]
            .into_iter()
            .collect(),
        ..GameDataSet::default()
    };
    let rules = PlayabilityRules {
        start_maps: vec!["Start".into()],
        goal_maps: vec!["Goal".into()],
        require_all_maps_reachable: true,
        require_walkable_maps: true,
        ..PlayabilityRules::default()
    };
    data.playability = rules;
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let report = ModpackCompiler::new(&assets).audit(&data);
    assert!(report.reachable_maps.iter().any(|map| map == "Goal"));
    assert!(
        report
            .graph_edges
            .iter()
            .any(|edge| edge.from == "Start" && edge.to == "Goal" && edge.kind == "connection")
    );
    assert!(!report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.code.as_str(),
        "unreachable_connection_destination" | "unreachable_goal_map" | "unreachable_map"
    )));
    data.tilesets
        .get_mut("johto")
        .unwrap()
        .collision
        .insert("2".into(), ["WALL"; 4].map(str::to_string).to_vec());
    let blocked = ModpackCompiler::new(&assets).audit(&data);
    assert!(!blocked.reachable_maps.iter().any(|map| map == "Goal"));
    assert!(blocked.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "unreachable_connection_destination" && diagnostic.subject == "Start"
    }));
}

#[test]
fn source_surf_topology_crosses_water_without_crossing_walls_or_granting_inventory() {
    let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
        return;
    };
    let assets_path = std::path::PathBuf::from(root).join("apps/web/assets/data");
    let start = test_map_module("Start", "START_MAP", Some("Goal"));
    let mut goal = test_map_module("Goal", "GOAL_MAP", None);
    goal.blocks = vec![2];
    let mut tileset = test_tileset_definition();
    tileset
        .collision
        .insert("2".into(), vec!["WATER".into(); 4]);
    let mut data = GameDataSet {
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        maps: [("Start".into(), start), ("Goal".into(), goal)]
            .into_iter()
            .collect(),
        moves: serde_json::from_slice(&std::fs::read(assets_path.join("moves.json")).unwrap())
            .unwrap(),
        field_moves: serde_json::from_slice(
            &std::fs::read(assets_path.join("field_moves.json")).unwrap(),
        )
        .unwrap(),
        playability: PlayabilityRules {
            start_maps: vec!["Start".into()],
            require_all_maps_reachable: true,
            ..PlayabilityRules::default()
        },
        ..GameDataSet::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
    data.playability.topology_traversal_modes = vec!["Walk".into(), "Surf".into()];
    assert!(
        ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
    assert!(data.playability.initial_items.is_empty());
    data.tilesets
        .get_mut("johto")
        .unwrap()
        .collision
        .insert("2".into(), vec!["WALL".into(); 4]);
    assert!(
        !ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
}

#[test]
fn explicit_script_warp_reaches_an_isolated_map_but_rejects_a_wall_destination() {
    let mut start = test_map_module("Start", "START_MAP", None);
    let mut goal = test_map_module("Goal", "GOAL_MAP", None);
    goal.blocks = vec![2];
    start.scripts.insert(
        "DoorScript".into(),
        serde_json::json!([
            {"command":"warp","args":["GOAL_MAP","0","0"]},
            {"command":"end","args":[]}
        ]),
    );
    start
        .map_script_section_commands
        .push(crystal_core::map::MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_NEWMAP".into(), "DoorScript".into()],
            command_index: 0,
        });
    start
        .script_map_commands
        .push(crystal_core::systems::script_warps::ScriptMapCommand {
            command: "warp".into(),
            target_map: Some("Goal".into()),
            x: Some(0),
            y: Some(0),
            facing: None,
            map_setup: None,
            source_script: "DoorScript".into(),
            command_index: 0,
        });
    let mut data = GameDataSet {
        maps: [("Start".into(), start), ("Goal".into(), goal)]
            .into_iter()
            .collect(),
        tilesets: [("johto".into(), test_tileset_definition())]
            .into_iter()
            .collect(),
        playability: PlayabilityRules {
            start_maps: vec!["Start".into()],
            require_all_maps_reachable: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let report = ModpackCompiler::new(&assets).audit(&data);
    assert!(
        report
            .graph_edges
            .iter()
            .any(|edge| edge.kind == "script_warp" && edge.to == "Goal")
    );
    assert!(report.reachable_maps.contains(&"Goal".into()));
    let callbacks = std::mem::take(
        &mut data
            .maps
            .get_mut("Start")
            .unwrap()
            .map_script_section_commands,
    );
    assert!(
        !ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
    data.maps
        .get_mut("Start")
        .unwrap()
        .map_script_section_commands = callbacks;
    data.tilesets
        .get_mut("johto")
        .unwrap()
        .collision
        .insert("2".into(), vec!["WALL".into(); 4]);
    assert!(
        !ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
}

#[test]
fn topology_uses_only_callable_authored_block_changes_without_mutating_maps() {
    let start = test_map_module("Start", "START_MAP", Some("Goal"));
    let mut goal = test_map_module("Goal", "GOAL_MAP", None);
    goal.blocks = vec![2];
    goal.scripts.insert(
        "OpenDoor".into(),
        serde_json::json!([
            {"command":"changeblock","args":["0","0","1"]},
            {"command":"endcallback","args":[]}
        ]),
    );
    goal.map_script_section_commands
        .push(crystal_core::map::MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_TILES".into(), "OpenDoor".into()],
            command_index: 0,
        });
    goal.script_block_changes
        .push(crystal_core::systems::script_blocks::ScriptBlockChange {
            x: 0,
            y: 0,
            block_id: 1,
            source_script: "OpenDoor".into(),
            command_index: 0,
        });
    let mut tileset = test_tileset_definition();
    tileset.collision.insert("2".into(), vec!["WALL".into(); 4]);
    let mut data = GameDataSet {
        maps: [("Start".into(), start), ("Goal".into(), goal)]
            .into_iter()
            .collect(),
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        playability: PlayabilityRules {
            start_maps: vec!["Start".into()],
            require_all_maps_reachable: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
    data.playability.topology_script_block_changes = true;
    assert!(
        ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
    assert_eq!(data.maps["Goal"].blocks, vec![2]);
    data.maps
        .get_mut("Goal")
        .unwrap()
        .map_script_section_commands
        .clear();
    assert!(
        !ModpackCompiler::new(&assets)
            .audit(&data)
            .reachable_maps
            .contains(&"Goal".into())
    );
}

#[test]
fn dormant_beta_data_remains_loaded_and_active_ingress_is_rejected() {
    let mut data = GameDataSet {
        maps: [
            ("Start".into(), test_map_module("Start", "START_MAP", None)),
            (
                "UnusedBeta".into(),
                test_map_module("UnusedBeta", "UNUSED_BETA", None),
            ),
        ]
        .into_iter()
        .collect(),
        tilesets: [("johto".into(), test_tileset_definition())]
            .into_iter()
            .collect(),
        playability: PlayabilityRules {
            start_maps: vec!["Start".into()],
            dormant_maps: vec!["UnusedBeta".into()],
            require_all_maps_reachable: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let report = ModpackCompiler::new(&assets).audit(&data);
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|error| error.subject == "UnusedBeta"
                && matches!(
                    error.code.as_str(),
                    "unreachable_map" | "invalid_dormant_map"
                ))
    );
    assert!(data.maps.contains_key("UnusedBeta"));
    data.maps
        .get_mut("Start")
        .unwrap()
        .attributes
        .connections
        .push(MapConnection {
            direction: "east".into(),
            target_map: "UnusedBeta".into(),
            offset: 0,
        });
    assert!(
        ModpackCompiler::new(&assets)
            .audit(&data)
            .diagnostics
            .iter()
            .any(|error| error.code == "active_ingress_to_dormant_map")
    );
}

#[test]
fn dormant_warp_certificate_requires_blocked_authored_geometry() {
    let mut start = test_map_module("Start", "START_MAP", None);
    let mut dormant = test_map_module("UnusedBeta", "UNUSED_BETA", None);
    dormant.events.warps.push(crystal_core::map::WarpEvent {
        index: 1, x: 0, y: 0, target_map: "START_MAP".into(),
        target_map_constant: "START_MAP".into(), target_warp_id: 1,
    });
    start.events.warps.push(crystal_core::map::WarpEvent {
        index: 1, x: 0, y: 0, target_map: "UNUSED_BETA".into(),
        target_map_constant: "UNUSED_BETA".into(), target_warp_id: 1,
    });
    let mut tileset = test_tileset_definition();
    tileset.collision.insert("1".into(), vec!["WALL".into(); 4]);
    let mut data = GameDataSet {
        maps: [("Start".into(), start), ("UnusedBeta".into(), dormant)].into_iter().collect(),
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        playability: PlayabilityRules {
            dormant_maps: vec!["UnusedBeta".into()],
            inaccessible_dormant_warps: [("Start".into(), vec![1])].into_iter().collect(),
            ..Default::default()
        }, ..Default::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let report = ModpackCompiler::new(&assets).audit(&data);
    assert!(!report.diagnostics.iter().any(|error| matches!(error.code.as_str(),
        "invalid_inaccessible_dormant_warp" | "unreachable_warp" | "unknown_warp_index")));
    data.tilesets.get_mut("johto").unwrap().collision.insert("1".into(), vec!["FLOOR".into(); 4]);
    assert!(ModpackCompiler::new(&assets).audit(&data).diagnostics.iter()
        .any(|error| error.code == "invalid_inaccessible_dormant_warp"));
}

#[test]
fn retired_gs_warp_pair_retains_structural_checks_and_requires_blocked_endpoint() {
    let mut start = test_map_module("Start", "START_MAP", None);
    let mut basement = test_map_module("Basement", "BASEMENT_MAP", None);
    basement.blocks = vec![2];
    start.events.warps.push(crystal_core::map::WarpEvent {
        index: 1, x: 0, y: 0, target_map: "BASEMENT_MAP".into(),
        target_map_constant: "BASEMENT_MAP".into(), target_warp_id: 1,
    });
    basement.events.warps.push(crystal_core::map::WarpEvent {
        index: 1, x: 0, y: 0, target_map: "START_MAP".into(),
        target_map_constant: "START_MAP".into(), target_warp_id: 1,
    });
    let mut tileset = test_tileset_definition();
    tileset.collision.insert("2".into(), vec!["WALL".into(); 4]);
    let mut data = GameDataSet {
        maps: [("Start".into(), start), ("Basement".into(), basement)].into_iter().collect(),
        tilesets: [("johto".into(), tileset)].into_iter().collect(),
        playability: PlayabilityRules {
            retired_gs_warps: [("Start".into(), vec![1]), ("Basement".into(), vec![1])].into_iter().collect(),
            ..Default::default()
        }, ..Default::default()
    };
    let assets = AssetRoot::new(env!("CARGO_MANIFEST_DIR"));
    let report = ModpackCompiler::new(&assets).audit(&data);
    assert!(!report.diagnostics.iter().any(|error| matches!(error.code.as_str(),
        "invalid_retired_gs_warp" | "unreachable_warp" | "unreachable_warp_destination")));
    data.maps.get_mut("Start").unwrap().events.warps[0].target_warp_id = 2;
    assert!(ModpackCompiler::new(&assets).audit(&data).diagnostics.iter().any(|error| error.code == "unknown_warp_index"));
    data.maps.get_mut("Start").unwrap().events.warps[0].target_warp_id = 1;
    data.tilesets.get_mut("johto").unwrap().collision.insert("2".into(), vec!["FLOOR".into(); 4]);
    assert!(ModpackCompiler::new(&assets).audit(&data).diagnostics.iter().any(|error| error.code == "invalid_retired_gs_warp"));
}
