//! Add walkable side trips to the twenty numbered routes, retaining native cache receipts.

use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::refresh_event_section;
use crate::northstar_cave_geometry::resting_tiles;
use crate::{CompiledGamePack, MapModule};

type Tile = [u16; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u8,
    routes: Vec<Route>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    route: u8,
    landmark: String,
    script: String,
    item: String,
    flag: String,
    source_map: String,
    from_tile: Tile,
    map_id: String,
    tile: Tile,
    target_kind: String,
    entry: Tile,
    edits: Vec<Edit>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    at: Tile,
    from: u16,
    to: u16,
}

pub(super) fn install(pack: &mut CompiledGamePack, source: &str) -> Result<Vec<String>> {
    let source: Source = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1 && source.routes.len() == 20,
        "expected all twenty route landmarks"
    );
    let mut changed = BTreeSet::new();
    for (number, route) in (1u8..=20).zip(source.routes) {
        let prefix = format!("MnR{number:02}C");
        ensure!(
            route.route == number
                && route.source_map == format!("{prefix}00R00")
                && route.map_id.starts_with(&prefix)
                && route.script == format!("MN_HIDDEN_R{number:02}")
                && route.flag == format!("EVENT_{}", route.script)
                && !route.landmark.trim().is_empty(),
            "invalid Route {number} cache source"
        );
        install_route(pack, &route)
            .with_context(|| format!("Route {number} / {}", route.landmark))?;
        changed.insert(route.source_map);
        changed.insert(route.map_id);
    }
    for id in &changed {
        let room = &pack.data.maps[id];
        let label = room
            .attributes
            .map_events_label
            .as_deref()
            .context("route event section")?;
        let section = pack
            .data
            .map_scripts
            .get(label)
            .context("published route events")?;
        ensure!(
            crate::parse_map_events(id, section)?.bg_events == room.events.bg_events,
            "split route events disagree with the published landmarks in {id}"
        );
    }
    Ok(changed.into_iter().collect())
}

fn install_route(pack: &mut CompiledGamePack, route: &Route) -> Result<()> {
    let mut source = pack
        .data
        .maps
        .get(&route.source_map)
        .context("original route cache owner")?
        .clone();
    let slot = source
        .events
        .bg_events
        .iter()
        .position(|bg| {
            bg.script == route.script
                && bg.event_type == "BGEVENT_ITEM"
                && [bg.x, bg.y] == route.from_tile
        })
        .context("existing collectible interaction at the original tile")?;
    let mut bg = source.events.bg_events.remove(slot);
    let script = source
        .scripts
        .remove(&route.script)
        .context("retained route cache program")?;
    let commands = script.as_array().context("native route cache commands")?;
    ensure!(
        commands.len() == 1
            && commands[0]["command"] == "hiddenitem"
            && commands[0]["args"][0].as_str() == Some(&route.item)
            && commands[0]["args"][1].as_str() == Some(&route.flag)
            && pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&route.flag),
        "native item and save receipt must match"
    );
    let pickup_slot = source
        .script_field_pickups
        .iter()
        .position(|p| {
            p.source_script == route.script
                && p.command == "hiddenitem"
                && p.quantity == 1
                && p.item_id.as_ref() == Some(&route.item)
                && p.event_flag.as_ref() == Some(&route.flag)
        })
        .context("original Itemfinder metadata")?;
    let pickup = source.script_field_pickups.remove(pickup_slot);
    let mut destination = if route.map_id == route.source_map {
        source
    } else {
        refresh_event_section(&mut source);
        authored_town_publication::replace(pack, source)?;
        pack.data
            .maps
            .get(&route.map_id)
            .context("destination route chunk")?
            .clone()
    };
    let before = &pack.data.maps[&route.map_id];
    apply_terrain(pack, before, &mut destination, route)?;
    ensure!(
        !destination.scripts.contains_key(&route.script)
            && destination
                .events
                .bg_events
                .iter()
                .all(|ev| [ev.x, ev.y] != route.tile),
        "occupied destination cache"
    );
    [bg.x, bg.y] = route.tile;
    destination.events.bg_events.push(bg);
    destination.scripts.insert(route.script.clone(), script);
    destination.script_field_pickups.push(pickup);
    refresh_event_section(&mut destination);
    authored_town_publication::replace(pack, destination)
}

fn neighbors([x, y]: Tile) -> impl Iterator<Item = Tile> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

