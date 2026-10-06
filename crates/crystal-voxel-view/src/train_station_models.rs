//! One original closed-part train prototype, decoded and cached only once.
use crate::interior_models::Model;
use std::sync::OnceLock;
pub(crate) fn train() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::parse(crate::model_storage::include_model!(
            "models/train_station/magnet_train_shell.mesh.json"
        ))
        .expect("validated original Magnet Train shell")
    })
}
