//! A small sculpted flower held against Erika's side hair.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::*;

fn oval(mesh: &mut SurfaceMeshData, center: Vec3, half: Vec3, rotation: Quat, rgb: [u8; 3]) {
    const ROWS: usize = 12;
    const SIDES: usize = 24;
    let first = mesh.positions.len();
    let color = Color::srgb_u8(rgb[0], rgb[1], rgb[2])
        .to_linear()
        .to_f32_array();
    let mut vertex = |d: Vec3| {
        mesh.positions
            .push((center + rotation * (d * half)).to_array());
        mesh.normals
            .push((rotation * (d / half).normalize()).to_array());
        mesh.colors.push(color);
        mesh.uvs.push([0.; 2]);
    };
    vertex(Vec3::Y);
    for row in 1..ROWS {
        for col in 0..SIDES {
            let phi = std::f32::consts::PI * row as f32 / ROWS as f32;
            let theta = std::f32::consts::TAU * col as f32 / SIDES as f32;
            vertex(Vec3::new(
                phi.sin() * theta.cos(),
                phi.cos(),
                phi.sin() * theta.sin(),
            ));
        }
    }
    vertex(-Vec3::Y);
    let south = mesh.positions.len() - 1;
    let mut face = |a: usize, b: usize, c: usize| {
        let p = [a, b, c].map(|i| Vec3::from_array(mesh.positions[i]));
        let n = [a, b, c]
            .map(|i| Vec3::from_array(mesh.normals[i]))
            .into_iter()
            .sum::<Vec3>();
        let ids = if (p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0. {
            [a, b, c]
        } else {
            [a, c, b]
        };
        mesh.indices.extend(ids.map(|i| i as u32));
    };
    for col in 0..SIDES {
        let next = (col + 1) % SIDES;
        face(first, first + 1 + col, first + 1 + next);
        for row in 0..ROWS - 2 {
            let a = first + 1 + row * SIDES + col;
            let b = first + 1 + row * SIDES + next;
            face(a, b, b + SIDES);
            face(a, b + SIDES, a + SIDES);
        }
        face(
            first + 1 + (ROWS - 2) * SIDES + col,
            first + 1 + (ROWS - 2) * SIDES + next,
            south,
        );
    }
}
pub(crate) fn append(mesh: &mut SurfaceMeshData, center: Vec3, half: Vec3) {
    let origin = center + half * Vec3::new(0.82, 0.42, 0.63);
    let normal = Vec3::new(0.72, 0.12, 0.68).normalize();
    let base = Quat::from_rotation_arc(Vec3::Z, normal);
    let scale = half.x / 0.44;
    for side in [-1., 1.] {
        let turn = base * Quat::from_rotation_z(side * 0.65);
        oval(
            mesh,
            origin + turn * (Vec3::Y * (-0.10 * scale)) - normal * 0.012,
            Vec3::new(0.032, 0.078, 0.012) * scale,
            turn,
            [90, 119, 65],
        );
    }
    for petal in 0..5 {
        let angle = std::f32::consts::TAU * petal as f32 / 5.;
        let turn = base * Quat::from_rotation_z(angle);
        oval(
            mesh,
            origin + turn * (Vec3::Y * (0.060 * scale)),
            Vec3::new(0.048, 0.070, 0.019) * scale,
            turn,
            if petal % 2 == 0 {
                [239, 222, 204]
            } else {
                [230, 200, 191]
            },
        );
    }
    oval(
        mesh,
        origin + normal * (0.020 * scale),
        Vec3::new(0.032, 0.032, 0.024) * scale,
        base,
        [202, 170, 86],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied female rig and paintings in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_erika_flower_preserves_existing_hair_and_head_bind_space() {
        use crate::new_bark_actors::character_meshes::CharacterKind::Erika;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = crate::open_people::PersonRig::parse(
            &std::fs::read(root.join("people/kenney/human-female.glb")).unwrap(),
        )
        .unwrap();
        let look = crate::npc_appearance::look(Erika);
        let plain = crate::npc_appearance::head_mesh(
            &rig,
            crate::npc_appearance::Look {
                accessories: "obi",
                ..look
            },
        )
        .unwrap();
        let decorated = crate::npc_appearance::head_mesh(&rig, look).unwrap();
        let count = plain.positions.len();
        assert_eq!(&decorated.positions[..count], &plain.positions);
        assert_eq!(&decorated.normals[..count], &plain.normals);
        assert_eq!(&decorated.colors[..count], &plain.colors);
        assert_eq!(&decorated.uvs[..count], &plain.uvs);
        assert_eq!(&decorated.indices[..plain.indices.len()], &plain.indices);
        assert!(decorated.positions.len() > count + 2000);
        let head_to_bind = rig.inverse_binds[rig.head_joint].inverse();
        for p in &decorated.positions[count..] {
            let p = head_to_bind.transform_point3(Vec3::from_array(*p));
            assert!(
                p.is_finite() && p.x > 0.20 && p.z > 0.10 && p.y > 2.7,
                "flower leaves the side-hair region: {p:?}"
            );
            assert!(p.cmple(rig.bounds.1 + Vec3::splat(0.25)).all());
        }
    }
    #[test]
    fn flower_has_closed_petals_and_finite_smooth_surfaces() {
        let mut mesh = SurfaceMeshData::default();
        append(&mut mesh, Vec3::ZERO, Vec3::new(0.44, 0.58, 0.52));
        assert!(mesh.positions.len() > 2000);
        let mut edges = std::collections::HashMap::new();
        for ids in mesh.indices.chunks_exact(3) {
            let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let n = ids
                .iter()
                .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                .sum::<Vec3>();
            assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 1e-10);
            for (a, b) in [(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0])] {
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        assert!(edges.values().all(|&n| n == 2));
        for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
            let p = Vec3::from_array(*p);
            let n = Vec3::from_array(*n);
            assert!(p.is_finite() && n.is_finite() && (n.length() - 1.).abs() < 1e-4);
            assert!(
                p.x > 0.20 && p.y > 0.02 && p.z > 0.20,
                "flower crosses the face"
            );
        }
    }
}
