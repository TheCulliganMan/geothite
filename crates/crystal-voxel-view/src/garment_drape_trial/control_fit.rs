//! Fit control translations to the detailed surface without changing the
//! affine frames used for shading or adding joints to the GPU palette.
use bevy::prelude::*;
use std::collections::BTreeMap;

struct Sample {
    vertex: usize,
    joints: [usize; 4],
    weights: [f32; 4],
    area: f32,
}

pub(super) struct Fit {
    samples: Vec<Sample>,
    rows: Vec<Vec<(usize, f32)>>,
    diagonal: Vec<f32>,
}

impl Fit {
    pub(super) fn empty() -> Self {
        Self { samples: Vec::new(), rows: Vec::new(), diagonal: Vec::new() }
    }

    pub(super) fn new(
        points: &[[f32; 3]],
        indices: &[usize],
        cloth: &[bool],
        joints: &[[u16; 4]],
        weights: &[[f32; 4]],
        originals: usize,
        controls: usize,
    ) -> Self {
        let mut areas = vec![0.; points.len()];
        for face in indices.chunks_exact(3) {
            let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(points[i]));
            let area = (p[1] - p[0]).cross(p[2] - p[0]).length() / 6.;
            for &i in face { areas[i] += area; }
        }
        let samples: Vec<_> = (0..points.len())
            .filter(|&i| cloth[i] && areas[i] > 0.)
            .map(|i| Sample {
                vertex: i,
                joints: std::array::from_fn(|s| {
                    if weights[i][s] == 0. { 0 } else {
                        assert!(joints[i][s] as usize >= originals);
                        joints[i][s] as usize - originals
                    }
                }),
                weights: weights[i],
                area: areas[i],
            }).collect();
        let mut rows = vec![BTreeMap::<usize, f32>::new(); controls];
        for sample in &samples {
            for a in 0..4 {
                if sample.weights[a] == 0. { continue; }
                for b in 0..4 {
                    if sample.weights[b] == 0. { continue; }
                    *rows[sample.joints[a]].entry(sample.joints[b]).or_default() +=
                        sample.area * sample.weights[a] * sample.weights[b];
                }
            }
        }
        let mut diagonal = vec![0.; controls];
        for (i, row) in rows.iter_mut().enumerate() {
            let d = row.entry(i).or_default();
            // Tiny Tikhonov regularization chooses stable translations for
            // unused or nearly dependent controls, with a zero-motion prior.
            *d = *d * 1.0001 + 1e-12;
            diagonal[i] = *d;
        }
        Self { samples, rows: rows.into_iter().map(|r| r.into_iter().collect()).collect(), diagonal }
    }

    pub(super) fn correct(
        &self,
        points: &[[f32; 3]],
        controls: &mut [Mat4],
        mut target: impl FnMut(usize, Vec3) -> Vec3,
    ) {
        if self.samples.is_empty() { return; }
        assert_eq!(controls.len(), self.rows.len());
        let mut residual = vec![Vec3::ZERO; controls.len()];
        for sample in &self.samples {
            let p = Vec3::from_array(points[sample.vertex]);
            let current: Vec3 = (0..4).map(|s|
                controls[sample.joints[s]].transform_point3(p) * sample.weights[s]).sum();
            let error = target(sample.vertex, p) - current;
            for s in 0..4 {
                residual[sample.joints[s]] += error * (sample.area * sample.weights[s]);
            }
        }
        let mut translation = vec![Vec3::ZERO; controls.len()];
        let mut preconditioned: Vec<_> = residual.iter().zip(&self.diagonal)
            .map(|(r, d)| *r / *d).collect();
        let mut direction = preconditioned.clone();
        let dot = |a: &[Vec3], b: &[Vec3]| a.iter().zip(b).map(|(a,b)| a.dot(*b)).sum::<f32>();
        let mut energy = dot(&residual, &preconditioned);
        let initial = energy;
        for _ in 0..24 {
            if energy <= initial * 1e-10 || energy <= 1e-24 { break; }
            let product: Vec<Vec3> = self.rows.iter().map(|row|
                row.iter().map(|&(j,w)| direction[j] * w).sum()).collect();
            let denominator = dot(&direction, &product);
            assert!(denominator.is_finite() && denominator > 0.);
            let alpha = energy / denominator;
            for i in 0..controls.len() {
                translation[i] += direction[i] * alpha;
                residual[i] -= product[i] * alpha;
                preconditioned[i] = residual[i] / self.diagonal[i];
            }
            let next = dot(&residual, &preconditioned);
            let beta = next / energy;
            for i in 0..controls.len() {
                direction[i] = preconditioned[i] + direction[i] * beta;
            }
            energy = next;
        }
        for (matrix, delta) in controls.iter_mut().zip(translation) {
            assert!(delta.is_finite());
            matrix.w_axis += delta.extend(0.);
        }
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(super) fn validate() {
    let points = [[0.,0.,0.],[1.,0.,0.],[1.,1.,0.],[0.,1.,0.],[0.4,0.6,0.]];
    let joints = vec![[0,1,2,3]; 5];
    let weights = vec![[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.],[0.24,0.16,0.24,0.36]];
    let fit = Fit::new(&points,&[0,1,4,1,2,4,2,3,4,3,0,4],&[true;5],&joints,&weights,0,4);
    let mut controls = vec![Mat4::from_rotation_z(0.17);4];
    let rotation = controls[0];
    let offset = Vec3::new(0.03,-0.02,0.04);
    fit.correct(&points,&mut controls,|_,p| rotation.transform_point3(p)+offset);
    for i in 0..points.len() {
        let actual:Vec3=(0..4).map(|s|controls[joints[i][s]as usize]
            .transform_point3(Vec3::from_array(points[i]))*weights[i][s]).sum();
        assert!(actual.distance(rotation.transform_point3(Vec3::from_array(points[i]))+offset)<0.00002);
    }
    for matrix in controls {
        assert_eq!(matrix.x_axis,rotation.x_axis);
        assert_eq!(matrix.y_axis,rotation.y_axis);
        assert_eq!(matrix.z_axis,rotation.z_axis);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn translations_recover_a_rigid_target_without_changing_shading_frames() {super::validate();}
}
