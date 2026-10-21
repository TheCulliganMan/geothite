//! Sparse interpolation of rigid arm blends through ordinary GPU skin joints.
use super::Dual;
use crate::{human_locomotion::world, open_people::PersonRig};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

type Key = [(u16, u8); 4];

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Recipe {
    originals: usize,
    steps: u8,
    sources: Vec<u16>,
    blends: Vec<Key>,
    centers: Vec<[f32; 3]>,
}

/// Surface-area weighting keeps texture seams and dense painted refinement
/// from pulling a blend's anchor toward an oversampled patch.
fn area_centers(
    mesh: &crate::mesh::SurfaceMeshData,
    joints: &[[u16; 4]],
    weights: &[[f32; 4]],
    originals: usize,
    count: usize,
) -> Vec<[f32; 3]> {
    let mut points = vec![Vec3::ZERO; count];
    let mut totals = vec![0_f32; count];
    for face in mesh.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let area = (p[1] - p[0]).cross(p[2] - p[0]).length() / 3.;
        let sum = p[0] + p[1] + p[2];
        for (corner, &index) in face.iter().enumerate() {
            let i = index as usize;
            for slot in 0..4 {
                let joint = usize::from(joints[i][slot]);
                if joint >= originals {
                    let h = joint - originals;
                    let weight = area * weights[i][slot];
                    // Integrate position times the linearly interpolated
                    // influence exactly over this triangle. Assigning all
                    // occupied area to its corner biases sparse lattice cells.
                    points[h] += (p[corner] + sum) * (0.25 * weight);
                    totals[h] += weight;
                }
            }
        }
    }
    points
        .into_iter()
        .zip(totals)
        .map(|(point, total)| {
            assert!(total > 0., "arm blend anchor needs an occupied surface");
            (point / total).to_array()
        })
        .collect()
}

/// Systematic rounding decomposes four continuous weights into at most four
/// neighboring integer lattice points. The interval lengths reconstruct the
/// original weights; canonical source-joint order keeps welded seams identical.
fn cells(joints: [u16; 4], weights: [f32; 4], steps: u8) -> Vec<(Key, f32)> {
    let mut merged = BTreeMap::<u16, f64>::new();
    for (j, w) in joints.into_iter().zip(weights) {
        if w > 0. {
            *merged.entry(j).or_default() += f64::from(w);
        }
    }
    let total: f64 = merged.values().sum();
    let mut source = [(0_u16, 0_f64); 4];
    for (target, (&j, &w)) in source.iter_mut().zip(&merged) {
        *target = (j, w / total * f64::from(steps));
    }
    let base = source.map(|(_, w)| w.floor() as u8);
    let mut prefix = [0_f64; 5];
    for i in 0..4 {
        prefix[i + 1] = prefix[i] + source[i].1 - f64::from(base[i]);
    }
    let mut cuts = vec![0., 1.];
    for &p in &prefix[1..4] {
        let value = (-p).rem_euclid(1.);
        if (1e-9..1. - 1e-9).contains(&value) {
            cuts.push(value);
        }
    }
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let mut result = Vec::with_capacity(4);
    for interval in cuts.windows(2) {
        let fraction = (interval[1] - interval[0]) as f32;
        if fraction < 1e-6 {
            continue;
        }
        let u = (interval[0] + interval[1]) * 0.5;
        let mut key = [(0, 0); 4];
        let mut used = 0;
        for i in 0..4 {
            let count = base[i] + ((prefix[i + 1] + u).floor() - (prefix[i] + u).floor()) as u8;
            if count > 0 {
                key[used] = (source[i].0, count);
                used += 1;
            }
        }
        assert_eq!(
            key.iter().map(|p| u16::from(p.1)).sum::<u16>(),
            u16::from(steps)
        );
        result.push((key, fraction));
    }
    let total: f32 = result.iter().map(|p| p.1).sum();
    for part in &mut result {
        part.1 /= total;
    }
    assert!(!result.is_empty() && result.len() <= 4);
    result
}

