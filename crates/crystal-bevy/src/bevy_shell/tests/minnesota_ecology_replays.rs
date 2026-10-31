//! Local field positions, party, time and completed trainer fights are declared.
//! Every later encounter, capture, bag change and Continue uses native input.
#[path = "minnesota_progression_replays.rs"]
mod progression;
#[path = "minnesota_ecology_water.rs"]
mod water;

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{checkpoint, encounter_and_capture, fixture_at_hour, verify_capture};
use crate::bevy_shell::tests::r10_gallery::continue_save;
use crate::bevy_shell::tests::winona::press;
use crate::bevy_shell::{VisibleShellController, mark_runtime_snapshot_dirty};

fn source() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::env::var_os("CRYSTAL_ECOLOGY_SOURCE").expect("runtime ecology source"),
        )
        .expect("read authored ecology source"),
    )
    .expect("ecology contract")
}

fn tile(value: &Value) -> [i16; 2] {
    serde_json::from_value(value.clone()).expect("native tile pair")
}

fn hour(time: &str) -> u8 {
    match time {
        "morning" => 6,
        "day" => 12,
        "night" => 22,
        other => panic!("native encounter period required: {other}"),
    }
}

fn completed_local_trainers(c: &mut VisibleShellController, runtime: &CrystalRuntime, map: &str) {
    assert!(c.snapshot().expect("local field fixture").battle.is_none());
    let room = &runtime.data().maps[map];
    for request in room
        .trainer_scripts
        .values()
        .chain(room.scripted_trainer_battles.iter().map(|b| &b.request))
    {
        if request.event_flag.starts_with("EVENT_") {
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(&request.event_flag, true)
                .expect("declared previously completed local trainer");
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and CRYSTAL_ECOLOGY_SOURCE with the complete biome revision"]
fn ecology_all66_grass_maps_native_local_resident_captures_and_continue() {
    let plan = source();
    let sites = plan["grass"].as_array().expect("authored grass sites");
    assert_eq!(sites.len(), 66);
    for site in sites {
        let map = site["map"].as_str().expect("map");
        let a = tile(&site["sample"]);
        let b = tile(&site["loop"]);
        let target = site["replay"]["species"].as_str().expect("local resident");
        let time = site["replay"]["time"].as_str().expect("native period");
        let (root, runtime, mut c) = fixture_at_hour(map, a, false, false, hour(time));
        completed_local_trainers(&mut c, &runtime, map);
        // No state/party/progress/position/time/RNG writes after this setup.
        checkpoint(&mut c, &format!("ecology-{map}-{time}-start"));
        encounter_and_capture(&mut c, &runtime, target, a, b);
        verify_capture(&c, &runtime, target);
        c = continue_save(root, runtime.clone(), &mut c);
        verify_capture(&c, &runtime, target);
        checkpoint(&mut c, &format!("ecology-{map}-{time}-caught"));
        eprintln!("verified native grass {map} {time}: {target}");
    }
}

#[test]
#[ignore = "requires the complete ecology revision and its habitat review source"]
fn ecology_reviewed_specialist_habitats_native_captures_and_continue() {
    let plan = source();
    let progression: Option<Value> = std::env::var_os("CRYSTAL_PROGRESSION_SOURCE").map(|path| {
        serde_json::from_str(&std::fs::read_to_string(path).expect("progression source"))
            .expect("progression JSON")
    });
    let edits = plan["habitat_review"]["relocations"]
        .as_array()
        .expect("habitat review");
    assert_eq!(edits.len(), 28);
    for edit in edits {
        let map = edit["map"].as_str().expect("reviewed map");
        if edit["sample"].is_null() {
            assert!(
                edit["loop"].is_null(),
                "inert table has no native grass loop"
            );
            continue;
        }
        let a = tile(&edit["sample"]);
        let b = tile(&edit["loop"]);
        let target = edit["after"].as_str().expect("local resident");
        let group = progression
            .as_ref()
            .and_then(|p| p["groups"].as_array())
            .and_then(|groups| {
                groups.iter().find(|g| {
                    g["maps"]
                        .as_array()
                        .is_some_and(|maps| maps.iter().any(|m| m == map))
                })
            });
        let target = if edit["zone"].is_null() {
            group
                .and_then(|g| g["promotions"][target].as_str())
                .unwrap_or(target)
        } else {
            target
        };
        let time = edit["time"].as_str().expect("native time");
        let (root, runtime, mut c) = fixture_at_hour(map, a, false, false, hour(time));
        completed_local_trainers(&mut c, &runtime, map);
        // All subsequent state changes come from production controller input.
        encounter_and_capture(&mut c, &runtime, target, a, b);
        verify_capture(&c, &runtime, target);
        c = continue_save(root, runtime.clone(), &mut c);
        verify_capture(&c, &runtime, target);
        eprintln!(
            "verified reviewed habitat {map} {:?}: {target}",
            edit["zone"]
        );
    }
}

#[test]
#[ignore = "requires the ecology revision with the Rochester resting-house companion"]
fn ecology_clinic_companion_decline_capacity_claim_and_continue() {
    use crate::bevy_shell::tests::minnesota_fossils::talk;
    use crystal_core::models::Dv;
    let (root, runtime) = crate::bevy_shell::tests::winona::load();
    let flag = "EVENT_MN_PIPULSE_COMPANION_RECEIVED";
    for (delivered, count) in [(false, 1), (true, 6), (true, 1)] {
        let mut c = crate::bevy_shell::tests::interior_life::fixture(
            &root,
            &runtime,
            "MnRochesterPatientHouse",
            (3, 6),
        );
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag("EVENT_MN_DELIVERED_ROCHESTER_PARCEL", delivered)
            .expect("declared parcel");
        state
            .flags
            .set_engine_flag("ENGINE_POKEDEX", true)
            .expect("research access");
        for _ in 0..count {
            assert!(
                state.storage.party.add_pokemon(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("legal research partner")
                )
            );
        }
        state.sync_party_from_storage();
        let money = state.money;
        mark_runtime_snapshot_dirty(&mut c.shell);
        // Setup ends here. All offers, gifts and persistence use native input.
        if delivered {
            talk(&mut c, GameButton::Up, &[false]);
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(flag)
                    .expect("claim")
            );
        }
        talk(
            &mut c,
            GameButton::Up,
            if delivered { &[true] } else { &[] },
        );
        let accepted = delivered && count < 6;
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("claim"),
            accepted
        );
        if accepted {
            let s = c.snapshot().expect("clinic companion");
            assert_eq!(s.party.slots[count].pokemon.species.id, "PIPULSE");
            assert_eq!(s.party.slots[count].pokemon.level, 8);
            assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .pokedex
                    .has_caught("PIPULSE")
            );
        }
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        talk(
            &mut c,
            GameButton::Up,
            if delivered && !accepted { &[true] } else { &[] },
        );
        assert_eq!(
            c.snapshot()
                .expect("continued clinic companion")
                .party
                .slots
                .len(),
            count + usize::from(accepted)
        );
        assert_eq!(c.shell.shell.session().state().money, money);
    }
}

