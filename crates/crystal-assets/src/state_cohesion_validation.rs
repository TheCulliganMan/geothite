//! Native projection, closed checkpoint throats and complete unrelated-data proof.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_town_geometry::kind;
use crate::state_cohesion_design::{Dossier, Plan};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn reverse(side: &str) -> Result<&'static str> {
    match side {
        "north" => Ok("south"),
        "south" => Ok("north"),
        "west" => Ok("east"),
        "east" => Ok("west"),
        _ => anyhow::bail!("invalid cohesion side"),
    }
}
fn flood(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: [u16; 2],
    blocked: &BTreeSet<[u16; 2]>,
    grass: bool,
) -> Result<BTreeSet<[u16; 2]>> {
    ensure!(
        start[0] < map.attributes.width * 2 && start[1] < map.attributes.height * 2,
        "flood start outside map"
    );
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for [u, v] in [
            [i32::from(x) - 1, i32::from(y)],
            [i32::from(x) + 1, i32::from(y)],
            [i32::from(x), i32::from(y) - 1],
            [i32::from(x), i32::from(y) + 1],
        ] {
            if u < 0
                || v < 0
                || u >= i32::from(map.attributes.width) * 2
                || v >= i32::from(map.attributes.height) * 2
            {
                continue;
            }
            let p = [u16::try_from(u)?, u16::try_from(v)?];
            if seen.contains(&p) || blocked.contains(&p) {
                continue;
            }
            let token = kind(pack, map, p)?;
            if token == "FLOOR" || grass && token == "TALL_GRASS" {
                seen.insert(p);
                queue.push_back(p);
            }
        }
    }
    Ok(seen)
}
pub(crate) fn geometry(
    pack: &CompiledGamePack,
    maps: &BTreeMap<String, MapModule>,
    plan: &Plan,
    dossiers: &BTreeMap<String, Dossier>,
    guarded: bool,
) -> Result<()> {
    for (name, d) in dossiers {
        let map = &maps[name];
        let mut ports = BTreeSet::new();
        ensure!(d.ports.len() == 2, "two source mouths required");
        for port in &d.ports {
            for p in port.core {
                ensure!(
                    p[0] < map.attributes.width * 2
                        && p[1] < map.attributes.height * 2
                        && match port.side.as_str() {
                            "west" => p[0] == 0,
                            "east" => p[0] + 1 == map.attributes.width * 2,
                            "north" => p[1] == 0,
                            "south" => p[1] + 1 == map.attributes.height * 2,
                            _ => false,
                        }
                        && kind(pack, map, p)? == "FLOOR",
                    "source mouth must be literal native boundary Floor"
                );
                ports.insert(p);
            }
        }
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                let token = kind(pack, map, [x, y])?;
                ensure!(
                    matches!(token, "FLOOR" | "TALL_GRASS" | "WALL"),
                    "connector introduces water/HM/ledge/warp collision"
                );
                if x == 0
                    || y == 0
                    || x + 1 == map.attributes.width * 2
                    || y + 1 == map.attributes.height * 2
                {
                    ensure!(
                        ports.contains(&[x, y]) || token == "WALL",
                        "unpaired connector edge must be visibly closed"
                    );
                }
            }
        }
        let actors = d.actors.iter().map(|a| a.tile).collect::<BTreeSet<_>>();
        let start = d.ports[0].core[0];
        let seen = flood(pack, map, start, &actors, true)?;
        ensure!(
            ports.iter().all(|p| seen.contains(p)),
            "actor-present connector travel disconnected"
        );
        for p in d
            .actors
            .iter()
            .map(|a| a.tile)
            .chain(d.signs.iter().map(|s| s.tile))
        {
            let adjacent = [
                [i32::from(p[0]) - 1, i32::from(p[1])],
                [i32::from(p[0]) + 1, i32::from(p[1])],
                [i32::from(p[0]), i32::from(p[1]) - 1],
                [i32::from(p[0]), i32::from(p[1]) + 1],
            ]
            .into_iter()
            .any(|[x, y]| {
                x >= 0
                    && y >= 0
                    && u16::try_from(x)
                        .ok()
                        .zip(u16::try_from(y).ok())
                        .is_some_and(|(x, y)| seen.contains(&[x, y]))
            });
            ensure!(
                adjacent,
                "new interaction has no reachable adjacent approach"
            );
        }
        let has_grass = map.blocks.iter().any(|block| {
            pack.data.tilesets[&map.attributes.tileset_name]
                .collision
                .get(&format!("{block:02x}"))
                .is_some_and(|quadrants| quadrants.iter().any(|token| token == "TALL_GRASS"))
        });
        ensure!(
            has_grass
                == plan
                    .encounter_bindings_proposed
                    .iter()
                    .any(|e| e.map == *name),
            "declared native grass crossing missing or unbound"
        );
        if has_grass {
            let dry = flood(pack, map, start, &actors, false)?;
            ensure!(
                !d.ports[1].core.iter().any(|p| dry.contains(p)),
                "declared crossing grass freely avoidable"
            );
        }
    }
    for g in &plan.guard_proposals {
        let map = &maps[&g.map];
        ensure!(
            g.outbound_only
                && g.inbound_free
                && g.sprite == "SPRITE_OFFICER"
                && !g.flags.is_empty(),
            "free-return native checkpoint required"
        );
        let owned_ids = g
            .actor_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let points = g
            .bodies
            .into_iter()
            .chain(g.inside)
            .chain(g.outside)
            .collect::<BTreeSet<_>>();
        ensure!(
            points.len() == 6
                && !map
                    .objects
                    .iter()
                    .filter(|o| !guarded
                        || !o
                            .object_identifier
                            .as_deref()
                            .is_some_and(|id| owned_ids.contains(id)))
                    .any(|o| points.contains(&[o.x, o.y]))
                && !map
                    .events
                    .warps
                    .iter()
                    .any(|o| points.contains(&[o.x, o.y]))
                && !map
                    .events
                    .bg_events
                    .iter()
                    .any(|o| points.contains(&[o.x, o.y]))
                && !map
                    .events
                    .coord_events
                    .iter()
                    .filter(|o| !guarded
                        || !(o.script_name == g.id || o.script_name == format!("{}Inward", g.id)))
                    .any(|o| points.contains(&[o.x, o.y])),
            "new checkpoint overlaps an original interaction/actor/event"
        );
        for p in g.bodies.into_iter().chain(g.inside).chain(g.outside) {
            ensure!(
                kind(pack, map, p)? == "FLOOR",
                "checkpoint lane not native Floor"
            );
        }
        ensure!(
            g.flags.iter().all(|f| pack
                .data
                .story_event_script_constants
                .global
                .contains_key(f))
                && (g.event.is_empty()
                    || pack
                        .data
                        .story_event_script_constants
                        .global
                        .contains_key(&g.event)),
            "checkpoint references undefined native prerequisite"
        );
        let ids = g
            .actor_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let mut blocked = map
            .objects
            .iter()
            .filter(|o| {
                !guarded
                    || !o
                        .object_identifier
                        .as_deref()
                        .is_some_and(|id| ids.contains(id))
            })
            .map(|o| [o.x, o.y])
            .collect::<BTreeSet<_>>();
        blocked.extend(g.bodies);
        let closed = flood(pack, map, g.inside[0], &blocked, true)?;
        ensure!(
            !g.outside.iter().any(|p| closed.contains(p)),
            "new checkpoint has ordinary side bypass"
        );
        for p in g.bodies {
            blocked.remove(&p);
        }
        let open = flood(pack, map, g.inside[0], &blocked, true)?;
        ensure!(
            g.outside.iter().all(|p| open.contains(p)),
            "qualified checkpoint passage disconnected"
        );
    }
    Ok(())
}
fn cross(
    pack: &CompiledGamePack,
    map: &MapModule,
    p: [u16; 2],
    side: &str,
) -> Result<(String, [u16; 2])> {
    let [x, y] = p.map(i16::try_from);
    let x = x?;
    let y = y?;
    let out = match side {
        "north" => TilePosition::new(x, -1),
        "south" => TilePosition::new(x, y + 1),
        "west" => TilePosition::new(-1, y),
        "east" => TilePosition::new(x + 1, y),
        _ => anyhow::bail!("invalid native crossing"),
    };
    let native = OverworldMapData::from_attributes(&map.id, &map.attributes, map.blocks.clone());
    let transition =
        connection_for_tile_checked(&native, out)?.context("native cohesion connection missing")?;
    let target = pack
        .data
        .resolve_connection_transition(&transition)?
        .destination;
    Ok((
        target.map_name,
        [u16::try_from(target.tile.x)?, u16::try_from(target.tile.y)?],
    ))
}
fn placements(pack: &CompiledGamePack, plan: &Plan) -> Result<()> {
    let outdoors = pack
        .data
        .maps
        .iter()
        .filter(|(n, m)| {
            n.starts_with("Mn")
                && m.attributes.environment.as_deref().is_some_and(|e| {
                    matches!(
                        e.to_ascii_lowercase().as_str(),
                        "town" | "route" | "outdoor"
                    )
                })
        })
        .map(|(n, m)| (n.clone(), m))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        outdoors.len() == 98 && plan.proposed_origins_metatiles.len() == 98,
        "all98 outdoor footprints required"
    );
    let mut pos = BTreeMap::from([("MnMinneapolis".to_owned(), [0i32, 0i32])]);
    let mut queue = VecDeque::from(["MnMinneapolis".to_owned()]);
    while let Some(name) = queue.pop_front() {
        let map = outdoors[&name];
        let [x, y] = pos[&name];
        for edge in &map.attributes.connections {
            let target = outdoors
                .get(&edge.target_map)
                .context("outdoor connection target")?;
            let at = match edge.direction.as_str() {
                "north" => [x + edge.offset, y - i32::from(target.attributes.height)],
                "south" => [x + edge.offset, y + i32::from(map.attributes.height)],
                "west" => [x - i32::from(target.attributes.width), y + edge.offset],
                "east" => [x + i32::from(map.attributes.width), y + edge.offset],
                _ => anyhow::bail!("invalid placement side"),
            };
            if let Some(old) = pos.get(&edge.target_map) {
                ensure!(*old == at, "inconsistent full-state origin cycle");
            } else {
                pos.insert(edge.target_map.clone(), at);
                queue.push_back(edge.target_map.clone());
            }
        }
    }
    ensure!(
        pos == plan.proposed_origins_metatiles,
        "full-state maps detached or authored origins differ"
    );
    let names = outdoors.keys().collect::<Vec<_>>();
    for (i, name) in names.iter().enumerate() {
        let [x, y] = pos[*name];
        let a = outdoors[*name];
        for other in &names[i + 1..] {
            let [u, v] = pos[*other];
            let b = outdoors[*other];
            ensure!(
                x + i32::from(a.attributes.width) <= u
                    || u + i32::from(b.attributes.width) <= x
                    || y + i32::from(a.attributes.height) <= v
                    || v + i32::from(b.attributes.height) <= y,
                "overlapping outdoor maps: {name}/{other}"
            );
        }
    }
    Ok(())
}
pub(crate) fn validate(
    base: &CompiledGamePack,
    pack: &CompiledGamePack,
    plan: &Plan,
    dossiers: &BTreeMap<String, Dossier>,
    expected: &BTreeMap<String, MapModule>,
) -> Result<()> {
    geometry(pack, expected, plan, dossiers, true)?;
    for link in &plan.new_reciprocal_links {
        for (name, side, target, offset, from, to) in [
            (
                &link.from_map,
                link.direction.as_str(),
                &link.to_map,
                link.offset,
                link.from_core,
                link.to_core,
            ),
            (
                &link.to_map,
                reverse(&link.direction)?,
                &link.from_map,
                link.reverse_offset,
                link.to_core,
                link.from_core,
            ),
        ] {
            let map = &pack.data.maps[name];
            ensure!(
                map.attributes
                    .connections
                    .iter()
                    .any(|e| e.direction == side && e.target_map == *target && e.offset == offset),
                "declared reciprocal link missing"
            );
            for (a, b) in from.into_iter().zip(to) {
                ensure!(
                    kind(pack, map, a)? == "FLOOR"
                        && kind(pack, &pack.data.maps[target], b)? == "FLOOR"
                        && cross(pack, map, a, side)? == (target.clone(), b),
                    "native two-lane connection projection differs"
                );
            }
        }
    }
    placements(pack, plan)?;
    let mut uncached_data = pack.data.clone();
    let mut baseline_data = base.data.clone();
    for (name, map) in expected {
        uncached_data
            .maps
            .remove(name)
            .context("remove cached owned map for real reassembly")?;
        ensure!(
            pack.data.maps.get(name) == Some(map),
            "published module differs from exact prepared delta: {name}"
        );
        let mut uncached = uncached_data.assemble_map_module_from_compiled_payloads(name)?;
        for label in [
            &map.attributes.map_scripts_label,
            &map.attributes.map_events_label,
        ] {
            uncached
                .scripts
                .remove(label.as_ref().context("transport label")?);
        }
        if let Some(old) = base.data.maps.get(name)
            && !plan.guard_proposals.iter().any(|g| g.map == *name)
        {
            geometry_only(old, map)?;
            baseline_data
                .maps
                .remove(name)
                .context("remove old cached terrain module")?;
            let mut baseline = baseline_data.assemble_map_module_from_compiled_payloads(name)?;
            for label in [
                &old.attributes.map_scripts_label,
                &old.attributes.map_events_label,
            ] {
                baseline
                    .scripts
                    .remove(label.as_ref().context("old transport label")?);
            }
            // The parent already has cached/uncached differences in some story
            // tables. Preserve both representations without repairing either.
            // Only the reviewed geometry may differ from its own uncached base.
            baseline.blocks = map.blocks.clone();
            baseline.attributes.width = map.attributes.width;
            baseline.attributes.connections = map.attributes.connections.clone();
            baseline.attributes.connection_flags = map.attributes.connection_flags.clone();
            ensure!(
                uncached == baseline,
                "inherited uncached terrain module changed beyond geometry: {name}: {}",
                differences(&baseline, &uncached)?
            );
            baseline_data.maps.insert(name.clone(), old.clone());
            ensure!(
                pack.data.npcs.get(name) == base.data.npcs.get(name),
                "old terrain NPC mirror changed: {name}"
            );
        } else {
            ensure!(
                &uncached == map,
                "uncached production module differs: {name}: {}",
                differences(map, &uncached)?
            );
        }
        uncached_data.maps.insert(name.clone(), map.clone());
    }
    drop(uncached_data);
    drop(baseline_data);
    let guard_events = plan
        .guard_proposals
        .iter()
        .map(|g| {
            base.data.maps[&g.map]
                .attributes
                .map_events_label
                .as_deref()
                .context("guard event label")
        })
        .collect::<Result<BTreeSet<_>>>()?;
    for (label, body) in &base.data.map_scripts {
        ensure!(
            guard_events.contains(label.as_str()) || pack.data.map_scripts.get(label) == Some(body),
            "original global raw body changed: {label}"
        );
    }
    let mut normalized = pack.data.clone();
    for source in &plan.new_maps {
        let map = normalized
            .maps
            .remove(&source.id)
            .context("new connector")?;
        normalized.map_attributes.remove(&source.id);
        normalized.map_dimensions.remove(&source.map_constant);
        normalized.runtime_map_metadata.remove(&source.map_constant);
        normalized
            .map_blocks
            .remove(map.attributes.blocks_label.as_ref().context("new blocks")?);
        normalized.npcs.remove(&source.id);
        normalized.wild_encounters.remove(&source.id);
        normalized
            .pokegear_landmarks
            .map_to_landmark
            .remove(&source.id);
        for label in map
            .scripts
            .keys()
            .chain(map.script_text_bodies.keys())
            .chain(map.attributes.map_scripts_label.iter())
            .chain(map.attributes.map_events_label.iter())
        {
            normalized.map_scripts.remove(label);
        }
        let landmark = base.data.pokegear_landmark_for_map(
            &plan
                .landmark_bindings
                .iter()
                .find(|l| l.map == source.id)
                .context("landmark binding")?
                .reference_native_map,
        )?;
        ensure!(
            pack.data.pokegear_landmark_for_map(&source.id)? == landmark
                && map.attributes.location.as_deref() == Some(landmark.constant.as_str()),
            "connector geographic registration missing"
        );
    }
    for (name, old) in &base.data.maps {
        let actual = &pack.data.maps[name];
        ensure!(
            actual == expected.get(name).unwrap_or(old),
            "unrelated original module changed: {name}"
        );
        if expected.contains_key(name) {
            if plan.guard_proposals.iter().any(|g| g.map == *name) {
                preserve_program_prefix(old, actual)?;
            }
            for y in 0..actual.attributes.height * 2 {
                for x in 0..actual.attributes.width * 2 {
                    let before = kind(base, old, [x, y])?;
                    if matches!(before, "WATER" | "WATERFALL" | "TALL_GRASS")
                        || before.contains("CUT")
                    {
                        ensure!(
                            kind(pack, actual, [x, y])? == before,
                            "old water/grass/Cut changed"
                        );
                    }
                }
            }
            for p in old
                .objects
                .iter()
                .map(|o| [o.x, o.y])
                .chain(old.events.warps.iter().map(|o| [o.x, o.y]))
                .chain(old.events.coord_events.iter().map(|o| [o.x, o.y]))
                .chain(old.events.bg_events.iter().map(|o| [o.x, o.y]))
            {
                ensure!(
                    kind(base, old, p)? == kind(pack, actual, p)?,
                    "original native anchor collision changed"
                );
            }
            // Restore only the declared owned modules and their wire mirrors.
            normalized.maps.insert(name.clone(), old.clone());
            normalized
                .map_attributes
                .insert(name.clone(), old.attributes.clone());
            let constant = old
                .attributes
                .map_constant
                .as_ref()
                .context("old map constant")?;
            normalized
                .map_dimensions
                .insert(constant.clone(), base.data.map_dimensions[constant].clone());
            normalized.runtime_map_metadata.insert(
                constant.clone(),
                base.data.runtime_map_metadata[constant].clone(),
            );
            let block = old
                .attributes
                .blocks_label
                .as_ref()
                .context("old block label")?;
            normalized
                .map_blocks
                .insert(block.clone(), base.data.map_blocks[block].clone());
            if let Some(npcs) = base.data.npcs.get(name) {
                normalized.npcs.insert(name.clone(), npcs.clone());
            } else {
                normalized.npcs.remove(name);
            }
            for label in actual
                .scripts
                .keys()
                .chain(actual.script_text_bodies.keys())
                .chain(actual.attributes.map_scripts_label.iter())
                .chain(actual.attributes.map_events_label.iter())
            {
                if !base.data.map_scripts.contains_key(label) {
                    normalized.map_scripts.remove(label);
                }
            }
            if plan.guard_proposals.iter().any(|g| g.map == *name) {
                let label = old
                    .attributes
                    .map_events_label
                    .as_ref()
                    .context("guard old event label")?;
                normalized
                    .map_scripts
                    .insert(label.clone(), base.data.map_scripts[label].clone());
            }
        }
    }
    normalized
        .story_event_script_constants
        .global
        .remove(&plan.registry.map_group_constant);
    for d in dossiers.values() {
        for a in &d.actors {
            normalized.story_event_script_constants.global.remove(&a.id);
            normalized
                .story_event_script_constants
                .global
                .remove(&a.event_flag);
            normalized.trainers.trainers.remove(&a.id);
        }
    }
    ensure!(
        serde_json::to_value(&normalized)? == serde_json::to_value(&base.data)?,
        "unrelated catalogs/scripts/recovery/flags/data changed"
    );
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_manifest == base.audio_manifest
            && pack.audio_compression == base.audio_compression,
        "original native art/audio/files changed"
    );
    let mut report = pack.report.clone();
    report.maps = base.report.maps;
    report.manifests = base.report.manifests.clone();
    report.reachable_maps = base.report.reachable_maps.clone();
    report.solvable_maps = base.report.solvable_maps.clone();
    ensure!(report == base.report, "unrelated report changed");
    Ok(())
}

