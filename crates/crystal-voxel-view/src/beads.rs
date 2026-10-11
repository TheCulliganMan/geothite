//! Rounded wooden prayer beads fitted to the supplied robe's chest surface.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};
use bevy::prelude::*;

const COUNT: usize = 36;
const RADIUS: f32 = 0.022;
const SIDES: usize = 20;
const ROWS: usize = 12;

fn radius(surface: &SurfaceMeshData, y: f32, dir: Vec2) -> Option<f32> {
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let mut points = Vec::new();
    for face in surface.indices.chunks_exact(3) {
        let p = face
            .iter()
            .map(|&i| Vec3::from_array(surface.positions[i as usize]))
            .collect::<Vec<_>>();
        // Close the medial chest slice where its surface joins the T-pose arms.
        // Arm intersections otherwise give the necklace outward shoulder spikes.
        if !p.iter().all(|p| p.x.abs() < 0.4) {
            continue;
        }
        for edge in 0..3 {
            let (a, b) = (p[edge], p[(edge + 1) % 3]);
            if (a.y - y) * (b.y - y) > 0. || (b.y - a.y).abs() < 1e-8 {
                continue;
            }
            let t = (y - a.y) / (b.y - a.y);
            if (0.0..=1.0).contains(&t) {
                let q = a.lerp(b, t);
                points.push(Vec2::new(q.x, q.z));
            }
        }
    }
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    points.dedup_by(|a, b| a.distance_squared(*b) < 1e-12);
    if points.len() < 3 {
        return None;
    }
    let mut hull: Vec<Vec2> = Vec::new();
    for &p in &points {
        while hull.len() > 1
            && cross(
                hull[hull.len() - 1] - hull[hull.len() - 2],
                p - hull[hull.len() - 1],
            ) <= 0.
        {
            hull.pop();
        }
        hull.push(p);
    }
    let lower = hull.len();
    for &p in points.iter().rev().skip(1) {
        while hull.len() > lower
            && cross(
                hull[hull.len() - 1] - hull[hull.len() - 2],
                p - hull[hull.len() - 1],
            ) <= 0.
        {
            hull.pop();
        }
        hull.push(p);
    }
    hull.pop();
    let mut result: Option<f32> = None;
    for i in 0..hull.len() {
        let a = hull[i];
        let edge = hull[(i + 1) % hull.len()] - a;
        let d = cross(dir, edge);
        if d.abs() < 1e-8 {
            continue;
        }
        let r = cross(a, edge) / d;
        let along = cross(a, dir) / d;
        if r > 0. && (-1e-5..=1.00001).contains(&along) {
            result = Some(result.map_or(r, |old| old.max(r)));
        }
    }
    result
}

fn face(mesh: &mut SurfaceMeshData, mut ids: [u32; 3]) {
    let p = ids.map(|i| Vec3::from_array(mesh.positions[i as usize]));
    let n = ids
        .map(|i| Vec3::from_array(mesh.normals[i as usize]))
        .into_iter()
        .sum::<Vec3>();
    if (p[1] - p[0]).cross(p[2] - p[0]).dot(n) < 0. {
        ids.swap(1, 2);
    }
    mesh.indices.extend(ids);
}

fn sphere(mesh: &mut SurfaceMeshData, center: Vec3, r: f32, color: [f32; 4]) {
    let first = mesh.positions.len() as u32;
    let mut vertex = |normal: Vec3| {
        mesh.positions.push((center + normal * r).to_array());
        mesh.normals.push(normal.to_array());
        mesh.colors.push(color);
        mesh.uvs.push([0.; 2]);
    };
    vertex(Vec3::Y);
    for row in 1..ROWS {
        for side in 0..SIDES {
            let a = std::f32::consts::PI * row as f32 / ROWS as f32;
            let b = std::f32::consts::TAU * side as f32 / SIDES as f32;
            vertex(Vec3::new(a.sin() * b.cos(), a.cos(), a.sin() * b.sin()));
        }
    }
    vertex(-Vec3::Y);
    let south = mesh.positions.len() as u32 - 1;
    let at = |row: usize, side: usize| first + 1 + ((row - 1) * SIDES + side % SIDES) as u32;
    for side in 0..SIDES {
        face(mesh, [first, at(1, side), at(1, side + 1)]);
        for row in 1..ROWS - 1 {
            face(mesh, [at(row, side), at(row + 1, side), at(row, side + 1)]);
            face(
                mesh,
                [at(row, side + 1), at(row + 1, side), at(row + 1, side + 1)],
            );
        }
        face(mesh, [south, at(ROWS - 1, side + 1), at(ROWS - 1, side)]);
    }
}

