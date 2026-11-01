//! The fair's physical eastern entrance onto the existing lower Pine Country road.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

use super::CityConnection;

pub(super) fn bind(pack: &mut CompiledGamePack, connection: &CityConnection) -> Result<()> {
    ensure!(
        connection.map == "MnR05C00R02"
            && connection.tileset == "mn_headwaters"
            && connection.dimensions_metatiles == Some([32, 8])
            && connection.fair_to_city_offset == 28
            && connection.edits.len() == 9,
        "retain the bounded lower Pine Country fair approach"
    );
    let mut road = pack
        .data
        .maps
        .get(&connection.map)
        .context("fair approach road")?
        .clone();
    ensure!(
        [road.attributes.width, road.attributes.height] == [32, 8]
            && road.attributes.tileset_name == connection.tileset
            && !road
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == "west"),
        "lower Pine Country extent, art or western connection changed"
    );
    for (x, edit) in connection.edits.iter().enumerate() {
        ensure!(
            edit.position == [x, 4] && edit.before == "05" && edit.after == "01",
            "the fair trail may open only the nine declared pine cells"
        );
        let index = 4 * usize::from(road.attributes.width) + x;
        ensure!(
            road.blocks[index] == 0x05,
            "fair trail native preimage differs"
        );
        for y in [8, 9] {
            for tile_x in [x as u16 * 2, x as u16 * 2 + 1] {
                let tile = [tile_x, y];
                ensure!(
                    kind(pack, &road, tile)? == "WALL"
                        && !road.objects.iter().any(|o| [o.x, o.y] == tile)
                        && !road.events.warps.iter().any(|w| [w.x, w.y] == tile)
                        && !road.events.coord_events.iter().any(|e| [e.x, e.y] == tile)
                        && !road.events.bg_events.iter().any(|e| [e.x, e.y] == tile),
                    "the fair trail must not replace water, gameplay gates or authored events"
                );
            }
        }
        road.blocks[index] = 0x01;
    }
    for x in 0..20 {
        for y in [8, 9] {
            ensure!(
                kind(pack, &road, [x, y])? == "FLOOR",
                "fair trail must reach the existing road"
            );
        }
    }
    road.attributes.connections.push(MapConnection {
        direction: "west".into(),
        target_map: "MnStateFairgrounds".into(),
        offset: -28,
    });
    road.attributes.connection_flags = Some(
        road.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    crate::authored_town_publication::replace(pack, road)?;
    for (name, target, at, outside, wanted) in [
        (
            "MnStateFairgrounds",
            "MnR05C00R02",
            [95, 64],
            [96, 64],
            [0, 8],
        ),
        (
            "MnStateFairgrounds",
            "MnR05C00R02",
            [95, 65],
            [96, 65],
            [0, 9],
        ),
        (
            "MnR05C00R02",
            "MnStateFairgrounds",
            [0, 8],
            [-1, 8],
            [95, 64],
        ),
        (
            "MnR05C00R02",
            "MnStateFairgrounds",
            [0, 9],
            [-1, 9],
            [95, 65],
        ),
    ] {
        let map = &pack.data.maps[name];
        ensure!(
            kind(pack, map, at)? == "FLOOR"
                && kind(pack, &pack.data.maps[target], wanted)? == "FLOOR",
            "both fair road lanes must be native floor"
        );
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        let trigger =
            connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                .context("actual fair road boundary")?;
        let destination = pack
            .data
            .resolve_connection_transition(&trigger)?
            .destination;
        ensure!(
            destination.map_name == target
                && destination.tile == TilePosition::new(wanted[0] as i16, wanted[1] as i16),
            "native fair road projection differs"
        );
    }
    Ok(())
}
