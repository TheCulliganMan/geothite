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
                && matches!(
                    m.attributes.environment.as_deref(),
                    Some("TOWN" | "ROUTE" | "OUTDOOR")
                )
        })
        .collect::<Vec<_>>();
    let indices = maps
        .iter()
        .enumerate()
        .map(|(i, m)| (m.id.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    let native = maps
        .iter()
        .map(|m| OverworldMapData::from_attributes(&m.id, &m.attributes, m.blocks.clone()))
        .collect::<Vec<_>>();
    let open = maps.iter().map(|m| floor(runtime, m)).collect::<Vec<_>>();
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
        for (dx, dy, button) in [
            (-1, 0, GameButton::Left),
            (1, 0, GameButton::Right),
            (0, -1, GameButton::Up),
            (0, 1, GameButton::Down),
        ] {
            let (x, y) = (p.1 + dx, p.2 + dy);
            let mut q = (p.0, x, y);
            let map = maps[p.0];
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
    assert!(
        seen.contains(&target),
        "no existing dry regional course from {} {:?} to {destination} {tile:?}",
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
            c.wait_frames(80)
                .expect("native ice and gate movement clock");
            let actual = c.snapshot().expect("actual regional step").overworld;
            if actual.map_name == maps[q.0].id && (actual.tile.x, actual.tile.y) == (q.1, q.2) {
                break;
            }
            if actual.map_name != before.map_name || actual.tile != before.tile {
                // Regional wardens can apply an inward two-step movement.
                // Replan from the authoritative result instead of moving state.
                walk_from(c, runtime, destination, tile, remaining_replans - 1);
                return;
            }
        }
        let actual = c.snapshot().expect("settled regional step").overworld;
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

fn floor(runtime: &CrystalRuntime, map: &MapModule) -> BTreeSet<(i16, i16)> {
    let bodies = map
        .objects
        .iter()
        .filter(|o| {
            // These authored coordinate programs remove their own wardens.
            // The journey has explicit badge/report fixtures; their actual
            // scripts still run and can reject input if those are insufficient.
            !(o.script.starts_with("MnCohesion")
                || matches!(
                    o.script.as_str(),
                    "MnBemidjiIronCheck" | "MnBemidjiItascaCheck"
                ))
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
            if matches!(
                kind.as_str(),
                "FLOOR" | "TALL_GRASS" | "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK" | "ICE" | "ICE_2B"
            ) && !bodies.contains(&(x, y))
            {
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
