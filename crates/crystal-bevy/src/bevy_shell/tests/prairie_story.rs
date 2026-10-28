//! Explicit local contracts plus a separate fully earned New Game chapter audit.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, indoor_walk};
use super::rochester::flee;
use super::winona::{at, continued, finish, load, press, travel};

const NOTE: &str = "EVENT_MN_PRAIRIE_NOTEBOOK";
fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("chapter event")
}
fn fixture(
    map: &str,
    tile: (i16, i16),
    care: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: tile.0,
            tile_y: tile.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit chapter fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    // Explicit empty-party local setup, not earned regional progression.
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[0] = care;
    state
        .flags
        .set_engine_flag("ENGINE_ZEPHYRBADGE", care)
        .expect("fixture Care flag");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, r, c)
}

pub(super) fn road(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    assert!(
        try_road_with(c, r, target, flee),
        "R02 walk ended at recovery"
    );
}

pub(super) fn try_road_with(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: (i16, i16),
    mut encounter: impl FnMut(&mut VisibleShellController),
) -> bool {
    let s = c.snapshot().expect("R02 start");
    let start = (s.overworld.tile.x, s.overworld.tile.y);
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR02",
            columns: 1,
            rows: 1,
            tileset: "mn_prairie_kanto",
            start,
            doors: BTreeMap::from([((11, 11), "MnR02Farmhouse"), ((52, 33), "MnR02ToolShed")]),
        },
    );
    for button in grid.path(start, target) {
        let old = c.snapshot().expect("road tile").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("road encounter").battle.is_some() {
                encounter(c);
            }
            let s = c.snapshot().expect("road movement");
            if s.overworld.map_name != "MnR02C00R00" {
                return false;
            }
            if s.overworld.tile != old {
                break;
            }
        }
    }
    at(c, "MnR02C00R00", target);
    true
}

fn read_notebook(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (26, 26));
    press(c, GameButton::Up);
    let before = c.snapshot().expect("before notebook");
    press(c, GameButton::A);
    assert!(!event(c, NOTE));
    let expected = super::northstar_dialogue::canonical_pages("MS02", "MnMKNotebookText");
    assert_eq!(
        c.snapshot()
            .expect("first page")
            .ui
            .text
            .and_then(|t| t.asm_text),
        Some(expected[0].clone())
    );
    press(c, GameButton::Down);
    at(c, "MnMankato", (26, 26));
    assert!(!event(c, NOTE));
    press(c, GameButton::A);
    assert!(
        !event(c, NOTE),
        "one acknowledgment cannot finish two pages"
    );
    assert_eq!(
        c.snapshot()
            .expect("second page")
            .ui
            .text
            .and_then(|t| t.asm_text),
        Some(expected[1].clone())
    );
    finish(c);
    assert!(event(c, NOTE));
    let after = c.snapshot().expect("recorded clue");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    let money = c.shell.shell.session().state().money;
    press(c, GameButton::Up);
    press(c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("notebook repeat")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|s| s.contains("same numbers"))
    );
    finish(c);
    assert_eq!(c.snapshot().expect("repeat").bag, before.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, false, false, false, false, false, false, false]
    );
}

#[test]
#[ignore = "requires prairie story via CRYSTAL_WINONA_TEST_PACK"]
fn prairie_story_local_care_gate_refuses_both_lanes_and_inbound_return_is_free() {
    for (y, route_y) in [(18, 14), (19, 15)] {
        let (_, _, mut c) = fixture("MnRochester", (4, y), false);
        for _ in 0..8 {
            if c.snapshot().expect("Care refusal").ui.text.is_some() {
                break;
            }
            press(&mut c, GameButton::Left);
            assert_eq!(
                c.snapshot().expect("locked departure").overworld.map_name,
                "MnRochester"
            );
        }
        assert!(c.snapshot().expect("Care refusal").ui.text.is_some());
        press(&mut c, GameButton::Left);
        at(&mut c, "MnRochester", (3, y));
        finish(&mut c);
        at(&mut c, "MnRochester", (4, y));
        assert!(!event(&c, NOTE));
        assert!(!c.shell.shell.session().state().badges.johto[0]);
        let (_, r, mut c) = fixture("MnR02C00R00", (59, route_y), false);
        travel(&mut c, GameButton::Right, "MnRochester", (0, y));
        travel(&mut c, GameButton::Right, "MnRochester", (4, y));
        indoor_walk(&mut c, &r, (15, 8));
        travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
        assert!(!c.shell.shell.session().state().badges.johto[0]);
        assert!(!event(&c, NOTE));
    }
}

