//! Whole-route walking/Surf boundaries across clipped native chunks.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

pub(crate) fn route_perimeter(
    pack: &CompiledGamePack,
    prefix: &str,
    [width, height]: [u16; 2],
    mouths: &[[u16; 2]],
) -> Result<()> {
    let mut bodies = BTreeSet::new();
    for row in 0..height.div_ceil(48) {
        for col in 0..width.div_ceil(64) {
            let name = format!("{prefix}C{col:02}R{row:02}");
            let map = pack.data.maps.get(&name).context("complete route chunk")?;
            ensure!(
                map.attributes.width * 2 == (width - col * 64).min(64)
                    && map.attributes.height * 2 == (height - row * 48).min(48),
                "clipped route dimensions differ"
            );
            bodies.extend(map.objects.iter().map(|o| [o.x + col * 64, o.y + row * 48]));
        }
    }
    let start = *mouths.first().context("both route mouths")?;
    for surf in [false, true] {
        let mut seen = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some([x, y]) = queue.pop_front() {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                ensure!(
                    mouths.contains(&[x, y]),
                    "{prefix} unbound walking/Surf edge at {x},{y}"
                );
            }
            for tile in [
                x.checked_sub(1).map(|x| [x, y]),
                y.checked_sub(1).map(|y| [x, y]),
                x.checked_add(1).map(|x| [x, y]),
                y.checked_add(1).map(|y| [x, y]),
            ]
            .into_iter()
            .flatten()
            {
                if tile[0] >= width
                    || tile[1] >= height
                    || seen.contains(&tile)
                    || bodies.contains(&tile)
                {
                    continue;
                }
                let name = format!("{prefix}C{:02}R{:02}", tile[0] / 64, tile[1] / 48);
                let k = kind(pack, &pack.data.maps[&name], [tile[0] % 64, tile[1] % 48])?;
                if matches!(k, "FLOOR" | "TALL_GRASS" | "LADDER" | "CAVE" | "DOOR")
                    || (surf && k == "WATER")
                {
                    seen.insert(tile);
                    if !matches!(k, "CAVE" | "DOOR") {
                        queue.push_back(tile);
                    }
                }
            }
        }
        ensure!(
            mouths.iter().all(|p| seen.contains(p)),
            "both {prefix} mouths need a field-move-free walking route"
        );
    }
    Ok(())
}
