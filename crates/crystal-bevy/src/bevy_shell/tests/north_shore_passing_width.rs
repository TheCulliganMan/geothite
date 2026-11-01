//! Declared local road fixtures; all checks and travel use production input.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::systems::time::GameDate;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::rochester::flee;
use super::winona::{at, finish, load, press, travel};

fn setup(
    map: &str,
    tile: (i16, i16),
    badges: bool,
    signal: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let mut c = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
                map_name: map.into(),
                tile_x: tile.0,
                tile_y: tile.1,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("declared coastal fixture"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["CINDORE", "FORTIGARD"] {
        let mut mon = runtime
            .data()
            .create_pokemon(species, 80, Default::default())
            .expect("regional partner");
        assert!(mon.moves.iter().all(|m| m.name != "SURF"));
        mon.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(mon));
    }
    state.sync_party_from_storage();
    state.badges.johto = [badges; 8];
    for flag in &BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, badges)
            .expect("declared prior badges");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", signal)
        .expect("declared signal");
    for map in runtime
        .data()
        .maps
        .values()
        .filter(|m| m.id.starts_with("MnR11C"))
    {
        for trainer in map.trainer_scripts.values() {
            state
                .flags
                .set_event_flag(&trainer.event_flag, true)
                .expect("declared previously beaten coastal trainer");
        }
    }
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    state
        .time
        .update_server_datetime(GameDate::new(2000, 1, 1), 10, 0, 0);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c.wait_frames(16).expect("fixture settling");
    // No state, position, party, flags, RNG or map edits after this boundary.
    (root, runtime, c)
}

fn walking(c: &VisibleShellController) {
    let state = c.shell.shell.session().state();
    assert_eq!(
        state.overworld.snapshot_identity().expect("native map").3,
        MovementMode::Normal
    );
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .flat_map(|p| &p.moves)
            .all(|m| m.name != "SURF")
    );
}

fn save(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let out = PathBuf::from(
        std::env::var_os("CRYSTAL_NORTH_SHORE_WIDTH_REVIEW_DIR")
            .expect("fresh coastal review directory"),
    );
    std::fs::create_dir_all(&out).expect("review directory");
    let p = out.join(format!("{name}.crystalsave"));
    assert!(!p.exists(), "retain every reached state");
    c.save(&p).expect("reached native save");
    p
}

