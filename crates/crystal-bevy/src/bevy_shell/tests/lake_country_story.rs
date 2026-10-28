//! Prior badges/party are explicit setup; all chapter progress uses native input.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};
use serde_json::{Value, json};

use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::rochester::flee;
use super::super::winona::{at, continued, travel};
use super::{
    VisibleShellController, battle, city_enter_from_north, controller, event, finish, interact,
    load, mark_runtime_snapshot_dirty, point, press, solve_puzzle, walk_to,
};

const REPORT: &str = "EVENT_MN_BRAINERD_REPORT";
const SURF: &str = "EVENT_MN_SURF_GIFT_RECEIVED";
fn prior(count: usize) -> [bool; 8] {
    std::array::from_fn(|i| i < count)
}
fn fixture(
    map: &str,
    tile: [u16; 2],
    badges: [bool; 8],
    empty: bool,
) -> (AssetRoot, CrystalRuntime, Value, VisibleShellController) {
    let (root, r, design) = load();
    let mut initial = design["gyms"][4].clone();
    initial["id"] = map.into();
    initial["arrival_core"] = json!(tile);
    let mut c = controller(&root, &r, &initial, false);
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto = badges;
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, badges[i])
            .expect("explicit prior badges");
    }
    state.storage.party.pokemon.fill(None);
    if !empty {
        let mut p = r
            .data()
            .create_pokemon("MEW", 45, Dv::default())
            .expect("legal initial partner");
        p.moves = ["PSYCHIC_M", "FLAMETHROWER", "SURF", "THUNDERBOLT"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: r.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    (root, r, design, c)
}
fn road(c: &mut VisibleShellController, r: &CrystalRuntime, n: u8, target: (i16, i16)) {
    assert!(try_road_with(c, r, n, target, |c| {
        if matches!(
            c.snapshot()
                .expect("road encounter")
                .battle
                .expect("battle")
                .kind,
            RuntimeBattleKind::Wild { .. }
        ) {
            flee(c);
        } else {
            battle(c, false);
            finish(c);
        }
    }));
}

pub(super) fn try_road_with(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    n: u8,
    target: (i16, i16),
    mut encounter: impl FnMut(&mut VisibleShellController),
) -> bool {
    let start = position(c);
    let (prefix, rows, doors) = if n == 5 {
        (
            "MnR05",
            3,
            BTreeMap::from([((16, 77), "MnR05RockCut"), ((30, 27), "MnR05RockCut")]),
        )
    } else {
        ("MnR06", 2, BTreeMap::from([((9, 61), "MnR06NeedleCabin")]))
    };
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix,
            columns: 1,
            rows,
            tileset: "mn_headwaters",
            start,
            doors,
        },
    );
    for button in grid.path(start, target) {
        let old = position(c);
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("road encounter").battle.is_some() {
                encounter(c);
            }
            if !c
                .snapshot()
                .expect("road outcome")
                .overworld
                .map_name
                .starts_with(prefix)
            {
                return false;
            }
            if position(c) != old {
                break;
            }
        }
        assert_ne!(position(c), old, "real road input must move");
    }
    assert_eq!(position(c), target);
    true
}
fn unchanged_story(c: &VisibleShellController) {
    for name in [
        "EVENT_MN_DISPATCH_NOTE",
        "EVENT_MN_COURIER_DEFEATED",
        "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_DEPOT_CLEARED",
    ] {
        assert!(!event(c, name), "road cannot grant {name}");
    }
}
fn refusal(c: &mut VisibleShellController, map: &str, x: u16, text: &str) {
    for _ in 0..8 {
        if c.snapshot().expect("checkpoint").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Up);
    }
    assert!(
        c.snapshot()
            .expect("specific missing badge")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|s| s.contains(text))
    );
    press(c, GameButton::Up);
    at(c, map, (x as i16, 3));
    finish(c);
    at(c, map, (x as i16, 4));
    unchanged_story(c);
}

#[test]
#[ignore = "requires lake-country successor via CRYSTAL_GYM_TEST_PACK"]
fn lake_country_controller_two_city_badges_each_required_in_both_lanes() {
    let mut capitol_only = prior(4);
    capitol_only[2] = false;
    for badges in [prior(2), prior(3), capitol_only] {
        for x in [32, 33] {
            let (root, r, _, mut c) = fixture("MnMinneapolis", [x, 4], badges, true);
            refusal(
                &mut c,
                "MnMinneapolis",
                x,
                if badges[2] { "CAPITOL" } else { "AVRIN" },
            );
            c = continued(c, &root, &r, "lake-local-gate-refusal");
            refusal(
                &mut c,
                "MnMinneapolis",
                x,
                if badges[2] { "CAPITOL" } else { "AVRIN" },
            );
        }
    }
    for (x, outx) in [(32, 22), (33, 23)] {
        let (root, r, _, mut c) = fixture("MnMinneapolis", [x, 4], prior(4), true);
        if x == 32 {
            c = continued(c, &root, &r, "lake-local-road-ready");
        }
        travel(&mut c, GameButton::Up, "MnR05C00R02", (outx, 15));
        unchanged_story(&c);
    }
}

