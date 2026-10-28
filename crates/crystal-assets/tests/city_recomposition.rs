//! Installed-pack regressions for a hand-recomposed city and stale inputs.

use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, recompose_authored_city};
use serde_json::{Value, json};

struct Fixture {
    base: CompiledGamePack,
    layout: String,
    contract: String,
}

fn path(variable: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .expect("explicit completed city recomposition fixture path")
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| Fixture {
        base: read_verified_compiled_game_pack(path("CRYSTAL_CITY_RECOMPOSITION_BASE"))
            .expect("verified installed major-city pack"),
        layout: std::fs::read_to_string(path("CRYSTAL_CITY_RECOMPOSITION_LAYOUT"))
            .expect("frozen explicit compact exterior"),
        contract: std::fs::read_to_string(path("CRYSTAL_CITY_RECOMPOSITION_CONTRACT"))
            .expect("frozen complete native relocation contract"),
    })
}

fn revised() -> &'static CompiledGamePack {
    static REVISED: OnceLock<CompiledGamePack> = OnceLock::new();
    REVISED.get_or_init(|| {
        let f = fixture();
        recompose_authored_city(&f.base, &f.layout, &f.contract).expect("safe native compact city")
    })
}

fn value(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("native fixture serialization")
}

fn normalize_water_gate(actual: &mut Value, before: &Value, contract: &Value) {
    if contract["north_water_checkpoint"] != true {
        return;
    }
    let labels = [
        "MnMPUpperPoolBuoyCheck",
        "MnMPUpperPoolBuoyRefresh",
        "MnMPUpperPoolBuoyOpen",
        "MnMPUpperPoolBuoyDone",
        "MnMPUpperPoolBuoyDenied",
        "MnMPUpperPoolBuoyDeniedText",
    ];
    let events = actual["events"]["coord_events"]
        .as_array_mut()
        .expect("coords");
    let old_count = before["events"]["coord_events"]
        .as_array()
        .expect("old coords")
        .len();
    let expected = [1, 4]
        .into_iter()
        .flat_map(|y| {
            (32..38).map(move |x| json!({"x":x,"y":y,"scene_id":"-1","script_name":labels[0]}))
        })
        .collect::<Vec<_>>();
    assert_eq!(&events[old_count..], expected);
    events.truncate(old_count);
    let sections = actual["map_script_section_commands"]
        .as_array_mut()
        .expect("script section");
    let old_sections = before["map_script_section_commands"]
        .as_array()
        .expect("old section");
    assert_eq!(&sections[..old_sections.len()], old_sections);
    assert_eq!(
        &sections[old_sections.len()..],
        [json!({
            "command":"callback", "args":["MAPCALLBACK_TILES",labels[1]],
            "command_index":old_sections.len()
        })]
    );
    sections.truncate(old_sections.len());
    let scripts = actual["scripts"].as_object_mut().expect("programs");
    assert_eq!(
        scripts.len(),
        before["scripts"].as_object().expect("old programs").len() + 6
    );
    assert_eq!(
        scripts[labels[2]],
        json!([
            {"command":"changeblock","args":["32","2","88"]},
            {"command":"changeblock","args":["34","2","53"]},
            {"command":"changeblock","args":["36","2","89"]},
            {"command":"end","args":[]}
        ])
    );
    for (label, target) in [(labels[0], labels[4]), (labels[1], labels[3])] {
        assert_eq!(
            scripts[label],
            json!([
                {"command":"checkflag","args":["ENGINE_PLAINBADGE"]},
                {"command":"iffalse","args":[target]},
                {"command":"checkflag","args":["ENGINE_FOGBADGE"]},
                {"command":"iffalse","args":[target]},
                {"command":"sjump","args":[labels[2]]}
            ])
        );
    }
    assert_eq!(scripts[labels[3]], json!([{"command":"end","args":[]}]));
    assert_eq!(
        scripts[labels[4]],
        json!([
            {"command":"opentext","args":[]},
            {"command":"writetext","args":[labels[5]]},
            {"command":"waitbutton","args":[]},
            {"command":"closetext","args":[]},
            {"command":"end","args":[]}
        ])
    );
    assert_eq!(
        scripts[labels[5]],
        json!([
            {"command":"text","args":["\"River gate closed.\""]},
            {"command":"line","args":["\"Earn CURRENT and\""]},
            {"command":"para","args":["\"CAPITOL BADGES.\""]},
            {"command":"done","args":[]}
        ])
    );
    for label in labels {
        assert!(scripts.remove(label).is_some());
    }
    let bodies = actual["script_text_bodies"]
        .as_object_mut()
        .expect("text bodies");
    assert_eq!(
        bodies.len(),
        before["script_text_bodies"]
            .as_object()
            .expect("old text")
            .len()
            + 1
    );
    assert!(bodies.remove(labels[5]).is_some());
    for (field, count) in [
        ("script_flag_commands", 4),
        ("script_control_commands", 9),
        ("script_text_commands", 4),
        ("script_block_changes", 3),
    ] {
        let old = before[field].as_array().expect("old compiled commands");
        let rows = actual[field]
            .as_array_mut()
            .expect("compiled gate commands");
        assert_eq!(rows.len(), old.len() + count, "gate command count {field}");
        assert_eq!(&rows[..old.len()], old, "old compiled programs {field}");
        for row in &rows[old.len()..] {
            assert!(labels.contains(&row["source_script"].as_str().expect("source program")));
        }
        rows.truncate(old.len());
    }
}

