use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::stable_grid::StableGrid;
use crate::{
    Coordinate, Feature, FeatureKind, H3Facility, H3SeamContract, MapSource, RoadAxis, WorldCell,
    WorldGrid, build_h3_seam_contract, finalize_h3_regional_transport_seams,
    preserve_h3_authoritative_water_seams,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapCell {
    /// Rectangular storage outside an H3 face. It renders as dense canopy and
    /// remains a hard wall, but is not authored terrain and is excluded from
    /// rock/tree/style counts.
    H3Void,
    Grass,
    Lawn,
    Clearing,
    Park,
    Flowers,
    Tree,
    ParkTree,
    SmallTree,
    SmallTreeSouth,
    Boulder,
    IceFloor,
    IceBoulder,
    RockFloor,
    Bench,
    TrashCan,
    Fountain,
    GroundSign,
    FenceNorthWest,
    FenceNorth,
    FenceNorthEast,
    FenceWest,
    FenceEast,
    FenceSouthWest,
    FenceSouth,
    FenceSouthEast,
    LedgeWest,
    LedgeMiddle,
    LedgeEast,
    CliffNorthWest,
    CliffNorth,
    CliffNorthEast,
    CliffWest,
    CliffCenter,
    CliffEast,
    CliffSouthWest,
    CliffSouth,
    CliffSouthEast,
    CliffInnerSouthWest,
    CliffInnerSouthEast,
    CliffStairs,
    Water,
    WaterAccessEast,
    WaterAccessWest,
    WaterAccessSouth,
    Pitch,
    Building,
    PokecenterNorthWest,
    PokecenterNorthEast,
    PokecenterSouthWest,
    PokecenterSouthEast,
    MartNorthWest,
    MartNorthEast,
    MartSouthWest,
    MartSouthEast,
    Rail,
    Trail,
    Street,
    Road,
    MajorRoad,
}

impl MapCell {
    fn priority(self) -> u8 {
        match self {
            Self::Grass => 0,
            Self::Lawn | Self::Clearing => 1,
            Self::Park | Self::Flowers => 2,
            Self::Tree
            | Self::ParkTree
            | Self::SmallTree
            | Self::SmallTreeSouth
            | Self::Boulder
            | Self::IceBoulder
            | Self::Bench
            | Self::TrashCan
            | Self::Fountain
            | Self::GroundSign
            | Self::FenceNorthWest
            | Self::FenceNorth
            | Self::FenceNorthEast
            | Self::FenceWest
            | Self::FenceEast
            | Self::FenceSouthWest
            | Self::FenceSouth
            | Self::FenceSouthEast
            | Self::LedgeWest
            | Self::LedgeMiddle
            | Self::LedgeEast
            | Self::CliffNorthWest
            | Self::CliffNorth
            | Self::CliffNorthEast
            | Self::CliffWest
            | Self::CliffCenter
            | Self::CliffEast
            | Self::CliffSouthWest
            | Self::CliffSouth
            | Self::CliffSouthEast
            | Self::CliffInnerSouthWest
            | Self::CliffInnerSouthEast
            | Self::CliffStairs
            | Self::H3Void => 3,
            Self::IceFloor | Self::RockFloor => 2,
            Self::Water
            | Self::WaterAccessEast
            | Self::WaterAccessWest
            | Self::WaterAccessSouth => 4,
            Self::Pitch => 5,
            Self::Building
            | Self::PokecenterNorthWest
            | Self::PokecenterNorthEast
            | Self::PokecenterSouthWest
            | Self::PokecenterSouthEast
            | Self::MartNorthWest
            | Self::MartNorthEast
            | Self::MartSouthWest
            | Self::MartSouthEast => 6,
            Self::Rail => 7,
            Self::Trail => 8,
            Self::Street => 9,
            Self::Road => 10,
            Self::MajorRoad => 11,
        }
    }
}

impl From<FeatureKind> for MapCell {
    fn from(value: FeatureKind) -> Self {
        match value {
            FeatureKind::Water => Self::Water,
            FeatureKind::Park => Self::Park,
            FeatureKind::Pitch => Self::Pitch,
            FeatureKind::Building => Self::Building,
            FeatureKind::Landmark => Self::Clearing,
            FeatureKind::Rail => Self::Rail,
            FeatureKind::Trail => Self::Trail,
            FeatureKind::Street => Self::Street,
            FeatureKind::Road => Self::Road,
            FeatureKind::MajorRoad => Self::MajorRoad,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridLabel {
    pub text: String,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratedGrid {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<crate::scene::ScenePlan>,
    pub source: MapSource,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<MapCell>,
    pub labels: Vec<GridLabel>,
}

impl GeneratedGrid {
    pub fn cell(&self, x: u16, y: u16) -> Option<MapCell> {
        (x < self.width && y < self.height)
            .then(|| self.cells[usize::from(y) * usize::from(self.width) + usize::from(x)])
    }

    pub fn home_cell(&self) -> (u16, u16) {
        self.scene
            .as_ref()
            .map(|s| s.spawn)
            .unwrap_or((self.width / 2, self.height / 2))
    }

    pub fn crystal_blocks(&self) -> Vec<u16> {
        let mut blocks = Vec::with_capacity(self.cells.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cell(x, y).unwrap_or(MapCell::Grass);
                let block = match cell {
                    MapCell::H3Void => h3_void_block(x, y),
                    MapCell::Grass | MapCell::Lawn => 0x02,
                    // Clearing is safe natural ground, not a road. The pale
                    // dirt block made isolated clearings look like torn path
                    // fragments in regional previews.
                    MapCell::Clearing => 0x02,
                    MapCell::Park => park_block(self, x, y),
                    MapCell::Flowers => flower_block(x, y),
                    MapCell::Tree => tree_block(self, x, y),
                    MapCell::ParkTree => u16::from(crate::GENERATED_PARK_TREE_METATILE),
                    MapCell::SmallTree => 0x2f,
                    MapCell::SmallTreeSouth => 0x3b,
                    MapCell::Boulder => 0x0a,
                    MapCell::IceFloor => u16::from(crate::GENERATED_ICE_FLOOR_METATILE),
                    MapCell::IceBoulder => u16::from(crate::GENERATED_ICE_BOULDER_METATILE),
                    // The canonical raised-earth center is a seamless brown
                    // cave/upland floor when repeated without contour edges.
                    MapCell::RockFloor => 0x71,
                    MapCell::Bench => 0x80,
                    MapCell::TrashCan => 0x81,
                    MapCell::Fountain => 0x82,
                    MapCell::GroundSign => 0x45,
                    MapCell::FenceNorthWest => {
                        u16::from(crate::GENERATED_PARK_FENCE_NORTH_WEST_METATILE)
                    }
                    MapCell::FenceNorth => u16::from(crate::GENERATED_PARK_FENCE_NORTH_METATILE),
                    MapCell::FenceNorthEast => {
                        u16::from(crate::GENERATED_PARK_FENCE_NORTH_EAST_METATILE)
                    }
                    MapCell::FenceWest => u16::from(crate::GENERATED_PARK_FENCE_WEST_METATILE),
                    MapCell::FenceEast => u16::from(crate::GENERATED_PARK_FENCE_EAST_METATILE),
                    MapCell::FenceSouthWest => {
                        u16::from(crate::GENERATED_PARK_FENCE_SOUTH_WEST_METATILE)
                    }
                    MapCell::FenceSouth => u16::from(crate::GENERATED_PARK_FENCE_SOUTH_METATILE),
                    MapCell::FenceSouthEast => {
                        u16::from(crate::GENERATED_PARK_FENCE_SOUTH_EAST_METATILE)
                    }
                    MapCell::LedgeWest => 0x52,
                    MapCell::LedgeMiddle => 0x57,
                    MapCell::LedgeEast => 0x53,
                    MapCell::CliffNorthWest => 0x6a,
                    MapCell::CliffNorth => 0x70,
                    MapCell::CliffNorthEast => 0x6b,
                    MapCell::CliffWest => 0x68,
                    MapCell::CliffCenter => 0x71,
                    MapCell::CliffEast => 0x69,
                    MapCell::CliffSouthWest => 0x6c,
                    MapCell::CliffSouth => 0x72,
                    MapCell::CliffSouthEast => 0x6d,
                    MapCell::CliffInnerSouthWest => 0x6e,
                    MapCell::CliffInnerSouthEast => 0x6f,
                    MapCell::CliffStairs => u16::from(crate::GENERATED_CLIFF_STAIRS_METATILE),
                    MapCell::Water => water_block(self, x, y),
                    MapCell::WaterAccessEast => 0x58,
                    MapCell::WaterAccessWest => 0x59,
                    MapCell::WaterAccessSouth => 0x76,
                    MapCell::Pitch => 0x02,
                    MapCell::Building => building_block(self, x, y),
                    MapCell::PokecenterNorthWest => 0x18,
                    MapCell::PokecenterNorthEast => 0x19,
                    MapCell::PokecenterSouthWest => 0x1a,
                    MapCell::PokecenterSouthEast => 0x1b,
                    MapCell::MartNorthWest => 0x18,
                    MapCell::MartNorthEast => 0x19,
                    MapCell::MartSouthWest => 0x1a,
                    MapCell::MartSouthEast => 0x33,
                    MapCell::Rail => 0x04,
                    MapCell::Trail => 0x07,
                    MapCell::Street | MapCell::Road | MapCell::MajorRoad => 0x07,
                };
                blocks.push(block);
            }
        }
        blocks
    }
}

/// Version-two pipeline: structural geometry is finalized once, then reserved.
pub(crate) fn generate_composed(
    source: MapSource,
    width: u16,
    height: u16,
) -> Result<GeneratedGrid> {
    let mut grid = GeneratedGrid {
        scene: None,
        source,
        width,
        height,
        cells: vec![MapCell::Grass; usize::from(width) * usize::from(height)],
        labels: Vec::new(),
    };
    grid.scene = Some(crate::scene::plan(&grid));
    for feature in grid
        .source
        .features
        .clone()
        .iter()
        .filter(|f| matches!(f.kind, FeatureKind::Water | FeatureKind::Pitch))
    {
        paint_feature(&mut grid, feature);
    }
    let seams = if let Some(plan) = &grid.source.h3 {
        Some(build_h3_seam_contract(plan, &grid.source, width, height)?)
    } else {
        None
    };
    if let Some(seams) = &seams {
        author_h3_boundary(&mut grid, seams)?;
    }
    let spawn = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|&(x, y)| is_walkable_cell(grid.cell(x, y).unwrap()))
        .min_by_key(|&(x, y)| (x.abs_diff(width / 2) + y.abs_diff(height / 2), y, x))
        .ok_or_else(|| anyhow::anyhow!("source has no safe land for a starting point"))?;
    grid.scene.as_mut().unwrap().spawn = spawn;
    plan_path_backbone(&mut grid)?;
    if let Some(seams) = &seams {
        author_h3_boundary(&mut grid, seams)?;
    }
    if grid
        .source
        .h3
        .as_ref()
        .is_none_or(|p| p.requests_facility(H3Facility::PokemonCenter))
    {
        place_pokecenter(&mut grid);
    }
    if grid
        .source
        .h3
        .as_ref()
        .is_none_or(|p| p.requests_facility(H3Facility::Mart))
    {
        place_mart(&mut grid);
    }
    place_city_landmarks(&mut grid);
    place_houses(&mut grid);
    // Existing geometry recipes are reused, but only mapped recreation creates a field.
    author_public_field(&mut grid);
    connect_h3_regional_backbone(&mut grid)?;
    if seams.is_some() {
        finalize_h3_regional_transport_seams(&mut grid)?;
        preserve_h3_authoritative_water_seams(&mut grid)?;
        connect_h3_regional_backbone(&mut grid)?;
        finalize_h3_regional_transport_seams(&mut grid)?;
    }
    create_water_access(&mut grid);
    crate::scene::compose_destinations(&mut grid)?;
    crate::scene::freeze_and_decorate(&mut grid)?;
    grid.labels = select_labels(&grid);
    Ok(grid)
}

pub(crate) fn reachable_for_cells(
    grid: &GeneratedGrid,
    cells: &[MapCell],
    spawn: (u16, u16),
) -> Vec<bool> {
    let mut copy = grid.clone();
    copy.cells = cells.to_vec();
    reachable_walkable_cells(&copy, spawn)
}

pub fn generate_grid(source: MapSource, width: u16, height: u16) -> Result<GeneratedGrid> {
    crate::scene::generate_composed_grid(source, width, height)
}

fn plan_path_backbone(grid: &mut GeneratedGrid) -> Result<()> {
    if grid.source.h3.is_some() {
        return plan_h3_path_backbone(grid);
    }
    let world_grid = WorldGrid::from_bounds(
        grid.source.center,
        grid.source.bounds,
        grid.width,
        grid.height,
    )?;
    let corridors = global_road_corridors(grid, world_grid)?;

    // The mapped layer is authored in global metatile addresses. Converting to
    // local coordinates is only the final crop, so moving the requested center
    // cannot slide a road onto a different lane. Water always wins: a mapped
    // corridor resumes on the far bank and a local Trail supplies the detour.
    paint_global_road_corridors(grid, world_grid, &corridors);
    bridge_mapped_roads_around_water(grid, world_grid, &corridors);

    connect_home_and_wild_sites(grid);
    Ok(())
}

fn plan_h3_path_backbone(grid: &mut GeneratedGrid) -> Result<()> {
    // H3 cells preserve real geometry in their own tangent frame. They never
    // synthesize a road along the six-sided boundary: only source linear
    // features are rasterized, and shared-edge contracts independently prove
    // which neighboring cells receive the continuation.
    paint_h3_source_transport(grid)?;
    connect_h3_regional_backbone(grid)?;
    connect_home_and_wild_sites(grid);
    Ok(())
}

fn paint_h3_source_transport(grid: &mut GeneratedGrid) -> Result<()> {
    let water = grid
        .cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            matches!(
                cell,
                MapCell::Water
                    | MapCell::WaterAccessEast
                    | MapCell::WaterAccessWest
                    | MapCell::WaterAccessSouth
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let features = grid
        .source
        .features
        .iter()
        .filter(|feature| {
            feature.surface_transport()
                && !feature.area
                && matches!(
                    feature.kind,
                    FeatureKind::Trail
                        | FeatureKind::Street
                        | FeatureKind::Road
                        | FeatureKind::MajorRoad
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    let selected_bridges = selected_h3_bridge_features(grid, &features)?;
    for feature in &features {
        paint_feature(grid, feature);
    }
    // Water wins over every untagged or unselected route. Regional attachment
    // has already validated and retained the exact source feature behind each
    // authoritative crossing, so only that explicit `bridge=yes` trace may be
    // repainted across the restored water mask.
    for index in water {
        grid.cells[index] = MapCell::Water;
    }
    for feature in &selected_bridges {
        paint_feature(grid, feature);
    }
    Ok(())
}

fn selected_h3_bridge_features(grid: &GeneratedGrid, features: &[Feature]) -> Result<Vec<Feature>> {
    let Some(plan) = grid.source.h3.as_ref() else {
        return Ok(Vec::new());
    };
    let seams = build_h3_seam_contract(plan, &grid.source, grid.width, grid.height)?;
    let Some(regional) = plan.regional.as_ref() else {
        // Standalone H3 transport has already been reduced to its chosen edge
        // features. Preserve explicit bridges there as well; no discarded or
        // merely nearby feature remains in the prepared source.
        return Ok(features
            .iter()
            .filter(|feature| feature.bridge)
            .cloned()
            .collect());
    };
    let mut selected = Vec::new();
    for feature in features.iter().filter(|feature| feature.bridge) {
        let contract_crossing = seams.edges.iter().any(|edge| {
            edge.viable_crossings.iter().any(|crossing| {
                crossing.bridge
                    && crossing.transport == feature.kind
                    && feature_contains_coordinate(feature, crossing.coordinate)
            })
        });
        // Treat anything not wholly inside the raster face as a boundary
        // feature even when an endpoint-touching segment is numerically missed
        // by geographic intersection. This is deliberately conservative: a
        // real internal bridge survives, while every possible edge exit still
        // needs an exact selected regional connection.
        let crosses_face = contract_crossing
            || !feature_is_wholly_inside_h3_face(plan, feature, grid.width, grid.height)?;
        let selected_crossing = regional.connections.iter().any(|connection| {
            connection.authoritative
                && connection.bridge
                && connection.transport == feature.kind
                && feature_contains_coordinate(feature, connection.coordinate)
        });
        if !crosses_face || selected_crossing {
            selected.push(feature.clone());
        }
    }
    Ok(selected)
}

fn feature_is_wholly_inside_h3_face(
    plan: &crate::H3CellPlan,
    feature: &Feature,
    width: u16,
    height: u16,
) -> Result<bool> {
    for &point in &feature.points {
        let (x, y) = plan.project_to_grid(point, width, height)?;
        if x < 0
            || y < 0
            || x >= i32::from(width)
            || y >= i32::from(height)
            || !plan.raster_contains_cell(x as u16, y as u16, width, height)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn feature_contains_coordinate(feature: &Feature, coordinate: Coordinate) -> bool {
    const MATCH_TOLERANCE_DEGREES: f64 = 1e-7;
    let longitude_scale = coordinate.lat.to_radians().cos().abs().max(1e-6);
    let local = |point: Coordinate| {
        let longitude = (point.lon - coordinate.lon + 180.0).rem_euclid(360.0) - 180.0;
        (longitude * longitude_scale, point.lat - coordinate.lat)
    };
    feature.points.windows(2).any(|segment| {
        let first = local(segment[0]);
        let second = local(segment[1]);
        let delta = (second.0 - first.0, second.1 - first.1);
        let length_squared = delta.0 * delta.0 + delta.1 * delta.1;
        if length_squared <= f64::EPSILON {
            return first.0.hypot(first.1) <= MATCH_TOLERANCE_DEGREES;
        }
        let fraction = (-(first.0 * delta.0 + first.1 * delta.1) / length_squared).clamp(0.0, 1.0);
        let closest = (first.0 + delta.0 * fraction, first.1 + delta.1 * fraction);
        closest.0.hypot(closest.1) <= MATCH_TOLERANCE_DEGREES
    })
}

fn connect_h3_regional_backbone(grid: &mut GeneratedGrid) -> Result<()> {
    let Some((plan, regional)) = grid.source.h3.as_ref().and_then(|plan| {
        plan.regional
            .as_ref()
            .map(|regional| (plan.clone(), regional.clone()))
    }) else {
        return Ok(());
    };
    let width = usize::from(grid.width);
    let mut closed_landings = std::collections::BTreeSet::<usize>::new();
    for crossing in &regional.closed_transport_crossings {
        for (x, y) in crate::h3::h3_raster_sample_band(&plan, grid, crossing.coordinate)? {
            let index = usize::from(y) * width + usize::from(x);
            closed_landings.insert(index);
            if matches!(
                grid.cells[index],
                MapCell::Trail | MapCell::Street | MapCell::Road | MapCell::MajorRoad
            ) {
                grid.cells[index] = MapCell::Grass;
            }
        }
    }
    let mut landings = std::collections::BTreeSet::<usize>::new();
    for connection in &regional.connections {
        let route = MapCell::from(connection.transport);
        // Use the same exact cardinal three-cell band as final seam
        // hardening and runtime transitions. Independent projection logic
        // previously produced diagonal or shifted route fragments.
        for (x, y) in crate::h3::h3_raster_sample_band(&plan, grid, connection.coordinate)? {
            let index = usize::from(y) * width + usize::from(x);
            grid.cells[index] = route;
            landings.insert(index);
        }
    }
    let connector_forbidden = regional_connector_forbidden_cells(grid, &landings, &closed_landings);

    // OSM streets are often separate polylines after downsampling. Join only
    // components that own declared regional landings to the principal source
    // route. Connecting every incidental OSM fragment created hundreds of
    // synthetic Trail cells and made dense city rooms fail their path budget.
    for _ in 0..regional.connections.len().saturating_add(1) {
        let components = transport_components(grid);
        if components.len() <= 1 {
            break;
        }
        let principal_index = components
            .iter()
            .enumerate()
            .max_by_key(|(_, component)| component.len())
            .map(|(index, _)| index)
            .expect("multiple route components have a principal component");
        let principal = &components[principal_index];
        let mut targets = components
            .iter()
            .enumerate()
            .filter(|(index, component)| {
                *index != principal_index && component.iter().any(|index| landings.contains(index))
            })
            .map(|(index, component)| (std::cmp::Reverse(component.len()), index))
            .collect::<Vec<_>>();
        targets.sort_unstable();
        let mut connected = false;
        if targets.is_empty() {
            break;
        }
        for (_, target_index) in targets {
            let target = &components[target_index];
            let mut pairs = target
                .iter()
                .flat_map(|&from| {
                    principal.iter().map(move |&to| {
                        let from_x = from % width;
                        let from_y = from / width;
                        let to_x = to % width;
                        let to_y = to / width;
                        (from_x.abs_diff(to_x) + from_y.abs_diff(to_y), from, to)
                    })
                })
                .collect::<Vec<_>>();
            pairs.sort_unstable();
            for (_, from, to) in pairs.into_iter().take(96) {
                if connector_forbidden.contains(&from) || connector_forbidden.contains(&to) {
                    continue;
                }
                let start = ((from % width) as i32, (from / width) as i32);
                let goal = ((to % width) as i32, (to / width) as i32);
                let Some(path) =
                    shortest_path_avoiding(grid, start, goal, true, &connector_forbidden).or_else(
                        || shortest_path_avoiding(grid, start, goal, false, &connector_forbidden),
                    )
                else {
                    continue;
                };
                if path
                    .iter()
                    .any(|&(x, y)| connector_forbidden.contains(&(y as usize * width + x as usize)))
                {
                    continue;
                }
                commit_regional_trail(grid, path);
                connected = true;
                break;
            }
            if connected {
                break;
            }
        }
        if !connected {
            bail!(
                "could not connect every selected regional landing to the principal route in H3 cell {}",
                plan.cell
            );
        }
    }

    // Disconnected route fragments neither lead to a selected neighboring
    // room nor belong to the principal local network. Remove them instead of
    // spending a large synthetic-path budget stitching every clipped OSM
    // fragment; all declared landings were connected above.
    let components = transport_components(grid);
    if let Some(principal) = components.iter().max_by_key(|component| component.len()) {
        let principal = principal
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for component in components {
            if component.iter().any(|index| principal.contains(index)) {
                continue;
            }
            if component.iter().any(|index| landings.contains(index)) {
                bail!(
                    "selected regional landing remained outside the principal route in H3 cell {}",
                    plan.cell
                );
            }
            for index in component {
                grid.cells[index] = MapCell::Grass;
            }
        }
    }
    Ok(())
}

fn regional_connector_forbidden_cells(
    grid: &GeneratedGrid,
    landings: &std::collections::BTreeSet<usize>,
    closed_landings: &std::collections::BTreeSet<usize>,
) -> std::collections::BTreeSet<usize> {
    let mut forbidden = closed_landings.clone();
    let width = usize::from(grid.width);
    for y in 0..grid.height {
        for x in 0..grid.width {
            let index = usize::from(y) * width + usize::from(x);
            if !landings.contains(&index) && crate::h3::route_cell_touches_h3_void(grid, x, y) {
                forbidden.insert(index);
            }
        }
    }
    for landing in landings {
        forbidden.remove(landing);
    }
    forbidden
}

fn commit_regional_trail(grid: &mut GeneratedGrid, path: Vec<(i32, i32)>) {
    for (x, y) in path {
        if !matches!(
            grid.cell(x as u16, y as u16),
            Some(
                MapCell::H3Void
                    | MapCell::Water
                    | MapCell::WaterAccessEast
                    | MapCell::WaterAccessWest
                    | MapCell::WaterAccessSouth
                    | MapCell::Trail
                    | MapCell::Street
                    | MapCell::Road
                    | MapCell::MajorRoad
                    | MapCell::Building
                    | MapCell::PokecenterNorthWest
                    | MapCell::PokecenterNorthEast
                    | MapCell::PokecenterSouthWest
                    | MapCell::PokecenterSouthEast
                    | MapCell::MartNorthWest
                    | MapCell::MartNorthEast
                    | MapCell::MartSouthWest
                    | MapCell::MartSouthEast
                    | MapCell::CliffNorthWest
                    | MapCell::CliffNorth
                    | MapCell::CliffNorthEast
                    | MapCell::CliffWest
                    | MapCell::CliffCenter
                    | MapCell::CliffEast
                    | MapCell::CliffSouthWest
                    | MapCell::CliffSouth
                    | MapCell::CliffSouthEast
                    | MapCell::CliffInnerSouthWest
                    | MapCell::CliffInnerSouthEast
            )
        ) {
            set_cell(grid, x, y, MapCell::Trail);
        }
    }
}

fn transport_components(grid: &GeneratedGrid) -> Vec<Vec<usize>> {
    let width = usize::from(grid.width);
    let height = usize::from(grid.height);
    let mut unseen = grid
        .cells
        .iter()
        .map(|cell| {
            matches!(
                cell,
                MapCell::Trail | MapCell::Street | MapCell::Road | MapCell::MajorRoad
            )
        })
        .collect::<Vec<_>>();
    let mut components = Vec::new();
    for start in 0..unseen.len() {
        if !unseen[start] {
            continue;
        }
        unseen[start] = false;
        let mut component = Vec::new();
        let mut frontier = std::collections::VecDeque::from([start]);
        while let Some(index) = frontier.pop_front() {
            component.push(index);
            let x = index % width;
            let y = index / width;
            for (next_x, next_y) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if next_x >= width || next_y >= height {
                    continue;
                }
                let next = next_y * width + next_x;
                if unseen[next] {
                    unseen[next] = false;
                    frontier.push_back(next);
                }
            }
        }
        components.push(component);
    }
    components
}

fn connect_home_and_wild_sites(grid: &mut GeneratedGrid) {
    let (home_x, home_y) = grid.home_cell();
    let center_x = i32::from(home_x);
    let center_y = i32::from(home_y);
    // The address and authored destinations connect to the geographic road
    // layer with Trail cells. This keeps synthetic gameplay connectors
    // semantically distinct from OSM Street/Road/MajorRoad corridors.
    let mut routes = transport_cells(grid);
    routes.sort_unstable_by_key(|&(x, y)| (x - center_x).abs() + (y - center_y).abs());
    if let Some(&nearest) = routes.first() {
        carve_path(grid, (center_x, center_y), nearest);
    } else {
        carve_path(grid, (1, center_y), (i32::from(grid.width) - 2, center_y));
    }

    let mut wild_components = terrain_components(grid, MapCell::Park);
    wild_components.sort_by_key(|component| std::cmp::Reverse(component.len()));
    for component in wild_components.into_iter().take(4) {
        let width = usize::from(grid.width);
        let members = component
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let route_cells = transport_cells(grid);
        let mut best_path = None::<Vec<(i32, i32)>>;
        for index in component {
            let x = index % width;
            let y = index / width;
            for (outside_x, outside_y) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if outside_x >= usize::from(grid.width)
                    || outside_y >= usize::from(grid.height)
                    || members.contains(&(outside_y * width + outside_x))
                {
                    continue;
                }
                let landing = (outside_x as i32, outside_y as i32);
                let mut nearest_routes = route_cells.clone();
                nearest_routes.sort_unstable_by_key(|&(route_x, route_y)| {
                    (route_x - landing.0).abs() + (route_y - landing.1).abs()
                });
                for route in nearest_routes.into_iter().take(12) {
                    let Some(path) = shortest_route_path(grid, landing, route) else {
                        continue;
                    };
                    if best_path
                        .as_ref()
                        .is_none_or(|best| path.len() < best.len())
                    {
                        best_path = Some(path);
                    }
                }
            }
        }
        if let Some(path) = best_path {
            commit_trail(grid, path);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldGate {
    North,
    South,
    West,
    East,
}

#[derive(Debug)]
struct PublicFieldProposal {
    origin_x: u16,
    origin_y: u16,
    gate: FieldGate,
    gate_cell: (i32, i32),
    connector: Vec<(i32, i32)>,
    pitch_overlap: usize,
    source_distance: i32,
}

/// Compresses real-world sports/playground polygons into one authored Crystal
/// destination. Tiny downsampled pitch fragments are not useful on their own;
/// a complete fenced field, named sign, open gate, and short route connection
/// preserve their geographic intent at gameplay scale.
fn author_public_field(grid: &mut GeneratedGrid) {
    const FIELD_WIDTH: u16 = 9;
    const FIELD_HEIGHT: u16 = 7;
    // A globally snapped east/west corridor can move by up to half of its
    // twenty-cell band. Leave enough room for that displacement plus the
    // field's gate approach while still rejecting long synthetic routes.
    const MAX_CONNECTOR_STEPS: usize = 16;

    let pitch_components = terrain_components(grid, MapCell::Pitch);
    if pitch_components.is_empty() {
        return;
    }
    let width = usize::from(grid.width);
    let original_pitch = grid
        .cells
        .iter()
        .map(|cell| *cell == MapCell::Pitch)
        .collect::<Vec<_>>();
    let mut proposals = Vec::new();

    for component in &pitch_components {
        let center_x =
            component.iter().map(|index| index % width).sum::<usize>() / component.len().max(1);
        let center_y =
            component.iter().map(|index| index / width).sum::<usize>() / component.len().max(1);
        let ideal_x = center_x as i32 - i32::from(FIELD_WIDTH / 2);
        let ideal_y = center_y as i32 - i32::from(FIELD_HEIGHT / 2);

        for offset_y in -4..=4 {
            for offset_x in -4..=4 {
                let origin_x = ideal_x + offset_x;
                let origin_y = ideal_y + offset_y;
                if origin_x < 2
                    || origin_y < 2
                    || origin_x + i32::from(FIELD_WIDTH) + 1 >= i32::from(grid.width)
                    || origin_y + i32::from(FIELD_HEIGHT) + 1 >= i32::from(grid.height)
                    || !h3_stamp_fits(grid, origin_x, origin_y, FIELD_WIDTH, FIELD_HEIGHT, 3)
                {
                    continue;
                }
                let footprint_is_clear = (0..FIELD_HEIGHT).all(|dy| {
                    (0..FIELD_WIDTH).all(|dx| {
                        matches!(
                            grid.cell(
                                (origin_x + i32::from(dx)) as u16,
                                (origin_y + i32::from(dy)) as u16
                            ),
                            Some(
                                MapCell::Grass | MapCell::Lawn | MapCell::Clearing | MapCell::Pitch
                            )
                        )
                    })
                });
                if !footprint_is_clear {
                    continue;
                }
                let pitch_overlap = (0..FIELD_HEIGHT)
                    .flat_map(|dy| (0..FIELD_WIDTH).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| {
                        let x = (origin_x + i32::from(dx)) as usize;
                        let y = (origin_y + i32::from(dy)) as usize;
                        original_pitch[y * width + x]
                    })
                    .count();
                if pitch_overlap == 0 {
                    continue;
                }

                for gate in [
                    FieldGate::North,
                    FieldGate::South,
                    FieldGate::West,
                    FieldGate::East,
                ] {
                    let gate_cell = field_gate_cell(origin_x, origin_y, gate);
                    let Some(connector) = scenic_connector(grid, gate_cell, MAX_CONNECTOR_STEPS)
                    else {
                        continue;
                    };
                    if connector.iter().skip(1).any(|&(x, y)| {
                        x >= origin_x
                            && x < origin_x + i32::from(FIELD_WIDTH)
                            && y >= origin_y
                            && y < origin_y + i32::from(FIELD_HEIGHT)
                    }) {
                        continue;
                    }
                    proposals.push(PublicFieldProposal {
                        origin_x: origin_x as u16,
                        origin_y: origin_y as u16,
                        gate,
                        gate_cell,
                        source_distance: (origin_x + i32::from(FIELD_WIDTH / 2) - center_x as i32)
                            .abs()
                            + (origin_y + i32::from(FIELD_HEIGHT / 2) - center_y as i32).abs(),
                        pitch_overlap,
                        connector,
                    });
                }
            }
        }
    }

    proposals.sort_by_key(|proposal| {
        (
            new_trail_steps(grid, &proposal.connector),
            std::cmp::Reverse(proposal.pitch_overlap),
            proposal.source_distance,
            path_turns(&proposal.connector),
            hash(proposal.origin_x, proposal.origin_y),
        )
    });
    let selected = proposals.into_iter().next();

    // Every unselected pitch fragment returns to ordinary lawn. Keeping them
    // as invisible semantic blobs made the map sparse without adding a room.
    for cell in &mut grid.cells {
        if *cell == MapCell::Pitch {
            *cell = MapCell::Grass;
        }
    }
    let Some(proposal) = selected else {
        return;
    };

    stamp_public_field(grid, &proposal);
    commit_trail(grid, proposal.connector.clone());
    let sign = field_sign_site(grid, &proposal);
    if let Some((sign_x, sign_y)) = sign
        && h3_protected_cell_fits(grid, sign_x as u16, sign_y as u16)
    {
        set_cell(grid, sign_x, sign_y, MapCell::GroundSign);
        grid.labels.push(GridLabel {
            text: nearest_pitch_name(grid, proposal.origin_x, proposal.origin_y)
                .unwrap_or_else(|| "NEIGHBORHOOD FIELD".to_string()),
            x: sign_x as u16,
            y: sign_y as u16,
        });
    }
}

fn field_gate_cell(origin_x: i32, origin_y: i32, gate: FieldGate) -> (i32, i32) {
    const FIELD_WIDTH: i32 = 9;
    const FIELD_HEIGHT: i32 = 7;
    match gate {
        FieldGate::North => (origin_x + FIELD_WIDTH / 2, origin_y),
        FieldGate::South => (origin_x + FIELD_WIDTH / 2, origin_y + FIELD_HEIGHT - 1),
        FieldGate::West => (origin_x, origin_y + FIELD_HEIGHT / 2),
        FieldGate::East => (origin_x + FIELD_WIDTH - 1, origin_y + FIELD_HEIGHT / 2),
    }
}

fn stamp_public_field(grid: &mut GeneratedGrid, proposal: &PublicFieldProposal) {
    const FIELD_WIDTH: u16 = 9;
    const FIELD_HEIGHT: u16 = 7;
    for dy in 0..FIELD_HEIGHT {
        for dx in 0..FIELD_WIDTH {
            let x = proposal.origin_x + dx;
            let y = proposal.origin_y + dy;
            if (i32::from(x), i32::from(y)) == proposal.gate_cell {
                set_cell(grid, i32::from(x), i32::from(y), MapCell::Trail);
                continue;
            }
            let cell = match (dx, dy) {
                (0, 0) => MapCell::FenceNorthWest,
                (x, 0) if x + 1 == FIELD_WIDTH => MapCell::FenceNorthEast,
                (_, 0) => MapCell::FenceNorth,
                (0, y) if y + 1 == FIELD_HEIGHT => MapCell::FenceSouthWest,
                (x, y) if x + 1 == FIELD_WIDTH && y + 1 == FIELD_HEIGHT => MapCell::FenceSouthEast,
                (_, y) if y + 1 == FIELD_HEIGHT => MapCell::FenceSouth,
                (0, _) => MapCell::FenceWest,
                (x, _) if x + 1 == FIELD_WIDTH => MapCell::FenceEast,
                _ => MapCell::Pitch,
            };
            set_cell(grid, i32::from(x), i32::from(y), cell);
        }
    }

    // A two-block interior approach keeps the gate readable and prevents the
    // field from becoming a closed decorative rectangle.
    let inward = match proposal.gate {
        FieldGate::North => (0, 1),
        FieldGate::South => (0, -1),
        FieldGate::West => (1, 0),
        FieldGate::East => (-1, 0),
    };
    for step in 0..=2 {
        set_cell(
            grid,
            proposal.gate_cell.0 + inward.0 * step,
            proposal.gate_cell.1 + inward.1 * step,
            MapCell::Trail,
        );
    }

    // Give the field a small authored garden focus so its interior reads as a
    // lived-in public park rather than one blank green rectangle. The offset
    // paired edge beds cannot divide the remaining playable field.
    for (dx, dy) in [(2_i32, 1_i32), (3, 1), (5, 5), (6, 5)] {
        let x = i32::from(proposal.origin_x) + dx;
        let y = i32::from(proposal.origin_y) + dy;
        if grid.cell(x as u16, y as u16) == Some(MapCell::Pitch) {
            set_cell(grid, x, y, MapCell::Flowers);
        }
    }
}

fn field_sign_site(grid: &GeneratedGrid, proposal: &PublicFieldProposal) -> Option<(i32, i32)> {
    let outward = match proposal.gate {
        FieldGate::North => (0, -1),
        FieldGate::South => (0, 1),
        FieldGate::West => (-1, 0),
        FieldGate::East => (1, 0),
    };
    let sideways = (-outward.1, outward.0);
    for side in [1, -1, 2, -2] {
        let x = proposal.gate_cell.0 + outward.0 + sideways.0 * side;
        let y = proposal.gate_cell.1 + outward.1 + sideways.1 * side;
        if x > 0
            && y > 0
            && x + 1 < i32::from(grid.width)
            && y + 1 < i32::from(grid.height)
            && matches!(
                grid.cell(x as u16, y as u16),
                Some(MapCell::Grass | MapCell::Lawn | MapCell::Clearing)
            )
            && !proposal.connector.contains(&(x, y))
        {
            return Some((x, y));
        }
    }
    None
}

fn nearest_pitch_name(grid: &GeneratedGrid, origin_x: u16, origin_y: u16) -> Option<String> {
    grid.source
        .features
        .iter()
        .filter(|feature| feature.kind == FeatureKind::Pitch)
        .filter_map(|feature| {
            let name = feature.name.clone()?;
            let point = feature.points.get(feature.points.len() / 2)?;
            let (x, y) = project(grid, *point);
            Some((
                (x - i32::from(origin_x)).abs() + (y - i32::from(origin_y)).abs(),
                name,
            ))
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, name)| name)
}

#[derive(Debug, Clone)]
struct GlobalRoadCorridor {
    kind: FeatureKind,
    axis: RoadAxis,
    lane: i64,
    start: i64,
    end: i64,
}

/// Normalize each source way independently onto a canonical global lane.
///
/// There is no response-local winner: another bbox may add a segment to the
/// same band, but it cannot replace or move a segment already shared by both
/// responses. East/west ways use one lane per twenty global cells. Only mapped
/// Road/MajorRoad ways survive north/south compression, on one lane per forty-
/// eight cells. Every hierarchy is exactly one metatile wide.
fn global_road_corridors(
    grid: &GeneratedGrid,
    world_grid: WorldGrid,
) -> Result<Vec<GlobalRoadCorridor>> {
    let mut corridors = Vec::new();
    for feature in grid.source.features.iter().filter(|feature| {
        feature.surface_transport()
            && matches!(
                feature.kind,
                FeatureKind::Street | FeatureKind::Road | FeatureKind::MajorRoad
            )
    }) {
        let projected = feature
            .points
            .iter()
            .map(|point| world_grid.project_cell(*point))
            .collect::<Result<Vec<_>>>()?;
        if projected.len() < 2 {
            continue;
        }
        let min_x = projected.iter().map(|point| point.x).min().unwrap_or(0);
        let max_x = projected.iter().map(|point| point.x).max().unwrap_or(0);
        let min_y = projected.iter().map(|point| point.y).min().unwrap_or(0);
        let max_y = projected.iter().map(|point| point.y).max().unwrap_or(0);
        let axis = if max_x - min_x >= max_y - min_y {
            RoadAxis::EastWest
        } else {
            RoadAxis::NorthSouth
        };
        if axis == RoadAxis::NorthSouth && feature.kind == FeatureKind::Street {
            continue;
        }
        let mut ordinates = projected
            .iter()
            .map(|point| match axis {
                RoadAxis::EastWest => point.y,
                RoadAxis::NorthSouth => point.x,
            })
            .collect::<Vec<_>>();
        ordinates.sort_unstable();
        let band_width = match axis {
            RoadAxis::EastWest => 20,
            RoadAxis::NorthSouth => 48,
        };
        let lane = canonical_global_lane(ordinates[ordinates.len() / 2], band_width);
        let (start, end) = match axis {
            RoadAxis::EastWest => (min_x, max_x),
            RoadAxis::NorthSouth => (min_y, max_y),
        };
        if end - start >= 6 {
            corridors.push(GlobalRoadCorridor {
                kind: feature.kind,
                axis,
                lane,
                start,
                end,
            });
        }
    }
    corridors.sort_by_key(|corridor| {
        (
            match corridor.axis {
                RoadAxis::EastWest => 0,
                RoadAxis::NorthSouth => 1,
            },
            corridor.lane,
            corridor.start,
            corridor.end,
            road_kind_priority(corridor.kind),
        )
    });
    Ok(corridors)
}

fn canonical_global_lane(value: i64, band_width: i64) -> i64 {
    value.div_euclid(band_width) * band_width + band_width / 2
}

fn road_kind_priority(kind: FeatureKind) -> i32 {
    match kind {
        FeatureKind::MajorRoad => 3,
        FeatureKind::Road => 2,
        FeatureKind::Street => 1,
        _ => 0,
    }
}

fn paint_global_road_corridors(
    grid: &mut GeneratedGrid,
    world_grid: WorldGrid,
    corridors: &[GlobalRoadCorridor],
) {
    let mut cells = std::collections::BTreeMap::<WorldCell, FeatureKind>::new();
    for corridor in corridors {
        for world in visible_corridor_cells(world_grid, corridor) {
            cells
                .entry(world)
                .and_modify(|kind| {
                    if road_kind_priority(corridor.kind) > road_kind_priority(*kind) {
                        *kind = corridor.kind;
                    }
                })
                .or_insert(corridor.kind);
        }
    }
    for (world, kind) in cells {
        let Some((x, y)) = world_grid.local_cell(world) else {
            continue;
        };
        if !matches!(
            grid.cell(x, y),
            Some(
                MapCell::Water
                    | MapCell::WaterAccessEast
                    | MapCell::WaterAccessWest
                    | MapCell::WaterAccessSouth
            )
        ) {
            set_cell(grid, i32::from(x), i32::from(y), MapCell::from(kind));
        }
    }
}

fn visible_corridor_cells(world_grid: WorldGrid, corridor: &GlobalRoadCorridor) -> Vec<WorldCell> {
    match corridor.axis {
        RoadAxis::EastWest => {
            let start = corridor.start.max(world_grid.west);
            let end = corridor
                .end
                .min(world_grid.west + i64::from(world_grid.width) - 1);
            (start..=end)
                .map(|x| WorldCell {
                    x,
                    y: corridor.lane,
                })
                .collect()
        }
        RoadAxis::NorthSouth => {
            let south = corridor
                .start
                .max(world_grid.north - i64::from(world_grid.height) + 1);
            let north = corridor.end.min(world_grid.north);
            (south..=north)
                .map(|y| WorldCell {
                    x: corridor.lane,
                    y,
                })
                .collect()
        }
    }
}

fn bridge_mapped_roads_around_water(
    grid: &mut GeneratedGrid,
    world_grid: WorldGrid,
    corridors: &[GlobalRoadCorridor],
) {
    for corridor in corridors {
        let visible = visible_corridor_cells(world_grid, corridor)
            .into_iter()
            .filter_map(|world| world_grid.local_cell(world))
            .collect::<Vec<_>>();
        let mut previous_bank = None::<(i32, i32)>;
        let mut crossed_water = false;
        for (x, y) in visible {
            let local = (i32::from(x), i32::from(y));
            match grid.cell(x, y) {
                Some(
                    MapCell::Water
                    | MapCell::WaterAccessEast
                    | MapCell::WaterAccessWest
                    | MapCell::WaterAccessSouth,
                ) => crossed_water = previous_bank.is_some(),
                Some(MapCell::Street | MapCell::Road | MapCell::MajorRoad) => {
                    if crossed_water
                        && let Some(bank) = previous_bank
                        && let Some(path) = shortest_route_path(grid, bank, local)
                    {
                        commit_trail(grid, path);
                    }
                    previous_bank = Some(local);
                    crossed_water = false;
                }
                _ => {
                    previous_bank = None;
                    crossed_water = false;
                }
            }
        }
    }
}

fn transport_cells(grid: &GeneratedGrid) -> Vec<(i32, i32)> {
    let width = usize::from(grid.width);
    grid.cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            matches!(
                cell,
                MapCell::Trail | MapCell::Street | MapCell::Road | MapCell::MajorRoad
            )
            .then_some(((index % width) as i32, (index / width) as i32))
        })
        .collect()
}

fn commit_trail(grid: &mut GeneratedGrid, path: Vec<(i32, i32)>) {
    for (x, y) in path {
        if matches!(
            grid.cell(x as u16, y as u16),
            Some(MapCell::Grass | MapCell::Lawn | MapCell::Clearing | MapCell::Trail)
        ) {
            set_cell(grid, x, y, MapCell::Trail);
        }
    }
}

fn carve_path(grid: &mut GeneratedGrid, start: (i32, i32), goal: (i32, i32)) {
    if let Some(path) = shortest_route_path(grid, start, goal) {
        commit_trail(grid, path);
    }
}

fn shortest_land_path(
    grid: &GeneratedGrid,
    start: (i32, i32),
    goal: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    shortest_path(grid, start, goal, false)
}

fn shortest_route_path(
    grid: &GeneratedGrid,
    start: (i32, i32),
    goal: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    shortest_path(grid, start, goal, true)
}

fn shortest_path(
    grid: &GeneratedGrid,
    start: (i32, i32),
    goal: (i32, i32),
    preserve_wild_grass: bool,
) -> Option<Vec<(i32, i32)>> {
    shortest_path_avoiding(
        grid,
        start,
        goal,
        preserve_wild_grass,
        &std::collections::BTreeSet::new(),
    )
}

fn shortest_path_avoiding(
    grid: &GeneratedGrid,
    start: (i32, i32),
    goal: (i32, i32),
    preserve_wild_grass: bool,
    excluded: &std::collections::BTreeSet<usize>,
) -> Option<Vec<(i32, i32)>> {
    let width = usize::from(grid.width);
    let height = usize::from(grid.height);
    let index = |x: i32, y: i32| y as usize * width + x as usize;
    let mut previous = vec![None; width * height];
    let mut frontier = std::collections::VecDeque::from([start]);
    previous[index(start.0, start.1)] = Some(start);
    while let Some((x, y)) = frontier.pop_front() {
        if (x, y) == goal {
            let mut path = vec![goal];
            let mut cursor = goal;
            while cursor != start {
                cursor = previous[index(cursor.0, cursor.1)]?;
                path.push(cursor);
            }
            path.reverse();
            return Some(path);
        }
        let mut neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
        neighbors
            .sort_by_key(|&(next_x, next_y)| (goal.0 - next_x).abs() + (goal.1 - next_y).abs());
        for (next_x, next_y) in neighbors {
            if next_x < 0 || next_y < 0 || next_x >= width as i32 || next_y >= height as i32 {
                continue;
            }
            let next = index(next_x, next_y);
            if previous[next].is_some()
                || ((next_x, next_y) != goal && excluded.contains(&next))
                || matches!(
                    grid.cell(next_x as u16, next_y as u16),
                    Some(
                        MapCell::H3Void
                            | MapCell::Water
                            | MapCell::WaterAccessEast
                            | MapCell::WaterAccessWest
                            | MapCell::WaterAccessSouth
                            | MapCell::Building
                            | MapCell::PokecenterNorthWest
                            | MapCell::PokecenterNorthEast
                            | MapCell::PokecenterSouthWest
                            | MapCell::PokecenterSouthEast
                            | MapCell::MartNorthWest
                            | MapCell::MartNorthEast
                            | MapCell::MartSouthWest
                            | MapCell::MartSouthEast
                            | MapCell::Bench
                            | MapCell::TrashCan
                            | MapCell::Fountain
                            | MapCell::GroundSign
                            | MapCell::FenceNorthWest
                            | MapCell::FenceNorth
                            | MapCell::FenceNorthEast
                            | MapCell::FenceWest
                            | MapCell::FenceEast
                            | MapCell::FenceSouthWest
                            | MapCell::FenceSouth
                            | MapCell::FenceSouthEast
                            | MapCell::LedgeWest
                            | MapCell::LedgeMiddle
                            | MapCell::LedgeEast
                            | MapCell::CliffNorthWest
                            | MapCell::CliffNorth
                            | MapCell::CliffNorthEast
                            | MapCell::CliffWest
                            | MapCell::CliffCenter
                            | MapCell::CliffEast
                            | MapCell::CliffSouthWest
                            | MapCell::CliffSouth
                            | MapCell::CliffSouthEast
                            | MapCell::CliffInnerSouthWest
                            | MapCell::CliffInnerSouthEast
                    )
                )
                || (preserve_wild_grass
                    && (next_x, next_y) != goal
                    && (next_x, next_y) != start
                    && matches!(
                        grid.cell(next_x as u16, next_y as u16),
                        Some(
                            MapCell::Park
                                | MapCell::Flowers
                                | MapCell::Tree
                                | MapCell::ParkTree
                                | MapCell::SmallTree
                                | MapCell::SmallTreeSouth
                                | MapCell::Boulder
                                | MapCell::Pitch
                                | MapCell::Rail
                        )
                    ))
            {
                continue;
            }
            previous[next] = Some((x, y));
            frontier.push_back((next_x, next_y));
        }
    }
    None
}

fn terrain_components(grid: &GeneratedGrid, wanted: MapCell) -> Vec<Vec<usize>> {
    let width = usize::from(grid.width);
    let height = usize::from(grid.height);
    let mut visited = vec![false; grid.cells.len()];
    let mut result = Vec::new();
    for start in 0..grid.cells.len() {
        if visited[start] || grid.cells[start] != wanted {
            continue;
        }
        let mut component = Vec::new();
        let mut frontier = vec![start];
        visited[start] = true;
        while let Some(current) = frontier.pop() {
            component.push(current);
            let x = current % width;
            let y = current / width;
            for (next_x, next_y) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if next_x >= width || next_y >= height {
                    continue;
                }
                let next = next_y * width + next_x;
                if !visited[next] && grid.cells[next] == wanted {
                    visited[next] = true;
                    frontier.push(next);
                }
            }
        }
        result.push(component);
    }
    result
}

fn paint_feature(grid: &mut GeneratedGrid, feature: &Feature) {
    let points = feature
        .points
        .iter()
        .map(|point| project(grid, *point))
        .collect::<Vec<_>>();
    if points.is_empty() {
        return;
    }
    let cell = MapCell::from(feature.kind);
    if feature.area && points.len() >= 3 {
        let holes = feature
            .details
            .inner_rings
            .iter()
            .map(|ring| ring.iter().map(|p| project(grid, *p)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for y in 0..grid.height {
            for x in 0..grid.width {
                let px = f64::from(x) + 0.5;
                let py = f64::from(y) + 0.5;
                if point_in_polygon(px, py, &points)
                    && !holes.iter().any(|ring| point_in_polygon(px, py, ring))
                {
                    paint(grid, i32::from(x), i32::from(y), cell);
                }
            }
        }
    }
    for pair in points.windows(2) {
        if !feature.area
            && matches!(
                feature.kind,
                FeatureKind::Water
                    | FeatureKind::Trail
                    | FeatureKind::Street
                    | FeatureKind::Road
                    | FeatureKind::MajorRoad
            )
        {
            paint_cardinal_line(grid, pair[0], pair[1], cell);
        } else {
            paint_line(grid, pair[0], pair[1], 0, cell);
        }
    }
}

pub(crate) fn project(grid: &GeneratedGrid, point: Coordinate) -> (i32, i32) {
    if let Some(plan) = &grid.source.h3 {
        return plan
            .project_to_grid(point, grid.width, grid.height)
            .expect("H3 plan was validated before generation");
    }
    let bounds = grid.source.bounds;
    let x = ((point.lon - bounds.west) / (bounds.east - bounds.west) * f64::from(grid.width - 1))
        .round();
    let y = ((bounds.north - point.lat) / (bounds.north - bounds.south)
        * f64::from(grid.height - 1))
    .round();
    (
        x.clamp(0.0, f64::from(grid.width - 1)) as i32,
        y.clamp(0.0, f64::from(grid.height - 1)) as i32,
    )
}

fn author_h3_boundary(grid: &mut GeneratedGrid, seams: &H3SeamContract) -> Result<()> {
    let plan = grid.source.h3.clone().expect("H3 seam requires H3 plan");
    let polygon = plan.raster_polygon(grid.width, grid.height)?;
    let width = usize::from(grid.width);
    let snapshot = grid.cells.clone();
    let mut inside = vec![false; grid.cells.len()];
    for y in 0..grid.height {
        for x in 0..grid.width {
            inside[usize::from(y) * width + usize::from(x)] =
                point_in_float_polygon(f64::from(x) + 0.5, f64::from(y) + 0.5, &polygon);
        }
    }
    let mut boundary = vec![false; grid.cells.len()];
    for y in 0..grid.height {
        for x in 0..grid.width {
            let index = usize::from(y) * width + usize::from(x);
            boundary[index] = inside[index]
                && [(-1_i32, 0_i32), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .any(|(dx, dy)| {
                        let check_x = i32::from(x) + dx;
                        let check_y = i32::from(y) + dy;
                        check_x < 0
                            || check_y < 0
                            || check_x >= i32::from(grid.width)
                            || check_y >= i32::from(grid.height)
                            || !inside[check_y as usize * width + check_x as usize]
                    });
        }
    }
    for y in 0..grid.height {
        for x in 0..grid.width {
            let index = usize::from(y) * width + usize::from(x);
            if inside[index] && !boundary[index] {
                continue;
            }
            grid.cells[index] = if inside[index] {
                // A storage-face join is not a biome boundary. Preserve the
                // authored terrain except a canopy cell that exists only on
                // this one-cell rim. Forest that continues toward the face
                // interior remains intact, as do every water and transport
                // cell. In particular, a water midpoint must never flood the
                // entire shared edge; exact OSM water painted in the local
                // raster is preserved and the batch grid-seam audit verifies
                // it at off-midpoint samples.
                if matches!(
                    snapshot[index],
                    MapCell::Water
                        | MapCell::WaterAccessEast
                        | MapCell::WaterAccessWest
                        | MapCell::WaterAccessSouth
                ) {
                    MapCell::Water
                } else {
                    snapshot[index]
                }
            } else {
                MapCell::H3Void
            };
        }
    }

    // Re-open only authoritative shared-edge crossings. The local route was
    // already painted from the same OSM polyline; this short landing merely
    // prevents the natural boundary art from capping its endpoint.
    for edge in seams.edges.iter().filter(|edge| edge.transport.is_some()) {
        let crossing = edge.crossing.expect("transport edge has crossing");
        let cell = MapCell::from(edge.transport.expect("transport edge kind"));
        for (x, y) in crate::h3::h3_raster_sample_band(&plan, grid, crossing)? {
            let index = usize::from(y) * width + usize::from(x);
            if !matches!(snapshot[index], MapCell::Water) {
                grid.cells[index] = cell;
            }
        }
    }
    feather_h3_sampled_canopy_outlines(grid)?;
    Ok(())
}

fn feather_h3_sampled_canopy_outlines(grid: &mut GeneratedGrid) -> Result<()> {
    let plan = grid
        .source
        .h3
        .clone()
        .expect("canopy seam feathering follows validated H3 boundary authoring");
    let profile = crate::build_h3_grid_seam_profile(grid)?;
    let width = usize::from(grid.width);
    let mut outlined = std::collections::BTreeSet::new();
    let mut continuing = std::collections::BTreeSet::new();
    for edge in profile.edges {
        for sample in edge.samples {
            let band = crate::h3::h3_raster_sample_band(&plan, grid, sample.coordinate)?;
            let border = usize::from(band[0].1) * width + usize::from(band[0].0);
            if !h3_canopy_cell(grid.cells[border]) {
                continue;
            }
            let inner = usize::from(band[2].1) * width + usize::from(band[2].0);
            if h3_canopy_cell(grid.cells[inner]) {
                continuing.insert(border);
            } else {
                outlined.insert(border);
            }
        }
    }
    for index in outlined.difference(&continuing).copied() {
        grid.cells[index] = match grid.cells[index] {
            MapCell::ParkTree => MapCell::Park,
            _ => MapCell::Grass,
        };
    }
    Ok(())
}

fn point_in_float_polygon(x: f64, y: f64, polygon: &[(f64, f64)]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current.1 > y) != (previous.1 > y)
            && x < (previous.0 - current.0) * (y - current.1) / (previous.1 - current.1) + current.0
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn paint_line(
    grid: &mut GeneratedGrid,
    from: (i32, i32),
    to: (i32, i32),
    radius: i32,
    cell: MapCell,
) {
    let (mut x, mut y) = from;
    let (x1, y1) = to;
    let dx = (x1 - x).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let dy = -(y1 - y).abs();
    let sy = if y < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        for by in -radius..=radius {
            for bx in -radius..=radius {
                paint(grid, x + bx, y + by, cell);
            }
        }
        if (x, y) == (x1, y1) {
            break;
        }
        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }
}

/// Paints a one-cell-wide line whose successive cells are cardinally connected.
///
/// Ordinary Bresenham output may advance both axes at once. That is visually
/// acceptable for outlines, but it turns a narrow river into disconnected
/// diagonal droplets that disconnect the retained waterway.
fn paint_cardinal_line(grid: &mut GeneratedGrid, from: (i32, i32), to: (i32, i32), cell: MapCell) {
    let (mut x, mut y) = from;
    let (x1, y1) = to;
    let dx = (x1 - x).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let dy = -(y1 - y).abs();
    let sy = if y < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        paint(grid, x, y, cell);
        if (x, y) == (x1, y1) {
            break;
        }
        let previous = (x, y);
        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
        if x != previous.0 && y != previous.1 {
            paint(grid, x, previous.1, cell);
        }
    }
}

pub(crate) fn point_in_polygon(x: f64, y: f64, polygon: &[(i32, i32)]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        let (xi, yi) = (f64::from(current.0), f64::from(current.1));
        let (xj, yj) = (f64::from(previous.0), f64::from(previous.1));
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn paint(grid: &mut GeneratedGrid, x: i32, y: i32, cell: MapCell) {
    if x < 0 || y < 0 || x >= i32::from(grid.width) || y >= i32::from(grid.height) {
        return;
    }
    let index = y as usize * usize::from(grid.width) + x as usize;
    if cell.priority() >= grid.cells[index].priority() {
        grid.cells[index] = cell;
    }
}

fn place_pokecenter(grid: &mut GeneratedGrid) {
    let center_x = i32::from(grid.width / 2);
    let center_y = i32::from(grid.height / 2);
    let target_y = center_y + i32::from(grid.height) / 6;
    let candidate = grid
        .source
        .features
        .iter()
        .filter(|feature| feature.kind == FeatureKind::Building && !feature.points.is_empty())
        .filter_map(|feature| {
            let candidate = project(grid, feature.anchor()?);
            ((candidate.0 - center_x)
                .abs()
                .max((candidate.1 - center_y).abs())
                >= 7
                && house_site_is_clear(grid, candidate.0, candidate.1))
            .then_some(candidate)
        })
        .min_by_key(|&(x, y)| {
            (
                (y - target_y).abs(),
                (x - center_x).abs(),
                hash(x as u16, y as u16),
            )
        });
    let candidate = candidate.or_else(|| {
        (2..grid.height.saturating_sub(4))
            .flat_map(|y| {
                (2..grid.width.saturating_sub(3)).map(move |x| (i32::from(x), i32::from(y)))
            })
            .filter(|&(x, y)| {
                (x - center_x).abs().max((y - center_y).abs()) >= 6
                    && house_site_is_clear(grid, x, y)
            })
            .min_by_key(|&(x, y)| ((y - target_y).abs() + (x - center_x).abs(), y, x))
    });
    let Some((x, y)) = candidate else {
        return;
    };
    for (dx, dy, cell) in [
        (0, 0, MapCell::PokecenterNorthWest),
        (1, 0, MapCell::PokecenterNorthEast),
        (0, 1, MapCell::PokecenterSouthWest),
        (1, 1, MapCell::PokecenterSouthEast),
    ] {
        set_cell(grid, x + dx, y + dy, cell);
    }
    clear_facility_forecourt(grid, x, y);
    connect_frontage(grid, (x, y + 2));
}

pub(crate) fn pokecenter_origin(grid: &GeneratedGrid) -> Option<(u16, u16)> {
    for y in 0..grid.height.saturating_sub(1) {
        for x in 0..grid.width.saturating_sub(1) {
            if grid.cell(x, y) == Some(MapCell::PokecenterNorthWest)
                && grid.cell(x + 1, y) == Some(MapCell::PokecenterNorthEast)
                && grid.cell(x, y + 1) == Some(MapCell::PokecenterSouthWest)
                && grid.cell(x + 1, y + 1) == Some(MapCell::PokecenterSouthEast)
            {
                return Some((x, y));
            }
        }
    }
    None
}

fn place_mart(grid: &mut GeneratedGrid) {
    let (target_x, target_y) = pokecenter_origin(grid)
        .map(|(x, y)| (i32::from(x) + 3, i32::from(y)))
        .unwrap_or((
            i32::from(grid.width / 2) + 6,
            i32::from(grid.height / 2) + i32::from(grid.height) / 6,
        ));
    let (home_x, home_y) = grid.home_cell();
    let mut candidates = std::collections::BTreeMap::<(i32, i32), bool>::new();
    for feature in grid
        .source
        .features
        .iter()
        .filter(|feature| feature.kind == FeatureKind::Building && !feature.points.is_empty())
    {
        candidates.insert(project(grid, feature.anchor().unwrap()), true);
    }
    // Sparse rural inputs may not contain a usable building centroid. The
    // fallback search still stays on the real transport plan and chooses one
    // deterministic service site rather than dropping a fake facade anywhere.
    for y in 2..grid.height.saturating_sub(4) {
        for x in 2..grid.width.saturating_sub(3) {
            candidates.entry((i32::from(x), i32::from(y))).or_default();
        }
    }
    let selected = candidates
        .into_iter()
        .filter(|&((x, y), _)| {
            (x - i32::from(home_x))
                .abs()
                .max((y - i32::from(home_y)).abs())
                >= 6
                && house_site_is_clear(grid, x, y)
        })
        .min_by_key(|&((x, y), real_site)| {
            (
                usize::from(!real_site),
                (x - target_x).abs() + (y - target_y).abs(),
                hash(x as u16, y as u16),
            )
        })
        .map(|(site, _)| site);
    let Some((x, y)) = selected else {
        return;
    };
    for (dx, dy, cell) in [
        (0, 0, MapCell::MartNorthWest),
        (1, 0, MapCell::MartNorthEast),
        (0, 1, MapCell::MartSouthWest),
        (1, 1, MapCell::MartSouthEast),
    ] {
        set_cell(grid, x + dx, y + dy, cell);
    }
    clear_facility_forecourt(grid, x, y);
    connect_frontage(grid, (x, y + 2));
}

pub(crate) fn mart_origin(grid: &GeneratedGrid) -> Option<(u16, u16)> {
    for y in 0..grid.height.saturating_sub(1) {
        for x in 0..grid.width.saturating_sub(1) {
            if grid.cell(x, y) == Some(MapCell::MartNorthWest)
                && grid.cell(x + 1, y) == Some(MapCell::MartNorthEast)
                && grid.cell(x, y + 1) == Some(MapCell::MartSouthWest)
                && grid.cell(x + 1, y + 1) == Some(MapCell::MartSouthEast)
            {
                return Some((x, y));
            }
        }
    }
    None
}

fn place_houses(grid: &mut GeneratedGrid) {
    let urban_intensity = urban_intensity(grid);
    let target_houses = target_house_count(grid);
    let minimum_spacing = match urban_intensity {
        2 => 4,
        1 => 5,
        _ => 6,
    };
    let candidates = grid
        .source
        .features
        .iter()
        .filter(|feature| feature.kind == FeatureKind::Building && !feature.points.is_empty())
        .filter_map(|feature| feature.anchor().map(|point| project(grid, point)))
        .collect::<std::collections::BTreeSet<_>>();
    let center_x = i32::from(grid.width / 2);
    let center_y = i32::from(grid.height / 2);
    let mut zones = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    if urban_intensity == 2 {
        let routes = transport_cells(grid);
        let mut ranked = candidates
            .iter()
            .copied()
            .map(|(x, y)| {
                let local_density = (-5..=5)
                    .flat_map(|dy| (-5..=5).map(move |dx| (x + dx, y + dy)))
                    .filter(|candidate| candidates.contains(candidate))
                    .count();
                let route_distance = routes
                    .iter()
                    .map(|&(route_x, route_y)| (route_x - x).abs() + (route_y - y).abs())
                    .min()
                    .unwrap_or(i32::MAX);
                (
                    std::cmp::Reverse(local_density),
                    route_distance,
                    hash(x as u16, y as u16),
                    x,
                    y,
                )
            })
            .collect::<Vec<_>>();
        ranked.sort_unstable();
        let mut anchors = Vec::<(i32, i32)>::new();
        for (_, route_distance, _, x, y) in ranked {
            if route_distance > 8
                || anchors.iter().any(|&(anchor_x, anchor_y)| {
                    (anchor_x - x).abs().max((anchor_y - y).abs()) < 18
                })
            {
                continue;
            }
            anchors.push((x, y));
            if anchors.len() == zones.len() {
                break;
            }
        }
        for &(x, y) in &candidates {
            if let Some((zone, distance)) = anchors
                .iter()
                .enumerate()
                .map(|(index, &(anchor_x, anchor_y))| {
                    (index, (anchor_x - x).abs().max((anchor_y - y).abs()))
                })
                .min_by_key(|&(index, distance)| (distance, index))
                && distance <= 14
            {
                zones[zone].push((x, y));
            }
        }
    } else {
        for (x, y) in candidates {
            let zone = usize::from(x >= center_x) + usize::from(y >= center_y) * 2;
            zones[zone].push((x, y));
        }
    }
    for zone in &mut zones {
        zone.sort_unstable_by_key(|&(x, y)| (hash(x as u16, y as u16), y, x));
    }

    let mut placed = Vec::<(i32, i32)>::new();
    let mut cursors = [0usize; 4];
    while placed.len() < target_houses {
        let mut progressed = false;
        for zone_index in 0..zones.len() {
            while let Some(&(x, y)) = zones[zone_index].get(cursors[zone_index]) {
                cursors[zone_index] += 1;
                progressed = true;
                let home_x = i32::from(grid.width / 2);
                let home_y = i32::from(grid.height / 2) - 2;
                if (x - home_x).abs().max((y - home_y).abs()) < 5
                    || placed.iter().any(|&(placed_x, placed_y)| {
                        (placed_x - x).abs().max((placed_y - y).abs()) < minimum_spacing
                    })
                    || !house_site_is_clear(grid, x, y)
                {
                    continue;
                }
                stamp_neighborhood_building(grid, x, y, urban_intensity);
                soften_house_yard(grid, x, y);
                connect_house_frontage_for_stamp(grid, x, y);
                placed.push((x, y));
                break;
            }
            if placed.len() == target_houses {
                break;
            }
        }
        if !progressed {
            break;
        }
    }
}

/// Stamp one complete 2x2 residential drawing. Crystal's standalone one-block
/// Goldenrod shops are canonical, but beside full houses they read as cropped
/// facade fragments in a regional overview. Reserve them for their source map;
/// generated neighborhoods use only complete modern/traditional residences.
fn stamp_neighborhood_building(grid: &mut GeneratedGrid, x: i32, y: i32, _urban_intensity: u8) {
    if grid.scene.is_some() {
        let address = StableGrid::for_grid(grid)
            .expect("valid grid")
            .cell(x as u16, y as u16)
            .expect("house inside grid");
        use crate::scene::StructureRecipe::*;
        let scene = grid.scene.as_ref().unwrap();
        let family = scene.districts[y as usize * usize::from(grid.width) + x as usize];
        let choices = match family {
            crate::VisualFamily::Urban => [ModernHouse, RedHouse, YellowHouse, TraditionalHouse],
            crate::VisualFamily::Woodland | crate::VisualFamily::Meadow => {
                [TraditionalHouse, TraditionalHouse, ModernHouse, RedHouse]
            }
            _ => [TraditionalHouse, ModernHouse, RedHouse, YellowHouse],
        };
        let offset = address.stable_hash(0x484f555345) as usize % choices.len();
        let style = (0..choices.len())
            .map(|i| choices[(i + offset) % choices.len()])
            .find(|recipe| {
                scene
                    .structures
                    .iter()
                    .filter(|s| {
                        s.recipe == *recipe
                            && s.origin
                                .0
                                .abs_diff(x as u16)
                                .max(s.origin.1.abs_diff(y as u16))
                                <= 14
                    })
                    .count()
                    < 2
            })
            .unwrap_or(choices[offset]);
        crate::scene::record_structure(grid, x as u16, y as u16, style);
    }
    for dy in 0..2 {
        for dx in 0..2 {
            set_cell(grid, x + dx, y + dy, MapCell::Building);
        }
    }
}

fn connect_house_frontage_for_stamp(grid: &mut GeneratedGrid, x: i32, y: i32) {
    connect_house_frontage(grid, x, y);
}

/// Add the exact Goldenrod department-store and Radio-Tower silhouettes to
/// sufficiently urban maps. They are scarce civic landmarks, selected near
/// separate route districts before ordinary houses consume their footprints.
fn place_city_landmarks(grid: &mut GeneratedGrid) {
    let features = grid.source.features.clone();
    let mut placed = 0;
    for feature in features.iter().filter(|f| f.kind == FeatureKind::Building) {
        let tall = feature
            .details
            .tags
            .get("building:levels")
            .and_then(|v| v.parse::<u16>().ok())
            .is_some_and(|n| n >= 4);
        let civic = feature
            .details
            .tags
            .get("amenity")
            .is_some_and(|s| matches!(s.as_str(), "townhall" | "library" | "university"));
        if !tall && !civic {
            continue;
        }
        let Some(point) = feature.anchor() else {
            continue;
        };
        let (px, py) = project(grid, point);
        let (width, height) = if civic { (2, 6) } else { (3, 4) };
        let site = (-3..=3)
            .flat_map(|dy| (-3..=3).map(move |dx| (px + dx, py + dy)))
            .filter(|&(x, y)| {
                x >= 2
                    && y >= 2
                    && x + i32::from(width) + 2 < i32::from(grid.width)
                    && y + i32::from(height) + 2 < i32::from(grid.height)
            })
            .map(|(x, y)| (x as u16, y as u16))
            .find(|&(x, y)| building_stamp_site_is_clear(grid, x, y, width, height));
        let Some((x, y)) = site else {
            continue;
        };
        let recipe = if civic {
            crate::StructureRecipe::RadioTower
        } else {
            crate::StructureRecipe::DepartmentStore
        };
        for dy in 0..height {
            for dx in 0..width {
                if recipe.block(dx, dy).is_some() {
                    set_cell(
                        grid,
                        i32::from(x + dx),
                        i32::from(y + dy),
                        MapCell::Building,
                    );
                }
            }
        }
        crate::scene::record_structure(grid, x, y, recipe);
        connect_frontage(
            grid,
            (
                i32::from(x + if width == 3 { 1 } else { 0 }),
                i32::from(y + height),
            ),
        );
        placed += 1;
        if placed >= 2 {
            break;
        }
    }
}

fn structure_avoids_courtyards(
    grid: &GeneratedGrid,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> bool {
    grid.scene.as_ref().is_none_or(|scene| {
        (y..y + height).all(|cy| {
            (x..x + width).all(|cx| {
                !scene
                    .courtyards
                    .get(usize::from(cy) * usize::from(grid.width) + usize::from(cx))
                    .copied()
                    .unwrap_or(true)
            })
        })
    })
}

fn building_stamp_site_is_clear(
    grid: &GeneratedGrid,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> bool {
    structure_avoids_courtyards(grid, x, y, width, height)
        && h3_stamp_fits(
            grid,
            i32::from(x) - 2,
            i32::from(y) - 2,
            width + 4,
            height + 5,
            3,
        )
        && (y..y + height).all(|check_y| {
            (x..x + width).all(|check_x| grid.cell(check_x, check_y) == Some(MapCell::Grass))
        })
        && (0..=7).any(|radius| near_transport(grid, x + width / 2, y + height, radius))
}

pub(crate) fn urban_intensity(grid: &GeneratedGrid) -> u8 {
    let building_features = grid
        .source
        .features
        .iter()
        .filter(|feature| feature.kind == FeatureKind::Building)
        .count();
    let normalized = building_features.saturating_mul(4096)
        / usize::from(grid.width)
            .saturating_mul(usize::from(grid.height))
            .max(1);
    if normalized >= 80 {
        2
    } else if normalized >= 30 {
        1
    } else {
        0
    }
}

pub(crate) fn target_house_count(grid: &GeneratedGrid) -> usize {
    grid.source
        .features
        .iter()
        .filter(|f| f.kind == FeatureKind::Building)
        .count()
        .min(40)
}

fn clear_facility_forecourt(grid: &mut GeneratedGrid, house_x: i32, house_y: i32) {
    for y in house_y - 1..=house_y + 3 {
        for x in house_x - 1..=house_x + 2 {
            if x >= 0 && y >= 0 && grid.cell(x as u16, y as u16) == Some(MapCell::Grass) {
                set_cell(grid, x, y, MapCell::Lawn);
            }
        }
    }
    // Service buildings need a readable porch, not a shared parking lot.
    // Keep the lawn on both sides and author only the west-door approach.
    for y in house_y + 2..=house_y + 3 {
        if matches!(
            grid.cell(house_x as u16, y as u16),
            Some(MapCell::Grass | MapCell::Lawn | MapCell::Clearing)
        ) {
            set_cell(grid, house_x, y, MapCell::Clearing);
        }
    }
}

fn soften_house_yard(grid: &mut GeneratedGrid, house_x: i32, house_y: i32) {
    for y in house_y - 1..=house_y + 3 {
        for x in house_x - 1..=house_x + 2 {
            if x >= 0 && y >= 0 && grid.cell(x as u16, y as u16) == Some(MapCell::Grass) {
                set_cell(grid, x, y, MapCell::Lawn);
            }
        }
    }
}

fn connect_house_frontage(grid: &mut GeneratedGrid, house_x: i32, house_y: i32) {
    connect_frontage(grid, (house_x, house_y + 2));
}

fn connect_frontage(grid: &mut GeneratedGrid, door: (i32, i32)) {
    let mut routes = transport_cells(grid);
    routes.sort_unstable_by_key(|&(x, y)| (x - door.0).abs() + (y - door.1).abs());
    let connector = routes
        .into_iter()
        .take(24)
        .filter_map(|route| shortest_route_path(grid, door, route))
        .filter(|path| new_trail_steps(grid, path) <= 10)
        .min_by_key(Vec::len);
    if let Some(path) = connector {
        commit_trail(grid, path);
    }
}

fn house_site_is_clear(grid: &GeneratedGrid, x: i32, y: i32) -> bool {
    if x < 1 || y < 1 || x + 2 >= i32::from(grid.width) || y + 3 >= i32::from(grid.height) {
        return false;
    }
    if !structure_avoids_courtyards(grid, x as u16, y as u16, 2, 2) {
        return false;
    }
    // Reserve the complete 2x2 facade, its visible 4x5 yard/frontage stamp,
    // and all three inward cells used by reciprocal seam reconciliation.
    // Merely keeping the facade two cells inside the polygon allowed a valid
    // house at x=2 to collide with sample depth two and forced a late choice
    // between truncating the house and preserving a geographic join.
    if !h3_stamp_fits(grid, x - 1, y - 1, 4, 5, 3) {
        return false;
    }
    // Keep ordinary facades as separate readable components from a previously
    // authored Department Store, Radio Tower, Center, or Mart. Without this
    // halo a valid 2x2 footprint could touch a landmark side and merge both
    // exact stamps into one malformed building component.
    if (y - 1..=y + 2).any(|check_y| {
        (x - 1..=x + 2).any(|check_x| {
            check_x >= 0
                && check_y >= 0
                && matches!(
                    grid.cell(check_x as u16, check_y as u16),
                    Some(
                        MapCell::Building
                            | MapCell::PokecenterNorthWest
                            | MapCell::PokecenterNorthEast
                            | MapCell::PokecenterSouthWest
                            | MapCell::PokecenterSouthEast
                            | MapCell::MartNorthWest
                            | MapCell::MartNorthEast
                            | MapCell::MartSouthWest
                            | MapCell::MartSouthEast
                    )
                )
        })
    }) {
        return false;
    }
    // Keep the exact 2x2 footprint off water, parks, rails, and roads. The
    // separate placement radius guarantees a visible yard around each house.
    for check_y in y..=y + 1 {
        for check_x in x..=x + 1 {
            let Some(cell) = grid.cell(check_x as u16, check_y as u16) else {
                return false;
            };
            if !matches!(cell, MapCell::Grass) {
                return false;
            }
        }
    }
    // The canonical residential and service facades put the door in the SE
    // quadrant of the southwest block, so the frontage block is below the
    // west (not east) half of the building.
    let door = (x, y + 2);
    transport_cells(grid)
        .into_iter()
        .any(|(road_x, road_y)| (road_x - door.0).abs() + (road_y - door.1).abs() <= 6)
}

pub(crate) fn reachable_walkable_cells(grid: &GeneratedGrid, start: (u16, u16)) -> Vec<bool> {
    let mut reached = vec![false; grid.cells.len()];
    let start = usize::from(start.1) * usize::from(grid.width) + usize::from(start.0);
    if !is_walkable_cell(grid.cells[start]) {
        return reached;
    }
    let mut frontier = std::collections::VecDeque::from([start]);
    reached[start] = true;
    while let Some(index) = frontier.pop_front() {
        let x = index % usize::from(grid.width);
        let y = index / usize::from(grid.width);
        for (next_x, next_y) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if next_x >= usize::from(grid.width) || next_y >= usize::from(grid.height) {
                continue;
            }
            let next = next_y * usize::from(grid.width) + next_x;
            if !reached[next] && is_walkable_cell(grid.cells[next]) {
                reached[next] = true;
                frontier.push_back(next);
            }
        }
    }
    reached
}

pub(crate) fn is_walkable_cell(cell: MapCell) -> bool {
    !matches!(
        cell,
        MapCell::H3Void
            | MapCell::Building
            | MapCell::PokecenterNorthWest
            | MapCell::PokecenterNorthEast
            | MapCell::PokecenterSouthWest
            | MapCell::PokecenterSouthEast
            | MapCell::MartNorthWest
            | MapCell::MartNorthEast
            | MapCell::MartSouthWest
            | MapCell::MartSouthEast
            | MapCell::Water
            | MapCell::WaterAccessEast
            | MapCell::WaterAccessWest
            | MapCell::WaterAccessSouth
            | MapCell::Tree
            | MapCell::ParkTree
            | MapCell::SmallTree
            | MapCell::SmallTreeSouth
            | MapCell::Boulder
            | MapCell::IceBoulder
            | MapCell::Bench
            | MapCell::TrashCan
            | MapCell::Fountain
            | MapCell::GroundSign
            | MapCell::FenceNorthWest
            | MapCell::FenceNorth
            | MapCell::FenceNorthEast
            | MapCell::FenceWest
            | MapCell::FenceEast
            | MapCell::FenceSouthWest
            | MapCell::FenceSouth
            | MapCell::FenceSouthEast
            | MapCell::LedgeWest
            | MapCell::LedgeMiddle
            | MapCell::LedgeEast
            | MapCell::CliffNorthWest
            | MapCell::CliffNorth
            | MapCell::CliffNorthEast
            | MapCell::CliffWest
            | MapCell::CliffCenter
            | MapCell::CliffEast
            | MapCell::CliffSouthWest
            | MapCell::CliffSouth
            | MapCell::CliffSouthEast
            | MapCell::CliffInnerSouthWest
            | MapCell::CliffInnerSouthEast
    )
}

fn set_cell(grid: &mut GeneratedGrid, x: i32, y: i32, cell: MapCell) {
    if x >= 0 && y >= 0 && x < i32::from(grid.width) && y < i32::from(grid.height) {
        let index = y as usize * usize::from(grid.width) + x as usize;
        if matches!(
            grid.cells[index],
            MapCell::Street | MapCell::Road | MapCell::MajorRoad
        ) && !matches!(cell, MapCell::Street | MapCell::Road | MapCell::MajorRoad)
        {
            return;
        }
        grid.cells[index] = cell;
    }
}

fn building_block(grid: &GeneratedGrid, x: u16, y: u16) -> u16 {
    if let Some(scene) = &grid.scene {
        return scene
            .structures
            .iter()
            .find_map(|structure| {
                structure.recipe.block(
                    x.checked_sub(structure.origin.0)?,
                    y.checked_sub(structure.origin.1)?,
                )
            })
            .unwrap_or(0x02);
    }
    0x02
}

fn park_block(_grid: &GeneratedGrid, x: u16, y: u16) -> u16 {
    if hash(x, y).is_multiple_of(3) {
        u16::from(crate::GENERATED_PARK_LONG_GRASS_METATILE)
    } else {
        0x03
    }
}

fn flower_block(x: u16, y: u16) -> u16 {
    if hash(x, y).is_multiple_of(2) {
        u16::from(crate::GENERATED_PARK_FLOWER_BED_METATILE)
    } else {
        0x04
    }
}

fn tree_block(_grid: &GeneratedGrid, _x: u16, _y: u16) -> u16 {
    0x05
}

fn h3_void_block(x: u16, y: u16) -> u16 {
    let district_x = x / 6;
    let district_y = y / 6;
    let seed = hash(district_x, district_y);
    let local = (x % 6, y % 6);
    let rock_shape = if seed & 1 == 0 {
        matches!(local, (1, 1) | (3, 0) | (4, 2) | (2, 3) | (0, 4))
    } else {
        matches!(local, (0, 1) | (2, 0) | (4, 1) | (3, 3) | (1, 4))
    };
    if seed.is_multiple_of(3) && rock_shape {
        0x0a
    } else if hash(x, y).is_multiple_of(7) {
        u16::from(crate::GENERATED_PARK_TREE_METATILE)
    } else {
        0x05
    }
}

fn near_transport(grid: &GeneratedGrid, x: u16, y: u16, radius: i32) -> bool {
    for check_y in i32::from(y) - radius..=i32::from(y) + radius {
        for check_x in i32::from(x) - radius..=i32::from(x) + radius {
            if check_x < 0
                || check_y < 0
                || check_x >= i32::from(grid.width)
                || check_y >= i32::from(grid.height)
            {
                continue;
            }
            if matches!(
                grid.cell(check_x as u16, check_y as u16),
                Some(MapCell::Trail | MapCell::Street | MapCell::Road | MapCell::MajorRoad)
            ) {
                return true;
            }
        }
    }
    false
}

fn h3_stamp_fits(
    grid: &GeneratedGrid,
    x: i32,
    y: i32,
    width: u16,
    height: u16,
    clearance: u16,
) -> bool {
    grid.source.h3.as_ref().is_none_or(|plan| {
        plan.raster_footprint_fits(x, y, width, height, clearance, grid.width, grid.height)
            .expect("H3 plan was validated before generation")
    })
}

fn h3_protected_cell_fits(grid: &GeneratedGrid, x: u16, y: u16) -> bool {
    h3_stamp_fits(grid, i32::from(x), i32::from(y), 1, 1, 3)
}

fn scenic_connector(
    grid: &GeneratedGrid,
    entry: (i32, i32),
    max_new_steps: usize,
) -> Option<Vec<(i32, i32)>> {
    let mut route_cells = transport_cells(grid);
    route_cells.sort_unstable_by_key(|&(x, y)| (x - entry.0).abs() + (y - entry.1).abs());
    route_cells
        .into_iter()
        .take(32)
        .filter_map(|route| shortest_land_path(grid, entry, route))
        .filter(|path| {
            new_trail_steps(grid, path) <= max_new_steps
                && path_turns(path) <= 2
                && path.iter().all(|&(x, y)| {
                    matches!(
                        grid.cell(x as u16, y as u16),
                        Some(
                            MapCell::Grass
                                | MapCell::Lawn
                                | MapCell::Clearing
                                | MapCell::Tree
                                | MapCell::ParkTree
                                | MapCell::SmallTree
                                | MapCell::SmallTreeSouth
                                | MapCell::Trail
                                | MapCell::Street
                                | MapCell::Road
                                | MapCell::MajorRoad
                        )
                    )
                })
        })
        .min_by_key(|path| (new_trail_steps(grid, path), path_turns(path), path.len()))
}

fn new_trail_steps(grid: &GeneratedGrid, path: &[(i32, i32)]) -> usize {
    path.iter()
        .filter(|&&(x, y)| {
            !matches!(
                grid.cell(x as u16, y as u16),
                Some(MapCell::Trail | MapCell::Street | MapCell::Road | MapCell::MajorRoad)
            )
        })
        .count()
}

fn path_turns(path: &[(i32, i32)]) -> usize {
    path.windows(3)
        .filter(|points| {
            let first = (points[1].0 - points[0].0, points[1].1 - points[0].1);
            let second = (points[2].0 - points[1].0, points[2].1 - points[1].1);
            first != second
        })
        .count()
}

fn create_water_access(grid: &mut GeneratedGrid) {
    let transports = grid
        .cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            matches!(
                cell,
                MapCell::Trail | MapCell::Street | MapCell::Road | MapCell::MajorRoad
            )
            .then_some((
                (index % usize::from(grid.width)) as i32,
                (index / usize::from(grid.width)) as i32,
            ))
        })
        .collect::<Vec<_>>();
    let Some(principal_water) = terrain_components(grid, MapCell::Water)
        .into_iter()
        .max_by_key(Vec::len)
    else {
        return;
    };
    let mut principal = vec![false; grid.cells.len()];
    for index in principal_water {
        principal[index] = true;
    }
    let mut best: Option<(usize, u16, u16, i32, i32, i32, i32, MapCell)> = None;
    for y in 1..grid.height - 1 {
        for x in 1..grid.width - 1 {
            let index = usize::from(y) * usize::from(grid.width) + usize::from(x);
            if !principal[index] || !h3_protected_cell_fits(grid, x, y) {
                continue;
            }
            for (land_x, land_y, access) in [
                (i32::from(x) - 1, i32::from(y), MapCell::WaterAccessEast),
                (i32::from(x) + 1, i32::from(y), MapCell::WaterAccessWest),
                (i32::from(x), i32::from(y) - 1, MapCell::WaterAccessSouth),
            ] {
                let Some(land) = grid.cell(land_x as u16, land_y as u16) else {
                    continue;
                };
                if matches!(
                    land,
                    MapCell::Water
                        | MapCell::WaterAccessEast
                        | MapCell::WaterAccessWest
                        | MapCell::WaterAccessSouth
                        | MapCell::Building
                        | MapCell::PokecenterNorthWest
                        | MapCell::PokecenterNorthEast
                        | MapCell::PokecenterSouthWest
                        | MapCell::PokecenterSouthEast
                        | MapCell::MartNorthWest
                        | MapCell::MartNorthEast
                        | MapCell::MartSouthWest
                        | MapCell::MartSouthEast
                        | MapCell::Bench
                        | MapCell::TrashCan
                        | MapCell::Fountain
                ) {
                    continue;
                }
                for &(road_x, road_y) in &transports {
                    let Some(path) = shortest_land_path(grid, (land_x, land_y), (road_x, road_y))
                    else {
                        continue;
                    };
                    if best.is_none_or(|(best_distance, ..)| path.len() < best_distance) {
                        best = Some((path.len(), x, y, land_x, land_y, road_x, road_y, access));
                    }
                }
            }
        }
    }
    let Some((_, water_x, water_y, land_x, land_y, road_x, road_y, access)) = best else {
        return;
    };
    set_cell(grid, i32::from(water_x), i32::from(water_y), access);
    if let Some(path) = shortest_land_path(grid, (land_x, land_y), (road_x, road_y)) {
        for (x, y) in path {
            if !matches!(
                grid.cell(x as u16, y as u16),
                Some(
                    MapCell::Water
                        | MapCell::WaterAccessEast
                        | MapCell::WaterAccessWest
                        | MapCell::WaterAccessSouth
                        | MapCell::Building
                        | MapCell::PokecenterNorthWest
                        | MapCell::PokecenterNorthEast
                        | MapCell::PokecenterSouthWest
                        | MapCell::PokecenterSouthEast
                        | MapCell::MartNorthWest
                        | MapCell::MartNorthEast
                        | MapCell::MartSouthWest
                        | MapCell::MartSouthEast
                )
            ) {
                set_cell(grid, x, y, MapCell::Trail);
            }
        }
    }
}

fn water_block(grid: &GeneratedGrid, x: u16, y: u16) -> u16 {
    let water = |neighbor_x: i32, neighbor_y: i32| {
        if neighbor_x < 0
            || neighbor_y < 0
            || neighbor_x >= i32::from(grid.width)
            || neighbor_y >= i32::from(grid.height)
        {
            return true;
        }
        matches!(
            grid.cell(neighbor_x as u16, neighbor_y as u16),
            Some(
                MapCell::Water
                    | MapCell::WaterAccessEast
                    | MapCell::WaterAccessWest
                    | MapCell::WaterAccessSouth
            )
        ) || (grid.source.h3.is_some()
            && grid.cell(neighbor_x as u16, neighbor_y as u16) == Some(MapCell::H3Void))
    };
    let north = water(i32::from(x), i32::from(y) - 1);
    let south = water(i32::from(x), i32::from(y) + 1);
    let west = water(i32::from(x) - 1, i32::from(y));
    let east = water(i32::from(x) + 1, i32::from(y));
    match (north, south, west, east) {
        (false, _, false, _) => 0x54,
        (false, _, _, false) => 0x55,
        (false, _, _, _) => 0x76,
        (_, _, false, _) if south => 0x58,
        (_, _, _, false) if south => 0x59,
        // johto_modern has no south-facing rocky bank. Canonical Johto maps
        // regularly place open water directly against land here; doing so is
        // both collision-correct and avoids a continuous wall of buoy teeth.
        (_, false, _, _) => 0x35,
        (_, _, false, _) => 0x58,
        (_, _, _, false) => 0x59,
        _ => 0x35,
    }
}

fn h3_canopy_cell(cell: MapCell) -> bool {
    matches!(
        cell,
        MapCell::Tree | MapCell::ParkTree | MapCell::SmallTree | MapCell::SmallTreeSouth
    )
}

fn select_labels(grid: &GeneratedGrid) -> Vec<GridLabel> {
    let mut candidates = grid
        .source
        .features
        .iter()
        .filter_map(|feature| {
            let name = feature.name.as_ref()?;
            let point = feature.points.get(feature.points.len() / 2)?;
            let (x, y) = project(grid, *point);
            Some((feature.points.len(), name.clone(), x, y))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    candidates.dedup_by(|left, right| left.1 == right.1);
    let signs = grid
        .cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            (*cell == MapCell::GroundSign).then_some((
                (index % usize::from(grid.width)) as u16,
                (index / usize::from(grid.width)) as u16,
            ))
        })
        .collect::<Vec<_>>();
    let mut labels = grid.labels.clone();
    let mut used_names = labels
        .iter()
        .map(|label| label.text.clone())
        .collect::<std::collections::BTreeSet<_>>();
    for (sign_x, sign_y) in signs {
        if labels
            .iter()
            .any(|label| label.x == sign_x && label.y == sign_y)
        {
            continue;
        }
        let nearest = candidates
            .iter()
            .filter(|(_, name, _, _)| !used_names.contains(name))
            .min_by_key(|(_, _, feature_x, feature_y)| {
                (i32::from(sign_x) - *feature_x).abs() + (i32::from(sign_y) - *feature_y).abs()
            });
        let Some((_, name, _, _)) = nearest else {
            continue;
        };
        used_names.insert(name.clone());
        labels.push(GridLabel {
            text: name.clone(),
            x: sign_x,
            y: sign_y,
        });
    }
    labels
}

fn hash(x: u16, y: u16) -> u32 {
    u32::from(x).wrapping_mul(0x45d9_f3b) ^ u32::from(y).wrapping_mul(0x27d4_eb2d)
}
