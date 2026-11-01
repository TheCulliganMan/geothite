//! Mankato's additive map and recovery identities.
use crate::authored_town_publication::{self, CityIdentity};
use crate::{CompiledGamePack, MapModule};
use anyhow::Result;

pub(super) fn publish(
    pack: &mut CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
) -> Result<()> {
    authored_town_publication::publish(
        pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_MANKATO",
            group_id: 75,
            landmark: "LANDMARK_MN_MANKATO",
            fallback_landmark: Some("LANDMARK_MN_PRAIRIE_FARM"),
            constant: "MN_MANKATO",
            name: "MANKATO",
            position: [72, 128],
            manifest: "minnesota-mankato-city-v1",
            town: "MnMankato",
            clinic: "MnMankatoPokecenter1F",
            gym: "MnMankatoGym",
        },
    )
}
