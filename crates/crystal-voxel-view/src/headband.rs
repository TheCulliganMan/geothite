//! A thin closed cloth band fitted to each supported character's head surface.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};
use bevy::prelude::*;

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn radius(mesh: &SurfaceMeshData, y: f32, center: Vec3, dir: Vec2) -> Option<f32> {
    let mut result: Option<f32> = None;
    for face in mesh.indices.chunks_exact(3) {
        let p = face
            .iter()
            .map(|&i| Vec3::from_array(mesh.positions[i as usize]))
            .collect::<Vec<_>>();
        let mut cuts = Vec::with_capacity(2);
        for edge in 0..3 {
            let a = p[edge];
            let b = p[(edge + 1) % 3];
            if (a.y - y) * (b.y - y) > 0. || (b.y - a.y).abs() < 1e-8 {
                continue;
            }
            let t = (y - a.y) / (b.y - a.y);
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let q = a.lerp(b, t) - center;
            cuts.push(Vec2::new(q.x, q.z));
        }
        if cuts.len() < 2 {
            continue;
        }
        let edge = cuts[1] - cuts[0];
        let denominator = cross(dir, edge);
        if denominator.abs() < 1e-8 {
            continue;
        }
        let distance = cross(cuts[0], edge) / denominator;
        let along = cross(cuts[0], dir) / denominator;
        if distance > 0. && (-1e-5..=1.00001).contains(&along) {
            result = Some(result.map_or(distance, |old| old.max(distance)));
        }
    }
    result
}

