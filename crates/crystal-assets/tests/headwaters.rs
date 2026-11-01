//! Native publication contracts; production controller traversal is separate.

use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    CompiledGamePack, build_headwaters_modpack, build_red_river_prairie_modpack,
    read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

fn fixture() -> (CompiledGamePack, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let source = std::env::var_os("CRYSTAL_HEADWATERS_BASE_PACK")
        .map_or_else(
            || root.join("content-packs/text-tui.crystalpack"),
            PathBuf::from,
        )
        .canonicalize()
        .expect("completed external source pack");
    let base = read_verified_compiled_game_pack(source).expect("verified external source");
    let read = |name| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored route input")
    };
    let baseline =
        build_red_river_prairie_modpack(&base, &read("MN_R16.layout.json"), &read("MN_R16.json"))
            .expect("stable revised R16 overlay");
    (baseline, read("MN_R14.layout.json"), read("MN_R14.json"))
}

#[test]
fn headwaters_composes_four_clipped_maps_and_preserves_source_world_and_revised_prairie() {
    let (base, layout, dossier) = fixture();
    let result = build_headwaters_modpack(&base, &layout, &dossier)
        .expect("reviewed four-map Headwaters publication");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 4);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("result audio"),
        base.audio_manifest().expect("source audio")
    );
    assert_eq!(result.data().items, base.data().items);
    for (name, module) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(module),
            "changed map {name}"
        );
    }
    for (name, metadata) in &base.data().runtime_map_metadata {
        assert_eq!(
            result.data().runtime_map_metadata.get(name),
            Some(metadata),
            "changed existing map metadata {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            result.data().trainers.trainers.get(name),
            Some(trainer),
            "changed source trainer {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(name),
            Some(tileset),
            "changed source art/collision {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    let design: Value = serde_json::from_str(&dossier).expect("dossier");
    let mut totals = [0, 0, 0, 0];
    for (row, height) in [(0, 24), (1, 8)] {
        for (column, width) in [(0, 32), (1, 16)] {
            let name = format!("MnR14C{column:02}R{row:02}");
            let module = &result.data().maps[&name];
            assert_eq!(
                (module.attributes.width, module.attributes.height),
                (width, height)
            );
            assert_eq!(
                module.blocks.len(),
                usize::from(width) * usize::from(height)
            );
            assert_eq!(module.attributes.tileset_name, "mn_headwaters");
            assert_eq!(
                module.attributes.music,
                base.data().maps["Route30"].attributes.music
            );
            assert_eq!(module.attributes.border_block, 5);
            assert_eq!(module.attributes.connections.len(), 2);
            for connection in &module.attributes.connections {
                let opposite = match connection.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "west" => "east",
                    "east" => "west",
                    _ => panic!("cardinal native connection"),
                };
                assert_eq!(connection.offset, 0);
                assert!(
                    result.data().maps[&connection.target_map]
                        .attributes
                        .connections
                        .iter()
                        .any(|back| back.target_map == name
                            && back.direction == opposite
                            && back.offset == 0)
                );
            }
            let metadata = &result.data().runtime_map_metadata[&name];
            assert_eq!(metadata.group_id, 44);
            assert_eq!(metadata.group_name, "MINNESOTA_HEADWATERS");
            assert_eq!(metadata.map_id, row * 2 + column + 1);
            assert_eq!((metadata.width, metadata.height), (width, height));
            assert_eq!(
                result
                    .data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("split native payload"),
                *module
            );
            assert!(result.data().wild_encounters.contains_key(&name));
            assert!(
                module.events.warps.is_empty(),
                "Headwaters has no interiors"
            );
            totals[0] += module.trainer_scripts.len();
            totals[1] += module.script_field_pickups.len();
            totals[2] += module.events.bg_events.len();
            totals[3] += module.objects.len();
            for object in &module.objects {
                assert!(object.x < width * 2 && object.y < height * 2);
                let actor = ["trainers", "residents", "items"]
                    .into_iter()
                    .flat_map(|key| design[key].as_array().expect("actors"))
                    .find(|actor| actor["id"].as_str() == Some(object.script.as_str()))
                    .expect("preserved authored actor identity");
                assert_eq!(
                    actor["tile"],
                    json!([object.x + column * 64, object.y + row * 48])
                );
                assert_eq!(object.object_identifier.as_deref(), actor["id"].as_str());
            }
        }
    }
    assert_eq!(totals, [6, 4, 6, 16]);
    assert_eq!(
        result
            .data()
            .maps
            .keys()
            .filter(|name| name.starts_with("MnR14"))
            .count(),
        4
    );
    assert_native_geometry(&result);
    for actor in design["trainers"].as_array().expect("trainers") {
        let id = actor["id"].as_str().expect("trainer ID");
        let native = &result.data().trainers.trainers[id];
        let party = native
            .party
            .iter()
            .map(|member| {
                assert!(
                    !member.moves.is_empty(),
                    "native level-derived moves missing for {id}"
                );
                json!({"species":member.species,"level":member.level})
            })
            .collect::<Vec<_>>();
        assert_eq!(
            Value::Array(party),
            actor["party"],
            "party changed for {id}"
        );
        let request = result
            .data()
            .maps
            .values()
            .find_map(|module| module.trainer_scripts.get(id))
            .expect("native trainer command");
        assert_eq!(
            Some(request.event_flag.as_str()),
            actor["defeated_flag"].as_str()
        );
        assert_eq!(
            Some(request.trainer_class.as_str()),
            actor["trainer_class"].as_str()
        );
    }
    for index in 7..=10 {
        assert!(
            !result
                .data()
                .trainers
                .trainers
                .contains_key(&format!("MN_R14_T{index:02}")),
            "reserved trainer identity must remain unused"
        );
    }
    for actor in design["items"].as_array().expect("pickups") {
        let id = actor["id"].as_str().expect("pickup ID");
        let module = result
            .data()
            .maps
            .values()
            .find(|module| module.objects.iter().any(|object| object.script == id))
            .expect("pickup map");
        let object = module
            .objects
            .iter()
            .find(|object| object.script == id)
            .expect("pickup object");
        let pickup = module
            .script_field_pickups
            .iter()
            .find(|pickup| pickup.source_script == id)
            .expect("compiled native item ball");
        assert_eq!(pickup.item_id.as_deref(), actor["item"].as_str());
        assert_eq!(
            u64::from(pickup.quantity),
            actor["quantity"].as_u64().expect("quantity")
        );
        assert_eq!(Some(object.event_flag.as_str()), actor["flag"].as_str());
    }
    let native = &result.runtime_files()["data/tilesets/mn_headwaters_metatiles.bin"];
    assert_eq!(native.len(), 131 * 16);
    assert_eq!(
        &native[..128 * 16],
        base.runtime_files()["data/tilesets/johto_metatiles.bin"].as_slice()
    );
    assert!(build_headwaters_modpack(&result, &layout, &dossier).is_err());
}

