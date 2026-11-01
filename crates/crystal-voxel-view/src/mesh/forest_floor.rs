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
    append_with_canopy(mesh, positions, g, coordinates, 1.);
}

pub(super) fn append_with_canopy(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    g: &GridGeometry,
    coordinates: [i32; 2],
    canopy: f32,
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
        canopy,
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
    canopy: f32,
) {
    let [w, e, n, s] = face_bounds(&positions);
    let [x, z] = coordinates;
    let unit = g.tile_width.min(g.tile_height);
    let base = positions[0][1] + unit * 0.002;
    let patch = meadow_noise(x as f32 * 0.23 + 17.0, z as f32 * 0.23 - 11.0) + 0.5;
    // Broad patches control density, avoiding equal-count confetti in every cell.
    // Accumulate litter in connected pockets, leaving other cells bare.
    // Fewer, larger leaves read as a carpet rather than uniform confetti.
    let count = if patch > 0.54 {
        3 + ((patch - 0.54) * 9.0) as i32
    } else {
        0
    };
    let count = (count as f32 * canopy.clamp(0., 1.)).round() as i32;
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
            0 => [0.095, 0.071, 0.081, 1.0],
            1 => [0.125, 0.091, 0.096, 1.0],
            2 => [0.155, 0.122, 0.113, 1.0],
            3 => [0.082, 0.115, 0.104, 1.0],
            _ => [0.115, 0.079, 0.101, 1.0],
        };
        leaf(
            mesh,
            center,
            unit * (0.10 + seed * 0.09),
            unit * (0.042 + seed * 0.034),
            angle,
            unit * (0.002 + seed * 0.004),
            color,
        );
    }
    // Keep sparse mineral grain, without generic extra flowers or a second
    // warm leaf layer. Source flower anchors are authored separately.
    let first_stone = mesh.colors.len();
    append_scatter(mesh, positions, g, coordinates, false, false);
    for color in &mut mesh.colors[first_stone..] {
        for channel in 0..3 { color[channel] *= 0.55; }
    }
    let seed = lattice(x + 379, z - 101);
    if canopy > 0.2 && seed > 0.83 {
        let center = Vec3::new((w + e) * 0.5, base, (n + s) * 0.5);
        let angle = seed * std::f32::consts::TAU;
        let direction = Vec3::new(angle.cos(), 0.0, angle.sin()) * unit * 0.22;
        stick(mesh, center - direction, center + direction, unit * 0.009);
        let fork = Vec3::new(-direction.z, 0.0, direction.x) * 0.5;
        stick(mesh, center, center + direction * 0.55 + fork, unit * 0.006);
    }
    // Low fern clusters favor mossy patches and stay well below the actor's body.
    if canopy > 0.2 && patch > 0.60 && seed < 0.38 {
        let center = Vec3::new((w + e) * 0.5, base, (n + s) * 0.5);
        for frond in 0..3 {
            let angle = seed * 9.0 + frond as f32 * 2.1;
            let direction = Vec3::new(angle.cos(), 0.0, angle.sin());
            let tip = center + direction * unit * 0.22 + Vec3::Y * unit * 0.29;
            let side = Vec3::new(-direction.z, 0.0, direction.x) * unit * 0.006;
            let a = (center - side).to_array();
            let b = (center + side).to_array();
            let c = (tip + side).to_array();
            let d = (tip - side).to_array();
            triangle(mesh, [a, b, c], [0.19, 0.27, 0.12, 1.0]);
            triangle(mesh, [a, c, d], [0.19, 0.27, 0.12, 1.0]);
            for step in 1..=3 {
                let t = step as f32 / 3.0;
                let origin = center + direction * unit * 0.22 * t + Vec3::Y * unit * 0.29 * t;
                for side in [-1.0, 1.0] {
                    leaf(
                        mesh,
                        origin,
                        unit * (0.085 - 0.032 * t),
                        unit * 0.034,
                        angle + side * 0.8,
                        unit * 0.011,
                        [0.22, 0.31, 0.145, 1.0],
                    );
                }
            }
        }
    }
}

