//! External-pack acceptance for the fixed full-state wooded connector pass.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, build_state_cohesion, read_verified_compiled_game_pack};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

struct Fixture {
    base: CompiledGamePack,
    contract: String,
    sources: BTreeMap<String, String>,
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(||{
        let source=std::env::var_os("CRYSTAL_STATE_COHESION_CONTRACT").map(PathBuf::from).unwrap_or_else(||root().join("planning/minnesota/connections/state-cohesion-v01/source-v01/integration.contract.json"));let contract=std::fs::read_to_string(source).expect("reviewed cohesion contract");let value:Value=serde_json::from_str(&contract).expect("JSON contract");let mut sources=BTreeMap::new();
        let patch=value["native_port_patches"].as_str().expect("patch path");sources.insert(patch.to_owned(),std::fs::read_to_string(root().join(patch)).expect("frozen terrain patch"));
        for map in value["new_maps"].as_array().expect("new maps"){for field in ["layout_file","dossier_file"]{let name=map[field].as_str().expect("source path");sources.insert(name.to_owned(),std::fs::read_to_string(root().join(name)).expect("frozen explicit source"));}}
        let base=std::env::var_os("CRYSTAL_STATE_COHESION_BASE").map(PathBuf::from).unwrap_or_else(||root().join("content-packs/minnesota-mississippi-connected-v02.crystalpack"));Fixture{base:read_verified_compiled_game_pack(base.canonicalize().expect("exact parent file")).expect("verified corrected river parent"),contract,sources}
    })
}
fn revised() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let f = fixture();
        build_state_cohesion(&f.base, &f.contract, &f.sources)
            .expect("bounded native cohesion compilation")
    })
}
fn token(pack: &CompiledGamePack, name: &str, p: [u16; 2]) -> String {
    let m = &pack.data().maps[name];
    let b =
        m.blocks[usize::from(p[1] / 2) * usize::from(m.attributes.width) + usize::from(p[0] / 2)];
    pack.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")]
        [usize::from(p[1] % 2) * 2 + usize::from(p[0] % 2)]
    .clone()
}
fn point(value: &Value) -> [u16; 2] {
    [
        u16::try_from(value[0].as_u64().expect("x")).expect("u16 x"),
        u16::try_from(value[1].as_u64().expect("y")).expect("u16 y"),
    ]
}

