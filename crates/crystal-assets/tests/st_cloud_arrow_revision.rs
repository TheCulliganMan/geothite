use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_st_cloud_arrows};
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
            root.join("content-packs/minnesota-st-paul-loop.crystalpack"),
        )
        .expect("published St. Paul parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/st-cloud-arrows.json"))
                .expect("arrow source");
        let revised =
            revise_st_cloud_arrows(&base, &historical, &candidate).expect("arrow revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-st-cloud-arrows.crystalpack"),
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
#[ignore = "requires the published St. Paul parent and Mankato terrace pack"]
fn st_cloud_arrows_preserve_prior_gyms_all_collisions_battles_bindings_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnStCloudGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnStCloudGym"];
    let original = &a.maps["MnStCloudGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps, original.events.warps);
    assert_eq!(room.objects, original.objects);
    assert_eq!(room.blocks, original.blocks);
    assert_eq!(
        a.tilesets["mn_gym_5"].collision,
        b.tilesets["mn_gym_5"].collision
    );
    assert_eq!(a.trainers, b.trainers);
    assert_eq!(
        a.story_event_script_constants,
        b.story_event_script_constants
    );
    assert_eq!(a.runtime_map_metadata, b.runtime_map_metadata);
    assert_eq!(a.map_dimensions, b.map_dimensions);
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    for (id, art) in &a.tilesets {
        if id != "mn_gym_5" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_5.json",
        "data/tilesets/mn_gym_5_metatiles.bin",
        "data/tilesets/mn_gym_5_palette_map.json",
        "gfx/tilesets/mn_gym_5.png",
        "gfx/tilesets/mn_gym_5.2bpp",
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
    uncached.maps.remove("MnStCloudGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnStCloudGym")
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
#[ignore = "requires the published St. Paul parent and St. Cloud arrow pack"]
fn st_cloud_arrows_reject_missing_marks_reversed_wind_changed_gate_party_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    let mut missing = source.clone();
    missing["gyms"][0]["wind_arrows"] = false.into();
    assert!(revise_st_cloud_arrows(&i.base, &i.historical, &missing.to_string()).is_err());
    let mut reversed = source.clone();
    reversed["gyms"][0]["winds"][">"] = "WALK_LEFT".into();
    assert!(revise_st_cloud_arrows(&i.base, &i.historical, &reversed.to_string()).is_err());
    let mut skip = source.clone();
    skip["gyms"][0]["gates"][1]["requires"] =
        serde_json::json!(["EVENT_MN_GYM_5_TRAINER_1_DEFEATED"]);
    assert!(revise_st_cloud_arrows(&i.base, &i.historical, &skip.to_string()).is_err());
    let mut geometry = source.clone();
    geometry["gyms"][0]["rows"][4] = "#...<<..#".into();
    assert!(revise_st_cloud_arrows(&i.base, &i.historical, &geometry.to_string()).is_err());
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_st_cloud_arrows(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_st_cloud_arrows(&i.revised, &i.historical, &i.candidate).is_err());
}
