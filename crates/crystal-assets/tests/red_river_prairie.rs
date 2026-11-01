//! Native publication contracts; production controller traversal is separate.

use std::fs;
use std::path::PathBuf;

use crystal_assets::{
    CompiledGamePack, build_red_river_prairie_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

fn fixture() -> (CompiledGamePack, String, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let source = std::env::var_os("CRYSTAL_RED_RIVER_PRAIRIE_BASE_PACK")
        .map_or_else(
            || root.join("content-packs/text-tui.crystalpack"),
            PathBuf::from,
        )
        .canonicalize()
        .expect("completed external source pack");
    let base = read_verified_compiled_game_pack(source).expect("verified external source");
    let read = |name| {
        fs::read_to_string(root.join("planning/minnesota/routes").join(name))
            .expect("authored prairie input")
    };
    (base, read("MN_R16.layout.json"), read("MN_R16.json"))
}

#[test]
fn prairie_publishes_clipped_native_maps_and_preserves_the_source_world() {
    let (base, layout, dossier) = fixture();
    let result = build_red_river_prairie_modpack(&base, &layout, &dossier)
        .expect("reviewed Red River Prairie publication");
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 2);
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
    for (column, width) in [(0, 32), (1, 16)] {
        let name = format!("MnR16C{column:02}R00");
        let neighbor = format!("MnR16C{:02}R00", 1 - column);
        let module = &result.data().maps[&name];
        assert_eq!(
            (module.attributes.width, module.attributes.height),
            (width, 24)
        );
        assert_eq!(module.blocks.len(), usize::from(width) * 24);
        assert_eq!(module.attributes.tileset_name, "mn_headwaters");
        assert_eq!(
            module.attributes.music,
            base.data().maps["Route34"].attributes.music
        );
        assert_eq!(module.attributes.border_block, 5);
        assert_eq!(module.attributes.connections.len(), 1);
        let connection = &module.attributes.connections[0];
        assert_eq!(connection.target_map, neighbor);
        assert_eq!(connection.offset, 0);
        assert_eq!(
            connection.direction,
            if column == 0 { "east" } else { "west" }
        );
        assert!(
            result.data().maps[&neighbor]
                .attributes
                .connections
                .iter()
                .any(|back| {
                    back.target_map == name
                        && back.offset == 0
                        && back.direction != connection.direction
                })
        );
        let metadata = &result.data().runtime_map_metadata[&name];
        assert_eq!(metadata.group_id, 56);
        assert_eq!(metadata.group_name, "MINNESOTA_RED_RIVER_PRAIRIE");
        assert_eq!(metadata.map_id, column + 1);
        assert_eq!((metadata.width, metadata.height), (width, 24));
        assert_eq!(
            result
                .data()
                .assemble_map_module_from_compiled_payloads(&name)
                .expect("native payload"),
            *module
        );
        assert!(result.data().wild_encounters.contains_key(&name));
        // There are no buildings or invented return warps on this route.
        assert!(module.events.warps.is_empty());
        totals[0] += module.trainer_scripts.len();
        totals[1] += module.script_field_pickups.len();
        totals[2] += module.events.bg_events.len();
        totals[3] += module.objects.len();
        for object in &module.objects {
            assert!(object.x < width * 2 && object.y < 48);
            let actor = ["trainers", "residents", "items"]
                .into_iter()
                .flat_map(|key| design[key].as_array().expect("actors"))
                .find(|actor| actor["id"].as_str() == Some(object.script.as_str()))
                .expect("original authored actor identity");
            assert_eq!(actor["tile"], json!([object.x + column * 64, object.y]));
            assert_eq!(object.object_identifier.as_deref(), actor["id"].as_str());
        }
    }
    assert_eq!(totals, [10, 4, 5, 17]);
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
    assert!(build_red_river_prairie_modpack(&result, &layout, &dossier).is_err());
}

fn append_layer(layout: &str, layer: Value) -> String {
    let mut value: Value = serde_json::from_str(layout).expect("layout");
    value["layers"].as_array_mut().expect("layers").push(layer);
    value.to_string()
}

#[test]
fn prairie_rejects_open_water_edges_incidental_cut_and_broken_crossings() {
    let (base, layout, dossier) = fixture();
    for (layer, diagnostic) in [
        (
            json!({"kind":"block_rect","name":"Invalid unconnected river mouth","bounds":[6,0,12,2],"block":"35"}),
            "Surf edge",
        ),
        (
            json!({"kind":"point","name":"Invalid decorative Cut canopy","position":[20,22],"block":"5b"}),
            "undeclared decorative Cut tree at 41,44",
        ),
        (
            json!({"kind":"point","name":"Removed native bridge rail","position":[8,11],"block":"01"}),
            "rail/deck",
        ),
        (
            json!({"kind":"point","name":"Opened native river pier end","position":[9,7],"block":"01"}),
            "pier",
        ),
    ] {
        let Err(error) =
            build_red_river_prairie_modpack(&base, &append_layer(&layout, layer), &dossier)
        else {
            panic!("changed collision contract must fail: {diagnostic}");
        };
        assert!(format!("{error:#}").contains(diagnostic), "{error:#}");
    }
    let bypass = append_layer(
        &layout,
        json!({
            "kind":"path", "name":"Invalid dry drainage dam", "material":"floor",
            "width_metatiles":1, "points":[[29,15],[29,10],[34,10],[34,15]]
        }),
    );
    let Err(error) = build_red_river_prairie_modpack(&base, &bypass, &dossier) else {
        panic!("decorative bypassed drainage crossing must fail");
    };
    assert!(
        format!("{error:#}").contains("necessary walking crossing"),
        "{error:#}"
    );
}

#[test]
fn prairie_rejects_weakened_badge_habitat_ledge_and_trainer_choice_contracts() {
    let (base, layout, dossier) = fixture();
    let mut bad: Value = serde_json::from_str(&dossier).expect("dossier");
    bad["required_flags"] = json!(["BADGE_7"]);
    assert!(build_red_river_prairie_modpack(&base, &layout, &bad.to_string()).is_err());
    bad = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["return_passage_core"] = json!([0, 0, 1, 1]);
    assert!(build_red_river_prairie_modpack(&base, &layout, &bad.to_string()).is_err());
    bad = serde_json::from_str(&dossier).expect("dossier");
    bad["one_way_ledges"][0]["minimum_return_steps"] = json!(500);
    assert!(build_red_river_prairie_modpack(&base, &layout, &bad.to_string()).is_err());
    for layer in [
        json!({"kind":"rect","name":"Removed required cottonwood habitat","material":"floor","bounds":[41,9,43,14]}),
        json!({"kind":"point","name":"Removed native hayfield drop","position":[22,8],"block":"01"}),
        json!({"kind":"path","name":"Free dry bypass below unbeaten Camper","material":"floor","width_metatiles":1,"points":[[14,12],[14,13],[15,13],[15,12]]}),
    ] {
        assert!(
            build_red_river_prairie_modpack(&base, &append_layer(&layout, layer), &dossier)
                .is_err(),
            "required habitat, directed bank or paid trainer alternative was lost"
        );
    }
}
