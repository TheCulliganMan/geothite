//! Native, opt-in art experiment. A frozen pose is inverse-skinned back into
//! its existing GPU mesh; this is not the production animation implementation.
//! It tests a length-limited continuous drape before committing to a runtime
//! skin field. No body vertices, texture pixels, topology or input state change.
use crate::open_people::PersonRig;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};

const TOP: f32 = 1.62;
const LOW: f32 = 0.05;
const SAMPLES: usize = 257;
const COLUMNS: usize = 48;
const CENTER_Z: f32 = 0.05;

struct UpperHull(Vec<Vec2>);
impl UpperHull {
    /// Projection of a 3D convex cage onto (height, radial support). Its upper
    /// chain gives exact slice supports without a 3D hull or a physics library.
    fn new(mut points: Vec<Vec2>) -> Self {
        points.sort_by(|a, b| a.x.total_cmp(&b.x).then(b.y.total_cmp(&a.y)));
        points.dedup_by(|a, b| a.x == b.x);
        let mut hull: Vec<Vec2> = Vec::new();
        for point in points {
            while hull.len() >= 2 {
                let a = hull[hull.len() - 2];
                let b = hull[hull.len() - 1];
                if (b - a).perp_dot(point - b) < 0. {
                    break;
                }
                hull.pop();
            }
            hull.push(point);
        }
        Self(hull)
    }
    fn support(&self, y: f32) -> Option<f32> {
        if y < self.0.first()?.x || y > self.0.last()?.x {
            return None;
        }
        let b = self.0.partition_point(|p| p.x < y).min(self.0.len() - 1);
        let a = b.saturating_sub(1);
        if a == b {
            return Some(self.0[a].y);
        }
        let t = (y - self.0[a].x) / (self.0[b].x - self.0[a].x);
        Some(self.0[a].y + (self.0[b].y - self.0[a].y) * t)
    }
}

struct Ring {
    height: f32,
    polar: Vec<Vec2>,
}
struct Profile {
    rings: Vec<Ring>,
}
impl Profile {
    fn from_shell(points: &[[f32; 3]]) -> Self {
        let width = points.iter().take_while(|p| p[1] == points[0][1]).count();
        assert_eq!(width, 41, "trial requires the authored closed shell");
        let mut end = width;
        while end < points.len() && points[end][1] < points[0][1] {
            end += width;
        }
        assert!(end < points.len() && end % width == 0);
        Self {
            rings: points[..end]
                .chunks_exact(width)
                .map(|ring| Ring {
                    height: ring[0][1],
                    polar: ring
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let radial = Vec2::new(p[0], p[2] - CENTER_Z);
                            let angle = if i == 40 {
                                std::f32::consts::TAU
                            } else {
                                radial.x.atan2(radial.y).rem_euclid(std::f32::consts::TAU)
                            };
                            Vec2::new(angle, radial.length())
                        })
                        .collect(),
                })
                .collect(),
        }
    }
    fn radius(&self, y: f32, angle: f32) -> f32 {
        let angle = angle.rem_euclid(std::f32::consts::TAU);
        let radius = |row: usize| {
            let polar = &self.rings[row].polar;
            let b = polar.partition_point(|p| p.x < angle).clamp(1, 40);
            let a = b - 1;
            let f = ((angle - polar[a].x) / (polar[b].x - polar[a].x)).clamp(0., 1.);
            polar[a].y + (polar[b].y - polar[a].y) * f
        };
        let lower = self
            .rings
            .partition_point(|ring| ring.height > y)
            .min(self.rings.len() - 1);
        let upper = lower.saturating_sub(1);
        if lower == upper {
            return radius(upper);
        }
        let t = ((self.rings[upper].height - y)
            / (self.rings[upper].height - self.rings[lower].height))
            .clamp(0., 1.);
        radius(upper) + (radius(lower) - radius(upper)) * t
    }
}