fn assert_art(before: &CompiledGamePack, after: &CompiledGamePack, contract: &Value) {
    let Some(block) = contract.get("waterfall_block").and_then(Value::as_u64) else {
        assert_eq!(before.runtime_files(), after.runtime_files());
        return;
    };
    assert_eq!(block, 0x99);
    let tileset = "mn_minneapolis_city";
    let metatiles = format!("data/tilesets/{tileset}_metatiles.bin");
    let graphics = format!("gfx/tilesets/{tileset}.2bpp");
    let old_blocks = &before.runtime_files()[&metatiles];
    let new_blocks = &after.runtime_files()[&metatiles];
    assert_eq!(new_blocks.len(), old_blocks.len() + 16);
    assert_eq!(&new_blocks[..old_blocks.len()], old_blocks);
    let glyph = new_blocks[old_blocks.len()];
    assert!(
        new_blocks[old_blocks.len()..]
            .iter()
            .all(|tile| *tile == glyph)
    );
    assert!(!old_blocks.contains(&glyph));
    let old = value(&before.data().tilesets[tileset]);
    let new = value(&after.data().tilesets[tileset]);
    let palette = new["palette_map"][usize::from(glyph)]
        .as_u64()
        .expect("palette");
    let slot = if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + 96
    };
    for &referenced in old_blocks {
        let color = old["palette_map"][usize::from(referenced)]
            .as_u64()
            .expect("old palette");
        let physical = if color & 8 == 0 {
            usize::from(referenced)
        } else {
            usize::from(referenced & 127) + 96
        };
        assert_ne!(
            physical, slot,
            "waterfall overwrote referenced native pixels"
        );
    }
    let source = &before.runtime_files()["data/tilesets/cave_metatiles.bin"];
    let source_glyph = source[0x2c * 16];
    let source_pixels = &before.runtime_files()["gfx/tilesets/cave.2bpp"]
        [usize::from(source_glyph) * 16..(usize::from(source_glyph) + 1) * 16];
    let mut expected = before.runtime_files()[&graphics].clone();
    expected[slot * 16..(slot + 1) * 16].copy_from_slice(source_pixels);
    assert_eq!(after.runtime_files()[&graphics], expected);
    let mut normalized = new.clone();
    normalized["collision"]
        .as_object_mut()
        .expect("collision")
        .remove("99");
    normalized["palette_map"][usize::from(glyph)] = old["palette_map"][usize::from(glyph)].clone();
    assert_eq!(normalized, old, "old native tileset definition changed");
    assert_eq!(
        new["collision"]["99"],
        value(&before.data().tilesets["cave"])["collision"]["2c"]
    );
    assert_eq!(
        palette & 7,
        value(&before.data().tilesets["cave"])["palette_map"][usize::from(source_glyph)]
            .as_u64()
            .expect("source palette")
            & 7
    );
    let allowed = [
        metatiles,
        graphics,
        format!("gfx/tilesets/{tileset}.png"),
        format!("data/tilesets/{tileset}.json"),
        format!("data/tilesets/{tileset}_palette_map.json"),
    ];
    assert_eq!(
        before.runtime_files().keys().collect::<Vec<_>>(),
        after.runtime_files().keys().collect::<Vec<_>>()
    );
    for (path, bytes) in before.runtime_files() {
        if !allowed.contains(path) {
            assert_eq!(
                &after.runtime_files()[path],
                bytes,
                "unrelated runtime file changed: {path}"
            );
        }
    }
}

