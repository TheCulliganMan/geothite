//! Sculpt four rounded fingertip lobes into the supplied connected finger slab.
//! The original wrist, palm base, thumb, UV islands and joint palette stay intact.
use crate::{mesh::SurfaceMeshData, open_people::PersonRig};
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};

fn key(p: [f32; 3]) -> [i32; 3] {
    p.map(|v| (v * 100_000.).round() as i32)
}
fn blend(
    mesh: &mut SurfaceMeshData,
    j: &mut Vec<[u16; 4]>,
    w: &mut Vec<[f32; 4]>,
    a: usize,
    b: usize,
) -> u32 {
    let i = mesh.positions.len() as u32;
    mesh.positions.push(
        Vec3::from_array(mesh.positions[a])
            .lerp(Vec3::from_array(mesh.positions[b]), 0.5)
            .to_array(),
    );
    mesh.normals.push(
        (Vec3::from_array(mesh.normals[a]) + Vec3::from_array(mesh.normals[b]))
            .normalize_or_zero()
            .to_array(),
    );
    mesh.uvs.push(
        Vec2::from_array(mesh.uvs[a])
            .lerp(Vec2::from_array(mesh.uvs[b]), 0.5)
            .to_array(),
    );
    mesh.colors.push(
        Vec4::from_array(mesh.colors[a])
            .lerp(Vec4::from_array(mesh.colors[b]), 0.5)
            .to_array(),
    );
    let mut skin = BTreeMap::<u16, f32>::new();
    for i in [a, b] {
        for n in 0..4 {
            *skin.entry(j[i][n]).or_default() += w[i][n] * 0.5;
        }
    }
    let mut skin = skin.into_iter().collect::<Vec<_>>();
    skin.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    skin.truncate(4);
    let sum = skin.iter().map(|p| p.1).sum::<f32>();
    j.push(std::array::from_fn(|i| skin.get(i).map_or(0, |v| v.0)));
    w.push(std::array::from_fn(|i| {
        skin.get(i).map_or(0., |v| v.1 / sum)
    }));
    i
}

/// Shared geometric edge marking also splits the neighboring face, so adding
/// finger detail does not introduce new T-junctions at the palm boundary.
fn refine(
    mesh: &mut SurfaceMeshData,
    j: &mut Vec<[u16; 4]>,
    w: &mut Vec<[f32; 4]>,
    selected: &[bool],
) {
    let old = std::mem::take(&mut mesh.indices);
    let edge = |a: [i32; 3], b: [i32; 3]| (a.min(b), a.max(b));
    let mut cuts = BTreeSet::new();
    for f in old.chunks_exact(3) {
        if !f.iter().all(|&i| selected[i as usize]) {
            continue;
        }
        for n in 0..3 {
            cuts.insert(edge(
                key(mesh.positions[f[n] as usize]),
                key(mesh.positions[f[(n + 1) % 3] as usize]),
            ));
        }
    }
    let mut cache = HashMap::new();
    for f in old.chunks_exact(3) {
        let v = [f[0], f[1], f[2]];
        let mut mid = [None; 3];
        for n in 0..3 {
            let a = v[n];
            let b = v[(n + 1) % 3];
            if cuts.contains(&edge(
                key(mesh.positions[a as usize]),
                key(mesh.positions[b as usize]),
            )) {
                mid[n] = Some(
                    *cache
                        .entry((a.min(b), a.max(b)))
                        .or_insert_with(|| blend(mesh, j, w, a as usize, b as usize)),
                );
            }
        }
        match mid.iter().filter(|p| p.is_some()).count() {
            0 => mesh.indices.extend(v),
            3 => {
                let [a, b, c] = mid.map(Option::unwrap);
                mesh.indices
                    .extend([v[0], a, c, a, v[1], b, c, b, v[2], a, b, c]);
            }
            1 => {
                let n = mid.iter().position(|p| p.is_some()).unwrap();
                let a = v[n];
                let b = v[(n + 1) % 3];
                let c = v[(n + 2) % 3];
                let m = mid[n].unwrap();
                mesh.indices.extend([a, m, c, m, b, c]);
            }
            2 => {
                let n = mid.iter().position(|p| p.is_none()).unwrap();
                let a = v[n];
                let b = v[(n + 1) % 3];
                let c = v[(n + 2) % 3];
                let bc = mid[(n + 1) % 3].unwrap();
                let ca = mid[(n + 2) % 3].unwrap();
                mesh.indices.extend([a, b, ca, b, bc, ca, bc, c, ca]);
            }
            _ => unreachable!(),
        }
    }
}

