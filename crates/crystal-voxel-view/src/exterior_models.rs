//! Original editable world-exterior meshes. No source textures or game data.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::{Vec2, Vec3};
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
    rigid_roof: bool,
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
    door_aperture: Option<[f32;2]>,
}
#[derive(Clone)]
pub(crate) struct Model {
    aperture: Option<[f32;2]>,
    rigid_roof: Vec<std::ops::Range<usize>>,
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
fn round_boundary_stone(surface: &mut SurfaceMeshData, min: [f32; 3], max: [f32; 3], shore: bool) {
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
    // Keep the planted foot, but lift the upper shoulder into a flatter
    // hand-cut crown. The old ellipsoidal top read as a loose river boulder.
    let height=max[1]-min[1];
    for (i,p) in points.iter_mut().enumerate() {
        if seated[i] {continue;}
        let h=((p.y-min[1])/height).clamp(0.,1.);
        // Offshore stones retain the authored sloping shoulders. Flattening
        // their upper half makes the repeated sea boundary look like posts.
        if !shore && h>0.5 {p.y=min[1]+(1.-2.*(1.-h).powi(2))*height;}
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
        let mut rigid_roof = Vec::new();
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
            if p.rigid_roof {rigid_roof.push(base as usize..surface.positions.len());}
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
            round_boundary_stone(&mut surface, min, max, export.name == "kanto_boundary_shore");
        }
        if (0..3).any(|i| !min[i].is_finite() || max[i] <= min[i]) {
            return Err("empty exterior mesh".into());
        }
        let east = export.door_face.as_deref() == Some("east");
        if export.door_aperture.is_some_and(|a| !east || !a.iter().all(|v|v.is_finite())
            || a[0]<=min[2] || a[1]>=max[2] || a[0]>=a[1]) {
            return Err("invalid exterior passage aperture".into());
        }
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
            aperture: export.door_aperture,
            rigid_roof,
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
    /// A capped post keeps one shared sculpt but follows its own live drawing.
    pub(crate) fn recolor_fitted_capped_post(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];3]) {
        let mineral = |color:[f32;4], amount:f32, warmth:[f32;3], gain:f32| {
            let luminance=color[0]*0.2126+color[1]*0.7152+color[2]*0.0722;
            std::array::from_fn::<_,4,_>(|c| if c==3 {1.} else {
                (color[c]*(1.-amount)+luminance*warmth[c]*amount)*gain
            })
        };
        let timber=mineral(palette[1],0.5,[1.,0.9,0.75],0.8);
        let cap=mineral(palette[2],0.85,[1.,0.98,0.94],1.);
        for (range,name,original) in &self.materials {
            let (paint,gain)=match name.as_str() {
                "Post | closed underside" => (palette[0],0.85),
                "Post | bevelled dark foot" => (timber,0.72),
                "Post | weathered timber shaft" => (timber,0.94),
                "Post | shaft facet" => (timber,1.02),
                "Post | collar shadow" => (timber,0.68),
                "Post | cap rim" => (cap,0.78),
                "Post | bevelled painted cap" => (cap,0.94),
                "Post | cap crown" => (cap,1.0),
                _ => continue,
            };
            for i in range.clone() {
                let color=target.colors[first+i];
                let shade=(color[0]+color[1]+color[2]) /
                    (original[0]+original[1]+original[2]).max(0.0001);
                for c in 0..3 {target.colors[first+i][c]=paint[c]*gain*shade;}
                // Use immutable sculpt coordinates: moving the source grid
                // cannot slide the grain, and day/night paint keeps one finish.
                let p=self.surface.positions[i];let n=self.surface.normals[i];
                let scale=1.55/(self.max[1]-self.min[1]);
                let across=if n[0].abs()>n[2].abs() {p[2]}else{p[0]};
                target.uvs[first+i]=match name.as_str() {
                    "Post | bevelled dark foot" | "Post | weathered timber shaft"
                        | "Post | shaft facet" | "Post | collar shadow" => [
                            (across-self.min[0])*scale,
                            crate::scenery_finish::TOWN_TIMBER_BASE+(p[1]-self.min[1])*scale],
                    "Post | cap rim" | "Post | bevelled painted cap" | "Post | cap crown" => [
                        (across-self.min[0])*scale,
                        crate::scenery_finish::TOWN_PAINT_BASE+96.+
                            (if n[1].abs()>0.5 {p[2]-self.min[2]}else{p[1]-self.min[1]})*scale],
                    _=>crate::scenery_finish::SOURCE_PAINT_UV,
                };
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
                let p=self.surface.positions[index];
                let text=name=="timber_light" && p[2]>self.max[2]*0.5;
                // Immutable sculpt coordinates keep grain fixed through scrolling
                // and palette changes. Raised notice strokes stay clean ink.
                target.uvs[first+index]=if name=="timber" {
                    [p[1]-self.min[1],crate::scenery_finish::TOWN_TIMBER_BASE+p[0]-self.min[0]]
                } else if name=="timber_light" && !text {
                    [p[0]-self.min[0],crate::scenery_finish::TOWN_TIMBER_BASE+p[1]-self.min[1]]
                } else if name=="cream" {
                    [p[0]-self.min[0],crate::scenery_finish::TOWN_PAINT_BASE+p[1]-self.min[1]]
                } else {crate::scenery_finish::SOURCE_PAINT_UV};
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
    /// The shared retail facade follows its live masonry, panes and roof.
    /// Stone footings, door hardware and fitted openings keep their identity.
    pub(crate) fn recolor_fitted_department(&self,target:&mut SurfaceMeshData,
        first:usize,p:[[f32;4];8]) {
        for (range,name,original) in &self.materials {
            let (paint,gain)=match name.as_str() {
                "plaster_light" | "plaster_service_wall" => (p[0],1.),
                "plaster_service_mortar" => (p[1],0.92),
                "plaster" => (p[1],0.90),
                "glass" => (p[2],0.9),
                "glass_light" => (p[3],1.),
                "glass_dark" => (p[2],0.65),
                "roof_red_dark" => (p[0],0.42),
                "roof_red" | "roof_red_warm" => (p[0],0.76),
                "roof_red_light" | "cream" => (p[1],1.),
                "roof_blue_dark" => (p[5],1.),
                "roof_blue" => (p[6],0.7),
                "roof_blue_light" => (p[7],0.85),
                _=>continue,
            };
            for index in range.clone() {
                for channel in 0..3 {
                    target.colors[first+index][channel]*=paint[channel]*gain/original[channel].max(0.0001);
                }
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }
    /// Shared modern Gym masonry and lettering follow the authenticated live
    /// drawing. Other civic assets keep their existing material producers.
    pub(crate) fn recolor_fitted_modern_gym(&self,target:&mut SurfaceMeshData,
        first:usize,p:[[f32;4];8]) {
        let mix=|a:[f32;4],b:[f32;4],t:f32|std::array::from_fn(|i|a[i]*(1.-t)+b[i]*t);
        for (range,name,original) in &self.materials {
            // Native ochre is sparse joint/weathering ink against a pale wall.
            // Keep that area balance instead of filling every brick with ink.
            let (paint,gain)=match name.as_str() {
                "plaster_service_wall" | "plaster_light" | "cream" => (p[1],0.96),
                "plaster_service_mortar" => (mix(p[1],p[0],0.25),0.82),
                "plaster" => (mix(p[1],p[0],0.40),0.85),
                "glass" => (p[2],0.9),"glass_light" => (p[3],1.),
                "glass_dark" => (p[2],0.65),"ink" => (p[4],1.),
                // The shared modern-roof producer has already painted its
                // field and border. Preserve it, including its door exclusion.
                _=>continue,
            };
            for index in range.clone() {
                for channel in 0..3 {
                    target.colors[first+index][channel]*=paint[channel]*gain/original[channel].max(0.0001);
                }
                let uv=if matches!(name.as_str(),"plaster_service_wall"|"plaster_service_mortar"|"plaster") {
                    let at=self.surface.positions[index];let normal=self.surface.normals[index];
                    let across=if normal[0].abs()>normal[2].abs() {at[2]-self.min[2]}else{at[0]-self.min[0]};
                    [across,crate::scenery_finish::TOWN_PAINT_BASE+96.+at[1]]
                } else {crate::scenery_finish::SOURCE_PAINT_UV};
                target.uvs[first+index]=uv;
            }
        }
    }
    /// Native broadcast pigment binds the existing authored material groups.
    /// Antenna steel, warning lamp and brass retain their separate identities.
    pub(crate) fn recolor_fitted_radio_tower(&self,target:&mut SurfaceMeshData,
        first:usize,palette:[[f32;4];6]) {
        for (range,name,original) in &self.materials {
            let (paint,gain)=match name.as_str() {
                "plaster_light" | "plaster" => (palette[0],1.),
                "stone_light" | "timber_light" | "timber_gold" => (palette[1],0.95),
                "stone" | "stone_dark" | "timber" | "violet_dark" | "roof_teal_dark" => (palette[0],0.78),
                "roof_blue_dark" => (palette[0],0.82),
                "roof_blue" => (palette[1],0.85),
                "roof_blue_light" => (palette[2],0.75),
                "glass" => (palette[3],0.85),
                "glass_light" => (palette[4],1.),
                "glass_dark" => (palette[5],0.8),
                _=>continue,
            };
            for index in range.clone() {
                if name.starts_with("stone") && self.surface.positions[index][1]>7.22 {continue;}
                for channel in 0..3 {
                    target.colors[first+index][channel]*=paint[channel]*gain/original[channel].max(0.0001);
                }
                target.uvs[first+index]=if matches!(name.as_str(),"plaster_light"|"plaster"|"stone"|"stone_dark") {
                    let p=self.surface.positions[index];let n=self.surface.normals[index];
                    let across=if n[0].abs()>n[2].abs() {p[2]-self.min[2]}else{p[0]-self.min[0]};
                    [across,crate::scenery_finish::TOWN_PAINT_BASE+96.+
                        if n[1].abs()>0.5 {p[2]-self.min[2]}else{p[1]-self.min[1]}]
                }else{crate::scenery_finish::SOURCE_PAINT_UV};
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

    /// The shared tall pagoda retains native ceramic, infill and cedar paint.
    /// Keep model-local grain and fitted geometry independent of time/scroll.
    pub(crate) fn recolor_fitted_pagoda(&self,target:&mut SurfaceMeshData,first:usize,
        roof:Option<[[f32;4];3]>,facade:Option<[[f32;3];3]>) {
        for (range,name,original) in &self.materials {
            let timber=name.starts_with("timber");
            let ceramic=name.starts_with("violet");
            let color=if ceramic {
                roof.map(|p|std::array::from_fn(|c|match name.as_str() {
                    "violet_dark"=>p[1][c]*0.5+p[0][c]*0.5,
                    "violet_light"=>p[1][c]*0.82+p[2][c]*0.18,
                    "violet_warm"=>p[1][c]*0.94,
                    _=>p[1][c],
                }))
            } else {facade.and_then(|p|match name.as_str() {
                "plaster_light"=>Some(p[0]),
                "timber_light"=>Some(p[1]),
                "timber_gold"=>Some(std::array::from_fn(|c|p[1][c]*0.9+p[0][c]*0.1)),
                "timber"=>Some(p[2]),
                _=>None,
            })};
            let Some(color)=color else {continue;};
            for i in range.clone() {
                for c in 0..3 {
                    target.colors[first+i][c]=color[c]*target.colors[first+i][c]/original[c].max(0.0001);
                }
                let p=self.surface.positions[i];let n=self.surface.normals[i];
                target.uvs[first+i]=if timber {
                    let across=if n[0].abs()>n[2].abs() {p[2]-self.min[2]}else{p[0]-self.min[0]};
                    [across,crate::scenery_finish::TOWN_TIMBER_BASE+p[1]-self.min[1]]
                } else if ceramic {
                    [p[0]-self.min[0],crate::scenery_finish::TOWN_PAINT_BASE+p[2]-self.min[2]]
                } else {crate::scenery_finish::SOURCE_PAINT_UV};
            }
        }
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
    /// Turn the same east/west kit toward a complete native north entrance.
    /// Fit both clear jamb edges, so neither of two warp lanes meets a wall.
    pub(crate) fn append_fitted_north_passage(&self,target:&mut SurfaceMeshData,
        bounds:[f32;4],base:f32,height:f32,opening:[f32;2])->bool {
        let Some(aperture)=self.aperture.filter(|_|self.east) else {return false;};
        let [w,e,n,s]=bounds;
        if !(w<opening[0] && opening[0]<opening[1] && opening[1]<e && n<s && height>0.) {
            return false;
        }
        let source=[self.min[2],aperture[0],aperture[1],self.max[2]];
        let destination=[w,opening[0],opening[1],e];
        let depth=(s-n)/(self.max[0]-self.min[0]);
        let first=target.positions.len() as u32;
        let light=Vec3::new(-0.35,0.85,0.4).normalize();
        for (index,((p,normal),color)) in self.surface.positions.iter().zip(&self.surface.normals).zip(&self.surface.colors).enumerate() {
            let section=if p[2]<aperture[0] {0}else if p[2]<aperture[1] {1}else{2};
            let (x,width)=if self.rigid_roof.iter().any(|r|r.contains(&index)) {
                let width=(e-w)/(self.max[2]-self.min[2]);
                (w+(p[2]-self.min[2])*width,width)
            } else {
                let width=(destination[section+1]-destination[section])/(source[section+1]-source[section]);
                (destination[section]+(p[2]-source[section])*width,width)
            };
            let normal=Vec3::new(normal[2]/width,normal[1]/height,-normal[0]/depth).normalize();
            let shade=0.62+0.38*normal.dot(light).max(0.);
            target.positions.push([x,
                base+(p[1]-self.min[1])*height,n+(self.max[0]-p[0])*depth]);
            target.normals.push(normal.to_array());
            target.colors.push([color[0]*shade,color[1]*shade,color[2]*shade,color[3]]);
            target.uvs.push([0.,0.]);
        }
        target.indices.extend(self.surface.indices.iter().map(|i|i+first));
        true
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

pub(crate) const KANTO_BOUNDARY_INSET: [f32; 2] = [0.06, 0.09];

/// The same fitted rock's closed waterline base, in its two-cell source plot.
/// Optical shore distance must not promote its rectangular atlas into land.
pub(crate) fn kanto_shore_contact(at: [i32; 2]) -> Vec<[Vec2; 3]> {
    let model = variant_model(Kind::KantoBoundaryShore, at);
    let inset = Vec2::from_array(KANTO_BOUNDARY_INSET);
    let min = Vec2::new(model.min[0], model.min[2]);
    let extent = Vec2::new(model.max[0], model.max[2]) - min;
    model.surface.indices.chunks_exact(3).filter_map(|tri| {
        let points = [tri[0],tri[1],tri[2]].map(|i| model.surface.positions[i as usize]);
        if points.iter().any(|p|p[1] > model.min[1]+0.00001) {return None;}
        let projected = points.map(|p| inset + (Vec2::new(p[0],p[2])-min)/extent
            * (Vec2::splat(2.)-inset*2.));
        ((projected[1]-projected[0]).perp_dot(projected[2]-projected[0]).abs()>0.000001)
            .then_some(projected)
    }).collect()
}

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
    fn notice_board_finish_uses_fixed_grain_and_keeps_raised_ink_clean() {
        let m=model(Kind::RouteSign);
        let render=|bounds,paint| {
            let mut mesh=SurfaceMeshData::default();
            m.append_fitted(&mut mesh,bounds,0.,8.,None);
            m.recolor_fitted_sign(&mut mesh,0,paint);mesh
        };
        let day=[[0.02,0.02,0.02,1.],[0.3,0.35,0.3,1.],[0.8,0.9,0.8,1.]];
        let first=render([0.,16.,0.,8.],day);
        let scrolled=render([32.,48.,16.,24.],day);
        let night=render([0.,16.,0.,8.],day.map(|p|[p[0]*0.3,p[1]*0.3,p[2]*0.5,1.]));
        assert_eq!(first.uvs,scrolled.uvs);
        assert_eq!(first.uvs,night.uvs);
        assert_eq!(first.positions,night.positions);
        assert_eq!(first.normals,night.normals);
        assert_eq!(first.indices,night.indices);
        assert_ne!(first.colors,night.colors);
        let mut wood=0;let mut enamel=0;let mut ink=0;
        for (i,&uv) in first.uvs.iter().enumerate() {
            let text=m.vertex_material(i)=="timber_light" && m.surface.positions[i][2]>m.max[2]*0.5;
            if text {assert_eq!(uv,crate::scenery_finish::SOURCE_PAINT_UV);ink+=1;}
            wood+=usize::from(crate::scenery_finish::town_timber_coordinates(uv).is_some());
            enamel+=usize::from(crate::scenery_finish::town_paint_kind(uv)==Some(0));
        }
        assert!(wood>100 && enamel>20 && ink>50);
    }

    #[test]
    fn department_running_bond_has_physical_relief_and_clear_window_pockets() {
        let m=model(Kind::ModernDepartment);
        let mut brick_front=f32::NEG_INFINITY;let mut mortar_front=f32::NEG_INFINITY;
        for (range,name,_) in &m.materials {
            for i in range.clone() {
                if m.surface.normals[i][2]<0.99 {continue;}
                if name=="plaster_service_wall" {brick_front=brick_front.max(m.surface.positions[i][2]);}
                if name=="plaster_service_mortar" {mortar_front=mortar_front.max(m.surface.positions[i][2]);}
            }
        }
        assert!((0.01..0.05).contains(&(brick_front-mortar_front)),"shallow closed brick faces above continuous mortar");
        let mut panes=Vec::new();let mut faces=Vec::new();
        for tri in m.surface.indices.chunks_exact(3) {
            let ids=[tri[0] as usize,tri[1] as usize,tri[2] as usize];
            if m.surface.normals[ids[0]][2]<0.99 {continue;}
            let p=ids.map(|i|Vec2::new(m.surface.positions[i][0],m.surface.positions[i][1]));
            if ids.iter().all(|&i|m.vertex_material(i)=="glass") {
                let center=(p[0]+p[1]+p[2])/3.;if center.y>2.5 {panes.push(center);}
            }
            if ids.iter().all(|&i|m.vertex_material(i)=="plaster_service_wall") {faces.push(p);}
        }
        assert!(panes.len()>8 && faces.len()>100,"exercise repeated windows and running-bond faces");
        for point in panes {for &[a,b,c] in &faces {
            let signs=[(b-a).perp_dot(point-a),(c-b).perp_dot(point-b),(a-c).perp_dot(point-c)];
            let inside=signs.iter().all(|s|*s>0.00001)||signs.iter().all(|s|*s< -0.00001);
            assert!(!inside,"brick cannot cover a recessed upper window");
        }}
        assert_eq!(m.min,[-2.975,0.,-1.95]);assert_eq!(m.max,[2.975,6.476,1.95]);
        assert!(m.surface.indices.len()/3<12000);
    }

    #[test]
    fn shared_department_live_paint_preserves_geometry_and_other_materials() {
        let m=model(Kind::ModernDepartment);let mut mesh=SurfaceMeshData::default();
        m.append_fitted(&mut mesh,[0.,96.,0.,128.],0.,16.,Some(40.));
        let before=mesh.clone();let palette=[[0.1,0.5,0.45,1.],[0.8,0.9,0.8,1.],
            [0.6,0.8,0.6,1.],[0.9,0.9,0.5,1.],[0.04,0.04,0.04,1.],
            [0.02,0.03,0.05,1.],[0.1,0.4,0.6,1.],[0.4,0.7,0.8,1.]];
        m.recolor_fitted_department(&mut mesh,0,palette);
        assert_eq!(mesh.positions,before.positions);assert_eq!(mesh.normals,before.normals);
        assert_eq!(mesh.indices,before.indices);
        let mut changed=0;let mut retained=0;let mut lettering=0;
        for i in 0..mesh.positions.len() {
            let name=m.vertex_material(i);
            if matches!(name,"plaster_light"|"plaster_service_wall"|"plaster_service_mortar"|"plaster"|"glass"|"glass_light"|"glass_dark"
                |"roof_red_dark"|"roof_red"|"roof_red_warm"|"roof_red_light"|"cream"
                |"roof_blue_dark"|"roof_blue"|"roof_blue_light") {
                assert_eq!(mesh.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                changed+=usize::from(mesh.colors[i]!=before.colors[i]);
            }else {assert_eq!(mesh.colors[i],before.colors[i]);assert_eq!(mesh.uvs[i],before.uvs[i]);retained+=1;}
            lettering+=usize::from(name=="cream");
        }
        assert!(changed>1000 && retained>100);
        assert!(lettering>120,"solid MART strokes replace the five blank plaque blocks");
    }

    #[test]
    fn pagoda_native_paint_preserves_fit_and_anchors_ceramic_and_cedar_grain() {
        let m=model(Kind::TinTower);
        let backing=m.surface.positions.iter().enumerate()
            .filter(|(i,_)|m.vertex_material(*i)=="violet_dark").map(|(_,p)|*p).collect::<Vec<_>>();
        let ridge_z=(backing.iter().map(|p|p[2]).fold(f32::INFINITY,f32::min)
            +backing.iter().map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max))*0.5;
        let levels=backing.iter().filter(|p|(p[2]-ridge_z).abs()<0.0001)
            .map(|p|(p[1]*10000.).round() as i32).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(levels.len(),8,"the native tall drawing has eight structural roof tiers");
        assert!(!m.material_ranges().any(|(_,name)|name.starts_with("glass")),
            "traditional shutters and cedar doors cannot retain office glazing");
        let roof=Some([[0.03,0.02,0.01,1.],[0.32,0.18,0.04,1.],[0.75,0.76,0.61,1.]]);
        let facade=Some([[0.75,0.76,0.61],[0.32,0.18,0.04],[0.03,0.02,0.01]]);
        let mut a=SurfaceMeshData::default();
        m.append_fitted(&mut a,[0.,96.,0.,128.],0.,16.,Some(48.));
        let before=a.clone();m.recolor_fitted_pagoda(&mut a,0,roof,facade);
        assert_eq!(a.positions,before.positions);assert_eq!(a.normals,before.normals);
        assert_eq!(a.indices,before.indices);assert_ne!(a.colors,before.colors);
        assert!(a.uvs.iter().any(|uv|crate::scenery_finish::town_paint_kind(*uv)==Some(0)));
        assert!(a.uvs.iter().any(|uv|crate::scenery_finish::town_timber_coordinates(*uv).is_some()));
        let mut b=SurfaceMeshData::default();
        m.append_fitted(&mut b,[80.,176.,96.,224.],0.,16.,Some(128.));
        m.recolor_fitted_pagoda(&mut b,0,roof,facade);
        assert_eq!(a.uvs,b.uvs,"moving the source grid cannot slide roof or cedar grain");
        assert!(a.colors.iter().zip(&b.colors).all(|(a,b)|a.iter().zip(b).all(|(a,b)|(a-b).abs()<0.000001)),
            "translated fitting may only introduce floating-point rounding in the face shade");
        b=SurfaceMeshData::default();
        m.append_fitted(&mut b,[80.,176.,96.,224.],0.,16.,Some(128.));
        m.recolor_fitted_pagoda(&mut b,0,Some([[0.01,0.01,0.03,1.];3]),
            Some([[0.04,0.05,0.12];3]));
        assert_eq!(a.uvs,b.uvs);assert_ne!(a.colors,b.colors,"native night exposure follows the same model");
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
    fn lighthouse_lens_is_visible_through_all_eight_lantern_bays() {
        let m=model(Kind::Lighthouse);
        assert_eq!(m.min,[-2.,0.,-2.]);assert_eq!(m.max,[2.,10.3,2.123529]);
        assert_eq!(m.anchor,Some([0.,0.,2.123529]));
        for bay in 0..8 {
            let angle=(bay as f32+0.5)*std::f32::consts::TAU/8.;
            let eye=Vec3::new(angle.cos()*3.*4./3.4,8.48,angle.sin()*3.*4./3.4);
            let direction=(Vec3::new(0.,8.48,0.)-eye).normalize();
            let mut nearest=None;
            for tri in m.surface.indices.chunks_exact(3) {
                let [a,b,c]=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_array(m.surface.positions[i as usize]));
                let edge=b-a;let other=c-a;let h=direction.cross(other);let det=edge.dot(h);
                if det.abs()<0.000001 {continue;}
                let offset=eye-a;let u=offset.dot(h)/det;
                let q=offset.cross(edge);let v=direction.dot(q)/det;
                let distance=other.dot(q)/det;
                if u<0. || v<0. || u+v>1. || distance<=0. {continue;}
                if nearest.is_none_or(|(old,_)|distance<old) {nearest=Some((distance,tri[0] as usize));}
            }
            let (_,vertex)=nearest.expect("a closed beacon occupies each clear window bay");
            assert_eq!(m.vertex_material(vertex),"yellow","bay {bay} must reveal the ribbed optic");
        }
    }

    #[test]
    fn lighthouse_fresnel_optic_is_one_closed_volume() {
        let m=model(Kind::Lighthouse);
        let mut edges=std::collections::BTreeMap::new();
        let mut graph=std::collections::BTreeMap::<[i64;3],Vec<[i64;3]>>::new();
        for tri in m.surface.indices.chunks_exact(3) {
            if m.vertex_material(tri[0]as usize)!="yellow" {continue;}
            let p=[tri[0],tri[1],tri[2]].map(|i|m.surface.positions[i as usize]);
            if p.iter().any(|p|p[1]<7.8 || p[1]>9.2) {continue;}
            let keys=p.map(|p|p.map(|v|(v*1_000_000.).round()as i64));
            for i in 0..3 {
                let (a,b)=(keys[i],keys[(i+1)%3]);
                *edges.entry(if a<b {(a,b)}else{(b,a)}).or_insert(0usize)+=1;
                graph.entry(a).or_default().push(b);graph.entry(b).or_default().push(a);
            }
        }
        assert!(edges.len()>100 && edges.values().all(|n|*n==2),"ribbed lens has no exposed seams or duplicate internal caps");
        let mut stack=vec![*graph.keys().next().unwrap()];let mut visited=std::collections::BTreeSet::new();
        while let Some(key)=stack.pop() {if visited.insert(key) {stack.extend(graph[&key].iter().copied());}}
        assert_eq!(visited.len(),graph.len(),"all optical courses form one connected volume");
    }

    #[test]
    fn lighthouse_balcony_has_air_above_its_stone_walkway() {
        let m = model(Kind::Lighthouse);
        // Cast down through the annulus, outside the lantern roof and inside
        // the guardrail. A filled handrail disk incorrectly intercepts these
        // rays at 8.235 instead of exposing the stone balcony at 7.73.
        for step in 0..16 {
            let angle = (step as f32 + 0.5) * std::f32::consts::TAU / 16.;
            let point = Vec2::new(angle.cos() * 1.26 * 4. / 3.4,
                angle.sin() * 1.26 * 4. / 3.4);
            let mut top = f32::NEG_INFINITY;
            for tri in m.surface.indices.chunks_exact(3) {
                let p = [tri[0],tri[1],tri[2]].map(|i| m.surface.positions[i as usize]);
                let [a,b,c] = p.map(|v| Vec2::new(v[0],v[2]));
                let area = (b-a).perp_dot(c-a);
                if area.abs() < 0.000001 { continue; }
                let v = (point-a).perp_dot(c-a) / area;
                let w = (b-a).perp_dot(point-a) / area;
                let u = 1.-v-w;
                if u.min(v).min(w) >= -0.00001 {
                    top = top.max(u*p[0][1]+v*p[1][1]+w*p[2][1]);
                }
            }
            assert!((top-7.73).abs()<0.01, "balcony ray {step}: {top}");
        }
    }

    #[test]
    fn modern_gym_has_two_paired_front_bays_and_three_solid_name_glyphs() {
        let m=model(Kind::ModernGym);
        assert_eq!(m.min,[-2.975,0.,-1.95]);assert_eq!(m.max,[2.975,3.376,1.95]);
        assert_eq!(m.anchor,Some([0.,0.,1.95]));
        let groups=|material:&str,lo:f32,hi:f32| {
            let mut spans=Vec::<[f32;2]>::new();
            for tri in m.surface.indices.chunks_exact(3) {
                let pts=[tri[0],tri[1],tri[2]].map(|i|m.surface.positions[i as usize]);
                if m.vertex_material(tri[0]as usize)!=material || m.surface.normals[tri[0]as usize][2]<0.99
                    || pts.iter().any(|p|p[2]<1.4 || p[1]<lo || p[1]>hi) {continue;}
                spans.push([pts.iter().map(|p|p[0]).fold(f32::INFINITY,f32::min),pts.iter().map(|p|p[0]).fold(f32::NEG_INFINITY,f32::max)]);
            }
            spans.sort_by(|a,b|a[0].total_cmp(&b[0]));let mut groups=Vec::<[f32;2]>::new();
            for span in spans {if let Some(last)=groups.last_mut() && span[0]<=last[1]+0.00001 {last[1]=last[1].max(span[1]);}else{groups.push(span);}}
            groups
        };
        let mut edges=std::collections::BTreeMap::new();
        let key=|i:u32|m.surface.positions[i as usize].map(|v|(v*1_000_000.).round()as i32);
        for tri in m.surface.indices.chunks_exact(3) {
            if m.vertex_material(tri[0]as usize)!="roof_blue_light" {continue;}
            for side in 0..3 {
                let (a,b)=(key(tri[side]),key(tri[(side+1)%3]));
                *edges.entry(if a<b {(a,b)}else{(b,a)}).or_insert(0)+=1;
            }
        }
        assert!(!edges.is_empty() && edges.values().all(|n|*n==2),"mitered coping is closed without intersecting end caps");
        let bays=groups("glass",1.23,2.01);
        assert_eq!(bays.len(),2);assert!(bays[0][1]< -1. && bays[1][0]>1.);
        let lettering=groups("ink",2.30,2.65);assert_eq!(lettering.len(),3);
        assert!(lettering[0][0]>-0.7 && lettering[2][1]<0.7);
        assert!(!m.materials.iter().any(|(_,name,_)|name=="gold"));
        let mut mesh=SurfaceMeshData::default();m.append_fitted(&mut mesh,[0.,96.,0.,64.],0.,16.,Some(48.));
        let before=mesh.clone();let paint=[[0.3,0.2,0.1,1.];8];m.recolor_fitted_modern_gym(&mut mesh,0,paint);
        assert_eq!(mesh.positions,before.positions);assert_eq!(mesh.normals,before.normals);assert_eq!(mesh.indices,before.indices);
        for (range,name,_) in &m.materials {if name.starts_with("roof_") {
            assert_eq!(mesh.colors[range.clone()],before.colors[range.clone()],"native modern roof and door keep their existing producer");
        }}
        assert!(mesh.uvs.iter().any(|uv|crate::scenery_finish::town_paint_kind(*uv)==Some(3)));
    }

    #[test]
    fn radio_tower_has_four_window_banks_per_upper_floor_and_recessed_vent_bands() {
        let m=model(Kind::RadioTower);
        assert_eq!(m.min,[-2.,0.,-2.]);assert_eq!(m.max,[2.,9.809999,2.]);
        let beacon=m.surface.positions.iter().enumerate().filter(|(i,_)|
            m.vertex_material(*i)=="coral").map(|(_,p)|*p).collect::<Vec<_>>();
        assert!(!beacon.is_empty() && beacon.iter().all(|p|p[0]>0.8 && p[2]>1.1 && p[1]>9.6),
            "the aerial beacon sits above the front corner rather than behind the roof centre");
        for band in [2.65,3.85,5.05,6.25] {
            let mut spans=Vec::<[f32;2]>::new();
            for tri in m.surface.indices.chunks_exact(3) {
                let p=tri.iter().map(|i|m.surface.positions[*i as usize]).collect::<Vec<_>>();
                if m.vertex_material(tri[0]as usize)!="glass" || m.surface.normals[tri[0]as usize][2]<0.99
                    || p.iter().any(|v|v[2]<1.4 || (v[1]-band).abs()>0.33) {continue;}
                spans.push([p.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min),p.iter().map(|v|v[0]).fold(f32::NEG_INFINITY,f32::max)]);
            }
            spans.sort_by(|a,b|a[0].total_cmp(&b[0]));let mut groups=Vec::<[f32;2]>::new();
            for span in spans {if let Some(last)=groups.last_mut() && span[0]<=last[1]+0.00001 {last[1]=last[1].max(span[1]);}else{groups.push(span);}}
            assert_eq!(groups.len(),4,"four native paired banks at height{band}");
            assert!(m.surface.positions.iter().enumerate().any(|(i,p)|m.vertex_material(i)=="stone_light"
                && p[2]>1.6 && (p[1]-(band-0.60)).abs()<0.12),"closed louvres sit below the glazing");
        }
    }

    #[test]
    fn battle_tower_has_a_convex_cyan_curtain_wall_and_coral_wings() {
        let export:Export=crate::model_storage::parse(JSON[Kind::BattleTower as usize]).unwrap();
        let glass=export.primitives.iter().find(|p|p.material=="glass_tower_cyan").expect("native cyan central drum");
        let coral=export.primitives.iter().find(|p|p.material=="tower_coral").expect("native coral flanking piers");
        assert!(coral.positions.chunks_exact(3).any(|p|p[0]< -3.));
        assert!(coral.positions.chunks_exact(3).any(|p|p[0]>3.));
        let front=glass.positions.chunks_exact(3).map(|p|p[2]).fold(f32::NEG_INFINITY,f32::max);
        let shoulder=glass.positions.chunks_exact(3).map(|p|p[2]).fold(f32::INFINITY,f32::min);
        assert!(front-shoulder>0.5,"curtain wall must bow forward instead of becoming a flat office facade");
        assert!(glass.normals.chunks_exact(3).any(|n|n[0]>0.5 && n[2]>0.3));
        assert!(glass.normals.chunks_exact(3).any(|n|n[0]< -0.5 && n[2]>0.3));
        assert!(export.primitives.iter().map(|p|p.indices.len()/3).sum::<usize>()<12000);
    }

    #[test]
    fn battle_tower_paired_entry_has_recessed_glass_across_both_native_door_slots() {
        let m=model(Kind::BattleTower);
        assert_eq!(m.min,[-5.,0.,-3.]);
        assert_eq!(m.max,[5.,8.356,3.]);
        assert_eq!(m.anchor,Some([0.,0.,3.]));
        for x in [-0.95,-0.45,0.45,0.95] {for y in [0.5,1.,1.6] {
            let point=Vec2::new(x,y);let mut front=None;
            for tri in m.surface.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|m.surface.positions[i as usize]);
                let [a,b,c]=p.map(|v|Vec2::new(v[0],v[1]));
                let area=(b-a).perp_dot(c-a);
                if area.abs()<0.000001 {continue;}
                let v=(point-a).perp_dot(c-a)/area;
                let w=(b-a).perp_dot(point-a)/area;let u=1.-v-w;
                if u.min(v).min(w)< -0.00001 {continue;}
                let z=u*p[0][2]+v*p[1][2]+w*p[2][2];
                if front.is_none_or(|(old,_)|z>old) {front=Some((z,tri[0] as usize));}
            }
            let (z,i)=front.expect("complete entrance leaf");
            assert!(m.vertex_material(i).starts_with("glass"),
                "door slot {x},{y} must reveal glazing, not opaque central masonry: {}",m.vertex_material(i));
            assert!(z>2.1 && z<2.24,"glass sits behind the jambs: {z}");
        }}
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
    fn north_gate_passage_clears_both_native_lanes_without_changing_its_asset() {
        let m=model(Kind::EastGate);
        let mut out=SurfaceMeshData::default();
        assert!(m.append_fitted_north_passage(&mut out,[0.,64.,0.,64.],0.,16.,[16.,48.]));
        assert_eq!(out.indices,m.surface.indices);
        assert_eq!(out.positions.len(),m.surface.positions.len());
        let reference=m.rigid_roof.first().expect("authored rigid roof domain").start;
        for i in m.rigid_roof.iter().flat_map(|r|r.clone()) {
            let original=(Vec3::from_array(m.surface.positions[i])-Vec3::from_array(m.surface.positions[reference])).length();
            let fitted=(Vec3::from_array(out.positions[i])-Vec3::from_array(out.positions[reference])).length();
            assert!((fitted-original*16.).abs()<0.0001,"the roof and truss must stay rigid across the widened opening");
        }
        for (p,n) in out.positions.iter().zip(&out.normals) {
            assert!(p[0]>=-0.001 && p[0]<=64.001 && p[2]>=-0.001 && p[2]<=64.001);
            assert!((Vec3::from_array(*n).length()-1.).abs()<0.00001);
        }
        // Horizontal rays above the existing plinth must pass completely
        // through the building at both authored trainer-foot columns.
        for x in [24.,40.] {for y in [6.,12.,20.,28.] {
            let point=Vec2::new(x,y);
            for tri in out.indices.chunks_exact(3) {
                let p=[tri[0],tri[1],tri[2]].map(|i|out.positions[i as usize]);
                let [a,b,c]=p.map(|v|Vec2::new(v[0],v[1]));
                let area=(b-a).perp_dot(c-a);
                if area.abs()<0.00001 {continue;}
                let v=(point-a).perp_dot(c-a)/area;
                let w=(b-a).perp_dot(point-a)/area;
                let u=1.-v-w;
                if u.min(v).min(w)>=-0.00001 {
                    let z=u*p[0][2]+v*p[1][2]+w*p[2][2];
                    assert!(z< -0.001 || z>64.001,"blocked native lane at {x},{y},{z}");
                }
            }
        }}
        let before=out.clone();
        assert!(!m.append_fitted_north_passage(&mut out,[0.,64.,0.,64.],0.,16.,[48.,16.]));
        assert_eq!(out,before);
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
