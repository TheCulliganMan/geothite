//! Bicycle geometry contains no baked human. Riders use the shared weighted skin.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::*;
use std::f32::consts::TAU;

// Outer tyre radius, including its tube cross-section, rests on y=0.
const WHEEL_HEIGHT: f32 = 0.205 + 0.025;

fn tube(mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, radius: f32, color: [f32; 4]) {
    let axis = (b - a).normalize();
    let u = axis
        .cross(if axis.x.abs() < 0.8 { Vec3::X } else { Vec3::Y })
        .normalize();
    let v = axis.cross(u);
    let base = mesh.positions.len() as u32;
    for point in [a, b] {
        for i in 0..16 {
            let angle = i as f32 / 16.0 * TAU;
            let n = u * angle.cos() + v * angle.sin();
            mesh.positions.push((point + n * radius).to_array());
            mesh.normals.push(n.to_array());
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push(color);
        }
    }
    for i in 0..16u32 {
        let j = (i + 1) % 16;
        mesh.indices.extend([
            base + i,
            base + j,
            base + 16 + i,
            base + j,
            base + 16 + j,
            base + 16 + i,
        ]);
    }
    for (point, n, ring, reverse) in [(a, -axis, 0, true), (b, axis, 16, false)] {
        let center = mesh.positions.len() as u32;
        mesh.positions.push(point.to_array());
        mesh.normals.push(n.to_array());
        mesh.uvs.push([0.0, 0.0]);
        mesh.colors.push(color);
        for i in 0..16u32 {
            let j = (i + 1) % 16;
            let mut triangle = [center, base + ring + i, base + ring + j];
            if reverse {
                triangle.swap(1, 2);
            }
            mesh.indices.extend(triangle);
        }
    }
}
fn wheel(mesh: &mut SurfaceMeshData, z: f32, r: f32, thickness: f32, color: [f32; 4]) {
    let base = mesh.positions.len() as u32;
    for i in 0..48 {
        let a = i as f32 / 48.0 * TAU;
        let radial = Vec3::new(0.0, a.cos(), a.sin());
        for j in 0..12 {
            let b = j as f32 / 12.0 * TAU;
            let n = radial * b.cos() + Vec3::X * b.sin();
            mesh.positions
                .push((Vec3::new(0.0, WHEEL_HEIGHT, z) + radial * r + n * thickness).to_array());
            mesh.normals.push(n.to_array());
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push(color);
        }
    }
    for i in 0..48u32 {
        for j in 0..12u32 {
            let a = base + i * 12 + j;
            let b = base + ((i + 1) % 48) * 12 + j;
            let c = base + ((i + 1) % 48) * 12 + (j + 1) % 12;
            let d = base + i * 12 + (j + 1) % 12;
            mesh.indices.extend([a, b, c, a, c, d]);
        }
    }
}
pub(crate) fn mesh(female: bool) -> SurfaceMeshData {
    let mut m = SurfaceMeshData::default();
    let ink = [0.025, 0.035, 0.045, 1.0];
    let metal = [0.58, 0.64, 0.68, 1.0];
    let paint = if female {
        [0.025, 0.36, 0.34, 1.0]
    } else {
        [0.65, 0.04, 0.05, 1.0]
    };
    for z in [-0.37, 0.40] {
        wheel(&mut m, z, 0.205, 0.025, ink);
        wheel(&mut m, z, 0.182, 0.011, metal);
        for i in 0..10 {
            let a = i as f32 / 10.0 * TAU;
            tube(
                &mut m,
                Vec3::new(0.0, WHEEL_HEIGHT, z),
                Vec3::new(0.0, WHEEL_HEIGHT + a.cos() * 0.178, z + a.sin() * 0.178),
                0.005,
                metal,
            );
        }
    }
    let seat = Vec3::new(0.0, 0.61, -0.12);
    let crank = Vec3::new(0.0, 0.265, -0.015);
    let steer = Vec3::new(0.0, 0.67, 0.33);
    let back = Vec3::new(0.0, WHEEL_HEIGHT, -0.37);
    let front = Vec3::new(0.0, WHEEL_HEIGHT, 0.40);
    for (a, b) in [
        (seat, crank),
        (crank, steer),
        (steer, seat),
        (back, seat),
        (back, crank),
        (steer, front),
    ] {
        tube(&mut m, a, b, 0.022, paint);
    }
    tube(&mut m, seat, Vec3::new(0.0, 0.665, -0.12), 0.016, metal);
    tube(
        &mut m,
        Vec3::new(-0.08, 0.665, -0.12),
        Vec3::new(0.08, 0.665, -0.12),
        0.035,
        ink,
    );
    tube(&mut m, steer, Vec3::new(0.0, 0.855, 0.37), 0.018, metal);
    tube(
        &mut m,
        Vec3::new(-0.25, 0.865, 0.35),
        Vec3::new(0.25, 0.865, 0.35),
        0.018,
        metal,
    );
    for s in [-1.0, 1.0] {
        tube(
            &mut m,
            Vec3::new(s * 0.19, 0.865, 0.35),
            Vec3::new(s * 0.27, 0.865, 0.35),
            0.023,
            ink,
        );
        tube(
            &mut m,
            crank,
            Vec3::new(s * 0.075, 0.265 + s * 0.05, s * 0.06),
            0.011,
            metal,
        );
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bicycle_surfaces_have_finite_unit_normals_and_outward_triangles() {
        for female in [false, true] {
            let m = mesh(female);
            let bottom = m.positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
            assert!(bottom.abs() < 1e-6, "bicycle tyres must rest on the floor: {bottom}");
            assert_eq!(m.positions.len(), m.normals.len());
            assert_eq!(m.positions.len(), m.colors.len());
            for n in &m.normals {
                let n = Vec3::from_array(*n);
                assert!(n.is_finite() && (n.length() - 1.0).abs() < 1e-5);
            }
            for face in m.indices.chunks_exact(3) {
                let a = Vec3::from_array(m.positions[face[0] as usize]);
                let b = Vec3::from_array(m.positions[face[1] as usize]);
                let c = Vec3::from_array(m.positions[face[2] as usize]);
                let normal = face
                    .iter()
                    .map(|&i| Vec3::from_array(m.normals[i as usize]))
                    .sum::<Vec3>();
                assert!(
                    (b - a).cross(c - a).dot(normal) > 0.0,
                    "inward bicycle triangle"
                );
            }
        }
    }
}

/// Pose the existing weighted rider in mesh coordinates; no second human mesh.
pub(crate) fn pose(rig: &crate::open_people::PersonRig, transforms: &mut [Transform], phase: f32) {
    fn world_rotation(
        rig: &crate::open_people::PersonRig,
        pose: &[Transform],
        node: usize,
    ) -> Quat {
        let mut chain = vec![node];
        let mut parent = rig.parents[node];
        while let Some(p) = parent {
            chain.push(p);
            parent = rig.parents[p];
        }
        chain
            .iter()
            .rev()
            .fold(Quat::IDENTITY, |rotation, &i| rotation * pose[i].rotation)
    }
    fn turn(rig: &crate::open_people::PersonRig, pose: &mut [Transform], node: usize, angle: f32) {
        let parent = rig.parents[node]
            .map(|p| world_rotation(rig, pose, p))
            .unwrap_or(Quat::IDENTITY);
        let mesh = world_rotation(rig, pose, rig.mesh_node);
        let axis = parent.inverse() * mesh * Vec3::X;
        pose[node].rotation =
            (Quat::from_axis_angle(axis.normalize(), angle) * pose[node].rotation).normalize();
    }
    for (side, leg) in rig.leg_joints.iter().enumerate() {
        let cycle = (phase + side as f32 * std::f32::consts::PI).sin() * 0.12;
        turn(rig, transforms, rig.skin_nodes[leg[0]], -1.1 + cycle);
        turn(rig, transforms, rig.skin_nodes[leg[1]], 1.7 - cycle);
    }
    for hand in &rig.hand_joints {
        if let Some(&wrist) = hand.iter().find(|&&joint| {
            rig.parents[rig.skin_nodes[joint]]
                .is_none_or(|parent| !hand.iter().any(|&other| rig.skin_nodes[other] == parent))
        }) {
            let forearm = rig.parents[rig.skin_nodes[wrist]];
            if let Some(forearm) = forearm {
                if let Some(upper) = rig.parents[forearm] {
                    turn(rig, transforms, upper, -1.0);
                }
                turn(rig, transforms, forearm, -0.2);
            }
        }
    }
}
