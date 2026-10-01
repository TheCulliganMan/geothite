//! Original gate counter modules and telephone; every module is full-volume.
use crate::interior_models::Model;
use std::sync::OnceLock;
pub(crate) fn segment(mask: u8) -> &'static Model {
    macro_rules! load {
        ($slot:ident, $path:literal) => {{
            static $slot: OnceLock<Model> = OnceLock::new();
            $slot.get_or_init(|| {
                Model::parse(crate::model_storage::include_model!($path))
                    .expect("validated gate module")
            })
        }};
    }
    match mask {
        0 => load!(COUNTER_00, "models/gate_counters/counter_00.mesh.json"),
        1 => load!(COUNTER_01, "models/gate_counters/counter_01.mesh.json"),
        2 => load!(COUNTER_02, "models/gate_counters/counter_02.mesh.json"),
        3 => load!(COUNTER_03, "models/gate_counters/counter_03.mesh.json"),
        4 => load!(COUNTER_04, "models/gate_counters/counter_04.mesh.json"),
        5 => load!(COUNTER_05, "models/gate_counters/counter_05.mesh.json"),
        6 => load!(COUNTER_06, "models/gate_counters/counter_06.mesh.json"),
        7 => load!(COUNTER_07, "models/gate_counters/counter_07.mesh.json"),
        8 => load!(COUNTER_08, "models/gate_counters/counter_08.mesh.json"),
        9 => load!(COUNTER_09, "models/gate_counters/counter_09.mesh.json"),
        10 => load!(COUNTER_0A, "models/gate_counters/counter_0a.mesh.json"),
        11 => load!(COUNTER_0B, "models/gate_counters/counter_0b.mesh.json"),
        12 => load!(COUNTER_0C, "models/gate_counters/counter_0c.mesh.json"),
        13 => load!(COUNTER_0D, "models/gate_counters/counter_0d.mesh.json"),
        14 => load!(COUNTER_0E, "models/gate_counters/counter_0e.mesh.json"),
        15 => load!(COUNTER_0F, "models/gate_counters/counter_0f.mesh.json"),
        _ => unreachable!("counter edge masks have four bits"),
    }
}
pub(crate) fn phone() -> &'static Model {
    static PHONE: OnceLock<Model> = OnceLock::new();
    PHONE.get_or_init(|| {
        Model::parse(crate::model_storage::include_model!(
            "models/gate_counters/phone_console.mesh.json"
        ))
        .expect("validated gate phone")
    })
}
