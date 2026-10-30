//! Initial placement/resources are fixtures; all journeys and service actions use input.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, neighbors};
use super::interior_life::{fixture, load, talk, travel, walk};

const CITY: &str = "MnDetroitLakes";
const RANGER: &str = "MnDetroitLakesRangerCabin";
const DONE: &str = "EVENT_MN_LAKE_OBSERVATIONS_REWARDED";
const NOTES: [&str; 3] = [
    "EVENT_MN_LAKE_OBSERVATION_01",
    "EVENT_MN_LAKE_OBSERVATION_02",
    "EVENT_MN_LAKE_OBSERVATION_03",
];

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production Detroit input");
    c.wait_frames(16).expect("visible settling");
}
fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native event")
}
fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u32 {
    u32::from(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MAX_REPEL"]),
    )
}
fn flags(c: &mut VisibleShellController, badge: bool, missing: Option<usize>) {
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[5] = badge;
    state
        .flags
        .set_engine_flag("ENGINE_MINERALBADGE", badge)
        .expect("consistent initial badge fixture");
    for (i, flag) in NOTES.iter().enumerate() {
        state
            .flags
            .set_event_flag(flag, missing != Some(i))
            .expect("initial note fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
}
fn choose(
    c: &mut VisibleShellController,
    target: usize,
    current: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..40 {
        let selected = current(c);
        if selected == target {
            return;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native cursor failed to reach {target}");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_all_rooms_residents_carpets_and_two_real_roads() {
    let (root, r, _) = load();
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota");
    let plan: Value = serde_json::from_str(
        &std::fs::read_to_string(directory.join("towns/MnDetroitLakes.json")).expect("town source"),
    )
    .expect("town JSON");
    let station: Value = serde_json::from_str(
        &std::fs::read_to_string(directory.join("transit/MnDetroitLakesStation.json"))
            .expect("station source"),
    )
    .expect("station JSON");
    let mut c = fixture(&root, &r, CITY, (6, 18));
    let mut partner = r
        .data()
        .create_pokemon("MEW", 50, Dv::default())
        .expect("initial legal Surf fixture");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: r.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("initial Surf permission");
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Every later change in this town tour is native input.
    for actor in plan["residents"].as_array().expect("five town residents") {
        let target = point(&actor["tile"]);
        let approach = match actor["facing"].as_str().expect("native facing") {
            "west" => (target.0 - 1, target.1),
            "east" => (target.0 + 1, target.1),
            _ => panic!("unreviewed town resident facing"),
        };
        walk(&mut c, &r, approach);
        assert!(!talk(&mut c, target).is_empty());
    }
    for sign in plan["signs"].as_array().expect("public town signs") {
        let target = point(&sign["tile"]);
        walk(&mut c, &r, (target.0, target.1 + 1));
        assert!(!talk(&mut c, target).is_empty());
    }
    for door in plan["doors"].as_array().expect("five facades") {
        let name = door["target"].as_str().expect("room");
        let room = if name == "MnDetroitLakesStation" {
            &station
        } else {
            plan["interiors"]
                .as_array()
                .expect("rooms")
                .iter()
                .find(|room| room["id"] == name)
                .expect("room contract")
        };
        walk(&mut c, &r, point(&door["tile"]));
        assert_eq!(c.snapshot().expect("real door").overworld.map_name, name);
        for actor in room["residents"].as_array().expect("residents") {
            if actor["service"] == "pokemart" {
                continue;
            }
            walk(&mut c, &r, point(&actor["approach"]));
            let target = actor.get("counter_tile").unwrap_or(&actor["tile"]);
            let text = talk(&mut c, point(target));
            assert!(!text.is_empty(), "visible local story");
        }
        for exit in room["exits"].as_array().expect("both carpets") {
            walk(&mut c, &r, point(&exit["tile"]));
            let back = c.snapshot().expect("exact exterior return").overworld;
            assert_eq!(back.map_name, CITY);
            assert_eq!((back.tile.x, back.tile.y), point(&door["return_tile"]));
            walk(&mut c, &r, point(&door["tile"]));
            assert_eq!(
                c.snapshot().expect("passive reentry").overworld.map_name,
                name
            );
        }
        travel(&mut c, &r, CITY);
    }
    for (edge, button, name, wanted, back) in [
        (
            (47, 18),
            GameButton::Right,
            "MnR15C00R00",
            (0, 24),
            GameButton::Left,
        ),
        (
            (0, 18),
            GameButton::Left,
            "MnDetroitLakesApproach",
            (47, 18),
            GameButton::Right,
        ),
        (
            (47, 19),
            GameButton::Right,
            "MnR15C00R00",
            (0, 25),
            GameButton::Left,
        ),
        (
            (0, 19),
            GameButton::Left,
            "MnDetroitLakesApproach",
            (47, 19),
            GameButton::Right,
        ),
    ] {
        walk(&mut c, &r, edge);
        for _ in 0..8 {
            press(&mut c, button);
            if c.snapshot()
                .expect("settled outbound road")
                .overworld
                .map_name
                == name
            {
                break;
            }
        }
        let s = c.snapshot().expect("real neighboring road").overworld;
        assert_eq!(s.map_name, name);
        assert_eq!((s.tile.x, s.tile.y), wanted);
        for _ in 0..8 {
            press(&mut c, back);
            if c.snapshot()
                .expect("settled returning road")
                .overworld
                .map_name
                == CITY
            {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("reciprocal road").overworld.map_name,
            CITY
        );
    }
    walk(&mut c, &r, (13, 23));
    press(&mut c, GameButton::Down);
    for _ in 0..96 {
        let s = c.snapshot().expect("native beach Surf prompt/launch");
        if s.overworld.mode == crystal_core::world::movement::MovementMode::Surf
            && (s.overworld.tile.x, s.overworld.tile.y) == (13, 24)
            && !s.ui.text_window_open
            && c.shell.field_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    for (input, target) in [
        (GameButton::Right, (14, 24)),
        (GameButton::Left, (13, 24)),
        (GameButton::Up, (13, 23)),
    ] {
        for _ in 0..8 {
            press(&mut c, input);
            let s = c
                .snapshot()
                .expect("native public lake movement settles")
                .overworld;
            if (s.tile.x, s.tile.y) == target {
                break;
            }
        }
        let s = c.snapshot().expect("native public lake movement").overworld;
        assert_eq!(s.map_name, CITY);
        assert_eq!((s.tile.x, s.tile.y), target);
    }
    assert_eq!(
        c.snapshot().expect("real beach dismount").overworld.mode,
        crystal_core::world::movement::MovementMode::Normal
    );
    walk(&mut c, &r, (24, 2));
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
    }
    let boundary = c.snapshot().expect("visible woodland boundary").overworld;
    assert_eq!(boundary.map_name, CITY);
    assert_eq!((boundary.tile.x, boundary.tile.y), (24, 2));
    let before = c.snapshot().expect("before town Continue").overworld;
    c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
    let after = c.snapshot().expect("town Continue").overworld;
    assert_eq!(after.map_name, before.map_name);
    assert_eq!(after.tile, before.tile);
    assert_eq!(after.facing, before.facing);
    assert_eq!(after.mode, before.mode);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_native_care_pc_and_priced_shop() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, "MnDetroitLakesPokecenter1F", (8, 13));
    let mut partner = r
        .data()
        .create_pokemon("GILLPUP", 20, Dv::default())
        .expect("legal care partner");
    partner.hp = 10;
    partner.status = Some("POISON".into());
    for m in &mut partner.moves {
        m.current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = 0;
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, &r, (7, 5));
    talk(&mut c, (7, 4));
    let healed = c.snapshot().expect("real free care");
    let partner = &healed.party.slots[0].pokemon;
    assert_eq!(partner.hp, partner.max_hp);
    assert!(partner.status.is_none());
    assert!(
        partner
            .moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    assert_eq!(c.shell.shell.session().state().money, 0);
    walk(&mut c, &r, (13, 4));
    press(&mut c, GameButton::Up);
    for _ in 0..96 {
        press(&mut c, GameButton::A);
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
    }
    assert!(c.shell.pc_hub_cursor.is_some(), "native PC hub");
    let at = c.snapshot().expect("PC").overworld.tile;
    press(&mut c, GameButton::Down);
    assert_eq!(c.snapshot().expect("PC owns input").overworld.tile, at);
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_none() && !c.has_pending_script_work() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    c = super::r10_gallery::continue_save(root.clone(), r.clone(), &mut c);
    travel(&mut c, &r, CITY);
    // A fresh local poison scenario inherits the actually earned clinic
    // registration. The healthy reached checkpoint is never rewritten.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = super::authored_route_fixture::reached_save_path(&format!(
        "detroit-earned-clinic-poison-scenario-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.save(&path)
        .expect("reached native Detroit clinic checkpoint");
    c = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("fresh declared poison scenario");
    super::authored_route_fixture::cleanup_reached_save(&path);
    let last_battle_result = c.shell.shell.session().state().battle_result;
    // Initial HP/status and money are fixtures; all later recovery uses input.
    let state = c.shell.shell.session_mut().state_mut();
    let partner = state.storage.party.pokemon[0].as_mut().expect("partner");
    partner.hp = 1;
    partner.status = Some("POISON".into());
    state.sync_party_from_storage();
    state.money = 101;
    mark_runtime_snapshot_dirty(&mut c.shell);
    for _ in 0..96 {
        if c.shell.field_notice.is_some()
            || !c.shell.battle_messages.is_empty()
            || c.shell.visible_blackout_phase.is_some()
            || c.snapshot()
                .expect("faint text ownership")
                .ui
                .text_window_open
        {
            press(&mut c, GameButton::A);
        } else {
            // Crystal turns on the first changed direction. Repeating it is
            // necessary to take a real step rather than alternating turns.
            press(&mut c, GameButton::Left);
            press(&mut c, GameButton::Left);
            press(&mut c, GameButton::Right);
            press(&mut c, GameButton::Right);
        }
        if c.snapshot().expect("poison recovery").overworld.map_name == "MnDetroitLakesPokecenter1F"
        {
            break;
        }
    }
    let recovery = c.snapshot().expect("registered native blackout");
    let state = c.shell.shell.session().state();
    assert_eq!(
        recovery.overworld.map_name,
        "MnDetroitLakesPokecenter1F",
        "tile={:?}, party={:?}, steps={:?}, last_spawn={:?}, whiteout={:?}, pending_poison={}, notice={:?}, messages={:?}, error={:?}, audio={:?}",
        recovery.overworld.tile,
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .map(|p| (p.hp, p.status.as_deref()))
            .collect::<Vec<_>>(),
        state.step_events,
        state.last_spawn_map_constant,
        c.shell.visible_blackout_phase,
        c.shell.pending_poison_blackout,
        c.shell.field_notice,
        c.shell.battle_messages,
        c.shell.last_error,
        c.shell
            .last_audio_events
            .iter()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
    );
    let recovered = c.snapshot().expect("blackout healed partner");
    assert_eq!(
        recovered.party.slots[0].pokemon.hp,
        recovered.party.slots[0].pokemon.max_hp
    );
    assert_eq!(
        c.shell.shell.session().state().battle_result,
        last_battle_result,
        "overworld recovery must not fabricate or consume a battle result"
    );
    assert_eq!(
        c.shell.shell.session().state().money,
        50,
        "native whiteout cost"
    );
    let mut shop = fixture(&root, &r, "MnDetroitLakesMart", (3, 6));
    shop.shell.shell.session_mut().state_mut().money = 10_000;
    mark_runtime_snapshot_dirty(&mut shop.shell);
    walk(&mut shop, &r, (3, 3));
    press(&mut shop, GameButton::Left);
    press(&mut shop, GameButton::A);
    for _ in 0..64 {
        if shop.shell.shop_top_cursor.is_some()
            && shop.shell.shop_welcome_seen
            && shop.shell.shop_notice.is_none()
        {
            break;
        }
        press(&mut shop, GameButton::A);
    }
    assert_eq!(
        shop.snapshot()
            .expect("native stock")
            .pending_shop
            .expect("shop")
            .inventory,
        r.data().marts.0["MART_MN_DETROIT_LAKES"]
    );
    let item = &r.data().items["GREAT_BALL"];
    let carried = shop.shell.shell.session().state().bag.quantity(item);
    press(&mut shop, GameButton::A);
    press(&mut shop, GameButton::A);
    press(&mut shop, GameButton::A);
    assert_eq!(shop.shell.shell.session().state().money, 10_000);
    press(&mut shop, GameButton::A);
    assert_eq!(
        shop.shell.shell.session().state().money,
        10_000 - u32::from(item.price)
    );
    assert_eq!(
        shop.shell.shell.session().state().bag.quantity(item),
        carried + 1
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_step_notices_preserve_input_and_do_not_repeat() {
    let (root, r, _) = load();
    let mut empty = fixture(&root, &r, CITY, (6, 18));
    let before = empty.shell.shell.session().state().clone();
    empty
        .shell
        .shell
        .resolve_poison_blackout_to_last_spawn()
        .expect_err("empty party is not a poison blackout");
    assert_eq!(
        empty.shell.shell.session().state(),
        &before,
        "denial is atomic"
    );
    for hp in [2, 1] {
        let mut c = fixture(&root, &r, CITY, (6, 18));
        let mut partner = r
            .data()
            .create_pokemon("CHIKORITA", 20, Dv::default())
            .expect("legal poisoned partner");
        partner.hp = hp;
        partner.status = Some(r.data().step_event_rules.poison_status.clone());
        let healthy = r
            .data()
            .create_pokemon("TOTODILE", 20, Dv::default())
            .expect("healthy second partner prevents blackout");
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        assert!(state.storage.party.add_pokemon(healthy));
        state.step_events.poison_step_count = r.data().step_event_rules.poison_step_interval - 1;
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut c.shell);
        let before = c.shell.shell.session().state().clone();
        c.shell
            .shell
            .resolve_poison_blackout_to_last_spawn()
            .expect_err("healthy partner prohibits poison recovery");
        assert_eq!(c.shell.shell.session().state(), &before, "denial is atomic");
        walk(&mut c, &r, (7, 18));
        assert_eq!(
            c.snapshot().expect("actual poison step").party.slots[0]
                .pokemon
                .hp,
            hp - 1
        );
        assert_eq!(c.shell.poison_flash_frames_remaining, 0);
        assert!(!c.shell.pending_poison_blackout);
        let count = |c: &VisibleShellController| {
            c.shell
                .last_audio_events
                .iter()
                .filter(|line| line.starts_with("poison step damaged="))
                .count()
        };
        assert_eq!(count(&c), 1);
        c.wait_frames(32).expect("idle does not repeat poison");
        assert_eq!(count(&c), 1);
        if hp == 1 {
            assert!(
                c.shell
                    .field_notice
                    .as_deref()
                    .is_some_and(|text| text.contains("fainted"))
            );
            let before = c.snapshot().expect("faint input owner").overworld.tile;
            press(&mut c, GameButton::Right);
            assert_eq!(
                c.snapshot().expect("text blocks movement").overworld.tile,
                before
            );
            for _ in 0..64 {
                if c.shell.field_notice.is_none() {
                    break;
                }
                press(&mut c, GameButton::A);
            }
            assert!(c.shell.field_notice.is_none());
        }
        walk(&mut c, &r, (8, 18));
        assert_eq!(
            c.snapshot()
                .expect("healthy party stays in town")
                .overworld
                .map_name,
            CITY
        );
    }

    let mut c = fixture(&root, &r, CITY, (6, 18));
    let state = c.shell.shell.session_mut().state_mut();
    state.repel_steps_remaining = 1;
    state.active_repel_item = Some("MAX_REPEL".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, &r, (7, 18));
    assert_eq!(
        c.shell.field_notice.as_deref(),
        Some("REPEL's effect wore off.")
    );
    let before = c.snapshot().expect("Repel input owner").overworld.tile;
    press(&mut c, GameButton::Right);
    assert_eq!(
        c.snapshot().expect("notice blocks walking").overworld.tile,
        before
    );
    for _ in 0..64 {
        if c.shell.field_notice.is_none() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.field_notice.is_none());
    walk(&mut c, &r, (8, 18));
    c.wait_frames(32).expect("idle after expiry");
    assert_eq!(
        c.shell
            .last_audio_events
            .iter()
            .filter(|line| line.as_str() == "MAX_REPEL wore off")
            .count(),
        1
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_walk_hatches_egg_and_returns_input() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CITY, (6, 18));
    let mut egg = r
        .data()
        .create_pokemon("TOGEPI", 5, Dv::default())
        .expect("legal egg");
    egg.is_egg = true;
    egg.nickname = "EGG".into();
    egg.hp = 0;
    egg.happiness = 1;
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(egg));
    state.step_events.step_count = r.data().step_event_rules.egg_step_trigger.wrapping_sub(1);
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, &r, (7, 18));
    assert_eq!(c.shell.field_notice.as_deref(), Some("Huh?"));
    assert!(c.shell.visible_egg_hatch.is_some());
    let before = c.snapshot().expect("hatch input owner").overworld.tile;
    press(&mut c, GameButton::Right);
    assert_eq!(
        c.snapshot().expect("hatch blocks walking").overworld.tile,
        before
    );
    for _ in 0..96 {
        if c.shell.pending_egg_hatch_nickname.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.visible_egg_hatch.is_none());
    assert!(
        c.shell.pending_egg_hatch_nickname.is_some(),
        "native hatch animation reaches nickname choice"
    );
    for _ in 0..64 {
        if c.shell.pending_egg_hatch_nickname.is_none() {
            break;
        }
        // B advances any remaining native prompt pages before declining.
        press(&mut c, GameButton::B);
    }
    assert!(c.shell.pending_egg_hatch_nickname.is_none());
    let hatched = &c.snapshot().expect("hatched partner").party.slots[0].pokemon;
    assert!(!hatched.is_egg);
    assert_eq!(hatched.nickname, "TOGEPI");
    walk(&mut c, &r, (8, 18));
    press(&mut c, GameButton::Start);
    assert!(
        c.shell.start_menu_cursor.is_some(),
        "native Start after hatch"
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_ranger_denies_each_missing_requirement() {
    let (root, r, _) = load();
    for (badge, missing) in [
        (false, None),
        (true, Some(0)),
        (true, Some(1)),
        (true, Some(2)),
    ] {
        let mut c = fixture(&root, &r, RANGER, (6, 4));
        flags(&mut c, badge, missing);
        let before = quantity(&c, &r);
        let text = talk(&mut c, (6, 3));
        assert!(text.contains(if badge { "Missing" } else { "Badge" }));
        assert_eq!(quantity(&c, &r), before);
        assert!(!event(&c, DONE));
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_observations_persist_on_the_real_itasca_return() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, "MnR19C00R01", (24, 10));
    c.shell.shell.session_mut().state_mut().badges.johto[5] = true;
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_engine_flag("ENGINE_MINERALBADGE", true)
        .expect("consistent initial survey badge");
    mark_runtime_snapshot_dirty(&mut c.shell);
    let grid = RouteGrid::new_with_grass(
        &r,
        RouteSpec {
            prefix: "MnR19",
            columns: 1,
            rows: 2,
            tileset: "johto",
            start: (24, 58),
            doors: BTreeMap::new(),
        },
    );
    for (i, target) in [(54, 14), (40, 34), (22, 44)].into_iter().enumerate() {
        let approach = neighbors(target)
            .into_iter()
            .map(|(p, _)| p)
            .find(|p| {
                grid.floor.contains(p) && !grid.blocked.contains(p) && !grid.hazards.contains(p)
            })
            .expect("public sign approach");
        grid.walk(&mut c, approach);
        talk(&mut c, target);
        assert!(event(&c, NOTES[i]));
        assert!(!event(&c, DONE));
    }
    let edge = grid
        .floor
        .iter()
        .copied()
        .find(|p| p.0 == 63 && p.1 < 48 && !grid.blocked.contains(p) && !grid.hazards.contains(p))
        .expect("original east road to Itasca");
    grid.walk(&mut c, edge);
    for _ in 0..8 {
        press(&mut c, GameButton::Right);
        if c.snapshot().expect("real survey return").overworld.map_name == "MnLakeItasca" {
            break;
        }
    }
    assert_eq!(
        c.snapshot()
            .expect("original Itasca road")
            .overworld
            .map_name,
        "MnLakeItasca"
    );
    c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
    for flag in NOTES {
        assert!(event(&c, flag));
    }
    assert!(
        !event(&c, DONE),
        "gathering notes is separate from the ranger hand-in"
    );
}

pub(super) fn toss_potion(c: &mut VisibleShellController) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("Start");
    let target = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("Pack");
    choose(c, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(c, GameButton::A);
    let target = c
        .snapshot()
        .expect("carried items")
        .bag
        .items
        .iter()
        .filter(|i| i.quantity > 0)
        .position(|i| i.item_id == "POTION")
        .expect("fixture Potion");
    choose(c, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("item cursor")
            .option_index
    });
    press(c, GameButton::A);
    let s = c.snapshot().expect("actions");
    let target = visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
        .expect("actions")
        .iter()
        .position(|a| *a == FieldPackAction::Toss)
        .expect("Toss");
    choose(c, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    press(c, GameButton::A);
    assert_eq!(c.shell.pack_toss.as_ref().expect("quantity").quantity, 1);
    press(c, GameButton::A);
    c.wait_frames(240).expect("native confirmation printer");
    press(c, GameButton::A);
    for _ in 0..96 {
        let snapshot = c.snapshot().expect("native Pack closure");
        if snapshot.ui.menu.is_none()
            && !snapshot.ui.text_window_open
            && c.shell.field_notice.is_none()
            && c.shell.bag_cursor.is_none()
            && !c.has_pending_script_work()
        {
            return;
        }
        c.wait_frames(240).expect("native toss notice printer");
        press(
            c,
            if c.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
                GameButton::A
            } else {
                GameButton::B
            },
        );
    }
    panic!("Toss did not release walking input");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Detroit Lakes"]
fn detroit_lakes_controller_full_pocket_continue_native_toss_and_reward_retry() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, RANGER, (6, 4));
    flags(&mut c, true, None);
    c.shell.shell.session_mut().state_mut().bag.items.clear();
    c.shell
        .shell
        .add_bag_item("POTION", 1)
        .expect("one tossable fixture item");
    while c.shell.shell.session().state().bag.items.len()
        < crystal_core::models::bag::ITEM_POCKET_CAPACITY
    {
        c.shell
            .shell
            .add_bag_item("BERRY", 99)
            .expect("full physical pocket fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    let text = talk(&mut c, (6, 3));
    assert!(text.to_lowercase().contains("full"));
    assert!(!event(&c, DONE));
    assert_eq!(quantity(&c, &r), 0);
    c = super::r10_gallery::continue_save(root.clone(), r.clone(), &mut c);
    assert!(!event(&c, DONE));
    toss_potion(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["POTION"]),
        0
    );
    talk(&mut c, (6, 3));
    assert_eq!(quantity(&c, &r), 1);
    assert!(event(&c, DONE));
    c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
    talk(&mut c, (6, 3));
    assert_eq!(quantity(&c, &r), 1);
}
