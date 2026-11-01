//! Rigid dual-quaternion blends for the continuous shoulder/elbow field.
use crate::open_people::PersonRig;
use bevy::prelude::*;
mod lattice;
pub(crate) use lattice::Recipe;
#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) use lattice::tests as validation;

#[derive(Clone, Copy)]
pub(crate) struct Dual {
    real: Quat,
    dual: Quat,
}

impl Dual {
    pub(crate) fn from_matrix(matrix: Mat4) -> Option<Self> {
        let (scale, real, translation) = matrix.to_scale_rotation_translation();
        if !matrix.is_finite() || (scale - Vec3::ONE).abs().max_element() > 0.001 {
            return None;
        }
        let real = real.normalize();
        let dual = Quat::from_xyzw(translation.x, translation.y, translation.z, 0.) * real * 0.5;
        Some(Self { real, dual })
    }

    pub(crate) fn blend(parts: &[(Self, f32)]) -> Self {
        let reference = parts
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap()
            .0
            .real;
        let zero = Quat::from_xyzw(0., 0., 0., 0.);
        let (mut real, mut dual) = (zero, zero);
        for &(part, weight) in parts {
            let weight = if reference.dot(part.real) < 0. {
                -weight
            } else {
                weight
            };
            real = real + part.real * weight;
            dual = dual + part.dual * weight;
        }
        let length = real.length();
        assert!(
            length > 1e-6 && length.is_finite(),
            "degenerate arm quaternion blend"
        );
        real = real / length;
        dual = dual / length;
        dual = dual - real * real.dot(dual);
        Self { real, dual }
    }

    pub(crate) fn transform(self) -> Transform {
        let translation = self.dual * self.real.conjugate() * 2.;
        Transform::from_translation(Vec3::new(translation.x, translation.y, translation.z))
            .with_rotation(self.real)
    }
}

pub(crate) struct SkinFrame {
    matrices: Vec<Option<Dual>>,
    arms: [[usize; 5]; 2],
    carriers: Vec<usize>,
}
impl SkinFrame {
    pub(crate) fn new(rig: &PersonRig, palette: &[Mat4]) -> Self {
        Self {
            matrices: palette.iter().copied().map(Dual::from_matrix).collect(),
            arms: [
                crate::arm_skin::ancestry(rig, 0),
                crate::arm_skin::ancestry(rig, 1),
            ],
            carriers: [
                crate::arm_skin::carriers(rig, 0),
                crate::arm_skin::carriers(rig, 1),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }
    pub(crate) fn deform(
        &self,
        joints: [u16; 4],
        weights: [f32; 4],
        point: Vec3,
        normal: Vec3,
    ) -> Option<(Vec3, Vec3)> {
        let attached = (0..4)
            .filter(|&s| {
                self.arms
                    .iter()
                    .any(|arm| arm[..2].contains(&(joints[s] as usize)))
            })
            .map(|s| weights[s])
            .sum::<f32>()
            > 0.001
            && (0..4)
                .filter(|&s| self.carriers.contains(&(joints[s] as usize)))
                .map(|s| weights[s])
                .sum::<f32>()
                > 0.999;
        if !attached {
            return None;
        }
        let mut parts = Vec::with_capacity(4);
        for slot in 0..4 {
            if weights[slot] > 0. {
                parts.push((self.matrices[joints[slot] as usize]?, weights[slot]));
            }
        }
        let transform = Dual::blend(&parts).transform();
        Some((
            transform.transform_point(point),
            transform.rotation * normal,
        ))
    }
}

/// Frozen native study only: solve back through the existing matrix palette
/// so the ordinary renderer displays the desired DQ surface at this one pose.
/// Production skinning and the source bind surface are unchanged by this trial.
#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
pub(crate) fn freeze(rig: &PersonRig, mesh: &mut Mesh, palette: &[Mat4]) {
    use bevy::render::mesh::VertexAttributeValues;
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("DQ preview positions")
    };
    let points = points.clone();
    let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("DQ preview normals")
    };
    let normals = normals.clone();
    let Some(VertexAttributeValues::Uint16x4(joints)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
    else {
        panic!("DQ preview joints")
    };
    let joints = joints.clone();
    let Some(VertexAttributeValues::Float32x4(weights)) =
        mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
    else {
        panic!("DQ preview weights")
    };
    let weights = weights.clone();
    let frame = SkinFrame::new(rig, palette);
    let mut positions = points.clone();
    let mut shaded = normals.clone();
    for i in 0..points.len() {
        let Some((q, n)) = frame.deform(
            joints[i],
            weights[i],
            Vec3::from_array(points[i]),
            Vec3::from_array(normals[i]),
        ) else {
            continue;
        };
        let matrix = (0..4)
            .map(|s| palette[joints[i][s] as usize] * weights[i][s])
            .fold(Mat4::ZERO, |a, b| a + b);
        assert!(
            matrix.determinant().abs() > 0.01,
            "DQ preview requires an invertible matrix palette"
        );
        positions[i] = matrix.inverse().transform_point3(q).to_array();
        shaded[i] = (Mat3::from_mat4(matrix).transpose() * n)
            .normalize()
            .to_array();
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, shaded);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quarter_turn_blend_preserves_radius_and_rigid_endpoints() {
        let a = Dual::from_matrix(Mat4::IDENTITY).unwrap();
        let b = Dual::from_matrix(Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2)).unwrap();
        let halfway = Dual::blend(&[(a, 0.5), (b, 0.5)]).transform();
        let point = halfway.transform_point(Vec3::X);
        assert!(point.distance(Vec3::new(0.5_f32.sqrt(), 0.5_f32.sqrt(), 0.)) < 1e-6);
        assert!((point.length() - 1.).abs() < 1e-6);
        for (t, expected) in [(0., Vec3::X), (1., Vec3::Y)] {
            assert!(
                Dual::blend(&[(a, 1. - t), (b, t)])
                    .transform()
                    .transform_point(Vec3::X)
                    .distance(expected)
                    < 1e-6
            );
        }
        assert!(
            ((Mat4::IDENTITY + Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2)) * 0.5)
                .transform_point3(Vec3::X)
                .length()
                < 0.71
        );
    }
    #[test]
    fn translation_and_antipodal_quaternions_blend_consistently() {
        let matrix =
            Mat4::from_rotation_translation(Quat::from_rotation_y(1.2), Vec3::new(0.7, 0.3, -0.5));
        let a = Dual::from_matrix(matrix).unwrap();
        let b = Dual {
            real: -a.real,
            dual: -a.dual,
        };
        let blended = Dual::blend(&[(a, 0.2), (b, 0.8)]).transform();
        assert!(
            blended
                .transform_point(Vec3::new(0.2, 1., -0.3))
                .distance(matrix.transform_point3(Vec3::new(0.2, 1., -0.3)))
                < 1e-6
        );
        assert!(Dual::from_matrix(Mat4::from_scale(Vec3::splat(0.72))).is_none());
    }
}
