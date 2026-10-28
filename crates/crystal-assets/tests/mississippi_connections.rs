//! External-source acceptance for additive river maps and exact native bridges.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_authored_river_connections, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Fixture {
    base: CompiledGamePack,
    manifest: String,
    layouts: BTreeMap<String, String>,
    bridges: String,
}
fn path(name: &str, default: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("canonical repository fixture root")
                .join(default)
        })
}
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let manifest = std::fs::read_to_string(path(
            "CRYSTAL_MISSISSIPPI_MANIFEST",
            "planning/minnesota/connections/mississippi-v02/manifest.json",
        ))
        .expect("frozen river manifest");
        let value: Value = serde_json::from_str(&manifest).expect("manifest JSON");
        let mut layouts = BTreeMap::new();
        for map in value["new_maps"].as_array().expect("new maps") {
            let name = map["layout"].as_str().expect("source path");
            layouts.entry(name.to_owned()).or_insert_with(|| {
                std::fs::read_to_string(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(name),
                )
                .expect("exact layout source")
            });
        }
        Fixture {
            base: read_verified_compiled_game_pack(
                path(
                    "CRYSTAL_MISSISSIPPI_BASE",
                    "content-packs/minnesota-twin-cities-compact-v01.crystalpack",
                )
                .canonicalize()
                .expect("canonical completed parent pack path"),
            )
            .expect("verified completed parent"),
            bridges: std::fs::read_to_string(path(
                "CRYSTAL_MISSISSIPPI_BRIDGES",
                value["bridge_inventory"]
                    .as_str()
                    .expect("bridge inventory"),
            ))
            .expect("exact bridge inventory"),
            manifest,
            layouts,
        }
    })
}
fn revised() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let f = fixture();
        build_authored_river_connections(&f.base, &f.manifest, &f.layouts, &f.bridges)
            .expect("validated additive native river pack")
    })
}
fn token(pack: &CompiledGamePack, map: &str, x: usize, y: usize) -> String {
    let module = &pack.data().maps[map];
    let block = module.blocks[y / 2 * usize::from(module.attributes.width) + x / 2];
    pack.data().tilesets[&module.attributes.tileset_name].collision[&format!("{block:02x}")]
        [y % 2 * 2 + x % 2]
        .clone()
}
fn foot(token: &str) -> &str {
    match token {
        "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK" => "FLOOR",
        "BRIDGE_NS_RAIL" | "BRIDGE_EW_RAIL" => "WALL",
        token => token,
    }
}

#[test]
#[ignore = "requires completed external compact Twin Cities parent and frozen river authoring inputs"]
fn mississippi_connections_preserves_old_story_rooms_audio_home_and_recovery() {
    let f = fixture();
    let actual = revised();
    let plan: Value = serde_json::from_str(&f.manifest).expect("plan");
    assert_eq!(
        actual.data().maps.len(),
        f.base.data().maps.len() + plan["new_maps"].as_array().expect("maps").len()
    );
    assert_eq!(
        actual.data().pokegear_landmarks.landmarks,
        f.base.data().pokegear_landmarks.landmarks,
        "no landmark quota consumed"
    );
    for row in plan["new_maps"].as_array().expect("new maps") {
        let name = row["id"].as_str().expect("river map");
        let parent =
            if name.starts_with("MnMississippiLowerWoods") || name == "MnMississippiWinonaMouth" {
                "MnWinona"
            } else if name.starts_with("MnMississippiConfluence") {
                "MnMinneapolis"
            } else {
                assert!(name.starts_with("MnMississippiUpstreamWoods"));
                "MnStCloud"
            };
        let expected = f
            .base
            .data()
            .pokegear_landmark_for_map(parent)
            .expect("native Minnesota parent");
        let actual_landmark = actual
            .data()
            .pokegear_landmark_for_map(name)
            .expect("actual new river town-map lookup");
        assert_eq!(actual_landmark, expected);
        assert!((1..=0x7f).contains(&actual_landmark.id));
        assert_eq!(
            actual.data().maps[name].attributes.location.as_deref(),
            Some(expected.constant.as_str())
        );
    }
    for (name, old) in &f.base.data().maps {
        let map = &actual.data().maps[name];
        let mut normalized = map.clone();
        normalized.blocks = old.blocks.clone();
        normalized.attributes.connections = old.attributes.connections.clone();
        normalized.attributes.connection_flags = old.attributes.connection_flags.clone();
        assert_eq!(&normalized, old, "all other typed content retained: {name}");
    }
    assert_eq!(
        actual.data().runtime_spawn_points,
        f.base.data().runtime_spawn_points
    );
    assert_eq!(
        actual.data().story_event_script_constants.global["SPAWN_HOME"],
        f.base.data().story_event_script_constants.global["SPAWN_HOME"]
    );
    assert_eq!(actual.data().audio, f.base.data().audio);
    assert_eq!(
        actual.data().encounter_slot_tables,
        f.base.data().encounter_slot_tables
    );
    for (label, program) in &f.base.data().map_scripts {
        assert_eq!(
            actual.data().map_scripts.get(label),
            Some(program),
            "old raw story {label}"
        );
    }
    for actor in plan["objects"].as_array().expect("actors") {
        let id = actor["id"].as_str().expect("actor id");
        let map = &actual.data().maps[actor["map"].as_str().expect("map")];
        assert!(
            map.trainer_scripts.contains_key(id),
            "optional native trainer program"
        );
        assert_eq!(
            actual.data().trainers.trainers[id].party.len(),
            actor["party"].as_array().expect("party").len()
        );
    }
}

