//! All seven field-machine sources and their native badge contracts.
use std::path::PathBuf;

use crystal_assets::{build_minnesota_hm_progression, read_verified_compiled_game_pack};

#[test]
#[ignore = "requires CRYSTAL_HM_PROGRESSION_BASE with the Minnesota campaign"]
fn hm_progression_preserves_field_rules_and_requires_each_native_badge() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_HM_PROGRESSION_BASE").expect("campaign parent"),
    ))
    .expect("verified parent");
    let result = build_minnesota_hm_progression(&base).expect("seven HMs");
    assert!(build_minnesota_hm_progression(&result).is_err());
    assert_eq!(base.data().maps.len(), result.data().maps.len());
    assert_eq!(base.data().field_moves, result.data().field_moves);
    assert_eq!(base.data().items, result.data().items);
    assert_eq!(base.data().trainers, result.data().trainers);
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.runtime_files() == result.runtime_files());
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/hm-progression.json"
    ))
    .expect("HM source");
    for gift in source.as_array().expect("seven sources") {
        let map_id = gift["map"].as_str().expect("map");
        let old = &base.data().maps[map_id];
        let new = &result.data().maps[map_id];
        assert_eq!(old.blocks, new.blocks);
        assert_eq!(old.events, new.events);
        for (label, body) in &old.scripts {
            assert_eq!(&new.scripts[label], body);
        }
        let root = format!("{}FieldHM", gift["actor"].as_str().expect("actor"));
        let body = &new.scripts[&root];
        assert_eq!(body[3]["command"], "checkflag");
        assert_eq!(body[3]["args"][0], gift["flag"]);
        assert_eq!(body[4]["command"], "iffalse");
        assert_eq!(body[6]["command"], "verbosegiveitem");
        assert_eq!(body[6]["args"][0], gift["item"]);
        assert!(
            !new.script_flag_commands
                .iter()
                .any(|command| command.source_script == root && command.command == "setevent"),
            "HM ownership must not grant story progress"
        );
    }
    let fly_points = result
        .data()
        .fly_destinations
        .iter()
        .filter(|(flag, _)| flag.starts_with("ENGINE_FLYPOINT_MN_"))
        .collect::<Vec<_>>();
    assert_eq!(fly_points.len(), 18, "every installed Minnesota hub");
    for (flag, destination) in fly_points {
        assert!(destination.destination_spawn_identifier >= 256);
        let spawn = &result.data().runtime_spawn_points
            [&destination.destination_spawn_identifier.to_string()];
        let map = &result.data().maps[&spawn.map_name];
        assert!(map.map_script_section_commands.iter().any(|command| {
            command.command == "callback"
                && command.args[0] == "MAPCALLBACK_NEWMAP"
                && map.script_flag_commands.iter().any(|set| {
                    set.source_script == command.args[1]
                        && set.command == "setflag"
                        && &set.flag_id == flag
                })
        }));
        assert!(
            !result.data().initialize_events.engine_flags.contains(flag),
            "unvisited hub must stay locked"
        );
    }
    for (key, spawn) in &base.data().runtime_spawn_points {
        assert_eq!(
            result.data().runtime_spawn_points.get(key),
            Some(spawn),
            "original home/recovery spawn {key}"
        );
    }
    let report = &result.data().maps["MnBrainerd"];
    assert_eq!(
        report.scripts["MN_BRAINERD_RANGER"][3]["args"][0],
        "ENGINE_MINERALBADGE"
    );
    assert_eq!(
        report.scripts["MnBrainerdRangerSurfAlreadyOwned"][0]["args"][0],
        "EVENT_MN_BRAINERD_REPORT"
    );
}
