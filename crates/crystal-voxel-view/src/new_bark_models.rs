//! Original, authored low-poly meshes exported by the New Bark Blender kit.
//!
//! These meshes contain no game artwork or simulation data. The render-only
//! placement module maps them onto complete, source-identified map drawings.
use std::sync::OnceLock;

use bevy::prelude::Vec3;
use serde::Deserialize;

use crate::mesh::SurfaceMeshData;

/// The connected, source-verified outdoor art slice. Shared by scenery, actors,
/// and presentation settings; interiors and all other maps retain their views.
pub(crate) fn supports_map(map: &str) -> bool {
    matches!(
        map,
        "NewBarkTown" | "Route29" | "CherrygroveCity" | "Route30" | "Route31" | "VioletCity"
    )
}

/// Diagnostic A/B override only. Default uses authored lower-complexity
/// foliage in the outer source halo; no objects or source cells are removed.
pub(crate) fn scenery_full_detail() -> bool {
    static FULL: OnceLock<bool> = OnceLock::new();
    *FULL.get_or_init(|| std::env::var_os("CRYSTAL_SCENERY_FULL_DETAIL").is_some())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ModelKind {
    House,
    PlayerHouse,
    Lab,
    Pokecenter,
    Mart,
    RouteGate,
    TraditionalHouse,
    VioletGym,
    SproutTower,
    Grass,
    GrassLod,
    Tree,
    TreeLod,
    Flowers,
}

impl ModelKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::House => "johto/cottage",
            Self::PlayerHouse => "johto/player_house",
            Self::Lab => "johto/elm_lab",
            Self::Pokecenter => "johto/pokecenter",
            Self::Mart => "johto/mart",
            Self::RouteGate => "johto/route_gate",
            Self::TraditionalHouse => "johto/traditional_house",
            Self::VioletGym => "johto/violet_gym",
            Self::SproutTower => "johto/sprout_tower",
            Self::Grass | Self::GrassLod => "johto/tall_grass",
            Self::Tree | Self::TreeLod => "johto/tree",
            Self::Flowers => "johto/flowers",
        }
    }
}

#[derive(Deserialize)]
struct Primitive {
    #[serde(default)]
    material: String,
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
    #[serde(default)]
    uvs: Vec<f32>,
    texture: Option<crate::scenery_surfaces::EncodedTexture>,
}

#[derive(Deserialize)]
struct Export {
    #[serde(default)]
    name: String,
    primitives: Vec<Primitive>,
    door_anchor: Option<[f32; 3]>,
}

pub(crate) struct Model {
    textured: Vec<crate::scenery_surfaces::ScenerySurface>,
    surface: SurfaceMeshData,
    solid_materials: Vec<(String, std::ops::Range<usize>, [f32; 4])>,
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
    door_anchor: Option<[f32; 3]>,
}

