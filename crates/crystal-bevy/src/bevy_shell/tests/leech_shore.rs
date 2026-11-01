//! Explicit prior badges/partner/RNG; progress thereafter uses native input.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeBattleSnapshot};
use serde_json::{Value, json};

use super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::rochester::flee;
use super::super::winona::{at, continued, travel};
use super::{
    VisibleShellController, battle, city_enter_from_north, controller, event, finish, interact,
    load, mark_runtime_snapshot_dirty, point, press, solve_puzzle, walk_to,
};

const SHORE: &str = "MnLeechLake";
const CARE: &str = "MnLeechLakeCareCabin";
fn badges(n: usize) -> [bool; 8] {
    std::array::from_fn(|i| i < n)
}
pub(super) fn fixture(
    map: &str,
    tile: [u16; 2],
    count: usize,
    empty: bool,
) -> (AssetRoot, CrystalRuntime, Value, VisibleShellController) {
    let (root, r, design) = load();
    let mut initial = design["gyms"][6].clone();
    initial["id"] = map.into();
    initial["arrival_core"] = json!(tile);
    let mut c = controller(&root, &r, &initial, false);
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto = badges(count);
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, i < count)
            .expect("prior badge fixture");
    }
    state.storage.party.pokemon.fill(None);
    if !empty {
        let mut p = r
            .data()
            .create_pokemon("MEW", 45, Dv::default())
            .expect("legal review partner");
        p.moves = ["PSYCHIC_M", "FLAMETHROWER", "CUT", "THUNDERBOLT"]
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
fn no_story(c: &VisibleShellController) {
    for name in [
        "EVENT_MN_BRAINERD_REPORT",
        "EVENT_MN_SURF_GIFT_RECEIVED",
        "EVENT_MN_HEADWATERS_READING",
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_COURIER_DEFEATED",
        "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(c, name), "no free story progress: {name}");
    }
}
fn road(c: &mut VisibleShellController, r: &CrystalRuntime, n: u8, target: (i16, i16)) {
    let start = position(c);
    let (prefix, columns) = if n == 7 { ("MnR07", 1) } else { ("MnR08", 2) };
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix,
            columns,
            rows: 2,
            tileset: "mn_headwaters",
            start,
            doors: BTreeMap::new(),
        },
    );
    for button in grid.path(start, target) {
        let before = position(c);
        for _ in 0..8 {
            press(c, button);
            if let Some(b) = c.snapshot().expect("road activity").battle {
                if matches!(b.kind, RuntimeBattleKind::Wild { .. }) {
                    flee(c);
                } else {
                    battle(c, false);
                    finish(c);
                }
            }
            if position(c) != before {
                break;
            }
        }
        assert_ne!(position(c), before, "native road step");
    }
    assert_eq!(position(c), target);
}
fn refuse(c: &mut VisibleShellController, map: &str, x: i16) {
    for _ in 0..8 {
        if c.snapshot().expect("checkpoint").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Up);
    }
    let snapshot = c.snapshot().expect("Lake refusal");
    assert!(
        snapshot
            .ui
            .text
            .as_ref()
            .and_then(|t| t.asm_text.as_ref())
            .is_some_and(|s| s.contains("LAKE")),
        "Lake refusal at {map} lane {x}: {:?}",
        snapshot.ui
    );
    press(c, GameButton::Up);
    at(c, map, (x, 3));
    finish(c);
    at(c, map, (x, 4));
    no_story(c);
}

// Selects only visible legal moves, allowing the caller to verify either the
// actual loss/recovery or victory. No outcome, HP, PP or reward is rewritten.
pub(super) fn play_with_move(c: &mut VisibleShellController, move_name: &str) {
    play_with_selected_move(c, |_| move_name);
}

