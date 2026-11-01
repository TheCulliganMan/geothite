use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_mankato_terraces};
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
            root.join("content-packs/minnesota-rochester-gates.crystalpack"),
        )
        .expect("published Rochester parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/mankato-terraces.json"))
                .expect("terrace source");
        let revised =
            revise_mankato_terraces(&base, &historical, &candidate).expect("terrace revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-mankato-terraces.crystalpack"),
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
#[ignore = "requires the published Rochester parent and Mankato terrace pack"]
fn mankato_terraces_preserve_rochester_other_maps_battles_bindings_constants_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnMankatoGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnMankatoGym"];
    let original = &a.maps["MnMankatoGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps, original.events.warps);
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
        if id != "mn_gym_2" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_2.json",
        "data/tilesets/mn_gym_2_metatiles.bin",
        "data/tilesets/mn_gym_2_palette_map.json",
        "gfx/tilesets/mn_gym_2.png",
        "gfx/tilesets/mn_gym_2.2bpp",
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
    uncached.maps.remove("MnMankatoGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnMankatoGym")
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
#[ignore = "requires the published Rochester parent and Mankato terrace pack"]
fn mankato_terraces_reject_fake_hop_ascent_bypass_missing_stair_changed_party_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    let mut fake = source.clone();
    fake["gyms"][0]["art"]["h"] = serde_json::json!(["cave", "02"]);
    assert!(
        revise_mankato_terraces(&i.base, &i.historical, &fake.to_string()).is_err(),
        "a floor cannot impersonate a native ledge"
    );
    let mut bypass = source.clone();
    bypass["gyms"][0]["rows"][5] = "l......r".into();
    assert!(
        revise_mankato_terraces(&i.base, &i.historical, &bypass.to_string()).is_err(),
        "opening the ascent wall must invalidate the one-way return contract"
    );
    let mut stair = source.clone();
    stair["gyms"][0]["rows"][6] = "l.p....r".into();
    assert!(revise_mankato_terraces(&i.base, &i.historical, &stair.to_string()).is_err());
    let mut painted_stair = source.clone();
    painted_stair["gyms"][0]["art"]["t"] = serde_json::json!(["cave", "02"]);
    assert!(
        revise_mankato_terraces(&i.base, &i.historical, &painted_stair.to_string()).is_err(),
        "ordinary floor art must not impersonate the promised visible stairs"
    );
    let mut skip = source.clone();
    skip["gyms"][0]["gates"][1]["requires"] =
        serde_json::json!(["EVENT_MN_GYM_2_TRAINER_1_DEFEATED"]);
    assert!(
        revise_mankato_terraces(&i.base, &i.historical, &skip.to_string()).is_err(),
        "Jorel must remain required to open the upper stair"
    );
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_mankato_terraces(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_mankato_terraces(&i.revised, &i.historical, &i.candidate).is_err());
}
