//! Plan existing dry regional terrain; every step is production controller input.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crystal_assets::MapModule;
use crystal_core::input::GameButton;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::super::r10_gallery::press;

type Node = (usize, i16, i16);

pub(super) fn walk(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    destination: &str,
    tile: (i16, i16),
) {
    walk_from(c, runtime, destination, tile, 32);
}

fn walk_from(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    destination: &str,
    tile: (i16, i16),
    remaining_replans: usize,
) {
    if remaining_replans == 0
        && let Some(out) = std::env::var_os("CRYSTAL_FALLS_WINDBREAK_REVIEW_DIR")
    {
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).expect("regional replan diagnostic directory");
        let path = out.join("actual-regional-replan-limit.crystalsave");
        assert!(!path.exists(), "retain reached regional replan diagnostics");
        c.save(path)
            .expect("actual reached regional replan checkpoint");
    }
    assert!(
        remaining_replans > 0,
        "regional coordinate events must settle"
    );
    let maps = runtime
        .data()
        .maps
        .values()
        .filter(|m| {
            m.id.starts_with("Mn")
                && (m.id == "MnGrandMaraisShoreGate"
                    || matches!(
                        m.attributes.environment.as_deref(),
                        Some("TOWN" | "ROUTE" | "OUTDOOR")
                    ))
        })
        .collect::<Vec<_>>();
    let mut indices = BTreeMap::new();
    for (index, map) in maps.iter().enumerate() {
        indices.insert(map.id.as_str(), index);
        if let Some(constant) = map.attributes.map_constant.as_deref() {
            indices.insert(constant, index);
        }
    }
    let native = maps
        .iter()
        .map(|m| OverworldMapData::from_attributes(&m.id, &m.attributes, m.blocks.clone()))
        .collect::<Vec<_>>();
    let open = maps
        .iter()
        .map(|m| floor(runtime, m, c))
        .collect::<Vec<_>>();
    let ice = maps.iter().map(|m| ice(runtime, m)).collect::<Vec<_>>();
    let origin = c.snapshot().expect("regional journey origin").overworld;
    let start: Node = (
        indices[origin.map_name.as_str()],
        origin.tile.x,
        origin.tile.y,
    );
    let target: Node = (indices[destination], tile.0, tile.1);
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    let mut previous = BTreeMap::new();
    while let Some(p) = queue.pop_front() {
        if p == target {
            break;
        }
        let map = maps[p.0];
        let standing_block =
            map.blocks[p.2 as usize / 2 * usize::from(map.attributes.width) + p.1 as usize / 2];
        let standing_kind = runtime.data().tilesets[&map.attributes.tileset_name].collision
            [&format!("{standing_block:02x}")][p.2 as usize % 2 * 2 + p.1 as usize % 2]
            .as_str();
        for (dx, dy, button) in [
            (-1, 0, GameButton::Left),
            (1, 0, GameButton::Right),
            (0, -1, GameButton::Up),
            (0, 1, GameButton::Down),
        ] {
            // Directional carpets activate when leaving the carpet, after
            // the player has stepped onto it. The gate's passive arrival
            // markers are ordinary floor and never become synthetic warps.
            let leaves_carpet = matches!(
                (standing_kind, dx, dy),
                ("WARP_CARPET_LEFT", -1, 0)
                    | ("WARP_CARPET_RIGHT", 1, 0)
                    | ("WARP_CARPET_UP", 0, -1)
                    | ("WARP_CARPET_DOWN", 0, 1)
            );
            if leaves_carpet
                && let Some(warp) = map
                    .events
                    .warps
                    .iter()
                    .find(|warp| warp.x as i16 == p.1 && warp.y as i16 == p.2)
            {
                let Some(&index) = indices.get(warp.target_map.as_str()) else {
                    continue;
                };
                let landing = maps[index]
                    .events
                    .warps
                    .iter()
                    .find(|landing| i32::from(landing.index) == i32::from(warp.target_warp_id))
                    .expect("authored gate arrival marker");
                let q = (index, landing.x as i16, landing.y as i16);
                if open[q.0].contains(&(q.1, q.2)) && seen.insert(q) {
                    previous.insert(q, (p, button));
                    queue.push_back(q);
                }
                continue;
            }
            let (x, y) = (p.1 + dx, p.2 + dy);
            let mut q = (p.0, x, y);
            if x < 0
                || y < 0
                || x >= map.attributes.width as i16 * 2
                || y >= map.attributes.height as i16 * 2
            {
                let Some(transition) =
                    connection_for_tile_checked(&native[p.0], TilePosition::new(x, y))
                        .expect("native regional projection")
                else {
                    continue;
                };
                let next = runtime
                    .data()
                    .resolve_connection_transition(&transition)
                    .expect("actual regional destination")
                    .destination;
                let Some(&index) = indices.get(next.map_name.as_str()) else {
                    continue;
                };
                q = (index, next.tile.x, next.tile.y);
            }
            let next = maps[q.0];
            if let Some(event) = next.events.coord_events.iter().find(|event| {
                event.x as i16 == q.1
                    && event.y as i16 == q.2
                    && event.script_name.ends_with("Inward")
            }) {
                let body = next.scripts[&event.script_name]
                    .as_array()
                    .expect("native inward checkpoint program");
                let movement = body.iter().find_map(|row| {
                    (row["command"] == "applymovement" && row["args"][0] == "PLAYER")
                        .then(|| row["args"][1].as_str().expect("native inward movement"))
                });
                if let Some(movement) = movement {
                    let steps = next.scripts[movement]
                        .as_array()
                        .expect("native inward movement steps");
                    let facing = match (dx, dy) {
                        (-1, 0) => "LEFT",
                        (1, 0) => "RIGHT",
                        (0, -1) => "UP",
                        (0, 1) => "DOWN",
                        _ => unreachable!("regional inputs are cardinal"),
                    };
                    if steps.len() == 3
                        && steps[2]["command"] == "step_end"
                        && steps[..2]
                            .iter()
                            .all(|step| step["command"] == "step" && step["args"][0] == facing)
                    {
                        // Native inward programs remove their own wardens and
                        // move an arrival two tiles. Read that actual landing;
                        // planning never removes a live actor or grants reverse
                        // permission through the checkpoint.
                        q.1 += dx * 2;
                        q.2 += dy * 2;
                    }
                }
            }
            let block = next.blocks
                [q.2 as usize / 2 * usize::from(next.attributes.width) + q.1 as usize / 2];
            let kind = runtime.data().tilesets[&next.attributes.tileset_name].collision
                [&format!("{block:02x}")][q.2 as usize % 2 * 2 + q.1 as usize % 2]
                .as_str();
            if matches!(kind, "HOP_DOWN" | "HOP_LEFT" | "HOP_RIGHT") {
                let direction = match kind {
                    "HOP_DOWN" => (0, 1),
                    "HOP_LEFT" => (-1, 0),
                    "HOP_RIGHT" => (1, 0),
                    _ => unreachable!("only native ledge directions enter this branch"),
                };
                if (dx, dy) != direction {
                    continue;
                }
                // One native press crosses the ledge and lands beyond it.
                // The reverse edge stays closed; no state is moved by planning.
                q.1 += dx;
                q.2 += dy;
            }
            if !open[q.0].contains(&(q.1, q.2)) {
                continue;
            }
            // Plan native sliding to its dry landing or blocked final ice tile.
            while ice[q.0].contains(&(q.1, q.2)) && open[q.0].contains(&(q.1 + dx, q.2 + dy)) {
                q.1 += dx;
                q.2 += dy;
            }
            if seen.insert(q) {
                previous.insert(q, (p, button));
                queue.push_back(q);
            }
        }
    }
    let reached_maps = seen
        .iter()
        .map(|p| maps[p.0].id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        seen.contains(&target),
        "no existing dry regional course from {} {:?} to {destination} {tile:?}; reached maps {reached_maps:?}",
        origin.map_name,
        (origin.tile.x, origin.tile.y)
    );
    let mut course = Vec::new();
    let mut p = target;
    while p != start {
        let (last, button) = previous[&p];
        course.push((p, button));
        p = last;
    }
    for (q, button) in course.into_iter().rev() {
        repel(c, runtime);
        let before = c.snapshot().expect("regional input origin").overworld;
        for _ in 0..8 {
            press(c, button);
            let settled = c.snapshot().expect("native regional input result");
            if settled.overworld.map_name == maps[q.0].id
                && (settled.overworld.tile.x, settled.overworld.tile.y) == (q.1, q.2)
                && settled.battle.is_none()
                && !settled.ui.text_window_open
                && settled.ui.menu.is_none()
                && !c.has_pending_script_work()
            {
                break;
            }
            c.wait_frames(80)
                .expect("native ice and gate movement clock");
            if c.snapshot().expect("regional encounter").battle.is_some() {
                super::super::rochester::flee(c);
            }
            let scene = c.snapshot().expect("native gate passage");
            if scene.overworld.map_name == "MnGrandMaraisShoreGate" && scene.ui.text_window_open {
                assert!(
                    scene.ui.pending_yes_no.is_none(),
                    "gate passage has no implicit choices"
                );
                super::super::winona::finish(c);
            }
            let actual = c.snapshot().expect("actual regional step").overworld;
            if actual.map_name == maps[q.0].id && (actual.tile.x, actual.tile.y) == (q.1, q.2) {
                break;
            }
            if actual.map_name != before.map_name || actual.tile != before.tile {
                // Regional wardens can apply an inward two-step movement.
                // Replan from the authoritative result instead of moving state.
                eprintln!(
                    "regional replan {remaining_replans}: expected {} ({},{}), actual {} ({},{}), destination {destination} {tile:?}",
                    maps[q.0].id, q.1, q.2, actual.map_name, actual.tile.x, actual.tile.y
                );
                walk_from(c, runtime, destination, tile, remaining_replans - 1);
                return;
            }
        }
        let actual = c.snapshot().expect("settled regional step").overworld;
        if (actual.map_name.as_str(), actual.tile.x, actual.tile.y)
            != (maps[q.0].id.as_str(), q.1, q.2)
        {
            let scene = c.snapshot().expect("blocked regional step diagnostic");
            eprintln!(
                "blocked regional step: phase={:?} battle={:?} text={:?} objects={:?}",
                scene.phase,
                scene.battle.as_ref().map(|battle| &battle.kind),
                scene.ui.text.as_ref().map(|text| &text.label),
                scene.visible_objects
            );
            if let Some(out) = std::env::var_os("CRYSTAL_FALLS_WINDBREAK_REVIEW_DIR") {
                let out = std::path::PathBuf::from(out);
                std::fs::create_dir_all(&out).expect("regional diagnostic directory");
                let path = out.join("actual-blocked-regional-step.crystalsave");
                assert!(!path.exists(), "retain earlier reached diagnostics");
                c.save(path)
                    .expect("actual reached regional diagnostic save");
            }
        }
        assert_eq!(
            (actual.map_name.as_str(), actual.tile.x, actual.tile.y),
            (maps[q.0].id.as_str(), q.1, q.2),
            "regional course remains authoritative"
        );
    }
    repel(c, runtime);
}

