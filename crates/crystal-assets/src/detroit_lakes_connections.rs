//! Insert the real lakeside town without discarding the wooded connector.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    replace(
        pack,
        "MnR15C00R00",
        "west",
        "MnDetroitLakesApproach",
        3,
        "MnDetroitLakes",
        3,
    )?;
    replace(
        pack,
        "MnDetroitLakesApproach",
        "east",
        "MnR15C00R00",
        -3,
        "MnDetroitLakes",
        0,
    )?;
    for (name, side, target, offset) in [
        ("MnDetroitLakes", "east", "MnR15C00R00", -3),
        ("MnDetroitLakes", "west", "MnDetroitLakesApproach", 0),
    ] {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("Detroit road endpoint")?
            .clone();
        crate::lake_country_geometry::connect(&mut map, side, target, offset)?;
        publish(pack, map);
    }
    for (name, side, edge, target, destination) in [
        (
            "MnDetroitLakes",
            "east",
            [[47, 18], [47, 19]],
            "MnR15C00R00",
            [[0, 24], [0, 25]],
        ),
        (
            "MnR15C00R00",
            "west",
            [[0, 24], [0, 25]],
            "MnDetroitLakes",
            [[47, 18], [47, 19]],
        ),
        (
            "MnDetroitLakes",
            "west",
            [[0, 18], [0, 19]],
            "MnDetroitLakesApproach",
            [[47, 18], [47, 19]],
        ),
        (
            "MnDetroitLakesApproach",
            "east",
            [[47, 18], [47, 19]],
            "MnDetroitLakes",
            [[0, 18], [0, 19]],
        ),
    ] {
        let map = &pack.data.maps[name];
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        for (at, wanted) in edge.into_iter().zip(destination) {
            ensure!(
                kind(pack, map, at)? == "FLOOR"
                    && kind(pack, &pack.data.maps[target], wanted)? == "FLOOR",
                "both Detroit seam endpoints need native FLOOR"
            );
            let [x, y] = at.map(|v| v as i16);
            let outside = match side {
                "east" => [x + 1, y],
                "west" => [x - 1, y],
                "north" => [x, y - 1],
                "south" => [x, y + 1],
                _ => unreachable!(),
            };
            let transition =
                connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                    .context("Detroit road seam")?;
            let actual = pack
                .data
                .resolve_connection_transition(&transition)?
                .destination;
            ensure!(
                actual.map_name == target
                    && actual.tile == TilePosition::new(wanted[0] as i16, wanted[1] as i16),
                "Detroit native seam projection differs"
            );
        }
    }
    Ok(())
}

fn replace(
    pack: &mut CompiledGamePack,
    name: &str,
    side: &str,
    old_target: &str,
    old_offset: i32,
    target: &str,
    offset: i32,
) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get(name)
        .context("existing Detroit connector")?
        .clone();
    let connection = map
        .attributes
        .connections
        .iter_mut()
        .find(|c| c.direction == side)
        .context("existing Detroit road")?;
    ensure!(
        connection.target_map == old_target && connection.offset == old_offset,
        "stale Detroit connector preimage"
    );
    connection.target_map = target.into();
    connection.offset = offset;
    publish(pack, map);
    Ok(())
}

fn publish(pack: &mut CompiledGamePack, map: MapModule) {
    pack.data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    pack.data.maps.insert(map.id.clone(), map);
}
