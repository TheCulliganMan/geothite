//! Whole-section collision, original-gate and explicit ICE-course validation.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::winter_sections_design::{Course, Route};

type Point = [u16; 2];
const DIRECTIONS: [[i16; 2]; 4] = [[0, 1], [0, -1], [1, 0], [-1, 0]];

fn step(point: Point, direction: [i16; 2]) -> Option<Point> {
    Some([
        point[0].checked_add_signed(direction[0])?,
        point[1].checked_add_signed(direction[1])?,
    ])
}

fn kind<'a>(
    pack: &'a CompiledGamePack,
    route: &Route,
    blocks: &[u16],
    alias: &str,
    point: Point,
) -> Result<&'a str> {
    let [width, height] = route.dimensions_metatiles;
    if point[0] >= width * 2 || point[1] >= height * 2 {
        return Ok("OUTSIDE");
    }
    let index = usize::from(point[1] / 2) * usize::from(width) + usize::from(point[0] / 2);
    let block = blocks
        .get(index)
        .context("winter whole block outside source")?;
    let collision = pack
        .data
        .tilesets
        .get(alias)
        .context("winter collision alias")?
        .collision
        .get(&format!("{block:02x}"))
        .context("winter native block collision")?;
    Ok(collision[usize::from(point[1] % 2 * 2 + point[0] % 2)].as_str())
}

fn dry(token: &str) -> bool {
    matches!(token, "FLOOR" | "TALL_GRASS" | "ICE")
}

fn distance(start: Point, end: Point, allowed: &BTreeSet<Point>) -> Option<usize> {
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([(start, 0)]);
    while let Some((at, length)) = queue.pop_front() {
        if at == end {
            return Some(length);
        }
        for next in DIRECTIONS
            .into_iter()
            .filter_map(|direction| step(at, direction))
        {
            if allowed.contains(&next) && seen.insert(next) {
                queue.push_back((next, length + 1));
            }
        }
    }
    None
}

fn trace(
    pack: &CompiledGamePack,
    route: &Route,
    blocks: &[u16],
    start: Point,
    direction: [i16; 2],
) -> Result<(Vec<Point>, Point)> {
    ensure!(DIRECTIONS.contains(&direction), "non-cardinal winter input");
    let mut at = step(start, direction).context("winter entry underflow")?;
    let mut path = Vec::new();
    while kind(pack, route, blocks, &route.new_tileset, at)? == "ICE" {
        path.push(at);
        ensure!(
            path.len()
                <= usize::from(route.dimensions_metatiles[0].max(route.dimensions_metatiles[1]))
                    * 2,
            "winter slide exceeds whole extent"
        );
        at = step(at, direction).context("winter slide leaves world coordinate range")?;
    }
    Ok((path, at))
}

fn course(
    pack: &CompiledGamePack,
    route: &Route,
    blocks: &[u16],
    bodies: &BTreeSet<Point>,
    course: &Course,
) -> Result<()> {
    ensure!(
        matches!(
            kind(pack, route, blocks, &route.new_tileset, course.from_core)?,
            "FLOOR" | "TALL_GRASS"
        ) && !bodies.contains(&course.from_core),
        "winter course needs an unoccupied ordinary approach"
    );
    let (path, end) = trace(pack, route, blocks, course.from_core, course.direction)?;
    ensure!(
        !path.is_empty()
            && path == course.ice_path_core
            && path.iter().all(|p| !bodies.contains(p)),
        "frozen native ICE course changed or meets actor"
    );
    match course.stop_kind.as_str() {
        "dry_or_grass_stop" => ensure!(
            end == course.stop_core
                && course.blocked_target_core.is_none()
                && course.ordinary_turn_exits.is_empty()
                && !bodies.contains(&end)
                && matches!(
                    kind(pack, route, blocks, &route.new_tileset, end)?,
                    "FLOOR" | "TALL_GRASS"
                ),
            "winter course lost its exact dry/grass stop"
        ),
        "intentional_wall_stop_on_ICE" => {
            ensure!(
                path.last() == Some(&course.stop_core)
                    && course.blocked_target_core == Some(end)
                    && matches!(
                        kind(pack, route, blocks, &route.new_tileset, end)?,
                        "WALL" | "HEADBUTT_TREE"
                    )
                    && !course.ordinary_turn_exits.is_empty(),
                "winter ICE wall stop lacks permanent guard/turn"
            );
            let mut turns = BTreeSet::new();
            for turn in &course.ordinary_turn_exits {
                let (path, end) = trace(pack, route, blocks, course.stop_core, turn.direction)?;
                ensure!(
                    turns.insert(turn.direction)
                        && path == turn.ice_path_core
                        && end == turn.dry_stop_core
                        && !bodies.contains(&end)
                        && path.iter().all(|p| !bodies.contains(p))
                        && matches!(
                            kind(pack, route, blocks, &route.new_tileset, end)?,
                            "FLOOR" | "TALL_GRASS"
                        ),
                    "winter wall-stop ordinary turn/return changed"
                );
            }
        }
        _ => anyhow::bail!("unknown winter resting-stop contract"),
    }
    Ok(())
}

