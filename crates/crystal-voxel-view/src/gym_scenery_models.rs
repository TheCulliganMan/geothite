//! Cached original Gym scenery prototypes; source bindings live separately.
use super::Model;
use std::sync::OnceLock;
#[path = "gym_scenery_occlusion.rs"]
mod occlusion;
pub(crate) fn gym_model(index: usize) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!("models/gym_scenery/planter_leafy.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/planter_round.mesh.json"),
            crate::model_storage::include_model!(
                "models/gym_scenery/azalea_broad_tree.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/gym_scenery/celadon_round_hedge.mesh.json"
            ),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_00.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_01.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_02.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_03.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_04.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_05.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_06.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_07.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_08.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_09.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0a.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0b.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0c.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0d.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0e.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0f.mesh.json"),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            let parsed = if index < 2 {
                occlusion::parse_planter(source)
            } else {
                Model::parse(source)
            };
            parsed.expect("validated original Gym model")
        })
        .collect()
    })[index]
}
/// Ordinals of internal triangles with source-proven complete neighbor cover.
/// The public model and full terrain mesh remain unchanged. Only the runtime
/// draw partition consumes these ordinals, retaining their complete geometry.
pub(crate) fn gym_wall_join_triangles(exposed: u8, covered: u8) -> &'static [usize] {
    let exposed = exposed & 15;
    let covered = covered & !exposed & 15;
    static BATCHES: OnceLock<Vec<OnceLock<Vec<usize>>>> = OnceLock::new();
    BATCHES.get_or_init(|| (0..256).map(|_| OnceLock::new()).collect())
        [usize::from(exposed) * 16 + usize::from(covered)]
    .get_or_init(|| {
        let model = gym_model(4 + usize::from(exposed));
        model
            .surface
            .indices
            .chunks_exact(3)
            .enumerate()
            .filter_map(|(ordinal, triangle)| {
                covered_join_triangle(model, triangle, covered).then_some(ordinal)
            })
            .collect()
    })
}

fn covered_join_triangle(model: &Model, indices: &[u32], covered: u8) -> bool {
    // N/E/S/W use the export's +Y up, +Z south coordinates. Exact plane and
    // outward-normal checks retain beveled, inset, or non-boundary geometry.
    [
        (1, 2, model.min[2], -1.0),
        (2, 0, model.max[0], 1.0),
        (4, 2, model.max[2], 1.0),
        (8, 0, model.min[0], -1.0),
    ]
    .into_iter()
    .any(|(bit, axis, plane, direction)| {
        covered & bit != 0
            && indices.iter().all(|&i| {
                model.surface.positions[i as usize][axis] == plane
                    && model.surface.normals[i as usize][axis] == direction
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gym_prototypes_decode_and_fit_finite_closed_volume_bounds() {
        for index in 0..20 {
            let mut mesh = crate::mesh::SurfaceMeshData::default();
            gym_model(index).append(&mut mesh, [-8., 8., 16., 32.], 0., 12.);
            assert!(!mesh.indices.is_empty());
            assert!(mesh
                .positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite())
                    && p[0] >= -8.001
                    && p[0] <= 8.001
                    && p[1] >= -0.001
                    && p[1] <= 12.001
                    && p[2] >= 15.999
                    && p[2] <= 32.001));
        }
    }
    #[test]
    fn every_removed_join_triangle_is_inside_the_actual_neighbor_prism() {
        let sources = [
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_00.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_01.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_02.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_03.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_04.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_05.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_06.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_07.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_08.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_09.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0a.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0b.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0c.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0d.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0e.mesh.json"),
            crate::model_storage::include_model!("models/gym_scenery/maze_wall_0f.mesh.json"),
        ];
        let exports: Vec<super::super::Export> = sources
            .into_iter()
            .map(|source| crate::model_storage::parse(source).unwrap())
            .collect();
        let mut checked = 0;
        let mut triangles = 0;
        for own in 0..16_u8 {
            for (bit, opposite, axis, plane, direction, perpendicular) in [
                (1, 4, 2, -0.5, -1.0, 10),
                (2, 8, 0, 0.5, 1.0, 5),
                (4, 1, 2, 0.5, 1.0, 10),
                (8, 2, 0, -0.5, -1.0, 5),
            ] {
                if own & bit != 0 {
                    continue;
                }
                for neighbor in 0..16_u8 {
                    if neighbor & opposite != 0 || neighbor & perpendicular & !own != 0 {
                        continue;
                    }
                    checked += 1;
                    let mut removed = 0;
                    for (part, source) in exports[usize::from(own)].primitives.iter().enumerate() {
                        for tri in source.indices.chunks_exact(3) {
                            if !tri.iter().all(|&i| {
                                source.positions[i as usize * 3 + axis] == plane
                                    && source.normals[i as usize * 3 + axis] == direction
                            }) {
                                continue;
                            }
                            assert!(part < 4, "only masonry courses may reach a closed join");
                            let other = &exports[usize::from(neighbor)].primitives[part];
                            for &i in tri {
                                let offset = i as usize * 3;
                                let mut point = bevy::prelude::Vec3::from_slice(
                                    &source.positions[offset..offset + 3],
                                );
                                point[axis] -= direction;
                                for face in other.indices.chunks_exact(3) {
                                    let offset = face[0] as usize * 3;
                                    let plane = bevy::prelude::Vec3::from_slice(
                                        &other.positions[offset..offset + 3],
                                    );
                                    let b = face[1] as usize * 3;
                                    let c = face[2] as usize * 3;
                                    let b =
                                        bevy::prelude::Vec3::from_slice(&other.positions[b..b + 3]);
                                    let c =
                                        bevy::prelude::Vec3::from_slice(&other.positions[c..c + 3]);
                                    let normal = (b - plane).cross(c - plane).normalize();
                                    assert!(
                                        normal.dot(point - plane) <= 0.00001,
                                        "join leaves closed neighbor: own={own} neighbor={neighbor} side={bit} part={part}"
                                    );
                                }
                            }
                            removed += 1;
                        }
                    }
                    assert_eq!(removed, 8);
                    triangles += removed;
                }
            }
        }
        assert_eq!((checked, triangles), (144, 1152));
    }
}
