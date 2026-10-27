//! Native timber fidelity, clipped publication and the measured local Cut link.
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
            .any(|schema| message.contains(schema)),
        "{case} failed JSON parsing instead of its native contract: {message}"
    );
}

fn inputs() -> (CompiledGamePack, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let base = crystal_assets::read_verified_compiled_game_pack(
        root.join("content-packs/text-tui.crystalpack"),
    )
    .expect("supplied source pack");
    let read = |name: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored route input")
    };
    (base, read("MN_R17.layout.json"), read("MN_R17.json"))
}

fn collision(pack: &CompiledGamePack, x: usize, y: usize, opened: bool) -> &str {
    let name = format!("MnR17C00R{:02}", y / 48);
    let module = &pack.data().maps[&name];
    let mut block = module.blocks[(y % 48) / 2 * 32 + x / 2];
    if opened && x / 2 == 18 && y / 2 == 34 {
        block =
            pack.data().field_moves.cut.replacements["mn_border_pines"][&0x0f].replacement_block_id;
    }
    &pack.data().tilesets["mn_border_pines"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn walk_cost(
    pack: &CompiledGamePack,
    start: (usize, usize),
    end: (usize, usize),
    opened: bool,
    surf: bool,
) -> usize {
    let mut occupied = BTreeSet::new();
    for row in 0..3 {
        for actor in &pack.data().maps[&format!("MnR17C00R{row:02}")].objects {
            occupied.insert((usize::from(actor.x), row * 48 + usize::from(actor.y)));
        }
    }
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([(start, 0)]);
    while let Some(((x, y), cost)) = queue.pop_front() {
        if (x, y) == end {
            return cost;
        }
        for (nx, ny) in [
            (x as i32 - 1, y as i32),
            (x as i32 + 1, y as i32),
            (x as i32, y as i32 - 1),
            (x as i32, y as i32 + 1),
        ] {
            if !(0..64).contains(&nx) || !(0..112).contains(&ny) {
                continue;
            }
            let at = (
                usize::try_from(nx).expect("bounded x"),
                usize::try_from(ny).expect("bounded y"),
            );
            if occupied.contains(&at) || seen.contains(&at) {
                continue;
            }
            let kind = collision(pack, at.0, at.1, opened);
            if matches!(kind, "FLOOR" | "TALL_GRASS") || (surf && kind.starts_with("WATER")) {
                seen.insert(at);
                queue.push_back((at, cost + 1));
            }
        }
    }
    panic!("authored link {start:?}→{end:?} is unreachable");
}

#[test]
fn border_pines_art_preserves_native_glyphs_and_exact_timbers() {
    let (base, _, _) = inputs();
    let result = crystal_assets::build_border_pines_art_pack(&base).expect("native timber import");
    assert_eq!(result.data().maps, base.data().maps);
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source {name}"
        );
    }
    let files = result.runtime_files();
    let original = &files["data/tilesets/mn_gunflint_metatiles.bin"];
    let blocks = &files["data/tilesets/mn_border_pines_metatiles.bin"];
    assert_eq!(blocks.len(), 60 * 16);
    assert_eq!(&blocks[..58 * 16], original);
    assert_eq!(files["gfx/tilesets/mn_border_pines.2bpp"].len(), 192 * 16);
    let glyph = |tileset: &str, glyph: u8| {
        let palette = result.data().tilesets[tileset].palette_map[usize::from(glyph)];
        let index = if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 0x7f) + 96
        };
        let pixels = &files[&format!("gfx/tilesets/{tileset}.2bpp")];
        (&pixels[index * 16..(index + 1) * 16], palette & 7)
    };
    for &id in original {
        assert_eq!(glyph("mn_gunflint", id), glyph("mn_border_pines", id));
    }
    let source = &files["data/tilesets/kanto_metatiles.bin"][0x77 * 16..0x78 * 16];
    for (target, order) in [(0x3a, [0, 1, 2, 3]), (0x3b, [2, 3, 0, 1])] {
        for (row, source_row) in order.into_iter().enumerate() {
            for col in 0..4 {
                assert_eq!(
                    glyph("kanto", source[source_row * 4 + col]),
                    glyph("mn_border_pines", blocks[target * 16 + row * 4 + col])
                );
            }
        }
    }
    assert_eq!(
        result.data().tilesets["mn_border_pines"].collision["3a"],
        ["FLOOR", "FLOOR", "WALL", "WALL"]
    );
    assert_eq!(
        result.data().tilesets["mn_border_pines"].collision["3b"],
        result.data().tilesets["johto"].collision["75"]
    );
    assert_eq!(
        result.data().field_moves.cut.replacements["mn_border_pines"][&0x0f].replacement_block_id,
        0x17
    );
    assert!(crystal_assets::build_border_pines_art_pack(&result).is_err());
}