fn reserved(room: &MapModule) -> BTreeSet<Tile> {
    let mut reserved: BTreeSet<_> = room
        .events
        .warps
        .iter()
        .map(|e| [e.x, e.y])
        .chain(room.events.bg_events.iter().map(|e| [e.x, e.y]))
        .chain(room.events.coord_events.iter().map(|e| [e.x, e.y]))
        .chain(room.objects.iter().map(|a| [a.x, a.y]))
        .collect();
    for change in &room.script_block_changes {
        for [dx, dy] in [[0, 0], [0, 1], [1, 0], [1, 1]] {
            reserved.insert([change.x + dx, change.y + dy]);
        }
    }
    for actor in room
        .objects
        .iter()
        .filter(|actor| actor.object_type == "OBJECTTYPE_TRAINER")
    {
        // Standing regional trainers can turn and challenge from eight squares.
        for distance in 1..=actor.radius.max(8) {
            reserved.extend(
                [
                    actor.x.checked_sub(distance).map(|x| [x, actor.y]),
                    actor.y.checked_sub(distance).map(|y| [actor.x, y]),
                    actor.x.checked_add(distance).map(|x| [x, actor.y]),
                    actor.y.checked_add(distance).map(|y| [actor.x, y]),
                ]
                .into_iter()
                .flatten(),
            );
        }
    }
    reserved
}

fn apply_terrain(
    pack: &CompiledGamePack,
    before: &MapModule,
    after: &mut MapModule,
    route: &Route,
) -> Result<()> {
    let [width, height] = [after.attributes.width, after.attributes.height];
    let reserved = reserved(before);
    let mut changed = BTreeSet::new();
    let mut new_ground = BTreeSet::new();
    for edit in &route.edits {
        let [x, y] = edit.at;
        ensure!(
            x > 0 && y > 0 && x + 1 < width && y + 1 < height,
            "route border must stay intact"
        );
        let index = usize::from(y) * usize::from(width) + usize::from(x);
        ensure!(
            after.blocks[index] == edit.from && edit.from != edit.to,
            "route block preimage at {:?}",
            edit.at
        );
        after.blocks[index] = edit.to;
        for [dx, dy] in [[0, 0], [0, 1], [1, 0], [1, 1]] {
            let at = [2 * x + dx, 2 * y + dy];
            ensure!(
                changed.insert(at) && !reserved.contains(&at) && kind(pack, before, at)? == "WALL",
                "only solid, unowned route interior may be opened at {at:?}"
            );
            let collision = kind(pack, after, at)?;
            ensure!(
                matches!(collision, "FLOOR" | "WALL" | "HEADBUTT_TREE"),
                "unexpected pickup terrain at {at:?}"
            );
            if collision == "FLOOR" {
                new_ground.insert(at);
            }
        }
    }
    ensure!(
        changed.contains(&route.tile)
            && matches!(route.target_kind.as_str(), "FLOOR" | "WALL")
            && kind(pack, after, route.tile)? == route.target_kind
            && kind(pack, before, route.entry)? == "FLOOR",
        "required native cache landmark and arrival"
    );
    check_mouths(pack, before, &new_ground, &changed, &reserved)?;
    for surf in [false, true] {
        let old = resting_tiles(pack, before, route.entry, surf)?;
        let new = resting_tiles(pack, after, route.entry, surf)?;
        ensure!(
            old.is_subset(&new) && new.difference(&old).all(|p| new_ground.contains(p)),
            "side trail changes previous access from {:?} (Surf={surf})",
            route.entry
        );
        let stand = neighbors(route.tile)
            .find(|p| new.contains(p) && new_ground.contains(p))
            .context("route landmark needs a stopping square inside its branch")?;
        ensure!(
            resting_tiles(pack, after, stand, surf)?.contains(&route.entry),
            "side trail has no return"
        );
    }
    Ok(())
}

// Every way into a new pocket must already be connected by a short walk in the
// original room with actors, sightlines, scene coordinates and shutters blocked.
// A loop can shorten that local walk without crossing any progression boundary.
fn check_mouths(
    pack: &CompiledGamePack,
    before: &MapModule,
    ground: &BTreeSet<Tile>,
    changed: &BTreeSet<Tile>,
    reserved: &BTreeSet<Tile>,
) -> Result<()> {
    let mut mouths = BTreeSet::new();
    for at in ground {
        for next in neighbors(*at).filter(|p| !changed.contains(p)) {
            match kind(pack, before, next)? {
                "WALL" | "CUT_TREE" | "HEADBUTT_TREE" | "WATER" | "WHIRLPOOL" | "WATERFALL" => {}
                "FLOOR" | "TALL_GRASS" | "LONG_GRASS" if !reserved.contains(&next) => {
                    mouths.insert(next);
                }
                other => anyhow::bail!(
                    "route branch meets an active or gated tile at {next:?} ({other})"
                ),
            }
        }
    }
    let start = *mouths
        .first()
        .context("side trail requires an existing walking entrance")?;
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([(start, 0)]);
    while let Some((at, depth)) = todo.pop_front() {
        if depth == 12 {
            continue;
        }
        for next in neighbors(at)
            .filter(|p| p[0] < before.attributes.width * 2 && p[1] < before.attributes.height * 2)
        {
            if !seen.contains(&next)
                && !reserved.contains(&next)
                && matches!(
                    kind(pack, before, next)?,
                    "FLOOR" | "TALL_GRASS" | "LONG_GRASS"
                )
            {
                seen.insert(next);
                todo.push_back((next, depth + 1));
            }
        }
    }
    ensure!(
        mouths.is_subset(&seen),
        "side trail joins different access tiers"
    );
    Ok(())
}
