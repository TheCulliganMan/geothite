//! Rearrange the selected closed CRT kit to match the office's side keyboard.
use super::*;

impl Model {
    fn pokecom_keyboard_vertex(&self, i: usize) -> bool {
        let p = self.surface.positions[i];
        self.native_computer_materials
            .as_ref()
            .is_some_and(|roles| roles[i] != 3)
            && (0.8474..=0.9286).contains(&p[1])
            && p[2] >= 0.0949
    }
    pub(crate) fn has_native_pokecom_station(&self) -> bool {
        let Some(roles) = &self.native_computer_materials else {
            return false;
        };
        // This adapter only understands the selected closed monitor/keyboard
        // kit. Unknown replacement geometry retains the source drawing.
        self.surface
            .positions
            .iter()
            .enumerate()
            .filter(|(i, _)| self.pokecom_keyboard_vertex(*i))
            .count()
            == 3072
            && roles.iter().filter(|r| **r == 3).count() == 252
            && self.surface.indices.chunks_exact(3).all(|tri| {
                let keys = tri
                    .iter()
                    .filter(|&&i| self.pokecom_keyboard_vertex(i as usize))
                    .count();
                keys == 0 || keys == 3
            })
    }
    pub(crate) fn append_native_pokecom_station(
        &self,
        out: &mut SurfaceMeshData,
        bounds: [f32; 4],
        base: f32,
        height: f32,
        paint: [[f32; 2]; 5],
    ) -> bool {
        if !self.has_native_pokecom_station()
            || bounds[1] <= bounds[0]
            || bounds[3] <= bounds[2]
            || height <= 0.
        {
            return false;
        }
        let first = out.positions.len();
        let half = (bounds[1] - bounds[0]) * 0.5;
        self.append_fitted_with_footprint(
            out,
            [bounds[0], bounds[0] + half, bounds[2], bounds[3]],
            base,
            height,
            true,
        );
        let roles = self.native_computer_materials.as_ref().unwrap();
        let depth_scale = (bounds[3] - bounds[2]) / (self.max[2] - self.min[2]);
        for (i, &role) in roles.iter().enumerate() {
            let p = &mut out.positions[first + i];
            if role == 3 {
                // A continuous desktop and its four legs support both halves.
                p[0] = bounds[0] + (p[0] - bounds[0]) * 2.;
                let n = Vec3::from_array(out.normals[first + i]);
                out.normals[first + i] = Vec3::new(n.x * 0.5, n.y, n.z).normalize().to_array();
            } else if self.pokecom_keyboard_vertex(i) {
                // Move the entire closed keyboard casing and all keys together.
                p[0] += half;
                p[2] -= 0.28 * depth_scale;
            }
            out.uvs[first + i] = paint[role as usize];
            out.colors[first + i] =
                authored_face_color([1.; 4], Vec3::from_array(out.normals[first + i]));
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_pokecom_station_moves_closed_keyboard_and_keeps_monitor_and_cache() {
        let m = model(ModelKind::Computer);
        assert!(m.has_native_pokecom_station());
        let snapshot = m.surface.positions.clone();
        for [width, depth, height] in [[32., 12., 22.], [48., 18., 33.], [24., 15., 19.]] {
            let bounds = [3., 3. + width, 7., 7. + depth];
            let paint = [[0.1, 0.2], [0.2, 0.3], [0.3, 0.4], [0.4, 0.5], [0.5, 0.6]];
            let mut before = SurfaceMeshData::default();
            m.append_fitted_with_footprint(
                &mut before,
                [3., 3. + width * 0.5, 7., 7. + depth],
                2.,
                height,
                true,
            );
            let mut after = SurfaceMeshData::default();
            assert!(m.append_native_pokecom_station(&mut after, bounds, 2., height, paint));
            assert_eq!(after.indices, before.indices);
            assert_eq!(after.positions.len(), before.positions.len());
            let roles = m.native_computer_materials.as_ref().unwrap();
            let mut keys = 0;
            for (i, (old, new)) in before.positions.iter().zip(&after.positions).enumerate() {
                if m.pokecom_keyboard_vertex(i) {
                    keys += 1;
                    assert!((new[0] - old[0] - width * 0.5).abs() < 0.00001);
                    assert!(new[0] > bounds[0] + width * 0.5 && new[0] < bounds[1]);
                    assert!(
                        (new[2] - old[2] + 0.28 * depth / (m.max[2] - m.min[2])).abs() < 0.00001
                    );
                    assert_eq!(new[1], old[1]);
                    assert_eq!(after.normals[i], before.normals[i]);
                } else if roles[i] != 3 {
                    assert_eq!(new, old);
                }
                assert!(
                    new[0] >= bounds[0]
                        && new[0] <= bounds[1]
                        && new[2] >= bounds[2]
                        && new[2] <= bounds[3]
                );
                assert!((Vec3::from_array(after.normals[i]).length() - 1.).abs() < 0.00001);
                assert_eq!(after.uvs[i], paint[roles[i] as usize]);
            }
            assert_eq!(keys, 3072);
        }
        assert_eq!(m.surface.positions, snapshot);
        let mut changed: serde_json::Value =
            serde_json::from_str(include_str!("../models/interiors/computer.mesh.json")).unwrap();
        changed["name"] = "unknown-computer".into();
        let unknown = Model::parse(changed.to_string().as_str()).unwrap();
        let mut out = SurfaceMeshData::default();
        assert!(!unknown.append_native_pokecom_station(
            &mut out,
            [0., 32., 0., 12.],
            0.,
            22.,
            [[0.; 2]; 5]
        ));
        assert!(out.positions.is_empty() && out.indices.is_empty());
    }
}
