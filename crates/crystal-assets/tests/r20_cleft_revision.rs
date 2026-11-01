//! Actual two-mouth cave revision and native lateral one-way geometry.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_authored_cave_terrain,
};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use serde_json::{Value, json};

const MAP: &str = "MnR20Cave1F";
struct Inputs {
    base: CompiledGamePack,
    revised: CompiledGamePack,
    old: String,
    new: String,
}

fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let path = std::env::var_os("CRYSTAL_R20_CLEFT_REVISION_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.join("content-packs/minnesota-northstar-cave-refuges-review.crystalpack")
            });
        let base = read_verified_compiled_game_pack(path.canonicalize().expect("baseline path"))
            .expect("completed R13 baseline");
        let source = root.join("planning/minnesota/renders/caves/r20-candidate/v02");
        let old = std::fs::read_to_string(source.join("MnR20Cave1F.previous.json"))
            .expect("frozen whole preimage");
        let new = std::fs::read_to_string(source.join("MnR20Cave1F.candidate.json"))
            .expect("frozen reviewed candidate");
        let revised = revise_authored_cave_terrain(&base, &old, &new)
            .expect("native leftward cleft revision");
        Inputs {
            base,
            revised,
            old,
            new,
        }
    })
}

fn row_change(source: &str, at: [usize; 2], block: &str) -> String {
    let mut value: Value = serde_json::from_str(source).expect("room");
    let mut row = value["rows"][at[1]]
        .as_str()
        .expect("row")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    row[at[0]] = block.into();
    value["rows"][at[1]] = json!(row.join(" "));
    value.to_string()
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen R20 v02 sources"]
fn r20_cleft_revision_retains_all_other_data_and_original_two_mouths() {
    let i = inputs();
    let before = i.base.data();
    let after = i.revised.data();
    let original = &before.maps[MAP];
    let room = &after.maps[MAP];
    let mut normalized = after.clone();
    normalized.maps.get_mut(MAP).expect("room").blocks = original.blocks.clone();
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .expect("owned blocks");
    normalized
        .map_blocks
        .insert(label.clone(), before.map_blocks[label].clone());
    assert_eq!(
        &normalized, before,
        "all other data, indices and scripts exact"
    );
    assert_eq!(i.revised.report(), i.base.report());
    assert_eq!(i.revised.runtime_files(), i.base.runtime_files());
    assert_eq!(i.revised.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.revised.audio_manifest().expect("after audio"),
        i.base.audio_manifest().expect("before audio")
    );
    assert_eq!([room.attributes.width, room.attributes.height], [9, 8]);
    assert_eq!(room.attributes.time_of_day.as_deref(), Some("nite"));
    assert_eq!(room.events, original.events);
    assert_eq!(room.objects.len(), 1);
    assert_eq!([room.objects[0].x, room.objects[0].y], [5, 5]);
    assert_eq!(room.objects[0].event_flag, "EVENT_MN_R20_ITEM_01");
    for (warp, at, target) in [
        (&room.events.warps[0], [5, 13], "MnR20C00R00"),
        (&room.events.warps[1], [13, 13], "MnR20C01R00"),
    ] {
        assert_eq!([warp.x, warp.y], at);
        assert_eq!(warp.target_map, target);
        assert_eq!(warp.target_warp_id, 2);
    }
    // Preserve the existing inactive eastern arrival record's own target1.
    // It is ordinary Floor, not a third active interior mouth.
    let east = &after.maps["MnR20C01R00"].events.warps;
    assert_eq!([east[1].x, east[1].y], [12, 32]);
    assert_eq!(east[1].target_warp_id, 1);
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen R20 v02 sources"]
fn r20_cleft_revision_native_left_hops_both_lanes_and_reverse_walls() {
    let i = inputs();
    let room = &i.revised.data().maps[MAP];
    let map = OverworldMapData::from_attributes(MAP, &room.attributes, room.blocks.clone());
    let collision = i
        .revised
        .data()
        .tileset_collision("cave")
        .expect("native collision");
    for y in [8, 9] {
        let mut player = PlayerMovementState::new(TilePosition::new(9, y));
        assert!(matches!(attempt_ledge_jump_with_occupied_tiles(
            &mut player, Direction::Left, &map, &collision, StepOptions::default(), &[]
        ), LedgeJumpOutcome::Jumped { to, .. } if to == TilePosition::new(7, y)));
        player.facing = Direction::Right;
        assert!(matches!(
            attempt_step_with_occupied_tiles(
                &mut player,
                Direction::Right,
                &map,
                &collision,
                StepOptions::default(),
                &[]
            ),
            StepOutcome::Blocked { .. }
        ));
    }
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen native R10 refinement sources"]
fn cave_revision_accepts_native_floor_lips_but_rejects_open_course_shoulders() {
    let i = inputs();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = root.join("planning/minnesota/renders/caves/r10-refinement/be41b5fb81f3ef85");
    let old = std::fs::read_to_string(source.join("MnR10Cave1F.previous.json"))
        .expect("installed R10 same-extent preimage");
    let new = std::fs::read_to_string(source.join("MnR10Cave1F.candidate.json"))
        .expect("frozen R10 refinement");
    let result = revise_authored_cave_terrain(&i.base, &old, &new)
        .expect("native Floor beside a downward lip is legitimate");
    let room = &result.data().maps["MnR10Cave1F"];
    let map = OverworldMapData::from_attributes(&room.id, &room.attributes, room.blocks.clone());
    let collision = result
        .data()
        .tileset_collision("cave")
        .expect("native collision");
    let mut player = PlayerMovementState::new(TilePosition::new(12, 6));
    player.facing = Direction::Left;
    assert!(matches!(attempt_step_with_occupied_tiles(
        &mut player, Direction::Left, &map, &collision, StepOptions::default(), &[]
    ), StepOutcome::Moved { to, .. } if to == TilePosition::new(11, 6)));
    player.facing = Direction::Down;
    assert!(matches!(
        attempt_step_with_occupied_tiles(
            &mut player,
            Direction::Down,
            &map,
            &collision,
            StepOptions::default(),
            &[]
        ),
        StepOutcome::Blocked { .. }
    ));
    assert!(
        revise_authored_cave_terrain(&i.base, &old, &row_change(&new, [5, 3], "02")).is_err(),
        "opening Floor alongside the Wall course creates a real side bypass"
    );
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen authored RockCut sources"]
fn cave_revision_certifies_native_rockcut_attributes_and_existing_healer() {
    let i = inputs();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/renders/caves/r05-rockcut-candidate/v02");
    let old = std::fs::read_to_string(source.join("MnR05RockCut.previous.json"))
        .expect("original whole RockCut");
    let new = std::fs::read_to_string(source.join("MnR05RockCut.candidate.json"))
        .expect("frozen RockCut revision");
    let result = revise_authored_cave_terrain(&i.base, &old, &new)
        .expect("actual cave attributes certify the existing RockCut room");
    let name = "MnR05RockCut";
    let before = &i.base.data().maps[name];
    let after = &result.data().maps[name];
    assert_eq!(after.attributes.tileset_name, "cave");
    assert_eq!(after.attributes.environment.as_deref(), Some("CAVE"));
    assert_eq!(after.events.warps.len(), 2);
    assert_eq!(after.objects, before.objects);
    assert_eq!(
        after.script_runtime_commands,
        before.script_runtime_commands
    );
    let mut normalized = result.data().clone();
    normalized.maps.insert(name.into(), before.clone());
    let label = before
        .attributes
        .blocks_label
        .as_ref()
        .expect("owned blocks");
    normalized
        .map_blocks
        .insert(label.clone(), i.base.data().map_blocks[label].clone());
    assert_eq!(
        &normalized,
        i.base.data(),
        "entire other data and healing service retained"
    );
    let mut false_old: Value = serde_json::from_str(&old).expect("old");
    let mut false_new: Value = serde_json::from_str(&new).expect("new");
    for value in [&mut false_old, &mut false_new] {
        value["residents"][0]["service"] = json!(null);
    }
    assert!(
        revise_authored_cave_terrain(&i.base, &false_old.to_string(), &false_new.to_string())
            .is_err(),
        "matching false source must not remove the real native healing service"
    );
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen R20 v02 sources"]
fn r20_cleft_revision_rejects_open_landing_shoulder_return_and_unpaired_mouths() {
    let i = inputs();
    for (at, block, reason) in [
        ([3, 4], "01", "blocked lateral landing"),
        ([4, 3], "02", "open upper shoulder"),
        ([4, 5], "02", "open lower shoulder"),
        ([4, 6], "01", "severed ordinary return slot"),
        ([2, 3], "01", "severed supply stair"),
        ([5, 5], "01", "severed return stair"),
        ([2, 6], "21", "removed original west mouth"),
        ([1, 1], "1b", "unpaired ladder"),
        ([0, 0], "02", "outer Floor"),
        ([0, 0], "0f", "outer Water"),
    ] {
        assert!(
            revise_authored_cave_terrain(&i.base, &i.old, &row_change(&i.new, at, block)).is_err(),
            "accepted {reason}"
        );
    }
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen R20 v02 sources"]
fn r20_cleft_revision_certifies_native_right_hop_geometry_without_publishing_it() {
    let i = inputs();
    // The native opposite-facing metatile exchanges its Wall and Hop columns.
    // The same ordinary return is valid in this bounded geometry. This variant
    // exercises directional validation, and is never the published candidate.
    let opposite = row_change(&i.new, [4, 4], "3c");
    let result = revise_authored_cave_terrain(&i.base, &i.old, &opposite)
        .expect("valid opposite native geometry");
    let room = &result.data().maps[MAP];
    let map = OverworldMapData::from_attributes(MAP, &room.attributes, room.blocks.clone());
    let collision = result
        .data()
        .tileset_collision("cave")
        .expect("native collision");
    for y in [8, 9] {
        let mut player = PlayerMovementState::new(TilePosition::new(8, y));
        assert!(matches!(attempt_ledge_jump_with_occupied_tiles(
            &mut player, Direction::Right, &map, &collision, StepOptions::default(), &[]
        ), LedgeJumpOutcome::Jumped { to, .. } if to == TilePosition::new(10, y)));
        player.facing = Direction::Left;
        assert!(matches!(
            attempt_step_with_occupied_tiles(
                &mut player,
                Direction::Left,
                &map,
                &collision,
                StepOptions::default(),
                &[]
            ),
            StepOutcome::Blocked { .. }
        ));
    }
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen R20 v02 sources"]
fn r20_cleft_revision_rejects_original_reward_anchor_and_warp_drift() {
    let i = inputs();
    for mutation in 0..7 {
        let mut value: Value = serde_json::from_str(&i.new).expect("candidate");
        match mutation {
            0 => value["items"][0]["item"] = json!("MASTER_BALL"),
            1 => value["items"][0]["quantity"] = json!(99),
            2 => value["items"][0]["flag"] = json!("EVENT_MN_GYM_6_DEFEATED"),
            3 => value["items"][0]["tile"] = json!([6, 5]),
            4 => value["exits"][1] = json!([12, 13]),
            5 => value["exit_doors"][1] = json!([76, 32]),
            _ => value["clear_arrival"] = json!([4, 12]),
        }
        assert!(
            revise_authored_cave_terrain(&i.base, &i.old, &value.to_string()).is_err(),
            "accepted source drift{mutation}"
        );
    }
}

#[test]
#[ignore = "requires completed external R13 baseline and frozen R20 v02 sources"]
fn r20_cleft_revision_uncached_assembly_and_actual_written_roundtrip() {
    let i = inputs();
    let mut original_split = i.base.data().clone();
    original_split.maps.remove(MAP);
    let mut expected = original_split
        .assemble_map_module_from_compiled_payloads(MAP)
        .expect("uncached preimage");
    expected.blocks = i.revised.data().maps[MAP].blocks.clone();
    let mut split = i.revised.data().clone();
    split.maps.remove(MAP);
    assert_eq!(
        split
            .assemble_map_module_from_compiled_payloads(MAP)
            .expect("uncached result"),
        expected
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r20-cleft-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    i.revised
        .write_preserving_storage(&path)
        .expect("write result");
    assert_eq!(
        read_verified_compiled_game_pack(&path).expect("strict written readback"),
        i.revised
    );
    std::fs::remove_file(path).expect("remove only scratch test pack");
    assert!(
        revise_authored_cave_terrain(&i.revised, &i.old, &i.new).is_err(),
        "stale revision rejected"
    );
}
