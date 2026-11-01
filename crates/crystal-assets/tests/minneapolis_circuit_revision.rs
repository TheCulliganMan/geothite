use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_minneapolis_circuit,
};
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
            root.join("content-packs/minnesota-mankato-terraces.crystalpack"),
        )
        .expect("published Mankato parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/minneapolis-circuit.json"))
                .expect("circuit source");
        let revised =
            revise_minneapolis_circuit(&base, &historical, &candidate).expect("circuit revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-minneapolis-circuit.crystalpack"),
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
#[ignore = "requires the published Mankato parent and Minneapolis circuit pack"]
fn minneapolis_circuit_preserves_prior_gyms_battles_bindings_constants_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnMinneapolisGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnMinneapolisGym"];
    let original = &a.maps["MnMinneapolisGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps, original.events.warps);
    let mut old_objects = original.objects.clone();
    let jossa = old_objects
        .iter_mut()
        .find(|o| o.script == "MN_GYM_3_TRAINER_2")
        .expect("original Jossa");
    jossa.x = 4;
    jossa.y = 6;
    assert_eq!(room.objects, old_objects);
    assert_ne!(room.blocks, original.blocks);
    assert_eq!(a.trainers, b.trainers);
    assert_eq!(
        a.story_event_script_constants,
        b.story_event_script_constants
    );
    assert_eq!(a.runtime_map_metadata, b.runtime_map_metadata);
    assert_eq!(a.map_dimensions, b.map_dimensions);
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    for (id, art) in &a.tilesets {
        if id != "mn_gym_3" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_3.json",
        "data/tilesets/mn_gym_3_metatiles.bin",
        "data/tilesets/mn_gym_3_palette_map.json",
        "gfx/tilesets/mn_gym_3.png",
        "gfx/tilesets/mn_gym_3.2bpp",
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
    uncached.maps.remove("MnMinneapolisGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnMinneapolisGym")
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
#[ignore = "requires the published Mankato parent and Minneapolis circuit pack"]
fn minneapolis_circuit_rejects_deadlock_skipped_trainer_hidden_barrier_party_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    let mut deadlock = source.clone();
    deadlock["gyms"][0]["switch_puzzle"]["first_requires"] =
        serde_json::json!(["EVENT_MN_GYM_3_TRAINER_2_DEFEATED"]);
    assert!(revise_minneapolis_circuit(&i.base, &i.historical, &deadlock.to_string()).is_err());
    let mut skip = source.clone();
    skip["gyms"][0]["gates"][1]["requires"] = serde_json::json!([
        "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_3_FIRST_SWITCH_ON"
    ]);
    assert!(revise_minneapolis_circuit(&i.base, &i.historical, &skip.to_string()).is_err());
    let mut invisible = source.clone();
    invisible["gyms"][0]["art"]["A"] = serde_json::json!(["facility", "03"]);
    assert!(revise_minneapolis_circuit(&i.base, &i.historical, &invisible.to_string()).is_err());
    let mut missing = source.clone();
    missing["gyms"][0]["barriers"] = serde_json::json!(["B"]);
    assert!(revise_minneapolis_circuit(&i.base, &i.historical, &missing.to_string()).is_err());
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_minneapolis_circuit(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_minneapolis_circuit(&i.revised, &i.historical, &i.candidate).is_err());
}
