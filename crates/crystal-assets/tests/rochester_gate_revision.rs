use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_rochester_gates};
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
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-rochester-readable.crystalpack"),
        )
        .expect("reviewed parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical design");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/rochester-gates.json"))
                .expect("gate design");
        let revised = revise_rochester_gates(&base, &historical, &candidate).expect("safe course");
        let published = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-rochester-gates.crystalpack"),
        )
        .expect("written publication");
        assert_eq!(
            revised.identity().expect("identity"),
            published.identity().expect("identity")
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
#[ignore = "requires readable parent and published two-switch Rochester pack"]
fn rochester_gate_revision_preserves_other_maps_battles_numbers_bindings_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, room) in &a.maps {
        if id != "MnRochesterGym" {
            assert_eq!(room, &b.maps[id], "unrelated room {id}");
        }
    }
    assert_eq!(a.trainers, b.trainers);
    for (id, number) in &a.story_event_script_constants.global {
        assert_eq!(
            Some(number),
            b.story_event_script_constants.global.get(id),
            "existing {id}"
        );
    }
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    for (id, tileset) in &a.tilesets {
        if id != "mn_gym_1" {
            assert_eq!(tileset, &b.tilesets[id], "unrelated art {id}");
        }
    }
    let files = [
        "data/tilesets/mn_gym_1.json",
        "data/tilesets/mn_gym_1_metatiles.bin",
        "data/tilesets/mn_gym_1_palette_map.json",
        "gfx/tilesets/mn_gym_1.png",
        "gfx/tilesets/mn_gym_1.2bpp",
    ];
    assert_eq!(
        i.base.runtime_files().len(),
        i.revised.runtime_files().len()
    );
    for (path, bytes) in i.base.runtime_files() {
        if !files.contains(&path.as_str()) {
            assert_eq!(
                Some(bytes),
                i.revised.runtime_files().get(path),
                "unrelated file {path}"
            );
        }
    }
    let room = &b.maps["MnRochesterGym"];
    assert_eq!(room.events.warps.len(), 3);
    for (new, old) in room
        .events
        .warps
        .iter()
        .zip(&a.maps["MnRochesterGym"].events.warps)
    {
        let mut expected = old.clone();
        expected.x = new.x;
        expected.y = new.y;
        assert_eq!(new, &expected, "indexed city destinations survive");
    }
    let mut uncached = b.clone();
    uncached.maps.remove("MnRochesterGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnRochesterGym")
        .expect("uncached native payloads");
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
    assert_eq!(rebuilt, *room);
}

#[test]
#[ignore = "requires readable parent and published two-switch Rochester pack"]
fn rochester_gate_revision_rejects_bypass_wrong_party_and_repeat_installation() {
    let i = inputs();
    let mut bypass: Value = serde_json::from_str(&i.candidate).expect("source");
    bypass["gyms"][0]["rows"][4] = "#BGpG##".into();
    let error = revise_rochester_gates(&i.base, &i.historical, &bypass.to_string())
        .expect_err("half-wall must not bypass the required lane change");
    assert!(
        error.to_string().contains("return to the right lane"),
        "{error:#}"
    );
    let mut wrong: Value = serde_json::from_str(&i.candidate).expect("source");
    wrong["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_rochester_gates(&i.base, &i.historical, &wrong.to_string()).is_err());
    wrong = serde_json::from_str(&i.candidate).expect("source");
    wrong["gyms"][0]["routers"][0]["modes"]
        .as_array_mut()
        .expect("positions")
        .pop();
    assert!(revise_rochester_gates(&i.base, &i.historical, &wrong.to_string()).is_err());
    assert!(revise_rochester_gates(&i.revised, &i.historical, &i.candidate).is_err());
}
