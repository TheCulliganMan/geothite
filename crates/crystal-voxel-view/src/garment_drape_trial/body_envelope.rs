//! Conservative local envelope of the actual posed body and neutral cloth.
//! Convert support planes to radial intersections; a support distance
//! is not a radius. Geometry below the reachable hem is clipped before fitting.
use super::{CENTER_Z, TOP};
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub(super) struct Collider {
    faces: Vec<[Vec3; 3]>,
}

fn contact_corners(point: Vec3) -> [Vec3; 4] {
    // A radial offset alone leaves no clearance across or above a raised
    // shoe. Include its tangential and vertical extent so the rolled hem can
    // pass over the whole foot, including the inside of the fabric.
    let t = ((TOP - point.y) / 0.18).clamp(0., 1.);
    let padding = 0.045 * t * t * (3. - 2. * t);
    let radial = Vec2::new(point.x, point.z - CENTER_Z).normalize_or(Vec2::Y);
    let center = point + Vec3::new(radial.x, 0., radial.y) * padding;
    let tangent = Vec3::new(-radial.y, 0., radial.x) * padding;
    let vertical = Vec3::Y * padding;
    [
        center - tangent - vertical,
        center - tangent + vertical,
        center + tangent - vertical,
        center + tangent + vertical,
    ]
}

impl Collider {
    pub(super) fn from_parts(
        points: &[[f32; 3]],
        indices: &[u32],
        joints: &[[u16; 4]],
        weights: &[[f32; 4]],
        palette: &[Mat4],
        inverse: Mat4,
    ) -> Self {
        let mut posed = vec![None; points.len()];
        let faces: Vec<[Vec3; 3]> = indices
            .chunks_exact(3)
            .filter(|f| f.iter().all(|&i| points[i as usize][1] < TOP + 0.12))
            .map(|f| {
                [f[0], f[1], f[2]].map(|i| {
                    let i = i as usize;
                    *posed[i].get_or_insert_with(|| {
                        let q: Vec3 = (0..4)
                            .map(|slot| {
                                palette[joints[i][slot] as usize]
                                    .transform_point3(Vec3::from_array(points[i]))
                                    * weights[i][slot]
                            })
                            .sum();
                        inverse.transform_point3(q)
                    })
                })
            })
            .collect();
        assert!(
            !faces.is_empty(),
            "cloth collision requires the final lower body"
        );
        Self { faces }
    }
    pub(super) fn envelope(self, low: f32, neutral: &[Vec3]) -> Envelope {
        let mut body = Vec::new();
        for face in self.faces {
            let mut polygon = face.to_vec();
            for (height, above) in [(low, true), (TOP, false)] {
                let mut clipped = Vec::new();
                for edge in 0..polygon.len() {
                    let a = polygon[edge];
                    let b = polygon[(edge + 1) % polygon.len()];
                    let keep_a = if above { a.y >= height } else { a.y <= height };
                    let keep_b = if above { b.y >= height } else { b.y <= height };
                    if keep_a {
                        clipped.push(a);
                    }
                    if keep_a != keep_b {
                        clipped.push(a.lerp(b, (height - a.y) / (b.y - a.y)));
                    }
                }
                polygon = clipped;
            }
            body.extend(polygon);
        }
        // Adjacent triangles reuse the same posed corners. Remove only exact
        // duplicates before expanding them; otherwise each contact corner is
        // allocated and sorted repeatedly for every incident body triangle.
        body.sort_by(|a, b| {
            a.x.total_cmp(&b.x)
                .then(a.y.total_cmp(&b.y))
                .then(a.z.total_cmp(&b.z))
        });
        body.dedup_by(|a, b| a.to_array().map(f32::to_bits) == b.to_array().map(f32::to_bits));
        let mut points = neutral.to_vec();
        // The fitted waist stays pinned. Clearance grows continuously below
        // it; clipping still excludes feet below the hem's reach.
        points.extend(body.into_iter().flat_map(contact_corners));
        for p in &mut points {
            p.z -= CENTER_Z;
        }
        points.sort_by(|a, b| {
            a.x.total_cmp(&b.x)
                .then(a.y.total_cmp(&b.y))
                .then(a.z.total_cmp(&b.z))
        });
        points.dedup_by(|a, b| a.abs_diff_eq(*b, 0.000001));
        let planes = if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_FIXED_ENVELOPE").is_some() {
            support_planes(&points)
        } else {
            adaptive_planes(&points)
        };
        Envelope { planes }
    }
}