struct Column {
    height: [f32; SAMPLES],
    radius: [f32; SAMPLES],
}
struct Field {
    profile: Profile,
    columns: Vec<Column>,
    hip: Mat4,
    waist: Vec<(f32, Mat4)>,
}
fn height(index: usize) -> f32 {
    TOP - (TOP - LOW) * index as f32 / (SAMPLES - 1) as f32
}

impl Field {
    fn new(
        rig: &PersonRig,
        points: &[[f32; 3]],
        joints: &[[u16; 4]],
        weights: &[[f32; 4]],
        palette: &[Mat4],
    ) -> Self {
        let profile = Profile::from_shell(points);
        let hip = palette[rig.hips_joint];
        let inverse = hip.inverse();
        let upper = rig.leg_joints.map(|leg| rig.skin_nodes[leg[0]]);
        let leg_joint: Vec<_> = rig
            .skin_nodes
            .iter()
            .map(|&node| {
                let mut cursor = Some(node);
                while let Some(node) = cursor {
                    if upper.contains(&node) {
                        return true;
                    }
                    cursor = rig.parents[node];
                }
                false
            })
            .collect();
        // Pose the supplied coarse leg cage with the actual LBS palette. This
        // retains the bent knee's real shape, including its volume loss. The
        // final refined body and outfit shoes still need collision verification.
        let cage: Vec<_> = rig
            .mesh
            .positions
            .iter()
            .zip(&rig.ids)
            .zip(&rig.weights)
            .filter(|((_, ids), weights)| {
                (0..4)
                    .filter(|&i| leg_joint[ids[i] as usize])
                    .map(|i| weights[i])
                    .sum::<f32>()
                    > 0.01
            })
            .map(|((&p, ids), weights)| {
                let posed: Vec3 = (0..4)
                    .map(|i| {
                        palette[ids[i] as usize].transform_point3(Vec3::from_array(p)) * weights[i]
                    })
                    .sum();
                inverse.transform_point3(posed) - Vec3::Z * CENTER_Z
            })
            .collect();
        let waist_row = profile
            .rings
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (a.height - TOP).abs().total_cmp(&(b.height - TOP).abs()))
            .unwrap()
            .0;
        let waist = (0..41)
            .map(|side| {
                let index = waist_row * 41 + side;
                let skin = (0..4)
                    .map(|slot| palette[joints[index][slot] as usize] * weights[index][slot])
                    .sum();
                (profile.rings[waist_row].polar[side].x, skin)
            })
            .collect();
        let mut columns = Vec::with_capacity(COLUMNS + 1);
        for side in 0..=COLUMNS {
            // The last column is bit-identical to the first, preserving seams.
            let angle = (side % COLUMNS) as f32 / COLUMNS as f32 * std::f32::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let direction = Vec2::new(sin, cos);
            let neutral: [f32; SAMPLES] = std::array::from_fn(|i| profile.radius(height(i), angle));
            let mut projected: Vec<_> = cage
                .iter()
                .map(|p| Vec2::new(p.y, p.xz().dot(direction) + 0.035))
                .collect();
            // The fitted waist and lower neutral profile close the convex
            // envelope smoothly across empty space above/below the leg cage.
            projected.push(Vec2::new(TOP, neutral[0]));
            projected.push(Vec2::new(LOW, neutral[SAMPLES - 1]));
            let hull = UpperHull::new(projected);
            let mut support = neutral;
            for i in 1..SAMPLES {
                if let Some(radius) = hull.support(height(i)) {
                    support[i] = support[i].max(radius);
                }
            }
            let mut arc = [0.; SAMPLES];
            let mut rest_arc = [0.; SAMPLES];
            for i in 1..SAMPLES {
                arc[i] =
                    arc[i - 1] + (support[i] - support[i - 1]).hypot(height(i) - height(i - 1));
                rest_arc[i] = rest_arc[i - 1]
                    + (neutral[i] - neutral[i - 1]).hypot(height(i) - height(i - 1));
            }
            let mut column = Column {
                height: [0.; SAMPLES],
                radius: [0.; SAMPLES],
            };
            for i in 0..SAMPLES {
                let below = arc.partition_point(|&s| s < rest_arc[i]).min(SAMPLES - 1);
                let above = below.saturating_sub(1);
                let t = if above == below {
                    0.
                } else {
                    ((rest_arc[i] - arc[above]) / (arc[below] - arc[above])).clamp(0., 1.)
                };
                column.height[i] = height(above) + (height(below) - height(above)) * t;
                column.radius[i] = support[above] + (support[below] - support[above]) * t;
            }
            columns.push(column);
        }
        Self {
            profile,
            columns,
            hip,
            waist,
        }
    }
    fn drape(&self, p: Vec3) -> Vec3 {
        let radial = Vec2::new(p.x, p.z - CENTER_Z);
        let angle = radial.x.atan2(radial.y).rem_euclid(std::f32::consts::TAU);
        let side = angle / std::f32::consts::TAU * COLUMNS as f32;
        let a = (side.floor() as usize).min(COLUMNS - 1);
        let f = side - a as f32;
        let row =
            ((TOP - p.y) / (TOP - LOW) * (SAMPLES - 1) as f32).clamp(0., (SAMPLES - 1) as f32);
        let upper = (row.floor() as usize).min(SAMPLES - 2);
        let t = row - upper as f32;
        let sample = |column: &Column| {
            Vec2::new(
                column.height[upper] + (column.height[upper + 1] - column.height[upper]) * t,
                column.radius[upper] + (column.radius[upper + 1] - column.radius[upper]) * t,
            )
        };
        let result = sample(&self.columns[a]).lerp(sample(&self.columns[a + 1]), f);
        let radius = result.y + radial.length() - self.profile.radius(p.y, angle);
        let (sin, cos) = angle.sin_cos();
        self.hip
            .transform_point3(Vec3::new(radius * sin, result.x, CENTER_Z + radius * cos))
    }
    fn map(&self, p: Vec3, skin: Mat4) -> Vec3 {
        if p.y >= TOP {
            return skin.transform_point3(p);
        }
        let radial = Vec2::new(p.x, p.z - CENTER_Z);
        let angle = radial.x.atan2(radial.y).rem_euclid(std::f32::consts::TAU);
        let b = self.waist.partition_point(|&(a, _)| a < angle).clamp(1, 40);
        let a = b - 1;
        let t = ((angle - self.waist[a].0) / (self.waist[b].0 - self.waist[a].0)).clamp(0., 1.);
        let waist_skin = self.waist[a].1 * (1. - t) + self.waist[b].1 * t;
        let radius =
            self.profile.radius(TOP, angle) + radial.length() - self.profile.radius(p.y, angle);
        let (sin, cos) = angle.sin_cos();
        let anchor = Vec3::new(radius * sin, TOP, CENTER_Z + radius * cos);
        let t = ((TOP - p.y) / 0.40).clamp(0., 1.);
        let fade = 1. - t * t * (3. - 2. * t);
        self.drape(p)
            + (waist_skin.transform_point3(anchor) - self.hip.transform_point3(anchor)) * fade
    }
}

