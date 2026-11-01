//! Bind the existing R16 western road to the new, otherwise closed town.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let route = pack
        .data
        .maps
        .get("MnR16C00R00")
        .context("Moorhead requires the existing R16 western chunk")?;
    ensure!(
        route.attributes.width == 32
            && route.attributes.height == 24
            && route.attributes.tileset_name == "mn_prairie_headwaters"
            && route.attributes.connections.len() == 1
            && route.attributes.connections[0].direction == "east"
            && route.attributes.connections[0].target_map == "MnR16C01R00"
            && route.attributes.connections[0].offset == 0,
        "original R16 west chunk and eastern link required"
    );
    ensure!(
        pack.data.maps["MnMoorhead"]
            .attributes
            .connections
            .is_empty(),
        "Moorhead must have only its declared prairie port"
    );
    for (name, direction, target, offset) in [
        ("MnMoorhead", "east", "MnR16C00R00", -6),
        ("MnR16C00R00", "west", "MnMoorhead", 6),
    ] {
        let mut map = pack.data.maps[name].clone();
        crate::lake_country_geometry::connect(&mut map, direction, target, offset)?;
        pack.data
            .map_attributes
            .insert(name.into(), map.attributes.clone());
        pack.data.maps.insert(name.into(), map);
    }
    for (name, target, at, outside, wanted) in [
        ("MnMoorhead", "MnR16C00R00", [31, 12], [32, 12], [0, 24]),
        ("MnMoorhead", "MnR16C00R00", [31, 13], [32, 13], [0, 25]),
        ("MnR16C00R00", "MnMoorhead", [0, 24], [-1, 24], [31, 12]),
        ("MnR16C00R00", "MnMoorhead", [0, 25], [-1, 25], [31, 13]),
    ] {
        let map = &pack.data.maps[name];
        ensure!(
            kind(pack, map, at)? == "FLOOR"
                && kind(pack, &pack.data.maps[target], wanted)? == "FLOOR",
            "both Moorhead connection lanes must be native FLOOR"
        );
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        let transition =
            connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                .context("Moorhead reciprocal road")?;
        let destination = pack
            .data
            .resolve_connection_transition(&transition)?
            .destination;
        ensure!(
            destination.map_name == target
                && destination.tile == TilePosition::new(wanted[0] as i16, wanted[1] as i16),
            "Moorhead native road projection differs"
        );
    }
    Ok(())
}
