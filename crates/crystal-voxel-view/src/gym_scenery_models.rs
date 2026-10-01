//! Cached original Gym scenery prototypes; source bindings live separately.
use super::Model;
use std::sync::OnceLock;
pub(crate) fn gym_model(index: usize) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            include_str!("../models/gym_scenery/planter_leafy.mesh.json"),
            include_str!("../models/gym_scenery/planter_round.mesh.json"),
            include_str!("../models/gym_scenery/azalea_broad_tree.mesh.json"),
            include_str!("../models/gym_scenery/celadon_round_hedge.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_00.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_01.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_02.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_03.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_04.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_05.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_06.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_07.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_08.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_09.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_0a.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_0b.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_0c.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_0d.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_0e.mesh.json"),
            include_str!("../models/gym_scenery/maze_wall_0f.mesh.json"),
        ]
        .into_iter()
        .map(|s| Model::parse(s).expect("validated original Gym model"))
        .collect()
    })[index]
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
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite())
                        && p[0] >= -8.001
                        && p[0] <= 8.001
                        && p[1] >= -0.001
                        && p[1] <= 12.001
                        && p[2] >= 15.999
                        && p[2] <= 32.001)
            );
        }
    }
}
