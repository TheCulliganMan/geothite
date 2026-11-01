//! Connect Falls to its actual R17 endpoint without inventing an icefield shortcut.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let route = pack
        .data
        .maps
        .get("MnR17C00R00")
        .context("R17 northern endpoint")?;
    ensure!(
        route.attributes.width == 32
            && route.attributes.height == 24
            && route.attributes.tileset_name == "mn_winter_outdoor"
            && route.attributes.connections.len() == 1
            && route.attributes.connections[0].direction == "south"
            && route.attributes.connections[0].target_map == "MnR17C00R01"
            && route.attributes.connections[0].offset == 0,
        "original R17 northern endpoint required"
    );
    for (name, direction, target, offset) in [
        ("MnInternationalFalls", "south", "MnR17C00R00", -8),
        ("MnR17C00R00", "north", "MnInternationalFalls", 8),
    ] {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("Falls road source")?
            .clone();
        crate::lake_country_geometry::connect(&mut map, direction, target, offset)?;
        pack.data
            .map_attributes
            .insert(name.into(), map.attributes.clone());
        pack.data.maps.insert(name.into(), map);
    }
    for lane in [20_u16, 21] {
        let signed = i16::try_from(lane)?;
        for (name, target, at, outside, wanted) in [
            (
                "MnInternationalFalls",
                "MnR17C00R00",
                [lane, 27],
                [signed, 28],
                [lane + 16, 0],
            ),
            (
                "MnR17C00R00",
                "MnInternationalFalls",
                [lane + 16, 0],
                [signed + 16, -1],
                [lane, 27],
            ),
        ] {
            let map = &pack.data.maps[name];
            let other = &pack.data.maps[target];
            ensure!(
                kind(pack, map, at)? == "FLOOR" && kind(pack, other, wanted)? == "FLOOR",
                "both Falls road lanes need native Floor"
            );
            let native =
                OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
            let transition =
                connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                    .context("Falls reciprocal road")?;
            let destination = pack
                .data
                .resolve_connection_transition(&transition)?
                .destination;
            ensure!(
                destination.map_name == target
                    && destination.tile
                        == TilePosition::new(i16::try_from(wanted[0])?, i16::try_from(wanted[1])?),
                "Falls road projection differs"
            );
        }
    }
    Ok(())
}
