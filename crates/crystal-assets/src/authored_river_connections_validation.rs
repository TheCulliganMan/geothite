//! Native seam, bridge-layer and complete unrelated-data preservation guards.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_river_connections_design::{BridgeInventory, Manifest, NewMap};
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(
    base: &CompiledGamePack,
    pack: &CompiledGamePack,
    plan: &Manifest,
    bridges: &BridgeInventory,
) -> Result<()> {
    for source in &plan.new_maps {
        let map = &pack.data.maps[&source.id];
        let location = crate::authored_river_connections::landmark(base, &source.id)?;
        ensure!(
            map.attributes.location.as_deref() == Some(location.as_str())
                && pack.data.pokegear_landmark_for_map(&source.id)?.constant == location,
            "new river native location and exact town-map mapping differ"
        );
        for link in &source.connections {
            let same_source = plan
                .new_maps
                .iter()
                .find(|other| other.id == link.target_map)
                .is_some_and(|other| adjacent_windows(source, other, &link.direction, link.offset));
            seam(
                pack,
                map,
                &link.direction,
                &link.target_map,
                link.offset,
                same_source,
            )
            .with_context(|| format!("{} {} -> {}", map.id, link.direction, link.target_map))?;
        }
        perimeter(pack, map)?;
    }
    for link in &plan.existing_connection_additions {
        let map = &pack.data.maps[&link.map];
        seam(
            pack,
            map,
            &link.direction,
            &link.target_map,
            link.offset,
            false,
        )
        .with_context(|| format!("{} {} -> {}", map.id, link.direction, link.target_map))?;
    }
    for revision in &plan.terrain_revisions {
        perimeter(pack, &pack.data.maps[&revision.map])?;
    }
    for bridge in &bridges.bridges {
        let map = &pack.data.maps[&bridge.map];
        let [x0, y0, x1, y1] = bridge.metatile_bounds_exclusive;
        let [start, end]: [[u16; 2]; 2] = bridge
            .water_mouths_core
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("two bridge mouths"))?;
        ensure!(
            kind(pack, map, start)? == "WATER" && kind(pack, map, end)? == "WATER",
            "bridge water mouths must be original Water"
        );
        let ns = bridge.axis == "north_south";
        ensure!(
            bridge.aperture_width_core == if ns { (x1 - x0) * 2 } else { (y1 - y0) * 2 },
            "bridge aperture width differs"
        );
        ensure!(
            if ns {
                start[0] == end[0]
                    && start[1] + 1 == y0 * 2
                    && end[1] == y1 * 2
                    && x0 * 2 <= start[0]
                    && start[0] < x1 * 2
            } else {
                start[1] == end[1]
                    && start[0] + 1 == x0 * 2
                    && end[0] == x1 * 2
                    && y0 * 2 <= start[1]
                    && start[1] < y1 * 2
            },
            "bridge mouths do not bound the declared underpass"
        );
        for y in y0 * 2..y1 * 2 {
            for x in x0 * 2..x1 * 2 {
                let old = kind(base, &base.data.maps[&bridge.map], [x, y])?;
                ensure!(
                    foot(kind(pack, map, [x, y])?) == old,
                    "bridge changed native foot collision"
                );
            }
        }
    }
    placements(pack, plan)?;
    preserve(base, pack, plan, bridges)
}

