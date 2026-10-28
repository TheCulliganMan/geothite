use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_brainerd_winch};
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
            root.join("content-packs/minnesota-st-cloud-arrows.crystalpack"),
        )
        .expect("published St. Cloud parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/brainerd-winch.json"))
                .expect("winch source");
        let revised =
            revise_brainerd_winch(&base, &historical, &candidate).expect("winch revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-brainerd-winch.crystalpack"),
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
#[ignore = "requires the published St. Cloud parent and Mankato terrace pack"]
fn brainerd_winch_preserves_prior_gyms_battles_bindings_constants_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnBrainerdGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnBrainerdGym"];
    let original = &a.maps["MnBrainerdGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps, original.events.warps);
    assert_eq!(room.objects, original.objects);
    assert_ne!(room.blocks, original.blocks);
    for y in 0..16 {
        for x in 0..10 {
            let index = y / 2 * 5 + x / 2;
            let quadrant = y % 2 * 2 + x % 2;
            let old = &a.tilesets["mn_gym_6"].collision[&format!("{:02x}", original.blocks[index])]
                [quadrant];
            let new =
                &b.tilesets["mn_gym_6"].collision[&format!("{:02x}", room.blocks[index])][quadrant];
            if (6..8).contains(&x) && (12..14).contains(&y) {
                assert_eq!(new, "WALL");
            } else {
                assert_eq!(old, new, "unrelated collision at {x},{y}");
            }
        }
    }

    assert_eq!(a.trainers, b.trainers);
    assert_eq!(
        a.story_event_script_constants,
        b.story_event_script_constants
    );
    assert_eq!(a.runtime_map_metadata, b.runtime_map_metadata);
    assert_eq!(a.map_dimensions, b.map_dimensions);
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    for (id, art) in &a.tilesets {
        if id != "mn_gym_6" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_6.json",
        "data/tilesets/mn_gym_6_metatiles.bin",
        "data/tilesets/mn_gym_6_palette_map.json",
        "gfx/tilesets/mn_gym_6.png",
        "gfx/tilesets/mn_gym_6.2bpp",
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
    uncached.maps.remove("MnBrainerdGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnBrainerdGym")
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
#[ignore = "requires the published St. Cloud parent and Brainerd winch pack"]
fn brainerd_winch_rejects_menu_control_missing_hardware_skipped_trainer_party_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    let mut menu = source.clone();
    menu["gyms"][0]["routers"][0]["toggle"] = false.into();
    assert!(revise_brainerd_winch(&i.base, &i.historical, &menu.to_string()).is_err());
    let mut operator = source.clone();
    operator["gyms"][0]["routers"][0]["operator"] = "MN_GYM_6_TRAINER_1".into();
    assert!(revise_brainerd_winch(&i.base, &i.historical, &operator.to_string()).is_err());
    let mut missing = source.clone();
    missing["gyms"][0]["rows"][6] = "lL.Lr".into();
    assert!(revise_brainerd_winch(&i.base, &i.historical, &missing.to_string()).is_err());
    let mut skip = source.clone();
    skip["gyms"][0]["gates"][2]["requires"] =
        serde_json::json!(["EVENT_MN_GYM_6_TRAINER_1_DEFEATED"]);
    assert!(revise_brainerd_winch(&i.base, &i.historical, &skip.to_string()).is_err());
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_brainerd_winch(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_brainerd_winch(&i.revised, &i.historical, &i.candidate).is_err());
}
