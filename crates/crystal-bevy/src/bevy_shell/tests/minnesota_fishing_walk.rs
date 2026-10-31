//! Live dry-bank walking includes the original grass and bridge approaches.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::VisibleShellController;

pub(super) fn path(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: (i16, i16),
) -> Option<Vec<GameButton>> {
    let snapshot = c.snapshot().expect("live fishing-bank approach");
    let start = (snapshot.overworld.tile.x, snapshot.overworld.tile.y);
    let map = &r.data().maps[&snapshot.overworld.map_name];
    let live = &c.shell.shell.session().overworld().map;
    let occupied = snapshot
        .visible_objects
        .iter()
        .map(|object| {
            let tile = c
                .shell
                .shell
                .session()
                .overworld()
                .object_runtime_tile_by_id(
                    object
                        .object_identifier
                        .as_deref()
                        .expect("native bank actor identity"),
                )
                .expect("live bank actor position");
            (tile.x, tile.y)
        })
        .collect::<BTreeSet<_>>();
    let mut previous = BTreeMap::new();
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        if at == target {
            let mut cursor = target;
            let mut route = Vec::new();
            while cursor != start {
                let (before, button) = previous[&cursor];
                route.push(button);
                cursor = before;
            }
            route.reverse();
            return Some(route);
        }
        for (next, button) in [
            ((at.0 - 1, at.1), GameButton::Left),
            ((at.0 + 1, at.1), GameButton::Right),
            ((at.0, at.1 - 1), GameButton::Up),
            ((at.0, at.1 + 1), GameButton::Down),
        ] {
            let (x, y) = next;
            if x < 0
                || y < 0
                || x >= live.width as i16 * 2
                || y >= live.height as i16 * 2
                || seen.contains(&next)
                || occupied.contains(&next)
            {
                continue;
            }
            let block = live.metatile_ids[y as usize / 2 * live.width as usize + x as usize / 2];
            let kind = &r.data().tilesets[&map.attributes.tileset_name].collision
                [&format!("{block:02x}")][y as usize % 2 * 2 + x as usize % 2];
            if !matches!(
                kind.as_str(),
                "FLOOR" | "TALL_GRASS" | "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK"
            ) {
                continue;
            }
            seen.insert(next);
            previous.insert(next, (at, button));
            queue.push_back(next);
        }
    }
    None
}
