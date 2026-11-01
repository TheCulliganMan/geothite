//! Small crosslaces fitted to the actual upper, with its interpolated skin.
use super::*;
use std::collections::BTreeMap;

#[derive(Clone)]
struct Point {
    position: Vec3,
    joints: [u16; 4],
    weights: [f32; 4],
}

fn surface(
    mesh: &SurfaceMeshData,
    ids: &[[u16; 4]],
    weights: &[[f32; 4]],
    x: f32,
    z: f32,
) -> Point {
    let mut result: Option<Point> = None;
    for face in mesh.indices.chunks_exact(3) {
        let indices = [face[0], face[1], face[2]].map(|i| i as usize);
        let p = indices.map(|i| Vec3::from_array(mesh.positions[i]));
        if p.iter().any(|p| p.y > 0.27) {
            continue;
        }
        let a = p[1] - p[0];
        let b = p[2] - p[0];
        let determinant = a.x * b.z - b.x * a.z;
        if determinant.abs() < 1e-10 {
            continue;
        }
        let dx = x - p[0].x;
        let dz = z - p[0].z;
        let u = (dx * b.z - b.x * dz) / determinant;
        let v = (a.x * dz - dx * a.z) / determinant;
        if u < -1e-6 || v < -1e-6 || u + v > 1.000001 {
            continue;
        }
        let blend = [1. - u - v, u, v];
        let y: f32 = p.iter().zip(blend).map(|(p, w)| p.y * w).sum();
        if result.as_ref().is_some_and(|old| old.position.y >= y) {
            continue;
        }
        let mut skin = BTreeMap::<u16, f32>::new();
        for (i, amount) in indices.into_iter().zip(blend) {
            for (&joint, &weight) in ids[i].iter().zip(&weights[i]) {
                *skin.entry(joint).or_default() += amount.max(0.) * weight;
            }
        }
        let mut skin: Vec<_> = skin.into_iter().filter(|(_, w)| *w > 0.).collect();
        skin.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        skin.truncate(4);
        let total: f32 = skin.iter().map(|(_, w)| w).sum();
        result = Some(Point {
            position: Vec3::new(x, y, z),
            joints: std::array::from_fn(|i| skin.get(i).map_or(0, |p| p.0)),
            weights: std::array::from_fn(|i| skin.get(i).map_or(0., |p| p.1 / total)),
        });
    }
    result.expect("supplied player upper must support the authored lace layout")
}

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    // Sample before adding anything: later laces must not attach to an earlier
    // lace tube instead of the supplied upper surface.
    let mut paths = Vec::new();
    for side in [-1., 1.] {
        for row in 0..3 {
            for direction in [-1., 1.] {
                let path = (0..=8)
                    .map(|step| {
                        let t = step as f32 / 8.;
                        let x = side * 0.36 + direction * (t - 0.5) * 0.11;
                        let z = -0.02 + row as f32 * 0.045 + t * 0.04;
                        let mut point = surface(mesh, ids, weights, x, z);
                        // The two crossing strands have separate, close layers.
                        point.position.y += if direction > 0. { 0.0018 } else { 0.0032 };
                        point
                    })
                    .collect::<Vec<_>>();
                paths.push(path);
            }
        }
    }
    let shade = Color::srgb_u8(190, 194, 194).to_linear().to_f32_array();
    const SIDES: usize = 8;
    const HALF_WIDTH: f32 = 0.0035;
    const HALF_THICKNESS: f32 = 0.0006;
    let first_vertex = mesh.positions.len();
    let first_index = mesh.indices.len();
    // Fit every ribbon edge to the original upper, including its ridges.
    // Restrict the immutable snapshot to foot triangles for these ray queries.
    let mut upper = mesh.clone();
    upper.indices = mesh
        .indices
        .chunks_exact(3)
        .filter(|face| face.iter().all(|&i| mesh.positions[i as usize][1] <= 0.27))
        .flatten()
        .copied()
        .collect();
    let upper_ids = ids.clone();
    let upper_weights = weights.clone();
    for path in paths {
        let first = mesh.positions.len() as u32;
        for (i, point) in path.iter().enumerate() {
            let tangent = (path[(i + 1).min(path.len() - 1)].position
                - path[i.saturating_sub(1)].position)
                .normalize();
            let across = tangent.cross(Vec3::Y).normalize();
            let up = across.cross(tangent).normalize();
            let layer = point.position.y
                - surface(
                    &upper,
                    &upper_ids,
                    &upper_weights,
                    point.position.x,
                    point.position.z,
                )
                .position
                .y;
            for radial in 0..SIDES {
                let angle = radial as f32 * std::f32::consts::TAU / SIDES as f32;
                let offset =
                    across * (HALF_WIDTH * angle.cos()) + up * (HALF_THICKNESS * angle.sin());
                let normal = (across * (angle.cos() / HALF_WIDTH)
                    + up * (angle.sin() / HALF_THICKNESS))
                    .normalize();
                let projected = point.position + offset;
                let mut fitted =
                    surface(&upper, &upper_ids, &upper_weights, projected.x, projected.z);
                fitted.position.y += layer + offset.y;
                mesh.positions.push(fitted.position.to_array());
                mesh.normals.push(normal.to_array());
                mesh.uvs.push([0.716, 0.465]);
                mesh.colors.push(shade);
                ids.push(fitted.joints);
                weights.push(fitted.weights);
            }
        }
        for ring in 0..path.len() - 1 {
            for radial in 0..SIDES {
                let a = first + (ring * SIDES + radial) as u32;
                let b = first + (ring * SIDES + (radial + 1) % SIDES) as u32;
                let c = a + SIDES as u32;
                let d = b + SIDES as u32;
                mesh.indices.extend([a, c, b, b, c, d]);
            }
        }
        // Closed ends sit at the eyelet positions on the upper.
        for (ring, reverse) in [(0, true), (path.len() - 1, false)] {
            let point = &path[ring];
            let center = mesh.positions.len() as u32;
            let tangent = if reverse {
                path[0].position - path[1].position
            } else {
                path[ring].position - path[ring - 1].position
            };
            mesh.positions.push(point.position.to_array());
            mesh.normals.push(tangent.normalize().to_array());
            mesh.uvs.push([0.716, 0.465]);
            mesh.colors.push(shade);
            ids.push(point.joints);
            weights.push(point.weights);
            for radial in 0..SIDES {
                let a = first + (ring * SIDES + radial) as u32;
                let b = first + (ring * SIDES + (radial + 1) % SIDES) as u32;
                mesh.indices.extend(if reverse {
                    [center, a, b]
                } else {
                    [center, b, a]
                });
            }
        }
    }
    // Rebuild only the added ribbon normals after conforming their width to
    // the upper. The supplied contact surface and its normals remain exact.
    let mut normals = vec![Vec3::ZERO; mesh.positions.len() - first_vertex];
    for face in mesh.indices[first_index..].chunks_exact(3) {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        for corner in 0..3 {
            let a = (p[(corner + 1) % 3] - p[corner]).normalize();
            let b = (p[(corner + 2) % 3] - p[corner]).normalize();
            let angle = a.dot(b).clamp(-1.0, 1.0).acos();
            normals[face[corner] as usize - first_vertex] += normal * angle;
        }
    }
    for (i, normal) in normals.into_iter().enumerate() {
        mesh.normals[first_vertex + i] = normal.normalize().to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires supplied player rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_crosslaces_are_closed_fitted_and_keep_the_contact_surface() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for name in ["human-male", "human-female"] {
            let rig = crate::open_people::PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            let before = mesh.clone();
            append(&mut mesh, &mut ids, &mut weights);
            let first = before.positions.len();
            assert_eq!(&mesh.positions[..first], before.positions);
            assert_eq!(&mesh.normals[..first], before.normals);
            assert_eq!(&mesh.uvs[..first], before.uvs);
            assert_eq!(&mesh.indices[..before.indices.len()], before.indices);
            assert_eq!(&ids[..first], rig.ids);
            assert_eq!(&weights[..first], rig.weights);
            assert_eq!(mesh.positions.len() - first, 12 * (9 * 8 + 2));
            let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
            for triangle in mesh.indices[before.indices.len()..].chunks_exact(3) {
                let p = [triangle[0], triangle[1], triangle[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-14);
                for corner in 0..3 {
                    let (a, b) = (triangle[corner], triangle[(corner + 1) % 3]);
                    let edge = edges.entry((a.min(b), a.max(b))).or_default();
                    edge.0 += 1;
                    edge.1 += if a < b { 1 } else { -1 };
                }
            }
            assert!(
                edges
                    .values()
                    .all(|&(count, orientation)| count == 2 && orientation == 0),
                "lace tubes are open or inconsistently wound"
            );
            for i in first..mesh.positions.len() {
                let p = Vec3::from_array(mesh.positions[i]);
                let upper = surface(&before, &rig.ids, &rig.weights, p.x, p.z);
                assert!(
                    p.y - upper.position.y > 0.00005 && p.y - upper.position.y < 0.008,
                    "lace floats or penetrates the supplied upper: {p:?}"
                );
                assert!((Vec3::from_array(mesh.normals[i]).length() - 1.).abs() < 1e-5);
                assert!(weights[i].iter().all(|&w| w.is_finite() && w >= 0.));
                assert!((weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
                assert!(ids[i].iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
            }
        }
    }
}
