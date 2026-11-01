//! Individual long-hair cuts, carried by the head at the roots and upper back
//! below the nape. They reuse the person's painting and ordinary GPU skin.

use crate::hair_paint::HairPaint;
use crate::hair_scalp::HairScalp;
use crate::human_idle::HumanIdle;
use crate::human_locomotion::{self, HumanLocomotion};
use crate::mesh::SurfaceMeshData;
use crate::new_bark_actors::character_meshes::CharacterKind;
use crate::npc_appearance::Look;
use crate::open_people::PersonRig;
use bevy::math::{Mat4, Vec3};
use bevy::transform::components::Transform;
use std::f32::consts::{PI, TAU};

const ROWS: usize = 48;
const SIDES: usize = 48;
const THICKNESS: f32 = 0.024;

pub(crate) fn enabled(look: Look) -> bool {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_CURTAIN_BASELINE").is_some() {
        return false;
    }
    look.hairstyle == "long"
}

#[derive(Clone, Copy)]
struct Cut {
    length: f32,
    width: f32,
    depth: f32,
    sweep: f32,
    tips: f32,
    locks: f32,
    phase: f32,
}

impl Cut {
    fn for_look(look: Look) -> Self {
        // Crystal's trainer pictures establish the cut; the matching overworld
        // front/back/profile frames establish its rear and side silhouette.
        match look.texture_key {
            "beauty" => Self {
                length: 1.08,
                width: 1.02,
                depth: 0.96,
                sweep: 0.12,
                tips: 0.22,
                locks: 5.,
                phase: 0.3,
            },
            "clair" => Self {
                length: 2.02,
                width: 0.88,
                depth: 1.14,
                sweep: 0.36,
                tips: 0.34,
                locks: 5.,
                phase: -0.4,
            },
            "daisy" => Self {
                length: 1.70,
                width: 0.88,
                depth: 0.99,
                sweep: 0.19,
                tips: 0.12,
                locks: 6.,
                phase: 0.2,
            },
            "jasmine" => Self {
                length: 2.75,
                width: 0.86,
                depth: 1.00,
                sweep: 0.28,
                tips: 0.36,
                locks: 7.,
                phase: 0.5,
            },
            "karen" => Self {
                length: 2.08,
                width: 0.99,
                depth: 1.07,
                sweep: 0.24,
                tips: 0.40,
                locks: 5.,
                phase: -0.2,
            },
            "rocket_girl" => Self {
                length: 1.42,
                width: 0.89,
                depth: 1.00,
                sweep: 0.18,
                tips: 0.11,
                locks: 6.,
                phase: 0.0,
            },
            "sabrina" => Self {
                length: 2.62,
                width: 0.78,
                depth: 1.01,
                sweep: 0.30,
                tips: 0.30,
                locks: 5.,
                phase: 0.1,
            },
            _ => unreachable!("only the seven authored long-hair cuts use this surface"),
        }
    }
}

struct HairCurtain {
    center: Vec3,
    half: Vec3,
    pivot: Vec3,
    cut: Cut,
    head_scale: f32,
    paired_gathers: bool,
    scalp: Option<HairScalp>,
}

