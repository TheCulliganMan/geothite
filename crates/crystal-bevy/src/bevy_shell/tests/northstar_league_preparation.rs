//! Earn late-route experience and prize money before native League registration.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::super::rochester::flee;
use super::super::super::rochester_earned::try_battle;
use super::super::super::winona::{continue_in_place, press, travel};
use super::{VisibleShellController, care, event, supply_targets, walk_to};

fn settle(c: &mut VisibleShellController, r: &CrystalRuntime) {
    for _ in 0..256 {
        let snapshot = c.snapshot().expect("actual preparation encounter");
        if let Some(battle) = snapshot.battle {
            match battle.kind {
                RuntimeBattleKind::Trainer {
                    trainer_id,
                    event_flag,
                    ..
                } => {
                    assert!(
                        try_battle(c, r, &trainer_id, &event_flag),
                        "actual preparation battle lost: {trainer_id}"
                    );
                }
                RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. } => flee(c),
            }
        } else if snapshot.ui.text_window_open || c.has_pending_script_work() {
            press(c, GameButton::A);
        } else {
            return;
        }
    }
    panic!("preparation scene did not release native input");
}

fn grid(c: &mut VisibleShellController, r: &CrystalRuntime) -> RouteGrid {
    let mut doors = BTreeMap::new();
    for row in 0..3 {
        let room = &r.data().maps[&format!("MnR13C00R{row:02}")];
        for warp in &room.events.warps {
            doors.insert((warp.x as i16, row * 48 + warp.y as i16), "authored room");
        }
    }
    RouteGrid::new_for_battles(
        r,
        RouteSpec {
            prefix: "MnR13",
            columns: 1,
            rows: 3,
            tileset: if r.data().maps["MnR13C00R00"].attributes.tileset_name
                == "mn_winter_north_shore"
            {
                "mn_winter_north_shore"
            } else {
                "johto"
            },
            start: position(c),
            doors,
        },
    )
}

fn walk_route(c: &mut VisibleShellController, r: &CrystalRuntime, target: (i16, i16)) {
    let grid = grid(c, r);
    for _ in 0..2048 {
        settle(c, r);
        assert!(
            c.snapshot()
                .expect("native route or recovery")
                .overworld
                .map_name
                .starts_with("MnR13")
        );
        let before = position(c);
        if before == target {
            return;
        }
        let button = *grid
            .path(before, target)
            .first()
            .expect("remaining native road step");
        for _ in 0..8 {
            press(c, button);
            settle(c, r);
            if position(c) != before {
                break;
            }
        }
        assert_ne!(position(c), before, "native preparation path blocked");
    }
    panic!("bounded actual R13 preparation walk");
}

fn budget(c: &mut VisibleShellController, r: &CrystalRuntime) -> u32 {
    let targets = supply_targets(c, r);
    let state = c.shell.shell.session().state();
    targets
        .into_iter()
        .map(|(id, desired)| {
            let item = &r.data().items[id];
            u32::from(desired.saturating_sub(state.bag.quantity(item))) * u32::from(item.price)
        })
        .sum()
}

/// Uses only ordinary walks, unused trainer conversations, battle menus and
/// free clinic care. Earned flags, money, experience, HP and RNG are never set.
pub(super) fn play(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    assert_eq!(
        c.snapshot()
            .expect("earned Ely preparation")
            .overworld
            .map_name,
        "MnEly"
    );
    if super::coverage::teach_carried(c, r) {
        continue_in_place(c, root, r, "campaign-league-earned-coverage");
    }
    // The seventh trainer occupies a field-access branch. This walking
    // preparation uses the six opponents with ordinary dry approaches.
    for id in [
        "MN_R13_T10",
        "MN_R13_T04",
        "MN_R13_T05",
        "MN_R13_T02",
        "MN_R13_T01",
        "MN_R13_T03",
    ] {
        if c.shell.shell.session().state().money >= budget(c, r) {
            break;
        }
        let (name, room, script) = r
            .data()
            .maps
            .iter()
            .find_map(|(name, room)| {
                room.trainer_scripts
                    .values()
                    .find(|script| script.trainer_id == id)
                    .map(|script| (name, room, script))
            })
            .expect("installed regional preparation trainer");
        if event(c, &script.event_flag) {
            continue;
        }
        let actor = room
            .objects
            .iter()
            .find(|actor| actor.script == script.source_script)
            .expect("actual preparation trainer actor");
        let row = name
            .rsplit_once('R')
            .expect("road row")
            .1
            .parse::<i16>()
            .expect("road row number");
        care(c, r);
        continue_in_place(c, root, r, &format!("league-preparation-before-{id}"));
        walk_to(c, r, (24, 1));
        travel(c, GameButton::Up, "MnEly", (24, 0));
        travel(c, GameButton::Up, "MnR13C00R02", (34, 31));
        let grid = grid(c, r);
        let x = actor.x as i16;
        let y = actor.y as i16 + row * 48;
        let (target, facing) = [
            ((x, y + 1), GameButton::Up),
            ((x - 1, y), GameButton::Right),
            ((x + 1, y), GameButton::Left),
            ((x, y - 1), GameButton::Down),
        ]
        .into_iter()
        .find(|(point, _)| {
            grid.floor.contains(point)
                && !grid.blocked.contains(point)
                && !grid.hazards.contains(point)
        })
        .expect("unoccupied native trainer approach");
        let money = c.shell.shell.session().state().money;
        walk_route(c, r, target);
        if !event(c, &script.event_flag) {
            press(c, facing);
            press(c, GameButton::A);
            settle(c, r);
        }
        assert!(
            event(c, &script.event_flag),
            "preparation victory must be earned: {id}"
        );
        assert!(
            c.shell.shell.session().state().money > money,
            "native prize money required"
        );
        continue_in_place(c, root, r, &format!("league-preparation-won-{id}"));
        walk_route(c, r, (34, 126));
        travel(c, GameButton::Down, "MnEly", (24, 0));
        travel(c, GameButton::Down, "MnEly", (24, 1));
        continue_in_place(c, root, r, &format!("league-preparation-return-{id}"));
    }
}
