//! Original, authored low-poly meshes exported by the New Bark Blender kit.
//!
//! These meshes contain no game artwork or simulation data. The render-only
//! placement module maps them onto complete, source-identified map drawings.
use std::sync::OnceLock;

use bevy::prelude::{Vec2, Vec3};
use serde::Deserialize;

use crate::mesh::SurfaceMeshData;
mod town_glass;

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
    if cfg!(feature = "scenery-full-detail") {
        return true;
    }
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
    #[serde(default)]
    roof_courses: Option<String>,
    primitives: Vec<Primitive>,
    door_anchor: Option<[f32; 3]>,
}

pub(crate) struct Model {
    textured: Vec<crate::scenery_surfaces::ScenerySurface>,
    surface: SurfaceMeshData,
    solid_materials: Vec<(String, std::ops::Range<usize>, [f32; 4])>,
    glass_uvs: Vec<(usize, [f32; 2])>,
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
    door_anchor: Option<[f32; 3]>,
}

impl Model {
    pub(crate) fn parse<'a>(
        json: impl Into<crate::model_storage::Source<'a>>,
    ) -> Result<Self, String> {
        let _trace = crate::loading_trace::span("asset.decode_scenery_model", || serde_json::json!({}));
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
            let authored_courses=matches!(export.name.as_str(),"house"|"pokecenter"|"mart")
                && export.roof_courses.as_deref()==Some("closed_prism_v1")
                && primitive.material.starts_with("roof_") && !primitive.material.ends_with("_dark");
            if authored_courses {
                let sun=Vec3::new(0.45,0.8,0.55).normalize();
                for i in base as usize..target.positions.len() {
                    let light=0.60+0.40*Vec3::from_array(target.normals[i]).dot(sun).max(0.);
                    for channel in 0..3 {target.colors[i][channel]*=light;}
                }
            }
            if texture.is_none() && !authored_courses
                && !crate::roof_tiles::refine(target, base as usize, index_start, &primitive.material)
                && !(matches!(export.name.as_str(), "traditional_house" | "violet_gym")
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
        let glass_uvs = if matches!(export.name.as_str(), "house" | "pokecenter" | "mart") {
            solid_materials.iter().filter(|(name, _, _)| name == "glass")
                .flat_map(|(_, range, _)| town_glass::coordinates(&surface, range.clone())).collect()
        } else { Vec::new() };
        Ok(Self {
            textured,
            surface,
            solid_materials,
            glass_uvs,
            min,
            max,
            door_anchor: export.door_anchor,
        })
    }

    pub(crate) fn mark_city_lights(&self, target: &mut SurfaceMeshData, first: usize) {
        let center=(Vec3::from_array(self.min)+Vec3::from_array(self.max))*0.5;
        for (material,range,_) in &self.solid_materials {
            if !matches!(material.as_str(),"glass"|"glass_light") {continue;}
            crate::city_lights::mark_window(target,first+range.start..first+range.end,first,false);
            for i in range.clone() {
                if crate::scenery_finish::town_glass_coordinates(target.uvs[first+i]).is_some() {continue;}
                let outward=Vec3::from_array(self.surface.normals[i]).dot(
                    (Vec3::from_array(self.surface.positions[i])-center)*Vec3::new(1.,0.,1.))>0.;
                let base=if crate::city_lights::source_window_tag(target.uvs[first+i]) {
                    crate::city_lights::SOURCE_UV_BASE
                } else {crate::city_lights::UV_BASE};
                target.uvs[first+i][1]=base+if outward && material=="glass" {0.}else{0.25};
            }
        }
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
                glass_uvs: Vec::new(),
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
                "violet_dark" | "roof_red_dark" | "roof_blue_dark" => palette[0],
                "violet" | "violet_warm" | "roof_red" | "roof_red_warm" | "roof_blue" | "roof_blue_warm" => field,
                "violet_light" | "roof_red_light" | "roof_blue_light" => light,
                _ => continue,
            };
            for index in range.clone() {
                for channel in 0..3 {
                    target.colors[first+index][channel] = color[channel]
                        * target.colors[first+index][channel] / original[channel].max(0.0001);
                }
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }

    /// Painted town roles carry authored local material coordinates/daylight.
    /// Live glass gains pane coordinates; doors, timber and unpainted faces keep their tags.
    pub(crate) fn light_fitted_source_paint(&self, target: &mut SurfaceMeshData, first: usize) {
        for (material,range,_) in &self.solid_materials {
            let kind=if material.starts_with("roof_") && !material.ends_with("_dark")
                || matches!(material.as_str(),"violet"|"violet_warm"|"violet_light") {0.}
                else {match material.as_str() {
                    "plaster"|"plaster_light"=>1.,"plaster_service_wall"=>2.,
                    "plaster_service_mortar"=>3.,_=>continue,
                }};
            for i in range.clone() {
                if target.uvs[first+i]==crate::scenery_finish::SOURCE_PAINT_UV
                    || crate::scenery_finish::town_paint_kind(target.uvs[first+i]).is_some() {
                    let p=self.surface.positions[i];let n=self.surface.normals[i];
                    let (u,v)=if n[1].abs()>0.5 {(p[0],p[2])}
                        else if n[0].abs()>n[2].abs() {(p[2],p[1])}else {(p[0],p[1])};
                    target.uvs[first+i]=[u,crate::scenery_finish::TOWN_PAINT_BASE+kind*32.+v];
                }
            }
        }
        for &(i, uv) in &self.glass_uvs {
            if target.uvs[first+i] == crate::scenery_finish::SOURCE_PAINT_UV
                || crate::scenery_finish::town_glass_coordinates(target.uvs[first+i]).is_some() {
                target.uvs[first+i] = uv;
            }
        }
    }

    /// Compact plaster and its low ochre sill follow the complete native wall.
    /// Model-local height distinguishes that sill from gables/chimney mortar.
    pub(crate) fn recolor_fitted_compact_wall(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];2]) {
        for (material,range,original) in &self.solid_materials {
            if !matches!(material.as_str(),"plaster"|"plaster_light") {continue;}
            for index in range.clone() {
                let color=palette[usize::from(material=="plaster_light"
                    && self.surface.positions[index][1]<0.5)];
                let old=target.colors[first+index];
                let shade=(old[0]+old[1]+old[2])/(original[0]+original[1]+original[2]);
                for channel in 0..3 {target.colors[first+index][channel]=color[channel]*shade;}
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }

    pub(crate) fn recolor_fitted_service_wall(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];2]) {
        for (material,range,original) in &self.solid_materials {
            let color=match material.as_str() {
                "plaster_service_wall"=>palette[0],
                "plaster_service_mortar"=>palette[1],
                _=>continue,
            };
            for index in range.clone() {
                let old=target.colors[first+index];
                let shade=(old[0]+old[1]+old[2])/(original[0]+original[1]+original[2]);
                for channel in 0..3 {target.colors[first+index][channel]=color[channel]*shade;}
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
            }
        }
    }

    /// Glazing and service door leaves share their live entrance paint.
    /// Structural timber, jambs, roof courses and fittings retain their roles.
    pub(crate) fn recolor_fitted_entrance(&self, target: &mut SurfaceMeshData,
        first: usize, palette: [[f32;4];2]) {
        for (material,range,original) in &self.solid_materials {
            let (paint,gain)=match material.as_str() {
                "glass" => (palette[0],0.84),
                "glass_light" => (palette[0],1.),
                "roof_teal_dark" => (palette[1],1.),
                _=>continue,
            };
            for index in range.clone() {
                let old=target.colors[first+index];
                let shade=(old[0]+old[1]+old[2])/(original[0]+original[1]+original[2]).max(0.0001);
                for channel in 0..3 {target.colors[first+index][channel]=paint[channel]*shade*gain;}
                target.uvs[first+index]=crate::scenery_finish::SOURCE_PAINT_UV;
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

pub(crate) fn model(kind: ModelKind) -> &'static Model {
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
    fn civic_reference_identity_is_separate_from_live_roof_and_door_paint() {
        for (kind,role) in [(ModelKind::Pokecenter,"medical_red"),(ModelKind::Mart,"shop_sign_blue")] {
            let m=model(kind);let mut painted=m.surface.clone();
            m.recolor_fitted_roof(&mut painted,0,[[0.01,0.01,0.01,1.],[0.,0.6,0.9,1.],[0.4,1.,1.,1.]]);
            m.recolor_fitted_entrance(&mut painted,0,[[0.9,1.,0.3,1.],[0.5,0.2,0.02,1.]]);
            let mut count=0;
            for (name,range,_) in &m.solid_materials {
                if name==role || name=="medical_white" {
                    count+=range.len();
                    assert_eq!(&painted.colors[range.clone()],&m.surface.colors[range.clone()]);
                }
            }
            assert!(count>100,"the raised identity must be real geometry");
            assert_eq!(painted.positions,m.surface.positions);
            assert_eq!(painted.indices,m.surface.indices);
            assert_eq!(m.door_anchor,Some([0.,0.,2.]));
        }
    }

    #[test]
    fn compact_house_visible_end_caps_keep_wall_material_when_roof_paint_changes() {
        let m=model(ModelKind::House);
        let mut sides=[false;2];
        for (material,range,_) in &m.solid_materials {
            if !matches!(material.as_str(),"plaster_light"|"roof_red_dark") {continue;}
            for i in range.clone() {
                let p=m.surface.positions[i];let n=m.surface.normals[i];
                if p[0].abs()>1.9 && p[1]>=2.17999 && n[0].abs()>0.99 {
                    assert_eq!(material,"plaster_light","source outline black must not paint a visible gable wall");
                    assert!(n[0]*p[0]>0.);
                    sides[usize::from(p[0]>0.)]=true;
                }
            }
        }
        assert_eq!(sides,[true,true]);
    }

    #[test]
    fn civic_brick_faces_have_shallow_relief_over_separate_mortar_backing() {
        for kind in [ModelKind::Pokecenter,ModelKind::Mart] {
            let m=model(kind);
            let mut face_z=f32::NEG_INFINITY;let mut mortar_z=f32::NEG_INFINITY;
            for (material,range,_) in &m.solid_materials {
                if !matches!(material.as_str(),"plaster_service_wall"|"plaster_service_mortar") {continue;}
                for i in range.clone() {
                    let p=m.surface.positions[i];let n=m.surface.normals[i];
                    if n[2]>0.99 {
                        if material=="plaster_service_wall" && (0.67..2.23).contains(&p[1]) {
                            face_z=face_z.max(p[2]);
                        } else if material=="plaster_service_mortar" {
                            // A continuous backing quad has vertices only at
                            // its outer corners, not at each brick-course height.
                            mortar_z=mortar_z.max(p[2]);
                        }
                    }
                }
            }
            assert!(face_z.is_finite() && mortar_z.is_finite());
            assert!((0.01..0.05).contains(&(face_z-mortar_z)),"{kind:?} brick must have restrained physical relief");
            assert_eq!(m.min,[-2.,0.,-2.]);
            assert_eq!(m.max,[2.,3.448,2.]);
            assert_eq!(m.door_anchor,Some([0.,0.,2.]));
        }
    }

    #[test]
    fn civic_running_bond_has_fine_body_courses_and_brick_gables_inside_the_original_roof() {
        for source in [crate::model_storage::include_model!("models/johto/pokecenter.mesh.json"),
            crate::model_storage::include_model!("models/johto/mart.mesh.json")] {
            let export: Export = crate::model_storage::parse(source).unwrap();
            let mut backing = [None, None];
            for part in &export.primitives {
                if part.material != "plaster_service_mortar" { continue; }
                for ids in part.indices.chunks_exact(3) {
                    let p = [ids[0],ids[1],ids[2]].map(|i|Vec3::from_slice(&part.positions[i as usize*3..]));
                    if p.iter().all(|v|v.y>=2.45999 && v.x.abs()>1.9)
                        && (p[1]-p[0]).cross(p[2]-p[0]).length()>1. {
                        backing[usize::from(p[0].x>0.)] = Some(p.map(|v|Vec2::new(v.y,v.z)));
                    }
                }
            }
            assert!(backing.iter().all(Option::is_some),"continuous original gable backing");
            let mut gable_faces=[0;2]; let mut body_faces=0;
            for part in &export.primitives {
                if part.material != "plaster_service_wall" { continue; }
                for ids in part.indices.chunks_exact(3) {
                    let p = [ids[0],ids[1],ids[2]].map(|i|Vec3::from_slice(&part.positions[i as usize*3..]));
                    let n = Vec3::from_slice(&part.normals[ids[0] as usize*3..]);
                    if n.z>0.99 && p.iter().all(|v|(0.66..2.23).contains(&v.y)) {body_faces+=1;}
                    if n.x.abs()<0.99 || !p.iter().all(|v|v.y>=2.46 && v.x.abs()>1.98) {continue;}
                    assert!(n.x*p[0].x>0.,"only the exposed brick front reaches this plane");
                    let side=usize::from(p[0].x>0.);gable_faces[side]+=1;
                    let triangle=backing[side].unwrap();
                    for point in p {
                        let point=Vec2::new(point.y,point.z);
                        for e in 0..3 {
                            let a=triangle[e];let b=triangle[(e+1)%3];let c=triangle[(e+2)%3];
                            assert!((b-a).perp_dot(point-a)*(b-a).perp_dot(c-a)>=-0.00001,
                                "brick must remain inside original gable triangle");
                        }
                    }
                }
            }
            assert!(body_faces>=200,"fine body courses, not the old oversized bricks");
            assert!(gable_faces.iter().all(|&count|count>=40),"resolved masonry on both gables");
        }
    }

    #[test]
    fn service_gable_end_walls_face_outward_and_share_source_masonry() {
        for kind in [ModelKind::Pokecenter,ModelKind::Mart] {
            let m=model(kind);
            let mut sides=[false;2];
            for (material,range,_) in &m.solid_materials {
                // Fascia/roof timber has legitimate inward-facing sides of
                // closed beams at this height. Only wall roles are end caps.
                if !matches!(material.as_str(),"plaster_service_wall"|"plaster_service_mortar"|"plaster"|"roof_red_dark"|"roof_blue_dark") {continue;}
                for i in range.clone() {
                    let p=m.surface.positions[i];let n=m.surface.normals[i];
                    if p[1]>=2.41999 && p[0].abs()>1.7 && n[0].abs()>0.99 {
                        if n[0]*p[0]<0. {
                            // Closed brick backs legitimately face inward,
                            // behind the exposed end wall and its bevels.
                            assert_eq!(material,"plaster_service_wall");
                            assert!(p[0].abs()<1.98,"inward face exposed at the gable crest: {p:?}");
                        } else {
                            assert!(matches!(material.as_str(),"plaster_service_wall"|"plaster_service_mortar"),"gable must share source masonry paint");
                            sides[usize::from(p[0]>0.)]=true;
                        }
                    }
                }
            }
            assert_eq!(sides,[true,true],"both authored gable ends must close {kind:?}");
        }
    }

    #[test]
    fn entrance_paint_preserves_geometry_and_non_glass_structural_materials() {
        for kind in [ModelKind::House,ModelKind::Pokecenter,ModelKind::Mart] {
            let m=model(kind);let mut mesh=SurfaceMeshData::default();
            m.append_fitted(&mut mesh,[0.,64.,0.,64.],0.,16.,Some(24.));
            let original=mesh.clone();
            m.recolor_fitted_entrance(&mut mesh,0,[[0.9,0.85,0.08,1.],[0.3,0.04,0.2,1.]]);
            assert_eq!(mesh.positions,original.positions);
            assert_eq!(mesh.normals,original.normals);
            assert_eq!(mesh.indices,original.indices);
            let (mut glass,mut doors)=(0,0);
            for (material,range,_) in &m.solid_materials {
                match material.as_str() {
                    "glass"|"glass_light" => {
                        glass+=range.len();
                        assert!(mesh.colors[range.clone()].iter().all(|c|c[0]>c[2]*5. && c[1]>c[2]*5.));
                        assert!(mesh.uvs[range.clone()].iter().all(|uv|*uv==crate::scenery_finish::SOURCE_PAINT_UV));
                    },
                    "roof_teal_dark" => {
                        doors+=range.len();
                        assert!(mesh.colors[range.clone()].iter().all(|c|c[0]>c[1]*4. && c[2]>c[1]*3.));
                    },
                    _ => {
                        assert_eq!(mesh.colors[range.clone()],original.colors[range.clone()]);
                        assert_eq!(mesh.uvs[range.clone()],original.uvs[range.clone()]);
                    },
                }
            }
            assert!(glass>0);
            if kind!=ModelKind::House {assert!(doors>0);}
        }
    }

    #[test]
    fn service_source_paint_preserves_trim_geometry_and_door_alignment() {
        for kind in [ModelKind::Pokecenter,ModelKind::Mart] {
            let m=model(kind);
            let mut mesh=SurfaceMeshData::default();
            m.append_fitted(&mut mesh,[0.,64.,0.,64.],0.,16.,Some(24.));
            let original=mesh.clone();
            m.recolor_fitted_roof(&mut mesh,0,[[0.01,0.02,0.05,1.],
                [0.05,0.4,0.5,1.],[0.2,0.8,0.9,1.]]);
            m.recolor_fitted_service_wall(&mut mesh,0,[[0.8,0.2,0.4,1.],[0.7,0.75,0.65,1.]]);
            assert_eq!(mesh.positions,original.positions);
            assert_eq!(mesh.normals,original.normals);
            assert_eq!(mesh.indices,original.indices);
            let (mut roofs,mut walls)=(0,0);
            for (material,range,_) in &m.solid_materials {
                if material.starts_with("roof_red")||material.starts_with("roof_blue") {
                    for c in &mesh.colors[range.clone()] {assert!(c[2]>c[0],"source cyan roof: {c:?}");}
                    assert!(mesh.uvs[range.clone()].iter().all(|uv|*uv==crate::scenery_finish::SOURCE_PAINT_UV));
                    roofs+=range.len();
                } else if material=="plaster_service_wall" {
                    for c in &mesh.colors[range.clone()] {assert!(c[0]>c[2] && c[2]>c[1],"source pink wall: {c:?}");}
                    assert!(mesh.uvs[range.clone()].iter().all(|uv|*uv==crate::scenery_finish::SOURCE_PAINT_UV));
                    walls+=range.len();
                } else if material=="plaster_service_mortar" {
                    for c in &mesh.colors[range.clone()] {assert!(c[1]>c[0] && c[0]>c[2],"source mortar: {c:?}");}
                    assert!(mesh.uvs[range.clone()].iter().all(|uv|*uv==crate::scenery_finish::SOURCE_PAINT_UV));
                } else {
                    assert_eq!(&mesh.colors[range.clone()],&original.colors[range.clone()],
                        "service paint must preserve {material}");
                    assert_eq!(&mesh.uvs[range.clone()],&original.uvs[range.clone()]);
                }
            }
            assert!(roofs>0 && walls>0);
        }
    }

    #[test]
    fn cabin_source_roof_color_preserves_shared_geometry_and_other_materials() {
        for source in [
            crate::model_storage::include_model!("models/johto/traditional_house.mesh.json"),
            crate::model_storage::include_model!("models/new_bark/house.mesh.json"),
            crate::model_storage::include_model!("models/new_bark/player_house.mesh.json"),
            crate::model_storage::include_model!("models/johto/pokecenter.mesh.json"),
            crate::model_storage::include_model!("models/johto/mart.mesh.json"),
            crate::model_storage::include_model!("models/johto/violet_gym.mesh.json"),
        ] {
        let m=Model::parse(source).unwrap();
        let mut mesh=SurfaceMeshData::default();
        m.append_fitted(&mut mesh,[0.,32.,0.,24.],0.,16.,None);
        let original=mesh.clone();
        m.recolor_fitted_roof(&mut mesh,0,[[0.04,0.04,0.04,1.],[0.18,0.11,0.02,1.],[0.4,0.25,0.03,1.]]);
        assert_eq!(mesh.positions,original.positions);
        assert_eq!(mesh.normals,original.normals);
        assert_eq!(mesh.indices,original.indices);
        let mut changed=0;
        for (material,range,_) in &m.solid_materials {
            if material.starts_with("violet") || material.starts_with("roof_red") || material.starts_with("roof_blue") {
                assert!(range.clone().any(|i|mesh.colors[i]!=original.colors[i]));
                changed+=range.len();
                assert!(mesh.uvs[range.clone()].iter().all(|uv|*uv==crate::scenery_finish::SOURCE_PAINT_UV));
            } else {
                assert_eq!(&mesh.colors[range.clone()],&original.colors[range.clone()],"{material} is not roof paint");
                assert_eq!(&mesh.uvs[range.clone()],&original.uvs[range.clone()]);
            }
        }
        assert!(changed>0);
        }
    }

    #[test]
    fn town_daylight_tags_only_live_painted_roles_without_mutating_paint() {
        for kind in [ModelKind::House,ModelKind::Pokecenter,ModelKind::Mart] {
            let m=model(kind);let mut mesh=m.surface_mesh();
            m.recolor_fitted_roof(&mut mesh,0,[[0.03,0.04,0.08,1.],[0.1,0.4,0.8,1.],[0.4,0.8,1.,1.]]);
            if kind==ModelKind::House {
                m.recolor_fitted_compact_wall(&mut mesh,0,[[0.7,0.9,0.7,1.],[0.5,0.3,0.04,1.]]);
            } else {m.recolor_fitted_service_wall(&mut mesh,0,[[0.8,0.3,0.5,1.],[0.7,0.8,0.7,1.]]);}
            m.recolor_fitted_entrance(&mut mesh,0,[[0.9,0.9,0.1,1.],[0.3,0.1,0.2,1.]]);
            let before=mesh.clone();m.light_fitted_source_paint(&mut mesh,0);
            assert_eq!(mesh.positions,before.positions);assert_eq!(mesh.normals,before.normals);
            assert_eq!(mesh.indices,before.indices);assert_eq!(mesh.colors,before.colors);
            let mut painted=0;let mut glazing=0;
            for (name,range,_) in &m.solid_materials {for i in range.clone() {
                if crate::scenery_finish::town_paint_kind(mesh.uvs[i]).is_some() {
                    assert_eq!(before.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                    assert!(name.starts_with("roof_") && !name.ends_with("_dark") || name.starts_with("plaster"));
                    painted+=1;
                } else if crate::scenery_finish::town_glass_coordinates(mesh.uvs[i]).is_some() {
                    assert_eq!(name,"glass");
                    assert_eq!(before.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                    glazing+=1;
                } else {
                    assert_eq!(mesh.uvs[i],before.uvs[i]);
                    assert_ne!(name,"glass","every authored source-painted pane needs local coordinates");
                    if name=="glass_light" {
                        assert_eq!(mesh.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);glazing+=1;
                    }
                }
            }}
            assert!(painted>0 && glazing>0);
            let mut unpainted=m.surface_mesh();let before=unpainted.clone();
            m.light_fitted_source_paint(&mut unpainted,0);assert_eq!(unpainted,before);
        }
    }

    #[test]
    fn town_material_coordinates_survive_translation_and_nonuniform_fitting() {
        for kind in [ModelKind::House,ModelKind::Pokecenter,ModelKind::Mart] {
            let m=model(kind);let mut fitted=Vec::new();
            for bounds in [[0.,32.,0.,24.],[128.,192.,-80.,-32.]] {
                let mut mesh=SurfaceMeshData::default();
                m.append_fitted(&mut mesh,bounds,0.,16.,None);
                m.recolor_fitted_roof(&mut mesh,0,[[0.04,0.04,0.04,1.],[0.1,0.4,0.8,1.],[0.4,0.8,1.,1.]]);
                if kind==ModelKind::House {
                    m.recolor_fitted_compact_wall(&mut mesh,0,[[0.7,0.9,0.7,1.],[0.5,0.3,0.04,1.]]);
                } else {m.recolor_fitted_service_wall(&mut mesh,0,[[0.8,0.3,0.5,1.],[0.7,0.8,0.7,1.]]);}
                m.recolor_fitted_entrance(&mut mesh,0,[[0.7,0.9,0.7,1.],[0.5,0.3,0.04,1.]]);
                m.light_fitted_source_paint(&mut mesh,0);fitted.push(mesh);
            }
            assert_ne!(fitted[0].positions,fitted[1].positions);
            assert_eq!(fitted[0].uvs,fitted[1].uvs,"grain must stay attached to authored surfaces");
            let coords=fitted[0].uvs.iter().filter(|&&uv|crate::scenery_finish::town_paint_kind(uv).is_some())
                .map(|uv|uv.map(f32::to_bits)).collect::<std::collections::BTreeSet<_>>();
            assert!(coords.len()>100,"material coordinates cannot be a constant style flag");
        }
    }

    #[test]
    fn compact_source_plaster_preserves_joinery_roof_and_geometry() {
        let m=model(ModelKind::House);let original=m.surface_mesh();let mut painted=original.clone();
        let palette=[[0.72,0.95,0.72,1.],[0.56,0.30,0.04,1.]];
        m.recolor_fitted_compact_wall(&mut painted,0,palette);
        assert_eq!(painted.positions,original.positions);
        assert_eq!(painted.normals,original.normals);
        assert_eq!(painted.indices,original.indices);
        let mut sill=0;let mut plaster=0;
        for (name,range,_) in &m.solid_materials {
            for i in range.clone() {
                if matches!(name.as_str(),"plaster"|"plaster_light") {
                    assert_eq!(painted.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                    let c=painted.colors[i];
                    if name=="plaster_light" && original.positions[i][1]<0.5 {
                        assert!(c[0]>c[1] && c[1]>c[2]);sill+=1;
                    } else {assert!(c[1]>c[0] && (c[0]-c[2]).abs()<1e-6);plaster+=1;}
                } else {
                    assert_eq!(painted.colors[i],original.colors[i],"changed {name}");
                    assert_eq!(painted.uvs[i],original.uvs[i],"changed {name}");
                }
            }
        }
        assert!(sill>0 && plaster>0);
    }

    #[test]
    fn closed_town_roof_courses_keep_small_scale_bounds_and_raw_triangle_budget() {
        for (source,depth,top,eave,ridge) in [
            (crate::model_storage::include_model!("models/new_bark/house.mesh.json"),3.,3.642,2.18,3.29),
            (crate::model_storage::include_model!("models/johto/pokecenter.mesh.json"),4.,3.448,2.46,3.35),
            (crate::model_storage::include_model!("models/johto/mart.mesh.json"),4.,3.448,2.46,3.35),
            (crate::model_storage::include_model!("models/johto/route_gate.mesh.json"),4.,3.448,2.46,3.35),
        ] {
            let export:Export=crate::model_storage::parse(source).unwrap();
            assert_eq!(export.roof_courses.as_deref(),Some("closed_prism_v1"));
            let m=Model::parse(source).unwrap();
            assert_eq!(m.min,[-2.,0.,-depth/2.]);
            assert_eq!(m.max,[2.,top,depth/2.]);
            assert_eq!(m.door_anchor,Some([0.,0.,depth/2.]));
            let mut fine_faces=0;
            for p in &export.primitives {
                if !p.material.starts_with("roof_") || p.material.ends_with("_dark") {continue;}
                let (_,range,_)=m.solid_materials.iter().find(|(name,_,_)|name.as_str()==p.material.as_str()).unwrap();
                let rendered=m.surface.indices.chunks_exact(3).filter(|tri|range.contains(&(tri[0] as usize))).count();
                assert_eq!(rendered,p.indices.len()/3,"authored ceramics must not be re-tessellated");
                for tri in p.indices.chunks_exact(3) {
                    let q=[tri[0],tri[1],tri[2]].map(|i|Vec3::from_slice(&p.positions[i as usize*3..]));
                    let face=(q[1]-q[0]).cross(q[2]-q[0]);
                    if q.iter().all(|v|v.y>eave && v.y<ridge-0.05)
                        && (0.35..0.98).contains(&face.normalize_or_zero().y) {
                        assert!(face.length()*0.5<0.055,"coarse roof panel remains on {}",export.name);
                        fine_faces+=1;
                    }
                }
            }
            assert!(fine_faces>900,"both slopes need small ceramic courses");
            assert!(export.primitives.iter().map(|p|p.indices.len()/3).sum::<usize>()<12000);
        }
    }

    #[test]
    fn refined_roof_panels_stay_above_the_structural_deck() {
        for source in [
            crate::model_storage::include_model!("models/new_bark/house.mesh.json"),
            crate::model_storage::include_model!("models/johto/pokecenter.mesh.json"),
            crate::model_storage::include_model!("models/johto/mart.mesh.json"),
            crate::model_storage::include_model!("models/johto/violet_gym.mesh.json"),
        ] {
            let export:Export=crate::model_storage::parse(source).unwrap();
            let model=Model::parse(source).unwrap();
            let mut eave=f32::INFINITY;
            for ids in model.surface.indices.chunks_exact(3) {
                let Some((material,_,_))=model.solid_materials.iter().find(|(_,range,_)|
                    range.contains(&(ids[0] as usize))) else {continue;};
                if !material.ends_with("_dark") {continue;}
                let p=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(model.surface.positions[i as usize]));
                let face=(p[1]-p[0]).cross(p[2]-p[0]);
                if face.length()*0.5>=0.8 && (0.35..0.98).contains(&face.normalize_or_zero().y) {
                    for point in p {eave=eave.min(point.y);}
                }
            }
            assert!(eave.is_finite(),"{} has no pitched structural deck",export.name);
            let mut sampled=0;
            for primitive in &export.primitives {
                if !(primitive.material.starts_with("roof_") || primitive.material.starts_with("violet"))
                    || primitive.material.ends_with("_dark") {continue;}
                for ids in primitive.indices.chunks_exact(3) {
                    let normal=Vec3::from_slice(&primitive.normals[ids[0] as usize*3..]);
                    if !(0.35..0.98).contains(&normal.y) {continue;}
                    let corners=[ids[0],ids[1],ids[2]].map(|i|
                        Vec3::from_slice(&primitive.positions[i as usize*3..]));
                    // Entry shelters share the roof paint but sit below it.
                    if corners.iter().any(|point|point.y<eave-0.00001) {continue;}
                    let area=(corners[1]-corners[0]).cross(corners[2]-corners[0]).length()*0.5;
                    if !(0.01..0.4).contains(&area) {continue;}
                    let center=corners.into_iter().sum::<Vec3>()/3.;
                    let q=Vec2::new(center.x,center.z);
                    let mut nearest:Option<(f32,&str)>=None;
                    for triangle in model.surface.indices.chunks_exact(3) {
                        let Some((material,_,_))=model.solid_materials.iter().find(|(_,range,_)|
                            range.contains(&(triangle[0] as usize))) else {continue;};
                        if !(material.starts_with("roof_") || material.starts_with("violet")) {continue;}
                        let p=[triangle[0],triangle[1],triangle[2]].map(|i|
                            Vec3::from_array(model.surface.positions[i as usize]));
                        let face=(p[1]-p[0]).cross(p[2]-p[0]);
                        // Vents and ridge caps legitimately sit above tiles.
                        // Only the broad pitched dark faces are the deck.
                        if material.ends_with("_dark")
                            && (face.length()*0.5<0.8 || !(0.35..0.98).contains(&face.normalize_or_zero().y)) {continue;}
                        let a=Vec2::new(p[0].x,p[0].z);
                        let ab=Vec2::new(p[1].x,p[1].z)-a;
                        let ac=Vec2::new(p[2].x,p[2].z)-a;
                        let det=ab.perp_dot(ac);
                        if det.abs()<0.000001 {continue;}
                        let u=(q-a).perp_dot(ac)/det;
                        let v=ab.perp_dot(q-a)/det;
                        if u< -0.00001 || v< -0.00001 || u+v>1.00001 {continue;}
                        let height=p[0].y+u*(p[1].y-p[0].y)+v*(p[2].y-p[0].y);
                        if nearest.is_none_or(|(old,_)|height>old) {nearest=Some((height,material));}
                    }
                    let (height,material)=nearest.expect("authored roof panel disappeared");
                    assert!(!material.ends_with("_dark") && height>=center.y-0.00001,
                        "{} panel at {center:?} lies below {material} at {height}",export.name);
                    sampled+=1;
                }
            }
            assert!(sampled>=40,"{} needs coverage of both roof slopes",export.name);
        }
    }
    #[test]
    fn textured_models_preserve_uvs_fit_source_bounds_and_share_material_groups() {
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