fn smooth(value: f32) -> f32 {
    let t = value.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

impl HairCurtain {
    fn new(
        rig: &PersonRig,
        look: Look,
        head_scale: f32,
        mesh: &SurfaceMeshData,
        joints: &[[u16; 4]],
        weights: &[[f32; 4]],
    ) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for ((&p, ids), weights) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
            let head: f32 = (0..4)
                .filter(|&slot| usize::from(ids[slot]) == rig.head_joint)
                .map(|slot| weights[slot])
                .sum();
            if head > 0.5 {
                min = min.min(Vec3::from_array(p));
                max = max.max(Vec3::from_array(p));
            }
        }
        assert!(
            min.is_finite() && max.is_finite(),
            "long hair requires a weighted head"
        );
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        Self {
            center,
            half,
            pivot: rig.inverse_binds[rig.head_joint]
                .inverse()
                .transform_point3(Vec3::ZERO),
            cut: Cut::for_look(look),
            head_scale,
            paired_gathers: look.texture_key == "jasmine",
            scalp: Some(HairScalp::new(
                mesh,
                joints,
                weights,
                rig.head_joint,
                center,
                half.y,
            )),
        }
    }

    fn profile(&self, t: f32) -> Vec3 {
        let cut = self.cut;
        let knots = [
            Vec3::new(0.94, 0.20, 0.24),
            Vec3::new(0.73, 0.82, 0.88),
            Vec3::new(0.40, 1.03, 1.08),
            Vec3::new(0.08, 1.12, 1.12),
            Vec3::new(-0.35, 1.10, 1.08),
            Vec3::new(-0.70, 1.06, 1.04),
            Vec3::new(-cut.length * 0.82, cut.width + 0.10, cut.depth + 0.04),
            Vec3::new(-cut.length, cut.width, cut.depth),
        ];
        let q = t.clamp(0., 1.) * (knots.len() - 1) as f32;
        let i = (q.floor() as usize).min(knots.len() - 2);
        let f = q - i as f32;
        let a = knots[i];
        let b = knots[i + 1];
        let before = if i == 0 { a - (b - a) } else { knots[i - 1] };
        let after = if i + 2 == knots.len() {
            b + (b - a)
        } else {
            knots[i + 2]
        };
        let slope = |a: Vec3, b: Vec3| {
            let limited = |x: f32, y: f32| {
                if x * y <= 0. {
                    0.
                } else {
                    x.signum() * ((x.abs() + y.abs()) * 0.5).min(2. * x.abs().min(y.abs()))
                }
            };
            Vec3::new(limited(a.x, b.x), limited(a.y, b.y), limited(a.z, b.z))
        };
        a * (2. * f.powi(3) - 3. * f * f + 1.)
            + slope(a - before, b - a) * (f.powi(3) - 2. * f * f + f)
            + b * (-2. * f.powi(3) + 3. * f * f)
            + slope(b - a, after - b) * (f.powi(3) - f * f)
    }

    fn head_weight(t: f32) -> f32 {
        1. - 0.78 * smooth((t - 0.35) / 0.65)
    }

    fn angle(t: f32, u: f32) -> f32 {
        let flow = smooth((t - 0.35) / 0.65);
        let opening = PI * (0.34 + 0.27 * flow);
        opening + (TAU - 2. * opening) * u + 0.07 * flow * (TAU * u).sin()
    }

    fn color(&self, paint: &HairPaint<'_>, t: f32, u: f32) -> [f32; 4] {
        // The rear scalp's atlas seam lies at the two edges of the portrait.
        // Use the inspected left hair strip on both sides of the new curtain,
        // keeping interpolation away from the face/ear islands. Its painted
        // crown waves turn into longitudinal locks below the nape.
        let across = (2. * u - 1.).abs();
        let drift = 0.018 * (PI * t).sin() * (TAU * across).sin();
        let strand = (across + drift).clamp(0., 1.);
        let crown = paint.hair([
            0.012 + 0.11 * across,
            (0.23 - 0.15 * self.profile(t).x).clamp(0.075, 0.225),
        ]);
        let source = paint.hair([0.012 + 0.11 * strand, 0.085 + 0.125 * strand]);
        let blend = smooth((t - 0.15) / 0.35);
        let rgb = std::array::from_fn(|i| {
            let lock = paint.base[i] * 0.70 + source[i] * 0.30;
            crown[i] + (lock - crown[i]) * blend
        });
        paint.color(rgb)
    }

    fn raw_point(&self, t: f32, u: f32, inner: bool) -> Vec3 {
        let profile = self.profile(t);
        let flow = smooth((t - 0.35) / 0.65);
        let angle = Self::angle(t, u);
        let lock = (angle * self.cut.locks + self.cut.phase + 0.5 * flow).cos();
        let ridge = 1. + 0.070 * (PI * t).sin().powi(2) * lock;
        let end = smooth((t - 0.72) / 0.28);
        let taper = 1. - 0.10 * end * (0.5 + 0.5 * lock);
        let thickness = if inner { THICKNESS } else { 0. };
        let width = self.half.x * profile.y * ridge * taper - thickness;
        let depth = self.half.z * profile.z * ridge * taper - thickness;
        self.center
            + Vec3::new(
                width * angle.sin(),
                self.half.y * (profile.x - self.cut.tips * end * (0.5 + 0.5 * lock)),
                depth * angle.cos() - 0.025 - self.half.z * self.cut.sweep * flow,
            )
    }

    fn surface_point(&self, t: f32, u: f32, inner: bool) -> Vec3 {
        let point = self.raw_point(t, u, inner);
        let Some(scalp) = &self.scalp else {
            return point;
        };
        let outer = self.raw_point(t, u, false);
        // Apply one displacement to both layers; fitting each separately
        // would collapse the thin inner/outer shell against the scalp.
        point + scalp.fit(outer) - outer
    }

    fn point(&self, t: f32, u: f32, inner: bool) -> Vec3 {
        let point = self.surface_point(t, u, inner);
        // Head presentation scales the scalp, while the upper back keeps its
        // original scale. Invert their neutral blend once at preparation, so
        // changing carriers does not enlarge the lower hair into a cloak.
        let head = Self::head_weight(t);
        let scale = head * self.head_scale + 1. - head;
        self.pivot + (point - self.pivot) * (self.head_scale / scale)
    }

    fn normal(&self, t: f32, u: f32, inner: bool) -> Vec3 {
        // Differentiate the presented surface, rather than its inverse skin.
        // The latter adds an artificial ring as the two carrier scales blend.
        let across = self.surface_point(t, (u + 0.0005).min(1.), inner)
            - self.surface_point(t, (u - 0.0005).max(0.), inner);
        let down = self.surface_point((t + 0.0005).min(1.), u, inner)
            - self.surface_point((t - 0.0005).max(0.), u, inner);
        down.cross(across).normalize() * if inner { -1. } else { 1. }
    }

    fn append(
        &self,
        rig: &PersonRig,
        paint: &HairPaint<'_>,
        mesh: &mut SurfaceMeshData,
        joints: &mut Vec<[u16; 4]>,
        weights: &mut Vec<[f32; 4]>,
    ) {
        let first = mesh.positions.len();
        let layer = (ROWS + 1) * (SIDES + 1);
        for inner in [false, true] {
            for row in 0..=ROWS {
                for col in 0..=SIDES {
                    let t = row as f32 / ROWS as f32;
                    let u = col as f32 / SIDES as f32;
                    mesh.positions.push(self.point(t, u, inner).to_array());
                    mesh.normals.push(self.normal(t, u, inner).to_array());
                    mesh.colors.push(self.color(paint, t, u));
                    mesh.uvs.push(paint.uv);
                    joints.push([rig.head_joint as u16, rig.chest_joint as u16, 0, 0]);
                    let head = Self::head_weight(t);
                    weights.push([head, 1. - head, 0., 0.]);
                }
            }
        }
        let quad = |mesh: &mut SurfaceMeshData, ids: [usize; 4]| {
            let [a, b, c, d] = ids.map(|i| (first + i) as u32);
            mesh.indices.extend([a, b, c, a, c, d]);
        };
        for row in 0..ROWS {
            for col in 0..SIDES {
                let a = row * (SIDES + 1) + col;
                let b = a + SIDES + 1;
                quad(mesh, [a, b, b + 1, a + 1]);
                quad(mesh, [a + layer, a + layer + 1, b + layer + 1, b + layer]);
            }
        }
        let mut rim = |ids: [usize; 4]| {
            let ids = [ids[0], ids[3], ids[2], ids[1]].map(|i| first + i);
            let p = ids.map(|i| Vec3::from_array(mesh.positions[i]));
            let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize().to_array();
            let base = mesh.positions.len() as u32;
            for i in ids {
                mesh.positions.push(mesh.positions[i]);
                mesh.normals.push(normal);
                mesh.colors.push(mesh.colors[i]);
                mesh.uvs.push(mesh.uvs[i]);
                joints.push(joints[i]);
                weights.push(weights[i]);
            }
            mesh.indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        };
        for col in 0..SIDES {
            rim([col, col + layer, col + layer + 1, col + 1]);
            let a = ROWS * (SIDES + 1) + col;
            rim([a, a + 1, a + layer + 1, a + layer]);
        }
        for row in 0..ROWS {
            let a = row * (SIDES + 1);
            let b = a + SIDES + 1;
            rim([a, b, b + layer, a + layer]);
            rim([a + SIDES, a + SIDES + layer, b + SIDES + layer, b + SIDES]);
        }
        if self.paired_gathers {
            for side in [-1., 1.] {
                self.append_gather(rig, paint, mesh, joints, weights, side);
            }
        }
    }

    /// Jasmine's two raised side gathers are visible in both the trainer and
    /// overworld pictures. Their broad roots enter the crown; the tips turn
    /// upward and back instead of becoming detached cones above the head.
    fn append_gather(
        &self,
        rig: &PersonRig,
        paint: &HairPaint<'_>,
        mesh: &mut SurfaceMeshData,
        joints: &mut Vec<[u16; 4]>,
        weights: &mut Vec<[f32; 4]>,
        side: f32,
    ) {
        const RINGS: usize = 24;
        const COLUMNS: usize = 16;
        let controls = [
            Vec3::new(0.63, 0.60, -0.15),
            Vec3::new(0.94, 0.72, -0.18),
            Vec3::new(1.20, 1.00, -0.22),
            Vec3::new(1.06, 1.08, -0.28),
        ]
        .map(|p| self.center + p * self.half * Vec3::new(side, 1., 1.));
        let center = |t: f32| {
            let s = 1. - t;
            controls[0] * s.powi(3)
                + controls[1] * (3. * s * s * t)
                + controls[2] * (3. * s * t * t)
                + controls[3] * t.powi(3)
        };
        let tangent =
            |t: f32| (center((t + 0.001).min(1.)) - center((t - 0.001).max(0.))).normalize();
        let point = |t: f32, angle: f32| {
            let direction = tangent(t);
            let u = (Vec3::Z - direction * direction.z).normalize();
            let v = direction.cross(u);
            let radius = 0.065 * (1. - t) + 0.060 * (PI * t).sin().max(0.).sqrt() + 0.006 * t;
            center(t) + (u * angle.cos() * 0.80 + v * angle.sin()) * radius
        };
        let first = mesh.positions.len() as u32;
        for row in 0..=RINGS {
            for col in 0..=COLUMNS {
                let t = row as f32 / RINGS as f32;
                let angle = TAU * col as f32 / COLUMNS as f32;
                let across = point(t, angle + 0.001) - point(t, angle - 0.001);
                let down = point((t + 0.001).min(1.), angle) - point((t - 0.001).max(0.), angle);
                mesh.positions.push(point(t, angle).to_array());
                mesh.normals.push(across.cross(down).normalize().to_array());
                mesh.colors.push(if row == 6 {
                    paint.color(paint.sample([0.075, 0.75]))
                } else {
                    paint.color(paint.hair([
                        0.02 + 0.04 * col as f32 / COLUMNS as f32,
                        0.10 + 0.05 * angle.cos(),
                    ]))
                });
                mesh.uvs.push(paint.uv);
                joints.push([rig.head_joint as u16, 0, 0, 0]);
                weights.push([1., 0., 0., 0.]);
            }
        }
        // Split both ribbon boundaries to keep the two pigment fields crisp.
        // Copies retain identical geometry and head weights; the thin ivory
        // ties reuse Jasmine's cloth patch without an extra mesh or material.
        let band_rows: [[u32; COLUMNS + 1]; 2] = [5, 7].map(|row| {
            std::array::from_fn(|col| {
                let i = first as usize + row * (COLUMNS + 1) + col;
                let index = mesh.positions.len() as u32;
                mesh.positions.push(mesh.positions[i]);
                mesh.normals.push(mesh.normals[i]);
                mesh.colors.push(paint.color(paint.sample([0.075, 0.75])));
                mesh.uvs.push(paint.uv);
                joints.push(joints[i]);
                weights.push(weights[i]);
                index
            })
        });
        for row in 0..RINGS {
            for col in 0..COLUMNS {
                let at = |r: usize, c: usize| {
                    if (5..7).contains(&row) && r == 5 {
                        band_rows[0][c]
                    } else if (5..7).contains(&row) && r == 7 {
                        band_rows[1][c]
                    } else {
                        first + (r * (COLUMNS + 1) + c) as u32
                    }
                };
                let (a, b, c, d) = (
                    at(row, col),
                    at(row, col + 1),
                    at(row + 1, col),
                    at(row + 1, col + 1),
                );
                mesh.indices.extend([a, b, c, b, d, c]);
            }
        }
        for (row, reverse) in [(0, true), (RINGS, false)] {
            let t = row as f32 / RINGS as f32;
            let pole = mesh.positions.len() as u32;
            mesh.positions.push(center(t).to_array());
            mesh.normals
                .push((tangent(t) * if reverse { -1. } else { 1. }).to_array());
            mesh.colors.push(paint.color(paint.hair([0.04, 0.05])));
            mesh.uvs.push(paint.uv);
            joints.push([rig.head_joint as u16, 0, 0, 0]);
            weights.push([1., 0., 0., 0.]);
            for col in 0..COLUMNS {
                let a = first + (row * (COLUMNS + 1) + col) as u32;
                mesh.indices.extend(if reverse {
                    [pole, a + 1, a]
                } else {
                    [pole, a, a + 1]
                });
            }
        }
    }
}