#[test]
#[ignore = "requires complete ecology revision with sealed Route 2 drainage"]
fn ecology_fenced_drainage_keeps_both_bridge_rails_closed_and_grass_active() {
    for (at, direction) in [([31, 23], GameButton::Up), ([31, 26], GameButton::Down)] {
        let (root, runtime, mut c) = fixture_at_hour("MnR02C00R00", at, true, true, 12);
        let wild = &runtime.data().wild_encounters["MnR02C00R00"];
        assert!(wild.water.is_none() && wild.water_rate.is_none());
        assert!(wild.grass.is_some());
        assert_eq!(
            runtime.data().maps["MnR02C00R00"]
                .attributes
                .fishing_group
                .as_deref(),
            Some("FISHGROUP_NONE")
        );
        // No state mutation after fixture setup; actual movement must be blocked.
        for _ in 0..8 {
            press(&mut c, direction);
            let s = c.snapshot().expect("closed bridge rail");
            assert_eq!([s.overworld.tile.x, s.overworld.tile.y], at);
            assert!(s.battle.is_none());
            assert!(
                !s.ui.text_window_open,
                "no false Surf offer from the bridge"
            );
        }
        c = continue_save(root, runtime, &mut c);
        let s = c.snapshot().expect("continued bridge");
        assert_eq!([s.overworld.tile.x, s.overworld.tile.y], at);
        checkpoint(&mut c, &format!("ecology-drainage-{}", at[1]));
    }
}
