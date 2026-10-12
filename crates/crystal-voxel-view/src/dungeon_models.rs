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
    #[serde(default)]
    name: String,
    primitives: Vec<Primitive>,
}
#[derive(Deserialize)]
struct Primitive {
    #[serde(default)]
    name: String,
    #[serde(default)]
    material: String,
    #[serde(default)]
    palette_regions: Vec<(usize, usize, String)>,
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
    surface: crate::shared_model_geometry::ModelSurface,
    gym_stone_pixels: Option<Vec<[usize; 2]>>,
    pewter_guardian_pixels: Option<Vec<[usize; 2]>>,
    tower_timber_pixels: Option<Vec<[usize; 2]>>,
    gym_orb_pixels: Option<Vec<[usize; 2]>>,
    min: [f32; 3],
    max: [f32; 3],
}
impl Model {
    fn parse<'a>(json: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
        let mut gym_stone_pixels = Vec::new();
        let mut named_gym_stone = export.name == "cave_boulder";
        let mut guardian_pixels = Vec::new();
        let mut named_guardian = export.name == "tower_guardian";
        let mut timber_pixels = Vec::new();
        let mut named_timber = matches!(export.name.as_str(), "timber_wall" | "timber_column");
        let mut orb_pixels = Vec::new();
        let mut named_orb = export.name == "league_podium";
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in export.primitives {
            let count = p.positions.len() / 3;
            let stone_pixel = match p.name.as_str() {
                "stone" => [8, 6],
                "stone_light" => [4, 4],
                _ => {
                    named_gym_stone = false;
                    [0, 0]
                }
            };
            gym_stone_pixels.extend(std::iter::repeat_n(stone_pixel, count));
            let mut next = 0;
            for (start, end, region) in &p.palette_regions {
                let pixel = match (region.as_str(), p.name.as_str()) {
                    ("head", "stone") => Some([2, 2]),
                    ("head", "stone_light") => Some([8, 1]),
                    ("head", "stone_dark") => Some([8, 10]),
                    ("pedestal", "stone") => Some([8, 24]),
                    ("pedestal", "stone_light") => Some([2, 18]),
                    ("pedestal", "stone_dark") => Some([4, 18]),
                    _ => None,
                };
                if *start != next || *end <= *start || *end > count || pixel.is_none() {
                    named_guardian = false;
                    break;
                }
                guardian_pixels.extend(std::iter::repeat_n(pixel.unwrap(), end - start));
                next = *end;
            }
            named_guardian &= next == count;
            let mut next = 0;
            for (start, end, region) in &p.palette_regions {
                let pixel = match (region.as_str(), p.name.as_str()) {
                    ("orb", "stone_dark") => Some([4, 4]),
                    ("orb", "stone_light") => Some([7, 3]),
                    ("pedestal", "stone") => Some([8, 18]),
                    ("pedestal", "stone_light") => Some([8, 16]),
                    ("pedestal", "stone_dark") => Some([4, 20]),
                    _ => None,
                };
                if *start != next || *end <= *start || *end > count || pixel.is_none() {
                    named_orb = false;
                    break;
                }
                orb_pixels.extend(std::iter::repeat_n(pixel.unwrap(), end - start));
                next = *end;
            }
            named_orb &= next == count;
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
            let index_start = surface.indices.len();
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
            if !crate::roof_tiles::refine(&mut surface, base as usize, index_start, &p.material) {
                crate::scenery_finish::refine(
                    &mut surface,
                    base as usize,
                    index_start,
                    &p.material,
                    &export.name,
                );
            }
            let timber_pixel = match p.name.as_str() {
                "wood" => [0, 0],
                "wood_light" => [0, 2],
                "wood_dark" => [0, 1],
                "steel" => [4, 0],
                "stone" if export.name == "timber_column" => [4, 0],
                _ => { named_timber = false; [0, 0] }
            };
            // Refinement belongs to the current named primitive; include its
            // added vertices rather than guessing ownership by original count.
            timber_pixels.extend(std::iter::repeat_n(
                timber_pixel, surface.positions.len() - base as usize));
        }
        crate::scenery_finish::contain(&mut surface, min, max);
        if (0..3).any(|a| !min[a].is_finite() || !max[a].is_finite() || max[a] - min[a] <= 1e-6) {
            return Err("dungeon model must have volume".into());
        }
        // Refined geometry cannot inherit semantic ownership by vertex index.
        let gym_stone_pixels = (named_gym_stone
            && gym_stone_pixels.len() == surface.positions.len())
            .then_some(gym_stone_pixels);
        let pewter_guardian_pixels = (named_guardian
            && guardian_pixels.len() == surface.positions.len()).then_some(guardian_pixels);
        let tower_timber_pixels = (named_timber
            && timber_pixels.len() == surface.positions.len()).then_some(timber_pixels);
        let gym_orb_pixels = (named_orb
            && orb_pixels.len() == surface.positions.len()).then_some(orb_pixels);
        Ok(Self { surface: surface.into(), gym_stone_pixels, pewter_guardian_pixels, tower_timber_pixels, gym_orb_pixels, min, max })
    }
    #[cfg(test)]
    pub(crate) fn shared_geometry(&self)->&std::sync::Arc<crate::shared_model_geometry::Geometry> {&self.surface.geometry}
    /// Only the canonical named stone parts use live gym palette anchors.
    /// Geometry, fitted normals and the cached cave palette remain untouched.
    pub(crate) fn append_gym_stone(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4],
        height: f32,
        source_uv: impl Fn([usize; 2]) -> [f32; 2],
    ) {
        let pixels = self.gym_stone_pixels.as_ref().expect("named canonical gym stone");
        self.append_source_colors(target, bounds, height, pixels, source_uv);
    }
    pub(crate) fn append_pewter_guardian(
        &self, target: &mut SurfaceMeshData, bounds: [f32; 4], height: f32,
        source_uv: impl Fn([usize; 2]) -> [f32; 2],
    ) {
        let pixels = self.pewter_guardian_pixels.as_ref().expect("named guardian assemblies");
        self.append_source_colors(target, bounds, height, pixels, source_uv);
    }
    pub(crate) fn append_sprout_guardian(
        &self, target: &mut SurfaceMeshData, bounds: [f32;4], height: f32,
        source_uv: impl Fn([usize;2]) -> [f32;2],
    ) {
        let regions = self.pewter_guardian_pixels.as_ref().expect("named guardian assemblies");
        // Semantic head/pedestal ranges stay on the cached original geometry.
        // Both verified Sprout head variants share these live color anchors.
        self.append_source_colors(target,bounds,height,regions,|region| {
            source_uv(match region {
                [2,2] => [8,6],
                [8,1] => [8,1],
                [8,10] => [6,0],
                [8,24] => [8,24],
                [2,18] => [2,26],
                [4,18] => [0,24],
                _ => unreachable!("validated guardian material ownership"),
            })
        });
    }
    pub(crate) fn append_tower_timber(
        &self, target: &mut SurfaceMeshData, bounds: [f32; 4], height: f32,
        source_uv: impl Fn([usize; 2]) -> [f32; 2],
    ) {
        let pixels = self.tower_timber_pixels.as_ref().expect("named tower timber materials");
        let first = target.positions.len();
        self.append_source_colors(target, bounds, height, pixels, source_uv);
        for i in 0..target.positions.len() - first {
            // Keep model-local timber grain when replacing the original paint
            // with a live palette swatch. Iron pegs retain their plain finish.
            if pixels[i] != [4, 0] {
                let grain = 1. + 0.10 * crate::scenery_finish::timber_grain(self.surface.positions[i]);
                for channel in &mut target.colors[first + i][..3] { *channel *= grain; }
            }
        }
    }
    pub(crate) fn append_gym_orb(
        &self, target: &mut SurfaceMeshData, bounds: [f32; 4], height: f32,
        source_uv: impl Fn([usize; 2]) -> [f32; 2],
    ) {
        let pixels = self.gym_orb_pixels.as_ref().expect("named gym orb and pedestal assemblies");
        self.append_source_colors(target, bounds, height, pixels, source_uv);
    }
    fn append_source_colors(
        &self, target: &mut SurfaceMeshData, bounds: [f32; 4], height: f32,
        pixels: &[[usize; 2]], source_uv: impl Fn([usize; 2]) -> [f32; 2],
    ) {
        let first = target.positions.len();
        self.append(target, bounds, 0., height);
        for (i, pixel) in pixels.iter().enumerate().take(target.positions.len() - first) {
            target.uvs[first + i] = source_uv(*pixel);
            target.colors[first + i] = crate::interior_models::authored_face_color(
                [1.; 4], Vec3::from_array(target.normals[first + i]),
            );
        }
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
    fn shared_assets_cave_and_ice_boulders_reuse_geometry_with_distinct_material_ownership() {
        let cave=model(Kind::CaveBoulder);let ice=model(Kind::IceBoulder);
        assert!(std::sync::Arc::ptr_eq(&cave.surface.geometry,&ice.surface.geometry));
        assert_ne!(cave.surface.colors,ice.surface.colors);
        assert!(cave.gym_stone_pixels.is_some());assert!(ice.gym_stone_pixels.is_none());
        let mut first=SurfaceMeshData::default();let mut second=SurfaceMeshData::default();
        cave.append(&mut first,[0.,16.,0.,16.],0.,10.);
        ice.append(&mut second,[0.,16.,0.,16.],0.,10.);
        assert_eq!(first.positions,second.positions);assert_eq!(first.indices,second.indices);
        assert_ne!(first.colors,second.colors);
    }

    #[test]
    fn gym_orb_named_assemblies_preserve_gray_pedestal_and_cached_geometry() {
        let source = model(Kind::LeaguePodium);
        let cached = source.surface.clone();
        let mut ordinary = SurfaceMeshData::default();
        let mut live = SurfaceMeshData::default();
        source.append(&mut ordinary, [-8., 8., -5., 5.], 0., 28.);
        source.append_gym_orb(&mut live, [-8., 8., -5., 5.], 28., |p| {
            [p[0] as f32 / 16., p[1] as f32 / 32.]
        });
        assert_eq!(ordinary.positions, live.positions);
        assert_eq!(ordinary.normals, live.normals);
        assert_eq!(ordinary.indices, live.indices);
        for p in [[4,4],[7,3],[8,18],[8,16],[4,20]] {
            assert!(live.uvs.contains(&[p[0] as f32 / 16., p[1] as f32 / 32.]));
        }
        assert_eq!(source.surface, cached);
        for kind in Kind::ALL {
            assert_eq!(model(kind).gym_orb_pixels.is_some(), kind == Kind::LeaguePodium);
        }
        let json = include_str!("../models/dungeons/league_podium.mesh.json");
        for changed in [json.replace("league_podium", "other"), json.replace("\"orb\"", "\"unknown\""),
                        json.replace("\"pedestal\"", "\"unknown\"")] {
            assert!(Model::parse(changed.as_str()).unwrap().gym_orb_pixels.is_none());
        }
        let mut incomplete: serde_json::Value = serde_json::from_str(json).unwrap();
        incomplete["primitives"][1]["palette_regions"].as_array_mut().unwrap().pop();
        assert!(Model::parse(serde_json::to_string(&incomplete).unwrap().as_str())
            .unwrap().gym_orb_pixels.is_none());
    }
    #[test]
    fn tower_timber_named_palette_keeps_closed_geometry_and_cached_paint() {
        let source = model(Kind::TimberWall);
        let cached = source.surface.clone();
        let mut ordinary = SurfaceMeshData::default();
        let mut live = SurfaceMeshData::default();
        source.append(&mut ordinary, [-8., 8., -3., 3.], 0., 16.);
        source.append_tower_timber(&mut live, [-8., 8., -3., 3.], 16., |p| {
            [p[0] as f32 / 8., p[1] as f32 / 8.]
        });
        assert_eq!(ordinary.positions, live.positions);
        assert_eq!(ordinary.normals, live.normals);
        assert_eq!(ordinary.indices, live.indices);
        for p in [[0, 0], [0, 2], [0, 1], [4, 0]] {
            assert!(live.uvs.contains(&[p[0] as f32 / 8., p[1] as f32 / 8.]));
        }
        let grain: Vec<_> = live.uvs.iter().enumerate()
            .filter(|(_, uv)| **uv == [0., 0.])
            .map(|(i, _)| live.colors[i][0] /
                crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(live.normals[i]))[0])
            .collect();
        assert!(grain.iter().all(|v| (0.899..=1.101).contains(v)));
        assert!(grain.iter().copied().fold(f32::NEG_INFINITY, f32::max)
            - grain.iter().copied().fold(f32::INFINITY, f32::min) > 0.01,
            "live wood color must retain visible model-local grain");
        assert_eq!(source.surface, cached);
        for kind in Kind::ALL {
            assert_eq!(model(kind).tower_timber_pixels.is_some(),
                matches!(kind, Kind::TimberWall | Kind::TimberColumn));
        }
        let json = include_str!("../models/dungeons/timber_wall.mesh.json");
        for changed in [json.replace("timber_wall", "other"), json.replace("wood_dark", "unknown")] {
            assert!(Model::parse(changed.as_str()).unwrap().tower_timber_pixels.is_none());
        }
    }
    #[test]
    fn sprout_guardian_live_gold_head_and_gray_base_preserve_cached_geometry() {
        let source = model(Kind::TowerGuardian);
        let cached = source.surface.clone();
        let mut ordinary = SurfaceMeshData::default();
        let mut tower = SurfaceMeshData::default();
        source.append(&mut ordinary, [-8., 8., -5., 5.], 0., 24.);
        source.append_sprout_guardian(&mut tower, [-8., 8., -5., 5.], 24., |p| {
            [p[0] as f32 / 16., p[1] as f32 / 32.]
        });
        assert_eq!(ordinary.positions, tower.positions);
        assert_eq!(ordinary.normals, tower.normals);
        assert_eq!(ordinary.indices, tower.indices);
        for p in [[8, 6], [8, 1], [6, 0], [8, 24], [2, 26], [0, 24]] {
            assert!(tower.uvs.contains(&[p[0] as f32 / 16., p[1] as f32 / 32.]));
        }
        assert_eq!(source.surface, cached);
    }
    #[test]
    fn pewter_guardian_named_regions_sample_head_and_pedestal_without_repainting_other_rooms() {
        let source = model(Kind::TowerGuardian);
        let cached = source.surface.clone();
        let mut ordinary = SurfaceMeshData::default();
        let mut gym = SurfaceMeshData::default();
        source.append(&mut ordinary, [-8., 8., -5., 5.], 0., 24.);
        source.append_pewter_guardian(&mut gym, [-8., 8., -5., 5.], 24., |p| {
            [p[0] as f32 / 16., p[1] as f32 / 32.]
        });
        assert_eq!(ordinary.positions, gym.positions);
        assert_eq!(ordinary.normals, gym.normals);
        assert_eq!(ordinary.indices, gym.indices);
        for p in [[2, 2], [8, 1], [8, 10], [8, 24], [2, 18], [4, 18]] {
            assert!(gym.uvs.contains(&[p[0] as f32 / 16., p[1] as f32 / 32.]));
        }
        assert_eq!(source.surface, cached);
        for kind in Kind::ALL {
            assert_eq!(model(kind).pewter_guardian_pixels.is_some(), kind == Kind::TowerGuardian);
        }
        let json = include_str!("../models/dungeons/tower_guardian.mesh.json");
        for changed in [json.replace("\"head\"", "\"unknown\""),
                        json.replace("tower_guardian", "other"),
                        json.replace("\"pedestal\"", "\"unknown\"")] {
            assert!(Model::parse(changed.as_str()).unwrap().pewter_guardian_pixels.is_none());
        }
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        for field in [0, 1] {
            let mut changed = original.clone();
            changed["primitives"][0]["palette_regions"][0][field] = serde_json::json!(1);
            assert!(Model::parse(serde_json::to_string(&changed).unwrap().as_str())
                .unwrap().pewter_guardian_pixels.is_none());
        }
        let mut incomplete = original.clone();
        incomplete["primitives"][0]["palette_regions"].as_array_mut().unwrap().pop();
        assert!(Model::parse(serde_json::to_string(&incomplete).unwrap().as_str())
            .unwrap().pewter_guardian_pixels.is_none());
    }
    #[test]
    fn gym_stone_uses_named_live_palette_without_mutating_cave_geometry_or_colors() {
        let source = model(Kind::CaveBoulder);
        let cached = source.surface.clone();
        let mut cave = SurfaceMeshData::default();
        let mut gym = SurfaceMeshData::default();
        source.append(&mut cave, [-8., 8., -6., 6.], 0., 12.);
        source.append_gym_stone(&mut gym, [-8., 8., -6., 6.], 12., |pixel| {
            [pixel[0] as f32 / 16., pixel[1] as f32 / 16.]
        });
        assert_eq!(cave.positions, gym.positions);
        assert_eq!(cave.normals, gym.normals);
        assert_eq!(cave.indices, gym.indices);
        assert!(gym.uvs.contains(&[0.5, 0.375]));
        assert!(gym.uvs.contains(&[0.25, 0.25]));
        for (&normal, &color) in gym.normals.iter().zip(&gym.colors) {
            assert_eq!(color, crate::interior_models::authored_face_color(
                [1.; 4], Vec3::from_array(normal),
            ));
        }
        assert_eq!(source.surface, cached);
        for kind in Kind::ALL {
            assert_eq!(model(kind).gym_stone_pixels.is_some(), kind == Kind::CaveBoulder);
        }
        let json = include_str!("../models/dungeons/cave_boulder.mesh.json");
        for changed in [json.replace("cave_boulder", "other"),
                        json.replace("stone_light", "unknown")] {
            assert!(Model::parse(changed.as_str()).unwrap().gym_stone_pixels.is_none());
        }
    }
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
            assert!(
                m.surface
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4)
            );
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
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4)
            );
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
pub(crate) use extension::{ExtensionKind, extension_model};

#[path = "special_room_models.rs"]
mod special_rooms;
pub(crate) use special_rooms::{RoomAsset, room_model};

#[path = "gym_scenery_models.rs"]
mod gym_scenery;
pub(crate) use gym_scenery::{gym_model, gym_wall_join_triangles, gym_flower_planter_base};

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
            assert!(
                tall.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.0001)
            );
            assert!(
                tall.colors
                    .iter()
                    .all(|c| c[3] == 1. && c.iter().all(|v| v.is_finite()))
            );
        }
    }
}