pub(crate) fn append(
    out: &mut SurfaceMeshData,
    rig: &PersonRig,
    center: Vec3,
    half: Vec3,
    look: Look,
) {
    if !matches!(look.texture_key, "morty" | "black_belt") {
        return;
    }
    // Fit to the same refined surface used by the production actor. The band
    // stays below the sculpted crown and above the eyes; no skin UVs change.
    let mut head = rig.mesh.clone();
    let mut ids = rig.ids.clone();
    let mut weights = rig.weights.clone();
    crate::head_surface::smooth(rig, &mut head, &mut ids, &mut weights);
    crate::hair_shapes::sculpt(rig, &mut head, &mut ids, &mut weights, look);
    const SIDES: usize = 64;
    const ROWS: usize = 3;
    let first = out.positions.len();
    let color = if look.texture_key == "black_belt" {
        Color::srgb_u8(235, 232, 215)
    } else {
        Color::srgb_u8(140, 114, 165)
    }.to_linear().to_f32_array();
    for layer in 0..2 {
        for row in 0..ROWS {
            for side in 0..SIDES {
                let y = center.y + half.y * (0.24 + 0.14 * row as f32 / (ROWS - 1) as f32);
                let angle = std::f32::consts::TAU * side as f32 / SIDES as f32;
                let dir = Vec2::new(angle.cos(), angle.sin());
                let Some(r) = radius(&head, y, center, dir) else {
                    out.positions.truncate(first);
                    out.normals.truncate(first);
                    out.colors.truncate(first);
                    out.uvs.truncate(first);
                    return;
                };
                let r = r + if layer == 0 { 0.014 } else { 0.007 };
                out.positions
                    .push([center.x + dir.x * r, y, center.z + dir.y * r]);
                let normal = Vec3::new(dir.x, 0., dir.y) * if layer == 0 { 1. } else { -1. };
                out.normals.push(normal.to_array());
                out.colors.push(color);
                out.uvs.push([0.; 2]);
            }
        }
    }
    let at = |layer: usize, row: usize, side: usize| {
        first + layer * ROWS * SIDES + row * SIDES + side % SIDES
    };
    fn face(out: &mut SurfaceMeshData, a: usize, b: usize, c: usize) {
        let p = [a, b, c].map(|i| Vec3::from_array(out.positions[i]));
        let n = [a, b, c]
            .map(|i| Vec3::from_array(out.normals[i]))
            .into_iter()
            .sum::<Vec3>();
        let ids = if (p[1] - p[0]).cross(p[2] - p[0]).dot(n) >= 0. {
            [a, b, c]
        } else {
            [a, c, b]
        };
        out.indices.extend(ids.map(|i| i as u32));
    }
    for side in 0..SIDES {
        for layer in 0..2 {
            for row in 0..ROWS - 1 {
                let a = at(layer, row, side);
                let b = at(layer, row, side + 1);
                let c = at(layer, row + 1, side + 1);
                let d = at(layer, row + 1, side);
                face(out, a, b, c);
                face(out, a, c, d);
            }
        }
        for row in [0, ROWS - 1] {
            let a = at(0, row, side);
            let b = at(0, row, side + 1);
            let c = at(1, row, side + 1);
            let d = at(1, row, side);
            // Cap winding is geometric; radial normals cancel across layers.
            let triangles = if row == 0 {
                [[a, b, c], [a, c, d]]
            } else {
                [[a, d, c], [a, c, b]]
            };
            out.indices
                .extend(triangles.into_iter().flatten().map(|i| i as u32));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied male rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_band_is_closed_fitted_and_attaches_in_head_bind_space() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        for kind in [CharacterKind::Morty, CharacterKind::BlackBelt] {
        let look = crate::npc_appearance::look(kind);
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for (i, p) in rig.mesh.positions.iter().enumerate() {
            let head: f32 = (0..4)
                .filter(|&j| rig.ids[i][j] as usize == rig.head_joint)
                .map(|j| rig.weights[i][j])
                .sum();
            if head > 0.5 {
                lo = lo.min(Vec3::from_array(*p));
                hi = hi.max(Vec3::from_array(*p));
            }
        }
        let center = (lo + hi) * 0.5;
        let half = (hi - lo) * 0.5;
        let mut mesh = SurfaceMeshData::default();
        append(&mut mesh, &rig, center, half, look);
        assert_eq!(mesh.positions.len(), 384);
        let mut edges = std::collections::BTreeMap::new();
        for triangle in mesh.indices.chunks_exact(3) {
            for i in 0..3 {
                let a = triangle[i];
                let b = triangle[(i + 1) % 3];
                let entry = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
            let p = triangle
                .iter()
                .map(|&i| Vec3::from_array(mesh.positions[i as usize]))
                .collect::<Vec<_>>();
            assert!((p[1] - p[0]).cross(p[2] - p[0]).length() > 1e-8);
        }
        assert!(
            edges
                .values()
                .all(|&(count, direction)| count == 2 && direction == 0)
        );
        for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
            assert!(Vec3::from_array(*p).is_finite());
            assert!((Vec3::from_array(*n).length() - 1.).abs() < 1e-5);
            assert!(p[1] >= center.y + half.y * 0.239 && p[1] <= center.y + half.y * 0.381);
        }
        for i in 0..192 {
            let outer = Vec3::from_array(mesh.positions[i]);
            let inner = Vec3::from_array(mesh.positions[i + 192]);
            assert!((outer.distance(inner) - 0.007).abs() < 1e-5);
        }
        let attached = crate::npc_appearance::head_mesh(&rig, look).unwrap();
        assert_eq!(attached.positions.len(), mesh.positions.len());
        for (a, b) in mesh.positions.iter().zip(&attached.positions) {
            assert!(
                rig.inverse_binds[rig.head_joint]
                    .transform_point3(Vec3::from_array(*a))
                    .distance(Vec3::from_array(*b))
                    < 1e-6
            );
        }
        for step in 0..8 {
            let mut pose = rig.nodes.clone();
            rig.sample("Run", rig.duration("Run") * step as f32 / 8., &mut pose);
            fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                parents[i]
                    .map(|p| world(p, pose, parents))
                    .unwrap_or(Mat4::IDENTITY)
                    * pose[i].to_matrix()
            }
            let skin = world(rig.mesh_node, &pose, &rig.parents).inverse()
                * world(rig.skin_nodes[rig.head_joint], &pose, &rig.parents);
            for p in &attached.positions {
                let rendered = skin.transform_point3(Vec3::from_array(*p));
                assert!(rendered.is_finite());
                assert!(rendered.cmpge(rig.bounds.0 - Vec3::splat(0.03)).all());
                assert!(rendered.cmple(rig.bounds.1 + Vec3::splat(0.03)).all());
            }
        }
        let mut control = SurfaceMeshData::default();
        append(
            &mut control,
            &rig,
            center,
            half,
            crate::npc_appearance::look(CharacterKind::Koga),
        );
        assert!(control.positions.is_empty());
        }
    }
}
