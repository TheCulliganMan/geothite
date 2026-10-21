//! Offline checks of the prepared leg/hip surface against animated fabric.
//! Counts strict triangle crossings, excluding coplanar and exact edge contact.
use bevy::{math::DVec3, prelude::*, render::mesh::VertexAttributeValues};
use std::collections::HashMap;

struct Vertex {
    position: Vec3,
    joints: [u16; 4],
    weights: [f32; 4],
}

pub(super) struct LowerLegs {
    vertices: Vec<Vertex>,
    faces: Vec<[usize; 3]>,
}

impl LowerLegs {
    pub(super) fn new(mesh: &Mesh, leg_joints: &[bool]) -> Self {
        Self::select(mesh, leg_joints, None)
    }

    /// Retain hip-weighted faces and whole triangles that straddle the waist.
    /// The trouser check must include the body inside the saddle and waistband,
    /// which the original below-knee garment check intentionally omits.
    pub(super) fn with_waist(mesh: &Mesh, leg_joints: &[bool], hips_joint: usize) -> Self {
        Self::select(mesh, leg_joints, Some(hips_joint))
    }

    fn select(mesh: &Mesh, leg_joints: &[bool], hips_joint: Option<usize>) -> Self {
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("contact body positions");
        };
        let Some(VertexAttributeValues::Uint16x4(joints)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        else {
            panic!("contact body joints");
        };
        let Some(VertexAttributeValues::Float32x4(weights)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        else {
            panic!("contact body weights");
        };
        let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
        let mut remap = vec![None; points.len()];
        let mut vertices = Vec::new();
        let faces: Vec<_> = indices
            .chunks_exact(3)
            .filter(|face| {
                let within_height = if hips_joint.is_some() {
                    // Keep an entire crossing triangle rather than clipping
                    // away the source body at the 1.69-high trouser waist.
                    face.iter().any(|&i| points[i][1] < 1.72)
                } else {
                    face.iter().all(|&i| points[i][1] < 1.2)
                };
                within_height
                    && face.iter().any(|&i| {
                        (0..4).any(|slot| {
                            let joint = joints[i][slot] as usize;
                            weights[i][slot] > 0.1
                                && (leg_joints[joint] || hips_joint == Some(joint))
                        })
                    })
            })
            .map(|face| {
                [face[0], face[1], face[2]].map(|i| {
                    *remap[i].get_or_insert_with(|| {
                        let index = vertices.len();
                        vertices.push(Vertex {
                            position: Vec3::from_array(points[i]),
                            joints: joints[i],
                            weights: weights[i],
                        });
                        index
                    })
                })
            })
            .collect();
        assert!(
            !faces.is_empty(),
            "contact check requires a retained leg/hip surface"
        );
        Self { vertices, faces }
    }

    pub(super) fn pose(&self, palette: &[Mat4]) -> Vec<[Vec3; 3]> {
        let points: Vec<Vec3> = self
            .vertices
            .iter()
            .map(|vertex| {
                (0..4)
                    .map(|slot| {
                        palette[vertex.joints[slot] as usize].transform_point3(vertex.position)
                            * vertex.weights[slot]
                    })
                    .sum()
            })
            .collect();
        self.faces
            .iter()
            .map(|face| face.map(|i| points[i]))
            .collect()
    }
}

#[derive(Clone, Copy)]
struct Bounds {
    low: Vec3,
    high: Vec3,
}

impl Bounds {
    fn new(triangle: [Vec3; 3]) -> Self {
        Self {
            low: triangle[0].min(triangle[1]).min(triangle[2]),
            high: triangle[0].max(triangle[1]).max(triangle[2]),
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.high.cmpge(other.low).all() && other.high.cmpge(self.low).all()
    }

    fn cells(self, mut visit: impl FnMut([i32; 3])) {
        let low = (self.low / 0.12).floor().as_ivec3();
        let high = (self.high / 0.12).floor().as_ivec3();
        for x in low.x..=high.x {
            for y in low.y..=high.y {
                for z in low.z..=high.z {
                    visit([x, y, z]);
                }
            }
        }
    }
}

fn segment_hits(start: DVec3, end: DVec3, triangle: [DVec3; 3]) -> bool {
    let direction = end - start;
    let edge1 = triangle[1] - triangle[0];
    let edge2 = triangle[2] - triangle[0];
    let p = direction.cross(edge2);
    let determinant = edge1.dot(p);
    if determinant.abs() < 1e-12 {
        return false;
    }
    let tvec = start - triangle[0];
    let u = tvec.dot(p) / determinant;
    let q = tvec.cross(edge1);
    let v = direction.dot(q) / determinant;
    let t = edge2.dot(q) / determinant;
    let epsilon = 1e-6;
    epsilon < u && epsilon < v && u + v < 1. - epsilon && epsilon < t && t < 1. - epsilon
}

fn intersects(a: [Vec3; 3], b: [Vec3; 3]) -> bool {
    let a = a.map(Vec3::as_dvec3);
    let b = b.map(Vec3::as_dvec3);
    [(a, b), (b, a)].into_iter().any(|(triangle, other)| {
        (0..3).any(|i| segment_hits(triangle[i], triangle[(i + 1) % 3], other))
    })
}

pub(super) fn crossings(body: &[[Vec3; 3]], cloth: &[[Vec3; 3]]) -> Vec<(usize, usize)> {
    let bounds: Vec<_> = cloth.iter().copied().map(Bounds::new).collect();
    let mut grid: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
    for (index, bounds) in bounds.iter().enumerate() {
        bounds.cells(|cell| grid.entry(cell).or_default().push(index));
    }
    let mut seen = vec![usize::MAX; cloth.len()];
    let mut pairs = Vec::new();
    for (index, &triangle) in body.iter().enumerate() {
        let body_bounds = Bounds::new(triangle);
        body_bounds.cells(|cell| {
            if let Some(candidates) = grid.get(&cell) {
                for &candidate in candidates {
                    if seen[candidate] == index {
                        continue;
                    }
                    seen[candidate] = index;
                    if body_bounds.overlaps(bounds[candidate])
                        && intersects(triangle, cloth[candidate])
                    {
                        pairs.push((index, candidate));
                    }
                }
            }
        });
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::RenderAssetUsages,
        render::mesh::{Indices, PrimitiveTopology},
    };

    #[test]
    fn waist_contact_keeps_hip_only_and_straddling_body_faces() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        let lower = [[0., 0.8, 0.], [0.2, 0.8, 0.], [0., 1., 0.]];
        let hip = [[0., 1.6, 0.], [0.2, 1.8, 0.], [0., 1.8, 0.]];
        let above = hip.map(|p| [p[0], p[1] + 1., p[2]]);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            lower
                .into_iter()
                .chain(hip)
                .chain(above)
                .collect::<Vec<_>>(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(
                vec![[0, 0, 0, 0]; 3]
                    .into_iter()
                    .chain(vec![[1, 0, 0, 0]; 6])
                    .collect(),
            ),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[1., 0., 0., 0.]; 9]);
        mesh.insert_indices(Indices::U32((0..9).collect()));