/// Exact edges of each projected upper hull replace a fixed set of slope
/// directions. A small bend between two fixed slopes otherwise becomes a
/// broad shelf. Keep all 72 azimuths and the same outward clearance.
fn adaptive_planes(points: &[Vec3]) -> Vec<(Vec3, f32)> {
    let mut ordered: Vec<_> = (0..points.len()).collect();
    ordered.sort_by(|&a, &b| points[a].y.total_cmp(&points[b].y));
    let mut planes = Vec::new();
    for side in 0..72 {
        let angle = side as f32 / 72. * TAU;
        let (sin, cos) = angle.sin_cos();
        let mut hull: Vec<(f64, f64, usize)> = Vec::new();
        for &i in &ordered {
            let p = points[i];
            let next = (
                f64::from(p.y),
                f64::from(p.x) * f64::from(sin) + f64::from(p.z) * f64::from(cos),
                i,
            );
            if let Some(previous) = hull.last() {
                if previous.0 == next.0 {
                    if previous.1 >= next.1 {
                        continue;
                    }
                    hull.pop();
                }
            }
            while hull.len() >= 2 {
                let a = hull[hull.len() - 2];
                let b = hull[hull.len() - 1];
                if (b.0 - a.0) * (next.1 - b.1) - (b.1 - a.1) * (next.0 - b.0) < 0. {
                    break;
                }
                hull.pop();
            }
            hull.push(next);
        }
        for pair in hull.windows(2) {
            let height = pair[1].0 - pair[0].0;
            let radius = pair[1].1 - pair[0].1;
            let length = height.hypot(radius);
            let normal = Vec3::new(
                (f64::from(sin) * height / length) as f32,
                (-radius / length) as f32,
                (f64::from(cos) * height / length) as f32,
            );
            let support = normal.dot(points[pair[0].2]).max(normal.dot(points[pair[1].2])) + 0.001;
            planes.push((normal, support));
        }
    }
    assert!(!planes.is_empty(), "garment envelope requires a vertical extent");
    planes
}

/// For each azimuth, positive radial support can only lie on the upper
/// (height, radial projection) hull. Keep indices into the complete body,
/// then evaluate the original 3D dot products at those candidates.
fn support_planes(points: &[Vec3]) -> Vec<(Vec3, f32)> {
    let mut ordered: Vec<_> = (0..points.len()).collect();
    ordered.sort_by(|&a, &b| points[a].y.total_cmp(&points[b].y));
    let mut planes = Vec::with_capacity(72 * 23);
    for side in 0..72 {
        let angle = side as f32 / 72. * TAU;
        let sin = angle.sin();
        let cos = angle.cos();
        let mut hull: Vec<(f64, f64, usize)> = Vec::new();
        for &index in &ordered {
            let p = points[index];
            let next = (
                f64::from(p.y),
                f64::from(p.x) * f64::from(sin) + f64::from(p.z) * f64::from(cos),
                index,
            );
            if let Some(previous) = hull.last() {
                if previous.0 == next.0 {
                    if previous.1 >= next.1 {
                        continue;
                    }
                    hull.pop();
                }
            }
            while hull.len() >= 2 {
                let a = hull[hull.len() - 2];
                let b = hull[hull.len() - 1];
                let cross = (b.0 - a.0) * (next.1 - b.1) - (b.1 - a.1) * (next.0 - b.0);
                if cross < 0. {
                    break;
                }
                hull.pop();
            }
            hull.push(next);
        }
        for slope in 1..24 {
            let vertical = -PI * 0.5 + PI * slope as f32 / 24.;
            let normal = Vec3::new(sin * vertical.cos(), vertical.sin(), cos * vertical.cos());
            let support = hull
                .iter()
                .map(|&(_, _, i)| normal.dot(points[i]))
                .fold(f32::NEG_INFINITY, f32::max)
                + 0.001;
            planes.push((normal, support));
        }
    }
    planes
}

