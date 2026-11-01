//! Smooth scalp/hair transitions projected from the supplied head UV surface.
use crate::{npc_appearance::Look, open_people::PersonRig};
use bevy::prelude::*;

pub(crate) fn paint(rig: &PersonRig, out: &mut [u8], size: (u32, u32), look: Look) {
    if look.hairstyle != "bald" || out.len() != (size.0 * size.1 * 4) as usize {
        return;
    }
    let head: Vec<_> = rig
        .ids
        .iter()
        .zip(&rig.weights)
        .map(|(j, w)| {
            (0..4)
                .filter(|&i| j[i] as usize == rig.head_joint)
                .map(|i| w[i])
                .sum::<f32>()
                > 0.5
        })
        .collect();
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for (&p, &head) in rig.mesh.positions.iter().zip(&head) {
        if head {
            lo = lo.min(Vec3::from_array(p));
            hi = hi.max(Vec3::from_array(p));
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return;
    }
    let center = (lo + hi) * 0.5;
    let half = (hi - lo) * 0.5;
    // Sample clean supplied forehead painting rather than introducing a second
    // skin palette. Never sample eyes, brows, nose or mouth.
    let mut skin = [0_f32; 3];
    let mut samples = 0_f32;
    for y in 43..48 {
        for x in 74..88 {
            let at = (((y * rig.texture_size.1 / 256) * rig.texture_size.0
                + x * rig.texture_size.0 / 256)
                * 4) as usize;
            for a in 0..3 {
                skin[a] += rig.rgba[at + a] as f32;
            }
            samples += 1.;
        }
    }
    skin = skin.map(|v| v / samples);
    let smooth = |t: f32| {
        let t = t.clamp(0., 1.);
        t * t * (3. - 2. * t)
    };
    let original = out.to_vec();
    let mut coverage_distance = vec![f32::INFINITY; (size.0 * size.1) as usize];
    for face in rig.mesh.indices.chunks_exact(3) {
        if !face.iter().all(|&i| head[i as usize]) {
            continue;
        }
        let p =
            [face[0], face[1], face[2]].map(|i| Vec3::from_array(rig.mesh.positions[i as usize]));
        let uv = [face[0], face[1], face[2]].map(|i| {
            Vec2::from_array(rig.mesh.uvs[i as usize]) * Vec2::new(size.0 as f32, size.1 as f32)
        });
        let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
        let area = cross(uv[1] - uv[0], uv[2] - uv[0]);
        if area.abs() < 1e-6 {
            continue;
        }
        let min = uv[0].min(uv[1]).min(uv[2]) - Vec2::splat(2.);
        let max = uv[0].max(uv[1]).max(uv[2]) + Vec2::splat(2.);
        for y in (min.y.floor() as i32).max(0)..=(max.y.ceil() as i32).min(size.1 as i32 - 1) {
            for x in (min.x.floor() as i32).max(0)..=(max.x.ceil() as i32).min(size.0 as i32 - 1) {
                let q = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let b = cross(q - uv[0], uv[2] - uv[0]) / area;
                let c = cross(uv[1] - uv[0], q - uv[0]) / area;
                let a = 1. - b - c;
                // A narrow gutter prevents linear sampling from exposing the
                // old hair color at the chart seam. Clamp to the head surface.
                let bary = Vec3::new(a, b, c).max(Vec3::ZERO);
                let bary = bary / bary.element_sum();
                let distance = q.distance(uv[0] * bary.x + uv[1] * bary.y + uv[2] * bary.z);
                if distance > 1.5 {
                    continue;
                }
                let [a, b, c] = bary.to_array();
                let ux = x as u32 * 256 / size.0;
                let uy = y as u32 * 256 / size.1;
                // Face and ear interiors retain the supplied painting. Only
                // original dark hair pixels in the head chart are recolored.
                let ear = uy >= 51 && ((40..60).contains(&ux) || (103..121).contains(&ux));
                if ux >= 160 || uy >= 75 || ((60..=102).contains(&ux) && uy >= 48) || ear {
                    continue;
                }
                let sx = x as u32 * rig.texture_size.0 / size.0;
                let sy = y as u32 * rig.texture_size.1 / size.1;
                let source = ((sy * rig.texture_size.0 + sx) * 4) as usize;
                if rig.rgba[source] >= 230
                    || rig.rgba[source + 1] >= 200
                    || rig.rgba[source + 2] >= 190
                {
                    continue;
                }
                let point = p[0] * a + p[1] * b + p[2] * c;
                let height = (point.y - center.y) / half.y;
                let front = smooth(((point.z - center.z) / half.z - 0.10) / 0.55);
                let line = 0.46 - 0.62 * front;
                let band = smooth((height + 0.20) / 0.12)
                    * (1. - smooth((height - line + 0.06) / 0.12))
                    * (1. - front);
                let shade = 0.94 + 0.06 * smooth((height + 0.15) / 1.15);
                let coverage = smooth((230. - rig.rgba[source] as f32) / 35.);
                let at = ((y as u32 * size.0 + x as u32) * 4) as usize;
                // Interior pixels win over neighboring triangle gutters;
                // overlapping gutters must not repeatedly blend one pixel.
                if distance > coverage_distance[at / 4] {
                    continue;
                }
                coverage_distance[at / 4] = distance;
                for axis in 0..3 {
                    let hair = look.hair[axis] as f32 * 0.72;
                    let target = skin[axis] * shade * (1. - band) + hair * band;
                    out[at + axis] = (original[at + axis] as f32 * (1. - coverage)
                        + target * coverage)
                        .round() as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_bark_actors::character_meshes::CharacterKind;
    #[test]
    #[ignore = "requires local Kenney rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_scalps_keep_face_ears_body_alpha_and_other_styles_unchanged() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        for kind in [
            CharacterKind::Blaine,
            CharacterKind::Elder,
            CharacterKind::Pharmacist,
            CharacterKind::PokefanM,
            CharacterKind::Sage,
        ] {
            let look = crate::npc_appearance::look(kind);
            let before = crate::npc_appearance::texture(&rig.rgba, rig.texture_size, look);
            let mut after = before.clone();
            paint(&rig, &mut after, rig.texture_size, look);
            let mut changed = 0;
            for (i, (a, b)) in before
                .chunks_exact(4)
                .zip(after.chunks_exact(4))
                .enumerate()
            {
                let x = i as u32 % rig.texture_size.0 * 256 / rig.texture_size.0;
                let y = i as u32 / rig.texture_size.0 * 256 / rig.texture_size.1;
                assert_eq!(a[3], b[3]);
                let ear = y >= 51 && ((40..60).contains(&x) || (103..121).contains(&x));
                if y >= 75 || x >= 160 || ((60..=102).contains(&x) && y >= 48) || ear {
                    assert_eq!(a, b, "{} altered face/body at {x},{y}", look.texture_key);
                }
                // Ear interiors are colored source skin, never a hair mask.
                let at = i * 4;
                if rig.rgba[at] >= 230 || rig.rgba[at + 1] >= 200 || rig.rgba[at + 2] >= 190 {
                    assert_eq!(a, b);
                }
                changed += usize::from(a != b);
            }
            assert!(
                changed > 500,
                "{} no meaningful scalp repaint",
                look.texture_key
            );
            let mut repeat = before;
            paint(&rig, &mut repeat, rig.texture_size, look);
            assert_eq!(after, repeat);
        }
        let look = crate::npc_appearance::look(CharacterKind::Officer);
        let before = crate::npc_appearance::texture(&rig.rgba, rig.texture_size, look);
        let mut after = before.clone();
        paint(&rig, &mut after, rig.texture_size, look);
        assert_eq!(before, after);
    }
}
