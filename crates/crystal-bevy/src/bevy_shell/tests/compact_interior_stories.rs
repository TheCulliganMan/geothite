//! Explicit local entry fixtures; conversations, door pairs and Continue use input.
use crystal_core::input::GameButton;
use crystal_core::state::EventFlagMemory;
use serde_json::Value;

use super::interior_life::{fixture, load, resident, talk, target_name, walk};
use super::r10_gallery::{continue_save, press};
use super::{VisibleShellController, mark_runtime_snapshot_dirty};

fn point(value: &Value) -> (i16, i16) {
    (
        value[0].as_i64().expect("x") as i16,
        value[1].as_i64().expect("y") as i16,
    )
}

fn active_flags(flags: &EventFlagMemory) -> EventFlagMemory {
    // Native map entry may materialize an unset flag as false (for example,
    // STATUSFLAGS_FLASH). Missing and false have identical gameplay meaning.
    let mut active = flags.clone();
    active.event_flags.retain(|_, value| *value);
    active.engine_flags.retain(|_, value| *value);
    active
}

fn enter(
    c: &mut VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
    name: &str,
    door: (i16, i16),
) {
    if name == "MnLeagueHallOfFame" {
        // The ordinary static room path finder cannot see the door opened by
        // the native League callback. Cross it with actual directional input.
        for _ in 0..8 {
            press(c, GameButton::Up);
            if c.snapshot().expect("opened League door").overworld.map_name == name {
                return;
            }
        }
    } else {
        walk(c, r, door);
    }
    assert_eq!(
        c.snapshot().expect("real entrance").overworld.map_name,
        name
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the compact room-story composition"]
fn compact_interior_stories_local_voices_real_doors_and_continue() {
    let (root, r, _) = load();
    let source: Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/interior-life/compact-story-supplement-v01.json"
    ))
    .expect("authored local voices");
    let mut heard = 0;
    for room in source["rooms"].as_array().expect("supplement rooms") {
        let Some(conversations) = room["conversations"].as_array() else {
            continue;
        };
        let name = room["map"].as_str().expect("room");
        let module = &r.data().maps[name];
        let exit = &module.events.warps[0];
        let parent = target_name(&r, &exit.target_map);
        let outside = &r.data().maps[&parent];
        let return_at = outside
            .events
            .warps
            .iter()
            .find(|w| i16::try_from(w.index).ok() == Some(exit.target_warp_id))
            .expect("real exterior return");
        let door = outside
            .events
            .warps
            .iter()
            .find(|w| {
                Some(w.target_map_constant.as_str()) == module.attributes.map_constant.as_deref()
            })
            .expect("real building entrance");
        for conversation in conversations {
            let mut c = fixture(&root, &r, &parent, (return_at.x as i16, return_at.y as i16));
            if name == "MnLeagueHallOfFame" {
                // This conversation fixture starts after all five League wins.
                // It does not establish earned access to the Hall of Fame.
                let state = c.shell.shell.session_mut().state_mut();
                state
                    .flags
                    .set_event_flag("EVENT_MN_LEAGUE_RUN_ACTIVE", true)
                    .expect("explicit completed-run fixture");
                state
                    .script_runtime
                    .memory
                    .insert("wMnLeagueRun".into(), "5".into());
                mark_runtime_snapshot_dirty(&mut c.shell);
                c = continue_save(root.clone(), r.clone(), &mut c);
            }
            enter(&mut c, &r, name, (door.x as i16, door.y as i16));
            assert_eq!(
                c.snapshot().expect("native entrance").overworld.map_name,
                name
            );
            let before = active_flags(&c.shell.shell.session().state().flags);
            let money = c.shell.shell.session().state().money;
            let bag = c.shell.shell.session().state().bag.clone();
            let approach = point(&conversation["actor"]["approach"]);
            let script = conversation["script"].as_str().expect("resident");
            for _ in 0..2 {
                walk(&mut c, &r, approach);
                let words = talk(&mut c, resident(&r, name, script));
                assert!(
                    words.contains(script),
                    "authored conversation: {name}/{script}"
                );
            }
            press(&mut c, GameButton::Start);
            assert!(c.shell.start_menu_cursor.is_some());
            press(&mut c, GameButton::B);
            c = continue_save(root.clone(), r.clone(), &mut c);
            assert_eq!(
                active_flags(&c.shell.shell.session().state().flags),
                before,
                "ordinary conversation and Continue changed progression: {name}/{script}"
            );
            for warp in module.events.warps.iter().take(2) {
                walk(&mut c, &r, (warp.x as i16, warp.y as i16));
                assert_eq!(
                    c.snapshot()
                        .expect("actual carpet return")
                        .overworld
                        .map_name,
                    parent
                );
                enter(&mut c, &r, name, (door.x as i16, door.y as i16));
                assert_eq!(
                    c.snapshot().expect("actual reentry").overworld.map_name,
                    name
                );
            }
            assert_eq!(
                active_flags(&c.shell.shell.session().state().flags),
                before,
                "ordinary conversation and door returns changed progression: {name}/{script}"
            );
            assert_eq!(c.shell.shell.session().state().money, money);
            assert_eq!(c.shell.shell.session().state().bag, bag);
            heard += 1;
        }
    }
    assert_eq!(
        heard, 5,
        "all new Ely voices and preserved Split Rock family/work stories"
    );
}