#[test]
#[ignore = "requires a completed external city pack and frozen recomposition inputs"]
fn city_recomposition_preserves_rooms_programs_and_native_identities() {
    let f = fixture();
    let after = revised();
    let contract: Value = serde_json::from_str(&f.contract).expect("contract");
    let city = contract["city"].as_str().expect("city identity");
    assert_eq!(f.base.data().maps.len(), after.data().maps.len());
    assert_art(&f.base, after, &contract);
    assert_eq!(
        value(&f.base.audio_manifest().expect("original audio manifest")),
        value(&after.audio_manifest().expect("revised audio manifest"))
    );
    assert_eq!(value(f.base.report()), value(after.report()));
    for (name, old) in &f.base.data().maps {
        let before = value(old);
        let mut actual = value(&after.data().maps[name]);
        if name == city {
            normalize_water_gate(&mut actual, &before, &contract);
            actual["blocks"] = before["blocks"].clone();
            actual["attributes"]["width"] = before["attributes"]["width"].clone();
            actual["attributes"]["height"] = before["attributes"]["height"].clone();
            for link in actual["attributes"]["connections"]
                .as_array_mut()
                .expect("city seams")
            {
                let original = before["attributes"]["connections"]
                    .as_array()
                    .expect("old city seams")
                    .iter()
                    .find(|old| old["direction"] == link["direction"])
                    .expect("same original seam");
                link["offset"] = original["offset"].clone();
            }
            for (field, kind) in [
                ("objects", "object"),
                ("bg_events", "background"),
                ("coord_events", "coordinate"),
                ("warps", "warp"),
            ] {
                let (old, current) = if field == "objects" {
                    (&before[field], &mut actual[field])
                } else {
                    (&before["events"][field], &mut actual["events"][field])
                };
                let old = old.as_array().expect("old indexed anchors");
                let current = current.as_array_mut().expect("new indexed anchors");
                assert_eq!(old.len(), current.len(), "native anchor roster changed");
                for (index, anchor) in current.iter_mut().enumerate() {
                    let expected = contract["anchor_overrides"]
                        .as_array()
                        .expect("complete indexed contract")
                        .iter()
                        .find(|a| a["kind"] == kind && a["index"] == index)
                        .expect("original anchor included exactly once");
                    assert_eq!(anchor["x"], expected["after"][0]);
                    assert_eq!(anchor["y"], expected["after"][1]);
                    anchor["x"] = old[index]["x"].clone();
                    anchor["y"] = old[index]["y"].clone();
                }
            }
            // The ordinary publisher regenerates this textual event section
            // from the separately verified indexed records above.
            actual["map_event_section_commands"] = before["map_event_section_commands"].clone();
        } else if let Some(link) = contract["connections"]
            .as_array()
            .expect("contracted neighbors")
            .iter()
            .find(|link| link["target"] == name.as_str())
        {
            let reverse = actual["attributes"]["connections"]
                .as_array_mut()
                .expect("neighbor seams")
                .iter_mut()
                .find(|c| c["target_map"] == city)
                .expect("reciprocal original city seam");
            assert_eq!(reverse["offset"], -link["offset"].as_i64().expect("offset"));
            reverse["offset"] = json!(-link["old_offset"].as_i64().expect("old offset"));
            if let Some(revision) = contract
                .get("neighbor_terrain")
                .and_then(Value::as_array)
                .and_then(|revisions| {
                    revisions
                        .iter()
                        .find(|revision| revision["map"] == name.as_str())
                })
            {
                let mut expected = before["blocks"].clone();
                let width = before["attributes"]["width"]
                    .as_u64()
                    .expect("neighbor width");
                for change in revision["changes"].as_array().expect("literal river delta") {
                    let x = change["metatile"][0].as_u64().expect("neighbor x");
                    let y = change["metatile"][1].as_u64().expect("neighbor y");
                    let index = usize::try_from(y * width + x).expect("native block address");
                    assert_eq!(expected[index], change["before"]);
                    expected[index] = change["after"].clone();
                }
                assert_eq!(actual["blocks"], expected, "neighbor delta differs");
                actual["blocks"] = before["blocks"].clone();
            }
        }
        assert_eq!(actual, before, "unexpected native module drift in {name}");
    }
    let mut data = value(after.data());
    let mut original = value(f.base.data());
    if contract
        .get("waterfall_block")
        .and_then(Value::as_u64)
        .is_some()
    {
        data["tilesets"]["mn_minneapolis_city"] =
            original["tilesets"]["mn_minneapolis_city"].clone();
    }
    // These split indexes are derived from the contracted maps/spawns and get
    // independent written-pack readback coverage. All other catalogues match.
    for field in [
        "maps",
        "map_attributes",
        "map_dimensions",
        "map_blocks",
        "npcs",
        "map_scripts",
        "runtime_map_metadata",
        "runtime_spawn_points",
    ] {
        data.as_object_mut().expect("dataset").remove(field);
        original
            .as_object_mut()
            .expect("original dataset")
            .remove(field);
    }
    assert_eq!(data, original, "non-map catalogues changed");
}

