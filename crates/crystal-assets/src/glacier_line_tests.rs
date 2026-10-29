//! Native tables, reciprocal bindings, exact preservation and unsafe rejections.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::glacier_line_service::{SERVICE, ServiceSource};
use crate::{build_glacier_line_service, read_verified_compiled_game_pack};

fn source() -> String {
    let path = std::env::var_os("CRYSTAL_GLACIER_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/transit/glacier-service-v01.json")
        });
    std::fs::read_to_string(path).expect("authored five-stop contract")
}

#[test]
#[ignore = "requires the exact CRYSTAL_GLACIER_BASE external parent"]
fn glacier_line_native_tables_twenty_bindings_roundtrip_preservation_and_rejections() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_GLACIER_BASE").expect("completed current parent"),
    ))
    .expect("strict parent");
    let text = source();
    let design: ServiceSource = serde_json::from_str(&text).expect("station anchors");
    let output = build_glacier_line_service(&base, &text).expect("guarded service publication");
    if let Some(path) = std::env::var_os("CRYSTAL_GLACIER_PACK") {
        let child = read_verified_compiled_game_pack(PathBuf::from(path))
            .expect("published service candidate");
        assert!(
            output == child,
            "current Rust source must reproduce the published native candidate"
        );
    }
    let mut rides = 0;
    for (origin, stop) in design.stations.iter().enumerate() {
        let room = &output.data.maps[&stop.map];
        assert_eq!(
            room,
            &output
                .data
                .assemble_map_module_from_compiled_payloads(&stop.map)
                .expect("complete typed/raw native station reconstruction")
        );
        let destinations: Vec<_> = design
            .stations
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != origin)
            .collect();
        for mask in 0..16 {
            let label = format!("{}Menu{mask}", stop.prefix());
            let menu = &room.script_menu_definitions[&format!("{label}Data")];
            let labels: Vec<_> = menu
                .commands
                .iter()
                .filter_map(|c| c.args.first().filter(|s| s.starts_with('"')))
                .collect();
            assert_eq!(labels.len(), 5, "stable four destinations plus CANCEL");
            for (row, (_, destination)) in destinations.iter().enumerate() {
                let expected = format!(
                    "\"{}{}@\"",
                    destination.menu_label,
                    if mask & (1 << row) == 0 { " X" } else { "" }
                );
                assert_eq!(
                    labels[row], &expected,
                    "correct unavailable marking and line order"
                );
            }
            assert_eq!(labels[4].as_str(), "\"CANCEL@\"");
            if mask > 0 {
                assert_eq!(
                    room.script_vertical_menus[&format!("{label}:1")]
                        .options
                        .len(),
                    5
                );
            } else {
                assert_eq!(
                    room.scripts[&label][0]["command"], "sjump",
                    "no empty ticket menu"
                );
            }
        }
        for (index, destination) in destinations {
            let label = format!("{}Ride{index}", stop.prefix());
            let body = room.scripts[&label]
                .as_array()
                .expect("explicit ride program");
            assert_eq!(body[0], json!({"command":"checkevent","args":[SERVICE]}));
            assert_eq!(body[1]["command"], "iffalse");
            assert_eq!(
                body[2],
                json!({"command":"checkevent","args":[destination.visited_flag]})
            );
            assert_eq!(body[3]["command"], "iffalse");
            let warp = room
                .script_map_commands
                .iter()
                .find(|c| c.source_script == label && c.command == "warpfacing")
                .expect("typed destination");
            assert_eq!(warp.target_map.as_deref(), Some(destination.map.as_str()));
            assert_eq!(
                [warp.x.expect("x"), warp.y.expect("y")],
                destination.ride_arrival_core
            );
            assert_eq!(warp.facing.as_deref(), Some("DOWN"));
            assert_eq!(
                body[warp.command_index + 1],
                json!({"command":"newloadmap","args":["MAPSETUP_TRAIN"]})
            );
            let animation = body
                .iter()
                .position(|c| c["command"] == "special")
                .expect("native train");
            assert_eq!(body[animation]["args"], json!(["MagnetTrain"]));
            assert_eq!(
                body[animation - 1]["args"],
                json!([if index < origin { "TRUE" } else { "FALSE" }])
            );
            assert!(animation > 3 && animation < warp.command_index);
            rides += 1;
        }
    }
    assert_eq!(rides, 20);
    let dir = std::env::temp_dir().join(format!(
        "geothite-glacier-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&dir).expect("owned verification folder");
    let path = dir.join("service.crystalpack");
    output
        .write_preserving_storage(&path)
        .expect("actual native serialization");
    let loaded = read_verified_compiled_game_pack(&path).expect("strict written pack");
    assert!(
        output == loaded,
        "complete written service equals the Rust build"
    );
    std::fs::remove_file(path).expect("remove owned scratch pack");
    std::fs::remove_dir(dir).expect("remove owned scratch folder");

    let input: Value = serde_json::from_str(&text).expect("source JSON");
    let negatives = [
        ("stale parent", "/base_content_hash", json!("stale")),
        (
            "Q06 is not passenger authority",
            "/service_event",
            json!("EVENT_MN_DEPOT_CLEARED"),
        ),
        (
            "Red Wing is excluded",
            "/stations/4/map",
            json!("MnRedWingStation"),
        ),
        (
            "wrong destination visit",
            "/stations/0/visited_flag",
            json!("EVENT_MN_STATION_STAPLES_VISITED"),
        ),
        (
            "arrival on conductor",
            "/stations/0/ride_arrival_core",
            json!([13, 9]),
        ),
        (
            "arrival in train door",
            "/stations/0/ride_arrival_core",
            json!([6, 5]),
        ),
        (
            "missing indexed arrival",
            "/stations/0/public_arrival_warp",
            json!(99),
        ),
        (
            "changed native rows",
            "/stations/0/blocks_sha256",
            json!("unreviewed"),
        ),
        (
            "unbound city door",
            "/stations/0/parent_entry_warp",
            json!(99),
        ),
        (
            "missing public exit",
            "/stations/0/exit_warp_indices",
            json!([]),
        ),
        (
            "boarding misses train",
            "/stations/0/player_board_steps",
            json!(["UP"]),
        ),
        (
            "conductor not restored",
            "/stations/0/conductor_restore_steps",
            json!([]),
        ),
    ];
    for (name, pointer, value) in negatives {
        let mut bad = input.clone();
        *bad.pointer_mut(pointer).expect("existing contract field") = value;
        assert!(
            build_glacier_line_service(&base, &bad.to_string()).is_err(),
            "accepted {name}"
        );
    }
}
