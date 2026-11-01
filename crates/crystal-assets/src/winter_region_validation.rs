//! Bounded winter geometry, native seams and exact residual preservation.
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_town_geometry::kind;
use crate::winter_region_design::{
    GROUP_CONSTANT, HUT, HUT_CONSTANT, NEIGHBOR, OUTDOOR, OUTDOOR_CONSTANT, Region,
};
use crate::{CompiledGamePack, MapModule};

type Point = [u16; 2];
const DIRECTIONS: [[i16; 2]; 4] = [[0, -1], [1, 0], [0, 1], [-1, 0]];

fn neighbor(at: Point, delta: [i16; 2], map: &MapModule) -> Option<Point> {
    let point = [
        at[0].checked_add_signed(delta[0])?,
        at[1].checked_add_signed(delta[1])?,
    ];
    (point[0] < map.attributes.width * 2 && point[1] < map.attributes.height * 2).then_some(point)
}

fn bodies(map: &MapModule) -> BTreeSet<Point> {
    map.objects.iter().map(|o| [o.x, o.y]).collect()
}

pub(crate) fn slide(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: Point,
    delta: [i16; 2],
    occupied: &BTreeSet<Point>,
) -> Result<Option<(Point, Vec<Point>)>> {
    let Some(mut at) = neighbor(start, delta, map) else {
        return Ok(None);
    };
    if occupied.contains(&at) || !matches!(kind(pack, map, at)?, "FLOOR" | "TALL_GRASS" | "ICE") {
        return Ok(None);
    }
    let mut course = vec![at];
    while kind(pack, map, at)? == "ICE" {
        let Some(next) = neighbor(at, delta, map) else {
            break;
        };
        if occupied.contains(&next)
            || !matches!(kind(pack, map, next)?, "FLOOR" | "TALL_GRASS" | "ICE")
        {
            break;
        }
        at = next;
        course.push(at);
        ensure!(course.len() <= 128, "ice course exceeds bounded map");
    }
    Ok(Some((at, course)))
}

fn reach(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: Point,
    ordinary: bool,
) -> Result<BTreeMap<Point, usize>> {
    let occupied = bodies(map);
    ensure!(
        kind(pack, map, start)? == "FLOOR" && !occupied.contains(&start),
        "safe snow/room arrival"
    );
    let mut seen = BTreeMap::from([(start, 0)]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for delta in DIRECTIONS {
            let Some((to, course)) = slide(pack, map, at, delta, &occupied)? else {
                continue;
            };
            if ordinary
                && course
                    .iter()
                    .any(|p| kind(pack, map, *p).is_ok_and(|k| k == "ICE"))
            {
                continue;
            }
            let steps = seen[&at] + course.len();
            if let Entry::Vacant(entry) = seen.entry(to) {
                entry.insert(steps);
                queue.push_back(to);
            }
        }
    }
    Ok(seen)
}

fn western_grass_cost(pack: &CompiledGamePack, map: &MapModule) -> Result<Option<u16>> {
    let occupied = bodies(map);
    let start = [37, 36];
    let mut scores = BTreeMap::from([(start, 0u16)]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for delta in DIRECTIONS {
            let Some(to) = neighbor(at, delta, map) else {
                continue;
            };
            if !(6..=37).contains(&to[0]) || !(28..=38).contains(&to[1]) || occupied.contains(&to) {
                continue;
            }
            let token = kind(pack, map, to)?;
            if !matches!(token, "FLOOR" | "TALL_GRASS") {
                continue;
            }
            let cost = scores[&at] + u16::from(token == "TALL_GRASS");
            if scores.get(&to).is_none_or(|old| cost < *old) {
                scores.insert(to, cost);
                queue.push_back(to);
            }
        }
    }
    Ok(scores.get(&[9, 28]).copied())
}

