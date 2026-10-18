//! Individual woodland plants scattered through the union of native tree plots.
//! Plant geometry is CC0 Quaternius artwork; no tree geometry lives here.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::{Mat3, Quat, Transform, Vec3};
use std::sync::OnceLock;

pub(crate) const SPECIES: usize = 4;
pub(crate) const VARIANTS: usize = SPECIES * 3;
pub(crate) const MAX_VERTICES: usize = 20_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plot {
    pub bounds: [f32; 4],
    pub cells: [u8; 2],
    /// Six by six source cells, including a one-cell halo around a 4x4 plot.
    /// Bit (x+1)+(z+1)*6 identifies a recognized plant-bearing tree cell.
    pub occupied: u64,
}
impl Plot {
    pub fn isolated(bounds: [f32; 4]) -> Self {
        let mut result = Self {
            bounds,
            cells: [2, 2],
            occupied: 0,
        };
        for z in 0..2 {
            for x in 0..2 {
                result.occupied |= 1 << ((x + 1) + (z + 1) * 6);
            }
        }
        result
    }
    fn cell(&self) -> [f32; 2] {
        [
            (self.bounds[1] - self.bounds[0]) / self.cells[0] as f32,
            (self.bounds[3] - self.bounds[2]) / self.cells[1] as f32,
        ]
    }
    fn contains(&self, x: i32, z: i32) -> bool {
        (-1..=4).contains(&x)
            && (-1..=4).contains(&z)
            && self.occupied & (1 << ((x + 1) + (z + 1) * 6)) != 0
    }
    /// Clearance to the union's exposed boundary, never an internal plot seam.
    fn clearance(&self, x: f32, z: f32) -> f32 {
        let [w, d] = self.cell();
        let mut distance = w.min(d) * 2.;
        for iz in -1..=self.cells[1] as i32 {
            for ix in -1..=self.cells[0] as i32 {
                if self.contains(ix, iz) {
                    continue;
                }
                let dx = (ix as f32 * w - x).max(x - (ix + 1) as f32 * w).max(0.);
                let dz = (iz as f32 * d - z).max(z - (iz + 1) as f32 * d).max(0.);
                distance = distance.min(dx.hypot(dz));
            }
        }
        distance
    }
}
fn seed(x: i32, z: i32, k: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b)
        ^ k.wrapping_mul(0x7feb352d);
    n ^= n >> 16;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    (n & 65535) as f32 / 65535.
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plant {
    pub variant: usize,
    pub transform: Transform,
}

