//! Conservative removal of strictly buried triangles in the two planter kits.
//! The original closed editable parts remain untouched. This cached import pass
//! changes neither the exterior union nor surviving triangle attributes/order.
use super::super::{Export, Model, Primitive};
use bevy::prelude::Vec3;
use std::collections::HashMap;

const INTERIOR_MARGIN: f32 = 0.00001;

struct ConvexPart {
    min: Vec3,
    max: Vec3,
    planes: Vec<(Vec3, Vec3)>,
}

fn point(primitive: &Primitive, index: u32) -> Vec3 {
    let offset = index as usize * 3;
    Vec3::from_slice(&primitive.positions[offset..offset + 3])
}

fn position_key(point: Vec3) -> [u32; 3] {
    // Signed zero denotes the same geometric vertex for the manifold check.
    point
        .to_array()
        .map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}

impl ConvexPart {
    fn verified(primitive: &Primitive) -> Option<Self> {
        if primitive.base_color[3] != 1.0 {
            return None;
        }
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut planes = Vec::new();
        let mut edges = HashMap::<([u32; 3], [u32; 3]), (u32, i32)>::new();
        for triangle in primitive.indices.chunks_exact(3) {
            let points = triangle_points(primitive, triangle);
            let cross = (points[1] - points[0]).cross(points[2] - points[0]);
            if !cross.is_finite() || cross.length_squared() <= 1e-16 {
                return None;
            }
            // Derive the half-space from the actual triangle positions/winding,
            // never from the export's rounded shading normals.
            planes.push((cross.normalize(), points[0]));
            for i in 0..3 {
                min = min.min(points[i]);
                max = max.max(points[i]);
                let a = position_key(points[i]);
                let b = position_key(points[(i + 1) % 3]);
                let (key, orientation) = if a < b { ((a, b), 1) } else { ((b, a), -1) };
                let entry = edges.entry(key).or_default();
                entry.0 += 1;
                entry.1 += orientation;
            }
        }
        if edges.is_empty()
            || edges
                .values()
                .any(|&(count, balance)| count != 2 || balance != 0)
        {
            return None;
        }
        // Closed but concave or inward-wound parts cannot certify containment.
        if primitive.positions.chunks_exact(3).any(|p| {
            let p = Vec3::from_slice(p);
            planes
                .iter()
                .any(|&(normal, origin)| normal.dot(p - origin) > INTERIOR_MARGIN)
        }) {
            return None;
        }
        Some(Self { min, max, planes })
    }

    fn strictly_contains(&self, triangle: [Vec3; 3]) -> bool {
        triangle.into_iter().all(|p| {
            p.cmpgt(self.min + Vec3::splat(INTERIOR_MARGIN)).all()
                && p.cmplt(self.max - Vec3::splat(INTERIOR_MARGIN)).all()
                && self
                    .planes
                    .iter()
                    .all(|&(normal, origin)| normal.dot(p - origin) < -INTERIOR_MARGIN)
        })
    }
}

fn triangle_points(primitive: &Primitive, triangle: &[u32]) -> [Vec3; 3] {
    [
        point(primitive, triangle[0]),
        point(primitive, triangle[1]),
        point(primitive, triangle[2]),
    ]
}

fn retained_triangles(export: &Export) -> Vec<bool> {
    let occluders: Vec<_> = export.primitives.iter().map(ConvexPart::verified).collect();
    let mut retained = Vec::new();
    for (own, primitive) in export.primitives.iter().enumerate() {
        for triangle in primitive.indices.chunks_exact(3) {
            let points = triangle_points(primitive, triangle);
            let buried = occluders.iter().enumerate().any(|(other, hull)| {
                other != own
                    && hull
                        .as_ref()
                        .is_some_and(|hull| hull.strictly_contains(points))
            });
            retained.push(!buried);
        }
    }
    retained
}

