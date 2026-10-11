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
        }
        if right - left > 0.1 && tip > 0.35 {
            result.push(Hand {
                bind,
                index,
                left,
                right,
                tip,
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
            let groove = [-0.5, 0., 0.5]
                .into_iter()
                .map(|g: f32| (-((s - g) / 0.075).powi(2)).exp())
                .fold(0.0_f32, f32::max);
            let t = ((q.y - 0.22) / (h.tip - 0.22)).clamp(0., 1.);
            let activation = t * t * (3. - 2. * t);
            q.y -= activation * 0.120 * groove;
            let middle_z = 0.020 + 0.14 * (q.y - 0.18);
            q.z = middle_z + (q.z - middle_z) * (1. - 0.40 * groove * activation);
            *p = h.bind.inverse().transform_point3(q).to_array();
            changed[i] = true;
            break;
        }
    }
    let mut normals = HashMap::<[i32; 3], Vec3>::new();
    for f in mesh.indices.chunks_exact(3) {
        let p = [f[0], f[1], f[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in f {
            *normals.entry(key(mesh.positions[i as usize])).or_default() += n;
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
            for hand in &h {
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
                for (i, gap) in [-0.5, 0., 0.5].into_iter().enumerate() {
                    let valley = tip(gap);
                    assert!(
                        valley.is_finite() && lobes[i].min(lobes[i + 1]) - valley > 0.035,
                        "{name} fingertip silhouette missing: {lobes:?} valley {gap}={valley}"
                    );
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
    }
}
