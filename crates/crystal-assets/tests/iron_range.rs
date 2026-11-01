//! Frozen quarry publication, source preservation and rejected traversal defects.
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
        .expect("repository path");
    let source = crystal_assets::read_verified_compiled_game_pack(
        root.join("content-packs/text-tui.crystalpack"),
    )
    .expect("supplied external source pack");
    let read = |name: &str| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored route input")
    };
    let base = crystal_assets::build_central_lakes_modpack(
        &source,
        &read("MN_R15.layout.json"),
        &read("MN_R15.json"),
    )
    .expect("existing Central Lakes overlay");
    (base, read("MN_R09.layout.json"), read("MN_R09.json"))
}

fn collision(pack: &CompiledGamePack, x: usize, y: usize) -> &str {
    let name = format!("MnR09C{:02}R{:02}", x / 64, y / 48);
    let module = &pack.data().maps[&name];
    let block = module.blocks[(y % 48) / 2 * usize::from(module.attributes.width) + (x % 64) / 2];
    &pack.data().tilesets["mn_iron_range"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

#[test]
fn iron_range_composes_preserves_source_and_publishes_the_passive_mine_return() {
    let (base, layout, dossier) = inputs();
    let result = crystal_assets::build_iron_range_modpack(&base, &layout, &dossier)
        .expect("native quarry route and mine shell");
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
    for row in 0..2 {
        for col in 0..2 {
            let name = format!("MnR09C{col:02}R{row:02}");
            let module = &result.data().maps[&name];
            assert_eq!(
                (module.attributes.width, module.attributes.height),
                (32, if row == 0 { 24 } else { 12 })
            );
            assert_eq!(module.attributes.tileset_name, "mn_iron_range");
            assert_eq!(module.attributes.border_block, 5);
            assert_eq!(
                module.attributes.map_group_constant.as_deref(),
                Some("MAPGROUP_MINNESOTA_IRON_RANGE")
            );
            assert_eq!(result.data().runtime_map_metadata[&name].group_id, 54);
            assert_eq!(
                result
                    .data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("split native module"),
                *module
            );
            assert!(result.data().wild_encounters.contains_key(&name));
            totals[0] += module.trainer_scripts.len();
            totals[1] += module.script_field_pickups.len();
            totals[2] += module.events.bg_events.len();
            totals[3] += module.objects.len();
            assert_eq!(module.attributes.connections.len(), 2);
            for connection in &module.attributes.connections {
                let opposite = match connection.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "east" => "west",
                    "west" => "east",
                    other => panic!("unexpected connection direction {other}"),
                };
                assert!(
                    result.data().maps[&connection.target_map]
                        .attributes
                        .connections
                        .iter()
                        .any(|back| {
                            back.direction == opposite
                                && back.target_map == name
                                && back.offset == 0
                        })
                );
            }
        }
    }
    assert_eq!(totals, [8, 4, 8, 15]);
    for x in 86..112 {
        for y in 30..32 {
            assert_eq!(
                collision(&result, x, y),
                if (100..102).contains(&x) {
                    "FLOOR"
                } else {
                    "WALL"
                }
            );
        }
    }
    assert_eq!(collision(&result, 112, 30), "WALL", "closed rail end");
    for y in 46..52 {
        for x in 14..18 {
            assert_eq!(collision(&result, x, y), "TALL_GRASS");
        }
    }
    assert_eq!(collision(&result, 81, 40), "HOP_DOWN");
    assert_eq!(collision(&result, 81, 41), "WALL");
    assert_eq!(collision(&result, 81, 42), "FLOOR");
    assert_eq!(collision(&result, 12, 57), "CUT_TREE");
    assert_eq!(
        result.data().field_moves.cut.replacements["mn_iron_range"][&0x63].replacement_block_id,
        0x3f
    );
    for y in 0..72 {
        for x in 0..128 {
            if x == 0 || x == 127 || y == 0 || y == 71 {
                assert!(
                    !collision(&result, x, y).starts_with("WATER"),
                    "outer water at {x},{y}"
                );
            }
        }
    }

    let exterior = &result.data().maps["MnR09C00R00"];
    assert_eq!(exterior.events.warps.len(), 2);
    assert_eq!(
        (exterior.events.warps[0].x, exterior.events.warps[0].y),
        (40, 41)
    );
    assert_eq!(
        (exterior.events.warps[1].x, exterior.events.warps[1].y),
        (40, 42)
    );
    assert_eq!(collision(&result, 40, 41), "CAVE");
    assert_eq!(
        collision(&result, 40, 42),
        "FLOOR",
        "return-only native event cannot retrigger entry"
    );
    let mine = &result.data().maps["MnR09Cave1F"];
    assert_eq!((mine.attributes.width, mine.attributes.height), (12, 11));
    assert_eq!(mine.attributes.border_block, 1);
    assert_eq!(mine.events.warps.len(), 1);
    let exit = &mine.events.warps[0];
    assert_eq!((exit.x, exit.y, exit.target_warp_id), (13, 19, 2));
    assert_eq!(
        Some(exit.target_map_constant.as_str()),
        exterior.attributes.map_constant.as_deref()
    );
    assert!(exterior.events.warps.iter().all(|warp| {
        warp.target_warp_id == 1
            && Some(warp.target_map_constant.as_str()) == mine.attributes.map_constant.as_deref()
    }));
    assert!(
        mine.objects
            .iter()
            .any(|actor| actor.script == "MN_R09_MINE_SCOUT" && (actor.x, actor.y) == (19, 7))
    );
    let medic_chunk = &result.data().maps["MnR09C01R01"];
    assert!(
        medic_chunk.script_runtime_commands.iter().any(|command| {
            command.source_script == "MN_R09_QUARRY_GUIDE"
                && command.command == "special"
                && command.args == ["HealParty"]
        }),
        "field medic must compile the existing native healing routine"
    );
    // Initial publication now includes the talk-triggered Q06 battle and cache.
    // The complete earned chapter still requires the Grand Rapids handover.
    assert!(
        mine.objects
            .iter()
            .any(|o| o.script == "MN_R09_DEPOT_FOREMAN")
    );
    assert!(
        mine.objects
            .iter()
            .any(|o| o.script == "MN_R09_SURVEY_CACHE")
    );
    assert!(
        mine.scripted_trainer_battles
            .iter()
            .any(|b| b.request.trainer_id == "MN_LODE_FOREMAN")
    );
    let design: Value = serde_json::from_str(&dossier).expect("frozen dossier");
    let rows: Vec<u16> = design["interiors"][0]["rows"]
        .as_array()
        .expect("room rows")
        .iter()
        .flat_map(|row| row.as_str().expect("room row").split_whitespace())
        .map(|token| u16::from_str_radix(token, 16).expect("native room block"))
        .collect();
    assert_eq!(mine.blocks, rows);
    for key in ["trainers", "residents", "items"] {
        for authored in design[key].as_array().expect("authored actors") {
            let id = authored["id"].as_str().expect("actor ID");
            let x = authored["tile"][0].as_u64().expect("actor x");
            let y = authored["tile"][1].as_u64().expect("actor y");
            let name = format!("MnR09C{:02}R{:02}", x / 64, y / 48);
            let actor = result.data().maps[&name]
                .objects
                .iter()
                .find(|actor| actor.script == id)
                .expect("compiled actor");
            assert_eq!(
                (u64::from(actor.x), u64::from(actor.y)),
                (x % 64, y % 48),
                "misplaced {id}"
            );
            if key == "items" {
                assert_eq!(
                    actor.event_flag,
                    authored["flag"].as_str().expect("pickup flag")
                );
            }
        }
    }
    let mut opponents = 0;
    for authored in design["trainers"].as_array().expect("trainer list") {
        let id = authored["id"].as_str().expect("trainer ID");
        let trainer = &result.data().trainers.trainers[id];
        let x = authored["tile"][0].as_u64().expect("trainer x");
        let y = authored["tile"][1].as_u64().expect("trainer y");
        let name = format!("MnR09C{:02}R{:02}", x / 64, y / 48);
        assert_eq!(
            result.data().maps[&name].trainer_scripts[id].event_flag,
            authored["defeated_flag"].as_str().expect("defeat flag")
        );
        let expected: Vec<_> = authored["party"]
            .as_array()
            .expect("party")
            .iter()
            .map(|member| {
                (
                    member["species"].as_str().expect("species"),
                    u8::try_from(member["level"].as_u64().expect("level")).expect("legal level"),
                )
            })
            .collect();
        let actual: Vec<_> = trainer
            .party
            .iter()
            .map(|member| (member.species.as_str(), member.level))
            .collect();
        assert_eq!(actual, expected, "changed party {id}");
        opponents += actual.len();
    }
    assert_eq!(opponents, 24);
    assert!(crystal_assets::build_iron_range_modpack(&result, &layout, &dossier).is_err());
}

#[test]
fn iron_range_rejects_broken_habitat_cut_ledge_and_safe_room_returns() {
    let (base, layout, dossier) = inputs();
    let base = crystal_assets::build_iron_range_art_pack(&base).expect("prepared native rail art");
    for flags in [
        json!(["BADGE_6", "EVENT_MN_HEADWATERS_REPORT"]),
        json!(["BADGE_7"]),
    ] {
        let mut changed: Value = serde_json::from_str(&dossier).expect("dossier");
        changed["required_flags"] = flags;
        assert!(
            crystal_assets::build_iron_range_modpack(&base, &layout, &changed.to_string()).is_err(),
            "Q06 prerequisites changed"
        );
    }
    for (name, mut layer) in [
        (
            "required habitat paved",
            json!({"kind":"rect","material":"floor","bounds":[7,23,9,26]}),
        ),
        (
            "walking bypass into Cut camp",
            json!({"kind":"rect","material":"lawn","bounds":[5,28,6,30]}),
        ),
        (
            "native ledge flattened",
            json!({"kind":"point","position":[40,20],"block":"01"}),
        ),
        (
            "rail crossing sealed",
            json!({"kind":"point","position":[50,15],"block":"83"}),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&layout).expect("layout");
        layer["name"] = json!(name);
        changed["layers"]
            .as_array_mut()
            .expect("layers")
            .push(layer);
        expect_design_rejection(
            crystal_assets::build_iron_range_modpack(&base, &changed.to_string(), &dossier),
            name,
        );
    }
    for (name, pointer, value) in [
        (
            "missing passive return",
            "/doors/0/return_warp_index",
            json!(null),
        ),
        (
            "return onto cave trigger",
            "/doors/0/arrival_tile",
            json!([40, 41]),
        ),
        ("wrong cave ladder", "/interiors/0/exits/0", json!([12, 19])),
        (
            "wrong mine mouth pairing",
            "/interiors/0/exit_doors/0",
            json!([42, 41]),
        ),
        (
            "false ledge return cost",
            "/one_way_ledges/0/minimum_return_steps",
            json!(500),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(&dossier).expect("dossier");
        *changed.pointer_mut(pointer).expect("authored field") = value;
        expect_design_rejection(
            crystal_assets::build_iron_range_modpack(&base, &layout, &changed.to_string()),
            name,
        );
    }
}

#[test]
fn iron_range_rejects_a_surf_landing_behind_the_intact_cut_tree() {
    let (base, layout, dossier) = inputs();
    let base = crystal_assets::build_iron_range_art_pack(&base).expect("native rail art");
    let mut changed: Value = serde_json::from_str(&layout).expect("authored layout");
    // Two native WATER metatiles create a launch/landing along the camp's
    // western wall. The original Cut approach stays walkable, the target is
    // still sealed on foot, and Surf can land at the named reward approach.
    // This must fail for the gate's Surf bypass, not an unrelated parser error.
    changed["layers"]
        .as_array_mut()
        .expect("layers")
        .push(json!({
            "kind":"rect", "name":"Surf landing behind intact sample-camp tree",
            "material":"water", "bounds":[5,28,6,30]
        }));
    let error = crystal_assets::build_iron_range_modpack(&base, &changed.to_string(), &dossier)
        .expect_err("Surf must not bypass the intact Cut gate");
    let message = format!("{error:#}");
    assert!(
        message.contains("MN_R09_SAMPLE_CAMP_CUT") && message.contains("Surf"),
        "mutation must be rejected specifically for the Cut gate's Surf bypass: {message}"
    );
}