impl Model {
    pub(crate) fn parse<'a>(
        json: impl Into<crate::model_storage::Source<'a>>,
    ) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(json)?;
        let mut surface = SurfaceMeshData::default();
        let mut solid_materials = Vec::new();
        let mut textured = Vec::new();
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for primitive in export.primitives {
            let vertex_count = primitive.positions.len() / 3;
            if vertex_count == 0
                || primitive.positions.len() % 3 != 0
                || primitive.normals.len() != primitive.positions.len()
                || primitive.indices.is_empty()
                || primitive.indices.len() % 3 != 0
                || primitive
                    .indices
                    .iter()
                    .any(|&i| i as usize >= vertex_count)
                || primitive
                    .positions
                    .iter()
                    .chain(&primitive.normals)
                    .any(|v| !v.is_finite())
                || primitive
                    .base_color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                return Err("invalid authored mesh primitive".into());
            }
            let texture = primitive
                .texture
                .map(crate::scenery_surfaces::SceneryTexture::decode)
                .transpose()?;
            if texture.is_some()
                && (primitive.uvs.len() != vertex_count * 2
                    || primitive.uvs.iter().any(|v| !v.is_finite()))
            {
                return Err("invalid external scenery UVs".into());
            }
            let mut part = SurfaceMeshData::default();
            let target = if texture.is_some() {
                &mut part
            } else {
                &mut surface
            };
            let base = target.positions.len() as u32;
            let index_start = target.indices.len();
            for position in primitive.positions.chunks_exact(3) {
                let position = [position[0], position[1], position[2]];
                for axis in 0..3 {
                    min[axis] = min[axis].min(position[axis]);
                    max[axis] = max[axis].max(position[axis]);
                }
                target.positions.push(position);
                target.uvs.push(if texture.is_some() {
                    let index = target.uvs.len() * 2;
                    [primitive.uvs[index], primitive.uvs[index + 1]]
                } else {
                    [0.0, 0.0]
                });
                target.colors.push(primitive.base_color);
            }
            for normal in primitive.normals.chunks_exact(3) {
                let normal = Vec3::new(normal[0], normal[1], normal[2]);
                if normal.length_squared() < 0.000_001 {
                    return Err("zero normal in authored mesh".into());
                }
                target.normals.push(normal.normalize().to_array());
            }
            target
                .indices
                .extend(primitive.indices.into_iter().map(|i| base + i));
            if texture.is_none()
                && !crate::roof_tiles::refine(target, base as usize, index_start, &primitive.material)
                && !(export.name == "traditional_house"
                    && crate::roof_tiles::refine_traditional(target, base as usize, index_start,
                        &primitive.material, primitive.base_color))
            {
                crate::scenery_finish::refine(
                    target,
                    base as usize,
                    index_start,
                    &primitive.material,
                    &export.name,
                );
            }
            if let Some(texture) = texture {
                textured.push(crate::scenery_surfaces::ScenerySurface {
                    mesh: part,
                    texture,
                });
            } else {
                solid_materials.push((primitive.material, base as usize..surface.positions.len(), primitive.base_color));
            }
        }
        crate::scenery_finish::contain(&mut surface, min, max);
        if (0..3).any(|axis| !min[axis].is_finite() || max[axis] <= min[axis]) {
            return Err("empty or flat authored model".into());
        }
        if export.door_anchor.is_some_and(|anchor| {
            anchor.iter().any(|v| !v.is_finite()) || anchor[0] <= min[0] || anchor[0] >= max[0]
        }) {
            return Err("invalid authored doorway anchor".into());
        }
        Ok(Self {
            textured,
            surface,
            solid_materials,
            min,
            max,
            door_anchor: export.door_anchor,
        })
    }

    pub(crate) fn append_textured_fitted(
        &self,
        groups: &mut Vec<crate::scenery_surfaces::ScenerySurface>,
        bounds: [f32; 4],
        base: f32,
        height: f32,
    ) {
        for part in &self.textured {
            let adapter = Self {
                surface: part.mesh.clone(),
                textured: Vec::new(),
                solid_materials: Vec::new(),
                min: self.min,
                max: self.max,
                door_anchor: self.door_anchor,
            };
            let mut mesh = SurfaceMeshData::default();
            adapter.append_fitted(&mut mesh, bounds, base, height, None);
            crate::scenery_surfaces::append_group(
                groups,
                crate::scenery_surfaces::ScenerySurface {
                    mesh,
                    texture: part.texture.clone(),
                },
            );
        }
    }

    pub(crate) fn surface_mesh(&self) -> SurfaceMeshData {
        self.surface.clone()
    }

    /// Recolor the existing named roof materials from the source drawing.
    /// Vertex ownership, fitted geometry and the existing facet/weather finish
    /// remain shared with every placement of this same cabin model.
    pub(crate) fn recolor_fitted_roof(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32; 4]; 3]) {
        // The source's high-contrast stripes describe one roof field. Keep
        // its hue while giving neighboring 3D courses quiet shade changes.
        let field: [f32;4] = std::array::from_fn(|i| palette[1][i]*0.7+palette[2][i]*0.3);
        let light: [f32;4] = std::array::from_fn(|i| field[i]*0.82+palette[2][i]*0.18);
        for (material, range, original) in &self.solid_materials {
            let color = match material.as_str() {
                "violet_dark" | "roof_red_dark" => palette[0],
                "violet" | "violet_warm" | "roof_red" | "roof_red_warm" => field,
                "violet_light" | "roof_red_light" => light,
                _ => continue,
            };
            for index in range.clone() {
                for channel in 0..3 {
                    target.colors[first+index][channel] = color[channel]
                        * target.colors[first+index][channel] / original[channel].max(0.0001);
                }
            }
        }
    }

    /// Keep the source plot's four edges fixed while aligning a recessed door
    /// and its threshold to the original off-center entrance. This deforms
    /// authored geometry, never source pixels. The inverse scale also updates
    /// normals, so the two sides remain correctly lit after the affine fit.
    pub(crate) fn append_fitted(
        &self,
        target: &mut SurfaceMeshData,
        bounds: [f32; 4], // west, east, north, south
        base_height: f32,
        height_scale: f32,
        door_x: Option<f32>,
    ) {
        let first_vertex = target.positions.len();
        let [west, east, north, south] = bounds;
        let z_scale = (south - north) / (self.max[2] - self.min[2]);
        let anchor = self
            .door_anchor
            .map(|p| p[0])
            .unwrap_or((self.min[0] + self.max[0]) * 0.5);
        let target_anchor = door_x.unwrap_or((west + east) * 0.5);
        self.append_transformed(target, |position, normal| {
            let (x, x_scale) = fit_horizontal(
                position[0],
                [self.min[0], self.max[0]],
                [west, east],
                anchor,
                target_anchor,
                door_x.is_some(),
            );
            let p = [
                x,
                base_height + (position[1] - self.min[1]) * height_scale,
                north + (position[2] - self.min[2]) * z_scale,
            ];
            let n = (Vec3::from_array(normal) / Vec3::new(x_scale, height_scale, z_scale))
                .normalize()
                .to_array();
            (p, n)
        });
        // The terrain shader preserves palette colors and only receives sun
        // visibility. Bake a gentle key/fill response for modeled facets.
        // Articulated actors have a separate rig importer and PBR materials.
        let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
        for index in first_vertex..target.positions.len() {
            let normal = Vec3::from_array(target.normals[index]);
            let shade = 0.62 + 0.38 * normal.dot(light).max(0.0);
            for channel in 0..3 {
                target.colors[index][channel] *= shade;
            }
        }
    }

    fn append_transformed(
        &self,
        target: &mut SurfaceMeshData,
        transform: impl Fn([f32; 3], [f32; 3]) -> ([f32; 3], [f32; 3]),
    ) {
        let base = target.positions.len() as u32;
        for (&position, &normal) in self.surface.positions.iter().zip(&self.surface.normals) {
            let (position, normal) = transform(position, normal);
            target.positions.push(position);
            target.normals.push(normal);
        }
        target.uvs.extend_from_slice(&self.surface.uvs);
        target.colors.extend_from_slice(&self.surface.colors);
        target
            .indices
            .extend(self.surface.indices.iter().map(|&i| base + i));
    }
}