#[test]
#[ignore = "requires a completed external city pack and frozen recomposition inputs"]
fn city_recomposition_rejects_stale_or_incomplete_native_contracts() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.contract).expect("contract");
    for case in [
        "module",
        "missing",
        "duplicate",
        "before",
        "spawn",
        "seam",
        "new_content",
    ] {
        let mut bad = original.clone();
        match case {
            "module" => bad["native_module_sha256"] = json!("0".repeat(64)),
            "missing" => {
                bad["anchor_overrides"]
                    .as_array_mut()
                    .expect("anchors")
                    .pop();
            }
            "duplicate" => {
                let first = bad["anchor_overrides"][0].clone();
                bad["anchor_overrides"]
                    .as_array_mut()
                    .expect("anchors")
                    .push(first);
            }
            "before" => bad["anchor_overrides"][0]["before"] = json!([0, 0]),
            "spawn" => {
                bad["spawn_overrides"]
                    .as_array_mut()
                    .expect("spawns")
                    .clear();
            }
            "seam" => bad["connections"][0]["old_offset"] = json!(999),
            "new_content" => bad["rooms"] = json!([]),
            _ => unreachable!(),
        }
        assert!(
            recompose_authored_city(&f.base, &f.layout, &bad.to_string()).is_err(),
            "invalid contract accepted: {case}"
        );
    }
}

#[test]
#[ignore = "requires a completed external city pack and frozen recomposition inputs"]
fn city_recomposition_rejects_blocked_entrances_and_unsafe_recovery() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.contract).expect("contract");
    let mut layout: Value = serde_json::from_str(&f.layout).expect("explicit layout");
    let warp = original["anchor_overrides"]
        .as_array()
        .expect("anchors")
        .iter()
        .find(|a| a["kind"] == "warp" && a["index"] == 0)
        .expect("first original city entrance");
    let x = warp["after"][0].as_u64().expect("door x") / 2;
    let y = warp["after"][1].as_u64().expect("door y") / 2;
    layout["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({
            "name":"negative-blocked-native-entrance", "kind":"block_rect",
            "bounds":[x,y,x+1,y+1], "block":"00"
        }));
    assert!(recompose_authored_city(&f.base, &layout.to_string(), &f.contract).is_err());
    let mut bad = original;
    assert!(
        !bad["spawn_overrides"]
            .as_array()
            .expect("outdoor spawn")
            .is_empty()
    );
    bad["spawn_overrides"][0]["after"] = json!([0, 0]);
    assert!(recompose_authored_city(&f.base, &f.layout, &bad.to_string()).is_err());
}

#[test]
#[ignore = "requires the external city fixture with retained river connections"]
fn city_recomposition_retained_rivers_require_complete_exact_water_lanes() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.contract).expect("river city contract");
    assert!(
        !original["water_seams"]
            .as_array()
            .expect("paired water lanes")
            .is_empty()
    );
    for case in ["missing", "projection", "duplicate_lane", "target"] {
        let mut bad = original.clone();
        match case {
            "missing" => bad["water_seams"] = json!([]),
            "projection" => {
                let x = bad["water_seams"][0]["route_edge"][0][0]
                    .as_u64()
                    .expect("paired x");
                bad["water_seams"][0]["route_edge"][0][0] = json!(x + 1);
            }
            "duplicate_lane" => {
                let first = bad["water_seams"][0]["town_edge"][0].clone();
                bad["water_seams"][0]["town_edge"][1] = first;
            }
            "target" => bad["water_seams"][0]["target"] = json!("MnStPaulPokecenter1F"),
            _ => unreachable!(),
        }
        assert!(
            recompose_authored_city(&f.base, &f.layout, &bad.to_string()).is_err(),
            "invalid retained river accepted: {case}"
        );
    }
}

