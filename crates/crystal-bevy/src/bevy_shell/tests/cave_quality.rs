//! Explicit local empty-party placements; all subsequent travel is production input.
#[path = "cave_quality_interactions.rs"]
mod interactions;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::{Point, neighbors};
use super::r10_gallery::{continue_save, flags, map, press, settle, tile};

fn kind<'a>(runtime: &'a CrystalRuntime, module: &MapModule, at: Point) -> &'a str {
    let x = usize::try_from(at.0).expect("bounded x");
    let y = usize::try_from(at.1).expect("bounded y");
    let block = module.blocks[y / 2 * usize::from(module.attributes.width) + x / 2];
    &runtime.data().tilesets["cave"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn point(value: &Value) -> Point {
    (
        value[0].as_i64().expect("x") as i16,
        value[1].as_i64().expect("y") as i16,
    )
}

fn walk(c: &mut VisibleShellController, module: &MapModule, floor: &BTreeSet<Point>, to: Point) {
    let start = tile(c);
    let mut parents = BTreeMap::from([(start, None)]);
    let mut todo = VecDeque::from([start]);
    while let Some(at) = todo.pop_front() {
        for (next, button) in neighbors(at) {
            if floor.contains(&next) && !parents.contains_key(&next) {
                parents.insert(next, Some((at, button)));
                todo.push_back(next);
            }
        }
    }
    assert!(
        parents.contains_key(&to),
        "{}: {start:?} cannot walk to {to:?}",
        module.id
    );
    let mut path = Vec::new();
    let mut at = to;
    while let Some((previous, button)) = parents[&at] {
        path.push((at, button));
        at = previous;
    }
    for (target, button) in path.into_iter().rev() {
        for _ in 0..8 {
            if tile(c) == target {
                break;
            }
            press(c, button);
        }
        assert_eq!(map(c), module.id);
        assert_eq!(tile(c), target, "ordinary floor step uses production input");
        assert!(c.snapshot().expect("geometry fixture").battle.is_none());
    }
}

#[test]
#[ignore = "requires the published nine-cave pack; local fixtures do not earn regional access"]
fn cave_quality_controller_all_nine_cameras_stairs_drops_mouths_and_continue() {
    run_cave_quality_rooms(&[
        "MnR01Cave1F",
        "MnR03Cave1F",
        "MnR05RockCut",
        "MnR09Cave1F",
        "MnR10Cave1F",
        "MnR12Cave1F",
        "MnR13Cave1F",
        "MnR17Cave1F",
        "MnR20Cave1F",
    ]);
}

#[test]
#[ignore = "requires the current R10 cave publication; local fixture does not earn access"]
fn cave_quality_controller_r10_cameras_stairs_drops_mouth_and_continue() {
    run_cave_quality_rooms(&["MnR10Cave1F"]);
}

#[test]
#[ignore = "requires the current regional R09 publication; local fixture does not earn access"]
fn cave_quality_controller_current_r09_cameras_stairs_drops_mouth_and_continue() {
    run_cave_quality_rooms(&["MnR09Cave1F"]);
}

#[test]
#[ignore = "requires the current R05 Rock Cut publication; local fixture does not earn access"]
fn cave_quality_controller_current_r05_cameras_stair_mouths_pickup_and_continue() {
    run_cave_quality_rooms(&["MnR05RockCut"]);
}

#[test]
#[ignore = "requires the current compact Gunflint shelter publication; local fixture does not earn access"]
fn cave_quality_controller_current_r12_cameras_stairs_drops_mouth_pickup_and_continue() {
    run_cave_quality_rooms(&["MnR12Cave1F"]);
}

#[test]
#[ignore = "requires the current R13 spring shelter publication; local fixture does not earn access"]
fn cave_quality_controller_current_r13_cameras_stairs_mouth_pickup_and_continue() {
    run_cave_quality_rooms(&["MnR13Cave1F"]);
}

#[test]
#[ignore = "requires the current R17 talus den publication; local fixture does not earn access"]
fn cave_quality_controller_current_r17_cameras_mouth_pickup_and_continue() {
    run_cave_quality_rooms(&["MnR17Cave1F"]);
}

#[test]
#[ignore = "requires the current R20 two-mouth cleft publication; local fixture does not earn access"]
fn cave_quality_controller_current_r20_cameras_stairs_hops_mouths_pickup_and_continue() {
    run_cave_quality_rooms(&["MnR20Cave1F"]);
}

#[test]
#[ignore = "requires the current R01 limestone hollows publication; local fixture does not earn access"]
fn cave_quality_controller_current_r01_cameras_stairs_mouth_pickup_and_continue() {
    run_cave_quality_rooms(&["MnR01Cave1F"]);
}

fn run_cave_quality_rooms(rooms: &[&str]) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_CAVE_QUALITY_PACK").expect("actual nine-cave publication"),
    )
    .canonicalize()
    .expect("local pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("strict verified pack"),
    )
    .expect("native runtime");
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/caves/johto-quality-v01");
    for &room in rooms {
        let source: Value = serde_json::from_str(
            &std::fs::read_to_string(directory.join(format!("{room}.json")))
                .expect("explicit authored room"),
        )
        .expect("source JSON");
        let arrival = point(&source["clear_arrival"]);
        let module = &runtime.data().maps[room];
        let mut expected_blocks = source["rows"]
            .as_array()
            .expect("complete authored cave rows")
            .iter()
            .flat_map(|row| row.as_str().expect("source row").split_whitespace())
            .map(|value| u16::from_str_radix(value, 16).expect("native block"))
            .collect::<Vec<_>>();
        // The current story adds a prerequisite-gated lower shaft to the
        // reviewed public mine. Keep the original terrain contract intact;
        // certify this exact overlay rather than accepting arbitrary drift.
        let story_mine =
            room == "MnR09Cave1F" && module.scripts.contains_key("MnNDrillPublicTiles");
        if story_mine {
            assert_eq!(
                [module.attributes.width, module.attributes.height],
                [12, 11]
            );
            assert_eq!(expected_blocks[5 * 12 + 9], 2);
            expected_blocks[5 * 12 + 9] = 0x2d;
            assert_eq!(
                module.scripts["MnNDrillPublicTiles"],
                serde_json::json!([
                    {"command": "checkevent", "args": ["EVENT_MN_DEPOT_FOREMAN_DEFEATED"]},
                    {"command": "iffalse", "args": ["MnNDrillPublicEnd"]},
                    {"command": "changeblock", "args": ["18", "10", "7"]},
                    {"command": "endcallback", "args": []}
                ])
            );
            assert_eq!(module.events.warps.len(), 3);
            for (warp, at) in module.events.warps[1..].iter().zip([(19, 9), (19, 11)]) {
                assert_eq!((warp.x, warp.y), at);
                assert_eq!(warp.target_warp_id, 2);
                assert_eq!(
                    runtime
                        .data()
                        .map_name_for_constant(&warp.target_map_constant)
                        .expect("story shaft destination"),
                    "MnIronDrillApproach"
                );
            }
        }
        assert_eq!(module.blocks, expected_blocks, "{room}: published source");
        assert_eq!(
            u64::from(module.attributes.border_block),
            source["border_block"]
                .as_u64()
                .expect("solid authored border"),
            "{room}: published border"
        );
        let shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
                map_name: room.into(),
                tile_x: arrival.0,
                tile_y: arrival.1,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit local geometry fixture");
        let mut c = VisibleShellController { shell };
        complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
            .expect("fixture name");
        c.set_runtime_journal_enabled(false);
        // No position, flags, party, inventory or RNG writes after this setup.
        let initial_flags = flags(&c);
        let bodies = module
            .objects
            .iter()
            .map(|o| (o.x as i16, o.y as i16))
            .collect::<BTreeSet<_>>();
        let mut floor = BTreeSet::new();
        for y in 0..module.attributes.height as i16 * 2 {
            for x in 0..module.attributes.width as i16 * 2 {
                if kind(&runtime, module, (x, y)) == "FLOOR" && !bodies.contains(&(x, y)) {
                    floor.insert((x, y));
                }
            }
        }
        if story_mine {
            assert_eq!(kind(&runtime, module, (19, 10)), "WALL");
            assert_eq!(kind(&runtime, module, (19, 11)), "WALL");
            walk(&mut c, module, &floor, (19, 9));
            for _ in 0..3 {
                press(&mut c, GameButton::Down);
                assert_eq!(map(&mut c), room, "closed shaft cannot enter the lair");
                assert_eq!(tile(&mut c), (19, 9));
            }
            c.wait_frames(60).expect("passive story-shaft arrival");
            assert_eq!(map(&mut c), room);
            assert_eq!(tile(&mut c), (19, 9));
        }
        for view in source["review_contract"]["viewpoints"]
            .as_array()
            .expect("every camera")
        {
            walk(&mut c, module, &floor, point(&view["tile"]));
            if let Some(directory) = std::env::var_os("CRYSTAL_CAVE_QUALITY_SAVES") {
                let directory = PathBuf::from(directory);
                std::fs::create_dir_all(&directory).expect("retained cave camera checkpoints");
                let label = view["id"].as_str().expect("named cave viewpoint");
                let path = directory.join(format!("{room}-{label}.crystalsave"));
                assert!(!path.exists(), "preserve previous reached cave checkpoints");
                c.save(path).expect("save actual reached cave camera");
            }
        }
        for (index, block) in module.blocks.iter().enumerate() {
            let x = (index % usize::from(module.attributes.width)) as i16 * 2;
            let y = (index / usize::from(module.attributes.width)) as i16 * 2;
            let lane = match block {
                0x12 => Some(x + 1),
                0x30 | 0x36 => Some(x),
                _ => None,
            };
            if let Some(lane) = lane {
                let mut course = Vec::new();
                if floor.contains(&(lane, y - 1)) {
                    course.push((lane, y - 1));
                } else {
                    let side = [lane - 1, lane + 1]
                        .into_iter()
                        .find(|&side| floor.contains(&(side, y)) && floor.contains(&(side, y - 1)))
                        .expect("native stair has a clear upper approach");
                    course.extend([(side, y - 1), (side, y)]);
                }
                course.extend([(lane, y), (lane, y + 1)]);
                if floor.contains(&(lane, y + 2)) {
                    course.push((lane, y + 2));
                } else {
                    // The lower mine stair meets a bent shaft. Its lower end
                    // exits sideways onto ordinary floor before turning south.
                    let side = [lane - 1, lane + 1]
                        .into_iter()
                        .find(|&side| {
                            floor.contains(&(side, y + 1)) && floor.contains(&(side, y + 2))
                        })
                        .expect("native stair has a clear lower approach");
                    course.extend([(side, y + 1), (side, y + 2)]);
                }
                assert!(
                    course.iter().all(|p| floor.contains(p)),
                    "{room} native stair course"
                );
                for to in course.iter().chain(course.iter().rev()) {
                    walk(&mut c, module, &floor, *to);
                }
            }
        }
        for y in 1..module.attributes.height as i16 * 2 - 1 {
            for x in 1..module.attributes.width as i16 * 2 - 1 {
                let (dx, dy, button, reverse) = match kind(&runtime, module, (x, y)) {
                    "HOP_DOWN" => (0, 1, GameButton::Down, GameButton::Up),
                    "HOP_LEFT" => (-1, 0, GameButton::Left, GameButton::Right),
                    "HOP_RIGHT" => (1, 0, GameButton::Right, GameButton::Left),
                    _ => continue,
                };
                let approach = (x - dx, y - dy);
                let landing = (x + dx * 2, y + dy * 2);
                walk(&mut c, module, &floor, approach);
                for _ in 0..8 {
                    if tile(&mut c) == landing {
                        break;
                    }
                    press(&mut c, button);
                    assert_ne!(
                        tile(&mut c),
                        (x + dx, y + dy),
                        "never stand on the wall course"
                    );
                }
                assert_eq!(tile(&mut c), landing, "{room} actual native hop");
                for _ in 0..3 {
                    press(&mut c, reverse);
                    assert_eq!(tile(&mut c), landing);
                }
                walk(&mut c, module, &floor, approach);
            }
        }
        assert_eq!(
            flags(&c),
            initial_flags,
            "geometry grants no quest progress"
        );
        for pickup in source["items"]
            .as_array()
            .expect("retained item discoveries")
        {
            let actor = point(&pickup["tile"]);
            let approach = neighbors(actor)
                .into_iter()
                .map(|(at, _)| at)
                .find(|at| floor.contains(at))
                .expect("reachable pickup approach");
            walk(&mut c, module, &floor, approach);
            let button = neighbors(approach)
                .into_iter()
                .find_map(|(at, button)| (at == actor).then_some(button))
                .expect("face original item");
            let item = &runtime.data().items[pickup["item"].as_str().expect("retained item")];
            let before = c.shell.shell.session().state().bag.quantity(item);
            press(&mut c, button);
            assert_eq!(
                tile(&mut c),
                approach,
                "uncollected item occupies its native tile"
            );
            press(&mut c, GameButton::A);
            assert!(
                c.shell.visible_field_item_notice.is_some()
                    || c.snapshot()
                        .expect("actual pickup notice")
                        .ui
                        .text_window_open
            );
            press(&mut c, GameButton::Down);
            assert_eq!(tile(&mut c), approach, "pickup presentation owns movement");
            settle(&mut c, "original cave discovery");
            let quantity = pickup["quantity"].as_u64().expect("native quantity") as u16;
            assert_eq!(
                c.shell.shell.session().state().bag.quantity(item),
                before + quantity
            );
            assert!(
                flags(&c)
                    .0
                    .contains(pickup["flag"].as_str().expect("retained event"))
            );
            press(&mut c, button);
            // The original item body is gone: facing its old tile is legal
            // movement, and cannot grant another copy of the reward.
            assert_eq!(
                c.shell.shell.session().state().bag.quantity(item),
                before + quantity
            );
        }
        walk(&mut c, module, &floor, arrival);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        for warp in &module.events.warps {
            if story_mine && warp.index > 1 {
                // Both story-shaft records are certified above. Opening and
                // traversing that shaft requires the separate battle chapter;
                // this empty-party geometry fixture verifies its closed state.
                continue;
            }
            let exterior_name = runtime
                .data()
                .map_name_for_constant(&warp.target_map_constant)
                .expect("retained exterior map identity");
            let clear = (warp.x as i16, warp.y as i16 - 1);
            walk(&mut c, module, &floor, clear);
            for _ in 0..8 {
                if map(&mut c) != room {
                    break;
                }
                press(&mut c, GameButton::Down);
            }
            assert_eq!(
                map(&mut c),
                exterior_name,
                "{room} original exit {}",
                warp.index
            );
            c.wait_frames(60).expect("settle exterior arrival");
            let exterior = &runtime.data().maps[&exterior_name];
            let active =
                exterior
                    .events
                    .warps
                    .iter()
                    .find(|w| {
                        runtime
                            .data()
                            .map_name_for_constant(&w.target_map_constant)
                            .expect("retained indoor map identity")
                            == room
                            && w.target_warp_id
                                == i16::try_from(warp.index).expect("bounded mouth index")
                            && runtime.data().tilesets[&exterior.attributes.tileset_name].collision
                                [&format!(
                                    "{:02x}",
                                    exterior.blocks[usize::from(w.y / 2)
                                        * usize::from(exterior.attributes.width)
                                        + usize::from(w.x / 2)]
                                )][usize::from(w.y % 2) * 2 + usize::from(w.x % 2)]
                                == "CAVE"
                    })
                    .expect("same-mouth active exterior entrance");
            let passive = (active.x as i16, active.y as i16 + 1);
            if tile(&mut c) != passive {
                for _ in 0..4 {
                    if tile(&mut c) == passive {
                        break;
                    }
                    press(&mut c, GameButton::Down);
                }
            }
            assert_eq!(tile(&mut c), passive, "safe exterior return");
            c.wait_frames(60).expect("passive floor cannot reenter");
            assert_eq!(map(&mut c), exterior_name);
            for _ in 0..8 {
                if map(&mut c) == room {
                    break;
                }
                press(&mut c, GameButton::Up);
            }
            assert_eq!(map(&mut c), room, "same exterior mouth reenters");
            for _ in 0..4 {
                if tile(&mut c) == clear {
                    break;
                }
                press(&mut c, GameButton::Up);
            }
            assert_eq!(tile(&mut c), clear, "correct indoor mouth pairing");
        }
    }
}