/// Preserve a central doorway band at the model's natural width. Stretching
/// opposite sides directly at x=0 would move the geometric midpoint of a panel
/// spanning that seam, even though its mathematical anchor stayed correct.
fn fit_horizontal(
    x: f32,
    source: [f32; 2],
    target: [f32; 2],
    anchor: f32,
    target_anchor: f32,
    preserve_doorway: bool,
) -> (f32, f32) {
    let [min, max] = source;
    let [west, east] = target;
    let scale = (east - west) / (max - min);
    if !preserve_doorway {
        return (west + (x - min) * scale, scale);
    }
    let half_band = (max - min) * 0.15;
    let left = anchor - half_band;
    let right = anchor + half_band;
    let target_left = target_anchor - half_band * scale;
    let target_right = target_anchor + half_band * scale;
    if x < left {
        let scale = (target_left - west) / (left - min);
        (west + (x - min) * scale, scale)
    } else if x > right {
        let scale = (east - target_right) / (max - right);
        (target_right + (x - right) * scale, scale)
    } else {
        (target_anchor + (x - anchor) * scale, scale)
    }
}

pub(crate) fn required_tree_model(kind: ModelKind) -> Option<&'static Model> {
    static TREE: OnceLock<Option<Model>> = OnceLock::new();
    static TREE_LOD: OnceLock<Option<Model>> = OnceLock::new();
    let (cache, path) = match kind {
        ModelKind::Tree => (&TREE, "models/new_bark/tree.mesh.json"),
        ModelKind::TreeLod => (&TREE_LOD, "models/johto/tree_lod.mesh.json"),
        _ => return None,
    };
    cache.get_or_init(|| crate::open_models::load_required(path, |source| Model::parse(source))).as_ref()
}