pub(super) fn parse_planter(source: &str) -> Result<Model, String> {
    // Normal importer validation precedes geometric indexing and remains the
    // authority for material attributes, normal normalization and fit bounds.
    let original = Model::parse(source)?;
    let export: Export = crate::model_storage::parse(source)?;
    let retained = retained_triangles(&export);
    if retained.iter().all(|&keep| keep) {
        return Ok(original);
    }
    let mut surface = crate::mesh::SurfaceMeshData::default();
    let mut remap = vec![u32::MAX; original.surface.positions.len()];
    for (triangle, keep) in original.surface.indices.chunks_exact(3).zip(retained) {
        if !keep {
            continue;
        }
        for &index in triangle {
            let old = index as usize;
            if remap[old] == u32::MAX {
                remap[old] = surface.positions.len() as u32;
                surface.positions.push(original.surface.positions[old]);
                surface.normals.push(original.surface.normals[old]);
                surface.uvs.push(original.surface.uvs[old]);
                surface.colors.push(original.surface.colors[old]);
            }
            surface.indices.push(remap[old]);
        }
    }
    Ok(Model {
        surface,
        min: original.min,
        max: original.max,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCES: [&str; 2] = [
        include_str!("../models/gym_scenery/planter_leafy.mesh.json"),
        include_str!("../models/gym_scenery/planter_round.mesh.json"),
    ];

    #[test]
    fn planter_cull_preserves_surviving_attributes_order_and_original_fit_bounds() {
        for (source, triangles, vertices) in [(SOURCES[0], 724, 1768), (SOURCES[1], 765, 1947)] {
            let original = Model::parse(source).unwrap();
            let export: Export = crate::model_storage::parse(source).unwrap();
            assert!(
                export
                    .primitives
                    .iter()
                    .all(|part| ConvexPart::verified(part).is_some())
            );
            let retained = retained_triangles(&export);
            let optimized = parse_planter(source).unwrap();
            assert_eq!(optimized.surface.indices.len() / 3, triangles);
            assert_eq!(optimized.surface.positions.len(), vertices);
            assert_eq!(optimized.min, original.min);
            assert_eq!(optimized.max, original.max);
            let expected: Vec<_> = original
                .surface
                .indices
                .chunks_exact(3)
                .zip(&retained)
                .filter(|(_, keep)| **keep)
                .flat_map(|(tri, _)| tri)
                .copied()
                .collect();
            assert_eq!(expected.len(), optimized.surface.indices.len());
            for (&before, &after) in expected.iter().zip(&optimized.surface.indices) {
                let before = before as usize;
                let after = after as usize;
                assert_eq!(
                    original.surface.positions[before],
                    optimized.surface.positions[after]
                );
                assert_eq!(
                    original.surface.normals[before],
                    optimized.surface.normals[after]
                );
                assert_eq!(original.surface.uvs[before], optimized.surface.uvs[after]);
                assert_eq!(
                    original.surface.colors[before],
                    optimized.surface.colors[after]
                );
            }
            let mut fitted_original = crate::mesh::SurfaceMeshData::default();
            let mut fitted_optimized = crate::mesh::SurfaceMeshData::default();
            original.append(&mut fitted_original, [-17.0, 11.0, -5.0, 2.0], 3.0, 19.0);
            optimized.append(&mut fitted_optimized, [-17.0, 11.0, -5.0, 2.0], 3.0, 19.0);
            for (&before, &after) in expected.iter().zip(&fitted_optimized.indices) {
                assert_eq!(
                    fitted_original.positions[before as usize],
                    fitted_optimized.positions[after as usize]
                );
                assert_eq!(
                    fitted_original.normals[before as usize],
                    fitted_optimized.normals[after as usize]
                );
                assert_eq!(
                    fitted_original.colors[before as usize],
                    fitted_optimized.colors[after as usize]
                );
            }
        }
    }

    #[test]
    fn planter_occlusion_requires_opaque_closed_convex_geometry_and_strict_interior() {
        let export: Export = crate::model_storage::parse(SOURCES[0]).unwrap();
        let mut base = export.primitives.into_iter().next().unwrap();
        let hull = ConvexPart::verified(&base).unwrap();
        let center = (hull.min + hull.max) * 0.5;
        assert!(hull.strictly_contains([center; 3]));
        let surface = point(&base, base.indices[0]);
        assert!(
            !hull.strictly_contains([surface; 3]),
            "coplanar points must remain"
        );
        assert!(
            !hull.strictly_contains([center, center, hull.max + Vec3::ONE]),
            "partial coverage must remain"
        );
        let normals = std::mem::replace(&mut base.normals, Vec::new());
        assert!(
            ConvexPart::verified(&base).is_some(),
            "rounded shading normals are not containment evidence"
        );
        base.normals = normals;
        base.base_color[3] = 0.99;
        assert!(ConvexPart::verified(&base).is_none());
        base.base_color[3] = 1.0;
        let triangle = base.indices.split_off(base.indices.len() - 3);
        assert!(
            ConvexPart::verified(&base).is_none(),
            "open part must never occlude"
        );
        base.indices.extend(triangle);
        for triangle in base.indices.chunks_exact_mut(3) {
            triangle.swap(1, 2);
        }
        assert!(
            ConvexPart::verified(&base).is_none(),
            "inward half-spaces must never occlude"
        );
    }
}