#[test]
#[ignore = "requires the frozen Falls city and paired native river continuation inputs"]
fn city_recomposition_rejects_false_water_seams_and_neighbor_drift() {
    let f = fixture();
    let original: Value = serde_json::from_str(&f.contract).expect("Falls integration contract");
    assert!(
        !original["water_seams"]
            .as_array()
            .expect("paired water lanes")
            .is_empty()
    );
    assert!(
        !original["neighbor_terrain"]
            .as_array()
            .expect("real neighboring channels")
            .is_empty()
    );
    for case in [
        "waterfall",
        "neighbor_hash",
        "neighbor_cell",
        "unrelated_map",
        "projection",
        "duplicate_lane",
    ] {
        let mut bad = original.clone();
        match case {
            "waterfall" => bad["waterfall_block"] = json!(0x98),
            "neighbor_hash" => {
                bad["neighbor_terrain"][0]["native_module_sha256"] = json!("0".repeat(64))
            }
            "neighbor_cell" => {
                let old = bad["neighbor_terrain"][0]["changes"][0]["before"]
                    .as_u64()
                    .expect("block preimage");
                bad["neighbor_terrain"][0]["changes"][0]["before"] = json!((old + 1) % 256);
            }
            "unrelated_map" => bad["neighbor_terrain"][0]["map"] = json!("NewBarkTown"),
            "projection" => {
                let x = bad["water_seams"][0]["route_edge"][0][0]
                    .as_u64()
                    .expect("water lane x");
                bad["water_seams"][0]["route_edge"][0][0] = json!(x + 1);
            }
            "duplicate_lane" => {
                bad["water_seams"][0]["town_edge"][1] =
                    bad["water_seams"][0]["town_edge"][0].clone()
            }
            _ => unreachable!(),
        }
        assert!(
            recompose_authored_city(&f.base, &f.layout, &bad.to_string()).is_err(),
            "invalid river contract accepted: {case}"
        );
    }
}

#[test]
#[ignore = "requires the frozen closed north-pool checkpoint recipe"]
fn city_recomposition_rejects_incomplete_or_impassable_water_gate() {
    let f = fixture();
    let contract: Value = serde_json::from_str(&f.contract).expect("water gate contract");
    assert_eq!(contract["north_water_checkpoint"], true);
    let mut disabled = contract.clone();
    disabled["north_water_checkpoint"] = json!(false);
    assert!(recompose_authored_city(&f.base, &f.layout, &disabled.to_string()).is_err());
    for (name, bounds, block) in [
        ("missing-west-buoy", [16, 1, 17, 2], "58"),
        ("wrong-native-buoy", [17, 1, 18, 2], "33"),
        ("blocked-south-approach", [16, 2, 17, 3], "00"),
        ("blocked-north-approach", [18, 0, 19, 1], "00"),
    ] {
        let mut layout: Value = serde_json::from_str(&f.layout).expect("closed base source");
        layout["layers"]
            .as_array_mut()
            .expect("layers")
            .push(json!({
                "name":name, "kind":"block_rect", "bounds":bounds, "block":block
            }));
        assert!(
            recompose_authored_city(&f.base, &layout.to_string(), &f.contract).is_err(),
            "invalid physical gate accepted: {name}"
        );
    }
    let mut shelf: Value = serde_json::from_str(&f.layout).expect("closed gate source");
    for (name, bounds, block) in [
        ("negative-upper-bank-shelf", [15, 0, 16, 1], "41"),
        ("negative-lower-bank-shelf", [15, 1, 16, 2], "06"),
    ] {
        shelf["layers"].as_array_mut().expect("layers").push(json!({
            "name":name,"kind":"block_rect","bounds":bounds,"block":block
        }));
    }
    assert!(
        recompose_authored_city(&f.base, &shelf.to_string(), &f.contract).is_err(),
        "a Surf dismount shelf must not bypass the visible six-lane gate"
    );
}
