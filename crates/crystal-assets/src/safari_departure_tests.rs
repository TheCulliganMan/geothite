//! Native court exit and field-transition settlement on the real preserve.

use std::collections::BTreeSet;

use crystal_core::input::GameButton;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari;
use crystal_core::world::map::{Direction, TilePosition};

use super::{configured, native_press, ranger_fixture};

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_native_departure_preserves_destination_and_closes_gate_on_return() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
        .expect("real admission");
    let zone = data.safari_zone.as_ref().expect("config");
    let storage = state.storage.clone();
    let money = state.money;
    // Explicit local starting position; crossing itself uses native movement.
    data.transition_overworld_session(
        &mut state,
        &mut map,
        &zone.gate_court.map_name,
        TilePosition::new(14, 15),
        SpawnMemoryUpdate::Preserve,
        &BTreeSet::new(),
    )
    .expect("court exit fixture");
    map.player.facing = Direction::Down;
    data.commit_overworld_snapshot(&mut state, &map, SpawnMemoryUpdate::Preserve);
    native_press(&data, &mut state, &mut map, GameButton::Down);
    assert_eq!(map.map.name, zone.gate_court.approach_map);
    assert_eq!(map.player.tile, TilePosition::new(16, 0));
    assert!(safari::session(&state).expect("valid").is_none());
    assert_eq!(state.storage, storage);
    assert_eq!(state.money, money);
    data.validate_saved_safari_references(&state)
        .expect("no live permit outside");
    native_press(&data, &mut state, &mut map, GameButton::Up);
    native_press(&data, &mut state, &mut map, GameButton::Up);
    assert_eq!(map.map.name, zone.gate_court.map_name);
    assert!(
        map.visible_object_at_checked(TilePosition::new(12, 5))
            .expect("guard")
            .is_some()
    );

    let (mut state, mut map) = ranger_fixture(&data);
    let before = (state.clone(), map.snapshot());
    assert!(
        data.transition_overworld_session(
            &mut state,
            &mut map,
            "MnBoundarySafariC00R00",
            TilePosition::new(12, 12),
            SpawnMemoryUpdate::Preserve,
            &BTreeSet::new()
        )
        .is_err(),
        "alternate native arrival cannot manufacture a permit"
    );
    assert_eq!(state, before.0);
    assert_eq!(map.snapshot(), before.1);
    data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
        .expect("real admission");
    // The same common native transition primitive underlies scripted/field
    // travel. It must retain the requested destination, not fabricate a return.
    data.transition_overworld_session(
        &mut state,
        &mut map,
        &zone.admission_map,
        zone.return_to.tile,
        SpawnMemoryUpdate::Preserve,
        &BTreeSet::new(),
    )
    .expect("native outward transition");
    assert_eq!(map.map.name, zone.admission_map);
    assert_eq!(map.player.tile, zone.return_to.tile);
    assert!(safari::session(&state).expect("valid").is_none());
    data.validate_saved_safari_references(&state)
        .expect("settled field transition");
}