/// Chooses learned moves from the visible opponent, then uses normal menu input.
pub(super) fn play_with_selected_move<'a>(
    c: &mut VisibleShellController,
    choose: impl Fn(&RuntimeBattleSnapshot) -> &'a str,
) {
    let before = c.snapshot().expect("before native leader");
    let mut started = false;
    let mut attacks = 0;
    for _ in 0..4096 {
        let s = c.snapshot().expect("native leader turn");
        let Some(b) = &s.battle else {
            if started {
                break;
            }
            press(c, GameButton::A);
            continue;
        };
        started = true;
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            c.wait_frames(30).expect("battle narration");
            GameButton::A
        } else if s.ui.menu.as_ref().is_some_and(|m| {
            m.layout
                .vertical_menus
                .iter()
                .flat_map(|v| &v.options)
                .any(|s| s.contains("change <PKMN>"))
        }) {
            GameButton::B
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let move_name = choose(b);
            let target = b
                .player_moves
                .iter()
                .position(|m| m.name == move_name && m.current_pp > 0)
                .expect("chosen native move with PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                attacks += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("native battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    let after = c.snapshot().expect("native leader result");
    assert!(started && attacks > 0 && after.battle.is_none());
    let pp = |moves: &[LearnedMove]| moves.iter().map(|m| u32::from(m.current_pp)).sum::<u32>();
    assert!(
        pp(&after.party.slots[0].pokemon.moves) < pp(&before.party.slots[0].pokemon.moves),
        "actual selected attacks spend PP"
    );
}
pub(super) fn from_brainerd(c: &mut VisibleShellController, r: &CrystalRuntime) {
    travel(c, GameButton::Up, "MnR07C00R01", (20, 47));
    road(c, r, 7, (20, 0));
    travel(c, GameButton::Up, SHORE, (32, 47));
    travel(c, GameButton::Up, SHORE, (32, 44));
}
fn bemidji_route_port(r: &CrystalRuntime) -> i16 {
    let connection = r.data().maps["MnR08C00R00"]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction.eq_ignore_ascii_case("north") && link.target_map == "MnBemidji")
        .expect("published Bemidji walking connection");
    24 + i16::try_from(connection.offset * 2).expect("native R08 offset")
}
pub(super) fn to_bemidji(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (32, 4));
    travel(c, GameButton::Up, "MnR08C00R01", (26, 47));
    road(c, r, 8, (bemidji_route_port(r), 0));
    travel(c, GameButton::Up, "MnBemidji", (24, 47));
    travel(c, GameButton::Up, "MnBemidji", (24, 46));
}
fn home(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (24, 46));
    travel(
        c,
        GameButton::Down,
        "MnR08C00R00",
        (bemidji_route_port(r), 0),
    );
    road(c, r, 8, (26, 95));
    travel(c, GameButton::Down, SHORE, (32, 0));
    travel(c, GameButton::Down, SHORE, (32, 4));
    walk_to(c, r, (32, 44));
    travel(c, GameButton::Down, "MnR07C00R00", (20, 0));
    road(c, r, 7, (20, 95));
    travel(c, GameButton::Down, "MnBrainerd", (32, 0));
    travel(c, GameButton::Down, "MnBrainerd", (32, 4));
}
pub(super) fn heal(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    city: &str,
    door: (u16, u16),
    room: &str,
) {
    city_enter_from_north(c, r, door, room, (10, 13));
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

#[test]
#[ignore = "requires external shore successor via CRYSTAL_GYM_TEST_PACK"]
fn leech_shore_controller_lake_only_both_lanes_and_free_unqualified_returns() {
    for city in ["MnBrainerd", SHORE] {
        for x in [32, 33] {
            let (root, r, _, mut c) = fixture(city, [x, 4], 5, true);
            refuse(&mut c, city, x as i16);
            c = continued(c, &root, &r, "leech-local-denied");
            refuse(&mut c, city, x as i16);
        }
    }
    for (route, city, rx, y) in [
        ("MnR07C00R01", "MnBrainerd", 20, 47),
        ("MnR08C00R01", SHORE, 26, 47),
    ] {
        for lane in [0, 1] {
            let (root, r, _, mut c) = fixture(route, [(rx + lane) as u16, y], 0, true);
            travel(&mut c, GameButton::Down, city, (32 + lane, 0));
            c = continued(c, &root, &r, "leech-local-free-inbound");
            travel(&mut c, GameButton::Down, city, (32 + lane, 4));
            let (door, room) = if city == SHORE {
                ((11, 14), CARE)
            } else {
                ((9, 22), "MnBrainerdPokecenter1F")
            };
            heal(&mut c, &r, city, door, room);
            no_story(&c);
            assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
        }
    }
}

#[test]
#[ignore = "explicit empty-party/six-prior-badge geometry setup"]
fn leech_shore_controller_walks_both_routes_without_ranger_surf_or_courier() {
    let (root, r, _, mut c) = fixture("MnBrainerd", [32, 4], 6, true);
    c = continued(c, &root, &r, "leech-local-road-ready");
    from_brainerd(&mut c, &r);
    c = continued(c, &root, &r, "leech-local-shore-ready");
    walk_to(&mut c, &r, (35, 22));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("fisher testimony")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|s| s.contains("report"))
    );
    finish(&mut c);
    walk_to(&mut c, &r, (40, 22));
    for _ in 0..4 {
        press(&mut c, GameButton::Right);
    }
    at(&mut c, SHORE, (40, 22));
    to_bemidji(&mut c, &r);
    c = continued(c, &root, &r, "leech-local-bemidji-ready");
    heal(&mut c, &r, "MnBemidji", (31, 10), "MnBemidjiPokecenter1F");
    home(&mut c, &r);
    no_story(&c);
    assert_eq!(c.shell.shell.session().state().badges.johto, badges(6));
    let _ = continued(c, &root, &r, "leech-local-empty-home");
}

