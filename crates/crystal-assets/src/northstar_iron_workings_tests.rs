//! Preservation and malformed-source checks for the interleaved mine overlay.
use std::path::PathBuf;

#[test]
#[ignore = "requires the immutable local headquarters parent"]
fn iron_workings_preserves_existing_chapters_audio_trainers_and_recovery() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = crate::read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-ice-headquarters-v5.crystalpack"),
    )
    .expect("unbound headquarters");
    let source = root.join("planning/minnesota/lairs/iron-workings");
    let network = std::fs::read_to_string(source.join("network.json")).expect("network");
    let layouts = ["haulage", "sump", "vent"]
        .map(|name| std::fs::read_to_string(source.join(format!("{name}.json"))).expect("layout"));
    let result =
        crate::build_northstar_iron_workings(&base, &network, &layouts).expect("mine overlay");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 3);
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.audio_manifest().expect("audio") == result.audio_manifest().expect("audio"));
    assert_eq!(base.runtime_files(), result.runtime_files());
    assert_eq!(
        base.data().runtime_spawn_points,
        result.data().runtime_spawn_points
    );
    assert_eq!(
        serde_json::to_value(&base.data().trainers).expect("trainers"),
        serde_json::to_value(&result.data().trainers).expect("trainers")
    );
    for (name, map) in &base.data().maps {
        let mut before = serde_json::to_value(map).expect("original map");
        let mut after = serde_json::to_value(&result.data().maps[name]).expect("result map");
        if matches!(name.as_str(), "MnIronDrillApproach" | "MnIronPumpGallery") {
            // Only the two reciprocal port destinations and their serialized
            // warp rows change. All executable story scripts remain identical.
            for value in [&mut before, &mut after] {
                value["events"]["warps"] = serde_json::json!([]);
                if let Some(rows) = value["map_event_section_commands"].as_array_mut() {
                    rows.retain(|row| row["command"] != "warp_event");
                }
            }
        }
        assert!(before == after, "unrelated original map {name}");
    }
    assert!(crate::build_northstar_iron_workings(&result, &network, &layouts).is_err());
    let mut invalid: serde_json::Value = serde_json::from_str(&network).expect("network");
    invalid["links"][0]["b"]["active"] = serde_json::json!([999, 22]);
    assert!(crate::build_northstar_iron_workings(&base, &invalid.to_string(), &layouts).is_err());
}
