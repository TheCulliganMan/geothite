//! Checked native St Cloud/R05/R06 seams retaining the existing route chunks.
use crate::CompiledGamePack;
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, target, direction, offset, size, edge) in [
        (
            "MnStCloud",
            "MnR05C00R00",
            "south",
            0,
            [24, 18],
            [[24, 35], [25, 35]],
        ),
        (
            "MnR05C00R00",
            "MnStCloud",
            "north",
            0,
            [32, 24],
            [[24, 0], [25, 0]],
        ),
        (
            "MnStCloud",
            "MnR06C00R01",
            "north",
            4,
            [24, 18],
            [[24, 0], [25, 0]],
        ),
        (
            "MnR06C00R01",
            "MnStCloud",
            "south",
            -4,
            [32, 20],
            [[16, 39], [17, 39]],
        ),
    ] {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("St Cloud route/city module")?
            .clone();
        ensure!(
            [map.attributes.width, map.attributes.height] == size
                && !map
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == direction),
            "St Cloud route/city extents or occupied seam"
        );
        for at in edge {
            ensure!(
                crate::authored_town_geometry::kind(pack, &map, at)? == "FLOOR",
                "St Cloud mouth must be native floor"
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
    for (name, target, from, to) in [
        ("MnStCloud", "MnR05C00R00", [24, 36], [24, 0]),
        ("MnStCloud", "MnR05C00R00", [25, 36], [25, 0]),
        ("MnR05C00R00", "MnStCloud", [24, -1], [24, 35]),
        ("MnR05C00R00", "MnStCloud", [25, -1], [25, 35]),
        ("MnStCloud", "MnR06C00R01", [24, -1], [16, 39]),
        ("MnStCloud", "MnR06C00R01", [25, -1], [17, 39]),
        ("MnR06C00R01", "MnStCloud", [16, 40], [24, 0]),
        ("MnR06C00R01", "MnStCloud", [17, 40], [25, 0]),
    ] {
        let map = &pack.data.maps[name];
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        let transition = connection_for_tile_checked(&native, TilePosition::new(from[0], from[1]))?
            .context("St Cloud edge projection")?;
        let resolved = pack.data.resolve_connection_transition(&transition)?;
        ensure!(
            resolved.destination.map_name == target
                && resolved.destination.tile == TilePosition::new(to[0], to[1]),
            "St Cloud route projection mismatch"
        );
    }
    Ok(())
}