#[test]
fn border_pines_publishes_five_maps_and_the_actual_local_shortcut() {
    let (base, layout, dossier) = inputs();
    let result = crystal_assets::build_border_pines_modpack(&base, &layout, &dossier)
        .expect("native forest route");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 5);
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
            "changed source art {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            result.data().trainers.trainers.get(name),
            Some(trainer),
            "changed opponent {name}"
        );
    }
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().items, base.data().items);
    assert_eq!(result.data().marts, base.data().marts);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("result audio"),
        base.audio_manifest().expect("base audio")
    );
    let mut totals = [0, 0, 0, 0];
    for row in 0..3 {
        let name = format!("MnR17C00R{row:02}");
        let module = &result.data().maps[&name];
        assert_eq!(
            (module.attributes.width, module.attributes.height),
            (32, if row == 2 { 8 } else { 24 })
        );
        assert_eq!(module.attributes.tileset_name, "mn_border_pines");
        assert_eq!(module.attributes.border_block, 5);
        assert_eq!(
            module.attributes.map_group_constant.as_deref(),
            Some("MAPGROUP_MINNESOTA_BORDER_PINES")
        );
        assert_eq!(result.data().runtime_map_metadata[&name].group_id, 57);
        assert_eq!(
            result
                .data()
                .assemble_map_module_from_compiled_payloads(&name)
                .expect("native module"),
            *module
        );
        assert!(result.data().wild_encounters.contains_key(&name));
        totals[0] += module.trainer_scripts.len();
        totals[1] += module.script_field_pickups.len();
        totals[2] += module.events.bg_events.len();
        totals[3] += module.objects.len();
        assert_eq!(
            module.attributes.connections.len(),
            if row == 1 { 2 } else { 1 }
        );
        for connection in &module.attributes.connections {
            let opposite = match connection.direction.as_str() {
                "north" => "south",
                "south" => "north",
                other => panic!("unexpected connection {other}"),
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
    assert_eq!(totals, [6, 3, 9, 12]);
    assert_eq!(collision(&result, 36, 69, false), "CUT_TREE");
    assert_eq!(collision(&result, 36, 69, true), "FLOOR");
    assert_eq!(walk_cost(&result, (36, 67), (36, 73), false, false), 66);
    assert_eq!(walk_cost(&result, (36, 67), (36, 73), true, false), 6);
    assert_eq!(walk_cost(&result, (36, 67), (36, 73), false, true), 66);
    assert_eq!(walk_cost(&result, (36, 67), (36, 73), true, true), 6);
    // The useful local cache/watcher return saves60. Regional traversal saves2.
    assert_eq!(walk_cost(&result, (32, 108), (36, 2), false, false), 162);
    assert_eq!(walk_cost(&result, (32, 108), (36, 2), true, false), 160);
    for x in 22..30 {
        assert_eq!(collision(&result, x, 64, false), "WALL");
        assert_eq!(collision(&result, x, 65, false), "FLOOR");
        assert_eq!(collision(&result, x, 66, false), "FLOOR");
        assert_eq!(collision(&result, x, 67, false), "WALL");
    }
    for y in 88..92 {
        for x in 22..28 {
            assert_eq!(collision(&result, x, y, false), "TALL_GRASS");
        }
    }
    let mut trees = Vec::new();
    for y in 0..112 {
        for x in 0..64 {
            let kind = collision(&result, x, y, false);
            if kind == "CUT_TREE" {
                trees.push((x, y));
            }
            if x == 0 || x == 63 || y == 0 || y == 111 {
                assert!(!kind.starts_with("WATER"), "outer water at{x},{y}");
            }
        }
    }
    assert_eq!(trees, [(36, 69)]);
    for (chunk, room, entry, exit) in [
        ("MnR17C00R00", "MnR17Cave1F", (10, 43), (9, 13)),
        ("MnR17C00R01", "MnR17RangerHut", (45, 1), (5, 7)),
    ] {
        let outside = &result.data().maps[chunk];
        let inside = &result.data().maps[room];
        assert_eq!(outside.events.warps.len(), 2);
        assert_eq!(
            (
                outside.events.warps[0].index,
                outside.events.warps[0].x,
                outside.events.warps[0].y
            ),
            (1, entry.0, entry.1)
        );
        assert_eq!(
            (
                outside.events.warps[1].index,
                outside.events.warps[1].x,
                outside.events.warps[1].y
            ),
            (2, entry.0, entry.1 + 1)
        );
        assert_eq!(inside.events.warps.len(), 1);
        assert_eq!(
            (
                inside.events.warps[0].x,
                inside.events.warps[0].y,
                inside.events.warps[0].target_warp_id
            ),
            (exit.0, exit.1, 2)
        );
        assert_eq!(
            Some(inside.events.warps[0].target_map_constant.as_str()),
            outside.attributes.map_constant.as_deref()
        );
    }
    let keeper = &result.data().maps["MnR17RangerHut"];
    assert!(
        keeper
            .script_runtime_commands
            .iter()
            .any(|command| command.source_script == "MN_R17_HUT_KEEPER"
                && command.command == "special"
                && command.args == ["HealParty"])
    );
    let design: Value = serde_json::from_str(&dossier).expect("dossier");
    let mut opponents = 0;
    for authored in design["trainers"].as_array().expect("trainer list") {
        let id = authored["id"].as_str().expect("trainer ID");
        let trainer = &result.data().trainers.trainers[id];
        let expected: Vec<_> = authored["party"]
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
        let actual: Vec<_> = trainer
            .party
            .iter()
            .map(|p| (p.species.as_str(), u64::from(p.level)))
            .collect();
        assert_eq!(actual, expected, "party{id}");
        opponents += actual.len();
    }
    assert_eq!(opponents, 18);
    assert!(crystal_assets::build_border_pines_modpack(&result, &layout, &dossier).is_err());
}

#[test]
fn border_pines_rejects_false_shortcuts_and_unsafe_native_room_indices() {
    let (base, layout, dossier) = inputs();
    let base = crystal_assets::build_border_pines_art_pack(&base).expect("prepared timber art");
    for (name, mut layer) in [
        (
            "walking sidestep",
            json!({"kind":"rect","material":"lawn","bounds":[17,34,18,36]}),
        ),
        (
            "paved mandatory sedge",
            json!({"kind":"rect","material":"lawn","bounds":[11,44,14,46]}),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&layout).expect("layout");
        layer["name"] = json!(name);
        changed["layers"]
            .as_array_mut()
            .expect("layers")
            .push(layer);
        expect_design_rejection(
            crystal_assets::build_border_pines_modpack(&base, &changed.to_string(), &dossier),
            name,
        );
    }
    for (name, pointer, value) in [
        ("wrong badge", "/required_flags", json!(["BADGE_6"])),
        (
            "false local saving",
            "/shortcut/minimum_saved_steps",
            json!(1000),
        ),
        (
            "nearby false target",
            "/shortcut/compare_to",
            json!([36, 68]),
        ),
        ("old global hut warp index", "/doors/1/warp_index", json!(3)),
        ("wrong cave ladder", "/interiors/0/exits/0", json!([0, 0])),
        (
            "return on hut trigger",
            "/doors/1/arrival_tile",
            json!([45, 49]),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&dossier).expect("dossier");
        *changed.pointer_mut(pointer).expect("authored field") = value;
        expect_design_rejection(
            crystal_assets::build_border_pines_modpack(&base, &layout, &changed.to_string()),
            name,
        );
    }
    let mut changed: Value = serde_json::from_str(&layout).expect("layout");
    changed["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({"kind":"rect","name":"Surf sidestep behind intact forest tree","material":"water","bounds":[17,34,18,36]}));
    let error = crystal_assets::build_border_pines_modpack(&base, &changed.to_string(), &dossier)
        .expect_err("Surf must not erase the useful Cut link");
    let message = format!("{error:#}");
    assert!(
        message.to_lowercase().contains("shortcut") && message.contains("Surf"),
        "expected the measured shortcut Surf rejection: {message}"
    );
}
