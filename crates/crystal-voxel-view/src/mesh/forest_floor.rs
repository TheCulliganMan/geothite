//! Small folded paper leaves, forked sticks and fern fronds on verified forest floor.
//! Placement is seeded in map coordinates; these details never affect navigation.
use super::*;

fn triangle(mesh: &mut SurfaceMeshData, mut p: [[f32; 3]; 3], color: [f32; 4]) {
    let mut normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
        .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]));
    if normal.y < 0.0 {
        p.swap(1, 2);
        normal = -normal;
    }
    let normal = normal.normalize_or_zero();
    let shade = 0.83 + normal.dot(Vec3::new(-0.35, 0.85, 0.4).normalize()).max(0.0) * 0.17;
    let first = mesh.positions.len() as u32;
    mesh.positions.extend(p);
    mesh.normals.extend([normal.to_array(); 3]);
    mesh.uvs.extend([[0.0; 2]; 3]);
    mesh.colors.extend(
        [[
            color[0] * shade,
            color[1] * shade,
            color[2] * shade,
            color[3],
        ]; 3],
    );
    mesh.indices.extend([first, first + 1, first + 2]);
}

fn leaf(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    length: f32,
    width: f32,
    angle: f32,
    fold: f32,
    color: [f32; 4],
) {
    let point = |x: f32, y: f32, z: f32| {
        (center
            + Vec3::new(
                angle.cos() * x - angle.sin() * z,
                y,
                angle.sin() * x + angle.cos() * z,
            ))
        .to_array()
    };
    let outline = [
        (-1.0, 0.0),
        (-0.65, 0.66),
        (0.25, 0.85),
        (1.0, 0.0),
        (0.45, -0.7),
        (-0.65, -0.6),
    ];
    let ridge = point(0.0, fold, 0.0);
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        triangle(
            mesh,
            [
                ridge,
                point(a.0 * length, 0.0, a.1 * width),
                point(b.0 * length, 0.0, b.1 * width),
            ],
            color,
        );
    }
}

fn stick(mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, width: f32) {
    let delta = b - a;
    let side = Vec3::new(-delta.z, 0.0, delta.x).normalize_or_zero() * width;
    append_solid_quad(
        mesh,
        [
            (a - side).to_array(),
            (a + side).to_array(),
            (b + side).to_array(),
            (b - side).to_array(),
        ],
        [0.0, 1.0, 0.0],
        [0.19, 0.125, 0.075, 1.0],
    );
}

pub(super) fn append(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    g: &GridGeometry,
    coordinates: [i32; 2],
) {
    let [w, _, n, _] = face_bounds(&positions);
    let origin = Vec3::new(w, positions[0][1], n);
    let first = mesh.positions.len();
    // Bake normals/shading near zero before translation so a camera grid shift
    // cannot change the folds through subtraction of large world coordinates.
    append_local(
        mesh,
        positions.map(|p| (Vec3::from_array(p) - origin).to_array()),
        g,
        coordinates,
    );
    for p in &mut mesh.positions[first..] {
        *p = (Vec3::from_array(*p) + origin).to_array();
    }
}