        let palette = [Mat4::IDENTITY, Mat4::from_translation(Vec3::Z)];
        let original = LowerLegs::new(&mesh, &[true, false]).pose(&palette);
        assert_eq!(original, vec![lower.map(Vec3::from_array)]);
        let through_waist = LowerLegs::with_waist(&mesh, &[true, false], 1).pose(&palette);
        assert_eq!(
            through_waist,
            vec![
                lower.map(Vec3::from_array),
                hip.map(|p| Vec3::from_array(p) + Vec3::Z)
            ]
        );
    }

    #[test]
    fn strict_crossing_is_counted_once_across_spatial_cells() {
        let flat = [Vec3::ZERO, Vec3::X, Vec3::Y];
        let through = [
            Vec3::new(0.2, 0.2, -1.),
            Vec3::new(0.2, 0.2, 1.),
            Vec3::new(0.3, 0.2, 1.),
        ];
        assert_eq!(crossings(&[flat], &[through]), vec![(0, 0)]);
        assert_eq!(crossings(&[through], &[flat]), vec![(0, 0)]);
        let shifted = through.map(|p| p + Vec3::X * 2.);
        assert!(crossings(&[flat], &[shifted]).is_empty());
        assert!(crossings(&[flat], &[flat]).is_empty());
        let touch = [Vec3::NEG_Z, Vec3::Z, Vec3::new(-0.3, 0., 1.)];
        assert!(crossings(&[flat], &[touch]).is_empty());
    }
}