struct Hand {
    bind: Mat4,
    index: BTreeSet<usize>,
    left: f32,
    right: f32,
    tip: f32,
    cap_center_z: f32,
    cap_depth: f32,
}
fn hands(rig: &PersonRig) -> Vec<Hand> {
    let mut result = Vec::new();
    for group in &rig.hand_joints {
        let Some(&root) = group.iter().find(|&&j| {
            rig.parents[rig.skin_nodes[j]]
                .is_none_or(|p| !group.iter().any(|&k| rig.skin_nodes[k] == p))
        }) else {
            continue;
        };
        let bind = rig.inverse_binds[root];
        let root_node = rig.skin_nodes[root];
        let index = group
            .iter()
            .copied()
            .filter(|&j| rig.parents[rig.skin_nodes[j]] == Some(root_node))
            .max_by(|&a, &b| {
                let point = |j: usize| {
                    bind.transform_point3(
                        rig.inverse_binds[j].inverse().transform_point3(Vec3::ZERO),
                    )
                    .y
                };
                point(a).total_cmp(&point(b))
            });
        let Some(index_root) = index else {
            continue;
        };
        let index = group
            .iter()
            .copied()
            .filter(|&j| {
                let mut n = Some(rig.skin_nodes[j]);
                while let Some(i) = n {
                    if i == rig.skin_nodes[index_root] {
                        return true;
                    }
                    n = rig.parents[i];
                }
                false
            })
            .collect::<BTreeSet<_>>();
        let mut left = f32::INFINITY;
        let mut right = f32::NEG_INFINITY;
        let mut tip = 0.0_f32;
        let mut index_points = Vec::new();
        for ((p, j), w) in rig.mesh.positions.iter().zip(&rig.ids).zip(&rig.weights) {
            let amount = (0..4)
                .filter(|&i| index.contains(&(j[i] as usize)))
                .map(|i| w[i])
                .sum::<f32>();
            if amount < 0.9 {
                continue;
            }
            let p = bind.transform_point3(Vec3::from_array(*p));
            left = left.min(p.x);
            right = right.max(p.x);
            tip = tip.max(p.y);
            index_points.push(p);
        }
        if right - left > 0.1 && tip > 0.35 {
            // The terminal surface narrows relative to the proximal slab.
            // Spacing fingers over the entire slab put the outer designed
            // center beyond the supplied cap, leaving an asymmetric blade.
            let cap_left = index_points
                .iter()
                .filter(|p| p.y > tip - 0.08)
                .map(|p| p.x)
                .fold(f32::INFINITY, f32::min);
            let cap_right = index_points
                .iter()
                .filter(|p| p.y > tip - 0.08)
                .map(|p| p.x)
                .fold(f32::NEG_INFINITY, f32::max);
            let cap_min = index_points
                .iter()
                .filter(|p| p.y > tip - 0.08)
                .map(|p| p.z)
                .fold(f32::INFINITY, f32::min);
            let cap_max = index_points
                .iter()
                .filter(|p| p.y > tip - 0.08)
                .map(|p| p.z)
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(cap_max - cap_min > 0.01, "supplied finger cap has no depth");
            assert!(
                cap_right - cap_left > 0.1,
                "supplied finger cap has no width"
            );
            result.push(Hand {
                bind,
                index,
                left: cap_left,
                right: cap_right,
                tip,
                cap_center_z: (cap_min + cap_max) * 0.5,
                cap_depth: (cap_max - cap_min) * 0.5,
            });
        }
    }
    result
}
fn eligible(p: [f32; 3], j: [u16; 4], w: [f32; 4], hand: &Hand, cut: f32) -> bool {
    (0..4)
        .filter(|&i| hand.index.contains(&(j[i] as usize)))
        .map(|i| w[i])
        .sum::<f32>()
        > 0.6
        && hand.bind.transform_point3(Vec3::from_array(p)).y > cut
}

