//! Frozen six-chunk Gunflint migration, original rooms and sealed Cut cache.
use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use crystal_assets::CompiledGamePack;
use serde_json::{Value, json};

fn expect_design_rejection(result: Result<CompiledGamePack>, case: &str) {
    let error = result.expect_err(case);
    let message = format!("{error:#}");
    assert!(
        !["missing field", "invalid type", "unknown variant"]
            .iter()
            .any(|s| message.contains(s)),
        "{case} rejected its schema instead of its native contract: {message}"
    );
}

fn inputs() -> (CompiledGamePack, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let source = crystal_assets::read_verified_compiled_game_pack(
        root.join("content-packs/text-tui.crystalpack"),
    )
    .expect("supplied source pack");
    let read = |name: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored input")
    };
    // R11 is being revised independently. Preserve a stable current R03 overlay.
    let base = crystal_assets::build_river_valley_modpack(
        &source,
        &read("MN_R03.layout.json"),
        &read("MN_R03.json"),
    )
    .expect("stable River Valley baseline");
    (base, read("MN_R12.layout.json"), read("MN_R12.json"))
}

fn collision(pack: &CompiledGamePack, x: usize, y: usize, cut: bool) -> &str {
    let module = &pack.data().maps[&format!("MnR12C{:02}R{:02}", x / 64, y / 48)];
    let mut block =
        module.blocks[(y % 48) / 2 * usize::from(module.attributes.width) + (x % 64) / 2];
    if cut && x / 2 == 27 && y / 2 == 11 {
        block = pack.data().field_moves.cut.replacements["mn_gunflint"][&0x0f].replacement_block_id;
    }
    &pack.data().tilesets["mn_gunflint"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn reachable(
    pack: &CompiledGamePack,
    cut: bool,
    surf: bool,
    grass: bool,
) -> BTreeSet<(usize, usize)> {
    let mut occupied = BTreeSet::new();
    for row in 0..3 {
        for col in 0..2 {
            for actor in &pack.data().maps[&format!("MnR12C{col:02}R{row:02}")].objects {
                occupied.insert((
                    col * 64 + usize::from(actor.x),
                    row * 48 + usize::from(actor.y),
                ));
            }
        }
    }
    let mut seen = BTreeSet::from([(124, 56)]);
    let mut queue = VecDeque::from([(124, 56)]);
    while let Some((x, y)) = queue.pop_front() {
        for (nx, ny) in [
            (x as i32 - 1, y as i32),
            (x as i32 + 1, y as i32),
            (x as i32, y as i32 - 1),
            (x as i32, y as i32 + 1),
        ] {
            if !(0..128).contains(&nx) || !(0..112).contains(&ny) {
                continue;
            }
            let p = (
                usize::try_from(nx).expect("bounded x"),
                usize::try_from(ny).expect("bounded y"),
            );
            let kind = collision(pack, p.0, p.1, cut);
            if (kind == "FLOOR"
                || (grass && kind == "TALL_GRASS")
                || (surf && kind.starts_with("WATER")))
                && !occupied.contains(&p)
                && seen.insert(p)
            {
                queue.push_back(p);
            }
        }
    }
    seen
}

#[test]
fn gunflint_publishes_six_clipped_chunks_original_rooms_and_preserves_river_valley() {
    let (base, layout, dossier) = inputs();
    let result = crystal_assets::build_gunflint_modpack(&base, &layout, &dossier)
        .expect("revised Gunflint overlay");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 8);
    for (name, module) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(module),
            "changed map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(name),
            Some(tileset),
            "changed art {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            result.data().trainers.trainers.get(name),
            Some(trainer),
            "changed opponent {name}"
        );
    }
    for (name, encounter) in &base.data().wild_encounters {
        assert_eq!(
            result.data().wild_encounters.get(name),
            Some(encounter),
            "changed encounters {name}"
        );
    }
    for (name, metadata) in &base.data().runtime_map_metadata {
        assert_eq!(
            result.data().runtime_map_metadata.get(name),
            Some(metadata),
            "changed metadata {name}"
        );
    }
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().items, base.data().items);
    assert_eq!(result.data().marts, base.data().marts);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(
        result.data().story_event_script_constants.global["SPAWN_HOME"],
        base.data().story_event_script_constants.global["SPAWN_HOME"]
    );
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("result audio manifest"),
        base.audio_manifest().expect("source audio manifest")
    );

    let mut totals = [0, 0, 0, 0];
    for row in 0..3 {
        for col in 0..2 {
            let name = format!("MnR12C{col:02}R{row:02}");
            let module = &result.data().maps[&name];
            assert_eq!(
                (module.attributes.width, module.attributes.height),
                (32, if row < 2 { 24 } else { 8 })
            );
            assert_eq!(module.attributes.tileset_name, "mn_gunflint");
            assert_eq!(
                result.data().runtime_map_metadata[module
                    .attributes
                    .map_constant
                    .as_deref()
                    .expect("native map constant")]
                .group_id,
                42
            );
            assert_eq!(
                result
                    .data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("native split payload"),
                *module
            );
            assert!(result.data().wild_encounters.contains_key(&name));
            totals[0] += module.trainer_scripts.len();
            totals[1] += module.script_field_pickups.len();
            totals[2] += module.events.bg_events.len();
            totals[3] += module.objects.len();
            assert_eq!(
                module.attributes.connections.len(),
                if row == 1 { 3 } else { 2 }
            );
            for connection in &module.attributes.connections {
                let opposite = match connection.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "east" => "west",
                    "west" => "east",
                    _ => panic!("cardinal connection"),
                };
                assert!(
                    result.data().maps[&connection.target_map]
                        .attributes
                        .connections
                        .iter()
                        .any(|back| back.direction == opposite
                            && back.target_map == name
                            && back.offset == 0)
                );
            }
        }
    }
    assert_eq!(totals, [10, 4, 6, 20]);
    assert!(!result.data().maps.contains_key("MnR12C02R00"));
    assert!(!result.data().maps.contains_key("MnR12C00R03"));
    let design: Value = serde_json::from_str(&dossier).expect("dossier");
    for (i, chunk, room, door, safe, dimensions) in [
        (0, "MnR12C01R00", "MnR12Cave1F", (2, 23), (66, 24), (16, 12)),
        (
            1,
            "MnR12C01R01",
            "MnR12RangerHut",
            (21, 35),
            (85, 84),
            (6, 5),
        ),
    ] {
        let outside = &result.data().maps[chunk];
        let inside = &result.data().maps[room];
        assert_eq!(
            outside.events.warps.len(),
            1,
            "trigger return contract has no passive record"
        );
        let entry = &outside.events.warps[0];
        assert_eq!(
            (entry.index, entry.x, entry.y, entry.target_warp_id),
            (1, door.0, door.1, 1)
        );
        assert_eq!(
            Some(entry.target_map_constant.as_str()),
            inside.attributes.map_constant.as_deref()
        );
        assert_eq!(
            (inside.attributes.width, inside.attributes.height),
            dimensions
        );
        assert_eq!(collision(&result, safe.0, safe.1, false), "FLOOR");
        assert_eq!(inside.events.warps.len(), if i == 0 { 1 } else { 2 });
        for exit in &inside.events.warps {
            assert_eq!(exit.target_warp_id, 1);
            assert_eq!(
                Some(exit.target_map_constant.as_str()),
                outside.attributes.map_constant.as_deref()
            );
        }
        let coordinates: Vec<_> = inside.events.warps.iter().map(|w| (w.x, w.y)).collect();
        assert_eq!(
            coordinates,
            if i == 0 {
                vec![(17, 23)]
            } else {
                vec![(4, 9), (5, 9)]
            }
        );
        let rows: Vec<u16> = design["interiors"][i]["rows"]
            .as_array()
            .expect("original room rows")
            .iter()
            .flat_map(|row| row.as_str().expect("room row").split_whitespace())
            .map(|token| u16::from_str_radix(token, 16).expect("native block"))
            .collect();
        assert_eq!(inside.blocks, rows, "original room art changed");
        assert!(
            !inside
                .script_runtime_commands
                .iter()
                .any(|c| c.command == "special" && c.args == ["HealParty"]),
            "no authored healing service exists in either original room"
        );
    }
    assert_eq!(
        (
            result.data().maps["MnR12Cave1F"].objects.len(),
            result.data().maps["MnR12Cave1F"].script_field_pickups.len()
        ),
        (2, 1)
    );
    assert_eq!(
        (
            result.data().maps["MnR12RangerHut"].objects.len(),
            result.data().maps["MnR12RangerHut"].events.bg_events.len()
        ),
        (1, 1)
    );
    for key in ["trainers", "residents", "items"] {
        for authored in design[key].as_array().expect("actors") {
            let id = authored["id"].as_str().expect("actor ID");
            let x = authored["tile"][0].as_u64().expect("x");
            let y = authored["tile"][1].as_u64().expect("y");
            let module = &result.data().maps[&format!("MnR12C{:02}R{:02}", x / 64, y / 48)];
            let actor = module
                .objects
                .iter()
                .find(|a| a.script == id)
                .expect("native actor");
            assert_eq!((u64::from(actor.x), u64::from(actor.y)), (x % 64, y % 48));
            if key == "items" {
                assert_eq!(
                    actor.event_flag,
                    authored["flag"].as_str().expect("item flag")
                );
            }
        }
    }
    let mut opponents = 0;
    for trainer in design["trainers"].as_array().expect("trainers") {
        let id = trainer["id"].as_str().expect("trainer ID");
        let expected: Vec<_> = trainer["party"]
            .as_array()
            .expect("party")
            .iter()
            .map(|p| {
                (
                    p["species"].as_str().expect("species"),
                    p["level"].as_u64().expect("level"),
                )
            })
            .collect();
        let actual: Vec<_> = result.data().trainers.trainers[id]
            .party
            .iter()
            .map(|p| (p.species.as_str(), u64::from(p.level)))
            .collect();
        assert_eq!(actual, expected, "original party {id}");
        opponents += actual.len();
    }
    assert_eq!(opponents, 24);
    assert_eq!(
        result.data().field_moves.cut.replacements["mn_gunflint"][&0x0f].replacement_block_id,
        0x17
    );
    let before = reachable(&result, false, false, true);
    let after = reachable(&result, true, false, true);
    assert!(before.contains(&(2, 56)));
    assert!(!before.contains(&(54, 19)));
    assert!(after.contains(&(54, 19)));
    assert_eq!(after.len() - before.len(), 26);
    assert!(
        !reachable(&result, false, true, true).contains(&(54, 19)),
        "Surf must not bypass the herb gate"
    );
    assert!(
        !reachable(&result, false, false, false).contains(&(2, 56)),
        "main habitat has a dry bypass"
    );
    let mut trees = Vec::new();
    for y in 0..112 {
        for x in 0..128 {
            let kind = collision(&result, x, y, false);
            if kind == "CUT_TREE" {
                trees.push((x, y));
            }
            if x == 0 || x == 127 || y == 0 || y == 111 {
                assert!(!kind.starts_with("WATER"), "outer water at {x},{y}");
            }
        }
    }
    assert_eq!(trees, [(54, 23)]);
    for (x0, y0, x1, y1) in [(56, 64, 58, 68), (98, 36, 108, 38), (32, 80, 34, 90)] {
        for y in y0..y1 {
            for x in x0..x1 {
                assert_eq!(collision(&result, x, y, false), "TALL_GRASS");
            }
        }
    }
    assert!(crystal_assets::build_gunflint_modpack(&result, &layout, &dossier).is_err());
}

