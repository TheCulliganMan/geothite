//! Initial local placements are fixtures; following, yielding and Continue use input.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::interior_life::{fixture, load};

type Point = (i16, i16);
const CITY: &str = "MnMinneapolis";

fn cycle(start: Point, width: i16, height: i16) -> Vec<Point> {
    let (x, y) = start;
    (x..x + width)
        .map(|px| (px, y))
        .chain((y..y + height).map(|py| (x + width, py)))
        .chain((x + 1..=x + width).rev().map(|px| (px, y + height)))
        .chain((y + 1..=y + height).rev().map(|py| (x, py)))
        .collect()
}

fn tile(c: &mut VisibleShellController) -> Point {
    let snapshot = c.snapshot().expect("native cycling view");
    assert_eq!(snapshot.overworld.map_name, CITY);
    assert!(snapshot.battle.is_none() && !snapshot.ui.text_window_open);
    assert!(c.shell.last_error.is_none());
    (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
}

fn rider(c: &VisibleShellController, id: &str) -> Point {
    let p = c
        .shell
        .shell
        .session()
        .overworld()
        .object_runtime_tile_by_id(id)
        .expect("authored city rider");
    (p.x, p.y)
}

fn walkable(runtime: &CrystalRuntime) -> BTreeSet<Point> {
    let map = &runtime.data().maps[CITY];
    let collision = &runtime.data().tilesets[&map.attributes.tileset_name].collision;
    let blocked: BTreeSet<_> = map
        .objects
        .iter()
        .filter(|o| !o.script.starts_with("MN_EXTRA_MPLS_CYCLE_"))
        .map(|o| (o.x as i16, o.y as i16))
        .chain(map.events.warps.iter().map(|e| (e.x as i16, e.y as i16)))
        .chain(
            map.events
                .coord_events
                .iter()
                .map(|e| (e.x as i16, e.y as i16)),
        )
        .collect();
    let mut floor = BTreeSet::new();
    for y in 1..63 {
        for x in 1..79 {
            let block = map.blocks[(y / 2 * 40 + x / 2) as usize];
            let kind = collision[&format!("{block:02x}")][(y % 2 * 2 + x % 2) as usize].as_str();
            if matches!(kind, "FLOOR" | "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK")
                && !blocked.contains(&(x, y))
            {
                floor.insert((x, y));
            }
        }
    }
    floor
}

fn follow_step(
    c: &mut VisibleShellController,
    floor: &BTreeSet<Point>,
    track: &[Point],
    index: usize,
) {
    let player = tile(c);
    let at = track[index];
    let upcoming: BTreeSet<_> = (0..=3).map(|i| track[(index + i) % track.len()]).collect();
    let distance = |a: Point, b: Point| (a.0 - b.0).abs() + (a.1 - b.1).abs();
    if distance(player, at) <= 3 && !upcoming.contains(&player) {
        c.wait_frames(1).expect("native bicycle stride");
        return;
    }
    let overworld = c.shell.shell.session().overworld();
    let mut occupied: BTreeSet<_> = overworld
        .objects
        .iter()
        .filter_map(|o| {
            o.object_identifier
                .as_deref()
                .and_then(|id| overworld.object_runtime_tile_by_id(id).ok())
                .map(|p| (p.x, p.y))
        })
        .collect();
    occupied.extend(upcoming);
    let mut prev = BTreeMap::from([(player, None)]);
    let mut queue = VecDeque::from([player]);
    let mut target = None;
    while let Some(p) = queue.pop_front() {
        if p != player && distance(p, at) <= 2 {
            target = Some(p);
            break;
        }
        for q in [
            (p.0 - 1, p.1),
            (p.0 + 1, p.1),
            (p.0, p.1 - 1),
            (p.0, p.1 + 1),
        ] {
            if floor.contains(&q) && !occupied.contains(&q) && !prev.contains_key(&q) {
                prev.insert(q, Some(p));
                queue.push_back(q);
            }
        }
    }
    let Some(mut next) = target else {
        c.wait_frames(1).expect("yield for a passing rider");
        return;
    };
    while prev[&next] != Some(player) {
        next = prev[&next].expect("follow path predecessor");
    }
    let button = match (next.0 - player.0, next.1 - player.1) {
        (-1, 0) => GameButton::Left,
        (1, 0) => GameButton::Right,
        (0, -1) => GameButton::Up,
        (0, 1) => GameButton::Down,
        other => panic!("nonadjacent cycling observer input {other:?}"),
    };
    c.press(button).expect("walk alongside the real cyclist");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with citywide Minneapolis cycling and CRYSTAL_CYCLING_REVIEW_DIR"]
fn minneapolis_citywide_cyclists_complete_distinct_routes_and_continue() {
    let (root, runtime, _) = load();
    let output = PathBuf::from(
        std::env::var_os("CRYSTAL_CYCLING_REVIEW_DIR")
            .expect("fresh ignored cycling evidence directory"),
    );
    std::fs::create_dir_all(&output).expect("cycling evidence directory");
    let floor = walkable(&runtime);
    let map = &runtime.data().maps[CITY];
    let mut results = Vec::new();
    for number in 1..=18 {
        let id = format!("MN_EXTRA_MPLS_CYCLE_{number:02}");
        let object = map
            .objects
            .iter()
            .find(|o| o.script == id)
            .expect("existing rider");
        let track = cycle(
            (object.x as i16, object.y as i16),
            object.move_range_x as i16,
            object.move_range_y as i16,
        );
        // Every fixture begins on ordinary floor near this rider; later camera
        // movement is real player input, including bridge crossings and yielding.
        let initial = floor
            .iter()
            .copied()
            .filter(|p| {
                let b = map.blocks[(p.1 / 2 * 40 + p.0 / 2) as usize];
                runtime.data().tilesets[&map.attributes.tileset_name].collision[&format!("{b:02x}")]
                    [(p.1 % 2 * 2 + p.0 % 2) as usize]
                    == "FLOOR"
                    && !map.objects.iter().any(|o| (o.x as i16, o.y as i16) == *p)
            })
            .min_by_key(|p| (p.0 - track[0].0).abs() + (p.1 - track[0].1).abs())
            .expect("safe initial rider approach");
        let mut c = fixture(&root, &runtime, CITY, initial);
        let mut visited = BTreeSet::new();
        let mut travel = 0;
        let mut previous = 0;
        let mut continued = false;
        let mut observations = Vec::new();
        for tick in 0..12000 {
            let p = rider(&c, &id);
            let index = track
                .iter()
                .position(|q| *q == p)
                .expect("rider stays on authored street");
            let advanced = (index + track.len() - previous) % track.len();
            assert!(
                advanced <= 3,
                "native bicycle may stride, never teleport: {id}"
            );
            travel += advanced;
            previous = index;
            visited.insert(p);
            if advanced > 0 {
                observations.push(serde_json::json!({"tick":tick,"rider":p,"player":tile(&mut c)}));
            }
            if !continued && travel >= track.len() + track.len() / 3 {
                let path = output.join(format!("{number:02}-actual-reached.crystalsave"));
                assert!(!path.exists(), "retain previous reached saves");
                let player_before = tile(&mut c);
                assert_ne!(p, track[0], "save away from the authored start");
                c.save(&path).expect("save actual reached cycling state");
                c = VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
                    .expect("production Continue while rider is mid-route");
                c.set_runtime_journal_enabled(false);
                assert_eq!(
                    rider(&c, &id),
                    p,
                    "Continue retains actual cyclist position"
                );
                assert_eq!(
                    tile(&mut c),
                    player_before,
                    "Continue retains observer position"
                );
                continued = true;
            }
            if travel >= track.len() * 2 && visited.len() == track.len() {
                break;
            }
            assert!(
                tick < 11999,
                "rider failed full native laps: {id}; {travel} steps; {} of {} tiles; player {:?}; rider {p:?}",
                visited.len(),
                track.len(),
                tile(&mut c)
            );
            follow_step(&mut c, &floor, &track, index);
        }
        assert!(continued);
        c.press(GameButton::Start)
            .expect("ordinary Start after cycling tour");
        assert!(c.snapshot().expect("Start").ui.menu.is_some());
        c.press(GameButton::B).expect("return to cycling street");
        if [2, 4, 9, 10, 13, 14, 15, 18].contains(&number) {
            let mut app = super::menu_render_test_app(c.shell);
            app.update();
            let world = app.world_mut();
            let images = world
                .remove_resource::<bevy::prelude::Assets<bevy::prelude::Image>>()
                .expect("native CPU source art");
            super::pokemon_overworld_context_canvas(world, &images)
                .save(output.join(format!("{number:02}-native.png")))
                .expect("retain native cycling view");
        }
        results.push(
            serde_json::json!({"id":id,"lap_tiles":track.len(),"visited_tiles":visited,
            "actual_forward_steps":travel,"Continue":continued,"observations":observations}),
        );
        std::fs::write(
            output.join("progress.json"),
            serde_json::to_vec_pretty(&results).expect("cycling progress JSON"),
        )
        .expect("retain native route observations");
    }
    assert_eq!(results.len(), 18);
}