// Independent full-point oracle; validation keeps the old support calculation.
fn support_planes_scan(points: &[Vec3]) -> Vec<(Vec3, f32)> {
    let mut planes = Vec::new();
    for side in 0..72 {
        let angle = side as f32 / 72. * TAU;
        for slope in 1..24 {
            let vertical = -PI * 0.5 + PI * slope as f32 / 24.;
            let normal = Vec3::new(
                angle.sin() * vertical.cos(),
                vertical.sin(),
                angle.cos() * vertical.cos(),
            );
            let support = points
                .iter()
                .map(|p| normal.dot(*p))
                .fold(f32::NEG_INFINITY, f32::max)
                + 0.001;
            planes.push((normal, support));
        }
    }
    planes
}

pub(super) struct Envelope {
    planes: Vec<(Vec3, f32)>,
}
impl Envelope {
    /// For a fixed ray, every support plane is an affine upper bound on the
    /// radius as a function of height. Their lower envelope is the same
    /// intersection as `radius`, without rescanning every plane at every row.
    pub(super) fn column(&self, point: Vec3) -> RadialColumn {
        let direction = Vec2::new(point.x, point.z - CENTER_Z).normalize_or(Vec2::Y);
        let mut lines: Vec<_> = self
            .planes
            .iter()
            .filter_map(|(normal, support)| {
                let denominator = normal.x * direction.x + normal.z * direction.y;
                (denominator > 0.00001).then(|| {
                    let denominator = f64::from(denominator);
                    (
                        -f64::from(normal.y) / denominator,
                        f64::from(*support) / denominator,
                    )
                })
            })
            .collect();
        // Descending slopes are active from low to high heights. Equal slopes
        // keep the tightest intercept, including either sign of zero.
        lines.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.total_cmp(&b.1)));
        let mut hull: Vec<Line> = Vec::new();
        for (slope, intercept) in lines {
            if hull.last().is_some_and(|line| line.slope == slope) {
                continue;
            }
            let mut start = f64::NEG_INFINITY;
            while let Some(previous) = hull.last() {
                start = (intercept - previous.intercept) / (previous.slope - slope);
                if start > previous.start {
                    break;
                }
                hull.pop();
            }
            if hull.is_empty() {
                start = f64::NEG_INFINITY;
            }
            hull.push(Line {
                start,
                slope,
                intercept,
            });
        }
        RadialColumn(hull)
    }

    // Keep the direct plane scan as an independent oracle for the optimized
    // column lookup. Its denominator threshold and clamp define the query.
    pub(super) fn radius(&self, point: Vec3) -> f32 {
        let direction = Vec2::new(point.x, point.z - CENTER_Z).normalize_or(Vec2::Y);
        self.planes
            .iter()
            .filter_map(|(normal, support)| {
                let denominator = normal.x * direction.x + normal.z * direction.y;
                (denominator > 0.00001).then(|| (support - normal.y * point.y) / denominator)
            })
            .fold(f32::INFINITY, f32::min)
            .max(0.)
    }
}