pub(crate) fn model(kind: ModelKind) -> &'static Model {
    if matches!(kind, ModelKind::Tree | ModelKind::TreeLod) {
        return required_tree_model(kind).expect("required tree asset; renderer must retain 2D when missing");
    }
    static HOUSE: OnceLock<Model> = OnceLock::new();
    static PLAYER_HOUSE: OnceLock<Model> = OnceLock::new();
    static LAB: OnceLock<Model> = OnceLock::new();
    static POKECENTER: OnceLock<Model> = OnceLock::new();
    static MART: OnceLock<Model> = OnceLock::new();
    static ROUTE_GATE: OnceLock<Model> = OnceLock::new();
    static TRADITIONAL_HOUSE: OnceLock<Model> = OnceLock::new();
    static VIOLET_GYM: OnceLock<Model> = OnceLock::new();
    static SPROUT_TOWER: OnceLock<Model> = OnceLock::new();
    static GRASS: OnceLock<Model> = OnceLock::new();
    static GRASS_LOD: OnceLock<Model> = OnceLock::new();

    static FLOWERS: OnceLock<Model> = OnceLock::new();
    let (cache, json) = match kind {
        ModelKind::House => (
            &HOUSE,
            crate::model_storage::include_model!("models/new_bark/house.mesh.json"),
        ),
        ModelKind::PlayerHouse => (
            &PLAYER_HOUSE,
            crate::model_storage::include_model!("models/new_bark/player_house.mesh.json"),
        ),
        ModelKind::Lab => (
            &LAB,
            crate::model_storage::include_model!("models/new_bark/lab.mesh.json"),
        ),
        ModelKind::Pokecenter => (
            &POKECENTER,
            crate::model_storage::include_model!("models/johto/pokecenter.mesh.json"),
        ),
        ModelKind::Mart => (
            &MART,
            crate::model_storage::include_model!("models/johto/mart.mesh.json"),
        ),
        ModelKind::RouteGate => (
            &ROUTE_GATE,
            crate::model_storage::include_model!("models/johto/route_gate.mesh.json"),
        ),
        ModelKind::TraditionalHouse => (
            &TRADITIONAL_HOUSE,
            crate::model_storage::include_model!("models/johto/traditional_house.mesh.json"),
        ),
        ModelKind::VioletGym => (
            &VIOLET_GYM,
            crate::model_storage::include_model!("models/johto/violet_gym.mesh.json"),
        ),
        ModelKind::SproutTower => (
            &SPROUT_TOWER,
            crate::model_storage::include_model!("models/johto/sprout_tower.mesh.json"),
        ),
        ModelKind::Grass => (
            &GRASS,
            crate::model_storage::include_model!("models/johto/grass.mesh.json"),
        ),

        ModelKind::GrassLod => (
            &GRASS_LOD,
            crate::model_storage::include_model!("models/johto/grass_lod.mesh.json"),
        ),
        ModelKind::Tree | ModelKind::TreeLod => unreachable!("handled required assets above"),
        ModelKind::Flowers => (
            &FLOWERS,
            crate::model_storage::include_model!("models/new_bark/flowers.mesh.json"),
        ),
    };
    cache.get_or_init(|| Model::parse(json).expect("checked-in designated authored mesh must be valid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cabin_source_roof_color_preserves_shared_geometry_and_other_materials() {
        for source in [
            crate::model_storage::include_model!("models/johto/traditional_house.mesh.json"),
            crate::model_storage::include_model!("models/new_bark/house.mesh.json"),
        ] {
        let m=Model::parse(source).unwrap();
        let mut mesh=SurfaceMeshData::default();
        m.append_fitted(&mut mesh,[0.,32.,0.,24.],0.,16.,None);
        let original=mesh.clone();
        m.recolor_fitted_roof(&mut mesh,0,[[0.04,0.04,0.04,1.],[0.18,0.11,0.02,1.],[0.4,0.25,0.03,1.]]);
        assert_eq!(mesh.positions,original.positions);
        assert_eq!(mesh.normals,original.normals);
        assert_eq!(mesh.indices,original.indices);
        assert_eq!(mesh.uvs,original.uvs);
        let mut changed=0;
        for (material,range,_) in &m.solid_materials {
            if material.starts_with("violet") || material.starts_with("roof_red") {
                assert!(range.clone().any(|i|mesh.colors[i]!=original.colors[i]));
                changed+=range.len();
            } else {
                assert_eq!(&mesh.colors[range.clone()],&original.colors[range.clone()],"{material} is not roof paint");
            }
        }
        assert!(changed>0);
        }
    }

    #[test]
    fn textured_trees_preserve_uvs_fit_source_bounds_and_share_material_groups() {
        use base64::Engine as _;
        let fixture = serde_json::json!({"primitives":[{
            "positions":[0,0,0, 2,0,0, 0,3,0, 0,0,4],
            "normals":[0,1,0, 0,1,0, 0,1,0, 0,1,0],
            "indices":[0,2,1, 0,1,3, 0,3,2, 1,2,3],
            "base_color":[1,1,1,1], "uvs":[0,0, 1,0, 0,1, 1,1],
            "texture":{"width":1,"height":1,"rgba_base64":base64::engine::general_purpose::STANDARD.encode([255,255,255,128]),"alpha_cutoff":0.5,"double_sided":true}
        }]});
        let model = Model::parse(fixture.to_string().as_str()).unwrap();
        assert!(model.surface_mesh().positions.is_empty());
        let mut groups = Vec::new();
        model.append_textured_fitted(&mut groups, [10., 14., 20., 28.], 5., 2.);
        model.append_textured_fitted(&mut groups, [30., 34., 40., 48.], 6., 2.);
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].mesh.positions[..4],
            [
                [10., 5., 20.],
                [14., 5., 20.],
                [10., 11., 20.],
                [10., 5., 28.]
            ]
        );
        assert_eq!(
            groups[0].mesh.uvs[..4],
            [[0., 0.], [1., 0.], [0., 1.], [1., 1.]]
        );
        assert_eq!(
            groups[0].mesh.indices[12..],
            [4, 6, 5, 4, 5, 7, 4, 7, 6, 5, 6, 7]
        );
        assert!(groups[0].mesh.normals.iter().all(|n| *n == [0., 1., 0.]));
        let mut invalid = fixture;
        invalid["primitives"][0]["uvs"] = serde_json::json!([0, 0]);
        assert!(Model::parse(invalid.to_string().as_str()).is_err());
    }

    #[test]
    fn embedded_models_have_complete_valid_triangles() {
        for kind in [
            ModelKind::House,
            ModelKind::PlayerHouse,
            ModelKind::Lab,
            ModelKind::Pokecenter,
            ModelKind::Mart,
            ModelKind::RouteGate,
            ModelKind::TraditionalHouse,
            ModelKind::VioletGym,
            ModelKind::SproutTower,
            ModelKind::Grass,
            ModelKind::GrassLod,
            ModelKind::Tree,
            ModelKind::TreeLod,
            ModelKind::Flowers,
        ] {
            let model = model(kind);
            let surfaces: Vec<_> = std::iter::once(&model.surface)
                .chain(model.textured.iter().map(|part| &part.mesh)).collect();
            assert!(surfaces.iter().any(|part| !part.indices.is_empty()));
            for surface in surfaces {
            assert_eq!(surface.positions.len(), surface.normals.len());
            assert_eq!(surface.positions.len(), surface.colors.len());
            assert_eq!(surface.positions.len(), surface.uvs.len());
            assert!(
                surface
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < surface.positions.len())
            );
            assert!(
                surface
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.001)
            );
            }
        }
    }

    #[test]
    fn authored_grass_lod_keeps_exact_bounds_and_reduces_geometry_by_over_half() {
        for (full, lod) in [
            (ModelKind::Grass, ModelKind::GrassLod),
        ] {
            let a = model(full);
            let b = model(lod);
            assert_eq!(a.min, b.min);
            assert_eq!(a.max, b.max);
            let counts = |m: &Model| std::iter::once(&m.surface)
                .chain(m.textured.iter().map(|part| &part.mesh))
                .fold((0, 0), |(indices, vertices), part|
                    (indices + part.indices.len(), vertices + part.positions.len()));
            let (ai, av) = counts(a);
            let (bi, bv) = counts(b);
            assert!(bi * 2 < ai);
            assert!(bv * 2 < av);
        }
    }

    #[test]
    fn authored_door_and_plot_edges_fit_without_crossing_the_source_plot() {
        for (kind, width, door) in [
            (ModelKind::House, 64.0, 24.0),
            (ModelKind::Lab, 96.0, 40.0),
            (ModelKind::Pokecenter, 64.0, 24.0),
            (ModelKind::Mart, 64.0, 24.0),
            (ModelKind::RouteGate, 64.0, 24.0),
            (ModelKind::TraditionalHouse, 64.0, 24.0),
            (ModelKind::TraditionalHouse, 96.0, 40.0),
            (ModelKind::VioletGym, 96.0, 40.0),
            (ModelKind::SproutTower, 64.0, 24.0),
        ] {
            let model = model(kind);
            let mut mesh = SurfaceMeshData::default();
            model.append_fitted(
                &mut mesh,
                [10.0, 10.0 + width, 20.0, 84.0],
                0.0,
                16.0,
                Some(10.0 + door),
            );
            for p in &mesh.positions {
                assert!((10.0 - 0.001..=10.0 + width + 0.001).contains(&p[0]));
                assert!((20.0 - 0.001..=84.001).contains(&p[2]));
                assert!(p[1] >= -0.001);
            }
            let anchor = model
                .door_anchor
                .expect("building needs an explicit door threshold");
            assert!((anchor[2] - model.max[2]).abs() < 0.001);
            assert!(mesh.positions.iter().any(|p| (p[2] - 84.0).abs() < 0.001));
        }
    }

    #[test]
    fn connected_building_thresholds_reach_the_plot_edge_below_the_canopy() {
        for kind in [
            ModelKind::Pokecenter,
            ModelKind::Mart,
            ModelKind::RouteGate,
            ModelKind::TraditionalHouse,
            ModelKind::VioletGym,
            ModelKind::SproutTower,
        ] {
            let m = model(kind);
            let anchor = m.door_anchor.unwrap();
            assert!(
                m.surface
                    .positions
                    .iter()
                    .any(|p| (p[2] - anchor[2]).abs() < 0.001 && p[1] < 0.25 && p[0].abs() < 0.8),
                "{kind:?}: native threshold must end in a real low stone step, not an overhead eave"
            );
            assert!(m.max[1] - m.min[1] < 6.5);
        }
    }

    #[test]
    fn off_center_door_panels_keep_their_true_geometric_center() {
        for (extent, width, door, half_panel) in [(2.0, 64.0, 24.0, 0.345), (3.0, 96.0, 40.0, 0.53)]
        {
            let fit = |x| fit_horizontal(x, [-extent, extent], [0.0, width], 0.0, door, true).0;
            assert_eq!(fit(-extent), 0.0);
            assert!((fit(extent) - width).abs() < 0.001);
            assert_eq!(fit(0.0), door);
            assert!(((fit(-half_panel) + fit(half_panel)) * 0.5 - door).abs() < 0.001);
            assert!((fit(half_panel) - fit(-half_panel) - half_panel * 2.0 * 16.0).abs() < 0.001);
        }
    }

    #[test]
    fn importer_rejects_out_of_range_indices() {
        let json = r#"{"primitives":[{"positions":[0,0,0,1,0,0,0,1,0],"normals":[0,0,1,0,0,1,0,0,1],"indices":[0,1,3],"base_color":[1,1,1,1]}]}"#;
        assert!(Model::parse(json).is_err());
    }
}