pub(crate) fn geometry(
    pack: &CompiledGamePack,
    outdoor: &MapModule,
    hut: &MapModule,
    region: &Region,
) -> Result<()> {
    ensure!(
        [outdoor.attributes.width, outdoor.attributes.height] == [32, 24]
            && [hut.attributes.width, hut.attributes.height] == [5, 4],
        "winter map extents"
    );
    let reached = reach(pack, outdoor, [60, 44], false)?;
    let occupied = bodies(outdoor);
    for y in 0..48 {
        for x in 0..64 {
            let at = [x, y];
            let token = kind(pack, outdoor, at)?;
            ensure!(
                token != "CUT_TREE" && token != "WATER",
                "unplanned Cut/Surf surface in winter region"
            );
            if x == 0 || x == 63 || y == 0 || y == 47 {
                ensure!(
                    if at == [63, 44] || at == [63, 45] {
                        token == "FLOOR"
                    } else {
                        token == "WALL"
                    },
                    "unconnected outer winter surface: {at:?}"
                );
            }
            if token == "FLOOR" && !occupied.contains(&at) {
                ensure!(
                    reached.contains_key(&at),
                    "unreachable ordinary snow tile: {at:?}"
                );
            }
        }
    }
    for object in &outdoor.objects {
        let at = [object.x, object.y];
        ensure!(
            kind(pack, outdoor, at)? == "FLOOR",
            "winter actor must stand on snow"
        );
        ensure!(
            DIRECTIONS
                .iter()
                .filter_map(|d| neighbor(at, *d, outdoor))
                .any(|p| reached.contains_key(&p)),
            "winter actor unreachable"
        );
    }
    for (from, delta, to, length) in [
        ([54, 36], [-1, 0], [37, 36], 17),
        ([37, 36], [1, 0], [54, 36], 17),
        ([19, 7], [0, 1], [19, 10], 3),
        ([19, 10], [0, -1], [19, 7], 3),
    ] {
        let (end, course) =
            slide(pack, outdoor, from, delta, &occupied)?.context("required real ice course")?;
        ensure!(
            end == to && course.len() == length && kind(pack, outdoor, end)? == "FLOOR",
            "required snow stopping bank differs"
        );
        ensure!(
            course[..course.len() - 1]
                .iter()
                .all(|p| kind(pack, outdoor, *p).is_ok_and(|k| k == "ICE")),
            "ice course is not actually slippery"
        );
    }
    ensure!(
        western_grass_cost(pack, outdoor)? == Some(6),
        "short western bank neck must retain six native encounter steps"
    );
    let dry = reach(pack, outdoor, [37, 36], true)?;
    ensure!(
        dry.contains_key(&[54, 36]),
        "ordinary snow return disconnected"
    );
    ensure!(
        !reach(pack, outdoor, [60, 44], true)?.contains_key(&[20, 11]),
        "cache island no longer requires ice"
    );
    ensure!(
        kind(pack, outdoor, [9, 27])? == "DOOR" && kind(pack, outdoor, [9, 28])? == "FLOOR",
        "real warming shelter facade/return"
    );
    let inside = reach(pack, hut, [5, 6], true)?;
    for object in &hut.objects {
        let at = [object.x, object.y];
        ensure!(
            kind(pack, hut, at)? == "FLOOR"
                && DIRECTIONS
                    .iter()
                    .filter_map(|d| neighbor(at, *d, hut))
                    .any(|p| inside.contains_key(&p)),
            "shelter actor/arrival conflict"
        );
    }
    ensure!(
        kind(pack, hut, [5, 7])? == "WARP_CARPET_DOWN"
            && hut.events.warps.len() == 2
            && outdoor.events.warps.len() == 2,
        "native shelter active/passive warps"
    );
    for (module, expected) in [
        (outdoor, [([9, 27], HUT_CONSTANT), ([9, 28], HUT_CONSTANT)]),
        (
            hut,
            [([5, 7], OUTDOOR_CONSTANT), ([5, 6], OUTDOOR_CONSTANT)],
        ),
    ] {
        for (i, (at, target)) in expected.iter().enumerate() {
            let warp = &module.events.warps[i];
            ensure!(
                warp.index == u16::try_from(i + 1)?
                    && [warp.x, warp.y] == *at
                    && warp.target_map_constant == *target
                    && warp.target_warp_id == 2,
                "exact native shelter pair"
            );
        }
    }
    ensure!(
        hut.script_runtime_commands
            .iter()
            .any(|c| c.source_script == "MN_ICE_WARMING_KEEPER"
                && c.command_index == 4
                && c.command == "special"
                && c.args == ["HealParty"]),
        "actual complete HealParty lowering required"
    );
    ensure!(
        hut.script_runtime_commands
            .iter()
            .all(|c| c.command != "blackoutmod"),
        "warming rest must not invent recovery"
    );
    ensure!(
        region.trainers.len() == 2 && region.items.len() == 1,
        "bounded winter content"
    );
    Ok(())
}

pub(crate) fn connections(pack: &CompiledGamePack) -> Result<()> {
    for (from, out_x, target, in_x) in [(OUTDOOR, 64, NEIGHBOR, 0), (NEIGHBOR, -1, OUTDOOR, 63)] {
        let map = pack.data.maps.get(from).context("winter seam source")?;
        let native =
            OverworldMapData::from_attributes(&map.id, &map.attributes, map.blocks.clone());
        for y in [44, 45] {
            let transition = connection_for_tile_checked(&native, TilePosition::new(out_x, y))?
                .context("native winter reciprocal seam")?;
            let destination = pack
                .data
                .resolve_connection_transition(&transition)?
                .destination;
            ensure!(
                destination.map_name == target && destination.tile == TilePosition::new(in_x, y),
                "winter seam offset/landing differs"
            );
            ensure!(
                kind(
                    pack,
                    &pack.data.maps[target],
                    [u16::try_from(in_x)?, u16::try_from(y)?]
                )? == "FLOOR",
                "winter seam needs ordinary snow/Floor"
            );
        }
    }
    Ok(())
}

