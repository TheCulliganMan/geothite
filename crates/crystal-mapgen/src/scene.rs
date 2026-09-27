//! Semantic composition before lowering to Crystal metatiles.
//! Recipes reference the external pack; they never embed a second art catalog.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Feature, FeatureKind, GeneratedGrid, MapCell, MapSource, stable_grid::StableGrid};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualFamily {
    Urban,
    Residential,
    Waterfront,
    Woodland,
    #[default]
    Meadow,
    Rocky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureRecipe {
    ModernHouse,
    TraditionalHouse,
    RedHouse,
    YellowHouse,
    DepartmentStore,
    RadioTower,
}

impl StructureRecipe {
    pub fn dimensions(self) -> (u16, u16) {
        match self {
            Self::DepartmentStore => (3, 4),
            Self::RadioTower => (2, 6),
            _ => (2, 2),
        }
    }

    pub fn block(self, x: u16, y: u16) -> Option<u16> {
        let (w, h) = self.dimensions();
        if x >= w || y >= h {
            return None;
        }
        Some(match self {
            Self::ModernHouse => [[0x18, 0x19], [0x16, 0x1e]][y as usize][x as usize],
            Self::TraditionalHouse => [[0x97, 0x98], [0x99, 0x9a]][y as usize][x as usize],
            Self::RedHouse => [[0x8f, 0x90], [0x91, 0x92]][y as usize][x as usize],
            Self::YellowHouse => [[0x93, 0x94], [0x95, 0x96]][y as usize][x as usize],
            Self::DepartmentStore => [
                [0x18, 0x1f, 0x19],
                [0x27, 0x23, 0x28],
                [0x27, 0x23, 0x28],
                [0x10, 0x17, 0x33],
            ][y as usize][x as usize],
            Self::RadioTower => match (x, y) {
                (0, 0) => 0x21,
                (0, 1 | 2) => 0x22,
                (0, 3) => 0x25,
                (1, 3) => 0x26,
                (0, 4) => 0x29,
                (1, 4) => 0x2a,
                (0, 5) => 0x2d,
                (1, 5) => 0x2e,
                _ => return None,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedStructure {
    pub id: String,
    pub source_id: Option<String>,
    pub recipe: StructureRecipe,
    pub origin: (u16, u16),
    /// Runtime (16-pixel) coordinates of the actual facade door.
    pub door: (u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenePlan {
    pub generator_version: u32,
    pub districts: Vec<VisualFamily>,
    /// Known land in a polygon hole must never be filled as water raster noise.
    pub islands: Vec<bool>,
    /// Mapped building holes remain open, even if the footprint centroid lies there.
    pub courtyards: Vec<bool>,
    /// Structural reservations are frozen before vegetation and prop placement.
    pub reserved: Vec<bool>,
    pub structures: Vec<PlannedStructure>,
    pub destinations: Vec<PlannedDestination>,
    pub spawn: (u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedDestination {
    pub family: VisualFamily,
    pub origin: (u16, u16),
    pub entrance: (u16, u16),
}

pub(crate) fn classify(feature: &Feature) -> Option<VisualFamily> {
    let tags = &feature.details.tags;
    let land = tags.get("landuse").map(String::as_str);
    let natural = tags.get("natural").map(String::as_str);
    if matches!(land, Some("commercial" | "retail" | "industrial")) {
        return Some(VisualFamily::Urban);
    }
    if land == Some("residential") || feature.kind == FeatureKind::Building {
        return Some(VisualFamily::Residential);
    }
    if land == Some("forest") || matches!(natural, Some("wood" | "scrub")) {
        return Some(VisualFamily::Woodland);
    }
    if matches!(natural, Some("bare_rock" | "scree")) {
        return Some(VisualFamily::Rocky);
    }
    if feature.kind == FeatureKind::Water
        || matches!(natural, Some("wetland" | "beach"))
        || tags.contains_key("leisure")
    {
        return Some(VisualFamily::Waterfront);
    }
    if feature.kind == FeatureKind::Park || land.is_some() {
        return Some(VisualFamily::Meadow);
    }
    None
}

pub(crate) fn plan(grid: &GeneratedGrid) -> ScenePlan {
    let mut districts = vec![VisualFamily::Meadow; grid.cells.len()];
    let mut islands = vec![false; grid.cells.len()];
    let mut courtyards = vec![false; grid.cells.len()];
    let mut ranked = grid.source.features.iter().collect::<Vec<_>>();
    // Broad land cover first; buildings and water supply local identity last.
    ranked.sort_by_key(|f| {
        (
            f.kind == FeatureKind::Building,
            f.kind == FeatureKind::Water,
            f.details.osm_id.clone(),
        )
    });
    for feature in ranked {
        let Some(family) = classify(feature) else {
            continue;
        };
        let points = feature
            .points
            .iter()
            .map(|p| crate::grid::project(grid, *p))
            .collect::<Vec<_>>();
        let holes = feature
            .details
            .inner_rings
            .iter()
            .map(|ring| {
                ring.iter()
                    .map(|p| crate::grid::project(grid, *p))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        // Restrict raster work to the projected feature envelope. Large city
        // extracts contain many small footprints, not map-sized polygons.
        if points.is_empty() {
            continue;
        }
        let min_x =
            (points.iter().map(|p| p.0).min().unwrap() - 3).clamp(0, i32::from(grid.width)) as u16;
        let max_x =
            (points.iter().map(|p| p.0).max().unwrap() + 4).clamp(0, i32::from(grid.width)) as u16;
        let min_y =
            (points.iter().map(|p| p.1).min().unwrap() - 3).clamp(0, i32::from(grid.height)) as u16;
        let max_y =
            (points.iter().map(|p| p.1).max().unwrap() + 4).clamp(0, i32::from(grid.height)) as u16;
        for y in min_y..max_y {
            for x in min_x..max_x {
                let i = usize::from(y) * usize::from(grid.width) + usize::from(x);
                let inside_hole = holes.iter().any(|ring| {
                    crate::grid::point_in_polygon(f64::from(x) + 0.5, f64::from(y) + 0.5, ring)
                });
                if feature.kind == FeatureKind::Water && inside_hole {
                    islands[i] = true;
                }
                if feature.kind == FeatureKind::Building && inside_hole {
                    courtyards[i] = true;
                }
                let contained = feature.area
                    && points.len() >= 3
                    && crate::grid::point_in_polygon(
                        f64::from(x) + 0.5,
                        f64::from(y) + 0.5,
                        &points,
                    );
                let nearby = matches!(feature.kind, FeatureKind::Building | FeatureKind::Water)
                    && points.iter().any(|&(px, py)| {
                        (px - i32::from(x)).abs().max((py - i32::from(y)).abs()) <= 3
                    });
                if !inside_hole
                    && (contained || nearby)
                    && !(feature.kind == FeatureKind::Building
                        && districts[i] == VisualFamily::Urban)
                {
                    districts[i] = family;
                }
            }
        }
    }
    ScenePlan {
        generator_version: crate::GENERATOR_VERSION,
        districts,
        islands,
        courtyards,
        reserved: vec![false; grid.cells.len()],
        structures: Vec::new(),
        destinations: Vec::new(),
        spawn: (grid.width / 2, grid.height / 2),
    }
}

/// Complete five-by-four scenes with a south entrance on an existing route.
/// Placement is conditional on context and free space, never a per-map quota.
pub(crate) fn compose_destinations(grid: &mut GeneratedGrid) -> Result<()> {
    use MapCell::*;
    let addressing = StableGrid::for_grid(grid)?;
    let mut candidates = Vec::new();
    for y in 3..grid.height.saturating_sub(7) {
        for x in 3..grid.width.saturating_sub(8) {
            if !matches!(
                grid.cell(x + 2, y + 4),
                Some(Trail | Street | Road | MajorRoad)
            ) {
                continue;
            }
            if !grid.source.h3.as_ref().is_none_or(|p| {
                p.raster_footprint_fits(
                    i32::from(x),
                    i32::from(y),
                    5,
                    4,
                    3,
                    grid.width,
                    grid.height,
                )
                .unwrap_or(false)
            }) {
                continue;
            }
            candidates.push((
                addressing.cell(x, y).unwrap().stable_hash(0x44455354494e),
                x,
                y,
            ));
        }
    }
    candidates.sort_unstable();
    for (_, x, y) in candidates {
        let scene = grid.scene.as_ref().unwrap();
        if x.abs_diff(scene.spawn.0).max(y.abs_diff(scene.spawn.1)) < 6
            || scene
                .destinations
                .iter()
                .any(|d| x.abs_diff(d.origin.0).max(y.abs_diff(d.origin.1)) < 14)
            || !(0..4).all(|dy| {
                (0..5).all(|dx| matches!(grid.cell(x + dx, y + dy), Some(Grass | Lawn | Clearing)))
            })
        {
            continue;
        }
        let family =
            scene.districts[usize::from(y + 1) * usize::from(grid.width) + usize::from(x + 2)];
        // Residential gardens belong to nearby homes; a neighborhood family
        // alone must not scatter identical plazas along every stretch of road.
        if family == VisualFamily::Residential
            && !scene
                .structures
                .iter()
                .any(|s| x.abs_diff(s.origin.0).max(y.abs_diff(s.origin.1)) <= 9)
        {
            continue;
        }
        let drawing = match family {
            VisualFamily::Waterfront => [
                [
                    FenceNorthWest,
                    FenceNorth,
                    FenceNorth,
                    FenceNorth,
                    FenceNorthEast,
                ],
                [FenceWest, Flowers, Bench, Grass, FenceEast],
                [FenceWest, Grass, Grass, Flowers, FenceEast],
                [
                    FenceSouthWest,
                    FenceSouth,
                    Grass,
                    FenceSouth,
                    FenceSouthEast,
                ],
            ],
            VisualFamily::Urban => [
                [Flowers, Trail, GroundSign, Trail, Flowers],
                [Bench, Trail, Fountain, Trail, TrashCan],
                [Trail, Trail, Trail, Trail, Trail],
                [Flowers, Trail, Trail, Trail, Flowers],
            ],
            VisualFamily::Residential => [
                [Tree, Grass, Flowers, Grass, Tree],
                [Grass, Flowers, Grass, Grass, Bench],
                [Grass, Grass, Grass, Flowers, Grass],
                [Flowers, Grass, Grass, Grass, Flowers],
            ],
            VisualFamily::Woodland => [
                [Tree, Tree, ParkTree, Tree, Tree],
                [Tree, Flowers, Grass, Flowers, Tree],
                [SmallTree, Grass, Grass, Grass, SmallTree],
                [SmallTreeSouth, Grass, Grass, Grass, SmallTreeSouth],
            ],
            VisualFamily::Rocky => [
                [Boulder, Grass, Boulder, Grass, Boulder],
                [Grass, LedgeWest, LedgeMiddle, LedgeEast, Grass],
                [Grass, Grass, Grass, Grass, Grass],
                [Boulder, Grass, Grass, Grass, Boulder],
            ],
            VisualFamily::Meadow => continue,
        };
        for (dy, row) in drawing.into_iter().enumerate() {
            for (dx, cell) in row.into_iter().enumerate() {
                grid.cells[(usize::from(y) + dy) * usize::from(grid.width) + usize::from(x) + dx] =
                    cell;
            }
        }
        grid.scene
            .as_mut()
            .unwrap()
            .destinations
            .push(PlannedDestination {
                family,
                origin: (x, y),
                entrance: (x + 2, y + 4),
            });
    }
    Ok(())
}

pub(crate) fn record_structure(grid: &mut GeneratedGrid, x: u16, y: u16, recipe: StructureRecipe) {
    let source_id = grid
        .source
        .features
        .iter()
        .filter(|f| f.kind == FeatureKind::Building)
        .min_by_key(|f| {
            f.points
                .iter()
                .map(|p| {
                    let (px, py) = crate::grid::project(grid, *p);
                    (px - i32::from(x)).abs() + (py - i32::from(y)).abs()
                })
                .min()
                .unwrap_or(i32::MAX)
        })
        .and_then(|f| f.details.osm_id.clone());
    let address = StableGrid::for_grid(grid)
        .ok()
        .and_then(|s| s.cell(x, y))
        .map(|a| a.stable_hash(0x535452554354))
        .unwrap_or(0);
    if let Some(scene) = &mut grid.scene {
        let (w, h) = recipe.dimensions();
        scene.structures.push(PlannedStructure {
            id: format!("structure_{address:016x}"),
            source_id,
            recipe,
            origin: (x, y),
            door: (x * 2 + if w == 3 { 3 } else { 1 }, (y + h) * 2 - 1),
        });
    }
}

pub(crate) fn freeze_and_decorate(grid: &mut GeneratedGrid) -> Result<()> {
    let addressing = StableGrid::for_grid(grid)?;
    let width = usize::from(grid.width);
    let snapshot = grid.cells.clone();
    let mut reserved = snapshot
        .iter()
        .map(|cell| {
            !matches!(
                cell,
                MapCell::Grass
                    | MapCell::Lawn
                    | MapCell::Park
                    | MapCell::Flowers
                    | MapCell::Clearing
            )
        })
        .collect::<Vec<_>>();
    // Keep a one-block apron around paths, water and every complete structure.
    for y in 0..grid.height {
        for x in 0..grid.width {
            if reserved[usize::from(y) * width + usize::from(x)] {
                continue;
            }
            reserved[usize::from(y) * width + usize::from(x)] = (-1_i32..=1).any(|dy| {
                (-1_i32..=1).any(|dx| {
                    let nx = i32::from(x) + dx;
                    let ny = i32::from(y) + dy;
                    nx >= 0
                        && ny >= 0
                        && nx < i32::from(grid.width)
                        && ny < i32::from(grid.height)
                        && !matches!(
                            snapshot[ny as usize * width + nx as usize],
                            MapCell::Grass
                                | MapCell::Lawn
                                | MapCell::Park
                                | MapCell::Flowers
                                | MapCell::Clearing
                        )
                })
            });
        }
    }
    let scene = grid.scene.as_ref().expect("composed grid has scene");
    for (i, courtyard) in scene.courtyards.iter().enumerate() {
        if *courtyard {
            reserved[i] = true;
        }
    }
    for destination in &scene.destinations {
        for y in destination.origin.1..destination.origin.1 + 4 {
            for x in destination.origin.0..destination.origin.0 + 5 {
                reserved[usize::from(y) * width + usize::from(x)] = true;
            }
        }
    }
    let districts = scene.districts.clone();
    let spawn = scene.spawn;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let i = usize::from(y) * width + usize::from(x);
            if reserved[i] || x.abs_diff(spawn.0).max(y.abs_diff(spawn.1)) < 3 {
                continue;
            }
            let address = addressing.cell(x, y).expect("grid address");
            // Coherent five-block patches, with a second stream for sparse accents.
            let gx = address.x_mod(5) as u64;
            let gy = address.y_mod(5) as u64;
            let anchor = address.offset(-(gx as i64), -(gy as i64));
            let noise = |dx, dy| anchor.offset(dx, dy).stable_hash(0x47524f5645) % 100;
            let patch = (noise(0, 0) * (5 - gx) * (5 - gy)
                + noise(5, 0) * gx * (5 - gy)
                + noise(0, 5) * (5 - gx) * gy
                + noise(5, 5) * gx * gy)
                / 25;
            let detail = address.stable_hash(0x44455441494c) % 100;
            let cell = match districts[i] {
                VisualFamily::Woodland if patch < 58 => MapCell::Tree,
                VisualFamily::Woodland if patch < 80 => MapCell::Park,
                VisualFamily::Rocky if patch < 24 && detail < 45 => MapCell::Boulder,
                VisualFamily::Waterfront if patch < 30 => MapCell::Park,
                VisualFamily::Waterfront if detail < 12 => MapCell::Flowers,
                VisualFamily::Residential if patch < 22 => MapCell::Tree,
                VisualFamily::Residential if detail < 8 => MapCell::Flowers,
                VisualFamily::Meadow if patch < 18 => MapCell::Park,
                VisualFamily::Meadow if detail < 5 => MapCell::Flowers,
                _ => MapCell::Grass,
            };
            grid.cells[i] = cell;
        }
    }
    grid.scene.as_mut().unwrap().reserved = reserved;
    // Reject blocking decorations that sever a previously connected destination.
    // This only removes optional scenery; it never carves source geometry.
    let reachable_before = crate::grid::reachable_for_cells(grid, &snapshot, spawn);
    loop {
        let reachable = crate::grid::reachable_walkable_cells(grid, spawn);
        if !reachable_before.iter().enumerate().any(|(i, was)| {
            *was && grid.scene.as_ref().unwrap().reserved[i]
                && crate::grid::is_walkable_cell(grid.cells[i])
                && !reachable[i]
        }) {
            break;
        }
        let mut changed = false;
        for (i, was) in reachable_before.iter().enumerate() {
            if *was && !reachable[i] && matches!(grid.cells[i], MapCell::Tree | MapCell::Boulder) {
                grid.cells[i] = snapshot[i];
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(())
}

pub(crate) fn audit(grid: &GeneratedGrid, center: bool, mart: bool) -> crate::MapAudit {
    let scene = grid.scene.as_ref().expect("scene audit");
    let reached = crate::grid::reachable_walkable_cells(grid, scene.spawn);
    let mut errors = Vec::new();
    if !reached.iter().any(|v| *v) {
        errors.push("spawn is blocked".into());
    }
    for (wanted, origin, name) in [
        (center, crate::grid::pokecenter_origin(grid), "Center"),
        (mart, crate::grid::mart_origin(grid), "Mart"),
    ] {
        if wanted != origin.is_some() {
            errors.push(format!(
                "{name} allocation does not match the complete facade"
            ));
        }
        if let Some((x, y)) = origin {
            let i = usize::from(y + 2) * usize::from(grid.width) + usize::from(x);
            if !reached.get(i).copied().unwrap_or(false) {
                errors.push(format!("{name} entrance is unreachable"));
            }
        }
    }
    for structure in &scene.structures {
        let (w, h) = structure.recipe.dimensions();
        for dy in 0..h {
            for dx in 0..w {
                if structure.recipe.block(dx, dy).is_some()
                    && grid.cell(structure.origin.0 + dx, structure.origin.1 + dy)
                        != Some(MapCell::Building)
                {
                    errors.push(format!("incomplete structure {}", structure.id));
                }
            }
        }
        let x = structure.door.0 / 2;
        let y = structure.door.1.div_ceil(2);
        if !reached
            .get(usize::from(y) * usize::from(grid.width) + usize::from(x))
            .copied()
            .unwrap_or(false)
        {
            errors.push(format!("unreachable door {}", structure.id));
        }
    }
    for destination in &scene.destinations {
        let (x, y) = destination.entrance;
        if !reached[usize::from(y) * usize::from(grid.width) + usize::from(x)] {
            errors.push(format!("unreachable {:?} destination", destination.family));
        }
    }
    let total = grid
        .cells
        .iter()
        .filter(|c| crate::grid::is_walkable_cell(**c))
        .count();
    let count = reached.iter().filter(|r| **r).count();
    let mut counts = BTreeMap::new();
    for family in &scene.districts {
        *counts
            .entry(format!("{family:?}").to_lowercase())
            .or_insert(0) += 1;
    }
    crate::MapAudit { passed: errors.is_empty(), district_counts: counts, structures: scene.structures.len(), encounter_cells: grid.cells.iter().filter(|c| matches!(c,MapCell::Park)).count(), walkable_reach_percent: count as f64 * 100.0 / total.max(1) as f64, errors, notes: vec!["Composition follows mapped districts; isolated natural islands need not be walkable from spawn.".into()] }
}

pub fn generate_composed_grid(
    mut source: MapSource,
    width: u16,
    height: u16,
) -> Result<GeneratedGrid> {
    ensure!(
        (24..=128).contains(&width) && (24..=128).contains(&height),
        "grid dimensions must be between 24 and 128 blocks"
    );
    ensure!(
        source.schema_version == crate::SOURCE_SCHEMA_VERSION,
        "unsupported source schema {}",
        source.schema_version
    );
    crate::geometry::sort_and_deduplicate_features(&mut source.features);
    crate::grid::generate_composed(source, width, height)
}