fn foot(token: &str) -> &str {
    match token {
        "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK" => "FLOOR",
        "BRIDGE_NS_RAIL" | "BRIDGE_EW_RAIL" => "WALL",
        token => token,
    }
}
fn water(token: &str) -> bool {
    token == "WATER" || token.starts_with("BRIDGE_")
}
fn reverse(side: &str) -> Result<&'static str> {
    Ok(match side {
        "north" => "south",
        "south" => "north",
        "west" => "east",
        "east" => "west",
        _ => anyhow::bail!("invalid river connection direction"),
    })
}
fn edge(map: &MapModule, side: &str) -> Result<Vec<[u16; 2]>> {
    let w = map.attributes.width * 2;
    let h = map.attributes.height * 2;
    Ok(match side {
        "north" => (0..w).map(|x| [x, 0]).collect(),
        "south" => (0..w).map(|x| [x, h - 1]).collect(),
        "west" => (0..h).map(|y| [0, y]).collect(),
        "east" => (0..h).map(|y| [w - 1, y]).collect(),
        _ => anyhow::bail!("invalid river edge"),
    })
}
fn crossing(
    pack: &CompiledGamePack,
    map: &MapModule,
    at: [u16; 2],
    side: &str,
) -> Result<Option<(String, [u16; 2])>> {
    let x = i16::try_from(at[0])?;
    let y = i16::try_from(at[1])?;
    let out = match side {
        "north" => TilePosition::new(x, -1),
        "south" => TilePosition::new(x, y + 1),
        "west" => TilePosition::new(-1, y),
        "east" => TilePosition::new(x + 1, y),
        _ => anyhow::bail!("invalid river crossing"),
    };
    let native = OverworldMapData::from_attributes(&map.id, &map.attributes, map.blocks.clone());
    let Some(transition) = connection_for_tile_checked(&native, out)? else {
        return Ok(None);
    };
    let destination = pack
        .data
        .resolve_connection_transition(&transition)?
        .destination;
    let point = [
        u16::try_from(destination.tile.x)?,
        u16::try_from(destination.tile.y)?,
    ];
    let target = pack
        .data
        .maps
        .get(&destination.map_name)
        .context("river target")?;
    ensure!(
        point[0] < target.attributes.width * 2 && point[1] < target.attributes.height * 2,
        "river edge projects outside neighbor"
    );
    Ok(Some((destination.map_name, point)))
}
fn adjacent_windows(a: &NewMap, b: &NewMap, side: &str, offset: i32) -> bool {
    if a.layout != b.layout || a.layout_sha256 != b.layout_sha256 || a.tileset != b.tileset {
        return false;
    }
    let [x, y] = a.source_origin_metatiles.map(i32::from);
    let [u, v] = b.source_origin_metatiles.map(i32::from);
    match side {
        "east" => u == x + i32::from(a.dimensions_metatiles[0]) && v == y + offset,
        "west" => u + i32::from(b.dimensions_metatiles[0]) == x && v == y + offset,
        "north" => v + i32::from(b.dimensions_metatiles[1]) == y && u == x + offset,
        "south" => v == y + i32::from(a.dimensions_metatiles[1]) && u == x + offset,
        _ => false,
    }
}

