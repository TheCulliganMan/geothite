//! Source-scoped non-human actor geometry; no item state or interaction logic.
use crate::mesh::SurfaceMeshData;
use crate::new_bark_models::{ModelKind, model};
use bevy::prelude::Vec3;

/// A dwarf orchard tree uses the original authored crown/trunk, with small
/// original berry geometry. The source sprite also remains a fruit tree after
/// picking; this prop is not an availability indicator and never reads the bag.
pub(super) fn fruit_tree() -> SurfaceMeshData {
    let tree = model(ModelKind::Tree);
    let mut mesh = tree.surface_mesh();
    let scale = 1.08 / (tree.max[1] - tree.min[1]);
    for p in &mut mesh.positions {
        p[0] *= scale;
        p[1] = (p[1] - tree.min[1]) * scale;
        p[2] *= scale;
    }
    for center in [
        [-0.14, 0.66, 0.37],
        [0.12, 0.71, 0.34],
        [0.02, 0.94, 0.175],
        [-0.37, 0.71, 0.03],
        [0.38, 0.73, 0.0],
        [0.04, 0.79, -0.325],
        [-0.15, 1.0, -0.13],
    ] {
        append_berry(&mut mesh, Vec3::from_array(center));
    }
    mesh
}

fn append_berry(mesh: &mut SurfaceMeshData, center: Vec3) {
    const SEGMENTS: usize = 10;
    const RINGS: usize = 6;
    let base = mesh.positions.len() as u32;
    for row in 0..=RINGS {
        let latitude = row as f32 / RINGS as f32 * std::f32::consts::PI;
        for col in 0..=SEGMENTS {
            let longitude = col as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let normal = Vec3::new(
                latitude.sin() * longitude.cos(),
                latitude.cos(),
                latitude.sin() * longitude.sin(),
            );
            mesh.positions
                .push((center + normal * Vec3::new(0.044, 0.049, 0.044)).to_array());
            mesh.normals.push(
                (normal / Vec3::new(0.044, 0.049, 0.044))
                    .normalize()
                    .to_array(),
            );
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push([0.67, 0.065, 0.082, 1.0]);
        }
    }
    for row in 0..RINGS {
        for col in 0..SEGMENTS {
            let a = base + (row * (SEGMENTS + 1) + col) as u32;
            let b = a + (SEGMENTS + 1) as u32;
            // Outward winding; skip degenerate triangles at the two poles.
            if row > 0 {
                mesh.indices.extend([a, a + 1, b]);
            }
            if row + 1 < RINGS {
                mesh.indices.extend([a + 1, b + 1, b]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fruit_tree_is_grounded_closed_and_uses_original_geometry() {
        let mesh = fruit_tree();
        let low = mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let high = mesh
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(low.abs() < 0.0001);
        assert!((1.0..1.2).contains(&high));
        assert!(mesh.indices.len() > model(ModelKind::Tree).surface_mesh().indices.len());
        let original_count = model(ModelKind::Tree).surface_mesh().positions.len();
        for tri in mesh.indices.chunks_exact(3) {
            let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let geometric = (p[1] - p[0]).cross(p[2] - p[0]);
            let normal = Vec3::from_array(mesh.normals[tri[0] as usize]);
            assert!(geometric.is_finite());
            if tri[0] as usize >= original_count {
                assert!(geometric.dot(normal) > 0.0);
            }
        }
    }
}
