//! Source-shaped rounded enclosure beside the original monitor and keyboard.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Part {
    Keep,
    Right,
    Casing,
    Front,
    Status,
}

const PARTS: [(&str, u8, Part); 24] = [
    ("Laboratory bench folded leg", 2, Part::Keep),
    ("Laboratory bench dark foot", 2, Part::Keep),
    (
        "Laboratory bench recessed back modesty panel",
        1,
        Part::Keep,
    ),
    ("Laboratory bench lower rear cross rail", 1, Part::Keep),
    ("Laboratory bench overhung paper laminate", 1, Part::Keep),
    ("Laboratory bench folded front edge", 1, Part::Keep),
    ("Continuous bench apron", 1, Part::Keep),
    ("Inset bench drawer", 2, Part::Keep),
    ("Slim drawer pull", 2, Part::Keep),
    ("Left display pedestal", 1, Part::Keep),
    ("Left terminal rear casing", 1, Part::Keep),
    ("Left terminal deep dark bezel", 1, Part::Keep),
    ("Left terminal live display recess", 1, Part::Keep),
    ("Left terminal narrow pale lip", 1, Part::Keep),
    ("Keyboard broad shallow case", 1, Part::Keep),
    ("Keyboard raised cap", 21, Part::Keep),
    ("Keyboard spacebar", 1, Part::Keep),
    ("Right instrument raised foot", 1, Part::Right),
    ("Right instrument faceted upright", 1, Part::Casing),
    ("Right instrument dark front inset", 1, Part::Front),
    ("Right instrument upper face", 1, Part::Front),
    ("Right instrument lower bay", 1, Part::Front),
    ("Right instrument pale side facet", 1, Part::Right),
    ("Right instrument narrow status marker", 1, Part::Status),
];
pub(super) fn classify(name: &str) -> Option<(usize, Part)> {
    PARTS
        .iter()
        .enumerate()
        .find(|(_, p)| p.0 == name)
        .map(|(i, p)| (i, p.2))
}
pub(super) fn expected_counts() -> [u8; 24] {
    PARTS.map(|p| p.1)
}

