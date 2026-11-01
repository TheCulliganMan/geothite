//! External-pack publication, exact lowering and unsafe-source rejection.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, build_grand_rapids_modpack};
use serde_json::{Value, json};

fn inputs() -> &'static (CompiledGamePack, String, String, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let path = std::env::var_os("CRYSTAL_GRAND_RAPIDS_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.join("content-packs/minnesota-northstar-cities-base.crystalpack")
            });
        (
            crystal_assets::read_verified_compiled_game_pack(path).expect("external review pack"),
            std::fs::read_to_string(root.join("planning/minnesota/towns/MnGrandRapids.json"))
                .expect("town source"),
            std::fs::read_to_string(
                root.join("planning/minnesota/towns/MnGrandRapids.layout.json"),
            )
            .expect("layout source"),
            std::fs::read_to_string(root.join("planning/minnesota/story.json"))
                .expect("canonical story"),
        )
    })
}

#[test]
#[ignore = "requires completed external story/geography pack"]
fn grand_rapids_preserves_source_and_lowers_only_acknowledged_handover() {
    let (base, town, layout, story) = inputs();
    let pack = build_grand_rapids_modpack(base, town, layout, story).expect("native Grand Rapids");
    let names = [
        "MnGrandRapids",
        "MnGrandRapidsPokecenter1F",
        "MnGrandRapidsMart",
        "MnGrandRapidsSurveyDepot",
    ];
    let mut preserved = pack.data().clone();
    for name in names {
        preserved.maps.remove(name);
        let map = &pack.data().maps[name];
        let constant = map
            .attributes
            .map_constant
            .as_deref()
            .expect("map constant");
        preserved.runtime_map_metadata.remove(constant);
        preserved.map_dimensions.remove(constant);
        preserved.map_attributes.remove(name);
        preserved
            .map_blocks
            .remove(map.attributes.blocks_label.as_deref().expect("block label"));
        for label in map.scripts.keys().map(String::as_str).chain([
            map.attributes
                .map_scripts_label
                .as_deref()
                .expect("script section"),
            map.attributes
                .map_events_label
                .as_deref()
                .expect("event section"),
        ]) {
            assert!(
                !base.data().map_scripts.contains_key(label),
                "new script collides with source {label}"
            );
            preserved.map_scripts.remove(label);
        }
        preserved.npcs.remove(name);
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(name)
                .expect("native reconstruction"),
            pack.data().maps[name]
        );
        assert!(pack.data().maps[name].attributes.connections.is_empty());
    }
    // Publication's only non-map catalogs are additive constants/landmark,
    // mart and two new recovery records. Every original entry stays exact.
    for (id, value) in &base.data().story_event_script_constants.global {
        assert_eq!(
            pack.data().story_event_script_constants.global.get(id),
            Some(value)
        );
    }
    for (id, value) in &base.data().runtime_spawn_points {
        assert_eq!(pack.data().runtime_spawn_points.get(id), Some(value));
    }
    for (id, value) in &base.data().marts.0 {
        assert_eq!(pack.data().marts.0.get(id), Some(value));
    }
    assert_eq!(
        pack.data().runtime_spawn_points.len(),
        base.data().runtime_spawn_points.len() + 2
    );
    preserved.story_event_script_constants = base.data().story_event_script_constants.clone();
    preserved.pokegear_landmarks = base.data().pokegear_landmarks.clone();
    preserved.runtime_spawn_points = base.data().runtime_spawn_points.clone();
    preserved.marts = base.data().marts.clone();
    if preserved != *base.data() {
        let old = serde_json::to_value(base.data()).expect("old dataset");
        let new = serde_json::to_value(&preserved).expect("new dataset");
        let changed = old
            .as_object()
            .expect("dataset object")
            .keys()
            .filter(|k| old[*k] != new[*k])
            .collect::<Vec<_>>();
        assert!(
            changed.is_empty(),
            "changed unrelated dataset fields: {changed:?}"
        );
    }
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("new audio"),
        base.audio_manifest().expect("old audio")
    );
    let room = &pack.data().maps["MnGrandRapidsSurveyDepot"];
    let source: Value = serde_json::from_str(story).expect("story");
    let milestone = source["milestones"]
        .as_array()
        .expect("milestones")
        .iter()
        .find(|m| m["id"] == "MS09")
        .expect("handover");
    for (label, expected) in milestone["native_texts"]
        .as_object()
        .expect("canonical text")
    {
        let lines = room.scripts[label]
            .as_array()
            .expect("text commands")
            .iter()
            .filter(|c| matches!(c["command"].as_str(), Some("text" | "line" | "para")))
            .map(|c| {
                c["args"][0]
                    .as_str()
                    .expect("text argument")
                    .trim_matches('"')
            })
            .collect::<Vec<_>>();
        assert_eq!(json!(lines), *expected, "canonical handover text {label}");
    }
    let root = &room.scripts["MN_DEPOT_RECIPIENT"];
    assert_eq!(
        root[3],
        json!({"command":"checkevent","args":["EVENT_MN_DEPOT_FOREMAN_DEFEATED"]})
    );
    assert_eq!(
        root[5],
        json!({"command":"checkevent","args":["EVENT_MN_MINE_INSTRUMENT_RECOVERED"]})
    );
    assert_eq!(root[9]["command"], "yesorno");
    assert_eq!(root[12]["command"], "waitbutton");
    assert_eq!(
        root[13],
        json!({"command":"setevent","args":["EVENT_MN_DEPOT_CLEARED"]})
    );
    let setters = names
        .into_iter()
        .flat_map(|name| pack.data().maps[name].scripts.values())
        .flat_map(|body| body.as_array().expect("commands"))
        .filter(|c| c["command"] == "setevent")
        .collect::<Vec<_>>();
    assert_eq!(setters.len(), 1);
    assert_eq!(setters[0], &root[13]);
    assert!(
        build_grand_rapids_modpack(&pack, town, layout, story).is_err(),
        "duplicate install"
    );
}

