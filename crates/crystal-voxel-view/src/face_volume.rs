//! A shallow anime face volume on the supplied painted head.
//! Preserve frontal feature coordinates while giving the nose and chin a
//! profile. The same deformation carries fitted facial accessories.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct Frame {
    center: Vec3,
    half: Vec3,
}

fn head_weight(rig: &PersonRig, ids: &[u16; 4], weights: &[f32; 4]) -> f32 {
    (0..4)
        .filter(|&i| ids[i] as usize == rig.head_joint)
        .map(|i| weights[i])
        .sum()
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

impl Frame {
    pub(crate) fn new(
        rig: &PersonRig,
        mesh: &SurfaceMeshData,
        ids: &[[u16; 4]],
        weights: &[[f32; 4]],
    ) -> Self {
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for (i, &p) in mesh.positions.iter().enumerate() {
            if head_weight(rig, &ids[i], &weights[i]) > 0.5 {
                lo = lo.min(Vec3::from_array(p));
                hi = hi.max(Vec3::from_array(p));
            }
        }
        assert!(lo.is_finite() && hi.is_finite() && (hi - lo).min_element() > 0.);
        Self {
            center: (lo + hi) * 0.5,
            half: (hi - lo) * 0.5,
        }
    }

    fn local(self, p: Vec3) -> Vec3 {
        (p - self.center) / self.half
    }

    fn active(self, p: Vec3) -> bool {
        let p = self.local(p);
        p.z > 0.35 && p.y < 0.25
    }

    fn offset(self, p: Vec3, look: Look) -> f32 {
        let p = self.local(p);
        if p.z <= 0.35 || p.y >= 0.25 {
            return 0.;
        }
        let front = smooth((p.z - 0.35) / 0.65);
        let upper = smooth((0.25 - p.y) / 0.30);
        // Keep the authored foremost nose patch. Recess the surrounding face
        // instead of growing a round button over its painted nose.
        let nose_height = smooth((p.y + 0.62) / 0.12) * smooth((-0.12 - p.y) / 0.18);
        let nose_width = smooth(1. - p.x.abs() / 0.13);
        let young = look.height <= 1.20;
        let face = if young { 0.050 } else { 0.065 };
        let chin = if young { 0.080 } else { 0.100 };
        let cheek = 0.025 * smooth((p.x.abs() - 0.25) / 0.55) * smooth((0.10 - p.y) / 0.55);
        let chin = chin * smooth((-p.y - 0.55) / 0.35);
        // Pin the skin's mixed-weight neck in sculpt(), without making a
        // reversed ridge by fading the pure-head jaw back to its old depth.
        -self.half.z * front * upper * (face * (1. - nose_height * nose_width) + cheek + chin)
    }

    fn gradient(self, p: Vec3, look: Look) -> Vec3 {
        let derivative = |axis: Vec3, step: f32| {
            (self.offset(p + axis * step, look) - self.offset(p - axis * step, look)) / (2. * step)
        };
        Vec3::new(
            derivative(Vec3::X, self.half.x * 0.001),
            derivative(Vec3::Y, self.half.y * 0.001),
            derivative(Vec3::Z, self.half.z * 0.001),
        )
    }

    fn map(self, p: Vec3, normal: Vec3, look: Look) -> (Vec3, Vec3) {
        if !self.active(p) {
            return (p, normal);
        }
        let d = self.offset(p, look);
        let g = self.gradient(p, look);
        let determinant = 1. + g.z;
        assert!(
            determinant > 0.30 && determinant.is_finite(),
            "face volume folded"
        );
        // Painted anime faces use a quiet lighting field. Transport a small
        // amount of the new curvature, avoiding a second dark nostril beneath
        // the one already drawn in the painting. Geometry keeps the full warp.
        let lighting = g * 0.25;
        let nz = normal.z / (1. + lighting.z);
        let normal =
            Vec3::new(normal.x - lighting.x * nz, normal.y - lighting.y * nz, nz).normalize();
        (p + Vec3::Z * d, normal)
    }
}

pub(crate) fn sculpt(
    rig: &PersonRig,
    look: Look,
    frame: Frame,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    let original = (mesh.clone(), ids.clone(), weights.clone());
    // Most hairstyles already refine this area. Add detail only when a face
    // still has long triangles through the nose, retaining joined UV seams.
    for _ in 0..4 {
        let mut selected = vec![false; mesh.positions.len()];
        for face in mesh.indices.chunks_exact(3) {
            let indices = [face[0] as usize, face[1] as usize, face[2] as usize];
            let p = indices.map(|i| frame.local(Vec3::from_array(mesh.positions[i])));
            if indices
                .iter()
                .any(|&i| head_weight(rig, &ids[i], &weights[i]) <= 0.70)
                || p.iter().any(|p| p.z < 0.20)
                || p.iter().all(|p| p.y > 0.25)
                || p.iter().all(|p| p.y < -1.02)
                || [(0, 1), (1, 2), (2, 0)]
                    .iter()
                    .all(|&(a, b)| p[a].distance(p[b]) <= 0.16)
            {
                continue;
            }
            for i in indices {
                selected[i] = true;
            }
        }
        if !selected.iter().any(|&v| v) {
            break;
        }
        crate::hand_surface::linear_selection_joined(mesh, ids, weights, &selected);
    }
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    let before = {
        let count = original.0.positions.len();
        assert_eq!(
            &mesh.positions[..count],
            original.0.positions.as_slice(),
            "linear face refinement moved a source vertex"
        );
        assert_eq!(
            &mesh.normals[..count],
            original.0.normals.as_slice(),
            "linear face refinement changed source shading"
        );
        assert_eq!(&mesh.uvs[..count], original.0.uvs.as_slice());
        assert_eq!(&mesh.colors[..count], original.0.colors.as_slice());
        assert_eq!(&ids[..count], original.1.as_slice());
        assert_eq!(&weights[..count], original.2.as_slice());
        mesh.clone()
    };
    for i in 0..mesh.positions.len() {
        if head_weight(rig, &ids[i], &weights[i]) <= 0.70 {
            continue;
        }
        let (p, n) = frame.map(
            Vec3::from_array(mesh.positions[i]),
            Vec3::from_array(mesh.normals[i]),
            look,
        );
        mesh.positions[i] = p.to_array();
        mesh.normals[i] = n.to_array();
    }
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    validate_warp(rig, look, frame, &before, mesh, ids, weights);
}

/// Head accessories are stored in head-joint local space. Carry their occupied
/// roots through exactly the same bind-space field as the supporting face.
pub(crate) fn attachment(rig: &PersonRig, look: Look, frame: Frame, mesh: &mut SurfaceMeshData) {
    let bind = rig.inverse_binds[rig.head_joint];
    let world = bind.inverse();
    let to_world_normal = world.inverse().transpose();
    let to_local_normal = bind.inverse().transpose();
    for (p, n) in mesh.positions.iter_mut().zip(&mut mesh.normals) {
        let point = world.transform_point3(Vec3::from_array(*p));
        if !frame.active(point) {
            continue;
        }
        let normal = to_world_normal
            .transform_vector3(Vec3::from_array(*n))
            .normalize();
        let (point, normal) = frame.map(point, normal, look);
        *p = bind.transform_point3(point).to_array();
        *n = to_local_normal
            .transform_vector3(normal)
            .normalize()
            .to_array();
    }
}

#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
fn front(mesh: &SurfaceMeshData, x: f32, y: f32) -> f32 {
    let mut z = f32::NEG_INFINITY;
    for face in mesh.indices.chunks_exact(3) {
        let [a, b, c] =
            [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let d = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
        if d.abs() < 1e-10 {
            continue;
        }
        let u = ((b.y - c.y) * (x - c.x) + (c.x - b.x) * (y - c.y)) / d;
        let v = ((c.y - a.y) * (x - c.x) + (a.x - c.x) * (y - c.y)) / d;
        if u >= 0. && v >= 0. && u + v <= 1. {
            z = z.max(a.z * u + b.z * v + c.z * (1. - u - v));
        }
    }
    z
}

#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
fn validate_warp(
    rig: &PersonRig,
    look: Look,
    frame: Frame,
    before: &SurfaceMeshData,
    after: &SurfaceMeshData,
    ids: &[[u16; 4]],
    weights: &[[f32; 4]],
) {
    assert_eq!(before.indices, after.indices);
    assert_eq!(before.uvs, after.uvs);
    assert_eq!(before.colors, after.colors);
    let mut changed = 0;
    for i in 0..before.positions.len() {
        let a = Vec3::from_array(before.positions[i]);
        let b = Vec3::from_array(after.positions[i]);
        assert_eq!(a.x.to_bits(), b.x.to_bits());
        assert_eq!(a.y.to_bits(), b.y.to_bits());
        let n = Vec3::from_array(after.normals[i]);
        assert!(b.is_finite() && n.is_finite() && (n.length() - 1.).abs() < 1e-4);
        if head_weight(rig, &ids[i], &weights[i]) <= 0.70 || !frame.active(a) {
            assert_eq!(a, b, "{} body/crown/neck moved", look.texture_key);
            assert_eq!(before.normals[i], after.normals[i]);
        }
        if a != b {
            changed += 1;
        }
    }
    assert!(
        changed > 15,
        "{} has no face-volume change",
        look.texture_key
    );
    for face in after.indices.chunks_exact(3) {
        if !face
            .iter()
            .any(|&i| before.positions[i as usize] != after.positions[i as usize])
        {
            continue;
        }
        let a = [face[0], face[1], face[2]].map(|i| Vec3::from_array(before.positions[i as usize]));
        let b = [face[0], face[1], face[2]].map(|i| Vec3::from_array(after.positions[i as usize]));
        let old = (a[1] - a[0]).cross(a[2] - a[0]);
        let new = (b[1] - b[0]).cross(b[2] - b[0]);
        assert!(
            new.length_squared() > old.length_squared() * 0.05,
            "collapsed face volume"
        );
        let normal = face
            .iter()
            .map(|&i| Vec3::from_array(after.normals[i as usize]))
            .sum::<Vec3>();
        let old_normal = face
            .iter()
            .map(|&i| Vec3::from_array(before.normals[i as usize]))
            .sum::<Vec3>();
        assert!(
            new.dot(normal) > 0.,
            "{} inward face-volume shading at {face:?}: old={}, new={}, local={:?}",
            look.texture_key,
            old.dot(old_normal),
            new.dot(normal),
            a.map(|p| frame.local(p))
        );
    }
    let nose = front(after, frame.center.x, frame.center.y - frame.half.y * 0.40);
    let mouth = front(after, frame.center.x, frame.center.y - frame.half.y * 0.65);
    assert!(
        nose - mouth > frame.half.z * 0.05,
        "{} still has a flat face profile",
        look.texture_key
    );
    let y = frame.center.y - frame.half.y * 0.82;
    assert!(
        front(before, frame.center.x, y) - front(after, frame.center.x, y) > frame.half.z * 0.08,
        "{} chin did not recede",
        look.texture_key
    );
}

#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
pub(crate) fn validate_cast() {
    use crate::new_bark_actors::character_meshes::{CharacterKind, SOURCE_KINDS};
    let mut count = 0;
    for kind in SOURCE_KINDS
        .iter()
        .map(|&(_, kind)| kind)
        .chain(std::iter::once(CharacterKind::Outdoorsman))
    {
        let look = crate::npc_appearance::look(kind);
        let rig = crate::open_people::get(look.base).unwrap();
        let appearance = crate::open_people_skin::prepare_appearance(rig, kind);
        let mesh = appearance.review_body();
        assert!(
            mesh.count_vertices() < 120_000,
            "{} exceeds prepared vertex budget",
            look.texture_key
        );
        count += 1;
    }
    assert_eq!(count, 75);
}