#[test]
fn gunflint_rejects_stale_geometry_bypasses_and_wrong_native_returns() {
    let (base, layout, dossier) = inputs();
    let base = crystal_assets::build_gunflint_art_pack(&base).expect("native forest art");
    for (name, pointer, value) in [
        ("old extent", "/dimensions_metatiles", json!([96, 96])),
        ("old chunk columns", "/chunk_columns", json!(3)),
        ("old chunk rows", "/chunk_rows", json!(4)),
        (
            "missing lighthouse prerequisite",
            "/required_flags",
            json!([
                "BADGE_1", "BADGE_2", "BADGE_3", "BADGE_4", "BADGE_5", "BADGE_6", "BADGE_7",
                "BADGE_8"
            ]),
        ),
        ("wrong cave exit", "/interiors/0/exits/0", json!([0, 0])),
        (
            "invented cave return",
            "/doors/0/arrival_tile",
            json!([66, 24]),
        ),
        ("old hut warp index", "/doors/1/warp_index", json!(3)),
        (
            "wrong gated target",
            "/cut_gates/0/targets/0",
            json!([54, 24]),
        ),
        (
            "wrong Cut replacement",
            "/cut_gates/0/replacement_block",
            json!("01"),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&dossier).expect("dossier");
        *changed.pointer_mut(pointer).expect("authored field") = value;
        expect_design_rejection(
            crystal_assets::build_gunflint_modpack(&base, &layout, &changed.to_string()),
            name,
        );
    }
    for (name, layer) in [
        (
            "paved main habitat",
            json!({"kind":"rect","material":"floor","bounds":[28,32,29,34]}),
        ),
        (
            "walking entry behind Cut",
            json!({"kind":"rect","material":"floor","bounds":[26,11,27,13]}),
        ),
        (
            "incidental Cut canopy",
            json!({"kind":"point","position":[2,2],"block":"0f"}),
        ),
        (
            "outer water hole",
            json!({"kind":"point","position":[0,28],"block":"02"}),
        ),
        (
            "fake cave door",
            json!({"kind":"point","position":[33,11],"block":"01"}),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&layout).expect("layout");
        let mut layer = layer;
        layer["name"] = json!(name);
        changed["layers"]
            .as_array_mut()
            .expect("layers")
            .push(layer);
        expect_design_rejection(
            crystal_assets::build_gunflint_modpack(&base, &changed.to_string(), &dossier),
            name,
        );
    }
}

#[test]
fn gunflint_rejects_a_surf_landing_behind_the_intact_herb_tree() {
    let (base, layout, dossier) = inputs();
    let base = crystal_assets::build_gunflint_art_pack(&base).expect("native forest art");
    let mut changed: Value = serde_json::from_str(&layout).expect("authored layout");
    changed["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({
            "kind":"rect", "name":"Surf landing behind intact herb tree",
            "material":"water", "bounds":[26,11,27,13]
        }));
    let error = crystal_assets::build_gunflint_modpack(&base, &changed.to_string(), &dossier)
        .expect_err("Surf cannot enter the sealed herb cache");
    let message = format!("{error:#}");
    assert!(
        message.contains("MN_R12_SHORTCUT") && message.contains("Surf"),
        "expected the actual gate's Surf bypass rejection: {message}"
    );
}
