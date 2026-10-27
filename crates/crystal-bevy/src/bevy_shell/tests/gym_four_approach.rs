//! Current Badge, dispatch evidence and the actual river approach to Gym4.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;
use serde_json::{Value, json};

use super::super::authored_route_fixture::{RouteGrid, RouteSpec};
use super::super::rochester::flee;
use super::super::winona::{at as location, travel};
use super::{
    VisibleShellController, at, battle, continue_save, controller, event, finish, interact, load,
    mark_runtime_snapshot_dirty, point, press, solve_puzzle, walk_to,
};

const CITY: &str = "MnMinneapolis";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

fn placement(gym: &Value, tile: [u16; 2]) -> Value {
    let mut initial = gym.clone();
    initial["id"] = CITY.into();
    initial["arrival_core"] = json!(tile);
    initial
}

fn refuse(c: &mut VisibleShellController, y: u16, text: &str) {
    for _ in 0..8 {
        if c.snapshot().expect("gate refusal").ui.text.is_some() {
            break;
        }
        press(c, GameButton::Right);
    }
    let snapshot = c.snapshot().expect("visible requirement");
    assert_eq!(snapshot.overworld.map_name, CITY);
    assert!(
        snapshot
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains(text))
    );
    let held = at(c);
    press(c, GameButton::Down);
    assert_eq!(at(c), held, "dialogue owns directional input");
    finish(c);
    assert_eq!(at(c), (58, y));
}

fn crossing(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let tile = c.snapshot().expect("R04 start").overworld.tile;
    let start = (tile.x, tile.y);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR04",
            columns: 1,
            rows: 1,
            tileset: "mn_river_park",
            start,
            doors: Default::default(),
        },
    );
    for button in grid.path(start, to) {
        let old = c.snapshot().expect("crossing tile").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("wild encounter").battle.is_some() {
                flee(c);
            }
            let now = c.snapshot().expect("played route step");
            assert_eq!(now.overworld.map_name, "MnR04C00R00");
            if now.overworld.tile != old {
                break;
            }
        }
        assert_ne!(c.snapshot().expect("completed step").overworld.tile, old);
    }
    location(c, "MnR04C00R00", to);
}

#[test]
#[ignore = "requires fourth-gym approach via CRYSTAL_GYM_TEST_PACK"]
fn gym_four_approach_controller_both_lanes_require_badge_allow_free_return() {
    let (root, r, design) = load();
    for (badge, note, text) in [(false, false, "AVRIN"), (false, true, "AVRIN")] {
        for (y, road_y) in [(24, 18), (25, 19)] {
            let mut c = controller(&root, &r, &placement(&design["gyms"][2], [58, y]), false);
            let state = c.shell.shell.session_mut().state_mut();
            state.badges.johto[2] = badge;
            state
                .flags
                .set_engine_flag("ENGINE_PLAINBADGE", badge)
                .expect("badge fixture");
            state
                .flags
                .set_event_flag(NOTE, note)
                .expect("evidence fixture");
            mark_runtime_snapshot_dirty(&mut c.shell);
            refuse(&mut c, y, text);
            c = continue_save(&mut c, &root, &r);
            refuse(&mut c, y, text);
            assert_eq!(event(&c, NOTE), note);
            assert!(!event(&c, "EVENT_MN_GYM_4_DEFEATED"));
            assert_eq!(c.shell.shell.session().state().badges.johto[2], badge);

            // Separate unqualified inbound placement proves free return even
            // after Continue outside the wardens. No progress flags are seeded.
            let mut initial = placement(&design["gyms"][2], [0, road_y]);
            initial["id"] = "MnR04C00R00".into();
            let mut c = controller(&root, &r, &initial, false);
            travel(&mut c, GameButton::Left, CITY, (63, y as i16));
            c = continue_save(&mut c, &root, &r);
            travel(&mut c, GameButton::Left, CITY, (58, y as i16));
            refuse(&mut c, y, "AVRIN");
        }
    }
}