#[test]
#[ignore = "requires lake-country successor via CRYSTAL_GYM_TEST_PACK"]
fn lake_country_controller_free_inbound_return_and_wind_only_departure() {
    for (map, city, route_x, city_x, y, clinic, door, arrival) in [
        (
            "MnR05C00R02",
            "MnMinneapolis",
            22,
            32,
            15,
            "MnMinneapolisPokecenter1F",
            (45, 12),
            (10, 13),
        ),
        (
            "MnR06C00R01",
            "MnStCloud",
            16,
            24,
            39,
            "MnStCloudPokecenter1F",
            (6, 10),
            (10, 13),
        ),
    ] {
        for lane in [0, 1] {
            let (root, r, _, mut c) =
                fixture(map, [(route_x + lane) as u16, y as u16], prior(0), true);
            travel(&mut c, GameButton::Down, city, (city_x + lane, 0));
            c = continued(c, &root, &r, "lake-local-free-inbound");
            travel(&mut c, GameButton::Down, city, (city_x + lane, 4));
            walk_to(&mut c, &r, (door.0 as u16, door.1 as u16));
            travel(&mut c, GameButton::Up, clinic, arrival);
            assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
            unchanged_story(&c);
        }
    }
    for x in [24, 25] {
        let (_, _, _, mut c) = fixture("MnStCloud", [x, 4], prior(4), true);
        refusal(&mut c, "MnStCloud", x, "SAVEN");
        let (root, r, _, mut c) = fixture("MnStCloud", [x, 4], prior(5), true);
        if x == 24 {
            c = continued(c, &root, &r, "lake-local-wind-road-ready");
        }
        travel(&mut c, GameButton::Up, "MnR06C00R01", (x as i16 - 8, 39));
        unchanged_story(&c);
    }
}

#[test]
#[ignore = "requires lake-country successor via CRYSTAL_GYM_TEST_PACK"]
fn lake_country_controller_all_unbound_caps_stop_real_movement() {
    for (map, tile, button) in [
        ("MnR06C00R00", [40, 2], GameButton::Up),
        ("MnR06C00R00", [44, 2], GameButton::Up),
        ("MnR06C00R01", [12, 35], GameButton::Down),
        ("MnR06C00R01", [14, 37], GameButton::Down),
        ("MnR06C00R01", [18, 37], GameButton::Down),
        ("MnR06C00R01", [20, 35], GameButton::Down),
        ("MnBrainerd", [32, 2], GameButton::Up),
        ("MnBrainerd", [2, 24], GameButton::Left),
        ("MnBrainerd", [61, 24], GameButton::Right),
    ] {
        let (_, r, _, mut c) = fixture(map, tile, prior(5), true);
        let m = &r.data().maps[map];
        let b = m.blocks
            [usize::from(tile[1] / 2) * usize::from(m.attributes.width) + usize::from(tile[0] / 2)];
        assert_eq!(
            r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")]
                [usize::from(tile[1] % 2 * 2 + tile[0] % 2)],
            "FLOOR",
            "legal cap approach"
        );
        for _ in 0..4 {
            press(&mut c, button);
        }
        at(&mut c, map, (tile[0] as i16, tile[1] as i16));
    }
}

#[test]
#[ignore = "empty-party geometry fixture; requires lake-country successor"]
fn lake_country_controller_postponed_courier_reaches_st_cloud_and_returns() {
    let (root, r, _, mut c) = fixture("MnMinneapolis", [32, 4], prior(4), true);
    travel(&mut c, GameButton::Up, "MnR05C00R02", (22, 15));
    road(&mut c, &r, 5, (24, 0));
    travel(&mut c, GameButton::Up, "MnStCloud", (24, 35));
    travel(&mut c, GameButton::Up, "MnStCloud", (24, 34));
    c = continued(c, &root, &r, "lake-local-st-cloud-ready");
    walk_to(&mut c, &r, (24, 4));
    refusal(&mut c, "MnStCloud", 24, "SAVEN");
    walk_to(&mut c, &r, (24, 34));
    travel(&mut c, GameButton::Down, "MnR05C00R00", (24, 0));
    road(&mut c, &r, 5, (22, 111));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (32, 0));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (32, 4));
    unchanged_story(&c);
    assert_eq!(c.shell.shell.session().state().badges.johto, prior(4));
    let _ = continued(c, &root, &r, "lake-local-postponed-return");
}

