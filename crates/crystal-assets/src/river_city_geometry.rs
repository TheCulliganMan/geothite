//! Visible river-bank caps and complete walking/Surf boundary validation.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Closure {
    metatile: [u16; 2],
    from: u16,
    to: u16,
}

pub(crate) fn seal(
    base: &CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
    closures: &[Closure],
) -> Result<()> {
    ensure!(
        closures.len() == 6,
        "six visible river-bank boundary caps required"
    );
    let map = maps.get_mut("MnR03C00R01").context("south river chunk")?;
    ensure!(
        [map.attributes.width, map.attributes.height] == [24, 8],
        "south river extent changed"
    );
    for (closure, (x, from)) in
        closures
            .iter()
            .zip([(7, 2), (8, 2), (10, 2), (11, 2), (13, 0x46), (18, 0x44)])
    {
        ensure!(
            closure.metatile == [x, 31] && closure.from == from && closure.to == 5,
            "fixed south river boundary closure required"
        );
        let index = 7 * 24 + usize::from(x);
        ensure!(
            map.blocks[index] == from,
            "stale river bank before boundary cap"
        );
        map.blocks[index] = closure.to;
        for tile in [[x * 2, 14], [x * 2 + 1, 14], [x * 2, 15], [x * 2 + 1, 15]] {
            ensure!(
                kind(base, map, tile)? == "WALL",
                "river cap must block walking and Surf"
            );
        }
    }
    Ok(())
}

pub(crate) fn validate(base: &CompiledGamePack, maps: &BTreeMap<String, MapModule>) -> Result<()> {
    let city = maps.get("MnMankato").context("Mankato")?;
    let closed = reachable(base, city, [40, 18], true)?;
    ensure!(
        !closed.contains(&[24, 0]) && !closed.contains(&[25, 0]),
        "Prairie Badge checkpoint has a walking/Surf bypass"
    );
    let route = [
        maps.get("MnR03C00R00").context("north river chunk")?,
        maps.get("MnR03C00R01").context("south river chunk")?,
    ];
    ensure!(
        [route[0].attributes.width, route[0].attributes.height] == [24, 24]
            && [route[1].attributes.width, route[1].attributes.height] == [24, 8],
        "complete two-chunk River Valley required"
    );
    let bodies = route
        .iter()
        .enumerate()
        .flat_map(|(row, map)| {
            map.objects
                .iter()
                .map(move |o| [o.x, o.y + row as u16 * 48])
        })
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::from([[18_u16, 63_u16]]);
    let mut queue = VecDeque::from([[18_u16, 63_u16]]);
    let mouths = [[18, 63], [19, 63], [20, 0], [21, 0]];
    while let Some([x, y]) = queue.pop_front() {
        if x == 0 || x == 47 || y == 0 || y == 63 {
            ensure!(
                mouths.contains(&[x, y]),
                "unbound River Valley walking/Surf edge at {x},{y}"
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
            if tile[0] >= 48 || tile[1] >= 64 || seen.contains(&tile) || bodies.contains(&tile) {
                continue;
            }
            let map = route[usize::from(tile[1] / 48)];
            let k = kind(base, map, [tile[0], tile[1] % 48])?;
            if matches!(k, "FLOOR" | "TALL_GRASS" | "LADDER" | "WATER" | "CAVE") {
                seen.insert(tile);
                if k != "CAVE" {
                    queue.push_back(tile);
                }
            }
        }
    }
    ensure!(
        mouths.iter().all(|tile| seen.contains(tile)),
        "both River Valley mouths must remain reachable"
    );
    Ok(())
}