/// Freeze only the review mesh at one actual production pose. Inverting its
/// current palette keeps the real Bevy material, UVs, skin path and camera.
pub(crate) fn freeze(key: &str, rig: &PersonRig, mesh: &mut Mesh, palette: &[Mat4]) {
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("trial positions");
    };
    let points = points.clone();
    let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("trial normals");
    };
    let normals = normals.clone();
    let Some(VertexAttributeValues::Uint16x4(joints)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
    else {
        panic!("trial joints");
    };
    let Some(VertexAttributeValues::Float32x4(weights)) =
        mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
    else {
        panic!("trial weights");
    };
    let field = Field::new(rig, &points, joints, weights, palette);
    let mut frozen = Vec::with_capacity(points.len());
    let mut frozen_normals = Vec::with_capacity(points.len());
    let mut target = Vec::with_capacity(points.len());
    for i in 0..points.len() {
        let skin: Mat4 = (0..4)
            .map(|j| palette[joints[i][j] as usize] * weights[i][j])
            .sum();
        let p = Vec3::from_array(points[i]);
        let posed = field.map(p, skin);
        let inverse = skin.inverse();
        assert!(
            inverse.is_finite(),
            "trial cannot invert collapsed source skin"
        );
        let bind = inverse.transform_point3(posed);
        assert!(
            skin.transform_point3(bind).distance(posed) < 0.0001,
            "frozen position must reproduce candidate"
        );
        let epsilon = 0.001;
        let derivative = |axis: Vec3| {
            (field.map(p + axis * epsilon, skin) - field.map(p - axis * epsilon, skin))
                / (2. * epsilon)
        };
        let jacobian = Mat3::from_cols(
            derivative(Vec3::X),
            derivative(Vec3::Y),
            derivative(Vec3::Z),
        );
        assert!(
            jacobian.is_finite() && jacobian.determinant() > 1e-6,
            "trial collapsed a fabric panel at {p:?}"
        );
        let normal = (jacobian.inverse().transpose() * Vec3::from_array(normals[i])).normalize();
        let bind_normal = (Mat3::from_mat4(skin).transpose() * normal).normalize();
        let reproduced = (Mat3::from_mat4(skin).inverse().transpose() * bind_normal).normalize();
        assert!(
            reproduced.dot(normal) > 0.9999,
            "frozen shading must reproduce candidate"
        );
        frozen.push(bind.to_array());
        frozen_normals.push(bind_normal.to_array());
        target.push(posed.to_array());
    }
    if let Some(directory) = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES") {
        std::fs::write(
            std::path::PathBuf::from(directory).join(format!("{key}.drape-trial-posed.json")),
            serde_json::to_vec(&serde_json::json!({"positions":target,
                "indices":mesh.indices().unwrap().iter().collect::<Vec<_>>(),
                "scope":"frozen native art trial; not production cloth animation"}))
            .unwrap(),
        )
        .unwrap();
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, frozen);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, frozen_normals);
}

