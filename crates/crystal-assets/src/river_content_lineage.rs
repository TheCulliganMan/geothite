//! Publication constraints for the Mississippi, fruit and frozen-water overlays.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

pub(crate) fn validate(pack: &CompiledGamePack) -> Result<()> {
    crate::minnesota_geometry_publication::validate(pack)?;
    let mut coast = 0;
    for (name, map) in &pack.data.maps {
        if !name.starts_with("MnR11C") {
            continue;
        }
        coast += 1;
        ensure!(
            map.attributes.tileset_name == "mn_north_shore",
            "North Shore must remain unfrozen: {name}"
        );
        let tileset = &pack.data.tilesets[&map.attributes.tileset_name];
        for block in &map.blocks {
            ensure!(
                tileset.collision[&format!("{block:02x}")]
                    .iter()
                    .all(|kind| kind != "ICE"),
                "North Shore contains sliding ice: {name}"
            );
        }
    }
    ensure!(coast == 8, "complete warm North Shore required");
    Ok(())
}