#[test]
#[ignore = "requires corrected external677-map Mississippi parent and frozen cohesion sources"]
fn state_cohesion_preserves_original_programs_rooms_river_audio_home_and_recovery() {
    let f = fixture();
    let actual = revised();
    let plan: Value = serde_json::from_str(&f.contract).expect("plan");
    let guarded = plan["guard_proposals"]
        .as_array()
        .expect("guards")
        .iter()
        .map(|g| g["map"].as_str().expect("guard map"))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual.data().maps.len(), 681);
    for (name, old) in &f.base.data().maps {
        let map = &actual.data().maps[name];
        assert_eq!(
            map.events.warps, old.events.warps,
            "all indexed room returns: {name}"
        );
        assert_eq!(map.scenes, old.scenes);
        assert!(
            map.objects.starts_with(&old.objects),
            "old actor order/fields retained: {name}"
        );
        assert!(
            map.events
                .coord_events
                .starts_with(&old.events.coord_events)
        );
        assert_eq!(map.events.bg_events, old.events.bg_events);
        for (label, body) in &old.scripts {
            assert_eq!(
                map.scripts.get(label),
                Some(body),
                "old raw program {label}"
            );
        }
        for (label, body) in &old.script_text_bodies {
            assert_eq!(map.script_text_bodies.get(label), Some(body));
        }
        if guarded.contains(name.as_str()) {
            let new_labels = map
                .scripts
                .keys()
                .filter(|label| !old.scripts.contains_key(*label))
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let before = serde_json::to_value(old).expect("old typed guarded module");
            let after = serde_json::to_value(map).expect("new typed guarded module");
            for field in [
                "script_control_commands",
                "script_flag_commands",
                "script_text_commands",
            ] {
                let retained = after[field]
                    .as_array()
                    .expect("typed array")
                    .iter()
                    .filter(|row| {
                        !row["source_script"]
                            .as_str()
                            .is_some_and(|source| new_labels.contains(source))
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    retained,
                    before[field]
                        .as_array()
                        .expect("old typed array")
                        .iter()
                        .collect::<Vec<_>>(),
                    "only precisely new script roots may be inserted; every old row/value/order exact: {name}/{field}"
                );
            }
        }
        if !guarded.contains(name.as_str()) {
            let mut normalized = map.clone();
            normalized.blocks = old.blocks.clone();
            normalized.attributes = old.attributes.clone();
            assert_eq!(
                &normalized, old,
                "no new program outside declared checkpoint maps: {name}"
            );
            assert_eq!(actual.data().npcs.get(name), f.base.data().npcs.get(name));
        }
        if name.starts_with("MnMississippi") {
            assert_eq!(map, old, "all17 corrected river modules exact");
        }
    }
    let event_labels = guarded
        .iter()
        .map(|name| {
            f.base.data().maps[*name]
                .attributes
                .map_events_label
                .as_deref()
                .expect("old guard MapEvents label")
        })
        .collect::<BTreeSet<_>>();
    for (label, body) in &f.base.data().map_scripts {
        if !event_labels.contains(label.as_str()) {
            assert_eq!(
                actual.data().map_scripts.get(label),
                Some(body),
                "every old global raw body exact: {label}"
            );
        }
    }
    // This specific old story module already differs between its cache and
    // native parser representation. Neither form may be silently repaired.
    let name = "MnBoundaryWaters";
    let mut before = f.base.data().clone();
    let mut after = actual.data().clone();
    before
        .maps
        .remove(name)
        .expect("old cached Boundary module");
    after.maps.remove(name).expect("new cached Boundary module");
    let mut old_parsed = before
        .assemble_map_module_from_compiled_payloads(name)
        .expect("true old uncached story");
    let new_parsed = after
        .assemble_map_module_from_compiled_payloads(name)
        .expect("true new uncached story");
    old_parsed.blocks = new_parsed.blocks.clone();
    old_parsed.attributes.connections = new_parsed.attributes.connections.clone();
    old_parsed.attributes.connection_flags = new_parsed.attributes.connection_flags.clone();
    assert_eq!(
        old_parsed, new_parsed,
        "inherited uncached Boundary story preserved with geometry only"
    );
    assert_eq!(
        actual.data().runtime_spawn_points,
        f.base.data().runtime_spawn_points
    );
    assert_eq!(
        actual.data().story_event_script_constants.global["SPAWN_HOME"],
        f.base.data().story_event_script_constants.global["SPAWN_HOME"]
    );
    assert_eq!(actual.runtime_files(), f.base.runtime_files());
    assert_eq!(actual.data().audio, f.base.data().audio);
    assert_eq!(
        actual.data().pokegear_landmarks.landmarks,
        f.base.data().pokegear_landmarks.landmarks
    );
    assert_eq!(
        actual.data().encounter_slot_tables,
        f.base.data().encounter_slot_tables
    );
    for (flag, value) in &f.base.data().story_event_script_constants.global {
        assert_eq!(
            actual.data().story_event_script_constants.global.get(flag),
            Some(value),
            "old flag/constant exact"
        );
    }
}

#[test]
#[ignore = "requires corrected external677-map Mississippi parent and frozen cohesion sources"]
fn state_cohesion_all13_two_lane_pairs_use_actual_reciprocal_native_projection() {
    let pack = revised();
    let plan: Value = serde_json::from_str(&fixture().contract).expect("plan");
    let links = plan["new_reciprocal_links"].as_array().expect("links");
    assert_eq!(links.len(), 13);
    for link in links {
        for reverse in [false, true] {
            let (from, to, side, lanes, targets) = if reverse {
                (
                    link["to_map"].as_str().expect("to"),
                    link["from_map"].as_str().expect("from"),
                    match link["direction"].as_str().expect("side") {
                        "north" => "south",
                        "south" => "north",
                        "west" => "east",
                        "east" => "west",
                        _ => panic!("known side"),
                    },
                    &link["to_core"],
                    &link["from_core"],
                )
            } else {
                (
                    link["from_map"].as_str().expect("from"),
                    link["to_map"].as_str().expect("to"),
                    link["direction"].as_str().expect("side"),
                    &link["from_core"],
                    &link["to_core"],
                )
            };
            let map = &pack.data().maps[from];
            let native =
                OverworldMapData::from_attributes(from, &map.attributes, map.blocks.clone());
            for i in 0..2 {
                let a = point(&lanes[i]);
                let b = point(&targets[i]);
                assert_eq!(token(pack, from, a), "FLOOR");
                assert_eq!(token(pack, to, b), "FLOOR");
                let x = i16::try_from(a[0]).expect("x");
                let y = i16::try_from(a[1]).expect("y");
                let out = match side {
                    "north" => TilePosition::new(x, -1),
                    "south" => TilePosition::new(x, y + 1),
                    "west" => TilePosition::new(-1, y),
                    "east" => TilePosition::new(x + 1, y),
                    _ => panic!("known side"),
                };
                let t = connection_for_tile_checked(&native, out)
                    .expect("native connection")
                    .expect("installed connection");
                let target = pack
                    .data()
                    .resolve_connection_transition(&t)
                    .expect("resolved native destination")
                    .destination;
                assert_eq!(target.map_name, to);
                assert_eq!(
                    target.tile,
                    TilePosition::new(
                        i16::try_from(b[0]).expect("x"),
                        i16::try_from(b[1]).expect("y")
                    )
                );
            }
        }
    }
}

