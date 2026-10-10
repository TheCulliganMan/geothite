//! Stable per-placement variation. Camera movement never changes a tree.
use crate::{
    mesh::{SurfaceMeshData, TerrainMeshData},
    new_bark_models::{self, Model, ModelKind},
};
use bevy::prelude::*;
use std::sync::OnceLock;

pub(crate) const VARIANT_PATHS: [&str; 4] = [
    "models/johto/tree_variants/tree_2.mesh.json",
    "models/johto/tree_variants/tree_3.mesh.json",
    "models/johto/tree_variants/tree_4.mesh.json",
    "models/johto/tree_variants/tree_5.mesh.json",
];

fn variant(kind: ModelKind, index: usize) -> &'static Model {
    static VARIANTS: [OnceLock<Option<Model>>; 4] = [const { OnceLock::new() }; 4];
    if index == 0 {
        return new_bark_models::model(kind);
    }
    VARIANTS[index - 1]
        .get_or_init(|| {
            crate::open_models::load(
                VARIANT_PATHS[index - 1],
                crate::model_storage::Source::Plain("null"),
                |source| match source {
                    crate::model_storage::Source::Plain("null") => Ok(None),
                    _ => Model::parse(source).map(Some),
                },
            )
        })
        .as_ref()
        .unwrap_or_else(|| new_bark_models::model(kind))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Variation {
    index: usize,
    height: f32,
    width: f32,
    turn: f32,
    tint: [f32; 3],
    offset: [f32; 2],
}
fn variation(x: i32, z: i32) -> Variation {
    let mut value = (x as u32).wrapping_mul(0x9e3779b9) ^ (z as u32).wrapping_mul(0x85ebca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846ca68b);
    value ^= value >> 16;
    let unit = |shift: u32| ((value >> shift) & 255) as f32 / 255.;
    Variation {
        index: value as usize % 5,
        height: 0.90 + unit(8) * 0.70,
        width: 0.84 + unit(16) * 0.16,
        turn: ((value >> 24) & 3) as f32 * std::f32::consts::FRAC_PI_2,
        offset: [unit(0) * 2.0 - 1.0, unit(24) * 2.0 - 1.0],
        tint: [
            0.94 + unit(0) * 0.08,
            0.96 + unit(8) * 0.06,
            0.94 + unit(16) * 0.09,
        ],
    }
}

/// Mature world trees use physical world scale, independently of source drawing height.
/// Crowns overlap within groves; trunks remain inside their authored blocked plots.
pub(crate) fn append_world_tree(
    mesh: &mut TerrainMeshData,
    kind: ModelKind,
    bounds: [f32; 4],
    base: f32,
    tile_height: f32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
) {
    let v = variation(coordinates[0], coordinates[1]);
    let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
    let center = [(bounds[0] + bounds[1]) * 0.5, (bounds[2] + bounds[3]) * 0.5];
    let density = neighbors.iter().filter(|&&n| n).count() as f32 / 4.0;
    let offset = [
        v.offset[0] * spacing * 0.30 * density,
        v.offset[1] * spacing * 0.30 * density,
    ];
    let crown = spacing * (1.0 + density * 1.45);
    let fitted = [
        center[0] + offset[0] - crown * 0.5,
        center[0] + offset[0] + crown * 0.5,
        center[1] + offset[1] - crown * 0.5,
        center[1] + offset[1] + crown * 0.5,
    ];
    // 9–16 source cells: mature trees clear normal roofs rather than inheriting
    // the two-cell sprite height. Arena trees retain their separate metre scale.
    append_tree_impl(
        mesh,
        kind,
        fitted,
        base,
        tile_height * 10.0,
        coordinates,
        (density > 0.0).then_some(Vec3::new(
            center[0] + offset[0],
            base,
            center[1] + offset[1],
        )),
    );
}

pub(crate) fn append_tree(
    mesh: &mut TerrainMeshData,
    kind: ModelKind,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
) {
    append_tree_impl(mesh, kind, bounds, base, height, coordinates, None);
}

