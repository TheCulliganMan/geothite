//! External current-parent publication, preservation and hostile-source checks.

use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_assets::{build_north_shore_shops, read_verified_compiled_game_pack};
use serde_json::Value;

#[test]
#[ignore = "requires CRYSTAL_NORTH_SHORE_SHOPS_PARENT and the local authoring contract"]
fn north_shore_shops_preserve_parent_and_reject_blocked_habitat_and_stale_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source =
        std::fs::read_to_string(root.join("planning/minnesota/landmarks/north-shore-shops.json"))
            .expect("shop source");
    let base = read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_NORTH_SHORE_SHOPS_PARENT").expect("completed current parent"),
    )
    .expect("verified parent");
    let result = build_north_shore_shops(&base, &source).expect("two enterable native shops");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 2);
    let owned = BTreeSet::from(["MnR11C00R01", "MnR11C00R03"]);
    for (name, map) in &base.data().maps {
        if !owned.contains(name.as_str()) {
            assert_eq!(
                result.data().maps.get(name),
                Some(map),
                "unrelated map {name}"
            );
        } else {
            let new = &result.data().maps[name];
            let mut residual = new.clone();
            residual.blocks = map.blocks.clone();
            residual.events = map.events.clone();
            residual.map_event_section_commands = map.map_event_section_commands.clone();
            residual
                .scripts
                .retain(|label, _| !label.starts_with("MN_NORTH_SHORE_"));
            residual
                .script_text_bodies
                .retain(|label, _| !label.starts_with("MN_NORTH_SHORE_"));
            residual
                .script_text_commands
                .retain(|command| !command.source_script.starts_with("MN_NORTH_SHORE_"));
            assert_eq!(
                &residual, map,
                "all existing coast programs and actors remain exact"
            );
            assert_eq!(
                &new.events.warps[..map.events.warps.len()],
                &map.events.warps
            );
            assert_eq!(
                &new.events.bg_events[..map.events.bg_events.len()],
                &map.events.bg_events
            );
            assert_eq!(new.events.coord_events, map.events.coord_events);
        }
    }
    assert_eq!(result.runtime_files(), base.runtime_files());
    assert_eq!(result.data().audio, base.data().audio);
    assert_eq!(result.data().wild_encounters, base.data().wild_encounters);
    assert_eq!(result.data().trainers, base.data().trainers);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(
        result.data().story_event_script_constants,
        base.data().story_event_script_constants
    );
    assert_eq!(result.data().tilesets, base.data().tilesets);

    let plan: Value = serde_json::from_str(&source).expect("JSON");
    for change in ["parent", "preimage", "grass", "door", "arrival", "identity"] {
        let mut bad = plan.clone();
        match change {
            "parent" => bad["parent_content_hash"] = "stale".into(),
            "preimage" => bad["shops"][0]["cells"][0]["before"] = "ff".into(),
            "grass" => {
                bad["shops"][0]["grass_bounds_core"][0] = serde_json::json!([30, 32, 32, 34])
            }
            "door" => bad["shops"][0]["door"] = serde_json::json!([31, 32]),
            "arrival" => {
                bad["shops"][0]["room"]["passive_arrival"]["tile"] = serde_json::json!([2, 7])
            }
            "identity" => bad["shops"][0]["native_map_id"] = 10.into(),
            _ => unreachable!(),
        }
        assert!(
            build_north_shore_shops(&base, &bad.to_string()).is_err(),
            "reject {change}"
        );
    }
}
