//! Prove the town's retained road and four river lanes before allowing its overlay.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::CompiledGamePack;
use crate::authored_town_geometry;

pub(crate) fn validate(pack: &CompiledGamePack) -> Result<BTreeSet<[u16; 2]>> {
    let town = pack.data.maps.get("MnWinona").context("Winona")?;
    ensure!(
        town.attributes.connections.len() == 2,
        "retain both original Winona connections"
    );
    let mouth = pack
        .data
        .maps
        .get("MnMississippiWinonaMouth")
        .context("river mouth")?;
    let bottom = i16::try_from(mouth.attributes.height * 2 - 1)?;
    let mut lanes = Vec::new();
    for y in [18_i16, 19] {
        lanes.push((
            "MnWinona",
            "MnR01C00R00",
            [0, y],
            [-1, y],
            [63, y + 6],
            "FLOOR",
        ));
        lanes.push((
            "MnR01C00R00",
            "MnWinona",
            [63, y + 6],
            [64, y + 6],
            [0, y],
            "FLOOR",
        ));
    }
    for x in 26..=29 {
        lanes.push((
            "MnWinona",
            "MnMississippiWinonaMouth",
            [x, 0],
            [x, -1],
            [x, bottom],
            "WATER",
        ));
        lanes.push((
            "MnMississippiWinonaMouth",
            "MnWinona",
            [x, bottom],
            [x, bottom + 1],
            [x, 0],
            "WATER",
        ));
    }
    for (source, target, at, outside, expected, kind) in lanes {
        let map = &pack.data.maps[source];
        let to_unsigned = |point: [i16; 2]| -> Result<[u16; 2]> {
            Ok([u16::try_from(point[0])?, u16::try_from(point[1])?])
        };
        ensure!(
            authored_town_geometry::kind(pack, map, to_unsigned(at)?)? == kind
                && authored_town_geometry::kind(
                    pack,
                    &pack.data.maps[target],
                    to_unsigned(expected)?
                )? == kind,
            "retained Winona lane collision changed: {source} to {target}"
        );
        let native = OverworldMapData::from_attributes(source, &map.attributes, map.blocks.clone());
        let crossing =
            connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                .context("retained Winona lane trigger")?;
        let arrival = pack
            .data
            .resolve_connection_transition(&crossing)?
            .destination;
        ensure!(
            arrival.map_name == target
                && arrival.tile == TilePosition::new(expected[0], expected[1]),
            "retained Winona lane projection changed: {source} to {target}"
        );
    }
    Ok(BTreeSet::from([
        [0, 18],
        [0, 19],
        [26, 0],
        [27, 0],
        [28, 0],
        [29, 0],
    ]))
}