/// Rejects changes to any cached gameplay field on an unguarded old module.
/// Width has one reviewed exception: R03's literal-Wall eastern margin.
pub(crate) fn geometry_only(old: &MapModule, actual: &MapModule) -> Result<()> {
    let mut expected = old.clone();
    expected.blocks = actual.blocks.clone();
    expected.attributes.connections = actual.attributes.connections.clone();
    expected.attributes.connection_flags = actual.attributes.connection_flags.clone();
    if old.id == "MnR03C00R01" {
        ensure!(
            [old.attributes.width, actual.attributes.width] == [24, 21],
            "R03 margin extent"
        );
        expected.attributes.width = 21;
    }
    ensure!(
        &expected == actual,
        "terrain-only cached gameplay fields changed: {}",
        old.id
    );
    Ok(())
}

fn preserve_program_prefix(old: &MapModule, actual: &MapModule) -> Result<()> {
    ensure!(
        actual.objects.starts_with(&old.objects)
            && actual.events.warps == old.events.warps
            && actual.events.bg_events == old.events.bg_events
            && actual
                .events
                .coord_events
                .starts_with(&old.events.coord_events),
        "checkpoint changed old actors/indexed event rows: {}",
        old.id
    );
    let before = serde_json::to_value(old)?;
    let after = serde_json::to_value(actual)?;
    let new_labels = actual
        .scripts
        .keys()
        .filter(|label| !old.scripts.contains_key(*label))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for (field, value) in before.as_object().context("old guarded module")? {
        if matches!(
            field.as_str(),
            "attributes" | "blocks" | "objects" | "events" | "map_event_section_commands"
        ) {
            continue;
        }
        let same = match value {
            serde_json::Value::Array(original)
                if matches!(
                    field.as_str(),
                    "script_control_commands" | "script_flag_commands" | "script_text_commands"
                ) =>
            {
                after[field].as_array().is_some_and(|rows| {
                    rows.iter()
                        .filter(|row| {
                            !row["source_script"]
                                .as_str()
                                .is_some_and(|source| new_labels.contains(source))
                        })
                        .eq(original.iter())
                })
            }
            serde_json::Value::Array(prefix) => after[field]
                .as_array()
                .is_some_and(|v| v.starts_with(prefix)),
            serde_json::Value::Object(labels) => labels
                .iter()
                .all(|(key, body)| after[field].get(key) == Some(body)),
            _ => &after[field] == value,
        };
        ensure!(
            same,
            "checkpoint changed old cached program field {}: {field}",
            old.id
        );
    }
    Ok(())
}