fn append_local(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    g: &GridGeometry,
    coordinates: [i32; 2],
) {
    let [w, e, n, s] = face_bounds(&positions);
    let [x, z] = coordinates;
    let unit = g.tile_width.min(g.tile_height);
    let base = positions[0][1] + unit * 0.002;
    let patch = meadow_noise(x as f32 * 0.23 + 17.0, z as f32 * 0.23 - 11.0);
    // Broad patches control density, avoiding equal-count confetti in every cell.
    let count = 4 + (patch * 9.0) as i32;
    for i in 0..count {
        let seed = lattice(x.wrapping_mul(7) + i, z.wrapping_mul(13) + 83);
        if seed < 0.18 {
            continue;
        }
        let center = Vec3::new(
            w + (e - w) * (0.22 + lattice(x + i * 19, z + 109) * 0.56),
            base,
            n + (s - n) * (0.22 + lattice(x + 211, z + i * 23) * 0.56),
        );
        let angle = lattice(x + 41, z + i * 31) * std::f32::consts::TAU;
        let color = match (seed * 5.0) as usize {
            0 => [0.27, 0.205, 0.125, 1.0],
            1 => [0.36, 0.265, 0.14, 1.0],
            2 => [0.42, 0.32, 0.18, 1.0],
            3 => [0.28, 0.295, 0.145, 1.0],
            _ => [0.32, 0.225, 0.13, 1.0],
        };
        leaf(
            mesh,
            center,
            unit * (0.07 + seed * 0.075),
            unit * (0.035 + seed * 0.025),
            angle,
            unit * (0.002 + seed * 0.004),
            color,
        );
    }
    let seed = lattice(x + 379, z - 101);
    if seed > 0.83 {
        let center = Vec3::new((w + e) * 0.5, base, (n + s) * 0.5);
        let angle = seed * std::f32::consts::TAU;
        let direction = Vec3::new(angle.cos(), 0.0, angle.sin()) * unit * 0.22;
        stick(mesh, center - direction, center + direction, unit * 0.009);
        let fork = Vec3::new(-direction.z, 0.0, direction.x) * 0.5;
        stick(mesh, center, center + direction * 0.55 + fork, unit * 0.006);
    }
    // Low fern clusters favor mossy patches and stay well below the actor's body.
    if patch > 0.48 && seed < 0.23 {
        let center = Vec3::new((w + e) * 0.5, base, (n + s) * 0.5);
        for frond in 0..(3 + (seed * 9.0) as i32) {
            let angle = seed * 9.0 + frond as f32 * 2.1;
            let direction = Vec3::new(angle.cos(), 0.0, angle.sin());
            let tip = center + direction * unit * 0.16 + Vec3::Y * unit * 0.19;
            let side = Vec3::new(-direction.z, 0.0, direction.x) * unit * 0.006;
            let a = (center - side).to_array();
            let b = (center + side).to_array();
            let c = (tip + side).to_array();
            let d = (tip - side).to_array();
            triangle(mesh, [a, b, c], [0.19, 0.27, 0.12, 1.0]);
            triangle(mesh, [a, c, d], [0.19, 0.27, 0.12, 1.0]);
            for step in 1..=4 {
                let t = step as f32 / 4.0;
                let origin = center + direction * unit * 0.16 * t + Vec3::Y * unit * 0.19 * t;
                for side in [-1.0, 1.0] {
                    leaf(
                        mesh,
                        origin,
                        unit * (0.055 - 0.022 * t),
                        unit * 0.022,
                        angle + side * 0.8,
                        unit * 0.011,
                        [0.22, 0.31, 0.145, 1.0],
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floor_details_stay_in_the_source_cell_and_follow_world_coordinates() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let p = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        for z in 0..16 {
            for x in 0..16 {
                let mut a = SurfaceMeshData::default();
                append(&mut a, p, &g, [x, z]);
                assert!(a.positions.iter().all(|v| v[0] > 0.
                    && v[0] < 8.
                    && v[2] > 0.
                    && v[2] < 8.
                    && v[1] > 0.
                    && v[1] < 2.0));
                let mut b = SurfaceMeshData::default();
                append(&mut b, p.map(|v| [v[0] + 8., v[1], v[2] - 8.]), &g, [x, z]);
                assert!(
                    a.colors
                        .iter()
                        .zip(&b.colors)
                        .all(|(a, b)| a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.00001))
                );
                for (a, b) in a.positions.iter().zip(&b.positions) {
                    assert!(
                        (a[0] + 8. - b[0]).abs() < 0.00001 && (a[2] - 8. - b[2]).abs() < 0.00001
                    );
                }
            }
        }
    }
}
