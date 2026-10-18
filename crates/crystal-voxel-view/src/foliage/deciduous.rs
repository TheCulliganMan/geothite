//! Three authored CC0 broadleaf models, kept upright and rooted in native plots.
use super::*;
use std::sync::OnceLock;

#[derive(serde::Deserialize)]
struct Model {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    foliage_start: usize,
}
fn models() -> &'static [Model; 3] {
    static MODELS: OnceLock<[Model; 3]> = OnceLock::new();
    MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!("models/deciduous/spreading.mesh.json"),
            crate::model_storage::include_model!("models/deciduous/forked.mesh.json"),
            crate::model_storage::include_model!("models/deciduous/upright.mesh.json"),
        ]
        .map(|source| {
            crate::model_storage::parse(source).expect("validated authored deciduous model")
        })
    })
}
pub(super) fn height(tile: f32, coordinates: [i32; 2]) -> f32 {
    let v = variation(coordinates[0], coordinates[1]);
    tile * (6.0 + non_ancient_height_factor(coordinates) * 1.1) * [0.94, 1.0, 1.12][v.index]
}
pub(super) fn width_ratio(coordinates: [i32; 2], prototype: [i32; 2]) -> f32 {
    variation(coordinates[0], coordinates[1]).width / variation(prototype[0], prototype[1]).width
}
pub(super) fn append_canopy(
    mesh: &mut TerrainMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
    _neighbors: [bool; 4],
) {
    let v = variation(coordinates[0], coordinates[1]);
    let model = &models()[v.index];
    let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
    let offset = grove_trunk_offset(coordinates, spacing);
    let root = Vec3::new(
        (bounds[0] + bounds[1]) * 0.5 + offset.x,
        base,
        (bounds[2] + bounds[3]) * 0.5 + offset.y,
    );
    let model_height = model.positions.iter().map(|p| p[1]).fold(0., f32::max);
    let mut scale = Vec3::new(
        spacing * 0.86 * v.width,
        height(tile, coordinates) / model_height,
        spacing * 0.86 * v.width,
    );
    let rotation = Quat::from_rotation_y(v.turn);
    let mut fit = 1f32;
    // Authoring roots are genuinely modeled. Fit their complete flare, rather
    // than stretching the source plot or clipping roots onto neighboring paths.
    for &p in &model.positions[..model.foliage_start] {
        if p[1] * scale.y > tile * 0.18 {
            continue;
        }
        let d = rotation * (Vec3::from_array(p) * scale);
        for (delta, center, low, high) in [
            (d.x, root.x, bounds[0], bounds[1]),
            (d.z, root.z, bounds[2], bounds[3]),
        ] {
            if delta > 0. {
                fit = fit.min((high - center) / delta);
            }
            if delta < 0. {
                fit = fit.min((low - center) / delta);
            }
        }
    }
    scale.x *= fit;
    scale.z *= fit;
    let normals = Mat3::from_quat(rotation) * Mat3::from_diagonal(scale.recip());
    let first = mesh.solid.positions.len() as u32;
    for (i, (&p, &n)) in model.positions.iter().zip(&model.normals).enumerate() {
        let point = root + rotation * (Vec3::from_array(p) * scale);
        let mut color = model.colors[i];
        if i >= model.foliage_start {
            let pigment = leaf_pigment(point, tile * 1.5);
            // Keep the three models in the same leafy woodland palette while
            // retaining the artist's baked shade variation on individual leaves.
            color = [
                (color[0] * 0.72 + color[1] * 0.22) * pigment[0] * v.tint[0],
                color[1] * 0.88 * pigment[1] * v.tint[1],
                (color[2] + color[1] * 0.28) * pigment[2] * v.tint[2],
                1.,
            ];
        }
        mesh.solid.positions.push(point.to_array());
        mesh.solid
            .normals
            .push((normals * Vec3::from_array(n)).normalize().to_array());
        mesh.solid.colors.push(color);
        mesh.solid.uvs.push([0.; 2]);
    }
    mesh.solid
        .indices
        .extend(model.indices.iter().map(|&i| first + i));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn three_authored_models_have_distinct_branches_and_solid_cupped_leaves() {
        let models = models();
        assert_ne!(models[0].positions.len(), models[1].positions.len());
        assert_ne!(models[1].positions.len(), models[2].positions.len());
        for m in models {
            assert!(m.foliage_start > 4000 && m.foliage_start < m.positions.len());
            assert!(m.positions.len() < 20_000);
            assert!(
                m.colors[m.foliage_start..].iter().all(|c| c[1] > 0.05),
                "source card borders must not turn modeled leaves black"
            );
            assert_eq!((m.positions.len() - m.foliage_start) % 12, 0);
            assert!(
                m.positions
                    .iter()
                    .all(|p| p[1] >= 0. && p.iter().all(|v| v.is_finite()))
            );
            for ids in m
                .indices
                .chunks_exact(3)
                .filter(|ids| ids.iter().all(|&i| i as usize >= m.foliage_start))
            {
                let p: [Vec3; 3] =
                    std::array::from_fn(|j| Vec3::from_array(m.positions[ids[j] as usize]));
                let face = (p[1] - p[0]).cross(p[2] - p[0]);
                let n = ids
                    .iter()
                    .map(|&i| Vec3::from_array(m.normals[i as usize]))
                    .sum::<Vec3>();
                assert!(face.length_squared() > 1e-14 && face.dot(n) > 0.);
            }
        }
    }
    #[test]
    fn deciduous_trees_stay_upright_rooted_and_deterministic_at_native_scales() {
        for tile in [8., 16.] {
            for coordinates in [[0, 0], [4, 8], [-12, 2], [29, 4], [7, 11], [1, 8]] {
                let mut a = TerrainMeshData::default();
                let mut b = TerrainMeshData::default();
                let bounds = [0., tile * 2., 0., tile * 2.];
                append_canopy(&mut a, bounds, 3., tile, coordinates, [false; 4]);
                append_canopy(&mut b, bounds, 3., tile, coordinates, [true; 4]);
                assert_eq!(a.solid.positions, b.solid.positions);
                let bottom = a
                    .solid
                    .positions
                    .iter()
                    .map(|p| p[1])
                    .fold(f32::INFINITY, f32::min);
                let top = a.solid.positions.iter().map(|p| p[1]).fold(0., f32::max);
                assert!((bottom - 3.).abs() < 0.001);
                assert!((top - 3. - height(tile, coordinates)).abs() < 0.001);
                assert!(top > 3. + tile * 6. && top < 3. + tile * 9.);
                assert!(
                    a.solid
                        .positions
                        .iter()
                        .filter(|p| p[1] < 3. + tile * 0.18)
                        .all(|p| p[0] >= -0.001
                            && p[0] <= tile * 2. + 0.001
                            && p[2] >= -0.001
                            && p[2] <= tile * 2. + 0.001)
                );
                assert!(
                    a.solid
                        .normals
                        .iter()
                        .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
                );
            }
        }
    }
}