fn append_tree_impl(
    mesh: &mut TerrainMeshData,
    kind: ModelKind,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
    root_anchor: Option<Vec3>,
) {
    let v = variation(coordinates[0], coordinates[1]);
    let tree = variant(kind, v.index);
    let mut center = Vec3::new(
        (bounds[0] + bounds[1]) * 0.5,
        base,
        (bounds[2] + bounds[3]) * 0.5,
    );
    // Tree plots are square; quarter turns retain the occupied source footprint.
    let side = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]) * v.width * 0.5;
    let margin = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]) * 0.5 - side;
    center.x += v.offset[0] * margin * 0.8;
    center.z += v.offset[1] * margin * 0.8;
    let fitted = [
        center.x - side,
        center.x + side,
        center.z - side,
        center.z + side,
    ];
    let scale = height * v.height / (tree.max[1] - tree.min[1]);
    let mut solid = SurfaceMeshData::default();
    tree.append_fitted(&mut solid, fitted, base, scale, None);
    let mut parts = Vec::new();
    tree.append_textured_fitted(&mut parts, fitted, base, scale);
    let rotate = Quat::from_rotation_y(v.turn);
    // Crown bounds are asymmetric, especially for split-trunk variants. Anchor
    // the actual low trunk geometry rather than assuming its root is centred.
    let mut root_min = Vec3::splat(f32::INFINITY);
    let mut root_max = Vec3::splat(f32::NEG_INFINITY);
    for &p in solid
        .positions
        .iter()
        .filter(|p| p[1] <= base + height * v.height * 0.02)
    {
        root_min = root_min.min(Vec3::from_array(p));
        root_max = root_max.max(Vec3::from_array(p));
    }
    let shift = root_anchor
        .filter(|_| root_min.is_finite())
        .map_or(Vec3::ZERO, |anchor| {
            let root = (root_min + root_max) * 0.5;
            let rotated = center + rotate * (root - center);
            Vec3::new(anchor.x - rotated.x, 0.0, anchor.z - rotated.z)
        });
    let finish = |surface: &mut SurfaceMeshData| {
        for position in &mut surface.positions {
            *position =
                (shift + center + rotate * (Vec3::from_array(*position) - center)).to_array();
        }
        let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
        for (normal, color) in surface.normals.iter_mut().zip(&mut surface.colors) {
            let original = Vec3::from_array(*normal);
            let rotated = rotate * original;
            // Reorient the existing key/fill bake along with the geometry.
            let shade = (0.62 + 0.38 * rotated.dot(light).max(0.0))
                / (0.62 + 0.38 * original.dot(light).max(0.0));
            *normal = rotated.to_array();
            for channel in 0..3 {
                color[channel] *= v.tint[channel] * shade;
            }
        }
    };
    finish(&mut solid);
    let first = mesh.solid.positions.len() as u32;
    mesh.solid.positions.extend(solid.positions);
    mesh.solid
        .cutaway_ranges
        .push(first as usize..mesh.solid.positions.len());
    mesh.solid.normals.extend(solid.normals);
    mesh.solid.uvs.extend(solid.uvs);
    mesh.solid.colors.extend(solid.colors);
    mesh.solid
        .indices
        .extend(solid.indices.into_iter().map(|i| i + first));
    for mut part in parts {
        finish(&mut part.mesh);
        part.mesh.cutaway_ranges = vec![0..part.mesh.positions.len()];
        mesh.scenery.push(part);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mature_trees_clear_house_roofs_and_keep_authored_trunk_plots() {
        let house = new_bark_models::model(ModelKind::PlayerHouse);
        let roof = (house.max[1] - house.min[1]) * 16.0;
        for z in 0..20 {
            for x in 0..20 {
                let v = variation(x, z);
                assert!(
                    80.0 * v.height > roof,
                    "mature crown must clear normal house roof"
                );
                assert!(v.offset.iter().all(|o| (o * 16.0 * 0.30).abs() < 8.0));
            }
        }
    }
    #[test]
    fn grove_variation_is_stable_diverse_and_bounded() {
        let mut counts = [0; 5];
        let mut turns = [0; 4];
        for z in -10..10 {
            for x in -10..10 {
                let v = variation(x, z);
                assert_eq!(v, variation(x, z));
                counts[v.index] += 1;
                turns[(v.turn / std::f32::consts::FRAC_PI_2).round() as usize] += 1;
                assert!((0.90..=1.601).contains(&v.height));
                assert!((0.84..=1.001).contains(&v.width));
            }
        }
        assert!(counts.iter().all(|n| *n > 40));
        assert!(turns.iter().all(|n| *n > 50));
    }
}
