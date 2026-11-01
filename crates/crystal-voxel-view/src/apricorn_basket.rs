//! Kurt's open woven harvest basket, attached through the production chest slot.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;

fn color(rgb: [u8; 3]) -> [f32; 4] {
    let c = rgb.map(|v| { let s = v as f32 / 255.;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) } });
    [c[0], c[1], c[2], 1.]
}

fn vertex(m: &mut SurfaceMeshData, p: Vec3, n: Vec3, rgb: [u8; 3]) {
    m.positions.push(p.to_array()); m.normals.push(n.normalize().to_array());
    m.colors.push(color(rgb)); m.uvs.push([0.; 2]);
}

fn face(m: &mut SurfaceMeshData, mut f: [u32; 3]) {
    let p = f.map(|i| Vec3::from_array(m.positions[i as usize]));
    let n = f.map(|i| Vec3::from_array(m.normals[i as usize])).into_iter().sum::<Vec3>();
    if (p[1] - p[0]).cross(p[2] - p[0]).dot(n) < 0. { f.swap(1, 2); }
    m.indices.extend(f);
}

fn grid(m: &mut SurfaceMeshData, start: u32, rows: usize, sides: usize) {
    for row in 0..rows { for side in 0..sides {
        let a = start + (row * (sides + 1) + side) as u32;
        let b = a + (sides + 1) as u32;
        face(m, [a, b, a + 1]); face(m, [a + 1, b, b + 1]);
    }}
}

fn tube(m: &mut SurfaceMeshData, path: &[Vec3], radius: f32, rgb: [u8; 3]) {
    const SIDES: usize = 12;
    let start = m.positions.len() as u32;
    for (i, &p) in path.iter().enumerate() {
        let tangent = (path[(i + 1).min(path.len() - 1)] - path[i.saturating_sub(1)]).normalize();
        let axis = if tangent.x.abs() < 0.8 { Vec3::X } else { Vec3::Y };
        let u = tangent.cross(axis).normalize(); let v = tangent.cross(u).normalize();
        for side in 0..=SIDES {
            let a = std::f32::consts::TAU * side as f32 / SIDES as f32;
            let n = u * a.cos() + v * a.sin(); vertex(m, p + n * radius, n, rgb);
        }
    }
    grid(m, start, path.len() - 1, SIDES);
}

fn fruit(m: &mut SurfaceMeshData, center: Vec3, rgb: [u8; 3]) {
    const ROWS: usize = 18; const SIDES: usize = 24;
    let start = m.positions.len() as u32;
    let radii = Vec3::new(0.077, 0.089, 0.077);
    for row in 0..=ROWS { for side in 0..=SIDES {
        let t = std::f32::consts::PI * row as f32 / ROWS as f32;
        let a = std::f32::consts::TAU * side as f32 / SIDES as f32;
        let n = Vec3::new(t.sin() * a.cos(), t.cos(), t.sin() * a.sin());
        vertex(m, center + radii * n, n / radii, rgb);
    }}
    grid(m, start, ROWS, SIDES);
    tube(m, &[center + Vec3::Y * 0.073, center + Vec3::new(0.014, 0.111, 0.)], 0.009, [79, 70, 35]);
}

fn build() -> SurfaceMeshData {
    const SIDES: usize = 96; const ROWS: usize = 24;
    let mut m = SurfaceMeshData::default();
    // Both surfaces meet a rolled rim. A shallow taper and subdued alternating
    // courses suggest wicker at game scale without individual noisy sticks.
    for inner in [false, true] {
        let start = m.positions.len() as u32;
        for row in 0..=ROWS { for side in 0..=SIDES {
            let t = row as f32 / ROWS as f32;
            let a = std::f32::consts::TAU * side as f32 / SIDES as f32;
            let weave = 0.003 * (a * 24.).sin() * (t * 12. * std::f32::consts::PI).cos();
            let radius = 0.215 + t * 0.075 + weave - if inner { 0.018 } else { 0. };
            let base = if inner { 1.508 } else { 1.49 };
            let p = Vec3::new(radius * a.sin(), base + t * (1.98 - base), -0.49 + radius * a.cos() * 0.67);
            let n = Vec3::new(a.sin(), -0.075 / 0.49, a.cos() / 0.67) * if inner { -1. } else { 1. };
            vertex(&mut m, p, n, if row % 4 < 2 { [154, 113, 62] } else { [171, 130, 75] });
        }}
        grid(&mut m, start, ROWS, SIDES);
        let center = m.positions.len() as u32;
        vertex(&mut m, Vec3::new(0., if inner { 1.508 } else { 1.49 }, -0.49), if inner { Vec3::Y } else { Vec3::NEG_Y }, [135, 97, 52]);
        for side in 0..SIDES { face(&mut m, [center, start + side as u32, start + side as u32 + 1]); }
    }
    let rim = (0..=SIDES).map(|i| { let a = std::f32::consts::TAU * i as f32 / SIDES as f32;
        Vec3::new(0.283 * a.sin(), 1.98, -0.49 + 0.189 * a.cos()) }).collect::<Vec<_>>();
    tube(&mut m, &rim, 0.021, [130, 91, 47]);
    for (p, c) in [(Vec3::new(-0.11, 1.995, -0.49), [129, 63, 51]),
        (Vec3::new(0.075, 2.005, -0.55), [95, 119, 57]),
        (Vec3::new(0.085, 1.985, -0.40), [178, 143, 61])] { fruit(&mut m, p, c); }
    for side in [-1., 1.] {
        // Strap ends sit inside the basket and route over the actual shoulder.
        let controls = [Vec3::new(side * 0.21, 1.72, -0.30), Vec3::new(side * 0.25, 2.13, -0.20),
            Vec3::new(side * 0.25, 2.49, -0.145), Vec3::new(side * 0.25, 2.535, 0.01),
            Vec3::new(side * 0.25, 2.49, 0.13), Vec3::new(side * 0.25, 2.24, 0.195),
            Vec3::new(side * 0.25, 1.83, 0.25), Vec3::new(side * 0.21, 1.61, -0.30)];
        let mut path = Vec::new();
        for pair in controls.windows(2) { for step in 0..8 { path.push(pair[0].lerp(pair[1], step as f32 / 8.)); }}
        path.push(*controls.last().unwrap()); tube(&mut m, &path, 0.020, [106, 78, 47]);
    }
    m
}

pub(crate) fn mesh(rig: &PersonRig) -> SurfaceMeshData {
    let mut m = build(); let bind = rig.inverse_binds[rig.chest_joint];
    let normal = bind.inverse().transpose();
    for (p, n) in m.positions.iter_mut().zip(&mut m.normals) {
        *p = bind.transform_point3(Vec3::from_array(*p)).to_array();
        *n = normal.transform_vector3(Vec3::from_array(*n)).normalize().to_array();
    }
    m
}
