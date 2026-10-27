//! Bounded native city/route connections and full walking/Surf perimeter proof.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cap {
    map: String,
    metatile: [u16; 2],
    previous: u16,
    replacement: u16,
}

pub(crate) fn cap(
    base: &CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
    caps: &[Cap],
) -> Result<()> {
    let expected = [
        ("MnR06C00R00", [20, 0], 2),
        ("MnR06C00R00", [22, 0], 2),
        ("MnR06C00R01", [6, 19], 2),
        ("MnR06C00R01", [7, 19], 2),
        ("MnR06C00R01", [9, 19], 2),
        ("MnR06C00R01", [10, 19], 2),
        ("MnBrainerd", [16, 0], 1),
        ("MnBrainerd", [0, 12], 1),
        ("MnBrainerd", [31, 12], 1),
    ];
    ensure!(
        caps.len() == expected.len(),
        "all nine visible unbound-edge caps required"
    );
    for (cap, (name, at, previous)) in caps.iter().zip(expected) {
        ensure!(
            cap.map == name
                && cap.metatile == at
                && cap.previous == previous
                && cap.replacement == 5,
            "fixed lake-country boundary cap required"
        );
        let map = maps.get_mut(name).context("capped native map")?;
        let i = usize::from(at[1]) * usize::from(map.attributes.width) + usize::from(at[0]);
        ensure!(
            map.blocks.get(i) == Some(&previous),
            "stale native cap source"
        );
        map.blocks[i] = 5;
        for tile in [
            [at[0] * 2, at[1] * 2],
            [at[0] * 2 + 1, at[1] * 2],
            [at[0] * 2, at[1] * 2 + 1],
            [at[0] * 2 + 1, at[1] * 2 + 1],
        ] {
            ensure!(
                kind(base, map, tile)? == "WALL",
                "native forest must block walking and Surf"
            );
        }
    }
    Ok(())
}
pub(crate) fn connect(map: &mut MapModule, side: &str, target: &str, offset: i32) -> Result<()> {
    ensure!(
        !map.attributes
            .connections
            .iter()
            .any(|c| c.direction == side),
        "duplicate northern city seam"
    );
    map.attributes.connections.push(MapConnection {
        direction: side.into(),
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
    Ok(())
}

pub(crate) fn validate(base: &CompiledGamePack, maps: &BTreeMap<String, MapModule>) -> Result<()> {
    for (name, start, mouths) in [
        (
            "MnMinneapolis",
            [32, 40],
            vec![[32, 0], [33, 0], [32, 47], [33, 47], [63, 24], [63, 25]],
        ),
        (
            "MnStCloud",
            [24, 32],
            vec![[24, 0], [25, 0], [24, 35], [25, 35]],
        ),
    ] {
        let map = &maps[name];
        let closed = reachable(base, map, start, true)?;
        let x = if name == "MnMinneapolis" { 32 } else { 24 };
        ensure!(
            !closed.contains(&[x, 0]) && !closed.contains(&[x + 1, 0]),
            "northern checkpoint has walking/Surf bypass"
        );
        let mut open = map.clone();
        open.objects.retain(|o| {
            !o.object_identifier.as_deref().is_some_and(|id| {
                id.starts_with("MN_MP_NORTH_WARDEN_")
                    || id.starts_with("MN_MP_EAST_WARDEN_")
                    || id.starts_with("MN_SC_NORTH_WARDEN_")
            })
        });
        crate::prairie_story_connections::perimeter(base, &open, start, &mouths)?;
    }
    crate::prairie_story_connections::perimeter(
        base,
        &maps["MnBrainerd"],
        [32, 44],
        &[[32, 47], [33, 47]],
    )?;
    for (prefix, height, mouths) in [
        ("MnR05", 112, [[24, 0], [25, 0], [22, 111], [23, 111]]),
        ("MnR06", 88, [[42, 0], [43, 0], [16, 87], [17, 87]]),
    ] {
        route_perimeter(base, maps, prefix, height, mouths)?;
    }
    Ok(())
}
fn route_perimeter(
    base: &CompiledGamePack,
    changed: &BTreeMap<String, MapModule>,
    prefix: &str,
    height: u16,
    mouths: [[u16; 2]; 4],
) -> Result<()> {
    let modules = (0..height.div_ceil(48))
        .map(|row| {
            let name = format!("{prefix}C00R{row:02}");
            changed
                .get(&name)
                .or_else(|| base.data.maps.get(&name))
                .context("complete road chunk")
        })
        .collect::<Result<Vec<_>>>()?;
    let bodies = modules
        .iter()
        .enumerate()
        .flat_map(|(row, m)| m.objects.iter().map(move |o| [o.x, o.y + row as u16 * 48]))
        .collect::<BTreeSet<_>>();
    for surf in [false, true] {
        let mut seen = BTreeSet::from([mouths[0]]);
        let mut queue = VecDeque::from([mouths[0]]);
        while let Some([x, y]) = queue.pop_front() {
            if x == 0 || x == 63 || y == 0 || y == height - 1 {
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
                if tile[0] >= 64
                    || tile[1] >= height
                    || seen.contains(&tile)
                    || bodies.contains(&tile)
                {
                    continue;
                }
                let k = kind(
                    base,
                    modules[usize::from(tile[1] / 48)],
                    [tile[0], tile[1] % 48],
                )?;
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
            "both {prefix} mouths must remain reachable without field moves"
        );
    }
    Ok(())
}
pub(crate) fn verify(pack: &CompiledGamePack) -> Result<()> {
    for (map, target, out, to) in [
        ("MnMinneapolis", "MnR05C00R02", [32, -1], [22, 15]),
        ("MnMinneapolis", "MnR05C00R02", [33, -1], [23, 15]),
        ("MnR05C00R02", "MnMinneapolis", [22, 16], [32, 0]),
        ("MnR05C00R02", "MnMinneapolis", [23, 16], [33, 0]),
        ("MnBrainerd", "MnR06C00R00", [32, 48], [42, 0]),
        ("MnBrainerd", "MnR06C00R00", [33, 48], [43, 0]),
        ("MnR06C00R00", "MnBrainerd", [42, -1], [32, 47]),
        ("MnR06C00R00", "MnBrainerd", [43, -1], [33, 47]),
    ] {
        let m = &pack.data.maps[map];
        let native = OverworldMapData::from_attributes(map, &m.attributes, m.blocks.clone());
        let t = connection_for_tile_checked(&native, TilePosition::new(out[0], out[1]))?
            .context("native northern connection")?;
        let d = pack.data.resolve_connection_transition(&t)?.destination;
        ensure!(
            d.map_name == target && d.tile == TilePosition::new(to[0], to[1]),
            "native northern connection differs"
        );
    }
    Ok(())
}