type Point = (u16, u16);

fn kind(pack: &CompiledGamePack, (x, y): Point, cut: bool) -> &str {
    let name = format!("MnR14C{:02}R{:02}", x / 64, y / 48);
    let map = &pack.data().maps[&name];
    let block = if cut && (x / 2, y / 2) == (35, 10) {
        0x3c
    } else {
        map.blocks[usize::from((y % 48) / 2) * usize::from(map.attributes.width)
            + usize::from((x % 64) / 2)]
    };
    &pack.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")]
        [usize::from(y % 2 * 2 + x % 2)]
}

fn adjacent((x, y): Point) -> impl Iterator<Item = Point> {
    [
        (i32::from(x) - 1, i32::from(y)),
        (i32::from(x) + 1, i32::from(y)),
        (i32::from(x), i32::from(y) - 1),
        (i32::from(x), i32::from(y) + 1),
    ]
    .into_iter()
    .filter_map(|(a, b)| {
        ((0..96).contains(&a) && (0..64).contains(&b)).then_some((a as u16, b as u16))
    })
}

fn distance(
    pack: &CompiledGamePack,
    from: Point,
    to: Point,
    cut: bool,
    sealed: &BTreeSet<Point>,
) -> Option<usize> {
    let mut seen = BTreeSet::from([from]);
    let mut queue = VecDeque::from([(from, 0)]);
    while let Some((at, steps)) = queue.pop_front() {
        if at == to {
            return Some(steps);
        }
        for next in adjacent(at) {
            if !sealed.contains(&next)
                && matches!(kind(pack, next, cut), "FLOOR" | "TALL_GRASS" | "LADDER")
                && seen.insert(next)
            {
                queue.push_back((next, steps + 1));
            }
        }
    }
    None
}