fn seam(
    pack: &CompiledGamePack,
    map: &MapModule,
    side: &str,
    target: &str,
    offset: i32,
    same_source: bool,
) -> Result<()> {
    let neighbor = pack.data.maps.get(target).context("reciprocal river map")?;
    ensure!(
        neighbor
            .attributes
            .connections
            .iter()
            .any(|c| c.direction == reverse(side).unwrap_or("")
                && c.target_map == map.id
                && Some(c.offset) == offset.checked_neg()),
        "river seam must be reciprocal"
    );
    let mut lanes = 0;
    for at in edge(map, side)? {
        if !water(kind(pack, map, at)?) {
            continue;
        }
        let Some((name, to)) = crossing(pack, map, at, side)? else {
            continue;
        };
        ensure!(name == target, "river transition differs from contract");
        if water(kind(pack, neighbor, to)?) {
            let back = crossing(pack, neighbor, to, reverse(side)?)?
                .context("reciprocal river projection")?;
            ensure!(
                back == (map.id.clone(), at),
                "river seam reverse projection differs"
            );
            lanes += 1;
        }
    }
    ensure!(
        lanes > 0 || same_source,
        "external river port has no reciprocal Water lanes"
    );
    if lanes == 0 {
        // Camera-adjacent pieces of one exact source canvas may meet along a
        // closed pine bank. This is not a river port: any non-solid crossing
        // must be an ordinary matching walking pair, never a fabricated water
        // passage. Unconnected/out-of-range passable edges still fail below.
        for at in edge(map, side)? {
            let source_kind = kind(pack, map, at)?;
            if matches!(source_kind, "WALL" | "BUOY") {
                continue;
            }
            let Some((name, to)) = crossing(pack, map, at, side)? else {
                anyhow::bail!("internal source edge has no neighbor");
            };
            ensure!(name == target, "internal source edge target differs");
            let target_kind = kind(pack, neighbor, to)?;
            ensure!(
                matches!(target_kind, "WALL" | "BUOY")
                    || (matches!(source_kind, "FLOOR" | "TALL_GRASS")
                        && matches!(target_kind, "FLOOR" | "TALL_GRASS")),
                "zero-Water source adjacency is neither a closed bank nor a walking pair"
            );
        }
    }
    Ok(())
}
fn perimeter(pack: &CompiledGamePack, map: &MapModule) -> Result<()> {
    for side in ["north", "south", "east", "west"] {
        for at in edge(map, side)? {
            if !matches!(
                kind(pack, map, at)?,
                "FLOOR"
                    | "TALL_GRASS"
                    | "WATER"
                    | "BRIDGE_NS_DECK"
                    | "BRIDGE_NS_RAIL"
                    | "BRIDGE_EW_DECK"
                    | "BRIDGE_EW_RAIL"
            ) {
                continue;
            }
            // A real neighboring solid bank closes the shoreline. Water need not
            // continue across every internal source chunk seam.
            ensure!(
                crossing(pack, map, at, side)?.is_some(),
                "{} unconnected river walking/Surf edge {at:?}",
                map.id
            );
        }
    }
    Ok(())
}
fn placements(pack: &CompiledGamePack, plan: &Manifest) -> Result<()> {
    let mut placed = BTreeMap::from([("MnMinneapolis".to_owned(), [0i32, 0i32])]);
    let mut queue = VecDeque::from(["MnMinneapolis".to_owned()]);
    let new = plan
        .new_maps
        .iter()
        .map(|m| m.id.as_str())
        .collect::<BTreeSet<_>>();
    while let Some(name) = queue.pop_front() {
        let from = &pack.data.maps[&name];
        let origin = placed[&name];
        for link in &from.attributes.connections {
            let to = pack
                .data
                .maps
                .get(&link.target_map)
                .context("placed river target")?;
            let at = match link.direction.as_str() {
                "east" => [
                    origin[0] + i32::from(from.attributes.width),
                    origin[1] + link.offset,
                ],
                "west" => [
                    origin[0] - i32::from(to.attributes.width),
                    origin[1] + link.offset,
                ],
                "north" => [
                    origin[0] + link.offset,
                    origin[1] - i32::from(to.attributes.height),
                ],
                "south" => [
                    origin[0] + link.offset,
                    origin[1] + i32::from(from.attributes.height),
                ],
                _ => anyhow::bail!("invalid map placement direction"),
            };
            if let Some(old) = placed.get(&link.target_map) {
                if new.contains(name.as_str()) || new.contains(link.target_map.as_str()) {
                    ensure!(
                        *old == at,
                        "new river connections imply inconsistent world placement"
                    );
                }
            } else {
                placed.insert(link.target_map.clone(), at);
                queue.push_back(link.target_map.clone());
            }
        }
    }
    for source in &plan.new_maps {
        ensure!(
            placed.get(&source.id) == Some(&source.world_origin_metatiles),
            "new river map is detached or world origin differs: {}",
            source.id
        );
    }
    for source in &plan.new_maps {
        let [x, y] = source.world_origin_metatiles;
        for (name, &[a, b]) in &placed {
            if name == &source.id {
                continue;
            }
            let other = &pack.data.maps[name];
            ensure!(
                x + i32::from(source.dimensions_metatiles[0]) <= a
                    || a + i32::from(other.attributes.width) <= x
                    || y + i32::from(source.dimensions_metatiles[1]) <= b
                    || b + i32::from(other.attributes.height) <= y,
                "new river map overlaps installed world footprint: {} / {name}",
                source.id
            );
        }
    }
    Ok(())
}

