//! Exact preservation across the reviewed Gym3 import into the northern story.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, MapModule, read_verified_compiled_game_pack,
    replace_installed_minneapolis_gym,
};

const MAP: &str = "MnMinneapolisGym";
const CONSTANT: &str = "MN_MINNEAPOLIS_GYM";
const TRAINERS: [&str; 3] = ["MN_GYM_3", "MN_GYM_3_TRAINER_1", "MN_GYM_3_TRAINER_2"];
const NEW_EVENTS: [&str; 3] = [
    "EVENT_MN_GYM_3_FIRST_GATE_OPEN",
    "EVENT_MN_GYM_3_FIRST_SWITCH_ON",
    "EVENT_MN_GYM_3_SECOND_SWITCH_ON",
];
const FILES: [&str; 5] = [
    "data/tilesets/mn_gym_3.json",
    "data/tilesets/mn_gym_3_metatiles.bin",
    "data/tilesets/mn_gym_3_palette_map.json",
    "gfx/tilesets/mn_gym_3.2bpp",
    "gfx/tilesets/mn_gym_3.png",
];

struct Inputs {
    base: CompiledGamePack,
    donor: CompiledGamePack,
    result: CompiledGamePack,
}

fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-limestone-review.crystalpack"),
        )
        .expect("completed northern story/maps/cabin input");
        let donor = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-gyms-minneapolis.crystalpack"),
        )
        .expect("captured canonical switch-pair gym");
        let result = replace_installed_minneapolis_gym(&base, &donor).expect("checked gym import");
        Inputs {
            base,
            donor,
            result,
        }
    })
}

fn labels(module: &MapModule) -> BTreeSet<String> {
    module
        .scripts
        .keys()
        .chain(module.script_text_bodies.keys())
        .cloned()
        .chain([
            module
                .attributes
                .map_scripts_label
                .clone()
                .expect("script section"),
            module
                .attributes
                .map_events_label
                .clone()
                .expect("event section"),
        ])
        .collect()
}

#[test]
#[ignore = "requires completed external northern story and reviewed Gym3 packs"]
fn gym3_import_preserves_the_entire_surrounding_story_maps_audio_and_native_numbers() {
    let i = inputs();
    let before = i.base.data();
    let after = i.result.data();
    assert_eq!(before.maps.len(), after.maps.len());
    assert!(after.maps.contains_key("MnRyansHouse"));
    let old = &before.maps[MAP];
    let new = &after.maps[MAP];
    assert_eq!([new.attributes.width, new.attributes.height], [5, 9]);
    for (name, module) in &before.maps {
        if name != MAP {
            assert_eq!(after.maps.get(name), Some(module), "unrelated map: {name}");
        }
    }
    assert!(
        new.events
            .warps
            .iter()
            .all(|w| w.target_map_constant == "MN_GYM_REVIEW_HALL" && w.target_warp_id == 9)
    );
    assert_eq!([new.events.warps[2].x, new.events.warps[2].y], [4, 15]);
    let mut restored = after.clone();
    restored.maps.insert(MAP.into(), old.clone());
    restored
        .map_attributes
        .insert(MAP.into(), before.map_attributes[MAP].clone());
    restored
        .map_dimensions
        .insert(CONSTANT.into(), before.map_dimensions[CONSTANT].clone());
    restored.runtime_map_metadata.insert(
        CONSTANT.into(),
        before.runtime_map_metadata[CONSTANT].clone(),
    );
    let block_label = old.attributes.blocks_label.as_ref().expect("blocks label");
    restored
        .map_blocks
        .insert(block_label.clone(), before.map_blocks[block_label].clone());
    restored.npcs.insert(MAP.into(), before.npcs[MAP].clone());
    let owned = labels(old)
        .union(&labels(new))
        .cloned()
        .collect::<BTreeSet<_>>();
    for label in owned {
        restored.map_scripts.remove(&label);
        if let Some(value) = before.map_scripts.get(&label) {
            restored.map_scripts.insert(label, value.clone());
        }
    }
    for name in TRAINERS {
        assert_eq!(
            after.trainers.trainers[name],
            i.donor.data().trainers.trainers[name]
        );
        let reviewed = &after.trainers.trainers[name];
        let numeric = after.story_event_script_constants.global[name];
        assert!(
            !after.trainers.trainers.values().any(|trainer| {
                trainer.trainer_id != name
                    && trainer.trainer_class == reviewed.trainer_class
                    && after.story_event_script_constants.global[&trainer.trainer_id] == numeric
            }),
            "reviewed trainer number collides: {name}"
        );
        restored.story_event_script_constants.global.insert(
            name.into(),
            before.story_event_script_constants.global[name],
        );
        restored
            .trainers
            .trainers
            .insert(name.into(), before.trainers.trainers[name].clone());
    }
    let globals = &after.story_event_script_constants.global;
    for (name, value) in &before.story_event_script_constants.global {
        if TRAINERS.contains(&name.as_str()) {
            continue;
        }
        assert_eq!(
            globals.get(name),
            Some(value),
            "existing symbol renumbered: {name}"
        );
    }
    let added = globals
        .keys()
        .filter(|k| !before.story_event_script_constants.global.contains_key(*k))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(added, NEW_EVENTS.into_iter().collect());
    let existing_max = before
        .story_event_script_constants
        .global
        .iter()
        .filter(|(k, _)| k.starts_with("EVENT_"))
        .map(|(_, n)| *n)
        .max()
        .expect("event table");
    assert_eq!(
        NEW_EVENTS
            .map(|name| globals[name])
            .into_iter()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([existing_max + 1, existing_max + 2, existing_max + 3])
    );
    for name in NEW_EVENTS {
        restored.story_event_script_constants.global.remove(name);
    }
    restored
        .tilesets
        .insert("mn_gym_3".into(), before.tilesets["mn_gym_3"].clone());
    assert!(restored == *before, "unrelated dataset field changed");
    let mut art = i.result.runtime_files().clone();
    for path in FILES {
        assert_eq!(art[path], i.donor.runtime_files()[path]);
        art.insert(path.into(), i.base.runtime_files()[path].clone());
    }
    assert_eq!(
        art,
        *i.base.runtime_files(),
        "unrelated runtime art changed"
    );
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.result.audio_manifest().expect("new audio"),
        i.base.audio_manifest().expect("old audio")
    );
    let mut report = i.result.report().clone();
    assert_eq!(
        report.manifests.pop().as_deref(),
        Some("minnesota-reviewed-gym3-replacement-v1")
    );
    assert_eq!(report, *i.base.report());
}

#[test]
#[ignore = "requires completed external northern story and reviewed Gym3 packs"]
fn gym3_import_rejects_duplicate_wrong_donor_and_unsupported_original_generation() {
    let i = inputs();
    assert!(replace_installed_minneapolis_gym(&i.result, &i.donor).is_err());
    assert!(replace_installed_minneapolis_gym(&i.base, &i.base).is_err());
    assert!(replace_installed_minneapolis_gym(&i.donor, &i.donor).is_err());
}