pub(super) fn repel(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    if c.shell.shell.session().state().repel_steps_remaining == 0
        && c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["MAX_REPEL"])
            > 0
    {
        crate::bevy_shell::tests::minneapolis_falls::grand_marais_harbor::use_max_repel(c, runtime);
    }
}

fn floor(
    runtime: &CrystalRuntime,
    map: &MapModule,
    c: &VisibleShellController,
) -> BTreeSet<(i16, i16)> {
    let bodies = map
        .objects
        .iter()
        .filter(|o| {
            // Model only the simple native checkpoint contract: all declared
            // flag checks must already pass before its wardens can disappear.
            // Actual coordinate programs still run under the controller. This
            // neither removes an actor nor grants a flag in the live session.
            let Some(body) = map.scripts.get(&o.script).and_then(|body| body.as_array()) else {
                return true;
            };
            let simple = body.iter().all(|command| {
                matches!(
                    command["command"].as_str(),
                    Some("checkflag" | "checkevent" | "iffalse" | "disappear" | "end")
                )
            });
            let checks = body
                .iter()
                .filter(|command| {
                    matches!(
                        command["command"].as_str(),
                        Some("checkflag" | "checkevent")
                    )
                })
                .collect::<Vec<_>>();
            let opens = simple
                && !checks.is_empty()
                && body.iter().any(|command| command["command"] == "disappear")
                && checks.iter().all(|command| {
                    let name = command["args"][0].as_str().expect("native checkpoint flag");
                    let flags = &c.shell.shell.session().state().flags;
                    if command["command"] == "checkflag" {
                        flags.is_engine_flag_set(name).expect("native engine gate")
                    } else {
                        flags.is_event_flag_set(name).expect("native event gate")
                    }
                });
            !opens
        })
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let collisions = &runtime.data().tilesets[&map.attributes.tileset_name].collision;
    let mut result = BTreeSet::new();
    for y in 0..map.attributes.height as i16 * 2 {
        for x in 0..map.attributes.width as i16 * 2 {
            let block =
                map.blocks[y as usize / 2 * usize::from(map.attributes.width) + x as usize / 2];
            let kind = &collisions[&format!("{block:02x}")][y as usize % 2 * 2 + x as usize % 2];
            let ordinary = matches!(
                kind.as_str(),
                "FLOOR"
                    | "TALL_GRASS"
                    | "BRIDGE_NS_DECK"
                    | "BRIDGE_EW_DECK"
                    | "ICE"
                    | "ICE_2B"
                    | "LADDER"
            );
            // A carpet is a standing tile; its directional exit is planned
            // separately. Actual building doors remain excluded.
            let carpet = matches!(
                kind.as_str(),
                "WARP_CARPET_LEFT" | "WARP_CARPET_RIGHT" | "WARP_CARPET_UP" | "WARP_CARPET_DOWN"
            );
            if (ordinary || carpet) && !bodies.contains(&(x, y)) {
                result.insert((x, y));
            }
        }
    }
    result
}

fn ice(runtime: &CrystalRuntime, map: &MapModule) -> BTreeSet<(i16, i16)> {
    let collisions = &runtime.data().tilesets[&map.attributes.tileset_name].collision;
    let mut result = BTreeSet::new();
    for y in 0..map.attributes.height as i16 * 2 {
        for x in 0..map.attributes.width as i16 * 2 {
            let block =
                map.blocks[y as usize / 2 * usize::from(map.attributes.width) + x as usize / 2];
            if matches!(
                collisions[&format!("{block:02x}")][y as usize % 2 * 2 + x as usize % 2].as_str(),
                "ICE" | "ICE_2B"
            ) {
                result.insert((x, y));
            }
        }
    }
    result
}
