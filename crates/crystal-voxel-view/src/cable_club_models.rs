//! Original Cable Club partition shells, cached once and batched with terrain.
use crate::interior_models::Model;
use std::sync::OnceLock;
pub(crate) fn divider() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::parse(include_str!("../models/cable_club/divider_long.mesh.json"))
            .expect("validated Cable Club divider")
    })
}
pub(crate) fn short_return() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::parse(include_str!(
            "../models/cable_club/vestibule_return.mesh.json"
        ))
        .expect("validated Cable Club return")
    })
}

pub(crate) fn time_capsule() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::parse(include_str!("../models/cable_club/time_capsule.mesh.json"))
            .expect("validated Cable Club Time Capsule")
    })
}

pub(crate) fn link_console() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::parse(include_str!("../models/cable_club/link_console.mesh.json"))
            .expect("validated Cable Club link console")
    })
}
