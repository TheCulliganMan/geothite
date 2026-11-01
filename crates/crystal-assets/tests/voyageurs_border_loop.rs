//! Native border publication and reciprocal cave contracts, not controller play.

use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    CompiledGamePack, build_red_river_prairie_modpack, build_voyageurs_border_loop_modpack,
    read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository")
}

fn authored(name: &str) -> String {
    fs::read_to_string(repository().join("planning/minnesota/routes").join(name))
        .expect("authored route source")
}

fn fixture() -> (CompiledGamePack, String, String) {
    let source = std::env::var_os("CRYSTAL_VOYAGEURS_BORDER_BASE_PACK")
        .map_or_else(
            || repository().join("content-packs/text-tui.crystalpack"),
            PathBuf::from,
        )
        .canonicalize()
        .expect("completed external source pack");
    (
        read_verified_compiled_game_pack(source).expect("verified external source"),
        authored("MN_R20.layout.json"),
        authored("MN_R20.json"),
    )
}

fn kind<'a>(pack: &'a CompiledGamePack, name: &str, [x, y]: [u16; 2]) -> &'a str {
    let module = &pack.data().maps[name];
    let block = module.blocks
        [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
    &pack.data().tilesets[&module.attributes.tileset_name].collision[&format!("{block:02x}")]
        [usize::from(y % 2) * 2 + usize::from(x % 2)]
}

#[test]
fn voyageurs_composes_five_maps_and_preserves_source_and_prior_prairie() {
    let (source, layout, dossier) = fixture();
    let base = build_red_river_prairie_modpack(
        &source,
        &authored("MN_R16.layout.json"),
        &authored("MN_R16.json"),
    )
    .expect("previous prairie overlay");
    let result = build_voyageurs_border_loop_modpack(&base, &layout, &dossier)
        .expect("native Voyageurs overlay");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 5);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(result.data().items, base.data().items);
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("result audio"),
        base.audio_manifest().expect("source audio")
    );
    for (name, map) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(map),
            "changed old map {name}"
        );
    }
    for (name, metadata) in &base.data().runtime_map_metadata {
        assert_eq!(result.data().runtime_map_metadata.get(name), Some(metadata));
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(result.data().trainers.trainers.get(name), Some(trainer));
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(name),
            Some(tileset),
            "changed art {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source file {name}"
        );
    }
    let mut totals = [0, 0, 0, 0];
    let design: Value = serde_json::from_str(&dossier).expect("dossier");
    for row in 0..2 {
        for column in 0..2 {
            let name = format!("MnR20C{column:02}R{row:02}");
            let module = &result.data().maps[&name];
            let height = if row == 0 { 24 } else { 8 };
            assert_eq!(
                (module.attributes.width, module.attributes.height),
                (32, height)
            );
            assert_eq!(module.attributes.tileset_name, "mn_headwaters");
            assert_eq!(
                module.attributes.music,
                base.data().maps["Route42"].attributes.music
            );
            assert_eq!(module.blocks.len(), 32 * usize::from(height));
            assert_eq!(module.attributes.connections.len(), 2);
            for connection in &module.attributes.connections {
                let opposite = match connection.direction.as_str() {
                    "north" => "south",
                    "south" => "north",
                    "east" => "west",
                    "west" => "east",
                    _ => panic!("native cardinal neighbor required"),
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
            assert_eq!(
                (metadata.group_id, metadata.map_id),
                (60, row * 2 + column + 1)
            );
            assert_eq!(metadata.group_name, "MINNESOTA_VOYAGEURS_BORDER_LOOP");
            assert_eq!((metadata.width, metadata.height), (32, height));
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
            for object in &module.objects {
                assert!(object.x < 64 && object.y < height * 2);
                let actor = ["trainers", "residents", "items"]
                    .into_iter()
                    .flat_map(|key| design[key].as_array().expect("actors"))
                    .find(|actor| actor["id"].as_str() == Some(object.script.as_str()))
                    .expect("original exterior actor");
                assert_eq!(
                    actor["tile"],
                    json!([object.x + column * 64, object.y + row * 48])
                );
                assert_eq!(object.object_identifier.as_deref(), actor["id"].as_str());
            }
        }
    }
    assert_eq!(totals, [10, 3, 5, 16]);
    for actor in design["trainers"].as_array().expect("trainers") {
        let id = actor["id"].as_str().expect("trainer identity");
        let native = &result.data().trainers.trainers[id];
        let party = native
            .party
            .iter()
            .map(|member| {
                assert!(
                    !member.moves.is_empty(),
                    "missing level-derived native moves for {id}"
                );
                json!({"species":member.species,"level":member.level})
            })
            .collect::<Vec<_>>();
        assert_eq!(Value::Array(party), actor["party"]);
        let request = result
            .data()
            .maps
            .values()
            .find_map(|module| module.trainer_scripts.get(id))
            .expect("native trainer script");
        assert_eq!(
            Some(request.event_flag.as_str()),
            actor["defeated_flag"].as_str()
        );
    }
    let cave = &result.data().maps["MnR20Cave1F"];
    assert_eq!((cave.attributes.width, cave.attributes.height), (9, 8));
    assert_eq!(cave.attributes.tileset_name, "cave");
    assert_eq!(cave.attributes.border_block, 1);
    assert_eq!(cave.objects.len(), 1);
    assert_eq!(cave.script_field_pickups.len(), 1);
    let metadata = &result.data().runtime_map_metadata["MnR20Cave1F"];
    assert_eq!((metadata.group_id, metadata.map_id), (60, 5));
    assert_eq!(cave.events.warps.len(), 2);
    let west = &result.data().maps["MnR20C00R00"];
    let east = &result.data().maps["MnR20C01R00"];
    for (map, tile, passive, target) in
        [(west, [60, 33], [60, 34], 1), (east, [12, 31], [12, 32], 2)]
    {
        assert_eq!(map.events.warps.len(), 2);
        assert_eq!([map.events.warps[0].x, map.events.warps[0].y], tile);
        assert_eq!(map.events.warps[0].target_warp_id, target);
        assert_eq!(
            Some(map.events.warps[0].target_map_constant.as_str()),
            cave.attributes.map_constant.as_deref()
        );
        assert_eq!([map.events.warps[1].x, map.events.warps[1].y], passive);
        assert_eq!(kind(&result, &map.id, tile), "CAVE");
        assert_eq!(kind(&result, &map.id, passive), "FLOOR");
    }
    for (index, exit, exterior) in [(0, [5, 13], west), (1, [13, 13], east)] {
        let warp = &cave.events.warps[index];
        assert_eq!([warp.x, warp.y], exit);
        assert_eq!(kind(&result, "MnR20Cave1F", exit), "LADDER");
        assert_eq!(
            Some(warp.target_map_constant.as_str()),
            exterior.attributes.map_constant.as_deref()
        );
        assert_eq!(
            warp.target_warp_id, 2,
            "room exit must use its own passive exterior return"
        );
    }
    for actor in design["items"]
        .as_array()
        .expect("exterior pickups")
        .iter()
        .chain(
            design["interiors"][0]["items"]
                .as_array()
                .expect("cave pickup"),
        )
    {
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
            .expect("native item ball");
        assert_eq!(pickup.item_id.as_deref(), actor["item"].as_str());
        assert_eq!(
            u64::from(pickup.quantity),
            actor["quantity"].as_u64().expect("quantity")
        );
        assert_eq!(Some(object.event_flag.as_str()), actor["flag"].as_str());
    }
    assert!(build_voyageurs_border_loop_modpack(&result, &layout, &dossier).is_err());
}

fn append_layer(layout: &str, layer: Value) -> String {
    let mut value: Value = serde_json::from_str(layout).expect("layout");
    value["layers"].as_array_mut().expect("layers").push(layer);
    value.to_string()
}

#[test]
fn voyageurs_rejects_wrong_mouths_and_weakened_border_authority() {
    let (base, layout, dossier) = fixture();
    for change in 0..8 {
        let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
        match change {
            0 => bad["interiors"][0]["exit_doors"] = json!([[76, 31], [60, 33]]),
            1 => bad["doors"][1]["target_warp"] = json!(1),
            2 => bad["interiors"][0]["exits"][1] = json!([5, 13]),
            3 => bad["doors"][1]["return_warp_index"] = json!(1),
            4 => bad["doors"][1]["return_tile"] = json!([76, 31]),
            5 => bad["quest"] = json!("Q07"),
            6 => bad["interiors"][0]["items"][0]["item"] = json!("PP_UP"),
            _ => bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]),
        }
        assert!(
            build_voyageurs_border_loop_modpack(&base, &layout, &bad.to_string()).is_err(),
            "wrong mouth, quest authority or return contract accepted: {change}"
        );
    }
    for omitted in 0..9 {
        let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
        bad["required_flags"]
            .as_array_mut()
            .expect("flags")
            .remove(omitted);
        assert!(
            build_voyageurs_border_loop_modpack(&base, &layout, &bad.to_string()).is_err(),
            "badge/lighthouse permission removed: {omitted}"
        );
    }
}

