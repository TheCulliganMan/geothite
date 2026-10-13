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
pub(crate) fn fit(mesh: &mut SurfaceMeshData, rig: &PersonRig, look: crate::npc_appearance::Look) -> Vec<Option<Vec<(usize, f32)>>> {
    let outer_vertices = mesh.positions.len() / 2;
    // Nurse's short fitted skirt needs a gradual flare below the waist. The
    // short generic seam blend makes a steep shelf that catches the light.
    // Keep the fully fitted waist and leave other authored garments unchanged.
    let fit_bottom = if look.texture_key == "nurse" { 1.10 } else { 1.46 };
    let fit_span = if look.texture_key == "nurse" { 0.52 } else { 0.16 };
    mesh.positions
        .iter_mut()
        .enumerate()
        .map(|(i, p)| {
            if p[1] < fit_bottom {
                return None;
            }
            let radial = Vec2::new(p[0], p[2] - 0.05);
            let radius = radial.length();
            if radius < 1e-6 {
                return None;
            }
            let direction = radial / radius;
            let sample = section(rig, p[1], direction)?;
            let t = ((p[1] - fit_bottom) / fit_span).clamp(0.0, 1.0);
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

/// Diffuse local fitted-body influences over the closed two-layer skirt.
/// Both layers share a weight field below the pinned waist, so thickness and
/// small body-section sampling changes cannot fold adjacent panels backwards.
pub(crate) fn smooth_fitted_skin(cloth: &mut crate::npc_appearance::CostumeMesh, skin_count: usize) {
    use std::collections::{BTreeMap, BTreeSet};
    let layer = cloth.surface.positions.len() / 2;
    assert_eq!(cloth.surface.positions.len(), layer * 2);
    let ring = cloth.surface.positions[..layer].iter()
        .take_while(|p| p[1] == cloth.surface.positions[0][1]).count();
    assert!(ring > 0 && layer % ring == 0);
    let bottom = cloth.surface.positions[layer - 1][1];
    let hem_row = (0..layer / ring).min_by(|&a, &b| {
        (cloth.surface.positions[a * ring][1] - bottom - 0.045).abs()
            .total_cmp(&(cloth.surface.positions[b * ring][1] - bottom - 0.045).abs())
    }).unwrap();
    let key = |p: [f32; 3]| p.map(|v| (v * 100_000.0).round() as i32);
    let mut welded = BTreeMap::new();
    let canonical: Vec<_> = cloth.surface.positions[..layer].iter().enumerate()
        .map(|(i, &p)| *welded.entry(key(p)).or_insert(i)).collect();
    let mut neighbors = vec![BTreeSet::new(); layer];
    for face in cloth.surface.indices.chunks_exact(3) {
        let points = [face[0], face[1], face[2]].map(|i| canonical[i as usize % layer]);
        for (a, b) in [(points[0], points[1]), (points[1], points[2]), (points[2], points[0])] {
            if a != b { neighbors[a].insert(b); neighbors[b].insert(a); }
        }
    }
    let mut fields = vec![vec![0.0; skin_count]; layer];
    for i in 0..layer {
        for vertex in [i, i + layer] {
            for slot in 0..4 {
                fields[i][cloth.joints[vertex][slot] as usize] += cloth.weights[vertex][slot] * 0.5;
            }
        }
    }
    for _ in 0..8 {
        let mut next = fields.clone();
        for i in 0..layer {
            if canonical[i] != i || cloth.surface.positions[i][1] >= 1.62 || neighbors[i].is_empty() {
                continue;
            }
            for joint in 0..skin_count {
                let mean = neighbors[i].iter().map(|&j| fields[j][joint]).sum::<f32>() / neighbors[i].len() as f32;
                next[i][joint] = fields[i][joint] * 0.5 + mean * 0.5;
            }
        }
        fields = next;
    }
    for i in 0..layer {
        if cloth.surface.positions[i][1] >= 1.62 { continue; }
        let t = ((1.62 - cloth.surface.positions[i][1]) / 0.10).clamp(0.0, 1.0);
        let blend = t * t * (3.0 - 2.0 * t);
        // The narrow rolled hem moves with one ring field. Weight gradients
        // across its .015-height fold can turn that fold inside out in Run.
        let sample = if cloth.surface.positions[i][1] <= bottom + 0.04501 {
            hem_row * ring + i % ring
        } else { i };
        let mut field = fields[canonical[sample]].clone();
        for value in &mut field { *value *= blend; }
        for slot in 0..4 {
            field[cloth.joints[i][slot] as usize] += cloth.weights[i][slot] * (1.0 - blend);
        }
        let mut ranked: Vec<_> = field.into_iter().enumerate().collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let total: f32 = ranked[..4].iter().map(|(_, weight)| weight).sum();
        assert!(total.is_finite() && total > 0.0);
        for vertex in [i, i + layer] {
            cloth.joints[vertex] = std::array::from_fn(|slot| ranked[slot].0 as u16);
            cloth.weights[vertex] = std::array::from_fn(|slot| ranked[slot].1 / total);
        }
    }
}

/// Fitting changes the side slopes. The flat cloth rims need separate normal
/// islands so a side normal cannot point against a rim triangle.
pub(crate) fn finish_normals(cloth: &mut crate::npc_appearance::CostumeMesh) {
    let mesh = &mut cloth.surface;
    let key = |p: [f32; 3]| p.map(|v| (v * 100_000.0).round() as i32);
    let mut normals = std::collections::BTreeMap::<[i32; 3], Vec3>::new();
    let mut rims = Vec::new();
    for (face_index, face) in mesh.indices.chunks_exact(3).enumerate() {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]);
        assert!(normal.is_finite() && normal.length_squared() > 1e-18);
        let (low, high) = p.iter().fold((f32::INFINITY, f32::NEG_INFINITY),
            |(low, high), p| (low.min(p.y), high.max(p.y)));
        if high - low < 1e-6 {
            rims.push((face_index, normal.normalize()));
        } else {
            for &i in face { *normals.entry(key(mesh.positions[i as usize])).or_default() += normal; }
        }
    }
    for (p, normal) in mesh.positions.iter().zip(&mut mesh.normals) {
        if let Some(sum) = normals.get(&key(*p)) {
            assert!(sum.is_finite() && sum.length_squared() > 1e-18);
            *normal = sum.normalize().to_array();
        }
    }
    for (face_index, normal) in rims {
        for corner in 0..3 {
            let at = face_index * 3 + corner;
            let i = mesh.indices[at] as usize;
            mesh.indices[at] = mesh.positions.len() as u32;
            mesh.positions.push(mesh.positions[i]);
            mesh.normals.push(normal.to_array());
            mesh.uvs.push(mesh.uvs[i]);
            mesh.colors.push(mesh.colors[i]);
            cloth.joints.push(cloth.joints[i]);
            cloth.weights.push(cloth.weights[i]);
        }
    }
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
            let Some(cloth) = crate::npc_appearance::costume_shell_mesh(rig, look) else {
                continue;
            };
            // This is a waist-attachment regression, not a clearance limit
            // for raised collars, emblems or lapels elsewhere on the costume.
            // Only credit roles that actually have geometry in the seam band.
            if !cloth.surface.positions.iter().any(|p| (1.62..=1.67).contains(&p[1])) {
                continue;
            }
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
                    if !(1.62..=1.67).contains(&p[1]) {
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
                        "{} fitted seam detaches at Run {step}/8: distance={distance}; bind={p:?}; posed={point:?}; joints={ids:?}; weights={weights:?}",
                        look.texture_key
                    );
                }
            }
        }
        assert!(count > 15, "exercise the whole supplied garment cast");
    }
}
