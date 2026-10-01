//! Original full-volume park fixtures; live pond art is composed by the mesher.
use crate::interior_models::Model;
use std::sync::OnceLock;
pub(crate) fn model(index: usize) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            include_str!("../models/park_scenery/litter_bin.mesh.json"),
            include_str!("../models/park_scenery/pedestal_fountain.mesh.json"),
            include_str!("../models/park_scenery/pond_basin.mesh.json"),
        ]
        .into_iter()
        .map(|data| Model::parse(data).expect("validated original park model"))
        .collect()
    })[index]
}
