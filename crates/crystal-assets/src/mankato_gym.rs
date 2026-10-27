//! Mankato binds only its captured canonical second gym.
use anyhow::Result;
use serde_json::Value;

use crate::authored_town_design::Room;
use crate::canonical_city_gym::{self, Binding};
use crate::{CompiledGamePack, MapModule};

pub(super) fn bind(
    pack: &CompiledGamePack,
    room: &Room,
    reference: &Value,
    source: &str,
) -> Result<MapModule> {
    canonical_city_gym::bind(
        pack,
        room,
        reference,
        source,
        Binding {
            slot: 2,
            town: "MnMankato",
            constant: "MN_MANKATO",
            return_warp: 6,
        },
    )
}