#[test]
#[ignore = "requires corrected external677-map Mississippi parent and frozen cohesion sources"]
fn state_cohesion_landmarks_real_optional_trainers_and_checkpoint_additions() {
    let actual = revised();
    let plan: Value = serde_json::from_str(&fixture().contract).expect("plan");
    for binding in plan["landmark_bindings"].as_array().expect("bindings") {
        let name = binding["map"].as_str().expect("map");
        let parent = binding["reference_native_map"]
            .as_str()
            .expect("native parent");
        let expected = fixture()
            .base
            .data()
            .pokegear_landmark_for_map(parent)
            .expect("real existing location");
        assert_eq!(
            actual
                .data()
                .pokegear_landmark_for_map(name)
                .expect("runtime caught-location lookup"),
            expected
        );
        assert_eq!(
            actual.data().maps[name].attributes.location.as_deref(),
            Some(expected.constant.as_str())
        );
    }
    for source in plan["new_maps"].as_array().expect("maps") {
        let dossier: Value = serde_json::from_str(
            &fixture().sources[source["dossier_file"].as_str().expect("dossier")],
        )
        .expect("dossier JSON");
        for a in dossier["actors"].as_array().expect("actors") {
            let id = a["id"].as_str().expect("id");
            let map = &actual.data().maps[source["id"].as_str().expect("map")];
            assert!(map.trainer_scripts.contains_key(id));
            assert_eq!(actual.data().trainers.trainers[id].party.len(), 2);
        }
    }
    for g in plan["guard_proposals"].as_array().expect("guards") {
        let map = &actual.data().maps[g["map"].as_str().expect("map")];
        let root = g["id"].as_str().expect("root");
        assert!(map.scripts.contains_key(root));
        for id in g["actor_ids"].as_array().expect("officers") {
            assert!(
                map.objects
                    .iter()
                    .any(|o| o.object_identifier.as_deref() == id.as_str())
            );
        }
        assert!(
            map.scripts[root]
                .as_array()
                .expect("native check program")
                .iter()
                .all(|c| !matches!(
                    c["command"].as_str(),
                    Some("setflag" | "setevent" | "giveitem")
                ))
        );
    }
    let m = &actual.data().maps["MnR03C00R01"];
    assert_eq!([m.attributes.width, m.attributes.height], [21, 8]);
    assert!(m.objects.iter().any(|o| [o.x, o.y] == [41, 7]));
}

#[test]
#[ignore = "requires corrected external677-map Mississippi parent and frozen cohesion sources"]
fn state_cohesion_rejects_stale_inputs_badge_downgrade_and_false_joins() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.contract).expect("plan");
    for (id, pointer, replacement) in [
        ("parent", "/base_content_hash", json!("stale")),
        ("occupied group", "/registry/group_id", json!(82)),
        (
            "stale preimage",
            "/native_preimages/0/module_sha256",
            json!("stale"),
        ),
        (
            "river preimage",
            "/river_module_preservation_preimages/0/module_sha256",
            json!("stale"),
        ),
        (
            "reciprocal offset",
            "/new_reciprocal_links/0/reverse_offset",
            json!(99),
        ),
        ("guard downgrade", "/guard_proposals/0/flags", json!([])),
        (
            "free return removed",
            "/guard_proposals/0/inbound_free",
            json!(false),
        ),
        ("bad guard body", "/guard_proposals/0/bodies/0/0", json!(7)),
        (
            "stale old edge",
            "/existing_connection_updates/0/old_offset",
            json!(4),
        ),
        ("fake town", "/new_maps/0/id", json!("MnDetroitLakes")),
        (
            "new landmark",
            "/landmark_bindings/0/add_landmark_row",
            json!(true),
        ),
        (
            "detached footprint",
            "/proposed_origins_metatiles/MnBemidjiNorthwoods/0",
            json!(999),
        ),
    ] {
        let mut broken = original.clone();
        *broken.pointer_mut(pointer).expect("tested contract field") = replacement;
        assert!(
            build_state_cohesion(&f.base, &broken.to_string(), &f.sources).is_err(),
            "reject {id}"
        );
    }
}