#[test]
#[ignore = "requires prairie story via CRYSTAL_WINONA_TEST_PACK"]
fn prairie_story_local_notebook_denies_without_care_and_preserves_progress() {
    let (_, _, mut c) = fixture("MnMankato", (26, 26), false);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("notebook prerequisite")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|s| s.contains("VAREL"))
    );
    finish(&mut c);
    assert!(!event(&c, NOTE));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
}

#[test]
#[ignore = "requires prairie story via CRYSTAL_WINONA_TEST_PACK"]
fn prairie_story_local_route_notebook_rooms_return_and_continue() {
    let (root, r, mut c) = fixture("MnRochester", (4, 18), true);
    let money = c.shell.shell.session().state().money;
    assert!(
        !event(&c, "EVENT_MN_GYM_1_DEFEATED"),
        "badge-only gate must not invent another prerequisite"
    );
    c = continued(c, &root, &r, "prairie-local-road-ready");
    travel(&mut c, GameButton::Left, "MnR02C00R00", (59, 14));
    road(&mut c, &r, (0, 30));
    travel(&mut c, GameButton::Left, "MnMankato", (47, 18));
    travel(&mut c, GameButton::Left, "MnMankato", (46, 18));
    assert!(!event(&c, NOTE));
    // Gym entry remains independent of the optional notebook.
    indoor_walk(&mut c, &r, (10, 12));
    travel(&mut c, GameButton::Up, "MnMankatoGym", (6, 17));
    indoor_walk(&mut c, &r, (6, 18));
    travel(&mut c, GameButton::Down, "MnMankato", (10, 12));
    indoor_walk(&mut c, &r, (26, 26));
    c = continued(c, &root, &r, "mankato-local-before-notebook");
    read_notebook(&mut c, &r);
    c = continued(c, &root, &r, "mankato-local-notebook");
    assert!(event(&c, NOTE));
    for (door, room, arrival, carpet) in [
        ((31, 14), "MnMankatoPokecenter1F", (10, 13), (10, 14)),
        ((37, 28), "MnMankatoMart", (6, 5), (6, 6)),
    ] {
        indoor_walk(&mut c, &r, door);
        travel(&mut c, GameButton::Up, room, arrival);
        indoor_walk(&mut c, &r, carpet);
        travel(&mut c, GameButton::Down, "MnMankato", door);
    }
    indoor_walk(&mut c, &r, (46, 18));
    travel(&mut c, GameButton::Right, "MnR02C00R00", (0, 30));
    road(&mut c, &r, (59, 14));
    travel(&mut c, GameButton::Right, "MnRochester", (0, 18));
    travel(&mut c, GameButton::Right, "MnRochester", (4, 18));
    assert!(event(&c, NOTE));
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(!event(&c, "EVENT_MN_GYM_2_DEFEATED"));
    let _ = continued(c, &root, &r, "mankato-local-return");
}

#[test]
#[ignore = "manual live-RNG New Game to Mankato audit; requires prairie story via CRYSTAL_WINONA_TEST_PACK"]
fn prairie_story_earned_new_game_care_r02_notebook_and_return() {
    let (root, r, mut c) = mankato_arrival();
    indoor_walk(&mut c, &r, (46, 18));
    travel(&mut c, GameButton::Right, "MnR02C00R00", (0, 30));
    road(&mut c, &r, (59, 14));
    travel(&mut c, GameButton::Right, "MnRochester", (0, 18));
    travel(&mut c, GameButton::Right, "MnRochester", (4, 18));
    assert!(event(&c, NOTE));
    let _ = continued(c, &root, &r, "prairie-earned-return");
}

pub(super) fn mankato_arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = super::rochester_earned::care_badge();
    indoor_walk(&mut c, &r, (15, 8));
    travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
    indoor_walk(&mut c, &r, (7, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(
        c.snapshot()
            .expect("care")
            .party
            .slots
            .iter()
            .all(|s| s.pokemon.hp == s.pokemon.max_hp)
    );
    indoor_walk(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, "MnRochester", (15, 8));
    indoor_walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Left, "MnR02C00R00", (59, 14));
    c = continued(c, &root, &r, "prairie-earned-road");
    road(&mut c, &r, (0, 30));
    travel(&mut c, GameButton::Left, "MnMankato", (47, 18));
    travel(&mut c, GameButton::Left, "MnMankato", (46, 18));
    read_notebook(&mut c, &r);
    c = continued(c, &root, &r, "mankato-earned-notebook");
    assert!(event(&c, NOTE));
    (root, r, c)
}