fn route_step(c: &mut VisibleShellController, button: GameButton, to: (i16, i16)) {
    for _ in 0..8 {
        if position(c) == to {
            walking(c);
            return;
        }
        press(c, button);
    }
    assert_eq!(position(c), to, "native deck step");
    walking(c);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_NORTH_SHORE_WIDTH_REVIEW_DIR"]
fn north_shore_width_controller_all_four_lanes_both_directions_and_solid_rails() {
    for (index, (x, y, length)) in [
        (40, 40, 2),
        (40, 76, 2),
        (44, 96, 4),
        (40, 110, 2),
        (34, 122, 2),
        (40, 162, 2),
        (46, 186, 2),
    ]
    .into_iter()
    .enumerate()
    {
        for lane in -1..=2 {
            let name = format!("MnR11C00R{:02}", (y - 1) / 48);
            let (_, _, mut c) = setup(&name, (x + lane, (y - 1) % 48), true, true);
            for yy in y..=y + length {
                route_step(&mut c, GameButton::Down, (x + lane, yy));
            }
            for yy in (y - 1..y + length).rev() {
                route_step(&mut c, GameButton::Up, (x + lane, yy));
            }
            route_step(&mut c, GameButton::Down, (x + lane, y));
            if lane == -1 {
                save(&mut c, &format!("bridge-{}", index + 1));
                for xx in x..=x + 2 {
                    route_step(&mut c, GameButton::Right, (xx, y));
                }
                for xx in (x - 1..x + 2).rev() {
                    route_step(&mut c, GameButton::Left, (xx, y));
                }
            }
            if lane == -1 || lane == 2 {
                let button = if lane == -1 {
                    GameButton::Left
                } else {
                    GameButton::Right
                };
                for yy in y..y + length {
                    for _ in 0..3 {
                        press(&mut c, button);
                        assert_eq!(position(&mut c), (x + lane, yy), "solid rail");
                    }
                    route_step(&mut c, GameButton::Down, (x + lane, yy + 1));
                }
            }
            walking(&c);
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with published passing-width checkpoints"]
fn north_shore_width_controller_every_town_lane_denies_missing_badges_or_signal() {
    for (badges, signal, reason) in [(false, true, "BadgesText"), (true, false, "EvidenceText")] {
        for (map, starts, button, prefix) in [
            (
                "MnDuluth",
                vec![(64, 4), (65, 4), (66, 4), (67, 4)],
                GameButton::Up,
                "MnDuluthShoreCheck",
            ),
            (
                "MnGrandMarais",
                vec![(4, 8), (4, 9), (4, 10), (4, 11)],
                GameButton::Left,
                "MnCohesionMaraisWest",
            ),
        ] {
            for tile in starts {
                let (_, _, mut c) = setup(map, tile, badges, signal);
                for _ in 0..8 {
                    press(&mut c, button);
                    if c.snapshot().expect("checkpoint").ui.text_window_open {
                        break;
                    }
                }
                let s = c.snapshot().expect("denial page");
                assert_eq!(
                    s.ui.text.as_ref().expect("visible gate denial").label,
                    format!("{prefix}{reason}")
                );
                let before = s.overworld.tile;
                press(&mut c, button);
                assert_eq!(
                    c.snapshot().expect("dialogue owns movement").overworld.tile,
                    before
                );
                finish(&mut c);
                at(&mut c, map, tile);
                walking(&c);
            }
        }
    }
}

fn road(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let start = position(c);
    let mut doors = BTreeMap::new();
    for row in 0..4 {
        for column in 0..2 {
            let name = format!("MnR11C{column:02}R{row:02}");
            for warp in &r.data().maps[&name].events.warps {
                doors.insert(
                    (column * 64 + warp.x as i16, row * 48 + warp.y as i16),
                    "authored room",
                );
            }
        }
    }
    let mut grid = RouteGrid::new_for_battles(
        r,
        RouteSpec {
            prefix: "MnR11",
            columns: 2,
            rows: 4,
            tileset: "mn_north_shore",
            start,
            doors,
        },
    );
    let original_bodies = grid.blocked.clone();
    for _ in 0..2048 {
        if c.snapshot()
            .expect("native road input owner")
            .ui
            .text_window_open
            || c.has_pending_script_work()
        {
            finish(c);
        }
        let old = position(c);
        if old == to {
            return;
        }
        let snapshot = c.snapshot().expect("live coastal actors");
        let origin = super::authored_route_fixture::origin(&snapshot.overworld.map_name);
        grid.blocked = original_bodies.clone();
        for object in &r.data().maps[&snapshot.overworld.map_name].objects {
            grid.blocked
                .remove(&(origin.0 + object.x as i16, origin.1 + object.y as i16));
        }
        for object in &snapshot.visible_objects {
            let tile = c
                .shell
                .shell
                .session()
                .overworld()
                .object_runtime_tile_by_id(
                    object
                        .object_identifier
                        .as_deref()
                        .expect("native actor identifier"),
                )
                .expect("live actor tile");
            grid.blocked.insert((origin.0 + tile.x, origin.1 + tile.y));
        }
        // Replan actual inputs after each step, so a moving hiker is walked
        // around instead of treated as their original exported position.
        let button = *grid.path(old, to).first().expect("remaining coastal step");
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("native road").battle.is_some() {
                flee(c);
            }
            if position(c) != old {
                break;
            }
        }
        walking(c);
    }
    panic!("native coastal journey exceeded its bounded course to {to:?}");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_NORTH_SHORE_WIDTH_REVIEW_DIR"]
fn north_shore_width_controller_new_town_lanes_roundtrip_without_surf_and_continue() {
    for lane in 64..=67 {
        let (root, r, mut c) = setup("MnDuluth", (lane, 4), true, true);
        travel(&mut c, GameButton::Up, "MnR11C00R03", (lane - 18, 47));
        travel(&mut c, GameButton::Down, "MnDuluth", (lane, 0));
        let path = save(&mut c, &format!("duluth-lane-{lane}"));
        let mut next = VisibleShellController::load_save(root, r, path, None)
            .expect("Continue at widened town mouth");
        next.set_runtime_journal_enabled(false);
        next.wait_frames(16).expect("Continue presentation");
        finish(&mut next);
        at(&mut next, "MnDuluth", (lane, 0));
        for _ in 0..4 {
            press(&mut next, GameButton::Down);
            if next.snapshot().expect("inward check").overworld.tile.y == 3 {
                break;
            }
        }
        at(&mut next, "MnDuluth", (lane, 3));
        walking(&next);
    }
    for lane in 8..=11 {
        let (_, _, mut c) = setup("MnGrandMarais", (4, lane), true, true);
        travel(&mut c, GameButton::Left, "MnR11C01R00", (7, lane));
        travel(&mut c, GameButton::Right, "MnGrandMarais", (0, lane));
        save(&mut c, &format!("marais-lane-{lane}"));
        walking(&c);
    }
    let (root, r, mut c) = setup("MnDuluth", (66, 4), true, true);
    travel(&mut c, GameButton::Up, "MnR11C00R03", (48, 47));
    road(&mut c, &r, (71, 10));
    travel(&mut c, GameButton::Right, "MnGrandMarais", (0, 10));
    save(&mut c, "reached-grand-marais-without-surf");
    travel(&mut c, GameButton::Left, "MnR11C01R00", (7, 10));
    road(&mut c, &r, (48, 191));
    travel(&mut c, GameButton::Down, "MnDuluth", (66, 0));
    let mut next =
        VisibleShellController::load_save(root, r, save(&mut c, "coast-roundtrip"), None)
            .expect("Continue after full coast walk");
    next.set_runtime_journal_enabled(false);
    next.wait_frames(16).expect("Continue");
    finish(&mut next);
    at(&mut next, "MnDuluth", (66, 0));
    walking(&next);
}