#[test]
#[ignore = "requires completed external story/geography pack"]
fn grand_rapids_rejects_false_custody_and_blocked_native_anchors() {
    let (base, town, layout, story) = inputs();
    let mut changed: Value = serde_json::from_str(story).expect("story");
    changed["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS09")
        .expect("handover")["requires"] = json!(["EVENT_MN_DEPOT_FOREMAN_DEFEATED"]);
    assert!(
        build_grand_rapids_modpack(base, town, layout, &changed.to_string()).is_err(),
        "win alone cannot hand over evidence"
    );
    let mut changed: Value = serde_json::from_str(town).expect("town");
    changed["interiors"][2]["native_constant"] = json!("NEW_BARK_TOWN");
    assert!(
        build_grand_rapids_modpack(base, &changed.to_string(), layout, story).is_err(),
        "cannot overwrite an original map constant"
    );
    let mut changed: Value = serde_json::from_str(town).expect("town");
    changed["interiors"][2]["residents"]
        .as_array_mut()
        .expect("actors")
        .remove(0);
    assert!(
        build_grand_rapids_modpack(base, &changed.to_string(), layout, story).is_err(),
        "cannot publish a depot without its handover"
    );
    let mut changed: Value = serde_json::from_str(layout).expect("layout");
    changed["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({"name":"blocked clinic return","kind":"point","position":[4,6],"block":"05"}));
    assert!(build_grand_rapids_modpack(base, town, &changed.to_string(), story).is_err());
    let mut changed: Value = serde_json::from_str(town).expect("town");
    changed["interiors"][2]["residents"][0]["tile"] = json!([12, 17]);
    assert!(
        build_grand_rapids_modpack(base, &changed.to_string(), layout, story).is_err(),
        "occupied arrival"
    );
}

fn installed() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let (base, town, layout, story) = inputs();
        build_grand_rapids_modpack(base, town, layout, story).expect("actual installed depot town")
    })
}

#[test]
#[ignore = "requires completed external city input and frozen Grand Rapids pack"]
fn grand_rapids_shared_checkpoint_matches_frozen_native_publication() {
    let (_, town, _, _) = inputs();
    let pack =
        crystal_assets::connect_grand_rapids(installed(), town).expect("shared guard emission");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let frozen = crystal_assets::read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-midgame-cities.crystalpack"),
    )
    .expect("frozen checked native publication");
    assert!(
        pack.data() == frozen.data(),
        "shared checkpoint changed Grand Rapids game data"
    );
    assert!(pack.runtime_files() == frozen.runtime_files());
    assert!(pack.compiled_audio() == frozen.compiled_audio());
}

