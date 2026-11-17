//! Pure checks for the optional five-cave extension to gym save migration.

use crystal_assets::GameDataSet;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use serde_json::{Value, json};

use super::gym_tests;
use super::{Compatibility, parse_compatibility, upgrade_checkpoint};
use crate::cave_save_migration::CAVES;
use crate::gym_save_migration::GYMS;

const TARGET: &str = "core+northstar-gym-overhaul-v1+northstar-cave-overhaul-v1";
const ARRIVALS: [TilePosition; 5] = [
    TilePosition::new(9, 10),
    TilePosition::new(4, 13),
    TilePosition::new(17, 22),
    TilePosition::new(5, 18),
    TilePosition::new(5, 18),
];

fn metadata_value() -> Value {
    let mut value = gym_tests::metadata_value();
    value["target_modpack_ids"] = json!([TARGET, format!("{TARGET}+hosted")]);
    value["cave_reentry"] = json!({
        "policy": "minnesota-five-caves-v1",
        "maps": CAVES.iter().zip(ARRIVALS).map(|(map, arrival)| json!({
            "map_id": map, "arrival": arrival, "facing": "down",
        })).collect::<Vec<_>>()
    });
    value
}

fn parse(value: &Value) -> anyhow::Result<Compatibility> {
    parse_compatibility(&serde_json::to_vec(value)?, TARGET)
}

fn metadata() -> Compatibility {
    parse(&metadata_value()).expect("combined metadata")
}

fn data() -> GameDataSet {
    let mut data = gym_tests::data();
    for map_id in CAVES {
        let mut room = gym_tests::map(map_id);
        room.attributes.width = 12;
        room.attributes.height = 13;
        room.blocks = vec![0; 12 * 13];
        data.maps.insert(map_id.into(), room);
    }
    data
}

fn source_state(map_id: &str) -> GameState {
    let mut state = gym_tests::source_state(map_id);
    for cave in CAVES.into_iter().chain(["MnUnchangedCave"]) {
        state
            .map_block_overrides
            .insert(cave.into(), [((2, 3), 7)].into());
        let memory = state.map_object_overrides["MnWinona"].clone();
        state.map_object_overrides.insert(cave.into(), memory);
    }
    for receipt in ["EVENT_CAVE_ITEM_RECEIVED", "EVENT_CAVE_TRAINER_DEFEATED"] {
        state
            .flags
            .set_event_flag(receipt, true)
            .expect("cave receipt");
    }
    state
}

fn clear_expected_snapshots(state: &mut GameState) {
    for map_id in GYMS.into_iter().chain(CAVES) {
        state.map_block_overrides.remove(map_id);
        state.map_object_overrides.remove(map_id);
    }
}

