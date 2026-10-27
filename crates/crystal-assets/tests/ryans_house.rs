//! Preserve the installed story while adding a real, optional waterfall cabin.
use std::path::PathBuf;

use crystal_assets::{build_ryans_house_modpack, read_verified_compiled_game_pack};

#[test]
#[ignore = "requires the completed external Northstar drainage/ICE pack"]
fn ryans_house_preserves_everything_outside_its_declared_bank_and_new_room() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-drainage-ice.crystalpack"),
    )
    .expect("current story input");
    let paths = [
        "revisions/northwoods-expansion/after/MN_R11.layout.json",
        "routes/MN_R11.layout.json",
        "revisions/northwoods-expansion/after/MN_R11.json",
        "routes/MN_R11.json",
        "playable/ryans-house.json",
    ];
    let source = paths
        .iter()
        .map(|p| std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("source"))
        .collect::<Vec<_>>();
    let pack = build_ryans_house_modpack(
        &base, &source[0], &source[1], &source[2], &source[3], &source[4],
    )
    .expect("native cabin");
    let old = &base.data().maps["MnR11C00R00"];
    let outside = &pack.data().maps["MnR11C00R00"];
    let mut normalized = outside.clone();
    let mut expected = old.blocks.clone();
    for ([x, y], block) in [
        ([11, 9], 0x2c),
        ([12, 9], 0x2d),
        ([11, 10], 0x2e),
        ([12, 10], 0x2f),
        ([11, 11], 2),
        ([12, 11], 2),
        ([13, 11], 0x45),
    ] {
        expected[y * 32 + x] = block;
    }
    assert_eq!(outside.blocks, expected);
    assert_eq!(outside.objects, old.objects);
    assert_eq!(outside.events.coord_events, old.events.coord_events);
    assert_eq!(outside.events.warps.len(), 2);
    assert_eq!(
        outside.events.bg_events.len(),
        old.events.bg_events.len() + 1
    );
    assert_eq!(
        &outside.events.bg_events[..old.events.bg_events.len()],
        old.events.bg_events
    );
    for (label, value) in &old.scripts {
        assert_eq!(
            outside.scripts.get(label),
            Some(value),
            "original bank script {label}"
        );
    }
    normalized.blocks = old.blocks.clone();
    normalized.events = old.events.clone();
    normalized.map_event_section_commands = old.map_event_section_commands.clone();
    normalized
        .scripts
        .retain(|label, _| !label.starts_with("MN_R11_RYANS_HOUSE_SIGN"));
    normalized
        .script_text_bodies
        .retain(|label, _| !label.starts_with("MN_R11_RYANS_HOUSE_SIGN"));
    normalized
        .script_text_commands
        .retain(|c| c.source_script != "MN_R11_RYANS_HOUSE_SIGN");
    assert!(normalized == *old, "unrelated bank content changed");
    let room = &pack.data().maps["MnRyansHouse"];
    assert_eq!(room.events.warps.len(), 3);
    assert_eq!(room.objects.len(), 1);
    assert!(
        room.script_flag_commands.is_empty()
            && room.script_economy_commands.is_empty()
            && room.script_item_grants.is_empty()
    );
    for name in ["MnR11C00R00", "MnRyansHouse"] {
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(name)
                .expect("split mirror"),
            pack.data().maps[name]
        );
    }
    let mut restored = pack.data().clone();
    restored.maps.remove("MnRyansHouse");
    restored.maps.insert("MnR11C00R00".into(), old.clone());
    restored.map_blocks.remove("MnRyansHouse_Blocks");
    restored.map_blocks.insert(
        "MnR11C00R00_Blocks".into(),
        base.data().map_blocks["MnR11C00R00_Blocks"].clone(),
    );
    restored.map_dimensions.remove("MN_RYANS_HOUSE");
    restored.map_attributes.remove("MnRyansHouse");
    restored.runtime_map_metadata.remove("MN_RYANS_HOUSE");
    restored.npcs.remove("MnRyansHouse");
    restored
        .pokegear_landmarks
        .map_to_landmark
        .remove("MnRyansHouse");
    restored.map_scripts.retain(|label, _| {
        !label.starts_with("MnRyansHouse")
            && !label.starts_with("MN_RYAN")
            && !label.starts_with("MN_R11_RYANS_HOUSE_SIGN")
    });
    restored.map_scripts.insert(
        "MnR11C00R00_MapEvents".into(),
        base.data().map_scripts["MnR11C00R00_MapEvents"].clone(),
    );
    assert!(
        restored == *base.data(),
        "catalog or original content outside the cabin changed"
    );
    assert_eq!(
        pack.audio_manifest().expect("new audio"),
        base.audio_manifest().expect("old audio")
    );
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert!(
        build_ryans_house_modpack(
            &pack, &source[0], &source[1], &source[2], &source[3], &source[4]
        )
        .is_err()
    );
    let mut bad: serde_json::Value = serde_json::from_str(&source[4]).expect("house");
    bad["door"]["target_warp"] = serde_json::json!(1);
    assert!(
        build_ryans_house_modpack(
            &base,
            &source[0],
            &source[1],
            &source[2],
            &source[3],
            &bad.to_string()
        )
        .is_err(),
        "active arrival cannot substitute for passive floor"
    );
}
