//! Whole-batch preservation and unsafe border/preimage rejection on local packs.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_authored_cave_terrain,
    revise_r09_mine_terrain,
};
use serde_json::Value;

const ROOMS: [&str; 9] = [
    "MnR01Cave1F",
    "MnR03Cave1F",
    "MnR05RockCut",
    "MnR09Cave1F",
    "MnR10Cave1F",
    "MnR12Cave1F",
    "MnR13Cave1F",
    "MnR17Cave1F",
    "MnR20Cave1F",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root without parent traversal")
}

fn source(room: &str, previous: bool) -> String {
    let suffix = if previous { ".previous.json" } else { ".json" };
    std::fs::read_to_string(root().join(format!(
        "planning/minnesota/caves/johto-quality-v01/{room}{suffix}"
    )))
    .expect("explicit authored cave source")
}

fn base() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        read_verified_compiled_game_pack(
            root().join("content-packs/minnesota-cohesion-r01-upper-loop-v01.crystalpack"),
        )
        .expect("completed local cohesion parent")
    })
}

fn revise(
    base: &CompiledGamePack,
    room: &str,
    old: &str,
    new: &str,
) -> anyhow::Result<CompiledGamePack> {
    if room == "MnR09Cave1F" {
        let story = std::fs::read_to_string(root().join("planning/minnesota/story.json"))?;
        revise_r09_mine_terrain(base, old, new, &story)
    } else {
        revise_authored_cave_terrain(base, old, new)
    }
}

#[test]
#[ignore = "requires the completed local cohesion parent and nine-cave publication"]
fn cave_quality_preserves_every_unowned_value_and_round_trips_all_nine_rooms() {
    let mut revised = base().clone();
    for room in ROOMS {
        revised = revise(&revised, room, &source(room, true), &source(room, false))
            .unwrap_or_else(|error| panic!("publish {room}: {error:#}"));
    }
    let written = read_verified_compiled_game_pack(
        root().join("content-packs/minnesota-cave-quality-v01-09.crystalpack"),
    )
    .expect("strict written final pack");
    assert_eq!(
        revised.identity().expect("identity"),
        written.identity().expect("written identity")
    );
    assert_eq!(revised.data(), written.data());
    assert_eq!(base().runtime_files(), revised.runtime_files());
    assert_eq!(base().compiled_audio(), revised.compiled_audio());
    let mut restored = revised.data().clone();
    for room in ROOMS {
        let original = &base().data().maps[room];
        let changed = &revised.data().maps[room];
        assert_ne!(original.blocks, changed.blocks, "each cave is revised");
        assert_eq!(changed.attributes.border_block, 0x2e);
        let mut expected = original.clone();
        expected.blocks = changed.blocks.clone();
        expected.attributes.border_block = 0x2e;
        assert_eq!(
            &expected, changed,
            "only blocks and border may change in {room}"
        );
        restored.maps.insert(room.into(), original.clone());
        restored
            .map_attributes
            .insert(room.into(), original.attributes.clone());
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .expect("owned blocks");
        restored
            .map_blocks
            .insert(label.clone(), base().data().map_blocks[label].clone());
        let mut uncached = revised.data().clone();
        uncached.maps.remove(room);
        let assembled = uncached
            .assemble_map_module_from_compiled_payloads(room)
            .expect("uncached map loads with revised border and terrain");
        assert_eq!(assembled.attributes, changed.attributes);
        assert_eq!(assembled.blocks, changed.blocks);
        assert_eq!(assembled.objects, changed.objects);
    }
    assert_eq!(
        &restored,
        base().data(),
        "all other maps, scripts, events, catalogs and recovery data remain exact"
    );
}

#[test]
#[ignore = "requires the completed local cohesion parent"]
fn cave_quality_rejects_walkable_water_partial_and_stale_border_recipes() {
    for room in ROOMS {
        let old = source(room, true);
        let new = source(room, false);
        for border in [0x02, 0x0f, 0x0a, 0x07] {
            let mut unsafe_source: Value = serde_json::from_str(&new).expect("source JSON");
            unsafe_source["border_block"] = border.into();
            assert!(
                revise(base(), room, &old, &unsafe_source.to_string()).is_err(),
                "{room} must reject unsafe border {border:02x}"
            );
        }
        let mut stale: Value = serde_json::from_str(&old).expect("source JSON");
        stale["border_block"] = 0x2e.into();
        assert!(
            revise(base(), room, &stale.to_string(), &new).is_err(),
            "{room} must reject a stale border preimage"
        );
    }
}

#[test]
#[ignore = "requires the completed local cohesion parent"]
fn cave_quality_rejects_unpaired_ladders_and_changed_reward_contracts() {
    let room = "MnR03Cave1F";
    let old = source(room, true);
    let mut new: Value = serde_json::from_str(&source(room, false)).expect("source JSON");
    new["items"][0]["quantity"] = 99.into();
    assert!(revise(base(), room, &old, &new.to_string()).is_err());
    let mut new: Value = serde_json::from_str(&source(room, false)).expect("source JSON");
    let mut row = new["rows"][1]
        .as_str()
        .expect("explicit row")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    row[4] = "0b".into();
    new["rows"][1] = row.join(" ").into();
    assert!(revise(base(), room, &old, &new.to_string()).is_err());
}
