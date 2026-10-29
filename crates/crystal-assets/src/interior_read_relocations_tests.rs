//! Event-only module repair, without publishing the historical exterior branch.
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::interior_read_relocations::{self, ReadRelocation};
use crate::read_verified_compiled_game_pack;

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_LAYOUT_BASE; historical room fixture only"]
fn interior_read_relocations_preserve_modules_and_reject_unsafe_targets() {
    let base = read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_INTERIOR_LAYOUT_BASE").expect("historical room input"),
    ))
    .expect("strict input");
    let source = include_str!("../../../planning/minnesota/interior-life/readable-details.json");
    let plan: Value = serde_json::from_str(source).expect("authoring");
    let mut repaired = base.clone();
    for room in plan["rooms"].as_array().expect("rooms") {
        let name = room["map"].as_str().expect("room name");
        let mut module = base.data().maps[name].clone();
        let arrival: [u16; 2] = serde_json::from_value(room["arrival"].clone()).expect("arrival");
        let reads: Vec<ReadRelocation> =
            serde_json::from_value(room["read_relocations"].clone()).expect("read anchors");
        interior_read_relocations::apply(&base, &mut module, arrival, &reads)
            .expect("module repair");
        authoring::refresh_event_section(&mut module);
        crate::authored_town_publication::replace(&mut repaired, module)
            .expect("native wire repair");
    }
    assert!(base.runtime_files() == repaired.runtime_files());
    assert!(base.compiled_audio() == repaired.compiled_audio());
    let (before, after) = (base.data(), repaired.data());
    for (name, old) in &before.maps {
        let new = &after.maps[name];
        if !matches!(name.as_str(), "MnBrainerdOutfitter" | "MnStaplesStation") {
            assert!(old == new, "unrelated map: {name}");
            continue;
        }
        let mut restored = new.clone();
        restored.events.bg_events = old.events.bg_events.clone();
        restored.map_event_section_commands = old.map_event_section_commands.clone();
        assert!(
            restored == *old,
            "only background anchors and wire rows move"
        );
    }
    let mut old_data = serde_json::to_value(before).expect("parent data");
    let mut new_data = serde_json::to_value(after).expect("repaired data");
    for key in ["maps", "npcs", "map_scripts"] {
        old_data.as_object_mut().expect("data object").remove(key);
        new_data.as_object_mut().expect("data object").remove(key);
    }
    assert!(old_data == new_data, "all other game content remains exact");
    assert_eq!(before.npcs.len(), after.npcs.len());
    for (name, old) in &before.npcs {
        if old != &after.npcs[name] {
            assert!(name == "MnBrainerdOutfitter" || name == "MnStaplesStation");
        }
    }
    assert_eq!(before.map_scripts.len(), after.map_scripts.len());
    for (label, old) in &before.map_scripts {
        if old != &after.map_scripts[label] {
            assert!(
                label == "MnBrainerdOutfitter_MapEvents" || label == "MnStaplesStation_MapEvents",
                "unrelated program: {label}"
            );
        }
    }
    let original = plan["rooms"][1]["read_relocations"][0].clone();
    for (field, value) in [
        ("before", json!([4, 10])),
        ("tile", json!([4, 11])),
        ("tile", json!([4, 12])),
        ("tile", json!([999, 999])),
        ("tile", json!([11, 9])),
        ("approach", json!([4, 12])),
        ("approach", json!([4, 10])),
        ("script", json!("MN_STAPLES_CONDUCTOR")),
    ] {
        let mut invalid = original.clone();
        invalid[field] = value;
        let relocation: ReadRelocation = serde_json::from_value(invalid).expect("typed case");
        let mut map = before.maps["MnStaplesStation"].clone();
        assert!(
            interior_read_relocations::apply(&base, &mut map, [9, 15], &[relocation]).is_err(),
            "unsafe {field} accepted"
        );
    }
    let repeated: Vec<ReadRelocation> =
        serde_json::from_value(json!([original, original])).expect("duplicate case");
    let mut map = before.maps["MnStaplesStation"].clone();
    assert!(interior_read_relocations::apply(&base, &mut map, [9, 15], &repeated).is_err());
    assert!(
        crate::minnesota_interior_lineage::require_compact_coast(&base).is_err(),
        "the historical fixture must not become a new published pack"
    );
}
