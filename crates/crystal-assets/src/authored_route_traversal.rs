//! Enforce encounter crossings without inventing routes or changing collision.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::authored_route_design::{Facing, RouteDesign};

/// Reject decorative replacements that remove the native hop or safe landing.
pub(crate) fn validate_ledges<'a>(
    design: &RouteDesign,
    kind: impl Fn([u16; 2]) -> Result<&'a str>,
) -> Result<()> {
    let [width, height] = design.dimensions_metatiles;
    let shift = |[x, y]: [u16; 2], dx: i32, dy: i32| -> Result<[u16; 2]> {
        Ok([
            u16::try_from(i32::from(x) + dx)?,
            u16::try_from(i32::from(y) + dy)?,
        ])
    };
    for ledge in &design.one_way_ledges {
        let [x0, y0, x1, y1] = ledge.bounds_core;
        ensure!(
            x0 < x1 && y0 < y1 && x1 <= width * 2 && y1 <= height * 2,
            "{}: invalid ledge bounds",
            ledge.id
        );
        let (hops, expected, dx, dy) = match ledge.direction.as_str() {
            "south" => {
                ensure!(y1 - y0 == 2, "{}: south ledge depth", ledge.id);
                (
                    (x0..x1).map(|x| [x, y0]).collect::<Vec<_>>(),
                    "HOP_DOWN",
                    0,
                    1,
                )
            }
            "west" => {
                ensure!(x1 - x0 == 2, "{}: west ledge depth", ledge.id);
                (
                    (y0..y1).map(|y| [x1 - 1, y]).collect::<Vec<_>>(),
                    "HOP_LEFT",
                    -1,
                    0,
                )
            }
            _ => anyhow::bail!("{}: unsupported authored ledge direction", ledge.id),
        };
        ensure!(
            hops.contains(&ledge.hop_from),
            "{}: hop outside face",
            ledge.id
        );
        for tile in hops {
            ensure!(
                kind(tile)? == expected && kind(shift(tile, dx, dy)?)? == "WALL",
                "{}: missing native hop or reverse wall",
                ledge.id
            );
        }
        ensure!(
            ledge.upper_approach == shift(ledge.hop_from, -dx, -dy)?
                && ledge.reverse_blocked == shift(ledge.hop_from, dx, dy)?
                && ledge.landing == shift(ledge.hop_from, dx * 2, dy * 2)?
                && matches!(kind(ledge.upper_approach)?, "FLOOR" | "TALL_GRASS")
                && matches!(kind(ledge.landing)?, "FLOOR" | "TALL_GRASS"),
            "{}: unsafe hop approach or landing",
            ledge.id
        );
        for &tile in &ledge.side_barriers {
            // Headbutt can start an encounter, but never removes the tree's
            // permanent wall collision. Cut trees can open and are not guards.
            ensure!(
                matches!(
                    kind(tile)?,
                    "WALL" | "WATER" | "HEADBUTT_TREE" | "HEADBUTT_TREE_1D"
                ),
                "{}: open ledge side",
                ledge.id
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_grass_crossings(
    design: &RouteDesign,
    walkable: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let start = design.endpoints[0].gate_metatile.map(|v| v * 2);
    let end = design.endpoints[1].gate_metatile.map(|v| v * 2);
    for crossing in &design.required_grass_crossings {
        ensure!(
            crossing.minimum_grass_steps > 0,
            "empty encounter challenge"
        );
        ensure!(
            minimum_grass_cost(start, end, walkable, &BTreeSet::new(), crossing.bounds_core)
                .is_some_and(|steps| steps >= crossing.minimum_grass_steps),
            "{}: encounter grass can be bypassed or is too short",
            crossing.id
        );
    }
    Ok(())
}

/// A drop must shorten a real walking loop, with a deliberate return passage.
/// An explicit local loop may coexist with longer walks elsewhere on the map;
/// the global shortest ordinary return must still satisfy the declared minimum.
pub(crate) fn validate_ledge_returns<'a>(
    design: &RouteDesign,
    walkable: &BTreeSet<[u16; 2]>,
    kind: impl Fn([u16; 2]) -> Result<&'a str>,
) -> Result<()> {
    // A warp trigger is not an ordinary step through this exterior. Hops are
    // deliberately excluded so a second drop cannot masquerade as an ascent.
    let mut floor = BTreeSet::new();
    for &tile in walkable {
        if matches!(kind(tile)?, "FLOOR" | "TALL_GRASS") {
            floor.insert(tile);
        }
    }
    let [width, height] = design.dimensions_metatiles;
    let extent = [0, 0, width * 2, height * 2];
    for ledge in &design.one_way_ledges {
        let [x0, y0, x1, y1] = ledge.return_passage_core;
        ensure!(
            x0 < x1
                && y0 < y1
                && x1 <= width * 2
                && y1 <= height * 2
                && ledge.minimum_return_steps >= 9,
            "{}: invalid or trivial ledge return contract",
            ledge.id
        );
        ensure!(
            floor.contains(&ledge.upper_approach) && floor.contains(&ledge.landing),
            "{}: inaccessible or occupied ledge approach/landing",
            ledge.id
        );
        let distance = minimum_grass_cost(
            ledge.landing,
            ledge.upper_approach,
            &floor,
            &BTreeSet::new(),
            extent,
        );
        ensure!(
            distance.is_some_and(|steps| steps >= ledge.minimum_return_steps),
            "{}: return is disconnected or can climb around the drop too cheaply",
            ledge.id
        );
        let passage = (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| [x, y]))
            .collect::<BTreeSet<_>>();
        ensure!(
            !passage.contains(&ledge.upper_approach) && !passage.contains(&ledge.landing),
            "{}: return passage overlaps a ledge endpoint",
            ledge.id
        );
        let mut local_floor = BTreeSet::new();
        let return_floor = if let Some([left, top, right, bottom]) = ledge.return_region_core {
            let inside = |[x, y]: [u16; 2]| left <= x && x < right && top <= y && y < bottom;
            ensure!(
                left < right
                    && top < bottom
                    && right <= width * 2
                    && bottom <= height * 2
                    && inside(ledge.upper_approach)
                    && inside(ledge.landing)
                    && passage.iter().all(|&tile| inside(tile)),
                "{}: invalid local return region or excluded endpoint/passage",
                ledge.id
            );
            local_floor.extend(floor.iter().copied().filter(|&tile| inside(tile)));
            ensure!(
                minimum_grass_cost(
                    ledge.landing,
                    ledge.upper_approach,
                    &local_floor,
                    &BTreeSet::new(),
                    extent
                )
                .is_some(),
                "{}: disconnected local ordinary return",
                ledge.id
            );
            &local_floor
        } else {
            &floor
        };
        ensure!(
            minimum_grass_cost(
                ledge.landing,
                ledge.upper_approach,
                return_floor,
                &passage,
                extent,
            )
            .is_none(),
            "{}: alternate ascent bypasses its declared return passage",
            ledge.id
        );
    }
    Ok(())
}

/// A paved battle lane and a grass alternative must both be actual choices.
pub(crate) fn validate_battle_grass_choices<'a>(
    design: &RouteDesign,
    walkable: &BTreeSet<[u16; 2]>,
    kind: impl Fn([u16; 2]) -> Result<&'a str>,
) -> Result<()> {
    let start = design.endpoints[0].gate_metatile.map(|v| v * 2);
    let end = design.endpoints[1].gate_metatile.map(|v| v * 2);
    let [width, height] = design.dimensions_metatiles;
    for choice in &design.battle_or_grass_choices {
        let [x0, y0, x1, y1] = choice.bounds_core;
        ensure!(
            x0 < x1
                && y0 < y1
                && x1 <= width * 2
                && y1 <= height * 2
                && choice.minimum_grass_steps > 0,
            "{}: invalid grass alternative",
            choice.id
        );
        for y in y0..y1 {
            for x in x0..x1 {
                ensure!(
                    kind([x, y])? == "TALL_GRASS",
                    "{}: grass alternative contains another terrain",
                    choice.id
                );
            }
        }
        let trainer = design
            .trainers
            .iter()
            .find(|t| t.id == choice.trainer_id)
            .context("choice trainer is missing")?;
        let (dx, dy) = match trainer.facing {
            Facing::North => (0, -1),
            Facing::South => (0, 1),
            Facing::East => (1, 0),
            Facing::West => (-1, 0),
        };
        let mut sight = BTreeSet::new();
        for n in 1..=i32::from(trainer.sight_range) {
            let (Ok(x), Ok(y)) = (
                u16::try_from(i32::from(trainer.tile[0]) + dx * n),
                u16::try_from(i32::from(trainer.tile[1]) + dy * n),
            ) else {
                break;
            };
            if !walkable.contains(&[x, y]) {
                break;
            }
            sight.insert([x, y]);
        }
        ensure!(
            minimum_grass_cost(start, end, walkable, &BTreeSet::new(), choice.bounds_core)
                == Some(0),
            "{}: missing paved option",
            choice.id
        );
        for (from, to) in [(start, end), (end, start)] {
            ensure!(
                minimum_grass_cost(from, to, walkable, &sight, choice.bounds_core)
                    .is_some_and(|steps| steps >= choice.minimum_grass_steps),
                "{}: grass alternative disconnected, too short or freely bypassed",
                choice.id
            );
        }
    }
    Ok(())
}

fn minimum_grass_cost(
    start: [u16; 2],
    end: [u16; 2],
    walkable: &BTreeSet<[u16; 2]>,
    blocked: &BTreeSet<[u16; 2]>,
    bounds: [u16; 4],
) -> Option<u16> {
    let [x0, y0, x1, y1] = bounds;
    let in_band = |[x, y]: [u16; 2]| (x0..x1).contains(&x) && (y0..y1).contains(&y);
    // Zero/one shortest paths measure required grass independently of distance.
    let mut costs = BTreeMap::from([(start, 0_u16)]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for next in [
            [x.saturating_sub(1), y],
            [x + 1, y],
            [x, y.saturating_sub(1)],
            [x, y + 1],
        ] {
            if !walkable.contains(&next) || blocked.contains(&next) {
                continue;
            }
            let cost = costs[&[x, y]] + u16::from(in_band(next));
            if costs.get(&next).is_some_and(|&prior| prior <= cost) {
                continue;
            }
            costs.insert(next, cost);
            if in_band(next) {
                queue.push_back(next);
            } else {
                queue.push_front(next);
            }
        }
    }
    costs.get(&end).copied()
}

#[cfg(test)]
#[path = "authored_route_traversal_tests.rs"]
mod tests;
