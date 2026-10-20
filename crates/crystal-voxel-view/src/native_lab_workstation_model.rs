//! The two authenticated lab drawings have three separate desktop units.
use super::*;

impl Model {
    pub(crate) fn append_native_lab_workstation(
        &self,
        out: &mut SurfaceMeshData,
        bounds: [f32; 4],
        height: f32,
        paint: [[f32; 2]; 8],
        desk: [[f32; 2]; 2],
    ) -> bool {
        if self.native_workstation_materials.is_none()
            || bounds[1] <= bounds[0]
            || bounds[3] <= bounds[2]
            || height <= 0.
        {
            return false;
        }
        let [w, e, n, s] = bounds;
        let point = |x: f32, y: f32, z: f32| {
            Vec3::new(
                w + (e - w) * x / 32.,
                height * y / 22.,
                n + (s - n) * z / 12.,
            )
        };
        let mut triangle = |a: Vec3, b: Vec3, c: Vec3, uv: [f32; 2]| {
            let normal = (b - a).cross(c - a).normalize();
            let first = out.positions.len() as u32;
            out.positions
                .extend([a.to_array(), b.to_array(), c.to_array()]);
            out.normals.extend([normal.to_array(); 3]);
            out.uvs.extend([uv; 3]);
            out.colors.extend([authored_face_color([1.; 4], normal); 3]);
            out.indices.extend([first, first + 1, first + 2]);
        };
        // Closed housings have small chamfers on their front silhouettes. The
        // source's upper equipment row describes height, not a modern widescreen.
        let mut body = |b: [f32; 6], bevel: f32, side: [f32; 2], front: [f32; 2]| {
            let [x0, x1, y0, y1, z0, z1] = b;
            let ring = [
                [x0 + bevel, y0],
                [x1 - bevel, y0],
                [x1, y0 + bevel],
                [x1, y1 - bevel],
                [x1 - bevel, y1],
                [x0 + bevel, y1],
                [x0, y1 - bevel],
                [x0, y0 + bevel],
            ];
            let center = |z| point((x0 + x1) * 0.5, (y0 + y1) * 0.5, z);
            for i in 0..8 {
                let j = (i + 1) % 8;
                let a = point(ring[i][0], ring[i][1], z0);
                let b = point(ring[j][0], ring[j][1], z0);
                let c = point(ring[j][0], ring[j][1], z1);
                let d = point(ring[i][0], ring[i][1], z1);
                triangle(center(z0), b, a, side);
                let worktop = side == desk[0] && front == desk[1];
                triangle(center(z1), d, c, if worktop { side } else { front });
                let edge = if worktop && i == 4 { desk[1] } else { side };
                triangle(a, b, c, edge);
                triangle(a, c, d, edge);
            }
        };
        // The low gold workbench supports all equipment; no tower under it.
        for x in [1., 28.] {
            for z in [1., 9.] {
                body([x, x + 2., 0., 7.2, z, z + 2.], 0.15, desk[0], desk[0]);
            }
        }
        body([0., 32., 7.2, 8.5, 0., 12.], 0.25, desk[0], desk[1]);
        body([2., 30., 3., 4., 9.8, 10.5], 0.1, desk[0], desk[0]);
        // Compact CRT at left, controller in the middle, tall right enclosure.
        body([2., 13., 9., 19., 1., 7.], 0.55, paint[2], paint[5]);
        body([3.4, 11.6, 10.5, 17.6, 7., 7.12], 0.25, paint[0], paint[0]);
        body([4.2, 10.8, 11.3, 16.8, 7.12, 7.2], 0.1, paint[0], paint[0]);
        body([4., 11., 8.5, 9.2, 2., 8.], 0.15, paint[2], paint[2]);
        body([14., 21.5, 8.5, 16.5, 1., 8.], 0.4, paint[2], paint[0]);
        body([21.5, 31., 8.5, 22., 0.5, 9.], 0.5, paint[2], paint[0]);
        body([23., 29.5, 10.5, 18., 9., 9.12], 0.25, paint[5], paint[3]);
        // The source's pale CRT strokes and the right unit's dark controls.
        body([5., 7., 15.3, 16.2, 7.2, 7.27], 0.05, paint[3], paint[3]);
        body([5., 9., 12.2, 13., 7.2, 7.27], 0.05, paint[3], paint[3]);
        body([24., 28., 15.7, 16.6, 9.12, 9.2], 0.05, paint[2], paint[2]);
        for x in [24., 27.] {
            body([x, x + 1., 12., 13., 9.12, 9.2], 0.05, paint[2], paint[2]);
        }
        body([16., 18., 11., 13.2, 8., 8.12], 0.1, paint[2], paint[2]);
        body([14.3, 20.8, 14.9, 15.5, 8., 8.12], 0.05, paint[5], paint[5]);
        // Low keyboard and front controls sit on the worktop, facing the aisle.
        body([2., 12.8, 8.5, 9.2, 8.5, 11.4], 0.25, paint[0], paint[0]);
        for x in [3., 5., 7., 9., 11.] {
            body([x, x + 1., 9.2, 9.4, 9., 10.5], 0.05, paint[3], paint[3]);
        }
        body([15., 20.5, 8.5, 9.3, 9., 11.4], 0.15, paint[2], paint[0]);
        for x in [16., 18.5] {
            body([x, x + 1., 9.3, 9.5, 9.5, 10.7], 0.05, paint[3], paint[3]);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_equipment_is_closed_outward_and_stays_supported_at_multiple_scales() {
        let model = model(ModelKind::LabWorkstation);
        let original = model.surface.positions.clone();
        let paint = std::array::from_fn(|i| [i as f32 / 10., 0.2]);
        let desk = [[0.8, 0.2], [0.9, 0.2]];
        for scale in [0.5, 1., 4.] {
            let mut out = SurfaceMeshData::default();
            assert!(model.append_native_lab_workstation(
                &mut out,
                [3., 3. + 32. * scale, 7., 7. + 12. * scale],
                22. * scale,
                paint,
                desk
            ));
            let mut volume = 0.;
            for tri in out.indices.chunks_exact(3) {
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(out.positions[i as usize]));
                let normal = Vec3::from_array(out.normals[tri[0] as usize]);
                assert!((b - a).cross(c - a).dot(normal) > 0.);
                assert!(normal.is_finite());
                volume += a.dot(b.cross(c)) / 6.;
                assert!(
                    tri.iter()
                        .all(|&i| out.uvs[i as usize] == out.uvs[tri[0] as usize])
                );
            }
            assert!(volume > 1000. * scale.powi(3));
            for p in &out.positions {
                assert!((3. - 0.001..=3. + 32. * scale + 0.001).contains(&p[0]));
                assert!((0.0..=22. * scale).contains(&p[1]));
                assert!((7. - 0.001..=7. + 12. * scale + 0.001).contains(&p[2]));
            }
            // Each closed chamfered body has eight sides and two triangulated
            // caps. Quantized edges must be shared twice, never an open panel.
            for component in out.indices.chunks(8 * 4 * 3) {
                let mut edges = std::collections::BTreeMap::new();
                let key = |i: u32| out.positions[i as usize].map(|x| (x * 10000.).round() as i32);
                for t in component.chunks_exact(3) {
                    for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                        let (a, b) = (key(a), key(b));
                        *edges
                            .entry(if a < b { (a, b) } else { (b, a) })
                            .or_insert(0) += 1;
                    }
                }
                assert!(edges.values().all(|&count| count == 2));
            }
        }
        assert_eq!(
            model.surface.positions, original,
            "shared generic/mansion asset is immutable"
        );
        let mut out = SurfaceMeshData::default();
        assert!(!model.append_native_lab_workstation(&mut out, [0., 0., 0., 1.], 22., paint, desk));
        assert!(out.positions.is_empty());
    }
}