impl Model {
    pub(crate) fn has_native_facility_workbench(&self) -> bool {
        self.native_workbench_parts.is_some() && self.native_facility_furniture_materials.is_some()
    }
    pub(crate) fn append_native_facility_workbench(
        &self,
        out: &mut SurfaceMeshData,
        bounds: [f32; 4],
        height: f32,
        paint: [[f32; 2]; 4],
    ) -> bool {
        let (Some(parts), Some(roles)) = (
            &self.native_workbench_parts,
            &self.native_facility_furniture_materials,
        ) else {
            return false;
        };
        let [w, e, n, s] = bounds;
        if e <= w || s <= n || height <= 0. {
            return false;
        }
        let scale = Vec3::new((e - w) / 32., height / 18., (s - n) / 18.);
        let origin = Vec3::new(w, 0., n);
        let mut fitted = SurfaceMeshData::default();
        self.append_fitted(&mut fitted, bounds, 0., height);
        let angle = -std::f32::consts::PI / 3.;
        let (sin, cos) = angle.sin_cos();
        let rotate = |p: Vec3| Vec3::new(cos * p.x + sin * p.z, p.y, -sin * p.x + cos * p.z);
        let right = |p: Vec3, normal: Vec3| {
            let size = Vec3::new(0.8, 9. / 8.3, 0.8);
            let pivot = Vec3::new(25.25, 9., 8.25);
            (
                (rotate((p - pivot) * size) + pivot) * scale + origin,
                (rotate(normal / size) / scale).normalize(),
            )
        };
        for tri in self.surface.indices.chunks_exact(3) {
            let part = parts[tri[0] as usize];
            if part == Part::Front {
                continue;
            }
            let first = out.positions.len() as u32;
            for &index in tri {
                let i = index as usize;
                let source_normal = Vec3::from_array(self.surface.normals[i]);
                let (position, normal) = if part == Part::Keep {
                    (
                        Vec3::from_array(fitted.positions[i]),
                        Vec3::from_array(fitted.normals[i]),
                    )
                } else {
                    let mut p = Vec3::from_array(self.surface.positions[i]);
                    if part == Part::Status {
                        p.x -= 3.25;
                        p.y += 1.3;
                    }
                    right(p, source_normal)
                };
                out.positions.push(position.to_array());
                out.normals.push(normal.to_array());
                let role = if part == Part::Casing && source_normal.y > 0.5 {
                    2
                } else {
                    roles[i] as usize
                };
                out.uvs.push(paint[role]);
                out.colors.push(authored_face_color([1.; 4], normal));
            }
            out.indices.extend([first, first + 1, first + 2]);
        }
        // Replace the old three rectangular inserts with one finished rounded
        // dark face. Transform the complete closed face and its indicator along
        // with the case; no part floats independently of its supporting foot.
        let mut triangle = |points: [Vec3; 3], role: usize| {
            let normal = (points[1] - points[0])
                .cross(points[2] - points[0])
                .normalize();
            let first = out.positions.len() as u32;
            for point in points {
                let (p, n) = right(point, normal);
                out.positions.push(p.to_array());
                out.normals.push(n.to_array());
                out.uvs.push(paint[role]);
                out.colors.push(authored_face_color([1.; 4], n));
            }
            out.indices.extend([first, first + 1, first + 2]);
        };
        let center = |z| Vec3::new(24.7, 13.4, z);
        let ring = |i: usize, z| {
            let angle = i as f32 * std::f32::consts::TAU / 16.;
            Vec3::new(24.7 + 3.3 * angle.cos(), 13.4 + 3.4 * angle.sin(), z)
        };
        for i in 0..16 {
            let j = (i + 1) % 16;
            let (a, b, c, d) = (ring(i, 12.7), ring(j, 12.7), ring(j, 13.15), ring(i, 13.15));
            triangle([center(13.15), d, c], 3);
            triangle([center(12.7), b, a], 1);
            triangle([a, b, c], 1);
            triangle([a, c, d], 1);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selected() -> Model {
        Model::parse(crate::model_storage::include_model!(
            "models/facility_radio/facility_workbench.mesh.json"
        ))
        .unwrap()
    }
    #[test]
    fn facility_enclosure_keeps_monitor_bench_and_keys_exact_and_faces_operator() {
        let m = selected();
        assert!(m.has_native_facility_workbench());
        let original = m.surface.clone();
        let parts = m.native_workbench_parts.as_ref().unwrap();
        let paint = [[0.1, 0.2], [0.3, 0.2], [0.5, 0.2], [0.7, 0.2]];
        for (sx, sy) in [(1., 1.), (0.5, 2.), (2., 0.5)] {
            let bounds = [3., 3. + 32. * sx, 7., 7. + 18. * sy];
            let mut old = SurfaceMeshData::default();
            m.append_fitted(&mut old, bounds, 0., 18. * sy);
            let expected = m
                .surface
                .indices
                .chunks_exact(3)
                .filter(|t| parts[t[0] as usize] == Part::Keep)
                .flat_map(|t| {
                    t.iter()
                        .map(|&i| (old.positions[i as usize], old.normals[i as usize]))
                })
                .collect::<Vec<_>>();
            let mut out = SurfaceMeshData::default();
            assert!(m.append_native_facility_workbench(&mut out, bounds, 18. * sy, paint));
            let actual = out.indices[..expected.len()]
                .iter()
                .map(|&i| (out.positions[i as usize], out.normals[i as usize]))
                .collect::<Vec<_>>();
            assert_eq!(
                actual, expected,
                "every original monitor, bench and keyboard triangle stays at its original fit"
            );
            for tri in out.indices.chunks_exact(3) {
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(out.positions[i as usize]));
                let normal = Vec3::from_array(out.normals[tri[0] as usize]);
                assert!((b - a).cross(c - a).dot(normal) > 0.);
                assert!((normal.length() - 1.).abs() < 0.00001);
            }
            assert!(out.positions.iter().all(|p| p[0] >= 3. - 0.0001
                && p[0] <= 3. + 32. * sx + 0.0001
                && p[1] >= 0.
                && p[1] <= 18. * sy + 0.0001
                && p[2] >= 7. - 0.0001
                && p[2] <= 7. + 18. * sy + 0.0001));
            assert!(
                out.positions
                    .iter()
                    .any(|p| p[0] < 3. + 16. * sx && p[1] > 16. * sy),
                "keep the original upright monitor"
            );
            assert!(
                out.positions
                    .iter()
                    .zip(&out.normals)
                    .zip(&out.uvs)
                    .any(|((p, n), uv)| *uv == paint[3]
                        && p[0] > 3. + 18. * sx
                        && p[1] > 11. * sy
                        && n[0] < -0.1
                        && n[2] > 0.1),
                "the rounded dark face points toward the keyboard/operator"
            );
            assert!(
                out.positions
                    .iter()
                    .zip(&out.normals)
                    .zip(&out.uvs)
                    .any(|((p, n), uv)| *uv == paint[2]
                        && p[0] > 3. + 18. * sx
                        && p[1] > 17.9 * sy
                        && n[1] > 0.9),
                "source pale crown is on top of the case"
            );
            assert_eq!(
                out.indices
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                out.positions.len()
            );
        }
        assert_eq!(
            m.surface, original,
            "generic cached source geometry remains immutable"
        );
    }
    #[test]
    fn facility_workbench_form_declines_partial_or_replaced_component_contracts() {
        let raw = crate::model_storage::include_model!(
            "models/facility_radio/facility_workbench.mesh.json"
        );
        let parsed: serde_json::Value = crate::model_storage::parse(raw).unwrap();
        for name in [
            "Keyboard raised cap",
            "Right instrument upper face",
            "Left display pedestal",
        ] {
            let mut changed = parsed.clone();
            let parts = changed["primitives"].as_array_mut().unwrap();
            let index = parts.iter().position(|p| p["name"] == name).unwrap();
            parts.remove(index);
            let m = Model::parse(&changed.to_string()).unwrap();
            assert!(!m.has_native_facility_workbench());
            let mut out = SurfaceMeshData::default();
            assert!(!m.append_native_facility_workbench(
                &mut out,
                [0., 32., 6., 24.],
                18.,
                [[0.; 2]; 4]
            ));
            assert!(out.positions.is_empty() && out.indices.is_empty());
        }
    }
}