pub(crate) fn validate_support() {
    validation::slices_match_independent_all_pairs_support();
}

mod validation {
    use super::*;
    pub(super) fn slices_match_independent_all_pairs_support() {
        for case in 0..12 {
            let mut points: Vec<_> = (0..32)
                .map(|i| {
                    Vec2::new(
                        (i % 9) as f32 * 0.13,
                        ((i * 17 + case * 7) as f32 * 0.83).sin() * 0.4,
                    )
                })
                .collect();
            points.extend([Vec2::new(-0.1, 0.1), Vec2::new(1.2, 0.2)]);
            let hull = UpperHull::new(points.clone());
            for sample in 0..=128 {
                let y = -0.1 + 1.3 * sample as f32 / 128.;
                let mut expected = f32::NEG_INFINITY;
                for a in &points {
                    for b in &points {
                        if a.x == y {
                            expected = expected.max(a.y);
                        }
                        if a.x < b.x && a.x <= y && y <= b.x {
                            expected = expected.max(a.y + (b.y - a.y) * (y - a.x) / (b.x - a.x));
                        }
                    }
                }
                let actual = hull.support(y).unwrap();
                assert!(
                    actual.is_finite() && (actual - expected).abs() < 0.00002,
                    "upper hull {actual} differs from pair support {expected} at {y}"
                );
            }
            assert!(hull.support(-0.2).is_none() && hull.support(1.3).is_none());
            for triple in hull.0.windows(3) {
                assert!((triple[1] - triple[0]).perp_dot(triple[2] - triple[1]) < 0.);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn projected_cage_slices_match_all_pairs_support() {
        super::validate_support();
    }
}
