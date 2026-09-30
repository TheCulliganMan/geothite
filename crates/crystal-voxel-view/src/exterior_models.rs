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
    KantoMuseum,
    Silph,
    KantoMansion,
    PowerPlant,
    ModernDaycare,
    ForestConifer,
    KantoCanopy,
    CutTree,
    ParkHedge,
    ShoreBoulder,
    RouteSign,
    ParkBench,
    TimberFence,
    StonePost,
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
            Self::KantoMuseum => "world-exterior/kanto_museum",
            Self::Silph => "world-exterior/silph",
            Self::KantoMansion => "world-exterior/kanto_mansion",
            Self::PowerPlant => "world-exterior/power_plant",
            Self::ModernDaycare => "world-exterior/modern_daycare",
            Self::ForestConifer => "world-exterior/forest_conifer",
            Self::KantoCanopy => "world-exterior/kanto_canopy",
            Self::CutTree => "world-exterior/cut_tree",
            Self::ParkHedge => "world-exterior/park_hedge",
            Self::ShoreBoulder => "world-exterior/shore_boulder",
            Self::RouteSign => "world-exterior/route_sign",
            Self::ParkBench => "world-exterior/park_bench",
            Self::TimberFence => "world-exterior/timber_fence",
            Self::StonePost => "world-exterior/stone_post",
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
    Kind::KantoMuseum,
    Kind::Silph,
    Kind::KantoMansion,
    Kind::PowerPlant,
    Kind::ModernDaycare,
    Kind::ForestConifer,
    Kind::KantoCanopy,
    Kind::CutTree,
    Kind::ParkHedge,
    Kind::ShoreBoulder,
    Kind::RouteSign,
    Kind::ParkBench,
    Kind::TimberFence,
    Kind::StonePost,
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
];
const JSON: &[&str] = &[
    include_str!("../models/world_exteriors/east_gate.mesh.json"),
    include_str!("../models/world_exteriors/tin_tower.mesh.json"),
    include_str!("../models/world_exteriors/burned_tower.mesh.json"),
    include_str!("../models/world_exteriors/lighthouse.mesh.json"),
    include_str!("../models/world_exteriors/johto_barn.mesh.json"),
    include_str!("../models/world_exteriors/forest_shrine.mesh.json"),
    include_str!("../models/world_exteriors/radio_tower.mesh.json"),
    include_str!("../models/world_exteriors/battle_tower.mesh.json"),
    include_str!("../models/world_exteriors/modern_house.mesh.json"),
    include_str!("../models/world_exteriors/modern_blank.mesh.json"),
    include_str!("../models/world_exteriors/modern_shop.mesh.json"),
    include_str!("../models/world_exteriors/kanto_house.mesh.json"),
    include_str!("../models/world_exteriors/kanto_flat_house.mesh.json"),
    include_str!("../models/world_exteriors/modern_center.mesh.json"),
    include_str!("../models/world_exteriors/modern_mart.mesh.json"),
    include_str!("../models/world_exteriors/modern_gym.mesh.json"),
    include_str!("../models/world_exteriors/modern_station.mesh.json"),
    include_str!("../models/world_exteriors/modern_department.mesh.json"),
    include_str!("../models/world_exteriors/modern_arcade.mesh.json"),
    include_str!("../models/world_exteriors/kanto_center.mesh.json"),
    include_str!("../models/world_exteriors/kanto_mart.mesh.json"),
    include_str!("../models/world_exteriors/kanto_gym.mesh.json"),
    include_str!("../models/world_exteriors/kanto_station.mesh.json"),
    include_str!("../models/world_exteriors/kanto_department.mesh.json"),
    include_str!("../models/world_exteriors/kanto_arcade.mesh.json"),
    include_str!("../models/world_exteriors/kanto_museum.mesh.json"),
    include_str!("../models/world_exteriors/silph.mesh.json"),
    include_str!("../models/world_exteriors/kanto_mansion.mesh.json"),
    include_str!("../models/world_exteriors/power_plant.mesh.json"),
    include_str!("../models/world_exteriors/modern_daycare.mesh.json"),
    include_str!("../models/world_exteriors/forest_conifer.mesh.json"),
    include_str!("../models/world_exteriors/kanto_canopy.mesh.json"),
    include_str!("../models/world_exteriors/cut_tree.mesh.json"),
    include_str!("../models/world_exteriors/park_hedge.mesh.json"),
    include_str!("../models/world_exteriors/shore_boulder.mesh.json"),
    include_str!("../models/world_exteriors/route_sign.mesh.json"),
    include_str!("../models/world_exteriors/park_bench.mesh.json"),
    include_str!("../models/world_exteriors/timber_fence.mesh.json"),
    include_str!("../models/world_exteriors/stone_post.mesh.json"),
    include_str!("../models/world_exteriors/kanto_blank.mesh.json"),
    include_str!("../models/world_exteriors/forest_gate.mesh.json"),
    include_str!("../models/world_exteriors/forest_gate_closed.mesh.json"),
    include_str!("../models/world_exteriors/highland_mesa.mesh.json"),
    include_str!("../models/world_exteriors/diglett_cave.mesh.json"),
    include_str!("../models/world_exteriors/tower_forecourt.mesh.json"),
    include_str!("../models/world_exteriors/lavender_radio_tower.mesh.json"),
    include_str!("../models/world_exteriors/kanto_route_gate.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_northwest.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_north.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_northeast.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_interior.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_southwest.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_south.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_southeast.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_notch_east.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_notch_west.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_stair_right.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_stair_corner.mesh.json"),
    include_str!("../models/world_exteriors/ice_shelf_stair_left.mesh.json"),
];

#[derive(Deserialize)]
struct Primitive {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
#[derive(Deserialize)]
struct Export {
    primitives: Vec<Primitive>,
    door_anchor: Option<[f32; 3]>,
    door_face: Option<String>,
}
pub(crate) struct Model {
    pub(crate) surface: SurfaceMeshData,
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
    anchor: Option<[f32; 3]>,
    east: bool,
}
impl Model {
    fn parse(json: &str) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
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
            surface,
            min,
            max,
            anchor: export.door_anchor,
            east,
        })
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
fn fit_door_axis(x: f32, source: [f32; 2], target: [f32; 2], anchor: f32, door: f32) -> (f32, f32) {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exterior_models_have_closed_volume_valid_normals_and_budget() {
        assert_eq!(ALL.len(), JSON.len());
        for &kind in ALL {
            let m = model(kind);
            assert!(m.surface.indices.len() / 3 < 12000, "{kind:?}");
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
}