pub(crate) struct Envelope {
    pub(crate) min: Vec3,
    pub(crate) max: Vec3,
}

impl Envelope {
    fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    fn include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
}

fn motion_envelope(
    rig: &PersonRig,
    kind: CharacterKind,
    head_scale: f32,
    points: &[[f32; 3]],
    joints: &[[u16; 4]],
    weights: &[[f32; 4]],
) -> Envelope {
    let carriers = [rig.head_joint, rig.chest_joint];
    let mut boxes = [Envelope::empty(), Envelope::empty()];
    for ((&p, ids), weights) in points.iter().zip(joints).zip(weights) {
        for (index, &joint) in carriers.iter().enumerate() {
            if (0..4).any(|s| usize::from(ids[s]) == joint && weights[s] > 0.) {
                boxes[index].include(Vec3::from_array(p));
            }
        }
    }
    let mut result = Envelope::empty();
    let mut include = |pose: &[Transform]| {
        let inverse = human_locomotion::world(rig, pose, rig.mesh_node).inverse();
        for (&joint, bounds) in carriers.iter().zip(&boxes) {
            let scale = if joint == rig.head_joint {
                head_scale
            } else {
                1.
            };
            let matrix = inverse
                * human_locomotion::world(rig, pose, rig.skin_nodes[joint])
                * Mat4::from_scale(Vec3::splat(scale))
                * rig.inverse_binds[joint];
            for x in [bounds.min.x, bounds.max.x] {
                for y in [bounds.min.y, bounds.max.y] {
                    for z in [bounds.min.z, bounds.max.z] {
                        result.include(matrix.transform_point3(Vec3::new(x, y, z)));
                    }
                }
            }
        }
    };
    let mut pose = rig.nodes.clone();
    for clip in ["Idle", "Run", "Jump"] {
        for frame in 0..=60 {
            rig.sample(clip, rig.duration(clip) * frame as f32 / 60., &mut pose);
            include(&pose);
        }
    }
    let gait = HumanLocomotion::for_character(rig, kind);
    for run in [0., 0.25, 0.5, 0.75, 1.] {
        for frame in 0..64 {
            gait.sample(rig, frame as f32 / 64. * TAU, run, &mut pose);
            include(&pose);
        }
    }
    let idle = HumanIdle::new(rig, kind);
    for frame in 0..24 {
        idle.sample(rig, &gait, frame as f32 * 1.83, &mut pose);
        include(&pose);
    }
    let margin = Vec3::splat(rig.height * 0.015);
    result.min -= margin;
    result.max += margin;
    result
}

