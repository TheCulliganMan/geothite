//! Fit the upper garment to a horizontal section of the supplied body.
//! Sampling source triangle skin weights keeps the seam attached to the torso.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;

pub(crate) struct Sample {
    pub(crate) radius: f32,
    pub(crate) influences: Vec<(usize, f32)>,
}
fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}
pub(crate) fn section(rig: &PersonRig, y: f32, direction: Vec2) -> Option<Sample> {
    let center = Vec2::new(0.0, 0.05);
    let mut closest = f32::INFINITY;
    let mut result = None;
    for face in rig.mesh.indices.chunks_exact(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        let points = ids.map(|i| Vec3::from_array(rig.mesh.positions[i]));
        let mut hits = Vec::with_capacity(3);
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let span = points[b].y - points[a].y;
            if span.abs() < 1e-7 {
                continue;
            }
            let t = (y - points[a].y) / span;
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let p = points[a].lerp(points[b], t);
            if hits
                .iter()
                .any(|(q, _): &(Vec2, [f32; 3])| q.distance_squared(Vec2::new(p.x, p.z)) < 1e-12)
            {
                continue;
            }
            let mut bary = [0.0; 3];
            bary[a] = 1.0 - t;
            bary[b] = t;
            hits.push((Vec2::new(p.x, p.z), bary));
        }
        if hits.len() != 2 {
            continue;
        }
        let segment = hits[1].0 - hits[0].0;
        let denominator = cross(direction, segment);
        if denominator.abs() < 1e-7 {
            continue;
        }
        let delta = hits[0].0 - center;
        let radius = cross(delta, segment) / denominator;
        let t = cross(delta, direction) / denominator;
        if radius <= 0.0 || radius >= closest || !(-1e-5..=1.00001).contains(&t) {
            continue;
        }
        let t = t.clamp(0.0, 1.0);
        let bary: [f32; 3] = std::array::from_fn(|i| hits[0].1[i] * (1.0 - t) + hits[1].1[i] * t);
        let mut weights = vec![0.0; rig.skin_nodes.len()];
        for (corner, &i) in ids.iter().enumerate() {
            for slot in 0..4 {
                weights[rig.ids[i][slot] as usize] += bary[corner] * rig.weights[i][slot];
            }
        }
        closest = radius;
        result = Some(Sample {
            radius,
            influences: weights
                .into_iter()
                .enumerate()
                .filter(|(_, w)| *w > 0.0)
                .collect(),
        });
    }
    result
}

/// Lower fabric hangs from the waist instead of inheriting the body's
/// sharp pelvis-to-thigh weight transition at each individual height.
pub(crate) fn waist_influences(rig: &PersonRig, p: [f32; 3]) -> Option<Vec<(usize, f32)>> {
    let direction = Vec2::new(p[0], p[2] - 0.05).normalize_or_zero();
    if direction.length_squared() < 0.5 {
        return None;
    }
    section(rig, 1.62, direction).map(|sample| sample.influences)
}

/// Returns interpolated body influences for vertices in the fitted seam.
/// The rest of the garment retains its authored profile and leg weight field.
pub(crate) fn fit(mesh: &mut SurfaceMeshData, rig: &PersonRig) -> Vec<Option<Vec<(usize, f32)>>> {
    let outer_vertices = mesh.positions.len() / 2;
    mesh.positions
        .iter_mut()
        .enumerate()
        .map(|(i, p)| {
            if p[1] < 1.46 {
                return None;
            }
            let radial = Vec2::new(p[0], p[2] - 0.05);
            let radius = radial.length();
            if radius < 1e-6 {
                return None;
            }
            let direction = radial / radius;
            let sample = section(rig, p[1], direction)?;
            let t = ((p[1] - 1.46) / 0.16).clamp(0.0, 1.0);
            let blend = t * t * (3.0 - 2.0 * t);
            // The inside lies just over the body; fabric thickness is real mesh.
            let padding = if i < outer_vertices { 0.018 } else { 0.002 };
            let fitted = direction * (radius * (1.0 - blend) + (sample.radius + padding) * blend);
            p[0] = fitted.x;
            p[2] = fitted.y + 0.05;
            Some(sample.influences)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
        parents[i]
            .map(|p| world(p, pose, parents))
            .unwrap_or(Mat4::IDENTITY)
            * pose[i].to_matrix()
    }
    fn triangle_distance(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> f32 {
        let n = (b - a).cross(c - a);
        if n.length_squared() > 1e-12 {
            let q = p - n * (p - a).dot(n) / n.length_squared();
            if [(a, b), (b, c), (c, a)]
                .iter()
                .all(|&(v, w)| (w - v).cross(q - v).dot(n) >= -1e-8)
            {
                return p.distance(q);
            }
        }
        [(a, b), (b, c), (c, a)]
            .iter()
            .map(|&(v, w)| {
                let edge = w - v;
                let t = ((p - v).dot(edge) / edge.length_squared().max(1e-12)).clamp(0.0, 1.0);
                p.distance(v + edge * t)
            })
            .fold(f32::INFINITY, f32::min)
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_garment_waists_remain_close_to_the_animated_body_surface() {
        use crate::new_bark_actors::character_meshes::SOURCE_KINDS;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let load = |name| {
            PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap()
        };
        let male = load("human-male");
        let female = load("human-female");
        let mut count = 0;
        for (_, kind) in SOURCE_KINDS {
            let look = crate::npc_appearance::look(kind);
            let rig = if look.base == "human-female" {
                &female
            } else {
                &male
            };
            let Some(cloth) = crate::npc_appearance::costume_mesh(rig, look) else {
                continue;
            };
            count += 1;
            let mut pose = rig.nodes.clone();
            for step in 0..8 {
                rig.sample("Run", rig.duration("Run") * step as f32 / 8.0, &mut pose);
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin: Vec<_> = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                    .collect();
                let deform = |p: [f32; 3], ids: &[u16; 4], weights: &[f32; 4]| {
                    (0..4)
                        .map(|i| {
                            skin[ids[i] as usize].transform_point3(Vec3::from_array(p)) * weights[i]
                        })
                        .sum::<Vec3>()
                };
                let body: Vec<_> = rig
                    .mesh
                    .positions
                    .iter()
                    .zip(&rig.ids)
                    .zip(&rig.weights)
                    .map(|((&p, ids), weights)| deform(p, ids, weights))
                    .collect();
                for ((&p, ids), weights) in cloth
                    .surface
                    .positions
                    .iter()
                    .zip(&cloth.joints)
                    .zip(&cloth.weights)
                {
                    if p[1] < 1.62 {
                        continue;
                    }
                    let point = deform(p, ids, weights);
                    let distance = rig
                        .mesh
                        .indices
                        .chunks_exact(3)
                        .map(|face| {
                            triangle_distance(
                                point,
                                body[face[0] as usize],
                                body[face[1] as usize],
                                body[face[2] as usize],
                            )
                        })
                        .fold(f32::INFINITY, f32::min);
                    assert!(
                        distance < 0.035,
                        "{} waist detaches at Run {step}/8: {distance}",
                        look.texture_key
                    );
                }
            }
        }
        assert!(count > 15, "exercise the whole supplied garment cast");
    }
}