struct Line {
    start: f64,
    slope: f64,
    intercept: f64,
}
pub(super) struct RadialColumn(Vec<Line>);
impl RadialColumn {
    pub(super) fn radius(&self, height: f32) -> f32 {
        assert!(height.is_finite(), "cloth column requires a finite height");
        if self.0.is_empty() {
            return f32::INFINITY;
        }
        let height = f64::from(height);
        let index = self
            .0
            .partition_point(|line| line.start <= height)
            .saturating_sub(1);
        let line = &self.0[index];
        (line.intercept + line.slope * height).max(0.) as f32
    }
}

pub(super) fn validate() {
    // An off-axis box radius is a ray/plane intersection. Treating its
    // directional support as a radius inflates the diagonal silhouette.
    let box_envelope = Envelope {
        planes: vec![
            (Vec3::X, 0.3),
            (Vec3::NEG_X, 0.3),
            (Vec3::Z, 0.5),
            (Vec3::NEG_Z, 0.5),
        ],
    };
    let diagonal = box_envelope.radius(Vec3::new(1., 0.8, CENTER_Z + 1.));
    assert!((diagonal - 0.3 * 2_f32.sqrt()).abs() < 0.00001);
    assert!((diagonal - (0.3 + 0.5) / 2_f32.sqrt()).abs() > 0.1);
    let neutral = [
        Vec3::new(-0.4, 0.8, CENTER_Z - 0.4),
        Vec3::new(0.4, 0.8, CENTER_Z + 0.4),
        Vec3::new(-0.4, TOP, CENTER_Z + 0.4),
        Vec3::new(0.4, TOP, CENTER_Z - 0.4),
    ];
    let empty = Collider { faces: vec![] }.envelope(0.8, &neutral);
    let below = Collider {
        faces: vec![[
            Vec3::new(20., 0.2, 0.),
            Vec3::new(20., 0.1, 1.),
            Vec3::new(21., 0.1, 0.),
        ]],
    }
    .envelope(0.8, &neutral);
    assert_eq!(
        empty.planes, below.planes,
        "a foot below the reachable hem widens the garment"
    );
    validate_contact_clearance();
    validate_columns();
    validate_adaptive_hulls();
}

fn validate_adaptive_hulls() {
    // A deliberately sharp change in occupied radius falls between the fixed
    // slope directions. Exact projected edges must retain that small bend.
    let mut wedge = Vec::new();
    for (y, radius) in [(0., 0.42), (0.47, 0.42), (1., 0.07)] {
        for x in [-radius, radius] {
            for z in [-radius, radius] {
                wedge.push(Vec3::new(x, y, z));
            }
        }
    }
    let exact = Envelope { planes: adaptive_planes(&wedge) };
    let fixed = Envelope { planes: support_planes(&wedge) };
    let point = Vec3::new(0., 0.47, CENTER_Z + 1.);
    assert!((exact.radius(point) - 0.42).abs() < 0.002);
    let middle = Vec3 { y: 0.60, ..point };
    let wanted = 0.42 + (0.07 - 0.42) * (0.60 - 0.47) / (1. - 0.47);
    assert!((exact.radius(middle) - wanted).abs() < 0.002);
    assert!(fixed.radius(middle) - exact.radius(middle) > 0.002,
        "fixture must expose the fixed-slope shelf");
    for seed in 0..6 {
        let points: Vec<_> = (0..128).map(|i| {
            let angle = (i * 31 + seed * 17) as f32 * 0.173;
            let radius = 0.25 + 0.55 * (angle * 0.7).sin().abs();
            Vec3::new(radius * angle.sin(), 0.25 + (i % 23) as f32 * 0.061,
                radius * angle.cos())
        }).collect();
        let envelope = Envelope { planes: adaptive_planes(&points) };
        for &(normal, support) in &envelope.planes {
            assert!(normal.is_finite() && (normal.length() - 1.).abs() < 0.00001);
            let all_point_support = points.iter().map(|p| normal.dot(*p))
                .fold(f32::NEG_INFINITY, f32::max);
            assert!((support - all_point_support - 0.001).abs() < 0.000002,
                "projected edge is not a supporting plane");
        }
        for side in 0..48 {
            let angle = side as f32 / 48. * TAU;
            let point = Vec3::new(angle.sin(), 0., CENTER_Z + angle.cos());
            let column = envelope.column(point);
            for row in 0..128 {
                let height = 0.25 + 1.2 * row as f32 / 128.;
                assert!((column.radius(height) - envelope.radius(Vec3 { y: height, ..point })).abs() < 0.00002);
            }
        }
    }
}

