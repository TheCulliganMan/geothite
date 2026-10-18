//! One original closed-part train prototype, decoded and cached only once.
use crate::interior_models::Model;
use std::sync::OnceLock;
// The round section and boarding opening must clear the enlarged player.
pub(crate) const HEIGHT_PIXELS: f32 = 33.0;
pub(crate) const BOARDING_CLEARANCE_PIXELS: f32 = 21.0 / 26.2 * HEIGHT_PIXELS;

pub(crate) fn train() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::parse(crate::model_storage::include_model!(
            "models/train_station/magnet_train_shell.mesh.json"
        ))
        .expect("validated original Magnet Train shell")
    })
}