pub(crate) fn sculpt(
    rig: &PersonRig,
    mesh: &mut SurfaceMeshData,
    j: &mut Vec<[u16; 4]>,
    w: &mut Vec<[f32; 4]>,
) {
    let hands = hands(rig);
    if hands.is_empty() {
        return;
    }
    for cut in [0.19, 0.19, 0.35] {
        let selected = mesh
            .positions
            .iter()
            .zip(j.iter())
            .zip(w.iter())
            .map(|((&p, &j), &w)| hands.iter().any(|h| eligible(p, j, w, h, cut)))
            .collect::<Vec<_>>();
        refine(mesh, j, w, &selected);
    }
    let mut changed = vec![false; mesh.positions.len()];
    for (i, p) in mesh.positions.iter_mut().enumerate() {
        for h in &hands {
            if !eligible(*p, j[i], w[i], h, 0.22) {
                continue;
            }
            let mut q = h.bind.transform_point3(Vec3::from_array(*p));
            let half = (h.right - h.left) * 0.5;
            let center = (h.right + h.left) * 0.5;
            let s = ((q.x - center) / half).clamp(-1., 1.);
            // Give each finger a broad circular terminal pad. The previous
            // cosine cut took too much height from the shoulders, producing
            // four pointed claws in the actual mounted and resting renders.
            let cell = ((s + 1.) * 2.).floor().clamp(0., 3.);
            let lateral = ((s - (-0.75 + 0.5 * cell)) / 0.25).abs();
            let radius = half * 0.25 * 0.82;
            // Smooth shoulders and a wider web avoid the circular profile's
            // vertical tangent, which creased the triangulated pad edges.
            let shoulder = lateral * lateral * (3. - 2. * lateral);
            let cap_drop = radius * shoulder * shoulder;
            let web = ((lateral - 0.50) / 0.50).clamp(0., 1.);
            let web = web * web * (3. - 2. * web);
            let lateral_drop = cap_drop + (0.105 - radius) * web;
            let t = ((q.y - 0.22) / (h.tip - 0.22)).clamp(0., 1.);
            let activation = t * t * (3. - 2. * t);
            // Round the terminal pad through its depth too. A lateral arch
            // alone leaves four flat blades when the relaxed hand curls.
            // This bounded height field retains the connected surface and
            // uses the supplied slab's actual depth, without adding pieces.
            let depth = ((q.z - h.cap_center_z) / h.cap_depth).clamp(-1., 1.);
            // The supplied cap already has a depth dome. A second deep
            // carve sharpened its tip; retain that dome with a small blend.
            q.y -= activation * (lateral_drop + 0.008 * depth * depth);
            // A rounded section across each pad replaces the shared flat
            // front/back plates. Retain a thin connected web in the valleys
            // and fade into the original palm rather than adding loose parts.
            let section = (lateral * std::f32::consts::FRAC_PI_2).cos().max(0.);
            // Apply depth shaping after tip shaping. Reusing the original
            // activation couples both contractions and can invert the local
            // surface near a valley; the sequential maps stay monotone.
            // Begin the depth blend beyond the palm crease. Compressing the
            // proximal surface made a hard ridge across the resting palm.
            let volume_t = ((q.y - 0.28) / (h.tip - 0.28)).clamp(0., 1.);
            let volume_activation = volume_t * volume_t * (3. - 2. * volume_t);
            let middle_z = h.cap_center_z;
            q.z = middle_z + (q.z - middle_z) * (1. - 0.75 * (1. - section) * volume_activation);
            *p = h.bind.inverse().transform_point3(q).to_array();
            changed[i] = true;
            break;
        }
    }
    let mut normals = HashMap::<[i32; 3], Vec3>::new();
    for f in mesh.indices.chunks_exact(3) {
        let p = [f[0], f[1], f[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        // Subdivision leaves unequal triangle sizes near a web. Weight by
        // each corner's angle so a large neighboring face cannot dominate
        // the normal of a small curved pad face across an atlas seam.
        for corner in 0..3 {
            let a = p[(corner + 1) % 3] - p[corner];
            let b = p[(corner + 2) % 3] - p[corner];
            let angle = a
                .normalize_or_zero()
                .dot(b.normalize_or_zero())
                .clamp(-1., 1.)
                .acos();
            let i = f[corner] as usize;
            *normals.entry(key(mesh.positions[i])).or_default() += n * angle;
        }
    }
    for i in 0..mesh.positions.len() {
        if changed[i] {
            if let Some(n) = normals.get(&key(mesh.positions[i])) {
                mesh.normals[i] = n.normalize_or_zero().to_array();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires supplied Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_digits_preserve_wrists_attributes_and_animation_bounds() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let mut shape_failures = Vec::new();
        for name in ["human-male", "human-female", "skater-male", "skater-female"] {
            let rig = PersonRig::parse(
                &std::fs::read(root.join(format!("people/kenney/{name}.glb"))).unwrap(),
            )
            .unwrap();
            let h = hands(&rig);
            assert_eq!(h.len(), 2);
            let mut mesh = rig.mesh.clone();
            let mut j = rig.ids.clone();
            let mut w = rig.weights.clone();
            crate::hand_surface::smooth(&rig, &mut mesh, &mut j, &mut w);
            let old = mesh.clone();
            let oldj = j.clone();
            let oldw = w.clone();
            let count = mesh.positions.len();
            sculpt(&rig, &mut mesh, &mut j, &mut w);
            assert!(mesh.positions.len() > count + 500);
            assert_eq!(&mesh.uvs[..count], &old.uvs);
            assert_eq!(&mesh.colors[..count], &old.colors);
            assert_eq!(&j[..count], &oldj);
            assert_eq!(&w[..count], &oldw);
            for i in 0..count {
                if !h
                    .iter()
                    .any(|h| eligible(old.positions[i], oldj[i], oldw[i], h, 0.22))
                {
                    assert_eq!(mesh.positions[i], old.positions[i]);
                    assert_eq!(mesh.normals[i], old.normals[i]);
                }
            }
            for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
                assert!(Vec3::from_array(*p).is_finite());
                assert!((Vec3::from_array(*n).length() - 1.).abs() < 1e-4);
            }
            for weight in &w {
                assert!(weight.iter().all(|&v| v >= 0. && v.is_finite()));
                assert!((weight.iter().sum::<f32>() - 1.).abs() < 1e-5);
            }
            for (side, hand) in h.iter().enumerate() {
                let tip = |s: f32| {
                    mesh.positions
                        .iter()
                        .zip(&j)
                        .zip(&w)
                        .filter_map(|((&p, &j), &w)| {
                            if !eligible(p, j, w, hand, 0.23) {
                                return None;
                            }
                            let q = hand.bind.transform_point3(Vec3::from_array(p));
                            let lateral = (q.x - (hand.left + hand.right) * 0.5)
                                / ((hand.right - hand.left) * 0.5);
                            ((lateral - s).abs() < 0.03).then_some(q.y)
                        })
                        .fold(f32::NEG_INFINITY, f32::max)
                };
                let lobes = [-0.75, -0.25, 0.25, 0.75].map(tip);
                for (center, apex) in [-0.75, -0.25, 0.25, 0.75].into_iter().zip(lobes) {
                    let shoulders = [tip(center - 0.12), tip(center + 0.12)];
                    if !(shoulders.iter().all(|p| p.is_finite())
                        && apex - (shoulders[0] + shoulders[1]) * 0.5 > 0.015)
                    {
                        shape_failures.push(format!(
                            "{name} side{side} fingertip remains flat: center {center}, apex {apex}, shoulders {shoulders:?}"
                        ));
                    }
                    let mut pad_center = f32::NEG_INFINITY;
                    let mut pad_sides = f32::NEG_INFINITY;
                    for ((&p, &j), &w) in mesh.positions.iter().zip(&j).zip(&w) {
                        if !eligible(p, j, w, hand, 0.23) {
                            continue;
                        }
                        let q = hand.bind.transform_point3(Vec3::from_array(p));
                        let lateral = (q.x - (hand.left + hand.right) * 0.5)
                            / ((hand.right - hand.left) * 0.5);
                        if (lateral - center).abs() > 0.08 {
                            continue;
                        }
                        let depth = ((q.z - hand.cap_center_z) / hand.cap_depth).abs();
                        if depth < 0.25 {
                            pad_center = pad_center.max(q.y);
                        }
                        if depth > 0.75 {
                            pad_sides = pad_sides.max(q.y);
                        }
                    }
                    if !(pad_center.is_finite()
                        && pad_sides.is_finite()
                        && pad_center - pad_sides > 0.008)
                    {
                        shape_failures.push(format!(
                            "{name} side{side} fingertip pad stays flat through depth: center {center}, peak {pad_center}, sides {pad_sides}"
                        ));
                    }
                }
                for (i, gap) in [-0.5, 0., 0.5].into_iter().enumerate() {
                    let valley = tip(gap);
                    if !(valley.is_finite() && lobes[i].min(lobes[i + 1]) - valley > 0.035) {
                        shape_failures.push(format!(
                            "{name} side{side} fingertip silhouette missing: {lobes:?} valley {gap}={valley}"
                        ));
                    }
                }
                let mut edges = BTreeMap::<([i32; 3], [i32; 3]), (usize, i32)>::new();
                for f in mesh.indices.chunks_exact(3) {
                    for n in 0..3 {
                        let a = mesh.positions[f[n] as usize];
                        let b = mesh.positions[f[(n + 1) % 3] as usize];
                        if hand.bind.transform_point3(Vec3::from_array(a)).y < 0.24
                            || hand.bind.transform_point3(Vec3::from_array(b)).y < 0.24
                            || !eligible(a, j[f[n] as usize], w[f[n] as usize], hand, 0.23)
                            || !eligible(
                                b,
                                j[f[(n + 1) % 3] as usize],
                                w[f[(n + 1) % 3] as usize],
                                hand,
                                0.23,
                            )
                        {
                            continue;
                        }
                        let (a, b) = (key(a), key(b));
                        let e = edges.entry((a.min(b), a.max(b))).or_default();
                        e.0 += 1;
                        e.1 += if a < b { 1 } else { -1 };
                    }
                }
                assert!(
                    edges.values().all(|&v| v == (2, 0)),
                    "{name} opened finger surface"
                );
            }
            fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
                parents[i]
                    .map(|p| world(p, pose, parents))
                    .unwrap_or(Mat4::IDENTITY)
                    * pose[i].to_matrix()
            }
            for clip in ["Idle", "Run", "Jump"] {
                for step in 0..=60 {
                    let mut pose = rig.nodes.clone();
                    rig.sample(clip, rig.duration(clip) * step as f32 / 60., &mut pose);
                    if clip != "Jump" {
                        crate::hand_pose::relax(&rig, &mut pose);
                    }
                    let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                    let palette = rig
                        .skin_nodes
                        .iter()
                        .zip(&rig.inverse_binds)
                        .map(|(&n, b)| inverse * world(n, &pose, &rig.parents) * *b)
                        .collect::<Vec<_>>();
                    for ((p, j), w) in mesh.positions.iter().zip(&j).zip(&w) {
                        let p = (0..4)
                            .map(|i| {
                                palette[j[i] as usize].transform_point3(Vec3::from_array(*p)) * w[i]
                            })
                            .sum::<Vec3>();
                        assert!(
                            p.cmpge(rig.bounds.0 - Vec3::splat(0.02)).all()
                                && p.cmple(rig.bounds.1 + Vec3::splat(0.02)).all(),
                            "{name} {clip} {step} escapes bounds: {p:?}"
                        );
                    }
                }
            }
        }
        // Collect shape defects across the entire supplied rig set, rather
        // than stopping on one fingertip and leaving the other hands unaudited.
        assert!(shape_failures.is_empty(), "{}", shape_failures.join("\n"));
    }
}