#[test]
#[ignore = "requires completed external compact Twin Cities parent and frozen river authoring inputs"]
fn mississippi_connections_bridge_pixels_exact_and_foot_layer_unchanged() {
    let f = fixture();
    let actual = revised();
    let inventory: Value = serde_json::from_str(&f.bridges).expect("bridges");
    // Native collision JSON is an optional transport sidecar. The mandatory
    // compiled collision table and bridge recipe remain exact either way.
    let aliases = inventory["bridges"]
        .as_array()
        .expect("bridge inventory")
        .iter()
        .map(|bridge| bridge["tileset"].as_str().expect("native alias"))
        .collect::<BTreeSet<_>>();
    for alias in aliases {
        let file = format!("data/tilesets/{alias}.json");
        match (
            f.base.runtime_files().get(&file),
            actual.runtime_files().get(&file),
        ) {
            (Some(before), Some(after)) => {
                assert_eq!(
                    serde_json::from_slice::<Value>(before).expect("original sidecar"),
                    serde_json::to_value(&f.base.data().tilesets[alias].collision)
                        .expect("original collision")
                );
                assert_eq!(
                    serde_json::from_slice::<Value>(after).expect("derived sidecar"),
                    serde_json::to_value(&actual.data().tilesets[alias].collision)
                        .expect("derived collision")
                );
            }
            (None, Some(after)) => assert_eq!(
                serde_json::from_slice::<Value>(after).expect("appended native sidecar"),
                serde_json::to_value(&actual.data().tilesets[alias].collision)
                    .expect("derived collision")
            ),
            (None, None) => {}
            (Some(_), None) => panic!("existing collision sidecar was removed: {file}"),
        }
        for (block, collision) in &f.base.data().tilesets[alias].collision {
            assert_eq!(
                actual.data().tilesets[alias].collision.get(block),
                Some(collision),
                "original mandatory collision entry {alias}/{block}"
            );
        }
    }
    for bridge in inventory["bridges"].as_array().expect("inventory") {
        let name = bridge["map"].as_str().expect("map");
        let alias = bridge["tileset"].as_str().expect("alias");
        let file = format!("data/tilesets/{alias}_metatiles.bin");
        let old = &f.base.runtime_files()[&file];
        let new = &actual.runtime_files()[&file];
        assert!(new.starts_with(old));
        for cell in bridge["metatiles"].as_array().expect("cells") {
            let x = cell["metatile"][0].as_u64().expect("x") as usize;
            let y = cell["metatile"][1].as_u64().expect("y") as usize;
            let map = &actual.data().maps[name];
            let block = usize::from(map.blocks[y * usize::from(map.attributes.width) + x]);
            let source = cell["block_decimal"].as_u64().expect("source") as usize;
            let axis = if bridge["axis"] == "north_south" {
                "NS"
            } else {
                "EW"
            };
            let expected_collision = cell["foot_collision"]
                .as_array()
                .expect("source quadrant inventory")
                .iter()
                .map(|token| {
                    format!(
                        "BRIDGE_{axis}_{}",
                        if token == "FLOOR" {
                            "DECK"
                        } else {
                            assert_eq!(token, "WALL");
                            "RAIL"
                        }
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                actual.data().tilesets[alias].collision[&format!("{block:02x}")],
                expected_collision,
                "mandatory native bridge collision recipe"
            );
            assert_eq!(
                &new[block * 16..block * 16 + 16],
                &old[source * 16..source * 16 + 16],
                "unchanged native glyph references"
            );
            for dy in 0..2 {
                for dx in 0..2 {
                    assert_eq!(
                        foot(&token(actual, name, x * 2 + dx, y * 2 + dy)),
                        token(&f.base, name, x * 2 + dx, y * 2 + dy)
                    );
                }
            }
        }
        for suffix in [".2bpp", ".png", ".pal"] {
            let file = format!("gfx/tilesets/{alias}{suffix}");
            assert_eq!(
                actual.runtime_files().get(&file),
                f.base.runtime_files().get(&file)
            );
        }
    }
}

#[test]
#[ignore = "requires completed external compact Twin Cities parent and frozen river authoring inputs"]
fn mississippi_connections_rejects_stale_or_unsafe_manifests_and_bridge_claims() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.manifest).expect("manifest");
    let inventory: Value = serde_json::from_str(&f.bridges).expect("bridges");
    let mut cases = Vec::new();
    for (id, pointer, value) in [
        ("parent", "/base_content_hash", json!("stale")),
        ("group", "/registry/group_id", json!(80)),
        ("source", "/new_maps/0/layout_sha256", json!("stale")),
        ("reciprocal", "/new_maps/0/connections/0/offset", json!(17)),
        (
            "preimage",
            "/terrain_revisions/0/native_module_sha256",
            json!("stale"),
        ),
        (
            "anchor",
            "/terrain_revisions/0/anchor_preimages/objects/0/0",
            json!(0),
        ),
        (
            "party",
            "/objects/0/party/0/species",
            json!("NOT_A_POKEMON"),
        ),
        ("flag", "/objects/0/event_flag", json!("EVENT_GOT_STARTER")),
        (
            "weights",
            "/encounter_contracts/0/slots/0/weight",
            json!(100),
        ),
    ] {
        let mut value_source = original.clone();
        *value_source.pointer_mut(pointer).expect("contract field") = value;
        cases.push((id, value_source));
    }
    for (id, source) in cases {
        assert!(
            build_authored_river_connections(&f.base, &source.to_string(), &f.layouts, &f.bridges)
                .is_err(),
            "rejected {id}"
        );
    }
    for (id, pointer, value) in [
        (
            "bridge original",
            "/bridges/0/metatiles/0/block_decimal",
            json!(0),
        ),
        (
            "bridge collision",
            "/bridges/0/metatiles/0/foot_collision/0",
            json!("FLOOR"),
        ),
        ("bridge mouth", "/bridges/0/water_mouths_core/0/0", json!(0)),
        ("bridge width", "/bridges/0/aperture_width_core", json!(1)),
    ] {
        let mut broken = inventory.clone();
        *broken.pointer_mut(pointer).expect("bridge field") = value;
        assert!(
            build_authored_river_connections(&f.base, &f.manifest, &f.layouts, &broken.to_string())
                .is_err(),
            "rejected {id}"
        );
    }
}

