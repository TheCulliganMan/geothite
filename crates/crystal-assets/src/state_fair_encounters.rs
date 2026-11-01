//! Local food and carnival habitats, bounded exactly by native encounter grass.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{ENCOUNTER_TIME_KEYS, WildEncounterData};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_town_geometry::{kind, reachable};

#[derive(Deserialize)]
pub(crate) struct Family {
    id: String,
    members: Vec<String>,
}

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    encounters: WildEncounterData,
    families: &[Family],
) -> Result<()> {
    let roster = crate::authored_regional_approach::policy()?;
    let name = "MnStateFairgrounds";
    ensure!(
        encounters.map_name == name && !pack.data.wild_encounters.contains_key(name),
        "fair encounter identity already occupied or incorrect"
    );
    let map = pack.data.maps.get(name).context("published fair map")?;
    let accessible = reachable(pack, map, [16, 68], false)?;
    for actor in &map.objects {
        let below = [actor.x, actor.y + 1];
        if below[1] < map.attributes.height * 2 && kind(pack, map, below)? == "COUNTER" {
            ensure!(
                kind(pack, map, [actor.x, actor.y + 2])? == "FLOOR",
                "fair counter customer tile must stay clear at {},{}",
                actor.x,
                actor.y + 2
            );
        }
    }
    let mut covered = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut available = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
    ensure!(!encounters.zones.is_empty(), "fair needs grass habitats");
    for zone in &encounters.zones {
        ensure!(
            ids.insert(&zone.id) && zone.min_x <= zone.max_x && zone.min_y <= zone.max_y,
            "duplicate or inverted fair habitat {}",
            zone.id
        );
        for y in zone.min_y..=zone.max_y {
            for x in zone.min_x..=zone.max_x {
                let tile = [u16::try_from(x)?, u16::try_from(y)?];
                ensure!(
                    kind(pack, map, tile)? == "TALL_GRASS"
                        && accessible.contains(&tile)
                        && covered.insert(tile),
                    "fair habitat {} must be reachable, nonoverlapping native grass at {tile:?}",
                    zone.id
                );
            }
        }
        for (index, (time, slots)) in ENCOUNTER_TIME_KEYS
            .iter()
            .zip([&zone.grass.morning, &zone.grass.day, &zone.grass.night])
            .enumerate()
        {
            ensure!(
                zone.grass_rates.get(*time).is_some_and(|rate| *rate > 0) && slots.len() == 7,
                "fair habitat {} needs seven native slots and an active {time} rate",
                zone.id
            );
            for slot in slots {
                ensure!(
                    roster.target(pack, &slot.species, None)? == slot.species,
                    "fair grass needs active Minnesota species: {}",
                    slot.species
                );
                available[index].insert(slot.species.as_str());
            }
        }
    }
    for y in 0..map.attributes.height * 2 {
        for x in 0..map.attributes.width * 2 {
            if kind(pack, map, [x, y])? == "TALL_GRASS" {
                ensure!(covered.contains(&[x, y]), "unbound fair grass at {x},{y}");
            }
        }
    }
    let mut family_ids = BTreeSet::new();
    let mut expected = BTreeSet::new();
    for family in families {
        ensure!(
            family_ids.insert(&family.id) && !family.members.is_empty(),
            "duplicate or empty fair family {}",
            family.id
        );
        for member in &family.members {
            ensure!(
                expected.insert(member.as_str()),
                "duplicate fair species {member}"
            );
        }
    }
    ensure!(!expected.is_empty(), "fair creature selection is empty");
    for (time, actual) in ENCOUNTER_TIME_KEYS.iter().zip(available) {
        ensure!(
            actual == expected,
            "fair {time} encounters omit or add a selected species"
        );
    }
    pack.data.wild_encounters.insert(name.into(), encounters);
    Ok(())
}
