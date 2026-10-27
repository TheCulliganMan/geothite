//! Exact two-tile public R04/St Paul walking seam, preserving route encounters.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::CompiledGamePack;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, target, direction, offset, size, edge) in [
        (
            "MnStPaul",
            "MnR04C00R00",
            "west",
            3,
            [32, 24],
            [[0, 24], [0, 25]],
        ),
        (
            "MnR04C00R00",
            "MnStPaul",
            "east",
            -3,
            [28, 18],
            [[55, 18], [55, 19]],
        ),
    ] {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("installed St Paul/R04")?
            .clone();
        ensure!(
            [map.attributes.width, map.attributes.height] == size
                && !map
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == direction),
            "St Paul/R04 extents or duplicate seam"
        );
        for at in edge {
            ensure!(
                crate::authored_town_geometry::kind(pack, &map, at)? == "FLOOR",
                "St Paul/R04 native mouth is not floor"
            );
        }
        map.attributes.connections.push(MapConnection {
            direction: direction.into(),
            target_map: target.into(),
            offset,
        });
        map.attributes.connection_flags = Some(
            map.attributes
                .connections
                .iter()
                .map(|c| c.direction.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join(" | "),
        );
        crate::authored_town_publication::replace(pack, map)?;
    }
    for (name, target, x, y, expected) in [
        ("MnStPaul", "MnR04C00R00", -1, 24, TilePosition::new(55, 18)),
        ("MnStPaul", "MnR04C00R00", -1, 25, TilePosition::new(55, 19)),
        ("MnR04C00R00", "MnStPaul", 56, 18, TilePosition::new(0, 24)),
        ("MnR04C00R00", "MnStPaul", 56, 19, TilePosition::new(0, 25)),
    ] {
        let map = &pack.data.maps[name];
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        let transition = connection_for_tile_checked(&native, TilePosition::new(x, y))?
            .context("St Paul edge projection")?;
        let resolved = pack.data.resolve_connection_transition(&transition)?;
        ensure!(
            resolved.destination.map_name == target && resolved.destination.tile == expected,
            "St Paul/R04 reciprocal projection differs"
        );
    }
    Ok(())
}
