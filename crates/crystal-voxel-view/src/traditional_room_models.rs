//! Three original cached prototypes, decoded by the existing storage reader.
use super::Model;
use std::sync::OnceLock;
pub(crate) fn traditional_model(index: usize) -> &'static Model {
    static MODELS: OnceLock<Vec<Model>> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/traditional_room/slatted_rail_bay.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/traditional_room/theater_platform.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/traditional_room/theater_backdrop.mesh.json"
            ),
        ]
        .into_iter()
        .map(|s| Model::parse(s).expect("validated original traditional-room model"))
        .collect()
    })[index]
}