/// Uses a conservative free-turn ICE graph for original Cut/bridge barriers.
/// This cannot certify a production journey; it makes gate bypass detection
/// stronger than checking only the intended frozen input sequence.
pub(crate) fn geometry(
    pack: &CompiledGamePack,
    route: &Route,
    before: &[u16],
    after: &[u16],
) -> Result<()> {
    let [width, height] = route.dimensions_metatiles;
    ensure!(
        before.len() == usize::from(width) * usize::from(height)
            && after.len() == before.len()
            && route.snowline_exclusive_y_by_meta_x.len() == usize::from(width),
        "winter whole source extent differs"
    );
    let mut delta = before.to_vec();
    let mut cells = BTreeSet::new();
    for cell in &route.changes {
        let [x, y] = cell.at;
        ensure!(
            x < width
                && y < height
                && cells.insert(cell.at)
                && cell.before != cell.after
                && y < route.snowline_exclusive_y_by_meta_x[usize::from(x)],
            "winter cell exceeds exact geographic section"
        );
        let index = usize::from(y) * usize::from(width) + usize::from(x);
        ensure!(delta[index] == cell.before, "stale winter cell preimage");
        delta[index] = cell.after;
    }
    ensure!(
        delta == after,
        "whole winter compile differs from exact declared cell delta"
    );
    let mut bodies = BTreeSet::new();
    let mut anchors = Vec::new();
    for source in &route.native_modules {
        let map = pack
            .data
            .maps
            .get(&source.map)
            .context("winter source native anchors")?;
        let origin = [source.column * 64, source.row * 48];
        for object in &map.objects {
            bodies.insert([origin[0] + object.x, origin[1] + object.y]);
        }
        anchors.extend(
            map.objects
                .iter()
                .map(|o| [origin[0] + o.x, origin[1] + o.y]),
        );
        anchors.extend(
            map.events
                .warps
                .iter()
                .map(|o| [origin[0] + o.x, origin[1] + o.y]),
        );
        anchors.extend(
            map.events
                .coord_events
                .iter()
                .map(|o| [origin[0] + o.x, origin[1] + o.y]),
        );
        anchors.extend(
            map.events
                .bg_events
                .iter()
                .map(|o| [origin[0] + o.x, origin[1] + o.y]),
        );
    }
    let declared_water = route
        .water_to_winter
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    ensure!(
        declared_water.len() == route.water_to_winter.len(),
        "duplicate winter freeze cells"
    );
    let mut actual_water = BTreeSet::new();
    let mut dry_before = BTreeSet::new();
    let mut dry_after = BTreeSet::new();
    let mut surf_after = BTreeSet::new();
    let mut ice = BTreeSet::new();
    for y in 0..height * 2 {
        for x in 0..width * 2 {
            let point = [x, y];
            let old = kind(pack, route, before, &route.old_tileset, point)?;
            let new = kind(pack, route, after, &route.new_tileset, point)?;
            ensure!(
                (old == "TALL_GRASS") == (new == "TALL_GRASS"),
                "winter erased original habitat"
            );
            match old {
                "FLOOR" => ensure!(
                    matches!(new, "FLOOR" | "ICE"),
                    "winter floor became obstacle"
                ),
                "WATER" if new != "WATER" => {
                    ensure!(
                        route.id == "MN_R17"
                            && (6..15).contains(&(x / 2))
                            && (25..32).contains(&(y / 2))
                            && declared_water.contains(&[x / 2, y / 2])
                            && matches!(new, "FLOOR" | "ICE"),
                        "winter changed unapproved water/shore gate"
                    );
                    actual_water.insert([x / 2, y / 2]);
                }
                _ => ensure!(
                    old == new,
                    "winter altered old directional collision/gate/warp"
                ),
            }
            if x == 0 || y == 0 || x + 1 == width * 2 || y + 1 == height * 2 {
                ensure!(old == new, "winter changed outside collision/Surf boundary");
            }
            if !bodies.contains(&point) {
                if dry(old) {
                    dry_before.insert(point);
                }
                if dry(new) {
                    dry_after.insert(point);
                }
                if dry(new) || new == "WATER" {
                    surf_after.insert(point);
                }
            }
            if new == "ICE" {
                ensure!(
                    !bodies.contains(&point),
                    "winter froze native actor footing"
                );
                ice.insert(point);
            }
        }
    }
    ensure!(
        actual_water == declared_water,
        "winter freeze list does not equal native water delta"
    );
    for point in &anchors {
        ensure!(
            kind(pack, route, before, &route.old_tileset, *point)?
                == kind(pack, route, after, &route.new_tileset, *point)?,
            "winter changed original native event anchor footing"
        );
    }
    let mut entries = BTreeSet::new();
    for point in &ice {
        for direction in DIRECTIONS {
            if let Some(start) = step(*point, [-direction[0], -direction[1]])
                && !bodies.contains(&start)
                && matches!(
                    kind(pack, route, after, &route.new_tileset, start)?,
                    "FLOOR" | "TALL_GRASS"
                )
            {
                entries.insert((start, direction));
            }
        }
    }
    let mut authored = BTreeSet::new();
    for planned in &route.slide_courses {
        ensure!(
            authored.insert((planned.from_core, planned.direction)),
            "duplicate winter course"
        );
        course(pack, route, after, &bodies, planned)?;
    }
    ensure!(
        entries == authored,
        "not every legal ICE entry has an exact resting/return contract"
    );
    if route.id == "MN_R11" {
        ensure!(
            dry_before == dry_after
                && route.cut.is_none()
                && route.protected_bridge_metatiles.is_empty()
                && route.required_grass_bounds_core.is_none(),
            "North Shore old directional footprint changed"
        );
        return Ok(());
    }
    let [start, end] = route.endpoints_core;
    ensure!(
        distance(start, end, &dry_before).is_some() && distance(start, end, &dry_after).is_some(),
        "winter cedar main Foot route disconnected"
    );
    for point in &anchors {
        let old_access = DIRECTIONS
            .into_iter()
            .filter_map(|d| step(*point, d))
            .any(|p| dry_before.contains(&p) && distance(start, p, &dry_before).is_some());
        let new_access = DIRECTIONS
            .into_iter()
            .filter_map(|d| step(*point, d))
            .any(|p| dry_after.contains(&p) && distance(start, p, &dry_after).is_some());
        ensure!(
            old_access == new_access,
            "winter unlocks an old gated target or strands an original approach"
        );
    }
    let cut = route.cut.as_ref().context("original cedar Cut contract")?;
    let index = usize::from(cut.tile[1] / 2) * usize::from(width) + usize::from(cut.tile[0] / 2);
    ensure!(
        after[index] == cut.source_block
            && kind(pack, route, after, &route.new_tileset, cut.tile)? == "CUT_TREE",
        "winter changed original live Cut tree"
    );
    let rule = pack
        .data
        .field_moves
        .cut
        .replacements
        .get(&route.new_tileset)
        .and_then(|rules| rules.get(&cut.source_block))
        .context("winter native Cut replacement")?;
    ensure!(
        rule.replacement_block_id == cut.replacement_block,
        "winter Cut replacement changed"
    );
    let mut opened = after.to_vec();
    opened[index] = cut.replacement_block;
    for surf in [false, true] {
        let closed = if surf { &surf_after } else { &dry_after };
        let mut open = BTreeSet::new();
        for y in 0..height * 2 {
            for x in 0..width * 2 {
                let point = [x, y];
                let token = kind(pack, route, &opened, &route.new_tileset, point)?;
                if !bodies.contains(&point) && (dry(token) || (surf && token == "WATER")) {
                    open.insert(point);
                }
            }
        }
        let before = distance(cut.compare_from, cut.compare_to, closed)
            .context("closed Cut target route")?;
        let after =
            distance(cut.compare_from, cut.compare_to, &open).context("opened Cut target route")?;
        ensure!(
            before == 66 && after == 6 && before.saturating_sub(after) >= cut.minimum_saved_steps,
            "winter bypasses or weakens the original useful Cut shortcut"
        );
    }
    let mut no_bridge = dry_after.clone();
    for [x, y] in &route.protected_bridge_metatiles {
        let index = usize::from(*y) * usize::from(width) + usize::from(*x);
        ensure!(
            before[index] == after[index],
            "winter changed native timber bridge/rails"
        );
        for dx in 0..2 {
            for dy in 0..2 {
                no_bridge.remove(&[x * 2 + dx, y * 2 + dy]);
            }
        }
    }
    ensure!(
        distance(start, end, &no_bridge).is_none(),
        "winter creates free timber bridge bypass"
    );
    let [x0, y0, x1, y1] = route
        .required_grass_bounds_core
        .context("original sedge neck")?;
    let mut no_grass = dry_after;
    for y in y0..y1 {
        for x in x0..x1 {
            ensure!(
                kind(pack, route, after, &route.new_tileset, [x, y])? == "TALL_GRASS",
                "old sedge neck grass lost"
            );
            no_grass.remove(&[x, y]);
        }
    }
    ensure!(
        distance(start, end, &no_grass).is_none(),
        "winter provides free sedge grass bypass"
    );
    Ok(())
}