#[test]
fn combined_upgrade_relocates_all_thirteen_rooms_and_preserves_other_state() {
    let metadata = metadata();
    let data = data();
    metadata
        .gym_reentry
        .as_ref()
        .expect("gym policy")
        .validate_arrivals(&data)
        .expect("gym arrivals");
    metadata
        .cave_reentry
        .as_ref()
        .expect("cave policy")
        .validate_arrivals(&data)
        .expect("dry cave arrivals");
    let positions = GYMS
        .into_iter()
        .enumerate()
        .map(|(index, name)| (name, TilePosition::new(1 + (index % 3) as i16, 5)))
        .chain(CAVES.into_iter().zip(ARRIVALS));
    for (map_id, arrival) in positions {
        let source = source_state(map_id);
        let original = source.clone();
        let upgraded =
            upgrade_checkpoint(source.clone(), &metadata).expect("idle combined upgrade");
        let mut expected = source.clone();
        clear_expected_snapshots(&mut expected);
        expected.overworld = OverworldMemory::Active {
            map_name: map_id.into(),
            tile: arrival,
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        assert_eq!(upgraded, expected, "{map_id}");
        assert_eq!(source, original, "source remains untouched for {map_id}");
    }
}

#[test]
fn combined_upgrade_preserves_unlisted_cave_position_mode_and_snapshots() {
    let mut source = source_state("MnUnchangedCave");
    if let OverworldMemory::Active { mode, .. } = &mut source.overworld {
        *mode = MovementMode::Surf;
    }
    let mut expected = source.clone();
    clear_expected_snapshots(&mut expected);
    assert_eq!(
        upgrade_checkpoint(source, &metadata()).expect("unlisted cave"),
        expected
    );
}

#[test]
fn gym_only_and_refinement_metadata_leave_cave_state_unchanged() {
    let source = source_state(CAVES[0]);
    let gym_only_value = gym_tests::metadata_value();
    let gym_only = parse_compatibility(
        &serde_json::to_vec(&gym_only_value).expect("gym metadata"),
        "core+northstar-gym-overhaul-v1",
    )
    .expect("schema2 without caves");
    let mut expected = source.clone();
    for gym in GYMS {
        expected.map_block_overrides.remove(gym);
        expected.map_object_overrides.remove(gym);
    }
    assert_eq!(
        upgrade_checkpoint(source.clone(), &gym_only).expect("gym-only upgrade"),
        expected
    );

    let mut refinement = gym_only_value;
    refinement["schema_version"] = json!(1);
    refinement["target_manifest"] = json!("northstar-refinement-v1");
    refinement["target_modpack_ids"] = json!(["core+northstar-refinement-v1"]);
    refinement
        .as_object_mut()
        .expect("metadata")
        .remove("gym_reentry");
    let old = parse_compatibility(
        &serde_json::to_vec(&refinement).expect("refinement metadata"),
        "core+northstar-refinement-v1",
    )
    .expect("schema1 unchanged");
    assert_eq!(
        upgrade_checkpoint(source.clone(), &old).expect("refinement upgrade"),
        source
    );
    refinement["cave_reentry"] = metadata_value()["cave_reentry"].clone();
    assert!(
        parse_compatibility(
            &serde_json::to_vec(&refinement).expect("invalid schema1"),
            "core+northstar-refinement-v1"
        )
        .is_err()
    );
}

#[test]
fn cave_metadata_requires_both_exact_markers_and_the_complete_allowlist() {
    let valid = metadata_value();
    let mut missing_hosted_marker = valid.clone();
    missing_hosted_marker["target_modpack_ids"][1] = json!("core+northstar-gym-overhaul-v1+hosted");
    assert!(parse(&missing_hosted_marker).is_err());
    let mut missing_raw_marker = valid.clone();
    missing_raw_marker["target_modpack_ids"][0] = json!("core+northstar-gym-overhaul-v1");
    assert!(
        parse_compatibility(
            &serde_json::to_vec(&missing_raw_marker).expect("missing raw marker"),
            "core+northstar-gym-overhaul-v1"
        )
        .is_err()
    );
    let mut wrong_component = valid.clone();
    let wrong_target = TARGET.replace("northstar-cave-overhaul-v1", "northstar-cave-overhaul-v10");
    wrong_component["target_modpack_ids"] = json!([wrong_target, format!("{wrong_target}+hosted")]);
    assert!(
        parse_compatibility(
            &serde_json::to_vec(&wrong_component).expect("wrong marker"),
            &wrong_target
        )
        .is_err()
    );
    for (pointer, replacement) in [
        ("/cave_reentry/policy", json!("all-caves")),
        ("/cave_reentry/maps/1/map_id", json!(CAVES[0])),
        ("/cave_reentry/maps/0/map_id", json!("MnUnchangedCave")),
        ("/cave_reentry/maps/0/map_id", json!(GYMS[0])),
        ("/cave_reentry/maps/0/arrival/y", json!(-1)),
        ("/cave_reentry/maps/0/facing", json!("north")),
    ] {
        let mut invalid = valid.clone();
        *invalid.pointer_mut(pointer).expect("metadata path") = replacement;
        assert!(parse(&invalid).is_err(), "accepted {invalid}");
    }
    let mut incomplete = valid;
    incomplete["cave_reentry"]["maps"]
        .as_array_mut()
        .expect("caves")
        .pop();
    assert!(parse(&incomplete).is_err());
}

#[test]
fn cave_arrivals_reject_ice_water_walls_actors_and_out_of_bounds() {
    let metadata = metadata();
    let policy = metadata.cave_reentry.as_ref().expect("cave policy");
    policy
        .validate_arrivals(&data())
        .expect("ordinary dry floor");
    for collision in ["ICE", "WATER", "WALL"] {
        let mut invalid = data();
        *invalid
            .tilesets
            .get_mut("gym-fixture")
            .expect("tileset")
            .collision
            .get_mut("0")
            .expect("base floor block") = vec![collision.into(); 4];
        assert!(
            policy.validate_arrivals(&invalid).is_err(),
            "accepted {collision}"
        );
    }
    let mut occupied = data();
    let mut actor = gym_tests::actor();
    actor.x = ARRIVALS[0].x as u16;
    actor.y = ARRIVALS[0].y as u16;
    occupied
        .maps
        .get_mut(CAVES[0])
        .expect("cave")
        .objects
        .push(actor);
    assert!(policy.validate_arrivals(&occupied).is_err());
    let mut outside = metadata_value();
    outside["cave_reentry"]["maps"][0]["arrival"]["x"] = json!(24);
    assert!(
        parse(&outside)
            .expect("bounded coordinate token")
            .cave_reentry
            .expect("caves")
            .validate_arrivals(&data())
            .is_err()
    );
}

#[test]
fn combined_upgrade_rejects_cave_fences_without_mutating_source() {
    let metadata = metadata();
    for map_id in CAVES {
        for fence in ["surf", "hidden", "fixed-facing", "scene"] {
            let mut source = source_state(map_id);
            match fence {
                "surf" => {
                    if let OverworldMemory::Active { mode, .. } = &mut source.overworld {
                        *mode = MovementMode::Surf;
                    }
                }
                "hidden" => {
                    source
                        .map_object_overrides
                        .get_mut(map_id)
                        .expect("cave snapshot")
                        .player_hidden = true
                }
                "fixed-facing" => {
                    source
                        .map_object_overrides
                        .get_mut(map_id)
                        .expect("cave snapshot")
                        .object_structs
                        .player_fixed_facing = true
                }
                "scene" => source.script_runtime.all_input_locked = true,
                _ => unreachable!("finite fence fixture list"),
            }
            let original = source.clone();
            assert!(
                upgrade_checkpoint(source.clone(), &metadata).is_err(),
                "accepted {map_id}/{fence}"
            );
            assert_eq!(source, original);
        }
    }
}