fn preserve(
    base: &CompiledGamePack,
    pack: &CompiledGamePack,
    plan: &Manifest,
    bridges: &BridgeInventory,
) -> Result<()> {
    let mut normalized = pack.data.clone();
    for source in &plan.new_maps {
        let map = normalized
            .maps
            .remove(&source.id)
            .context("added river map")?;
        normalized.map_attributes.remove(&source.id);
        normalized.map_dimensions.remove(&source.map_constant);
        normalized.runtime_map_metadata.remove(&source.map_constant);
        normalized.map_blocks.remove(
            map.attributes
                .blocks_label
                .as_ref()
                .context("river block label")?,
        );
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
    }
    for (name, old) in &base.data.maps {
        let actual = pack.data.maps.get(name).context("preserved old map")?;
        let mut expected = old.clone();
        expected.blocks = actual.blocks.clone();
        expected.attributes.connections = actual.attributes.connections.clone();
        expected.attributes.connection_flags = actual.attributes.connection_flags.clone();
        ensure!(
            &expected == actual,
            "old story/room/actor/program changed: {name}"
        );
        // Unchanged legacy maps may contain off-map dormant source objects.
        // Their complete modules and all old collision entries are preserved;
        // only revised terrain needs fresh indexed-anchor collision checks.
        if actual.blocks != old.blocks {
            for at in old
                .objects
                .iter()
                .map(|o| [o.x, o.y])
                .chain(old.events.warps.iter().map(|o| [o.x, o.y]))
                .chain(old.events.coord_events.iter().map(|o| [o.x, o.y]))
                .chain(old.events.bg_events.iter().map(|o| [o.x, o.y]))
            {
                ensure!(
                    foot(kind(pack, actual, at)?) == foot(kind(base, old, at)?),
                    "old native anchor collision changed: {name} {at:?}"
                );
            }
        }
        normalized.maps.insert(name.clone(), old.clone());
        normalized
            .map_attributes
            .insert(name.clone(), old.attributes.clone());
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("old river block label")?;
        normalized
            .map_blocks
            .insert(label.clone(), base.data.map_blocks[label].clone());
    }
    normalized
        .story_event_script_constants
        .global
        .remove(&plan.registry.map_group_constant);
    for actor in &plan.objects {
        if let Some(flag) = &actor.event_flag {
            normalized.story_event_script_constants.global.remove(flag);
        }
        if actor.role == "optional_trainer" {
            normalized
                .story_event_script_constants
                .global
                .remove(&actor.id);
            normalized.trainers.trainers.remove(&actor.id);
        }
    }
    let aliases = bridges
        .bridges
        .iter()
        .map(|b| b.tileset.as_str())
        .collect::<BTreeSet<_>>();
    for alias in &aliases {
        normalized
            .tilesets
            .insert((*alias).to_owned(), base.data.tilesets[*alias].clone());
    }
    ensure!(
        serde_json::to_value(&normalized)? == serde_json::to_value(&base.data)?,
        "unrelated game data changed outside bounded river additions"
    );
    let mut files = pack.runtime_files.clone();
    for alias in aliases {
        for path in [
            format!("data/tilesets/{alias}.json"),
            format!("data/tilesets/{alias}_metatiles.bin"),
        ] {
            match base.runtime_files.get(&path) {
                Some(original) => {
                    files.insert(path, original.clone());
                }
                None => {
                    // Original native tilesets may embed their collision table
                    // without a runtime JSON sidecar. The bridge writer adds
                    // that exact owned sidecar; it is not an unrelated file.
                    ensure!(path.ends_with(".json"), "missing original bridge metatiles");
                    files.remove(&path);
                }
            }
        }
    }
    ensure!(
        files == base.runtime_files
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_manifest == base.audio_manifest
            && pack.audio_compression == base.audio_compression,
        "unrelated art/audio bytes changed"
    );
    let mut report = pack.report.clone();
    report.maps = base.report.maps;
    report.manifests = base.report.manifests.clone();
    report.reachable_maps = base.report.reachable_maps.clone();
    report.solvable_maps = base.report.solvable_maps.clone();
    ensure!(report == base.report, "unrelated compile report changed");
    Ok(())
}
