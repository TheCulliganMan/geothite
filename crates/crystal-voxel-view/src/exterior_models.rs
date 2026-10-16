//! Original editable world-exterior meshes. No source textures or game data.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum Kind {
    EastGate,
    TinTower,
    BurnedTower,
    Lighthouse,
    JohtoBarn,
    ForestShrine,
    RadioTower,
    BattleTower,
    ModernHouse,
    ModernBlank,
    ModernShop,
    KantoHouse,
    KantoFlatHouse,
    ModernCenter,
    ModernMart,
    ModernGym,
    ModernStation,
    ModernDepartment,
    ModernArcade,
    KantoCenter,
    KantoMart,
    KantoGym,
    KantoStation,
    KantoDepartment,
    KantoArcade,
    Silph,
    KantoMansion,
    PowerPlant,
    ModernDaycare,
    KantoCanopy,
    CutTree,
    ParkHedge,
    ShoreBoulder,
    RouteSign,
    ParkBench,
    TimberFence,
    KantoBlank,
    ForestGate,
    ForestGateClosed,
    HighlandMesa,
    DiglettCave,
    TowerForecourt,
    LavenderRadioTower,
    KantoRouteGate,
    IceShelfNorthwest,
    IceShelfNorth,
    IceShelfNortheast,
    IceShelfInterior,
    IceShelfSouthwest,
    IceShelfSouth,
    IceShelfSoutheast,
    IceShelfNotchEast,
    IceShelfNotchWest,
    IceShelfStairRight,
    IceShelfStairCorner,
    IceShelfStairLeft,
    KantoBoundaryLand,
    KantoBoundaryShore,
    KantoBoundaryPath,
    KantoCappedPost,
}
impl Kind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::EastGate => "world-exterior/east_gate",
            Self::TinTower => "world-exterior/tin_tower",
            Self::BurnedTower => "world-exterior/burned_tower",
            Self::Lighthouse => "world-exterior/lighthouse",
            Self::JohtoBarn => "world-exterior/johto_barn",
            Self::ForestShrine => "world-exterior/forest_shrine",
            Self::RadioTower => "world-exterior/radio_tower",
            Self::BattleTower => "world-exterior/battle_tower",
            Self::ModernHouse => "world-exterior/modern_house",
            Self::ModernBlank => "world-exterior/modern_blank",
            Self::ModernShop => "world-exterior/modern_shop",
            Self::KantoHouse => "world-exterior/kanto_house",
            Self::KantoFlatHouse => "world-exterior/kanto_flat_house",
            Self::ModernCenter => "world-exterior/modern_center",
            Self::ModernMart => "world-exterior/modern_mart",
            Self::ModernGym => "world-exterior/modern_gym",
            Self::ModernStation => "world-exterior/modern_station",
            Self::ModernDepartment => "world-exterior/modern_department",
            Self::ModernArcade => "world-exterior/modern_arcade",
            Self::KantoCenter => "world-exterior/kanto_center",
            Self::KantoMart => "world-exterior/kanto_mart",
            Self::KantoGym => "world-exterior/kanto_gym",
            Self::KantoStation => "world-exterior/kanto_station",
            Self::KantoDepartment => "world-exterior/kanto_department",
            Self::KantoArcade => "world-exterior/kanto_arcade",
            Self::Silph => "world-exterior/silph",
            Self::KantoMansion => "world-exterior/kanto_mansion",
            Self::PowerPlant => "world-exterior/power_plant",
            Self::ModernDaycare => "world-exterior/modern_daycare",
            Self::KantoCanopy => "world-exterior/kanto_canopy",
            Self::CutTree => "world-exterior/cut_tree",
            Self::ParkHedge => "world-exterior/park_hedge",
            Self::ShoreBoulder => "world-exterior/shore_boulder",
            Self::RouteSign => "world-exterior/route_sign",
            Self::ParkBench => "world-exterior/park_bench",
            Self::TimberFence => "world-exterior/timber_fence",
            Self::KantoBlank => "world-exterior/kanto_blank",
            Self::ForestGate => "world-exterior/forest_gate",
            Self::ForestGateClosed => "world-exterior/forest_gate_closed",
            Self::HighlandMesa => "world-exterior/highland_mesa",
            Self::DiglettCave => "world-exterior/diglett_cave",
            Self::TowerForecourt => "world-exterior/tower_forecourt",
            Self::LavenderRadioTower => "world-exterior/lavender_radio_tower",
            Self::KantoRouteGate => "world-exterior/kanto_route_gate",
            Self::IceShelfNorthwest => "world-exterior/ice_shelf_northwest",
            Self::IceShelfNorth => "world-exterior/ice_shelf_north",
            Self::IceShelfNortheast => "world-exterior/ice_shelf_northeast",
            Self::IceShelfInterior => "world-exterior/ice_shelf_interior",
            Self::IceShelfSouthwest => "world-exterior/ice_shelf_southwest",
            Self::IceShelfSouth => "world-exterior/ice_shelf_south",
            Self::IceShelfSoutheast => "world-exterior/ice_shelf_southeast",
            Self::IceShelfNotchEast => "world-exterior/ice_shelf_notch_east",
            Self::IceShelfNotchWest => "world-exterior/ice_shelf_notch_west",
            Self::IceShelfStairRight => "world-exterior/ice_shelf_stair_right",
            Self::IceShelfStairCorner => "world-exterior/ice_shelf_stair_corner",
            Self::IceShelfStairLeft => "world-exterior/ice_shelf_stair_left",
            Self::KantoBoundaryLand => "world-exterior/kanto_boundary_land",
            Self::KantoBoundaryShore => "world-exterior/kanto_boundary_shore",
            Self::KantoBoundaryPath => "world-exterior/kanto_boundary_path",
            Self::KantoCappedPost => "world-exterior/kanto_capped_post",
        }
    }
}

