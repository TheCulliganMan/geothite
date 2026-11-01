use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_duluth_gates};
use serde_json::Value;

struct Inputs {
    base: CompiledGamePack,
    revised: CompiledGamePack,
    historical: String,
    candidate: String,
}

fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-bemidji-return.crystalpack"),
        )
        .expect("published Bemidji parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/duluth-gold-gates.json"))
                .expect("gold-gate source");
        let revised =
            revise_duluth_gates(&base, &historical, &candidate).expect("gold-gate revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-duluth-gold-gates.crystalpack"),
        )
        .expect("written revision");
        assert_eq!(
            revised.identity().expect("identity"),
            written.identity().expect("written identity")
        );
        Inputs {
            base,
            revised,
            historical,
            candidate,
        }
    })
}

#[test]
#[ignore = "requires the published Bemidji parent and Duluth gate pack"]
fn duluth_gates_preserve_prior_gyms_all_collisions_battles_bindings_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnDuluthGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnDuluthGym"];
    let original = &a.maps["MnDuluthGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps, original.events.warps);
    assert_eq!(room.objects, original.objects);
    assert_eq!(room.blocks, original.blocks);
    assert_eq!(
        a.tilesets["mn_gym_8"].collision,
        b.tilesets["mn_gym_8"].collision
    );
    assert_eq!(a.trainers, b.trainers);
    assert_eq!(
        a.story_event_script_constants,
        b.story_event_script_constants
    );
    assert_eq!(a.field_moves, b.field_moves);
    assert_eq!(a.runtime_map_metadata, b.runtime_map_metadata);
    assert_eq!(a.map_dimensions, b.map_dimensions);
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    for (id, art) in &a.tilesets {
        if id != "mn_gym_8" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_8.json",
        "data/tilesets/mn_gym_8_metatiles.bin",
        "data/tilesets/mn_gym_8_palette_map.json",
        "gfx/tilesets/mn_gym_8.png",
        "gfx/tilesets/mn_gym_8.2bpp",
    ];
    assert_eq!(
        i.base.runtime_files().len(),
        i.revised.runtime_files().len()
    );
    for (path, bytes) in i.base.runtime_files() {
        if !owned.contains(&path.as_str()) {
            assert_eq!(
                Some(bytes),
                i.revised.runtime_files().get(path),
                "unrelated file {path}"
            );
        }
    }
    let mut uncached = b.clone();
    uncached.maps.remove("MnDuluthGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnDuluthGym")
        .expect("uncached native module");
    for label in [
        room.attributes.map_scripts_label.as_ref(),
        room.attributes.map_events_label.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if !room.scripts.contains_key(label) {
            rebuilt.scripts.remove(label);
        }
    }
    assert_eq!(*room, rebuilt);
}

#[test]
#[ignore = "requires the published Bemidji parent and Duluth gate pack"]
fn duluth_gates_reject_silver_doors_altered_ice_stops_skipped_trainers_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    for symbol in ["X", "Y", "Z", "V"] {
        let mut indistinct = source.clone();
        indistinct["gyms"][0]["art"][symbol] = serde_json::json!(["elite_four_room", "14"]);
        assert!(revise_duluth_gates(&i.base, &i.historical, &indistinct.to_string()).is_err());
    }
    let mut altered_stop = source.clone();
    altered_stop["gyms"][0]["art"]["B"] = serde_json::json!(["elite_four_room", "2d"]);
    assert!(revise_duluth_gates(&i.base, &i.historical, &altered_stop.to_string()).is_err());
    let mut altered_patch = source.clone();
    altered_patch["gyms"][0]["rows"][5] = "lIr#IIB".into();
    assert!(revise_duluth_gates(&i.base, &i.historical, &altered_patch.to_string()).is_err());
    let mut skip = source.clone();
    skip["gyms"][0]["gates"][3]["requires"] =
        serde_json::json!(["EVENT_MN_GYM_8_TRAINER_1_DEFEATED"]);
    assert!(revise_duluth_gates(&i.base, &i.historical, &skip.to_string()).is_err());
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_duluth_gates(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_duluth_gates(&i.revised, &i.historical, &i.candidate).is_err());
}
