//! External-pack exact preservation, native seams and unsafe-source rejection.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, connect_duluth};
use serde_json::{Value, json};

const CITY: &str = "MnDuluth";
const CHANGED: [&str; 3] = [CITY, "MnR10C01R00", "MnR11C00R03"];

fn inputs() -> &'static (CompiledGamePack, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let path = std::env::var_os("CRYSTAL_DULUTH_CONNECTION_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.join("content-packs/minnesota-northstar-expanded-base.crystalpack")
            });
        (
            crystal_assets::read_verified_compiled_game_pack(path).expect("external harbor pack"),
            std::fs::read_to_string(
                root.join("planning/minnesota/towns/MnDuluth.connections.json"),
            )
            .expect("connection source"),
        )
    })
}

#[test]
#[ignore = "requires completed external harbor/city pack"]
fn duluth_connections_preserve_content_and_emit_only_declared_native_checkpoints() {
    let (base, source) = inputs();
    let pack = connect_duluth(base, source).expect("native city connections");
    let source: Value = serde_json::from_str(source).expect("source");
    let mut restored = pack.data().clone();
    for name in CHANGED {
        let old = &base.data().maps[name];
        let map = &pack.data().maps[name];
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(name)
                .expect("split mirrors"),
            *map
        );
        let mut attributes = map.attributes.clone();
        attributes.connections = old.attributes.connections.clone();
        attributes.connection_flags = old.attributes.connection_flags.clone();
        assert_eq!(attributes, old.attributes);
        let mut blocks = old.blocks.clone();
        for p in source["regional_connections"]["patches"]
            .as_array()
            .expect("patches")
            .iter()
            .filter(|p| p["map"] == name)
        {
            let at: [usize; 2] = serde_json::from_value(p["metatile"].clone()).expect("metatile");
            let i = at[1] * usize::from(old.attributes.width) + at[0];
            assert_eq!(
                blocks[i],
                u16::from_str_radix(p["expected"].as_str().expect("expected"), 16).expect("hex")
            );
            blocks[i] =
                u16::from_str_radix(p["block"].as_str().expect("new block"), 16).expect("hex");
        }
        assert_eq!(map.blocks, blocks);
        for (label, commands) in &old.scripts {
            assert_eq!(
                map.scripts.get(label),
                Some(commands),
                "original script {label}"
            );
        }
        assert_eq!(&map.objects[..old.objects.len()], old.objects);
        if name != CITY {
            assert_eq!(map.objects, old.objects);
            assert_eq!(map.events, old.events);
        } else {
            assert_eq!(map.objects.len(), old.objects.len() + 4);
            assert_eq!(map.events.warps, old.events.warps);
            let mut signs = old.events.bg_events.clone();
            let sign = signs
                .iter_mut()
                .find(|e| e.script == "MnDuluth_DULUTH_SIGN")
                .expect("existing west sign");
            assert_eq!([sign.x, sign.y], [4, 24]);
            [sign.x, sign.y] = [4, 22];
            assert_eq!(map.events.bg_events, signs);
            assert_eq!(
                map.events.coord_events.len(),
                old.events.coord_events.len() + 8
            );
        }
        restored.maps.insert(name.into(), old.clone());
    }
    for (name, value) in &base.data().map_attributes {
        if !CHANGED.contains(&name.as_str()) {
            assert_eq!(pack.data().map_attributes.get(name), Some(value));
        }
    }
    for (label, value) in &base.data().map_blocks {
        if ![
            "MnR10C01R00_Blocks",
            "MnR11C00R03_Blocks",
            "MnDuluth_Blocks",
        ]
        .contains(&label.as_str())
        {
            assert_eq!(pack.data().map_blocks.get(label), Some(value));
        }
    }
    for (label, value) in &base.data().map_scripts {
        if label != "MnDuluth_MapEvents" {
            assert_eq!(
                pack.data().map_scripts.get(label),
                Some(value),
                "original split label {label}"
            );
        }
    }
    for (name, value) in &base.data().npcs {
        if name != CITY {
            assert_eq!(pack.data().npcs.get(name), Some(value));
        }
    }
    let city = &pack.data().maps[CITY];
    for (label, body) in city
        .scripts
        .iter()
        .filter(|(k, _)| !base.data().maps[CITY].scripts.contains_key(*k))
    {
        assert!(label.starts_with("MnDuluthRangeCheck") || label.starts_with("MnDuluthShoreCheck"));
        assert!(
            !body
                .as_array()
                .expect("native commands")
                .iter()
                .any(|c| matches!(
                    c["command"].as_str(),
                    Some("setevent" | "setflag" | "giveitem" | "givepoke" | "givemoney")
                )),
            "checkpoint grants state: {label}"
        );
    }
    for (label, count, event) in [
        ("MnDuluthRangeCheck", 1, "EVENT_MN_DEPOT_CLEARED"),
        ("MnDuluthShoreCheck", 8, "EVENT_MN_LIGHTHOUSE_SIGNAL"),
    ] {
        let commands = city.scripts[label].as_array().expect("check script");
        assert_eq!(
            commands
                .iter()
                .filter(|c| c["command"] == "checkflag")
                .count(),
            count
        );
        assert_eq!(
            commands[count * 2],
            json!({"command":"checkevent","args":[event]})
        );
    }
    restored.map_attributes = base.data().map_attributes.clone();
    restored.map_blocks = base.data().map_blocks.clone();
    restored.map_scripts = base.data().map_scripts.clone();
    restored.npcs = base.data().npcs.clone();
    assert!(restored == *base.data(), "unrelated catalog changed");
    assert!(pack.runtime_files() == base.runtime_files());
    assert!(pack.compiled_audio() == base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("new audio"),
        base.audio_manifest().expect("old audio")
    );
    assert!(
        connect_duluth(&pack, &source.to_string()).is_err(),
        "duplicate installation"
    );
    assert_eq!(
        city.objects
            .iter()
            .filter_map(|o| o.object_identifier.as_deref())
            .collect::<BTreeSet<_>>()
            .len(),
        city.objects.len(),
        "unique native objects"
    );
}

#[test]
#[ignore = "requires completed external harbor/city pack"]
fn duluth_connections_reject_shortcuts_misalignment_and_open_shoulders() {
    let (base, source) = inputs();
    for change in [
        "missing_badge",
        "win_only",
        "wrong_offset",
        "wrong_guard",
        "open_shoulder",
        "missing_patch",
    ] {
        let mut s: Value = serde_json::from_str(source).expect("source");
        let r = &mut s["regional_connections"];
        match change {
            "missing_badge" => {
                r["passages"][1]["badge_flags"]
                    .as_array_mut()
                    .expect("badges")
                    .remove(7);
            }
            "win_only" => {
                r["passages"][0]["required_event"] = json!("EVENT_MN_DEPOT_FOREMAN_DEFEATED")
            }
            "wrong_offset" => r["passages"][1]["offset"] = json!(0),
            "wrong_guard" => r["passages"][1]["bodies"][0] = json!([33, 5]),
            "open_shoulder" => r["patches"][0]["block"] = json!("01"),
            "missing_patch" => {
                r["patches"].as_array_mut().expect("patches").pop();
            }
            _ => unreachable!("fixed cases"),
        }
        assert!(
            connect_duluth(base, &s.to_string()).is_err(),
            "accepted {change}"
        );
    }
}
