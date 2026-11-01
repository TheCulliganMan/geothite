use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_bemidji_return};
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
            root.join("content-packs/minnesota-brainerd-winch.crystalpack"),
        )
        .expect("published Brainerd parent");
        let historical =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
                .expect("historical source");
        let candidate =
            std::fs::read_to_string(root.join("planning/minnesota/gyms/bemidji-return.json"))
                .expect("garden source");
        let revised =
            revise_bemidji_return(&base, &historical, &candidate).expect("garden revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-bemidji-return.crystalpack"),
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
#[ignore = "requires the published Brainerd parent and Mankato terrace pack"]
fn bemidji_return_preserves_prior_gyms_native_cut_battles_bindings_art_and_audio() {
    let i = inputs();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (id, map) in &a.maps {
        if id != "MnBemidjiGym" {
            assert_eq!(map, &b.maps[id], "unrelated map {id}");
        }
    }
    let room = &b.maps["MnBemidjiGym"];
    let original = &a.maps["MnBemidjiGym"];
    assert_eq!(room.attributes, original.attributes);
    assert_eq!(room.events.warps, original.events.warps);
    assert_eq!(room.objects, original.objects);
    assert_ne!(room.blocks, original.blocks);
    for y in 0..20 {
        for x in 0..12 {
            let index = y / 2 * 6 + x / 2;
            let q = y % 2 * 2 + x % 2;
            let old =
                &a.tilesets["mn_gym_7"].collision[&format!("{:02x}", original.blocks[index])][q];
            let new = &b.tilesets["mn_gym_7"].collision[&format!("{:02x}", room.blocks[index])][q];
            if (6..8).contains(&x) && (10..12).contains(&y) {
                assert_eq!(new, "FLOOR");
            } else {
                assert_eq!(old, new, "unchanged collision at {x},{y}");
            }
        }
    }

    assert_eq!(a.trainers, b.trainers);
    let mut constants = b.story_event_script_constants.clone();
    let added = constants
        .global
        .remove("EVENT_MN_GYM_7_RETURN_OPEN")
        .expect("one new return event");
    assert!(
        a.story_event_script_constants
            .global
            .iter()
            .filter(|(k, _)| k.starts_with("EVENT_"))
            .all(|(_, v)| *v < added)
    );
    assert_eq!(a.story_event_script_constants, constants);
    let mut old_fields = a.field_moves.clone();
    let mut new_fields = b.field_moves.clone();
    let old_cut = old_fields
        .cut
        .replacements
        .remove("mn_gym_7")
        .expect("original owned Cut behavior");
    let new_cut = new_fields
        .cut
        .replacements
        .remove("mn_gym_7")
        .expect("reinstalled owned Cut behavior");
    assert_eq!(old_fields, new_fields, "all unrelated field behavior");
    assert_eq!(old_cut.len(), 2);
    assert_eq!(new_cut.len(), 2);
    for (x, y) in [(3_usize, 12_usize), (7, 6)] {
        let old = &old_cut[&original.blocks[y / 2 * 6 + x / 2]];
        let new = &new_cut[&room.blocks[y / 2 * 6 + x / 2]];
        let mut normalized = new.clone();
        normalized.replacement_block_id = old.replacement_block_id;
        assert_eq!(*old, normalized, "native replacement metadata");
    }

    assert_eq!(a.runtime_map_metadata, b.runtime_map_metadata);
    assert_eq!(a.map_dimensions, b.map_dimensions);
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    for (id, art) in &a.tilesets {
        if id != "mn_gym_7" {
            assert_eq!(art, &b.tilesets[id], "unrelated tileset {id}");
        }
    }
    let owned = [
        "data/tilesets/mn_gym_7.json",
        "data/tilesets/mn_gym_7_metatiles.bin",
        "data/tilesets/mn_gym_7_palette_map.json",
        "gfx/tilesets/mn_gym_7.png",
        "gfx/tilesets/mn_gym_7.2bpp",
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
    uncached.maps.remove("MnBemidjiGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnBemidjiGym")
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
#[ignore = "requires the published Brainerd parent and Bemidji return pack"]
fn bemidji_return_rejects_early_door_cut_bypass_changed_replacement_party_and_repeat() {
    let i = inputs();
    let source: Value = serde_json::from_str(&i.candidate).expect("source");
    let mut early = source.clone();
    early["gyms"][0]["gates"][0]["requires"] =
        serde_json::json!(["EVENT_MN_GYM_7_TRAINER_1_DEFEATED"]);
    assert!(revise_bemidji_return(&i.base, &i.historical, &early.to_string()).is_err());
    let mut bypass = source.clone();
    bypass["gyms"][0]["rows"][6] = "#c#=##".into();
    assert!(revise_bemidji_return(&i.base, &i.historical, &bypass.to_string()).is_err());
    let mut cut = source.clone();
    cut["gyms"][0]["cut_gates"][0]["replacement_symbol"] = ".".into();
    assert!(revise_bemidji_return(&i.base, &i.historical, &cut.to_string()).is_err());
    let mut no_shortcut = source.clone();
    no_shortcut["gyms"][0]["rows"][5] = "p.##.p".into();
    assert!(revise_bemidji_return(&i.base, &i.historical, &no_shortcut.to_string()).is_err());
    let mut party = source;
    party["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(revise_bemidji_return(&i.base, &i.historical, &party.to_string()).is_err());
    assert!(revise_bemidji_return(&i.revised, &i.historical, &i.candidate).is_err());
}