#[test]
#[ignore = "local damaged-partner setup; native healing/room returns/Continue"]
fn leech_shore_controller_care_witness_both_room_carpets_and_persistence() {
    let (root, r, _, mut c) = fixture(SHORE, [11, 14], 6, false);
    {
        let state = c.shell.shell.session_mut().state_mut();
        let p = state.storage.party.pokemon[0].as_mut().expect("partner");
        p.hp = 1;
        p.moves[0].current_pp = 0;
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    city_enter_from_north(&mut c, &r, (11, 14), CARE, (10, 13));
    walk_to(&mut c, &r, (7, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    at(&mut c, CARE, (7, 5));
    finish(&mut c);
    let p = &c.snapshot().expect("restored partner").party.slots[0].pokemon;
    assert_eq!(p.hp, p.max_hp);
    assert_eq!(p.moves[0].current_pp, r.data().moves["PSYCHIC_M"].pp);
    c = continued(c, &root, &r, "leech-local-native-care");
    for (room, door, arrival, carpets) in [
        (CARE, (11, 14), (10, 13), [(10, 15), (11, 15)]),
        ("MnLeechLakeFisherHouse", (9, 36), (5, 6), [(5, 7), (6, 7)]),
    ] {
        for (index, carpet) in carpets.into_iter().enumerate() {
            if room != CARE || index > 0 {
                city_enter_from_north(&mut c, &r, door, room, arrival);
            }
            if room != CARE {
                walk_to(&mut c, &r, (6, 4));
                press(&mut c, GameButton::Up);
                press(&mut c, GameButton::A);
                finish(&mut c);
            }
            walk_to(&mut c, &r, (carpet.0, carpet.1 - 1));
            travel(
                &mut c,
                GameButton::Down,
                SHORE,
                (door.0 as i16, door.1 as i16),
            );
            press(&mut c, GameButton::Down);
            at(&mut c, SHORE, (door.0 as i16, door.1 as i16 + 1));
        }
    }
    no_story(&c);
    assert_eq!(c.shell.shell.session().state().money, 0);
    let _ = continued(c, &root, &r, "leech-local-cabins-complete");
}

#[test]
#[ignore = "one level45/Cut/six-prior-badge local setup; native Pine earned"]
fn leech_shore_controller_continuous_pine_loss_retry_and_return_to_brainerd() {
    let (root, r, mut c) = pine_course();
    home(&mut c, &r);
    no_story(&c);
    let _ = continued(c, &root, &r, "leech-local-earned-home");
}

pub(super) fn pine_course() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, design, mut c) = fixture("MnBrainerd", [32, 4], 6, false);
    from_brainerd(&mut c, &r);
    heal(&mut c, &r, SHORE, (11, 14), CARE);
    to_bemidji(&mut c, &r);
    heal(&mut c, &r, "MnBemidji", (31, 10), "MnBemidjiPokecenter1F");
    let gym = &design["gyms"][6];
    city_enter_from_north(
        &mut c,
        &r,
        (10, 10),
        "MnBemidjiGym",
        point(&gym["arrival_core"]),
    );
    interact(&mut c, &r, point(&gym["guide_at"]));
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["HM_CUT"]),
        1
    );
    solve_puzzle(&mut c, &r, gym);
    let exit = point(&gym["exit_tiles_core"][0]);
    walk_to(&mut c, &r, (exit.0, exit.1 - 1));
    travel(&mut c, GameButton::Down, "MnBemidji", (10, 10));
    heal(&mut c, &r, "MnBemidji", (31, 10), "MnBemidjiPokecenter1F");
    city_enter_from_north(
        &mut c,
        &r,
        (10, 10),
        "MnBemidjiGym",
        point(&gym["arrival_core"]),
    );
    interact(&mut c, &r, point(&gym["leader_at"]));
    let before_loss = c.shell.shell.session().state().money;
    // Road bends change encounter/divider consumption before this battle.
    // The already-known weaker neutral Cut provides a real loss attempt
    // without rewriting party HP, the opponent, or RNG after setup.
    play_with_move(&mut c, "CUT");
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_7_DEFEATED"));
    assert_eq!(c.shell.shell.session().state().badges.johto, badges(6));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_GIGA_DRAIN"]),
        0
    );
    assert!(c.shell.shell.session().state().money < before_loss);
    at(&mut c, "MnBemidjiPokecenter1F", (10, 13));
    c = continued(c, &root, &r, "leech-local-pine-loss");
    // Ordinary blackout care restores the partner; use the already learned
    // super-effective Fire move on the retry rather than changing the fixture.
    walk_to(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, "MnBemidji", (31, 10));
    city_enter_from_north(
        &mut c,
        &r,
        (10, 10),
        "MnBemidjiGym",
        point(&gym["arrival_core"]),
    );
    interact(&mut c, &r, point(&gym["leader_at"]));
    play_with_move(&mut c, "FLAMETHROWER");
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_DEFEATED"));
    assert_eq!(c.shell.shell.session().state().badges.johto, badges(7));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_GIGA_DRAIN"]),
        1
    );
    c = continued(c, &root, &r, "leech-local-earned-pine");
    walk_to(&mut c, &r, (exit.0, exit.1 - 1));
    travel(&mut c, GameButton::Down, "MnBemidji", (10, 10));
    (root, r, c)
}