#[test]
#[ignore = "requires fourth-gym approach via CRYSTAL_GYM_TEST_PACK"]
fn gym_four_approach_controller_qualified_both_lanes_reconcile_without_extra_victory_flags() {
    let (root, r, design) = load();
    for note in [false, true] {
        for (y, road_y) in [(24, 18), (25, 19)] {
            let mut c = controller(&root, &r, &placement(&design["gyms"][3], [58, y]), false);
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_engine_flag("ENGINE_PLAINBADGE", true)
                .expect("badge fixture");
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(NOTE, note)
                .expect("evidence fixture");
            // Gate/stdio setup is deliberately empty-party. The separate course
            // below verifies actual combat and earned qualifications.
            let state = c.shell.shell.session_mut().state_mut();
            state.storage.party.pokemon.fill(None);
            state.sync_party_from_storage();
            mark_runtime_snapshot_dirty(&mut c.shell);
            if y == 24
                && !note
                && let Some(directory) = std::env::var_os("CRYSTAL_GYM_FOUR_SAVE_DIR")
            {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory).expect("review save directory");
                c.save(directory.join("qualified.crystalsave"))
                    .expect("explicit local review save");
            }
            assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
            travel(&mut c, GameButton::Right, "MnR04C00R00", (0, road_y));
            c = continue_save(&mut c, &root, &r);
            travel(&mut c, GameButton::Left, CITY, (63, y as i16));
            travel(&mut c, GameButton::Left, CITY, (58, y as i16));
            travel(&mut c, GameButton::Right, "MnR04C00R00", (0, road_y));
            assert!(!event(&c, "EVENT_MN_GYM_4_DEFEATED"));
            assert_eq!(event(&c, NOTE), note);
        }
    }
}

#[test]
#[ignore = "strong-party/two-prior-badge fixture; requires fourth-gym approach"]
fn gym_four_approach_controller_rescue_third_badge_r04_fourth_badge_and_return() {
    let (root, r, design) = load();
    let third = &design["gyms"][2];
    let fourth = &design["gyms"][3];
    // One initial city placement, first two badges, level45 legal partner and
    // money. Every subsequent flag, badge, battle, resource and move is earned.
    let mut c = controller(&root, &r, &placement(third, [27, 14]), false);
    let mut partner = r
        .data()
        .create_pokemon("MEW", 45, Dv::default())
        .expect("legal initial partner");
    partner.moves = ["PSYCHIC_M", "FLAMETHROWER", "SURF", "THUNDERBOLT"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: r.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon[0] = Some(partner);
    state.sync_party_from_storage();
    state.money = 10000;
    mark_runtime_snapshot_dirty(&mut c.shell);
    super::city_enter_from_north(&mut c, &r, (27, 14), "MnMinneapolisDispatcherHouse", (5, 6));
    c = super::super::minneapolis::play_rescue(&root, &r, c);
    walk_to(&mut c, &r, (5, 6));
    travel(&mut c, GameButton::Down, CITY, (27, 14));
    walk_to(&mut c, &r, (58, 24));
    refuse(&mut c, 24, "AVRIN");
    assert!(
        event(&c, NOTE),
        "evidence alone cannot open the third-badge gate"
    );
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
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_THUNDERPUNCH"]),
        1
    );
    let exit = point(&third["exit_tiles_core"][0]);
    walk_to(&mut c, &r, (exit.0, exit.1 - 1));
    travel(&mut c, GameButton::Down, CITY, (42, 24));

    // Refill the actual spent PP through free native care before the next gym.
    super::city_enter_from_north(&mut c, &r, (45, 12), "MnMinneapolisPokecenter1F", (10, 13));
    walk_to(&mut c, &r, (7, 5));
    let money = c.shell.shell.session().state().money;
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    let healed = c.snapshot().expect("native clinic care");
    assert!(healed.party.slots.iter().all(|p| {
        p.pokemon.hp == p.pokemon.max_hp
            && p.pokemon.status.is_none()
            && p.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, money);
    walk_to(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, CITY, (45, 12));
    walk_to(&mut c, &r, (58, 24));
    c = continue_save(&mut c, &root, &r);
    travel(&mut c, GameButton::Right, "MnR04C00R00", (0, 18));
    crossing(&mut c, &r, (55, 18));
    travel(&mut c, GameButton::Right, "MnStPaul", (0, 24));
    travel(&mut c, GameButton::Right, "MnStPaul", (1, 24));
    super::city_enter_from_north(
        &mut c,
        &r,
        (54, 16),
        "MnStPaulGym",
        point(&fourth["arrival_core"]),
    );
    solve_puzzle(&mut c, &r, fourth);
    interact(&mut c, &r, point(&fourth["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_STEEL_WING"]),
        1
    );
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, false, false, false, false]
    );
    c = continue_save(&mut c, &root, &r);
    let exit = point(&fourth["exit_tiles_core"][0]);
    walk_to(&mut c, &r, (exit.0, exit.1 - 1));
    travel(&mut c, GameButton::Down, "MnStPaul", (54, 16));
    walk_to(&mut c, &r, (1, 24));
    travel(&mut c, GameButton::Left, "MnR04C00R00", (55, 18));
    crossing(&mut c, &r, (0, 18));
    travel(&mut c, GameButton::Left, CITY, (63, 24));
    travel(&mut c, GameButton::Left, CITY, (58, 24));
    assert!(
        !event(&c, "EVENT_MN_COURIER_DEFEATED"),
        "this approach never invents the courier chapter"
    );
    let _ = continue_save(&mut c, &root, &r);
}
