//! Actual native banks and rod catalogs, without ordinary combat substitutes.

use std::collections::BTreeSet;

use crystal_core::random::ReplayDivider;
use crystal_core::state::BattleMemory;
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari::{self, SafariEncounterSource, SafariPhase};
use crystal_core::world::encounters::TimeOfDay;
use crystal_core::world::fishing::{FISHING_RODS, resolve_fishing_slot};
use crystal_core::world::map::{Direction, TilePosition};

use super::{configured, ranger_fixture};
use crate::safari_zone::collision;

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_native_rods_keep_real_bank_water_tables_and_capture_ownership() {
    let data = configured();
    let zone = data.safari_zone.as_ref().expect("config");
    let (name, bank, facing, water) = zone
        .maps
        .keys()
        .find_map(|name| {
            let map = &data.maps[name];
            (0..map.attributes.height as i16 * 2)
                .flat_map(|y| {
                    (0..map.attributes.width as i16 * 2).map(move |x| TilePosition::new(x, y))
                })
                .find_map(|bank| {
                    if !collision(&data, name, bank).is_ok_and(|c| c == "FLOOR") {
                        return None;
                    }
                    [
                        (Direction::Up, 0, -1),
                        (Direction::Down, 0, 1),
                        (Direction::Left, -1, 0),
                        (Direction::Right, 1, 0),
                    ]
                    .into_iter()
                    .find_map(|(facing, dx, dy)| {
                        let water = TilePosition::new(bank.x + dx, bank.y + dy);
                        (collision(&data, name, water).is_ok_and(|c| c == "WATER")
                            && zone
                                .habitat_at(&data, name, water)
                                .is_ok_and(|h| h.is_some()))
                        .then(|| (name.clone(), bank, facing, water))
                    })
                })
        })
        .expect("native floor bank adjoining real preserve water");
    for rod in FISHING_RODS {
        for time in [TimeOfDay::Day, TimeOfDay::Night] {
            let (mut state, mut map) = ranger_fixture(&data);
            data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
                .expect("real permit");
            // Explicit bank/time fixture; all following wildlife comes from the
            // ordinary native rod roll, held-item/DV draws and saved source.
            data.transition_overworld_session(
                &mut state,
                &mut map,
                &name,
                bank,
                SpawnMemoryUpdate::Preserve,
                &BTreeSet::new(),
            )
            .expect("native shore fixture");
            map.player.facing = facing;
            data.commit_overworld_snapshot(&mut state, &map, SpawnMemoryUpdate::Preserve);
            state.time.time_of_day = time;
            let storage = state.storage.clone();
            let steps = safari::session(&state)
                .expect("valid")
                .expect("permit")
                .steps_remaining;
            let mut divider =
                ReplayDivider::new((0u32..16384).map(|n| ((n * 73 + 19) % 256) as u8));
            let cast = (0..128)
                .find_map(|_| {
                    let cast = data
                        .cast_fishing_rod_in_session_with_divider(
                            &mut state,
                            &map,
                            rod,
                            &mut divider,
                        )
                        .expect("native rod transaction");
                    assert!(cast.wild_battle.is_none());
                    cast.safari_encounter.is_some().then_some(cast)
                })
                .expect("native fishing bite");
            assert_eq!(cast.checked_water_target, water);
            let wild = cast.safari_encounter.expect("actual native fish");
            assert_eq!(wild.origin_tile, bank);
            assert_eq!(wild.origin_map_name, name);
            let SafariEncounterSource::Fishing {
                water_tile,
                rod: saved_rod,
                group,
                time_of_day,
                slot_roll,
            } = &wild.source
            else {
                panic!("saved rod source");
            };
            assert_eq!(*water_tile, water);
            assert_eq!(saved_rod, rod);
            assert_eq!(*time_of_day, time);
            let expected = resolve_fishing_slot(&data.fishing, group, rod, time, *slot_roll)
                .expect("same native table");
            assert_eq!(wild.pokemon.species.id, expected.species);
            assert_eq!(wild.pokemon.level, expected.level);
            assert!(
                wild.pokemon.species.id != "QWILFISH",
                "Minnesota table must not become its Johto namesake"
            );
            assert_eq!(state.storage, storage);
            assert_eq!(state.battle, BattleMemory::Inactive);
            let active = safari::session(&state).expect("valid").expect("permit");
            assert_eq!(
                active.steps_remaining, steps,
                "casting is not a movement commit"
            );
            assert_eq!(active.balls_remaining, 30);
            assert_eq!(active.phase, SafariPhase::Encounter(wild.clone()));
            data.validate_saved_safari_references(&state)
                .expect("real bank source");
            let before = state.clone();
            assert!(
                data.cast_fishing_rod_in_session_with_divider(&mut state, &map, rod, &mut divider)
                    .is_err()
            );
            assert_eq!(state, before, "capture owner prevents a second cast");
            let mut bad = active;
            let SafariPhase::Encounter(fish) = &mut bad.phase else {
                panic!("fish");
            };
            let SafariEncounterSource::Fishing { group, .. } = &mut fish.source else {
                panic!("source");
            };
            *group = "FISHGROUP_NONE".into();
            safari::set_session(&mut state, &bad).expect("structural bad-source fixture");
            assert!(
                data.validate_saved_safari_references(&state).is_err(),
                "another group's table cannot validate this bank"
            );
        }
    }
}
