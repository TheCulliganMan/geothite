//! Map-event identities are independent of Crystal's twelve live NPC slots.

use super::{ObjectOverrideSaveError, OverworldObjectMapMemory, validate_saved_object_overrides};
use crate::world::map::TilePosition;
use crate::world::session::{OverworldObjectStructMemory, OverworldObjectStructRosterMemory};

fn memory(index: u8) -> OverworldObjectMapMemory {
    OverworldObjectMapMemory {
        object_structs: OverworldObjectStructRosterMemory {
            structs: vec![OverworldObjectStructMemory {
                slot: 1,
                map_object_index: index,
                live_tile: TilePosition::new(10, 10),
                last_tile: None,
                initial_tile: TilePosition::new(10, 10),
                facing: None,
                step_duration: None,
                last_tile_occupied_remaining_frames: 0,
                pending_random_wait: false,
                initialized_fixed_spin: false,
                strength_push_direction: None,
                strength_moving: false,
                fixed_facing: false,
                sliding: false,
                visible: true,
                normal_following: false,
                following_not_exact_leader_slot: None,
            }],
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn authored_map_event_identities_above_fifteen_round_trip_in_real_npc_slots() {
    for index in [16, 18, 254, 255] {
        let original = memory(index);
        original
            .validate_saved_state("AuthoredCity")
            .expect("nonzero byte-sized map-event identity");
        let restored: OverworldObjectMapMemory = serde_json::from_value(
            serde_json::to_value(&original).expect("saved current-map object image"),
        )
        .expect("restore an NPC from a later authored event");
        assert_eq!(restored, original);
        validate_saved_object_overrides(
            "AuthoredCity",
            &restored,
            |_| Some((128, 96)),
            |_| Some(255),
            |_| true,
        )
        .expect("compiled map contains the referenced event");
    }
}

#[test]
fn authored_event_indices_keep_slot_duplicate_zero_and_compiled_range_guards() {
    assert!(
        memory(0)
            .validate_saved_state("AuthoredCity")
            .expect_err("zero is not a map event")
            .contains("nonzero")
    );
    let mut outside_slot = memory(18);
    outside_slot.object_structs.structs[0].slot = 13;
    assert!(
        outside_slot
            .validate_saved_state("AuthoredCity")
            .expect_err("live NPC slots remain limited to twelve")
            .contains("1..=12")
    );
    let mut duplicate = memory(18);
    let mut second = duplicate.object_structs.structs[0].clone();
    second.slot = 2;
    duplicate.object_structs.structs.push(second);
    assert!(
        duplicate
            .validate_saved_state("AuthoredCity")
            .expect_err("two slots cannot own the same map event")
            .contains("duplicates map-object index")
    );
    assert!(matches!(
        validate_saved_object_overrides(
            "AuthoredCity",
            &memory(18),
            |_| Some((128, 96)),
            |_| Some(17),
            |_| true
        ),
        Err(ObjectOverrideSaveError::MissingStructMapObject {
            map_object_index: 18,
            object_count: 17,
            ..
        })
    ));
}
