//! Immutable regional worlds. Player state stays in the runtime save format.
use crate::{GeneratedGrid, H3BatchConnections, ModpackOptions};
use anyhow::{Context, Result, ensure};
use crystal_assets::{modpack::CompiledMapExtension, read_verified_compiled_game_pack};
use crystal_core::map::{MapEventSectionCommand, WarpEvent};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldGenerationIdentity {
    pub generator_version: u32,
    pub base_content_hash: String,
    pub source_revision: String,
    pub grid_width: u16,
    pub grid_height: u16,
    pub world_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapAllocation {
    pub map_name: String,
    pub map_constant: String,
    pub group_id: u16,
    pub map_id: u16,
    pub spawn_id: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldMapRegistry {
    pub maps: BTreeMap<String, MapAllocation>,
}

impl WorldMapRegistry {
    /// Sorted semantic keys, sequential numeric IDs, and explicit collision checks.
    /// Existing allocations are never reassigned when new cells are added.
    pub fn allocate(
        &mut self,
        keys: impl IntoIterator<Item = String>,
        occupied_maps: &BTreeSet<(u16, u16)>,
        occupied_spawns: &BTreeSet<u16>,
    ) -> Result<()> {
        let mut allocated = self.maps.clone();
        let mut used_maps = occupied_maps.clone();
        let mut used_spawns = occupied_spawns.clone();
        let mut names = BTreeSet::new();
        for (key, value) in &self.maps {
            ensure!(
                value.map_name == format!("World_{key}")
                    && value.map_constant == format!("WORLD_{}", key.to_ascii_uppercase()),
                "registry map name differs from its semantic key"
            );
            ensure!(
                value.group_id >= 250
                    && value.group_id <= i16::MAX as u16
                    && (1..=255).contains(&value.map_id)
                    && value.spawn_id > 0,
                "registry contains invalid runtime identifiers"
            );
            ensure!(
                names.insert(value.map_constant.clone()),
                "registry has duplicate map names"
            );
            ensure!(
                used_maps.insert((value.group_id, value.map_id))
                    && used_spawns.insert(value.spawn_id),
                "map registry collides with existing content"
            );
        }
        for key in keys.into_iter().collect::<BTreeSet<_>>() {
            if allocated.contains_key(&key) {
                continue;
            }
            ensure!(
                !key.is_empty() && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'),
                "invalid map key {key}"
            );
            ensure!(
                names.insert(format!("WORLD_{}", key.to_ascii_uppercase())),
                "map keys collide when converted to constants"
            );
            let (group_id, map_id) = (250..=i16::MAX as u16)
                .flat_map(|group| (1..=255).map(move |map| (group, map)))
                .find(|id| !used_maps.contains(id))
                .context("runtime map identifiers exhausted")?;
            let spawn_id = (1..=u16::MAX)
                .rev()
                .find(|id| !used_spawns.contains(id))
                .context("spawn identifiers exhausted")?;
            used_maps.insert((group_id, map_id));
            used_spawns.insert(spawn_id);
            allocated.insert(
                key.clone(),
                MapAllocation {
                    map_name: format!("World_{key}"),
                    map_constant: format!("WORLD_{}", key.to_ascii_uppercase()),
                    group_id,
                    map_id,
                    spawn_id,
                },
            );
        }
        self.maps = allocated;
        Ok(())
    }
}

pub struct RegionBuildOptions<'a> {
    pub base_pack: &'a Path,
    pub output_pack: &'a Path,
    pub start_cell: &'a str,
    pub registry: WorldMapRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRegion {
    pub identity: WorldGenerationIdentity,
    pub registry: WorldMapRegistry,
    pub start_map: String,
    pub pack_path: String,
}

fn digest<T: Serialize>(value: &T) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}

pub fn build_region_modpack(
    grids: &[GeneratedGrid],
    links: &H3BatchConnections,
    mut options: RegionBuildOptions<'_>,
) -> Result<GeneratedRegion> {
    let links = canonical_links(links);
    ensure!(!grids.is_empty(), "a region needs at least one cell");
    let base = read_verified_compiled_game_pack(options.base_pack)?;
    let base_identity = base.identity()?;
    let mut cells = BTreeMap::new();
    for grid in grids {
        let cell = &grid
            .source
            .h3
            .as_ref()
            .context("region cells require H3 plans")?
            .cell;
        ensure!(
            grid.width == links.grid_width && grid.height == links.grid_height,
            "regional grid dimensions differ"
        );
        ensure!(
            cells.insert(cell.clone(), grid).is_none(),
            "duplicate region cell {cell}"
        );
        let audit = crate::audit_grid(grid);
        ensure!(
            audit.passed,
            "cell {cell} failed its playable scene audit: {}",
            audit.errors.join("; ")
        );
    }
    ensure!(
        cells.contains_key(options.start_cell),
        "start cell is absent"
    );
    validate_graph(&cells.keys().cloned().collect(), &links, options.start_cell)?;
    let source_revision = digest(
        &cells
            .iter()
            .map(|(id, grid)| (id, &grid.source))
            .collect::<Vec<_>>(),
    )?;
    let world_id = digest(&(
        crate::GENERATOR_VERSION,
        &base_identity.content_hash,
        &source_revision,
        &links,
        options.start_cell,
    ))?;
    let identity = WorldGenerationIdentity {
        generator_version: crate::GENERATOR_VERSION,
        base_content_hash: base_identity.content_hash,
        source_revision,
        grid_width: links.grid_width,
        grid_height: links.grid_height,
        world_id,
    };
    let manifest = format!("geographic-world-{}", identity.world_id);
    let mut tileset =
        crate::build_johto_modern_generated_tileset_extension(&base, format!("{manifest}-tiles"))?;
    let portal_block = u16::try_from(tileset.metatiles.len() / 16)?;
    ensure!(portal_block < 256, "no metatile slot for regional portals");
    // Same pixels as the trail. All four quadrants activate a deliberate portal.
    let trail = tileset.metatiles[7 * 16..8 * 16].to_vec();
    tileset.metatiles.extend(trail);
    tileset
        .definition
        .collision
        .insert(format!("{portal_block:02x}"), vec!["WARP_PANEL".into(); 4]);
    let extended = base.with_tileset_extension(tileset)?;
    let mut batches = BTreeMap::new();
    for (cell, grid) in &cells {
        let opts = ModpackOptions {
            base_pack: options.base_pack,
            output_pack: options.output_pack,
            manifest_id: &manifest,
            start_new_game_here: cell == options.start_cell,
        };
        let extensions = crate::modpack::map_extensions(&extended, grid, &opts)?;
        batches.insert(cell.clone(), extensions);
    }
    let keys = batches
        .iter()
        .flat_map(|(cell, maps)| maps.iter().map(move |m| format!("{cell}_{}", m.map_name)))
        .collect::<Vec<_>>();
    let occupied_maps = base
        .data()
        .runtime_map_metadata
        .values()
        .map(|m| (m.group_id, m.map_id))
        .collect();
    let occupied_spawns = base
        .data()
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .collect();
    options
        .registry
        .allocate(keys, &occupied_maps, &occupied_spawns)?;
    let mut exteriors = BTreeMap::new();
    let mut extensions = Vec::new();
    for (cell, maps) in batches {
        let names = maps
            .iter()
            .flat_map(|m| {
                let allocation = &options.registry.maps[&format!("{cell}_{}", m.map_name)];
                [
                    (m.map_name.clone(), allocation.map_name.clone()),
                    (m.map_constant.clone(), allocation.map_constant.clone()),
                ]
            })
            .collect::<BTreeMap<_, _>>();
        exteriors.insert(cell.clone(), extensions.len());
        for mut extension in maps {
            let allocation = &options.registry.maps[&format!("{cell}_{}", extension.map_name)];
            let mut value = serde_json::to_value(&extension.module)?;
            namespace(&mut value, &cell, &names);
            extension.module = serde_json::from_value(value)?;
            extension.map_name = allocation.map_name.clone();
            extension.map_constant = allocation.map_constant.clone();
            extension.module.id = allocation.map_name.clone();
            extension.metadata.name = allocation.map_name.clone();
            extension.metadata.constant = allocation.map_constant.clone();
            extension.metadata.group_id = allocation.group_id;
            extension.metadata.map_id = allocation.map_id;
            let group = format!("WORLD_GROUP_{}", allocation.group_id);
            extension.metadata.group_name = group.clone();
            extension.module.attributes.map_group_constant = Some(group.clone());
            extension.spawn.group_name = group;
            extension.spawn.map_name = allocation.map_name.clone();
            extension.spawn.map_constant = allocation.map_constant.clone();
            extension.spawn.group_id = i16::try_from(allocation.group_id)?;
            extension.spawn.map_id = i16::try_from(allocation.map_id)?;
            extension.spawn.identifier = allocation.spawn_id;
            extension.spawn_key = allocation.spawn_id.to_string();
            extension.manifest_id = format!("{manifest}-{}", allocation.map_name);
            if let Some(encounters) = &mut extension.wild_encounters {
                encounters.map_name = allocation.map_name.clone();
            }
            extensions.push(extension);
        }
    }
    let mut occupied = BTreeMap::<String, BTreeSet<(u16, u16)>>::new();
    for (cell, index) in &exteriors {
        occupied.insert(
            cell.clone(),
            extensions[*index]
                .module
                .events
                .warps
                .iter()
                .map(|w| (w.x / 2, w.y / 2))
                .collect(),
        );
    }
    for (cell, index) in &exteriors {
        let taken = occupied.get_mut(cell).unwrap();
        for object in &extensions[*index].module.objects {
            for y in object.y.saturating_sub(2)..=object.y.saturating_add(2) {
                for x in object.x.saturating_sub(2)..=object.x.saturating_add(2) {
                    taken.insert((x / 2, y / 2));
                }
            }
        }
    }
    for link in &links.links {
        for (cell, neighbor, side, gate) in [
            (
                &link.first_cell,
                &link.second_cell,
                link.first_side,
                link.first_gate,
            ),
            (
                &link.second_cell,
                &link.first_cell,
                link.second_side,
                link.second_gate,
            ),
        ] {
            let grid = cells[cell];
            let plan = grid.source.h3.as_ref().unwrap();
            let portal = plan
                .portals
                .iter()
                .find(|p| p.edge_id == link.edge_id && &p.neighbor == neighbor && p.side == side)
                .context("link differs from the H3 edge contract")?;
            let connection = plan
                .regional
                .as_ref()
                .context("region cell lacks a regional plan")?
                .connections
                .iter()
                .find(|c| c.edge_id == portal.edge_id && !c.boundary_exit)
                .context("runtime link was not selected by the region plan")?;
            ensure!(
                gate == crate::h3::h3_raster_landing(
                    plan,
                    grid.width,
                    grid.height,
                    connection.coordinate
                )?,
                "link does not land on the selected crossing"
            );
            ensure!(
                occupied.get_mut(cell).unwrap().insert(gate),
                "regional gates overlap another entrance"
            );
        }
    }
    for link in &links.links {
        let first = exteriors[&link.first_cell];
        let second = exteriors[&link.second_cell];
        let first_constant = extensions[first].map_constant.clone();
        let second_constant = extensions[second].map_constant.clone();
        let first_landing = next_warp_index(&extensions[first])? + 4;
        let second_landing = next_warp_index(&extensions[second])? + 4;
        add_portal(
            &mut extensions[first],
            cells[&link.first_cell],
            link.first_gate,
            &second_constant,
            second_landing,
            portal_block,
            occupied.get_mut(&link.first_cell).unwrap(),
        )?;
        add_portal(
            &mut extensions[second],
            cells[&link.second_cell],
            link.second_gate,
            &first_constant,
            first_landing,
            portal_block,
            occupied.get_mut(&link.second_cell).unwrap(),
        )?;
    }
    let start_map = extensions[exteriors[options.start_cell]].map_name.clone();
    let packed = extended.with_map_extensions(extensions)?;
    // Pack assembly must not change original maps, runtime files, or audio.
    for (key, map) in &base.data().maps {
        ensure!(
            packed.data().maps.get(key) == Some(map),
            "original map changed: {key}"
        );
    }
    for (key, bytes) in base.runtime_files() {
        ensure!(
            packed.runtime_files().get(key) == Some(bytes),
            "original runtime asset changed: {key}"
        );
    }
    ensure!(
        base.compiled_audio() == packed.compiled_audio()
            && serde_json::to_vec(&base.audio_manifest()?)?
                == serde_json::to_vec(&packed.audio_manifest()?)?,
        "original audio changed"
    );
    if let Some(parent) = options.output_pack.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if options.output_pack.exists() {
        let published = read_verified_compiled_game_pack(options.output_pack)?;
        ensure!(
            published.identity()? == packed.identity()?,
            "published world differs; choose a new output directory to preserve its saves"
        );
        return Ok(GeneratedRegion {
            identity,
            registry: options.registry,
            start_map,
            pack_path: options.output_pack.display().to_string(),
        });
    }
    let temporary = options.output_pack.with_extension("pending.crystalpack");
    let result = packed
        .write_preserving_storage(&temporary)
        .and_then(|_| std::fs::rename(&temporary, options.output_pack).map_err(Into::into));
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result?;
    Ok(GeneratedRegion {
        identity,
        registry: options.registry,
        start_map,
        pack_path: options.output_pack.display().to_string(),
    })
}

fn next_warp_index(extension: &CompiledMapExtension) -> Result<u16> {
    let index = extension
        .module
        .events
        .warps
        .iter()
        .map(|w| w.index)
        .max()
        .unwrap_or(0);
    ensure!(
        index < i16::MAX as u16 - 5,
        "map warp identifiers exhausted"
    );
    Ok(index + 1)
}

fn add_portal(
    extension: &mut CompiledMapExtension,
    grid: &GeneratedGrid,
    gate: (u16, u16),
    target: &str,
    landing_id: u16,
    portal_block: u16,
    occupied: &mut BTreeSet<(u16, u16)>,
) -> Result<()> {
    let (x, y) = gate;
    let landing = portal_landing(grid, gate, occupied)?;
    occupied.insert(landing);
    let index = next_warp_index(extension)?;
    for dy in 0..2 {
        for dx in 0..2 {
            extension.module.events.warps.push(WarpEvent {
                index: index + dy * 2 + dx,
                x: x * 2 + dx,
                y: y * 2 + dy,
                target_map_constant: target.into(),
                target_map: target.into(),
                target_warp_id: i16::try_from(landing_id)?,
            });
        }
    }
    // A destination record on ordinary FLOOR supplies a safe non-triggering landing.
    extension.module.events.warps.push(WarpEvent {
        index: index + 4,
        x: landing.0 * 2 + 1,
        y: landing.1 * 2 + 1,
        target_map_constant: target.into(),
        target_map: target.into(),
        target_warp_id: i16::try_from(landing_id)?,
    });
    extension.module.blocks[usize::from(y) * usize::from(grid.width) + usize::from(x)] =
        portal_block;
    refresh_warps(&mut extension.module);
    Ok(())
}

fn portal_landing(
    grid: &GeneratedGrid,
    gate: (u16, u16),
    occupied: &BTreeSet<(u16, u16)>,
) -> Result<(u16, u16)> {
    let (x, y) = gate;
    ensure!(
        x < grid.width && y < grid.height,
        "portal is outside its map"
    );
    let reached = crate::grid::reachable_walkable_cells(grid, grid.home_cell());
    ensure!(
        reached[usize::from(y) * usize::from(grid.width) + usize::from(x)],
        "portal cannot be reached from the cell spawn"
    );
    [
        (x.checked_sub(1), Some(y)),
        (x.checked_add(1), Some(y)),
        (Some(x), y.checked_sub(1)),
        (Some(x), y.checked_add(1)),
    ]
    .into_iter()
    .filter_map(|(x, y)| Some((x?, y?)))
    .filter(|&(x, y)| x < grid.width && y < grid.height)
    .filter(|p| !occupied.contains(p))
    .filter(|&(x, y)| {
        matches!(
            grid.cell(x, y),
            Some(
                crate::MapCell::Grass
                    | crate::MapCell::Lawn
                    | crate::MapCell::Clearing
                    | crate::MapCell::Trail
                    | crate::MapCell::Street
                    | crate::MapCell::Road
                    | crate::MapCell::MajorRoad
            )
        )
    })
    .filter(|&(x, y)| reached[usize::from(y) * usize::from(grid.width) + usize::from(x)])
    .min_by_key(|&(x, y)| x.abs_diff(grid.width / 2) + y.abs_diff(grid.height / 2))
    .context("portal has no safe inward landing")
}

fn canonical_links(links: &H3BatchConnections) -> H3BatchConnections {
    let mut links = links.clone();
    for link in &mut links.links {
        if link.first_cell > link.second_cell {
            std::mem::swap(&mut link.first_cell, &mut link.second_cell);
            std::mem::swap(&mut link.first_side, &mut link.second_side);
            std::mem::swap(&mut link.first_gate, &mut link.second_gate);
        }
        link.first_ordinal = 0;
        link.second_ordinal = 0;
    }
    links.links.sort_by(|a, b| a.edge_id.cmp(&b.edge_id));
    links
}

pub(crate) fn refresh_warps(module: &mut crystal_assets::modpack::MapModule) {
    module
        .map_event_section_commands
        .retain(|c| c.command != "warp_event" && c.command != "def_warp_events");
    let mut commands = vec![MapEventSectionCommand {
        command: "def_warp_events".into(),
        args: vec![],
        command_index: 0,
    }];
    commands.extend(module.events.warps.iter().map(|w| MapEventSectionCommand {
        command: "warp_event".into(),
        args: vec![
            w.x.to_string(),
            w.y.to_string(),
            w.target_map_constant.clone(),
            w.target_warp_id.to_string(),
        ],
        command_index: 0,
    }));
    commands.append(&mut module.map_event_section_commands);
    for (index, command) in commands.iter_mut().enumerate() {
        command.command_index = index;
    }
    module.map_event_section_commands = commands;
}

fn namespace(value: &mut serde_json::Value, cell: &str, names: &BTreeMap<String, String>) {
    fn rename(value: &str, cell: &str, names: &BTreeMap<String, String>) -> String {
        if let Some(name) = names.get(value) {
            name.clone()
        } else if value.starts_with("Generated") || value.starts_with("GENERATED_") {
            format!("G{cell}_{value}")
        } else {
            value.into()
        }
    }
    match value {
        serde_json::Value::String(s) => *s = rename(s, cell, names),
        serde_json::Value::Array(values) => {
            for v in values {
                namespace(v, cell, names);
            }
        }
        serde_json::Value::Object(values) => {
            let old = std::mem::take(values);
            for (key, mut v) in old {
                namespace(&mut v, cell, names);
                values.insert(rename(&key, cell, names), v);
            }
        }
        _ => {}
    }
}

fn validate_graph(cells: &BTreeSet<String>, links: &H3BatchConnections, start: &str) -> Result<()> {
    let mut edges = BTreeSet::new();
    let mut neighbors = BTreeMap::<&str, Vec<&str>>::new();
    for link in &links.links {
        ensure!(
            cells.contains(&link.first_cell)
                && cells.contains(&link.second_cell)
                && link.first_cell != link.second_cell,
            "invalid regional link endpoints"
        );
        ensure!(edges.insert(&link.edge_id), "duplicate regional edge");
        ensure!(
            link.first_side.opposite() == link.second_side,
            "non-reciprocal regional sides"
        );
        neighbors
            .entry(&link.first_cell)
            .or_default()
            .push(&link.second_cell);
        neighbors
            .entry(&link.second_cell)
            .or_default()
            .push(&link.first_cell);
    }
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        for next in neighbors.get(cell).into_iter().flatten() {
            if seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    ensure!(
        seen.len() == cells.len(),
        "regional link graph is disconnected"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portal_landing_avoids_other_gates_and_rejects_trapped_gate() {
        let source = crate::MapSource {
            schema_version: crate::SOURCE_SCHEMA_VERSION,
            center: crate::Coordinate { lat: 0.5, lon: 0.5 },
            bounds: crate::BoundingBox {
                south: 0.,
                west: 0.,
                north: 1.,
                east: 1.,
            },
            attribution: "fixture".into(),
            features: vec![],
            h3: None,
        };
        let mut grid = GeneratedGrid {
            source,
            width: 24,
            height: 24,
            cells: vec![crate::MapCell::Grass; 24 * 24],
            labels: vec![],
            scene: None,
        };
        let gates = BTreeSet::from([(10, 10), (11, 10)]);
        let landing = portal_landing(&grid, (10, 10), &gates).unwrap();
        assert!(!gates.contains(&landing));
        assert_eq!(landing.0.abs_diff(10) + landing.1.abs_diff(10), 1);
        for (x, y) in [(9, 10), (11, 10), (10, 9), (10, 11)] {
            grid.cells[y * 24 + x] = crate::MapCell::Water;
        }
        assert!(portal_landing(&grid, (10, 10), &gates).is_err());
    }

    #[test]
    fn registry_rejects_case_collisions_and_tampered_allocations() {
        let mut registry = WorldMapRegistry::default();
        assert!(
            registry
                .allocate(["a".into(), "A".into()], &BTreeSet::new(), &BTreeSet::new())
                .is_err()
        );
        assert!(registry.maps.is_empty());
        registry
            .allocate(["a".into()], &BTreeSet::new(), &BTreeSet::new())
            .unwrap();
        let value = registry.maps.values_mut().next().unwrap();
        value.map_name = "wrong".into();
        assert!(
            registry
                .allocate([], &BTreeSet::new(), &BTreeSet::new())
                .is_err()
        );
    }

    #[test]
    fn link_identity_ignores_iteration_order_and_endpoint_orientation() {
        let mut first = crate::H3BatchLink {
            edge_id: "a".into(),
            first_ordinal: 9,
            first_cell: "b".into(),
            first_side: crate::HexSide::NorthEast,
            first_gate: (1, 2),
            second_ordinal: 4,
            second_cell: "c".into(),
            second_side: crate::HexSide::SouthWest,
            second_gate: (3, 4),
        };
        let a = H3BatchConnections {
            schema_version: 1,
            grid_width: 96,
            grid_height: 96,
            links: vec![first.clone()],
        };
        std::mem::swap(&mut first.first_cell, &mut first.second_cell);
        std::mem::swap(&mut first.first_side, &mut first.second_side);
        std::mem::swap(&mut first.first_gate, &mut first.second_gate);
        first.first_ordinal = 1;
        first.second_ordinal = 2;
        let mut b = a.clone();
        b.links = vec![first];
        assert_eq!(canonical_links(&a), canonical_links(&b));
    }

    #[test]
    fn registry_is_order_independent_and_preserves_allocations() {
        let mut a = WorldMapRegistry::default();
        let mut b = a.clone();
        let occupied = BTreeSet::from([(250, 1)]);
        let spawns = BTreeSet::from([u16::MAX]);
        a.allocate(["b".into(), "a".into()], &occupied, &spawns)
            .unwrap();
        b.allocate(["a".into(), "b".into()], &occupied, &spawns)
            .unwrap();
        assert_eq!(a, b);
        let old = a.maps["a"].clone();
        a.allocate(["c".into()], &occupied, &spawns).unwrap();
        assert_eq!(old, a.maps["a"]);
        assert_eq!(old.map_id, 2);
        assert_eq!(old.spawn_id, u16::MAX - 1);
    }
    #[test]
    fn namespace_keeps_global_scripts_and_changes_local_keys_and_references() {
        let mut value = serde_json::json!({"GeneratedResident1Script":["GeneratedResident1Text","PokecenterNurseScript","GENERATED_NEIGHBORHOOD"]});
        namespace(
            &mut value,
            "abc",
            &BTreeMap::from([("GENERATED_NEIGHBORHOOD".into(), "WORLD_ABC".into())]),
        );
        assert_eq!(
            value["Gabc_GeneratedResident1Script"][1],
            "PokecenterNurseScript"
        );
        assert_eq!(value["Gabc_GeneratedResident1Script"][2], "WORLD_ABC");
    }
}