#[test]
#[ignore = "requires completed external compact Twin Cities parent and frozen river authoring inputs"]
fn mississippi_connections_strict_written_roundtrip_and_uncached_new_programs() {
    let actual = revised();
    let plan: Value = serde_json::from_str(&fixture().manifest).expect("plan");
    for source in plan["new_maps"].as_array().expect("new maps") {
        let name = source["id"].as_str().expect("name");
        let mut data = actual.data().clone();
        let expected = data.maps.remove(name).expect("native new river map");
        let mut assembled = data
            .assemble_map_module_from_compiled_payloads(name)
            .expect("production uncached assembly");
        for label in [
            &expected.attributes.map_scripts_label,
            &expected.attributes.map_events_label,
        ] {
            assembled
                .scripts
                .remove(label.as_ref().expect("transport label"));
        }
        assert_eq!(assembled, expected, "all native compiled indexes: {name}");
    }
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical roundtrip repository root")
        .join(".local/mississippi-tests");
    std::fs::create_dir_all(&directory).expect("ignored scratch");
    let directory = directory
        .canonicalize()
        .expect("canonical ignored roundtrip directory");
    let output = directory.join(format!("roundtrip-{}.crystalpack", std::process::id()));
    actual
        .write_preserving_storage(&output)
        .expect("native writer");
    let checked = read_verified_compiled_game_pack(&output).expect("strict completed pack reload");
    assert_eq!(
        checked.identity().expect("identity"),
        actual.identity().expect("identity")
    );
    assert_eq!(checked.data(), actual.data());
    std::fs::remove_file(output).expect("remove task scratch");
}