#[test]
#[ignore = "requires the compact readable-detail pack in CRYSTAL_INTERIOR_PACK"]
fn compact_interior_stories_bookshelf_read_survives_continue_without_progression() {
    let (root, r, _) = load();
    let name = "MnBrainerdOutfitter";
    let mut c = fixture(&root, &r, name, (6, 5));
    let before = active_flags(&c.shell.shell.session().state().flags);
    let bag = c.shell.shell.session().state().bag.clone();
    let money = c.shell.shell.session().state().money;
    for continued in [false, true] {
        if continued {
            c = continue_save(root.clone(), r.clone(), &mut c);
        }
        walk(&mut c, &r, (3, 2));
        for _ in 0..2 {
            assert!(talk(&mut c, (3, 1)).contains("MN_BRAINERD_CAMP_GUIDE"));
        }
        assert_eq!(active_flags(&c.shell.shell.session().state().flags), before);
        assert_eq!(c.shell.shell.session().state().bag, bag);
        assert_eq!(c.shell.shell.session().state().money, money);
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the compact room-story composition"]
fn compact_interior_stories_fog_memory_never_activates_league_or_camp_flags() {
    let (root, r, _) = load();
    let room = "MnDuluthLookoutLodge2F";
    let script = "MN_DULUTH_X_LOOKOUTLODGE_F2_HOST";
    let layout: Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/interior-life/club-layouts.json"
    ))
    .expect("native room approaches");
    let contract = layout["rooms"]
        .as_array()
        .expect("layouts")
        .iter()
        .find(|v| v["map"] == room)
        .expect("lookout room contract");
    let approach = contract["actor_approaches"]
        .as_array()
        .expect("actors")
        .iter()
        .find(|v| v["script"] == script)
        .expect("host approach");
    let mut c = fixture(&root, &r, room, point(&approach["tile"]));
    for flag in [
        "EVENT_MN_INTERIOR_FOG_LOOKOUT",
        "EVENT_MN_LEAGUE_RUN_ACTIVE",
        "EVENT_MN_LEAGUE_CHAMPION",
        "EVENT_MN_GUNFLINT_CAMP_SUPPLY",
    ] {
        assert!(
            !c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("event")
        );
    }
    talk(&mut c, resident(&r, room, script));
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_INTERIOR_FOG_LOOKOUT")
            .expect("remembered conversation")
    );
    for flag in [
        "EVENT_MN_LEAGUE_RUN_ACTIVE",
        "EVENT_MN_LEAGUE_CHAMPION",
        "EVENT_MN_GUNFLINT_CAMP_SUPPLY",
    ] {
        assert!(
            !c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("independent event"),
            "unearned flag: {flag}"
        );
    }
}