/// Bounded diagnostic: field names/counts and record identities, never program dumps.
fn differences(expected: &MapModule, actual: &MapModule) -> Result<String> {
    let a = serde_json::to_value(expected)?;
    let b = serde_json::to_value(actual)?;
    let mut rows = Vec::new();
    for (field, before) in a.as_object().context("expected module object")? {
        let after = &b[field];
        if before == after {
            continue;
        }
        let mut row = serde_json::json!({"field":field});
        if field == "events" {
            let mut details = Vec::new();
            for key in ["warps", "coord_events", "bg_events"] {
                let x = before[key].as_array().context("expected event rows")?;
                let y = after[key].as_array().context("actual event rows")?;
                if x != y {
                    let at = x
                        .iter()
                        .zip(y)
                        .position(|(a, b)| a != b)
                        .unwrap_or(x.len().min(y.len()));
                    details.push(serde_json::json!({"kind":key,"counts":[x.len(),y.len()],"first_index":at,"expected":x.get(at),"actual":y.get(at)}));
                }
            }
            row["details"] = serde_json::json!(details);
        }
        if let (Some(x), Some(y)) = (before.as_array(), after.as_array()) {
            row["counts"] = serde_json::json!([x.len(), y.len()]);
            let at = x
                .iter()
                .zip(y)
                .position(|(a, b)| a != b)
                .unwrap_or(x.len().min(y.len()));
            row["first_index"] = serde_json::json!(at);
            for (name, value) in [("expected", x.get(at)), ("actual", y.get(at))] {
                if let Some(value) = value {
                    let keys = [
                        "source_script",
                        "script_name",
                        "label",
                        "command_index",
                        "command",
                        "x",
                        "y",
                        "scene_id",
                    ];
                    let identity = keys
                        .into_iter()
                        .filter_map(|key| value.get(key).map(|v| (key.to_owned(), v.clone())))
                        .collect::<serde_json::Map<_, _>>();
                    row[name] = serde_json::Value::Object(identity);
                }
            }
        }
        rows.push(row);
    }
    Ok(serde_json::to_string(&rows)?)
}