/// The random field is anchored to source cells, not tree centers or chunks.
/// No border fronds, flat mats, rectangular clipping or stretched prototypes.
pub(crate) fn plants(plot: Plot, base: f32, coordinates: [i32; 2]) -> Vec<Plant> {
    if plot.cells.contains(&0) || plot.cells.iter().any(|&n| n > 4) {
        return Vec::new();
    }
    let [w, d] = plot.cell();
    let unit = w.min(d);
    if unit <= 0. {
        return Vec::new();
    }
    let mut result = Vec::new();
    for iz in 0..plot.cells[1] as i32 {
        for ix in 0..plot.cells[0] as i32 {
            let (x, z) = (coordinates[0] + ix, coordinates[1] + iz);
            for layer in 0..2 {
                let k = layer * 43;
                if seed(x, z, k) > if layer == 0 { 0.94 } else { 0.27 } {
                    continue;
                }
                let u = 0.08 + seed(x, z, k + 1) * 0.84;
                let v = 0.08 + seed(x, z, k + 2) * 0.84;
                let px = (ix as f32 + u) * w;
                let pz = (iz as f32 + v) * d;
                let choice = seed(x, z, k + 3);
                let species = if layer == 1 {
                    2
                } else if choice < 0.38 {
                    0
                } else if choice < 0.77 {
                    1
                } else if choice < 0.91 {
                    2
                } else {
                    3
                };
                let radius = unit
                    * match species {
                        0 => 0.67 + seed(x, z, k + 4) * 0.33,
                        1 => 0.30 + seed(x, z, k + 4) * 0.25,
                        2 => 0.34 + seed(x, z, k + 4) * 0.25,
                        _ => 0.16 + seed(x, z, k + 4) * 0.12,
                    };
                // Varied setbacks leave a scalloped outline only at exposed edges.
                let setback = unit * (0.06 + seed(x, z, k + 5) * 0.19);
                let radius = radius.min(plot.clearance(px, pz) - setback);
                if radius < unit * if species == 3 { 0.12 } else { 0.23 } {
                    continue;
                }
                let tone = ((seed(x.div_euclid(3), z.div_euclid(3), 71) * 3.) as usize).min(2);
                result.push(Plant {
                    variant: species + tone * SPECIES,
                    transform: Transform {
                        translation: Vec3::new(plot.bounds[0] + px, base, plot.bounds[2] + pz),
                        rotation: Quat::from_rotation_y(seed(x, z, k + 6) * std::f32::consts::TAU),
                        scale: Vec3::new(
                            radius,
                            radius
                                * (0.85 + seed(x, z, k + 7) * 0.38)
                                * if species == 0 { 1.65 } else { 1. },
                            radius,
                        ),
                    },
                });
            }
        }
    }
    result
}
#[derive(serde::Deserialize)]
struct Model {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}
fn models() -> &'static [Model; SPECIES] {
    static MODELS: OnceLock<[Model; SPECIES]> = OnceLock::new();
    MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!("models/understory/fern.mesh.json"),
            crate::model_storage::include_model!("models/understory/woodland_stem.mesh.json"),
            crate::model_storage::include_model!("models/understory/clover.mesh.json"),
            crate::model_storage::include_model!("models/understory/wildflower.mesh.json"),
        ]
        .map(|source| {
            let model: Model =
                crate::model_storage::parse(source).expect("validated CC0 woodland plant");
            assert!(model.positions.len() <= MAX_VERTICES);
            assert_eq!(model.normals.len(), model.positions.len());
            assert_eq!(model.colors.len(), model.positions.len());
            assert!(
                model
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < model.positions.len())
            );
            model
        })
    })
}
pub(crate) fn prototype(variant: usize) -> SurfaceMeshData {
    let model = &models()[variant % SPECIES];
    let tone = variant / SPECIES;
    let mut mesh = SurfaceMeshData {
        positions: model.positions.clone(),
        normals: model.normals.clone(),
        colors: model.colors.clone(),
        indices: model.indices.clone(),
        uvs: vec![[0.; 2]; model.positions.len()],
        ..Default::default()
    };
    for c in &mut mesh.colors {
        if variant % SPECIES != 3 || (c[1] > c[0] * 1.15 && c[1] > c[2] * 1.2) {
            // The source kit includes purple clover and autumn leaves. Preserve
            // their painted veins/gradients while fitting evergreen woodland.
            let luminance = (c[0] * 0.25 + c[1] * 0.60 + c[2] * 0.15).max(0.045);
            let palette = [[0.35, 0.72, 0.36], [0.48, 0.94, 0.34], [0.52, 0.94, 0.52]][tone];
            *c = [luminance * palette[0], luminance * palette[1], luminance * palette[2], 1.];
        }
    }
    mesh
}
pub(crate) fn append(
    mesh: &mut SurfaceMeshData,
    plot: Plot,
    base: f32,
    _tile: f32,
    coordinates: [i32; 2],
) {
    static MESHES: OnceLock<[SurfaceMeshData; VARIANTS]> = OnceLock::new();
    let prototypes = MESHES.get_or_init(|| std::array::from_fn(prototype));
    for plant in plants(plot, base, coordinates) {
        let source = &prototypes[plant.variant];
        let first = mesh.positions.len() as u32;
        let normals = Mat3::from_quat(plant.transform.rotation)
            * Mat3::from_diagonal(plant.transform.scale.recip());
        mesh.positions.extend(source.positions.iter().map(|&p| {
            plant
                .transform
                .transform_point(Vec3::from_array(p))
                .to_array()
        }));
        mesh.normals.extend(
            source
                .normals
                .iter()
                .map(|&n| (normals * Vec3::from_array(n)).normalize().to_array()),
        );
        mesh.colors.extend_from_slice(&source.colors);
        mesh.uvs.extend_from_slice(&source.uvs);
        mesh.indices
            .extend(source.indices.iter().map(|&i| first + i));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn splitting_connected_plots_preserves_every_world_anchored_plant() {
        let mask = (0..4)
            .flat_map(|z| (0..4).map(move |x| 1u64 << ((x + 1) + (z + 1) * 6)))
            .sum();
        let whole = Plot {
            bounds: [0., 32., 0., 32.],
            cells: [4, 4],
            occupied: mask,
        };
        let left = Plot {
            bounds: [0., 16., 0., 32.],
            cells: [2, 4],
            occupied: mask,
        };
        let mut right_mask = 0;
        for z in 0..4 {
            for x in -1..2 {
                right_mask |= 1 << ((x + 1) + (z + 1) * 6);
            }
        }
        let right = Plot {
            bounds: [16., 32., 0., 32.],
            cells: [2, 4],
            occupied: right_mask,
        };
        let mut split = plants(left, 3., [0, 0]);
        split.extend(plants(right, 3., [2, 0]));
        let all = plants(whole, 3., [0, 0]);
        assert_eq!(split.len(), all.len());
        for plant in all {
            assert!(
                split.contains(&plant),
                "plant changes at internal plot seam: {plant:?}"
            );
        }
        assert!(
            split
                .iter()
                .any(|p| (p.transform.translation.x - 16.).abs() < p.transform.scale.x)
        );
    }
    #[test]
    fn plants_have_varied_silhouettes_and_irregular_exposed_edges() {
        let mut species = std::collections::HashSet::new();
        let mut setbacks = Vec::new();
        for x in -12..12 {
            let plot = Plot::isolated([0., 16., 0., 16.]);
            let plants = plants(plot, 3., [x, 17]);
            assert_eq!(plants, super::plants(plot, 3., [x, 17]));
            for p in plants {
                species.insert(p.variant % SPECIES);
                let root = p.transform.translation;
                let edge = root.x.min(16. - root.x).min(root.z).min(16. - root.z);
                assert!(
                    edge > p.transform.scale.x,
                    "plants must stay off authored paths"
                );
                assert_eq!(root.y, 3.);
                assert_eq!(p.transform.scale.x, p.transform.scale.z);
                setbacks.push(edge - p.transform.scale.x);
            }
        }
        assert_eq!(species.len(), SPECIES);
        assert!(
            setbacks.iter().copied().fold(0., f32::max)
                - setbacks.iter().copied().fold(f32::INFINITY, f32::min)
                > 2.
        );
    }
    #[test]
    fn imported_plants_are_grounded_finite_and_use_real_leaf_silhouettes() {
        for variant in 0..VARIANTS {
            let mesh = prototype(variant);
            assert!(mesh.positions.len() > 100 && mesh.positions.len() <= MAX_VERTICES);
            assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| p[1] >= 0. && p[0].hypot(p[2]) <= 1.0001)
            );
            assert!(mesh.positions.iter().any(|p| p[1] == 0.));
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
            );
            assert!(mesh.uvs.iter().all(|&uv| uv == [0.; 2]));
            assert!(mesh.colors.iter().all(|c| c[3] == 1.));
        }
    }
}
