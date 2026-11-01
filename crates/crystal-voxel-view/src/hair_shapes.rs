//! Scalp sculpting on the supplied skin, preserving its atlas and live rig.
use crate::{mesh::SurfaceMeshData, npc_appearance::Look, open_people::PersonRig};
use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) fn sculpt(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
    look: Look,
) -> f32 {
    if matches!(look.texture_key, "chris" | "kris") {
        capped_short(rig, mesh, ids, weights);
        return 0.;
    }
    if look.hat != "none" || !(matches!(look.hairstyle, "spikes" | "flattop" | "sidepart" | "curls" | "bob" | "long") || matches!(look.texture_key, "whitney" | "misty" | "janine"))
    {
        return 0.0;
    }
    let head = |i: usize, ids: &[[u16; 4]], weights: &[[f32; 4]]| {
        (0..4)
            .filter(|&j| ids[i][j] as usize == rig.head_joint)
            .map(|j| weights[i][j])
            .sum::<f32>()
            > 0.5
    };
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for (i, p) in mesh.positions.iter().enumerate() {
        if head(i, ids, weights) {
            lo = lo.min(Vec3::from_array(*p));
            hi = hi.max(Vec3::from_array(*p));
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return 0.0;
    }
    let center = (lo + hi) * 0.5;
    let half = (hi - lo) * 0.5;
    let threshold = center.y + half.y * 0.42;
    // Refine head faces, including their UV seam copies. Unchanged edges stay
    // collinear at the neck; the crown never meets an unrefined scalp edge.
    // Midpoints are shared along each indexed edge and remain in bind space.
    let refinements = if look.texture_key == "falkner" { 2 } else { 3 };
    for _ in 0..refinements {
        let needs_detail=mesh.indices.chunks_exact(3).any(|face| {
            let i=[face[0]as usize,face[1]as usize,face[2]as usize];
            if i.iter().any(|&i|!head(i,ids,weights)) {return false;}
            let p=i.map(|i|Vec3::from_array(mesh.positions[i])/half);
            [(0,1),(1,2),(2,0)].iter().any(|&(a,b)|p[a].distance(p[b])>0.16)
        });
        if !needs_detail {break;}
        let mut edges = HashMap::new();
        let old = std::mem::take(&mut mesh.indices);
        for face in old.chunks_exact(3) {
            let [a, b, c] = [face[0], face[1], face[2]];
            if ![a, b, c].iter().all(|&i| head(i as usize, ids, weights)) {
                mesh.indices.extend([a, b, c]);
                continue;
            }
            let mut midpoint = |a: u32, b: u32| -> u32 {
                *edges.entry((a.min(b), a.max(b))).or_insert_with(|| {
                    let (a, b) = (a as usize, b as usize);
                    let index = mesh.positions.len() as u32;
                    mesh.positions.push(std::array::from_fn(|j| {
                        (mesh.positions[a][j] + mesh.positions[b][j]) * 0.5
                    }));
                    mesh.normals.push(
                        (Vec3::from_array(mesh.normals[a]) + Vec3::from_array(mesh.normals[b]))
                            .normalize()
                            .to_array(),
                    );
                    mesh.colors.push(std::array::from_fn(|j| {
                        (mesh.colors[a][j] + mesh.colors[b][j]) * 0.5
                    }));
                    mesh.uvs.push(std::array::from_fn(|j| {
                        (mesh.uvs[a][j] + mesh.uvs[b][j]) * 0.5
                    }));
                    let mut combined = HashMap::<u16, f32>::new();
                    for i in [a, b] {
                        for j in 0..4 {
                            *combined.entry(ids[i][j]).or_default() += weights[i][j] * 0.5;
                        }
                    }
                    let mut joints: Vec<_> =
                        combined.into_iter().filter(|(_, w)| *w > 0.0).collect();
                    joints.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                    joints.truncate(4);
                    let sum = joints.iter().map(|(_, w)| w).sum::<f32>();
                    ids.push(std::array::from_fn(|j| joints.get(j).map_or(0, |p| p.0)));
                    weights.push(std::array::from_fn(|j| {
                        joints.get(j).map_or(0.0, |p| p.1 / sum)
                    }));
                    index
                })
            };
            let ab = midpoint(a, b);
            let bc = midpoint(b, c);
            let ca = midpoint(c, a);
            mesh.indices
                .extend([a, ab, ca, ab, b, bc, ca, bc, c, ab, bc, ca]);
        }
    }
    let clumps: &[(f32, f32, f32)] = match look.texture_key {
        "blue" => &[
            (-0.62, 0.22, 0.78),
            (-0.16, 0.44, 1.0),
            (0.43, 0.29, 0.90),
            (-0.46, -0.32, 0.88),
            (0.20, -0.35, 0.79),
            (0.61, -0.24, 0.65),
        ],
        "brock" => &[
            (-0.51, 0.25, 0.92),
            (0.0, 0.40, 1.0),
            (0.52, 0.24, 0.92),
            (-0.28, -0.33, 0.86),
            (0.28, -0.33, 0.86),
        ],
        "elm" => &[
            (-0.38, 0.31, 1.0),
            (0.32, 0.28, 0.55),
            (-0.12, -0.12, 0.64),
            (0.42, -0.40, 0.40),
        ],
        "koga" => &[(-0.48, -0.20, 0.85), (0.0, -0.40, 1.0), (0.48, -0.20, 0.85)],
        "lance" => &[
            (-0.47, -0.19, 0.85),
            (0.0, -0.45, 1.0),
            (0.49, -0.16, 0.88),
            (-0.23, 0.35, 0.64),
            (0.29, 0.33, 0.70),
        ],
        "oak" => &[(-0.39, 0.31, 0.74), (0.08, 0.44, 1.0), (0.46, 0.22, 0.67)],
        "rival" => &[
            (-0.62, -0.03, 0.75),
            (-0.28, 0.31, 0.85),
            (0.13, 0.27, 1.0),
            (0.55, -0.08, 0.92),
            (0.02, -0.38, 0.77),
        ],
        "cooltrainer_m" => &[
            (-0.48, 0.18, 0.72),
            (0.02, 0.41, 1.0),
            (0.49, 0.11, 0.82),
            (0.02, -0.36, 0.70),
        ],
        _ => &[
            (-0.48, 0.26, 0.90),
            (0.0, 0.41, 1.0),
            (0.48, 0.26, 0.90),
            (0.0, -0.35, 0.70),
        ],
    };
    let mut blends = vec![0.0; mesh.positions.len()];
    let mut extension = 0.0_f32;
    for i in 0..mesh.positions.len() {
        let side_curl =
            look.hairstyle == "curls" && (mesh.positions[i][0] - center.x).abs() > half.x * 0.70;
        let local_threshold = if side_curl {
            center.y + half.y * 0.20
        } else {
            threshold
        };
        if !head(i, ids, weights) || mesh.positions[i][1] <= local_threshold {
            continue;
        }
        let uv = mesh.uvs[i];
        let (w, h) = rig.texture_size;
        let x = (uv[0] * w as f32).clamp(0.0, w as f32 - 1.0) as usize;
        let y = (uv[1] * h as f32).clamp(0.0, h as f32 - 1.0) as usize;
        let pixel = &rig.rgba[(y * w as usize + x) * 4..][..3];
        if pixel[0] >= 125 || pixel[1] >= 120 || pixel[2] >= 115 {
            continue;
        }
        let original = Vec3::from_array(mesh.positions[i]);
        let mut p = original;
        let t = ((p.y - local_threshold) / (half.y * 0.33)).clamp(0.0, 1.0);
        let blend = t * t * (3.0 - 2.0 * t);
        blends[i] = blend;
        // Linear refinement alone preserves the source crown's long planar
        // facets. Round its support surface before adding authored hair masses,
        // fading to the exact source scalp at the hairline.
        let local = (p - center) / half;
        let rounded = center + local.normalize_or_zero() * half;
        p = p.lerp(rounded, blend * 0.80);
        let x = (p.x - center.x) / half.x;
        let z = (p.z - center.z) / half.z;
        if look.hairstyle == "sidepart" {
            let (part, volume, sweep) = match look.texture_key {
                // Low swept crown; the long asymmetric fringe is actual
                // geometry on the head accessory, not a raised central quiff.
                "falkner" => (0.42, 0.12, 0.06),
                // Keep Will's painted side part, with a low swept crown.
                // The taller mass read as a pointed hat in actual renders.
                "will" => (0.35, 0.12, -0.035),
                "morty" => (0.12, 0.24, -0.18),
                "super_nerd" => (0.30, 0.15, 0.06),
                _ => (-0.24, 0.20, 0.08),
            };
            // Two combed masses meet along a shallow, curved part. Preserve
            // the skin and scalp root; the groove is a recess in raised hair.
            let seam = part + 0.13 * z + 0.06 * z * z;
            let distance = x - seam;
            let groove = (-distance * distance / 0.015).exp();
            let broad = (-(x * x / 1.15 + z * z / 1.45)).exp();
            let large_side = if part > 0.0 { -1.0 } else { 1.0 };
            let sweep_mass = (-(distance - large_side * 0.26).powi(2) / 0.20).exp();
            p.y += half.y
                * volume
                * blend
                * broad
                * (0.45 + 0.55 * sweep_mass)
                * (1.0 - 0.45 * groove);
            p.x += half.x * large_side * volume * 0.22 * blend * (1.0 - groove);
            p.z += half.z * sweep * blend * (0.4 + 0.6 * broad);
        } else if look.hairstyle == "curls" {
            let mut curl = 0.0_f32;
            for (cx, cz, strength) in [
                (-0.65, 0.12, 0.88),
                (-0.32, 0.49, 1.0),
                (0.13, 0.56, 0.88),
                (0.52, 0.29, 0.95),
                (0.65, -0.16, 0.84),
                (0.36, -0.52, 0.96),
                (-0.08, -0.60, 0.85),
                (-0.50, -0.37, 0.91),
                (-0.26, 0.0, 0.94),
                (0.20, -0.04, 1.0),
                (-0.87, 0.30, 0.72),
                (-0.88, -0.18, 0.84),
                (0.88, 0.21, 0.80),
                (0.86, -0.29, 0.74),
            ] {
                let radius = (x - cx).powi(2) + (z - cz).powi(2);
                curl += (-radius / 0.043).exp() * strength;
            }
            p.y += half.y * 0.32 * curl * blend;
            p.x += half.x * 0.10 * x * curl * blend;
            p.z += half.z * 0.10 * z * curl * blend;
        } else if look.hairstyle == "bob" {
            let (part, volume) = match look.texture_key {
                "mom" => (-0.28, 0.045),
                "erika" => (0.18, 0.025),
                "bugsy" => (-0.10, 0.060),
                _ => (0.30, 0.035),
            };
            // The curtain is smooth, but its visible source crown was still
            // faceted. Keep a quiet swept crown rather than adding spikes.
            p.y += half.y * volume * blend * (-(x-part).powi(2)/0.6).exp();
        } else if look.hairstyle == "long" {
            let (part, volume, sweep) = match look.texture_key {
                "beauty" => (-0.25, 0.055, 0.025),
                "clair" => (0.25, 0.065, -0.035),
                "daisy" => (-0.15, 0.045, 0.015),
                "jasmine" => (-0.30, 0.035, 0.020),
                "karen" => (0.10, 0.060, -0.025),
                _ => (0.30, 0.050, -0.015),
            };
            // Long curtains already curve around the scalp; round the exposed
            // source crown with quiet swept volume and a shallow part.
            let seam = part + 0.10*z;
            let mass = (-(x-seam).powi(2)/0.60-z*z/1.50).exp();
            let groove = (-(x-seam).powi(2)/0.020).exp();
            p.y += half.y*volume*blend*mass*(1.0-0.35*groove);
            p.z += half.z*sweep*blend*mass;
        } else if look.hairstyle == "sidepony" {
            // A softly swept support surface leads toward Misty's side gather.
            // The source hairline and lower ponytail root remain fixed.
            let part = -0.25 + 0.10*z;
            let mass = (-(x-part).powi(2)/0.85-z*z/1.45).exp();
            let groove = (-(x-part).powi(2)/0.022).exp();
            p.y += half.y*0.050*blend*mass*(1.0-0.30*groove);
            p.z -= half.z*0.025*blend*mass;
        } else if look.hairstyle == "highpony" {
            // Round Janine's scalp toward the raised rear gather while
            // leaving the original hairline and attachment root fixed.
            let mass = (-x*x/1.10-(z+0.20).powi(2)/1.30).exp();
            p.y += half.y*0.040*blend*mass;
            p.z -= half.z*0.018*blend*mass;
        } else if look.hairstyle == "pigtails" {
            // Whitney's two gathers meet a shallow center part. Keep their
            // source roots fixed while rounding the broad polygonal crown.
            let part = 0.06 + 0.08*z;
            let mass = (-x*x/1.20-z*z/1.50).exp();
            let groove = (-(x-part).powi(2)/0.025).exp();
            p.y += half.y*0.045*blend*mass*(1.0-0.45*groove);
        } else if look.hairstyle == "flattop" {
            p.y = p.y.lerp(hi.y - half.y * 0.10, blend);
        } else {
            let (rise, sweep) = match look.texture_key {
                "blue" => (0.42, 0.15),
                "brock" => (0.30, 0.02),
                "elm" => (0.25, -0.05),
                "rival" => (0.40, -0.18),
                "lance" => (0.42, -0.12),
                "koga" => (0.25, -0.15),
                "oak" => (0.24, 0.12),
                "cooltrainer_m" => (0.33, 0.10),
                _ => (0.23, 0.0),
            };
            // Broad overlapping clumps rise from the original crown. Localized
            // peaks provide a ragged silhouette without detached cone roots.
            let mut peak = 0.0_f32;
            for &(cx, cz, strength) in clumps {
                let distance = (x - cx).powi(2) + (z - cz).powi(2);
                let mass = if matches!(look.texture_key, "lance" | "rival" | "blue" | "brock" | "elm") {
                    // Broader cosine clumps have rounded tips and fade into
                    // the scalp instead of forming three long conical peaks.
                    let t = (distance.sqrt()/0.50).clamp(0.0,1.0);
                    0.5*(1.0+(std::f32::consts::PI*t).cos())
                } else { (1.0 - distance.sqrt() / 0.42).max(0.0).powf(0.85) };
                peak = peak.max(mass * strength);
            }
            p.y += half.y * rise * peak * blend;
            p.z += half.z * sweep * blend * (0.3 + peak);
            p.x += half.x * 0.07 * x * peak * blend;
        }
        extension = extension.max(p.distance(original));
        mesh.positions[i] = p.to_array();
    }
    // Smooth across atlas seams while leaving untouched face normals intact.
    let key = |p: [f32; 3]| p.map(|v| (v * 100000.0).round() as i32);
    let mut normals = HashMap::<[i32; 3], Vec3>::new();
    for face in mesh.indices.chunks_exact(3) {
        if !face.iter().all(|&i| head(i as usize, ids, weights)) {
            continue;
        }
        let [a, b, c] = face.try_into().unwrap();
        let [a, b, c] = [a, b, c].map(|i: u32| Vec3::from_array(mesh.positions[i as usize]));
        let n = (b - a).cross(c - a);
        for &i in face {
            *normals.entry(key(mesh.positions[i as usize])).or_default() += n;
        }
    }
    for (i, n) in mesh.normals.iter_mut().enumerate() {
        if blends[i] > 0.0 {
            if let Some(sum) = normals.get(&key(mesh.positions[i])) {
                if sum.length_squared() > 1e-12 {
                    *n = Vec3::from_array(*n)
                        .lerp(sum.normalize(), blends[i])
                        .normalize()
                        .to_array();
                }
            }
        }
    }
    extension
}

/// Refine only the exposed dark rear scalp under the player's cap. The painted
/// hairline, skin/ears, front face and crown support retain their boundaries.
fn capped_short(rig: &PersonRig, mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>, weights: &mut Vec<[f32; 4]>) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for ((p, joints), weights) in mesh.positions.iter().zip(ids.iter()).zip(weights.iter()) {
        let head: f32 = (0..4).filter(|&j| joints[j] as usize == rig.head_joint)
            .map(|j| weights[j]).sum();
        if head > 0.5 {
            min = min.min(Vec3::from_array(*p));
            max = max.max(Vec3::from_array(*p));
        }
    }
    if !min.is_finite() || !max.is_finite() { return; }
    let center = (min + max) * 0.5;
    let half = (max - min) * 0.5;
    let (width, height) = rig.texture_size;
    let selection = |mesh: &SurfaceMeshData, ids: &[[u16; 4]], weights: &[[f32; 4]]| {
         mesh.positions.iter().zip(&mesh.uvs).zip(ids.iter()).zip(weights.iter())
            .map(|(((p, uv), joints), weights)| {
                let head: f32 = (0..4).filter(|&j| joints[j] as usize == rig.head_joint)
                    .map(|j| weights[j]).sum();
                if head < 0.85 || p[2] >= center.z + half.z * 0.3
                    || p[1] <= center.y - half.y * 0.45
                    || p[1] >= max.y - half.y * 0.10 { return false; }
                let x = (uv[0] * width as f32).clamp(0., width as f32 - 1.) as usize;
                let y = (uv[1] * height as f32).clamp(0., height as f32 - 1.) as usize;
                let pixel = &rig.rgba[(y * width as usize + x) * 4..][..3];
                pixel[0] < 125 && pixel[1] < 120 && pixel[2] < 115
            }).collect::<Vec<_>>()
    };
    let selected = selection(mesh, ids, weights);
    crate::hand_surface::smooth_selection(mesh, ids, weights, &selected);
    let selected = selection(mesh, ids, weights);
    let key = |p: [f32; 3]| p.map(|v| (v * 100_000.).round() as i32);
    let mut pinned = std::collections::BTreeSet::new();
    for face in mesh.indices.chunks_exact(3) {
        if face.iter().any(|&i| !selected[i as usize]) {
            for &i in face { pinned.insert(key(mesh.positions[i as usize])); }
        }
    }
    for i in 0..mesh.positions.len() {
        if !selected[i] || pinned.contains(&key(mesh.positions[i])) { continue; }
        let original = Vec3::from_array(mesh.positions[i]);
        let local = (original - center) / half;
        let lower = ((local.y + 0.45) / 0.45).clamp(0., 1.);
        let upper = ((0.90 - local.y) / 0.20).clamp(0., 1.);
        let blend = 0.65 * lower * upper;
        let rounded = center + local.normalize_or_zero() * half;
        let p = original.lerp(rounded, blend);
        mesh.positions[i] = p.to_array();
        let normal = (p - center) / (half * half);
        if normal.length_squared() > 1e-12 {
            mesh.normals[i] = Vec3::from_array(mesh.normals[i])
                .lerp(normal.normalize(), blend).normalize_or_zero().to_array();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn verify_capped_player(rig: &PersonRig) {
        let mut mesh = rig.mesh.clone();
        let mut ids = rig.ids.clone();
        let mut weights = rig.weights.clone();
        crate::head_surface::smooth(rig, &mut mesh, &mut ids, &mut weights);
        crate::head_surface::adventure_profile(rig, &mut mesh, &mut ids, &mut weights);
        crate::head_surface::player_profile(rig, &mut mesh, &ids, &weights);
        let before = mesh.clone();
        let old_ids = ids.clone();
        let old_weights = weights.clone();
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (i, &p) in before.positions.iter().enumerate() {
            let head: f32 = (0..4).filter(|&j| old_ids[i][j] as usize == rig.head_joint)
                .map(|j| old_weights[i][j]).sum();
            if head > 0.5 {
                min = min.min(Vec3::from_array(p));
                max = max.max(Vec3::from_array(p));
            }
        }
        let center = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        let (width, height) = rig.texture_size;
        capped_short(rig, &mut mesh, &mut ids, &mut weights);
        assert!(mesh.positions.len() > before.positions.len());
        assert!(mesh.positions.len() < 25_000, "capped hair geometry budget");
        assert_eq!(mesh.positions.len(), ids.len());
        assert_eq!(ids.len(), weights.len());
        let mut moved = 0;
        for i in 0..before.positions.len() {
            assert_eq!(mesh.uvs[i], before.uvs[i]);
            assert_eq!(mesh.colors[i], before.colors[i]);
            assert_eq!(ids[i], old_ids[i]);
            assert_eq!(weights[i], old_weights[i]);
            let head: f32 = (0..4).filter(|&j| old_ids[i][j] as usize == rig.head_joint)
                .map(|j| old_weights[i][j]).sum();
            let uv = before.uvs[i];
            let x = (uv[0] * width as f32).clamp(0., width as f32 - 1.) as usize;
            let y = (uv[1] * height as f32).clamp(0., height as f32 - 1.) as usize;
            let pixel = &rig.rgba[(y * width as usize + x) * 4..][..3];
            let skin = pixel[0] >= 125 || pixel[1] >= 120 || pixel[2] >= 115;
            // Protect body, neck, facial plane, cap support, ears and skin.
            if head < 0.85 || before.positions[i][2] >= center.z + half.z * 0.3
                || before.positions[i][1] >= max.y - half.y * 0.10 || skin {
                assert_eq!(mesh.positions[i], before.positions[i], "protected player surface moved");
                assert_eq!(mesh.normals[i], before.normals[i], "protected player shading changed");
            }
            if mesh.positions[i] != before.positions[i] { moved += 1; }
        }
        assert!(moved > 0, "hair candidate must change its support surface");
        for i in 0..mesh.positions.len() {
            assert!(Vec3::from_array(mesh.positions[i]).is_finite());
            assert!((Vec3::from_array(mesh.normals[i]).length()-1.).abs() < 0.001);
            assert!(weights[i].iter().all(|&w| w.is_finite() && w >= 0.));
            assert!((weights[i].iter().sum::<f32>()-1.).abs() < 0.001);
            assert!(ids[i].iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
        }
        assert!(mesh.indices.iter().all(|&i| (i as usize) < mesh.positions.len()));
    }
    #[test]
    #[ignore = "requires supplied player rig in CRYSTAL_OPEN_MODEL_ROOT"]
    fn capped_player_hair_preserves_mapping_face_body_and_skin() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for base in ["human-male", "human-female"] {
            let rig = PersonRig::parse(&std::fs::read(root.join(format!("people/kenney/{base}.glb")))
                .unwrap()).unwrap();
            verify_capped_player(&rig);
        }
    }
    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_crowns_preserve_face_skin_and_join_across_atlas_seams() {
        use crate::new_bark_actors::character_meshes::{CharacterKind, SOURCE_KINDS};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let mut styled = 0;
        let mut crowns = Vec::new();
        for kind in SOURCE_KINDS
            .iter()
            .map(|&(_, kind)| kind)
            .chain([CharacterKind::Outdoorsman])
        {
            let look = crate::npc_appearance::look(kind);
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{}.glb", look.base))).unwrap(),
            )
            .unwrap();
            let mut mesh = rig.mesh.clone();
            let mut ids = rig.ids.clone();
            let mut weights = rig.weights.clone();
            let extension = sculpt(&rig, &mut mesh, &mut ids, &mut weights, look);
            if matches!(look.texture_key, "chris" | "kris") {
                verify_capped_player(&rig);
                continue;
            }
            if look.hat != "none"
                || !(matches!(look.hairstyle, "spikes" | "flattop" | "sidepart" | "curls" | "bob" | "long") || matches!(look.texture_key, "whitney" | "misty" | "janine"))
            {
                assert_eq!(extension, 0.0);
                assert_eq!(mesh.positions, rig.mesh.positions);
                assert_eq!(mesh.normals, rig.mesh.normals);
                assert_eq!(mesh.indices, rig.mesh.indices);
                assert_eq!(mesh.uvs, rig.mesh.uvs);
                assert_eq!(ids, rig.ids);
                assert_eq!(weights, rig.weights);
                continue;
            }
            assert!(
                !crowns.contains(&mesh.positions),
                "{} duplicates another styled crown",
                look.texture_key
            );
            crowns.push(mesh.positions.clone());
            styled += 1;
            assert!(
                extension > 0.01 && extension < 0.5,
                "{} displacement budget",
                look.texture_key
            );
            assert!(
                mesh.positions.len() < 40_000,
                "{} geometry budget",
                look.texture_key
            );
            assert_eq!(mesh.positions.len(), ids.len());
            assert_eq!(ids.len(), weights.len());
            for i in 0..rig.mesh.positions.len() {
                assert_eq!(mesh.uvs[i], rig.mesh.uvs[i]);
                assert_eq!(mesh.colors[i], rig.mesh.colors[i]);
                assert_eq!(ids[i], rig.ids[i]);
                assert_eq!(weights[i], rig.weights[i]);
                let uv = rig.mesh.uvs[i];
                let (w, h) = rig.texture_size;
                let x = (uv[0] * w as f32).clamp(0.0, w as f32 - 1.0) as usize;
                let y = (uv[1] * h as f32).clamp(0.0, h as f32 - 1.0) as usize;
                let pixel = &rig.rgba[(y * w as usize + x) * 4..][..3];
                if (rig.mesh.positions[i][1] < 3.4
                    && !(look.hairstyle == "curls" && rig.mesh.positions[i][0].abs() > 0.30))
                    || pixel[0] >= 125
                    || pixel[1] >= 120
                    || pixel[2] >= 115
                {
                    assert_eq!(mesh.positions[i], rig.mesh.positions[i]);
                    assert_eq!(mesh.normals[i], rig.mesh.normals[i]);
                }
            }
            for (i, p) in mesh.positions.iter().enumerate() {
                assert!(Vec3::from_array(*p).is_finite());
                assert!((Vec3::from_array(mesh.normals[i]).length() - 1.0).abs() < 0.001);
                assert!(weights[i].iter().all(|v| v.is_finite() && *v >= 0.0));
                assert!((weights[i].iter().sum::<f32>() - 1.0).abs() < 0.001);
                assert!(ids[i].iter().all(|&j| (j as usize) < rig.skin_nodes.len()));
            }
            let key = |p: [f32; 3]| p.map(|v| (v * 10_000.0).round() as i32);
            let mut edges = HashMap::<([i32; 3], [i32; 3]), usize>::new();
            for face in mesh.indices.chunks_exact(3) {
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(mesh.positions[i as usize]));
                assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-16);
                let face_normal = (p[1] - p[0]).cross(p[2] - p[0]);
                let shading_normal: Vec3 = face
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
                    .sum();
                assert!(
                    face_normal.dot(shading_normal) >= -1e-6,
                    "{} crown face disagrees with its shading",
                    look.texture_key
                );
                for j in 0..3 {
                    let a = mesh.positions[face[j] as usize];
                    let b = mesh.positions[face[(j + 1) % 3] as usize];
                    if a[1] < 3.5 || b[1] < 3.5 {
                        continue;
                    }
                    let (a, b) = (key(a), key(b));
                    *edges
                        .entry(if a < b { (a, b) } else { (b, a) })
                        .or_default() += 1;
                }
            }
            fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                parents[i]
                    .map(|p| world(p, pose, parents))
                    .unwrap_or(Mat4::IDENTITY)
                    * pose[i].to_matrix()
            }
            for step in 0..8 {
                let mut pose = rig.nodes.clone();
                rig.sample("Run", rig.duration("Run") * step as f32 / 8.0, &mut pose);
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin: Vec<_> = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                    .collect();
                let lo = rig.bounds.0 - Vec3::splat(extension + 0.002);
                let hi = rig.bounds.1 + Vec3::splat(extension + 0.002);
                for ((p, joints), weights) in mesh.positions.iter().zip(&ids).zip(&weights) {
                    let p = Vec3::from_array(*p);
                    let animated: Vec3 = (0..4)
                        .map(|j| skin[joints[j] as usize].transform_point3(p) * weights[j])
                        .sum();
                    assert!(
                        animated.is_finite()
                            && animated.cmpge(lo).all()
                            && animated.cmple(hi).all(),
                        "{} animated crown outside culling envelope at phase {}",
                        look.texture_key,
                        step
                    );
                }
            }
            assert!(!edges.is_empty());
            assert!(
                edges.values().all(|&n| n == 2),
                "{} opened scalp seam",
                look.texture_key
            );
        }
        assert_eq!(styled, 29);
    }
}
