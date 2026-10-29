//! Restore the authored League walking seam after regional connection overlays.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, y, xs) in [
        ("MnBoundaryWaters", 47, [32, 33]),
        ("MnR13C00R00", 0, [30, 31]),
    ] {
        let map = pack.data.maps.get(name).context("League walking border")?;
        ensure!(
            map.attributes.width == 32 && map.attributes.height == 24,
            "League border dimensions changed"
        );
        let collision = &pack.data.tilesets[&map.attributes.tileset_name].collision;
        for x in xs {
            let block = map.blocks[y / 2 * usize::from(map.attributes.width) + x / 2];
            ensure!(
                collision[&format!("{block:02x}")][y % 2 * 2 + x % 2] == "FLOOR",
                "League border floor changed: {name} ({x}, {y})"
            );
        }
    }
    // The town's canonical connection contract uses one metatile of offset:
    // camp (32/33,47) meets road (30/31,0). A later overlay reset both offsets
    // to zero, making the earned ending route lead into the road's wall.
    for (name, direction, target, offset) in [
        ("MnBoundaryWaters", "south", "MnR13C00R00", 1),
        ("MnR13C00R00", "north", "MnBoundaryWaters", -1),
    ] {
        let mut map = pack.data.maps[name].clone();
        let mut matches = map
            .attributes
            .connections
            .iter_mut()
            .filter(|link| link.direction == direction && link.target_map == target);
        let link = matches.next().context("reciprocal League connection")?;
        ensure!(
            link.offset == 0 || link.offset == offset,
            "League connection offset changed"
        );
        link.offset = offset;
        ensure!(matches.next().is_none(), "duplicate League connection");
        crate::authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}
