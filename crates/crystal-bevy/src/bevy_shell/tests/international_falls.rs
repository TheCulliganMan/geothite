//! Explicit resources/defeated-trainer fixtures; all journeys and quests use input.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, travel, walk};
use super::r10_gallery::{continue_save, press};

#[path = "international_falls_journey.rs"]
mod journey;

const CITY: &str = "MnInternationalFalls";
const CLINIC: &str = "MnInternationalFallsPokecenter1F";
const REPORT: &str = "EVENT_MN_BORDER_REPORT";
const ARRIVED: &str = "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL";
const GIVEN: &str = "EVENT_MN_BORDER_CIRCUIT_REWARDED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn flag(c: &VisibleShellController, n: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(n)
        .expect("native event")
}
fn qualify(c: &mut VisibleShellController) {
    let state = c.shell.shell.session_mut().state_mut();
    for (i, name) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state.badges.johto[i] = true;
        state
            .flags
            .set_engine_flag(name, true)
            .expect("badge fixture");
    }
    state
        .flags
        .set_event_flag(SIGNAL, true)
        .expect("signal fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
}
fn cross(c: &mut VisibleShellController, b: GameButton, map: &str, at: (i16, i16)) {
    for _ in 0..8 {
        press(c, b);
        if c.snapshot().expect("seam").overworld.map_name == map {
            break;
        }
    }
    let s = c.snapshot().expect("native arrival").overworld;
    assert_eq!(s.map_name, map);
    assert_eq!((s.tile.x, s.tile.y), at);
}

fn drain(c: &mut VisibleShellController) -> String {
    let mut pages = String::new();
    for _ in 0..240 {
        let snapshot = c.snapshot().expect("native visible script");
        if let Some(text) = snapshot.ui.text {
            pages.push_str(&text.asm_text.unwrap_or_default());
            pages.push('\n');
        }
        if !snapshot.ui.text_window_open && !c.has_pending_script_work() {
            return pages;
        }
        press(c, GameButton::A);
    }
    panic!("native script did not release input: {pages}");
}

fn register(c: &mut VisibleShellController, yes: bool) {
    press(c, GameButton::Up);
    press(c, GameButton::A);
    for _ in 0..64 {
        let snapshot = c
            .shell
            .shell
            .snapshot()
            .expect("authoritative ranger offer");
        if snapshot.ui.pending_yes_no.is_some()
            && super::super::visible_field_dialogue_is_entirely_consumed(&c.shell, &snapshot)
        {
            let target = usize::from(!yes);
            for _ in 0..4 {
                let index = c
                    .shell
                    .yes_no_cursor
                    .as_ref()
                    .expect("visible native choice")
                    .option_index;
                if index == target {
                    press(c, GameButton::A);
                    drain(c);
                    return;
                }
                press(
                    c,
                    if index < target {
                        GameButton::Down
                    } else {
                        GameButton::Up
                    },
                );
            }
            panic!("ranger choice did not reach requested native answer");
        }
        press(c, GameButton::A);
    }
    panic!("ranger offer did not finish its visible pages");
}

/// Plan ordinary snow/grass steps only; the production controller owns each move.
fn dry_walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let map = c
        .snapshot()
        .expect("current border route")
        .overworld
        .map_name;
    journey::walk(c, r, &map, to);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with International Falls"]