/// Sparse accents share the opaque ground batch: no entities, textures,
/// transparency, animation systems or additional materials. All placement is
/// authored-cell local and seeded in map space, including negative origins.
pub(super) fn append_scatter(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    g: &GridGeometry,
    coordinates: [i32; 2],
    woodland: bool,
    flowers: bool,
) {
    let [w, e, n, s] = face_bounds(&positions);
    let unit = g.tile_width.min(g.tile_height).min(e - w).min(s - n);
    let [x, z] = coordinates;
    let seed = lattice(x.wrapping_add(719), z.wrapping_sub(317));
    // Build near zero to keep face normals stable when the viewport scrolls.
    let origin = Vec3::new(w, positions[0][1], n);
    let first = mesh.positions.len();
    let center = Vec3::new(
        (e - w) * (0.28 + lattice(x + 127, z - 59) * 0.44),
        unit * 0.003,
        (s - n) * (0.28 + lattice(x - 83, z + 239) * 0.44),
    );
    let angle = seed * std::f32::consts::TAU;
    if seed < 0.18 {
        // A low irregular six-sided stone, with a mineral ridge rather than
        // a flat speck. Small enough to keep the walking surface readable.
        let radius = unit * (0.08 + seed * 0.38);
        let top = center + Vec3::new(radius * 0.12, radius * 0.55, -radius * 0.1);
        for side in 0..6 {
            let at = |i: i32| {
                let a = angle + i as f32 * std::f32::consts::TAU / 6.0;
                center + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius * 0.75)
            };
            let color = if woodland {
                [0.35, 0.37, 0.29, 1.0]
            } else {
                [0.48 + seed * 0.4, 0.43 + seed * 0.3, 0.34 + seed * 0.2, 1.0]
            };
            triangle(
                mesh,
                [top.to_array(), at(side).to_array(), at(side + 1).to_array()],
                color,
            );
        }
    } else if woodland
        && seed < 0.43
        && meadow_noise(x as f32 * 0.23 + 17.0, z as f32 * 0.23 - 11.0) > 0.04
    {
        for i in 0..2 {
            leaf(
                mesh,
                center + Vec3::new(i as f32 * unit * 0.10, 0.0, 0.0),
                unit * 0.11,
                unit * 0.045,
                angle + i as f32 * 1.7,
                unit * 0.005,
                [0.43, 0.29 + seed * 0.12, 0.12, 1.0],
            );
        }
    } else if flowers
        && seed > 0.72
        && meadow_noise(x as f32 * 0.19 - 13.0, z as f32 * 0.19 + 29.0) > 0.12
    {
        // Tiny clusters rather than a flower on every tile. Five opaque petals
        // sit below ankle height; the warm center remains legible at game scale.
        let color = if seed > 0.96 {
            [0.79, 0.69, 0.88, 1.0]
        } else {
            [0.91, 0.84, 0.57, 1.0]
        };
        for bloom in 0..3 {
            let a = angle + bloom as f32 * 2.094395;
            let root =
                center + Vec3::new(a.cos() * unit * 0.11, unit * 0.065, a.sin() * unit * 0.11);
            triangle(
                mesh,
                [
                    (root - Vec3::Y * unit * 0.062 - Vec3::X * unit * 0.006).to_array(),
                    (root - Vec3::Y * unit * 0.062 + Vec3::X * unit * 0.006).to_array(),
                    root.to_array(),
                ],
                [0.19, 0.31, 0.09, 1.0],
            );
            for petal in 0..5 {
                let a = angle + petal as f32 * std::f32::consts::TAU / 5.0;
                let d = Vec3::new(a.cos(), 0.0, a.sin()) * unit * 0.092;
                let side = Vec3::new(-a.sin(), 0.0, a.cos()) * unit * 0.042;
                triangle(
                    mesh,
                    [
                        root.to_array(),
                        (root + d + side).to_array(),
                        (root + d - side).to_array(),
                    ],
                    color,
                );
            }
            let heart = root + Vec3::Y * unit * 0.002;
            triangle(
                mesh,
                [
                    (heart + Vec3::X * unit * 0.019).to_array(),
                    (heart + Vec3::Z * unit * 0.019).to_array(),
                    (heart - Vec3::X * unit * 0.019 - Vec3::Z * unit * 0.019).to_array(),
                ],
                [0.82, 0.52, 0.13, 1.0],
            );
            leaf(
                mesh,
                root - Vec3::Y * unit * 0.04,
                unit * 0.08,
                unit * 0.025,
                angle,
                unit * 0.01,
                [0.20, 0.34, 0.11, 1.0],
            );
        }
    }
    for p in &mut mesh.positions[first..] {
        *p = (Vec3::from_array(*p) + origin).to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clearing_stays_quiet_and_tree_contacts_keep_bounded_litter() {
        let g = GridGeometry { width: 1, height: 1, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let p = [[0.,0.,0.],[0.,0.,8.],[8.,0.,8.],[8.,0.,0.]];
        let mut clear_triangles = 0;
        let mut canopy_triangles = 0;
        for z in -8..8 { for x in -8..8 {
            let mut clear = SurfaceMeshData::default();
            let mut canopy = SurfaceMeshData::default();
            append_with_canopy(&mut clear,p,&g,[x,z],0.);
            append_with_canopy(&mut canopy,p,&g,[x,z],1.);
            assert!(clear.indices.len() <= 18, "clearing has only sparse mineral grain");
            assert!(clear.positions.iter().all(|p|p[1]<1.));
            assert!(canopy.positions.iter().all(|p|p[0]>0. && p[0]<8. && p[2]>0. && p[2]<8.));
            clear_triangles += clear.indices.len()/3;
            canopy_triangles += canopy.indices.len()/3;
        }}
        assert!(canopy_triangles > clear_triangles * 3);
        assert!(clear_triangles < 256 * 2, "quiet walking space must dominate");
    }

    #[test]
    fn accents_are_sparse_bounded_and_stable_across_scrolling() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let p = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        let mut empty = 0;
        let mut stones = 0;
        let mut blooms = 0;
        let mut ferns = 0;
        let mut accent_triangles = 0;
        let mut forest_triangles = 0;
        for z in -16..16 {
            for x in -16..16 {
                let mut a = SurfaceMeshData::default();
                append_scatter(&mut a, p, &g, [x, z], true, true);
                assert!(a.indices.len() / 3 <= 39, "bounded accents per cell");
                assert!(a.positions.iter().all(|v| v[0] > 0.
                    && v[0] < 8.
                    && v[2] > 0.
                    && v[2] < 8.
                    && v[1] > 0.
                    && v[1] < 1.));
                accent_triangles += a.indices.len() / 3;
                match a.indices.len() / 3 {
                    0 => empty += 1,
                    6 => stones += 1,
                    39 => blooms += 1,
                    _ => {}
                }
                let mut b = SurfaceMeshData::default();
                append_scatter(
                    &mut b,
                    p.map(|v| [v[0] + 64., v[1], v[2] - 32.]),
                    &g,
                    [x, z],
                    true,
                    true,
                );
                assert_eq!(a.colors, b.colors);
                assert_eq!(a.normals, b.normals);
                assert_eq!(a.indices, b.indices);
                for (a, b) in a.positions.iter().zip(&b.positions) {
                    assert!((a[0] + 64. - b[0]).abs() < 0.00001);
                    assert!((a[2] - 32. - b[2]).abs() < 0.00001);
                }
                let mut forest = SurfaceMeshData::default();
                append(&mut forest, p, &g, [x, z]);
                assert!(forest.indices.len() / 3 <= 220, "fern and litter budget");
                forest_triangles += forest.indices.len() / 3;
                if forest.positions.iter().any(|v| v[1] > 1.0) {
                    ferns += 1;
                }
            }
        }
        assert!(accent_triangles < 1024 * 10, "average accent budget");
        assert!(
            forest_triangles < 1024 * 64,
            "average woodland detail budget"
        );
        eprintln!(
            "terrain detail triangles/cell: accents={:.2} forest={:.2}",
            accent_triangles as f32 / 1024.0,
            forest_triangles as f32 / 1024.0
        );
        assert!(
            ferns > 20,
            "centered noise must actually permit fern patches"
        );
        assert!(
            empty > 350 && stones > 100 && blooms > 20,
            "patches must include clear floor, stones and flowers"
        );
    }

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
                    && v[1] < 3.0));
                let mut b = SurfaceMeshData::default();
                append(&mut b, p.map(|v| [v[0] + 8., v[1], v[2] - 8.]), &g, [x, z]);
                assert!(a
                    .colors
                    .iter()
                    .zip(&b.colors)
                    .all(|(a, b)| a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.00001)));
                for (a, b) in a.positions.iter().zip(&b.positions) {
                    assert!(
                        (a[0] + 8. - b[0]).abs() < 0.00001 && (a[2] - 8. - b[2]).abs() < 0.00001
                    );
                }
            }
        }
    }
}
