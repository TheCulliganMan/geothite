//! Local empty-party R20 fixture; all dungeon movement uses production input.
#[path = "voyageurs_surveyor.rs"]
mod surveyor;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::world::collision::{
    is_direction_blocked, is_direction_blocked_leaving, permissions,
};
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::{Point, neighbors};
use super::r10_gallery::{completed_visible_page, continue_save, map, press, settle, tile};

const UPPER: &str = "MnVoyageursIce1F";
const LOWER: &str = "MnVoyageursIceB1F";

fn kind<'a>(runtime: &'a CrystalRuntime, m: &MapModule, at: Point) -> &'a str {
    if at.0 < 0
        || at.1 < 0
        || at.0 >= m.attributes.width as i16 * 2
        || at.1 >= m.attributes.height as i16 * 2
    {
        return "WALL";
    }
    let (x, y) = (at.0 as usize, at.1 as usize);
    let block = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    &runtime.data().tilesets[&m.attributes.tileset_name].collision[&format!("{block:02x}")]
        [y % 2 * 2 + x % 2]
}

fn route(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    to: Point,
) -> Vec<(Point, GameButton)> {
    let m = &runtime.data().maps[&map(c)];
    let state = c.shell.shell.session().state();
    let bodies = m
        .objects
        .iter()
        .filter(|o| {
            o.event_flag == "-1"
                || !state
                    .flags
                    .is_event_flag_set(&o.event_flag)
                    .expect("registered reward flag")
        })
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(tile(c), None)]);
    let mut queue = VecDeque::from([tile(c)]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for (next, button) in neighbors(at) {
            let d = (next.0 - at.0, next.1 - at.1);
            let facing = match button {
                GameButton::Left => Direction::Left,
                GameButton::Right => Direction::Right,
                GameButton::Up => Direction::Up,
                GameButton::Down => Direction::Down,
                _ => unreachable!("neighbor planner yields only directional inputs"),
            };
            let mut stop = at;
            let mut next = next;
            loop {
                let k = kind(runtime, m, next);
                if (kind(runtime, m, stop) == "UP_WALL"
                    && is_direction_blocked_leaving(permissions::UP_WALL, facing))
                    || (k == "UP_WALL" && is_direction_blocked(permissions::UP_WALL, facing))
                {
                    break;
                }
                if bodies.contains(&next)
                    || !(matches!(k, "FLOOR" | "ICE" | "ICE_2B" | "UP_WALL")
                        || (next == to && k == "LADDER"))
                {
                    break;
                }
                stop = next;
                if !matches!(k, "ICE" | "ICE_2B") {
                    break;
                }
                next = (next.0 + d.0, next.1 + d.1);
            }
            if let Entry::Vacant(entry) = parents.entry(stop) {
                entry.insert(Some((at, button)));
                queue.push_back(stop);
            }
        }
    }
    assert!(
        parents.contains_key(&to),
        "{}: no slide course {:?} -> {to:?}",
        m.id,
        tile(c)
    );
    let mut result = Vec::new();
    let mut at = to;
    while let Some((before, button)) = parents[&at] {
        result.push((at, button));
        at = before;
    }
    result.reverse();
    result
}

fn travel(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point, destination: &str) {
    let course = route(c, runtime, to);
    for (expected, button) in course {
        for _ in 0..4 {
            press(c, button);
            c.wait_frames(80).expect("native slide clock");
            if tile(c) == expected || map(c) == destination {
                break;
            }
        }
        if map(c) == destination && !destination.is_empty() {
            return;
        }
        assert_eq!(tile(c), expected, "ice stops at native rock/dry ground");
        assert!(c.snapshot().expect("ice course").battle.is_none());
    }
    if !destination.is_empty() {
        assert_eq!(map(c), destination);
    }
}

fn collect(c: &mut VisibleShellController, runtime: &CrystalRuntime, id: &str, item_id: &str) {
    let m = &runtime.data().maps[&map(c)];
    let object = m
        .objects
        .iter()
        .find(|o| o.script == id)
        .expect("authored reward");
    let at = (object.x as i16, object.y as i16);
    let (approach, button) = neighbors(at)
        .into_iter()
        .filter(|(p, _)| kind(runtime, m, *p) == "FLOOR")
        .map(|(p, _)| {
            let toward = neighbors(p)
                .into_iter()
                .find(|(q, _)| *q == at)
                .expect("adjacent")
                .1;
            (p, toward)
        })
        .min_by_key(|(p, _)| route(c, runtime, *p).len())
        .expect("dry reward approach");
    travel(c, runtime, approach, "");
    let item = &runtime.data().items[item_id];
    let before = c.shell.shell.session().state().bag.quantity(item);
    press(c, button);
    press(c, GameButton::A);
    settle(c, "ice reward notice");
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(&object.event_flag)
            .expect("reward flag")
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
    press(c, GameButton::A);
    settle(c, "collected reward");
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
}