fn international_falls_controller_decline_each_departure_guard_and_free_inbound_care() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CLINIC, (2, 6));
    assert!(talk(&mut c, (2, 5)).contains("seventh"));
    assert!(!flag(&c, REPORT));
    qualify(&mut c);
    register(&mut c, false);
    assert!(!flag(&c, REPORT));
    register(&mut c, true);
    assert!(flag(&c, REPORT));
    assert!(!flag(&c, ARRIVED));
    assert!(talk(&mut c, (2, 5)).contains("far camp"));
    assert!(!flag(&c, GIVEN));
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(flag(&c, REPORT));
    let omissions = BADGE_ENGINE_FLAGS[..8]
        .iter()
        .copied()
        .map(Some)
        .chain([None, None]);
    for (missing, badge) in omissions.enumerate() {
        let mut gate = fixture(&root, &r, "MnRainyRiverIcefields", (60, 45));
        qualify(&mut gate);
        let state = gate.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag(REPORT, true)
            .expect("check-in fixture");
        if let Some(badge) = badge {
            state.badges.johto[missing] = false;
            state
                .flags
                .set_engine_flag(badge, false)
                .expect("one missing badge");
        } else {
            state
                .flags
                .set_event_flag(if missing == 8 { REPORT } else { SIGNAL }, false)
                .expect("one missing prerequisite");
        }
        mark_runtime_snapshot_dirty(&mut gate.shell);
        for _ in 0..4 {
            press(&mut gate, GameButton::Right);
            if gate.snapshot().expect("gate text").ui.text_window_open {
                break;
            }
        }
        assert!(
            gate.snapshot()
                .expect("departure denied")
                .ui
                .text_window_open,
            "missing guard {missing}"
        );
        let at = gate.snapshot().expect("owned gate position").overworld.tile;
        press(&mut gate, GameButton::Down);
        assert_eq!(
            gate.snapshot()
                .expect("gate dialogue owns input")
                .overworld
                .tile,
            at
        );
        assert!(drain(&mut gate).contains("winter loop"));
        let s = gate.snapshot().expect("safe denial return").overworld;
        assert_eq!((s.tile.x, s.tile.y), (60, 45));
        assert!(!flag(&gate, ARRIVED));
        assert!(!flag(&gate, GIVEN));
    }
    let mut incoming = fixture(&root, &r, "MnR17C00R00", (36, 0));
    cross(&mut incoming, GameButton::Up, CITY, (20, 27));
    travel(&mut incoming, &r, CLINIC);
    assert_eq!(
        incoming
            .snapshot()
            .expect("unqualified inbound care")
            .overworld
            .map_name,
        CLINIC
    );
    let mut wrong = fixture(&root, &r, "MnBoundaryWaters", (1, 24));
    qualify(&mut wrong);
    wrong
        .shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(REPORT, true)
        .expect("registered wrong-way fixture");
    mark_runtime_snapshot_dirty(&mut wrong.shell);
    press(&mut wrong, GameButton::Left);
    assert!(
        !flag(&wrong, ARRIVED),
        "westbound approach cannot stand in for the circuit arrival"
    );
    let mut unregistered = fixture(&root, &r, "MnR20C01R00", (63, 36));
    qualify(&mut unregistered);
    cross(
        &mut unregistered,
        GameButton::Right,
        "MnBoundaryWaters",
        (0, 24),
    );
    assert!(
        !flag(&unregistered, ARRIVED),
        "actual arrival still requires the ranger check-in"
    );
    let mut second_lane = fixture(&root, &r, "MnR20C01R00", (63, 37));
    qualify(&mut second_lane);
    second_lane
        .shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(REPORT, true)
        .expect("registered second-lane fixture");
    mark_runtime_snapshot_dirty(&mut second_lane.shell);
    cross(
        &mut second_lane,
        GameButton::Right,
        "MnBoundaryWaters",
        (0, 25),
    );
    assert!(
        flag(&second_lane, ARRIVED),
        "the second real outlet lane also records arrival"
    );
    let mut other_road = fixture(&root, &r, "MnR12C00R00", (0, 28));
    qualify(&mut other_road);
    other_road
        .shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(REPORT, true)
        .expect("other-road registration fixture");
    mark_runtime_snapshot_dirty(&mut other_road.shell);
    cross(
        &mut other_road,
        GameButton::Left,
        "MnBoundaryWaters",
        (63, 24),
    );
    assert!(
        !flag(&other_road, ARRIVED),
        "another region's arrival cannot complete Voyageurs"
    );
    let mut loaded = fixture(&root, &r, "MnBoundaryWaters", (0, 24));
    qualify(&mut loaded);
    loaded
        .shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(REPORT, true)
        .expect("saved-edge registration fixture");
    mark_runtime_snapshot_dirty(&mut loaded.shell);
    loaded = continue_save(root, r.clone(), &mut loaded);
    assert!(
        !flag(&loaded, ARRIVED),
        "Continue at the edge cannot invent a new-map arrival"
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with International Falls"]
fn international_falls_controller_walks_the_real_border_loop_and_retries_full_pack_after_continue()
{
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CLINIC, (2, 6));
    qualify(&mut c);
    // Optional matches are already defeated in this journey fixture. No route,
    // check-in, arrival or reward flag is invented by the movement planner.
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_MN_HEADWATERS_REPORT", true)
        .expect("explicit prior Headwaters evidence fixture for the east road");
    // This local border-quest case supplies already completed upstream mine
    // access, as well as the badges and lighthouse signal. The current Grand
    // Rapids road requires the actual depot handover before eastbound travel.
    // The border report, far-camp arrival and reward are still earned below.
    for name in [
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        "EVENT_MN_DEPOT_CLEARED",
    ] {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(name, true)
            .expect("declared completed upstream mine fixture");
    }
    let defeated = r
        .data()
        .maps
        .values()
        .filter(|m| m.id.starts_with("Mn") && !m.id.contains("Gym"))
        .flat_map(|m| m.trainer_scripts.values().map(|t| t.event_flag.clone()))
        .collect::<BTreeSet<_>>();
    for name in defeated {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(&name, true)
            .expect("defeated optional match fixture");
    }
    let mut partner = r
        .data()
        .create_pokemon("POUCHID", 80, Dv::default())
        .expect("active regional journey partner fixture");
    partner.item = Some("SMOKE_BALL".into());
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon.fill(None);
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    c.shell.shell.session_mut().state_mut().bag.items.clear();
    c.shell
        .shell
        .add_bag_item("POTION", 1)
        .expect("tossable fixture item");
    c.shell
        .shell
        .add_bag_item("MAX_REPEL", 99)
        .expect("carried journey Repels");
    while c.shell.shell.session().state().bag.items.len()
        < crystal_core::models::bag::ITEM_POCKET_CAPACITY
    {
        c.shell
            .shell
            .add_bag_item("BERRY", 99)
            .expect("full physical pocket fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    register(&mut c, true);
    assert!(flag(&c, REPORT));
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(!flag(&c, ARRIVED));
    travel(&mut c, &r, CITY);
    journey::walk(&mut c, &r, "MnRainyRiverIcefields", (60, 45));
    dry_walk(&mut c, &r, (63, 45));
    cross(&mut c, GameButton::Right, "MnR20C00R00", (0, 45));
    dry_walk(&mut c, &r, (63, 24));
    cross(&mut c, GameButton::Right, "MnR20C01R00", (0, 24));
    dry_walk(&mut c, &r, (63, 36));
    assert!(!flag(&c, ARRIVED));
    cross(&mut c, GameButton::Right, "MnBoundaryWaters", (0, 24));
    assert!(
        flag(&c, ARRIVED),
        "actual eastbound native connection must record the far camp"
    );
    assert!(!flag(&c, GIVEN));
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(flag(&c, ARRIVED));
    cross(&mut c, GameButton::Left, "MnR20C01R00", (63, 36));
    dry_walk(&mut c, &r, (0, 24));
    cross(&mut c, GameButton::Left, "MnR20C00R00", (63, 24));
    dry_walk(&mut c, &r, (0, 44));
    cross(&mut c, GameButton::Left, "MnRainyRiverIcefields", (63, 44));
    journey::walk(&mut c, &r, CITY, (20, 25));
    travel(&mut c, &r, CLINIC);
    walk(&mut c, &r, (2, 6));
    assert!(talk(&mut c, (2, 5)).contains("full"));
    assert!(!flag(&c, GIVEN));
    let item = &r.data().items["PP_UP"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 0);
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(!flag(&c, GIVEN));
    super::detroit_lakes::toss_potion(&mut c);
    talk(&mut c, (2, 5));
    assert!(flag(&c, GIVEN));
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    c = continue_save(root, r.clone(), &mut c);
    assert!(talk(&mut c, (2, 5)).contains("neighbors"));
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
}
#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with InternationalFalls"]
fn international_falls_controller_residents_rooms_carpets_and_real_southern_road() {
    let (root, r, _) = load();
    let plan: Value = serde_json::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/towns/MnInternationalFalls.json"),
        )
        .expect("InternationalFalls source"),
    )
    .expect("native source");
    let mut c = fixture(&root, &r, CITY, (20, 25));
    for actor in plan["residents"].as_array().expect("town residents") {
        walk(&mut c, &r, point(&actor["approach"]));
        assert!(!talk(&mut c, point(&actor["tile"])).is_empty());
    }
    for placard in plan["signs"].as_array().expect("public placards") {
        walk(&mut c, &r, point(&placard["approach"]));
        assert!(!talk(&mut c, point(&placard["tile"])).is_empty());
    }
    for door in plan["doors"].as_array().expect("two real facades") {
        let name = door["target"].as_str().expect("room");
        let room = plan["interiors"]
            .as_array()
            .expect("rooms")
            .iter()
            .find(|v| v["id"] == name)
            .expect("paired room");
        walk(&mut c, &r, point(&door["tile"]));
        assert_eq!(c.snapshot().expect("native door").overworld.map_name, name);
        for actor in room["residents"].as_array().expect("residents") {
            if actor["service"] == "pokemart" {
                continue;
            }
            walk(&mut c, &r, point(&actor["approach"]));
            assert!(
                !talk(
                    &mut c,
                    point(actor.get("counter_tile").unwrap_or(&actor["tile"]))
                )
                .is_empty()
            );
        }
        for exit in room["exits"].as_array().expect("paired carpets") {
            walk(&mut c, &r, point(&exit["tile"]));
            let s = c.snapshot().expect("exact town return").overworld;
            assert_eq!(s.map_name, CITY);
            assert_eq!((s.tile.x, s.tile.y), point(&door["return_tile"]));
            walk(&mut c, &r, point(&door["tile"]));
            assert_eq!(
                c.snapshot().expect("passive reentry").overworld.map_name,
                name
            );
        }
        travel(&mut c, &r, CITY);
    }
    qualify(&mut c);
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(REPORT, true)
        .expect("road fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    for lane in [20, 21] {
        walk(&mut c, &r, (lane, 27));
        cross(&mut c, GameButton::Down, "MnR17C00R00", (lane + 16, 0));
        cross(&mut c, GameButton::Up, CITY, (lane, 27));
    }
    walk(&mut c, &r, (26, 14));
    for _ in 0..4 {
        press(&mut c, GameButton::Right);
    }
    let boundary = c
        .snapshot()
        .expect("visible forest closes the false shortcut")
        .overworld;
    assert_eq!(boundary.map_name, CITY);
    assert_eq!((boundary.tile.x, boundary.tile.y), (27, 14));
    let before = c.snapshot().expect("town save").overworld;
    c = continue_save(root, r.clone(), &mut c);
    let after = c.snapshot().expect("continued town").overworld;
    assert_eq!((after.map_name, after.tile), (before.map_name, before.tile));
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with InternationalFalls"]
fn international_falls_controller_free_care_pc_poison_recovery_and_priced_stock() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CLINIC, (3, 8));
    let mut partner = r
        .data()
        .create_pokemon("CINDORE", 20, Dv::default())
        .expect("care fixture");
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
    walk(&mut c, &r, (3, 3));
    talk(&mut c, (3, 2));
    let s = c.snapshot().expect("native free care");
    let p = &s.party.slots[0].pokemon;
    assert_eq!(p.hp, p.max_hp);
    assert!(p.status.is_none());
    assert!(
        p.moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    assert_eq!(c.shell.shell.session().state().money, 0);
    walk(&mut c, &r, (9, 2));
    press(&mut c, GameButton::Up);
    for _ in 0..96 {
        press(&mut c, GameButton::A);
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
    }
    assert!(c.shell.pc_hub_cursor.is_some());
    let at = c.snapshot().expect("PC owner").overworld.tile;
    press(&mut c, GameButton::Down);
    assert_eq!(c.snapshot().expect("PC blocks walking").overworld.tile, at);
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_none() && !c.has_pending_script_work() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    c = continue_save(root.clone(), r.clone(), &mut c);
    travel(&mut c, &r, CITY);
    let state = c.shell.shell.session_mut().state_mut();
    let battle_result = state.battle_result;
    let p = state.storage.party.pokemon[0].as_mut().expect("partner");
    p.hp = 1;
    p.status = Some("POISON".into());
    state.sync_party_from_storage();
    state.money = 101;
    mark_runtime_snapshot_dirty(&mut c.shell);
    for _ in 0..128 {
        if c.shell.field_notice.is_some()
            || !c.shell.battle_messages.is_empty()
            || c.shell.visible_blackout_phase.is_some()
            || c.snapshot()
                .expect("whiteout input owner")
                .ui
                .text_window_open
        {
            press(&mut c, GameButton::A);
        } else {
            for b in [
                GameButton::Left,
                GameButton::Left,
                GameButton::Right,
                GameButton::Right,
            ] {
                press(&mut c, b);
            }
        }
        if c.snapshot().expect("recovery").overworld.map_name == CLINIC {
            break;
        }
    }
    let s = c.snapshot().expect("registered native recovery");
    assert_eq!(s.overworld.map_name, CLINIC);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), (3, 8));
    assert_eq!(s.party.slots[0].pokemon.hp, s.party.slots[0].pokemon.max_hp);
    assert_eq!(c.shell.shell.session().state().money, 50);
    assert_eq!(c.shell.shell.session().state().battle_result, battle_result);
    let mut shop = fixture(&root, &r, "MnInternationalFallsMart", (3, 6));
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
            .expect("stock")
            .pending_shop
            .expect("native shop")
            .inventory,
        r.data().marts.0["MART_MN_INTERNATIONAL_FALLS"]
    );
    let item = &r.data().items["ULTRA_BALL"];
    let before = shop.shell.shell.session().state().bag.quantity(item);
    for _ in 0..3 {
        press(&mut shop, GameButton::A);
    }
    assert_eq!(shop.shell.shell.session().state().money, 10_000);
    press(&mut shop, GameButton::A);
    assert_eq!(
        shop.shell.shell.session().state().money,
        10_000 - u32::from(item.price)
    );
    assert_eq!(
        shop.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
}