pub(crate) fn preserve(
    base: &CompiledGamePack,
    pack: &CompiledGamePack,
    expected: &BTreeMap<String, MapModule>,
    constants: &[String],
    region: &Region,
) -> Result<()> {
    ensure!(
        pack.data.maps.len() == base.data.maps.len() + 2 && expected.len() == 2,
        "exactly two winter maps added"
    );
    let mut normalized = pack.clone();
    for name in [OUTDOOR, HUT] {
        let map = normalized
            .data
            .maps
            .remove(name)
            .context("new winter map")?;
        ensure!(
            expected.get(name) == Some(&map),
            "new winter program/module differs from authored production assembly"
        );
        let constant = map
            .attributes
            .map_constant
            .as_ref()
            .context("winter constant")?;
        normalized.data.map_attributes.remove(name);
        normalized.data.map_dimensions.remove(constant);
        normalized.data.runtime_map_metadata.remove(constant);
        normalized.data.map_blocks.remove(
            map.attributes
                .blocks_label
                .as_ref()
                .context("winter blocks label")?,
        );
        normalized.data.npcs.remove(name);
        normalized.data.wild_encounters.remove(name);
        normalized
            .data
            .pokegear_landmarks
            .map_to_landmark
            .remove(name);
        for label in map
            .scripts
            .keys()
            .chain(map.script_text_bodies.keys())
            .chain(map.attributes.map_scripts_label.iter())
            .chain(map.attributes.map_events_label.iter())
        {
            ensure!(
                !base.data.map_scripts.contains_key(label),
                "new winter root collides with old program"
            );
            normalized.data.map_scripts.remove(label);
        }
        let native_location = base
            .data
            .pokegear_landmark_for_map(NEIGHBOR)?
            .constant
            .clone();
        ensure!(
            pack.data.pokegear_landmark_for_map(name)?.constant == native_location
                && map.attributes.location.as_ref() == Some(&native_location),
            "exact existing R20 landmark binding"
        );
    }
    for (name, old) in &base.data.maps {
        let mut want = old.clone();
        if name == NEIGHBOR {
            crate::lake_country_geometry::connect(&mut want, "west", OUTDOOR, 0)?;
        }
        let wanted_attributes = if name == NEIGHBOR {
            &want.attributes
        } else {
            base.data
                .map_attributes
                .get(name)
                .context("old attributes mirror")?
        };
        ensure!(
            pack.data.maps.get(name) == Some(&want)
                && pack.data.map_attributes.get(name) == Some(wanted_attributes),
            "old map/program/actor/terrain drift: {name}"
        );
    }
    normalized
        .data
        .maps
        .insert(NEIGHBOR.into(), base.data.maps[NEIGHBOR].clone());
    normalized
        .data
        .map_attributes
        .insert(NEIGHBOR.into(), base.data.map_attributes[NEIGHBOR].clone());
    normalized
        .data
        .story_event_script_constants
        .global
        .remove(GROUP_CONSTANT);
    for constant in constants {
        ensure!(
            !base
                .data
                .story_event_script_constants
                .global
                .contains_key(constant),
            "new winter constant overwrote native value"
        );
        normalized
            .data
            .story_event_script_constants
            .global
            .remove(constant);
    }
    for trainer in &region.trainers {
        normalized.data.trainers.trainers.remove(&trainer.id);
    }
    ensure!(
        pack.data.wild_encounters.get(OUTDOOR) == Some(&crate::winter_region_modpack::wild(region))
            && !pack.data.wild_encounters.contains_key(HUT),
        "exact winter native grass table only"
    );
    let mut report = base.report.clone();
    report.maps += 2;
    for (id, name) in [(1, OUTDOOR), (2, HUT)] {
        report
            .manifests
            .push(format!("minnesota-rainy-river-ice-{id}"));
        report.reachable_maps.push(name.into());
        report.solvable_maps.push(name.into());
    }
    ensure!(
        pack.report == report,
        "unrelated winter publication report change"
    );
    normalized.report = base.report.clone();
    normalized.identity = base.identity.clone();
    ensure!(
        &normalized == base,
        "change beyond exact new winter content and R20 connection"
    );
    Ok(())
}