impl Recipe {
    pub(crate) fn prepare(
        rig: &PersonRig,
        mesh: &crate::mesh::SurfaceMeshData,
        joints: &mut [[u16; 4]],
        weights: &mut [[f32; 4]],
    ) -> Self {
        let arms = [
            crate::arm_skin::ancestry(rig, 0),
            crate::arm_skin::ancestry(rig, 1),
        ];
        let torso: Vec<_> = [
            crate::arm_skin::carriers(rig, 0),
            crate::arm_skin::carriers(rig, 1),
        ]
        .into_iter()
        .flatten()
        .collect();
        let eligible: Vec<_> = joints
            .iter()
            .zip(weights.iter())
            .map(|(j, w)| {
                (0..4)
                    .filter(|&s| arms.iter().any(|arm| arm[..2].contains(&(j[s] as usize))))
                    .map(|s| w[s])
                    .sum::<f32>()
                    > 0.001
                    && (0..4)
                        .filter(|&s| torso.contains(&(j[s] as usize)))
                        .map(|s| w[s])
                        .sum::<f32>()
                        > 0.999
            })
            .collect();
        let mut selected = None;
        for steps in (8..=12).rev() {
            let cell_map: Vec<_> = joints
                .iter()
                .zip(weights.iter())
                .zip(&eligible)
                .map(|((&j, &w), &yes)| yes.then(|| cells(j, w, steps)))
                .collect();
            let keys: BTreeSet<_> = cell_map
                .iter()
                .flatten()
                .flatten()
                .map(|p| p.0)
                .filter(|key| key.iter().filter(|p| p.1 > 0).count() > 1)
                .collect();
            if keys.len() + rig.skin_nodes.len() <= 256 {
                selected = Some((steps, cell_map, keys));
                break;
            }
        }
        let (steps, cell_map, keys) =
            selected.expect("arm blend lattice exceeds the GPU joint budget");
        let sources: Vec<_> = keys
            .iter()
            .flatten()
            .filter(|p| p.1 > 0)
            .map(|p| p.0)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let lookup: BTreeMap<_, _> = keys
            .iter()
            .enumerate()
            .map(|(i, &key)| (key, rig.skin_nodes.len() + i))
            .collect();
        let blends = keys
            .iter()
            .map(|key| {
                key.map(|(j, w)| {
                    if w == 0 {
                        (0, 0)
                    } else {
                        (sources.binary_search(&j).unwrap() as u16, w)
                    }
                })
            })
            .collect();
        for ((j, w), cell) in joints.iter_mut().zip(weights.iter_mut()).zip(cell_map) {
            let Some(cell) = cell else { continue };
            *j = [0; 4];
            *w = [0.; 4];
            for (slot, (key, fraction)) in cell.into_iter().enumerate() {
                let nonzero: Vec<_> = key.iter().filter(|p| p.1 > 0).collect();
                j[slot] = if nonzero.len() == 1 {
                    nonzero[0].0
                } else {
                    lookup[&key] as u16
                };
                w[slot] = fraction;
            }
        }
        let centers = area_centers(mesh, joints, weights, rig.skin_nodes.len(), keys.len());
        let result = Self {
            originals: rig.skin_nodes.len(),
            steps,
            sources,
            blends,
            centers,
        };
        result
            .validate()
            .expect("invalid generated arm blend recipe");
        result
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.originals == 0
            || self.originals + self.blends.len() > 256
            || !(8..=12).contains(&self.steps)
            || self.sources.is_empty()
            || self.centers.len() != self.blends.len()
            || self.centers.iter().flatten().any(|v| !v.is_finite())
            || !self.sources.windows(2).all(|p| p[0] < p[1])
            || self
                .sources
                .iter()
                .any(|&j| usize::from(j) >= self.originals)
        {
            return Err("invalid arm deformation sources or joint budget".into());
        }
        for blend in &self.blends {
            if blend.iter().map(|p| u16::from(p.1)).sum::<u16>() != u16::from(self.steps)
                || blend.iter().filter(|p| p.1 > 0).count() < 2
                || blend
                    .iter()
                    .any(|&(j, w)| w > 0 && usize::from(j) >= self.sources.len())
            {
                return Err("invalid arm deformation blend".into());
            }
        }
        Ok(())
    }
    pub(crate) fn helper_count(&self) -> usize {
        self.blends.len()
    }
    pub(crate) fn original_count(&self) -> usize {
        self.originals
    }
    pub(crate) fn resolution(&self) -> u8 {
        self.steps
    }
    pub(crate) fn validate_mesh(&self, mesh: &Mesh) -> Result<(), String> {
        use bevy::render::mesh::VertexAttributeValues;
        self.validate()?;
        let Some(VertexAttributeValues::Uint16x4(joints)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        else {
            return Err("arm deformation requires skin joints".into());
        };
        if joints
            .iter()
            .flatten()
            .any(|&j| usize::from(j) >= self.originals + self.helper_count())
        {
            return Err("arm deformation joint index leaves its palette".into());
        }
        Ok(())
    }

    fn update_with(&self, mut matrix: impl FnMut(usize) -> Mat4, output: &mut Vec<Transform>) {
        let matrices: Vec<_> = self.sources.iter().map(|&j| matrix(j as usize)).collect();
        let sources: Vec<_> = matrices
            .iter()
            .map(|&matrix| {
                Dual::from_matrix(matrix).expect("arm deformation carrier must stay rigid")
            })
            .collect();
        let strength = 1.;
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        let strength = std::env::var("CRYSTAL_PEOPLE_REVIEW_ARM_CENTER_STRENGTH")
            .map(|v| {
                v.parse::<f32>()
                    .expect("arm center strength must be a number")
            })
            .unwrap_or(strength);
        output.clear();
        output.extend(
            self.blends
                .iter()
                .zip(&self.centers)
                .map(|(blend, &center)| {
                    let parts = blend
                        .map(|(j, w)| (sources[j as usize], f32::from(w) / f32::from(self.steps)));
                    let mut pose = Dual::blend(&parts).transform();
                    let center = Vec3::from_array(center);
                    let target: Vec3 = blend
                        .iter()
                        .filter(|p| p.1 > 0)
                        .map(|&(j, w)| {
                            matrices[j as usize].transform_point3(center)
                                * (f32::from(w) / f32::from(self.steps))
                        })
                        .sum();
                    // Track the weighted joint path at the occupied center, retaining
                    // rigid rotation and local radius around it. This avoids curling
                    // the flank inward while the upper arm lowers.
                    pose.translation += (target - pose.transform_point(center)) * strength;
                    pose
                }),
        );
    }
    pub(crate) fn update(&self, rig: &PersonRig, pose: &[Transform], output: &mut Vec<Transform>) {
        assert_eq!(rig.skin_nodes.len(), self.originals);
        let inverse = world(rig, pose, rig.mesh_node).inverse();
        self.update_with(
            |j| inverse * world(rig, pose, rig.skin_nodes[j]) * rig.inverse_binds[j],
            output,
        );
    }
    pub(crate) fn extend_palette(&self, palette: &[Mat4]) -> Vec<Mat4> {
        assert_eq!(palette.len(), self.originals);
        let mut transforms = Vec::with_capacity(self.helper_count());
        self.update_with(|j| palette[j], &mut transforms);
        let mut result = palette.to_vec();
        result.extend(transforms.into_iter().map(|t| t.to_matrix()));
        result
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn validate() {
        lattice_reconstructs_weights_and_is_independent_of_slot_order();
        blend_center_follows_joint_path_without_shrinking_local_radius();
        surface_anchors_follow_area_instead_of_vertex_density();
        varying_influence_anchors_use_the_surface_integral();
    }

    #[cfg_attr(test, test)]
    fn lattice_reconstructs_weights_and_is_independent_of_slot_order() {
        for a in 0..=11 {
            for b in 0..=11 {
                for c in 0..=11 {
                    let weights = [a as f32 + 0.17, b as f32 + 0.31, c as f32 + 0.73, 1.41];
                    let total: f32 = weights.iter().sum();
                    let weights = weights.map(|w| w / total);
                    let source = [3, 7, 11, 19];
                    let mapped = cells(source, weights, 12);
                    let permuted = cells(
                        [19, 7, 3, 11],
                        [weights[3], weights[1], weights[0], weights[2]],
                        12,
                    );
                    assert_eq!(mapped, permuted);
                    let mut reconstructed = BTreeMap::<u16, f32>::new();
                    for (key, fraction) in mapped {
                        for (j, w) in key {
                            if w > 0 {
                                *reconstructed.entry(j).or_default() +=
                                    f32::from(w) / 12. * fraction;
                            }
                        }
                    }
                    for (j, w) in source.into_iter().zip(weights) {
                        assert!((reconstructed[&j] - w).abs() < 2e-6);
                    }
                }
            }
        }
        for weights in [
            [1., 0., 0., 0.],
            [0.5, 0.5, 0., 0.],
            [0.25; 4],
            [0.33333334, 0.6666666, 0., 0.],
        ] {
            assert!(cells([3, 7, 11, 19], weights, 12).len() <= 4);
        }
    }

    #[cfg_attr(test, test)]
    fn blend_center_follows_joint_path_without_shrinking_local_radius() {
        let center = Vec3::new(0.4, 0.2, 0.3);
        let recipe = Recipe {
            originals: 2,
            steps: 12,
            sources: vec![0, 1],
            blends: vec![[(0, 6), (1, 6), (0, 0), (0, 0)]],
            centers: vec![center.to_array()],
        };
        recipe.validate().unwrap();
        let a = Mat4::from_translation(Vec3::new(-0.1, 0.2, 0.));
        let b = Mat4::from_rotation_translation(
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            Vec3::new(0.3, -0.1, 0.2),
        );
        let mut output = Vec::new();
        recipe.update_with(|i| [a, b][i], &mut output);
        let pose = output[0];
        let expected = (a.transform_point3(center) + b.transform_point3(center)) * 0.5;
        assert!(pose.transform_point(center).distance(expected) < 2e-6);
        assert!((pose.transform_point(center + Vec3::X).distance(expected) - 1.).abs() < 2e-6);
        assert!((pose.rotation.length() - 1.).abs() < 1e-6);
        assert_eq!(pose.scale, Vec3::ONE);
        recipe.update_with(|_| b, &mut output);
        for point in [Vec3::ZERO, center, Vec3::new(1., -2., 0.5)] {
            assert!(
                output[0]
                    .transform_point(point)
                    .distance(b.transform_point3(point))
                    < 2e-6,
                "shared rigid motion acquired an anchor offset"
            );
        }
    }

    #[cfg_attr(test, test)]
    fn surface_anchors_follow_area_instead_of_vertex_density() {
        let mut mesh = crate::mesh::SurfaceMeshData {
            positions: vec![
                [0., 0., 0.],
                [2., 0., 0.],
                [0., 2., 0.],
                [4., 0., 0.],
                [5., 0., 0.],
                [4., 1., 0.],
            ],
            indices: vec![0, 1, 2, 3, 4, 5],
            ..Default::default()
        };
        let mut joints = vec![[2, 0, 0, 0]; 6];
        let mut weights = vec![[1., 0., 0., 0.]; 6];
        let a = Vec3::from_array(area_centers(&mesh, &joints, &weights, 2, 1)[0]);
        assert!(a.distance(Vec3::new(1.4, 0.6, 0.)) < 1e-6);
        mesh.positions
            .extend([[4.5, 0., 0.], [4.5, 0.5, 0.], [4., 0.5, 0.]]);
        joints.extend([[2, 0, 0, 0]; 3]);
        weights.extend([[1., 0., 0., 0.]; 3]);
        mesh.indices = vec![0, 1, 2, 3, 6, 8, 6, 4, 7, 8, 7, 5, 6, 7, 8];
        let b = Vec3::from_array(area_centers(&mesh, &joints, &weights, 2, 1)[0]);
        assert!(
            a.distance(b) < 1e-6,
            "subdividing one painting moved the arm anchor"
        );
    }

    #[cfg_attr(test, test)]
    fn varying_influence_anchors_use_the_surface_integral() {
        let mut mesh = crate::mesh::SurfaceMeshData {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            indices: vec![0, 1, 2],
            ..Default::default()
        };
        let mut ids = vec![[2, 0, 0, 0]; 3];
        let mut weights = vec![[0., 1., 0., 0.], [1., 0., 0., 0.], [0., 1., 0., 0.]];
        // Influence is x over the unit right triangle. Its occupied center is
        // (integral x*x / integral x, integral x*y / integral x) = (1/2,1/4).
        let expected = Vec3::new(0.5, 0.25, 0.);
        assert!(
            Vec3::from_array(area_centers(&mesh, &ids, &weights, 2, 1)[0]).distance(expected)
                < 1e-6
        );
        mesh.positions
            .extend([[0.5, 0., 0.], [0.5, 0.5, 0.], [0., 0.5, 0.]]);
        ids.extend([[2, 0, 0, 0]; 3]);
        weights.extend([[0.5, 0.5, 0., 0.], [0.5, 0.5, 0., 0.], [0., 1., 0., 0.]]);
        mesh.indices = vec![0, 3, 5, 3, 1, 4, 5, 4, 2, 3, 4, 5];
        assert!(
            Vec3::from_array(area_centers(&mesh, &ids, &weights, 2, 1)[0]).distance(expected)
                < 1e-6,
            "subdividing a varying influence moved its occupied center"
        );
    }
}