pub(crate) fn mesh(rig: &PersonRig, look: Look) -> Option<SurfaceMeshData> {
    if !look.accessories.split(',').any(|a| a == "beads") {
        return None;
    }
    const SAMPLES: usize = 144;
    let mut path = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let angle = std::f32::consts::TAU * i as f32 / SAMPLES as f32;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let y = 2.49 - 0.31 * angle.sin().max(0.).powf(0.8);
        let r = radius(&rig.mesh, y, dir)? + RADIUS + 0.012;
        path.push(Vec3::new(dir.x * r, y, dir.y * r));
    }
    let lengths = (0..SAMPLES)
        .map(|i| path[i].distance(path[(i + 1) % SAMPLES]))
        .collect::<Vec<_>>();
    let total = lengths.iter().sum::<f32>();
    let mut centers = Vec::with_capacity(COUNT);
    for i in 0..COUNT {
        let mut distance = total * i as f32 / COUNT as f32;
        let mut segment = 0;
        while segment < SAMPLES - 1 && distance > lengths[segment] {
            distance -= lengths[segment];
            segment += 1;
        }
        let p = path[segment].lerp(path[(segment + 1) % SAMPLES], distance / lengths[segment]);
        let dir = Vec2::new(p.x, p.z).normalize();
        let r = radius(&rig.mesh, p.y, dir)? + RADIUS + 0.012;
        centers.push(Vec3::new(dir.x * r, p.y, dir.y * r));
    }
    let mut mesh = SurfaceMeshData::default();
    for (i, &center) in centers.iter().enumerate() {
        let color = Color::srgb_u8(105 + (i % 3) as u8 * 4, 61 + (i % 3) as u8 * 2, 34)
            .to_linear()
            .to_f32_array();
        sphere(&mut mesh, center, RADIUS, color);
    }
    // A continuous thin cord joins the beads and closes around the neck.
    let first = mesh.positions.len() as u32;
    const CORD_SIDES: usize = 8;
    for i in 0..COUNT {
        let tangent = (centers[(i + 1) % COUNT] - centers[(i + COUNT - 1) % COUNT]).normalize();
        let radial = Vec3::new(centers[i].x, 0., centers[i].z).normalize();
        let u = (radial - tangent * radial.dot(tangent)).normalize();
        let v = tangent.cross(u).normalize();
        for side in 0..CORD_SIDES {
            let a = std::f32::consts::TAU * side as f32 / CORD_SIDES as f32;
            let n = u * a.cos() + v * a.sin();
            mesh.positions.push((centers[i] + n * 0.004).to_array());
            mesh.normals.push(n.to_array());
            mesh.colors
                .push(Color::srgb_u8(57, 34, 23).to_linear().to_f32_array());
            mesh.uvs.push([0.; 2]);
        }
    }
    let at =
        |row: usize, side: usize| first + (row % COUNT * CORD_SIDES + side % CORD_SIDES) as u32;
    for row in 0..COUNT {
        for side in 0..CORD_SIDES {
            face(
                &mut mesh,
                [at(row, side), at(row + 1, side), at(row, side + 1)],
            );
            face(
                &mut mesh,
                [at(row, side + 1), at(row + 1, side), at(row + 1, side + 1)],
            );
        }
    }
    let bind = rig.inverse_binds[rig.chest_joint];
    let normal = bind.inverse().transpose();
    for (p, n) in mesh.positions.iter_mut().zip(&mut mesh.normals) {
        *p = bind.transform_point3(Vec3::from_array(*p)).to_array();
        *n = normal
            .transform_vector3(Vec3::from_array(*n))
            .normalize()
            .to_array();
    }
    Some(mesh)
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
    #[test]
    #[ignore = "requires supplied male rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_beads_are_closed_round_fitted_and_follow_chest_animation() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let look = crate::npc_appearance::look(CharacterKind::Sage);
        let mesh = super::mesh(&rig, look).expect("fitted necklace");
        assert!(super::mesh(&rig, crate::npc_appearance::look(CharacterKind::Elder)).is_none());
        let bind = rig.inverse_binds[rig.chest_joint].inverse();
        let stride = 2 + (ROWS - 1) * SIDES;
        let centers = (0..COUNT)
            .map(|i| {
                bind.transform_point3(Vec3::from_array(mesh.positions[i * stride]))
                    - Vec3::Y * RADIUS
            })
            .collect::<Vec<_>>();
        for i in 0..COUNT {
            let distance = centers[i].distance(centers[(i + 1) % COUNT]);
            assert!(
                (RADIUS * 1.95..RADIUS * 3.4).contains(&distance),
                "crowded or separated necklace beads: {distance}"
            );
        }
        for i in 0..COUNT {
            let p = Vec3::from_array(mesh.positions[i * stride]);
            let center = bind.transform_point3(p) - Vec3::Y * RADIUS;
            let dir = Vec2::new(center.x, center.z).normalize();
            let surface = radius(&rig.mesh, center.y, dir).unwrap();
            let gap = Vec2::new(center.x, center.z).length() - RADIUS - surface;
            assert!(
                (0.010..0.014).contains(&gap),
                "floating or clipped bead: {gap}"
            );
            assert!(center.x.abs() < 0.4 && (2.17..2.50).contains(&center.y));
            for p in &mesh.positions[i * stride..(i + 1) * stride] {
                let distance = bind.transform_point3(Vec3::from_array(*p)).distance(center);
                assert!((distance - RADIUS).abs() < 1e-5, "nonround bead");
            }
        }
        let mut edges = std::collections::BTreeMap::new();
        for face in mesh.indices.chunks_exact(3) {
            let p = face
                .iter()
                .map(|&i| Vec3::from_array(mesh.positions[i as usize]))
                .collect::<Vec<_>>();
            let n = (p[1] - p[0]).cross(p[2] - p[0]);
            let normal = face
                .iter()
                .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                .sum::<Vec3>();
            assert!(
                n.length_squared() > 1e-16 && n.dot(normal) > 0.,
                "invalid surface winding"
            );
            for e in 0..3 {
                let (a, b) = (face[e], face[(e + 1) % 3]);
                let value = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                value.0 += 1;
                value.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges.values().all(|&v| v == (2, 0)),
            "open bead or cord surface"
        );
        assert!(
            mesh.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
        );
        for clip in ["Idle", "Run"] {
            for step in 0..=60 {
                let mut pose = rig.nodes.clone();
                let phase = step as f32 / 60.;
                rig.sample(clip, rig.duration(clip) * phase, &mut pose);
                let chest = world(rig.skin_nodes[rig.chest_joint], &pose, &rig.parents);
                let mut body_lo = Vec3::splat(f32::INFINITY);
                let mut body_hi = Vec3::splat(f32::NEG_INFINITY);
                for ((p, j), w) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
                    let p = Vec3::from_array(*p);
                    let mut q = Vec3::ZERO;
                    for slot in 0..4 {
                        q += (world(rig.skin_nodes[j[slot] as usize], &pose, &rig.parents)
                            * rig.inverse_binds[j[slot] as usize])
                            .transform_point3(p)
                            * w[slot];
                    }
                    body_lo = body_lo.min(q);
                    body_hi = body_hi.max(q);
                }
                for p in &mesh.positions {
                    let p = chest.transform_point3(Vec3::from_array(*p));
                    assert!(
                        p.cmpge(body_lo - Vec3::splat(0.08)).all()
                            && p.cmple(body_hi + Vec3::splat(0.08)).all(),
                        "{clip} necklace escaped body bounds"
                    );
                }
            }
        }
    }
}