fn heal(c: &mut VisibleShellController, r: &CrystalRuntime, city: &str, door: (u16, u16)) {
    let clinic = if city == "MnStCloud" {
        "MnStCloudPokecenter1F"
    } else {
        "MnBrainerdPokecenter1F"
    };
    city_enter_from_north(c, r, door, clinic, (10, 13));
    walk_to(c, r, (7, 5));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    assert!(
        c.snapshot()
            .expect("native care")
            .party
            .slots
            .iter()
            .all(|s| s.pokemon.hp == s.pokemon.max_hp
                && s.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp))
    );
    walk_to(c, r, (10, 14));
    travel(c, GameButton::Down, city, (door.0 as i16, door.1 as i16));
}
fn leave(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    gym: &Value,
    city: &str,
    door: (u16, u16),
) {
    let exit = point(&gym["exit_tiles_core"][0]);
    walk_to(c, r, (exit.0, exit.1 - 1));
    travel(c, GameButton::Down, city, (door.0 as i16, door.1 as i16));
}
fn earn(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    gym: &Value,
    city: &str,
    door: (u16, u16),
    clinic: (u16, u16),
) {
    let name = gym["id"].as_str().expect("gym");
    city_enter_from_north(c, r, door, name, point(&gym["arrival_core"]));
    solve_puzzle(c, r, gym);
    leave(c, r, gym, city, door);
    heal(c, r, city, clinic);
    city_enter_from_north(c, r, door, name, point(&gym["arrival_core"]));
    interact(c, r, point(&gym["leader_at"]));
    battle(c, false);
    finish(c);
    assert!(event(
        c,
        &format!(
            "EVENT_MN_GYM_{}_DEFEATED",
            gym["slot"].as_u64().expect("slot")
        )
    ));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[gym["reward"].as_str().expect("native TM")]),
        1
    );
    leave(c, r, gym, city, door);
}

#[test]
#[ignore = "one level45/four-prior-badge setup; requires lake-country successor"]
fn lake_country_controller_earns_wind_lake_and_ranger_then_walks_home() {
    let (root, r, design, mut c) = fixture("MnMinneapolis", [32, 4], prior(4), false);
    travel(&mut c, GameButton::Up, "MnR05C00R02", (22, 15));
    road(&mut c, &r, 5, (24, 0));
    travel(&mut c, GameButton::Up, "MnStCloud", (24, 35));
    travel(&mut c, GameButton::Up, "MnStCloud", (24, 34));
    heal(&mut c, &r, "MnStCloud", (6, 10));
    earn(
        &mut c,
        &r,
        &design["gyms"][4],
        "MnStCloud",
        (8, 28),
        (6, 10),
    );
    c = continued(c, &root, &r, "lake-local-earned-wind");
    walk_to(&mut c, &r, (24, 4));
    travel(&mut c, GameButton::Up, "MnR06C00R01", (16, 39));
    road(&mut c, &r, 6, (42, 0));
    travel(&mut c, GameButton::Up, "MnBrainerd", (32, 47));
    travel(&mut c, GameButton::Up, "MnBrainerd", (32, 46));
    walk_to(&mut c, &r, (25, 20));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(!event(&c, REPORT) && !event(&c, SURF));
    heal(&mut c, &r, "MnBrainerd", (9, 22));
    earn(
        &mut c,
        &r,
        &design["gyms"][5],
        "MnBrainerd",
        (12, 36),
        (9, 22),
    );
    c = continued(c, &root, &r, "lake-local-earned-lake");
    walk_to(&mut c, &r, (25, 20));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    for text in ["The fish moved on", "Ask the fishers"] {
        assert!(!event(&c, REPORT) && !event(&c, SURF));
        assert!(
            c.snapshot()
                .expect("ranger report")
                .ui
                .text
                .and_then(|t| t.asm_text)
                .is_some_and(|s| s.contains(text))
        );
        press(&mut c, GameButton::Down);
        at(&mut c, "MnBrainerd", (25, 20));
        press(&mut c, GameButton::A);
    }
    finish(&mut c);
    assert!(event(&c, REPORT) && event(&c, SURF));
    let item = &r.data().items["HM_SURF"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    c = continued(c, &root, &r, "lake-local-ranger-surf");
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(item), 1);
    walk_to(&mut c, &r, (32, 46));
    travel(&mut c, GameButton::Down, "MnR06C00R00", (42, 0));
    road(&mut c, &r, 6, (16, 87));
    travel(&mut c, GameButton::Down, "MnStCloud", (24, 0));
    travel(&mut c, GameButton::Down, "MnStCloud", (24, 4));
    walk_to(&mut c, &r, (24, 34));
    travel(&mut c, GameButton::Down, "MnR05C00R00", (24, 0));
    road(&mut c, &r, 5, (22, 111));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (32, 0));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (32, 4));
    assert_eq!(c.shell.shell.session().state().badges.johto, prior(6));
    unchanged_story(&c);
    let _ = continued(c, &root, &r, "lake-local-earned-home");
}