#[test]
#[ignore = "requires completed external story/geography pack"]
fn grand_rapids_connections_preserve_content_and_check_separate_custody() {
    let base = installed();
    let (_, town, _, _) = inputs();
    let pack =
        crystal_assets::connect_grand_rapids(base, town).expect("guarded reciprocal walking links");
    let changed = ["MnGrandRapids", "MnR09C01R00", "MnR10C00R00"];
    let mut preserved = pack.data().clone();
    for name in changed {
        let old = &base.data().maps[name];
        let map = &pack.data().maps[name];
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(name)
                .expect("split payload"),
            *map
        );
        let mut attrs = map.attributes.clone();
        attrs.connections = old.attributes.connections.clone();
        attrs.connection_flags = old.attributes.connection_flags.clone();
        assert_eq!(attrs, old.attributes);
        for (label, body) in &old.scripts {
            assert_eq!(
                map.scripts.get(label),
                Some(body),
                "changed existing script {label}"
            );
        }
        assert_eq!(&map.objects[..old.objects.len()], old.objects);
        if name != "MnGrandRapids" {
            assert_eq!(map.objects, old.objects);
            assert_eq!(map.events, old.events);
        }
        preserved.maps.insert(name.into(), old.clone());
    }
    for (name, value) in &base.data().map_attributes {
        if !changed.contains(&name.as_str()) {
            assert_eq!(pack.data().map_attributes.get(name), Some(value));
        }
    }
    for (name, value) in &base.data().map_blocks {
        if !["MnGrandRapids_Blocks", "MnR10C00R00_Blocks"].contains(&name.as_str()) {
            assert_eq!(pack.data().map_blocks.get(name), Some(value));
        }
    }
    for (name, value) in &base.data().map_scripts {
        if name != "MnGrandRapids_MapEvents" {
            assert_eq!(
                pack.data().map_scripts.get(name),
                Some(value),
                "changed original native label {name}"
            );
        }
    }
    for (name, value) in &base.data().npcs {
        if name != "MnGrandRapids" {
            assert_eq!(pack.data().npcs.get(name), Some(value));
        }
    }
    preserved.map_attributes = base.data().map_attributes.clone();
    preserved.map_blocks = base.data().map_blocks.clone();
    preserved.map_scripts = base.data().map_scripts.clone();
    preserved.npcs = base.data().npcs.clone();
    assert!(
        preserved == *base.data(),
        "changed unrelated data outside the declared map payloads"
    );
    assert!(
        pack.runtime_files() == base.runtime_files(),
        "changed original files"
    );
    assert!(
        pack.compiled_audio() == base.compiled_audio(),
        "changed original audio"
    );
    assert_eq!(
        pack.audio_manifest().expect("new audio manifest"),
        base.audio_manifest().expect("old audio manifest")
    );
    let native = &pack.data().maps["MnGrandRapids"];
    assert_eq!(
        native.objects.len(),
        base.data().maps["MnGrandRapids"].objects.len() + 4
    );
    assert_eq!(native.events.coord_events.len(), 8);
    for (root, event) in [
        ("MnGrandRapidsWestCheck", "EVENT_MN_HEADWATERS_REPORT"),
        ("MnGrandRapidsEastCheck", "EVENT_MN_DEPOT_CLEARED"),
    ] {
        assert_eq!(
            native.scripts[root][0],
            json!({"command":"checkflag","args":["ENGINE_GLACIERBADGE"]})
        );
        assert_eq!(
            native.scripts[root][2],
            json!({"command":"checkevent","args":[event]})
        );
    }
    let added = native.scripts.iter().filter(|(name, _)| {
        !base.data().maps["MnGrandRapids"]
            .scripts
            .contains_key(*name)
    });
    for (name, body) in added {
        assert!(
            !body.as_array().expect("commands").iter().any(|c| matches!(
                c["command"].as_str(),
                Some("setevent" | "giveitem" | "setflag" | "givepoke")
            )),
            "checkpoint grants progression: {name}"
        );
    }
    assert!(
        crystal_assets::connect_grand_rapids(&pack, town).is_err(),
        "duplicate install"
    );
}

#[test]
#[ignore = "requires completed external story/geography pack"]
fn grand_rapids_connections_reject_victory_shortcut_and_occupied_guard_plane() {
    let (base, town, layout, story) = inputs();
    let mut changed: Value = serde_json::from_str(town).expect("town");
    changed["regional_connections"]["passages"][1]["required_event"] =
        json!("EVENT_MN_DEPOT_FOREMAN_DEFEATED");
    assert!(
        crystal_assets::connect_grand_rapids(installed(), &changed.to_string()).is_err(),
        "victory alone cannot release R10"
    );
    let mut changed: Value = serde_json::from_str(town).expect("town");
    changed["regional_connections"]["patches"]
        .as_array_mut()
        .expect("patches")
        .remove(2);
    assert!(
        crystal_assets::connect_grand_rapids(installed(), &changed.to_string()).is_err(),
        "guard throat must be closed"
    );
    let mut changed: Value = serde_json::from_str(town).expect("town");
    changed["residents"][0]["tile"] = json!([44, 24]);
    let occupied = build_grand_rapids_modpack(base, &changed.to_string(), layout, story)
        .expect("ordinary actor on valid town floor");
    let error = crystal_assets::connect_grand_rapids(&occupied, &changed.to_string())
        .expect_err("original actor occupies checkpoint plane");
    assert!(
        format!("{error:#}").contains("overlaps original interaction"),
        "wrong rejection: {error:#}"
    );
}
