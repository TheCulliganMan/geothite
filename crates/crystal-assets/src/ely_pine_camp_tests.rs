//! External current-pack checks; no generated pack or source is embedded.
use std::path::PathBuf;

use serde_json::Value;

use super::{MAP, hash, rebuild_ely_pine_camp};

#[test]
#[ignore = "requires CRYSTAL_ELY_PINE_CAMP_PARENT and the authored Ely source files"]
fn ely_pine_camp_preserves_anchors_and_rejects_stranded_services_or_open_edges() {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_ELY_PINE_CAMP_PARENT").expect("completed current parent"),
    )
    .expect("verified parent");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/towns/ely-pine-camp-v01");
    let before = std::fs::read_to_string(root.join("before.json")).expect("before");
    let after = std::fs::read_to_string(root.join("after.json")).expect("after");
    let activities = std::fs::read_to_string(root.join("activities.json")).expect("activities");
    let contract = std::fs::read_to_string(root.join("contract.json")).expect("contract");
    let child = rebuild_ely_pine_camp(&base, &before, &after, &activities, &contract)
        .expect("complete Ely refinement");
    assert_eq!(child.data.maps.len(), base.data.maps.len());
    assert_eq!(child.data.maps[MAP].events, base.data.maps[MAP].events);
    assert_eq!(
        child.data.maps[MAP].objects[0],
        base.data.maps[MAP].objects[0]
    );
    assert_eq!(
        child.data.maps[MAP].attributes,
        base.data.maps[MAP].attributes
    );
    assert_eq!(child.runtime_files, base.runtime_files);
    assert_eq!(child.data.trainers, base.data.trainers);
    assert_eq!(child.data.wild_encounters, base.data.wild_encounters);
    for (name, map) in &base.data.maps {
        if name != MAP {
            assert_eq!(child.data.maps[name], *map);
        }
    }
    for (label, x, y, block) in [
        ("open outer edge", 0, 8, "31"),
        ("closed north connection", 12, 0, "83"),
        ("blocked clinic return", 5, 11, "83"),
        ("broken lodge facade", 12, 6, "31"),
        ("blocked supply approach", 17, 2, "83"),
    ] {
        let mut terrain: Value = serde_json::from_str(&after).expect("source");
        let mut row = terrain["rows"][y]
            .as_str()
            .expect("row")
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        row[x] = block.into();
        terrain["rows"][y] = row.join(" ").into();
        let source = terrain.to_string();
        let mut binding: Value = serde_json::from_str(&contract).expect("binding");
        binding["source_sha256"] = hash(source.as_bytes()).into();
        assert!(
            rebuild_ely_pine_camp(&base, &before, &source, &activities, &binding.to_string())
                .is_err(),
            "{label}"
        );
    }
    let mut blocked: Value = serde_json::from_str(&activities).expect("activities");
    blocked["residents"][0]["tile"] = serde_json::json!([20, 5]);
    blocked["residents"][0]["approach"] = serde_json::json!([21, 5]);
    let blocked = blocked.to_string();
    let mut binding: Value = serde_json::from_str(&contract).expect("binding");
    binding["activities_sha256"] = hash(blocked.as_bytes()).into();
    assert!(
        rebuild_ely_pine_camp(&base, &before, &after, &blocked, &binding.to_string()).is_err(),
        "a reachable resident must not occupy the original sign approach"
    );
    let mut stale: Value = serde_json::from_str(&contract).expect("binding");
    stale["native_module_sha256"] = "stale".into();
    assert!(
        rebuild_ely_pine_camp(&base, &before, &after, &activities, &stale.to_string()).is_err()
    );
}