fn assert_native_geometry(pack: &CompiledGamePack) {
    let empty = BTreeSet::new();
    assert_eq!(distance(pack, (71, 21), (71, 19), false, &empty), Some(34));
    assert_eq!(distance(pack, (71, 21), (71, 19), true, &empty), Some(2));
    assert!(distance(pack, (92, 26), (2, 42), false, &empty).is_some());
    let crossing = (36..40)
        .flat_map(|y| (42..54).map(move |x| (x, y)))
        .collect();
    assert_eq!(
        distance(pack, (92, 26), (2, 42), false, &crossing),
        None,
        "the boardwalk must be independently necessary"
    );
    for x in 42..54 {
        for (y, expected) in [(36, "WALL"), (37, "FLOOR"), (38, "FLOOR"), (39, "WALL")] {
            assert_eq!(kind(pack, (x, y), false), expected, "bridge {x},{y}");
        }
    }
    for point in [(10, 16), (10, 17), (10, 18), (10, 19), (11, 16), (11, 19)] {
        assert_eq!(
            kind(pack, point, false),
            "WALL",
            "closed survey landing {point:?}"
        );
    }
    for (point, expected) in [
        ((29, 23), "FLOOR"),
        ((29, 24), "HOP_DOWN"),
        ((29, 25), "WALL"),
        ((29, 26), "FLOOR"),
    ] {
        assert_eq!(kind(pack, point, false), expected);
    }
    let cuts = (0..64)
        .flat_map(|y| (0..96).map(move |x| (x, y)))
        .filter(|&point| kind(pack, point, false) == "CUT_TREE")
        .collect::<Vec<_>>();
    assert_eq!(cuts, [(71, 20)]);
    let mut ports = BTreeSet::new();
    for y in 0..64 {
        for x in 0..96 {
            if x != 0 && x != 95 && y != 0 && y != 63 {
                continue;
            }
            let terrain = kind(pack, (x, y), false);
            assert!(
                !matches!(terrain, "WATER" | "WHIRLPOOL" | "WATERFALL"),
                "outer Surf leak at {x},{y}"
            );
            if matches!(terrain, "FLOOR" | "TALL_GRASS" | "LADDER") {
                ports.insert((x, y));
            }
        }
    }
    assert_eq!(
        ports,
        BTreeSet::from([(0, 42), (0, 43), (95, 26), (95, 27)])
    );
    for y in 34..42 {
        for x in 56..60 {
            assert_eq!(kind(pack, (x, y), false), "TALL_GRASS");
        }
    }
}

fn append_layer(layout: &str, layer: Value) -> String {
    let mut value: Value = serde_json::from_str(layout).expect("layout");
    value["layers"].as_array_mut().expect("layers").push(layer);
    value.to_string()
}

#[test]
fn headwaters_rejects_broken_bridge_habitat_ledge_cut_and_outer_water() {
    let (base, layout, dossier) = fixture();
    for layer in [
        json!({"kind":"block_rect","name":"Removed necessary boardwalk","bounds":[21,18,27,20],"block":"35"}),
        json!({"kind":"rect","name":"Removed required understory","bounds":[28,17,30,21],"material":"floor"}),
        json!({"kind":"path","name":"Free dry bypass around required understory","points":[[27,18],[27,16],[30,16],[30,18]],"width_metatiles":1,"material":"floor"}),
        json!({"kind":"path","name":"Free dry river dam above necessary boardwalk","points":[[20,18],[20,16],[27,16],[27,18]],"width_metatiles":1,"material":"floor"}),
        json!({"kind":"point","name":"Removed actual native bank face","position":[14,12],"block":"01"}),
        json!({"kind":"point","name":"Removed actual Cut source","position":[35,10],"block":"3c"}),
        json!({"kind":"point","name":"Decorative Cut canopy","position":[2,2],"block":"5b"}),
        json!({"kind":"block_rect","name":"Unconnected river mouth","bounds":[22,0,25,2],"block":"35"}),
    ] {
        assert!(
            build_headwaters_modpack(&base, &append_layer(&layout, layer), &dossier).is_err(),
            "changed native topology contract was accepted"
        );
    }
}

#[test]
fn headwaters_rejects_weakened_story_terrain_and_false_shortcut_contracts() {
    let (base, layout, dossier) = fixture();
    for (field, value) in [
        ("required_flags", json!(["BADGE_7"])),
        ("required_grass_crossings", json!([])),
        ("one_way_ledges", json!([])),
        ("cut_gates", json!([])),
        ("shortcut", Value::Null),
    ] {
        let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
        bad[field] = value;
        assert!(
            build_headwaters_modpack(&base, &layout, &bad.to_string()).is_err(),
            "removed Headwaters contract {field} was accepted"
        );
    }
    for (field, value) in [
        ("minimum_saved_steps", json!(33)),
        ("compare_from", json!([71, 21])),
        ("compare_to", json!([71, 21])),
    ] {
        let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
        // Changing compare_from alone to its existing value would be no mutation.
        if field == "compare_from" {
            bad["shortcut"]
                .as_object_mut()
                .expect("shortcut")
                .remove(field);
        } else {
            bad["shortcut"][field] = value;
        }
        assert!(
            build_headwaters_modpack(&base, &layout, &bad.to_string()).is_err(),
            "false shortcut comparison {field} accepted"
        );
    }
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]);
    assert!(build_headwaters_modpack(&base, &layout, &bad.to_string()).is_err());
    bad = serde_json::from_str(&dossier).expect("dossier");
    bad["cut_gates"][0]["replacement_block"] = json!("01");
    assert!(build_headwaters_modpack(&base, &layout, &bad.to_string()).is_err());
    bad = serde_json::from_str(&dossier).expect("dossier");
    bad["trainers"][0]["tile"] = json!([0, 0]);
    assert!(build_headwaters_modpack(&base, &layout, &bad.to_string()).is_err());
    let bypass = append_layer(
        &layout,
        json!({"kind":"point","name":"Free Cut sidestep","position":[36,10],"block":"01"}),
    );
    assert!(
        build_headwaters_modpack(&base, &bypass, &dossier).is_err(),
        "false Cut savings through side opening"
    );
}
