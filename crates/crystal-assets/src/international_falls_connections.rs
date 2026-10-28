//! Bind the northern road and a bounded snow approach to the existing icefields.
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
        .context("existing R17 north chunk")?;
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
    let ice = pack
        .data
        .maps
        .get("MnRainyRiverIcefields")
        .context("completed Rainy River region")?;
    ensure!(
        ice.attributes.width == 32
            && ice.attributes.height == 24
            && ice.attributes.tileset_name == "mn_winter_outdoor"
            && ice.attributes.connections.len() == 1
            && ice.attributes.connections[0].direction == "east"
            && ice.attributes.connections[0].target_map == "MnR20C00R00"
            && ice.attributes.connections[0].offset == 0,
        "existing icefield/R20 route must remain intact"
    );
    let mut ice = ice.clone();
    for x in 0..3 {
        let block = &mut ice.blocks[14 * 32 + x];
        ensure!(*block == 0x3f, "stale western snow approach preimage");
        *block = 0x3c;
    }
    ensure!(
        ice.blocks[14 * 32 + 3] == 0x3c,
        "western trail must meet existing hut snow"
    );
    crate::authored_town_publication::replace(pack, ice)?;
    for (name, direction, target, offset) in [
        ("MnInternationalFalls", "south", "MnR17C00R00", -8),
        ("MnR17C00R00", "north", "MnInternationalFalls", 8),
        ("MnInternationalFalls", "east", "MnRainyRiverIcefields", -7),
        ("MnRainyRiverIcefields", "west", "MnInternationalFalls", 7),
    ] {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("Falls connection source")?
            .clone();
        crate::lake_country_geometry::connect(&mut map, direction, target, offset)?;
        pack.data
            .map_attributes
            .insert(name.into(), map.attributes.clone());
        pack.data.maps.insert(name.into(), map);
    }
    for (name, target, at, outside, wanted) in [
        (
            "MnInternationalFalls",
            "MnR17C00R00",
            [20, 27],
            [20, 28],
            [36, 0],
        ),
        (
            "MnInternationalFalls",
            "MnR17C00R00",
            [21, 27],
            [21, 28],
            [37, 0],
        ),
        (
            "MnR17C00R00",
            "MnInternationalFalls",
            [36, 0],
            [36, -1],
            [20, 27],
        ),
        (
            "MnR17C00R00",
            "MnInternationalFalls",
            [37, 0],
            [37, -1],
            [21, 27],
        ),
        (
            "MnInternationalFalls",
            "MnRainyRiverIcefields",
            [31, 14],
            [32, 14],
            [0, 28],
        ),
        (
            "MnInternationalFalls",
            "MnRainyRiverIcefields",
            [31, 15],
            [32, 15],
            [0, 29],
        ),
        (
            "MnRainyRiverIcefields",
            "MnInternationalFalls",
            [0, 28],
            [-1, 28],
            [31, 14],
        ),
        (
            "MnRainyRiverIcefields",
            "MnInternationalFalls",
            [0, 29],
            [-1, 29],
            [31, 15],
        ),
        (
            "MnR20C01R00",
            "MnBoundaryWaters",
            [63, 36],
            [64, 36],
            [0, 24],
        ),
        (
            "MnR20C01R00",
            "MnBoundaryWaters",
            [63, 37],
            [64, 37],
            [0, 25],
        ),
        (
            "MnBoundaryWaters",
            "MnR20C01R00",
            [0, 24],
            [-1, 24],
            [63, 36],
        ),
        (
            "MnBoundaryWaters",
            "MnR20C01R00",
            [0, 25],
            [-1, 25],
            [63, 37],
        ),
    ] {
        let map = pack
            .data
            .maps
            .get(name)
            .context("Falls or Voyageurs connection source")?;
        let destination_map = pack
            .data
            .maps
            .get(target)
            .context("Falls or Voyageurs destination")?;
        ensure!(
            u32::from(at[0]) < u32::from(map.attributes.width) * 2
                && u32::from(at[1]) < u32::from(map.attributes.height) * 2
                && u32::from(wanted[0]) < u32::from(destination_map.attributes.width) * 2
                && u32::from(wanted[1]) < u32::from(destination_map.attributes.height) * 2,
            "Falls/Voyageurs endpoint outside its native map"
        );
        ensure!(
            kind(pack, map, at)? == "FLOOR" && kind(pack, destination_map, wanted)? == "FLOOR",
            "both Falls road lanes need native FLOOR"
        );
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        let transition =
            connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                .context("Falls reciprocal connection")?;
        let destination = pack
            .data
            .resolve_connection_transition(&transition)?
            .destination;
        ensure!(
            destination.map_name == target
                && destination.tile == TilePosition::new(wanted[0] as i16, wanted[1] as i16),
            "Falls native connection projection differs"
        );
    }
    Ok(())
}