#[test]
#[ignore = "requires corrected external677-map Mississippi parent and frozen cohesion sources"]
fn state_cohesion_rejects_visible_water_leak_grass_bypass_and_invalid_party() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.contract).expect("plan");
    for (id, layer) in [
        (
            "unpaired water",
            json!({"name":"invalid border","kind":"point","position":[0,0],"block":"35"}),
        ),
        (
            "incidental Cut",
            json!({"name":"invalid HM","kind":"point","position":[4,9],"block":"61"}),
        ),
        (
            "free dry grass bypass",
            json!({"name":"erase intended grass","kind":"block_remap","bounds":[0,0,24,18],"from":"03","to":"01"}),
        ),
    ] {
        let mut plan = original.clone();
        let key = plan["new_maps"][0]["layout_file"]
            .as_str()
            .expect("layout")
            .to_owned();
        let mut input: Value = serde_json::from_str(&f.sources[&key]).expect("layout JSON");
        input["layers"].as_array_mut().expect("layers").push(layer);
        let text = input.to_string();
        plan["new_maps"][0]["layout_sha256"] =
            json!(format!("{:x}", Sha256::digest(text.as_bytes())));
        let mut sources = f.sources.clone();
        sources.insert(key, text);
        assert!(
            build_state_cohesion(&f.base, &plan.to_string(), &sources).is_err(),
            "reject {id}"
        );
    }
    let mut plan = original;
    let key = plan["new_maps"][0]["dossier_file"]
        .as_str()
        .expect("dossier")
        .to_owned();
    let mut dossier: Value = serde_json::from_str(&f.sources[&key]).expect("dossier");
    dossier["actors"][0]["party"][0]["species"] = json!("NOT_A_POKEMON");
    let text = dossier.to_string();
    plan["new_maps"][0]["dossier_sha256"] = json!(format!("{:x}", Sha256::digest(text.as_bytes())));
    let mut sources = f.sources.clone();
    sources.insert(key, text);
    assert!(build_state_cohesion(&f.base, &plan.to_string(), &sources).is_err());
}

#[test]
#[ignore = "requires corrected external677-map Mississippi parent and frozen cohesion sources"]
fn state_cohesion_written_roundtrip_and_uncached_mirrors() {
    let actual = revised();
    let mut data = actual.data().clone();
    let plan: Value = serde_json::from_str(&fixture().contract).expect("plan");
    for row in plan["new_maps"].as_array().expect("new maps") {
        let name = row["id"].as_str().expect("name");
        let expected = data.maps.remove(name).expect("cached native module");
        let mut assembled = data
            .assemble_map_module_from_compiled_payloads(name)
            .expect("production uncached parser");
        for label in [
            &expected.attributes.map_scripts_label,
            &expected.attributes.map_events_label,
        ] {
            assembled
                .scripts
                .remove(label.as_ref().expect("transport label"));
        }
        assert_eq!(assembled, expected, "new map fully reparses");
        data.maps.insert(name.to_owned(), expected);
    }
    let path = std::env::temp_dir().join(format!(
        "geothite-state-cohesion-{}-{}.crystalpack",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    assert!(!path.exists());
    actual
        .write_preserving_storage(&path)
        .expect("write native external pack");
    let loaded = read_verified_compiled_game_pack(path.canonicalize().expect("written file"))
        .expect("strict actual written roundtrip");
    assert_eq!(
        serde_json::to_value(loaded.data()).expect("roundtrip data"),
        serde_json::to_value(actual.data()).expect("expected data")
    );
    assert_eq!(loaded.runtime_files(), actual.runtime_files());
    std::fs::remove_file(path).expect("remove only own successful scratch pack");
}