fn validate_contact_clearance() {
    let toe = Vec3::new(0.25, 0.72, CENTER_Z + 0.90);
    let radial = Vec3::new(toe.x, 0., toe.z - CENTER_Z).normalize();
    let tangent = Vec3::new(-radial.z, 0., radial.x);
    let envelope = Collider {
        faces: vec![[toe, toe + tangent * 0.001, toe + Vec3::Y * 0.001]],
    }
    .envelope(0.3, &[toe]);
    let centered = toe - Vec3::Z * CENTER_Z;
    // Require the same clearance in every outward-facing plane, including
    // oblique planes above and beside the shoe. A radial-only offset fails.
    for (normal, support) in envelope.planes {
        if normal.dot(radial) >= 0. {
            assert!(
                support - normal.dot(centered) >= 0.045 - 0.00001,
                "shoe clearance is missing in direction {normal:?}"
            );
        }
    }
    let waist = Vec3::new(0.25, TOP, CENTER_Z + 0.25);
    assert_eq!(contact_corners(waist), [waist; 4]);
}

fn validate_columns() {
    // Deliberately include duplicate slopes, signed zero, near-parallel rays
    // and line switches. The oracle still scans the original 3D planes.
    let mut cases = vec![Envelope {
        planes: vec![
            (Vec3::new(1., 0., 0.), 0.6),
            (Vec3::new(1., -0., 0.), 0.4),
            (Vec3::new(1., 1., 0.), 1.4),
            (Vec3::new(1., -1., 0.), -0.2),
            (Vec3::new(0.000009, 0.5, 0.), 0.7),
            (Vec3::new(0.000010, 0.5, 0.), 0.7),
            (Vec3::new(0.000011, 0.5, 0.), 0.7),
            (Vec3::Z, 0.5),
            (Vec3::NEG_Z, 0.7),
            (Vec3::NEG_X, 0.5),
        ],
    }];
    for seed in 0..4 {
        let points: Vec<_> = (0..64)
            .map(|i| {
                let angle = (i * 31 + seed * 17) as f32 * 0.173;
                let radius = 0.25 + 0.55 * (angle * 0.7).sin().abs();
                Vec3::new(
                    radius * angle.sin(),
                    0.25 + (i % 13) as f32 * 0.11,
                    radius * angle.cos(),
                )
            })
            .collect();
        let planes = support_planes(&points);
        let scanned = support_planes_scan(&points);
        for ((normal, support), (old_normal, old_support)) in planes.iter().zip(scanned) {
            assert_eq!(*normal, old_normal);
            assert!(
                (support - old_support).abs() < 0.000002,
                "projected hull support {support} differs from point scan {old_support}"
            );
        }
        cases.push(Envelope { planes });
    }
    for envelope in cases {
        for side in 0..=48 {
            let angle = side as f32 / 48. * TAU;
            let point = Vec3::new(angle.sin(), 0., CENTER_Z + angle.cos());
            let column = envelope.column(point);
            for row in 0..=128 {
                let height = 0.2 + 1.5 * row as f32 / 128.;
                let expected = envelope.radius(Vec3 { y: height, ..point });
                let actual = column.radius(height);
                assert!(
                    (expected - actual).abs() < 0.00002,
                    "radial column {actual} differs from plane scan {expected} at {point:?}, y={height}"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn contact_envelope_uses_radial_intersections_and_clips_unreachable_geometry() {
        super::validate();
    }
}
