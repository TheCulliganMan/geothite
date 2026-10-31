//! Small, reachable young-Pokémon pockets alongside the main encounter curve.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{WildEncounterData, WildEncounterZone};
use serde::Deserialize;

use crate::CompiledGamePack;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct YoungGrassHabitat {
    zone: WildEncounterZone,
    entrance: [u16; 2],
}

pub(super) fn install(
    base: &CompiledGamePack,
    name: &str,
    data: &mut WildEncounterData,
    habitats: &[YoungGrassHabitat],
) -> Result<()> {
    if habitats.is_empty() {
        return Ok(());
    }
    let map = base.data().maps.get(name).context("young habitat map")?;
    let original = base
        .data()
        .wild_encounters
        .get(name)
        .and_then(|w| w.grass.as_ref())
        .context("young pocket requires an existing ordinary grass table")?;
    let residents = original
        .morning
        .iter()
        .chain(&original.day)
        .chain(&original.night)
        .map(|s| s.species.as_str())
        .collect::<BTreeSet<_>>();
    let evolved = base
        .data()
        .evolutions
        .0
        .values()
        .flatten()
        .map(|e| e.species.as_str())
        .collect::<BTreeSet<_>>();
    for habitat in habitats {
        let zone = &habitat.zone;
        ensure!(
            name.starts_with("Mn")
                && !zone.id.is_empty()
                && zone.min_x >= 0
                && zone.min_y >= 0
                && zone.max_x >= zone.min_x
                && zone.max_y >= zone.min_y
                && i32::from(zone.max_x) < i32::from(map.attributes.width) * 2
                && i32::from(zone.max_y) < i32::from(map.attributes.height) * 2,
            "young grass bounds must fit their regional map: {name}"
        );
        let area = i32::from(zone.max_x - zone.min_x + 1) * i32::from(zone.max_y - zone.min_y + 1);
        ensure!((2..=16).contains(&area), "young pocket must remain small");
        ensure!(
            data.zones.iter().all(|old| {
                old.id != zone.id
                    && (old.max_x < zone.min_x
                        || old.min_x > zone.max_x
                        || old.max_y < zone.min_y
                        || old.min_y > zone.max_y)
            }),
            "young pocket must not shadow an existing habitat"
        );
        ensure!(
            zone.grass_rates
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                == BTreeSet::from(["morning", "day", "night"])
                && zone.grass_rates.values().all(|r| (1..=25).contains(r)),
            "young pocket requires ordinary active rates in all periods"
        );
        for slots in [&zone.grass.morning, &zone.grass.day, &zone.grass.night] {
            ensure!(slots.len() == 7, "young pocket retains native slot weights");
            ensure!(
                slots.iter().all(|s| {
                    (3..=10).contains(&s.level)
                        && residents.contains(s.species.as_str())
                        && !evolved.contains(s.species.as_str())
                }),
                "young pocket requires low-level basic forms already resident on its map"
            );
        }
        ensure!(
            map.events
                .warps
                .iter()
                .any(|w| { [w.x, w.y.saturating_add(1)] == habitat.entrance }),
            "young habitat access must start outside an actual town door"
        );
        let bodies = map
            .objects
            .iter()
            .map(|o| [o.x, o.y])
            .collect::<BTreeSet<_>>();
        let terrain = |[x, y]: [u16; 2]| -> &str {
            let block = map.blocks
                [usize::from(y / 2) * usize::from(map.attributes.width) + usize::from(x / 2)];
            &base.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
                [usize::from(y % 2 * 2 + x % 2)]
        };
        let mut seen = BTreeSet::from([habitat.entrance]);
        let mut pending = VecDeque::from([habitat.entrance]);
        while let Some([x, y]) = pending.pop_front() {
            for next in [
                x.checked_sub(1).map(|x| [x, y]),
                y.checked_sub(1).map(|y| [x, y]),
                x.checked_add(1).map(|x| [x, y]),
                y.checked_add(1).map(|y| [x, y]),
            ]
            .into_iter()
            .flatten()
            {
                if next[0] < map.attributes.width * 2
                    && next[1] < map.attributes.height * 2
                    && !bodies.contains(&next)
                    && matches!(terrain(next), "FLOOR" | "TALL_GRASS")
                    && seen.insert(next)
                {
                    pending.push_back(next);
                }
            }
        }
        let grass = seen
            .iter()
            .filter(|p| terrain(**p) == "TALL_GRASS")
            .collect::<Vec<_>>();
        let inside = grass
            .iter()
            .filter(|p| zone.contains(p[0] as i16, p[1] as i16))
            .count();
        ensure!(
            inside >= 2 && inside < grass.len(),
            "both the young pocket and ordinary grass must be reachable on foot"
        );
        data.zones.push(zone.clone());
    }
    Ok(())
}

#[cfg(test)]
#[path = "minnesota_young_grass_tests.rs"]
mod tests;
