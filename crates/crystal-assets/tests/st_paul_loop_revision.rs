use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_st_paul_loop};
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
            root.join("content-packs/minnesota-minneapolis-circuit.crystalpack"),
        )
        .expect("published Minneapolis parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/st-paul-loop.json"))
                .expect("loop source");
        let revised = revise_st_paul_loop(&base, &historical, &candidate).expect("loop revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-st-paul-loop.crystalpack"),
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
#[ignore = "requires the published Minneapolis parent and St. Paul loop pack"]
fn st_paul_loop_preserves_prior_gyms_battles_bindings_constants_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnStPaulGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnStPaulGym"];
    let original = &a.maps["MnStPaulGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps.len(), original.events.warps.len());
    assert_eq!(room.events.warps[..3], original.events.warps[..3]);
    let source: Value = serde_json::from_str(&i.candidate).expect("loop source");
    let ends = source["gyms"][0]["lifts"]
        .as_array()
        .expect("pairs")
        .iter()
        .flat_map(|lift| lift["ends_core"].as_array().expect("ends"));
    for ((new, old), at) in room.events.warps[3..]
        .iter()
        .zip(&original.events.warps[3..])
        .zip(ends)
    {
        let mut expected = old.clone();
        expected.x = at[0].as_u64().expect("x") as u16;
        expected.y = at[1].as_u64().expect("y") as u16;
        assert_eq!(
            *new, expected,
            "only the declared internal pad position changes"
        );
    }
    assert_eq!(room.objects, original.objects);
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
        if id != "mn_gym_4" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_4.json",
        "data/tilesets/mn_gym_4_metatiles.bin",
        "data/tilesets/mn_gym_4_palette_map.json",
        "gfx/tilesets/mn_gym_4.png",
        "gfx/tilesets/mn_gym_4.2bpp",
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
    uncached.maps.remove("MnStPaulGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnStPaulGym")
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
#[ignore = "requires the published Minneapolis parent and St. Paul loop pack"]
fn st_paul_loop_rejects_broken_pair_painted_pad_skipped_trainer_party_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    let mut broken = source.clone();
    broken["gyms"][0]["lifts"][2]["ends_core"][1] = serde_json::json!([11, 7]);
    assert!(revise_st_paul_loop(&i.base, &i.historical, &broken.to_string()).is_err());
    let mut painted = source.clone();
    painted["gyms"][0]["art"]["p"] = serde_json::json!(["underground", "0d"]);
    assert!(revise_st_paul_loop(&i.base, &i.historical, &painted.to_string()).is_err());
    let mut skip = source.clone();
    skip["gyms"][0]["gates"][0]["requires"] =
        serde_json::json!(["EVENT_MN_GYM_4_TRAINER_1_DEFEATED"]);
    assert!(revise_st_paul_loop(&i.base, &i.historical, &skip.to_string()).is_err());
    let mut extra_pad = source.clone();
    extra_pad["gyms"][0]["rows"][6] = "p..v...".into();
    assert!(revise_st_paul_loop(&i.base, &i.historical, &extra_pad.to_string()).is_err());
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_st_paul_loop(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_st_paul_loop(&i.revised, &i.historical, &i.candidate).is_err());
}