pub(crate) const ALL: &[Kind] = &[
    Kind::EastGate,
    Kind::TinTower,
    Kind::BurnedTower,
    Kind::Lighthouse,
    Kind::JohtoBarn,
    Kind::ForestShrine,
    Kind::RadioTower,
    Kind::BattleTower,
    Kind::ModernHouse,
    Kind::ModernBlank,
    Kind::ModernShop,
    Kind::KantoHouse,
    Kind::KantoFlatHouse,
    Kind::ModernCenter,
    Kind::ModernMart,
    Kind::ModernGym,
    Kind::ModernStation,
    Kind::ModernDepartment,
    Kind::ModernArcade,
    Kind::KantoCenter,
    Kind::KantoMart,
    Kind::KantoGym,
    Kind::KantoStation,
    Kind::KantoDepartment,
    Kind::KantoArcade,
    Kind::Silph,
    Kind::KantoMansion,
    Kind::PowerPlant,
    Kind::ModernDaycare,
    Kind::KantoCanopy,
    Kind::CutTree,
    Kind::ParkHedge,
    Kind::ShoreBoulder,
    Kind::RouteSign,
    Kind::ParkBench,
    Kind::TimberFence,
    Kind::KantoBlank,
    Kind::ForestGate,
    Kind::ForestGateClosed,
    Kind::HighlandMesa,
    Kind::DiglettCave,
    Kind::TowerForecourt,
    Kind::LavenderRadioTower,
    Kind::KantoRouteGate,
    Kind::IceShelfNorthwest,
    Kind::IceShelfNorth,
    Kind::IceShelfNortheast,
    Kind::IceShelfInterior,
    Kind::IceShelfSouthwest,
    Kind::IceShelfSouth,
    Kind::IceShelfSoutheast,
    Kind::IceShelfNotchEast,
    Kind::IceShelfNotchWest,
    Kind::IceShelfStairRight,
    Kind::IceShelfStairCorner,
    Kind::IceShelfStairLeft,
    Kind::KantoBoundaryLand,
    Kind::KantoBoundaryShore,
    Kind::KantoBoundaryPath,
    Kind::KantoCappedPost,
];
const JSON: &[crate::model_storage::Source<'static>] = &[
    crate::model_storage::include_model!("models/world_exteriors/east_gate.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/tin_tower.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/burned_tower.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/lighthouse.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/johto_barn.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/forest_shrine.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/radio_tower.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/battle_tower.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_house.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_blank.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_shop.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_house.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_flat_house.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_center.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_mart.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_gym.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_station.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_department.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_arcade.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_center.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_mart.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_gym.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_station.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_department.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_arcade.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/silph.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_mansion.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/power_plant.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/modern_daycare.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_canopy.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/cut_tree.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/park_hedge.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/shore_boulder.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/route_sign.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/park_bench.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/timber_fence.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_blank.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/forest_gate.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/forest_gate_closed.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/highland_mesa.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/diglett_cave.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/tower_forecourt.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/lavender_radio_tower.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_route_gate.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_northwest.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_north.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_northeast.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_interior.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_southwest.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_south.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_southeast.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_notch_east.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_notch_west.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_stair_right.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_stair_corner.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/ice_shelf_stair_left.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_boundary_land.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_boundary_shore.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_boundary_path.mesh.json"),
    crate::model_storage::include_model!("models/world_exteriors/kanto_capped_post.mesh.json"),
];

