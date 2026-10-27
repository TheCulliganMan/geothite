//! Verify the installed preserve expansion against completed external content.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, expand_boundary_safari_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    before: String,
    after: String,
    zone: String,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}
fn source(key: &str, name: &str) -> String {
    std::fs::read_to_string(
        std::env::var_os(key)
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join(name)),
    )
    .expect("frozen preserve authoring input")
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let base = read_verified_compiled_game_pack(std::env::var_os("CRYSTAL_SAFARI_EXPANSION_PACK")
            .map(PathBuf::from).unwrap_or_else(||root().join("content-packs/minnesota-cities-review.crystalpack")))
            .expect("completed installed preserve baseline");
        let before = source("CRYSTAL_SAFARI_EXPANSION_PREVIOUS_LAYOUT","planning/minnesota/revisions/northwoods-expansion/before/boundary-waters-safari.layout.json");
        let after = source("CRYSTAL_SAFARI_EXPANSION_LAYOUT","planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.layout.json");
        let zone = source("CRYSTAL_SAFARI_EXPANSION_ZONE","planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.json");
        let result = expand_boundary_safari_modpack(&base,&before,&after,&zone).expect("native preserve expansion");
        Inputs{base,result,before,after,zone}
    })
}

#[test]
#[ignore = "requires completed external preserve pack and frozen northern authoring inputs"]
fn safari_expansion_preserves_unrelated_content_and_existing_service_programs() {
    let i = inputs();
    let before = i.base.data();
    let after = i.result.data();
    assert_eq!(after.maps.len(), before.maps.len() + 2);
    for (name, module) in &before.maps {
        if name.starts_with("MnBoundarySafariC") {
            continue;
        }
        let mut candidate = after.maps[name].clone();
        if name == "MnBoundaryRootBeerCabin" {
            candidate.events.warps = module.events.warps.clone();
            candidate.map_event_section_commands = module.map_event_section_commands.clone();
        }
        assert_eq!(&candidate, module, "unrelated executable map drift: {name}");
    }
    assert_eq!(before.runtime_spawn_points, after.runtime_spawn_points);
    assert_eq!(
        before.story_event_script_constants,
        after.story_event_script_constants
    );
    assert_eq!(before.pokemon, after.pokemon);
    assert_eq!(before.items, after.items);
    assert_eq!(before.moves, after.moves);
    assert_eq!(before.marts, after.marts);
    assert_eq!(before.trainers, after.trainers);
    assert_eq!(before.wild_encounters, after.wild_encounters);
    assert_eq!(before.field_moves, after.field_moves);
    assert_eq!(i.base.compiled_audio(), i.result.compiled_audio());
    assert_eq!(
        i.base.audio_manifest().expect("base audio"),
        i.result.audio_manifest().expect("expanded audio")
    );
    let own = BTreeSet::from([
        "MnBoundarySafariC00R00_MapEvents",
        "MnBoundarySafariC01R00_MapEvents",
        "MnBoundarySafariC00R01_MapEvents",
        "MnBoundarySafariC01R01_MapEvents",
        "MnBoundaryRootBeerCabin_MapEvents",
    ]);
    for (label, payload) in &before.map_scripts {
        if !own.contains(label.as_str()) {
            assert_eq!(
                after.map_scripts.get(label),
                Some(payload),
                "script drift: {label}"
            );
        }
    }
    for (path, payload) in i.base.runtime_files() {
        if path.contains("mn_boundary_safari") {
            continue;
        }
        assert_eq!(
            i.result.runtime_files().get(path),
            Some(payload),
            "unrelated art/source file drift: {path}"
        );
    }
}

#[test]
#[ignore = "requires completed external preserve pack and frozen northern authoring inputs"]
fn safari_expansion_retains_ids_and_paired_cabin_with_six_exact_clipped_chunks() {
    let i = inputs();
    let data = i.result.data();
    let layout: Value = serde_json::from_str(&i.after).expect("layout");
    let mut links = 0;
    for (name, id, width, height) in [
        ("MnBoundarySafariC00R00", 1, 32, 24),
        ("MnBoundarySafariC01R00", 2, 32, 24),
        ("MnBoundarySafariC00R01", 3, 32, 24),
        ("MnBoundarySafariC01R01", 4, 32, 24),
        ("MnBoundarySafariC00R02", 6, 32, 8),
        ("MnBoundarySafariC01R02", 7, 32, 8),
    ] {
        let module = &data.maps[name];
        let metadata = data.runtime_map_metadata_for_name(name).expect("metadata");
        assert_eq!(
            (
                metadata.group_id,
                metadata.map_id,
                metadata.width,
                metadata.height
            ),
            (61, id, width, height)
        );
        assert_eq!(module.blocks.len(), usize::from(width * height));
        links += module.attributes.connections.len();
        for edge in &module.attributes.connections {
            assert_eq!(edge.offset, 0);
            let reverse = match edge.direction.as_str() {
                "west" => "east",
                "east" => "west",
                "north" => "south",
                "south" => "north",
                _ => panic!("unexpected direction"),
            };
            assert!(
                data.maps[&edge.target_map]
                    .attributes
                    .connections
                    .iter()
                    .any(|back| back.direction == reverse
                        && back.target_map == name
                        && back.offset == 0)
            );
        }
    }
    assert_eq!(links, 14);
    assert_eq!(
        data.runtime_map_metadata_for_name("MnBoundaryRootBeerCabin")
            .expect("cabin")
            .map_id,
        5
    );
    let door = &layout["root_beer_lady"]["door"]["tile"];
    let x = door[0].as_u64().expect("x");
    let y = door[1].as_u64().expect("y");
    let exterior = format!("MnBoundarySafariC{:02}R{:02}", x / 64, y / 48);
    assert_eq!(data.maps[&exterior].events.warps[0].target_warp_id, 3);
    assert_eq!(
        data.maps[&exterior].events.warps[0].x,
        u16::try_from(x % 64).expect("x")
    );
    assert!(
        data.maps["MnBoundaryRootBeerCabin"]
            .events
            .warps
            .iter()
            .all(|warp| warp.target_map_constant
                == data.maps[&exterior]
                    .attributes
                    .map_constant
                    .as_deref()
                    .expect("constant")
                && warp.target_warp_id == 2)
    );
}

