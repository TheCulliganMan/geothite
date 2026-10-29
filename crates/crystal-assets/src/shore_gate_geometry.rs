//! Native paired passage and retained Cedar Approach road projections.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(
    pack: &CompiledGamePack,
    exterior: &MapModule,
    gate: &MapModule,
) -> Result<()> {
    let west = reachable(pack, exterior, [18, 14], true)?;
    let east = reachable(pack, exterior, [25, 14], true)?;
    ensure!(
        [[0, 14], [0, 15], [19, 14], [19, 15]]
            .iter()
            .all(|p| west.contains(p))
            && !west.contains(&[25, 14])
            && [[26, 23], [27, 23], [24, 14], [24, 15]]
                .iter()
                .all(|p| east.contains(p))
            && !east.contains(&[18, 14]),
        "the shore gate must separate the existing road, without a walking or Surf bypass"
    );
    for lane in 0..2 {
        for (entry, return_at, carpet) in [
            ([19, 14 + lane], [18, 14 + lane], "WARP_CARPET_RIGHT"),
            ([24, 14 + lane], [25, 14 + lane], "WARP_CARPET_LEFT"),
        ] {
            ensure!(
                kind(pack, exterior, entry)? == carpet
                    && kind(pack, exterior, return_at)? == "FLOOR",
                "gate exteriors need native side carpets and passive floor returns"
            );
        }
        for x in [1, 8] {
            let start = [x, 4 + lane];
            ensure!(
                kind(pack, gate, start)? == "FLOOR",
                "passive gate arrival must be floor"
            );
            let public = reachable(pack, gate, start, false)?;
            ensure!(
                [[0, 4], [0, 5], [9, 4], [9, 5], [5, 4], [7, 4]]
                    .iter()
                    .all(|p| public.contains(p)),
                "both native lanes must reach both exits and the two resident approaches"
            );
        }
    }
    ensure!(
        kind(pack, gate, [5, 2])? == "FLOOR"
            && kind(pack, gate, [5, 3])? == "COUNTER"
            && kind(pack, gate, [7, 5])? == "FLOOR",
        "native counter and public resident footing"
    );
    for lane in 0..2 {
        crossing(
            pack,
            exterior,
            [0, 14 + lane],
            [-1, 14 + lane],
            "MnR12C00R01",
            [63, 6 + lane],
        )?;
        crossing(
            pack,
            &pack.data.maps["MnR12C00R01"],
            [63, 6 + lane],
            [64, 6 + lane],
            &exterior.id,
            [0, 14 + lane],
        )?;
        crossing(
            pack,
            exterior,
            [26 + lane, 23],
            [26 + lane, 24],
            "MnGrandMarais",
            [18 + lane, 0],
        )?;
        crossing(
            pack,
            &pack.data.maps["MnGrandMarais"],
            [18 + lane, 0],
            [18 + lane, -1],
            &exterior.id,
            [26 + lane, 23],
        )?;
    }
    Ok(())
}

fn crossing(
    pack: &CompiledGamePack,
    map: &MapModule,
    at: [i16; 2],
    outside: [i16; 2],
    target: &str,
    expected: [i16; 2],
) -> Result<()> {
    let unsigned =
        |p: [i16; 2]| -> Result<[u16; 2]> { Ok([u16::try_from(p[0])?, u16::try_from(p[1])?]) };
    ensure!(
        kind(pack, map, unsigned(at)?)? == "FLOOR"
            && kind(pack, &pack.data.maps[target], unsigned(expected)?)? == "FLOOR",
        "retained Cedar road lane collision changed"
    );
    let native = OverworldMapData::from_attributes(&map.id, &map.attributes, map.blocks.clone());
    let transition =
        connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
            .context("retained road connection")?;
    let arrival = pack
        .data
        .resolve_connection_transition(&transition)?
        .destination;
    ensure!(
        arrival.map_name == target && arrival.tile == TilePosition::new(expected[0], expected[1]),
        "retained Cedar road projection changed"
    );
    Ok(())
}