#[derive(Deserialize)]
struct Primitive {
    #[serde(default)]
    material: String,
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct Export {
    #[serde(default)]
    name: String,
    primitives: Vec<Primitive>,
    door_anchor: Option<[f32; 3]>,
    door_face: Option<String>,
}
#[derive(Clone)]
pub(crate) struct Model {
    materials: Vec<(std::ops::Range<usize>, String, [f32; 4])>,
    pub(crate) surface: SurfaceMeshData,
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
    pub(crate) anchor: Option<[f32; 3]>,
    east: bool,
}

/// Fair the authored closed stone volume without adding render triangles.
/// Welding by position keeps material-band seams closed. The underside stays
/// seated; paired positive/negative smoothing avoids shrinking the whole rock.
fn round_boundary_stone(surface: &mut SurfaceMeshData, min: [f32; 3], max: [f32; 3]) {
    let mut unique = std::collections::BTreeMap::new();
    let mut points = Vec::<Vec3>::new();
    let mut remap = Vec::with_capacity(surface.positions.len());
    for &p in &surface.positions {
        let key = p.map(f32::to_bits);
        let id = *unique.entry(key).or_insert_with(|| {
            let id = points.len();
            points.push(Vec3::from_array(p));
            id
        });
        remap.push(id);
    }
    let mut neighbors = vec![std::collections::BTreeSet::new(); points.len()];
    for tri in surface.indices.chunks_exact(3) {
        let ids = [tri[0], tri[1], tri[2]].map(|i| remap[i as usize]);
        for (a, b) in [(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0])] {
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }
    let seated = points
        .iter()
        .map(|p| (p.y - min[1]).abs() < 0.00001)
        .collect::<Vec<_>>();
    for weight in [0.32, -0.34, 0.32, -0.34] {
        let old = points.clone();
        for (i, p) in points.iter_mut().enumerate() {
            if seated[i] || neighbors[i].is_empty() {
                continue;
            }
            let average =
                neighbors[i].iter().map(|&j| old[j]).sum::<Vec3>() / neighbors[i].len() as f32;
            *p = (old[i] + (average - old[i]) * weight)
                .clamp(Vec3::from_array(min), Vec3::from_array(max));
        }
    }
    let mut normals = vec![Vec3::ZERO; points.len()];
    let mut pigments = vec![Vec3::ZERO; points.len()];
    let mut weights = vec![0.0; points.len()];
    for tri in surface.indices.chunks_exact(3) {
        let ids = [tri[0], tri[1], tri[2]].map(|i| remap[i as usize]);
        let normal = (points[ids[1]] - points[ids[0]]).cross(points[ids[2]] - points[ids[0]]);
        let area = normal.length();
        for (corner, i) in ids.into_iter().enumerate() {
            normals[i] += normal;
            let color = surface.colors[tri[corner] as usize];
            pigments[i] += Vec3::new(color[0], color[1], color[2]) * area;
            weights[i] += area;
        }
    }
    for (i, &id) in remap.iter().enumerate() {
        surface.positions[i] = points[id].to_array();
        surface.normals[i] = normals[id].normalize_or_zero().to_array();
        if weights[id] > 0.0 {
            surface.colors[i][..3].copy_from_slice(&(pigments[id] / weights[id]).to_array());
        }
    }
}
impl Model {
    fn parse<'a>(json: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
        let mut materials = Vec::new();
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in export.primitives {
            let n = p.positions.len() / 3;
            if n == 0
                || p.positions.len() % 3 != 0
                || p.normals.len() != p.positions.len()
                || p.indices.is_empty()
                || p.indices.len() % 3 != 0
                || p.indices.iter().any(|&i| i as usize >= n)
                || p.positions.iter().chain(&p.normals).any(|v| !v.is_finite())
                || p.base_color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err("invalid exterior primitive".into());
            }
            let base = surface.positions.len() as u32;
            let index_start = surface.indices.len();
            for (v, n) in p.positions.chunks_exact(3).zip(p.normals.chunks_exact(3)) {
                let pos = [v[0], v[1], v[2]];
                let normal = Vec3::new(n[0], n[1], n[2]);
                if normal.length_squared() < 0.000001 {
                    return Err("zero exterior normal".into());
                }
                for i in 0..3 {
                    min[i] = min[i].min(pos[i]);
                    max[i] = max[i].max(pos[i]);
                }
                surface.positions.push(pos);
                surface.normals.push(normal.normalize().to_array());
                surface.colors.push(p.base_color);
                surface.uvs.push([0.0, 0.0]);
            }
            surface
                .indices
                .extend(p.indices.into_iter().map(|i| i + base));
            // These gates already contain closed, overlapping ceramic shingles.
            // Re-tessellating their half faces as old slab courses destroys the
            // authored profile and multiplies their runtime triangle cost.
            let authored_roof = matches!(export.name.as_str(), "forest_gate" | "forest_gate_closed" | "east_gate")
                && p.material.starts_with("roof_");
            if authored_roof {
                let sun = Vec3::new(0.45, 0.8, 0.55).normalize();
                for i in base as usize..surface.positions.len() {
                    let light = 0.78 + 0.22 * Vec3::from_array(surface.normals[i]).dot(sun).max(0.);
                    for channel in 0..3 { surface.colors[i][channel] *= light; }
                }
            }
            if !authored_roof
                && !crate::roof_tiles::refine(&mut surface, base as usize, index_start, &p.material) {
                crate::scenery_finish::refine(
                    &mut surface,
                    base as usize,
                    index_start,
                    &p.material,
                    &export.name,
                );
            }
            materials.push((
                base as usize..surface.positions.len(),
                p.material,
                p.base_color,
            ));
        }
        crate::scenery_finish::contain(&mut surface, min, max);
        if matches!(
            export.name.as_str(),
            "kanto_boundary_land" | "kanto_boundary_shore" | "kanto_boundary_path"
        ) {
            round_boundary_stone(&mut surface, min, max);
        }
        if (0..3).any(|i| !min[i].is_finite() || max[i] <= min[i]) {
            return Err("empty exterior mesh".into());
        }
        let east = export.door_face.as_deref() == Some("east");
        if let Some(a) = export.door_anchor {
            let along = if east { 2 } else { 0 };
            let seam = if east { 0 } else { 2 };
            if a.iter().any(|v| !v.is_finite())
                || a[along] <= min[along]
                || a[along] >= max[along]
                || (a[seam] - max[seam]).abs() > 0.001
            {
                return Err("invalid exterior door seam".into());
            }
        }
        Ok(Self {
            materials,
            surface,
            min,
            max,
            anchor: export.door_anchor,
            east,
        })
    }
    pub(crate) fn material_ranges(&self) -> impl Iterator<Item = (std::ops::Range<usize>, &str)> {
        self.materials.iter().map(|(range, name, _)| (range.clone(), name.as_str()))
    }
    pub(crate) fn vertex_material(&self, vertex: usize) -> &str {
        self.materials
            .iter()
            .find(|(range, _, _)| range.contains(&vertex))
            .map_or("", |(_, name, _)| name.as_str())
    }
    /// Keep the modeled joinery, but derive fence paint from the live atlas.
    pub(crate) fn recolor_fitted_fence(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];3]) {
        for (range, name, original) in &self.materials {
            let color: [f32;4] = match name.as_str() {
                "timber_light" => std::array::from_fn(|i| palette[2][i]*0.65+palette[1][i]*0.35),
                "timber_gold" => std::array::from_fn(|i| palette[2][i]*0.45+palette[1][i]*0.55),
                "timber" => std::array::from_fn(|i| palette[0][i]*0.75+palette[1][i]*0.25),
                _ => continue,
            };
            for index in range.clone() {
                let existing=target.colors[first+index];
                let shade=(existing[0]+existing[1]+existing[2])/
                    (original[0]+original[1]+original[2]).max(0.0001);
                for channel in 0..3 {
                    target.colors[first+index][channel]=color[channel]*shade;
                }
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }
    /// Notice-board paint follows the live source, including the dark marks.
    pub(crate) fn recolor_fitted_sign(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];3]) {
        for (range,name,original) in &self.materials {
            let paint = match name.as_str() {
                "cream" => palette[2],
                "timber" | "yellow" => palette[1],
                "timber_light" => palette[1],
                _ => continue,
            };
            for index in range.clone() {
                // This material combines the posts and raised notice marks.
                // Only the marks lie in front of the recessed panel.
                let paint=if name=="timber_light" && self.surface.positions[index][2] > self.max[2]*0.5 {
                    palette[0]
                } else { paint };
                let color=target.colors[first+index];
                let shade=(color[0]+color[1]+color[2]) /
                    (original[0]+original[1]+original[2]).max(0.0001);
                for channel in 0..3 { target.colors[first+index][channel]=paint[channel]*shade; }
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }
    /// Forest masonry, pale structural trim and yellow window paint each come
    /// from their own live drawing. Roof, door and insignia retain their roles.
    pub(crate) fn recolor_fitted_forest_wall(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];6]) {
        for (range,name,original) in &self.materials {
            let (paint,shade)=match name.as_str() {
                "plaster" => (palette[0],1.),
                "plaster_forest_mortar" => (palette[1],0.82),
                "plaster_shadow" => (palette[0],0.78),
                "plaster_light" | "timber_light" | "stone_light" => (palette[1],1.),
                "timber" | "timber_gold" | "stone" => (palette[0],0.76),
                "stone_dark" => (palette[3],1.),
                "forest_gate_door_frame" => (palette[4],1.),
                "forest_gate_door" => (palette[5],1.),
                "forest_gate_door_panel" => (palette[5],0.72),
                "glass" => (palette[2],0.9),
                "glass_light" => (palette[2],1.),
                "glass_dark" => (palette[2],0.65),
                _ => continue,
            };
            for index in range.clone() {
                for channel in 0..3 {
                    target.colors[first+index][channel] *= paint[channel]*shade/original[channel].max(0.0001);
                }
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }
    /// Source swatches replace only named roof paint. Keep the existing
    /// weather grain, fitted geometry and facet shading of each placement.
    pub(crate) fn recolor_fitted_roof(
        &self,
        target: &mut SurfaceMeshData,
        first: usize,
        palette: [[f32; 4]; 3],
    ) {
        self.recolor_roof_above(target,first,palette,f32::NEG_INFINITY);
    }

    pub(crate) fn recolor_fitted_modern_roof(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];3]) {
        // front_entry's closed door leaf ends at1.735 in authored model space.
        // It shares roof_teal_dark with upper architecture; the material name
        // alone must not turn that entrance into the source roof's dark outline.
        self.recolor_roof_above(target,first,palette,1.75);
    }

    fn recolor_roof_above(&self, target: &mut SurfaceMeshData, first: usize,
        palette: [[f32;4];3], minimum_height: f32) {
        let field: [f32; 4] = std::array::from_fn(|i| palette[1][i] * 0.7 + palette[2][i] * 0.3);
        let light: [f32; 4] = std::array::from_fn(|i| field[i] * 0.82 + palette[2][i] * 0.18);
        for (range, name, original) in &self.materials {
            if !name.starts_with("roof_") {
                continue;
            }
            let color = if name.ends_with("_dark") {
                palette[0]
            } else if name.ends_with("_light") {
                light
            } else {
                field
            };
            for index in range.clone() {
                if self.surface.positions[index][1]<minimum_height {continue;}
                for channel in 0..3 {
                    target.colors[first + index][channel] = color[channel]
                        * target.colors[first + index][channel]
                        / original[channel].max(0.0001);
                }
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }
    /// Fit only authored geometry. The native source plot edges and door seam
    /// stay exact. East entrances stretch along Z rather than inventing a south door.
    pub(crate) fn append_fitted(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4],
        base_height: f32,
        height_scale: f32,
        door: Option<f32>,
    ) {
        let [w, e, n, s] = bounds;
        let scales = [
            (e - w) / (self.max[0] - self.min[0]),
            height_scale,
            (s - n) / (self.max[2] - self.min[2]),
        ];
        let vertex_base = target.positions.len() as u32;
        let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
        for ((&p, &normal), &color) in self
            .surface
            .positions
            .iter()
            .zip(&self.surface.normals)
            .zip(&self.surface.colors)
        {
            let mut pos = [
                w + (p[0] - self.min[0]) * scales[0],
                base_height + (p[1] - self.min[1]) * height_scale,
                n + (p[2] - self.min[2]) * scales[2],
            ];
            let mut scale = scales;
            if let (Some(a), Some(d)) = (self.anchor, door) {
                let axis = if self.east { 2 } else { 0 };
                let limits = if self.east { [n, s] } else { [w, e] };
                let (v, k) = fit_door_axis(
                    p[axis],
                    [self.min[axis], self.max[axis]],
                    limits,
                    a[axis],
                    d,
                );
                pos[axis] = v;
                scale[axis] = k;
            }
            let norm = (Vec3::from_array(normal) / Vec3::from_array(scale)).normalize();
            let shade = 0.62 + 0.38 * norm.dot(light).max(0.0);
            let mut c = color;
            for v in &mut c[..3] {
                *v *= shade;
            }
            target.positions.push(pos);
            target.normals.push(norm.to_array());
            target.colors.push(c);
            target.uvs.push([0.0, 0.0]);
        }
        target
            .indices
            .extend(self.surface.indices.iter().map(|&i| i + vertex_base));
    }
}
pub(crate) fn fit_door_axis(
    x: f32,
    source: [f32; 2],
    target: [f32; 2],
    anchor: f32,
    door: f32,
) -> (f32, f32) {
    let [a, b] = source;
    let [c, d] = target;
    let scale = (d - c) / (b - a);
    // Keep a continuous doorway band, bounded so extreme off-center doors do
    // not reverse the smaller wing or move it outside the original drawing.
    let half = ((b - a) * 0.12)
        .min((door - c).min(d - door) / scale * 0.80)
        .max(0.0001);
    let l = anchor - half;
    let r = anchor + half;
    let tl = door - half * scale;
    let tr = door + half * scale;
    if x < l {
        let k = (tl - c) / (l - a);
        (c + (x - a) * k, k)
    } else if x > r {
        let k = (d - tr) / (b - r);
        (tr + (x - r) * k, k)
    } else {
        (door + (x - anchor) * scale, scale)
    }
}
pub(crate) fn model(kind: Kind) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        JSON.iter()
            .map(|json| Model::parse(json).expect("checked exterior mesh"))
            .collect()
    })[kind as usize]
}
#[path = "exterior_variants.rs"]
mod variants;
pub(crate) use variants::variant_model;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modern_town_roof_paint_preserves_shared_material_entrances() {
        let mut preserved_doors=0;
        for kind in [Kind::ModernHouse,Kind::ModernBlank,Kind::ModernShop,
            Kind::ModernCenter,Kind::ModernMart,Kind::ModernGym,Kind::ModernStation,
            Kind::ModernDaycare,Kind::ModernArcade] {
            let m=model(kind);let mut mesh=SurfaceMeshData::default();
            m.append_fitted(&mut mesh,[0.,96.,0.,64.],0.,16.,None);
            let before=mesh.clone();
            m.recolor_fitted_modern_roof(&mut mesh,0,[[0.03,0.03,0.03,1.],
                [0.04,0.2,0.9,1.],[0.1,0.7,1.,1.]]);
            assert_eq!(mesh.positions,before.positions);
            assert_eq!(mesh.normals,before.normals);
            assert_eq!(mesh.indices,before.indices);
            let mut roof=0;
            for i in 0..mesh.positions.len() {
                let material=m.vertex_material(i);
                if material.starts_with("roof_") && m.surface.positions[i][1]>1.75 {
                    roof+=1;
                    assert_eq!(mesh.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                } else {
                    assert_eq!(mesh.colors[i],before.colors[i],"retain {kind:?} {material}");
                    assert_eq!(mesh.uvs[i],before.uvs[i]);
                    preserved_doors+=usize::from(material=="roof_teal_dark");
                }
            }
            assert!(roof>0);
        }
        assert!(preserved_doors>100,"exercise the real authored door leaves");
    }

    #[test]
    fn fitted_house_roof_palette_preserves_geometry_and_non_roof_materials() {
        for kind in [
            Kind::KantoHouse,
            Kind::KantoFlatHouse,
            Kind::KantoMart,
            Kind::KantoBlank,
            Kind::ForestGate,
            Kind::ForestGateClosed,
        ] {
            let m = model(kind);
            let mut out = SurfaceMeshData::default();
            m.append_fitted(&mut out, [0., 64., 0., 64.], 0., 10., None);
            let before = out.clone();
            m.recolor_fitted_roof(
                &mut out,
                0,
                [
                    [0.03, 0.03, 0.03, 1.],
                    [0.04, 0.2, 0.9, 1.],
                    [0.1, 0.7, 1., 1.],
                ],
            );
            assert_eq!(out.positions, before.positions);
            assert_eq!(out.normals, before.normals);
            assert_eq!(out.indices, before.indices);
            let mut changed = 0;
            for (i, (&old, &new)) in before.colors.iter().zip(&out.colors).enumerate() {
                let material = m.vertex_material(i);
                if material.starts_with("roof_") {
                    assert_eq!(out.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                    changed += usize::from(old != new);
                    if !material.ends_with("_dark") {
                        assert!(new[2] > new[0] * 3.);
                    }
                } else {
                    assert_eq!(new, old, "walls, doorway and trim keep authored paint");
                    assert_eq!(out.uvs[i],before.uvs[i]);
                }
            }
            assert!(changed > 0);
        }
    }
    #[test]
    fn forest_gate_left_door_fits_the_warp_without_squeezing_its_windows() {
        let m=model(Kind::ForestGate);
        let anchor=m.anchor.expect("source left entrance");
        assert!((anchor[0]+1.5).abs()<0.01);
        let mut out=SurfaceMeshData::default();
        m.append_fitted(&mut out,[0.,96.,0.,96.],0.,1.,Some(24.));
        for (original,fitted) in m.surface.positions.iter().zip(&out.positions) {
            let linear=(original[0]-m.min[0])*96./(m.max[0]-m.min[0]);
            assert!((fitted[0]-linear).abs()<0.12,"source facade spacing must remain regular");
        }
        assert!((fit_door_axis(anchor[0],[m.min[0],m.max[0]],[0.,96.],anchor[0],24.).0-24.).abs()<0.0001);
    }
    #[test]
    fn forest_gate_front_has_three_high_four_pane_windows_and_no_closed_door() {
        for kind in [Kind::ForestGate,Kind::ForestGateClosed] {
            let m=model(kind);
            let mut x:Vec<_>=m.surface.positions.iter().zip(&m.surface.normals).enumerate()
                .filter(|(i,(p,n))|m.vertex_material(*i)=="glass" && p[1]>2.1 && p[2]>2.5 && n[2]>0.99)
                .map(|(_, (p,_))|p[0]).collect();
            x.sort_by(f32::total_cmp);x.dedup_by(|a,b|(*a-*b).abs()<0.00001);
            let mut groups=vec![Vec::new()];
            for v in x {
                if groups.last().unwrap().last().is_some_and(|last:&f32|v-last>0.3) {groups.push(Vec::new());}
                groups.last_mut().unwrap().push(v);
            }
            assert_eq!(groups.len(),3,"source has three high windows: {kind:?}");
            assert!(groups.iter().all(|window|window.len()==8),"four separate panes per window");
            if kind==Kind::ForestGateClosed {
                assert!(m.anchor.is_none());
                assert!(m.materials.iter().all(|(_,name,_)|!name.starts_with("forest_gate_door") && name!="yellow"),
                    "closed masonry front has neither an invented door nor gold crests");
            }
        }
    }
    #[test]
    fn exterior_models_have_closed_volume_valid_normals_and_budget() {
        assert_eq!(ALL.len(), JSON.len());
        for &kind in ALL {
            let m = model(kind);
            // Mansion courses gain the existing curved ceramic tessellation
            // after import; keep its authored mesh and runtime budgets distinct.
            let budget = match kind {
                Kind::TinTower => 13000,
                Kind::KantoMansion => 17000,
                _ => 12000,
            };
            let export: Export = crate::model_storage::parse(JSON[kind as usize]).unwrap();
            let authored: usize = export.primitives.iter().map(|p| p.indices.len() / 3).sum();
            assert!(authored < if kind==Kind::TinTower {13000}else{12000}, "{kind:?}: {authored} authored triangles");
            let runtime = m.surface.indices.len() / 3;
            assert!(runtime < budget, "{kind:?}: {runtime} runtime triangles");
            assert!(m.surface.indices.len() / 3 > 24, "{kind:?}");
            assert_eq!(m.surface.positions.len(), m.surface.normals.len());
            assert_eq!(m.surface.positions.len(), m.surface.colors.len());
            for n in &m.surface.normals {
                assert!((Vec3::from_array(*n).length() - 1.0).abs() < 0.001);
            }
            for axis in 0..3 {
                assert!(m.max[axis] > m.min[axis]);
            }
        }
    }
    #[test]
    fn east_gate_closed_courses_cover_the_deck_after_side_door_fitting() {
        let m=model(Kind::EastGate);
        for depth in [32.,64.,96.] {
            let door=depth*0.75;
            let mut fitted=SurfaceMeshData::default();
            m.append_fitted(&mut fitted,[0.,64.,0.,depth],0.,16.,Some(door));
            // Stay inside the ceramic fields, away from intentional narrow
            // vertical joints between columns.
            for x in [8.37,16.37,24.37,32.37,40.37,48.37,56.37] {for z in (4..61).step_by(2) {
                let z=z as f32*depth/64.;let mut top=None;
                for tri in fitted.indices.chunks_exact(3) {
                    let p=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(fitted.positions[i as usize]));
                    let a=p[0];let b=p[1];let c=p[2];
                    let den=(b.z-c.z)*(a.x-c.x)+(c.x-b.x)*(a.z-c.z);
                    if den.abs()<0.00001 {continue;}
                    let u=((b.z-c.z)*(x-c.x)+(c.x-b.x)*(z-c.z))/den;
                    let v=((c.z-a.z)*(x-c.x)+(a.x-c.x)*(z-c.z))/den;
                    if u.min(v).min(1.-u-v)< -0.00001 {continue;}
                    let height=u*a.y+v*b.y+(1.-u-v)*c.y;
                    if top.is_none_or(|(y,_)|height>y) {top=Some((height,tri[0] as usize));}
                }
                let (height,index)=top.expect("the gate roof must be closed");
                let (_,name,_)=m.materials.iter().find(|(range,_,_)|range.contains(&index)).unwrap();
                assert!(height>16.*2.3 && !name.ends_with("_dark"),
                    "exposed roof backing at({x},{z}), door{door}: {name}, height{height}");
            }}
        }
    }

    #[test]
    fn east_entry_is_a_side_seam_with_real_open_passage() {
        let m = model(Kind::EastGate);
        assert!(m.east);
        assert_eq!(m.anchor.unwrap()[0], m.max[0]);
        let mut out = SurfaceMeshData::default();
        m.append_fitted(&mut out, [0.0, 64.0, 0.0, 64.0], 0.0, 16.0, Some(48.0));
        assert!(
            out.positions
                .iter()
                .all(|v| v[0] >= -0.001 && v[0] <= 64.001 && v[2] >= -0.001 && v[2] <= 64.001)
        );
        assert!(
            out.positions
                .iter()
                .any(|v| (v[0] - 64.0).abs() < 0.001 && v[1] < 4.0)
        );
    }
    #[test]
    fn all_door_bands_preserve_center_without_reversing_side_wings() {
        for target in [8.0, 24.0, 48.0, 56.0] {
            let f = |x| fit_door_axis(x, [-2.0, 2.0], [0.0, 64.0], 0.0, target);
            assert_eq!(f(-2.0).0, 0.0);
            assert!((f(2.0).0 - 64.0).abs() < 0.001);
            assert_eq!(f(0.0).0, target);
            for i in 0..100 {
                assert!(f(-2.0 + i as f32 * 0.04).1 > 0.0);
            }
        }
    }
    #[test]
    fn malformed_exterior_mesh_is_rejected() {
        assert!(Model::parse(r#"{"primitives":[]}"#).is_err());
    }

    #[test]
    fn rounded_boundary_stones_keep_contacts_and_share_normals_across_material_seams() {
        for kind in [
            Kind::KantoBoundaryLand,
            Kind::KantoBoundaryShore,
            Kind::KantoBoundaryPath,
        ] {
            let export: Export = crate::model_storage::parse(JSON[kind as usize]).unwrap();
            let original = export
                .primitives
                .iter()
                .flat_map(|p| p.positions.chunks_exact(3).map(|p| [p[0], p[1], p[2]]))
                .collect::<Vec<_>>();
            let m = model(kind);
            assert_eq!(m.surface.positions.len(), original.len());
            assert!(
                original
                    .iter()
                    .zip(&m.surface.positions)
                    .any(|(a, b)| a != b),
                "rounding must change the coarse silhouette, not just its paint"
            );
            let mut normals = std::collections::BTreeMap::new();
            let mut pigments = std::collections::BTreeMap::new();
            for (i, (&p, &normal)) in m
                .surface
                .positions
                .iter()
                .zip(&m.surface.normals)
                .enumerate()
            {
                if original[i][1] == 0. {
                    assert_eq!(p, original[i], "the seated perimeter must remain fixed");
                }
                assert!((Vec3::from_array(normal).length() - 1.).abs() < 0.0001);
                if let Some(previous) = pigments.insert(p.map(f32::to_bits), m.surface.colors[i]) {
                    assert_eq!(
                        previous, m.surface.colors[i],
                        "the cap cannot retain triangular pigment seams"
                    );
                }
                if let Some(previous) = normals.insert(p.map(f32::to_bits), normal) {
                    assert_eq!(
                        previous, normal,
                        "material boundaries cannot create a hard normal seam"
                    );
                }
            }
        }
    }

    #[test]
    fn kanto_boundary_rocks_are_closed_volumes_below_five_hundred_triangles() {
        for kind in [
            Kind::KantoBoundaryLand,
            Kind::KantoBoundaryShore,
            Kind::KantoBoundaryPath,
        ] {
            let m = model(kind);
            assert!(m.surface.indices.len() / 3 < 500, "{kind:?}");
            assert!(m.max[0] - m.min[0] > 1.9);
            assert!(m.max[2] - m.min[2] > 1.6);
            assert!(m.max[1] - m.min[1] > 1.1);
            assert_eq!(m.min[1], 0.0, "closed underside stays on ground");
            let vertex =
                |i: u32| m.surface.positions[i as usize].map(|v| (v * 1_000_000.0).round() as i32);
            let mut edges = std::collections::BTreeMap::new();
            let mut signed_volume = 0.0;
            for triangle in m.surface.indices.chunks_exact(3) {
                let a = Vec3::from_array(m.surface.positions[triangle[0] as usize]);
                let b = Vec3::from_array(m.surface.positions[triangle[1] as usize]);
                let c = Vec3::from_array(m.surface.positions[triangle[2] as usize]);
                assert!((b - a).cross(c - a).length_squared() > 0.0000001);
                signed_volume += a.dot(b.cross(c)) / 6.0;
                for [from, to] in [
                    [triangle[0], triangle[1]],
                    [triangle[1], triangle[2]],
                    [triangle[2], triangle[0]],
                ] {
                    let a = vertex(from);
                    let b = vertex(to);
                    let (key, direction) = if a < b {
                        ((a, b), 1_i32)
                    } else {
                        ((b, a), -1_i32)
                    };
                    let entry = edges.entry(key).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += direction;
                }
            }
            assert!(
                signed_volume > 1.0,
                "{kind:?} has real outward-facing volume"
            );
            assert!(
                edges
                    .values()
                    .all(|&(count, balance)| count == 2 && balance == 0),
                "{kind:?} has no open rim or non-manifold backing"
            );
            assert!(m.surface.normals.iter().any(|v| v[1] < -0.99));
            assert!(m.surface.normals.iter().any(|v| v[1] > 0.8));
            for (axis, sign) in [(0, -1.0), (0, 1.0), (2, -1.0), (2, 1.0)] {
                assert!(m.surface.normals.iter().any(|v| v[axis] * sign > 0.7));
            }
        }
        assert_ne!(
            model(Kind::KantoBoundaryLand).surface.colors,
            model(Kind::KantoBoundaryShore).surface.colors
        );
        assert_ne!(
            model(Kind::KantoBoundaryLand).surface.colors,
            model(Kind::KantoBoundaryPath).surface.colors
        );
    }
}
