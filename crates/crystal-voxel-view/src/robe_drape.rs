//! Soft folds and existing painted fabric on Kurt's lower robe.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, npc_textures::Texture};
use bevy::prelude::*;
use std::collections::BTreeMap;

fn linear(v: u8) -> f32 {
    let s = f32::from(v) / 255.;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn fabric(painting: &Texture, angle: f32, t: f32) -> Vec3 {
    // Only the plain left robe island: exclude crossed collar, center seam,
    // shoe/hand/skin islands. Periodic coordinates keep the rear seam matched.
    let u = 0.075 + 0.035 * angle.sin();
    let v = 0.62 + 0.32 * t;
    let (w, h) = painting.size;
    let x = (u * w as f32) as i32;
    let y = (v * h as f32) as i32;
    let mut sum = Vec3::ZERO;
    for dy in -2..=2 {
        for dx in -2..=2 {
            let px = (x + dx).clamp(0, w as i32 - 1) as u32;
            let py = (y + dy).clamp(0, h as i32 - 1) as u32;
            let at = ((py * w + px) * 4) as usize;
            sum += Vec3::from_array(std::array::from_fn(|i| linear(painting.rgba[at + i])));
        }
    }
    sum / 25.
}

pub(crate) fn finish(mesh: &mut SurfaceMeshData, look: Look, bottom: f32) {
    if !matches!(look.texture_key, "kurt" | "kurt_outside") {
        return;
    }
    let Some(painting) = crate::npc_textures::get(look.texture_key) else {
        return;
    };
    drape(mesh, painting, bottom);
}

fn drape(mesh: &mut SurfaceMeshData, painting: &Texture, bottom: f32) {
    let smooth = |x: f32| {
        let t = x.clamp(0., 1.);
        t * t * (3. - 2. * t)
    };
    let mut changed = vec![false; mesh.positions.len()];
    for (i, p) in mesh.positions.iter_mut().enumerate() {
        // Keep the exact fitted waist and its normal field. Fold amplitude
        // grows below it, with only outward radial motion on both shell layers.
        let fade = smooth((1.52 - p[1]) / 0.30);
        if fade == 0. {
            continue;
        }
        let radial = Vec2::new(p[0], p[2] - 0.05);
        let a = radial.x.atan2(radial.y);
        let t = ((1.52 - p[1]) / (1.52 - bottom)).clamp(0., 1.);
        let wave = (5. * a + 0.35 * t).sin();
        let amount = 0.020 * fade * wave * wave;
        let delta = radial.normalize_or_zero() * amount;
        p[0] += delta.x;
        p[2] += delta.y;
        let old = Vec3::new(mesh.colors[i][0], mesh.colors[i][1], mesh.colors[i][2]);
        let color = old.lerp(fabric(painting, a, t), fade);
        mesh.colors[i] = [color.x, color.y, color.z, mesh.colors[i][3]];
        changed[i] = true;
    }
    // Weld duplicate geometric seam vertices for the area-normal field,
    // retaining separate inner/outer surfaces and unchanged waist normals.
    let key = |p: [f32; 3]| p.map(|v| (v * 100_000.).round() as i32);
    let mut sums = BTreeMap::<[i32; 3], Vec3>::new();
    for f in mesh.indices.chunks_exact(3) {
        let p = [f[0], f[1], f[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in f {
            *sums.entry(key(mesh.positions[i as usize])).or_default() += n;
        }
    }
    for (i, n) in mesh.normals.iter_mut().enumerate() {
        if changed[i] {
            *n = sums[&key(mesh.positions[i])].normalize().to_array();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied Kurt atlas in CRYSTAL_OPEN_MODEL_ROOT"]
    fn painted_drape_preserves_fitted_waist_closed_shell_and_safe_fabric() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let painting = crate::npc_textures::decode(
            &std::fs::read(root.join("people/npc-textures/kurt.png")).unwrap(),
        )
        .unwrap();
        let mut mesh = SurfaceMeshData::default();
        // Closed two-layer shell with enough rows to include waist, fade and hem.
        let sides = 40;
        let rows = 21;
        let layer = (sides + 1) * rows;
        for inner in [false, true] {
            for row in 0..rows {
                for side in 0..=sides {
                    let a = std::f32::consts::TAU * side as f32 / sides as f32;
                    let y = 1.67 - row as f32 * 0.05;
                    let r = 0.36 + (1.67 - y) * 0.16 - if inner { 0.018 } else { 0. };
                    mesh.positions.push([r * a.sin(), y, 0.05 + r * a.cos()]);
                    mesh.normals.push([a.sin(), 0., a.cos()]);
                    mesh.colors.push([0.01, 0.03, 0.07, 1.]);
                    mesh.uvs.push([0.; 2]);
                }
            }
        }
        let mut quad = |a: usize, b: usize, c: usize, d: usize| {
            mesh.indices
                .extend([a as u32, b as u32, c as u32, a as u32, c as u32, d as u32])
        };
        for row in 0..rows - 1 {
            for side in 0..sides {
                let a = row * (sides + 1) + side;
                let b = a + sides + 1;
                quad(a, b, b + 1, a + 1);
                quad(a + layer, a + layer + 1, b + layer + 1, b + layer);
            }
        }
        for side in 0..sides {
            quad(side, side + 1, side + layer + 1, side + layer);
            let a = (rows - 1) * (sides + 1) + side;
            quad(a, a + layer, a + layer + 1, a + 1);
        }
        let before = mesh.clone();
        drape(&mut mesh, &painting, 0.67);
        assert_eq!(mesh.indices, before.indices);
        assert_eq!(mesh.uvs, before.uvs);
        let mut changed = 0;
        for i in 0..mesh.positions.len() {
            let p = Vec3::from_array(mesh.positions[i]);
            let q = Vec3::from_array(before.positions[i]);
            assert_eq!(p.y, q.y);
            assert!(p.distance(q) <= 0.02001);
            if q.y >= 1.52 {
                assert_eq!(p, q);
                assert_eq!(mesh.colors[i], before.colors[i]);
                assert_eq!(mesh.normals[i], before.normals[i]);
            } else {
                changed += usize::from(p.distance(q) > 0.005);
                let n = Vec3::from_array(mesh.normals[i]);
                assert!(n.is_finite() && (n.length() - 1.).abs() < 1e-5);
                let c = mesh.colors[i];
                assert!(c.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
                assert!(
                    c[2] > c[1] && c[1] > c[0] && c[2] < 0.15,
                    "wrong island color: {c:?}"
                );
            }
        }
        assert!(changed > 500);
        let key = |p: [f32; 3]| p.map(|v| (v * 100_000.).round() as i32);
        let mut edges = BTreeMap::<_, (u32, i32)>::new();
        for f in mesh.indices.chunks_exact(3) {
            let p = [f[0], f[1], f[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            assert!((p[1] - p[0]).cross(p[2] - p[0]).length() > 1e-7);
            for n in 0..3 {
                let a = key(mesh.positions[f[n] as usize]);
                let b = key(mesh.positions[f[(n + 1) % 3] as usize]);
                let e = edges.entry((a.min(b), a.max(b))).or_default();
                e.0 += 1;
                e.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(edges.values().all(|&e| e == (2, 0)), "drape opens shell");
        // Check the exact periodic duplicate columns on both shell layers.
        for base in [0, layer] {
            for row in 0..rows {
                let i = base + row * (sides + 1);
                let j = i + sides;
                assert!(
                    Vec3::from_array(mesh.positions[i])
                        .distance(Vec3::from_array(mesh.positions[j]))
                        < 1e-6
                );
                assert!(
                    Vec3::from_array(mesh.normals[i]).distance(Vec3::from_array(mesh.normals[j]))
                        < 1e-6
                );
                assert_eq!(mesh.colors[i], mesh.colors[j]);
            }
        }
    }
}
