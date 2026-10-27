//! Native second-badge departure plus a continuous local Gym2-to-Gym3 course.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::rochester::flee;
use super::super::winona::{at as location, travel};
use super::{
    VisibleShellController, at, battle, continue_save, controller, event, finish, interact, load,
    mark_runtime_snapshot_dirty, point, press, solve_puzzle, walk_to,
};

fn placement(gym: &Value, map: &str, tile: [u16; 2]) -> Value {
    let mut initial = gym.clone();
    initial["id"] = map.into();
    initial["arrival_core"] = serde_json::json!(tile);
    initial
}

fn road(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let from = position(c);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR03",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start: from,
            doors: std::collections::BTreeMap::from([((8, 37), "MnR03Cave1F")]),
        },
    );
    for button in grid.path(from, to) {
        let old = position(c);
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("route encounter").battle.is_some() {
                flee(c);
            }
            if position(c) != old {
                break;
            }
        }
        assert_ne!(position(c), old, "real route step must finish");
        assert!(
            c.snapshot()
                .expect("road module")
                .overworld
                .map_name
                .starts_with("MnR03C00R")
        );
    }
    assert_eq!(position(c), to);
}

fn refusal(c: &mut VisibleShellController, x: u16) {
    for _ in 0..8 {
        if c.snapshot().expect("badge refusal").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Up);
    }
    let s = c.snapshot().expect("visible qualification");
    assert_eq!(s.overworld.map_name, "MnMankato");
    assert!(
        s.ui.text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("TORVEN"))
    );
    let held = at(c);
    press(c, GameButton::Up);
    assert_eq!(at(c), held, "text owns directional input");
    finish(c);
    assert_eq!(at(c), (x, 4));
    assert!(!c.shell.shell.session().state().badges.johto[1]);
    assert!(!event(c, "EVENT_MN_GYM_2_DEFEATED"));
    assert!(!event(c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
}

#[test]
#[ignore = "requires published River Valley candidate via CRYSTAL_GYM_TEST_PACK"]
fn river_cities_controller_both_gate_lanes_refuse_and_inbound_return_is_free() {
    let (root, r, design) = load();
    for (x, road_x) in [(24, 18), (25, 19)] {
        let mut c = controller(
            &root,
            &r,
            &placement(&design["gyms"][1], "MnMankato", [x, 4]),
            false,
        );
        refusal(&mut c, x);
        c = continue_save(&mut c, &root, &r);
        refusal(&mut c, x);
        let mut c = controller(
            &root,
            &r,
            &placement(&design["gyms"][1], "MnR03C00R01", [road_x, 15]),
            false,
        );
        travel(&mut c, GameButton::Down, "MnMankato", (x as i16, 0));
        c = continue_save(&mut c, &root, &r);
        travel(&mut c, GameButton::Down, "MnMankato", (x as i16, 4));
        refusal(&mut c, x);
    }
}

#[test]
#[ignore = "local boundary placements; requires published River Valley candidate"]
fn river_cities_controller_south_bank_caps_stop_stray_edge_departures() {
    let (root, r, design) = load();
    // These six approaches led to unpaired floor lanes in the partial build.
    // The native visible forest now stops movement before the outer boundary.
    for x in [14, 16, 20, 22, 26, 37] {
        let mut c = controller(
            &root,
            &r,
            &placement(&design["gyms"][2], "MnR03C00R01", [x, 13]),
            false,
        );
        for _ in 0..4 {
            press(&mut c, GameButton::Down);
        }
        location(&mut c, "MnR03C00R01", (x as i16, 13));
    }
}

#[test]
#[ignore = "requires published River Valley candidate via CRYSTAL_GYM_TEST_PACK"]
fn river_cities_controller_badge_alone_opens_both_lanes_without_notebook() {
    let (root, r, design) = load();
    for (x, road_x) in [(24, 18), (25, 19)] {
        // Initial prior badges and strong party are explicit fixtures. There
        // is no gym2 victory event, notebook or invented quest prerequisite.
        let mut c = controller(
            &root,
            &r,
            &placement(&design["gyms"][2], "MnMankato", [x, 4]),
            false,
        );
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_engine_flag("ENGINE_HIVEBADGE", true)
            .expect("explicit badge fixture");
        // This separate traversal/MCP fixture deliberately has no party. It
        // proves geometry and badge ownership, not a populated earned journey.
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .storage
            .party
            .pokemon
            .fill(None);
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut c.shell);
        assert!(!event(&c, "EVENT_MN_GYM_2_DEFEATED"));
        assert!(!event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
        if x == 24 {
            c = super::super::winona::continued(c, &root, &r, "river-local-road-ready");
        }
        travel(&mut c, GameButton::Up, "MnR03C00R01", (road_x, 15));
        c = continue_save(&mut c, &root, &r);
        travel(&mut c, GameButton::Down, "MnMankato", (x as i16, 0));
        travel(&mut c, GameButton::Down, "MnMankato", (x as i16, 4));
        travel(&mut c, GameButton::Up, "MnR03C00R01", (road_x, 15));
    }
}

#[test]
#[ignore = "strong-party/prior-badge fixture; requires published River Valley candidate"]
fn river_cities_controller_earns_second_then_walks_r03_and_earns_third_badge() {
    let (root, r, design) = load();
    let second = &design["gyms"][1];
    let third = &design["gyms"][2];
    // One initial placement, first badge and level100 legal partner. Every
    // subsequent battle, badge, puzzle, step, crossing and save is real input.
    let mut c = controller(&root, &r, &placement(second, "MnMankato", [40, 18]), false);
    super::city_enter_from_north(
        &mut c,
        &r,
        (10, 12),
        "MnMankatoGym",
        point(&second["arrival_core"]),
    );
    solve_puzzle(&mut c, &r, second);
    interact(&mut c, &r, point(&second["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[1]);
    assert!(event(&c, "EVENT_MN_GYM_2_DEFEATED"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_MUD_SLAP"]),
        1
    );
    super::mankato_leave(&mut c, &r, point(&second["exit_tiles_core"][0]), (10, 12));
    walk_to(&mut c, &r, (24, 4));
    c = continue_save(&mut c, &root, &r);
    travel(&mut c, GameButton::Up, "MnR03C00R01", (18, 15));
    assert!(!event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
    road(&mut c, &r, (20, 0));
    c = continue_save(&mut c, &root, &r);
    travel(&mut c, GameButton::Up, "MnMinneapolis", (32, 47));
    travel(&mut c, GameButton::Up, "MnMinneapolis", (32, 46));
    super::city_enter_from_north(
        &mut c,
        &r,
        (42, 24),
        "MnMinneapolisGym",
        point(&third["arrival_core"]),
    );
    solve_puzzle(&mut c, &r, third);
    interact(&mut c, &r, point(&third["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_3_DEFEATED"));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, false, false, false, false, false]
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_THUNDERPUNCH"]),
        1
    );
    c = continue_save(&mut c, &root, &r);
    let carpet = point(&third["exit_tiles_core"][0]);
    walk_to(&mut c, &r, (carpet.0, carpet.1 - 1));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (42, 24));
    walk_to(&mut c, &r, (32, 46));
    travel(&mut c, GameButton::Down, "MnR03C00R00", (20, 0));
    road(&mut c, &r, (18, 63));
    travel(&mut c, GameButton::Down, "MnMankato", (24, 0));
    travel(&mut c, GameButton::Down, "MnMankato", (24, 4));
    walk_to(&mut c, &r, (10, 12));
    location(&mut c, "MnMankato", (10, 12));
    assert!(!event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
    let _ = continue_save(&mut c, &root, &r);
}

#[test]
#[ignore = "local strong-party/two-badge setup; requires both gym and Minneapolis pack variables"]
fn river_cities_controller_walks_to_ice_wins_rescues_reads_and_returns() {
    use super::super::authored_route_fixture::indoor_walk;
    use super::super::minneapolis::{fixture, play_rescue};

    // One initial legal level45 partner and two prior badges. No later state
    // edits: walking, wild escapes, ICE win, rescue and evidence are native.
    let (root, r, mut c) = fixture("MnMankato", (24, 4), true, false, false);
    travel(&mut c, GameButton::Up, "MnR03C00R01", (18, 15));
    road(&mut c, &r, (20, 0));
    travel(&mut c, GameButton::Up, "MnMinneapolis", (32, 47));
    travel(&mut c, GameButton::Up, "MnMinneapolis", (32, 46));
    indoor_walk(&mut c, &r, (27, 14));
    travel(
        &mut c,
        GameButton::Up,
        "MnMinneapolisDispatcherHouse",
        (5, 6),
    );
    c = play_rescue(&root, &r, c);
    indoor_walk(&mut c, &r, (5, 6));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (27, 14));
    indoor_walk(&mut c, &r, (32, 46));
    travel(&mut c, GameButton::Down, "MnR03C00R00", (20, 0));
    road(&mut c, &r, (18, 63));
    travel(&mut c, GameButton::Down, "MnMankato", (24, 0));
    travel(&mut c, GameButton::Down, "MnMankato", (24, 4));
    assert!(event(&c, "EVENT_MN_DISPATCH_NOTE"));
    assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    let _ = super::super::winona::continued(c, &root, &r, "river-local-ice-return");
}
