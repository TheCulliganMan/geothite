//! Original editable dungeon kit exported by `tools/build-dungeon-models.py`.
//!
//! Geometry is independent of game artwork. Source-aware placement is kept in
//! `mesh::modeled_dungeons`; this importer never reads collision or map data.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    CaveBoulder,
    DarkBoulder,
    IceBoulder,
    IceMass,
    TowerGuardian,
    AlphGuardian,
    StoneTablet,
    GymPlaque,
    WarehouseCrate,
    ShipBarrel,
    ShipStool,
    ShipRack,
    ShipBunk,
    PortholeBulkhead,
    TimberColumn,
    WarningBeacon,
    GymBin,
    LeaguePodium,
    TimberWall,
}
impl Kind {
    pub(crate) const ALL: [Self; 19] = [
        Self::CaveBoulder,
        Self::DarkBoulder,
        Self::IceBoulder,
        Self::IceMass,
        Self::TowerGuardian,
        Self::AlphGuardian,
        Self::StoneTablet,
        Self::GymPlaque,
        Self::WarehouseCrate,
        Self::ShipBarrel,
        Self::ShipStool,
        Self::ShipRack,
        Self::ShipBunk,
        Self::PortholeBulkhead,
        Self::TimberColumn,
        Self::WarningBeacon,
        Self::GymBin,
        Self::LeaguePodium,
        Self::TimberWall,
    ];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::CaveBoulder => "dungeon:cave-boulder",
            Self::DarkBoulder => "dungeon:dark-boulder",
            Self::IceBoulder => "dungeon:ice-boulder",
            Self::IceMass => "dungeon:ice-mass",
            Self::TowerGuardian => "dungeon:tower-guardian",
            Self::AlphGuardian => "dungeon:alph-guardian",
            Self::StoneTablet => "dungeon:stone-tablet",
            Self::GymPlaque => "dungeon:gym-plaque",
            Self::WarehouseCrate => "dungeon:warehouse-crate",
            Self::ShipBarrel => "dungeon:ship-barrel",
            Self::ShipStool => "dungeon:ship-stool",
            Self::ShipRack => "dungeon:ship-rack",
            Self::ShipBunk => "dungeon:ship-bunk",
            Self::PortholeBulkhead => "dungeon:porthole-bulkhead",
            Self::TimberColumn => "dungeon:timber-column",
            Self::WarningBeacon => "dungeon:warning-beacon",
            Self::GymBin => "dungeon:gym-bin",
            Self::LeaguePodium => "dungeon:league-podium",
            Self::TimberWall => "dungeon:timber-wall",
        }
    }
}
#[derive(Deserialize)]
struct Export {
    primitives: Vec<Primitive>,
}
#[derive(Deserialize)]
struct Primitive {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
    // Optional semantic material anchor into a verified source drawing. Assets
    // store coordinates only; palette pixels are always supplied by the runtime.
    #[serde(default)]
    source_pixel: Option<[u16; 2]>,
}
pub(crate) struct Model {
    surface: SurfaceMeshData,
    min: [f32; 3],
    max: [f32; 3],
}
impl Model {
    fn parse<'a>(json: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in export.primitives {
            let count = p.positions.len() / 3;
            if count == 0
                || p.positions.len() % 3 != 0
                || p.normals.len() != p.positions.len()
                || p.indices.is_empty()
                || p.indices.len() % 3 != 0
                || p.indices.iter().any(|&i| i as usize >= count)
                || p.positions.iter().chain(&p.normals).any(|v| !v.is_finite())
                || p.base_color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err("invalid dungeon primitive".into());
            }
            let base = surface.positions.len() as u32;
            for v in p.positions.chunks_exact(3) {
                let v = [v[0], v[1], v[2]];
                for a in 0..3 {
                    min[a] = min[a].min(v[a]);
                    max[a] = max[a].max(v[a]);
                }
                surface.positions.push(v);
                surface.uvs.push(
                    p.source_pixel
                        .map_or([0.0; 2], |p| [p[0] as f32, p[1] as f32]),
                );
                surface.colors.push(p.base_color);
            }
            for n in p.normals.chunks_exact(3) {
                let n = Vec3::new(n[0], n[1], n[2]);
                if n.length_squared() < 1e-8 {
                    return Err("zero dungeon normal".into());
                }
                surface.normals.push(n.normalize().to_array());
            }
            surface
                .indices
                .extend(p.indices.into_iter().map(|i| i + base));
        }
        if (0..3).any(|a| !min[a].is_finite() || !max[a].is_finite() || max[a] - min[a] <= 1e-6) {
            return Err("dungeon model must have volume".into());
        }
        Ok(Self { surface, min, max })
    }
    /// Fit the entire closed, authored volume inside a verified physical plot.
    /// Inverse-transpose normal scaling keeps beveled faces correctly lit.
    pub(crate) fn append(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4],
        base_height: f32,
        height: f32,
    ) {
        let [west, east, north, south] = bounds;
        if height <= 0.0 || east <= west || south <= north {
            return;
        }
        let scale = Vec3::new(
            (east - west) / (self.max[0] - self.min[0]),
            height / (self.max[1] - self.min[1]),
            (south - north) / (self.max[2] - self.min[2]),
        );
        let origin = Vec3::new(west, base_height, north);
        let min = Vec3::from_array(self.min);
        let base = target.positions.len() as u32;
        for ((&p, &n), &color) in self
            .surface
            .positions
            .iter()
            .zip(&self.surface.normals)
            .zip(&self.surface.colors)
        {
            target
                .positions
                .push((origin + (Vec3::from_array(p) - min) * scale).to_array());
            let normal = (Vec3::from_array(n) / scale).normalize();
            target.normals.push(normal.to_array());
            target
                .colors
                .push(crate::interior_models::authored_face_color(color, normal));
        }
        target.uvs.extend_from_slice(&self.surface.uvs);
        target
            .indices
            .extend(self.surface.indices.iter().map(|&i| i + base));
    }
    /// Taller ship wall fit with an unchanged central window band. The
    /// original round brass/glass/rivet assembly lies wholly inside native
    /// y=5..14 of its 16px panel and translates upward without scaling. Only
    /// the plain lower/upper wall is extended. No cached vertices are changed.
    pub(crate) fn append_porthole(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4],
        base_height: f32,
        native_height: f32,
        height: f32,
    ) {
        if height <= native_height || native_height <= 0. {
            self.append(target, bounds, base_height, height);
            return;
        }
        let [west, east, north, south] = bounds;
        if east <= west || south <= north {
            return;
        }
        let sx = (east - west) / (self.max[0] - self.min[0]);
        let sy = native_height / (self.max[1] - self.min[1]);
        let sz = (south - north) / (self.max[2] - self.min[2]);
        let low = native_height * (5. / 16.);
        let high = native_height * (14. / 16.);
        let shift = (height - native_height) * 0.5;
        let base = target.positions.len() as u32;
        for ((&p, &n), &color) in self
            .surface
            .positions
            .iter()
            .zip(&self.surface.normals)
            .zip(&self.surface.colors)
        {
            let old_y = (p[1] - self.min[1]) * sy;
            let (y, derivative) = if old_y < low {
                let d = (low + shift) / low;
                (old_y * d, d)
            } else if old_y > high {
                let d = (native_height - high + shift) / (native_height - high);
                (height - (native_height - old_y) * d, d)
            } else {
                (old_y + shift, 1.)
            };
            target.positions.push([
                west + (p[0] - self.min[0]) * sx,
                base_height + y,
                north + (p[2] - self.min[2]) * sz,
            ]);
            let normal = (Vec3::from_array(n) / Vec3::new(sx, sy * derivative, sz)).normalize();
            target.normals.push(normal.to_array());
            target
                .colors
                .push(crate::interior_models::authored_face_color(color, normal));
        }
        target.uvs.extend_from_slice(&self.surface.uvs);
        target
            .indices
            .extend(self.surface.indices.iter().map(|&i| i + base));
    }
}
pub(crate) fn model(kind: Kind) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!("models/dungeons/cave_boulder.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/dark_boulder.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/ice_boulder.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/ice_mass.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/tower_guardian.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/alph_guardian.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/stone_tablet.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/gym_plaque.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/warehouse_crate.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/ship_barrel.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/ship_stool.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/ship_rack.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/ship_bunk.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/porthole_bulkhead.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/timber_column.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/warning_beacon.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/gym_bin.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/league_podium.mesh.json"),
            crate::model_storage::include_model!("models/dungeons/timber_wall.mesh.json"),
        ]
        .into_iter()
        .map(|s| Model::parse(s).expect("checked dungeon mesh"))
        .collect()
    })[kind as usize]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_dungeon_faces_are_lit_once_after_nonuniform_normal_fitting() {
        let source = model(Kind::CaveBoulder);
        let original = source.surface.colors.clone();
        let mut mesh = SurfaceMeshData::default();
        source.append(&mut mesh, [-17.0, 11.0, -5.0, 2.0], 3.0, 19.0);
        let length = mesh.colors.len();
        let mut orientation_matters = false;
        for (i, &base) in original.iter().enumerate() {
            let fitted = Vec3::from_array(mesh.normals[i]);
            let expected = crate::interior_models::authored_face_color(base, fitted);
            assert_eq!(mesh.colors[i], expected);
            assert_eq!(mesh.colors[i][3], base[3]);
            orientation_matters |= expected
                != crate::interior_models::authored_face_color(
                    base,
                    Vec3::from_array(source.surface.normals[i]),
                );
        }
        assert!(
            orientation_matters,
            "lighting must use fitted world normals, not the export normals"
        );
        assert_ne!(
            mesh.colors, original,
            "unlit base colors would flatten the model"
        );
        source.append(&mut mesh, [-17.0, 11.0, -5.0, 2.0], 3.0, 19.0);
        assert_eq!(mesh.colors[..length], mesh.colors[length..]);
        assert_eq!(
            source.surface.colors, original,
            "cached source palettes must remain untouched"
        );
    }

    #[test]
    fn all_dungeon_assets_are_finite_full_volume_and_budgeted() {
        for kind in Kind::ALL {
            let m = model(kind);
            assert!(m.surface.indices.len() / 3 >= 24, "{kind:?}");
            assert!(m.surface.indices.len() / 3 < 8000, "{kind:?}");
            assert!(m
                .surface
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4));
            assert_eq!(m.surface.positions.len(), m.surface.colors.len());
            assert_eq!(m.surface.positions.len(), m.surface.uvs.len());
        }
    }
    #[test]
    fn dungeon_fits_preserve_exact_bounds_and_normal_lengths() {
        for kind in Kind::ALL {
            let mut mesh = SurfaceMeshData::default();
            model(kind).append(&mut mesh, [-17.0, 11.0, -5.0, 2.0], 3.0, 19.0);
            for p in &mesh.positions {
                assert!(p[0] >= -17.001 && p[0] <= 11.001);
                assert!(p[1] >= 2.999 && p[1] <= 22.001);
                assert!(p[2] >= -5.001 && p[2] <= 2.001);
            }
            assert!(mesh
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4));
        }
    }
    #[test]
    fn malformed_or_flat_models_are_rejected() {
        assert!(Model::parse(r#"{"primitives":[]}"#).is_err());
        assert!(Model::parse(r#"{"primitives":[{"positions":[0,0,0],"normals":[0,0,0],"indices":[0,0,0],"base_color":[1,1,1,1]}]}"#).is_err());
    }
}

#[path = "dungeon_extension_models.rs"]
mod extension;
pub(crate) use extension::{extension_model, ExtensionKind};

#[path = "special_room_models.rs"]
mod special_rooms;
pub(crate) use special_rooms::{room_model, RoomAsset};

#[path = "gym_scenery_models.rs"]
mod gym_scenery;
pub(crate) use gym_scenery::{gym_model, gym_wall_join_triangles};

#[path = "traditional_room_models.rs"]
mod traditional_room;
pub(crate) use traditional_room::traditional_model;

#[cfg(test)]
mod ship_wall_height_tests {
    use super::*;
    #[test]
    fn taller_porthole_keeps_window_band_round_and_native_height_is_bit_identical() {
        let model = model(Kind::PortholeBulkhead);
        for scale in [0.5, 1., 1.5] {
            let bounds = [-8. * scale, 8. * scale, 3. * scale, 6. * scale];
            let mut native = SurfaceMeshData::default();
            let mut same = SurfaceMeshData::default();
            let mut tall = SurfaceMeshData::default();
            model.append(&mut native, bounds, 2., 16. * scale);
            model.append_porthole(&mut same, bounds, 2., 16. * scale, 16. * scale);
            model.append_porthole(&mut tall, bounds, 2., 16. * scale, 28. * scale);
            assert_eq!(native, same);
            assert_eq!(native.indices, tall.indices);
            assert_eq!(native.uvs, tall.uvs);
            assert_eq!(native.positions.len(), tall.positions.len());
            let mut fixed = 0;
            for (a, b) in native.positions.iter().zip(&tall.positions) {
                assert_eq!((a[0], a[2]), (b[0], b[2]));
                let y = (a[1] - 2.) / scale;
                if (5. ..=14.).contains(&y) {
                    assert!((b[1] - a[1] - 6. * scale).abs() < 0.0001);
                    fixed += 1;
                }
                assert!((2. - 0.0001..=2. + 28. * scale + 0.0001).contains(&b[1]));
            }
            assert!(fixed > 100);
            assert!(
                (tall
                    .positions
                    .iter()
                    .map(|p| p[1])
                    .fold(f32::NEG_INFINITY, f32::max)
                    - 2.
                    - 28. * scale)
                    .abs()
                    < 0.0001
            );
            assert!(tall
                .normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.0001));
            assert!(tall
                .colors
                .iter()
                .all(|c| c[3] == 1. && c.iter().all(|v| v.is_finite())));
        }
    }
}