#[test]
#[ignore = "requires completed external preserve pack and frozen northern authoring inputs"]
fn safari_expansion_written_pack_reassembles_all_changed_maps_without_cached_modules() {
    let i = inputs();
    let path = std::env::temp_dir().join(format!(
        "geothite-safari-expansion-{}.crystalpack",
        std::process::id()
    ));
    i.result
        .write_preserving_storage(&path)
        .expect("write actual expanded pack");
    let written = read_verified_compiled_game_pack(&path).expect("strict written pack");
    let mut split = written.data().clone();
    for name in
        written.data().maps.keys().filter(|name| {
            name.starts_with("MnBoundarySafariC") || *name == "MnBoundaryRootBeerCabin"
        })
    {
        let mut expected = split.maps.remove(name).expect("cached module");
        // Uncached assembly retains these two raw transport-section labels in
        // scripts; cached authoring stores the same commands in typed sections.
        // Add only those exact payloads to compare every executable field.
        for label in [
            &expected.attributes.map_scripts_label,
            &expected.attributes.map_events_label,
        ] {
            let label = label.as_ref().expect("owned section label");
            expected
                .scripts
                .insert(label.clone(), split.map_scripts[label].clone());
        }
        let actual = split
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached split assembly");
        let actual_json = serde_json::to_value(&actual).expect("actual fields");
        let expected_json = serde_json::to_value(&expected).expect("expected fields");
        let changed = actual_json
            .as_object()
            .expect("module")
            .iter()
            .filter_map(|(key, value)| (expected_json.get(key) != Some(value)).then_some(key))
            .collect::<Vec<_>>();
        assert!(
            changed.is_empty(),
            "typed/split mismatch {name}, fields: {changed:?}"
        );
    }
    std::fs::remove_file(path).expect("remove temporary non-render verification pack");
}

#[test]
#[ignore = "requires completed external preserve pack and frozen northern authoring inputs"]
fn safari_expansion_rejects_boundary_gate_stair_and_ledge_bypasses() {
    let i = inputs();
    let original: Value = serde_json::from_str(&i.after).expect("layout");
    for (label, point, block) in [
        ("outer Surf exit", [0, 20], "35"),
        ("broken gate", [5, 50], "05"),
        ("missing native stair", [55, 16], "71"),
        ("walkable ledge face", [17, 29], "71"),
    ] {
        let mut layout = original.clone();
        layout["layers"]
            .as_array_mut()
            .expect("layers")
            .push(json!({"kind":"point","name":label,"position":point,"block":block}));
        assert!(
            expand_boundary_safari_modpack(&i.base, &i.before, &layout.to_string(), &i.zone)
                .is_err(),
            "accepted {label}"
        );
    }
}

#[test]
#[ignore = "requires completed external preserve pack and frozen northern authoring inputs"]
fn safari_expansion_rejects_stale_actors_service_text_and_art_recipe() {
    let i = inputs();
    let original: Value = serde_json::from_str(&i.after).expect("layout");
    let mut changed = original.clone();
    changed["landmark_actors"][0]["lines"] = json!(["Changed service"]);
    assert!(
        expand_boundary_safari_modpack(&i.base, &i.before, &changed.to_string(), &i.zone).is_err()
    );
    let mut changed = original.clone();
    changed["derived_blocks"]
        .as_array_mut()
        .expect("art")
        .last_mut()
        .expect("stair")["source_block"] = json!("30");
    assert!(
        expand_boundary_safari_modpack(&i.base, &i.before, &changed.to_string(), &i.zone).is_err()
    );
    let mut changed: Value = serde_json::from_str(&i.before).expect("previous source");
    changed["landmark_actors"][0]["tile"] = json!([48, 43]);
    assert!(
        expand_boundary_safari_modpack(&i.base, &changed.to_string(), &i.after, &i.zone).is_err()
    );
}