#[test]
fn voyageurs_rejects_removed_or_bypassed_crossings_and_open_habitat_or_water() {
    let (base, layout, dossier) = fixture();
    for layer in [
        json!({"kind":"rect","name":"Removed western boardwalk","material":"forest","bounds":[19,20,25,22]}),
        json!({"kind":"rect","name":"Removed required shore habitat","material":"floor","bounds":[56,15,58,21]}),
        json!({"kind":"point","name":"Removed native shelter ledge","position":[9,25],"block":"01"}),
        json!({"kind":"block_rect","name":"Opened unconnected northern lake edge","bounds":[2,0,3,3],"block":"35"}),
        json!({"kind":"point","name":"Incidental decorative Cut tree","position":[16,29],"block":"5b"}),
    ] {
        assert!(
            build_voyageurs_border_loop_modpack(&base, &append_layer(&layout, layer), &dossier)
                .is_err(),
            "removed crossing, habitat, hop, Surf boundary or Cut contract accepted"
        );
    }
    let bypass = append_layer(
        &layout,
        json!({
            "kind":"path","name":"Invalid dry western outlet dam","material":"floor",
            "width_metatiles":1,"points":[[17,21],[17,23],[25,23],[25,21]]
        }),
    );
    let Err(error) = build_voyageurs_border_loop_modpack(&base, &bypass, &dossier) else {
        panic!("bypassed western outlet crossing must fail");
    };
    assert!(
        format!("{error:#}").contains("necessary walking crossing"),
        "{error:#}"
    );
}