#[test]
#[ignore = "local legal Surf-partner/six-prior-badge setup"]
fn leech_shore_controller_separate_surf_launch_closed_coast_and_dismount() {
    let (root, r, _, mut c) = fixture(SHORE, [31, 29], 6, false);
    {
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("partner")
            .moves[2] = LearnedMove {
            name: "SURF".into(),
            current_pp: r.data().moves["SURF"].pp,
            pp_ups: 0,
        };
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    for _ in 0..128 {
        if c.snapshot().expect("native launch").overworld.mode == MovementMode::Surf {
            break;
        }
        press(&mut c, GameButton::A);
    }
    at(&mut c, SHORE, (32, 29));
    assert_eq!(
        c.snapshot().expect("mounted").overworld.mode,
        MovementMode::Surf
    );
    for x in 33..=61 {
        travel(&mut c, GameButton::Right, SHORE, (x, 29));
    }
    for _ in 0..4 {
        press(&mut c, GameButton::Right);
    }
    at(&mut c, SHORE, (61, 29));
    c = continued(c, &root, &r, "leech-local-surf-coast");
    assert_eq!(
        c.snapshot().expect("continued Surf").overworld.mode,
        MovementMode::Surf
    );
    for x in (31..=60).rev() {
        travel(&mut c, GameButton::Left, SHORE, (x, 29));
    }
    assert_eq!(
        c.snapshot().expect("native dismount").overworld.mode,
        MovementMode::Normal
    );
    no_story(&c);
}

#[test]
#[ignore = "local Magikarp/money/prior-badge setup; actual care and blackout"]
fn leech_shore_controller_native_care_is_actual_blackout_recovery() {
    let (root, r, _, mut c) = fixture(SHORE, [32, 44], 6, false);
    {
        let state = c.shell.shell.session_mut().state_mut();
        let mut p = r
            .data()
            .create_pokemon("MAGIKARP", 5, Dv::default())
            .expect("legal loss partner");
        p.hp = 1;
        p.moves = vec![LearnedMove {
            name: "SPLASH".into(),
            current_pp: r.data().moves["SPLASH"].pp,
            pp_ups: 0,
        }];
        state.storage.party.pokemon[0] = Some(p);
        state.money = 1200;
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    heal(&mut c, &r, SHORE, (11, 14), CARE);
    assert_eq!(c.shell.shell.session().state().money, 1200);
    walk_to(&mut c, &r, (32, 44));
    travel(&mut c, GameButton::Down, "MnR07C00R00", (20, 0));
    let start = position(&mut c);
    let grid = RouteGrid::new_with_grass(
        &r,
        RouteSpec {
            prefix: "MnR07",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start,
            doors: BTreeMap::new(),
        },
    );
    let mut fought = false;
    'route: for button in grid.path(start, (14, 13)) {
        let before = position(&mut c);
        for _ in 0..8 {
            press(&mut c, button);
            if c.snapshot().expect("route loss encounter").battle.is_some() {
                battle(&mut c, true);
                fought = true;
                break 'route;
            }
            if position(&mut c) != before {
                break;
            }
        }
    }
    if !fought {
        at(&mut c, "MnR07C00R00", (14, 13));
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        battle(&mut c, true);
    }
    finish(&mut c);
    at(&mut c, CARE, (10, 13));
    let s = c.snapshot().expect("native shore blackout care");
    assert_eq!(s.party.slots[0].pokemon.hp, s.party.slots[0].pokemon.max_hp);
    assert_eq!(
        s.party.slots[0].pokemon.moves[0].current_pp,
        r.data().moves["SPLASH"].pp
    );
    assert!(c.shell.shell.session().state().money < 1200);
    no_story(&c);
    let _ = continued(c, &root, &r, "leech-local-care-blackout");
}
