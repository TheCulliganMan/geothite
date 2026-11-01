//! Keep the authored living skeleton; trim whole leafless terminal limbs.
//! The source limbs intersect the trunk as separate modeled components, so
//! removing a dead-end component leaves the supporting trunk surface intact.
use super::Model;
use bevy::prelude::Vec3;
use std::collections::BTreeMap;

fn root(parents: &mut [usize], mut i: usize) -> usize {
    while parents[i] != i {
        parents[i] = parents[parents[i]];
        i = parents[i];
    }
    i
}
fn join(parents: &mut [usize], a: usize, b: usize) {
    let a = root(parents, a);
    let b = root(parents, b);
    parents[a] = b;
}

struct Limb {
    vertices: Vec<usize>,
    low: f32,
    high: f32,
}

pub(super) struct LivingGeometry {
    pub indices: Vec<u32>,
    pub shoot_sites: Vec<usize>,
}

pub(super) fn living_geometry(model: &Model) -> LivingGeometry {
    let mut parents = (0..model.foliage_start).collect::<Vec<_>>();
    let mut welded = BTreeMap::new();
    for (i, p) in model.positions[..model.foliage_start].iter().enumerate() {
        let key = p.map(|v| (v * 100_000.).round() as i32);
        if let Some(&other) = welded.get(&key) {
            join(&mut parents, i, other);
        } else {
            welded.insert(key, i);
        }
    }
    let triangles = model
        .indices
        .chunks_exact(3)
        .filter(|ids| ids.iter().all(|&i| (i as usize) < model.foliage_start))
        .collect::<Vec<_>>();
    for ids in &triangles {
        join(&mut parents, ids[0] as usize, ids[1] as usize);
        join(&mut parents, ids[0] as usize, ids[2] as usize);
    }
    let mut groups = BTreeMap::<usize, Vec<usize>>::new();
    for i in 0..model.foliage_start {
        groups.entry(root(&mut parents, i)).or_default().push(i);
    }
    let limbs = groups
        .into_values()
        .map(|vertices| {
            let low = vertices
                .iter()
                .map(|&i| model.positions[i][1])
                .fold(f32::INFINITY, f32::min);
            let high = vertices
                .iter()
                .map(|&i| model.positions[i][1])
                .fold(0., f32::max);
            Limb {
                vertices,
                low,
                high,
            }
        })
        .collect::<Vec<_>>();
    let mut owners = vec![0; model.foliage_start];
    for (limb, data) in limbs.iter().enumerate() {
        for &i in &data.vertices {
            owners[i] = limb;
        }
    }
    let foliage_floor = model.positions[model.foliage_start..]
        .iter()
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    let epsilon = model.source_height * 0.001;
    let mut living = limbs
        .iter()
        .map(|limb| limb.low <= epsilon || limb.high >= foliage_floor)
        .collect::<Vec<_>>();

    // Infer each limb's lower support from its attachment end. Following only
    // lower components produces an acyclic support chain. Retain the complete
    // chain for every limb reaching the foliage, even if its parent is bare.
    let supports = limbs
        .iter()
        .enumerate()
        .map(|(index, limb)| {
            if limb.low <= epsilon {
                return None;
            }
            let end = limb.low + (limb.high - limb.low) * 0.12;
            limbs
                .iter()
                .enumerate()
                .filter(|(other, candidate)| *other != index && candidate.low < limb.low - epsilon)
                .map(|(other, candidate)| {
                    let distance = limb
                        .vertices
                        .iter()
                        .filter(|&&i| model.positions[i][1] <= end)
                        .flat_map(|&i| {
                            candidate.vertices.iter().map(move |&j| {
                                Vec3::from_array(model.positions[i])
                                    .distance_squared(Vec3::from_array(model.positions[j]))
                            })
                        })
                        .fold(f32::INFINITY, f32::min);
                    (other, distance)
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(other, _)| other)
        })
        .collect::<Vec<_>>();
    for i in 0..limbs.len() {
        if !living[i] {
            continue;
        }
        let mut support = supports[i];
        while let Some(parent) = support {
            living[parent] = true;
            support = supports[parent];
        }
    }
    let indices = triangles
        .into_iter()
        .filter(|ids| living[owners[ids[0] as usize]])
        .flatten()
        .copied()
        .collect();
    let mut shoot_sites = Vec::new();
    for (limb_index, limb) in limbs.iter().enumerate() {
        if !living[limb_index] || limb.high < foliage_floor {
            continue;
        }
        let low = if limb.low <= epsilon {
            foliage_floor * 0.9
        } else {
            limb.low
        };
        let ceiling = limb
            .high
            .min(foliage_floor + (model.source_height - foliage_floor) * 0.45);
        if ceiling <= low {
            continue;
        }
        for station in 0..6 {
            let y = low + (ceiling - low) * (station as f32 + 0.5) / 6.;
            for turn in 0..3 {
                let angle = (turn as f32 / 3. + station as f32 * 0.381966) * std::f32::consts::TAU;
                let outward = Vec3::new(angle.cos(), 0., angle.sin());
                let vertex = limb
                    .vertices
                    .iter()
                    .copied()
                    .filter(|&i| model.normals[i][1].abs() < 0.8)
                    .min_by(|&a, &b| {
                        let score = |i: usize| {
                            (model.positions[i][1] - y).abs()
                                + model.source_height
                                    * 0.018
                                    * (1. - Vec3::from_array(model.normals[i]).dot(outward))
                        };
                        score(a).total_cmp(&score(b))
                    });
                let Some(vertex) = vertex else {
                    continue;
                };
                if !shoot_sites.contains(&vertex) {
                    shoot_sites.push(vertex);
                }
            }
        }
        if limb.low <= epsilon {
            let top = *limb
                .vertices
                .iter()
                .max_by(|&&a, &&b| model.positions[a][1].total_cmp(&model.positions[b][1]))
                .unwrap();
            if !shoot_sites.contains(&top) {
                shoot_sites.push(top);
            }
        }
    }
    LivingGeometry {
        indices,
        shoot_sites,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_support_is_retained_beneath_a_leafy_child_while_a_dead_side_limb_is_removed() {
        let model = Model {
            positions: vec![
                [0., 0., 0.],
                [0.1, 0.7, 0.],
                [0., 0.9, 0.],
                [0.06, 0.65, 0.01],
                [0.6, 0.9, 0.01],
                [0.58, 1.1, 0.01],
                [0.57, 1.02, 0.02],
                [0.7, 1.4, 0.02],
                [0.5, 1.5, 0.02],
                [-0.02, 0.5, 0.03],
                [-0.6, 0.7, 0.03],
                [-0.7, 0.6, 0.03],
                [0., 1.3, 0.],
            ],
            normals: vec![[0., 0., 1.]; 13],
            colors: Vec::new(),
            indices: (0..12).collect(),
            foliage_start: 12,
            living_wood: Vec::new(),
            source_height: 1.5,
            shoot_sites: Vec::new(),
        };
        assert_eq!(living_geometry(&model).indices, (0..9).collect::<Vec<_>>());
    }

    #[test]
    fn shipped_living_skeletons_keep_every_foliage_support_and_remove_only_complete_limbs() {
        let models = super::super::models();
        for model in models {
            let kept = model
                .living_wood
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            let floor = model.positions[model.foliage_start..]
                .iter()
                .map(|p| p[1])
                .fold(f32::INFINITY, f32::min);
            for ids in model
                .indices
                .chunks_exact(3)
                .filter(|ids| ids.iter().all(|&i| (i as usize) < model.foliage_start))
            {
                if ids.iter().any(|i| kept.contains(i)) {
                    assert!(
                        ids.iter().all(|i| kept.contains(i)),
                        "trimming must not cut through a modeled limb"
                    );
                }
                if ids.iter().any(|&i| {
                    model.positions[i as usize][1] >= floor
                        || model.positions[i as usize][1] < model.source_height * 0.001
                }) {
                    assert!(
                        ids.iter().all(|i| kept.contains(i)),
                        "foliage supports and planted roots must remain"
                    );
                }
            }
        }
        assert!(
            models[2].living_wood.len()
                < models[2]
                    .indices
                    .iter()
                    .filter(|&&i| (i as usize) < models[2].foliage_start)
                    .count()
                    * 3
                    / 4,
            "the upright model's leafless side forks must actually be trimmed"
        );
    }
}