#[test]
#[ignore = "requires published Voyageurs ice pack; local fixture does not earn regional access"]
fn voyageurs_ice_controller_slides_rewards_ladders_return_and_continue() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_ICE_DUNGEON_PACK").expect("published ice pack"))
            .canonicalize()
            .expect("local pack");
    let checkpoint = std::env::var_os("CRYSTAL_ICE_DUNGEON_CHECKPOINT").map(PathBuf::from);
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified ice pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR20C01R00".into(),
            tile_x: 6,
            tile_y: 36,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local R20 fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    c.set_runtime_journal_enabled(false);
    for expected in [(6, 35), (6, 34)] {
        for _ in 0..4 {
            press(&mut c, GameButton::Up);
            if tile(&mut c) == expected {
                break;
            }
        }
        assert_eq!(tile(&mut c), expected);
    }
    press(&mut c, GameButton::Up);
    assert_eq!(map(&mut c), UPPER);
    assert_eq!(tile(&mut c), (5, 19));
    travel(&mut c, &runtime, (4, 16), "");
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
    c.wait_frames(240).expect("print native guide page");
    let first = completed_visible_page(&mut c).expect("visible guide page");
    assert!(first.1.contains("ice carries"));
    let before = tile(&mut c);
    press(&mut c, GameButton::Up);
    assert_eq!(tile(&mut c), before, "guide dialogue owns movement");
    press(&mut c, GameButton::A);
    c.wait_frames(240).expect("print next guide page");
    let second = completed_visible_page(&mut c).expect("next visible guide page");
    assert_ne!(first, second, "A advances native dialogue one page");
    settle(&mut c, "ice guide releases input");
    collect(&mut c, &runtime, "MnVoyageursIceHeal", "ICE_HEAL");
    travel(&mut c, &runtime, (23, 5), LOWER);
    assert_eq!(tile(&mut c), (5, 19));
    travel(&mut c, &runtime, (5, 18), "");
    c.press(GameButton::Up).expect("start north slide");
    assert_eq!(tile(&mut c).0, 5);
    let y = tile(&mut c).1;
    c.press(GameButton::Right)
        .expect("perpendicular input during forced slide");
    assert_eq!(tile(&mut c).0, 5, "ice owns direction before stopping");
    assert!(tile(&mut c).1 < y, "ice continues north");
    c.wait_frames(80).expect("settle against rock");
    assert_eq!(tile(&mut c), (5, 11));
    collect(&mut c, &runtime, "MnVoyageursIceNeverMelt", "NEVERMELTICE");
    collect(&mut c, &runtime, "MnVoyageursIceRevive", "MAX_REVIVE");
    travel(&mut c, &runtime, (19, 12), "");
    assert_eq!(kind(&runtime, &runtime.data().maps[LOWER], (19, 12)), "ICE");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    c.wait_frames(16)
        .expect("continued ice stop remains stationary");
    assert_eq!(tile(&mut c), (19, 12));
    if let Some(path) = checkpoint.as_ref() {
        c.save(path)
            .expect("export actual local ice-stop checkpoint");
    }
    travel(&mut c, &runtime, (5, 21), UPPER);
    assert_eq!(tile(&mut c), (23, 6));
    travel(&mut c, &runtime, (5, 21), "MnR20C01R00");
    assert_eq!(tile(&mut c), (6, 34));
    if let Some(path) = checkpoint.as_ref() {
        c.save(path.with_extension("outside.crystalsave"))
            .expect("export actual same-mouth exterior checkpoint");
    }
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
        if map(&mut c) == UPPER {
            break;
        }
    }
    assert_eq!(map(&mut c), UPPER, "same-mouth reentry");
    assert_eq!(tile(&mut c), (5, 19));
    travel(&mut c, &runtime, (23, 5), LOWER);
    for flag in [
        "EVENT_MN_VOYAGEURS_ICE_HEAL",
        "EVENT_MN_VOYAGEURS_ICE_NEVERMELT",
        "EVENT_MN_VOYAGEURS_ICE_REVIVE",
    ] {
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("persistent reward")
        );
    }
}