/// Add the authored cut after body fitting, preserving all existing vertices,
/// material splits and skin weights. No additional skeleton joints are needed.
pub(crate) fn append(
    rig: &PersonRig,
    kind: CharacterKind,
    head_scale: f32,
    mesh: &mut SurfaceMeshData,
    joints: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) -> Option<Envelope> {
    let look = crate::npc_appearance::look(kind);
    if !enabled(look) {
        return None;
    }
    let first = mesh.positions.len();
    let painting = crate::npc_textures::get(look.texture_key)
        .expect("long hair requires its validated character painting");
    let paint = HairPaint::new(painting);
    let curtain = HairCurtain::new(rig, look, head_scale, mesh, joints, weights);
    curtain.append(rig, &paint, mesh, joints, weights);
    Some(motion_envelope(
        rig,
        kind,
        head_scale,
        &mesh.positions[first..],
        &joints[first..],
        &weights[first..],
    ))
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod validation {
    use std::collections::BTreeMap;
    use std::f32::consts::TAU;

    use bevy::math::{Mat4, Vec3};
    use bevy::transform::components::Transform;

    use super::append;
    #[cfg(test)]
    use super::{Cut, HairCurtain};
    use crate::hair_paint::HairPaint;
    use crate::human_idle::HumanIdle;
    use crate::human_locomotion::{self, HumanLocomotion};
    use crate::new_bark_actors::character_meshes::CharacterKind;

    #[cfg(test)]
    #[test]
    fn normal_follows_the_presented_surface_through_the_carrier_scale_transition() {
        let curtain = HairCurtain {
            center: Vec3::new(0., 3.18, 0.),
            half: Vec3::new(0.44, 0.58, 0.52),
            pivot: Vec3::new(0., 2.75, 0.),
            cut: Cut::for_look(crate::npc_appearance::look(CharacterKind::Karen)),
            head_scale: 0.72,
            paired_gathers: false,
            scalp: None,
        };
        let head = Mat4::from_translation(curtain.pivot)
            * Mat4::from_scale(Vec3::splat(curtain.head_scale))
            * Mat4::from_translation(-curtain.pivot);
        for inner in [false, true] {
            let presented = |t: f32, u: f32| {
                let p = curtain.point(t, u, inner);
                let weight = HairCurtain::head_weight(t);
                head.transform_point3(p) * weight + p * (1. - weight)
            };
            for row in 1..48 {
                let t = row as f32 / 48.;
                for col in 1..12 {
                    let u = col as f32 / 12.;
                    let across = presented(t, u + 0.0005) - presented(t, u - 0.0005);
                    let down = presented(t + 0.0005, u) - presented(t - 0.0005, u);
                    let actual = down.cross(across).normalize() * if inner { -1. } else { 1. };
                    assert!(
                        1. - actual.dot(curtain.normal(t, u, inner)) < 0.00002,
                        "hair shading ring at row={row}, col={col}, inner={inner}"
                    );
                }
            }
        }
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied human rigs")]
    pub(crate) fn supplied_long_hair_keeps_closed_cuts_and_bends_between_head_and_back() {
        use CharacterKind::{Beauty, Clair, Daisy, Jasmine, Karen, RocketGirl, Sabrina};
        let mut poses = 0;
        let mut minimum_normal = 1_f32;
        for kind in [Beauty, Clair, Daisy, Jasmine, Karen, RocketGirl, Sabrina] {
            let look = crate::npc_appearance::look(kind);
            let rig = crate::open_people::get(look.base).expect("supplied hair rig");
            let mut mesh = rig.mesh.clone();
            let mut joints = rig.ids.clone();
            let mut weights = rig.weights.clone();
            let original = mesh.clone();
            let old_points = mesh.positions.len();
            let old_faces = mesh.indices.len();
            let bounds = append(rig, kind, 0.72, &mut mesh, &mut joints, &mut weights)
                .expect("the production body must contain its long-hair cut");
            assert_eq!(mesh.positions[..old_points], original.positions);
            assert_eq!(mesh.normals[..old_points], original.normals);
            assert_eq!(mesh.colors[..old_points], original.colors);
            assert_eq!(mesh.uvs[..old_points], original.uvs);
            assert_eq!(mesh.indices[..old_faces], original.indices);
            assert_eq!(joints[..old_points], rig.ids);
            assert_eq!(weights[..old_points], rig.weights);
            assert_eq!(mesh.cutaway_ranges, original.cutaway_ranges);
            let points = &mesh.positions[old_points..];
            let normals = &mesh.normals[old_points..];
            let joints = &joints[old_points..];
            let weights = &weights[old_points..];
            assert!(
                points.len() < 7_500,
                "{} hair vertex budget",
                look.texture_key
            );
            assert!(weights.iter().any(|w| w[0] == 1.), "occupied scalp roots");
            assert!(
                weights.iter().any(|w| w[1] > 0.7),
                "lower hair must follow the back"
            );
            let painting =
                crate::npc_textures::get(look.texture_key).expect("supplied hair painting");
            let paint = HairPaint::new(painting);
            assert!(
                paint.neutral.iter().all(|&v| v > 0.90),
                "{} needs its protected neutral portrait texel",
                look.texture_key
            );
            assert!(mesh.uvs[old_points..].iter().all(|&uv| uv == paint.uv));
            assert!(
                mesh.colors[old_points..]
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite() && (0. ..=1.).contains(v))
            );
            for i in 0..points.len() {
                assert!(Vec3::from_array(points[i]).is_finite());
                assert!((Vec3::from_array(normals[i]).length() - 1.).abs() < 1e-4);
                assert!((weights[i].iter().sum::<f32>() - 1.).abs() < 1e-6);
                assert!(
                    weights[i]
                        .iter()
                        .all(|&w| w.is_finite() && (0. ..=1.).contains(&w))
                );
                for slot in 0..4 {
                    assert!(
                        weights[i][slot] == 0.
                            || [rig.head_joint, rig.chest_joint]
                                .contains(&usize::from(joints[i][slot]))
                    );
                }
            }
            // Weld coincident rim corners only for topology. Their stored
            // copies keep a fabric-thickness crease in the lighting field.
            let keys: Vec<_> = points
                .iter()
                .map(|p| p.map(|v| (v * 100_000.).round() as i32))
                .collect();
            let mut edges = BTreeMap::new();
            let faces: Vec<[usize; 3]> = mesh.indices[old_faces..]
                .as_chunks::<3>()
                .0
                .iter()
                .map(|f| [f[0], f[1], f[2]].map(|i| i as usize - old_points))
                .collect();
            for face in &faces {
                for edge in 0..3 {
                    let a = keys[face[edge]];
                    let b = keys[face[(edge + 1) % 3]];
                    assert_ne!(a, b, "{} collapsed hair edge", look.texture_key);
                    let entry = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += if a < b { 1 } else { -1 };
                }
            }
            assert!(
                edges.values().all(|&v| v == (2, 0)),
                "{} open or inverted hair rim",
                look.texture_key
            );

            let gait = HumanLocomotion::for_character(rig, kind);
            let idle = HumanIdle::new(rig, kind);
            let mut pose = rig.nodes.clone();
            let mut inspect = |pose: &[Transform], label: &str| {
                let inverse = human_locomotion::world(rig, pose, rig.mesh_node).inverse();
                let matrices = rig
                    .skin_nodes
                    .iter()
                    .enumerate()
                    .map(|(joint, &node)| {
                        let scale = if joint == rig.head_joint { 0.72 } else { 1. };
                        inverse
                            * human_locomotion::world(rig, pose, node)
                            * Mat4::from_scale(Vec3::splat(scale))
                            * rig.inverse_binds[joint]
                    })
                    .collect::<Vec<_>>();
                let posed: Vec<_> = points
                    .iter()
                    .enumerate()
                    .map(|(i, &p)| {
                        let matrix = (0..4).fold(Mat4::ZERO, |sum, s| {
                            sum + matrices[usize::from(joints[i][s])] * weights[i][s]
                        });
                        let point = matrix.transform_point3(Vec3::from_array(p));
                        assert!(
                            point.is_finite()
                                && point.cmpge(bounds.min).all()
                                && point.cmple(bounds.max).all(),
                            "{} hair escapes culling at {label}, vertex {i}",
                            look.texture_key
                        );
                        (
                            point,
                            matrix
                                .transform_vector3(Vec3::from_array(normals[i]))
                                .normalize(),
                        )
                    })
                    .collect();
                for (index, face) in faces.iter().enumerate() {
                    let p = face.map(|i| posed[i].0);
                    let cross = (p[1] - p[0]).cross(p[2] - p[0]);
                    assert!(
                        cross.is_finite() && cross.length_squared() > 1e-14,
                        "{} collapsed hair face {index} at {label}",
                        look.texture_key
                    );
                    for &i in face {
                        let agreement = cross.normalize().dot(posed[i].1);
                        minimum_normal = minimum_normal.min(agreement);
                        assert!(
                            agreement > 0.,
                            "{} inward hair face {index}, vertex {i} at {label}: {agreement}",
                            look.texture_key
                        );
                    }
                }
                poses += 1;
            };
            for run in [0., 0.25, 0.5, 0.75, 1.] {
                for frame in 0..32 {
                    gait.sample(rig, frame as f32 / 32. * TAU, run, &mut pose);
                    inspect(&pose, &format!("run={run}, frame={frame}/32"));
                }
            }
            for frame in 0..24 {
                idle.sample(rig, &gait, frame as f32 * 1.83, &mut pose);
                inspect(&pose, &format!("idle={frame}"));
            }
            for clip in ["Idle", "Run", "Jump"] {
                for frame in 0..32 {
                    rig.sample(clip, rig.duration(clip) * frame as f32 / 32., &mut pose);
                    inspect(&pose, &format!("{clip}, frame={frame}/32"));
                }
            }
        }
        println!(
            "Long-hair cuts: seven identities, {poses} poses; closed winding, existing body/skin/art preserved, head/back carriers and culling; minimum outward normal={minimum_normal}"
        );
    }
}
